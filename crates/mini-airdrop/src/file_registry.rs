//! Campaign-bound, cross-process claim reservations. All successful writes
//! include file and directory durability barriers. Local disk rollback still
//! needs an independently retained canonical payout history; this is bookkeeping.

use crate::error::{AirdropError, Result};
use crate::registry::{ClaimedRegistry, ReservationOutcome};
use did_mini::Did;
use mini_crypto::hash::blake3_256;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

const RECORD_DOMAIN: &[u8] = b"mini-airdrop/claim-reservation-record/v2";
const KEY_DOMAIN: &[u8] = b"mini-airdrop/claim-reservation-key/v2";
const MAX_RECORD_BYTES: u64 = 2048;

fn disk(error: std::io::Error) -> AirdropError {
    AirdropError::RegistryWriteFailed(error.to_string())
}

#[derive(Debug)]
pub struct FileClaimedRegistry {
    dir: PathBuf,
}

impl FileClaimedRegistry {
    /// Open a v2 registry. Legacy 64-bit filenames cannot establish campaign
    /// or identity binding: refuse them for explicit reconciliation/migration.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        mini_durable::create_dir_all(&dir).map_err(disk)?;
        let registry = Self { dir };
        let _lock =
            mini_durable::lock_exclusive(&registry.dir.join(".registry.lock")).map_err(disk)?;
        for entry in fs::read_dir(&registry.dir).map_err(disk)? {
            let entry = entry.map_err(disk)?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".claim") && !Self::claim_name(&name) {
                return Err(AirdropError::CorruptReservationRecord);
            }
        }
        Ok(registry)
    }

    fn claim_name(name: &str) -> bool {
        name.len() == 70
            && name.ends_with(".claim")
            && name.as_bytes()[..64].iter().all(u8::is_ascii_hexdigit)
    }

    /// Count only complete reservation names, never locks or crash temp files.
    /// Directory errors propagate rather than reporting an empty campaign.
    pub fn len(&self) -> Result<usize> {
        let mut count = 0;
        for entry in fs::read_dir(&self.dir).map_err(disk)? {
            if Self::claim_name(&entry.map_err(disk)?.file_name().to_string_lossy()) {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    fn binding(campaign: &[u8], identity: &Did) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(campaign.len() as u64).to_be_bytes());
        bytes.extend_from_slice(campaign);
        bytes.extend_from_slice(&(identity.as_str().len() as u64).to_be_bytes());
        bytes.extend_from_slice(identity.as_str().as_bytes());
        bytes
    }

    fn reservation_path(&self, campaign: &[u8], identity: &Did) -> PathBuf {
        let mut input = KEY_DOMAIN.to_vec();
        input.extend(Self::binding(campaign, identity));
        let hash = blake3_256(&input);
        let name: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        self.dir.join(format!("{name}.claim"))
    }

    fn encode_record(campaign: &[u8], identity: &Did, digest: [u8; 32], at_ms: u64) -> Vec<u8> {
        let mut out = RECORD_DOMAIN.to_vec();
        out.extend(Self::binding(campaign, identity));
        out.extend_from_slice(&digest);
        out.extend_from_slice(&at_ms.to_be_bytes());
        let checksum = blake3_256(&out);
        out.extend_from_slice(&checksum);
        out
    }

    fn read_record(
        path: &Path,
        campaign: &[u8],
        identity: &Did,
    ) -> Result<Option<([u8; 32], u64)>> {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(disk(e)),
        };
        let mut bytes = Vec::new();
        file.take(MAX_RECORD_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(disk)?;
        let mut prefix = RECORD_DOMAIN.to_vec();
        prefix.extend(Self::binding(campaign, identity));
        let end = prefix.len() + 40;
        if bytes.len() != end + 32
            || bytes.len() as u64 > MAX_RECORD_BYTES
            || !bytes.starts_with(&prefix)
            || blake3_256(&bytes[..end]) != bytes[end..]
        {
            return Err(AirdropError::CorruptReservationRecord);
        }
        let digest = bytes[prefix.len()..prefix.len() + 32].try_into().unwrap();
        let at_ms = u64::from_be_bytes(bytes[prefix.len() + 32..end].try_into().unwrap());
        Ok(Some((digest, at_ms)))
    }
}

impl ClaimedRegistry for FileClaimedRegistry {
    /// Advisory only; ambiguity is treated as reserved. Authorization always
    /// uses the fallible, locked `try_reserve` operation below.
    fn already_claimed(&self, campaign: &[u8], identity: &Did) -> bool {
        !matches!(
            Self::read_record(
                &self.reservation_path(campaign, identity),
                campaign,
                identity
            ),
            Ok(None)
        )
    }

    fn try_reserve(
        &mut self,
        campaign: &[u8],
        identity: &Did,
        digest: [u8; 32],
        at_ms: u64,
    ) -> Result<ReservationOutcome> {
        if campaign.len() > crate::snapshot::MAX_CAMPAIGN_ID_BYTES {
            return Err(AirdropError::CampaignIdTooLong);
        }
        let _lock = mini_durable::lock_exclusive(&self.dir.join(".registry.lock")).map_err(disk)?;
        let path = self.reservation_path(campaign, identity);
        if let Some((existing, _)) = Self::read_record(&path, campaign, identity)? {
            if existing != digest {
                return Err(AirdropError::AlreadyClaimed);
            }
            // A previous attempt may have failed after rename but before its
            // directory barrier. Never let an idempotent retry skip durability.
            fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .and_then(|f| f.sync_all())
                .map_err(disk)?;
            mini_durable::sync_parent(&path).map_err(disk)?;
            return Ok(ReservationOutcome::IdempotentRetry);
        }
        mini_durable::atomic_replace(
            &path,
            &Self::encode_record(campaign, identity, digest, at_ms),
        )
        .map_err(disk)?;
        Ok(ReservationOutcome::Fresh)
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
    fn campaigns_are_independent_and_records_cannot_be_transplanted() {
        let dir = temp_dir("binding");
        let a = root();
        let b = root();
        let digest = digest_for(&a, 10, b"payee");
        let mut registry = FileClaimedRegistry::open(&dir).unwrap();
        assert_eq!(
            registry.try_reserve(b"a", &a, digest, 1).unwrap(),
            ReservationOutcome::Fresh
        );
        assert_eq!(
            registry.try_reserve(b"b", &a, digest, 1).unwrap(),
            ReservationOutcome::Fresh
        );
        let original = registry.reservation_path(b"a", &a);
        assert_eq!(original.file_name().unwrap().len(), 70);
        fs::copy(&original, registry.reservation_path(b"a", &b)).unwrap();
        assert_eq!(
            registry.try_reserve(b"a", &b, digest, 2).unwrap_err(),
            AirdropError::CorruptReservationRecord
        );
        fs::copy(&original, registry.reservation_path(b"c", &a)).unwrap();
        assert_eq!(
            registry.try_reserve(b"c", &a, digest, 2).unwrap_err(),
            AirdropError::CorruptReservationRecord
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn legacy_records_fail_closed_and_temporary_files_are_not_claims() {
        let dir = temp_dir("legacy");
        let registry = FileClaimedRegistry::open(&dir).unwrap();
        fs::write(dir.join("ab.claim.tmp.crash"), b"partial").unwrap();
        assert_eq!(registry.len().unwrap(), 0);
        fs::write(dir.join("0123456789abcdef.claim"), b"legacy").unwrap();
        assert!(matches!(
            FileClaimedRegistry::open(&dir),
            Err(AirdropError::CorruptReservationRecord)
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn oversized_or_checksum_corrupted_records_are_refused() {
        let dir = temp_dir("integrity");
        let a = root();
        let digest = digest_for(&a, 10, b"payee");
        let mut registry = FileClaimedRegistry::open(&dir).unwrap();
        registry.try_reserve(b"a", &a, digest, 1).unwrap();
        let path = registry.reservation_path(b"a", &a);
        let mut bytes = fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::write(&path, bytes).unwrap();
        assert_eq!(
            registry.try_reserve(b"a", &a, digest, 2).unwrap_err(),
            AirdropError::CorruptReservationRecord
        );
        fs::write(&path, vec![1; MAX_RECORD_BYTES as usize + 1]).unwrap();
        assert_eq!(
            registry.try_reserve(b"a", &a, digest, 2).unwrap_err(),
            AirdropError::CorruptReservationRecord
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_fresh_registry_has_no_claims() {
        let dir = temp_dir("fresh");
        let registry = FileClaimedRegistry::open(&dir).unwrap();
        assert!(registry.is_empty().unwrap());
        assert!(!registry.already_claimed(b"campaign-1", &root()));
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
                registry
                    .try_reserve(b"campaign-1", &r, digest, 1_000)
                    .unwrap(),
                ReservationOutcome::Fresh
            );
        }

        let reopened = FileClaimedRegistry::open(&dir).unwrap();
        assert!(reopened.already_claimed(b"campaign-1", &r));
        assert_eq!(reopened.len().unwrap(), 1);
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
                .try_reserve(b"campaign-1", &a, digest_for(&a, 100, b"pa"), 100)
                .unwrap();
            registry
                .try_reserve(b"campaign-1", &b, digest_for(&b, 200, b"pb"), 200)
                .unwrap();
        }

        let reopened = FileClaimedRegistry::open(&dir).unwrap();
        assert!(reopened.already_claimed(b"campaign-1", &a));
        assert!(reopened.already_claimed(b"campaign-1", &b));
        assert_eq!(reopened.len().unwrap(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_second_reservation_for_a_different_outcome_is_refused() {
        let dir = temp_dir("conflict");
        let r = root();
        let mut registry = FileClaimedRegistry::open(&dir).unwrap();

        registry
            .try_reserve(
                b"campaign-1",
                &r,
                digest_for(&r, 1_000, b"honest-payee"),
                100,
            )
            .unwrap();

        // Same identity root, but a different resolved outcome (e.g. an
        // attacker trying to redirect an already-claimed entitlement) --
        // this is a real conflicting claim, not a retry.
        let conflicting = registry.try_reserve(
            b"campaign-1",
            &r,
            digest_for(&r, 1_000, b"attacker-payee"),
            200,
        );
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
            registry
                .try_reserve(b"campaign-1", &r, digest, 100)
                .unwrap(),
            ReservationOutcome::Fresh
        );
        assert_eq!(
            registry
                .try_reserve(b"campaign-1", &r, digest, 999)
                .unwrap(),
            ReservationOutcome::IdempotentRetry
        );
        // A third retry, from a completely fresh registry instance over
        // the same directory, still agrees.
        let mut reopened = FileClaimedRegistry::open(&dir).unwrap();
        assert_eq!(
            reopened
                .try_reserve(b"campaign-1", &r, digest, 12345)
                .unwrap(),
            ReservationOutcome::IdempotentRetry
        );
        assert_eq!(reopened.len().unwrap(), 1);
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
                registry.try_reserve(b"campaign-1", &r, digest_for(&r, 1_000, payee), 100)
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
            .try_reserve(b"campaign-1", &r, digest_for(&r, 1_000, b"payee"), 100)
            .unwrap();

        // Simulate a crash mid-write: chop bytes off the marker file
        // directly, bypassing this crate's own writer.
        let path = registry.reservation_path(b"campaign-1", &r);
        let mut bytes = fs::read(&path).unwrap();
        bytes.truncate(bytes.len() - 3);
        fs::write(&path, &bytes).unwrap();

        assert!(registry.already_claimed(b"campaign-1", &r));
        let retry = registry.try_reserve(b"campaign-1", &r, digest_for(&r, 1_000, b"payee"), 200);
        assert_eq!(retry.unwrap_err(), AirdropError::CorruptReservationRecord);
        let _ = fs::remove_dir_all(&dir);
    }
}
