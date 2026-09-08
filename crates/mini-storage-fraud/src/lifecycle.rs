//! Audited replica standing with checked ongoing possession.
//!
//! Proposer weight is available only through ProviderStanding. Its capacity
//! cannot be created from declarations or summed by external callers. Every
//! active replica has both a verified registration and verified window responses.
//! Time and beacon inputs still require canonical anchoring before production;
//! these checks do not establish replication uniqueness or auditor independence.

use std::collections::BTreeMap;

use mini_crypto::HashAlgorithm;
use mini_spacetime::StorageChallenge;

use crate::claim::VerifiedReplicaClaim;
use crate::codec::Writer;
use crate::error::{FraudError, Result};
use crate::seal::seal_commitment_digest;

/// Domain separator for per-window challenge derivation.
pub const WINDOW_CHALLENGE_DOMAIN: &[u8] = b"mininet/mini-storage-fraud/window-challenge/v1";

/// Largest number of challenges one window may demand, bounding both the
/// prover's work and the verifier's.
pub const MAX_CHALLENGES_PER_WINDOW: u32 = 1024;

pub use mini_spacetime::StorageUnitPolicy;

/// Audited, active capacity minted only by this crate's lifecycle checks.
/// It has no public constructor from a number, commitment, or observation, and
/// cannot be added to itself to multiply one replica's weight.
///
/// ```compile_fail
/// use mini_storage_fraud::ProvenCapacity;
/// use mini_spacetime::{StorageCommitment, StorageUnitPolicy};
/// let claim = StorageCommitment { merkle_root: [0; 32], block_count: 1_000_000, block_size_bytes: 32 };
/// let forged = ProvenCapacity::from_commitment(&claim, &StorageUnitPolicy::gibibytes());
/// ```
///
/// ```compile_fail
/// use mini_spacetime::{ObservedCapacity, StorageCommitment, StorageUnitPolicy};
/// let claim = StorageCommitment { merkle_root: [0; 32], block_count: 1_000_000, block_size_bytes: 32 };
/// let capacity = ObservedCapacity::from_commitment(&claim, &StorageUnitPolicy::gibibytes());
/// mini_spacetime::proposer_weight(capacity, 1, &Default::default());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvenCapacity {
    units: u64,
    committed_bytes: u64,
}

impl ProvenCapacity {
    fn none() -> Self {
        Self {
            units: 0,
            committed_bytes: 0,
        }
    }
    fn saturating_add(self, other: Self) -> Self {
        Self {
            units: self.units.saturating_add(other.units),
            committed_bytes: self.committed_bytes.saturating_add(other.committed_bytes),
        }
    }
    pub fn units(&self) -> u64 {
        self.units
    }
    pub fn committed_bytes(&self) -> u64 {
        self.committed_bytes
    }
}

/// Derive capacity from the audited seal.
///
/// Goes through the claim's own [`mini_spacetime::StorageCommitment`], which
/// is itself derived from the audited seal rather than supplied alongside it
/// — so there is exactly one statement anywhere about how much this replica
/// covers, and it is inside the object a quorum checked.
pub fn capacity_units_of(
    claim: &VerifiedReplicaClaim,
    policy: &StorageUnitPolicy,
) -> mini_spacetime::ObservedCapacity {
    mini_spacetime::ObservedCapacity::from_commitment(&claim.storage_commitment(), policy)
}

/// How often a registered replica must prove it still holds what it sealed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPolicy {
    window_ms: u64,
    challenges_per_window: u32,
    grace_windows: u32,
}

impl WindowPolicy {
    pub fn new(window_ms: u64, challenges_per_window: u32, grace_windows: u32) -> Result<Self> {
        if window_ms == 0 || challenges_per_window == 0 {
            return Err(FraudError::InvalidPolicy);
        }
        if challenges_per_window > MAX_CHALLENGES_PER_WINDOW {
            return Err(FraudError::InvalidPolicy);
        }
        Ok(Self {
            window_ms,
            challenges_per_window,
            grace_windows,
        })
    }

    /// Daily windows, 32 challenges each, two windows of grace.
    ///
    /// The grace is not leniency about fraud — it is an admission that a
    /// missed window and an unreachable peer look identical from here.
    pub fn daily() -> Self {
        Self {
            window_ms: 86_400_000,
            challenges_per_window: 32,
            grace_windows: 2,
        }
    }

    pub fn window_ms(&self) -> u64 {
        self.window_ms
    }

    pub fn challenges_per_window(&self) -> u32 {
        self.challenges_per_window
    }

    pub fn grace_windows(&self) -> u32 {
        self.grace_windows
    }

    /// Which window `now_ms` falls in, counting from `genesis_ms`.
    pub fn window_at(&self, genesis_ms: u64, now_ms: u64) -> u64 {
        now_ms.saturating_sub(genesis_ms) / self.window_ms
    }
}

/// Where a registered replica stands right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReplicaState {
    /// Proving on schedule. Contributes its full derived capacity.
    Active,
    /// Missed at least one window but still inside the grace allowance.
    /// Contributes nothing while degraded — capacity follows proof, not
    /// history — but recovers fully on the next good window.
    Degraded { missed_windows: u32 },
    /// Missed beyond grace. Contributes nothing and does not self-recover;
    /// re-entry means registering again, because a replica nobody has seen
    /// for that long is not distinguishable from one that is gone.
    Suspended,
    /// Withdrawn by the provider. Terminal and voluntary.
    Retired,
}

impl ReplicaState {
    /// Whether capacity counts in this state.
    pub fn counts_capacity(&self) -> bool {
        matches!(self, ReplicaState::Active)
    }
}

/// One registered replica's ongoing obligation and standing.
#[derive(Debug, Clone)]
pub struct ReplicaLifecycle {
    claim: VerifiedReplicaClaim,
    genesis_ms: u64,
    state: ReplicaState,
    /// The window this replica became obligated to prove in. Misses are
    /// counted from here until the first successful window replaces it.
    obligated_from: u64,
    last_proven_window: Option<u64>,
    highest_window_seen: u64,
}

impl ReplicaLifecycle {
    /// Begin tracking a verified claim from `registered_at_ms`.
    ///
    /// Starts `Degraded { missed_windows: 0 }`, not `Active`: registration
    /// proves the replica was sealed, not that it is still held. The first
    /// answered window is what makes it active.
    pub fn begin(
        claim: VerifiedReplicaClaim,
        genesis_ms: u64,
        registered_at_ms: u64,
        policy: &WindowPolicy,
    ) -> Self {
        let window = policy.window_at(genesis_ms, registered_at_ms);
        Self {
            claim,
            genesis_ms,
            state: ReplicaState::Degraded { missed_windows: 0 },
            obligated_from: window,
            last_proven_window: None,
            highest_window_seen: window,
        }
    }

    pub fn claim(&self) -> &VerifiedReplicaClaim {
        &self.claim
    }

    pub fn state(&self) -> ReplicaState {
        self.state
    }

    pub fn last_proven_window(&self) -> Option<u64> {
        self.last_proven_window
    }

    /// The challenges this replica must answer for `window`.
    ///
    /// Leaf indices are derived from the seal digest, the window index, and a
    /// `beacon` the **verifier** supplies. The provider contributes nothing to
    /// the derivation, so it cannot pre-compute which nodes it will be asked
    /// for and keep only those. The beacon must come from somewhere the
    /// provider does not control and must not be reused across windows; a
    /// recent block hash or a fresh verifier nonce both work, and this crate
    /// cannot check that it is either.
    pub fn challenges_for(
        &self,
        window: u64,
        beacon: &[u8],
        policy: &WindowPolicy,
    ) -> Vec<StorageChallenge> {
        let node_count = self.claim.seal().node_count as u64;
        let digest = seal_commitment_digest(self.claim.seal());
        (0..policy.challenges_per_window)
            .map(|index| {
                let mut writer = Writer::new();
                writer.raw(WINDOW_CHALLENGE_DOMAIN);
                writer.raw(&digest);
                writer.u64(window);
                writer.bytes(beacon);
                writer.u32(index);
                let drawn = HashAlgorithm::Blake3.digest(&writer.finish());
                let raw = u64::from_be_bytes(drawn[0..8].try_into().expect("32-byte digest"));
                StorageChallenge {
                    leaf_index: (raw % node_count) as usize,
                }
            })
            .collect()
    }

    /// Verify the exact window challenges before recording possession. Missing,
    /// extra, substituted, or invalid Merkle responses never grant capacity.
    pub fn record_proven_window(
        &mut self,
        window: u64,
        beacon: &[u8],
        responses: &[mini_spacetime::StorageChallengeResponse],
        policy: &WindowPolicy,
    ) -> Result<()> {
        if window < self.highest_window_seen {
            return Err(FraudError::WindowAlreadyProven);
        }
        let challenges = self.challenges_for(window, beacon, policy);
        if responses.len() != challenges.len()
            || !challenges
                .iter()
                .zip(responses)
                .all(|(challenge, response)| {
                    mini_spacetime::verify_storage_challenge(
                        &self.claim.storage_commitment(),
                        challenge,
                        response,
                    )
                })
        {
            return Err(FraudError::AuditFailed);
        }
        if matches!(self.state, ReplicaState::Retired | ReplicaState::Suspended) {
            return Err(FraudError::ReplicaNotProving);
        }
        if let Some(last) = self.last_proven_window {
            if window <= last {
                // Replaying an already-credited window must not extend a
                // streak or reverse a lapse.
                return Err(FraudError::WindowAlreadyProven);
            }
        }
        self.advance_to(window, policy);
        if matches!(self.state, ReplicaState::Suspended) {
            return Err(FraudError::ReplicaNotProving);
        }
        self.last_proven_window = Some(window);
        self.state = ReplicaState::Active;
        Ok(())
    }

    /// Move the clock forward without a proof, crediting nothing.
    ///
    /// Idempotent for windows already accounted for, so a verifier polling
    /// repeatedly inside one window does not accumulate phantom misses.
    pub fn advance_to(&mut self, window: u64, policy: &WindowPolicy) {
        if window <= self.highest_window_seen {
            return;
        }
        self.highest_window_seen = window;
        if matches!(self.state, ReplicaState::Retired | ReplicaState::Suspended) {
            return;
        }

        // Misses are counted from the last window that counts as satisfied:
        // the most recent proven one, or the registration window if the
        // replica has never proven. Registration itself is not a proof, but
        // demanding one in the very window a replica registered would punish
        // arriving late in a window, so that window is the baseline rather
        // than the first miss.
        let satisfied = self.last_proven_window.unwrap_or(self.obligated_from);
        let missed = window.saturating_sub(satisfied).saturating_sub(1);
        let missed = u32::try_from(missed).unwrap_or(u32::MAX);

        self.state = if missed == 0 {
            self.state
        } else if missed > policy.grace_windows {
            ReplicaState::Suspended
        } else {
            ReplicaState::Degraded {
                missed_windows: missed,
            }
        };
    }

    /// Withdraw voluntarily. Terminal.
    pub fn retire(&mut self) {
        self.state = ReplicaState::Retired;
    }

    /// Capacity this replica currently contributes: its derived figure while
    /// active, nothing otherwise.
    pub fn proven_capacity(&self, units: &StorageUnitPolicy) -> ProvenCapacity {
        if self.state.counts_capacity() {
            let observed = capacity_units_of(&self.claim, units);
            ProvenCapacity {
                units: observed.units(),
                committed_bytes: observed.committed_bytes(),
            }
        } else {
            ProvenCapacity::none()
        }
    }

    pub fn genesis_ms(&self) -> u64 {
        self.genesis_ms
    }
}

/// Every replica one provider is tracking, and what they add up to.
#[derive(Debug, Default)]
pub struct ProviderStanding {
    replicas: BTreeMap<[u8; 32], ReplicaLifecycle>,
    provider_root: Option<did_mini::Did>,
}

impl ProviderStanding {
    pub fn new() -> Self {
        Self::default()
    }

    /// Track one replica for this provider. Duplicate roots are rejected so
    /// replaying an old lifecycle cannot resurrect lapsed or retired capacity.
    pub fn track(&mut self, lifecycle: ReplicaLifecycle) -> Result<()> {
        if self
            .replicas
            .contains_key(&lifecycle.claim().replica_root())
        {
            return Err(FraudError::AlreadyRegistered);
        }
        let root = lifecycle.claim().provider_root();
        if self.provider_root.as_ref().is_some_and(|held| held != root) {
            return Err(FraudError::ProviderMismatch);
        }
        self.provider_root = Some(root.clone());
        self.replicas
            .insert(lifecycle.claim().replica_root(), lifecycle);
        Ok(())
    }

    pub fn get_mut(&mut self, replica_root: &[u8; 32]) -> Option<&mut ReplicaLifecycle> {
        self.replicas.get_mut(replica_root)
    }

    pub fn get(&self, replica_root: &[u8; 32]) -> Option<&ReplicaLifecycle> {
        self.replicas.get(replica_root)
    }

    pub fn len(&self) -> usize {
        self.replicas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.replicas.is_empty()
    }

    /// Advance every tracked replica to `window`.
    pub fn advance_to(&mut self, window: u64, policy: &WindowPolicy) {
        for lifecycle in self.replicas.values_mut() {
            lifecycle.advance_to(window, policy);
        }
    }

    /// Total capacity currently proven across every actively-proving replica.
    ///
    /// Saturating: a provider tracking absurdly many replicas cannot wrap this
    /// into a small number.
    /// Keyed by replica root, so no replica is counted twice — the one
    /// aggregate operation is private, preventing external double-counting.
    pub fn proven_capacity(&self, units: &StorageUnitPolicy) -> ProvenCapacity {
        self.replicas
            .values()
            .map(|lifecycle| lifecycle.proven_capacity(units))
            .fold(ProvenCapacity::none(), ProvenCapacity::saturating_add)
    }

    /// This provider's block-production selection weight (D-0477),
    /// derived entirely from its own tracked, audited, lifecycle-checked
    /// replicas.
    ///
    /// The only capacity-bearing parameter is checked standing; no public
    /// lower-layer weight function accepts caller-constructed measurements.
    pub fn block_production_weight(
        &self,
        units: &StorageUnitPolicy,
        distinct_regions: u32,
        params: &crate::ProposerParams,
    ) -> u64 {
        crate::weight::proposer_weight(self.proven_capacity(units), distinct_regions, params)
    }
}
