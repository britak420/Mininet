//! Binding a validator's real identity to an already-established anonymous
//! [`mini_bearer::Channel`] — roadmap R8's last remaining named gap:
//! `Channel`'s handshake proves two ends share a fresh, private session, but
//! nothing about *which* validator is on the other end of it.
//!
//! ## Why this does not replace `Channel`'s own handshake
//!
//! `Channel`'s anonymous, forward-secret handshake is a deliberate design
//! choice (D-0015 \[FREEZE\]), not an oversight an identity-carrying
//! transport handshake would leak who is talking to whom before a single
//! consensus byte moves. `mini-bearer`'s own docs already name the fix:
//! "Authenticity is a payload concern; presence attestations sign over
//! `Channel::channel_binding` so a signature cannot be transplanted onto a
//! different channel." `mini-presence` already does exactly this for two
//! devices' co-presence. This module is the same construction for a
//! validator: after the ordinary anonymous handshake completes, either side
//! can *additionally* prove which validator it is by signing the channel's
//! own binding with an already-delegated, [`Capabilities::VOTE`]-capable
//! device key — the same capability [`mini_chain::verify_vote`] already
//! requires to cast a vote, not a new one invented for this.
//!
//! ## What binding to `channel_binding` buys, for free
//!
//! [`ValidatorHandshakeAttestation`] carries no nonce, timestamp, or epoch —
//! deliberately. `channel_binding` is unique per handshake (fresh ephemeral
//! X25519 keys every time), so an attestation that verifies against *this*
//! channel's binding cannot be replayed against any other channel, past or
//! future, with the same or a different peer. Adding a nonce field on top
//! would only add something nothing checks.
//!
//! ## Identity is not admission (F-14)
//!
//! A device holding a real, validly-delegated `VOTE` capability from its
//! root is not by itself proof that root was ever admitted to *this*
//! deployment's validator set — or that it still is, after a removal.
//! Confusing "this identity checks out" with "this identity is
//! authorized" is exactly the mistake the anonymous channel underneath
//! invites if a caller stops one property short: [`Channel`]'s own
//! handshake proves two ends share a fresh, private session and nothing
//! about who; this module's attestation additionally proves *which*
//! validator identity is on the other end; neither proves that identity
//! currently belongs to the validator set a caller actually cares about.
//! [`verify_validator_handshake`]/[`recv_validator_handshake`] now take a
//! [`mini_chain::ValidatorSet`] and check membership as a distinct,
//! required step — not derived from, or substitutable for, the
//! delegation/`VOTE`-capability check above it.
//!
//! ## What this still does not close
//!
//! - **Opt-in, not required.** [`crate::net::TcpMesh`]'s links stay
//!   anonymous by its own existing design ("consensus messages
//!   self-identify... the transport only needs to move bytes to everyone,
//!   not know who is who") — this module is a capability a caller reaches
//!   for when link-level identity matters (admitting only known validators
//!   to a connection, attributing a wedged or hostile link to a root), not
//!   a replacement for that design or something wired into `TcpMesh`
//!   itself.
//! - **Proves delegation, not honesty.** A validator that authenticates
//!   correctly is still free to go silent, censor, or propose invalid
//!   blocks — this closes "which identity is this," not "is this identity
//!   behaving."
//! - **No revocation check beyond the KEL a caller already has.** The same
//!   freshness limits [`mini_chain::verify_vote`]/`assess_kel_assurance`
//!   already carry apply here: a verifier holding a stale-but-not-yet-known-
//!   revoked device KEL still accepts that device's attestation.
//! - **Membership freshness is the caller's own responsibility.** A
//!   [`mini_chain::ValidatorSet`] is a snapshot; this module has no way to
//!   know whether the one it was handed reflects the current epoch. A
//!   caller checking against a stale set can still admit a since-removed
//!   root, or reject a since-added one — the same freshness obligation
//!   `verify_finality`'s own callers already carry for the sets they pass
//!   it.

use did_mini::{verify_delegation, Capabilities, Controller, Did, IndexedSig, Kel};
use mini_bearer::{Bearer, Channel};
use mini_chain::{ValidatorOracle, ValidatorSet};
use mini_crypto::{Signature, SignatureSuite};

use crate::error::{ConsensusError, Result};

/// AEAD associated data for a validator-handshake frame — distinct from
/// every other purpose [`Channel`] serves in this crate (`crate::net`'s
/// `CATCHUP_AAD`/`STATE_SYNC_AAD`/`CONSENSUS_AAD`, `crate::discovery`'s
/// `PEX_AAD`), so this ciphertext can never be replayed as if it meant
/// something else even though all reuse the same `Channel` primitive.
const HANDSHAKE_AAD: &[u8] = b"mini-consensus/validator-handshake-channel/v1";

/// Domain tag for the bytes a validator device signs — distinct from every
/// other signed transcript in this tree (`mini-chain`'s `VOTE_SIGN_DOMAIN`,
/// `mini-consensus::wire`'s proposal domain, `mini-presence`'s attestation
/// domain), so a signature over one can never be replayed as valid over
/// another even if the remaining bytes happened to collide.
const HANDSHAKE_SIGN_DOMAIN: &[u8] = b"mini-consensus/validator-handshake-sign/v1";

/// Wire framing tag, parsed but never signed — the same separation
/// `mini-chain::Vote` keeps between its own signed transcript and its wire
/// framing.
const WIRE_DOMAIN: &[u8] = b"mini-consensus/validator-handshake/v1";

/// Hard cap on device signatures accepted in one wire-encoded attestation.
/// Mirrors `did_mini::MAX_SIGNATURES` (F-10) rather than restating a
/// smaller number, matching [`mini_chain::Vote`]'s own bound on the same
/// untrusted field: a cap below did-mini's own would let a legitimate
/// threshold identity's attestation verify in memory and then fail to
/// decode its own encoding.
const MAX_SIGS: usize = did_mini::MAX_SIGNATURES;

/// Hard cap on the length of a `did:mini` string accepted from the wire —
/// far above any real SCID, purely an allocation bound on untrusted input.
const MAX_DID_BYTES: usize = 512;

/// A validator device's signed proof that it is on the other end of one
/// specific, already-established [`mini_bearer::Channel`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorHandshakeAttestation {
    /// The channel this attestation is bound to — [`verify_validator_handshake`]
    /// rejects any mismatch against the channel it is actually checked over.
    pub channel_binding: [u8; 32],
    /// The claimed validator identity root.
    pub validator_root: Did,
    /// The delegated device that signed this attestation.
    pub validator_device: Did,
    signature: Vec<IndexedSig>,
}

impl ValidatorHandshakeAttestation {
    fn transcript(channel_binding: &[u8; 32]) -> Vec<u8> {
        let mut w = Vec::with_capacity(HANDSHAKE_SIGN_DOMAIN.len() + 32);
        w.extend_from_slice(HANDSHAKE_SIGN_DOMAIN);
        w.extend_from_slice(channel_binding);
        w
    }

    /// The device signatures over this attestation's transcript.
    pub fn signature(&self) -> &[IndexedSig] {
        &self.signature
    }

    /// Canonical wire bytes: domain-tagged, every field width- or
    /// length-prefixed, so no two distinct attestations ever encode alike
    /// and the exact same bytes are produced on every platform — the same
    /// discipline [`mini_chain::Vote::to_wire_bytes`] uses.
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut w = Vec::new();
        w.extend_from_slice(WIRE_DOMAIN);
        w.extend_from_slice(&self.channel_binding);
        put_str(&mut w, self.validator_root.as_str());
        put_str(&mut w, self.validator_device.as_str());
        w.extend_from_slice(&(self.signature.len() as u32).to_be_bytes());
        for sig in &self.signature {
            w.extend_from_slice(&sig.index.to_be_bytes());
            w.push(sig.signature.suite().tag());
            w.extend_from_slice(&sig.signature.to_bytes());
        }
        w
    }

    /// Reconstruct an attestation from [`Self::to_wire_bytes`]. Purely
    /// structural: it validates the framing but makes no trust decision —
    /// [`verify_validator_handshake`] remains the only thing that decides
    /// whether it counts.
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        let mut r = SliceReader::new(bytes);
        let domain = r.take(WIRE_DOMAIN.len())?;
        if domain != WIRE_DOMAIN {
            return Err(ConsensusError::Malformed);
        }
        let mut channel_binding = [0u8; 32];
        channel_binding.copy_from_slice(r.take(32)?);
        let validator_root = take_did(&mut r)?;
        let validator_device = take_did(&mut r)?;
        let sig_count = r.u32()? as usize;
        if sig_count > MAX_SIGS {
            return Err(ConsensusError::Malformed);
        }
        let mut signature = Vec::with_capacity(sig_count);
        for _ in 0..sig_count {
            let index = r.u32()?;
            let suite = SignatureSuite::from_tag(r.u8()?).map_err(|_| ConsensusError::Malformed)?;
            let sig_bytes = r.take(suite.signature_len())?;
            let sig = Signature::from_suite_bytes(suite, sig_bytes)
                .map_err(|_| ConsensusError::Malformed)?;
            signature.push(IndexedSig {
                index,
                signature: sig,
            });
        }
        if !r.finished() {
            return Err(ConsensusError::Malformed);
        }
        Ok(ValidatorHandshakeAttestation {
            channel_binding,
            validator_root,
            validator_device,
            signature,
        })
    }
}

/// Sign an attestation binding `device` to `channel_binding`. Does not
/// itself check that `device` holds [`Capabilities::VOTE`] — that is
/// enforced on the verifying side ([`verify_validator_handshake`]), matching
/// this tree's convention that signing is local and free, verification is
/// where trust decisions happen.
pub fn sign_validator_handshake(
    channel_binding: [u8; 32],
    validator_root: &Did,
    device: &Controller,
) -> ValidatorHandshakeAttestation {
    let transcript = ValidatorHandshakeAttestation::transcript(&channel_binding);
    let signature = device.sign_message(&transcript);
    ValidatorHandshakeAttestation {
        channel_binding,
        validator_root: validator_root.clone(),
        validator_device: device.did(),
        signature,
    }
}

/// Verify one attestation against the channel it is presented over and the
/// supplied KELs: the channel binding must match exactly (so an attestation
/// captured on one channel can never be replayed on another), the supplied
/// KELs must actually be the attestation's claimed root/device, the device
/// must currently be a delegated, unrevoked, `VOTE`-capable device of that
/// root, the root must be a member of `validators` (F-14 — a real,
/// validly-delegated `VOTE`-capable root is not by itself proof it was ever
/// admitted to *this* deployment's validator set, or that it still is after
/// removal; this is a separate, caller-supplied fact this function does not
/// otherwise have any way to know), and the signature must verify over the
/// exact transcript. Returns the verified validator root on success.
pub fn verify_validator_handshake(
    attestation: &ValidatorHandshakeAttestation,
    expected_channel_binding: [u8; 32],
    validators: &ValidatorSet,
    root_kel: &Kel,
    device_kel: &Kel,
) -> Result<Did> {
    if attestation.channel_binding != expected_channel_binding {
        return Err(ConsensusError::ValidatorHandshakeChannelMismatch);
    }
    if device_kel.did().as_str() != attestation.validator_device.as_str() {
        return Err(ConsensusError::ValidatorHandshakeIdentityMismatch);
    }
    if root_kel.did().as_str() != attestation.validator_root.as_str() {
        return Err(ConsensusError::ValidatorHandshakeIdentityMismatch);
    }
    let caps = verify_delegation(root_kel, device_kel)?;
    if !caps.contains(Capabilities::VOTE) {
        return Err(ConsensusError::ValidatorHandshakeMissingVoteCapability);
    }
    if !validators.contains(&attestation.validator_root) {
        return Err(ConsensusError::ValidatorHandshakeNotAMember);
    }
    let transcript = ValidatorHandshakeAttestation::transcript(&attestation.channel_binding);
    device_kel.verify_message(&transcript, attestation.signature())?;
    Ok(attestation.validator_root.clone())
}

/// Sign and send a validator-handshake attestation over an already-
/// established [`Channel`] — the caller has already completed the ordinary
/// anonymous handshake (e.g. via [`mini_bearer::Initiator`]/
/// [`mini_bearer::Responder`]) before calling this.
pub fn send_validator_handshake(
    bearer: &mut dyn Bearer,
    channel: &mut Channel,
    validator_root: &Did,
    device: &Controller,
) -> Result<()> {
    let attestation = sign_validator_handshake(channel.channel_binding(), validator_root, device);
    let sealed = channel.seal(&attestation.to_wire_bytes(), HANDSHAKE_AAD)?;
    bearer.send(&sealed)?;
    Ok(())
}

/// Receive and verify a validator-handshake attestation over an already-
/// established [`Channel`], resolving the claimed root/device against
/// `oracle` and admission against `validators` (F-14 — see
/// [`verify_validator_handshake`]'s own doc comment for why this is a
/// separate fact from KEL/delegation resolution). Returns the verified
/// validator root.
pub fn recv_validator_handshake(
    bearer: &mut dyn Bearer,
    channel: &mut Channel,
    validators: &ValidatorSet,
    oracle: &dyn ValidatorOracle,
) -> Result<Did> {
    let sealed = bearer.recv()?;
    let plaintext = channel.open(&sealed, HANDSHAKE_AAD)?;
    let attestation = ValidatorHandshakeAttestation::from_wire_bytes(&plaintext)?;
    let root_kel = oracle
        .kel(&attestation.validator_root)
        .ok_or(ConsensusError::ValidatorHandshakeIdentityMismatch)?;
    let device_kel = oracle
        .kel(&attestation.validator_device)
        .ok_or(ConsensusError::ValidatorHandshakeIdentityMismatch)?;
    verify_validator_handshake(
        &attestation,
        channel.channel_binding(),
        validators,
        root_kel,
        device_kel,
    )
}

fn put_str(w: &mut Vec<u8>, s: &str) {
    w.extend_from_slice(&(s.len() as u32).to_be_bytes());
    w.extend_from_slice(s.as_bytes());
}

fn take_did(r: &mut SliceReader<'_>) -> Result<Did> {
    let len = r.u32()? as usize;
    if len > MAX_DID_BYTES {
        return Err(ConsensusError::Malformed);
    }
    let s = core::str::from_utf8(r.take(len)?).map_err(|_| ConsensusError::Malformed)?;
    Did::parse(s).map_err(|_| ConsensusError::Malformed)
}

/// A minimal cursor over untrusted bytes: every read is bounds-checked and
/// returns [`ConsensusError::Malformed`] on truncation, so a short or lying
/// frame can never index out of range or over-allocate — the same discipline
/// [`mini_chain::Vote::from_wire_bytes`]'s own reader uses.
struct SliceReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        SliceReader { buf, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or(ConsensusError::Malformed)?;
        let slice = self
            .buf
            .get(self.pos..end)
            .ok_or(ConsensusError::Malformed)?;
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32> {
        let mut a = [0u8; 4];
        a.copy_from_slice(self.take(4)?);
        Ok(u32::from_be_bytes(a))
    }

    fn finished(&self) -> bool {
        self.pos == self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_mini::Capabilities;

    fn validator() -> (Controller, Controller) {
        let mut root = Controller::incept_single_from_seeds(&[21u8; 32], &[22u8; 32]).unwrap();
        let device =
            Controller::incept_device_single_from_seeds(&root.did(), &[23u8; 32], &[24u8; 32])
                .unwrap();
        root.delegate_device(&device.did(), Capabilities::primary())
            .unwrap();
        (root, device)
    }

    /// A `ValidatorSet` containing exactly `root`.
    fn member_set(root: &Did) -> ValidatorSet {
        ValidatorSet::new(vec![root.clone()]).unwrap()
    }

    #[test]
    fn a_signed_attestation_survives_a_wire_round_trip_byte_for_byte() {
        let (root, device) = validator();
        let attestation = sign_validator_handshake([7u8; 32], &root.did(), &device);
        let bytes = attestation.to_wire_bytes();
        let back = ValidatorHandshakeAttestation::from_wire_bytes(&bytes).unwrap();
        assert_eq!(attestation, back);
        assert_eq!(
            verify_validator_handshake(
                &back,
                [7u8; 32],
                &member_set(&root.did()),
                &root.kel(),
                &device.kel()
            )
            .unwrap(),
            root.did()
        );
    }

    #[test]
    fn a_17_key_threshold_devices_attestation_round_trips_past_the_old_16_signature_cap() {
        // F-10: MAX_SIGS was hardcoded to 16, below did-mini's own
        // MAX_SIGNATURES (64) -- a legitimate threshold device with more
        // than 16 current keys could sign an attestation in memory
        // (`Controller::sign_message` emits one `IndexedSig` per current
        // key) and then fail to decode its own wire encoding. 17 is the
        // smallest count that exercises the old cap's exact off-by-one.
        let keys: Vec<_> = (0..17)
            .map(|_| mini_crypto::SigningKey::generate().unwrap())
            .collect();
        let next_keys: Vec<_> = (0..17)
            .map(|_| mini_crypto::SigningKey::generate().unwrap())
            .collect();
        let device = Controller::incept(keys, 17, next_keys, 17).unwrap();
        let root = Controller::incept_single().unwrap();
        let attestation = sign_validator_handshake([7u8; 32], &root.did(), &device);
        assert_eq!(attestation.signature.len(), 17);
        let back =
            ValidatorHandshakeAttestation::from_wire_bytes(&attestation.to_wire_bytes()).unwrap();
        assert_eq!(attestation, back);
    }

    #[test]
    fn an_attestation_from_unappointed_or_uncapable_devices_does_not_verify() {
        let (root, device) = validator();
        // A device the root never delegated at all.
        let stranger =
            Controller::incept_device_single_from_seeds(&root.did(), &[99u8; 32], &[98u8; 32])
                .unwrap();
        let forged = sign_validator_handshake([1u8; 32], &root.did(), &stranger);
        assert!(verify_validator_handshake(
            &forged,
            [1u8; 32],
            &member_set(&root.did()),
            &root.kel(),
            &stranger.kel()
        )
        .is_err());

        // A real device but a mismatched claimed root.
        let other_root = Controller::incept_single_from_seeds(&[41u8; 32], &[42u8; 32]).unwrap();
        let mismatched = sign_validator_handshake([1u8; 32], &other_root.did(), &device);
        assert!(verify_validator_handshake(
            &mismatched,
            [1u8; 32],
            &member_set(&other_root.did()),
            &other_root.kel(),
            &device.kel()
        )
        .is_err());
    }

    #[test]
    fn an_attestation_cannot_be_transplanted_onto_a_different_channel() {
        let (root, device) = validator();
        let attestation = sign_validator_handshake([5u8; 32], &root.did(), &device);
        assert!(verify_validator_handshake(
            &attestation,
            [6u8; 32], // a different channel's binding
            &member_set(&root.did()),
            &root.kel(),
            &device.kel()
        )
        .is_err());
    }

    #[test]
    fn a_device_without_vote_capability_cannot_authenticate_as_a_validator() {
        let mut root = Controller::incept_single_from_seeds(&[31u8; 32], &[32u8; 32]).unwrap();
        let device =
            Controller::incept_device_single_from_seeds(&root.did(), &[33u8; 32], &[34u8; 32])
                .unwrap();
        // Delegated, but only for posting -- never appointed to vote.
        root.delegate_device(&device.did(), Capabilities::POST)
            .unwrap();
        let attestation = sign_validator_handshake([9u8; 32], &root.did(), &device);
        assert_eq!(
            verify_validator_handshake(
                &attestation,
                [9u8; 32],
                &member_set(&root.did()),
                &root.kel(),
                &device.kel()
            ),
            Err(ConsensusError::ValidatorHandshakeMissingVoteCapability)
        );
    }

    // F-14: a valid, correctly-delegated VOTE-capable root is not by itself
    // proof of admission to a particular validator set -- these tests
    // reproduce the finding's own concrete example directly.

    #[test]
    fn a_genuinely_vote_capable_root_that_was_never_admitted_is_refused() {
        let (root, device) = validator();
        let attestation = sign_validator_handshake([10u8; 32], &root.did(), &device);
        // Every other check passes -- real delegation, real VOTE
        // capability, correct channel binding, valid signature -- but
        // `root` is not in the validator set the caller actually checks
        // against (e.g. some other identity root entirely).
        let someone_else = Controller::incept_single().unwrap();
        let unrelated_members = member_set(&someone_else.did());
        assert_eq!(
            verify_validator_handshake(
                &attestation,
                [10u8; 32],
                &unrelated_members,
                &root.kel(),
                &device.kel()
            ),
            Err(ConsensusError::ValidatorHandshakeNotAMember)
        );

        // The identical attestation against a set that actually contains
        // `root` still verifies normally -- refusing the nonmember case
        // did not also break the legitimate one.
        assert_eq!(
            verify_validator_handshake(
                &attestation,
                [10u8; 32],
                &member_set(&root.did()),
                &root.kel(),
                &device.kel()
            )
            .unwrap(),
            root.did()
        );
    }

    #[test]
    fn a_validator_removed_from_the_set_is_refused_even_with_a_fresh_attestation() {
        // "Obsolete membership epoch," restated as the caller's own
        // responsibility (this module's doc comment says so explicitly):
        // a `ValidatorSet` that no longer contains a once-admitted root
        // refuses it, exactly as if it had never been admitted. This
        // module cannot itself know an epoch changed -- it only ever
        // checks the set it is handed.
        let (root, device) = validator();
        let attestation = sign_validator_handshake([11u8; 32], &root.did(), &device);
        let current_epoch_members = member_set(&Controller::incept_single().unwrap().did());
        assert_eq!(
            verify_validator_handshake(
                &attestation,
                [11u8; 32],
                &current_epoch_members,
                &root.kel(),
                &device.kel()
            ),
            Err(ConsensusError::ValidatorHandshakeNotAMember)
        );
    }

    #[test]
    fn a_truncated_frame_is_rejected_not_panicked() {
        let (root, device) = validator();
        let bytes = sign_validator_handshake([2u8; 32], &root.did(), &device).to_wire_bytes();
        for cut in 0..bytes.len() {
            assert_eq!(
                ValidatorHandshakeAttestation::from_wire_bytes(&bytes[..cut]).unwrap_err(),
                ConsensusError::Malformed
            );
        }
    }

    #[test]
    fn trailing_garbage_after_a_valid_attestation_is_rejected() {
        let (root, device) = validator();
        let mut bytes = sign_validator_handshake([3u8; 32], &root.did(), &device).to_wire_bytes();
        bytes.push(0);
        assert_eq!(
            ValidatorHandshakeAttestation::from_wire_bytes(&bytes).unwrap_err(),
            ConsensusError::Malformed
        );
    }

    #[test]
    fn an_absurd_signature_count_is_rejected_before_allocating() {
        let (root, device) = validator();
        let good = sign_validator_handshake([4u8; 32], &root.did(), &device).to_wire_bytes();
        let count_pos = good.len() - (4 + 1 + SignatureSuite::Ed25519.signature_len()) - 4;
        let mut bytes = good.clone();
        bytes[count_pos..count_pos + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            ValidatorHandshakeAttestation::from_wire_bytes(&bytes).unwrap_err(),
            ConsensusError::Malformed
        );
    }

    #[test]
    fn a_signature_over_the_undomained_channel_binding_does_not_verify() {
        // Domain-confusion regression, mirroring mini-chain::vote's own: a
        // signature over the bare channel binding with no domain tag must
        // not verify as a handshake attestation.
        let (root, device) = validator();
        let binding = [8u8; 32];
        let legacy_signature = device.sign_message(&binding);
        let forged = ValidatorHandshakeAttestation {
            channel_binding: binding,
            validator_root: root.did(),
            validator_device: device.did(),
            signature: legacy_signature,
        };
        let members = member_set(&root.did());
        assert!(
            verify_validator_handshake(&forged, binding, &members, &root.kel(), &device.kel())
                .is_err()
        );

        let genuine = sign_validator_handshake(binding, &root.did(), &device);
        verify_validator_handshake(&genuine, binding, &members, &root.kel(), &device.kel())
            .unwrap();
    }

    // The real-transport wrapper: two real processes' worth of sockets, an
    // ordinary anonymous Channel handshake first, then this module's
    // attestation carried over it.
    use std::collections::BTreeMap;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    use mini_bearer::{Initiator, Responder, TcpBearer};

    #[derive(Default)]
    struct Directory(BTreeMap<String, Kel>);
    impl Directory {
        fn insert(&mut self, kel: Kel) {
            self.0.insert(kel.scid().to_string(), kel);
        }
    }
    impl ValidatorOracle for Directory {
        fn kel(&self, did: &Did) -> Option<&Kel> {
            self.0.get(did.scid())
        }
    }

    #[test]
    fn a_validator_authenticates_itself_over_a_real_socket() {
        let (root, device) = validator();
        let mut directory = Directory::default();
        directory.insert(root.kel().clone());
        directory.insert(device.kel().clone());

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let expected_root = root.did();
        let members = member_set(&root.did());

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut bearer = TcpBearer::from_stream(stream).unwrap();
            let hello = bearer.recv().unwrap();
            let (mut channel, hello_response) = Responder::respond(&hello).unwrap();
            bearer.send(&hello_response).unwrap();
            recv_validator_handshake(&mut bearer, &mut channel, &members, &directory).unwrap()
        });

        let stream = TcpStream::connect(addr).unwrap();
        let mut bearer = TcpBearer::from_stream(stream).unwrap();
        let (initiator, hello) = Initiator::start().unwrap();
        bearer.send(&hello).unwrap();
        let hello_response = bearer.recv().unwrap();
        let mut channel = initiator.finish(&hello_response).unwrap();
        send_validator_handshake(&mut bearer, &mut channel, &root.did(), &device).unwrap();

        let verified_root = server.join().unwrap();
        assert_eq!(verified_root, expected_root);
    }

    #[test]
    fn a_validator_handshake_crosses_the_wire_as_ciphertext_never_plaintext() {
        // Same regression class as crate::net's and crate::discovery's own
        // ciphertext tests: play the responder's handshake role by hand so
        // the raw sealed attestation can be inspected before it is opened.
        let (root, device) = validator();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let device_did_bytes = device.did().as_str().as_bytes().to_vec();

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut bearer = TcpBearer::from_stream(stream).unwrap();
            let hello = bearer.recv().unwrap();
            let (channel, hello_response) = Responder::respond(&hello).unwrap();
            bearer.send(&hello_response).unwrap();

            let sealed = bearer.recv().unwrap();
            assert!(
                !sealed
                    .windows(device_did_bytes.len())
                    .any(|w| w == device_did_bytes.as_slice()),
                "the claimed device's did:mini string must never appear on the wire unencrypted"
            );
            channel
        });

        let stream = TcpStream::connect(addr).unwrap();
        let mut bearer = TcpBearer::from_stream(stream).unwrap();
        let (initiator, hello) = Initiator::start().unwrap();
        bearer.send(&hello).unwrap();
        let hello_response = bearer.recv().unwrap();
        let mut channel = initiator.finish(&hello_response).unwrap();
        send_validator_handshake(&mut bearer, &mut channel, &root.did(), &device).unwrap();

        server.join().unwrap();
    }
}
