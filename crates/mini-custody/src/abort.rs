//! The ceremony's only failure-handling path.
//!
//! The old `mini_treasury::frost_dkg` module had a public complaint/
//! rebuttal mechanism (an accused participant could be excluded from a
//! DKG run if enough other participants agreed) -- and that mechanism's
//! `accuser` field was exactly where the Gate #93 audit's second critical
//! finding lived (F93-02: an unvalidated index-0 accuser). Rather than
//! carry that whole exclusion-voting surface forward more carefully, this
//! crate removes it: any invalid, missing, ambiguous, or mismatched input
//! at any phase aborts the *entire* ceremony, restarting from a fresh
//! manifest (new `attempt`, new randomness) with the full roster. A
//! treasury DKG ceremony is infrequent (Section 5's 180-day rotation
//! cadence) with a known, governance-approved roster; the audit report's
//! own reasoning (R93-05) is that keeping the secret-handling surface
//! minimal is worth more here than the liveness a partial-exclusion path
//! would buy.
//!
//! [`AbortNoticeV1`] is the one signal this module defines: one roster
//! member's signed claim that the ceremony identified by `session_id`
//! should stop. Unlike [`crate::session::manifest_fully_accepted`] or
//! [`crate::session::completion_confirmed`], a *single* valid notice is
//! enough -- there is no quorum to reach and no rebuttal to wait for.

use did_mini::Did;
use mini_crypto::SigningKey;

use crate::error::{CustodyError, Result};
use crate::manifest::DkgSessionManifestV1;
use crate::wire::push_str;

/// Why a ceremony aborted. Deliberately coarse and purely diagnostic: no
/// reason here ever excludes a specific participant from a future
/// ceremony attempt -- see the module docs for why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortReason {
    /// Fewer than 11-of-11 roster members accepted the manifest.
    ManifestAcceptanceIncomplete,
    /// The Round-1 consistent-broadcast view did not match across the
    /// roster.
    Round1ViewMismatch,
    /// A Round-1 package failed `frost_ristretto255`'s own
    /// proof-of-knowledge check.
    Round1PackageInvalid,
    /// A Round-2 package failed to verify against its sender's Round-1
    /// commitment.
    Round2PackageInvalid,
    /// Fewer than 11-of-11 matching completion attestations were
    /// collected.
    CompletionIncomplete,
    /// The transport layer ([`crate::transport`]) could not deliver or
    /// authenticate a message.
    TransportFailure,
    /// A caller-defined ceremony deadline passed before the roster
    /// finished a phase.
    Timeout,
}

impl AbortReason {
    fn tag(self) -> &'static str {
        match self {
            AbortReason::ManifestAcceptanceIncomplete => "manifest-acceptance-incomplete",
            AbortReason::Round1ViewMismatch => "round1-view-mismatch",
            AbortReason::Round1PackageInvalid => "round1-package-invalid",
            AbortReason::Round2PackageInvalid => "round2-package-invalid",
            AbortReason::CompletionIncomplete => "completion-incomplete",
            AbortReason::TransportFailure => "transport-failure",
            AbortReason::Timeout => "timeout",
        }
    }
}

fn abort_notice_message(session_id: &[u8; 32], reason: AbortReason) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"mininet/custody/dkg/abort-notice/v1");
    out.extend_from_slice(session_id);
    push_str(&mut out, reason.tag());
    out
}

/// One roster member's signed claim that `session_id`'s ceremony should
/// stop, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbortNoticeV1 {
    pub session_id: [u8; 32],
    pub reason: AbortReason,
    pub custody_did: Did,
    pub signature: mini_crypto::Signature,
}

/// Sign an [`AbortNoticeV1`] for `manifest`'s session as `custody_did`.
pub fn sign_abort_notice(
    manifest: &DkgSessionManifestV1,
    custody_did: &Did,
    reason: AbortReason,
    device_signing_key: &SigningKey,
) -> Result<AbortNoticeV1> {
    manifest
        .frost_identifier_of(custody_did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    let session_id = manifest.session_id();
    let message = abort_notice_message(&session_id, reason);
    Ok(AbortNoticeV1 {
        session_id,
        reason,
        custody_did: custody_did.clone(),
        signature: device_signing_key.sign(&message),
    })
}

/// Verify one [`AbortNoticeV1`] against `manifest`: session binds and the
/// signer really is a roster member.
pub fn verify_abort_notice(manifest: &DkgSessionManifestV1, notice: &AbortNoticeV1) -> Result<()> {
    if notice.session_id != manifest.session_id() {
        return Err(CustodyError::Round2EnvelopeMisbound);
    }
    let position = manifest
        .frost_identifier_of(&notice.custody_did)
        .ok_or(CustodyError::InvalidManifest("DID is not a roster member"))?;
    let participant = &manifest.roster[(position - 1) as usize];
    let message = abort_notice_message(&notice.session_id, notice.reason);
    participant
        .device_verifying_key
        .verify(&message, &notice.signature)
        .map_err(|_| CustodyError::BadSignature)
}

/// `true` if any single valid [`AbortNoticeV1`] exists for `manifest`'s
/// session. One honest participant is enough to stop a ceremony -- there
/// is no rebuttal and no exclusion vote (see module docs).
pub fn any_abort_confirmed(manifest: &DkgSessionManifestV1, notices: &[AbortNoticeV1]) -> bool {
    notices
        .iter()
        .any(|notice| verify_abort_notice(manifest, notice).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mini_crypto::{encoding, HashAlgorithm, Multihash, SigningKey};

    use crate::manifest::{CustodyDomain, CustodyParticipantV1, DkgSessionManifestV1};

    fn test_did(seed: u8) -> Did {
        let multihash = Multihash::of(HashAlgorithm::Blake3, &[seed]);
        let scid = encoding::encode(encoding::BASE58BTC, &multihash.to_bytes()).unwrap();
        Did::from_scid(&scid).unwrap()
    }

    fn participant(seed: u8) -> (CustodyParticipantV1, SigningKey) {
        let device = SigningKey::from_seed(&[seed; 32]);
        let transport = SigningKey::from_seed(&[seed.wrapping_add(100); 32]);
        (
            CustodyParticipantV1 {
                custody_did: test_did(seed),
                device_verifying_key: device.verifying_key(),
                transport_identity_key: transport.verifying_key(),
            },
            device,
        )
    }

    fn sample_manifest_and_keys() -> (DkgSessionManifestV1, Vec<SigningKey>) {
        let mut entries: Vec<_> = (0u8..11).map(participant).collect();
        entries.sort_by(|a, b| a.0.custody_did.as_str().cmp(b.0.custody_did.as_str()));
        let roster = entries.iter().map(|e| e.0.clone()).collect();
        let keys = entries.into_iter().map(|e| e.1).collect();
        let manifest = DkgSessionManifestV1 {
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
        };
        (manifest, keys)
    }

    #[test]
    fn a_single_valid_notice_confirms_abort() {
        let (manifest, keys) = sample_manifest_and_keys();
        let notice = sign_abort_notice(
            &manifest,
            &manifest.roster[3].custody_did,
            AbortReason::Round1ViewMismatch,
            &keys[3],
        )
        .unwrap();
        assert!(any_abort_confirmed(&manifest, &[notice]));
    }

    #[test]
    fn no_notices_means_no_abort() {
        let (manifest, _keys) = sample_manifest_and_keys();
        assert!(!any_abort_confirmed(&manifest, &[]));
    }

    #[test]
    fn a_notice_for_a_different_session_does_not_confirm() {
        let (manifest, keys) = sample_manifest_and_keys();
        let mut notice = sign_abort_notice(
            &manifest,
            &manifest.roster[0].custody_did,
            AbortReason::Timeout,
            &keys[0],
        )
        .unwrap();
        notice.session_id = [0xffu8; 32];
        assert!(!any_abort_confirmed(&manifest, &[notice]));
    }

    #[test]
    fn a_notice_from_a_non_roster_did_is_rejected() {
        let (manifest, _keys) = sample_manifest_and_keys();
        let stranger_key = SigningKey::from_seed(&[200u8; 32]);
        let stranger = test_did(201);
        assert!(
            sign_abort_notice(&manifest, &stranger, AbortReason::Timeout, &stranger_key).is_err()
        );
    }
}
