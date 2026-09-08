//! The chain half of the shielded settlement path (roadmap R5).
//!
//! What is under test is **ordering**, not cryptography — the chain
//! deliberately cannot see a private claim's contents, so it cannot check
//! that one produced the key image it is finalizing. See
//! `mini_execution::nullifier`'s module docs for why that boundary exists
//! and what it leaves open.
//!
//! ## The seam these tests pin
//!
//! `mini-private-payment` cannot be linked from here: it reaches
//! `mini-value`, this crate reaches `mini-chain`, and a dependency edge
//! between them would be the first value-to-voice path in the tree (P1,
//! Directive 16). So the two halves are pinned from both sides against the
//! same literal bytes. The map this file's
//! `the_finalized_map_is_the_one_the_shielded_side_expects` produces is
//! asserted, key image for key image, by
//! `a_chain_shaped_ledger_resolves_a_shielded_conflict` in
//! `mini-private-payment/tests/chain_backed.rs`. If either side changes
//! what it means by "finalized", one of the two fails.

use mini_execution::{
    apply_block_with_verifier, ClaimVerifier, LedgerState, NullifierRecord, SettlementBlockBody,
    MAX_KEY_IMAGE_BYTES, MAX_NULLIFIERS_PER_BLOCK,
};

const CLAIM_A: [u8; 32] = [0xa1; 32];
const CLAIM_B: [u8; 32] = [0xb2; 32];
const CLAIM_C: [u8; 32] = [0xc3; 32];

// Explicit test-only ordering fixture. Production has no permissive default.
fn apply_block(
    state: &LedgerState,
    body: &SettlementBlockBody,
) -> mini_execution::Result<LedgerState> {
    apply_block_with_verifier(
        state,
        body,
        Some(&AllowListVerifier {
            allowed: vec![CLAIM_A, CLAIM_B, CLAIM_C],
        }),
    )
}

/// Key images are 32 bytes of Ristretto point, opaque here.
fn image(tag: u8) -> Vec<u8> {
    vec![tag; 32]
}

fn body(records: Vec<NullifierRecord>) -> SettlementBlockBody {
    SettlementBlockBody::new(Vec::new()).with_nullifiers(records)
}

#[test]
fn a_shielded_spend_is_finalized_and_readable_by_its_key_image() {
    let state = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![NullifierRecord::new(image(1), CLAIM_A)]),
    )
    .unwrap();

    assert_eq!(state.finalized_nullifier(&image(1)), Some(CLAIM_A));
    assert_eq!(state.finalized_nullifier(&image(2)), None);
    assert_eq!(state.nullifier_count(), 1);
}

#[test]
fn the_first_claim_to_take_a_key_image_keeps_it_permanently() {
    // M1/M3, on the shielded side: body order decides, and the loser is
    // dropped rather than merged, netted, or preferred for being later.
    let state = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(1), CLAIM_B),
        ]),
    )
    .unwrap();

    assert_eq!(state.finalized_nullifier(&image(1)), Some(CLAIM_A));
    assert_eq!(state.nullifier_count(), 1, "the loser added nothing");

    // ...and across blocks, not only within one.
    let later = apply_block(&state, &body(vec![NullifierRecord::new(image(1), CLAIM_C)])).unwrap();
    assert_eq!(later.finalized_nullifier(&image(1)), Some(CLAIM_A));
}

#[test]
fn re_including_a_claim_already_finalized_changes_nothing() {
    // Networks re-deliver. A duplicate is not a double-spend, and treating
    // it as one would make ordinary gossip look like fraud.
    let first = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(2), CLAIM_A),
        ]),
    )
    .unwrap();
    let again = apply_block(
        &first,
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(2), CLAIM_A),
        ]),
    )
    .unwrap();

    assert_eq!(first.commitment(), again.commitment());
}

#[test]
fn a_claim_that_overlaps_on_one_input_takes_none_of_them() {
    // The bug this grouping exists to prevent. Claim A spends {1, 2}; claim
    // B spends {3, 2}. They collide on image 2 only, and not at the same
    // position. Applied record-by-record, B would take image 3 and lose
    // image 2 -- half a double-spend, finalized as a success, with output 3
    // burned by a claim no verifier accepts.
    //
    // That is the merge M1 forbids, arriving through partial application.
    let state = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(2), CLAIM_A),
            NullifierRecord::new(image(3), CLAIM_B),
            NullifierRecord::new(image(2), CLAIM_B),
        ]),
    )
    .unwrap();

    assert_eq!(state.finalized_nullifier(&image(1)), Some(CLAIM_A));
    assert_eq!(state.finalized_nullifier(&image(2)), Some(CLAIM_A));
    assert_eq!(
        state.finalized_nullifier(&image(3)),
        None,
        "B lost image 2, so B must not have taken image 3 either"
    );
    assert_eq!(state.nullifier_count(), 2);
}

#[test]
fn the_same_holds_when_the_collision_arrives_in_a_later_block() {
    let first = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![NullifierRecord::new(image(2), CLAIM_A)]),
    )
    .unwrap();
    let second = apply_block(
        &first,
        &body(vec![
            NullifierRecord::new(image(3), CLAIM_B),
            NullifierRecord::new(image(2), CLAIM_B),
        ]),
    )
    .unwrap();

    assert_eq!(second.finalized_nullifier(&image(3)), None);
    assert_eq!(second.nullifier_count(), 1);
}

#[test]
fn a_malformed_record_rejects_the_entire_body_even_before_a_later_good_record() {
    for broken in [Vec::new(), vec![9u8; MAX_KEY_IMAGE_BYTES + 1]] {
        for records in [
            vec![
                NullifierRecord::new(image(1), CLAIM_A),
                NullifierRecord::new(broken.clone(), CLAIM_A),
            ],
            vec![
                NullifierRecord::new(broken, CLAIM_A),
                NullifierRecord::new(image(1), CLAIM_A),
            ],
        ] {
            let state = shielded_genesis().state().clone();
            assert_eq!(
                apply_block(&state, &body(records)),
                Err(mini_execution::ExecutionError::InvalidShieldedClaim)
            );
            assert_eq!(state.nullifier_count(), 0);
        }
    }
}

#[test]
fn a_rejected_body_leaves_its_key_images_free_for_a_later_claim() {
    let first = shielded_genesis().state().clone();
    assert!(apply_block(
        &first,
        &body(vec![NullifierRecord::new(Vec::new(), CLAIM_A)])
    )
    .is_err());
    let second = apply_block(&first, &body(vec![NullifierRecord::new(image(1), CLAIM_B)])).unwrap();
    assert_eq!(second.finalized_nullifier(&image(1)), Some(CLAIM_B));
}

#[test]
fn shielded_spends_change_the_state_commitment() {
    // Otherwise a block header's state_root would not commit to them, and a
    // node could serve a state that had quietly forgotten a spend.
    let empty = shielded_genesis().state().clone();
    let with_spend =
        apply_block(&empty, &body(vec![NullifierRecord::new(image(1), CLAIM_A)])).unwrap();
    assert_ne!(empty.commitment(), with_spend.commitment());

    // And the commitment is over content, not insertion order: two states
    // reaching the same set of spends by different routes agree.
    let forward = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(2), CLAIM_B),
        ]),
    )
    .unwrap();
    let reversed = apply_block(
        &apply_block(
            &shielded_genesis().state().clone(),
            &body(vec![NullifierRecord::new(image(2), CLAIM_B)]),
        )
        .unwrap(),
        &body(vec![NullifierRecord::new(image(1), CLAIM_A)]),
    )
    .unwrap();
    assert_ne!(
        forward.commitment(),
        reversed.commitment(),
        "canonical replay order is now committed alongside membership"
    );
}

#[test]
fn the_body_hash_covers_the_shielded_records() {
    let without = SettlementBlockBody::new(Vec::new());
    let with = body(vec![NullifierRecord::new(image(1), CLAIM_A)]);
    assert_ne!(without.hash(), with.hash());

    // Reordering the records is a different body: order is what decides
    // conflicts, so it must not be free to permute.
    let one_way = body(vec![
        NullifierRecord::new(image(1), CLAIM_A),
        NullifierRecord::new(image(1), CLAIM_B),
    ]);
    let other_way = body(vec![
        NullifierRecord::new(image(1), CLAIM_B),
        NullifierRecord::new(image(1), CLAIM_A),
    ]);
    assert_ne!(one_way.hash(), other_way.hash());
}

#[test]
fn an_oversized_shielded_list_is_refused_before_anything_is_applied() {
    let too_many: Vec<_> = (0..MAX_NULLIFIERS_PER_BLOCK + 1)
        .map(|i| NullifierRecord::new(vec![(i % 251) as u8; 32], CLAIM_A))
        .collect();
    assert!(apply_block(&shielded_genesis().state().clone(), &body(too_many)).is_err());
}

#[test]
fn a_snapshot_round_trip_preserves_every_shielded_spend() {
    // A restored state that forgot its nullifiers would treat every output
    // it had already finalized as unspent -- a replay of every private
    // payment the chain had ever seen, arriving through state sync.
    let state = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(2), CLAIM_A),
            NullifierRecord::new(image(3), CLAIM_B),
        ]),
    )
    .unwrap();

    let bytes = state.to_snapshot_bytes().unwrap();
    let restored = LedgerState::from_snapshot_bytes(&bytes).unwrap();

    assert_eq!(restored, state);
    assert_eq!(restored.commitment(), state.commitment());
    assert_eq!(restored.finalized_nullifier(&image(1)), Some(CLAIM_A));
    assert_eq!(restored.finalized_nullifier(&image(3)), Some(CLAIM_B));
}

#[test]
fn the_finalized_map_is_the_one_the_shielded_side_expects() {
    // The seam, pinned from this side. The literal pairs below are asserted
    // again -- as the map a shielded wallet reconciles against -- by
    // `a_chain_shaped_ledger_resolves_a_shielded_conflict` in
    // mini-private-payment/tests/chain_backed.rs. The two crates cannot be
    // linked (P1), so this pair of tests is what keeps them honest about
    // each other.
    //
    // Two claims, one shared input: A spends {0x11, 0x22}, B spends
    // {0x33, 0x22}. A is first in body order, so A takes both of its
    // inputs and B takes nothing.
    let state = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(vec![0x11; 32], CLAIM_A),
            NullifierRecord::new(vec![0x22; 32], CLAIM_A),
            NullifierRecord::new(vec![0x33; 32], CLAIM_B),
            NullifierRecord::new(vec![0x22; 32], CLAIM_B),
        ]),
    )
    .unwrap();

    assert_eq!(state.finalized_nullifier(&[0x11; 32]), Some(CLAIM_A));
    assert_eq!(state.finalized_nullifier(&[0x22; 32]), Some(CLAIM_A));
    assert_eq!(state.finalized_nullifier(&[0x33; 32]), None);
    assert_eq!(state.nullifier_count(), 2);
}

// --- ClaimVerifier gating (D-0474, roadmap R8) ---
//
// This crate cannot link `mini-private-payment` (P1) even in tests, so
// these use a hand-rolled `ClaimVerifier` that decides purely from
// `NullifierRecord`'s own opaque fields -- exactly the same surface a
// real implementation (`mini-shielded-verify`) sees, just without any
// real cryptography behind the decision. What's under test is the gating
// mechanism in `apply_nullifiers`, not any particular verifier's logic.

/// Approves every group whose digest is in an explicit allow-list,
/// rejects everything else -- including a digest it was never asked
/// about, which is the honest "no evidence, no trust" default a real
/// verifier must also have.
struct AllowListVerifier {
    allowed: Vec<[u8; 32]>,
}

impl ClaimVerifier for AllowListVerifier {
    fn verify_claim(
        &self,
        _network: &[u8; 32],
        digest: &[u8; 32],
        _group: &[NullifierRecord],
    ) -> Option<mini_execution::ShieldedClaimEffects> {
        self.allowed.contains(digest).then(|| test_effects(digest))
    }
    fn verify_genesis_allocation(
        &self,
        _network: &[u8; 32],
        _allocation: &mini_execution::ShieldedGenesisAllocation,
    ) -> bool {
        true
    }
}

#[test]
fn an_unverified_group_rejects_the_body() {
    let verifier = AllowListVerifier { allowed: vec![] };
    assert_eq!(
        apply_block_with_verifier(
            &shielded_genesis().state().clone(),
            &body(vec![NullifierRecord::new(image(1), CLAIM_A)]),
            Some(&verifier)
        ),
        Err(mini_execution::ExecutionError::InvalidShieldedClaim)
    );
}

#[test]
fn a_verified_group_finalizes_exactly_as_without_a_verifier() {
    let verifier = AllowListVerifier {
        allowed: vec![CLAIM_A],
    };
    let state = apply_block_with_verifier(
        &shielded_genesis().state().clone(),
        &body(vec![
            NullifierRecord::new(image(1), CLAIM_A),
            NullifierRecord::new(image(2), CLAIM_A),
        ]),
        Some(&verifier),
    )
    .unwrap();

    assert_eq!(state.finalized_nullifier(&image(1)), Some(CLAIM_A));
    assert_eq!(state.finalized_nullifier(&image(2)), Some(CLAIM_A));
    assert_eq!(state.nullifier_count(), 2);
}

#[test]
fn every_default_execution_entry_rejects_shielded_spends_without_a_verifier() {
    let state = shielded_genesis().state().clone();
    let b = body(vec![NullifierRecord::new(image(1), CLAIM_A)]);
    assert_eq!(
        mini_execution::apply_block(&state, &b),
        Err(mini_execution::ExecutionError::MissingClaimVerifier)
    );
    assert_eq!(
        apply_block_with_verifier(&state, &b, None),
        Err(mini_execution::ExecutionError::MissingClaimVerifier)
    );
    assert_eq!(state.nullifier_count(), 0);
}

#[test]
fn one_unverified_sibling_rejects_a_whole_body_atomically() {
    let verifier = AllowListVerifier {
        allowed: vec![CLAIM_A],
    };
    let state = shielded_genesis().state().clone();
    assert_eq!(
        apply_block_with_verifier(
            &state,
            &body(vec![
                NullifierRecord::new(image(1), CLAIM_A),
                NullifierRecord::new(image(2), CLAIM_B)
            ]),
            Some(&verifier)
        ),
        Err(mini_execution::ExecutionError::InvalidShieldedClaim)
    );
    assert_eq!(state.nullifier_count(), 0);
}

#[test]
fn a_verified_conflicting_group_cannot_steal_a_finalized_key_image() {
    // The key-image-freedom check still runs first: a claim group must be
    // takeable (M1) before verification is asked to weigh in at all, so a
    // verifier can never be used to "steal" an already-held key image just
    // because it approves the claim.
    struct ApprovesEverything;
    impl ClaimVerifier for ApprovesEverything {
        fn verify_claim(
            &self,
            _network: &[u8; 32],
            digest: &[u8; 32],
            _group: &[NullifierRecord],
        ) -> Option<mini_execution::ShieldedClaimEffects> {
            Some(test_effects(digest))
        }
        fn verify_genesis_allocation(
            &self,
            _network: &[u8; 32],
            _allocation: &mini_execution::ShieldedGenesisAllocation,
        ) -> bool {
            true
        }
    }
    let after_a = apply_block(
        &shielded_genesis().state().clone(),
        &body(vec![NullifierRecord::new(image(1), CLAIM_A)]),
    )
    .unwrap();

    let after_b = apply_block_with_verifier(
        &after_a,
        &body(vec![NullifierRecord::new(image(1), CLAIM_B)]),
        Some(&ApprovesEverything),
    )
    .unwrap();

    // Still A's, unchanged -- CLAIM_B never got a chance to be verified.
    assert_eq!(after_b.finalized_nullifier(&image(1)), Some(CLAIM_A));
}

#[test]
fn copying_a_real_key_image_under_a_forged_digest_never_takes_it_or_blocks_the_real_claim() {
    // PR #327's F-07: "A proposer sees a pending valid payment's key
    // image and includes that image under a different digest first.
    // Honest execution can then consume the conflict key without a
    // valid corresponding payment." A verifier that only has evidence
    // for the real digest (CLAIM_REAL) -- the honest "no evidence, no
    // trust" shape `mini_shielded_verify::ShieldedClaimVerifier` also
    // has, since it looks evidence up strictly by digest -- must refuse
    // the forged group entirely, leaving the real key image free for
    // the real claim to take whenever its own evidence arrives.
    const CLAIM_REAL: [u8; 32] = [0x51; 32];
    const CLAIM_FORGED: [u8; 32] = [0x5f; 32];
    let stolen_key_image = image(9);

    let verifier = AllowListVerifier {
        allowed: vec![CLAIM_REAL],
    };

    // The attacker's forged claim reaches the chain first, copying the
    // real payment's own key image under a digest it has no evidence
    // for.
    let after_attack = shielded_genesis().state().clone();
    assert_eq!(
        apply_block_with_verifier(
            &after_attack,
            &body(vec![NullifierRecord::new(
                stolen_key_image.clone(),
                CLAIM_FORGED
            )]),
            Some(&verifier),
        ),
        Err(mini_execution::ExecutionError::InvalidShieldedClaim)
    );
    assert_eq!(after_attack.finalized_nullifier(&stolen_key_image), None);

    // The real claim, presented later (a later position in the same
    // block or a later block -- this proves the later-block case, the
    // stronger claim), still finds the key image free and finalizes
    // normally.
    let after_real = apply_block_with_verifier(
        &after_attack,
        &body(vec![NullifierRecord::new(
            stolen_key_image.clone(),
            CLAIM_REAL,
        )]),
        Some(&verifier),
    )
    .unwrap();
    assert_eq!(
        after_real.finalized_nullifier(&stolen_key_image),
        Some(CLAIM_REAL),
        "the real claim must still be able to take its own key image afterward"
    );
    assert_eq!(after_real.nullifier_count(), 1);
}

fn test_effects(digest: &[u8; 32]) -> mini_execution::ShieldedClaimEffects {
    mini_execution::ShieldedClaimEffects {
        ring_members: vec![mini_execution::ShieldedOutput {
            public_key: vec![7; 32],
            amount_commitment: vec![8; 32],
        }],
        outputs: vec![mini_execution::ShieldedOutput {
            public_key: digest.to_vec(),
            amount_commitment: vec![8; 32],
        }],
        fee_micro: 0,
    }
}
fn shielded_genesis() -> mini_execution::LedgerChain {
    mini_execution::LedgerChain::genesis_with_shielded_allocations(
        mini_settlement::MININET_NETWORK_ID,
        vec![mini_execution::ShieldedGenesisAllocation {
            output: mini_execution::ShieldedOutput {
                public_key: vec![7; 32],
                amount_commitment: vec![8; 32],
            },
            amount_micro: 1_000,
        }],
        &AllowListVerifier { allowed: vec![] },
    )
    .unwrap()
}
