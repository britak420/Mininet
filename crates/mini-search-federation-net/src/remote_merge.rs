//! Track F6 Phase 2: fold one remote peer's already-ranked
//! [`crate::WireResult`]s (from [`crate::remote_query`]) into a caller's own
//! local/pulled [`mini_search_federation::FederatedResult`]s, using the
//! exact same deterministic dedup/tiebreak policy
//! [`mini_search_federation::federate_query`] applies across its own
//! sources ([`mini_search_federation::merge_federated_results`], unmodified
//! and reused directly rather than reimplemented here).
//!
//! F6 Phase 1's own doc named this the deliberately deferred follow-up:
//! `federate_query`'s typed merge expects a real `Corpus`/
//! `DocumentContextTable`-backed [`mini_search_federation::FederationSource`],
//! not a flat list of remote-computed results, so a query response cannot
//! simply be handed to `federate_query` as another source. This module is
//! the missing conversion step: a [`crate::WireResult`] becomes a real
//! [`mini_query::ResultProvenance`] (rejecting any bps value a compliant
//! peer could never have produced -- see [`federated_result_from_wire`]),
//! tagged with the peer's [`mini_web_types::ProviderPseudonym`].
//!
//! **That tag is caller-asserted, not cryptographically verified.** A query
//! response carries no `Object`/signature wrapping (see `query`'s module
//! doc) and F6 provides no caller/provider authentication beyond the
//! channel itself (`docs/design/f6-private-query-transport.md`). A caller
//! names `remote_provider` from whatever out-of-band knowledge it already
//! has of who it dialed (an advertisement it resolved, a session it set up
//! itself) -- exactly as honest, and exactly as unverified, as every other
//! Track F provider label already is once results leave a single signed
//! object's custody.

use mini_query::ResultProvenance;
use mini_search_federation::{merge_federated_results, FederatedResult};
use mini_web_types::{
    CrawlObservationId, ProviderPseudonym, RankingExplanation, SearchResult, WeightBps,
};

use crate::error::{NetError, Result};
use crate::query::{validate_wire_result, AuthenticatedQueryResults, WireResult};

/// Convert one [`WireResult`] into a typed [`FederatedResult`] tagged with
/// `provider`. Rejects (`NetError::Protocol`) any `relevance_score_bps` or
/// `explanation` component above [`WeightBps::MAX`] -- values a compliant
/// [`crate::serve_query`] can never produce, since [`mini_query::search`]
/// only ever emits validated [`WeightBps`]. This conversion invokes the
/// same shared validator as the F6 wire codec because [`WireResult`] is public
/// and can be constructed locally without passing through the decoder. Invalid,
/// noncanonical, oversized, or non-displayable local/legacy inputs therefore
/// fail closed before entering the typed federated merge; the typed `WeightBps`
/// conversion below repeats the score check as defense in depth.
pub fn federated_result_from_wire(
    wire: WireResult,
    provider: ProviderPseudonym,
) -> Result<FederatedResult> {
    // `WireResult` is public and can be constructed locally or supplied by a
    // legacy caller without traversing the F6 decoder. Reuse the exact same
    // canonical URL, field, multihash, score, and displayability validator here
    // before the value enters the typed federated merge.
    validate_wire_result(&wire)?;
    let bps = |v: u16| WeightBps::new(v).map_err(|_| NetError::Protocol);
    let relevance_score_bps = bps(wire.relevance_score_bps)?;
    let explanation = RankingExplanation {
        lexical_bps: bps(wire.explanation[0])?,
        phrase_bps: bps(wire.explanation[1])?,
        link_bps: bps(wire.explanation[2])?,
        freshness_bps: bps(wire.explanation[3])?,
        originality_bps: bps(wire.explanation[4])?,
        diversity_bps: bps(wire.explanation[5])?,
    };
    let result = SearchResult {
        url: wire.url,
        title: wire.title,
        snippet: wire.snippet,
        relevance_score_bps,
        availability: wire.availability,
        ranking_profile: wire.ranking_profile,
        explanation,
    };
    // FederatedResult::remote_asserted is the only constructor available
    // to this crate -- it always tags ResultOrigin::RemoteAsserted, so
    // the merge in mini_search_federation::federate_query can never be
    // told this score was locally verified when it was not (PR #327
    // finding F-21).
    Ok(FederatedResult::remote_asserted(
        ResultProvenance {
            result,
            source_observation: CrawlObservationId(wire.source_observation),
            index_segment: wire.index_segment,
        },
        provider,
    ))
}

/// Merge one remote peer's [`crate::remote_query`] results into a caller's
/// own local/pulled [`FederatedResult`]s, applying
/// [`mini_search_federation::merge_federated_results`]'s deterministic
/// dedup/tiebreak policy across the combined set. `remote_provider` labels
/// which peer supplied `remote` -- see the module doc's caveat that this is
/// caller-asserted, not cryptographically verified. Fails closed
/// (`NetError::Protocol`) on the first out-of-range wire result rather than
/// silently dropping it and returning a partial merge.
pub fn merge_remote_results(
    local: Vec<FederatedResult>,
    remote: Vec<WireResult>,
    remote_provider: ProviderPseudonym,
    max_results: usize,
) -> Result<Vec<FederatedResult>> {
    let mut combined = local;
    combined.reserve(remote.len());
    for wire in remote {
        combined.push(federated_result_from_wire(wire, remote_provider.clone())?);
    }
    Ok(merge_federated_results(combined, max_results))
}

/// Merge authenticated remote results without accepting a caller-selected
/// provider label. The label is carried by [`AuthenticatedQueryResults`], which
/// can only be produced by the named-peer query path on an authenticated
/// transport connection.
pub fn merge_authenticated_remote_results(
    local: Vec<FederatedResult>,
    remote: AuthenticatedQueryResults,
    max_results: usize,
) -> Result<Vec<FederatedResult>> {
    let (provider, results) = remote.into_parts();
    merge_remote_results(local, results, provider, max_results)
}

#[cfg(test)]
mod tests {
    use mini_crypto::{HashAlgorithm, Multihash};
    use mini_search_federation::ResultOrigin;
    use mini_web_types::{
        AvailabilityState, CanonicalUrl, IndexSegmentId, NormalizedHost, RankingProfileId, Scheme,
    };

    use super::*;

    fn digest(seed: &[u8]) -> Multihash {
        Multihash::of(HashAlgorithm::Blake3, seed)
    }

    fn url(path: &str) -> CanonicalUrl {
        CanonicalUrl::new(
            Scheme::Https,
            NormalizedHost::new("example.org").unwrap(),
            None,
            path,
            None,
        )
        .unwrap()
    }

    fn wire_result(path: &str, score: u16) -> WireResult {
        WireResult {
            url: url(path),
            title: "title".to_string(),
            snippet: "snippet".to_string(),
            relevance_score_bps: score,
            availability: AvailabilityState::Available,
            ranking_profile: RankingProfileId(digest(b"profile")),
            explanation: [score, 0, 0, 0, 0, 0],
            source_observation: digest(b"obs"),
            index_segment: IndexSegmentId(digest(b"segment")),
        }
    }

    fn provider(seed: &[u8]) -> ProviderPseudonym {
        ProviderPseudonym(digest(seed))
    }

    #[test]
    fn a_valid_wire_result_converts_and_round_trips_its_fields() {
        let wire = wire_result("/a", 4_000);
        let result = federated_result_from_wire(wire.clone(), provider(b"p1")).unwrap();
        assert_eq!(result.result().result.url, wire.url);
        assert_eq!(
            result.result().result.relevance_score_bps.value(),
            wire.relevance_score_bps
        );
        assert_eq!(
            result.result().source_observation.0,
            wire.source_observation
        );
        assert_eq!(result.provider().clone(), provider(b"p1"));
    }

    #[test]
    fn a_locally_constructed_filtered_result_cannot_bypass_f6_validation() {
        let mut wire = wire_result("/a", 100);
        wire.availability =
            AvailabilityState::Restricted(mini_web_types::RestrictionReason::UserFilter);
        assert_eq!(
            federated_result_from_wire(wire, provider(b"p1")),
            Err(NetError::Protocol)
        );
    }

    #[test]
    fn an_out_of_range_relevance_score_is_rejected() {
        let wire = wire_result("/a", WeightBps::MAX.value() + 1);
        assert_eq!(
            federated_result_from_wire(wire, provider(b"p1")),
            Err(NetError::Protocol)
        );
    }

    #[test]
    fn an_out_of_range_explanation_component_is_rejected() {
        let mut wire = wire_result("/a", 100);
        wire.explanation[3] = WeightBps::MAX.value() + 1;
        assert_eq!(
            federated_result_from_wire(wire, provider(b"p1")),
            Err(NetError::Protocol)
        );
    }

    #[test]
    fn merging_preserves_competing_remote_claims_without_score_authority() {
        let local =
            vec![federated_result_from_wire(wire_result("/a", 1_000), provider(b"local")).unwrap()];
        let remote = vec![wire_result("/a", 9_000), wire_result("/b", 500)];
        let merged = merge_remote_results(local, remote, provider(b"remote"), 10).unwrap();

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].result().result.url, url("/a"));
        assert_eq!(merged[0].remote_claims().len(), 2);
        assert!(merged[0].remote_claims().iter().any(|claim| claim
            .result
            .result
            .relevance_score_bps
            .value()
            == 9_000));
        assert!(merged[0].remote_claims().iter().any(|claim| claim
            .result
            .result
            .relevance_score_bps
            .value()
            == 1_000));
        assert_eq!(merged[1].result().result.url, url("/b"));
    }

    #[test]
    fn merging_respects_max_results_across_the_combined_set() {
        let local =
            vec![federated_result_from_wire(wire_result("/a", 1_000), provider(b"local")).unwrap()];
        let remote = vec![wire_result("/b", 900), wire_result("/c", 800)];
        let merged = merge_remote_results(local, remote, provider(b"remote"), 2).unwrap();
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].result().result.url, url("/a"));
        assert_eq!(merged[1].result().result.url, url("/b"));
    }

    #[test]
    fn an_invalid_remote_result_fails_the_whole_merge_rather_than_dropping_silently() {
        let local =
            vec![federated_result_from_wire(wire_result("/a", 1_000), provider(b"local")).unwrap()];
        let remote = vec![wire_result("/b", WeightBps::MAX.value() + 1)];
        assert_eq!(
            merge_remote_results(local, remote, provider(b"remote"), 10),
            Err(NetError::Protocol)
        );
    }

    /// A genuine [`mini_search_federation::FederatedResult`] built the
    /// only way `ResultOrigin::LocallyComputed` can ever actually happen:
    /// a real `federate_query` call over a real, in-process
    /// `FederationSource`, at canonical URL `/a` on `example.org` (the
    /// same URL `wire_result("/a", ..)` above names), so it can compete
    /// head-to-head against a wire-asserted result for the identical
    /// document.
    fn genuine_local_result(inbound_links: u32) -> FederatedResult {
        use mini_lexical_index::{Field, IndexBuilder, UrlId};
        use mini_query::{parse_query, DocumentContext, DocumentContextTable};
        use mini_ranker::{Corpus, DocumentMeta};
        use mini_search_federation::{federate_query, FederationSource};
        use mini_web_types::RankingProfile;

        let doc_id = UrlId(digest(b"local-doc-a"));
        let mut builder = IndexBuilder::new();
        builder.add_document(
            doc_id.clone(),
            &[
                (Field::Title, "rust guide"),
                (Field::Body, "rust programming"),
            ],
        );
        let mut corpus = Corpus::new();
        corpus.insert(
            &doc_id,
            DocumentMeta {
                url: url("/a"),
                title: "rust guide".to_string(),
                snippet: "rust programming".to_string(),
                observed_at_ms: 0,
                inbound_links,
                content_digest: digest(b"local-doc-a"),
                availability: AvailabilityState::Available,
            },
        );
        let mut contexts = DocumentContextTable::new();
        contexts.insert(
            &doc_id,
            DocumentContext {
                language: Some("en".to_string()),
                media_type: None,
                source_observation: CrawlObservationId(digest(b"local-obs")),
            },
        );
        let index = builder.build();
        let source = FederationSource {
            provider: provider(b"local"),
            index: &index,
            corpus: &corpus,
            contexts: &contexts,
            index_segment: IndexSegmentId(digest(b"local-segment")),
        };
        let profile = RankingProfile::public_default(RankingProfileId(digest(b"public-default")));
        let parsed = parse_query("rust programming");
        let mut merged = federate_query(&[source], &profile, &parsed, 0, 10).unwrap();
        assert_eq!(
            merged.len(),
            1,
            "the fixture query must match exactly the one seeded document"
        );
        merged.remove(0)
    }

    #[test]
    fn a_genuinely_local_result_is_tagged_locally_computed_by_construction() {
        // Sanity check on the test fixture itself: federate_query really
        // does tag its own output LocallyComputed, and the wire path
        // really does tag its own output RemoteAsserted -- if either
        // stopped being true the tests below would pass for the wrong
        // reason.
        let local = genuine_local_result(1);
        assert_eq!(local.origin(), ResultOrigin::LocallyComputed);
        let remote = federated_result_from_wire(wire_result("/a", 1), provider(b"p")).unwrap();
        assert_eq!(remote.origin(), ResultOrigin::RemoteAsserted);
    }

    #[test]
    fn a_hostile_remote_max_score_assertion_cannot_outrank_a_genuine_local_result() {
        // PR #327 finding F-21's own concrete example and named
        // acceptance test ("hostile max-score provider"): a malicious or
        // compromised remote peer reports the maximum possible score for
        // a URL this process already scored for real, honestly, from its
        // own held index. The remote's self-asserted number must not win
        // merely because it is bigger.
        let local_result = genuine_local_result(3);
        let local_score = local_result.result().result.relevance_score_bps.value();
        assert!(
            local_score < WeightBps::MAX.value(),
            "fixture must not already be maxed out, or this test proves nothing"
        );

        let hostile_remote = wire_result("/a", WeightBps::MAX.value());
        let merged = merge_remote_results(
            vec![local_result],
            vec![hostile_remote],
            provider(b"attacker"),
            10,
        )
        .unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].origin(), ResultOrigin::LocallyComputed);
        assert_eq!(
            merged[0].result().result.relevance_score_bps.value(),
            local_score,
            "the locally-verified score must survive unchanged, not be replaced by the hostile claim"
        );
    }

    #[test]
    fn a_genuine_local_result_still_loses_to_a_better_genuine_local_result() {
        // The origin tiebreak must not become "local always wins,
        // regardless of anything else" -- two genuinely LocallyComputed
        // results for the same URL (e.g. two local FederationSources
        // that both happen to hold the same document) still resolve by
        // score, exactly as before this finding's fix.
        let weak = genuine_local_result(1);
        let strong = genuine_local_result(50);
        let merged = merge_federated_results(vec![weak.clone(), strong.clone()], 10);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].origin(), ResultOrigin::LocallyComputed);
        assert!(
            merged[0].result().result.relevance_score_bps.value()
                >= weak.result().result.relevance_score_bps.value()
        );
    }
    #[test]
    fn hostile_remote_scores_cannot_change_order_and_disagreement_survives_remerge() {
        let a = federated_result_from_wire(wire_result("/a", 1), provider(b"p1")).unwrap();
        let mut conflicting = wire_result("/a", 10_000);
        conflicting.ranking_profile = RankingProfileId(digest(b"other-profile"));
        conflicting.source_observation = digest(b"other-observation");
        conflicting.title = "contradictory title".into();
        let b = federated_result_from_wire(conflicting, provider(b"p2")).unwrap();
        let c = federated_result_from_wire(wire_result("/z", 10_000), provider(b"p3")).unwrap();
        let forward = merge_federated_results(vec![a.clone(), b.clone(), c.clone()], 10);
        let reverse = merge_federated_results(vec![c, b.clone(), a], 10);
        assert_eq!(forward, reverse);
        assert_eq!(forward[0].result().result.url, url("/a"));
        assert_eq!(forward[0].remote_claims().len(), 2);
        let again = merge_federated_results(vec![forward[0].clone(), b], 10);
        assert_eq!(again[0], forward[0]);
        let real = genuine_local_result(1);
        let hostile =
            federated_result_from_wire(wire_result("/aaa", 10_000), provider(b"hostile")).unwrap();
        assert_eq!(
            merge_federated_results(vec![hostile, real.clone()], 1),
            vec![real]
        );
    }
}
