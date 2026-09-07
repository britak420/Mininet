//! Receipt collection protocol (audit #12 finding F4, invariant M3) —
//! Phase 4 of `docs/design/kel-witness-receipts-and-duplicity-gossip.md`'s
//! committed phased plan, building on Phase 1's receipt vocabulary
//! ([`crate::witness`], D-0321), Phase 2's state machine
//! ([`crate::witness_state`], D-0326), and Phase 3's policy-binding fix
//! ([`crate::WitnessJournal::observe_declared`], extending D-0459).
//!
//! Phases 1-3 answered "what does a witness sign, and when." This module
//! answers "what bytes cross the wire to ask it" — the typed request/
//! response messages the design doc named as its own Phase 4 example
//! (`SubmitEventForWitnessing`, `FetchWitnessCertificate`), landed here as
//! pure message shapes and pure handler functions, no socket, matching this
//! crate's established pattern of landing protocol logic before the real-
//! transport adapter that needs one (Phase 1's receipt types before Phase
//! 2's state machine, that state machine before Phase 3's KEL wiring).
//!
//! ## Why there is no `FetchWitnessCertificate` server operation
//!
//! [`crate::WitnessedEventCertificate::assemble`] already exists (Phase 1)
//! and is a pure function over receipts a caller already holds — a
//! multi-witness certificate is something a *requester* builds locally by
//! gathering one [`SubmitEventForWitnessingResponse`] from each witness it
//! asks, not something any single witness can hand back on request (no
//! witness has another witness's receipts without Phase 5's gossip, which
//! does not exist yet). [`FetchWitnessReceiptRequest`] is what a *single*
//! witness can honestly answer: its own already-issued receipt for a given
//! `(identity, sequence)`, so a client that lost its copy — or an aggregator
//! collecting from several witnesses — need not resubmit a whole KEL to get
//! a receipt back.
//!
//! ## The security property this module is built to hold, not just check
//!
//! [`SubmitEventForWitnessingRequest`] carries a [`crate::Kel`] and nothing
//! else — no policy field. That is deliberate, not an omission: a request
//! that *could* carry a policy would reopen exactly the forgery D-0459
//! closed for `assess_kel_assurance`, on the signing side instead of the
//! verifying side — a requester names its own witnesses, claims a policy
//! naming them, and an honest witness signs a receipt for an identity that
//! never appointed it. [`handle_submit_for_witnessing`] calls
//! [`crate::WitnessJournal::observe_declared`], which reads the policy from
//! the submitted KEL's own controller-signed history — the forgery is
//! unrepresentable in this module's request type, not merely rejected by a
//! runtime check.
//!
//! ## Scope: Phase 4 only
//!
//! No network transport, no persistence, no gossip, no rotation, no
//! transparency log (Phases 5-8 remain future work, each its own PR per
//! this crate's established discipline). A real socket adapter carrying
//! these messages — the equivalent of `mini-consensus::discovery` for
//! `mini-net::pex` — is left to whatever crate first needs a running
//! witness service; nothing here assumes one.

use mini_crypto::SigningKey;

use crate::error::{IdentityError, Result};
use crate::kel::Kel;
use crate::witness::{decode_did, encode_did, WitnessId, WitnessReceipt};
use crate::witness_state::{ControllerDuplicityProof, WitnessJournal, WitnessObservation};
use crate::Did;

/// Hard cap on a submitted KEL's encoded size — generous for any real
/// identity's history, far below a size that would let an untrusted
/// request force a large allocation before [`Kel::from_bytes`]'s own
/// per-event/per-count bounds ever run.
const MAX_KEL_WIRE_BYTES: usize = 8 * 1024 * 1024;

/// Ask one witness to observe the head event of `kel` and, if it accepts,
/// issue a receipt. Carries the full KEL rather than just the head event
/// because a witness must independently verify the chain is valid and read
/// the declared witness policy from it — never trust either as a bare
/// claim (see this module's doc for why there is no separate policy
/// field).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitEventForWitnessingRequest {
    pub kel: Kel,
}

impl SubmitEventForWitnessingRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = crate::codec::Writer::new();
        w.bytes(&self.kel.to_bytes());
        w.into_bytes()
    }

    /// Decode from [`Self::encode`]'s wire form. Purely structural — it
    /// validates the framing and delegates to [`Kel::from_bytes`], which
    /// does **not** itself verify the chain. A decoded request is exactly
    /// as untrusted as one that arrived any other way; only
    /// [`handle_submit_for_witnessing`] decides whether it counts.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = crate::codec::Reader::new(bytes);
        let kel_bytes = r.bytes_limited("kel", MAX_KEL_WIRE_BYTES)?;
        let kel = Kel::from_bytes(&kel_bytes)?;
        if !r.finished() {
            return Err(IdentityError::TrailingBytes);
        }
        Ok(SubmitEventForWitnessingRequest { kel })
    }
}

/// Why a witness refused to act on a [`SubmitEventForWitnessingRequest`] —
/// a bounded, typed reason rather than an arbitrary message string, so a
/// caller can react to it without parsing prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// The submitted KEL did not verify (bad signatures, broken chain,
    /// pre-rotation mismatch, or it was empty).
    ChainInvalid,
    /// The submitted KEL declares no witnesses at all — there is no
    /// authentic policy to check membership against.
    NoWitnessPolicyDeclared,
    /// The KEL declares a real policy, but this witness is not a member
    /// of it.
    WitnessNotInPolicy,
    /// The submitted event neither matches, extends, precedes, nor
    /// conflicts-at-the-same-sequence with this witness's accepted state —
    /// see [`crate::witness_state`]'s "conflicting descendant" limit.
    ConflictingDescendant { sequence: u64 },
}

impl RejectionReason {
    fn tag(self) -> u8 {
        match self {
            RejectionReason::ChainInvalid => 1,
            RejectionReason::NoWitnessPolicyDeclared => 2,
            RejectionReason::WitnessNotInPolicy => 3,
            RejectionReason::ConflictingDescendant { .. } => 4,
        }
    }
}

/// One witness's answer to a [`SubmitEventForWitnessingRequest`]. Every
/// variant is a legitimate, typed protocol outcome — including a rejection
/// — not a Rust-level exception; matching this workspace's convention of
/// carrying an unhappy path as a payload variant (e.g.
/// `mini_consensus::StateSyncPayload::Unavailable`) rather than an `Err`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitEventForWitnessingResponse {
    /// First-seen acceptance or a valid direct successor: a fresh receipt.
    Accepted(WitnessReceipt),
    /// Exact duplicate of an already-accepted event: the same receipt this
    /// witness issued before, never re-signed.
    AlreadyAccepted(WitnessReceipt),
    /// Older than this witness's accepted state. Carries only the accepted
    /// sequence — deliberately nothing more, per the research report's own
    /// caution against revealing excess history to an unauthenticated
    /// requester.
    Stale { accepted_sequence: u64 },
    /// A real, independently-verifiable proof that the requester's
    /// controller signed two different events at one sequence. Boxed for
    /// the same reason [`WitnessObservation::ControllerDuplicity`] is.
    ControllerDuplicity(Box<ControllerDuplicityProof>),
    /// The witness declined to act on the request at all.
    Rejected(RejectionReason),
}

impl SubmitEventForWitnessingResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = crate::codec::Writer::new();
        match self {
            SubmitEventForWitnessingResponse::Accepted(receipt) => {
                w.u8(1);
                w.bytes(&receipt.encode());
            }
            SubmitEventForWitnessingResponse::AlreadyAccepted(receipt) => {
                w.u8(2);
                w.bytes(&receipt.encode());
            }
            SubmitEventForWitnessingResponse::Stale { accepted_sequence } => {
                w.u8(3);
                w.u64(*accepted_sequence);
            }
            SubmitEventForWitnessingResponse::ControllerDuplicity(proof) => {
                w.u8(4);
                w.bytes(&proof.encode());
            }
            SubmitEventForWitnessingResponse::Rejected(reason) => {
                w.u8(5);
                w.u8(reason.tag());
                if let RejectionReason::ConflictingDescendant { sequence } = reason {
                    w.u64(*sequence);
                }
            }
        }
        w.into_bytes()
    }

    /// Decode from [`Self::encode`]'s wire form. Strict: rejects an unknown
    /// tag and trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = crate::codec::Reader::new(bytes);
        let response = match r.u8()? {
            1 => {
                let receipt_bytes = r.bytes_limited("receipt", MAX_RECEIPT_BYTES)?;
                SubmitEventForWitnessingResponse::Accepted(WitnessReceipt::decode(&receipt_bytes)?)
            }
            2 => {
                let receipt_bytes = r.bytes_limited("receipt", MAX_RECEIPT_BYTES)?;
                SubmitEventForWitnessingResponse::AlreadyAccepted(WitnessReceipt::decode(
                    &receipt_bytes,
                )?)
            }
            3 => SubmitEventForWitnessingResponse::Stale {
                accepted_sequence: r.u64()?,
            },
            4 => {
                let proof_bytes = r.bytes_limited("duplicity_proof", MAX_DUPLICITY_PROOF_BYTES)?;
                SubmitEventForWitnessingResponse::ControllerDuplicity(Box::new(
                    ControllerDuplicityProof::decode(&proof_bytes)?,
                ))
            }
            5 => {
                let reason = match r.u8()? {
                    1 => RejectionReason::ChainInvalid,
                    2 => RejectionReason::NoWitnessPolicyDeclared,
                    3 => RejectionReason::WitnessNotInPolicy,
                    4 => RejectionReason::ConflictingDescendant { sequence: r.u64()? },
                    _ => return Err(IdentityError::BadEvent),
                };
                SubmitEventForWitnessingResponse::Rejected(reason)
            }
            _ => return Err(IdentityError::BadEvent),
        };
        if !r.finished() {
            return Err(IdentityError::TrailingBytes);
        }
        Ok(response)
    }
}

/// Generous ceiling on one encoded receipt — far above any real suite's
/// signature size, purely an allocation bound on untrusted input.
const MAX_RECEIPT_BYTES: usize = 8 * 1024;
/// Generous ceiling on one encoded duplicity proof (two full events).
const MAX_DUPLICITY_PROOF_BYTES: usize = 256 * 1024;

/// Handle one [`SubmitEventForWitnessingRequest`] against `journal`, acting
/// as witness `witness_id` and signing with `witness_key`. Never trusts the
/// request's KEL is valid or that its declared policy names this witness —
/// both are checked here, via [`WitnessJournal::observe_declared`], before
/// anything is signed.
pub fn handle_submit_for_witnessing(
    journal: &mut WitnessJournal,
    request: &SubmitEventForWitnessingRequest,
    witness_id: WitnessId,
    witness_key: &SigningKey,
    observed_epoch: u64,
) -> SubmitEventForWitnessingResponse {
    match journal.observe_declared(&request.kel, witness_id, witness_key, observed_epoch) {
        Ok(WitnessObservation::Accepted(receipt)) => {
            SubmitEventForWitnessingResponse::Accepted(receipt)
        }
        Ok(WitnessObservation::AlreadyAccepted(receipt)) => {
            SubmitEventForWitnessingResponse::AlreadyAccepted(receipt)
        }
        Ok(WitnessObservation::Stale { accepted_sequence }) => {
            SubmitEventForWitnessingResponse::Stale { accepted_sequence }
        }
        Ok(WitnessObservation::ControllerDuplicity(proof)) => {
            SubmitEventForWitnessingResponse::ControllerDuplicity(proof)
        }
        Err(IdentityError::NoWitnessPolicyDeclared) => {
            SubmitEventForWitnessingResponse::Rejected(RejectionReason::NoWitnessPolicyDeclared)
        }
        Err(IdentityError::WitnessNotInPolicy) => {
            SubmitEventForWitnessingResponse::Rejected(RejectionReason::WitnessNotInPolicy)
        }
        Err(IdentityError::WitnessConflictingDescendant { sequence }) => {
            SubmitEventForWitnessingResponse::Rejected(RejectionReason::ConflictingDescendant {
                sequence,
            })
        }
        Err(_) => SubmitEventForWitnessingResponse::Rejected(RejectionReason::ChainInvalid),
    }
}

/// Ask one witness to resend the receipt it already issued for
/// `(identity, sequence)` — no KEL resubmission needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchWitnessReceiptRequest {
    pub identity: Did,
    pub sequence: u64,
}

impl FetchWitnessReceiptRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = crate::codec::Writer::new();
        encode_did(&mut w, &self.identity);
        w.u64(self.sequence);
        w.into_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = crate::codec::Reader::new(bytes);
        let identity = decode_did(&mut r)?;
        let sequence = r.u64()?;
        if !r.finished() {
            return Err(IdentityError::TrailingBytes);
        }
        Ok(FetchWitnessReceiptRequest { identity, sequence })
    }
}

/// One witness's answer to a [`FetchWitnessReceiptRequest`] — read-only and
/// side-effect-free: it can only return a receipt this witness already
/// signed, never manufacture one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchWitnessReceiptResponse {
    Found(WitnessReceipt),
    NotFound,
}

impl FetchWitnessReceiptResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = crate::codec::Writer::new();
        match self {
            FetchWitnessReceiptResponse::Found(receipt) => {
                w.u8(1);
                w.bytes(&receipt.encode());
            }
            FetchWitnessReceiptResponse::NotFound => w.u8(2),
        }
        w.into_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = crate::codec::Reader::new(bytes);
        let response = match r.u8()? {
            1 => {
                let receipt_bytes = r.bytes_limited("receipt", MAX_RECEIPT_BYTES)?;
                FetchWitnessReceiptResponse::Found(WitnessReceipt::decode(&receipt_bytes)?)
            }
            2 => FetchWitnessReceiptResponse::NotFound,
            _ => return Err(IdentityError::BadEvent),
        };
        if !r.finished() {
            return Err(IdentityError::TrailingBytes);
        }
        Ok(response)
    }
}

/// Handle one [`FetchWitnessReceiptRequest`] against `journal`. Returns
/// exactly what this witness already committed to — nothing computed,
/// nothing re-signed.
pub fn handle_fetch_witness_receipt(
    journal: &WitnessJournal,
    request: &FetchWitnessReceiptRequest,
) -> FetchWitnessReceiptResponse {
    match journal.state_for(&request.identity) {
        Some(state) if state.accepted_sequence == request.sequence => {
            FetchWitnessReceiptResponse::Found(state.issued_receipt().clone())
        }
        _ => FetchWitnessReceiptResponse::NotFound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Controller;

    fn a_witness() -> (Did, SigningKey) {
        let witness_root = Controller::incept_single().unwrap();
        (witness_root.did(), SigningKey::generate().unwrap())
    }

    fn appointed_identity() -> (Controller, WitnessId, SigningKey, u64) {
        let mut owner = Controller::incept_single().unwrap();
        let (witness_did, witness_key) = a_witness();
        let witness_id = WitnessId(witness_did);
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();
        (owner, witness_id, witness_key, 100)
    }

    #[test]
    fn a_request_round_trips_through_encode_decode() {
        let (owner, ..) = appointed_identity();
        let request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let bytes = request.encode();
        assert_eq!(
            SubmitEventForWitnessingRequest::decode(&bytes).unwrap(),
            request
        );
    }

    #[test]
    fn an_appointed_witness_is_accepted_and_the_response_round_trips() {
        let (owner, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let response =
            handle_submit_for_witnessing(&mut journal, &request, witness_id, &witness_key, epoch);
        let SubmitEventForWitnessingResponse::Accepted(receipt) = &response else {
            panic!("expected Accepted, got {response:?}");
        };
        receipt.verify(&witness_key.verifying_key()).unwrap();

        let bytes = response.encode();
        assert_eq!(
            SubmitEventForWitnessingResponse::decode(&bytes).unwrap(),
            response
        );
    }

    #[test]
    fn resubmitting_the_same_event_returns_the_same_receipt_not_a_new_one() {
        let (owner, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let first = handle_submit_for_witnessing(
            &mut journal,
            &request,
            witness_id.clone(),
            &witness_key,
            epoch,
        );
        let second = handle_submit_for_witnessing(
            &mut journal,
            &request,
            witness_id,
            &witness_key,
            epoch + 1,
        );
        let SubmitEventForWitnessingResponse::Accepted(first_receipt) = first else {
            panic!("expected Accepted");
        };
        let SubmitEventForWitnessingResponse::AlreadyAccepted(second_receipt) = second else {
            panic!("expected AlreadyAccepted");
        };
        assert_eq!(first_receipt, second_receipt, "never re-signed");
    }

    #[test]
    fn an_identity_that_appointed_no_witnesses_is_rejected_not_silently_signed() {
        let owner = Controller::incept_single().unwrap();
        let (_, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let response =
            handle_submit_for_witnessing(&mut journal, &request, witness_id, &witness_key, epoch);
        assert_eq!(
            response,
            SubmitEventForWitnessingResponse::Rejected(RejectionReason::NoWitnessPolicyDeclared)
        );
    }

    #[test]
    fn a_witness_not_named_in_the_declared_policy_is_rejected() {
        let (owner, ..) = appointed_identity();
        // A completely different witness, never appointed by `owner`.
        let (stranger_did, stranger_key) = a_witness();
        let stranger_id = WitnessId(stranger_did);
        let mut journal = WitnessJournal::new();
        let request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let response =
            handle_submit_for_witnessing(&mut journal, &request, stranger_id, &stranger_key, 100);
        assert_eq!(
            response,
            SubmitEventForWitnessingResponse::Rejected(RejectionReason::WitnessNotInPolicy)
        );
    }

    #[test]
    fn a_request_cannot_smuggle_a_favorable_policy_because_there_is_no_policy_field() {
        // The regression this module exists to make structurally
        // impossible, not just rejected at runtime: SubmitEventForWitnessingRequest
        // has exactly one field, `kel`. There is no way for a caller --
        // honest or attacking -- to hand a witness a policy naming itself;
        // the only source of truth is `kel.declared_witness_policy()`,
        // read after `Kel::verify` inside `observe_declared`. This test
        // exists so a future refactor that added a policy field back would
        // fail obviously (the struct literal below would need a second
        // field, which every construction site in this file would then
        // have to supply explicitly).
        let (owner, ..) = appointed_identity();
        let SubmitEventForWitnessingRequest { kel } = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        assert_eq!(kel, owner.kel().clone());
    }

    #[test]
    fn a_stale_event_is_reported_without_a_receipt() {
        let (mut owner, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let first_request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        handle_submit_for_witnessing(
            &mut journal,
            &first_request,
            witness_id.clone(),
            &witness_key,
            epoch,
        );

        // Rotate forward, then present the *old* (now-stale) KEL snapshot
        // captured before the rotation.
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();
        let stale_request = first_request;
        let advanced_request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        handle_submit_for_witnessing(
            &mut journal,
            &advanced_request,
            witness_id.clone(),
            &witness_key,
            epoch + 1,
        );

        let response = handle_submit_for_witnessing(
            &mut journal,
            &stale_request,
            witness_id,
            &witness_key,
            epoch + 2,
        );
        assert_eq!(
            response,
            SubmitEventForWitnessingResponse::Stale {
                accepted_sequence: 2
            }
        );
    }

    #[test]
    fn fetch_returns_the_already_issued_receipt_without_resigning() {
        let (owner, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let submit_request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let submitted = handle_submit_for_witnessing(
            &mut journal,
            &submit_request,
            witness_id,
            &witness_key,
            epoch,
        );
        let SubmitEventForWitnessingResponse::Accepted(issued) = submitted else {
            panic!("expected Accepted");
        };

        let fetch_request = FetchWitnessReceiptRequest {
            identity: owner.did(),
            sequence: 1,
        };
        let fetched = handle_fetch_witness_receipt(&journal, &fetch_request);
        assert_eq!(fetched, FetchWitnessReceiptResponse::Found(issued));

        let bytes = fetch_request.encode();
        assert_eq!(
            FetchWitnessReceiptRequest::decode(&bytes).unwrap(),
            fetch_request
        );
    }

    #[test]
    fn fetch_for_an_unknown_identity_or_sequence_returns_not_found() {
        let (owner, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let submit_request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        handle_submit_for_witnessing(
            &mut journal,
            &submit_request,
            witness_id,
            &witness_key,
            epoch,
        );

        let wrong_sequence = FetchWitnessReceiptRequest {
            identity: owner.did(),
            sequence: 99,
        };
        assert_eq!(
            handle_fetch_witness_receipt(&journal, &wrong_sequence),
            FetchWitnessReceiptResponse::NotFound
        );

        let unknown_identity = Controller::incept_single().unwrap();
        let wrong_identity = FetchWitnessReceiptRequest {
            identity: unknown_identity.did(),
            sequence: 0,
        };
        assert_eq!(
            handle_fetch_witness_receipt(&journal, &wrong_identity),
            FetchWitnessReceiptResponse::NotFound
        );
    }

    #[test]
    fn a_truncated_submit_request_is_rejected_not_panicked() {
        let (owner, ..) = appointed_identity();
        let bytes = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        }
        .encode();
        for cut in 0..bytes.len() {
            assert!(SubmitEventForWitnessingRequest::decode(&bytes[..cut]).is_err());
        }
    }

    #[test]
    fn trailing_garbage_after_a_valid_response_is_rejected() {
        let (owner, witness_id, witness_key, epoch) = appointed_identity();
        let mut journal = WitnessJournal::new();
        let request = SubmitEventForWitnessingRequest {
            kel: owner.kel().clone(),
        };
        let response =
            handle_submit_for_witnessing(&mut journal, &request, witness_id, &witness_key, epoch);
        let mut bytes = response.encode();
        bytes.push(0xff);
        assert!(SubmitEventForWitnessingResponse::decode(&bytes).is_err());
    }

    #[test]
    fn an_unknown_response_tag_is_rejected() {
        assert!(SubmitEventForWitnessingResponse::decode(&[0xee]).is_err());
        assert!(FetchWitnessReceiptResponse::decode(&[0xee]).is_err());
    }
}
