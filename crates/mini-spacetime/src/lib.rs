//! Merkle possession proofs and storage byte measurements.
//!
//! These primitives do not authorize rewards or block-production weight. The
//! audited registration and lifecycle boundary lives in mini-storage-fraud.
//! Possession alone does not prove replication uniqueness or human uniqueness.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

mod error;
mod isqrt;
mod merkle;
mod proof;
mod storage_proof;

pub use error::{Result, SpaceTimeError};
pub use isqrt::isqrt;
pub use merkle::{MerkleProof, MerkleTree};
pub use proof::{NoProof, ProofOfSpaceTimeSource};
pub use storage_proof::{
    verify_storage_challenge, MerkleStorageProof, ObservedCapacity, ProofHistory, StorageChallenge,
    StorageChallengeResponse, StorageCommitment, StorageUnitPolicy, StorageWindowPolicy,
};
