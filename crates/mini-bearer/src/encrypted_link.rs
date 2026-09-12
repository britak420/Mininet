//! [`EncryptedLink`] — any [`Bearer`] plus a [`Channel`] handshake, composed
//! once so a caller managing many simultaneous links (a BLE mesh, `mini-net`'s
//! future wide-area transport) does not hand-roll the same seal/open
//! bookkeeping [`crate::channel`] already provides. [`crate::TcpBearer`]'s own
//! consumer, `mini-consensus::net::TcpMesh`, does exactly this composition by
//! hand today because it predates this type and operates on a raw
//! [`std::net::TcpStream`] rather than a [`Bearer`] — this type is for every
//! future caller that already has a [`Bearer`] and just wants it encrypted.
//!
//! No new cryptography: the exact same ephemeral X25519 + HKDF-SHA256 +
//! ChaCha20-Poly1305 construction [`Initiator`]/[`Responder`]/[`Channel`]
//! already implement, and `mini-sync`/`mini-cli`/`mini-consensus` already use
//! over their own transports.

use crate::bearer::Bearer;
use crate::channel::{Channel, Initiator, Responder, AEAD_TAG_BYTES};
use crate::error::{BearerError, Result};

/// Associated data binding an [`EncryptedLink`] ciphertext to this specific
/// use, so a frame sealed for one purpose can never be replayed as if it
/// meant another — the same domain-separation discipline every sealed
/// transcript in this tree follows (e.g. `mini-consensus`'s own `CONSENSUS_AAD`/
/// `CATCHUP_AAD`).
const ENCRYPTED_LINK_AAD: &[u8] = b"mini-bearer/encrypted-link/v1";

/// Any [`Bearer`] with a [`Channel`] handshake already completed over it.
/// Confidentiality and forward secrecy are hop-by-hop, exactly [`Channel`]'s
/// own security model ("anonymous connection, valid payload" — see this
/// crate's top-level docs): this link authenticates nothing about the peer,
/// only that both ends share a fresh, private session.
#[derive(Debug)]
pub struct EncryptedLink<B: Bearer> {
    bearer: B,
    channel: Channel,
}

impl<B: Bearer> EncryptedLink<B> {
    /// Complete the handshake as the **initiator** over an already-connected
    /// `bearer` (blocks until the responder's hello arrives). Use this side
    /// for whichever end *dialed* — the BLE central that just connected to a
    /// peripheral, the TCP side that just called `connect`, matching every
    /// other handshake asymmetry in this tree (dialer initiates, accepter
    /// responds).
    pub fn dial(mut bearer: B) -> Result<Self> {
        let (initiator, hello) = Initiator::start()?;
        bearer.send(&hello)?;
        let response = bearer.recv()?;
        let channel = initiator.finish(&response)?;
        Ok(EncryptedLink { bearer, channel })
    }

    /// Complete the handshake as the **responder** over an already-connected
    /// `bearer` (blocks until the initiator's hello arrives). Use this side
    /// for whichever end *accepted* — the BLE peripheral a central just
    /// connected to, a TCP listener's accepted stream.
    pub fn accept(mut bearer: B) -> Result<Self> {
        let hello = bearer.recv()?;
        let (channel, response) = Responder::respond(&hello)?;
        bearer.send(&response)?;
        Ok(EncryptedLink { bearer, channel })
    }

    /// The established channel's binding — identical on both ends, unique
    /// per session. See [`Channel::channel_binding`].
    pub fn channel_binding(&self) -> [u8; 32] {
        self.channel.channel_binding()
    }

    /// The largest plaintext [`Self::send`] can actually carry over this
    /// link's bearer, if narrower than [`crate::MAX_CHANNEL_PLAINTEXT_BYTES`]
    /// (which [`Channel::seal`] itself already enforces). `None` means no
    /// bound beyond that. Lets a caller managing several links (e.g.
    /// `mini_mesh::MeshNode::broadcast`) reject a payload up front instead
    /// of discovering per-link that [`Self::send`] would have failed.
    pub fn max_sendable_plaintext_bytes(&self) -> Option<usize> {
        self.bearer
            .max_frame_bytes()
            .map(|max| max.saturating_sub(AEAD_TAG_BYTES))
    }

    /// Seal and send one frame.
    ///
    /// Checks [`Self::max_sendable_plaintext_bytes`] *before* calling
    /// [`Channel::seal`]: [`Channel::seal`] advances the channel's AEAD send
    /// counter unconditionally on success, so if sealing happened first and
    /// only the subsequent `bearer.send` failed because the bearer simply
    /// cannot carry a frame this large (e.g. a BLE bearer's chunk-count
    /// limit at its negotiated MTU -- far smaller than the channel's own
    /// ~16 MiB cap), the counter would already be ahead of what the peer
    /// received, permanently desyncing an otherwise perfectly healthy link
    /// over a condition this side could have known about in advance. This
    /// check makes that case a plain rejection instead of a terminal one.
    pub fn send(&mut self, plaintext: &[u8]) -> Result<()> {
        if let Some(max) = self.max_sendable_plaintext_bytes() {
            if plaintext.len() > max {
                return Err(BearerError::FrameTooLarge {
                    max,
                    got: plaintext.len(),
                });
            }
        }
        let ciphertext = self.channel.seal(plaintext, ENCRYPTED_LINK_AAD)?;
        self.bearer.send(&ciphertext)
    }

    /// Block until the next frame arrives, then open it.
    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let ciphertext = self.bearer.recv()?;
        self.channel.open(&ciphertext, ENCRYPTED_LINK_AAD)
    }

    /// Open the next frame if one is already available, `Ok(None)` otherwise.
    /// Never blocks.
    pub fn try_recv(&mut self) -> Result<Option<Vec<u8>>> {
        match self.bearer.try_recv()? {
            Some(ciphertext) => Ok(Some(self.channel.open(&ciphertext, ENCRYPTED_LINK_AAD)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inprocess::{pair, InProcessBearer};

    /// [`EncryptedLink::dial`]/[`EncryptedLink::accept`] each block on the
    /// bearer's `recv()` until the other side's hello/response arrives, so
    /// -- exactly like a real dial and a real accept -- the two sides must
    /// run concurrently, never sequentially on one thread (calling `dial`
    /// then `accept` in program order on the same thread deadlocks: `dial`
    /// blocks forever waiting for a response only a not-yet-called
    /// `accept` could ever send). This spawns the accepter on its own
    /// thread, the same shape a real two-process/two-device handshake
    /// naturally has.
    fn handshake(
        bearer_a: InProcessBearer,
        bearer_b: InProcessBearer,
    ) -> (
        EncryptedLink<InProcessBearer>,
        EncryptedLink<InProcessBearer>,
    ) {
        let accepter = std::thread::spawn(move || EncryptedLink::accept(bearer_b).unwrap());
        let dialer = EncryptedLink::dial(bearer_a).unwrap();
        (dialer, accepter.join().unwrap())
    }

    #[test]
    fn a_dialed_and_accepted_link_round_trip_a_frame_each_way() {
        let (bearer_a, bearer_b) = pair();
        let (mut a, mut b) = handshake(bearer_a, bearer_b);
        assert_eq!(a.channel_binding(), b.channel_binding());

        a.send(b"hello from the dialer").unwrap();
        assert_eq!(b.recv().unwrap(), b"hello from the dialer");

        b.send(b"hello from the accepter").unwrap();
        assert_eq!(a.recv().unwrap(), b"hello from the accepter");
    }

    #[test]
    fn try_recv_returns_none_with_nothing_pending_then_the_frame_once_sent() {
        let (bearer_a, bearer_b) = pair();
        let (mut a, mut b) = handshake(bearer_a, bearer_b);
        assert_eq!(b.try_recv().unwrap(), None);
        a.send(b"eventually").unwrap();
        assert_eq!(b.try_recv().unwrap(), Some(b"eventually".to_vec()));
    }

    #[test]
    fn two_independent_links_have_independent_bindings() {
        let (bearer_a1, bearer_b1) = pair();
        let (bearer_a2, bearer_b2) = pair();
        let (link1, _peer1) = handshake(bearer_a1, bearer_b1);
        let (link2, _peer2) = handshake(bearer_a2, bearer_b2);
        assert_ne!(link1.channel_binding(), link2.channel_binding());
    }

    #[test]
    fn a_closed_peer_surfaces_as_an_error_not_a_panic() {
        let (bearer_a, bearer_b) = pair();
        let (mut a, b) = handshake(bearer_a, bearer_b);
        drop(b);
        assert!(a.send(b"anyone there?").is_err() || a.recv().is_err());
    }

    /// Wraps an [`InProcessBearer`] with a caller-chosen
    /// [`Bearer::max_frame_bytes`], standing in for a real bearer with a
    /// narrower-than-[`crate::MAX_FRAME_BYTES`] limit (e.g.
    /// [`crate::AndroidBleBearer`] at a small MTU) without needing a real
    /// BLE radio to prove the behavior.
    struct BoundedBearer {
        inner: InProcessBearer,
        max: usize,
    }

    impl Bearer for BoundedBearer {
        fn send(&mut self, frame: &[u8]) -> Result<()> {
            self.inner.send(frame)
        }
        fn recv(&mut self) -> Result<Vec<u8>> {
            self.inner.recv()
        }
        fn try_recv(&mut self) -> Result<Option<Vec<u8>>> {
            self.inner.try_recv()
        }
        fn max_frame_bytes(&self) -> Option<usize> {
            Some(self.max)
        }
    }

    #[test]
    fn a_payload_over_the_bearers_own_capacity_is_rejected_without_desyncing_the_channel() {
        let (bearer_a, bearer_b) = pair();
        // 32-byte ciphertext cap => 16-byte plaintext cap (AEAD_TAG_BYTES == 16).
        let accepter = std::thread::spawn(move || EncryptedLink::accept(bearer_b).unwrap());
        let mut a = EncryptedLink::dial(BoundedBearer {
            inner: bearer_a,
            max: 32,
        })
        .unwrap();
        let mut b = accepter.join().unwrap();

        let oversized = vec![0u8; 100];
        let err = a.send(&oversized).unwrap_err();
        assert!(matches!(err, BearerError::FrameTooLarge { .. }));

        // The rejection must happen before Channel::seal ever runs: if the
        // send counter had advanced anyway, this next, perfectly
        // normal-size message would fail to decrypt on the peer's side.
        a.send(b"still fine").unwrap();
        assert_eq!(b.recv().unwrap(), b"still fine");
    }
}
