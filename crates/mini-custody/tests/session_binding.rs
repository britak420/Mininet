//! Regression coverage for two Codex-review findings on PR #333: neither
//! [`session::round1_view_confirmed`] nor [`session::completion_confirmed`]
//! used to compare the signed session id against the manifest actually
//! being checked -- a complete, internally-consistent, validly-signed set
//! of acks/attestations from a *different* ceremony attempt (same roster,
//! retried with a new `attempt`, hence a different `session_id`) could
//! therefore satisfy the barrier for a manifest it was never signed for.
//! Both functions now bind to `manifest.session_id()` directly.

use did_mini::Did;
use mini_crypto::{encoding, HashAlgorithm, Multihash, SigningKey};

use mini_custody::manifest::{CustodyDomain, CustodyParticipantV1, DkgSessionManifestV1};
use mini_custody::session;

fn test_did(seed: u8) -> Did {
    let multihash = Multihash::of(HashAlgorithm::Blake3, &[seed]);
    let scid = encoding::encode(encoding::BASE58BTC, &multihash.to_bytes()).unwrap();
    Did::from_scid(&scid).unwrap()
}

struct TestParticipant {
    did: Did,
    device_key: SigningKey,
    // A distinct key from `device_key`: `manifest::validate` rejects a
    // roster that reuses one key across both protocol roles.
    transport_key: SigningKey,
}

fn build_roster(attempt: u32) -> (DkgSessionManifestV1, Vec<TestParticipant>) {
    let mut participants: Vec<TestParticipant> = (0u8..11)
        .map(|seed| TestParticipant {
            did: test_did(seed),
            device_key: SigningKey::from_seed(&[seed; 32]),
            transport_key: SigningKey::from_seed(&[seed.wrapping_add(100); 32]),
        })
        .collect();
    participants.sort_by(|a, b| a.did.as_str().cmp(b.did.as_str()));

    let roster: Vec<CustodyParticipantV1> = participants
        .iter()
        .map(|p| CustodyParticipantV1 {
            custody_did: p.did.clone(),
            device_verifying_key: p.device_key.verifying_key(),
            transport_identity_key: p.transport_key.verifying_key(),
        })
        .collect();

    let manifest = DkgSessionManifestV1 {
        network_id: [1u8; 32],
        custody_domain: CustodyDomain(1),
        custody_epoch: 1,
        attempt,
        roster,
        authorization_object_id: [2u8; 32],
        previous_epoch: 0,
        previous_group_key: [0u8; 32],
        software_release_id: [3u8; 32],
        expiry_height: 0,
    };
    manifest.validate().unwrap();
    (manifest, participants)
}

#[test]
fn round1_view_acks_from_a_retried_attempt_do_not_satisfy_a_different_manifest() {
    let (manifest_a, participants) = build_roster(0);
    let (manifest_b, _) = build_roster(1); // same roster, different attempt -> different session_id
    assert_ne!(manifest_a.session_id(), manifest_b.session_id());

    let root = [7u8; 32];
    let session_id_a = manifest_a.session_id();
    let acks: Vec<_> = participants
        .iter()
        .map(|p| session::sign_round1_view_ack(session_id_a, root, &p.did, &p.device_key))
        .collect();

    assert!(session::round1_view_confirmed(&manifest_a, &acks, &root));
    assert!(!session::round1_view_confirmed(&manifest_b, &acks, &root));
}

#[test]
fn completion_attestations_from_a_retried_attempt_do_not_satisfy_a_different_manifest() {
    let (manifest_a, participants) = build_roster(0);
    let (manifest_b, _) = build_roster(1);
    assert_ne!(manifest_a.session_id(), manifest_b.session_id());

    let session_id_a = manifest_a.session_id();
    let round1_root = [8u8; 32];
    let public_package_hash = [9u8; 32];
    let group_public_key = [10u8; 32];
    let attestations: Vec<_> = participants
        .iter()
        .map(|p| {
            session::sign_completion_attestation(
                session_id_a,
                round1_root,
                public_package_hash,
                group_public_key,
                &p.did,
                &p.device_key,
            )
        })
        .collect();

    assert!(session::completion_confirmed(&manifest_a, &attestations));
    assert!(!session::completion_confirmed(&manifest_b, &attestations));
}
