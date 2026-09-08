//! Closes the gap this crate's own docs and `examples/gossip_live_demo.rs`
//! both name as still open: PEX discovers a dialable address
//! (`tests/pex_over_tcp.rs` already proves that alone), and
//! `GossipRouter`/`fanout_peers` dedup-flood a message across a *supplied*
//! peer list (`tests/net.rs` proves that alone) — but nothing before this
//! test proved the two compose: a node dialing and gossiping to a peer it
//! only ever learned about through peer exchange, never through a
//! hardcoded address, over real sockets.
//!
//! Topology: A knows only B. B already knows C (as if from an earlier
//! encounter). A runs one PEX round against B, discovers C purely from
//! that exchange, selects C as a fanout target via
//! [`mini_net::dialable_fanout`] (never a hardcoded peer list), dials C
//! directly — a fresh connection A never had before this test ran — and
//! gossips a message to it. C accepts the message exactly once.

use std::net::{TcpListener, TcpStream};
use std::thread;

use mini_bearer::{Bearer, TcpBearer};
use mini_crypto::HashAlgorithm;
use mini_net::{absorb_response, build_response, dialable_fanout, AddressBook};
use mini_net::{GossipRouter, PeerId, PexMessage, RoutingTable};

fn encode_message(text: &str) -> ([u8; 32], Vec<u8>) {
    let id = HashAlgorithm::Blake3.digest(text.as_bytes());
    let mut frame = Vec::with_capacity(32 + text.len());
    frame.extend_from_slice(&id);
    frame.extend_from_slice(text.as_bytes());
    (id, frame)
}

fn decode_message(frame: &[u8]) -> ([u8; 32], &str) {
    let (id_bytes, payload) = frame.split_at(32);
    let mut id = [0u8; 32];
    id.copy_from_slice(id_bytes);
    (id, std::str::from_utf8(payload).unwrap())
}

#[test]
fn a_node_gossips_to_a_peer_it_only_ever_learned_about_through_pex_over_real_tcp() {
    // C: listens for gossip, known to B ahead of time, never told to A
    // directly by anything except the PEX exchange below.
    let c_id = PeerId::generate().unwrap();
    let c_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let c_addr = c_listener.local_addr().unwrap();
    let c_thread = thread::spawn(move || {
        let (stream, _) = c_listener.accept().unwrap();
        let mut bearer = TcpBearer::from_stream(stream).unwrap();
        let mut router = GossipRouter::new(16);
        let frame = bearer.recv().unwrap();
        let (msg_id, text) = decode_message(&frame);
        assert!(
            router.record_seen(msg_id),
            "C must accept the message the first time it arrives"
        );
        text.to_string()
    });

    // B: knows C already; will hand A that record over a live PEX round.
    let b_id = PeerId::generate().unwrap();
    let mut b_routing = RoutingTable::new(b_id);
    let mut b_book = AddressBook::new();
    b_routing.insert(c_id);
    b_book.insert(c_id, c_addr);

    let b_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let b_addr = b_listener.local_addr().unwrap();
    let b_thread = thread::spawn(move || {
        let (stream, peer_addr) = b_listener.accept().unwrap();
        let mut bearer = TcpBearer::from_stream(stream).unwrap();
        let request_bytes = bearer.recv().unwrap();
        let PexMessage::Request(requester_id) = PexMessage::decode(&request_bytes).unwrap() else {
            panic!("expected a Request");
        };
        absorb_response(
            &[mini_net::PeerRecord {
                id: requester_id,
                addr: peer_addr,
            }],
            &mut b_routing,
            &mut b_book,
        );
        let response = build_response(&b_routing, &b_book, &requester_id);
        bearer.send(&response.encode()).unwrap();
    });

    // A: knows only B's address up front.
    let a_id = PeerId::generate().unwrap();
    let mut a_routing = RoutingTable::new(a_id);
    let mut a_book = AddressBook::new();

    let stream = TcpStream::connect(b_addr).unwrap();
    let mut bearer = TcpBearer::from_stream(stream).unwrap();
    bearer.send(&PexMessage::Request(a_id).encode()).unwrap();
    let response_bytes = bearer.recv().unwrap();
    let PexMessage::Response(records) = PexMessage::decode(&response_bytes).unwrap() else {
        panic!("expected a Response");
    };
    absorb_response(&records, &mut a_routing, &mut a_book);
    b_thread.join().unwrap();

    assert!(
        a_routing.contains(&c_id),
        "A must have learned C purely through the PEX round above"
    );

    // The fanout selection itself must be real, not hand-picked: ask for
    // C by routing position, exactly as a gossiping node would when
    // deciding who to forward a message to next.
    let targets = dialable_fanout(&a_routing, &a_book, &a_id, 4, Some(&b_id));
    assert_eq!(targets.len(), 1, "C is the only dialable peer besides B");
    assert_eq!(targets[0].id, c_id);
    assert_eq!(targets[0].addr, c_addr);

    // Dial the fanout target -- a connection A never had until this
    // moment -- and gossip to it.
    let mut a_router = GossipRouter::new(16);
    let (msg_id, frame) = encode_message("hello from A, discovered via PEX");
    assert!(a_router.record_seen(msg_id));
    let mut to_c = TcpBearer::from_stream(TcpStream::connect(targets[0].addr).unwrap()).unwrap();
    to_c.send(&frame).unwrap();

    let received = c_thread.join().unwrap();
    assert_eq!(received, "hello from A, discovered via PEX");
}

#[test]
fn a_message_already_seen_is_never_forwarded_back_to_its_own_sender() {
    // Three real routing tables/address books, no sockets needed for this
    // one -- it isolates the exclusion guarantee `dialable_fanout` gives a
    // caller wiring real gossip: the peer a message arrived from is never
    // itself a valid forwarding target, closing exactly the loop a naive
    // "fan out to my closest peers" implementation would create.
    let local = PeerId::generate().unwrap();
    let sender = PeerId::generate().unwrap();
    let other = PeerId::generate().unwrap();

    let mut routing = RoutingTable::new(local);
    let mut book = AddressBook::new();
    routing.insert(sender);
    routing.insert(other);
    book.insert(sender, "127.0.0.1:9001".parse().unwrap());
    book.insert(other, "127.0.0.1:9002".parse().unwrap());

    let targets = dialable_fanout(&routing, &book, &local, 8, Some(&sender));
    assert!(targets.iter().all(|record| record.id != sender));
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].id, other);
}
