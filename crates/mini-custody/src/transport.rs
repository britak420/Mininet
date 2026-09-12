//! Binds `mini_bearer::Channel` to a specific DKG session and a specific
//! sender/receiver identity pair, for carrying Round-2 packages (secret
//! key material) between two roster members over a real transport.
//!
//! `Channel`'s own docs are explicit about what its handshake alone
//! provides: "confidentiality, integrity, forward secrecy, and
//! unlinkability -- but *not* endpoint authentication, by design". This
//! module adds that missing authentication in two layers, both driven by
//! [`crate::manifest::CustodyParticipantV1::transport_identity_key`], not
//! by assuming the channel itself implies who is on the other end:
//!
//! 1. [`TransportBindingV1`] -- a one-time signed assertion, exchanged
//!    right after the handshake, that a specific roster member (by DID)
//!    is holding one end of a channel with a specific
//!    [`mini_bearer::Channel::channel_binding`] value, for a specific DKG
//!    session. Verified against the manifest's roster before any Round-2
//!    package is sent over the channel.
//! 2. Every sealed envelope after that binds session id, sender, and
//!    receiver into the AEAD associated data (see [`envelope_aad`]), so a
//!    ciphertext cannot be replayed onto a different session or a
//!    different sender/receiver pairing even if an attacker could redirect
//!    it to a live channel.
//!
//! No network I/O happens here -- see this crate's top-level "no network
//! transport" honest limit. The caller owns the actual `Channel` (built
//! over whichever real `Bearer` it has, `mini_bearer::TcpBearer` included)
//! and drives message exchange; this module only decides whether what was
//! signed/sealed is valid.

use did_mini::Did;
use mini_bearer::Channel;
use mini_crypto::SigningKey;

use crate::error::{CustodyError, Result};
use crate::manifest::DkgSessionManifestV1;
use crate::wire::push_bytes;

fn transport_binding_message(
    session_id: &[u8; 32],
    channel_binding: &[u8; 32],
    sender: &Did,
    receiver: &Did,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"mininet/custody/dkg/transport-binding/v1");
    out.extend_from_slice(session_id);
    out.extend_from_slice(channel_binding);
    push_bytes(&mut out, sender.as_str().as_bytes());
    push_bytes(&mut out, receiver.as_str().as_bytes());
    out
}

/// One roster member's signed claim to be holding one end of a specific
/// channel, for a specific session, talking to a specific peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportBindingV1 {
    pub session_id: [u8; 32],
    pub channel_binding: [u8; 32],
    pub sender: Did,
    pub receiver: Did,
    pub signature: mini_crypto::Signature,
}

/// Sign a [`TransportBindingV1`] as `sender`, claiming `channel` for
/// `manifest`'s session and for `receiver` as the intended peer. Signs
/// with the device's `transport_identity_key` signing key, never the
/// `device_verifying_key` one -- these are deliberately separate keys
/// (see [`crate::manifest::CustodyParticipantV1`]'s docs).
pub fn sign_transport_binding(
    manifest: &DkgSessionManifestV1,
    channel: &Channel,
    sender: &Did,
    receiver: &Did,
    transport_signing_key: &SigningKey,
) -> Result<TransportBindingV1> {
    manifest
        .frost_identifier_of(sender)
        .ok_or(CustodyError::InvalidManifest(
            "sender is not a roster member",
        ))?;
    manifest
        .frost_identifier_of(receiver)
        .ok_or(CustodyError::InvalidManifest(
            "receiver is not a roster member",
        ))?;
    let session_id = manifest.session_id();
    let channel_binding = channel.channel_binding();
    let message = transport_binding_message(&session_id, &channel_binding, sender, receiver);
    Ok(TransportBindingV1 {
        session_id,
        channel_binding,
        sender: sender.clone(),
        receiver: receiver.clone(),
        signature: transport_signing_key.sign(&message),
    })
}

/// Verify a [`TransportBindingV1`] against `manifest` and the local
/// `channel`: the session matches, the channel binding matches this exact
/// channel (so the assertion cannot be replayed onto a different one), the
/// claimed sender/receiver match what the caller expected, and the
/// signature verifies under the roster's recorded `transport_identity_key`
/// for that sender.
pub fn verify_transport_binding(
    manifest: &DkgSessionManifestV1,
    channel: &Channel,
    expected_sender: &Did,
    expected_receiver: &Did,
    binding: &TransportBindingV1,
) -> Result<()> {
    if binding.session_id != manifest.session_id() {
        return Err(CustodyError::Round2EnvelopeMisbound);
    }
    if binding.channel_binding != channel.channel_binding() {
        return Err(CustodyError::Round2EnvelopeMisbound);
    }
    if &binding.sender != expected_sender || &binding.receiver != expected_receiver {
        return Err(CustodyError::Round2EnvelopeMisbound);
    }
    let position =
        manifest
            .frost_identifier_of(&binding.sender)
            .ok_or(CustodyError::InvalidManifest(
                "sender is not a roster member",
            ))?;
    let participant = &manifest.roster[(position - 1) as usize];
    let message = transport_binding_message(
        &binding.session_id,
        &binding.channel_binding,
        &binding.sender,
        &binding.receiver,
    );
    participant
        .transport_identity_key
        .verify(&message, &binding.signature)
        .map_err(|_| CustodyError::BadSignature)
}

/// AAD for a sealed Round-2 envelope: session id, sender DID, receiver
/// DID. Bound into every `Channel::seal`/`Channel::open` call so a
/// ciphertext authenticated for one session or one sender/receiver pairing
/// is never accepted for another, even over a channel that is otherwise
/// live and valid.
fn envelope_aad(session_id: &[u8; 32], sender: &Did, receiver: &Did) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(session_id);
    push_bytes(&mut out, sender.as_str().as_bytes());
    push_bytes(&mut out, receiver.as_str().as_bytes());
    out
}

/// Seal a serialized Round-2 package (or any other ceremony payload) for
/// `receiver`, as `sender`, bound to `session_id`.
pub fn seal_round2_envelope(
    channel: &mut Channel,
    session_id: &[u8; 32],
    sender: &Did,
    receiver: &Did,
    serialized_payload: &[u8],
) -> Result<Vec<u8>> {
    let aad = envelope_aad(session_id, sender, receiver);
    channel
        .seal(serialized_payload, &aad)
        .map_err(|e| CustodyError::Transport(e.to_string()))
}

/// Open a Round-2 envelope received from `sender`, addressed to `receiver`
/// (normally the local participant), bound to `session_id`. Fails closed
/// (AEAD authentication failure) if any of those three do not match what
/// the sender actually sealed under.
pub fn open_round2_envelope(
    channel: &mut Channel,
    session_id: &[u8; 32],
    sender: &Did,
    receiver: &Did,
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let aad = envelope_aad(session_id, sender, receiver);
    channel
        .open(ciphertext, &aad)
        .map_err(|e| CustodyError::Transport(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mini_bearer::{Initiator, Responder};
    use mini_crypto::{encoding, HashAlgorithm, Multihash, SigningKey};

    use crate::manifest::{CustodyDomain, CustodyParticipantV1, DkgSessionManifestV1};

    fn test_did(seed: u8) -> Did {
        let multihash = Multihash::of(HashAlgorithm::Blake3, &[seed]);
        let scid = encoding::encode(encoding::BASE58BTC, &multihash.to_bytes()).unwrap();
        Did::from_scid(&scid).unwrap()
    }

    fn participant(seed: u8) -> (CustodyParticipantV1, SigningKey, SigningKey) {
        let device = SigningKey::from_seed(&[seed; 32]);
        let transport = SigningKey::from_seed(&[seed.wrapping_add(100); 32]);
        (
            CustodyParticipantV1 {
                custody_did: test_did(seed),
                device_verifying_key: device.verifying_key(),
                transport_identity_key: transport.verifying_key(),
            },
            device,
            transport,
        )
    }

    fn sample_manifest_and_keys() -> (DkgSessionManifestV1, Vec<(SigningKey, SigningKey)>) {
        let mut entries: Vec<_> = (0u8..11).map(participant).collect();
        entries.sort_by(|a, b| a.0.custody_did.as_str().cmp(b.0.custody_did.as_str()));
        let roster = entries.iter().map(|e| e.0.clone()).collect();
        let keys = entries.into_iter().map(|(_, d, t)| (d, t)).collect();
        let manifest = DkgSessionManifestV1 {
            network_id: [7u8; 32],
            custody_domain: CustodyDomain(1),
            custody_epoch: 0,
            attempt: 0,
            roster,
            authorization_object_id: [9u8; 32],
            previous_epoch: 0,
            previous_group_key: [0u8; 32],
            software_release_id: [3u8; 32],
            expiry_height: 0,
        };
        (manifest, keys)
    }

    fn established_pair() -> (Channel, Channel) {
        let (initiator, hello) = Initiator::start().unwrap();
        let (responder_channel, responder_hello) = Responder::respond(&hello).unwrap();
        let initiator_channel = initiator.finish(&responder_hello).unwrap();
        (initiator_channel, responder_channel)
    }

    #[test]
    fn a_valid_transport_binding_verifies() {
        let (manifest, keys) = sample_manifest_and_keys();
        let (initiator_channel, _responder_channel) = established_pair();
        let sender = &manifest.roster[0].custody_did;
        let receiver = &manifest.roster[1].custody_did;
        let transport_key = &keys[0].1;

        let binding = sign_transport_binding(
            &manifest,
            &initiator_channel,
            sender,
            receiver,
            transport_key,
        )
        .unwrap();

        verify_transport_binding(&manifest, &initiator_channel, sender, receiver, &binding)
            .unwrap();
    }

    #[test]
    fn a_binding_for_a_different_channel_is_rejected() {
        let (manifest, keys) = sample_manifest_and_keys();
        let (initiator_channel, _responder_channel) = established_pair();
        let (other_channel, _) = established_pair();
        let sender = &manifest.roster[0].custody_did;
        let receiver = &manifest.roster[1].custody_did;
        let transport_key = &keys[0].1;

        let binding = sign_transport_binding(
            &manifest,
            &initiator_channel,
            sender,
            receiver,
            transport_key,
        )
        .unwrap();

        assert!(
            verify_transport_binding(&manifest, &other_channel, sender, receiver, &binding)
                .is_err()
        );
    }

    #[test]
    fn a_binding_signed_by_the_wrong_key_is_rejected() {
        let (manifest, keys) = sample_manifest_and_keys();
        let (initiator_channel, _responder_channel) = established_pair();
        let sender = &manifest.roster[0].custody_did;
        let receiver = &manifest.roster[1].custody_did;
        // Sign with participant 1's transport key while claiming to be
        // participant 0.
        let wrong_key = &keys[1].1;

        let binding =
            sign_transport_binding(&manifest, &initiator_channel, sender, receiver, wrong_key)
                .unwrap();

        assert!(verify_transport_binding(
            &manifest,
            &initiator_channel,
            sender,
            receiver,
            &binding
        )
        .is_err());
    }

    #[test]
    fn round2_envelopes_round_trip_and_are_bound_to_sender_receiver() {
        let (manifest, _keys) = sample_manifest_and_keys();
        let (mut initiator_channel, mut responder_channel) = established_pair();
        let session_id = manifest.session_id();
        let sender = manifest.roster[0].custody_did.clone();
        let receiver = manifest.roster[1].custody_did.clone();
        let payload = b"round-2 secret share bytes";

        let sealed = seal_round2_envelope(
            &mut initiator_channel,
            &session_id,
            &sender,
            &receiver,
            payload,
        )
        .unwrap();
        let opened = open_round2_envelope(
            &mut responder_channel,
            &session_id,
            &sender,
            &receiver,
            &sealed,
        )
        .unwrap();
        assert_eq!(opened, payload);
    }

    #[test]
    fn round2_envelope_opened_with_swapped_identities_is_rejected() {
        let (manifest, _keys) = sample_manifest_and_keys();
        let (mut initiator_channel, mut responder_channel) = established_pair();
        let session_id = manifest.session_id();
        let sender = manifest.roster[0].custody_did.clone();
        let receiver = manifest.roster[1].custody_did.clone();
        let payload = b"round-2 secret share bytes";

        let sealed = seal_round2_envelope(
            &mut initiator_channel,
            &session_id,
            &sender,
            &receiver,
            payload,
        )
        .unwrap();
        // Peer tries to open it as if sender/receiver were swapped.
        assert!(open_round2_envelope(
            &mut responder_channel,
            &session_id,
            &receiver,
            &sender,
            &sealed
        )
        .is_err());
    }

    #[test]
    fn transport_binding_rejects_non_roster_dids() {
        let (manifest, keys) = sample_manifest_and_keys();
        let (initiator_channel, _responder_channel) = established_pair();
        let stranger = test_did(200);
        let receiver = manifest.roster[1].custody_did.clone();
        let transport_key = &keys[0].1;

        assert!(sign_transport_binding(
            &manifest,
            &initiator_channel,
            &stranger,
            &receiver,
            transport_key
        )
        .is_err());
    }
}
