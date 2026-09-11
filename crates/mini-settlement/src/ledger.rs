//! The seam between this crate's protocol logic and a real canonical chain.
//!
//! `mini-chain` today verifies finality of an abstract block; it has no
//! account/balance execution engine yet (that's the state-machine layer
//! roadmap #36-#45 build toward). [`CanonicalLedgerView`] is the same kind
//! of seam `mini-forge::KelDirectory`/`IdentityOracle` and
//! `mini_presence::ReplayGuard` already use: this crate's reconciliation
//! logic is fully specified and testable *now*, against any implementation
//! of this trait, without needing the real chain to exist first.

/// A read-only view of canonical, finalized settlement state for one payer.
/// A real implementation is chain-backed; [`crate::InMemoryLedgerView`] is
/// for tests only.
/// Consensus-authenticated reason an exact claim did not execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalRejection {
    WrongNetwork,
    UnsupportedPayee,
    StaleSequence,
    InsufficientFunds,
    /// A [`crate::PaymentClaimV2`]'s [`crate::ChainAnchorV2`] is not a
    /// recognized ancestor of the canonical chain state this view
    /// represents (Gate #28, D-0513) — e.g. it names a height/block id
    /// from a fork that never became canonical.
    UnrecognizedAnchor,
}

pub trait CanonicalLedgerView {
    /// The highest sequence this ledger has finalized a claim at for `payer`,
    /// if any. `None` means this payer has never had a claim finalized.
    fn finalized_sequence(&self, payer: &[u8]) -> Option<u64>;

    /// The digest ([`crate::claim_digest`]) of the claim this ledger
    /// finalized for `payer` at exactly `sequence`, if any. Only meaningful
    /// when `sequence <= finalized_sequence(payer)`.
    fn finalized_claim_digest(&self, payer: &[u8], sequence: u64) -> Option<[u8; 32]>;

    /// A canonical rejection for this exact signed claim digest, if retained.
    fn rejected_claim(&self, _digest: &[u8; 32]) -> Option<CanonicalRejection> {
        None
    }

    /// The current canonical chain height, used to evaluate a
    /// [`crate::PaymentClaimV2`]'s height-anchored validity window
    /// (Gate #28, D-0513). A `CanonicalLedgerView` that only supports V1
    /// claims never has this called and may leave the default.
    fn current_height(&self) -> u64 {
        0
    }

    /// Whether `(height, block_id)` is a recognized ancestor of the
    /// current canonical chain state this view represents. A real ledger
    /// answers this from actual chain history. Default `false` (fail
    /// closed): a `CanonicalLedgerView` that does not implement V2
    /// anchoring must not have every claimed anchor treated as valid by
    /// accident.
    fn is_recognized_anchor(&self, _height: u64, _block_id: &[u8; 32]) -> bool {
        false
    }
}

/// A trivial in-memory [`CanonicalLedgerView`] — test-only. Production
/// needs a real chain-execution-backed implementation; see this crate's
/// README for what that requires.
#[derive(Debug, Default)]
pub struct InMemoryLedgerView {
    finalized: std::collections::HashMap<Vec<u8>, Vec<(u64, [u8; 32])>>,
    rejected: std::collections::HashMap<[u8; 32], CanonicalRejection>,
    height: u64,
    recognized_anchors: std::collections::HashSet<(u64, [u8; 32])>,
}

impl InMemoryLedgerView {
    /// A new, empty ledger view with nothing finalized.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `digest` as finalized for `payer` at `sequence`. Test-only
    /// helper — a real ledger reaches this state via actual chain
    /// execution and finality, not a direct setter.
    pub fn finalize(&mut self, payer: &[u8], sequence: u64, digest: [u8; 32]) {
        self.finalized
            .entry(payer.to_vec())
            .or_default()
            .push((sequence, digest));
    }

    pub fn reject(&mut self, digest: [u8; 32], reason: CanonicalRejection) {
        self.rejected.insert(digest, reason);
    }

    /// Set the current canonical chain height this view reports. Test-only
    /// helper for exercising [`crate::PaymentClaimV2`] expiry.
    pub fn set_height(&mut self, height: u64) {
        self.height = height;
    }

    /// Record `(height, block_id)` as a recognized canonical ancestor.
    /// Test-only helper — a real ledger answers this from actual chain
    /// history, never a direct allow-list.
    pub fn recognize_anchor(&mut self, height: u64, block_id: [u8; 32]) {
        self.recognized_anchors.insert((height, block_id));
    }
}

impl CanonicalLedgerView for InMemoryLedgerView {
    fn finalized_sequence(&self, payer: &[u8]) -> Option<u64> {
        self.finalized
            .get(payer)
            .and_then(|entries| entries.iter().map(|(n, _)| *n).max())
    }

    fn finalized_claim_digest(&self, payer: &[u8], sequence: u64) -> Option<[u8; 32]> {
        self.finalized
            .get(payer)?
            .iter()
            .find(|(n, _)| *n == sequence)
            .map(|(_, d)| *d)
    }

    fn rejected_claim(&self, digest: &[u8; 32]) -> Option<CanonicalRejection> {
        self.rejected.get(digest).copied()
    }

    fn current_height(&self) -> u64 {
        self.height
    }

    fn is_recognized_anchor(&self, height: u64, block_id: &[u8; 32]) -> bool {
        self.recognized_anchors.contains(&(height, *block_id))
    }
}
