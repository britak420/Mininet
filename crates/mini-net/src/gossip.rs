//! Dedup-flooding gossip broadcast: the same "forward once, then drop
//! duplicates" shape as gossipsub's message cache, reimplemented as
//! Mininet-owned code (D-0034 point 3) rather than a dependency on it.
//!
//! ## Honest limits
//!
//! [`fanout_peers`]/[`dialable_fanout`] select deterministically
//! (closest-first) — simple, and fine when unpredictability doesn't
//! matter (a test, a single-writer local simulation). Real gossip
//! networks randomize fanout specifically to resist an attacker
//! positioning itself as every honest peer's "closest" neighbor and
//! silently dropping traffic (an eclipse attack, `docs/THREAT_MODEL.md`'s
//! "Routing attacks"/"Eclipse attacks" rows) — [`randomized_fanout_peers`]/
//! [`randomized_dialable_fanout`] are that hardening (D-0473). Neither
//! variant defends a candidate pool an attacker already fully controls;
//! randomizing *which* of 100% attacker-controlled candidates gets
//! selected changes nothing. What it raises is the cost of a *partial*
//! eclipse: an attacker who occupies some but not all of a victim's
//! nearby routing positions can no longer guarantee selection just by
//! being nearest, since selection no longer depends on distance alone.
//! Bucket refresh by liveness ping (`routing.rs`'s own honest limit,
//! still `pending`) is the complementary hardening this does not provide.

use std::collections::{HashMap, HashSet, VecDeque};

use mini_crypto::HashAlgorithm;

use crate::peer::PeerId;
use crate::pex::{AddressBook, PeerRecord};
use crate::routing::RoutingTable;

/// Tracks recently-seen message ids so a peer forwards each message at most
/// once, bounded so an attacker flooding distinct message ids cannot grow
/// this past its configured capacity (the same "cap before it can be used
/// as a resource-exhaustion vector" stance `mini-sync`'s KEL cache takes).
#[derive(Debug)]
pub struct GossipRouter {
    seen: HashSet<[u8; 32]>,
    order: VecDeque<[u8; 32]>,
    capacity: usize,
}

impl GossipRouter {
    /// A router that remembers at most `capacity` message ids before
    /// evicting the oldest to make room for new ones.
    pub fn new(capacity: usize) -> Self {
        GossipRouter {
            seen: HashSet::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    /// Record a message id as seen. Returns `true` the first time this id
    /// is recorded (the caller should forward it on), `false` on every
    /// subsequent call with the same id (already propagated — drop it).
    pub fn record_seen(&mut self, msg_id: [u8; 32]) -> bool {
        if !self.seen.insert(msg_id) {
            return false;
        }
        self.order.push_back(msg_id);
        if self.order.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        true
    }

    /// How many message ids are currently remembered.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Whether no message ids are currently remembered.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

/// Select up to `fanout` peers to forward a message to from a candidate
/// list already ordered nearest-first (e.g. from
/// [`crate::routing::RoutingTable::closest_peers`]). See the module-level
/// honest limit: this is deterministic, not randomized.
pub fn fanout_peers(candidates: &[PeerId], fanout: usize) -> Vec<PeerId> {
    candidates.iter().take(fanout).copied().collect()
}

/// Every peer this node both routes to (nearest `target` first) and can
/// dial, minus `exclude` — the candidate pool [`dialable_fanout`] and
/// [`randomized_dialable_fanout`] both select from, factored out so
/// neither reimplements the same routing/address-book/exclusion
/// composition.
fn dialable_candidates(
    routing: &RoutingTable,
    book: &AddressBook,
    target: &PeerId,
    exclude: Option<&PeerId>,
) -> Vec<PeerRecord> {
    routing
        .closest_peers(target, routing.len())
        .into_iter()
        .filter(|id| exclude != Some(id))
        .filter_map(|id| book.get(&id).map(|addr| PeerRecord { id, addr }))
        .collect()
}

/// Select up to `fanout` peers to forward a message to, the way a real
/// caller actually can: [`RoutingTable`] alone names *ids*, not addresses
/// ([`PeerId`]'s own docs), so a peer this node has only ever heard about
/// through routing — never through a [`crate::pex::PexMessage::Response`]
/// or a live connection's observed source address — cannot be dialed yet
/// and must not be handed to a caller as a fanout target. This composes
/// [`RoutingTable::closest_peers`], [`AddressBook::get`] and
/// [`fanout_peers`] into the one query a gossiping node actually needs:
/// the nearest peers to `target` it can both route to *and* dial, skipping
/// `exclude` (typically the peer a message just arrived from, so gossip
/// never bounces straight back to its own sender) and anything
/// routing-known but still address-less.
///
/// Pure and transport-agnostic, matching this crate's existing
/// "logic first, real socket later" pattern (see the crate-level docs'
/// honest limits) — nothing here dials anything; a caller does that with
/// the [`PeerRecord`]s returned. See the module-level honest limit: this
/// selects deterministically; [`randomized_dialable_fanout`] is the
/// eclipse-hardened alternative.
pub fn dialable_fanout(
    routing: &RoutingTable,
    book: &AddressBook,
    target: &PeerId,
    fanout: usize,
    exclude: Option<&PeerId>,
) -> Vec<PeerRecord> {
    dialable_candidates(routing, book, target, exclude)
        .into_iter()
        .take(fanout)
        .collect()
}

/// Domain separation for [`randomized_fanout_peers`]'s selection-key
/// derivation, so this can never collide with an unrelated seeded
/// derivation elsewhere in the tree (the same discipline
/// [`crate::gossip`]'s sibling crates' digest functions already apply).
const RANDOMIZED_FANOUT_DOMAIN: &[u8] = b"mini-net/gossip/randomized-fanout/v1";

/// Select up to `fanout` peers from `candidates`, pseudo-randomly rather
/// than by the caller's own ordering — the mitigation [`fanout_peers`]'s
/// own honest limit names as pending. An attacker who can position itself
/// as a victim's closest neighbor across every relevant routing bucket
/// controls every forwarding target forever under closest-first selection
/// alone (an eclipse attack); this makes selection depend on `seed`
/// instead of position.
///
/// Deterministic-from-seed: each candidate's selection key is
/// `BLAKE3(domain || seed || id)`, sorted ascending, first `fanout` kept —
/// the same domain-separated keyed-derivation shape
/// `mini_porep::sample_challenges` already uses for auditor challenge
/// sampling (D-0064). Reproducible for a caller (or a test) that knows
/// `seed`; unpredictable for a candidate peer that does not choose it. A
/// caller wanting genuine per-round unpredictability derives `seed` from
/// something no candidate controls in advance — fresh local randomness
/// ([`mini_crypto::random_32`]) drawn once per fanout decision, or
/// something message-specific a candidate could not have precomputed for.
/// A caller that reuses one fixed `seed` forever gets a *different* static
/// selection than [`fanout_peers`]'s closest-first order, not an
/// unpredictable one — `seed` freshness is what buys the eclipse
/// resistance, not the hashing alone.
pub fn randomized_fanout_peers(candidates: &[PeerId], fanout: usize, seed: &[u8]) -> Vec<PeerId> {
    let mut keyed: Vec<([u8; 32], PeerId)> = candidates
        .iter()
        .map(|id| {
            let mut bytes = Vec::with_capacity(RANDOMIZED_FANOUT_DOMAIN.len() + seed.len() + 32);
            bytes.extend_from_slice(RANDOMIZED_FANOUT_DOMAIN);
            bytes.extend_from_slice(seed);
            bytes.extend_from_slice(&id.0);
            (HashAlgorithm::Blake3.digest(&bytes), *id)
        })
        .collect();
    keyed.sort_by_key(|(key, _)| *key);
    keyed.into_iter().take(fanout).map(|(_, id)| id).collect()
}

/// [`randomized_fanout_peers`] composed with the same dialable-candidate
/// gathering [`dialable_fanout`] uses — the eclipse-hardened counterpart
/// to that function, over the same pool ([`RoutingTable::closest_peers`]
/// filtered to what [`AddressBook`] can actually dial, minus `exclude`).
/// See [`randomized_fanout_peers`]'s own docs for what `seed` must be to
/// actually buy the intended resistance.
pub fn randomized_dialable_fanout(
    routing: &RoutingTable,
    book: &AddressBook,
    target: &PeerId,
    fanout: usize,
    exclude: Option<&PeerId>,
    seed: &[u8],
) -> Vec<PeerRecord> {
    let mut by_id: HashMap<PeerId, PeerRecord> =
        dialable_candidates(routing, book, target, exclude)
            .into_iter()
            .map(|record| (record.id, record))
            .collect();
    let ids: Vec<PeerId> = by_id.keys().copied().collect();
    randomized_fanout_peers(&ids, fanout, seed)
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect()
}
