//! [`DkgSessionManifestV1`] — the one immutable canonical object every
//! custody DKG ceremony (Gate #93 external audit report, Section 6) binds
//! to. Every later ceremony message (manifest acceptance, Round-1 view
//! acknowledgement, completion attestation) signs over
//! [`DkgSessionManifestV1::session_id`], so nothing about a ceremony's
//! identity, roster, threshold, or purpose can be ambiguous or silently
//! substituted partway through.

use did_mini::Did;
use mini_crypto::VerifyingKey;

use crate::error::{CustodyError, Result};
use crate::wire::{
    domain_hash, push_bytes, push_str, read_bytes, read_str, read_u16, read_u32, read_u64,
};

/// Production custody roster size. Fixed, not configurable per session --
/// a different N/T is a `CustodyPolicy` version bump (the audit report's
/// own Section 5 rule), never a per-ceremony parameter.
pub const SIGNER_COUNT: u16 = 11;

/// Production custody signing threshold.
pub const THRESHOLD: u16 = 7;

/// The FROST ciphersuite tag this crate's ceremony produces keys for.
/// Wire-stable string, not a `#[non_exhaustive]` enum, because it is
/// compared byte-for-byte against manifest bytes from peers, not matched
/// in Rust code paths (there is exactly one production suite; see the
/// crypto audit report's own Section 5.1/5.18 "no admin migration switch"
/// requirement for why a second suite is a new manifest *version*, not a
/// runtime choice here).
pub const FROST_SUITE_ID: &str = "FROST_RISTRETTO255_SHA512_V1";

/// One custody domain: an independent FROST group key, independent
/// rotation history, independent everything -- see [`crate::domains`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CustodyDomain(pub u16);

/// One roster member. Two distinct keys are deliberate: `device_verifying_key`
/// authenticates *protocol messages* this participant signs (manifest
/// acceptance, view acks, completion attestations); `transport_identity_key`
/// is what [`crate::transport`] binds the underlying encrypted channel's
/// identity to. `mini_bearer::Channel`'s own docs are explicit that the
/// channel itself authenticates nothing about the peer ("not endpoint
/// authentication, by design") -- binding happens one layer up, using
/// this key, not by assuming the channel implies identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodyParticipantV1 {
    pub custody_did: Did,
    pub device_verifying_key: VerifyingKey,
    pub transport_identity_key: VerifyingKey,
}

impl CustodyParticipantV1 {
    fn encode(&self, out: &mut Vec<u8>) {
        push_str(out, self.custody_did.as_str());
        push_bytes(out, &self.device_verifying_key.to_bytes());
        push_bytes(out, &self.transport_identity_key.to_bytes());
    }

    fn decode(input: &mut &[u8]) -> Result<Self> {
        let did = Did::parse(read_str(input)?)
            .map_err(|_| CustodyError::InvalidManifest("malformed custody_did"))?;
        let device_bytes = read_bytes(input)?;
        let device_verifying_key =
            VerifyingKey::from_suite_bytes(mini_crypto::SignatureSuite::Ed25519, device_bytes)
                .map_err(|_| CustodyError::InvalidManifest("malformed device_verifying_key"))?;
        let transport_bytes = read_bytes(input)?;
        let transport_identity_key =
            VerifyingKey::from_suite_bytes(mini_crypto::SignatureSuite::Ed25519, transport_bytes)
                .map_err(|_| CustodyError::InvalidManifest("malformed transport_identity_key"))?;
        Ok(CustodyParticipantV1 {
            custody_did: did,
            device_verifying_key,
            transport_identity_key,
        })
    }
}

/// The canonical, immutable ceremony identity -- Section 6 of the Gate #93
/// external audit report. Every field a real ceremony needs to be
/// unambiguous about before any DKG round runs at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DkgSessionManifestV1 {
    pub network_id: [u8; 32],
    pub custody_domain: CustodyDomain,
    pub custody_epoch: u64,
    pub attempt: u32,
    /// Exactly [`SIGNER_COUNT`] entries, in canonical roster order
    /// (lexicographic by `custody_did` SCID bytes -- see
    /// [`DkgSessionManifestV1::validate`]). Position in this vector
    /// (0-indexed) plus one is the participant's FROST identifier: the
    /// first entry is identifier `1`, ..., the eleventh is identifier
    /// `11`. Identifier `0` is structurally impossible to reach: nothing
    /// in this manifest, or in `frost_ristretto255::Identifier`'s own
    /// `TryFrom<u16>`, can ever produce it.
    pub roster: Vec<CustodyParticipantV1>,
    pub authorization_object_id: [u8; 32],
    pub previous_epoch: u64,
    /// Zero only at the very first epoch for this domain.
    pub previous_group_key: [u8; 32],
    pub software_release_id: [u8; 32],
    /// Zero only for a pre-genesis ceremony.
    pub expiry_height: u64,
}

impl DkgSessionManifestV1 {
    /// Canonical wire bytes. `session_id` hashes exactly this.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"mininet-custody-dkg");
        out.push(1u8); // version
        out.extend_from_slice(&self.network_id);
        out.extend_from_slice(&self.custody_domain.0.to_be_bytes());
        out.extend_from_slice(&self.custody_epoch.to_be_bytes());
        out.extend_from_slice(&self.attempt.to_be_bytes());
        out.extend_from_slice(&SIGNER_COUNT.to_be_bytes());
        out.extend_from_slice(&THRESHOLD.to_be_bytes());
        push_str(&mut out, FROST_SUITE_ID);
        out.extend_from_slice(&(self.roster.len() as u16).to_be_bytes());
        for participant in &self.roster {
            participant.encode(&mut out);
        }
        out.extend_from_slice(&self.authorization_object_id);
        out.extend_from_slice(&self.previous_epoch.to_be_bytes());
        out.extend_from_slice(&self.previous_group_key);
        out.extend_from_slice(&self.software_release_id);
        out.extend_from_slice(&self.expiry_height.to_be_bytes());
        out
    }

    /// Parse and fully validate a manifest from canonical wire bytes --
    /// see [`Self::validate`] for exactly what "valid" means. Trailing
    /// bytes are rejected.
    pub fn decode(mut input: &[u8]) -> Result<Self> {
        let original = input;
        if input.len() < 19 || &input[..19] != b"mininet-custody-dkg" {
            return Err(CustodyError::InvalidManifest("bad magic"));
        }
        input = &input[19..];
        if input.first() != Some(&1u8) {
            return Err(CustodyError::InvalidManifest("unsupported version"));
        }
        input = &input[1..];
        let network_id = crate::wire::read_array::<32>(&mut input)?;
        let custody_domain = CustodyDomain(read_u16(&mut input)?);
        let custody_epoch = read_u64(&mut input)?;
        let attempt = read_u32(&mut input)?;
        let signer_count = read_u16(&mut input)?;
        let threshold = read_u16(&mut input)?;
        let suite = read_str(&mut input)?;
        if suite != FROST_SUITE_ID {
            return Err(CustodyError::InvalidManifest("unsupported suite"));
        }
        let roster_len = read_u16(&mut input)? as usize;
        // Rejected immediately, before allocating or parsing a single
        // roster entry: `validate()` rejects any length other than
        // `SIGNER_COUNT` anyway, but only after every entry has already
        // been parsed. Without this, an untrusted manifest can declare
        // `roster_len = 65_535` and force tens of thousands of DID/key
        // parses (each one real allocation and Ed25519 decoding work)
        // before hitting that inevitable rejection.
        if roster_len != SIGNER_COUNT as usize {
            return Err(CustodyError::InvalidManifest("roster size != 11"));
        }
        let mut roster = Vec::with_capacity(roster_len);
        for _ in 0..roster_len {
            roster.push(CustodyParticipantV1::decode(&mut input)?);
        }
        let authorization_object_id = crate::wire::read_array::<32>(&mut input)?;
        let previous_epoch = read_u64(&mut input)?;
        let previous_group_key = crate::wire::read_array::<32>(&mut input)?;
        let software_release_id = crate::wire::read_array::<32>(&mut input)?;
        let expiry_height = read_u64(&mut input)?;
        if !input.is_empty() {
            return Err(CustodyError::InvalidManifest("trailing bytes"));
        }

        let manifest = DkgSessionManifestV1 {
            network_id,
            custody_domain,
            custody_epoch,
            attempt,
            roster,
            authorization_object_id,
            previous_epoch,
            previous_group_key,
            software_release_id,
            expiry_height,
        };

        if signer_count != SIGNER_COUNT || threshold != THRESHOLD {
            return Err(CustodyError::InvalidManifest(
                "signer_count/threshold mismatch",
            ));
        }
        manifest.validate()?;

        // Round-trip check: decode must reproduce exactly these bytes, so
        // a manifest is never accepted in a form that would sign/hash
        // differently than its own canonical encoding (the same
        // discipline `mini-mesh`/`mini-private-payment` canonical codecs
        // already apply).
        if manifest.encode() != original {
            return Err(CustodyError::InvalidManifest("non-canonical encoding"));
        }

        Ok(manifest)
    }

    /// Structural validation beyond "parses": exact roster size/threshold,
    /// canonical (lexicographic-by-DID) roster order, no duplicate DID or
    /// device/transport key across the roster. Does **not** check
    /// `previous_group_key`/`previous_epoch` against any external state
    /// (the ceremony driver -- not this pure data type -- is what knows
    /// the domain's actual current epoch) or `expiry_height` against a
    /// live chain height (same reason).
    pub fn validate(&self) -> Result<()> {
        if !crate::domains::is_known_domain(self.custody_domain) {
            return Err(CustodyError::InvalidManifest(
                "custody_domain is not one of the declared production domains",
            ));
        }
        if self.roster.len() != SIGNER_COUNT as usize {
            return Err(CustodyError::InvalidManifest("roster size != 11"));
        }
        for window in self.roster.windows(2) {
            if window[0].custody_did.as_str() >= window[1].custody_did.as_str() {
                return Err(CustodyError::InvalidManifest(
                    "roster not in canonical lexicographic DID order",
                ));
            }
        }
        // `CustodyParticipantV1::encode`/`decode` only round-trip Ed25519
        // key bytes (`decode` always reconstructs via
        // `VerifyingKey::from_suite_bytes(SignatureSuite::Ed25519, ..)`,
        // dropping whatever suite the key actually was). A roster built
        // in-process with e.g. an `MlDsa65` key would validate fine here
        // but fail to decode on every peer that receives the encoded wire
        // bytes -- reject the mismatch immediately, at the same layer that
        // already validates the roster, instead of letting it surface
        // later as a confusing decode failure on someone else's node.
        for participant in &self.roster {
            if participant.device_verifying_key.suite() != mini_crypto::SignatureSuite::Ed25519
                || participant.transport_identity_key.suite()
                    != mini_crypto::SignatureSuite::Ed25519
            {
                return Err(CustodyError::InvalidManifest(
                    "roster key suite is not Ed25519 -- the only suite this manifest's wire format can encode",
                ));
            }
        }
        // Uniqueness is checked across the *union* of both key roles, not
        // independently within each: device and transport keys authorize
        // different things (protocol-message signing vs. transport-channel
        // identity, see `CustodyParticipantV1`'s docs), so one
        // participant's transport key appearing as another's device key
        // would hand the first participant ceremony-signing authority
        // under the second's identity -- checking the two sets separately
        // missed exactly that cross-role reuse.
        let mut seen_keys = std::collections::BTreeSet::new();
        for participant in &self.roster {
            if !seen_keys.insert(participant.device_verifying_key.to_bytes()) {
                return Err(CustodyError::InvalidManifest(
                    "a roster key (device or transport) is reused",
                ));
            }
            if !seen_keys.insert(participant.transport_identity_key.to_bytes()) {
                return Err(CustodyError::InvalidManifest(
                    "a roster key (device or transport) is reused",
                ));
            }
        }
        Ok(())
    }

    /// This participant's FROST identifier: `1 + position in roster`.
    /// Returns `None` if `did` is not a roster member.
    pub fn frost_identifier_of(&self, did: &Did) -> Option<u16> {
        self.roster
            .iter()
            .position(|p| &p.custody_did == did)
            .map(|pos| (pos + 1) as u16)
    }

    /// The session ID every signed ceremony message binds to:
    /// `BLAKE3("mininet/custody/dkg-session/v1" || canonical manifest bytes)`.
    pub fn session_id(&self) -> [u8; 32] {
        domain_hash(b"mininet/custody/dkg-session/v1", &[&self.encode()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mini_crypto::{encoding, Multihash, SigningKey};

    /// A real, `validate_scid`-passing test DID: an actual strong
    /// multihash (BLAKE3 over `seed`) wrapped in base58btc, exactly the
    /// same construction any real `did:mini` SCID uses -- not a
    /// hand-typed string guessing at the wire format.
    fn test_did(seed: u8) -> Did {
        let multihash = Multihash::of(mini_crypto::HashAlgorithm::Blake3, &[seed]);
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

    fn sample_manifest() -> DkgSessionManifestV1 {
        let mut roster: Vec<CustodyParticipantV1> = (0u8..11).map(participant).collect();
        roster.sort_by(|a, b| a.custody_did.as_str().cmp(b.custody_did.as_str()));
        roster.dedup_by(|a, b| a.custody_did == b.custody_did);
        // The synthetic DIDs above are already guaranteed distinct by
        // construction (seed folded into the SCID tail), so this should
        // never actually drop anyone; asserted so a future change to
        // `participant()` can't silently shrink the roster below 11.
        assert_eq!(
            roster.len(),
            11,
            "synthetic roster must have 11 distinct DIDs"
        );
        DkgSessionManifestV1 {
            network_id: [7u8; 32],
            custody_domain: CustodyDomain(1),
            custody_epoch: 0,
            attempt: 0,
            roster,
            authorization_object_id: [9u8; 32],
            previous_epoch: 0,
            previous_group_key: [0u8; 32],
            software_release_id: [3u8; 32],
            expiry_height: 0,
        }
    }

    #[test]
    fn a_valid_manifest_round_trips_through_encode_decode() {
        let manifest = sample_manifest();
        let bytes = manifest.encode();
        let decoded = DkgSessionManifestV1::decode(&bytes).unwrap();
        assert_eq!(decoded, manifest);
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let manifest = sample_manifest();
        let mut bytes = manifest.encode();
        bytes.push(0xff);
        assert!(DkgSessionManifestV1::decode(&bytes).is_err());
    }

    #[test]
    fn wrong_roster_size_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.roster.pop();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn an_oversized_declared_roster_len_is_rejected_before_parsing_any_entry() {
        // Regression test for a Codex finding: `decode` used to allocate and
        // parse `roster_len` entries before ever checking that value
        // against SIGNER_COUNT, so an untrusted manifest declaring
        // roster_len = 65_535 forced real DID/key-parsing work for every
        // one of them before the inevitable rejection. Hand-build a prefix
        // matching `encode`'s exact field order, with a huge roster_len and
        // zero actual roster bytes following it: the fix must reject with
        // "roster size != 11" immediately, not some unrelated truncation
        // error from trying (and failing) to parse a first entry.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"mininet-custody-dkg");
        bytes.push(1u8); // version
        bytes.extend_from_slice(&[7u8; 32]); // network_id
        bytes.extend_from_slice(&1u16.to_be_bytes()); // custody_domain
        bytes.extend_from_slice(&1u64.to_be_bytes()); // custody_epoch
        bytes.extend_from_slice(&0u32.to_be_bytes()); // attempt
        bytes.extend_from_slice(&SIGNER_COUNT.to_be_bytes());
        bytes.extend_from_slice(&THRESHOLD.to_be_bytes());
        crate::wire::push_str(&mut bytes, FROST_SUITE_ID);
        bytes.extend_from_slice(&u16::MAX.to_be_bytes()); // roster_len: absurd
                                                          // No roster entries, no trailing fields at all.

        let err = DkgSessionManifestV1::decode(&bytes).unwrap_err();
        assert_eq!(err, CustodyError::InvalidManifest("roster size != 11"));
    }

    #[test]
    fn a_non_ed25519_roster_key_is_rejected() {
        // Regression test for a Codex finding: encode/decode only round-trip
        // Ed25519 key bytes, silently reconstructing anything else as
        // Ed25519 on decode (which then fails on length alone on every real
        // peer). Reject the mismatch here, at validate(), rather than
        // letting a roster with a non-Ed25519 key look valid in-process.
        let mut manifest = sample_manifest();
        let ml_dsa = mini_crypto::SigningKey::generate_ml_dsa_65()
            .unwrap()
            .verifying_key();
        manifest.roster[0].device_verifying_key = ml_dsa;
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn non_canonical_roster_order_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.roster.swap(0, 1);
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn an_undeclared_custody_domain_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.custody_domain = CustodyDomain(5); // not one of the four declared domains
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn every_declared_production_domain_validates() {
        for domain in [
            crate::domains::BTC_CUSTODY,
            crate::domains::XMR_CUSTODY,
            crate::domains::XRPL_CUSTODY,
            crate::domains::BOUNTY_PAYOUT_CUSTODY,
        ] {
            let mut manifest = sample_manifest();
            manifest.custody_domain = domain;
            manifest.validate().unwrap();
        }
    }

    #[test]
    fn reusing_one_participants_transport_key_as_anothers_device_key_is_rejected() {
        let mut manifest = sample_manifest();
        // Participant 1's device key becomes participant 0's transport key:
        // same key, two different roles, two different participants.
        let stolen = manifest.roster[1].device_verifying_key.clone();
        manifest.roster[0].transport_identity_key = stolen;
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn any_field_change_changes_the_session_id() {
        let a = sample_manifest();
        let mut b = a.clone();
        b.attempt += 1;
        assert_ne!(a.session_id(), b.session_id());
    }

    #[test]
    fn frost_identifiers_map_1_through_11_in_roster_order() {
        let manifest = sample_manifest();
        for (i, participant) in manifest.roster.iter().enumerate() {
            assert_eq!(
                manifest.frost_identifier_of(&participant.custody_did),
                Some((i + 1) as u16)
            );
        }
    }

    #[test]
    fn unknown_did_has_no_identifier() {
        let manifest = sample_manifest();
        let stranger = test_did(200);
        assert_eq!(manifest.frost_identifier_of(&stranger), None);
    }
}
