//! Content hashing for Mininet.
//!
//! ## Frozen invariant — strong hash, never SHA-1
//!
//! SPEC-11 \[FREEZE\]: *"Canonical content addressing uses a STRONG hash
//! (SHA-256 / BLAKE3 multihash), because Git's default SHA-1 object id is
//! collision-broken."* SPEC-01 §3 echoes this for the `did:mini` identifier.
//!
//! This invariant is enforced **structurally**, not by convention: the
//! [`HashAlgorithm`] enum has no SHA-1 variant, so no caller can produce a SHA-1
//! content address through this API. The corresponding multihash code (`0x11`) is
//! likewise rejected on decode (see [`crate::multihash`]).

use blake3::Hasher as Blake3Hasher;
use sha2::{Digest, Sha256};

/// The set of hash algorithms permitted for canonical content addressing.
///
/// Deliberately small. There is **no** `Sha1` variant and there never will be:
/// SHA-1 is collision-broken and forbidden by SPEC-11's frozen hash-hardening
/// rule. Adding a weak algorithm here would be a constitution-level regression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// BLAKE3, 256-bit output. Default for new content addresses.
    Blake3,
    /// SHA2-256, 256-bit output. Retained for Git-object interop (SPEC-11's
    /// SHA-256 git-object plan) and broad ecosystem compatibility.
    Sha256,
}

impl HashAlgorithm {
    /// The unsigned-varint multihash code for this algorithm.
    ///
    /// `0x1e` = blake3, `0x12` = sha2-256 (per the multicodec table).
    pub const fn multihash_code(self) -> u64 {
        match self {
            HashAlgorithm::Blake3 => 0x1e,
            HashAlgorithm::Sha256 => 0x12,
        }
    }

    /// Digest length in bytes (both supported algorithms emit 32 bytes here).
    pub const fn digest_len(self) -> usize {
        32
    }

    /// Hash `data` with this algorithm, returning the raw 32-byte digest.
    pub fn digest(self, data: &[u8]) -> [u8; 32] {
        match self {
            HashAlgorithm::Blake3 => {
                let mut h = Blake3Hasher::new();
                h.update(data);
                *h.finalize().as_bytes()
            }
            HashAlgorithm::Sha256 => {
                let mut h = Sha256::new();
                h.update(data);
                let out = h.finalize();
                let mut digest = [0u8; 32];
                digest.copy_from_slice(&out);
                digest
            }
        }
    }
}

impl HashAlgorithm {
    /// Begin an incremental hash under this algorithm — for content too
    /// large to hold in memory as one buffer, where [`Self::digest`] would
    /// require exactly that. Produces the identical digest [`Self::digest`]
    /// would over the same bytes, however the caller splits `update` calls
    /// (both algorithms' underlying constructions are streaming ciphers over
    /// a running state, not fixed-block one-shot functions).
    pub fn incremental(self) -> IncrementalHash {
        IncrementalHash {
            inner: match self {
                HashAlgorithm::Blake3 => Incremental::Blake3(Box::new(Blake3Hasher::new())),
                HashAlgorithm::Sha256 => Incremental::Sha256(Sha256::new()),
            },
        }
    }
}

enum Incremental {
    Blake3(Box<Blake3Hasher>),
    Sha256(Sha256),
}

/// A hash in progress, fed via [`Self::update`] and closed via
/// [`Self::finalize`]. See [`HashAlgorithm::incremental`].
pub struct IncrementalHash {
    inner: Incremental,
}

impl core::fmt::Debug for IncrementalHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let algorithm = match &self.inner {
            Incremental::Blake3(_) => "Blake3",
            Incremental::Sha256(_) => "Sha256",
        };
        f.debug_struct("IncrementalHash")
            .field("algorithm", &algorithm)
            .finish()
    }
}

impl IncrementalHash {
    /// Feed more bytes into the hash. Call any number of times, in any
    /// chunking, before [`Self::finalize`].
    pub fn update(&mut self, data: &[u8]) {
        match &mut self.inner {
            Incremental::Blake3(h) => {
                h.update(data);
            }
            Incremental::Sha256(h) => {
                h.update(data);
            }
        }
    }

    /// Close the hash and return its 32-byte digest.
    pub fn finalize(self) -> [u8; 32] {
        match self.inner {
            Incremental::Blake3(h) => *h.finalize().as_bytes(),
            Incremental::Sha256(h) => {
                let out = h.finalize();
                let mut digest = [0u8; 32];
                digest.copy_from_slice(&out);
                digest
            }
        }
    }
}

/// The default algorithm for *new* Mininet content addresses.
pub const DEFAULT_HASH: HashAlgorithm = HashAlgorithm::Blake3;

/// Convenience: BLAKE3-256 digest of `data`.
pub fn blake3_256(data: &[u8]) -> [u8; 32] {
    HashAlgorithm::Blake3.digest(data)
}

/// Convenience: SHA2-256 digest of `data`.
pub fn sha2_256(data: &[u8]) -> [u8; 32] {
    HashAlgorithm::Sha256.digest(data)
}

/// The multihash code for the (forbidden) SHA-1 algorithm.
///
/// Exposed only so the decoder and tests can explicitly **reject** it. There is
/// no code path in this crate that hashes with SHA-1.
pub const FORBIDDEN_SHA1_CODE: u64 = 0x11;

#[cfg(test)]
mod incremental_tests {
    use super::*;

    fn data(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    #[test]
    fn a_single_update_matches_one_shot_digest_for_every_algorithm() {
        for algo in [HashAlgorithm::Blake3, HashAlgorithm::Sha256] {
            let bytes = data(5_000);
            let mut incremental = algo.incremental();
            incremental.update(&bytes);
            assert_eq!(incremental.finalize(), algo.digest(&bytes));
        }
    }

    #[test]
    fn splitting_updates_anywhere_never_changes_the_result() {
        for algo in [HashAlgorithm::Blake3, HashAlgorithm::Sha256] {
            let bytes = data(10_007); // not a multiple of any obvious chunk size
            let expected = algo.digest(&bytes);
            for split_at in [0, 1, 1024, 4096, bytes.len() - 1, bytes.len()] {
                let mut incremental = algo.incremental();
                incremental.update(&bytes[..split_at]);
                incremental.update(&bytes[split_at..]);
                assert_eq!(
                    incremental.finalize(),
                    expected,
                    "algo {algo:?} split at {split_at}"
                );
            }
        }
    }

    #[test]
    fn many_small_updates_match_one_large_update() {
        for algo in [HashAlgorithm::Blake3, HashAlgorithm::Sha256] {
            let bytes = data(9_999);
            let mut incremental = algo.incremental();
            for chunk in bytes.chunks(7) {
                incremental.update(chunk);
            }
            assert_eq!(incremental.finalize(), algo.digest(&bytes));
        }
    }

    #[test]
    fn an_empty_hash_matches_the_one_shot_empty_digest() {
        for algo in [HashAlgorithm::Blake3, HashAlgorithm::Sha256] {
            let incremental = algo.incremental();
            assert_eq!(incremental.finalize(), algo.digest(&[]));
        }
    }
}
