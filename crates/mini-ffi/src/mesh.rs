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

use mini_bearer::{Bearer, EncryptedLink};

use crate::ble::{android_bearer, BleRadio};

/// One message drained from the mesh: its content id (BLAKE3 of the raw
/// payload — see `mini_mesh::message_id`) and the payload itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshMessage {
    pub id: Vec<u8>,
    pub payload: Vec<u8>,
}

/// UniFFI object wrapping one device's `mini_mesh::MeshNode`. No wrapper
/// lock of its own (a Codex review finding on PR #333: an earlier revision
/// put one here, which meant `flush_reflood`'s potentially slow send held
/// it for the whole call and blocked `poll()` from making progress on
/// every *other* link too) — `mini_mesh::MeshNode` is internally
/// synchronized per-link, exactly so `poll`/`flush_reflood`/`broadcast` can
/// be called concurrently from separate threads/schedules as this crate's
/// own docs already ask callers to do.
#[derive(Debug)]
pub struct MeshHandle {
    inner: mini_mesh::MeshNode,
}

impl MeshHandle {
    /// A mesh with no links yet.
    pub fn new() -> Self {
        MeshHandle {
            inner: mini_mesh::MeshNode::new(),
        }
    }

    /// Add a link for a BLE connection this device **dialed** (a central
    /// connection it just made to someone else's peripheral). Blocks for
    /// the `Channel` handshake round trip; `radio`/`mtu` are the same pair
    /// [`crate::ble::BleBearerHandle::new`] takes.
    pub fn add_dialed_link(&self, radio: Box<dyn BleRadio>, mtu: u32) -> Result<(), MeshError> {
        let bearer = android_bearer(radio, mtu);
        let boxed: Box<dyn Bearer + Send> = Box::new(bearer);
        let link = EncryptedLink::dial(boxed).map_err(|_| MeshError::HandshakeFailed)?;
        self.inner.add_link(link);
        Ok(())
    }

    /// Add a link for a BLE connection this device **accepted** (a
    /// peripheral connection a central just made to it). Blocks for the
    /// `Channel` handshake round trip.
    pub fn add_accepted_link(&self, radio: Box<dyn BleRadio>, mtu: u32) -> Result<(), MeshError> {
        let bearer = android_bearer(radio, mtu);
        let boxed: Box<dyn Bearer + Send> = Box::new(bearer);
        let link = EncryptedLink::accept(boxed).map_err(|_| MeshError::HandshakeFailed)?;
        self.inner.add_link(link);
        Ok(())
    }

    /// How many links are currently held.
    pub fn link_count(&self) -> u32 {
        self.inner.link_count() as u32
    }

    /// Send `payload` to every held link. Returns its content id.
    /// [`MeshError::PayloadTooLarge`] if `payload` exceeds
    /// [`mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES`] -- rejected up front,
    /// before touching any link, so an oversized local payload never
    /// mistakenly prunes every healthy link the way a real post-seal send
    /// failure legitimately does (see `mini_mesh::MeshNode::broadcast`'s
    /// own docs).
    pub fn broadcast(&self, payload: Vec<u8>) -> Result<Vec<u8>, MeshError> {
        self.inner
            .broadcast(&payload)
            .map(|id| id.to_vec())
            .map_err(|_| MeshError::PayloadTooLarge)
    }

    /// Drain and dedup whatever has arrived on any link so far, queuing new
    /// payloads to be reflooded — genuinely never blocks (unlike an earlier
    /// revision): the actual, potentially slow sends are
    /// [`Self::flush_reflood`]'s job. Call this often, from whatever thread
    /// polls for new messages; call `flush_reflood` separately, ideally
    /// from a different thread, since on a real platform bearer (Android
    /// GATT) a single send can wait seconds for a peer's acknowledgement —
    /// see `mini_mesh::MeshNode::poll`'s own docs for why the two are split.
    pub fn poll(&self) -> Vec<MeshMessage> {
        self.inner
            .poll()
            .into_iter()
            .map(|(id, payload)| MeshMessage {
                id: id.to_vec(),
                payload,
            })
            .collect()
    }

    /// Actually send every payload [`Self::poll`] has queued for reflooding.
    /// This is the potentially **blocking** half — see
    /// `mini_mesh::MeshNode::flush_reflood`'s own docs. Call it from a
    /// dedicated thread/schedule separate from whatever calls [`Self::poll`],
    /// so one slow peer's acknowledgement can never stall receiving and
    /// delivering messages from every other link.
    pub fn flush_reflood(&self) {
        self.inner.flush_reflood();
    }

    /// Convenience: [`Self::poll`] immediately followed by
    /// [`Self::flush_reflood`] on the same thread. Blocks like the
    /// pre-split `poll()` used to — fine for tests or a bearer where sends
    /// are always fast, but a real Android BLE deployment should call
    /// [`Self::poll`] and [`Self::flush_reflood`] separately instead.
    pub fn poll_and_flush(&self) -> Vec<MeshMessage> {
        let messages = self.poll();
        self.flush_reflood();
        messages
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
    /// A `broadcast` payload exceeded [`mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES`].
    PayloadTooLarge,
}

impl core::fmt::Display for MeshError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MeshError::HandshakeFailed => f.write_str("mesh link handshake failed"),
            MeshError::PayloadTooLarge => f.write_str("mesh broadcast payload too large"),
        }
    }
}

impl std::error::Error for MeshError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ble::BleRadioError;
    use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
    use std::sync::Mutex;

    struct MockRadio {
        tx: Sender<Vec<u8>>,
        rx: Mutex<Receiver<Vec<u8>>>,
        disconnected: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
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
        fn disconnect(&self) {
            if let Some(flag) = &self.disconnected {
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
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
                disconnected: None,
            }),
            Box::new(MockRadio {
                tx: tx_b,
                rx: Mutex::new(rx_a),
                disconnected: None,
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

        let id = mesh_a.broadcast(b"hello mesh".to_vec()).unwrap();
        let received = mesh_b.poll_and_flush();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, id);
        assert_eq!(received[0].payload, b"hello mesh");
    }

    #[test]
    fn a_link_pruned_after_its_peer_disappears_tells_the_platform_radio_to_disconnect() {
        // Regression test for a Codex finding on PR #333: when
        // mini_mesh::MeshNode prunes a link after a terminal try_recv
        // failure, dropping the Rust-side EncryptedLink used to have no
        // way to tell the platform (Android GATT) side to actually close
        // the connection -- the link's platform-side resources (a GATT
        // connection, a BlePeripheralServer.LinkState/BleMeshService
        // centralLinks entry) stayed live and occupied. RadioAdapter's
        // Drop impl (mini-ffi/src/ble.rs) now calls BleRadio::disconnect()
        // whenever the bearer wrapping a radio is dropped for any reason,
        // including exactly this mesh-pruning path.
        let (tx_a, rx_a) = channel();
        let (tx_b, rx_b) = channel();
        let disconnected = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let radio_a: Box<dyn BleRadio> = Box::new(MockRadio {
            tx: tx_a,
            rx: Mutex::new(rx_b),
            disconnected: Some(std::sync::Arc::clone(&disconnected)),
        });
        let radio_b: Box<dyn BleRadio> = Box::new(MockRadio {
            tx: tx_b,
            rx: Mutex::new(rx_a),
            disconnected: None,
        });

        let mesh_a = MeshHandle::new();
        let mesh_b = std::sync::Arc::new(MeshHandle::new());
        let mesh_b_accepter = std::sync::Arc::clone(&mesh_b);
        let accepter =
            std::thread::spawn(move || mesh_b_accepter.add_accepted_link(radio_b, 64).unwrap());
        mesh_a.add_dialed_link(radio_a, 64).unwrap();
        accepter.join().unwrap();
        assert_eq!(mesh_a.link_count(), 1);
        assert!(!disconnected.load(std::sync::atomic::Ordering::SeqCst));

        // Drop mesh_b's own side entirely -- this is the last surviving
        // handle to it (the accepter thread's clone was dropped when the
        // thread finished), so this drops mesh_b's MeshNode, its
        // EncryptedLink, its radio_b, and radio_b's `tx_b`: the sender
        // that fed radio_a's `rx`. Once that sender is gone, radio_a's
        // next read observes a terminal, not just an empty, channel.
        drop(mesh_b);

        // poll() drains radio_a: try_read_chunk() now reports
        // TryRecvError::Disconnected -> BleRadioError::Failed ->
        // BearerError::Closed -> MeshNode::poll()'s retain_mut prunes the
        // link -> the boxed AndroidBleBearer<RadioAdapter> (and so
        // RadioAdapter) is dropped -> RadioAdapter::drop() calls
        // radio_a.disconnect().
        mesh_a.poll_and_flush();
        assert_eq!(mesh_a.link_count(), 0);
        assert!(disconnected.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn an_oversized_broadcast_surfaces_payload_too_large_without_dropping_links() {
        let (radio_a, radio_b) = mock_pair();
        let mesh_a = MeshHandle::new();
        let mesh_b = std::sync::Arc::new(MeshHandle::new());
        let mesh_b_accepter = std::sync::Arc::clone(&mesh_b);
        let accepter =
            std::thread::spawn(move || mesh_b_accepter.add_accepted_link(radio_b, 64).unwrap());
        mesh_a.add_dialed_link(radio_a, 64).unwrap();
        accepter.join().unwrap();

        let oversized = vec![0u8; mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES + 1];
        let err = mesh_a.broadcast(oversized).unwrap_err();
        assert_eq!(err, MeshError::PayloadTooLarge);
        assert_eq!(
            mesh_a.link_count(),
            1,
            "the link must survive a rejected oversized payload"
        );

        // Still usable afterward.
        mesh_a.broadcast(b"fine".to_vec()).unwrap();
        assert_eq!(mesh_b.poll_and_flush().len(), 1);
    }
}
