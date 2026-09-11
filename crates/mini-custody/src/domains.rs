//! Independent key state per [`crate::manifest::CustodyDomain`].
//!
//! The Gate #93 audit report's Section 5 names four production custody
//! domains explicitly, each with its own independently-generated group
//! key so that compromising one domain's ceremony output never exposes
//! another's: BTC, XMR, and XRPL chain custody, plus the bounty payout
//! signing authority `mini_bounty` already anonymizes recipients for.
//! Those four are declared here as named constants -- a fifth domain is a
//! deliberate code change, not a value anything can pass in at runtime.

use std::collections::BTreeMap;

use crate::error::{CustodyError, Result};
use crate::manifest::{CustodyDomain, DkgSessionManifestV1};

/// Bitcoin UTXO custody signing authority.
pub const BTC_CUSTODY: CustodyDomain = CustodyDomain(1);
/// Monero custody signing authority.
pub const XMR_CUSTODY: CustodyDomain = CustodyDomain(2);
/// XRPL custody signing authority.
pub const XRPL_CUSTODY: CustodyDomain = CustodyDomain(3);
/// `mini_bounty` anonymous developer-bounty payout signing authority.
pub const BOUNTY_PAYOUT_CUSTODY: CustodyDomain = CustodyDomain(4);

/// One domain's currently active custody key state, as recorded locally
/// after a ceremony completes. This crate never fetches or reconciles
/// this against any external chain -- see the crate's top-level "no
/// external chain integration" honest limit; a caller updates it only
/// after independently confirming [`crate::session::completion_confirmed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainKeyStateV1 {
    pub domain: CustodyDomain,
    pub epoch: u64,
    pub group_public_key: [u8; 32],
}

/// In-memory registry of every domain's current key state. Pure
/// bookkeeping/validation -- no persistence, no network.
#[derive(Debug, Default)]
pub struct CustodyDomainRegistry {
    states: BTreeMap<u16, DomainKeyStateV1>,
}

impl CustodyDomainRegistry {
    pub fn new() -> Self {
        CustodyDomainRegistry {
            states: BTreeMap::new(),
        }
    }

    /// The domain's currently recorded key state, or `None` if it has
    /// never completed a ceremony under this registry.
    pub fn state_of(&self, domain: CustodyDomain) -> Option<DomainKeyStateV1> {
        self.states.get(&domain.0).copied()
    }

    /// Validate that `manifest` correctly chains from this registry's
    /// recorded state for its domain: `previous_epoch`/`previous_group_key`
    /// must match this domain's recorded state exactly (or both be zero,
    /// for a domain's first-ever ceremony under this registry), and
    /// `custody_epoch` must be exactly one past `previous_epoch`.
    pub fn validate_chain(&self, manifest: &DkgSessionManifestV1) -> Result<()> {
        match self.states.get(&manifest.custody_domain.0) {
            Some(state) => {
                if manifest.previous_epoch != state.epoch
                    || manifest.previous_group_key != state.group_public_key
                {
                    return Err(CustodyError::InvalidManifest(
                        "manifest does not chain from this domain's recorded key state",
                    ));
                }
            }
            None => {
                if manifest.previous_epoch != 0 || manifest.previous_group_key != [0u8; 32] {
                    return Err(CustodyError::InvalidManifest(
                        "first ceremony for a domain must have zero previous_epoch/previous_group_key",
                    ));
                }
            }
        }
        if manifest.custody_epoch != manifest.previous_epoch + 1 {
            return Err(CustodyError::InvalidManifest(
                "custody_epoch must be exactly previous_epoch + 1",
            ));
        }
        Ok(())
    }

    /// Record a domain's new key state after a ceremony completes. The
    /// caller must have already confirmed
    /// [`crate::session::completion_confirmed`] -- this type has no way
    /// to enforce that itself, since it never sees the attestations.
    pub fn record_completion(
        &mut self,
        domain: CustodyDomain,
        epoch: u64,
        group_public_key: [u8; 32],
    ) {
        self.states.insert(
            domain.0,
            DomainKeyStateV1 {
                domain,
                epoch,
                group_public_key,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_mini::Did;
    use mini_crypto::{encoding, HashAlgorithm, Multihash, SigningKey};

    use crate::manifest::CustodyParticipantV1;

    fn test_did(seed: u8) -> Did {
        let multihash = Multihash::of(HashAlgorithm::Blake3, &[seed]);
        let scid = encoding::encode(encoding::BASE58BTC, &multihash.to_bytes()).unwrap();
        Did::from_scid(&scid).unwrap()
    }

    fn participant(seed: u8) -> CustodyParticipantV1 {
        let device = SigningKey::from_seed(&[seed; 32]);
        let transport = SigningKey::from_seed(&[seed.wrapping_add(100); 32]);
        CustodyParticipantV1 {
            custody_did: test_did(seed),
            device_verifying_key: device.verifying_key(),
            transport_identity_key: transport.verifying_key(),
        }
    }

    fn sample_manifest(
        domain: CustodyDomain,
        previous_epoch: u64,
        previous_group_key: [u8; 32],
    ) -> DkgSessionManifestV1 {
        let mut roster: Vec<CustodyParticipantV1> = (0u8..11).map(participant).collect();
        roster.sort_by(|a, b| a.custody_did.as_str().cmp(b.custody_did.as_str()));
        DkgSessionManifestV1 {
            network_id: [7u8; 32],
            custody_domain: domain,
            custody_epoch: previous_epoch + 1,
            attempt: 0,
            roster,
            authorization_object_id: [9u8; 32],
            previous_epoch,
            previous_group_key,
            software_release_id: [3u8; 32],
            expiry_height: 0,
        }
    }

    #[test]
    fn a_first_ceremony_for_a_domain_must_have_zero_previous_state() {
        let registry = CustodyDomainRegistry::new();
        let manifest = sample_manifest(BTC_CUSTODY, 0, [0u8; 32]);
        registry.validate_chain(&manifest).unwrap();
    }

    #[test]
    fn a_first_ceremony_with_nonzero_previous_state_is_rejected() {
        let registry = CustodyDomainRegistry::new();
        let manifest = sample_manifest(BTC_CUSTODY, 0, [1u8; 32]);
        assert!(registry.validate_chain(&manifest).is_err());
    }

    #[test]
    fn a_rotation_must_chain_from_the_recorded_state() {
        let mut registry = CustodyDomainRegistry::new();
        registry.record_completion(BTC_CUSTODY, 1, [5u8; 32]);
        let good = sample_manifest(BTC_CUSTODY, 1, [5u8; 32]);
        registry.validate_chain(&good).unwrap();

        let bad = sample_manifest(BTC_CUSTODY, 1, [9u8; 32]);
        assert!(registry.validate_chain(&bad).is_err());
    }

    #[test]
    fn domains_are_independent() {
        let mut registry = CustodyDomainRegistry::new();
        registry.record_completion(BTC_CUSTODY, 1, [5u8; 32]);
        // XMR has never completed a ceremony, even though BTC has.
        let xmr_first = sample_manifest(XMR_CUSTODY, 0, [0u8; 32]);
        registry.validate_chain(&xmr_first).unwrap();
        assert!(registry.state_of(XMR_CUSTODY).is_none());
        assert!(registry.state_of(BTC_CUSTODY).is_some());
    }
}
