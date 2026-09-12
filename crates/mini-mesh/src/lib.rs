//! Transport-generic dedup-flood mesh relay (`docs/design/ble-mesh-relay.md`).
//!
//! [`MeshNode`] generalizes `mini_consensus::net::TcpMesh` + `run_to_height`'s
//! proven relay shape — a dynamic set of point-to-point encrypted links,
//! dedup-flood re-gossip across them, live over any **connected** graph even
//! without a direct edge between every pair — off raw `TcpStream`s and onto
//! any [`mini_bearer::Bearer`], so the same relay works over BLE (or a test's
//! [`mini_bearer::InProcessBearer`]) without a second implementation of the
//! algorithm. See the design doc for the full architecture, what a BLE-mesh
//! device actually does with this (`mini-ffi`'s UniFFI surface, the Android
//! multi-connection wiring), and the honest limits — relay nodes see
//! plaintext, there is no routing (only flooding), no peer discovery, and
//! nothing here has been proven on real BLE hardware.
//!
//! No new cryptography: [`mini_bearer::EncryptedLink`] is the exact
//! established `Channel` construction this whole tree already uses.
//! [`mini_net::GossipRouter`] is the exact dedup cache `mini-net`'s own
//! gossip already uses — reused directly, not reimplemented a third time
//! after `mini-net`'s own copy and `mini-consensus::net`'s `SeenCache`.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mini_bearer::{Bearer, BearerError, EncryptedLink, MAX_CHANNEL_PLAINTEXT_BYTES};
use mini_crypto::HashAlgorithm;
use mini_net::GossipRouter;

/// A mesh link, individually lockable so [`MeshNode::poll`] never has to
/// wait behind [`MeshNode::flush_reflood`]'s potentially slow send on a
/// *different* link -- see the module docs on the two methods for why this
/// exists (a Codex review finding on PR #333: an earlier revision put every
/// link behind one lock shared with the caller's own `MeshHandle`, so one
/// slow peer's blocking send starved receiving on every other link too).
type LinkHandle = Arc<Mutex<EncryptedLink<Box<dyn Bearer + Send>>>>;

/// How many recently-seen message ids [`MeshNode`] remembers before evicting
/// the oldest — bounds memory under a flood of distinct messages, the same
/// stance every seen-cache in this tree already takes.
pub const DEFAULT_SEEN_CAPACITY: usize = 65_536;

/// How many messages [`MeshNode::poll`] drains from one link before moving
/// on to the next, per call. Without this bound a single link that always
/// has more waiting (a fast or hostile peer using a write-without-response
/// characteristic, e.g.) could starve every other link's traffic and grow
/// one `poll()` call's allocation without limit; a caller that needs more
/// throughput from one link simply calls `poll()` again.
pub const MAX_MESSAGES_PER_LINK_PER_POLL: usize = 64;

/// How many not-yet-sent reflood payloads [`MeshNode::poll`] queues before it
/// starts dropping the oldest. `poll()` itself only stages payloads here —
/// see [`MeshNode::flush_reflood`] for why the *sending* is a separate,
/// caller-scheduled step and this queue exists at all. Sized generously
/// relative to one `poll()` batch's worst case
/// (`MAX_MESSAGES_PER_LINK_PER_POLL` distinct new messages per link).
pub const MAX_PENDING_REFLOOD: usize = 4_096;

/// Total bytes [`MeshNode::poll`] lets the reflood queue hold before it
/// starts dropping the oldest entries, regardless of how many entries that
/// is (a Codex review finding on PR #333: bounding only by *count* still
/// let a high-capacity peer queue up to `MAX_PENDING_REFLOOD *
/// MAX_CHANNEL_PLAINTEXT_BYTES` — tens of gigabytes — since each accepted
/// payload can be nearly `MAX_CHANNEL_PLAINTEXT_BYTES` on its own). Sized
/// generously above one realistic `poll()` batch's worst case, still far
/// below the old count-only bound's actual worst case.
pub const MAX_PENDING_REFLOOD_BYTES: usize = 64 * 1024 * 1024;

/// Aggregate budget for the `new_messages` batch [`MeshNode::poll`] returns
/// from a *single call*, across every link in the mesh combined -- not just
/// per link. [`MAX_MESSAGES_PER_LINK_PER_POLL`] alone only bounds one link's
/// contribution; with an unbounded number of links, a generic high-capacity
/// bearer could otherwise let one `poll()` call retain an arbitrarily large
/// batch (individual payloads approaching `MAX_CHANNEL_PLAINTEXT_BYTES`
/// each) before the caller or UniFFI delivery layer has any chance to shed
/// it (a Codex review finding on PR #333). Once either bound is hit,
/// remaining links are simply left unpolled this round -- their traffic is
/// not lost, only deferred to the caller's next `poll()` call, the same
/// stance [`MeshNode::poll`] already takes for a single busy link.
pub const MAX_NEW_MESSAGES_PER_POLL: usize = MAX_PENDING_REFLOOD;

/// Byte counterpart of [`MAX_NEW_MESSAGES_PER_POLL`], for the same reason
/// [`MAX_PENDING_REFLOOD_BYTES`] exists alongside [`MAX_PENDING_REFLOOD`]:
/// a handful of near-maximum-size payloads can hit a byte budget long
/// before an entry-count budget would ever trip.
pub const MAX_NEW_MESSAGES_PER_POLL_BYTES: usize = MAX_PENDING_REFLOOD_BYTES;

/// A content id for a mesh payload: the BLAKE3 digest of its raw bytes.
/// Every hop computes the same id independently from the same bytes, so
/// nothing needs to carry an id on the wire — the payload *is* its own id,
/// the same content-addressing this tree uses everywhere else.
pub fn message_id(payload: &[u8]) -> [u8; 32] {
    HashAlgorithm::Blake3.digest(payload)
}

/// One device's live set of mesh links plus the dedup state that turns
/// flooding across them into a working multi-hop relay. A link is any
/// already-handshaken [`EncryptedLink`] over any [`Bearer`] — an in-process
/// pair in a test, a real BLE connection in production (see the design doc
/// for how a BLE link becomes one of these). Links can be added as they
/// connect; nothing about this type assumes a fixed topology decided up
/// front, unlike `TcpMesh`.
pub struct MeshNode {
    links: Mutex<Vec<LinkHandle>>,
    seen: Mutex<GossipRouter>,
    /// Payloads [`Self::poll`] has already deduped and needs reflooded, not
    /// yet actually sent — see [`Self::flush_reflood`].
    pending_reflood: Mutex<PendingReflood>,
}

/// [`MeshNode::pending_reflood`]'s queue plus a running byte total, kept
/// consistent together so eviction can enforce [`MAX_PENDING_REFLOOD`] and
/// [`MAX_PENDING_REFLOOD_BYTES`] as one operation rather than two locks (or
/// one lock with the byte total silently drifting from the queue's real
/// contents).
#[derive(Debug, Default)]
struct PendingReflood {
    queue: VecDeque<Vec<u8>>,
    bytes: usize,
}

impl PendingReflood {
    /// Queue `payload`, evicting the oldest entries first until both the
    /// entry-count and total-byte bounds hold (including for the entry
    /// just pushed).
    fn push(&mut self, payload: Vec<u8>) {
        self.bytes += payload.len();
        self.queue.push_back(payload);
        while self.queue.len() > MAX_PENDING_REFLOOD || self.bytes > MAX_PENDING_REFLOOD_BYTES {
            let Some(evicted) = self.queue.pop_front() else {
                break;
            };
            self.bytes = self.bytes.saturating_sub(evicted.len());
        }
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        let payload = self.queue.pop_front()?;
        self.bytes = self.bytes.saturating_sub(payload.len());
        Some(payload)
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

impl std::fmt::Debug for MeshNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshNode")
            .field("links", &self.lock_links().len())
            .field("seen", &self.lock_seen().len())
            .finish()
    }
}

impl MeshNode {
    /// A mesh node with no links yet, remembering up to [`DEFAULT_SEEN_CAPACITY`]
    /// message ids.
    pub fn new() -> Self {
        Self::with_seen_capacity(DEFAULT_SEEN_CAPACITY)
    }

    /// A mesh node with a caller-chosen dedup-cache capacity.
    pub fn with_seen_capacity(seen_capacity: usize) -> Self {
        MeshNode {
            links: Mutex::new(Vec::new()),
            seen: Mutex::new(GossipRouter::new(seen_capacity)),
            pending_reflood: Mutex::new(PendingReflood::default()),
        }
    }

    // A poisoned lock (a panic while held) still hands back its contents --
    // one panicking caller must not permanently wedge every future
    // poll()/flush_reflood()/broadcast() call on this node.
    fn lock_links(&self) -> std::sync::MutexGuard<'_, Vec<LinkHandle>> {
        self.links.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn lock_seen(&self) -> std::sync::MutexGuard<'_, GossipRouter> {
        self.seen.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn lock_pending(&self) -> std::sync::MutexGuard<'_, PendingReflood> {
        self.pending_reflood
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    }

    /// Remove exactly the given links (by identity, not position, since the
    /// live list may have changed under us between snapshotting it and
    /// finishing a send/receive pass) from the held set.
    fn prune_dead(&self, dead: &[LinkHandle]) {
        if dead.is_empty() {
            return;
        }
        self.lock_links()
            .retain(|h| !dead.iter().any(|d| Arc::ptr_eq(d, h)));
    }

    /// Add a new live link — e.g. a BLE connection that just finished its
    /// [`EncryptedLink`] handshake. Takes effect on the next [`Self::broadcast`]
    /// or [`Self::poll`].
    pub fn add_link(&self, link: EncryptedLink<Box<dyn Bearer + Send>>) {
        self.lock_links().push(Arc::new(Mutex::new(link)));
    }

    /// How many links are currently held. A link that has failed a
    /// `try_recv` is pruned by [`Self::poll`] (see its own docs), so this
    /// can only drop, never silently accumulate dead connections; it does
    /// not shrink on a `send` failure alone (best-effort, matching every
    /// other bearer-broadcast in this tree) until that link's next failed
    /// `try_recv`.
    pub fn link_count(&self) -> usize {
        self.lock_links().len()
    }

    /// Send `payload` to every held link and mark it seen, so an echo of it
    /// flooded back by a peer is deduped rather than re-flooded — the same
    /// discipline `mini_consensus::net::handle_emits`'s `Emit::Broadcast`
    /// case already applies. A link whose `send` fails is dropped from the
    /// mesh, not just best-effort-ignored: [`EncryptedLink::send`] seals
    /// (and so advances its AEAD send counter) before handing ciphertext to
    /// the bearer, so a bearer-level failure *after* a successful seal
    /// leaves that link's counter ahead of what the peer actually received
    /// — permanently desynced, the same terminal condition [`Self::poll`]
    /// already prunes on a `try_recv` failure, just on the send side.
    /// Returns the message id, so a caller can recognize its own broadcast
    /// if it comes back through [`Self::poll`] from elsewhere (deduped, not
    /// delivered twice, but the id is still useful to log).
    ///
    /// Rejects a `payload` over [`MAX_CHANNEL_PLAINTEXT_BYTES`], or over any
    /// held link's own [`EncryptedLink::max_sendable_plaintext_bytes`], up
    /// front, before touching any link: either bound would reject the same
    /// payload identically on *every* link with that bound (it is a
    /// property of the payload versus a fixed limit, not of any one
    /// connection's live state), so without this check every held link's
    /// `send` would fail together and the same `retain_mut`-based pruning
    /// that correctly drops a genuinely desynced link would instead empty
    /// the whole mesh over one oversized local message. A BLE-backed link's
    /// bound is typically far smaller than `MAX_CHANNEL_PLAINTEXT_BYTES`
    /// (its negotiated MTU limits how many chunks a `u16` chunk count can
    /// express) — this is what actually catches that case, not the channel
    /// cap alone. A rejected payload is never marked seen: it was never
    /// actually sent, so nothing needs deduping against it.
    ///
    /// **Honest limit**: this rejects a payload that does not fit the
    /// *smallest*-capacity held link, even if it would fit every other one
    /// — this mesh floods to every link, so a payload that cannot reach one
    /// held link cannot be broadcast at all today. A future heterogeneous-
    /// bearer mesh wanting partial delivery to only the links that can
    /// carry a given payload would need real routing, not flooding; out of
    /// scope here (see the design doc's "no routing" honest limit).
    pub fn broadcast(&self, payload: &[u8]) -> Result<[u8; 32], BearerError> {
        if payload.len() > MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(BearerError::FrameTooLarge {
                max: MAX_CHANNEL_PLAINTEXT_BYTES,
                got: payload.len(),
            });
        }
        let snapshot: Vec<LinkHandle> = self.lock_links().clone();
        if let Some(min_capacity) = snapshot
            .iter()
            .filter_map(|handle| {
                handle
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .max_sendable_plaintext_bytes()
            })
            .min()
        {
            if payload.len() > min_capacity {
                return Err(BearerError::FrameTooLarge {
                    max: min_capacity,
                    got: payload.len(),
                });
            }
        }
        let id = message_id(payload);
        self.lock_seen().record_seen(id);
        let mut dead = Vec::new();
        for handle in &snapshot {
            let mut link = handle.lock().unwrap_or_else(|p| p.into_inner());
            if link.send(payload).is_err() {
                dead.push(Arc::clone(handle));
            }
        }
        self.prune_dead(&dead);
        Ok(id)
    }

    /// Drain every link of up to [`MAX_MESSAGES_PER_LINK_PER_POLL`] waiting
    /// messages and dedup them: the first time this node sees a given
    /// payload, it is queued for reflooding (see [`Self::flush_reflood`] —
    /// **not** sent here) and returned to the caller; a repeat is silently
    /// dropped. A link whose `try_recv` fails (the bearer closed, the peer
    /// disconnected, a decode/decrypt failure) is a **terminal** failure for
    /// that link — it is dropped from the mesh in the same call, not
    /// retried on every future `poll()` — so this never panics and never
    /// accumulates dead links.
    ///
    /// Genuinely non-blocking, unlike an earlier revision: every
    /// [`Bearer::try_recv`] this calls is documented not to block, but
    /// sending is a different story — on a real platform bearer (e.g.
    /// Android GATT), one write can wait seconds for a peer's
    /// acknowledgement, and a large relayed payload can be many chunks.
    /// Calling `send` here for every reflooded payload would let one slow
    /// or hostile peer stall this call for as long as that peer keeps
    /// acking slowly. Queuing instead and leaving the actual sends to
    /// [`Self::flush_reflood`] means a caller can run that on its own
    /// thread/schedule, never blocking the fast receive-and-dedup path
    /// this function's own contract promises.
    ///
    /// Per-link locking (a Codex review finding on PR #333) means this also
    /// never blocks *waiting* for [`Self::flush_reflood`]: a link currently
    /// mid-send is simply skipped this round (its `try_lock` fails) rather
    /// than stalling every other link's receive progress behind it — it is
    /// tried again on the next `poll()` call once free.
    pub fn poll(&self) -> Vec<([u8; 32], Vec<u8>)> {
        let mut new_messages = Vec::new();
        let mut new_messages_bytes: usize = 0;
        let snapshot: Vec<LinkHandle> = self.lock_links().clone();
        let mut dead = Vec::new();

        'links: for handle in &snapshot {
            let Ok(mut link) = handle.try_lock() else {
                // Busy (flush_reflood is sending on it right now) -- move
                // on rather than waiting; nothing here is lost, only
                // deferred to the next poll().
                continue;
            };
            for _ in 0..MAX_MESSAGES_PER_LINK_PER_POLL {
                // Aggregate cap across every link this call, not just this
                // one: see MAX_NEW_MESSAGES_PER_POLL/_BYTES. Remaining
                // links (and the rest of this one) are left for the next
                // poll() call rather than growing this batch further.
                if new_messages.len() >= MAX_NEW_MESSAGES_PER_POLL
                    || new_messages_bytes >= MAX_NEW_MESSAGES_PER_POLL_BYTES
                {
                    break 'links;
                }
                match link.try_recv() {
                    Ok(Some(payload)) => {
                        let id = message_id(&payload);
                        if self.lock_seen().record_seen(id) {
                            // Bounded (both by entry count and total bytes):
                            // under sustained overload, the oldest
                            // not-yet-reflooded payload is dropped rather
                            // than growing this queue without bound or
                            // making poll() itself start blocking to keep
                            // up. The payload is still returned to the
                            // caller below either way — this only bounds
                            // the *relay* obligation to other links, not
                            // local delivery.
                            self.lock_pending().push(payload.clone());
                            new_messages_bytes += payload.len();
                            new_messages.push((id, payload));
                        }
                        // A repeat: already relayed and delivered once, drop it.
                    }
                    Ok(None) => break,
                    // Terminal for this link: drop it from the mesh instead
                    // of retrying a dead connection forever.
                    Err(_) => {
                        dead.push(Arc::clone(handle));
                        break;
                    }
                }
            }
        }

        self.prune_dead(&dead);
        new_messages
    }

    /// Actually send every payload [`Self::poll`] has queued for reflooding,
    /// to every link still live at the time each send is attempted. This is
    /// the potentially **blocking** half of relaying — call it from
    /// whatever thread/schedule your platform can afford to have wait on a
    /// slow peer, never from the same tight loop that calls [`Self::poll`].
    ///
    /// Same terminal-on-send-failure pruning `broadcast()` uses — *except*
    /// for `FrameTooLarge`, which is never terminal: both `Channel::seal`'s
    /// own [`MAX_CHANNEL_PLAINTEXT_BYTES`] check and `EncryptedLink::send`'s
    /// bearer-capacity check (`max_sendable_plaintext_bytes`) run *before*
    /// seal, so a `FrameTooLarge` here means this link's AEAD counter never
    /// moved — it is simply too narrow (e.g. a BLE link's small MTU) to
    /// carry *this* relayed payload, which unlike a local `broadcast()`
    /// call was never checked against this link's capacity up front (it
    /// arrived from a *different*, possibly higher-capacity link, so no
    /// single preflight could have caught it). Pruning a perfectly healthy
    /// narrow link over one relayed message it cannot carry would partition
    /// it from the rest of the mesh for every future message too, including
    /// ones it easily could have carried; skip it for this message and keep
    /// it instead. Any other error still means the counter *did* advance
    /// (or the bearer itself failed) and the link really is desynced.
    ///
    /// Returns the number of payloads actually drained from the queue (sent
    /// to at least an attempt on every live link, whether or not every
    /// individual send succeeded).
    ///
    /// Per-link locking (a Codex review finding on PR #333) means a slow
    /// send here only ever blocks [`Self::poll`]'s *next* attempt on this
    /// exact link, never on any other link and never on this method itself
    /// — an earlier revision held one lock across the whole `MeshNode` for
    /// the entire flush, so a single slow peer's GATT write starved
    /// receiving on every other link too.
    pub fn flush_reflood(&self) -> usize {
        let mut flushed = 0;
        loop {
            let Some(payload) = self.lock_pending().pop() else {
                break;
            };
            let snapshot: Vec<LinkHandle> = self.lock_links().clone();
            let mut dead = Vec::new();
            for handle in &snapshot {
                let mut link = handle.lock().unwrap_or_else(|p| p.into_inner());
                match link.send(&payload) {
                    Ok(()) => {}
                    Err(BearerError::FrameTooLarge { .. }) => {}
                    Err(_) => dead.push(Arc::clone(handle)),
                }
            }
            self.prune_dead(&dead);
            flushed += 1;
        }
        flushed
    }

    /// How many payloads [`Self::poll`] has queued for reflooding but
    /// [`Self::flush_reflood`] has not yet sent.
    pub fn pending_reflood_count(&self) -> usize {
        self.lock_pending().len()
    }

    /// Convenience: [`Self::poll`] immediately followed by
    /// [`Self::flush_reflood`] on the same thread, for a caller that either
    /// doesn't need the split (tests, an in-process/TCP bearer where sends
    /// are fast) or hasn't yet wired a separate send-flushing schedule.
    /// Blocks exactly like the pre-split `poll()` used to. A platform bearer
    /// where a single send can stall (e.g. Android GATT) should call
    /// [`Self::poll`] and [`Self::flush_reflood`] separately, from
    /// different threads, instead of this.
    pub fn poll_and_flush(&self) -> Vec<([u8; 32], Vec<u8>)> {
        let messages = self.poll();
        self.flush_reflood();
        messages
    }
}

impl Default for MeshNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mini_bearer::InProcessBearer;

    fn boxed(bearer: InProcessBearer) -> Box<dyn Bearer + Send> {
        Box::new(bearer)
    }

    /// Connects `a` (dialer) to `b` (accepter) and adds the resulting link to
    /// each node. `dial`/`accept` each block on a `recv()` until the other
    /// side's hello/response arrives, so they must run concurrently, never
    /// sequentially on one thread (see [`mini_bearer::EncryptedLink`]'s own
    /// tests for why calling both in program order on one thread deadlocks)
    /// -- the accepter runs on its own thread here, the same shape a real
    /// two-device handshake naturally has.
    fn link(a: &mut MeshNode, b: &mut MeshNode) {
        let (bearer_a, bearer_b) = mini_bearer::pair();
        let boxed_b = boxed(bearer_b);
        let accepter = std::thread::spawn(move || EncryptedLink::accept(boxed_b).unwrap());
        a.add_link(EncryptedLink::dial(boxed(bearer_a)).unwrap());
        b.add_link(accepter.join().unwrap());
    }

    #[test]
    fn two_directly_linked_nodes_exchange_a_broadcast() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        a.broadcast(b"hello mesh").unwrap();
        let received = b.poll_and_flush();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1, b"hello mesh");
    }

    #[test]
    fn a_repeated_broadcast_is_deduped_not_delivered_twice() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        // Two separate links between the same two nodes: b should still only
        // ever deliver one copy of an id it has already seen.
        link(&mut a, &mut b);

        a.broadcast(b"only once").unwrap();
        let received = b.poll_and_flush();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1, b"only once");
    }

    /// The multi-hop proof: a **line** topology, A—B—C—D, with no direct
    /// A↔C, A↔D, or B↔D link — mirroring `mini-consensus`'s own real-socket
    /// four-node line-topology proof (D-0205) that dedup-flood relay makes
    /// any *connected* graph live, generalized here off TCP and onto any
    /// `Bearer`, proven hardware-free with `InProcessBearer`.
    #[test]
    fn a_broadcast_from_one_end_of_a_line_topology_reaches_the_other_end_via_relay() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        let mut c = MeshNode::new();
        let mut d = MeshNode::new();
        link(&mut a, &mut b);
        link(&mut b, &mut c);
        link(&mut c, &mut d);

        a.broadcast(b"from the far end").unwrap();

        // Round 1: B receives directly from A and re-floods (including back
        // toward C, its only other link).
        let at_b = b.poll_and_flush();
        assert_eq!(
            at_b,
            vec![(
                message_id(b"from the far end"),
                b"from the far end".to_vec()
            )]
        );

        // Round 2: C receives B's relay and re-floods toward D.
        let at_c = c.poll_and_flush();
        assert_eq!(
            at_c,
            vec![(
                message_id(b"from the far end"),
                b"from the far end".to_vec()
            )]
        );

        // Round 3: D, three hops from A with no direct link to A at all,
        // receives it purely through relay.
        let at_d = d.poll_and_flush();
        assert_eq!(
            at_d,
            vec![(
                message_id(b"from the far end"),
                b"from the far end".to_vec()
            )]
        );
    }

    #[test]
    fn a_disconnected_pair_never_hears_a_broadcast_from_the_other_side() {
        // Two separate two-node meshes, never linked to each other at all --
        // a disconnected graph stays disconnected, exactly as it must on any
        // real network.
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        let mut x = MeshNode::new();
        let mut y = MeshNode::new();
        link(&mut x, &mut y);

        a.broadcast(b"never leaves this pair").unwrap();
        assert!(b.poll_and_flush().len() == 1);
        assert!(x.poll_and_flush().is_empty());
        assert!(y.poll_and_flush().is_empty());
    }

    #[test]
    fn a_node_recognizes_its_own_broadcast_echoed_back_and_does_not_redeliver_it() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        let id = a.broadcast(b"mine").unwrap();
        let at_b = b.poll_and_flush();
        assert_eq!(at_b, vec![(id, b"mine".to_vec())]);

        // b re-floods back toward a as part of poll()'s relay step; a must
        // not redeliver its own already-seen message.
        assert!(a.poll_and_flush().is_empty());
    }

    #[test]
    fn link_count_reflects_added_links() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        assert_eq!(a.link_count(), 0);
        link(&mut a, &mut b);
        assert_eq!(a.link_count(), 1);
        assert_eq!(b.link_count(), 1);
    }

    #[test]
    fn a_link_whose_peer_is_gone_is_pruned_by_broadcast_not_just_poll() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);
        assert_eq!(a.link_count(), 1);

        // Drop b before a ever calls poll(): the only way a's link can
        // learn its peer is gone is through a failed send, not try_recv.
        drop(b);

        a.broadcast(b"anyone there?").unwrap();
        assert_eq!(
            a.link_count(),
            0,
            "broadcast's own send failure must prune the dead link, not just poll()'s try_recv"
        );
    }

    #[test]
    fn a_dead_link_is_pruned_during_polls_own_reflood_step_too() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        let mut c = MeshNode::new();
        link(&mut a, &mut b);
        link(&mut b, &mut c);
        assert_eq!(b.link_count(), 2);

        // c is gone before b ever polls: b's receive from a will succeed
        // (that link is fine), but the reflood step's send toward c must
        // fail and prune that link too, not just the try_recv-failed ones.
        drop(c);

        a.broadcast(b"relay this").unwrap();
        let received = b.poll_and_flush();
        assert_eq!(received.len(), 1);
        assert_eq!(
            b.link_count(),
            1,
            "the reflood step must prune the link whose send failed, not just retain both"
        );
    }

    #[test]
    fn a_link_whose_peer_is_gone_is_pruned_by_poll_not_retried_forever() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);
        assert_eq!(a.link_count(), 1);

        // b (and the bearer half its link holds) is gone: a's link is now
        // permanently broken, the same as a real disconnected/decrypt-failed
        // BLE connection.
        drop(b);

        assert!(a.poll_and_flush().is_empty());
        assert_eq!(
            a.link_count(),
            0,
            "a dead link must be dropped, not retried on every future poll()"
        );
    }

    #[test]
    fn poll_never_drains_more_than_the_per_link_cap_in_one_call() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        for i in 0..(MAX_MESSAGES_PER_LINK_PER_POLL + 10) {
            a.broadcast(format!("message {i}").as_bytes()).unwrap();
        }
        let first_poll = b.poll_and_flush();
        assert_eq!(first_poll.len(), MAX_MESSAGES_PER_LINK_PER_POLL);

        // The remaining messages are still waiting, not lost -- a second
        // poll() picks up exactly the rest.
        let second_poll = b.poll_and_flush();
        assert_eq!(second_poll.len(), 10);
    }

    #[test]
    fn an_oversized_broadcast_is_rejected_without_touching_any_link() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        let mut c = MeshNode::new();
        link(&mut a, &mut b);
        link(&mut a, &mut c);
        assert_eq!(a.link_count(), 2);

        // Every held link would reject a payload this large identically --
        // it is a property of the payload, not of any one connection -- so
        // this must fail up front, before ever touching a link, rather than
        // calling send() on each one and having retain_mut's pruning treat
        // that shared, non-link-specific rejection as if every link had
        // independently gone bad.
        let oversized = vec![0u8; MAX_CHANNEL_PLAINTEXT_BYTES + 1];
        let result = a.broadcast(&oversized);
        assert!(matches!(result, Err(BearerError::FrameTooLarge { .. })));
        assert_eq!(
            a.link_count(),
            2,
            "an oversized local payload must not prune any healthy link"
        );

        // Both links are still genuinely usable afterward.
        a.broadcast(b"still works").unwrap();
        assert_eq!(b.poll_and_flush().len(), 1);
        assert_eq!(c.poll_and_flush().len(), 1);
    }

    /// Wraps an [`InProcessBearer`] with a caller-chosen
    /// [`Bearer::max_frame_bytes`], standing in for a real bearer with a
    /// narrower-than-[`mini_bearer::MAX_FRAME_BYTES`] limit -- e.g. a real
    /// BLE bearer's own chunk-count limit at a small MTU -- without needing
    /// a real BLE radio to prove the mesh-level behavior.
    struct BoundedBearer {
        inner: InProcessBearer,
        max: usize,
    }

    impl Bearer for BoundedBearer {
        fn send(&mut self, frame: &[u8]) -> mini_bearer::Result<()> {
            self.inner.send(frame)
        }
        fn recv(&mut self) -> mini_bearer::Result<Vec<u8>> {
            self.inner.recv()
        }
        fn try_recv(&mut self) -> mini_bearer::Result<Option<Vec<u8>>> {
            self.inner.try_recv()
        }
        fn max_frame_bytes(&self) -> Option<usize> {
            Some(self.max)
        }
    }

    /// Same shape as [`link`], but `a`'s side of the link is bounded to
    /// `max_ciphertext_bytes`.
    fn link_with_bound(a: &mut MeshNode, b: &mut MeshNode, max_ciphertext_bytes: usize) {
        let (bearer_a, bearer_b) = mini_bearer::pair();
        let bounded_a: Box<dyn Bearer + Send> = Box::new(BoundedBearer {
            inner: bearer_a,
            max: max_ciphertext_bytes,
        });
        let boxed_b = boxed(bearer_b);
        let accepter = std::thread::spawn(move || EncryptedLink::accept(boxed_b).unwrap());
        a.add_link(EncryptedLink::dial(bounded_a).unwrap());
        b.add_link(accepter.join().unwrap());
    }

    #[test]
    fn a_broadcast_too_large_for_one_links_bearer_is_rejected_without_touching_any_link() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        let mut c = MeshNode::new();
        // b's link is bounded far below MAX_CHANNEL_PLAINTEXT_BYTES -- the
        // same shape a real BLE link at a small MTU has (its chunk-count
        // limit, not the channel's own ~16 MiB cap, is the real bound) --
        // while c's link has no extra bound at all.
        link_with_bound(&mut a, &mut b, 64);
        link(&mut a, &mut c);
        assert_eq!(a.link_count(), 2);

        // Well under MAX_CHANNEL_PLAINTEXT_BYTES (so the earlier, coarser
        // check would let it through) but over what b's link can carry.
        let payload = vec![0u8; 100];
        let result = a.broadcast(&payload);
        assert!(matches!(result, Err(BearerError::FrameTooLarge { .. })));
        assert_eq!(
            a.link_count(),
            2,
            "a payload too large for one link's bearer must not prune any healthy link, \
             including that link itself"
        );

        // Both links still work afterward.
        a.broadcast(b"fits fine").unwrap();
        assert_eq!(b.poll_and_flush().len(), 1);
        assert_eq!(c.poll_and_flush().len(), 1);
    }

    #[test]
    fn a_relayed_payload_too_large_for_one_links_bearer_does_not_prune_that_link() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        let mut c = MeshNode::new();
        link(&mut a, &mut b);
        // Unlike a direct broadcast() call (which checks the payload
        // against every held link up front), a relayed message arrives
        // from a *different* link than the ones it gets reflooded across
        // -- no single preflight on a's side could have known c's bearer
        // capacity, so this is the one path where poll()'s own reflood
        // step is what actually has to make the right call.
        link_with_bound(&mut b, &mut c, 64);
        assert_eq!(b.link_count(), 2);

        // Fits a<->b (unbounded) and the channel cap, but not b<->c's
        // bearer capacity.
        let payload = vec![0u8; 100];
        a.broadcast(&payload).unwrap();

        let received = b.poll_and_flush();
        assert_eq!(received.len(), 1, "b still receives it from a");
        assert_eq!(
            b.link_count(),
            2,
            "the reflood send toward c must fail with FrameTooLarge, not prune the link -- \
             c's AEAD counter never advanced, so it is not desynced, just unable to carry \
             this one oversized message"
        );
        assert!(
            c.poll_and_flush().is_empty(),
            "c genuinely never received the oversized relay -- that part is a real, honest \
             delivery gap, just not a reason to drop the connection"
        );

        // The link toward c is still genuinely usable for anything that
        // actually fits it.
        b.broadcast(b"fits fine").unwrap();
        assert_eq!(c.poll_and_flush().len(), 1);
    }

    #[test]
    fn poll_stages_a_relay_but_never_sends_it_until_flush_reflood_is_called() {
        // The core proof for the Codex finding this split closes: poll()
        // must be safe to call from a tight, frequent loop even when a
        // relay is due, because it never itself calls a possibly-blocking
        // Bearer::send -- only flush_reflood does.
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        let mut c = MeshNode::new();
        link(&mut a, &mut b);
        link(&mut b, &mut c);

        a.broadcast(b"relay me").unwrap();

        // b receives and dedups a's broadcast via poll() alone...
        let received = b.poll();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1, b"relay me");
        // ...but has not yet sent it onward to c.
        assert_eq!(b.pending_reflood_count(), 1);
        assert!(c.poll().is_empty());

        // Only flush_reflood actually performs the (potentially blocking) send.
        let flushed = b.flush_reflood();
        assert_eq!(flushed, 1);
        assert_eq!(b.pending_reflood_count(), 0);

        // Now c's own poll() finally sees it.
        let at_c = c.poll();
        assert_eq!(at_c.len(), 1);
        assert_eq!(at_c[0].1, b"relay me");
    }

    #[test]
    fn pending_reflood_drops_the_oldest_past_capacity_rather_than_growing_unbounded() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);
        // b has no other link to relay to, but poll() still queues every
        // distinct payload for reflooding regardless of whether any link
        // would actually receive it -- exercise the cap directly.
        for i in 0..(MAX_PENDING_REFLOOD + 10) {
            a.broadcast(format!("msg-{i}").as_bytes()).unwrap();
            b.poll();
        }
        assert_eq!(b.pending_reflood_count(), MAX_PENDING_REFLOOD);
    }

    #[test]
    fn pending_reflood_drops_the_oldest_once_the_byte_budget_is_exceeded_well_under_the_count_cap()
    {
        // Regression test for a Codex finding on PR #333: bounding only by
        // entry count still let a high-capacity peer queue up to
        // MAX_PENDING_REFLOOD * MAX_CHANNEL_PLAINTEXT_BYTES (tens of
        // gigabytes), since each accepted payload can be nearly
        // MAX_CHANNEL_PLAINTEXT_BYTES on its own. Large payloads here must
        // hit the byte budget and start evicting long before the count cap
        // (MAX_PENDING_REFLOOD, in the thousands) is anywhere close.
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        let big = vec![0u8; 1_000_000]; // 1 MiB
        let how_many = MAX_PENDING_REFLOOD_BYTES / big.len() + 4;
        assert!(
            how_many < MAX_PENDING_REFLOOD,
            "test setup must exercise the byte budget, not the count cap"
        );
        for _ in 0..how_many {
            // Each payload must be distinct or poll()'s own dedup (not the
            // reflood queue's bound) would be what's actually exercised.
            let mut payload = big.clone();
            payload.extend_from_slice(&mini_crypto::random_32().unwrap());
            a.broadcast(&payload).unwrap();
            b.poll();
        }
        assert!(
            b.pending_reflood_count() < how_many,
            "the byte budget must have evicted something well before the count cap would"
        );
    }

    /// A [`Bearer`] whose `send` blocks until a test-controlled gate opens —
    /// stands in for a real platform bearer's slow write (e.g. Android GATT
    /// waiting on a peer's acknowledgement) without needing real hardware.
    struct GatedSendBearer {
        inner: InProcessBearer,
        gate: std::sync::mpsc::Receiver<()>,
    }

    impl Bearer for GatedSendBearer {
        fn send(&mut self, frame: &[u8]) -> mini_bearer::Result<()> {
            let _ = self.gate.recv();
            self.inner.send(frame)
        }
        fn recv(&mut self) -> mini_bearer::Result<Vec<u8>> {
            self.inner.recv()
        }
        fn try_recv(&mut self) -> mini_bearer::Result<Option<Vec<u8>>> {
            self.inner.try_recv()
        }
    }

    #[test]
    fn poll_on_a_healthy_link_makes_progress_while_flush_reflood_is_blocked_sending_on_another() {
        // The core proof for the Codex finding this per-link locking closes:
        // an earlier revision put every link behind one lock shared with
        // flush_reflood's own blocking send, so a single slow peer starved
        // receiving on every other link too. Here `a` holds a link to a
        // gated (slow) peer `slow_peer` and a normal link to `fast_peer`;
        // flush_reflood is deliberately stuck sending toward `slow_peer` on
        // a background thread while the main thread's poll() must still see
        // `fast_peer`'s waiting message promptly.
        let a = std::sync::Arc::new(MeshNode::new());
        let slow_peer = MeshNode::new();
        let mut fast_peer = MeshNode::new();

        // a <-> slow_peer, with a's send side gated. `dial()` itself sends
        // exactly one frame (its hello), so pre-load one permit for the
        // handshake to go through -- the gate is empty again immediately
        // afterward, ready to block the real test send below.
        let (tx_gate, rx_gate) = std::sync::mpsc::channel::<()>();
        tx_gate.send(()).unwrap();
        let (bearer_a_slow, bearer_slow_a) = mini_bearer::pair();
        let gated: Box<dyn Bearer + Send> = Box::new(GatedSendBearer {
            inner: bearer_a_slow,
            gate: rx_gate,
        });
        let boxed_slow = boxed(bearer_slow_a);
        let accepter = std::thread::spawn(move || EncryptedLink::accept(boxed_slow).unwrap());
        a.add_link(EncryptedLink::dial(gated).unwrap());
        slow_peer.add_link(accepter.join().unwrap());

        // a <-> fast_peer, an ordinary unblocked link.
        link_shared(&a, &mut fast_peer);

        // Queue a message for both links to relay.
        fast_peer.broadcast(b"from fast peer").unwrap();
        // a receives it from fast_peer and queues it for reflood to every
        // link, including the gated one toward slow_peer.
        a.poll();
        assert_eq!(a.pending_reflood_count(), 1);

        // Start flush_reflood on a background thread: it will send to
        // fast_peer's link first or slow_peer's link first depending on
        // internal ordering, but either way it will block once it reaches
        // the gated link, since the gate has not been opened yet.
        let a_flusher = std::sync::Arc::clone(&a);
        let flusher = std::thread::spawn(move || a_flusher.flush_reflood());

        // Give the flusher a moment to actually reach (and block on) the
        // gated send.
        std::thread::sleep(std::time::Duration::from_millis(50));

        // fast_peer sends a second, independent message toward `a` on the
        // OTHER link. poll() must see it promptly -- it must not be stuck
        // waiting for flush_reflood's lock on the gated link.
        fast_peer
            .broadcast(b"second message, different link")
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut received = Vec::new();
        while received.is_empty() && std::time::Instant::now() < deadline {
            received = a.poll();
            if received.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        assert_eq!(
            received.len(),
            1,
            "poll() must keep receiving on the healthy link while flush_reflood \
             is still blocked sending on the gated one"
        );
        assert_eq!(received[0].1, b"second message, different link");

        // Release the gate so the background flusher can finish cleanly.
        // More than one payload can be queued for the gated link by now
        // (poll()'s retry loop above may have queued the second message
        // too), so send enough permits to cover every remaining send
        // rather than just one.
        for _ in 0..8 {
            let _ = tx_gate.send(());
        }
        flusher.join().unwrap();
    }

    /// Same connection dance as [`link`], but for an already-shared
    /// [`std::sync::Arc<MeshNode>`] on the dialing side.
    fn link_shared(a: &std::sync::Arc<MeshNode>, b: &mut MeshNode) {
        let (bearer_a, bearer_b) = mini_bearer::pair();
        let boxed_b = boxed(bearer_b);
        let accepter = std::thread::spawn(move || EncryptedLink::accept(boxed_b).unwrap());
        a.add_link(EncryptedLink::dial(boxed(bearer_a)).unwrap());
        b.add_link(accepter.join().unwrap());
    }
}
