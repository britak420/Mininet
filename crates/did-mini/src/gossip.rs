//! KEL head gossip summaries (audit #12 finding F4, invariant M3) — Phase 5
//! of `docs/design/kel-witness-receipts-and-duplicity-gossip.md`'s
//! committed phased plan, building on Phase 1's receipt vocabulary
//! ([`crate::witness`], D-0321), Phase 2's state machine
//! ([`crate::witness_state`], D-0326), and Phase 4's receipt collection
//! protocol ([`crate::witness_protocol`], D-0464).
//!
//! The design doc's own words for this phase: "peers gossip compact
//! `KelHeadSummary`s during ordinary relevant interactions (not a
//! standalone always-on service); disagreements trigger targeted evidence
//! retrieval, not full-log flooding." This module is the first half of
//! that sentence — the summary type and the pure comparison that decides
//! whether two summaries agree, and if not, at what sequence they part
//! ways.
//!
//! ## Why there is no new "fetch evidence" request type in this module
//!
//! The second half of that sentence — "targeted evidence retrieval" — does
//! not need a new protocol message. Phase 4 already shipped exactly the
//! two operations evidence retrieval needs:
//! [`crate::SubmitEventForWitnessingRequest`] carries a whole [`crate::Kel`]
//! and re-verifies it from scratch via
//! [`crate::WitnessJournal::observe_declared`], and
//! [`crate::FetchWitnessReceiptRequest`] returns a single witness's own
//! already-issued receipt. A disagreement or a peer running ahead is
//! resolved the same way any first-contact KEL is: fetch the actual chain
//! (over whatever transport a caller already has — `mini-sync` replication,
//! `mini-forge`'s retrieval, or a direct witness fetch) and run it through
//! [`crate::Kel::verify`] / `observe_declared` / [`crate::assess_kel_assurance`],
//! all of which already exist. Inventing a parallel fetch type here would
//! duplicate machinery this crate already has, the same reasoning Phase 4's
//! own module doc gives for not adding a `FetchWitnessCertificate` op.
//!
//! ## Scope: Phase 5's summary/comparison slice only
//!
//! [`KelHeadSummary`], its canonical encoding, and [`compare_head_summaries`]
//! — pure data and a pure function, no network, no piggybacking wiring onto
//! `mini-sync`/`mini-relay`/`mini-forge`'s existing traffic (that wiring is
//! a separate, later PR for whichever of those crates first carries this
//! message, the same real-transport-adapter split this crate has applied at
//! every prior phase). No witness-rotation handling (Phase 7): a summary's
//! `witness_policy_generation` is carried and compared as an opaque value,
//! never used here to decide whether a rotation is legitimate.

use crate::codec::{Reader, Writer};
use crate::error::{IdentityError, Result};
use crate::kel::Kel;
use crate::limits::MAX_MULTIHASH_BYTES;
use crate::witness::{decode_did, encode_did};
use crate::witness_state::WitnessIdentityState;
use crate::Did;

/// A compact, gossip-sized claim about one identity's current KEL head —
/// deliberately far smaller than the KEL itself, matching the research
/// report's "targeted evidence retrieval, not full-log flooding" design.
/// Carries no signature: a summary is a hint two peers compare, not
/// evidence on its own, exactly like [`crate::witness::WitnessId`]'s own
/// unauthenticated-hint peers already documented for `mini-net::pex`-style
/// discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KelHeadSummary {
    pub identity: Did,
    pub sequence: u64,
    /// Multihash bytes of the head event (matches
    /// `did_mini::event::Event::digest`'s own output shape).
    pub event_digest: Vec<u8>,
    /// The declared witness policy's generation, if this identity has ever
    /// appointed witnesses at all (D-0459) — `None` for an identity that
    /// has not, which is a real and common case, not an error.
    pub witness_policy_generation: Option<u64>,
}

impl KelHeadSummary {
    /// Summarize `kel`'s current head — the exact claim a controller (or
    /// anyone holding its KEL) can honestly gossip about it.
    pub fn summarize(kel: &Kel) -> Result<Self> {
        let event = kel.events().last().ok_or(IdentityError::EmptyKel)?;
        Ok(KelHeadSummary {
            identity: kel.did(),
            sequence: event.sn,
            event_digest: event.digest(),
            witness_policy_generation: kel.declared_witness_policy().map(|p| p.generation),
        })
    }

    /// Summarize what a witness has itself accepted for `state`'s identity
    /// — the exact claim a witness can honestly gossip without exposing
    /// anything beyond what [`crate::witness_protocol::FetchWitnessReceiptRequest`]
    /// would already reveal to anyone who asked.
    pub fn from_witness_state(state: &WitnessIdentityState) -> Self {
        KelHeadSummary {
            identity: state.identity.clone(),
            sequence: state.accepted_sequence,
            event_digest: state.accepted_event_digest.clone(),
            witness_policy_generation: Some(state.witness_policy_generation),
        }
    }

    /// Encode to the canonical wire form.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        encode_did(&mut w, &self.identity);
        w.u64(self.sequence);
        w.bytes(&self.event_digest);
        match self.witness_policy_generation {
            None => w.u8(0),
            Some(gen) => {
                w.u8(1);
                w.u64(gen);
            }
        }
        w.into_bytes()
    }

    /// Decode from [`Self::encode`]'s wire form. Strict: rejects trailing
    /// bytes and a malformed presence tag.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let identity = decode_did(&mut r)?;
        let sequence = r.u64()?;
        let event_digest = r.bytes_limited("event_digest", MAX_MULTIHASH_BYTES)?;
        let witness_policy_generation = match r.u8()? {
            0 => None,
            1 => Some(r.u64()?),
            _ => return Err(IdentityError::BadEvent),
        };
        if !r.finished() {
            return Err(IdentityError::TrailingBytes);
        }
        Ok(KelHeadSummary {
            identity,
            sequence,
            event_digest,
            witness_policy_generation,
        })
    }
}

/// The result of comparing two [`KelHeadSummary`]s for the *same* identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadAgreement {
    /// Same sequence, same event digest — no disagreement to chase.
    Agrees,
    /// Same sequence, different event digest — real evidence that two
    /// different events were produced at the same position. Neither
    /// summary alone proves which (if either) is legitimate; resolving it
    /// means fetching the actual KEL(s) and re-verifying (see this
    /// module's doc for why that does not need a new request type).
    Disagreement { at_sequence: u64 },
    /// `mine` claims a later sequence than `other` — `other` may simply be
    /// behind, or may be looking at a branch `mine` has never seen; the
    /// summaries alone cannot tell which.
    Ahead { by: u64 },
    /// `mine` claims an earlier sequence than `other` — the mirror of
    /// [`HeadAgreement::Ahead`].
    Behind { by: u64 },
}

/// Compare two [`KelHeadSummary`]s. Returns
/// [`IdentityError::MismatchedGossipIdentity`] if they name different
/// identities — comparing across identities is always a caller bug, never
/// a legitimate "disagreement."
pub fn compare_head_summaries(
    mine: &KelHeadSummary,
    other: &KelHeadSummary,
) -> Result<HeadAgreement> {
    if mine.identity != other.identity {
        return Err(IdentityError::MismatchedGossipIdentity);
    }
    Ok(match mine.sequence.cmp(&other.sequence) {
        core::cmp::Ordering::Equal => {
            if mine.event_digest == other.event_digest {
                HeadAgreement::Agrees
            } else {
                HeadAgreement::Disagreement {
                    at_sequence: mine.sequence,
                }
            }
        }
        core::cmp::Ordering::Greater => HeadAgreement::Ahead {
            by: mine.sequence - other.sequence,
        },
        core::cmp::Ordering::Less => HeadAgreement::Behind {
            by: other.sequence - mine.sequence,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Controller;

    fn a_summary_for(kel: &Kel) -> KelHeadSummary {
        KelHeadSummary::summarize(kel).unwrap()
    }

    #[test]
    fn a_summary_round_trips_through_encode_decode() {
        let owner = Controller::incept_single().unwrap();
        let summary = a_summary_for(&owner.kel());
        let bytes = summary.encode();
        assert_eq!(KelHeadSummary::decode(&bytes).unwrap(), summary);
    }

    #[test]
    fn an_identity_with_no_witness_policy_summarizes_generation_as_none() {
        let owner = Controller::incept_single().unwrap();
        let summary = a_summary_for(&owner.kel());
        assert_eq!(summary.witness_policy_generation, None);
    }

    #[test]
    fn an_identity_with_a_witness_policy_summarizes_its_generation() {
        let mut owner = Controller::incept_single().unwrap();
        let witness_root = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![witness_root.did()], 1)
            .unwrap();
        let summary = a_summary_for(&owner.kel());
        assert_eq!(summary.witness_policy_generation, Some(1));
    }

    #[test]
    fn identical_summaries_agree() {
        let owner = Controller::incept_single().unwrap();
        let mine = a_summary_for(&owner.kel());
        let other = mine.clone();
        assert_eq!(
            compare_head_summaries(&mine, &other).unwrap(),
            HeadAgreement::Agrees
        );
    }

    #[test]
    fn same_sequence_different_digest_is_a_disagreement() {
        let owner = Controller::incept_single().unwrap();
        let mine = a_summary_for(&owner.kel());
        let mut other = mine.clone();
        other.event_digest = vec![0xFF; 34];
        assert_eq!(
            compare_head_summaries(&mine, &other).unwrap(),
            HeadAgreement::Disagreement { at_sequence: 0 }
        );
        // Symmetric: swapping which side is "mine" still names the same
        // disputed sequence.
        assert_eq!(
            compare_head_summaries(&other, &mine).unwrap(),
            HeadAgreement::Disagreement { at_sequence: 0 }
        );
    }

    #[test]
    fn a_higher_sequence_is_ahead_by_the_difference() {
        let mut owner = Controller::incept_single().unwrap();
        let behind = a_summary_for(&owner.kel());
        let witness_root = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![witness_root.did()], 1)
            .unwrap();
        owner
            .appoint_witnesses(vec![witness_root.did()], 1)
            .unwrap();
        let ahead = a_summary_for(&owner.kel());
        assert_eq!(ahead.sequence - behind.sequence, 2);
        assert_eq!(
            compare_head_summaries(&ahead, &behind).unwrap(),
            HeadAgreement::Ahead { by: 2 }
        );
        assert_eq!(
            compare_head_summaries(&behind, &ahead).unwrap(),
            HeadAgreement::Behind { by: 2 }
        );
    }

    #[test]
    fn comparing_summaries_for_different_identities_is_rejected() {
        let a = Controller::incept_single().unwrap();
        let b = Controller::incept_single().unwrap();
        let summary_a = a_summary_for(&a.kel());
        let summary_b = a_summary_for(&b.kel());
        assert_eq!(
            compare_head_summaries(&summary_a, &summary_b),
            Err(IdentityError::MismatchedGossipIdentity)
        );
    }

    #[test]
    fn from_witness_state_carries_the_accepted_head_exactly() {
        let mut owner = Controller::incept_single().unwrap();
        let witness_root = Controller::incept_single().unwrap();
        owner
            .appoint_witnesses(vec![witness_root.did()], 1)
            .unwrap();

        let mut journal = crate::WitnessJournal::new();
        let witness_id = crate::WitnessId(witness_root.did());
        let witness_key = mini_crypto::SigningKey::generate().unwrap();
        let outcome = journal
            .observe_declared(&owner.kel(), witness_id, &witness_key, 100)
            .unwrap();
        assert!(matches!(outcome, crate::WitnessObservation::Accepted(_)));

        let state = journal.state_for(&owner.did()).unwrap();
        let from_state = KelHeadSummary::from_witness_state(state);
        let from_kel = a_summary_for(&owner.kel());
        assert_eq!(from_state.identity, from_kel.identity);
        assert_eq!(from_state.sequence, from_kel.sequence);
        assert_eq!(from_state.event_digest, from_kel.event_digest);
        assert_eq!(
            compare_head_summaries(&from_state, &from_kel).unwrap(),
            HeadAgreement::Agrees
        );
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let owner = Controller::incept_single().unwrap();
        let summary = a_summary_for(&owner.kel());
        let mut bytes = summary.encode();
        bytes.push(0xFF);
        assert!(KelHeadSummary::decode(&bytes).is_err());
    }

    #[test]
    fn a_truncated_summary_is_rejected_not_panicked() {
        let owner = Controller::incept_single().unwrap();
        let bytes = a_summary_for(&owner.kel()).encode();
        for cut in 0..bytes.len() {
            assert!(KelHeadSummary::decode(&bytes[..cut]).is_err());
        }
    }

    #[test]
    fn an_unknown_generation_presence_tag_is_rejected() {
        let owner = Controller::incept_single().unwrap();
        let summary = a_summary_for(&owner.kel());
        let mut w = Writer::new();
        encode_did(&mut w, &summary.identity);
        w.u64(summary.sequence);
        w.bytes(&summary.event_digest);
        w.u8(2); // neither 0 (None) nor 1 (Some)
        let bytes = w.into_bytes();
        assert!(KelHeadSummary::decode(&bytes).is_err());
    }
}
