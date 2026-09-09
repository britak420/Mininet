//! The concrete "whoever cares" validator-side shielded-spend verifier
//! (D-0474, [roadmap R8](../../../docs/ROADMAP_TO_RELEASE.md)) —
//! `mini_execution::nullifier`'s own docs name this exact split:
//! "Verification belongs where it already is — `mini_private_payment
//! ::verify`, run by whoever cares — and the chain orders the results."
//! This crate is that "whoever cares," kept in its own crate specifically
//! so `mini-execution`/`mini-consensus`/`mini-chain` never link
//! `mini-private-payment`/`mini-value` (the voice/value wall, P1,
//! Directive 16) — the same reasoning `mini_chain::ValidatorOracle`'s own
//! split already established for identity/KEL material.
//!
//! ## How a validator uses this
//!
//! ```ignore
//! let evidence = Arc::new(ClaimEvidencePool::new());
//! // ... elsewhere, as full claims arrive over the network:
//! evidence.insert(claim_wire_bytes)?;
//! // ... constructing the node:
//! let verifier = Arc::new(ShieldedClaimVerifier::new(network_id, evidence.clone()));
//! let node = ConsensusNode::new(config).with_claim_verifier(verifier);
//! ```
//!
//! ## Honest limits
//!
//! - **No claim-evidence gossip protocol.** [`ClaimEvidencePool`] is a
//!   plain local store; how full claim bytes actually reach a validator
//!   (a dedicated gossip topic, a request/response protocol, an existing
//!   mempool) is not this crate's job — it only defines the shape a
//!   validator's own evidence-gathering component must fill.
//! - **No pruning/eviction policy.** A pool that only ever grows is a
//!   real operational concern for a long-running validator; this crate
//!   states that rather than pretending a policy already exists.
//! - **No accountability/measurement layer.** Roadmap R8 asks for a
//!   validator set that "verifies claims and is measured for it" — this
//!   crate provides the verification; recording *which* validators ran
//!   it (an evidence trail analogous to `mini_consensus::evidence`'s
//!   equivocation proofs) remains unbuilt.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use mini_execution::{
    ClaimVerifier, NullifierRecord, ShieldedClaimEffects, ShieldedGenesisAllocation, ShieldedOutput,
};
use mini_private_payment::PrivatePaymentClaim;

/// A claim's wire bytes failed to decode — see
/// [`PrivatePaymentClaim::decode`]'s own "structural validation only"
/// scope; this never means the claim is invalid, only unstorable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MalformedClaim;

/// A validator's local, non-canonical store of full shielded-spend claim
/// bytes it has learned about — never part of the canonical block body or
/// wire protocol (`mini_execution::nullifier`'s own docs explain why that
/// boundary is load-bearing, not an oversight). Keyed by each claim's own
/// `transcript_digest`, computed here rather than taken on a caller's
/// word, so a lookup can never retrieve bytes that don't actually belong
/// to the digest asked for.
#[derive(Debug, Default)]
pub struct ClaimEvidencePool {
    claims: RwLock<BTreeMap<[u8; 32], Vec<u8>>>,
}

impl ClaimEvidencePool {
    /// A fresh, empty pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a claim's wire bytes, keyed by its own `transcript_digest`.
    /// Only structural decoding is checked here — a decoded-but-invalid
    /// claim is still stored, exactly like `PrivatePaymentClaim::decode`'s
    /// own contract: "a decoded claim is not a verified one."
    /// [`ShieldedClaimVerifier::verify_claim`] is what decides trust, at
    /// lookup time, so a stale or since-superseded evidence entry can
    /// never grant more trust than the actual claim it holds bears.
    pub fn insert(&self, claim_bytes: Vec<u8>) -> Result<[u8; 32], MalformedClaim> {
        let claim = PrivatePaymentClaim::decode(&claim_bytes).map_err(|_| MalformedClaim)?;
        let digest = claim.transcript_digest();
        self.claims
            .write()
            .expect("claim evidence pool lock poisoned")
            .insert(digest, claim_bytes);
        Ok(digest)
    }

    /// The stored wire bytes for `digest`, if this pool has ever seen one.
    pub fn get(&self, digest: &[u8; 32]) -> Option<Vec<u8>> {
        self.claims
            .read()
            .expect("claim evidence pool lock poisoned")
            .get(digest)
            .cloned()
    }

    /// How many claims this pool currently holds.
    pub fn len(&self) -> usize {
        self.claims
            .read()
            .expect("claim evidence pool lock poisoned")
            .len()
    }

    /// Whether this pool holds no claims yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The concrete [`ClaimVerifier`]: looks a claim up in its own
/// [`ClaimEvidencePool`], decodes and verifies it with
/// `mini_private_payment::verify`, and confirms the result's own key
/// images and transcript digest exactly match what it is being asked to
/// vouch for — a claim existing under the right digest is necessary but
/// never sufficient; it must also actually verify and its key images must
/// exactly match the group, no more and no fewer.
#[derive(Debug)]
pub struct ShieldedClaimVerifier {
    network_id: [u8; 32],
    evidence: Arc<ClaimEvidencePool>,
}

impl ShieldedClaimVerifier {
    /// A verifier checking claims against `network_id`, reading evidence
    /// from `evidence` — typically shared (via the same `Arc`) with
    /// whatever component populates the pool as claims arrive.
    pub fn new(network_id: [u8; 32], evidence: Arc<ClaimEvidencePool>) -> Self {
        ShieldedClaimVerifier {
            network_id,
            evidence,
        }
    }
}

impl ClaimVerifier for ShieldedClaimVerifier {
    fn verify_claim(
        &self,
        network_id: &[u8; 32],
        digest: &[u8; 32],
        group: &[NullifierRecord],
    ) -> Option<ShieldedClaimEffects> {
        if network_id != &self.network_id {
            return None;
        }
        let Some(bytes) = self.evidence.get(digest) else {
            // No evidence, no trust — the honest default a caller-injected
            // verifier must have (see mini-execution::ClaimVerifier's own
            // docs: never on the claim's mere presence, and here there is
            // not even that).
            return None;
        };
        let Ok(claim) = PrivatePaymentClaim::decode(&bytes) else {
            return None;
        };
        let Ok(verified) = mini_private_payment::verify(&claim, &self.network_id) else {
            return None;
        };
        if verified.transcript_digest() != digest {
            return None;
        }
        let mut expected: BTreeSet<Vec<u8>> = verified.key_images().map(|k| k.to_vec()).collect();
        if expected.len() != group.len() {
            return None;
        }
        for record in group {
            // Defense in depth: `apply_nullifiers` already only ever
            // groups records sharing one digest before calling this, but
            // a verifier is a trust boundary — it must not assume every
            // future caller upholds that precondition perfectly.
            if record.claim_digest != *digest {
                return None;
            }
            if !expected.remove(&record.key_image) {
                return None;
            }
        }
        if !expected.is_empty() {
            return None;
        }
        if !claim.outputs.iter().all(|output| {
            mini_value::one_time_key_is_well_formed(&output.output.one_time_address)
                && mini_value::one_time_key_is_well_formed(&output.output.tx_public_key)
        }) {
            return None;
        }
        Some(ShieldedClaimEffects {
            ring_members: claim
                .inputs
                .iter()
                .flat_map(|input| {
                    input
                        .ring
                        .iter()
                        .zip(&input.ring_commitments)
                        .map(|(key, commitment)| ShieldedOutput {
                            public_key: key.clone(),
                            amount_commitment: commitment.clone(),
                        })
                })
                .collect(),
            outputs: claim
                .outputs
                .iter()
                .map(|output| ShieldedOutput {
                    public_key: output.output.one_time_address.clone(),
                    amount_commitment: output.amount_commitment.clone(),
                })
                .collect(),
            fee_micro: claim.fee_micro,
        })
    }
    fn verify_genesis_allocation(
        &self,
        network_id: &[u8; 32],
        allocation: &ShieldedGenesisAllocation,
    ) -> bool {
        network_id == &self.network_id
            && mini_value::one_time_key_is_well_formed(&allocation.output.public_key)
            && allocation.output.amount_commitment
                == mini_value::public_amount_commitment(allocation.amount_micro)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_pool_holds_nothing() {
        let pool = ClaimEvidencePool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
        assert_eq!(pool.get(&[0u8; 32]), None);
    }

    #[test]
    fn malformed_bytes_are_rejected_before_storage() {
        let pool = ClaimEvidencePool::new();
        assert_eq!(pool.insert(vec![0xff; 4]), Err(MalformedClaim));
        assert!(pool.is_empty());
    }

    #[test]
    fn a_verifier_with_no_evidence_never_trusts_a_claim() {
        let evidence = Arc::new(ClaimEvidencePool::new());
        let verifier = ShieldedClaimVerifier::new([7u8; 32], evidence);
        let group = [NullifierRecord::new(vec![1u8; 32], [9u8; 32])];
        assert!(verifier
            .verify_claim(&[7; 32], &[9u8; 32], &group)
            .is_none());
    }

    #[test]
    fn a_verifier_rejects_a_group_whose_own_digest_does_not_match_the_records() {
        // Defense-in-depth path: even if a caller mismatches digest and
        // group (which honest callers never do), the verifier must not
        // trust it.
        let evidence = Arc::new(ClaimEvidencePool::new());
        let verifier = ShieldedClaimVerifier::new([7u8; 32], evidence);
        let mismatched_group = [NullifierRecord::new(vec![1u8; 32], [1u8; 32])];
        assert!(verifier
            .verify_claim(&[7; 32], &[9u8; 32], &mismatched_group)
            .is_none());
    }
}
