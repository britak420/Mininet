//! Regression coverage for two Codex-review findings on PR #333: neither
//! [`session::round1_view_confirmed`] nor [`session::completion_confirmed`]
//! used to compare the signed session id against the manifest actually
//! being checked -- a complete, internally-consistent, validly-signed set
//! of acks/attestations from a *different* ceremony attempt (same roster,
//! retried with a new `attempt`, hence a different `session_id`) could
//! therefore satisfy the barrier for a manifest it was never signed for.
//! Both functions now bind to `manifest.session_id()` directly.

use std::collections::BTreeMap;

use did_mini::Did;
use frost_ristretto255::Identifier;
use mini_crypto::{encoding, HashAlgorithm, Multihash, SigningKey};
use rand_core::OsRng;

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

/// Regression test for a Codex finding: `dkg_part2` used to be a bare
/// wrapper callable without ever checking `round1_view_confirmed`, even
/// though this crate's own docs describe the Round-1 barrier as an
/// enforced ceremony property. A `Round1ViewConfirmation` obtained for one
/// manifest must not let Round 2 proceed for a *different* manifest (same
/// roster, retried with a new `attempt`, hence a different session id).
#[test]
fn a_round1_confirmation_from_a_different_manifest_is_rejected_by_dkg_part2() {
    let (manifest_a, participants) = build_roster(0);
    let (manifest_b, _) = build_roster(1);
    assert_ne!(manifest_a.session_id(), manifest_b.session_id());

    // Complete a genuine Round-1 confirmation for manifest_a.
    let mut round1_secrets = BTreeMap::new();
    let mut round1_packages = BTreeMap::new();
    for p in &participants {
        let (secret, package) = session::dkg_part1(&manifest_a, &p.did, OsRng).unwrap();
        let identifier =
            Identifier::try_from(manifest_a.frost_identifier_of(&p.did).unwrap()).unwrap();
        round1_secrets.insert(identifier, secret);
        round1_packages.insert(identifier, package);
    }
    let session_id_a = manifest_a.session_id();
    let ordered_hashes: Vec<[u8; 32]> = participants
        .iter()
        .map(|p| {
            let id_u16 = manifest_a.frost_identifier_of(&p.did).unwrap();
            let identifier = Identifier::try_from(id_u16).unwrap();
            let bytes = round1_packages[&identifier].serialize().unwrap();
            session::round1_package_hash(&session_id_a, id_u16, &bytes).unwrap()
        })
        .collect();
    let ordered_hashes: [[u8; 32]; 11] = ordered_hashes.try_into().unwrap();
    let root = session::round1_root(&session_id_a, &ordered_hashes);
    let acks: Vec<_> = participants
        .iter()
        .map(|p| session::sign_round1_view_ack(session_id_a, root, &p.did, &p.device_key))
        .collect();
    let confirmation_a = session::confirm_round1_view(&manifest_a, &acks, &root).unwrap();

    // A genuine confirmation for manifest_a must not authorize Round 2
    // against manifest_b, even with structurally valid Round-1 packages.
    let some_identifier = Identifier::try_from(1u16).unwrap();
    let own_secret = round1_secrets.remove(&some_identifier).unwrap();
    let others: BTreeMap<Identifier, frost_ristretto255::keys::dkg::round1::Package> =
        round1_packages
            .iter()
            .filter(|(id, _)| **id != some_identifier)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();
    let err = session::dkg_part2(&manifest_b, &confirmation_a, own_secret, &others).unwrap_err();
    assert_eq!(err, mini_custody::CustodyError::Round1ViewMismatch);
}
