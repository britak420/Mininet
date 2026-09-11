//! The DKG ceremony's actual steps: manifest acceptance, the Round-1
//! consistent-broadcast barrier, and unanimous completion. Every function
//! here is pure (no I/O, no network) -- exactly the shape
//! `mini_treasury::frost_sign`'s `round1_commit`/`round2_sign` already
//! establish for this tree's other threshold-crypto call sequencing: the
//! caller drives message exchange over whatever real transport it has
//! ([`crate::transport`] for Round 2 specifically), this module only
//! decides whether what came back is valid.
//!
//! DKG math itself ([`dkg_part1`]/[`dkg_part2`]/[`dkg_part3`]) is a thin
//! wrapper translating between this crate's roster/manifest identifiers
//! and `frost_ristretto255::keys::dkg`'s own `part1`/`part2`/`part3` --
//! see this crate's top-level docs for why that library, not a second
//! hand-rolled implementation, does the actual cryptography.

use std::collections::BTreeMap;

use did_mini::Did;
use frost_ristretto255::keys::dkg::{round1, round2};
use frost_ristretto255::keys::{KeyPackage, PublicKeyPackage};
use frost_ristretto255::Identifier;
use mini_crypto::SigningKey;

use crate::error::{CustodyError, Result};
use crate::manifest::DkgSessionManifestV1;
use crate::wire::{domain_hash, push_bytes};

fn identifier_for(manifest: &DkgSessionManifestV1, did: &Did) -> Result<Identifier> {
    let position = manifest
        .frost_identifier_of(did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    Identifier::try_from(position)
        .map_err(|_| CustodyError::InvalidManifest("roster position out of range"))
}

fn frost_err(e: frost_ristretto255::Error) -> CustodyError {
    CustodyError::Frost(e.to_string())
}

// ---------------------------------------------------------------------
// Phase A: manifest acceptance
// ---------------------------------------------------------------------

/// Every roster member signs this before any DKG round begins -- see the
/// crate docs' Phase A. Binds `session_id`, so an acceptance for one
/// ceremony can never be replayed as if it were for another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestAcceptanceV1 {
    pub session_id: [u8; 32],
    pub custody_did: Did,
    pub frost_identifier: u16,
    pub signature: mini_crypto::Signature,
}

fn manifest_acceptance_message(
    session_id: &[u8; 32],
    custody_did: &Did,
    frost_identifier: u16,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"mininet/custody/dkg/manifest-acceptance/v1");
    out.extend_from_slice(session_id);
    push_bytes(&mut out, custody_did.as_str().as_bytes());
    out.extend_from_slice(&frost_identifier.to_be_bytes());
    out
}

/// Sign a [`ManifestAcceptanceV1`] for `manifest` as `custody_did`, whose
/// FROST identifier this manifest assigns.
pub fn sign_manifest_acceptance(
    manifest: &DkgSessionManifestV1,
    custody_did: &Did,
    device_signing_key: &SigningKey,
) -> Result<ManifestAcceptanceV1> {
    let frost_identifier = manifest
        .frost_identifier_of(custody_did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    let session_id = manifest.session_id();
    let message = manifest_acceptance_message(&session_id, custody_did, frost_identifier);
    Ok(ManifestAcceptanceV1 {
        session_id,
        custody_did: custody_did.clone(),
        frost_identifier,
        signature: device_signing_key.sign(&message),
    })
}

/// Verify one [`ManifestAcceptanceV1`] against `manifest`: session binds,
/// the signer really is the roster member it claims, and the roster's
/// recorded FROST identifier for that member matches what was signed.
pub fn verify_manifest_acceptance(
    manifest: &DkgSessionManifestV1,
    acceptance: &ManifestAcceptanceV1,
) -> Result<()> {
    if acceptance.session_id != manifest.session_id() {
        return Err(CustodyError::Round2EnvelopeMisbound);
    }
    let expected_identifier = manifest
        .frost_identifier_of(&acceptance.custody_did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    if expected_identifier != acceptance.frost_identifier {
        return Err(CustodyError::InvalidManifest("frost_identifier mismatch"));
    }
    let participant = &manifest.roster[(expected_identifier - 1) as usize];
    let message = manifest_acceptance_message(
        &acceptance.session_id,
        &acceptance.custody_did,
        acceptance.frost_identifier,
    );
    participant
        .device_verifying_key
        .verify(&message, &acceptance.signature)
        .map_err(|_| CustodyError::BadSignature)
}

/// `true` once every one of `manifest`'s 11 roster members has a valid,
/// matching acceptance in `acceptances`. DKG must not start before this
/// -- see the crate docs' Phase A.
pub fn manifest_fully_accepted(
    manifest: &DkgSessionManifestV1,
    acceptances: &[ManifestAcceptanceV1],
) -> bool {
    if acceptances.len() != manifest.roster.len() {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    for acceptance in acceptances {
        if verify_manifest_acceptance(manifest, acceptance).is_err() {
            return false;
        }
        if !seen.insert(acceptance.custody_did.clone()) {
            return false;
        }
    }
    seen.len() == manifest.roster.len()
}

// ---------------------------------------------------------------------
// Phase B: DKG Round 1
// ---------------------------------------------------------------------

/// Run `frost_ristretto255::keys::dkg::part1` for `own_did` under
/// `manifest`. `rng` is caller-supplied so tests can use a deterministic
/// one and production can use the OS CSPRNG -- the same split every other
/// `random_scalar`-shaped call in this tree already makes.
pub fn dkg_part1<R: rand_core::RngCore + rand_core::CryptoRng>(
    manifest: &DkgSessionManifestV1,
    own_did: &Did,
    rng: R,
) -> Result<(round1::SecretPackage, round1::Package)> {
    let identifier = identifier_for(manifest, own_did)?;
    frost_ristretto255::keys::dkg::part1(
        identifier,
        crate::manifest::SIGNER_COUNT,
        crate::manifest::THRESHOLD,
        rng,
    )
    .map_err(frost_err)
}

// ---------------------------------------------------------------------
// Phase C: consistent Round-1 broadcast
// ---------------------------------------------------------------------

/// `BLAKE3("mininet/custody/dkg/round1-package/v1" || session_id || sender_id || serialized_package)`.
pub fn round1_package_hash(
    session_id: &[u8; 32],
    sender_identifier: u16,
    serialized_package: &[u8],
) -> Result<[u8; 32]> {
    Ok(domain_hash(
        b"mininet/custody/dkg/round1-package/v1",
        &[
            session_id,
            &sender_identifier.to_be_bytes(),
            serialized_package,
        ],
    ))
}

/// The Round-1 consistent-broadcast root: every roster member's package
/// hash, ordered by FROST identifier, hashed together. Every honest
/// participant who received the *same* 11 packages computes the *same*
/// root; a diverging root means some sender equivocated and the ceremony
/// must abort (crate docs' Phase C).
pub fn round1_root(session_id: &[u8; 32], ordered_package_hashes: &[[u8; 32]; 11]) -> [u8; 32] {
    let mut parts: Vec<&[u8]> = vec![session_id];
    for hash in ordered_package_hashes {
        parts.push(hash);
    }
    domain_hash(b"mininet/custody/dkg/round1-root/v1", &parts)
}

/// One roster member's signed acknowledgement that they computed
/// `round1_root` for `session_id`. `part2` must not run until 11 of these,
/// all agreeing on the same root, are collected -- see
/// [`round1_view_confirmed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round1ViewAckV1 {
    pub session_id: [u8; 32],
    pub round1_root: [u8; 32],
    pub custody_did: Did,
    pub signature: mini_crypto::Signature,
}

fn round1_view_ack_message(session_id: &[u8; 32], root: &[u8; 32]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"mininet/custody/dkg/round1-view-ack/v1");
    out.extend_from_slice(session_id);
    out.extend_from_slice(root);
    out
}

pub fn sign_round1_view_ack(
    session_id: [u8; 32],
    round1_root: [u8; 32],
    custody_did: &Did,
    device_signing_key: &SigningKey,
) -> Round1ViewAckV1 {
    let message = round1_view_ack_message(&session_id, &round1_root);
    Round1ViewAckV1 {
        session_id,
        round1_root,
        custody_did: custody_did.clone(),
        signature: device_signing_key.sign(&message),
    }
}

fn verify_round1_view_ack(
    manifest: &DkgSessionManifestV1,
    ack: &Round1ViewAckV1,
    expected_root: &[u8; 32],
) -> Result<()> {
    // Compared directly against the manifest's own session id -- not a
    // caller-supplied "expected session id" a retried ceremony's stale
    // attempt could satisfy. A caller that (by bug or by a malicious
    // coordinator's retry) passed the previous attempt's session id as
    // "expected" would otherwise let that attempt's acks validate against
    // this manifest even though they were never signed for it.
    if ack.session_id != manifest.session_id() || &ack.round1_root != expected_root {
        return Err(CustodyError::Round1ViewMismatch);
    }
    let position = manifest
        .frost_identifier_of(&ack.custody_did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    let participant = &manifest.roster[(position - 1) as usize];
    let message = round1_view_ack_message(&ack.session_id, &ack.round1_root);
    participant
        .device_verifying_key
        .verify(&message, &ack.signature)
        .map_err(|_| CustodyError::BadSignature)
}

/// `true` only if `acks` contains exactly 11 valid acknowledgements, one
/// per distinct roster member, every one of them agreeing on the *same*
/// `expected_root` and bound to `manifest`'s own session id. This is the
/// barrier `part2` must wait behind.
pub fn round1_view_confirmed(
    manifest: &DkgSessionManifestV1,
    acks: &[Round1ViewAckV1],
    expected_root: &[u8; 32],
) -> bool {
    if acks.len() != manifest.roster.len() {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    for ack in acks {
        if verify_round1_view_ack(manifest, ack, expected_root).is_err() {
            return false;
        }
        if !seen.insert(ack.custody_did.clone()) {
            return false;
        }
    }
    seen.len() == manifest.roster.len()
}

/// Proof that the 11-of-11 Round-1 consistent-broadcast barrier held for
/// one exact `(session_id, round1_root)` pair. The only way to construct
/// one is [`confirm_round1_view`], which performs the full
/// [`round1_view_confirmed`] check -- so [`dkg_part2`] requiring this type
/// as a parameter makes the barrier a compile-time requirement, not a
/// convention callers might skip (the gap a Codex review found: this
/// crate's own docs called Round-1 confirmation an "enforced ceremony
/// property," but nothing previously stopped a caller from invoking
/// `dkg_part2` without ever checking `round1_view_confirmed` first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Round1ViewConfirmation {
    session_id: [u8; 32],
    round1_root: [u8; 32],
}

/// The only constructor for [`Round1ViewConfirmation`]. Fails exactly when
/// [`round1_view_confirmed`] would return `false`.
pub fn confirm_round1_view(
    manifest: &DkgSessionManifestV1,
    acks: &[Round1ViewAckV1],
    expected_root: &[u8; 32],
) -> Result<Round1ViewConfirmation> {
    if !round1_view_confirmed(manifest, acks, expected_root) {
        return Err(CustodyError::Round1ViewMismatch);
    }
    Ok(Round1ViewConfirmation {
        session_id: manifest.session_id(),
        round1_root: *expected_root,
    })
}

// ---------------------------------------------------------------------
// Phase D/F: DKG Round 2 / part 3
// ---------------------------------------------------------------------

/// Wraps `frost_ristretto255::keys::dkg::part2`. `round1_packages` must be
/// every *other* roster member's Round-1 package (not this participant's
/// own), keyed by their FROST identifier -- the same map shape
/// `part2`/`part3` themselves require. `confirmation` must have been
/// obtained from [`confirm_round1_view`] for this exact `manifest`
/// (checked below) -- there is no other way to construct a
/// [`Round1ViewConfirmation`], so there is no way to reach this function
/// without having passed the Round-1 barrier first.
pub fn dkg_part2(
    manifest: &DkgSessionManifestV1,
    confirmation: &Round1ViewConfirmation,
    secret_package: round1::SecretPackage,
    round1_packages: &BTreeMap<Identifier, round1::Package>,
) -> Result<(round2::SecretPackage, BTreeMap<Identifier, round2::Package>)> {
    if confirmation.session_id != manifest.session_id() {
        return Err(CustodyError::Round1ViewMismatch);
    }
    frost_ristretto255::keys::dkg::part2(secret_package, round1_packages).map_err(frost_err)
}

/// Wraps `frost_ristretto255::keys::dkg::part3`. Any error here (a
/// missing, invalid, or equivocated package) means the whole ceremony
/// aborts -- see [`crate::abort`] -- never a partial/excluded-continue
/// path.
pub fn dkg_part3(
    round2_secret_package: &round2::SecretPackage,
    round1_packages: &BTreeMap<Identifier, round1::Package>,
    round2_packages: &BTreeMap<Identifier, round2::Package>,
) -> Result<(KeyPackage, PublicKeyPackage)> {
    frost_ristretto255::keys::dkg::part3(round2_secret_package, round1_packages, round2_packages)
        .map_err(frost_err)
}

// ---------------------------------------------------------------------
// Phase G: final key agreement
// ---------------------------------------------------------------------

/// `BLAKE3("mininet/custody/dkg/public-package/v1" || session_id || canonical serialized PublicKeyPackage)`.
pub fn public_package_hash(
    session_id: &[u8; 32],
    public_key_package: &PublicKeyPackage,
) -> Result<[u8; 32]> {
    let serialized = public_key_package.serialize().map_err(frost_err)?;
    Ok(domain_hash(
        b"mininet/custody/dkg/public-package/v1",
        &[session_id, &serialized],
    ))
}

/// One roster member's signed attestation that they reached the given
/// `group_public_key` for `session_id`/`round1_root`. Ceremony success
/// requires exactly 11 of these, all identical apart from `custody_did`
/// and `signature` -- see [`completion_confirmed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionAttestationV1 {
    pub session_id: [u8; 32],
    pub round1_root: [u8; 32],
    pub public_package_hash: [u8; 32],
    pub group_public_key: [u8; 32],
    pub custody_did: Did,
    pub signature: mini_crypto::Signature,
}

fn completion_message(
    session_id: &[u8; 32],
    round1_root: &[u8; 32],
    public_package_hash: &[u8; 32],
    group_public_key: &[u8; 32],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"mininet/custody/dkg/completion/v1");
    out.extend_from_slice(session_id);
    out.extend_from_slice(round1_root);
    out.extend_from_slice(public_package_hash);
    out.extend_from_slice(group_public_key);
    out
}

#[allow(clippy::too_many_arguments)]
pub fn sign_completion_attestation(
    session_id: [u8; 32],
    round1_root: [u8; 32],
    public_package_hash: [u8; 32],
    group_public_key: [u8; 32],
    custody_did: &Did,
    device_signing_key: &SigningKey,
) -> CompletionAttestationV1 {
    let message = completion_message(
        &session_id,
        &round1_root,
        &public_package_hash,
        &group_public_key,
    );
    CompletionAttestationV1 {
        session_id,
        round1_root,
        public_package_hash,
        group_public_key,
        custody_did: custody_did.clone(),
        signature: device_signing_key.sign(&message),
    }
}

fn verify_completion_attestation(
    manifest: &DkgSessionManifestV1,
    attestation: &CompletionAttestationV1,
) -> Result<()> {
    let position = manifest
        .frost_identifier_of(&attestation.custody_did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    let participant = &manifest.roster[(position - 1) as usize];
    let message = completion_message(
        &attestation.session_id,
        &attestation.round1_root,
        &attestation.public_package_hash,
        &attestation.group_public_key,
    );
    participant
        .device_verifying_key
        .verify(&message, &attestation.signature)
        .map_err(|_| CustodyError::BadSignature)
}

/// `true` only if `attestations` has exactly 11 valid, distinct-signer
/// entries that all agree on the *same*
/// session/root/public-package-hash/group-key, and that session is
/// `manifest`'s own. A DKG-generated key is never treated as active
/// custody authority on anything less -- crate docs' Phase G.
pub fn completion_confirmed(
    manifest: &DkgSessionManifestV1,
    attestations: &[CompletionAttestationV1],
) -> bool {
    if attestations.len() != manifest.roster.len() {
        return false;
    }
    let first = match attestations.first() {
        Some(a) => a,
        None => return false,
    };
    // A complete, internally-consistent attestation set from an earlier
    // ceremony attempt (same roster, retried with a new `attempt`) must
    // never be accepted for this manifest just because it's complete and
    // mutually consistent -- it has to actually be an attestation of
    // *this* session.
    if first.session_id != manifest.session_id() {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    for attestation in attestations {
        if attestation.session_id != first.session_id
            || attestation.round1_root != first.round1_root
            || attestation.public_package_hash != first.public_package_hash
            || attestation.group_public_key != first.group_public_key
        {
            return false;
        }
        if verify_completion_attestation(manifest, attestation).is_err() {
            return false;
        }
        if !seen.insert(attestation.custody_did.clone()) {
            return false;
        }
    }
    seen.len() == manifest.roster.len()
}
