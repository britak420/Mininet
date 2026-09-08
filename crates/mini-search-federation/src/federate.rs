//! Federated merge with explicit score provenance. Local computation wins
//! over remote assertions; remote self-scores never choose a URL's displayed
//! representative or its position. Every distinct remote claim for a retained
//! URL survives in `remote_claims`, including profile and observation links.
//! Locally computed scores still depend on source-supplied metadata: recomputing
//! a formula is not verification of the metadata's factual truth.

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
    pub(crate) result: ResultProvenance,
    provider: ProviderPseudonym,
    origin: ResultOrigin,
    remote_claims: Vec<RemoteClaim>,
    pub(crate) reweighted: bool,
}

/// One preserved assertion, not verified ranking evidence. Source ids are
/// locators until the signed observation/index material has been retrieved and
/// checked. Conflicting titles, profiles, scores and source links are retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteClaim {
    pub result: ResultProvenance,
    pub provider: ProviderPseudonym,
}

impl FederatedResult {
    /// Build a [`FederatedResult`] from a remote peer's self-reported
    /// evidence. Always tagged [`ResultOrigin::RemoteAsserted`] --
    /// callers cannot choose otherwise. Used by
    /// `mini-search-federation-net`'s remote-query path; nothing in this
    /// crate calls it, since [`federate_query`] already holds what it
    /// needs to construct a `LocallyComputed` result directly.
    pub fn remote_asserted(result: ResultProvenance, provider: ProviderPseudonym) -> Self {
        let claim = RemoteClaim {
            result: result.clone(),
            provider: provider.clone(),
        };
        FederatedResult {
            result,
            provider,
            origin: ResultOrigin::RemoteAsserted,
            remote_claims: vec![claim],
            reweighted: false,
        }
    }

    pub fn result(&self) -> &ResultProvenance {
        &self.result
    }
    pub fn provider(&self) -> &ProviderPseudonym {
        &self.provider
    }
    pub fn remote_claims(&self) -> &[RemoteClaim] {
        &self.remote_claims
    }
    /// True when profile weights were reapplied without recomputing the
    /// order-dependent diversity signal. Never describe this as a fresh rank.
    pub fn reuses_original_diversity(&self) -> bool {
        self.reweighted
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
                remote_claims: Vec::new(),
                reweighted: false,
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
    for mut candidate in results {
        let key = candidate.result.result.url.canonical_string();
        if let Some(mut existing) = merged.remove(&key) {
            let mut claims = std::mem::take(&mut existing.remote_claims);
            claims.append(&mut candidate.remote_claims);
            claims.sort_by(remote_claim_order);
            claims.dedup();
            let mut selected = if better(&candidate, &existing) {
                candidate
            } else {
                existing
            };
            selected.remote_claims = claims;
            merged.insert(key, selected);
        } else {
            merged.insert(key, candidate);
        }
    }
    let mut out: Vec<FederatedResult> = merged.into_values().collect();
    out.sort_by(|a, b| {
        // A remote maximum cannot crowd local evidence out of max_results,
        // including when the hostile URL differs from every local URL.
        origin_order(a.origin)
            .cmp(&origin_order(b.origin))
            .then_with(|| {
                if a.origin == ResultOrigin::LocallyComputed {
                    b.result
                        .result
                        .relevance_score_bps
                        .value()
                        .cmp(&a.result.result.relevance_score_bps.value())
                } else {
                    std::cmp::Ordering::Equal
                }
            })
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

fn origin_order(origin: ResultOrigin) -> u8 {
    match origin {
        ResultOrigin::LocallyComputed => 0,
        ResultOrigin::RemoteAsserted => 1,
    }
}

fn remote_claim_order(a: &RemoteClaim, b: &RemoteClaim) -> std::cmp::Ordering {
    a.provider
        .0
        .to_bytes()
        .cmp(&b.provider.0.to_bytes())
        // This is a local display tiebreak, not a consensus encoding or a
        // truth score. Include every claim field to make same-provider
        // equivocation and repeated merges deterministic as well.
        .then_with(|| format!("{:?}", a.result).cmp(&format!("{:?}", b.result)))
}

fn better(a: &FederatedResult, b: &FederatedResult) -> bool {
    if a.origin != b.origin {
        return a.origin == ResultOrigin::LocallyComputed;
    }
    if a.origin == ResultOrigin::RemoteAsserted {
        return remote_claim_order(
            &RemoteClaim {
                result: a.result.clone(),
                provider: a.provider.clone(),
            },
            &RemoteClaim {
                result: b.result.clone(),
                provider: b.provider.clone(),
            },
        )
        .is_lt();
    }
    let a_score = a.result.result.relevance_score_bps.value();
    let b_score = b.result.result.relevance_score_bps.value();
    a_score > b_score || (a_score == b_score && a.provider.0.to_bytes() < b.provider.0.to_bytes())
}
