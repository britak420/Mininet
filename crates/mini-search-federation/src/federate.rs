//! F3: federated query — merge candidates from multiple providers while
//! preserving provenance (`docs/research/
//! MININET_NATIVE_INTAKE_PUBLIC_COMMONS_AND_OPEN_WEB_SEARCH_20260718.md`
//! §29).
//!
//! This module runs the *unmodified* `mini_query::search` once per
//! provider's own `(IndexSegment, Corpus, DocumentContextTable)`, then
//! merges the resulting per-provider result lists into one list. It does
//! not re-rank, re-score, or re-implement any of Track E6-E8's scoring,
//! filtering, or provenance logic -- merging is the only new behavior
//! here, and it is a pure, deterministic function of each provider's
//! already-computed, already-provenanced results.
//!
//! ## The merge policy
//!
//! Every provider is queried with the identical `profile`, `parsed`
//! query, and `now_ms`, so `relevance_score_bps` values are directly
//! comparable across providers (D-0312's determinism guarantee holds
//! per-call, and nothing here breaks it across calls). The merge:
//!
//! 1. Concatenates every provider's results, each tagged with the
//!    [`mini_web_types::ProviderPseudonym`] that supplied it and a
//!    [`ResultOrigin`] (see below).
//! 2. Deduplicates by canonical URL string: when two providers return the
//!    same URL, [`ResultOrigin::LocallyComputed`] beats
//!    [`ResultOrigin::RemoteAsserted`] regardless of the claimed score;
//!    otherwise the higher `relevance_score_bps` wins; ties break on the
//!    smaller provider pseudonym bytes -- deterministic regardless of
//!    input provider order.
//! 3. Sorts the deduplicated set by score descending, tie-breaking on
//!    canonical URL string bytes (mirroring `mini_ranker::rank`'s own
//!    `UrlId`-byte tiebreak discipline), and truncates to `max_results`.
//!
//! A provider cannot inflate its own influence by returning more results
//! than it has documents for, or by duplicating one document under
//! several URLs, without an accompanying score high enough to win the
//! deduplication step under the *same* deterministic scoring
//! [`mini_query::search`] already applies -- this module adds no new
//! trust in what a provider claims about its own content beyond what
//! `search`'s own D-0312 invariants (no pay-to-rank, no personalization,
//! availability-filtered) already provide per provider.
//!
//! ## Score provenance (PR #327 finding F-21)
//!
//! Every result this module itself produces (via [`federate_query`], over
//! a caller-held [`FederationSource`]) was scored by this process's own
//! `mini_query::search` call over data the caller actually holds --
//! [`ResultOrigin::LocallyComputed`]. `mini-search-federation-net`'s
//! remote-query path folds in a peer's *self-reported*
//! `relevance_score_bps` for content this process never independently
//! scored -- [`ResultOrigin::RemoteAsserted`]. Provenance (which peer
//! claimed a score) is not the same as truth (whether the score is
//! accurate): a hostile remote peer can report the maximum possible score
//! for anything to try to win every dedup comparison. Marking the origin
//! and preferring locally-verified evidence in [`better`] closes exactly
//! that: a remote assertion can no longer silently outrank this process's
//! own independently-computed result for the same URL merely by claiming
//! a bigger number. It does **not** resolve two competing remote,
//! unverified assertions against each other -- there is no local evidence
//! to prefer between them, and this module does not claim a ranking truth
//! oracle (see this finding's own Boundary note).

use std::collections::HashMap;

use mini_lexical_index::IndexSegment;
use mini_query::{search, DocumentContextTable, ParsedQuery, ResultProvenance};
use mini_ranker::Corpus;
use mini_web_types::{IndexSegmentId, ProviderPseudonym, RankingProfile};

use crate::error::Result;

/// One provider's queryable local state: its index segment, the metadata
/// corpus `mini_query::search` needs, the E8 provenance context table, and
/// the segment's own content-addressed id (attached to every result it
/// produces, exactly as [`mini_query::search`] already does for a single
/// provider).
#[derive(Debug)]
pub struct FederationSource<'a> {
    pub provider: ProviderPseudonym,
    pub index: &'a IndexSegment,
    pub corpus: &'a Corpus,
    pub contexts: &'a DocumentContextTable,
    pub index_segment: IndexSegmentId,
}

/// Whether a [`FederatedResult`]'s score was independently computed by
/// this process over data it actually holds, or is a remote peer's own,
/// unverified claim. See the module docs' "Score provenance" section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultOrigin {
    /// Scored by this process's own `mini_query::search` call over a
    /// [`FederationSource`] it actually holds.
    LocallyComputed,
    /// A remote peer's self-reported score, folded in without local
    /// verification (`mini-search-federation-net`'s remote-query path).
    RemoteAsserted,
}

/// One merged result: the same [`mini_query::ResultProvenance`] a single
/// provider's `search` would have produced, plus which provider it came
/// from and whether that score was locally verified or remotely
/// asserted.
///
/// `origin` is deliberately not `pub`: if any caller could freely set it,
/// a careless or hostile source could simply claim `LocallyComputed` for
/// content it never independently verified and defeat [`better`]'s whole
/// point. [`FederatedResult::remote_asserted`] is the only constructor
/// available outside this module, and it can only ever produce
/// [`ResultOrigin::RemoteAsserted`] -- [`federate_query`], in this same
/// module, is the only code anywhere that can produce
/// [`ResultOrigin::LocallyComputed`], because it is the only code that
/// actually ran `mini_query::search` itself (PR #327 finding F-21; the
/// same unforgeable-typed-domain discipline D-0490/D-0495 already apply
/// elsewhere in this workspace).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedResult {
    pub result: ResultProvenance,
    pub provider: ProviderPseudonym,
    origin: ResultOrigin,
}

impl FederatedResult {
    /// Build a [`FederatedResult`] from a remote peer's self-reported
    /// evidence. Always tagged [`ResultOrigin::RemoteAsserted`] --
    /// callers cannot choose otherwise. Used by
    /// `mini-search-federation-net`'s remote-query path; nothing in this
    /// crate calls it, since [`federate_query`] already holds what it
    /// needs to construct a `LocallyComputed` result directly.
    pub fn remote_asserted(result: ResultProvenance, provider: ProviderPseudonym) -> Self {
        FederatedResult {
            result,
            provider,
            origin: ResultOrigin::RemoteAsserted,
        }
    }

    /// Whether this result's score was independently computed by this
    /// process, or is an unverified remote claim.
    pub fn origin(&self) -> ResultOrigin {
        self.origin
    }
}

/// Query every source and deterministically merge the results (see module
/// docs for the exact policy). Each source is queried for up to
/// `max_results` of its own candidates before merging, so a source with
/// many strong local matches cannot be starved by a source with few.
pub fn federate_query(
    sources: &[FederationSource],
    profile: &RankingProfile,
    parsed: &ParsedQuery,
    now_ms: u64,
    max_results: usize,
) -> Result<Vec<FederatedResult>> {
    let mut candidates = Vec::new();
    for source in sources {
        let results = search(
            source.index,
            source.corpus,
            source.contexts,
            profile,
            parsed,
            source.index_segment.clone(),
            now_ms,
            max_results,
        )?;
        for result in results {
            candidates.push(FederatedResult {
                result,
                provider: source.provider.clone(),
                origin: ResultOrigin::LocallyComputed,
            });
        }
    }
    Ok(merge_federated_results(candidates, max_results))
}

/// The dedup/sort/truncate merge policy documented at module level, exposed
/// standalone so a caller who already holds [`FederatedResult`]s from
/// somewhere other than a fresh [`search`] call over a local
/// [`FederationSource`] -- e.g. `mini-search-federation-net`'s Track F6
/// remote-query results, tagged with the answering peer's
/// [`mini_web_types::ProviderPseudonym`] -- can fold them into the same
/// deterministic merge [`federate_query`] itself uses, rather than
/// reimplementing the dedup/tiebreak policy. `federate_query` is exactly
/// this function applied to results freshly computed from local sources.
pub fn merge_federated_results(
    results: Vec<FederatedResult>,
    max_results: usize,
) -> Vec<FederatedResult> {
    let mut merged: HashMap<String, FederatedResult> = HashMap::new();
    for candidate in results {
        let key = candidate.result.result.url.canonical_string();
        match merged.get(&key) {
            None => {
                merged.insert(key, candidate);
            }
            Some(existing) => {
                if better(&candidate, existing) {
                    merged.insert(key, candidate);
                }
            }
        }
    }

    let mut out: Vec<FederatedResult> = merged.into_values().collect();
    out.sort_by(|a, b| {
        b.result
            .result
            .relevance_score_bps
            .value()
            .cmp(&a.result.result.relevance_score_bps.value())
            .then_with(|| {
                a.result
                    .result
                    .url
                    .canonical_string()
                    .cmp(&b.result.result.url.canonical_string())
            })
    });
    out.truncate(max_results);
    out
}

/// `a` wins over `b` if `a` is [`ResultOrigin::LocallyComputed`] and `b` is
/// not (PR #327 finding F-21: a remote peer's self-asserted score must
/// never outrank this process's own independently-verified evidence for
/// the same URL, no matter what score it claims); otherwise, when both
/// share the same origin, `a` wins if it scores strictly higher, or on a
/// tie if its provider pseudonym bytes are smaller -- deterministic
/// regardless of the order sources were queried in.
fn better(a: &FederatedResult, b: &FederatedResult) -> bool {
    if a.origin != b.origin {
        return a.origin == ResultOrigin::LocallyComputed;
    }
    let a_score = a.result.result.relevance_score_bps.value();
    let b_score = b.result.result.relevance_score_bps.value();
    if a_score != b_score {
        return a_score > b_score;
    }
    a.provider.0.to_bytes() < b.provider.0.to_bytes()
}
