# BLE multi-hop mesh relay

Closes the gap named in the founder's direction of 2026-09-11: devices should
be able to find each other and form a real network over BLE alone, so that a
group of nearby phones stays reachable to each other even if the internet
itself is down — not just two phones pairing directly. Companion to
`docs/BETA_STATUS.md` (the two-phone keystone target) and roadmap R10/R11;
this document is the scope package for the piece beyond either: **N devices,
not 2, relaying for each other over a possibly-disconnected direct-radio
graph.**

## What already exists, so this isn't invented twice

Per this repository's own "do not re-propose what already exists" rule
(`CLAUDE.md`), the actual relay *algorithm* this needs is already built and
proven, twice:

- `mini_net::GossipRouter` — bounded dedup ("forward the first time a
  message id is seen, drop every repeat") — pure, transport-agnostic, fully
  unit-tested, no bearer dependency.
- `mini_consensus::net::TcpMesh` + `run_to_height` — a **live, real-socket,
  proven implementation of exactly this shape**: a set of point-to-point
  encrypted links, dedup-flood re-gossip across them, and the explicit,
  tested claim "any **connected** graph is live — a vote reaches a
  non-adjacent peer via relay" (D-0205's real four-node *line*-topology TCP
  test: A—B—C—D, no direct A↔D link, A's message still reaches D). That test
  is the existing proof this exact relay shape works; a BLE mesh needs the
  same shape over a different bearer, not a new algorithm.

What's missing is a **transport-generic** version of that shape (TcpMesh is
hand-written against raw `TcpStream`, not the `mini_bearer::Bearer` trait,
so it cannot hold a BLE link), plus the real Android-side plumbing to hold
more than one BLE link at a time.

## Architecture

```
   BlePeripheralRadio ---\                              /--- BleCentralRadio
   (this phone as GATT     \                            /    (this phone as GATT
    server, N centrals      >--- mini_mesh::MeshNode ---<     client, M peripherals
    connected)              /    (one per device)        \    connected)
                           /                               \
                    each connection => one mini_bearer::Bearer
                    => wrapped as one EncryptedLink (Channel)
                    => held as one edge in the mesh
```

1. **`mini_bearer::Bearer`** (already real) is the one thing every kind of
   link — BLE, in-process, TCP — already implements identically. A device's
   set of *simultaneous* BLE connections (some where it's the GATT server,
   some where it's a GATT client to someone else's server) is therefore
   just a `Vec<Box<dyn Bearer + Send>>` from this layer's point of view; it
   does not need to know BLE exists.
2. **`mini_bearer::EncryptedLink<B: Bearer>`** (new, small, this crate) wraps
   one `Bearer` in a `Channel` handshake — the *same* ephemeral X25519 +
   HKDF-SHA256 + ChaCha20-Poly1305 construction `mini-sync`/`mini-cli`/
   `mini-consensus` already use, composed, not reinvented (no new
   cryptography). Dial/accept asymmetry mirrors BLE's own natural
   asymmetry: the GATT **central** (the side that discovered and connected)
   is the handshake **initiator**; the GATT **peripheral** (the side that
   advertised and accepted) is the **responder** — exactly `TcpMesh`'s own
   "dialer is initiator, accepter is responder" convention, just mapped
   onto BLE's roles instead of TCP's.
3. **`mini_mesh::MeshNode`** (new crate, depends on `mini-bearer` +
   `mini-net`) is `TcpMesh`/`run_to_height` generalized off raw sockets:
   holds a dynamic set of `EncryptedLink`s (links can be added at runtime as
   new BLE connections form — TcpMesh's topology is fixed at construction,
   this one is not, because BLE peers are discovered continuously, not
   handed a static address list up front), reuses `mini_net::GossipRouter`
   directly for dedup (no third reimplementation of the same seen-cache),
   and exposes `broadcast(payload)` / `poll() -> Vec<(message_id, payload)>`
   with the dedup-flood re-gossip already happening inside `poll()`.
4. **Android**: `BlePeripheralRadio` (existing, D-0502) is redesigned to
   track *multiple* connected centrals — `BluetoothGattServer` already
   supports this natively (every callback already carries the
   `BluetoothDevice` it's about) — and to hand each new connection to the
   app as a fresh `BleRadio`, one per remote central, instead of assuming
   exactly one. `BleCentralRadio` needs no such change: it is already
   "one instance = one connection," and a device wanting M simultaneous
   outbound links just holds M instances. Each new `BleRadio` becomes one
   `mini_mesh::MeshNode` edge as it's ready.

## Honest limits, stated up front rather than discovered later

- **Relay nodes see plaintext.** `EncryptedLink`'s confidentiality is
  hop-by-hop (defeats a passive BLE sniffer between any two phones), not
  end-to-end — an intermediate relaying phone necessarily decrypts a
  message to forward it, the same as any store-and-forward relay
  (`mini-consensus`'s own consensus nodes see every vote they relay too).
  A payload that must stay hidden from relay nodes needs its own
  end-to-end encryption before it enters the mesh; that is the payload's
  job, not this layer's, same stance the rest of this tree already takes.
- **No peer discovery here.** How a device *finds* which nearby devices to
  connect to (BLE advertising/scanning, matching a service UUID) is
  `BlePeripheralRadio`/`BleCentralRadio`'s job, unchanged by this design.
  This document is only about what happens once links already exist.
- **No routing, only flooding.** Every message reaches every connected
  device via dedup-flood, not a shortest-path route to one recipient — the
  same choice `mini-consensus` makes, appropriate for small mesh sizes
  (a handful of nearby phones), not evaluated at internet scale. Directed
  delivery (only device X should read this) is the payload's own job via
  addressing/encryption, exactly like the relay-sees-plaintext limit above.
- **No mesh size, battery, or range testing of any kind.** This document
  and the code it describes cover the algorithm and the Android wiring.
  Real behavior — how many simultaneous GATT connections a phone actually
  sustains, range, battery cost, what happens when a relay phone walks
  away mid-flood — is exactly `docs/gates/hardware-test-protocol.md`'s
  territory (roadmap R10, `outside`) and is **not** closed by code, here or
  anywhere else, regardless of how this reads once merged.
- **Unverified.** Same honest limit as D-0502: written without a JDK/
  Android SDK or BLE hardware available in this development environment.
  The `mini-mesh` crate's own logic is fully proven with `InProcessBearer`
  multi-node topology tests (no hardware needed, mirroring D-0205's
  existing TCP proof). The Android wiring on top of it is not, and cannot
  be, proven here.

## What PR #333 actually closes vs. what stays open

**Closes (real, tested, hardware-free):** the transport-generic
`EncryptedLink`/`mini_mesh::MeshNode` relay logic, with a multi-hop
line-topology test over `InProcessBearer` proving dedup-flood relay across
a disconnected direct graph — the same shape D-0205 already proved for TCP,
now bearer-generic.

**Closes (real code, unverified without hardware):** `BlePeripheralRadio`'s
multi-central redesign, the `mini-ffi` UniFFI surface exposing
`mini_mesh::MeshNode` to Kotlin, and the Android app wiring (runtime BLE
permission flow, simultaneous advertise+scan, handing new links to the
mesh).

**Stays open, not code:** the real two-or-more-device acceptance test
(roadmap R10/R11, hardware gate #97), and everything
`docs/gates/hardware-test-protocol.md` already names as needing a mobile
engineer with real phones. This document does not, and cannot, close
those — see this repo's own rule that a design doc's job is to prepare the
scope package and then stop, never to quietly downgrade a hardware gate
into something code can satisfy.
