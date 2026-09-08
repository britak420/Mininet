//! A real, on-disk [`ClaimedRegistry`] backed by one atomically-published
//! marker file per identity root, so [`FileClaimedRegistry::try_reserve`]
//! is a genuine atomic reservation rather than an in-memory check followed
//! by a separate write (PR #327 finding F-20). A crash immediately after
//! a successful reservation still leaves it durably recorded before any
//! caller goes on to trigger a real payout.
//!
//! Publishing is write-temp-then-`hard_link`, not a direct
//! `create_new` on the final path. An earlier version of this file did
//! use a direct exclusive create, and a real concurrency test (below)
//! caught the bug that shape has: `create_new` makes the *name* appear
//! atomically, but the file's *content* is written in a separate step
//! after that, so a second caller racing the first can observe the name
//! existing with zero or partial bytes and misread that as a corrupt
//! record instead of a legitimate prior claim. Writing the full record to
//! a private, uniquely-named temp file first (fsynced before it is ever
//! linked) and then publishing it at the final name with [`fs::hard_link`]
//! -- which fails atomically with `AlreadyExists` if a reservation is
//! already there -- means the final path never exists with anything but
//! complete content; there is no window left to observe.
//!
//! Every read (`already_claimed`, and the existing-record comparison
//! inside `try_reserve`) goes straight to disk rather than an in-process
//! cache -- the filesystem is the single source of truth, so two
//! `FileClaimedRegistry` instances over the same directory (two threads,
//! or two separate processes) agree immediately, not only after one
//! reopens.

use std::collections::hash_map::DefaultHasher;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use did_mini::Did;

use crate::error::{AirdropError, Result};
use crate::registry::{ClaimedRegistry, ReservationOutcome};

const RECORD_DOMAIN: &[u8] = b"mini-airdrop/claim-reservation-record/v1";
/// Fixed on-disk record size: domain tag + 32-byte outcome digest +
/// 8-byte big-endian `at_ms`. Any file at a reservation path that is not
/// exactly this size is corrupt, not a format this crate ever wrote.
const RECORD_BYTES: usize = RECORD_DOMAIN.len() + 32 + 8;

/// A [`ClaimedRegistry`] backed by one file per identity root inside a
/// directory, named by a content hash of the root's scid (never the raw
/// scid text) so no identity string ever has to be a safe filename on
/// every target filesystem.
#[derive(Debug)]
pub struct FileClaimedRegistry {
    dir: PathBuf,
}

impl FileClaimedRegistry {
    /// Open (or create) the registry directory at `dir`. Unlike the old
    /// single-append-log design, nothing is preloaded into memory --
    /// every subsequent call reads the specific file it needs directly,
    /// which is what makes cross-instance agreement immediate rather than
    /// reopen-to-see.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir).map_err(|e| AirdropError::RegistryWriteFailed(e.to_string()))?;
        Ok(FileClaimedRegistry { dir })
    }

    /// How many claims are currently on record. Always a fresh directory
    /// listing, never a cache.
    pub fn len(&self) -> usize {
        fs::read_dir(&self.dir)
            .map(|entries| entries.filter_map(std::result::Result::ok).count())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A filesystem-safe, collision-resistant filename for `identity_root`
    /// that never embeds the raw scid text. `DefaultHasher` is fine here:
    /// this is a directory-entry name picked for filesystem safety, not a
    /// security boundary -- the actual claim authorization is the KEL
    /// signature `crate::claim::verify_and_resolve_claim` already checked
    /// before ever calling into this registry.
    fn reservation_path(&self, identity_root: &Did) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        identity_root.as_str().hash(&mut hasher);
        self.dir.join(format!("{:016x}.claim", hasher.finish()))
    }

    /// A private, per-attempt staging path in the same directory (so
    /// [`fs::hard_link`] to the final [`Self::reservation_path`] stays a
    /// same-filesystem, atomic operation). Uniqueness only needs to avoid
    /// two concurrent attempts colliding with each other, not resist a
    /// guessing adversary -- the exclusivity that actually matters is
    /// `hard_link`'s onto the final name.
    fn staging_path(&self, identity_root: &Did) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        identity_root.as_str().hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        self.dir.join(format!(
            "{:016x}.claim.tmp.{}.{}",
            hasher.finish(),
            std::process::id(),
            nanos
        ))
    }

    fn encode_record(outcome_digest: [u8; 32], at_ms: u64) -> Vec<u8> {
        let mut out = Vec::with_capacity(RECORD_BYTES);
        out.extend_from_slice(RECORD_DOMAIN);
        out.extend_from_slice(&outcome_digest);
        out.extend_from_slice(&at_ms.to_be_bytes());
        out
    }

    /// Read back an existing reservation record, or `None` if the path
    /// simply does not exist. `Err` means the path exists but this crate
    /// cannot trust what is in it (wrong size, wrong domain tag) -- a
    /// truncated-mid-write crash artifact, and this fails closed rather
    /// than guessing whether it represents a real prior claim.
    fn read_record(path: &Path) -> Result<Option<([u8; 32], u64)>> {
        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(AirdropError::RegistryWriteFailed(e.to_string())),
        };
        let mut buf = Vec::with_capacity(RECORD_BYTES);
        file.read_to_end(&mut buf)
            .map_err(|e| AirdropError::RegistryWriteFailed(e.to_string()))?;
        if buf.len() != RECORD_BYTES || &buf[..RECORD_DOMAIN.len()] != RECORD_DOMAIN {
            return Err(AirdropError::CorruptReservationRecord);
        }
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&buf[RECORD_DOMAIN.len()..RECORD_DOMAIN.len() + 32]);
        let mut at_bytes = [0u8; 8];
        at_bytes.copy_from_slice(&buf[RECORD_DOMAIN.len() + 32..]);
        Ok(Some((digest, u64::from_be_bytes(at_bytes))))
    }
}

impl ClaimedRegistry for FileClaimedRegistry {
    fn already_claimed(&self, identity_root: &Did) -> bool {
        self.reservation_path(identity_root).is_file()
    }

    fn try_reserve(
        &mut self,
        identity_root: &Did,
        outcome_digest: [u8; 32],
        at_ms: u64,
    ) -> Result<ReservationOutcome> {
        let path = self.reservation_path(identity_root);

        // Fast path: a plain read already tells us the common cases
        // (idempotent retry, genuine conflict) without writing anything.
        // This is only an optimization -- the exclusivity guarantee below
        // does not depend on this read having seen the true current state.
        if let Some((existing_digest, _)) = Self::read_record(&path)? {
            return if existing_digest == outcome_digest {
                Ok(ReservationOutcome::IdempotentRetry)
            } else {
                Err(AirdropError::AlreadyClaimed)
            };
        }

        // Write the complete record to a private staging file first --
        // fully fsynced before it is ever visible at the final name.
        let staging = self.staging_path(identity_root);
        let record = Self::encode_record(outcome_digest, at_ms);
        let write_result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .and_then(|mut file| file.write_all(&record).and_then(|()| file.sync_all()));
        if let Err(e) = write_result {
            let _ = fs::remove_file(&staging);
            return Err(AirdropError::RegistryWriteFailed(e.to_string()));
        }

        // Publish atomically: hard_link fails with AlreadyExists if a
        // reservation is already at `path`, and otherwise the link (and
        // therefore `path`'s complete content) becomes visible in one
        // filesystem operation -- no partial-content window.
        let publish_result = fs::hard_link(&staging, &path);
        let _ = fs::remove_file(&staging);

        match publish_result {
            Ok(()) => Ok(ReservationOutcome::Fresh),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => match Self::read_record(&path)? {
                Some((existing_digest, _)) if existing_digest == outcome_digest => {
                    Ok(ReservationOutcome::IdempotentRetry)
                }
                Some(_) => Err(AirdropError::AlreadyClaimed),
                None => Err(AirdropError::RegistryWriteFailed(
                    "reservation marker vanished during read-back".to_string(),
                )),
            },
            Err(e) => Err(AirdropError::RegistryWriteFailed(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::outcome_digest;
    use crate::claim::ClaimOutcome;
    use did_mini::Controller;

    fn root() -> Did {
        Controller::incept_single().unwrap().did()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mini-airdrop-test-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    fn digest_for(root: &Did, amount_micro: u64, recipient: &[u8]) -> [u8; 32] {
        outcome_digest(&ClaimOutcome {
            identity_root: root.clone(),
            amount_micro,
            recipient: recipient.to_vec(),
        })
    }

    #[test]
    fn a_fresh_registry_has_no_claims() {
        let dir = temp_dir("fresh");
        let registry = FileClaimedRegistry::open(&dir).unwrap();
        assert!(registry.is_empty());
        assert!(!registry.already_claimed(&root()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_reservation_persists_across_reopening_the_same_directory() {
        let dir = temp_dir("reopen");
        let r = root();
        let digest = digest_for(&r, 1_000, b"payee");

        {
            let mut registry = FileClaimedRegistry::open(&dir).unwrap();
            assert_eq!(
                registry.try_reserve(&r, digest, 1_000).unwrap(),
                ReservationOutcome::Fresh
            );
        }

        let reopened = FileClaimedRegistry::open(&dir).unwrap();
        assert!(reopened.already_claimed(&r));
        assert_eq!(reopened.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn multiple_claims_all_survive_a_reopen() {
        let dir = temp_dir("multi");
        let a = root();
        let b = root();

        {
            let mut registry = FileClaimedRegistry::open(&dir).unwrap();
            registry
                .try_reserve(&a, digest_for(&a, 100, b"pa"), 100)
                .unwrap();
            registry
                .try_reserve(&b, digest_for(&b, 200, b"pb"), 200)
                .unwrap();
        }

        let reopened = FileClaimedRegistry::open(&dir).unwrap();
        assert!(reopened.already_claimed(&a));
        assert!(reopened.already_claimed(&b));
        assert_eq!(reopened.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_second_reservation_for_a_different_outcome_is_refused() {
        let dir = temp_dir("conflict");
        let r = root();
        let mut registry = FileClaimedRegistry::open(&dir).unwrap();

        registry
            .try_reserve(&r, digest_for(&r, 1_000, b"honest-payee"), 100)
            .unwrap();

        // Same identity root, but a different resolved outcome (e.g. an
        // attacker trying to redirect an already-claimed entitlement) --
        // this is a real conflicting claim, not a retry.
        let conflicting = registry.try_reserve(&r, digest_for(&r, 1_000, b"attacker-payee"), 200);
        assert_eq!(conflicting.unwrap_err(), AirdropError::AlreadyClaimed);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_retry_of_the_exact_same_outcome_is_idempotent_not_an_error() {
        // The finding's own concrete example: a valid claimant is
        // reserved, then whatever comes after (signing, submission)
        // fails. A retry with the identical resolved outcome must not be
        // refused -- no funds moved, so this cannot read as a second
        // award, but it also must not permanently strand the claimant.
        let dir = temp_dir("idempotent");
        let r = root();
        let digest = digest_for(&r, 1_000, b"payee");
        let mut registry = FileClaimedRegistry::open(&dir).unwrap();

        assert_eq!(
            registry.try_reserve(&r, digest, 100).unwrap(),
            ReservationOutcome::Fresh
        );
        assert_eq!(
            registry.try_reserve(&r, digest, 999).unwrap(),
            ReservationOutcome::IdempotentRetry
        );
        // A third retry, from a completely fresh registry instance over
        // the same directory, still agrees.
        let mut reopened = FileClaimedRegistry::open(&dir).unwrap();
        assert_eq!(
            reopened.try_reserve(&r, digest, 12345).unwrap(),
            ReservationOutcome::IdempotentRetry
        );
        assert_eq!(reopened.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_writers_racing_the_same_identity_root_never_both_win_fresh() {
        // The finding's own concrete example: "two writers both pass the
        // initial unclaimed check before either persists." Two real OS
        // threads, each with its own `FileClaimedRegistry` instance
        // (standing in for two separate processes with no shared
        // in-memory state), race to reserve the same identity root for
        // two *different* outcomes, synchronized to start together.
        // Exactly one may win Fresh; the other must be refused as a
        // conflicting claim -- never both silently succeeding, and never
        // a race-dependent flake either way.
        use std::sync::{Arc, Barrier};
        use std::thread;

        let dir = temp_dir("race");
        let r = root();
        let barrier = Arc::new(Barrier::new(2));

        let run = |payee: &'static [u8], barrier: Arc<Barrier>, dir: PathBuf, r: Did| {
            thread::spawn(move || {
                let mut registry = FileClaimedRegistry::open(&dir).unwrap();
                barrier.wait();
                registry.try_reserve(&r, digest_for(&r, 1_000, payee), 100)
            })
        };

        let t1 = run(b"payee-a", Arc::clone(&barrier), dir.clone(), r.clone());
        let t2 = run(b"payee-b", Arc::clone(&barrier), dir.clone(), r.clone());
        let first_result = t1.join().unwrap();
        let second_result = t2.join().unwrap();

        let outcomes = [first_result.is_ok(), second_result.is_ok()];
        assert_eq!(
            outcomes.iter().filter(|ok| **ok).count(),
            1,
            "exactly one of the two racing reservations must win: {first_result:?} / {second_result:?}"
        );
        let loser = if first_result.is_err() {
            first_result
        } else {
            second_result
        };
        assert_eq!(loser.unwrap_err(), AirdropError::AlreadyClaimed);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_truncated_reservation_record_is_refused_not_silently_trusted() {
        let dir = temp_dir("truncated");
        let r = root();
        let mut registry = FileClaimedRegistry::open(&dir).unwrap();
        registry
            .try_reserve(&r, digest_for(&r, 1_000, b"payee"), 100)
            .unwrap();

        // Simulate a crash mid-write: chop bytes off the marker file
        // directly, bypassing this crate's own writer.
        let path = registry.reservation_path(&r);
        let mut bytes = fs::read(&path).unwrap();
        bytes.truncate(bytes.len() - 3);
        fs::write(&path, &bytes).unwrap();

        assert!(registry.already_claimed(&r));
        let retry = registry.try_reserve(&r, digest_for(&r, 1_000, b"payee"), 200);
        assert_eq!(retry.unwrap_err(), AirdropError::CorruptReservationRecord);
        let _ = fs::remove_dir_all(&dir);
    }
}
