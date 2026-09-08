//! Shielded-spend validity crosses the voice/value wall through opaque records
//! and an injected verifier. Execution, voting, commitment, and recovery require
//! independent claim verification; a quorum certificate never substitutes for it.
//! The consensus crates cannot inspect amounts or use them as vote weights.
//! Missing evidence rejects a candidate until the evidence is available.

use mini_crypto::HashAlgorithm;

/// Hard cap on shielded-spend records per block, applied before any
/// allocation — the same discipline [`crate::MAX_CLAIMS_PER_BLOCK`] applies
/// to the transparent list.
pub const MAX_NULLIFIERS_PER_BLOCK: usize = 4_096;

/// Longest key image the ledger will store. Ristretto key images are 32
/// bytes; the ceiling leaves room for a post-quantum successor without
/// letting an untrusted field amplify state memory.
pub const MAX_KEY_IMAGE_BYTES: usize = 64;

/// One shielded spend's claim on one output.
///
/// A claim spending several outputs contributes several records, all
/// carrying that claim's digest. That grouping is load-bearing — see
/// [`crate::apply_block`] for why a group is all-or-nothing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NullifierRecord {
    /// The double-spend nullifier, opaque here.
    ///
    /// Produced by `mini_private_payment::VerifiedPrivateClaim::key_images`,
    /// which this crate cannot name and does not link.
    pub key_image: Vec<u8>,
    /// The digest of the claim that spent it — the claim's identity, from
    /// `VerifiedPrivateClaim::transcript_digest`.
    pub claim_digest: [u8; 32],
}

impl NullifierRecord {
    pub fn new(key_image: impl Into<Vec<u8>>, claim_digest: [u8; 32]) -> Self {
        NullifierRecord {
            key_image: key_image.into(),
            claim_digest,
        }
    }

    /// Whether this record is storable at all: a key image must be
    /// non-empty and within [`MAX_KEY_IMAGE_BYTES`].
    ///
    /// Checked before anything is inserted, so an oversized field from an
    /// untrusted proposer cannot amplify state memory.
    pub fn is_well_formed(&self) -> bool {
        !self.key_image.is_empty() && self.key_image.len() <= MAX_KEY_IMAGE_BYTES
    }

    /// Canonical bytes, for the body hash.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut w = Vec::with_capacity(4 + self.key_image.len() + 32);
        w.extend_from_slice(&(self.key_image.len() as u32).to_be_bytes());
        w.extend_from_slice(&self.key_image);
        w.extend_from_slice(&self.claim_digest);
        w
    }

    pub fn digest(&self) -> [u8; 32] {
        let mut w = Vec::new();
        w.extend_from_slice(b"mini-execution/nullifier-record/v1");
        w.extend_from_slice(&self.canonical_bytes());
        HashAlgorithm::Blake3.digest(&w)
    }
}

/// The validator-set half of R8's still-open validity rule (D-0474):
/// independently confirms a real, valid claim produced a group of
/// same-digest [`NullifierRecord`]s, without this crate ever depending on
/// the cryptography that proves it.
///
/// A claim spending several outputs contributes several [`NullifierRecord`]s
/// that all carry its digest ([`NullifierRecord::claim_digest`]) — `group`
/// is exactly that set, and `digest` is the value they all share. An
/// implementor typically looks the digest up in its own locally-held claim
/// evidence (never part of the canonical block body or wire protocol —
/// nothing here can require that without this crate learning what a claim
/// even is), decodes and verifies it with whatever cryptography it links,
/// and confirms the result's own key images and transcript digest exactly
/// match `group`/`digest` rather than merely existing.
///
/// Required for every shielded state transition and checkpoint. A missing
/// verifier rejects shielded input; transparent-only bodies remain supported.
/// `mini-shielded-verify` provides the concrete cryptographic implementation
/// without adding a value dependency to the consensus crates.
pub trait ClaimVerifier: Send + Sync {
    /// Returns `true` only if a real claim verifies and its key
    /// images/transcript digest exactly match `group`/`digest` — never on
    /// trust, never on the claim's mere presence.
    fn verify_claim(
        &self,
        network_id: &[u8; 32],
        digest: &[u8; 32],
        group: &[NullifierRecord],
    ) -> Option<crate::ShieldedClaimEffects>;

    /// Verify explicit genesis funding and public commitment encodings.
    fn verify_genesis_allocation(
        &self,
        network_id: &[u8; 32],
        allocation: &crate::ShieldedGenesisAllocation,
    ) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_is_well_formed_only_within_its_bounds() {
        assert!(NullifierRecord::new(vec![7u8; 32], [1u8; 32]).is_well_formed());
        assert!(NullifierRecord::new(vec![7u8; MAX_KEY_IMAGE_BYTES], [1u8; 32]).is_well_formed());
        // Empty names no output; oversized amplifies state memory from an
        // untrusted field.
        assert!(!NullifierRecord::new(Vec::new(), [1u8; 32]).is_well_formed());
        assert!(
            !NullifierRecord::new(vec![7u8; MAX_KEY_IMAGE_BYTES + 1], [1u8; 32]).is_well_formed()
        );
    }

    #[test]
    fn the_length_prefix_stops_two_different_records_hashing_alike() {
        // Without the prefix, ("ab", digest) and ("a", "b" ++ digest) would
        // serialize to the same bytes. The key image is attacker-chosen, so
        // this is a real collision to close rather than a theoretical one.
        let left = NullifierRecord::new(b"ab".to_vec(), [0u8; 32]);
        let right = NullifierRecord::new(b"a".to_vec(), [0u8; 32]);
        assert_ne!(left.digest(), right.digest());
        assert_ne!(left.canonical_bytes(), right.canonical_bytes());
    }

    #[test]
    fn the_record_digest_is_a_stable_vector() {
        // Pinned: this is what a block body commits to, so a change here is
        // a consensus-format change.
        let record = NullifierRecord::new(vec![0xabu8; 32], [0xcd; 32]);
        let hex: String = record.digest().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "779184dc2b4dbc6fe3e383fa851abdda5cb76f3a51d4f9698fab488b02587935"
        );
    }
}
