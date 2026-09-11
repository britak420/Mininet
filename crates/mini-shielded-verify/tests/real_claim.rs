//! Proves `ShieldedClaimVerifier` against a genuine claim — real stealth
//! derivation, a real MLSAG spend proof, a real Bulletproof range proof —
//! not just the decode-failure/no-evidence cases `src/lib.rs`'s own unit
//! tests already cover. This is deliberately a minimal, self-contained
//! fixture rather than importing `mini-private-payment`'s own
//! `tests/support` module: Rust doesn't let one crate's integration tests
//! import another's, and duplicating the handful of lines this needs is
//! cheaper than restructuring that crate's test-only fixtures into a
//! shared library just for this.

use mini_execution::{ClaimVerifier, NullifierRecord};
use mini_private_payment::{
    build, InMemoryOutputSet, OutputSet, PaymentPurpose, PaymentRequest, Recipient, MIN_RING_SIZE,
};
use mini_shielded_verify::{ClaimEvidencePool, ShieldedClaimVerifier};
use mini_value::StealthKeypair;
use std::sync::Arc;

const NETWORK: [u8; 32] = [0x5a; 32];

fn a_real_claim_and_its_nullifier_group() -> (
    Vec<u8>,
    [u8; 32],
    Vec<NullifierRecord>,
    Vec<mini_execution::ShieldedGenesisAllocation>,
) {
    let recipient_keys = StealthKeypair::generate().unwrap();
    let spend_key = StealthKeypair::generate().unwrap();

    let mut outputs = InMemoryOutputSet::new();
    let mut genesis = Vec::new();
    for _ in 0..(MIN_RING_SIZE - 1) {
        let decoy = StealthKeypair::generate().unwrap();
        let blinding = [0; 32];
        let commitment = mini_value::pedersen_commitment(1_000, &blinding).unwrap();
        outputs.push(decoy.spend_public_bytes().to_vec(), commitment);
        genesis.push(mini_execution::ShieldedGenesisAllocation {
            output: mini_execution::ShieldedOutput {
                public_key: decoy.spend_public_bytes().to_vec(),
                amount_commitment: commitment.to_vec(),
            },
            amount_micro: 1_000,
        });
    }
    let spend_blinding = [0; 32];
    let spend_commitment = mini_value::pedersen_commitment(500, &spend_blinding).unwrap();
    outputs.push(spend_key.spend_public_bytes().to_vec(), spend_commitment);
    genesis.push(mini_execution::ShieldedGenesisAllocation {
        output: mini_execution::ShieldedOutput {
            public_key: spend_key.spend_public_bytes().to_vec(),
            amount_commitment: spend_commitment.to_vec(),
        },
        amount_micro: 500,
    });
    let set_index = outputs.len() - 1;

    let request = PaymentRequest {
        network_id: NETWORK,
        spends: vec![mini_private_payment::SpendableOutput {
            set_index,
            one_time_secret: spend_key.spend_secret_bytes(),
            value_micro: 500,
            blinding: spend_blinding,
        }],
        recipients: vec![Recipient {
            spend_public: recipient_keys.spend_public_bytes().to_vec(),
            view_public: recipient_keys.view_public_bytes().to_vec(),
            amount_micro: 500,
            purpose: PaymentPurpose::new(b"test".to_vec()),
        }],
        fee_micro: 0,
        ring_size: MIN_RING_SIZE,
        valid_until_ms: 10_000,
        last_known_chain: b"height:1".to_vec(),
        decoy_entropy: mini_crypto::random_32().unwrap(),
    };
    let (claim, _built_outputs) = build(&request, &outputs).unwrap();

    let digest = claim.transcript_digest();
    let group: Vec<NullifierRecord> = claim
        .inputs
        .iter()
        .map(|input| NullifierRecord::new(input.signature.key_image.clone(), digest))
        .collect();
    (claim.encode(), digest, group, genesis)
}

#[test]
fn a_genuine_claim_is_approved_end_to_end() {
    let (claim_bytes, digest, group, genesis) = a_real_claim_and_its_nullifier_group();

    let evidence = Arc::new(ClaimEvidencePool::new());
    let stored_digest = evidence.insert(claim_bytes).unwrap();
    assert_eq!(stored_digest, digest);

    let verifier = ShieldedClaimVerifier::new(NETWORK, evidence);
    assert!(
        verifier.verify_claim(&NETWORK, &digest, &group).is_some(),
        "a genuinely valid claim, correctly stored and correctly grouped, must verify"
    );
    let previous =
        mini_execution::LedgerState::with_shielded_genesis(NETWORK, genesis, &verifier).unwrap();
    let body = mini_execution::SettlementBlockBody::new(vec![]).with_nullifiers(group.clone());
    assert_eq!(
        mini_execution::apply_block(&previous, &body),
        Err(mini_execution::ExecutionError::MissingClaimVerifier)
    );
    let next =
        mini_execution::apply_block_with_verifier(&previous, &body, Some(&verifier)).unwrap();
    for record in &group {
        assert_eq!(next.finalized_nullifier(&record.key_image), Some(digest));
    }
    let restored =
        mini_execution::LedgerState::from_snapshot_bytes(&next.to_snapshot_bytes().unwrap())
            .unwrap();
    assert_eq!(
        restored.verify_shielded_claims(None),
        Err(mini_execution::ExecutionError::MissingClaimVerifier)
    );
    restored.verify_shielded_claims(Some(&verifier)).unwrap();
    assert_eq!(restored, next);
}

#[test]
fn a_tampered_claim_is_rejected_even_though_it_was_stored() {
    let (mut claim_bytes, _digest, _group, _genesis) = a_real_claim_and_its_nullifier_group();
    // Flip a byte inside the encoded transcript -- decode still succeeds
    // (structural validation only), but the digest this now decodes to no
    // longer matches what the caller thinks it inserted, or the spend
    // proof no longer verifies against it.
    let flip_at = claim_bytes.len() / 2;
    claim_bytes[flip_at] ^= 0xff;

    let evidence = Arc::new(ClaimEvidencePool::new());
    // The pool computes its own key from whatever the tampered bytes
    // actually decode to -- it never trusts a caller-asserted digest.
    let Ok(stored_digest) = evidence.insert(claim_bytes) else {
        // Tampering corrupted the encoding badly enough to fail to decode
        // at all -- also an acceptable outcome; nothing gets stored.
        return;
    };

    let verifier = ShieldedClaimVerifier::new(NETWORK, evidence);
    let group = [NullifierRecord::new(vec![0u8; 32], stored_digest)];
    assert!(
        verifier
            .verify_claim(&NETWORK, &stored_digest, &group)
            .is_none(),
        "a tampered claim must never verify, even under its own resulting digest"
    );
}

#[test]
fn wrong_network_id_is_rejected() {
    let (claim_bytes, digest, group, _genesis) = a_real_claim_and_its_nullifier_group();
    let evidence = Arc::new(ClaimEvidencePool::new());
    evidence.insert(claim_bytes).unwrap();

    let verifier = ShieldedClaimVerifier::new([0xee; 32], evidence);
    assert!(verifier.verify_claim(&NETWORK, &digest, &group).is_none());
}
