//! The fresh-key-per-rotation rule (Gate #93 audit report R93-04).
//!
//! A custody key rotation must produce a **fresh, independently generated**
//! DKG output -- never same-key resharing or refresh. Resharing
//! mathematically re-distributes shares of the *same* secret among a new
//! roster; a departing signer's old share remains a valid share of that
//! same secret the whole time it existed, so same-key resharing never
//! actually revokes anything. Only a fresh key (a brand-new DKG ceremony,
//! [`crate::session`]'s Phase A-G, over a brand-new roster where it
//! changed) revokes a departing signer's authority. The default interval
//! is 180 days (audit report Section 5's policy default) -- enforced by
//! whatever schedules ceremonies, not read from a clock here.

use crate::error::{CustodyError, Result};
use crate::manifest::{CustodyDomain, DkgSessionManifestV1};

/// Default rotation interval, in seconds -- 180 days.
pub const ROTATION_INTERVAL_SECONDS: u64 = 180 * 24 * 60 * 60;

/// The typed record of one domain's custody authority moving from
/// `old_epoch`/`old_group_public_key` to `new_epoch`/`new_group_public_key`.
///
/// This type only carries and validates the claim -- it is not itself an
/// authorization. Per this tree's typed-domain-signing rule (no generic
/// `sign(bytes)`/`finalize(state)`), a real transition is authorized by
/// whatever specific, already-existing governance or ownership mechanism
/// applies to that domain (e.g. `mini_forge::governance` approval for a
/// software-triggered rotation), which signs over this record's fields --
/// not by a new ad hoc signature invented in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustodyKeyTransitionV1 {
    pub domain: CustodyDomain,
    pub old_epoch: u64,
    pub old_group_public_key: [u8; 32],
    pub new_epoch: u64,
    pub new_group_public_key: [u8; 32],
}

impl CustodyKeyTransitionV1 {
    /// Build a transition from a manifest that reached Phase G completion
    /// and the resulting group key -- the ordinary "domain just finished a
    /// DKG ceremony" case.
    pub fn from_completed_ceremony(
        manifest: &DkgSessionManifestV1,
        new_group_public_key: [u8; 32],
    ) -> Self {
        CustodyKeyTransitionV1 {
            domain: manifest.custody_domain,
            old_epoch: manifest.previous_epoch,
            old_group_public_key: manifest.previous_group_key,
            new_epoch: manifest.custody_epoch,
            new_group_public_key,
        }
    }

    /// Reject same-key "rotation" outright -- the entire point of this
    /// type (R93-04). A transition whose new key equals the old key
    /// authorizes nothing: it revokes no departing signer's ability to
    /// sign, since their share of that key remains mathematically valid.
    pub fn validate(&self) -> Result<()> {
        if self.new_epoch <= self.old_epoch {
            return Err(CustodyError::InvalidTransition(
                "new_epoch must be strictly greater than old_epoch",
            ));
        }
        if self.new_group_public_key == self.old_group_public_key {
            return Err(CustodyError::InvalidTransition(
                "a rotation must produce a fresh group key -- same-key resharing/refresh is rejected in production",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::BTC_CUSTODY;

    #[test]
    fn a_genuine_rotation_to_a_fresh_key_validates() {
        let transition = CustodyKeyTransitionV1 {
            domain: BTC_CUSTODY,
            old_epoch: 1,
            old_group_public_key: [1u8; 32],
            new_epoch: 2,
            new_group_public_key: [2u8; 32],
        };
        transition.validate().unwrap();
    }

    #[test]
    fn same_key_resharing_is_rejected() {
        let transition = CustodyKeyTransitionV1 {
            domain: BTC_CUSTODY,
            old_epoch: 1,
            old_group_public_key: [1u8; 32],
            new_epoch: 2,
            new_group_public_key: [1u8; 32],
        };
        assert!(transition.validate().is_err());
    }

    #[test]
    fn a_non_increasing_epoch_is_rejected() {
        let transition = CustodyKeyTransitionV1 {
            domain: BTC_CUSTODY,
            old_epoch: 2,
            old_group_public_key: [1u8; 32],
            new_epoch: 2,
            new_group_public_key: [2u8; 32],
        };
        assert!(transition.validate().is_err());
    }

    #[test]
    fn from_completed_ceremony_carries_manifest_fields_through() {
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

        let mut roster: Vec<CustodyParticipantV1> = (0u8..11).map(participant).collect();
        roster.sort_by(|a, b| a.custody_did.as_str().cmp(b.custody_did.as_str()));
        let manifest = DkgSessionManifestV1 {
            network_id: [7u8; 32],
            custody_domain: BTC_CUSTODY,
            custody_epoch: 2,
            attempt: 0,
            roster,
            authorization_object_id: [9u8; 32],
            previous_epoch: 1,
            previous_group_key: [1u8; 32],
            software_release_id: [3u8; 32],
            expiry_height: 0,
        };
        let new_key = [2u8; 32];
        let transition = CustodyKeyTransitionV1::from_completed_ceremony(&manifest, new_key);
        assert_eq!(transition.domain, BTC_CUSTODY);
        assert_eq!(transition.old_epoch, 1);
        assert_eq!(transition.old_group_public_key, [1u8; 32]);
        assert_eq!(transition.new_epoch, 2);
        assert_eq!(transition.new_group_public_key, new_key);
        transition.validate().unwrap();
    }
}
