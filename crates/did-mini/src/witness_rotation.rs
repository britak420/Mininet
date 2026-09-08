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
//! [`verify_witness_rotation`] adds §17.3's other half — the "new witness
//! readiness threshold" — as a thin AND-composition, not a second module.
//! A new witness's *ordinary* first `observe`/`observe_declared` receipt for
//! the rotation event already signs under `witness_policy_generation =
//! event.sn` (the *new* generation, since D-0459 derives the policy a
//! witness signs under from the event's own declared policy) — Phase 1's
//! existing machinery already produces exactly the statement §17.3 asks
//! for, with no new signing code. What was missing was checking *both*
//! thresholds hold for the *same* event before trusting a rotation at
//! §17.3's higher assurance level; `verify_witness_rotation` is that check.
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
//! that happens to carry the same generation number. The new-policy half
//! needs even less: [`WitnessedEventCertificate::verify`] against the *new*
//! policy already does everything §17.3 asks, unchanged — no wrapper type
//! for it exists because none is needed.
//!
//! ## Scope: §17.2 and §17.3 only
//!
//! **Not yet built:** §17.4's unavailable-witness recovery path
//! (deliberately harder: it must work *without* the old witnesses'
//! cooperation, the opposite assumption this module makes); and no wiring
//! into `did_mini::assess_kel_assurance` — whether or when a real verifier
//! should *require* either assurance level before trusting a
//! witness-policy rotation remains the same kind of founder-facing policy
//! call earlier phases already left open for their own consuming
//! decisions.

use std::collections::HashSet;

use mini_crypto::{SigningKey, VerifyingKey};

use crate::codec::{Reader, Writer};
use crate::controller::Controller;
use crate::error::{IdentityError, Result};
use crate::event::IndexedSig;
use crate::kel::Kel;
use crate::witness::{
    decode_did, encode_did, sign_witness_receipt, WitnessId, WitnessPolicy, WitnessReceipt,
    WitnessReceiptStatement, WitnessReceiptVersion, WitnessedEventCertificate,
};
use crate::witness_state::WitnessJournal;
use crate::Did;

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

/// Verify a witness-set rotation at research report §17.3's higher
/// assurance level: **both** the retiring policy's threshold (via
/// [`verify_policy_transition`], proving old witnesses authorized their own
/// replacement) **and** the incoming policy's own threshold (via
/// [`WitnessedEventCertificate::verify`] against the *new* [`WitnessPolicy`]
/// `new_kel` declares, proving enough incoming witnesses accepted
/// responsibility) must independently hold for the *same* event.
///
/// `new_policy_certificate` is `None` only when `new_kel`'s head retires the
/// witness policy entirely — there is no new witness set to prove readiness
/// for, so the readiness half is vacuously satisfied. Passing `None` while a
/// real new policy exists is treated as zero readiness receipts, not a
/// missing argument: [`IdentityError::WitnessThresholdNotMet`] names the
/// real threshold against a `got` of zero, the same shape a caller would see
/// from an empty certificate.
///
/// This performs the old-policy check via [`verify_policy_transition`]
/// unchanged, so every one of its own error cases (a stale/unrelated KEL, a
/// non-transition, the old threshold unmet) applies here identically before
/// the new-policy half is ever reached.
pub fn verify_witness_rotation(
    old_policy: &WitnessPolicy,
    new_kel: &Kel,
    old_policy_certificate: &WitnessedEventCertificate,
    new_policy_certificate: Option<&WitnessedEventCertificate>,
    resolve_witness_key: impl Fn(&WitnessId) -> Option<VerifyingKey>,
) -> Result<()> {
    verify_policy_transition(
        old_policy,
        new_kel,
        old_policy_certificate,
        &resolve_witness_key,
    )?;
    let new_policy = match new_kel.declared_witness_policy() {
        None => return Ok(()),
        Some(policy) => policy,
    };
    let certificate = new_policy_certificate.ok_or(IdentityError::WitnessThresholdNotMet {
        needed: new_policy.threshold,
        got: 0,
    })?;
    let event = new_kel.events().last().ok_or(IdentityError::EmptyKel)?;
    if certificate.identity != new_kel.did()
        || certificate.sequence != event.sn
        || certificate.event_digest != event.digest()
    {
        return Err(IdentityError::WitnessReceiptMismatch);
    }
    certificate.verify(&new_policy, resolve_witness_key)
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

// --- §17.4: unavailable-witness recovery (D-0475) ---
//
// "A witness set may become unavailable. The protocol needs a recovery
// path that cannot be triggered casually... No witness set should be
// able to hold an identity permanently hostage." (research report §17.4)
//
// Deliberately the opposite assumption from §17.2/§17.3 above: those
// require the *old* witnesses to cooperate (certify their own
// replacement); this exists for exactly the case they cannot or will
// not. Nothing here can be backed by a third-party signature the way an
// old-witness certificate is, because the whole premise is that no such
// third party is reachable. What raises the cost of triggering this path
// is compounded, caller-configured friction instead: a documented
// waiting period and a minimum count of *distinct* old witnesses shown
// unreachable (a real new-witness-readiness certificate is still
// required whenever a successor policy exists — see
// `verify_dead_witness_recovery`'s own docs). The research report itself
// only lists these as "possible requirements", not settled numbers:
// every threshold here is caller-supplied
// ([`DeadWitnessRecoveryPolicy`]), never a value this module invents —
// the same "open protocol questions, not derived figures" discipline
// `mini_storage_fraud::ReplicaLifecycle` already applies to its own
// window/challenge parameters.

/// Caller-configured friction for [`verify_dead_witness_recovery`] — see
/// this section's own module-level docs for why every field here is a
/// policy choice, not a value this crate derives or defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeadWitnessRecoveryPolicy {
    /// Minimum count of *distinct* old witnesses that must be attested
    /// unreachable — a single missing witness must never be enough to
    /// justify replacing the whole set.
    pub min_unreachable_witnesses: usize,
    /// Minimum epochs that must separate an attestation's own
    /// `first_unreachable_epoch` and `last_attempt_epoch` — the waiting
    /// period. Coarse epochs, the same unit
    /// [`crate::WitnessReceiptStatement::observed_epoch`] already uses
    /// and for the same reason (research report §8.7: an exact timestamp
    /// increases clock dependency and leaks witness timing).
    pub min_waiting_period_epochs: u64,
}

/// The controller's own signed claim that `witness_id` — a member of the
/// witness policy generation named here — has been unreachable from
/// `first_unreachable_epoch` through `last_attempt_epoch`.
///
/// Signed by the **controller**, not a witness: §17.2/§17.3's receipts
/// work because a third party the old policy already trusted signs off;
/// here that third party is exactly what is missing, so nothing can be
/// backed by anyone but the party asking for recovery. This is **not
/// independent proof of unavailability** — a controller willing to lie
/// about its own witnesses can sign this exactly as it could sign
/// anything else in its own KEL. What it buys is accountability, not
/// unforgeability: a false attestation is a durable, attributable,
/// non-repudiable claim under the controller's own signature, verified
/// against whatever keys were actually authoritative during the claimed
/// waiting period (via [`Kel::verify_message_at`]) rather than trusted
/// as a bare unsigned assertion — the same trade
/// [`crate::WitnessReceiptStatement::observed_epoch`]'s own
/// "self-reported, like everywhere else in this tree that lacks a time
/// anchor" precedent already makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessUnavailabilityAttestation {
    pub identity: Did,
    pub witness_id: WitnessId,
    pub witness_policy_generation: u64,
    pub first_unreachable_epoch: u64,
    pub last_attempt_epoch: u64,
}

/// Domain separation for [`WitnessUnavailabilityAttestation::encode`], so
/// this can never collide with an unrelated signed statement elsewhere in
/// the tree (the same discipline every other domain-tagged digest/wire
/// type in this crate already applies).
const UNAVAILABILITY_DOMAIN: &[u8] = b"did-mini/witness-unavailability/v1";

impl WitnessUnavailabilityAttestation {
    /// Encode to the canonical wire form these bytes are signed over.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.bytes(UNAVAILABILITY_DOMAIN);
        encode_did(&mut w, &self.identity);
        encode_did(&mut w, &self.witness_id.0);
        w.u64(self.witness_policy_generation);
        w.u64(self.first_unreachable_epoch);
        w.u64(self.last_attempt_epoch);
        w.into_bytes()
    }

    /// Decode from [`Self::encode`]'s wire form. Strict: rejects a wrong
    /// or missing domain tag, and trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let domain = r.bytes_limited("domain", UNAVAILABILITY_DOMAIN.len())?;
        if domain != UNAVAILABILITY_DOMAIN {
            return Err(IdentityError::BadEvent);
        }
        let identity = decode_did(&mut r)?;
        let witness_id = WitnessId(decode_did(&mut r)?);
        let witness_policy_generation = r.u64()?;
        let first_unreachable_epoch = r.u64()?;
        let last_attempt_epoch = r.u64()?;
        if !r.finished() {
            return Err(IdentityError::TrailingBytes);
        }
        Ok(WitnessUnavailabilityAttestation {
            identity,
            witness_id,
            witness_policy_generation,
            first_unreachable_epoch,
            last_attempt_epoch,
        })
    }

    /// Sign this attestation with `controller`'s current keys — the typed
    /// entry point matching [`Controller::sign_message`]'s own "detached
    /// payload" pattern, so callers never hand-assemble the message bytes
    /// a signature is supposed to cover.
    pub fn sign(&self, controller: &Controller) -> Vec<IndexedSig> {
        controller.sign_message(&self.encode())
    }
}

/// Verify a witness-set rotation via research report §17.4's recovery
/// path: no old-witness cooperation at all, compensated by
/// `policy`-gated friction instead.
///
/// `attestations` must name enough *distinct* old witnesses
/// (`policy.min_unreachable_witnesses`) each with a waiting period of at
/// least `policy.min_waiting_period_epochs`, each verified against
/// `new_kel`'s own key state immediately before the recovery event (the
/// controller's keys at attestation-signing time — durable, historical
/// verification, not "authorized right now").
///
/// Exactly like [`verify_witness_rotation`], `new_policy_certificate` is
/// only required when `new_kel`'s head actually declares a successor
/// witness policy — retirement (choosing to go unwitnessed) needs no
/// readiness proof, because there is no new witness set to prove
/// readiness for. Forbidding retirement here would itself be a way this
/// path could hold an identity hostage — the controller must remain free
/// to choose "no witnesses" as its own exit, not just "different
/// witnesses".
pub fn verify_dead_witness_recovery(
    policy: &DeadWitnessRecoveryPolicy,
    old_policy: &WitnessPolicy,
    new_kel: &Kel,
    attestations: &[(WitnessUnavailabilityAttestation, Vec<IndexedSig>)],
    new_policy_certificate: Option<&WitnessedEventCertificate>,
    resolve_witness_key: impl Fn(&WitnessId) -> Option<VerifyingKey>,
) -> Result<()> {
    new_kel.verify()?;
    let event = new_kel.events().last().ok_or(IdentityError::EmptyKel)?;
    let new_policy = new_kel.declared_witness_policy();
    if !is_policy_change(old_policy, new_policy.as_ref()) {
        return Err(IdentityError::NotAWitnessPolicyChange);
    }

    let attesting_sn = event.sn.saturating_sub(1);
    let mut distinct = HashSet::new();
    for (attestation, sigs) in attestations {
        if attestation.identity != new_kel.did()
            || attestation.witness_policy_generation != old_policy.generation
        {
            return Err(IdentityError::WitnessReceiptMismatch);
        }
        if !old_policy.contains(&attestation.witness_id) {
            return Err(IdentityError::WitnessNotInPolicy);
        }
        let span = attestation
            .last_attempt_epoch
            .checked_sub(attestation.first_unreachable_epoch)
            .ok_or(IdentityError::RecoveryWaitingPeriodNotMet {
                needed_epochs: policy.min_waiting_period_epochs,
                got_epochs: 0,
            })?;
        if span < policy.min_waiting_period_epochs {
            return Err(IdentityError::RecoveryWaitingPeriodNotMet {
                needed_epochs: policy.min_waiting_period_epochs,
                got_epochs: span,
            });
        }
        new_kel.verify_message_at(attesting_sn, &attestation.encode(), sigs)?;
        distinct.insert(attestation.witness_id.clone());
    }
    if distinct.len() < policy.min_unreachable_witnesses {
        return Err(IdentityError::InsufficientUnavailabilityEvidence {
            needed: policy.min_unreachable_witnesses,
            got: distinct.len(),
        });
    }

    let new_policy = match new_policy {
        None => return Ok(()),
        Some(new_policy) => new_policy,
    };
    let certificate = new_policy_certificate.ok_or(IdentityError::WitnessThresholdNotMet {
        needed: new_policy.threshold,
        got: 0,
    })?;
    if certificate.identity != new_kel.did()
        || certificate.sequence != event.sn
        || certificate.event_digest != event.digest()
    {
        return Err(IdentityError::WitnessReceiptMismatch);
    }
    certificate.verify(&new_policy, resolve_witness_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Controller;
    use crate::WitnessObservation;

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

    /// One genuine witness-set swap (old witness `w1` retired, new witness
    /// `new_witness` appointed, threshold 1 on both sides) plus everything a
    /// `verify_witness_rotation` call needs: the old policy, the rotated
    /// KEL, an old-policy certificate signed by `w1`, and the new witness's
    /// own ordinary first receipt for the same event (§17.3's "new witness
    /// readiness" statement, produced by existing Phase 1 machinery -- not
    /// by anything in this module).
    struct Rotation {
        old_policy: WitnessPolicy,
        owner: Controller,
        w1: WitnessId,
        w1_key: SigningKey,
        old_policy_certificate: WitnessedEventCertificate,
        new_witness: WitnessId,
        new_witness_key: SigningKey,
        new_receipt: WitnessReceipt,
    }

    impl Rotation {
        /// Resolves both the retiring witness's and the incoming witness's
        /// keys -- everything `verify_witness_rotation` ever asks this
        /// fixture for.
        fn resolve(&self, id: &WitnessId) -> Option<VerifyingKey> {
            if *id == self.w1 {
                Some(self.w1_key.verifying_key())
            } else if *id == self.new_witness {
                Some(self.new_witness_key.verifying_key())
            } else {
                None
            }
        }

        fn new_policy_certificate(&self) -> WitnessedEventCertificate {
            let new_policy = self.owner.kel().declared_witness_policy().unwrap();
            WitnessedEventCertificate::assemble(
                self.owner.did(),
                self.new_receipt.statement.sequence,
                self.new_receipt.statement.event_digest.clone(),
                new_policy.generation,
                vec![self.new_receipt.clone()],
            )
            .unwrap()
        }
    }

    fn genuine_rotation() -> Rotation {
        let (w1, w1_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, w1.clone(), &w1_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        let (new_witness, new_witness_key) = a_witness();
        owner
            .appoint_witnesses(vec![new_witness.0.clone()], 1)
            .unwrap();

        let old_receipt = journal
            .certify_policy_transition(&owner.kel(), w1.clone(), &w1_key, 200)
            .unwrap();
        let old_policy_certificate = WitnessedEventCertificate::assemble(
            owner.did(),
            old_receipt.statement.sequence,
            old_receipt.statement.event_digest.clone(),
            old_policy.generation,
            vec![old_receipt],
        )
        .unwrap();

        // The new witness's own ordinary first observation of this
        // identity -- no special API, exactly what Phase 1-4 already give
        // any witness seeing an identity for the first time.
        let mut new_witness_journal = WitnessJournal::new();
        let new_receipt = match new_witness_journal
            .observe_declared(&owner.kel(), new_witness.clone(), &new_witness_key, 300)
            .unwrap()
        {
            WitnessObservation::Accepted(receipt) => receipt,
            other => panic!("expected Accepted, got {other:?}"),
        };

        Rotation {
            old_policy,
            owner,
            w1,
            w1_key,
            old_policy_certificate,
            new_witness,
            new_witness_key,
            new_receipt,
        }
    }

    #[test]
    fn verify_witness_rotation_succeeds_when_both_thresholds_are_met() {
        let rotation = genuine_rotation();
        let new_certificate = rotation.new_policy_certificate();

        verify_witness_rotation(
            &rotation.old_policy,
            &rotation.owner.kel(),
            &rotation.old_policy_certificate,
            Some(&new_certificate),
            |id| rotation.resolve(id),
        )
        .unwrap();
    }

    #[test]
    fn verify_witness_rotation_fails_when_new_policy_readiness_is_missing() {
        let rotation = genuine_rotation();

        assert_eq!(
            verify_witness_rotation(
                &rotation.old_policy,
                &rotation.owner.kel(),
                &rotation.old_policy_certificate,
                None,
                |id| rotation.resolve(id),
            ),
            Err(IdentityError::WitnessThresholdNotMet { needed: 1, got: 0 })
        );
    }

    #[test]
    fn verify_witness_rotation_surfaces_an_old_policy_failure_unchanged() {
        let rotation = genuine_rotation();
        let new_certificate = rotation.new_policy_certificate();

        // A KEL that never actually rotated past the old policy: the same
        // failure `verify_policy_transition` already covers on its own,
        // reached through the composed function instead.
        let stale_owner = Controller::incept_single().unwrap();
        assert!(verify_witness_rotation(
            &rotation.old_policy,
            &stale_owner.kel(),
            &rotation.old_policy_certificate,
            Some(&new_certificate),
            |id| rotation.resolve(id),
        )
        .is_err());
    }

    #[test]
    fn verify_witness_rotation_enforces_the_new_policys_own_threshold() {
        let (w1, w1_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, w1.clone(), &w1_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        // A new policy requiring *two* witnesses.
        let (new_w1, new_w1_key) = a_witness();
        let (new_w2, new_w2_key) = a_witness();
        owner
            .appoint_witnesses(vec![new_w1.0.clone(), new_w2.0.clone()], 2)
            .unwrap();

        let old_receipt = journal
            .certify_policy_transition(&owner.kel(), w1, &w1_key, 200)
            .unwrap();
        let old_policy_certificate = WitnessedEventCertificate::assemble(
            owner.did(),
            old_receipt.statement.sequence,
            old_receipt.statement.event_digest.clone(),
            old_policy.generation,
            vec![old_receipt],
        )
        .unwrap();

        // Only one of the two required new witnesses acknowledges.
        let mut new_w1_journal = WitnessJournal::new();
        let new_receipt = match new_w1_journal
            .observe_declared(&owner.kel(), new_w1.clone(), &new_w1_key, 300)
            .unwrap()
        {
            WitnessObservation::Accepted(receipt) => receipt,
            other => panic!("expected Accepted, got {other:?}"),
        };
        let new_policy = owner.kel().declared_witness_policy().unwrap();
        let new_certificate = WitnessedEventCertificate::assemble(
            owner.did(),
            new_receipt.statement.sequence,
            new_receipt.statement.event_digest.clone(),
            new_policy.generation,
            vec![new_receipt],
        )
        .unwrap();

        let resolve = |id: &WitnessId| {
            if *id == new_w1 {
                Some(new_w1_key.verifying_key())
            } else if *id == new_w2 {
                Some(new_w2_key.verifying_key())
            } else {
                None
            }
        };
        assert!(verify_witness_rotation(
            &old_policy,
            &owner.kel(),
            &old_policy_certificate,
            Some(&new_certificate),
            resolve,
        )
        .is_err());
    }

    #[test]
    fn verify_witness_rotation_succeeds_on_retirement_with_no_new_certificate_needed() {
        let (w1, w1_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, w1.clone(), &w1_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();
        owner.retire_witnesses().unwrap();

        let old_receipt = journal
            .certify_policy_transition(&owner.kel(), w1.clone(), &w1_key, 200)
            .unwrap();
        let old_policy_certificate = WitnessedEventCertificate::assemble(
            owner.did(),
            old_receipt.statement.sequence,
            old_receipt.statement.event_digest.clone(),
            old_policy.generation,
            vec![old_receipt],
        )
        .unwrap();

        let resolve = |id: &WitnessId| {
            if *id == w1 {
                Some(w1_key.verifying_key())
            } else {
                None
            }
        };
        verify_witness_rotation(
            &old_policy,
            &owner.kel(),
            &old_policy_certificate,
            None,
            resolve,
        )
        .unwrap();
    }

    #[test]
    fn verify_witness_rotation_rejects_a_new_policy_certificate_over_the_wrong_event() {
        let rotation = genuine_rotation();

        // A certificate that claims to cover this rotation but actually
        // names a different (fabricated) event digest.
        let mut new_certificate = rotation.new_policy_certificate();
        new_certificate.event_digest = vec![0xAA; 32];

        assert!(verify_witness_rotation(
            &rotation.old_policy,
            &rotation.owner.kel(),
            &rotation.old_policy_certificate,
            Some(&new_certificate),
            |id| rotation.resolve(id),
        )
        .is_err());
    }

    // --- §17.4: unavailable-witness recovery ---

    fn lenient_policy() -> DeadWitnessRecoveryPolicy {
        DeadWitnessRecoveryPolicy {
            min_unreachable_witnesses: 1,
            min_waiting_period_epochs: 100,
        }
    }

    fn attempt(
        owner: &Did,
        witness_id: &WitnessId,
        generation: u64,
        first: u64,
        last: u64,
    ) -> WitnessUnavailabilityAttestation {
        WitnessUnavailabilityAttestation {
            identity: owner.clone(),
            witness_id: witness_id.clone(),
            witness_policy_generation: generation,
            first_unreachable_epoch: first,
            last_attempt_epoch: last,
        }
    }

    /// A controller that appointed one old witness, then unilaterally
    /// (no witness cooperation at all) rotates straight to a fresh
    /// witness set -- exactly the shape a real dead-witness recovery
    /// takes: only the controller's own signature moves the KEL.
    struct Recovery {
        old_policy: WitnessPolicy,
        old_witness: WitnessId,
        owner: Controller,
        new_witness: WitnessId,
        new_witness_key: SigningKey,
        new_receipt: WitnessReceipt,
        /// An unavailability attestation for `old_witness`, signed while
        /// the pre-rotation keys were still current -- `attesting_sn`
        /// inside `verify_dead_witness_recovery` checks exactly that key
        /// state, matching the real order of events: evidence accumulates
        /// and gets attested *before* the controller decides to
        /// unilaterally rotate away from it.
        attestation: WitnessUnavailabilityAttestation,
        attestation_sig: Vec<IndexedSig>,
    }

    impl Recovery {
        fn resolve(&self, id: &WitnessId) -> Option<VerifyingKey> {
            if *id == self.new_witness {
                Some(self.new_witness_key.verifying_key())
            } else {
                None
            }
        }

        fn new_policy_certificate(&self) -> WitnessedEventCertificate {
            let new_policy = self.owner.kel().declared_witness_policy().unwrap();
            WitnessedEventCertificate::assemble(
                self.owner.did(),
                self.new_receipt.statement.sequence,
                self.new_receipt.statement.event_digest.clone(),
                new_policy.generation,
                vec![self.new_receipt.clone()],
            )
            .unwrap()
        }
    }

    fn a_dead_witness_recovery() -> Recovery {
        let (old_witness, _old_witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner =
            appointed_and_observed(&mut journal, old_witness.clone(), &_old_witness_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        // Sign *before* rotating -- `owner`'s current keys are still the
        // pre-recovery ones at this point.
        let attestation = attempt(&owner.did(), &old_witness, old_policy.generation, 0, 200);
        let attestation_sig = attestation.sign(&owner);

        let (new_witness, new_witness_key) = a_witness();
        owner
            .appoint_witnesses(vec![new_witness.0.clone()], 1)
            .unwrap();

        let mut new_witness_journal = WitnessJournal::new();
        let new_receipt = match new_witness_journal
            .observe_declared(&owner.kel(), new_witness.clone(), &new_witness_key, 300)
            .unwrap()
        {
            WitnessObservation::Accepted(receipt) => receipt,
            other => panic!("expected Accepted, got {other:?}"),
        };

        Recovery {
            old_policy,
            old_witness,
            owner,
            new_witness,
            new_witness_key,
            new_receipt,
            attestation,
            attestation_sig,
        }
    }

    #[test]
    fn dead_witness_recovery_succeeds_with_sufficient_evidence_and_readiness() {
        let recovery = a_dead_witness_recovery();
        let policy = lenient_policy();
        let event = recovery.owner.kel().events().last().unwrap().clone();
        let attestation = recovery.attestation.clone();
        let sig = recovery.attestation_sig.clone();
        let new_certificate = recovery.new_policy_certificate();

        verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation, sig)],
            Some(&new_certificate),
            |id| recovery.resolve(id),
        )
        .unwrap();
        // Sanity: this really was a rotation event, not an inception.
        assert!(event.sn > 0);
    }

    #[test]
    fn dead_witness_recovery_fails_the_waiting_period_check() {
        let recovery = a_dead_witness_recovery();
        let policy = lenient_policy(); // needs 100 epochs
        let attestation = attempt(
            &recovery.owner.did(),
            &recovery.old_witness,
            recovery.old_policy.generation,
            0,
            50, // only 50 epochs of documented unavailability
        );
        let sig = attestation.sign(&recovery.owner);
        let new_certificate = recovery.new_policy_certificate();

        let err = verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation, sig)],
            Some(&new_certificate),
            |id| recovery.resolve(id),
        )
        .unwrap_err();
        assert_eq!(
            err,
            IdentityError::RecoveryWaitingPeriodNotMet {
                needed_epochs: 100,
                got_epochs: 50,
            }
        );
    }

    #[test]
    fn dead_witness_recovery_fails_when_too_few_distinct_witnesses_are_attested() {
        let recovery = a_dead_witness_recovery();
        let policy = DeadWitnessRecoveryPolicy {
            min_unreachable_witnesses: 2, // more than the one witness this fixture has
            min_waiting_period_epochs: 100,
        };
        let attestation = recovery.attestation.clone();
        let sig = recovery.attestation_sig.clone();
        let new_certificate = recovery.new_policy_certificate();

        let err = verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation, sig)],
            Some(&new_certificate),
            |id| recovery.resolve(id),
        )
        .unwrap_err();
        assert_eq!(
            err,
            IdentityError::InsufficientUnavailabilityEvidence { needed: 2, got: 1 }
        );
    }

    #[test]
    fn dead_witness_recovery_ignores_duplicate_attestations_for_the_same_witness() {
        // Naming the same witness twice must not count as two distinct
        // witnesses -- the whole point of `min_unreachable_witnesses`.
        let recovery = a_dead_witness_recovery();
        let policy = DeadWitnessRecoveryPolicy {
            min_unreachable_witnesses: 2,
            min_waiting_period_epochs: 100,
        };
        let attestation = recovery.attestation.clone();
        let sig = recovery.attestation_sig.clone();
        let new_certificate = recovery.new_policy_certificate();

        let err = verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation.clone(), sig.clone()), (attestation, sig)],
            Some(&new_certificate),
            |id| recovery.resolve(id),
        )
        .unwrap_err();
        assert_eq!(
            err,
            IdentityError::InsufficientUnavailabilityEvidence { needed: 2, got: 1 }
        );
    }

    #[test]
    fn dead_witness_recovery_requires_new_policy_readiness_when_a_successor_policy_exists() {
        let recovery = a_dead_witness_recovery();
        let policy = lenient_policy();
        let attestation = recovery.attestation.clone();
        let sig = recovery.attestation_sig.clone();

        let err = verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation, sig)],
            None, // no readiness certificate at all
            |_| None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            IdentityError::WitnessThresholdNotMet { needed: 1, got: 0 }
        );
    }

    #[test]
    fn dead_witness_recovery_rejects_an_attestation_for_a_witness_outside_the_old_policy() {
        let recovery = a_dead_witness_recovery();
        let policy = lenient_policy();
        let (stranger, _) = a_witness();
        let attestation = attempt(
            &recovery.owner.did(),
            &stranger, // never part of old_policy
            recovery.old_policy.generation,
            0,
            200,
        );
        let sig = attestation.sign(&recovery.owner);
        let new_certificate = recovery.new_policy_certificate();

        let err = verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation, sig)],
            Some(&new_certificate),
            |id| recovery.resolve(id),
        )
        .unwrap_err();
        assert_eq!(err, IdentityError::WitnessNotInPolicy);
    }

    #[test]
    fn dead_witness_recovery_rejects_an_attestation_signed_by_someone_else() {
        // A forged attestation the controller never actually signed --
        // signed instead by an unrelated fresh controller's own keys.
        let recovery = a_dead_witness_recovery();
        let policy = lenient_policy();
        let impostor = Controller::incept_single().unwrap();
        let attestation = attempt(
            &recovery.owner.did(),
            &recovery.old_witness,
            recovery.old_policy.generation,
            0,
            200,
        );
        let forged_sig = attestation.sign(&impostor);
        let new_certificate = recovery.new_policy_certificate();

        assert!(verify_dead_witness_recovery(
            &policy,
            &recovery.old_policy,
            &recovery.owner.kel(),
            &[(attestation, forged_sig)],
            Some(&new_certificate),
            |id| recovery.resolve(id),
        )
        .is_err());
    }

    #[test]
    fn dead_witness_recovery_succeeds_on_retirement_with_no_readiness_certificate_needed() {
        let (old_witness, old_witness_key) = a_witness();
        let mut journal = WitnessJournal::new();
        let mut owner = appointed_and_observed(&mut journal, old_witness.clone(), &old_witness_key);
        let old_policy = journal
            .state_for(&owner.did())
            .unwrap()
            .accepted_policy()
            .clone();

        // Sign *before* retiring -- `owner`'s current keys are still the
        // pre-recovery ones at this point.
        let attestation = attempt(&owner.did(), &old_witness, old_policy.generation, 0, 200);
        let sig = attestation.sign(&owner);
        owner.retire_witnesses().unwrap();
        let policy = lenient_policy();

        verify_dead_witness_recovery(
            &policy,
            &old_policy,
            &owner.kel(),
            &[(attestation, sig)],
            None, // retirement needs no new-witness readiness proof
            |_| None,
        )
        .unwrap();
    }

    #[test]
    fn a_witness_unavailability_attestation_round_trips_through_encode_decode() {
        let owner = Controller::incept_single().unwrap();
        let (witness_id, _) = a_witness();
        let original = attempt(&owner.did(), &witness_id, 3, 10, 500);
        let decoded = WitnessUnavailabilityAttestation::decode(&original.encode()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn a_truncated_attestation_is_rejected_not_partially_parsed() {
        let owner = Controller::incept_single().unwrap();
        let (witness_id, _) = a_witness();
        let full = attempt(&owner.did(), &witness_id, 3, 10, 500).encode();
        for cut in 0..full.len() {
            assert!(
                WitnessUnavailabilityAttestation::decode(&full[..cut]).is_err(),
                "truncating to {cut} bytes must be rejected"
            );
        }
    }
}
