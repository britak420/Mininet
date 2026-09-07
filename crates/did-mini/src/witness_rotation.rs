//! Old-policy authorization for a witness-set-changing establishment event
//! (audit #12 finding F4, invariant M3) — Phase 7's first slice of
//! `docs/design/kel-witness-receipts-and-duplicity-gossip.md`'s committed
//! phased plan, implementing the founder-supplied research report's §17.2:
//! "The witness-policy-changing event should require certification under
//! the old active policy. Otherwise, a compromised controller could remove
//! honest witnesses before presenting a fork."
//!
//! ## What this closes
//!
//! Before this module, `Controller::appoint_witnesses`/`retire_witnesses`
//! could unilaterally replace an identity's entire witness set with a
//! single self-signed rotation event. Ordinary KEL chain validity
//! (`Kel::verify`) says nothing about witnesses at all — nothing required
//! the *old* witnesses to have ever seen, let alone agreed to, their own
//! removal. A compromised controller could therefore drop every honest
//! witness and switch to attacker-controlled ones in one unwitnessed
//! rotation, then safely equivocate: no witness the old policy trusted
//! would ever be asked to notice.
//!
//! [`WitnessJournal::certify_policy_transition`] is the witness-side fix: a
//! witness that already holds accepted state for an identity (from an
//! earlier `observe`/`observe_declared` call) can certify that a specific,
//! chain-valid, direct-successor establishment event legitimately changes
//! that identity's witness policy — signing under its own *old* retained
//! policy generation, using the exact same [`WitnessReceiptStatement`]/
//! [`WitnessReceipt`] wire types Phase 1 already defined.
//! [`verify_policy_transition`] is the reader side: enough such receipts,
//! bundled the same way as any other [`WitnessedEventCertificate`] (Phase
//! 1's existing `assemble`), checked against the *old* [`WitnessPolicy`],
//! is the "old witness threshold" half of §17.2/§17.3.
//!
//! ## Why no new receipt or certificate type
//!
//! A certification is not a new kind of statement — it is an ordinary
//! [`WitnessReceiptStatement`] whose `witness_policy_generation` names the
//! policy the signer is *retiring from*, over the event that retires it.
//! Reusing the exact Phase 1 types means [`WitnessedEventCertificate::
//! verify`] already does the threshold/membership/signature checking this
//! needs, unchanged — [`verify_policy_transition`] only adds the one check
//! that function cannot: confirming the certificate is actually about a
//! *real* policy transition, not a certificate over an ordinary rotation
//! that happens to carry the same generation number.
//!
//! ## Scope: old-policy authorization only
//!
//! This slice implements §17.2 only. **Not yet built:** §17.3's "new
//! witness readiness threshold" (receipts from the *new* witnesses proving
//! they accepted responsibility — structurally identical to this module
//! once built, just signed under the new generation instead of the old
//! one); §17.4's unavailable-witness recovery path (deliberately harder:
//! it must work *without* the old witnesses' cooperation, the opposite
//! assumption this module makes); and no wiring into `did_mini::
//! assess_kel_assurance` — whether or when a real verifier should
//! *require* this certification before trusting a witness-policy rotation
//! remains the same kind of founder-facing policy call earlier phases
//! already left open for their own consuming decisions.

use mini_crypto::{SigningKey, VerifyingKey};

use crate::error::{IdentityError, Result};
use crate::kel::Kel;
use crate::witness::{
    sign_witness_receipt, WitnessId, WitnessPolicy, WitnessReceipt, WitnessReceiptStatement,
    WitnessReceiptVersion, WitnessedEventCertificate,
};
use crate::witness_state::WitnessJournal;

impl WitnessJournal {
    /// Certify, as an *old* witness, that `kel`'s head event is a valid,
    /// direct-successor establishment event that legitimately changes the
    /// witness policy this witness last accepted for `kel`'s identity.
    ///
    /// Requires this journal to already hold accepted state for the
    /// identity (from an earlier `observe`/`observe_declared`) — a witness
    /// cannot certify a transition away from a policy it was never
    /// actually part of. The signed statement's `witness_policy_generation`
    /// names the *retiring* generation, not whatever policy `kel` itself
    /// now declares, so the resulting receipt can only ever verify against
    /// the *old* [`WitnessPolicy`], via [`WitnessedEventCertificate::verify`]
    /// unchanged.
    ///
    /// Never mutates this journal's own retained state: certifying a
    /// transition is a distinct act from accepting the new head as this
    /// witness's own ongoing state — a witness the new policy drops
    /// entirely still gets to certify its own removal.
    ///
    /// Errors: [`IdentityError::NoRetainedWitnessState`] if this witness
    /// never observed the identity before; whatever [`Kel::verify`] returns
    /// if `kel` is not internally valid; [`IdentityError::
    /// WitnessConflictingDescendant`] if `kel`'s head is not a direct
    /// successor of what this witness last accepted;
    /// [`IdentityError::WitnessNotInPolicy`] if `witness_id` was not
    /// actually a member of the *old* policy; [`IdentityError::
    /// NotAWitnessPolicyChange`] if the head event does not actually
    /// change the witness set or threshold.
    pub fn certify_policy_transition(
        &self,
        kel: &Kel,
        witness_id: WitnessId,
        witness_key: &SigningKey,
        observed_epoch: u64,
    ) -> Result<WitnessReceipt> {
        kel.verify()?;
        let identity = kel.did();
        let old_state = self
            .state_for(&identity)
            .ok_or(IdentityError::NoRetainedWitnessState)?;
        let event = kel.events().last().ok_or(IdentityError::EmptyKel)?;
        if event.sn != old_state.accepted_sequence + 1
            || event.prior != old_state.accepted_event_digest
        {
            return Err(IdentityError::WitnessConflictingDescendant { sequence: event.sn });
        }
        let old_policy = old_state.accepted_policy();
        if !old_policy.contains(&witness_id) {
            return Err(IdentityError::WitnessNotInPolicy);
        }
        let new_policy = kel.declared_witness_policy();
        if !is_policy_change(old_policy, new_policy.as_ref()) {
            return Err(IdentityError::NotAWitnessPolicyChange);
        }
        let event_digest = event.digest();
        let statement = WitnessReceiptStatement {
            version: WitnessReceiptVersion::V1,
            identity,
            sequence: event.sn,
            event_digest,
            prior_event_digest: if event.prior.is_empty() {
                None
            } else {
                Some(event.prior.clone())
            },
            event_kind: (&event.kind).into(),
            witness_policy_generation: old_policy.generation,
            witness_id,
            observed_epoch,
        };
        Ok(sign_witness_receipt(statement, witness_key))
    }
}

/// Verify that `certificate` proves a threshold of `old_policy`'s own
/// witnesses certified `new_kel`'s head event as a legitimate transition
/// away from that policy (research report §17.2).
///
/// Independently confirms the transition is real — `new_kel`'s declared
/// policy genuinely differs from `old_policy` — rather than trusting the
/// certificate's mere existence, and that the certificate is actually about
/// `new_kel`'s own head event (identity, sequence, digest all matched),
/// before delegating threshold/membership/signature checking to
/// [`WitnessedEventCertificate::verify`] unchanged.
pub fn verify_policy_transition(
    old_policy: &WitnessPolicy,
    new_kel: &Kel,
    certificate: &WitnessedEventCertificate,
    resolve_witness_key: impl Fn(&WitnessId) -> Option<VerifyingKey>,
) -> Result<()> {
    new_kel.verify()?;
    let event = new_kel.events().last().ok_or(IdentityError::EmptyKel)?;
    let new_policy = new_kel.declared_witness_policy();
    if !is_policy_change(old_policy, new_policy.as_ref()) {
        return Err(IdentityError::NotAWitnessPolicyChange);
    }
    if certificate.identity != new_kel.did()
        || certificate.sequence != event.sn
        || certificate.event_digest != event.digest()
    {
        return Err(IdentityError::WitnessReceiptMismatch);
    }
    certificate.verify(old_policy, resolve_witness_key)
}

/// Whether `new` (the policy `new_kel`'s head event itself declares, if
/// any) is a real change from `old` — different threshold, or a different
/// witness *set* (membership, not list order: re-declaring the same
/// witnesses in a different sequence is not a policy change).
fn is_policy_change(old: &WitnessPolicy, new: Option<&WitnessPolicy>) -> bool {
    match new {
        None => true,
        Some(new) => {
            old.threshold != new.threshold || !same_witness_set(&old.witnesses, &new.witnesses)
        }
    }
}

fn same_witness_set(a: &[WitnessId], b: &[WitnessId]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut a_sorted: Vec<&WitnessId> = a.iter().collect();
    let mut b_sorted: Vec<&WitnessId> = b.iter().collect();
    a_sorted.sort_by(|x, y| x.0.as_str().cmp(y.0.as_str()));
    b_sorted.sort_by(|x, y| x.0.as_str().cmp(y.0.as_str()));
    a_sorted == b_sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Controller;

    fn a_witness() -> (WitnessId, SigningKey) {
        let root = Controller::incept_single().unwrap();
        (WitnessId(root.did()), SigningKey::generate().unwrap())
    }

    /// A controller with an initial witness policy already accepted by
    /// `witness_id`/`witness_key` in `journal`.
    fn appointed_and_observed(
        journal: &mut WitnessJournal,
        witness_id: WitnessId,
        witness_key: &SigningKey,
    ) -> Controller {
        let mut owner = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();
        journal
            .observe_declared(&owner.kel(), witness_id, witness_key, 100)
            .unwrap();
        owner
    }

    #[test]
    fn certifying_a_genuine_witness_set_change_produces_a_valid_old_policy_receipt() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        let (new_witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![new_witness_id.0], 1).unwrap();

        let receipt = journal
            .certify_policy_transition(&owner.kel(), witness_id.clone(), &witness_key, 200)
            .unwrap();
        assert_eq!(
            receipt.statement.witness_policy_generation,
            old_policy.generation
        );
        receipt.verify(&witness_key.verifying_key()).unwrap();

        let resolve = |id: &WitnessId| {
            if *id == witness_id {
                Some(witness_key.verifying_key())
            } else {
                None
            }
        };
        let cert = WitnessedEventCertificate::assemble(
            owner.did(),
            receipt.statement.sequence,
            receipt.statement.event_digest.clone(),
            old_policy.generation,
            vec![receipt],
        )
        .unwrap();
        verify_policy_transition(&old_policy, &owner.kel(), &cert, resolve).unwrap();
    }

    #[test]
    fn certifying_a_transition_never_mutates_the_journals_own_state() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);
        let before = journal.state_for(&owner.did()).unwrap().clone();

        let (new_witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![new_witness_id.0], 1).unwrap();
        journal
            .certify_policy_transition(&owner.kel(), witness_id, &witness_key, 200)
            .unwrap();

        let after = journal.state_for(&owner.did()).unwrap().clone();
        assert_eq!(before, after, "certifying must not change accepted state");
    }

    #[test]
    fn certification_is_deterministic_across_calls() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);
        let (new_witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![new_witness_id.0], 1).unwrap();

        let first = journal
            .certify_policy_transition(&owner.kel(), witness_id.clone(), &witness_key, 200)
            .unwrap();
        let second = journal
            .certify_policy_transition(&owner.kel(), witness_id, &witness_key, 200)
            .unwrap();
        assert_eq!(first, second, "Ed25519 signing is deterministic");
    }

    #[test]
    fn an_identity_never_observed_before_cannot_be_certified() {
        let (witness_id, witness_key) = a_witness();
        let journal = WitnessJournal::new();
        let mut owner = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();
        assert_eq!(
            journal.certify_policy_transition(&owner.kel(), witness_id, &witness_key, 100),
            Err(IdentityError::NoRetainedWitnessState)
        );
    }

    #[test]
    fn a_gapped_successor_is_rejected() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);

        // Two rotations without re-observing the intermediate one: the
        // witness's retained state is one sequence behind, so this is not
        // a direct successor from the witness's point of view.
        let (mid_witness, _) = a_witness();
        owner.appoint_witnesses(vec![mid_witness.0], 1).unwrap();
        let (final_witness, _) = a_witness();
        owner.appoint_witnesses(vec![final_witness.0], 1).unwrap();

        let sequence = owner.kel().events().last().unwrap().sn;
        assert_eq!(
            journal.certify_policy_transition(&owner.kel(), witness_id, &witness_key, 100),
            Err(IdentityError::WitnessConflictingDescendant { sequence })
        );
    }

    #[test]
    fn a_witness_outside_the_old_policy_cannot_certify() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id, &witness_key);

        let (stranger_id, stranger_key) = a_witness();
        let (new_witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![new_witness_id.0], 1).unwrap();

        assert_eq!(
            journal.certify_policy_transition(&owner.kel(), stranger_id, &stranger_key, 100),
            Err(IdentityError::WitnessNotInPolicy)
        );
    }

    #[test]
    fn an_ordinary_non_policy_rotation_cannot_be_certified() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);

        // Re-appoint the *same* witness at the *same* threshold: not a
        // policy change, even though it is a real rotation event.
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();

        assert_eq!(
            journal.certify_policy_transition(&owner.kel(), witness_id, &witness_key, 100),
            Err(IdentityError::NotAWitnessPolicyChange)
        );
    }

    #[test]
    fn reordering_the_same_witness_set_is_not_a_policy_change() {
        let (w1, w1_key) = a_witness();
        let (w2, _) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![w1.0.clone(), w2.0.clone()], 1)
            .unwrap();
        journal
            .observe_declared(&owner.kel(), w1.clone(), &w1_key, 100)
            .unwrap();

        // Same two witnesses, opposite order, same threshold.
        owner
            .appoint_witnesses(vec![w2.0.clone(), w1.0.clone()], 1)
            .unwrap();

        assert_eq!(
            journal.certify_policy_transition(&owner.kel(), w1, &w1_key, 100),
            Err(IdentityError::NotAWitnessPolicyChange)
        );
    }

    #[test]
    fn a_threshold_only_change_is_a_policy_change() {
        let (w1, w1_key) = a_witness();
        let (w2, _) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![w1.0.clone(), w2.0.clone()], 1)
            .unwrap();
        journal
            .observe_declared(&owner.kel(), w1.clone(), &w1_key, 100)
            .unwrap();

        // Same witness set, higher threshold.
        owner
            .appoint_witnesses(vec![w1.0.clone(), w2.0.clone()], 2)
            .unwrap();

        journal
            .certify_policy_transition(&owner.kel(), w1, &w1_key, 100)
            .unwrap();
    }

    #[test]
    fn retiring_the_witness_policy_entirely_is_a_policy_change() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);
        owner.retire_witnesses().unwrap();

        journal
            .certify_policy_transition(&owner.kel(), witness_id, &witness_key, 100)
            .unwrap();
    }

    #[test]
    fn verify_policy_transition_rejects_a_certificate_over_a_non_transition() {
        let (witness_id, witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, witness_id.clone(), &witness_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        // A real transition, receipted correctly...
        let (new_witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![new_witness_id.0], 1).unwrap();
        let receipt = journal
            .certify_policy_transition(&owner.kel(), witness_id.clone(), &witness_key, 200)
            .unwrap();
        let cert = WitnessedEventCertificate::assemble(
            owner.did(),
            receipt.statement.sequence,
            receipt.statement.event_digest.clone(),
            old_policy.generation,
            vec![receipt],
        )
        .unwrap();

        // ...but checked against a KEL that never actually rotated past
        // the old policy (a stale/unrelated snapshot): the certificate's
        // claimed sequence/digest cannot match this KEL's real head.
        let stale_owner = Controller::incept_single().unwrap();
        let resolve = |id: &WitnessId| {
            if *id == witness_id {
                Some(witness_key.verifying_key())
            } else {
                None
            }
        };
        assert!(verify_policy_transition(&old_policy, &stale_owner.kel(), &cert, resolve).is_err());
    }

    #[test]
    fn verify_policy_transition_enforces_the_old_policys_threshold() {
        let (w1, w1_key) = a_witness();
        let (w2, w2_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![w1.0.clone(), w2.0.clone()], 2)
            .unwrap();
        journal
            .observe_declared(&owner.kel(), w1.clone(), &w1_key, 100)
            .unwrap();
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        let (new_witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![new_witness_id.0], 1).unwrap();

        // Only one of the two required old witnesses certifies.
        let receipt = journal
            .certify_policy_transition(&owner.kel(), w1.clone(), &w1_key, 200)
            .unwrap();
        let cert = WitnessedEventCertificate::assemble(
            owner.did(),
            receipt.statement.sequence,
            receipt.statement.event_digest.clone(),
            old_policy.generation,
            vec![receipt],
        )
        .unwrap();
        let resolve = |id: &WitnessId| {
            if *id == w1 {
                Some(w1_key.verifying_key())
            } else if *id == w2 {
                Some(w2_key.verifying_key())
            } else {
                None
            }
        };
        assert!(verify_policy_transition(&old_policy, &owner.kel(), &cert, resolve).is_err());
    }
}
