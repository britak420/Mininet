//! Community-governed BTC/XMR-to-MINI contribution bookkeeping and
//! threshold-signature custody (whitepaper §8.2 "how the rich contribute" /
//! §10 treasury custody).
//!
//! **Safe to build now (ordinary bookkeeping and arithmetic):**
//!
//! - [`rate`] — a governed exchange-rate history and the multiplication
//!   that turns a contribution into a minted amount at whatever rate was in
//!   effect. Has no opinion on how the rate is set (ordinary flat-vote
//!   governance) or whether a contribution actually arrived.
//! - [`receipt::ContributionReceipt`] — the bookkeeping record of a claimed
//!   contribution (asset, amount, rate, minted MINI).
//! - [`signers`] — **who** is authorized to approve treasury actions and
//!   whether enough of them agreed, mirroring `mini_forge`'s governance
//!   approval-counting pattern: distinct-identity counting only, no
//!   weight field, no path to extra voting power for being a signer (P1
//!   unchanged). This is identity-level authorization ("is this person on
//!   the committee"), a separate question from [`frost_sign`]'s
//!   cryptographic signing ("here is a valid signature the committee
//!   produced").
//!
//! **Founder-overridden (D-0037), AI-authored prototype — real threshold-
//! signature custody, not a stub, but explicitly pending external
//! cryptography audit:**
//!
//! - [`frost_keygen`]/[`frost_sign`] — FROST (Flexible Round-Optimized
//!   Schnorr Threshold signatures, Komlo & Goldberg): any `threshold`-sized
//!   subset of a committee can jointly produce one ordinary Schnorr
//!   signature under a shared group public key, without any single device
//!   ever holding the full secret key. See `examples/frost_live_demo.rs`
//!   for a runnable multi-device signing session over real (simulated)
//!   message-passing.
//! - [`frost_dkg`] — real distributed key generation (Pedersen DKG with
//!   Feldman VSS, following FROST KeyGen from the original FROST paper —
//!   **not** RFC 9591, whose scope is signing and which excludes key
//!   generation; see the module's own docs): every participant runs an
//!   independent Feldman VSS of their own random value, so no single
//!   device — not even the "dealer," since there is no dealer — ever
//!   holds the full group secret. Includes a complaint/rebuttal exclusion
//!   mechanism so one misbehaving participant cannot indefinitely block
//!   the honest majority from completing key generation (D-0059/D-0060,
//!   closing D-0048's DKG half; index-0 disclosure bug fixed by D-0506).
//! - [`frost_reshare`] — committee rotation: an active old-committee subset
//!   redistributes shares of the *same* group secret to a new committee,
//!   without ever reconstructing that secret and without ever changing its
//!   public key (checked directly, not just algebraically). Does **not**
//!   revoke the old committee's shares — see the module's own honest
//!   limit.
//!
//! **`frost_dkg`/`frost_reshare` are behind the `legacy-hand-rolled-dkg`
//! feature, off by default (D-0507):** an external audit found this crate
//! extending a second, bespoke DKG implementation finding by finding
//! rather than composing an already-audited one. `mini-custody` is the
//! production DKG ceremony layer now, wrapping `frost_ristretto255::keys
//! ::dkg` (NCC-Group-audited). This feature exists only to keep the old
//! implementation's own test/historical coverage buildable, not as a
//! recommendation to reach for it in new work.
//!
//! **`frost_sign` is likewise behind `legacy-hand-rolled-signing`, off by
//! default (Gate #72):** the same audit pattern, one layer over — this
//! crate re-derives the two-round FROST *signing* protocol (binding
//! factors, Lagrange interpolation, the Schnorr challenge) from raw
//! `curve25519-dalek` arithmetic rather than composing
//! `frost_ristretto255::round1`/`round2`/`aggregate` directly.
//! `mini_custody::signing` is the production signing layer now, including
//! a durable nonce-commitment journal carrying forward `frost_sign`'s own
//! `DurableFrostSigner` crash-safety property.
//!
//! **Still deliberately not built here (whitepaper: "a permanent honeypot
//! by nature"; D-0035 point 5's external-audit requirement stands even
//! under D-0037's authorship policy change for this specific gap):**
//!
//! - [`receipt::ExternalReceiptOracle`] — verifying a Bitcoin or Monero
//!   transaction actually paid the treasury is real cross-chain
//!   engineering (confirmation depth, reorg safety, Monero's view-key
//!   scanning). `NoExternalReceiptOracle` is the correct, permanent
//!   stand-in — this is not a cryptographic-design gap FROST or any other
//!   primitive here closes, it is a whole separate integration surface.
//!
//! [FREEZE reminder — D-0037] Every primitive in this crate — signing,
//! DKG, and resharing alike — is founder-reviewed, not externally audited.
//! Nothing here should be read as "custody solved" for real funds until
//! that audit happens (`docs/gates/dkg-audit-scope.md` for the DKG-
//! specific scope, separate from ordinary signing review). Every call
//! site that reaches trusted-dealer keygen, DKG, or resharing must name an
//! explicit acknowledgment type
//! ([`frost_keygen::AcknowledgedPrototypeOnly`] /
//! [`frost_dkg::AcknowledgedUnauditedDkg`]) — none of these paths is
//! reachable by accident. [`frost_sign::SigningNonces`] and
//! [`frost_dkg::DkgRound1Secret`] both zeroize on drop and redact their
//! `Debug` output (issue #93).
//!
//! This crate is bookkeeping, governance-membership data, and a threshold-
//! signature prototype — not a deployable treasury.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

mod curve;
mod error;
// Kept compiling unconditionally (own internal `#[cfg(test)]` coverage
// runs regardless of the `legacy-hand-rolled-dkg` feature), but their
// public items go unused by anything outside their own tests when that
// feature -- and therefore the `pub use` re-export below -- is off.
#[cfg_attr(not(feature = "legacy-hand-rolled-dkg"), allow(dead_code))]
mod frost_dkg;
mod frost_keygen;
#[cfg_attr(not(feature = "legacy-hand-rolled-dkg"), allow(dead_code))]
mod frost_reshare;
#[cfg_attr(not(feature = "legacy-hand-rolled-signing"), allow(dead_code))]
mod frost_sign;
mod rate;
mod receipt;
mod signers;

pub use error::{Result, TreasuryError};
// Off by default (D-0507): see the crate docs' "legacy-hand-rolled-dkg"
// section for why. `mod frost_dkg;`/`mod frost_reshare;` above stay
// unconditional so their own internal test coverage keeps building and
// running regardless of this feature -- only the *public, downstream-
// visible* API is gated.
#[cfg(feature = "legacy-hand-rolled-dkg")]
pub use frost_dkg::{
    dkg_finalize, dkg_generate_round2_shares, dkg_resolve, dkg_round1, dkg_verify_received_share,
    verify_round1_package, AcknowledgedUnauditedDkg, DkgComplaint, DkgRebuttal, DkgResolution,
    DkgRound1Package, DkgRound1Secret,
};
pub use frost_keygen::{
    trusted_dealer_keygen, AcknowledgedPrototypeOnly, KeyPackage, PublicKeyPackage,
    MAX_PARTICIPANTS as MAX_FROST_PARTICIPANTS,
};
#[cfg(feature = "legacy-hand-rolled-dkg")]
pub use frost_reshare::{reshare_finalize, reshare_round1, verify_reshare_round1_package};
// Off by default (Gate #72 remediation): see the crate docs' and this
// feature's own Cargo.toml comment for why. `mod frost_sign;` above stays
// unconditional so its own internal test coverage keeps building and
// running regardless of this feature -- only the public, downstream-
// visible API is gated. `mini_custody::signing` is the production path.
#[cfg(feature = "legacy-hand-rolled-signing")]
pub use frost_sign::{
    aggregate, round1_commit, round2_sign, verify, verify_signature_share, DurableFrostSigner,
    DurableSigningNonces, NonceCommitment, Signature, SigningNonces, SigningPackage,
};
pub use rate::{mint_amount_micro, RateEntry, RateHistory, RATE_SCALE};
pub use receipt::{
    ContributionKind, ContributionReceipt, ExternalReceiptOracle, NoExternalReceiptOracle,
};
pub use signers::{count_valid_approvals, meets_threshold, TreasurySignerSet, MAX_SIGNERS};
