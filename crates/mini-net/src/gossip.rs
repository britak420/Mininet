//! Dedup-flooding gossip broadcast: the same "forward once, then drop
//! duplicates" shape as gossipsub's message cache, reimplemented as
//! Mininet-owned code (D-0034 point 3) rather than a dependency on it.
//!
//! ## Honest limits (first slice)
//!
//! Fanout selection here is deterministic (closest-first), not randomized.
//! Real gossip networks randomize fanout specifically to resist an attacker
//! positioning itself as every honest peer's "closest" neighbor and
//! silently dropping traffic (an eclipse attack); randomized, weighted
//! fanout selection is `pending` and tracked as follow-up hardening before
//! this crate carries real traffic.

use std::collections::{HashSet, VecDeque};

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
/// honest limit: this is deterministic, not randomized, for this slice.
pub fn fanout_peers(candidates: &[PeerId], fanout: usize) -> Vec<PeerId> {
    candidates.iter().take(fanout).copied().collect()
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
/// the [`PeerRecord`]s returned.
pub fn dialable_fanout(
    routing: &RoutingTable,
    book: &AddressBook,
    target: &PeerId,
    fanout: usize,
    exclude: Option<&PeerId>,
) -> Vec<PeerRecord> {
    routing
        .closest_peers(target, routing.len())
        .into_iter()
        .filter(|id| exclude != Some(id))
        .filter_map(|id| book.get(&id).map(|addr| PeerRecord { id, addr }))
        .take(fanout)
        .collect()
}
