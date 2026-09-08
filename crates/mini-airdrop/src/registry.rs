//! Replay prevention: has this identity root already redeemed this
//! campaign? Mirrors the same seam `mini_settlement::ClaimWatcher` and
//! `mini_settlement::CanonicalLedgerView` already use in this workspace --
//! a trait this crate's verification logic is fully specified and tested
//! against today (via [`InMemoryClaimedRegistry`]), that a real persisted
//! backend implements later with no change to the verification rules
//! themselves.
//!
//! ## Atomic reservation, not check-then-write (PR #327 finding F-20)
//!
//! The previous shape of this trait was two separate calls --
//! `already_claimed` to check, `mark_claimed` to write -- with no
//! atomicity between them. Two callers (two threads sharing a registry,
//! or two processes each holding their own [`crate::file_registry::
//! FileClaimedRegistry`] over the same path) could both observe
//! "not yet claimed" before either had persisted anything, and both go on
//! to report a successful, distinct [`crate::claim::ClaimOutcome`] for the
//! same entitlement -- a real double-award, not a theoretical one.
//! [`ClaimedRegistry::try_reserve`] closes that gap by making "check and
//! record" one call a backend must implement atomically (a transaction under a cross-process file lock for [`crate::file_registry::
//! FileClaimedRegistry`]).
//!
//! `try_reserve` also fixes the finding's other named failure: "a valid
//! claimant is marked claimed, then signing/submission fails; on retry
//! the system refuses the claim although no funds arrived." Recording an
//! `outcome_digest` alongside the reservation lets a retry of the exact
//! same resolved claim (same identity root, same amount, same recipient)
//! come back as [`ReservationOutcome::IdempotentRetry`] instead of an
//! error -- the caller gets the same [`crate::claim::ClaimOutcome`] again
//! to retry whatever downstream step failed, rather than being
//! permanently stranded. A *different* outcome for an already-reserved
//! identity root (a genuine double-claim attempt, e.g. a different
//! recipient) still fails with
//! [`crate::error::AirdropError::AlreadyClaimed`] exactly as before.
//!
//! This crate still never produces a `mini_settlement::PaymentClaim` or a
//! canonical payment digest -- that honest separation (this is claim
//! *bookkeeping*, not proof of payment) is unchanged; see this crate's
//! and `mini-airdrop-treasury`'s top-level docs.

use did_mini::Did;

use crate::error::Result;

/// The result of a successful [`ClaimedRegistry::try_reserve`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservationOutcome {
    /// No reservation existed for this identity root before this call;
    /// one was just durably created.
    Fresh,
    /// A reservation already existed for this identity root, for this
    /// *exact same* `outcome_digest` -- an idempotent retry, not a new
    /// award. The caller resolved the identical claim again (e.g. after a
    /// downstream signing/submission failure left the original attempt
    /// unpaid) and may safely treat this exactly like `Fresh`.
    IdempotentRetry,
}

/// A record of which identity roots have already claimed a campaign.
///
/// See the module docs above for why the write side is one atomic
/// `try_reserve` call rather than a separate check-then-write pair.
pub trait ClaimedRegistry {
    /// `true` if `identity_root` has already claimed this campaign.
    fn already_claimed(&self, campaign_id: &[u8], identity_root: &Did) -> bool;

    /// Atomically check-and-record a claim for `identity_root` at
    /// `at_ms`, bound to `outcome_digest` (see
    /// [`crate::claim::outcome_digest`]). Called only after every other
    /// check in [`crate::claim::verify_and_resolve_claim`] has already
    /// passed -- a failed verification never reserves anything.
    ///
    /// - No prior reservation for `identity_root`: durably record one and
    ///   return `Ok(`[`ReservationOutcome::Fresh`]`)`.
    /// - A prior reservation exists with the *same* `outcome_digest`:
    ///   return `Ok(`[`ReservationOutcome::IdempotentRetry`]`)` without
    ///   creating a second record.
    /// - A prior reservation exists with a *different* `outcome_digest`:
    ///   return `Err(`[`crate::error::AirdropError::AlreadyClaimed`]`)`.
    /// - The backend cannot durably determine which of the above holds
    ///   (write failure, or an existing record it cannot decode): return
    ///   `Err` and record nothing new -- never guess.
    ///
    /// An `Err` here propagates straight out of `verify_and_resolve_claim`,
    /// so a caller never receives a `ClaimOutcome` for a claim this
    /// registry did not actually manage to reserve.
    fn try_reserve(
        &mut self,
        campaign_id: &[u8],
        identity_root: &Did,
        outcome_digest: [u8; 32],
        at_ms: u64,
    ) -> Result<ReservationOutcome>;
}

/// A trivial in-memory [`ClaimedRegistry`] -- test-only, and for any real
/// deployment where losing all claim history on process restart is
/// actually acceptable. See [`crate::file_registry::FileClaimedRegistry`]
/// for a real persisted implementation.
#[derive(Debug, Default)]
pub struct InMemoryClaimedRegistry {
    claimed: std::collections::HashMap<(Vec<u8>, Did), (u64, [u8; 32])>,
}

impl InMemoryClaimedRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// When `identity_root` claimed, if it has.
    pub fn claimed_at(&self, campaign_id: &[u8], identity_root: &Did) -> Option<u64> {
        self.claimed
            .get(&(campaign_id.to_vec(), identity_root.clone()))
            .map(|(at_ms, _)| *at_ms)
    }
}

impl ClaimedRegistry for InMemoryClaimedRegistry {
    fn already_claimed(&self, campaign_id: &[u8], identity_root: &Did) -> bool {
        self.claimed
            .contains_key(&(campaign_id.to_vec(), identity_root.clone()))
    }

    fn try_reserve(
        &mut self,
        campaign_id: &[u8],
        identity_root: &Did,
        outcome_digest: [u8; 32],
        at_ms: u64,
    ) -> Result<ReservationOutcome> {
        use std::collections::hash_map::Entry;
        match self
            .claimed
            .entry((campaign_id.to_vec(), identity_root.clone()))
        {
            Entry::Vacant(slot) => {
                slot.insert((at_ms, outcome_digest));
                Ok(ReservationOutcome::Fresh)
            }
            Entry::Occupied(slot) => {
                if slot.get().1 == outcome_digest {
                    Ok(ReservationOutcome::IdempotentRetry)
                } else {
                    Err(crate::error::AirdropError::AlreadyClaimed)
                }
            }
        }
    }
}
