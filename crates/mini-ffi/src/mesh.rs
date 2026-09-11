//! UniFFI boundary over `mini_mesh::MeshNode` (`docs/design/ble-mesh-relay.md`,
//! D-0503) — lets Kotlin hand each real BLE connection, as it becomes ready,
//! to one shared mesh that dedup-flood-relays across all of them, instead of
//! driving each connection as an isolated point-to-point [`crate::ble::BleBearerHandle`].
//!
//! A [`MeshHandle`] owns the mesh; Kotlin calls [`MeshHandle::add_dialed_link`]
//! for a BLE **central** connection it just made (this side dialed, so it is
//! the `Channel` handshake initiator) and [`MeshHandle::add_accepted_link`]
//! for a BLE **peripheral** connection a central just made to it (this side
//! accepted, so it is the handshake responder) — the same dial/accept
//! asymmetry `mini_bearer::EncryptedLink`'s own docs describe. Both block the
//! calling thread for the handshake round trip, matching every other
//! blocking `mini-ffi` call Kotlin already drives off its IO dispatcher
//! (`RootCore::begin_pairing_offer`/`finish_pairing_offer`).
//!
//! **Honest limit**, same as [`crate::ble`] and every consumer of it: no new
//! cryptography (this reuses `EncryptedLink`'s already-established
//! `Channel` construction unchanged), but nothing here has been exercised
//! against a real radio in this development environment. `mini_mesh`'s own
//! multi-hop relay logic is separately, fully proven with
//! `mini_bearer::InProcessBearer` (no hardware needed); this module is only
//! the thin UniFFI adapter around it.

use std::sync::Mutex;

use mini_bearer::{Bearer, EncryptedLink};

use crate::ble::{android_bearer, BleRadio};

/// One message drained from the mesh: its content id (BLAKE3 of the raw
/// payload — see `mini_mesh::message_id`) and the payload itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshMessage {
    pub id: Vec<u8>,
    pub payload: Vec<u8>,
}

/// UniFFI object wrapping one device's `mini_mesh::MeshNode`.
#[derive(Debug)]
pub struct MeshHandle {
    inner: Mutex<mini_mesh::MeshNode>,
}

impl MeshHandle {
    /// A mesh with no links yet.
    pub fn new() -> Self {
        MeshHandle {
            inner: Mutex::new(mini_mesh::MeshNode::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, mini_mesh::MeshNode> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// Add a link for a BLE connection this device **dialed** (a central
    /// connection it just made to someone else's peripheral). Blocks for
    /// the `Channel` handshake round trip; `radio`/`mtu` are the same pair
    /// [`crate::ble::BleBearerHandle::new`] takes.
    pub fn add_dialed_link(&self, radio: Box<dyn BleRadio>, mtu: u32) -> Result<(), MeshError> {
        let bearer = android_bearer(radio, mtu);
        let boxed: Box<dyn Bearer + Send> = Box::new(bearer);
        let link = EncryptedLink::dial(boxed).map_err(|_| MeshError::HandshakeFailed)?;
        self.lock().add_link(link);
        Ok(())
    }

    /// Add a link for a BLE connection this device **accepted** (a
    /// peripheral connection a central just made to it). Blocks for the
    /// `Channel` handshake round trip.
    pub fn add_accepted_link(&self, radio: Box<dyn BleRadio>, mtu: u32) -> Result<(), MeshError> {
        let bearer = android_bearer(radio, mtu);
        let boxed: Box<dyn Bearer + Send> = Box::new(bearer);
        let link = EncryptedLink::accept(boxed).map_err(|_| MeshError::HandshakeFailed)?;
        self.lock().add_link(link);
        Ok(())
    }

    /// How many links are currently held.
    pub fn link_count(&self) -> u32 {
        self.lock().link_count() as u32
    }

    /// Send `payload` to every held link. Returns its content id.
    pub fn broadcast(&self, payload: Vec<u8>) -> Vec<u8> {
        self.lock().broadcast(&payload).to_vec()
    }

    /// Drain and dedup-flood-relay whatever has arrived on any link so far.
    /// Never blocks.
    pub fn poll(&self) -> Vec<MeshMessage> {
        self.lock()
            .poll()
            .into_iter()
            .map(|(id, payload)| MeshMessage {
                id: id.to_vec(),
                payload,
            })
            .collect()
    }
}

impl Default for MeshHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// FFI-facing failure from [`MeshHandle`]. Carries no message across the
/// boundary, matching [`crate::ble::BleBearerError`]'s own reasoning: a
/// failed handshake's exact cause (radio failure, malformed hello, a peer
/// that vanished mid-handshake) is platform detail this crate has no use
/// for beyond "did this succeed."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError {
    /// The `Channel` handshake did not complete.
    HandshakeFailed,
}

impl core::fmt::Display for MeshError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("mesh link handshake failed")
    }
}

impl std::error::Error for MeshError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ble::BleRadioError;
    use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

    struct MockRadio {
        tx: Sender<Vec<u8>>,
        rx: Mutex<Receiver<Vec<u8>>>,
    }

    impl BleRadio for MockRadio {
        fn write_chunk(&self, chunk: Vec<u8>) -> Result<(), BleRadioError> {
            self.tx.send(chunk).map_err(|_| BleRadioError::Failed)
        }
        fn read_chunk(&self) -> Result<Vec<u8>, BleRadioError> {
            self.rx
                .lock()
                .unwrap()
                .recv()
                .map_err(|_| BleRadioError::Failed)
        }
        fn try_read_chunk(&self) -> Result<Option<Vec<u8>>, BleRadioError> {
            match self.rx.lock().unwrap().try_recv() {
                Ok(v) => Ok(Some(v)),
                Err(TryRecvError::Empty) => Ok(None),
                Err(TryRecvError::Disconnected) => Err(BleRadioError::Failed),
            }
        }
    }

    fn mock_pair() -> (Box<dyn BleRadio>, Box<dyn BleRadio>) {
        let (tx_a, rx_a) = channel();
        let (tx_b, rx_b) = channel();
        (
            Box::new(MockRadio {
                tx: tx_a,
                rx: Mutex::new(rx_b),
            }),
            Box::new(MockRadio {
                tx: tx_b,
                rx: Mutex::new(rx_a),
            }),
        )
    }

    #[test]
    fn two_meshes_linked_over_mock_radios_exchange_a_broadcast() {
        let (radio_a, radio_b) = mock_pair();
        let mesh_a = MeshHandle::new();
        let mesh_b = std::sync::Arc::new(MeshHandle::new());

        // add_dialed_link/add_accepted_link each block on a handshake
        // round trip (see mini_bearer::EncryptedLink's own tests for why
        // that means the two sides must run concurrently, never
        // sequentially on one thread) -- in real use this is naturally two
        // separate devices; here mesh_b's accept runs on its own thread,
        // sharing the same handle via Arc so the test can still use it
        // (link_count/poll) after joining.
        let mesh_b_accepter = std::sync::Arc::clone(&mesh_b);
        let accepter =
            std::thread::spawn(move || mesh_b_accepter.add_accepted_link(radio_b, 64).unwrap());
        // mesh_a dialed (it is the BLE central here), mesh_b accepted.
        mesh_a.add_dialed_link(radio_a, 64).unwrap();
        accepter.join().unwrap();
        assert_eq!(mesh_a.link_count(), 1);
        assert_eq!(mesh_b.link_count(), 1);

        let id = mesh_a.broadcast(b"hello mesh".to_vec());
        let received = mesh_b.poll();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, id);
        assert_eq!(received[0].payload, b"hello mesh");
    }
}
