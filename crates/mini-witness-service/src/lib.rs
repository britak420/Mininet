//! A durable, crash-recoverable backing store for `did_mini::WitnessJournal`
//! — design doc Phase 6 ("persistent witness service: durable state, crash
//! recovery, bounded retention, quotas"), following Phase 4's receipt
//! collection protocol (D-0464) and the phases before it.
//!
//! ## Why this is a separate crate, not a `did-mini` module
//!
//! `did-mini`'s own `Cargo.toml` states its scope deliberately: "this crate
//! is security-critical and must stay easy to review and reproduce...
//! It has NO network or chain dependency." Filesystem persistence is a new
//! category of capability that crate has never carried, and this workspace
//! already has a standing pattern for exactly this split — a pure, in-memory
//! state machine in one crate, anchored to a real process/store in another:
//! `mini-chain`'s finality math anchored by `mini-consensus`, `mini-update`'s
//! freshness policy anchored by `mini-installer`. This crate is that anchor
//! for `did_mini::WitnessJournal`.
//!
//! ## Why this never re-implements the state machine
//!
//! [`PersistentWitnessJournal`] persists only what is needed to reconstruct
//! *exactly* the same decision on replay: the accepted KEL and the
//! observation epoch used at the time. On replay it derives the witness
//! policy the same way every call does — from the KEL's own most recent
//! establishment event via [`did_mini::Kel::declared_witness_policy`], per
//! D-0459 — and hands both to [`did_mini::WitnessJournal::observe_verified`]
//! unchanged. Crash recovery is "replay the same pure functions over what
//! was durably recorded," never a bespoke restore path with its own trust
//! logic — the same discipline this tree already applies to rebuilding
//! execution state from a snapshot rather than trusting a persisted derived
//! value. Because Ed25519 signing is deterministic, replaying an acceptance
//! reproduces the exact same [`did_mini::WitnessReceipt`] bytes a requester
//! who received it before a restart still holds — proven by this crate's
//! own round-trip test, not assumed.
//!
//! A durable record is written *before* [`PersistentWitnessJournal::observe_declared`]
//! returns an `Accepted` outcome to its caller, not after — so a caller can
//! never observe (and hand a requester) a receipt this store has not yet
//! made durable. What this crate does **not** guarantee: an `fsync` barrier
//! against power loss mid-write. `fs::write` then `fs::rename` is atomic
//! against a *killed process* (the old, complete file is never partially
//! overwritten in place), the same guarantee most user-space atomic-replace
//! patterns settle for, but it is not a claim of durability across an
//! OS-level crash or power failure — stated here rather than left implicit.
//!
//! ## What this does not do
//!
//! No network transport — carrying [`did_mini::witness_protocol`]'s
//! messages over a real socket is separate, later work for whichever crate
//! first runs a witness service, the same way `mini-consensus::discovery`
//! is a separate adapter over `mini-net::pex`. No gossip (Phase 5). No
//! witness-rotation-aware pruning (Phase 7): retiring an old record is a
//! host operation this crate does not perform on its own. Bounded by
//! identity *count* only ([`MAX_TRACKED_IDENTITIES`]); nothing here bounds
//! disk space per identity beyond one KEL's own existing size cap.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use did_mini::{Did, IdentityError, Kel, WitnessId, WitnessIdentityState};
use did_mini::{WitnessJournal, WitnessObservation};
use mini_crypto::{HashAlgorithm, SigningKey};

/// Hard cap on distinct identities one persistent journal will track,
/// applied before accepting a *new* identity — an allocation/disk bound,
/// never revisited for an identity this journal already tracks. Use
/// [`PersistentWitnessJournal::open_with_capacity`] to set a different
/// bound (tests use a small one to exercise this path cheaply).
pub const MAX_TRACKED_IDENTITIES: usize = 100_000;

/// Domain tag for one persisted record, so a file from a future incompatible
/// format is refused rather than misread.
const RECORD_DOMAIN: &[u8] = b"mini-witness-service/record/v1";

/// Errors this crate's persistence layer can produce, wrapping `did-mini`'s
/// own errors for the state-machine replay it performs.
#[derive(Debug)]
#[non_exhaustive]
pub enum WitnessServiceError {
    /// A filesystem operation failed.
    Io(std::io::Error),
    /// `did-mini`'s own state machine rejected an observation or a
    /// persisted KEL failed to decode/verify on replay.
    Identity(IdentityError),
    /// A persisted record's bytes were truncated, wrong-domain, or carried
    /// trailing bytes.
    CorruptRecord,
    /// Accepting a new identity would exceed this journal's configured
    /// capacity.
    TooManyIdentities,
}

impl core::fmt::Display for WitnessServiceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WitnessServiceError::Io(e) => write!(f, "witness service storage: {e}"),
            WitnessServiceError::Identity(e) => write!(f, "witness service: {e}"),
            WitnessServiceError::CorruptRecord => {
                write!(f, "witness service: persisted record is corrupt")
            }
            WitnessServiceError::TooManyIdentities => {
                write!(f, "witness service: identity capacity exceeded")
            }
        }
    }
}

impl std::error::Error for WitnessServiceError {}

impl From<std::io::Error> for WitnessServiceError {
    fn from(e: std::io::Error) -> Self {
        WitnessServiceError::Io(e)
    }
}

impl From<IdentityError> for WitnessServiceError {
    fn from(e: IdentityError) -> Self {
        WitnessServiceError::Identity(e)
    }
}

/// Result alias for this crate.
pub type Result<T> = core::result::Result<T, WitnessServiceError>;

/// A [`did_mini::WitnessJournal`] whose accepted events survive a restart.
///
/// Every accepted observation is durably recorded before this type reports
/// it to its own caller. On [`Self::open`], every previously-recorded
/// identity is replayed through its own declared witness policy and
/// [`did_mini::WitnessJournal::observe_verified`] to rebuild exactly the
/// in-memory state (and receipts) a process that never stopped would still
/// hold.
#[derive(Debug)]
pub struct PersistentWitnessJournal {
    root: PathBuf,
    journal: WitnessJournal,
    known: HashSet<Did>,
    max_identities: usize,
}

impl PersistentWitnessJournal {
    /// Open (or create) a persistent witness journal rooted at `root`, with
    /// the default [`MAX_TRACKED_IDENTITIES`] capacity.
    pub fn open(
        root: impl Into<PathBuf>,
        witness_id: WitnessId,
        witness_key: &SigningKey,
    ) -> Result<Self> {
        Self::open_with_capacity(root, witness_id, witness_key, MAX_TRACKED_IDENTITIES)
    }

    /// Like [`Self::open`], with an explicit identity capacity.
    pub fn open_with_capacity(
        root: impl Into<PathBuf>,
        witness_id: WitnessId,
        witness_key: &SigningKey,
        max_identities: usize,
    ) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        let mut journal = WitnessJournal::new();
        let mut known = HashSet::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let path = entry.path();
            // A `.tmp` file is a write this process (or a prior one) never
            // finished — see `persist`'s write-then-rename discipline. The
            // previous, complete `.state` file (if any) is untouched and is
            // what recovery must use instead; the stray temp file is simply
            // ignored, never treated as a record.
            if path.extension().and_then(|e| e.to_str()) != Some("state") {
                continue;
            }
            let bytes = fs::read(&path)?;
            let (kel_bytes, observed_epoch) = decode_record(&bytes)?;
            let kel = Kel::from_bytes(&kel_bytes)?;
            let identity = kel.did();
            observe_declared_on(
                &mut journal,
                &kel,
                witness_id.clone(),
                witness_key,
                observed_epoch,
            )?;
            known.insert(identity);
        }
        Ok(PersistentWitnessJournal {
            root,
            journal,
            known,
            max_identities,
        })
    }

    /// Observe `kel`'s head event as `witness_id`, signing with
    /// `witness_key` at `observed_epoch`, deriving the witness policy from
    /// `kel`'s own declared policy rather than accepting one as a
    /// parameter (D-0459) — durable. An `Accepted` outcome is written to
    /// disk before this call returns; every other outcome
    /// (`AlreadyAccepted`, `Stale`, `ControllerDuplicity`) never changed
    /// this journal's accepted state, so nothing new is written.
    pub fn observe_declared(
        &mut self,
        kel: &Kel,
        witness_id: WitnessId,
        witness_key: &SigningKey,
        observed_epoch: u64,
    ) -> Result<WitnessObservation> {
        let identity = kel.did();
        if !self.known.contains(&identity) && self.known.len() >= self.max_identities {
            return Err(WitnessServiceError::TooManyIdentities);
        }
        let outcome = observe_declared_on(
            &mut self.journal,
            kel,
            witness_id,
            witness_key,
            observed_epoch,
        )?;
        if matches!(outcome, WitnessObservation::Accepted(_)) {
            self.persist(&identity, kel, observed_epoch)?;
            self.known.insert(identity);
        }
        Ok(outcome)
    }

    /// This witness's retained state for `identity`, if it has observed
    /// anything for it yet.
    pub fn state_for(&self, identity: &Did) -> Option<&WitnessIdentityState> {
        self.journal.state_for(identity)
    }

    /// How many distinct identities this journal currently tracks.
    pub fn tracked_identity_count(&self) -> usize {
        self.known.len()
    }

    fn record_path(&self, identity: &Did) -> PathBuf {
        // Hashed rather than the raw SCID string: `Did` already restricts
        // its charset, but naming a file directly from untrusted-shaped
        // input is exactly the kind of thing this tree never does without
        // a reason not to — a fixed-width digest closes any path-shaped
        // concern by construction instead of relying on a caller elsewhere
        // getting SCID validation right forever.
        let hash = HashAlgorithm::Blake3.digest(identity.as_str().as_bytes());
        let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        self.root.join(format!("{hex}.state"))
    }

    fn persist(&self, identity: &Did, kel: &Kel, observed_epoch: u64) -> Result<()> {
        let bytes = encode_record(&kel.to_bytes(), observed_epoch);
        let final_path = self.record_path(identity);
        let tmp_path = final_path.with_extension("tmp");
        fs::write(&tmp_path, &bytes)?;
        fs::rename(&tmp_path, &final_path)?;
        Ok(())
    }
}

/// Observe `kel`'s head event, deriving the witness policy from `kel`'s own
/// most recent establishment event rather than accepting one as a
/// parameter — the D-0459 rule (never let a caller hand a verifier the
/// standard its own claim is judged against), applied here by composing
/// [`Kel::declared_witness_policy`] with [`WitnessJournal::observe_verified`]
/// rather than trusting an externally supplied [`did_mini::WitnessPolicy`].
fn observe_declared_on(
    journal: &mut WitnessJournal,
    kel: &Kel,
    witness_id: WitnessId,
    witness_key: &SigningKey,
    observed_epoch: u64,
) -> Result<WitnessObservation> {
    let policy = kel
        .declared_witness_policy()
        .ok_or(IdentityError::NoWitnessPolicyDeclared)?;
    Ok(journal.observe_verified(kel, &policy, witness_id, witness_key, observed_epoch)?)
}

fn encode_record(kel_bytes: &[u8], observed_epoch: u64) -> Vec<u8> {
    let mut w = Vec::with_capacity(RECORD_DOMAIN.len() + 4 + kel_bytes.len() + 8);
    w.extend_from_slice(RECORD_DOMAIN);
    w.extend_from_slice(&(kel_bytes.len() as u32).to_be_bytes());
    w.extend_from_slice(kel_bytes);
    w.extend_from_slice(&observed_epoch.to_be_bytes());
    w
}

fn decode_record(bytes: &[u8]) -> Result<(Vec<u8>, u64)> {
    let prefix_len = RECORD_DOMAIN.len();
    if bytes.len() < prefix_len + 4 {
        return Err(WitnessServiceError::CorruptRecord);
    }
    if &bytes[..prefix_len] != RECORD_DOMAIN {
        return Err(WitnessServiceError::CorruptRecord);
    }
    let mut pos = prefix_len;
    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&bytes[pos..pos + 4]);
    let kel_len = u32::from_be_bytes(len_bytes) as usize;
    pos += 4;
    let end = pos
        .checked_add(kel_len)
        .ok_or(WitnessServiceError::CorruptRecord)?;
    if bytes.len() != end + 8 {
        return Err(WitnessServiceError::CorruptRecord);
    }
    let kel_bytes = bytes[pos..end].to_vec();
    let mut epoch_bytes = [0u8; 8];
    epoch_bytes.copy_from_slice(&bytes[end..end + 8]);
    let observed_epoch = u64::from_be_bytes(epoch_bytes);
    Ok((kel_bytes, observed_epoch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_mini::Controller;

    fn temp_root(tag: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "mini-witness-service-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        root
    }

    fn a_witness() -> (WitnessId, SigningKey) {
        let root = Controller::incept_single().unwrap();
        (WitnessId(root.did()), SigningKey::generate().unwrap())
    }

    fn appointed_identity() -> Controller {
        let mut owner = Controller::incept_single().unwrap();
        let (witness_id, _) = a_witness();
        owner.appoint_witnesses(vec![witness_id.0], 1).unwrap();
        owner
    }

    #[test]
    fn an_accepted_receipt_survives_a_restart_byte_for_byte() {
        let root = temp_root("restart");
        let (witness_id, witness_key) = a_witness();
        let owner = appointed_identity();
        // Re-appoint the same witness the journal will actually authenticate as.
        let mut owner = owner;
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();

        let before = {
            let mut store =
                PersistentWitnessJournal::open(&root, witness_id.clone(), &witness_key).unwrap();
            let outcome = store
                .observe_declared(&owner.kel(), witness_id.clone(), &witness_key, 100)
                .unwrap();
            let WitnessObservation::Accepted(receipt) = outcome else {
                panic!("expected Accepted");
            };
            receipt
        };
        // Drop `store` here -- the equivalent of the process exiting.

        let after = {
            let store =
                PersistentWitnessJournal::open(&root, witness_id.clone(), &witness_key).unwrap();
            store
                .state_for(&owner.did())
                .expect("state survives a restart")
                .issued_receipt()
                .clone()
        };

        assert_eq!(before, after, "the exact same receipt, not a re-signed one");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_second_open_answers_a_resubmission_with_already_accepted() {
        let root = temp_root("already-accepted");
        let (witness_id, witness_key) = a_witness();
        let mut owner = appointed_identity();
        owner
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();

        {
            let mut store =
                PersistentWitnessJournal::open(&root, witness_id.clone(), &witness_key).unwrap();
            store
                .observe_declared(&owner.kel(), witness_id.clone(), &witness_key, 100)
                .unwrap();
        }

        let mut reopened =
            PersistentWitnessJournal::open(&root, witness_id.clone(), &witness_key).unwrap();
        let outcome = reopened
            .observe_declared(&owner.kel(), witness_id, &witness_key, 200)
            .unwrap();
        assert!(matches!(outcome, WitnessObservation::AlreadyAccepted(_)));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_stray_tmp_file_from_an_interrupted_write_is_ignored_on_open() {
        let root = temp_root("stray-tmp");
        let (witness_id, witness_key) = a_witness();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("deadbeef.tmp"), b"not a real record").unwrap();

        // Must open cleanly: the stray .tmp is never mistaken for a record.
        let store = PersistentWitnessJournal::open(&root, witness_id, &witness_key).unwrap();
        assert_eq!(store.tracked_identity_count(), 0);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_corrupt_state_file_is_rejected_not_silently_skipped() {
        let root = temp_root("corrupt");
        let (witness_id, witness_key) = a_witness();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("0000.state"), b"garbage, not a valid record").unwrap();

        let result = PersistentWitnessJournal::open(&root, witness_id, &witness_key);
        assert!(
            result.is_err(),
            "corrupt durable state must not be silently ignored"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_full_journal_refuses_a_new_identity_but_not_an_existing_one() {
        let root = temp_root("capacity");
        let (witness_id, witness_key) = a_witness();
        let mut first = appointed_identity();
        first
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();
        let mut second = appointed_identity();
        second
            .appoint_witnesses(vec![witness_id.0.clone()], 1)
            .unwrap();

        let mut store = PersistentWitnessJournal::open_with_capacity(
            &root,
            witness_id.clone(),
            &witness_key,
            1,
        )
        .unwrap();
        store
            .observe_declared(&first.kel(), witness_id.clone(), &witness_key, 1)
            .unwrap();

        // A second call for the *same* (already-tracked) identity must
        // still be allowed even though the journal is "full."
        let repeat = store
            .observe_declared(&first.kel(), witness_id.clone(), &witness_key, 2)
            .unwrap();
        assert!(matches!(repeat, WitnessObservation::AlreadyAccepted(_)));

        // A genuinely new identity is refused once at capacity.
        let refused = store.observe_declared(&second.kel(), witness_id, &witness_key, 3);
        assert!(matches!(
            refused,
            Err(WitnessServiceError::TooManyIdentities)
        ));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn an_identity_without_a_declared_policy_is_rejected_and_nothing_is_persisted() {
        let root = temp_root("no-policy");
        let (witness_id, witness_key) = a_witness();
        let owner = Controller::incept_single().unwrap(); // never appoints witnesses

        let mut store =
            PersistentWitnessJournal::open(&root, witness_id.clone(), &witness_key).unwrap();
        let result = store.observe_declared(&owner.kel(), witness_id, &witness_key, 1);
        assert!(result.is_err());
        assert_eq!(store.tracked_identity_count(), 0);
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir_all(&root).ok();
    }
}
