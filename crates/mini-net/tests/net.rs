use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use mini_net::{
    dialable_fanout, fanout_peers, AddressBook, GossipRouter, PeerId, PeerRecord, RoutingTable,
    BUCKET_SIZE,
};

fn peer(seed: u8) -> PeerId {
    // Deterministic, non-random fixture ids for reproducible tests — the
    // same "fixed values are fine, they aren't secrets" convention used for
    // nonces elsewhere in this workspace (see mini-crypto::random_32 docs).
    // Real peer ids must always come from `PeerId::generate`.
    let mut bytes = [0u8; 32];
    bytes[31] = seed;
    PeerId(bytes)
}

#[test]
fn generated_peer_ids_are_unpredictable_and_distinct() {
    let a = PeerId::generate().unwrap();
    let b = PeerId::generate().unwrap();
    assert_ne!(a.0, [0u8; 32]);
    assert_ne!(a.0, b.0);
}

#[test]
fn xor_distance_is_zero_iff_equal() {
    let a = peer(1);
    let b = peer(2);
    assert_eq!(a.xor_distance(&a), [0u8; 32]);
    assert_ne!(a.xor_distance(&b), [0u8; 32]);
}

#[test]
fn bucket_index_is_none_for_self() {
    let a = peer(7);
    assert_eq!(a.bucket_index(&a), None);
}

#[test]
fn bucket_index_reflects_shared_prefix_length() {
    let local = peer(0);
    // Differ only in the very last bit -> maximal shared prefix -> bucket 0.
    let mut near = [0u8; 32];
    near[31] = 1;
    let near = PeerId(near);
    // Differ in the very first bit -> minimal shared prefix -> bucket 255.
    let mut far = [0u8; 32];
    far[0] = 0x80;
    let far = PeerId(far);

    assert_eq!(local.bucket_index(&near), Some(0));
    assert_eq!(local.bucket_index(&far), Some(255));
}

#[test]
fn routing_table_rejects_local_id() {
    let local = peer(1);
    let mut table = RoutingTable::new(local);
    assert!(!table.insert(local));
    assert_eq!(table.len(), 0);
}

#[test]
fn routing_table_rejects_duplicate_insert() {
    let local = peer(1);
    let mut table = RoutingTable::new(local);
    let other = peer(2);
    assert!(table.insert(other));
    assert!(!table.insert(other));
    assert_eq!(table.len(), 1);
}

#[test]
fn routing_table_bucket_has_bounded_capacity() {
    let local = peer(0);
    let mut table = RoutingTable::new(local);
    // Fix the highest set bit of the last byte (bit 5) so every generated
    // id has the same bucket index against `local` (all-zero), varying
    // only the low 5 bits to get distinct ids within that one bucket.
    let same_bucket_peer = |low_bits: u8| -> PeerId {
        let mut bytes = [0u8; 32];
        bytes[31] = 0x20 | (low_bits & 0x1F);
        PeerId(bytes)
    };
    let mut inserted = 0;
    for low_bits in 0..(BUCKET_SIZE as u8 + 5) {
        if table.insert(same_bucket_peer(low_bits)) {
            inserted += 1;
        }
    }
    assert_eq!(inserted, BUCKET_SIZE);
    assert_eq!(table.len(), BUCKET_SIZE);
}

#[test]
fn closest_peers_are_sorted_nearest_first() {
    let local = peer(0);
    let mut table = RoutingTable::new(local);
    for seed in [10u8, 3, 200, 50] {
        table.insert(peer(seed));
    }
    let target = peer(0);
    let closest = table.closest_peers(&target, 2);
    assert_eq!(closest.len(), 2);
    assert_eq!(closest[0], peer(3));
    assert_eq!(closest[1], peer(10));
}

#[test]
fn gossip_router_forwards_once_then_drops_duplicates() {
    let mut router = GossipRouter::new(10);
    let msg_id = [42u8; 32];
    assert!(router.record_seen(msg_id));
    assert!(!router.record_seen(msg_id));
    assert!(!router.record_seen(msg_id));
    assert_eq!(router.len(), 1);
}

#[test]
fn gossip_router_evicts_oldest_past_capacity() {
    let mut router = GossipRouter::new(2);
    let a = [1u8; 32];
    let b = [2u8; 32];
    let c = [3u8; 32];
    assert!(router.record_seen(a));
    assert!(router.record_seen(b));
    assert!(router.record_seen(c));
    assert_eq!(router.len(), 2);
    // `a` was evicted to make room for `c`, so it is treated as new again.
    assert!(router.record_seen(a));
}

#[test]
fn fanout_peers_caps_at_requested_size() {
    let candidates = vec![peer(1), peer(2), peer(3), peer(4)];
    let selected = fanout_peers(&candidates, 2);
    assert_eq!(selected, vec![peer(1), peer(2)]);

    let all = fanout_peers(&candidates, 10);
    assert_eq!(all.len(), 4);
}

fn addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}

#[test]
fn dialable_fanout_skips_routing_known_peers_with_no_address() {
    let local = peer(0);
    let mut routing = RoutingTable::new(local);
    let dialable = peer(10);
    let addressless = peer(20);
    routing.insert(dialable);
    routing.insert(addressless);

    let mut book = AddressBook::new();
    book.insert(dialable, addr(9001));
    // addressless deliberately has no recorded address -- routing-known
    // only, e.g. from a PEX response nobody has yet answered a follow-up
    // request for.

    let selected = dialable_fanout(&routing, &book, &local, 10, None);
    assert_eq!(
        selected,
        vec![PeerRecord {
            id: dialable,
            addr: addr(9001)
        }]
    );
}

#[test]
fn dialable_fanout_excludes_the_named_peer_even_if_dialable() {
    let local = peer(0);
    let mut routing = RoutingTable::new(local);
    let sender = peer(10);
    let other = peer(20);
    routing.insert(sender);
    routing.insert(other);

    let mut book = AddressBook::new();
    book.insert(sender, addr(9001));
    book.insert(other, addr(9002));

    // Without exclusion both would be dialable; excluding the sender must
    // drop it even though it has a perfectly good address on file.
    let selected = dialable_fanout(&routing, &book, &local, 10, Some(&sender));
    assert_eq!(
        selected,
        vec![PeerRecord {
            id: other,
            addr: addr(9002)
        }]
    );
}

#[test]
fn dialable_fanout_caps_at_the_requested_size_nearest_first() {
    let local = peer(0);
    let mut routing = RoutingTable::new(local);
    let mut book = AddressBook::new();
    for seed in [50u8, 3, 200, 10] {
        routing.insert(peer(seed));
        book.insert(peer(seed), addr(seed as u16));
    }

    let selected = dialable_fanout(&routing, &book, &local, 2, None);
    assert_eq!(selected.len(), 2);
    // closest_peers orders nearest-first by XOR distance to `local` (peer
    // 0); the two smallest seeds happen to be nearest for this fixture.
    assert_eq!(selected[0].id, peer(3));
    assert_eq!(selected[1].id, peer(10));
}

#[test]
fn dialable_fanout_is_empty_when_nothing_is_dialable() {
    let local = peer(0);
    let mut routing = RoutingTable::new(local);
    routing.insert(peer(1));
    let book = AddressBook::new();

    assert!(dialable_fanout(&routing, &book, &local, 10, None).is_empty());
}
