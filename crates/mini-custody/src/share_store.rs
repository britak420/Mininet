//! Per-signer encrypted `KeyPackage` persistence.
//!
//! The DKG ceremony's final output for each participant
//! (`frost_ristretto255::keys::KeyPackage`) is long-lived secret key
//! material. Threshold custody's whole point is that any single share, on
//! its own, authorizes nothing (Section 5 of the Gate #93 audit report:
//! 7-of-11) -- but a share sitting at rest on one signer's device is still
//! worth protecting, and losing even one more than `SIGNER_COUNT -
//! THRESHOLD` of them to a single compromised device is the failure mode
//! this exists to slow down. This module wraps a share with an AEAD key
//! stretched from an operator-supplied passphrase via Argon2id -- a
//! genuinely new primitive for this tree (password/passphrase stretching),
//! not a duplicate of anything `mini_crypto` already wraps. Callers write
//! a sealed package only after
//! [`crate::session::completion_confirmed`] succeeds -- never before, and
//! never for a ceremony that aborted.
//!
//! This module never touches `frost_ristretto255` types directly: it
//! seals and opens whatever bytes the caller passes in (the caller's own
//! `KeyPackage::serialize()` output), keeping it independent of the exact
//! FROST ciphersuite in use.

use argon2::Argon2;
use mini_crypto::{AeadKey, AeadNonce, AeadSuite};
use zeroize::Zeroize;

use crate::error::{CustodyError, Result};

/// Argon2 salt length used for wrapping-key derivation.
pub const SALT_LEN: usize = 16;

/// An encrypted, at-rest `KeyPackage` for one signer, one session.
/// `session_id` is authenticated as AEAD associated data (not just stored
/// alongside the ciphertext), so a sealed package can never be silently
/// relabeled as belonging to a different session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedKeyPackageV1 {
    pub session_id: [u8; 32],
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

fn derive_wrapping_key(passphrase: &[u8], salt: &[u8; SALT_LEN]) -> Result<AeadKey> {
    let mut key_bytes = [0u8; 32];
    let result = Argon2::default().hash_password_into(passphrase, salt, &mut key_bytes);
    let key = match result {
        Ok(()) => AeadKey::from_suite_bytes(AeadSuite::ChaCha20Poly1305, &key_bytes)
            .map_err(|_| CustodyError::ShareStorageAuthenticationFailed),
        Err(_) => Err(CustodyError::ShareStorageAuthenticationFailed),
    };
    key_bytes.zeroize();
    key
}

/// Seal `serialized_key_package` under a fresh random salt and nonce,
/// wrapped by `passphrase` via Argon2id, bound to `session_id`.
pub fn seal_key_package(
    session_id: [u8; 32],
    passphrase: &[u8],
    serialized_key_package: &[u8],
) -> Result<SealedKeyPackageV1> {
    // Built directly from a slice of fresh entropy, never through a
    // zeroed-then-overwritten intermediate array: that idiom (`[0u8; N]`
    // followed by `copy_from_slice`) reads to static analysis as a
    // hard-coded value flowing to a cryptographic-salt sink even though
    // the zero bytes are never actually used (CodeQL flagged exactly
    // this at the previous revision of this function).
    let salt: [u8; SALT_LEN] = mini_crypto::random_32().map_err(|_| CustodyError::Entropy)?
        [..SALT_LEN]
        .try_into()
        .expect("SALT_LEN <= 32, the length of random_32()'s output");
    let key = derive_wrapping_key(passphrase, &salt)?;
    let nonce = AeadNonce::generate().map_err(|_| CustodyError::Entropy)?;
    let ciphertext = key
        .encrypt(&nonce, serialized_key_package, &session_id)
        .map_err(|_| CustodyError::ShareStorageAuthenticationFailed)?;
    Ok(SealedKeyPackageV1 {
        session_id,
        salt,
        nonce: nonce.to_bytes(),
        ciphertext,
    })
}

/// Open a [`SealedKeyPackageV1`], recovering the caller's original
/// serialized `KeyPackage` bytes. Fails closed
/// (`ShareStorageAuthenticationFailed`) on a wrong passphrase or any
/// tampering with the ciphertext or `session_id`.
///
/// `expected_session_id` must be supplied by the caller from outside this
/// record — never read from `sealed` itself. `session_id` is authenticated
/// AEAD associated data, so it can't be tampered with in isolation, but a
/// storage attacker doesn't need to: they can replace the *entire* stored
/// object (session id, salt, nonce, and ciphertext together) with a
/// still-genuine, still-decryptable record from an earlier ceremony,
/// silently rolling this signer back to an obsolete share. Requiring the
/// caller's own independently-known expected session id closes that gap —
/// the caller decides which session's share it meant to open, not whatever
/// the storage layer currently hands back.
pub fn open_key_package(
    sealed: &SealedKeyPackageV1,
    expected_session_id: &[u8; 32],
    passphrase: &[u8],
) -> Result<Vec<u8>> {
    if &sealed.session_id != expected_session_id {
        return Err(CustodyError::ShareStorageAuthenticationFailed);
    }
    let key = derive_wrapping_key(passphrase, &sealed.salt)?;
    let nonce = AeadNonce::from_bytes(&sealed.nonce)
        .map_err(|_| CustodyError::ShareStorageAuthenticationFailed)?;
    key.decrypt(&nonce, &sealed.ciphertext, &sealed.session_id)
        .map_err(|_| CustodyError::ShareStorageAuthenticationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sealed_package_opens_with_the_right_passphrase() {
        let session_id = [3u8; 32];
        let payload = b"pretend serialized KeyPackage bytes";
        let sealed =
            seal_key_package(session_id, b"correct horse battery staple", payload).unwrap();
        let opened =
            open_key_package(&sealed, &session_id, b"correct horse battery staple").unwrap();
        assert_eq!(opened, payload);
    }

    #[test]
    fn the_wrong_passphrase_fails_closed() {
        let session_id = [3u8; 32];
        let payload = b"pretend serialized KeyPackage bytes";
        let sealed =
            seal_key_package(session_id, b"correct horse battery staple", payload).unwrap();
        assert!(open_key_package(&sealed, &session_id, b"wrong passphrase").is_err());
    }

    #[test]
    fn relabeling_to_a_different_session_id_fails_closed() {
        let session_id = [3u8; 32];
        let payload = b"pretend serialized KeyPackage bytes";
        let mut sealed =
            seal_key_package(session_id, b"correct horse battery staple", payload).unwrap();
        sealed.session_id = [4u8; 32];
        // Caller still expects the original session -- the relabel is
        // caught by the expected-session check before decryption is even
        // attempted.
        assert!(open_key_package(&sealed, &session_id, b"correct horse battery staple").is_err());
    }

    #[test]
    fn a_whole_record_swapped_in_from_an_earlier_session_is_rejected() {
        // Regression test for a Codex finding: session_id is authenticated
        // AEAD associated data, so it can't be tampered with in isolation
        // -- but a storage attacker doesn't need to tamper with one field,
        // they can replace the *entire* stored object with a still-genuine,
        // still-decryptable record from an earlier ceremony. Without an
        // independently-supplied expected session id, that whole-record
        // swap decrypts successfully and silently rolls the signer back to
        // an obsolete share.
        let old_session = [3u8; 32];
        let new_session = [9u8; 32];
        let old_payload = b"old, obsolete KeyPackage bytes";
        let old_sealed =
            seal_key_package(old_session, b"correct horse battery staple", old_payload).unwrap();

        // The caller (who completed a *new* ceremony) expects to open the
        // new session's share, but a compromised storage layer hands back
        // the old, still-valid-on-its-own-terms record instead.
        let err = open_key_package(&old_sealed, &new_session, b"correct horse battery staple")
            .unwrap_err();
        assert_eq!(err, CustodyError::ShareStorageAuthenticationFailed);
    }

    #[test]
    fn tampered_ciphertext_fails_closed() {
        let session_id = [3u8; 32];
        let payload = b"pretend serialized KeyPackage bytes";
        let mut sealed =
            seal_key_package(session_id, b"correct horse battery staple", payload).unwrap();
        let last = sealed.ciphertext.len() - 1;
        sealed.ciphertext[last] ^= 0xff;
        assert!(open_key_package(&sealed, &session_id, b"correct horse battery staple").is_err());
    }

    #[test]
    fn two_seals_of_the_same_payload_use_different_salts_and_nonces() {
        let session_id = [3u8; 32];
        let payload = b"pretend serialized KeyPackage bytes";
        let a = seal_key_package(session_id, b"pw", payload).unwrap();
        let b = seal_key_package(session_id, b"pw", payload).unwrap();
        assert_ne!(a.salt, b.salt);
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }
}
