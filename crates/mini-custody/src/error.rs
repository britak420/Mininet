//! Errors for the custody DKG ceremony, transport binding, share storage,
//! and rotation.

/// Result alias for this crate.
pub type Result<T> = core::result::Result<T, CustodyError>;

/// A custody-ceremony or custody-storage failure. Every variant here is a
/// **fail-closed abort** signal, never a partial-success one — see the
/// crate's `abort` module docs for why a DKG ceremony has no "continue
/// after this" path.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CustodyError {
    /// A [`crate::manifest::DkgSessionManifestV1`] field failed validation
    /// (wrong signer count/threshold, non-canonical roster order, zero
    /// FROST identifier, etc).
    InvalidManifest(&'static str),
    /// Fewer than every roster member accepted the manifest before DKG was
    /// asked to start.
    IncompleteManifestAcceptance,
    /// A signature over a signed ceremony message (acceptance, view-ack,
    /// completion attestation, transition authorization) did not verify.
    BadSignature,
    /// The underlying `frost_ristretto255` DKG step failed. Carries no
    /// further detail beyond what the library itself exposed (already
    /// redacted of any secret material by construction — DKG errors never
    /// carry scalars).
    Frost(String),
    /// A Round-1 package hash did not match what a peer claimed, or fewer
    /// than the full roster produced matching Round-1 view
    /// acknowledgements — the "everyone must see the same broadcast"
    /// barrier failed.
    Round1ViewMismatch,
    /// A Round-2 envelope arrived that does not bind to the session in
    /// progress (wrong session ID, wrong Round-1 root, wrong sender/
    /// receiver identity).
    Round2EnvelopeMisbound,
    /// The underlying `mini_bearer::channel::Channel` seal/open failed
    /// (AEAD authentication failure, counter exhaustion, oversize frame).
    /// Carries the error's message rather than the error itself so
    /// `CustodyError` can stay `Clone`/`PartialEq`/`Eq`.
    Transport(String),
    /// Fewer than 11-of-11 completion attestations matched exactly.
    IncompleteCompletion,
    /// A share-storage AEAD open failed (wrong wrapping key, or the
    /// ciphertext/header was tampered with).
    ShareStorageAuthenticationFailed,
    /// A [`crate::rotation::CustodyKeyTransitionV1`] field failed
    /// validation (old/new key mismatch, wrong domain, etc).
    InvalidTransition(&'static str),
    /// The local OS CSPRNG failed. Never falls back to a weaker source —
    /// see `mini_crypto::random_32`'s own contract.
    Entropy,
    /// A [`crate::signing::DurableCustodySigner`] on-disk nonce-commitment
    /// journal operation failed: I/O error, corrupt/truncated record,
    /// checksum mismatch, capacity exceeded, or the signer instance no
    /// longer matches what the journal was opened against. Always
    /// fail-closed — see that module's docs for why a durable signer never
    /// tries to "repair" a journal it cannot fully account for.
    SigningJournal(String),
}

impl core::fmt::Display for CustodyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CustodyError::InvalidManifest(reason) => {
                write!(f, "invalid DKG session manifest: {reason}")
            }
            CustodyError::IncompleteManifestAcceptance => {
                write!(f, "not every roster member accepted the manifest")
            }
            CustodyError::BadSignature => write!(f, "signature verification failed"),
            CustodyError::Frost(msg) => write!(f, "FROST DKG error: {msg}"),
            CustodyError::Round1ViewMismatch => {
                write!(f, "Round-1 consistent-broadcast view mismatch")
            }
            CustodyError::Round2EnvelopeMisbound => {
                write!(f, "Round-2 envelope does not bind to this session")
            }
            CustodyError::Transport(msg) => write!(f, "transport channel error: {msg}"),
            CustodyError::IncompleteCompletion => {
                write!(f, "fewer than 11-of-11 matching completion attestations")
            }
            CustodyError::ShareStorageAuthenticationFailed => {
                write!(f, "share storage authentication failed")
            }
            CustodyError::InvalidTransition(reason) => {
                write!(f, "invalid custody key transition: {reason}")
            }
            CustodyError::Entropy => write!(f, "OS entropy source failed"),
            CustodyError::SigningJournal(msg) => write!(f, "signing journal error: {msg}"),
        }
    }
}

impl std::error::Error for CustodyError {}
