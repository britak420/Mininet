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

use mini_bearer::{Bearer, EncryptedLink};
use mini_crypto::HashAlgorithm;
use mini_net::GossipRouter;

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
    links: Vec<EncryptedLink<Box<dyn Bearer + Send>>>,
    seen: GossipRouter,
}

impl std::fmt::Debug for MeshNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshNode")
            .field("links", &self.links.len())
            .field("seen", &self.seen.len())
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
            links: Vec::new(),
            seen: GossipRouter::new(seen_capacity),
        }
    }

    /// Add a new live link — e.g. a BLE connection that just finished its
    /// [`EncryptedLink`] handshake. Takes effect on the next [`Self::broadcast`]
    /// or [`Self::poll`].
    pub fn add_link(&mut self, link: EncryptedLink<Box<dyn Bearer + Send>>) {
        self.links.push(link);
    }

    /// How many links are currently held. A link that has failed a
    /// `try_recv` is pruned by [`Self::poll`] (see its own docs), so this
    /// can only drop, never silently accumulate dead connections; it does
    /// not shrink on a `send` failure alone (best-effort, matching every
    /// other bearer-broadcast in this tree) until that link's next failed
    /// `try_recv`.
    pub fn link_count(&self) -> usize {
        self.links.len()
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
    pub fn broadcast(&mut self, payload: &[u8]) -> [u8; 32] {
        let id = message_id(payload);
        self.seen.record_seen(id);
        self.links.retain_mut(|link| link.send(payload).is_ok());
        id
    }

    /// Drain every link of up to [`MAX_MESSAGES_PER_LINK_PER_POLL`] waiting
    /// messages (non-blocking) and dedup-flood-relay them: the first time
    /// this node sees a given payload, it is re-sent across every
    /// still-live link (so a non-adjacent device hears it via relay — what
    /// makes any **connected** graph live, not just a full mesh) and
    /// returned to the caller; a repeat is silently dropped. A link whose
    /// `try_recv` fails (the bearer closed, the peer disconnected, a
    /// decode/decrypt failure) is a **terminal** failure for that link — it
    /// is dropped from the mesh in the same call, not retried on every
    /// future `poll()` — so this never panics and never accumulates dead
    /// links.
    pub fn poll(&mut self) -> Vec<([u8; 32], Vec<u8>)> {
        let mut new_messages = Vec::new();
        let mut to_reflood: Vec<Vec<u8>> = Vec::new();

        let seen = &mut self.seen;
        self.links.retain_mut(|link| {
            for _ in 0..MAX_MESSAGES_PER_LINK_PER_POLL {
                match link.try_recv() {
                    Ok(Some(payload)) => {
                        let id = message_id(&payload);
                        if seen.record_seen(id) {
                            to_reflood.push(payload.clone());
                            new_messages.push((id, payload));
                        }
                        // A repeat: already relayed and delivered once, drop it.
                    }
                    Ok(None) => return true,
                    // Terminal for this link: drop it from the mesh instead
                    // of retrying a dead connection forever.
                    Err(_) => return false,
                }
            }
            true
        });

        // Same terminal-on-send-failure pruning as broadcast(): a link that
        // fails mid-reflood is desynced (its AEAD counter already advanced
        // past what the peer received) and is dropped, not retried.
        for payload in &to_reflood {
            self.links.retain_mut(|link| link.send(payload).is_ok());
        }

        new_messages
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

        a.broadcast(b"hello mesh");
        let received = b.poll();
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

        a.broadcast(b"only once");
        let received = b.poll();
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

        a.broadcast(b"from the far end");

        // Round 1: B receives directly from A and re-floods (including back
        // toward C, its only other link).
        let at_b = b.poll();
        assert_eq!(
            at_b,
            vec![(
                message_id(b"from the far end"),
                b"from the far end".to_vec()
            )]
        );

        // Round 2: C receives B's relay and re-floods toward D.
        let at_c = c.poll();
        assert_eq!(
            at_c,
            vec![(
                message_id(b"from the far end"),
                b"from the far end".to_vec()
            )]
        );

        // Round 3: D, three hops from A with no direct link to A at all,
        // receives it purely through relay.
        let at_d = d.poll();
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

        a.broadcast(b"never leaves this pair");
        assert!(b.poll().len() == 1);
        assert!(x.poll().is_empty());
        assert!(y.poll().is_empty());
    }

    #[test]
    fn a_node_recognizes_its_own_broadcast_echoed_back_and_does_not_redeliver_it() {
        let mut a = MeshNode::new();
        let mut b = MeshNode::new();
        link(&mut a, &mut b);

        let id = a.broadcast(b"mine");
        let at_b = b.poll();
        assert_eq!(at_b, vec![(id, b"mine".to_vec())]);

        // b re-floods back toward a as part of poll()'s relay step; a must
        // not redeliver its own already-seen message.
        assert!(a.poll().is_empty());
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

        a.broadcast(b"anyone there?");
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

        a.broadcast(b"relay this");
        let received = b.poll();
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

        assert!(a.poll().is_empty());
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
            a.broadcast(format!("message {i}").as_bytes());
        }
        let first_poll = b.poll();
        assert_eq!(first_poll.len(), MAX_MESSAGES_PER_LINK_PER_POLL);

        // The remaining messages are still waiting, not lost -- a second
        // poll() picks up exactly the rest.
        let second_poll = b.poll();
        assert_eq!(second_poll.len(), 10);
    }
}
