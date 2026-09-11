//! Canonical shielded membership and accounting, using only opaque byte pairs.
//! Cryptographic proof verification is supplied through `ClaimVerifier`; no
//! balances or commitments reach validator/governance vote-weight interfaces.

use std::collections::{BTreeMap, BTreeSet};

use crate::{ClaimVerifier, ExecutionError, NullifierRecord, Result};

const MAX_OUTPUTS: usize = crate::MAX_LEDGER_SNAPSHOT_ENTRIES;
const MAX_RING_MEMBERS: usize = 16 * 128;

/// One canonical one-time public key and its amount commitment. Both are opaque
/// 32-byte encodings; the concrete verifier validates their cryptographic format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShieldedOutput {
    pub public_key: Vec<u8>,
    pub amount_commitment: Vec<u8>,
}

impl ShieldedOutput {
    fn well_formed(&self) -> bool {
        self.public_key.len() == 32 && self.amount_commitment.len() == 32
    }
}

/// Explicit genesis funding. The concrete verifier checks the commitment equals
/// this public amount with zero blinding. Genesis is network configuration, not
/// a transaction that can mint funds later or a claim of production authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShieldedGenesisAllocation {
    pub output: ShieldedOutput,
    pub amount_micro: u64,
}

/// Effects returned only after verifying the claim's complete cryptographic
/// transcript. Execution independently checks canonical membership and output
/// uniqueness before applying these effects and the claim's nullifiers together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShieldedClaimEffects {
    /// Every ring member, including decoys, with its exact committed amount.
    pub ring_members: Vec<ShieldedOutput>,
    /// All outputs, in the order committed by the claim transcript.
    pub outputs: Vec<ShieldedOutput>,
    pub fee_micro: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ShieldedLedger {
    genesis: Vec<ShieldedGenesisAllocation>,
    outputs: BTreeMap<Vec<u8>, Vec<u8>>,
    applied: Vec<[u8; 32]>,
    pool_micro: u128,
}

impl ShieldedLedger {
    pub(crate) fn genesis(
        network: &[u8; 32],
        mut allocations: Vec<ShieldedGenesisAllocation>,
        verifier: &dyn ClaimVerifier,
    ) -> Result<Self> {
        if allocations.len() > MAX_OUTPUTS {
            return Err(ExecutionError::SnapshotTooLarge);
        }
        allocations.sort_by(|a, b| a.output.cmp(&b.output));
        let mut ledger = Self::default();
        for allocation in &allocations {
            if !allocation.output.well_formed()
                || !verifier.verify_genesis_allocation(network, allocation)
                || ledger
                    .outputs
                    .insert(
                        allocation.output.public_key.clone(),
                        allocation.output.amount_commitment.clone(),
                    )
                    .is_some()
            {
                return Err(ExecutionError::InvalidShieldedClaim);
            }
            ledger.pool_micro = ledger
                .pool_micro
                .checked_add(u128::from(allocation.amount_micro))
                .ok_or(ExecutionError::AmountOverflow)?;
        }
        ledger.genesis = allocations;
        Ok(ledger)
    }

    pub(crate) fn pool_micro(&self) -> u128 {
        self.pool_micro
    }
    pub(crate) fn outputs(&self) -> &BTreeMap<Vec<u8>, Vec<u8>> {
        &self.outputs
    }

    pub(crate) fn genesis_commitment(&self) -> [u8; 32] {
        let mut bytes = b"mini-execution/shielded-genesis/v1".to_vec();
        for allocation in &self.genesis {
            bytes.extend_from_slice(&allocation.output.public_key);
            bytes.extend_from_slice(&allocation.output.amount_commitment);
            bytes.extend_from_slice(&allocation.amount_micro.to_be_bytes());
        }
        mini_crypto::HashAlgorithm::Blake3.digest(&bytes)
    }

    pub(crate) fn apply(&mut self, digest: [u8; 32], effects: ShieldedClaimEffects) -> Result<u64> {
        if self.applied.contains(&digest) {
            return Ok(0);
        }
        if effects.ring_members.is_empty()
            || effects.ring_members.len() > MAX_RING_MEMBERS
            || effects.outputs.is_empty()
            || effects.outputs.len() > 16
            || self.outputs.len().saturating_add(effects.outputs.len()) > MAX_OUTPUTS
            || self.applied.len() >= MAX_OUTPUTS
        {
            return Err(ExecutionError::InvalidShieldedClaim);
        }
        for member in &effects.ring_members {
            if !member.well_formed()
                || self.outputs.get(&member.public_key) != Some(&member.amount_commitment)
            {
                return Err(ExecutionError::UnknownShieldedInput);
            }
        }
        let mut new_keys = BTreeSet::new();
        for output in &effects.outputs {
            if !output.well_formed()
                || self.outputs.contains_key(&output.public_key)
                || !new_keys.insert(&output.public_key)
            {
                return Err(ExecutionError::DuplicateShieldedOutput);
            }
        }
        self.pool_micro = self
            .pool_micro
            .checked_sub(u128::from(effects.fee_micro))
            .ok_or(ExecutionError::SupplyConservationViolation)?;
        for output in effects.outputs {
            self.outputs
                .insert(output.public_key, output.amount_commitment);
        }
        self.applied.push(digest);
        Ok(effects.fee_micro)
    }

    /// Reconstruct outputs from the configured genesis and the exact historical
    /// order. A checkpoint cannot use its own invented outputs as membership proof.
    pub(crate) fn verify_replay(
        &self,
        network: &[u8; 32],
        nullifiers: &BTreeMap<Vec<u8>, [u8; 32]>,
        verifier: Option<&dyn ClaimVerifier>,
    ) -> Result<()> {
        if self == &Self::default() && nullifiers.is_empty() {
            return Ok(());
        }
        let verifier = verifier.ok_or(ExecutionError::MissingClaimVerifier)?;
        let mut replay = Self::genesis(network, self.genesis.clone(), verifier)?;
        let mut groups: BTreeMap<[u8; 32], Vec<NullifierRecord>> = BTreeMap::new();
        for (image, digest) in nullifiers {
            groups
                .entry(*digest)
                .or_default()
                .push(NullifierRecord::new(image.clone(), *digest));
        }
        for digest in &self.applied {
            let group = groups
                .remove(digest)
                .ok_or(ExecutionError::InvalidShieldedClaim)?;
            let effects = verifier
                .verify_claim(network, digest, &group)
                .ok_or(ExecutionError::InvalidShieldedClaim)?;
            replay.apply(*digest, effects)?;
        }
        if !groups.is_empty() || replay != *self {
            return Err(ExecutionError::InvalidShieldedClaim);
        }
        Ok(())
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut out = b"mini-execution/shielded-ledger/v1".to_vec();
        out.extend_from_slice(&(self.genesis.len() as u32).to_be_bytes());
        for allocation in &self.genesis {
            out.extend_from_slice(&allocation.output.public_key);
            out.extend_from_slice(&allocation.output.amount_commitment);
            out.extend_from_slice(&allocation.amount_micro.to_be_bytes());
        }
        out.extend_from_slice(&(self.outputs.len() as u32).to_be_bytes());
        for (key, commitment) in &self.outputs {
            out.extend_from_slice(key);
            out.extend_from_slice(commitment);
        }
        out.extend_from_slice(&(self.applied.len() as u32).to_be_bytes());
        for digest in &self.applied {
            out.extend_from_slice(digest);
        }
        out.extend_from_slice(&self.pool_micro.to_be_bytes());
        out
    }

    pub(crate) fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        fn take<'a>(bytes: &mut &'a [u8], count: usize) -> Result<&'a [u8]> {
            if bytes.len() < count {
                return Err(ExecutionError::SnapshotMalformed);
            }
            let (value, rest) = bytes.split_at(count);
            *bytes = rest;
            Ok(value)
        }
        fn count(bytes: &mut &[u8]) -> Result<usize> {
            let count = u32::from_be_bytes(take(bytes, 4)?.try_into().unwrap()) as usize;
            if count > MAX_OUTPUTS {
                return Err(ExecutionError::SnapshotTooLarge);
            }
            Ok(count)
        }
        if take(&mut bytes, b"mini-execution/shielded-ledger/v1".len())?
            != b"mini-execution/shielded-ledger/v1"
        {
            return Err(ExecutionError::SnapshotMalformed);
        }
        let mut ledger = Self::default();
        let mut genesis_keys = BTreeSet::new();
        for _ in 0..count(&mut bytes)? {
            let public_key = take(&mut bytes, 32)?.to_vec();
            let amount_commitment = take(&mut bytes, 32)?.to_vec();
            let amount_micro = u64::from_be_bytes(take(&mut bytes, 8)?.try_into().unwrap());
            if !genesis_keys.insert(public_key.clone()) {
                return Err(ExecutionError::SnapshotMalformed);
            }
            ledger.genesis.push(ShieldedGenesisAllocation {
                output: ShieldedOutput {
                    public_key,
                    amount_commitment,
                },
                amount_micro,
            });
        }
        for _ in 0..count(&mut bytes)? {
            let key = take(&mut bytes, 32)?.to_vec();
            let commitment = take(&mut bytes, 32)?.to_vec();
            if ledger.outputs.insert(key, commitment).is_some() {
                return Err(ExecutionError::SnapshotMalformed);
            }
        }
        let mut seen = BTreeSet::new();
        for _ in 0..count(&mut bytes)? {
            let digest = take(&mut bytes, 32)?.try_into().unwrap();
            if !seen.insert(digest) {
                return Err(ExecutionError::SnapshotMalformed);
            }
            ledger.applied.push(digest);
        }
        ledger.pool_micro = u128::from_be_bytes(take(&mut bytes, 16)?.try_into().unwrap());
        if !bytes.is_empty() {
            return Err(ExecutionError::SnapshotMalformed);
        }
        Ok(ledger)
    }
}
