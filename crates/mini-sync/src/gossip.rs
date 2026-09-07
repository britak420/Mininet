//! KEL head gossip summaries carried as ordinary sync objects — the "real
//! transport wiring" half of design doc Phase 5
//! (`docs/design/kel-witness-receipts-and-duplicity-gossip.md`), continuing
//! `did_mini::gossip`'s pure summary/comparison types (D-0466). The design
//! doc's own words for this phase: "peers gossip compact `KelHeadSummary`s
//! during ordinary relevant interactions (not a standalone always-on
//! service); disagreements trigger targeted evidence retrieval, not
//! full-log flooding." This module is that piggyback: a
//! [`did_mini::KelHeadSummary`] wrapped as an ordinary object rides the
//! exact traffic this crate already carries.
//!
//! ## Why this needs no new wire protocol
//!
//! [`gossip_summary_carrier`] wraps a summary the same way
//! [`crate::ingest::kel_carrier`] wraps a KEL — as an ordinary
//! [`Object`] — so it replicates through the unmodified MINI/SYNC1
//! reconciliation protocol
//! ([`crate::protocol::sync_bidirectional`]/[`crate::protocol::serve_pull`])
//! with zero new messages. Unlike a KEL carrier, a gossip summary is not
//! self-certifying (nothing about a compact claim proves itself), so it is
//! never given a special [`crate::Ingest::check`] branch — it flows through
//! the exact same ordinary-object provenance path every other authored
//! object already uses: unknown author, rejected; known author, checked
//! against their own cached KEL. No new ingest rule, no new trust
//! boundary.
//!
//! ## What a carrier proves, and what it does not
//!
//! Passing [`crate::Ingest::check`] proves who is *relaying* the claim —
//! their device's signature, checked against their own already-cached KEL
//! — which is accountability against spam, not a claim about the
//! *subject* identity's real head. `summary.identity` need not be (and
//! usually is not) the relaying peer's own identity: relaying what you
//! have observed about someone else is the entire point of gossip.
//! [`did_mini::compare_head_summaries`] is still the only thing that can
//! say whether a summary agrees with what a receiver already believes, and
//! a `Disagreement`/`Ahead` outcome is only ever a reason to fetch and
//! independently verify the real KEL — never something this module treats
//! as settled on its own.
//!
//! ## Scope: piggyback carrier + comparison only
//!
//! [`gossip_summary_carrier`] builds the object; [`compare_gossip_carrier`]
//! decodes an already-ingested one and compares it against a receiver's
//! own [`KelCache`] — the same locally-held "what do I currently believe"
//! state [`crate::Ingest`] already maintains, so no separate witness-state
//! dependency is needed for this comparison to be useful. Turning a
//! `Disagreement`/`Ahead` outcome into an automatic
//! [`crate::request_retrieval`] call is a host policy choice, deliberately
//! left to the caller — the same adapter-not-policy split this workspace
//! already applies to `mini_consensus::discovery::pex_over_tcp` (never
//! auto-wired into `TcpMesh::establish`).

use did_mini::{compare_head_summaries, Controller, Did, HeadAgreement, KelHeadSummary};
use mini_objects::{Object, ObjectBuilder, ObjectError, ObjectType, Payload};

use crate::ingest::KelCache;

/// The custom object type that carries a [`KelHeadSummary`]: payload = its
/// canonical bytes.
pub const GOSSIP_SUMMARY_CARRIER: &str = "mini/kel-gossip-summary";

/// Generous ceiling on one encoded summary carrier — far above any real
/// summary's size (one DID, a sequence number, one digest, an optional
/// generation), purely an allocation bound on untrusted input.
pub const MAX_GOSSIP_SUMMARY_CARRIER_BYTES: usize = 4096;

/// Wrap `summary` as a carrier object, signed by `device` acting for
/// `human` — the *relaying* peer's own identity, not necessarily
/// `summary.identity`.
pub fn gossip_summary_carrier(
    summary: &KelHeadSummary,
    human: &Did,
    device: &Controller,
) -> core::result::Result<Object, ObjectError> {
    ObjectBuilder::new(ObjectType::Custom(GOSSIP_SUMMARY_CARRIER.to_string()))
        .payload(Payload::Public(summary.encode()))
        .sign(human, device)
}

/// What comparing an ingested gossip-summary carrier against a [`KelCache`]
/// produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GossipCarrierOutcome {
    /// Not a gossip-summary carrier at all.
    NotACarrier,
    /// The payload did not decode as a well-formed [`KelHeadSummary`], or
    /// exceeded [`MAX_GOSSIP_SUMMARY_CARRIER_BYTES`].
    Malformed,
    /// The receiver has no cached KEL for the summary's subject identity —
    /// nothing to compare against yet. Not itself an error: most gossip
    /// received is about identities a peer has never met.
    NoLocalKnowledge,
    /// A real comparison against the receiver's own cached KEL for the
    /// same identity.
    Compared(HeadAgreement),
}

/// Decode `obj` as a [`GOSSIP_SUMMARY_CARRIER`] and compare its claim
/// against `cache`'s own record for the same identity, if any.
///
/// `obj` is assumed to have already passed [`crate::Ingest::check`] (or an
/// equivalent provenance check) — this function only *interprets* an
/// already-trusted-as-relayed carrier; it does not itself verify
/// authorship.
pub fn compare_gossip_carrier(cache: &KelCache, obj: &Object) -> GossipCarrierOutcome {
    if obj.object_type != ObjectType::Custom(GOSSIP_SUMMARY_CARRIER.to_string()) {
        return GossipCarrierOutcome::NotACarrier;
    }
    let bytes = match &obj.payload {
        Payload::Public(b) if b.len() <= MAX_GOSSIP_SUMMARY_CARRIER_BYTES => b,
        Payload::Public(_) | Payload::Encrypted(_) => return GossipCarrierOutcome::Malformed,
    };
    let summary = match KelHeadSummary::decode(bytes) {
        Ok(s) => s,
        Err(_) => return GossipCarrierOutcome::Malformed,
    };
    let Some(known) = cache.get(&summary.identity) else {
        return GossipCarrierOutcome::NoLocalKnowledge;
    };
    let ours = match KelHeadSummary::summarize(known) {
        Ok(s) => s,
        Err(_) => return GossipCarrierOutcome::Malformed,
    };
    match compare_head_summaries(&ours, &summary) {
        Ok(agreement) => GossipCarrierOutcome::Compared(agreement),
        // Unreachable in practice -- `ours` is built from `summary.identity`
        // itself, so the two always name the same identity -- but handled
        // rather than unwrapped, since a future refactor of this function
        // must not be able to turn a real mismatch into a panic.
        Err(_) => GossipCarrierOutcome::Malformed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_mini::{Capabilities, Controller as DidController};

    fn a_relayer() -> (Did, DidController) {
        let device = DidController::incept_single().unwrap();
        (device.did(), device)
    }

    fn a_summary_for(kel: &did_mini::Kel) -> KelHeadSummary {
        KelHeadSummary::summarize(kel).unwrap()
    }

    #[test]
    fn a_carrier_round_trips_its_summary() {
        let subject = DidController::incept_single().unwrap();
        let summary = a_summary_for(&subject.kel());
        let (relayer_did, relayer_device) = a_relayer();
        let obj = gossip_summary_carrier(&summary, &relayer_did, &relayer_device).unwrap();
        assert_eq!(
            obj.object_type,
            ObjectType::Custom(GOSSIP_SUMMARY_CARRIER.to_string())
        );
        let Payload::Public(bytes) = &obj.payload else {
            panic!("expected a public payload");
        };
        assert_eq!(KelHeadSummary::decode(bytes).unwrap(), summary);
    }

    #[test]
    fn comparing_against_an_empty_cache_reports_no_local_knowledge() {
        let subject = DidController::incept_single().unwrap();
        let summary = a_summary_for(&subject.kel());
        let (relayer_did, relayer_device) = a_relayer();
        let obj = gossip_summary_carrier(&summary, &relayer_did, &relayer_device).unwrap();
        let cache = KelCache::new();
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::NoLocalKnowledge
        );
    }

    #[test]
    fn a_summary_matching_the_cached_kel_agrees() {
        let subject = DidController::incept_single().unwrap();
        let summary = a_summary_for(&subject.kel());
        let (relayer_did, relayer_device) = a_relayer();
        let obj = gossip_summary_carrier(&summary, &relayer_did, &relayer_device).unwrap();
        let mut cache = KelCache::new();
        cache.insert_verified(subject.kel().clone());
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::Compared(HeadAgreement::Agrees)
        );
    }

    #[test]
    fn a_summary_disagreeing_at_the_same_sequence_is_reported() {
        let subject = DidController::incept_single().unwrap();
        let mut summary = a_summary_for(&subject.kel());
        summary.event_digest = vec![0xEE; 34];
        let (relayer_did, relayer_device) = a_relayer();
        let obj = gossip_summary_carrier(&summary, &relayer_did, &relayer_device).unwrap();
        let mut cache = KelCache::new();
        cache.insert_verified(subject.kel().clone());
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::Compared(HeadAgreement::Disagreement { at_sequence: 0 })
        );
    }

    #[test]
    fn stale_incoming_gossip_leaves_our_cache_ahead() {
        // `compare_gossip_carrier` compares OUR cached knowledge against
        // the incoming claim (`compare_head_summaries(ours, summary)`), so
        // the result describes our cache relative to the gossip. Cache
        // holds the *rotated* (newer) KEL; the incoming gossip claims the
        // *pre-rotation* (older) head — our cache is `Ahead` of it.
        let mut subject = DidController::incept_single().unwrap();
        let stale_summary = a_summary_for(&subject.kel());
        let witness_root = DidController::incept_single().unwrap();
        subject
            .appoint_witnesses(vec![witness_root.did()], 1)
            .unwrap();
        assert!(a_summary_for(&subject.kel()).sequence > stale_summary.sequence);

        let mut cache = KelCache::new();
        cache.insert_verified(subject.kel().clone());

        let (relayer_did, relayer_device) = a_relayer();
        let stale_obj =
            gossip_summary_carrier(&stale_summary, &relayer_did, &relayer_device).unwrap();
        assert_eq!(
            compare_gossip_carrier(&cache, &stale_obj),
            GossipCarrierOutcome::Compared(HeadAgreement::Ahead { by: 1 })
        );
    }

    #[test]
    fn fresher_incoming_gossip_leaves_our_cache_behind() {
        // Cache holds only the *pre-rotation* (older) KEL; the incoming
        // gossip claims the *rotated* (newer) head — our cache is
        // `Behind` it, exactly the signal that should trigger a real KEL
        // fetch (this module's own doc: a caller's job, not this one's).
        let mut subject = DidController::incept_single().unwrap();
        let mut cache = KelCache::new();
        cache.insert_verified(subject.kel().clone());

        let witness_root = DidController::incept_single().unwrap();
        subject
            .appoint_witnesses(vec![witness_root.did()], 1)
            .unwrap();
        let fresher_summary = a_summary_for(&subject.kel());

        let (relayer_did, relayer_device) = a_relayer();
        let fresher_obj =
            gossip_summary_carrier(&fresher_summary, &relayer_did, &relayer_device).unwrap();
        assert_eq!(
            compare_gossip_carrier(&cache, &fresher_obj),
            GossipCarrierOutcome::Compared(HeadAgreement::Behind { by: 1 })
        );
    }

    #[test]
    fn a_non_carrier_object_is_reported_as_not_a_carrier() {
        let (relayer_did, relayer_device) = a_relayer();
        let obj = ObjectBuilder::new(ObjectType::POST)
            .payload(Payload::Public(b"hello".to_vec()))
            .sign(&relayer_did, &relayer_device)
            .unwrap();
        let cache = KelCache::new();
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::NotACarrier
        );
    }

    #[test]
    fn a_truncated_payload_is_reported_as_malformed() {
        let (relayer_did, relayer_device) = a_relayer();
        let obj = ObjectBuilder::new(ObjectType::Custom(GOSSIP_SUMMARY_CARRIER.to_string()))
            .payload(Payload::Public(vec![1, 2, 3]))
            .sign(&relayer_did, &relayer_device)
            .unwrap();
        let cache = KelCache::new();
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::Malformed
        );
    }

    #[test]
    fn an_encrypted_payload_is_reported_as_malformed() {
        let subject = DidController::incept_single().unwrap();
        let summary = a_summary_for(&subject.kel());
        let (relayer_did, relayer_device) = a_relayer();
        let obj = ObjectBuilder::new(ObjectType::Custom(GOSSIP_SUMMARY_CARRIER.to_string()))
            .payload(Payload::Encrypted(summary.encode()))
            .sign(&relayer_did, &relayer_device)
            .unwrap();
        let cache = KelCache::new();
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::Malformed
        );
    }

    #[test]
    fn an_oversized_payload_is_reported_as_malformed() {
        let (relayer_did, relayer_device) = a_relayer();
        let obj = ObjectBuilder::new(ObjectType::Custom(GOSSIP_SUMMARY_CARRIER.to_string()))
            .payload(Payload::Public(vec![
                0u8;
                MAX_GOSSIP_SUMMARY_CARRIER_BYTES + 1
            ]))
            .sign(&relayer_did, &relayer_device)
            .unwrap();
        let cache = KelCache::new();
        assert_eq!(
            compare_gossip_carrier(&cache, &obj),
            GossipCarrierOutcome::Malformed
        );
    }

    #[test]
    fn a_gossip_summary_carrier_flows_through_ordinary_ingest_like_any_authored_object() {
        // Not self-certifying like a KEL carrier -- must go through the
        // same author-provenance path every other object uses: unknown
        // author is rejected, known author (root + delegated device
        // cached, with POST capability) is accepted.
        let mut root = DidController::incept_single().unwrap();
        let device = DidController::incept_device(
            &root.did(),
            vec![mini_crypto::SigningKey::generate().unwrap()],
            1,
            vec![mini_crypto::SigningKey::generate().unwrap()],
            1,
        )
        .unwrap();
        root.delegate_device(&device.did(), Capabilities::primary())
            .unwrap();

        let subject = DidController::incept_single().unwrap();
        let summary = a_summary_for(&subject.kel());
        let obj = gossip_summary_carrier(&summary, &root.did(), &device).unwrap();

        let mut cache = KelCache::new();
        // Unknown author: rejected.
        assert_eq!(
            crate::Ingest::check(&mut cache, &obj),
            crate::IngestOutcome::UnknownAuthor
        );

        // Known author (root + delegated device both cached): accepted
        // like any ordinary object.
        cache.insert_verified(root.kel().clone());
        cache.insert_verified(device.kel().clone());
        assert_eq!(
            crate::Ingest::check(&mut cache, &obj),
            crate::IngestOutcome::Accepted
        );
    }
}
