//! Real-socket proof that `mini_mesh::MeshNode` relays over genuine OS I/O,
//! not just the in-process channels `src/lib.rs`'s own unit tests use.
//!
//! Same four-node **line** topology as those unit tests (A—B—C—D, no direct
//! A↔C/A↔D/B↔D edge) and the same proof `mini-consensus`'s D-0205 real-socket
//! line-topology test established for TCP consensus messages — generalized
//! here to an arbitrary mesh payload, over real `TcpBearer` connections on
//! loopback TCP sockets across four real threads, each running its own
//! independent poll loop exactly as a real device would.

use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use mini_bearer::{Bearer, EncryptedLink, TcpBearer};
use mini_mesh::MeshNode;

fn boxed(bearer: TcpBearer) -> Box<dyn Bearer + Send> {
    Box::new(bearer)
}

/// Poll `mesh` until at least one message has been relayed through it or
/// `deadline` passes, returning every message seen. A real device runs this
/// loop forever; a test needs it bounded.
fn poll_until_nonempty(mesh: &MeshNode, deadline: Instant) -> Vec<([u8; 32], Vec<u8>)> {
    loop {
        let messages = mesh.poll_and_flush();
        if !messages.is_empty() || Instant::now() >= deadline {
            return messages;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// Keeps `mesh` polling (and therefore relaying) until `deadline`, ignoring
/// what it sees -- for an intermediate relay node (B, C) that must keep
/// forwarding even though the test only asserts delivery at the far end.
fn relay_until(mesh: &MeshNode, deadline: Instant) {
    while Instant::now() < deadline {
        mesh.poll_and_flush();
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn a_broadcast_from_one_end_of_a_real_tcp_line_topology_reaches_the_other_end_via_relay() {
    let listener_ab = TcpListener::bind("127.0.0.1:0").unwrap();
    let listener_bc = TcpListener::bind("127.0.0.1:0").unwrap();
    let listener_cd = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr_ab = listener_ab.local_addr().unwrap();
    let addr_bc = listener_bc.local_addr().unwrap();
    let addr_cd = listener_cd.local_addr().unwrap();

    const PAYLOAD: &[u8] = b"from the far end, over real sockets";
    // B and C's relay loops intentionally run for this whole window
    // regardless of how quickly delivery actually happens (a real relay
    // node keeps relaying, it doesn't stop after one message) -- kept
    // short so the test itself stays fast; loopback TCP delivery is
    // milliseconds, not seconds.
    let deadline_secs = 2;

    // D: accepts the C-D link, then polls (receives) until it sees the
    // message or times out.
    let (d_tx, d_rx) = channel();
    let d = thread::spawn(move || {
        let (stream, _) = listener_cd.accept().unwrap();
        let bearer = TcpBearer::from_stream(stream).unwrap();
        let link = EncryptedLink::accept(boxed(bearer)).unwrap();
        let mesh = MeshNode::new();
        mesh.add_link(link);
        let deadline = Instant::now() + Duration::from_secs(deadline_secs);
        let received = poll_until_nonempty(&mesh, deadline);
        d_tx.send(received).unwrap();
    });

    // C: accepts the B-C link, dials the C-D link, then keeps relaying
    // (polling) until the test's deadline -- it never itself originates or
    // consumes the message, only forwards it.
    let c = thread::spawn(move || {
        let (stream, _) = listener_bc.accept().unwrap();
        let bc_bearer = TcpBearer::from_stream(stream).unwrap();
        let bc_link = EncryptedLink::accept(boxed(bc_bearer)).unwrap();

        let cd_bearer = TcpBearer::connect(addr_cd).unwrap();
        let cd_link = EncryptedLink::dial(boxed(cd_bearer)).unwrap();

        let mesh = MeshNode::new();
        mesh.add_link(bc_link);
        mesh.add_link(cd_link);
        relay_until(&mesh, Instant::now() + Duration::from_secs(deadline_secs));
    });

    // B: accepts the A-B link, dials the B-C link, relays the same way.
    let b = thread::spawn(move || {
        let (stream, _) = listener_ab.accept().unwrap();
        let ab_bearer = TcpBearer::from_stream(stream).unwrap();
        let ab_link = EncryptedLink::accept(boxed(ab_bearer)).unwrap();

        let bc_bearer = TcpBearer::connect(addr_bc).unwrap();
        let bc_link = EncryptedLink::dial(boxed(bc_bearer)).unwrap();

        let mesh = MeshNode::new();
        mesh.add_link(ab_link);
        mesh.add_link(bc_link);
        relay_until(&mesh, Instant::now() + Duration::from_secs(deadline_secs));
    });

    // A: dials the A-B link and broadcasts once. No direct connection to
    // C or D exists anywhere in this test -- the only path to D is via
    // relay through B and C.
    let a = thread::spawn(move || {
        let ab_bearer = TcpBearer::connect(addr_ab).unwrap();
        let ab_link = EncryptedLink::dial(boxed(ab_bearer)).unwrap();
        let mesh = MeshNode::new();
        mesh.add_link(ab_link);
        mesh.broadcast(PAYLOAD).unwrap();
        // Keep the link alive long enough for B to finish reading it --
        // dropping `mesh` (and its TCP connection) immediately after
        // broadcast() queues the write could race the peer's read.
        thread::sleep(Duration::from_millis(300));
    });

    a.join().unwrap();
    b.join().unwrap();
    c.join().unwrap();
    d.join().unwrap();

    let received = d_rx.recv().unwrap();
    assert_eq!(
        received.len(),
        1,
        "D should receive exactly one relayed message"
    );
    assert_eq!(received[0].1, PAYLOAD);
    assert_eq!(received[0].0, mini_mesh::message_id(PAYLOAD));
}
