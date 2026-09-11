//! End-to-end proof that this crate's Phase A-G ceremony state machine
//! correctly drives `frost_ristretto255::keys::dkg`'s real DKG math for a
//! full 11-party roster, and that the resulting `KeyPackage`s are actually
//! usable to produce a valid 7-of-11 threshold Schnorr signature verifying
//! under the group's `PublicKeyPackage`. This is the concrete replacement
//! for the vulnerable hand-rolled DKG the Gate #93 audit report found
//! (`mini_treasury::frost_dkg`) -- if this test passes, the ceremony this
//! crate wraps produces a real, working FROST key, not just well-typed
//! plumbing around one.

use std::collections::BTreeMap;

use did_mini::Did;
use frost_ristretto255::keys::dkg::{round1, round2};
use frost_ristretto255::keys::{KeyPackage, PublicKeyPackage};
use frost_ristretto255::{
    round1 as sign_round1, round2 as sign_round2, Identifier, SigningPackage,
};
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
}

fn build_roster() -> (DkgSessionManifestV1, Vec<TestParticipant>) {
    let mut participants: Vec<TestParticipant> = (0u8..11)
        .map(|seed| TestParticipant {
            did: test_did(seed),
            device_key: SigningKey::from_seed(&[seed; 32]),
        })
        .collect();
    participants.sort_by(|a, b| a.did.as_str().cmp(b.did.as_str()));

    let roster: Vec<CustodyParticipantV1> = participants
        .iter()
        .map(|p| CustodyParticipantV1 {
            custody_did: p.did.clone(),
            device_verifying_key: p.device_key.verifying_key(),
            transport_identity_key: p.device_key.verifying_key(),
        })
        .collect();

    let manifest = DkgSessionManifestV1 {
        network_id: [1u8; 32],
        custody_domain: CustodyDomain(1),
        custody_epoch: 1,
        attempt: 0,
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
fn full_11_party_ceremony_produces_a_usable_threshold_key() {
    let (manifest, participants) = build_roster();

    // Phase A: every roster member accepts the manifest.
    let acceptances: Vec<_> = participants
        .iter()
        .map(|p| session::sign_manifest_acceptance(&manifest, &p.did, &p.device_key).unwrap())
        .collect();
    assert!(session::manifest_fully_accepted(&manifest, &acceptances));

    // Phase B: every participant runs DKG Round 1.
    let mut round1_secrets: BTreeMap<Identifier, round1::SecretPackage> = BTreeMap::new();
    let mut round1_packages: BTreeMap<Identifier, round1::Package> = BTreeMap::new();
    for p in &participants {
        let identifier =
            Identifier::try_from(manifest.frost_identifier_of(&p.did).unwrap()).unwrap();
        let (secret, package) = session::dkg_part1(&manifest, &p.did, OsRng).unwrap();
        round1_secrets.insert(identifier, secret);
        round1_packages.insert(identifier, package);
    }

    // Phase C: consistent Round-1 broadcast. Every participant computes
    // the package hashes in canonical (identifier) order and the same
    // root, then signs a view-ack over it.
    let session_id = manifest.session_id();
    let ordered_hashes: Vec<[u8; 32]> = participants
        .iter()
        .map(|p| {
            let id_u16 = manifest.frost_identifier_of(&p.did).unwrap();
            let identifier = Identifier::try_from(id_u16).unwrap();
            let bytes = round1_packages[&identifier].serialize().unwrap();
            session::round1_package_hash(&session_id, id_u16, &bytes).unwrap()
        })
        .collect();
    let ordered_hashes: [[u8; 32]; 11] = ordered_hashes.try_into().unwrap();
    let root = session::round1_root(&session_id, &ordered_hashes);

    let acks: Vec<_> = participants
        .iter()
        .map(|p| session::sign_round1_view_ack(session_id, root, &p.did, &p.device_key))
        .collect();
    assert!(session::round1_view_confirmed(
        &manifest,
        &acks,
        &session_id,
        &root
    ));

    // Phase D: DKG Round 2. Each participant's `round1_packages` argument
    // is every *other* participant's package.
    let mut round2_secrets: BTreeMap<Identifier, round2::SecretPackage> = BTreeMap::new();
    let mut round2_outboxes: BTreeMap<Identifier, BTreeMap<Identifier, round2::Package>> =
        BTreeMap::new();
    for p in &participants {
        let identifier =
            Identifier::try_from(manifest.frost_identifier_of(&p.did).unwrap()).unwrap();
        let own_secret = round1_secrets.remove(&identifier).unwrap();
        let others: BTreeMap<Identifier, round1::Package> = round1_packages
            .iter()
            .filter(|(id, _)| **id != identifier)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();
        let (secret2, outbox) = session::dkg_part2(own_secret, &others).unwrap();
        round2_secrets.insert(identifier, secret2);
        round2_outboxes.insert(identifier, outbox);
    }

    // Phase F: DKG Round 3 (part3) -- each participant collects, from every
    // other participant's outbox, the Round-2 package addressed to them.
    let mut key_packages: BTreeMap<Identifier, KeyPackage> = BTreeMap::new();
    let mut public_key_packages: Vec<PublicKeyPackage> = Vec::new();
    for p in &participants {
        let identifier =
            Identifier::try_from(manifest.frost_identifier_of(&p.did).unwrap()).unwrap();
        let round1_others: BTreeMap<Identifier, round1::Package> = round1_packages
            .iter()
            .filter(|(id, _)| **id != identifier)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();
        let round2_incoming: BTreeMap<Identifier, round2::Package> = round2_outboxes
            .iter()
            .filter(|(sender, _)| **sender != identifier)
            .map(|(sender, outbox)| (*sender, outbox.get(&identifier).unwrap().clone()))
            .collect();
        let (key_package, public_key_package) = session::dkg_part3(
            &round2_secrets[&identifier],
            &round1_others,
            &round2_incoming,
        )
        .unwrap();
        key_packages.insert(identifier, key_package);
        public_key_packages.push(public_key_package);
    }

    // Every participant must land on exactly the same group public key.
    let group_public_key = *public_key_packages[0].verifying_key();
    for pkp in &public_key_packages {
        assert_eq!(pkp.verifying_key(), &group_public_key);
    }

    // Phase G: unanimous completion attestation.
    let public_package_hash =
        session::public_package_hash(&session_id, &public_key_packages[0]).unwrap();
    let group_public_key_bytes: [u8; 32] =
        group_public_key.serialize().unwrap().try_into().unwrap();
    let attestations: Vec<_> = participants
        .iter()
        .map(|p| {
            session::sign_completion_attestation(
                session_id,
                root,
                public_package_hash,
                group_public_key_bytes,
                &p.did,
                &p.device_key,
            )
        })
        .collect();
    assert!(session::completion_confirmed(&manifest, &attestations));

    // The real point: the DKG-produced key packages must be usable to
    // produce a valid FROST threshold signature over the group key, using
    // exactly `THRESHOLD` (7) of the 11 participants -- proving this
    // crate's ceremony wiring produces a working key, not just
    // well-typed plumbing around one.
    let signers: Vec<Identifier> = key_packages.keys().take(7).copied().collect();
    let message = b"mini-custody full-ceremony smoke test message";

    let mut nonces_map = BTreeMap::new();
    let mut commitments_map = BTreeMap::new();
    for id in &signers {
        let key_package = &key_packages[id];
        let (nonces, commitments) = sign_round1::commit(key_package.signing_share(), &mut OsRng);
        nonces_map.insert(*id, nonces);
        commitments_map.insert(*id, commitments);
    }

    let signing_package = SigningPackage::new(commitments_map, message);

    let mut signature_shares = BTreeMap::new();
    for id in &signers {
        let key_package = &key_packages[id];
        let nonces = &nonces_map[id];
        let share = sign_round2::sign(&signing_package, nonces, key_package).unwrap();
        signature_shares.insert(*id, share);
    }

    let group_signature =
        frost_ristretto255::aggregate(&signing_package, &signature_shares, &public_key_packages[0])
            .unwrap();

    group_public_key.verify(message, &group_signature).unwrap();
}
