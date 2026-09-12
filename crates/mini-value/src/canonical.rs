//! Canonical wire decoding for scalars and points — the fix for the Gate
//! #72 external cryptography audit's F72-01 (non-canonical wire scalar
//! decoding). Every module in this crate that used to hand-roll its own
//! `decompress_scalar`/`decompress_point` pair now goes through here
//! instead, so the rule lives in one place rather than being re-derived
//! (correctly or not) at each call site.
//!
//! # Why `Scalar::from_bytes_mod_order` was the bug
//!
//! `curve25519-dalek`'s scalar field has order `l ≈ 2^252.4`, but a
//! `Scalar` is encoded in 32 bytes (`2^256` possible values). That means
//! roughly 16 distinct byte strings reduce to the same in-field value under
//! `from_bytes_mod_order`/`from_bytes_mod_order_wide`. For a *freshly
//! generated* scalar (a random nonce, a hash-to-scalar output) that
//! ambiguity is irrelevant: nobody is comparing byte strings, only the
//! resulting field element ever gets used. For a scalar that arrives over
//! the wire as part of something signed, proved, or hashed for identity
//! purposes — a ring-signature response, a key image, a Bulletproof
//! folded scalar — silently accepting any of those ~16 encodings as
//! equivalent is a malleability bug: the same logical value now has
//! multiple valid encodings, which breaks any assumption that encode/
//! decode/encode is byte-identical, and produces multiple valid digests
//! for what should be one canonical object.
//!
//! [`canonical_scalar`] uses `Scalar::from_canonical_bytes` instead, which
//! rejects any of those non-canonical representations outright. Reducing
//! (`from_bytes_mod_order_wide`) stays exactly where the audit says it
//! belongs: hashing to a scalar ([`crate::curve::hash_to_scalar`]) and
//! generating a fresh random one ([`crate::curve::random_scalar`]) — never
//! parsing something somebody else sent.
//!
//! # Points were already canonical
//!
//! Ristretto's whole reason for existing over raw Edwards points is a
//! canonical encoding: [`CompressedRistretto::decompress`] already rejects
//! non-canonical byte strings (checked directly against
//! `curve25519-dalek`'s own `decompress` implementation, which verifies
//! the encoded field element is below the field modulus before doing
//! anything else). [`canonical_point`] exists to collect the handful of
//! duplicated `decompress_point` helpers this crate had grown into one
//! place, and to add the semantic non-identity check
//! ([`canonical_nonidentity_point`]) the audit's Section 5.3 calls for at
//! specific point roles (spend/view/tx/one-time keys, key images,
//! commitments) — decoding validity and "is this a meaningful key" are
//! different questions, and callers that need the second one said so.

use crate::curve::{CompressedRistretto, RistrettoPoint, Scalar};

/// Decode a wire-supplied scalar. Rejects anything that is not the unique
/// canonical 32-byte encoding of a value in `[0, l)` — including a value
/// that is numerically in range but was encoded non-minimally (any of the
/// ~16 non-canonical byte strings reducing to the same field element).
///
/// Use this for every scalar that arrives from outside the local process:
/// signature responses, challenges, key images treated as scalars,
/// Bulletproof folded scalars, blinding factors read back off a claim.
/// Never for a value this process itself just hashed or freshly randomized
/// — those go through [`crate::curve::hash_to_scalar`]/
/// [`crate::curve::random_scalar`] instead, whose wide reduction is
/// correct and required there.
pub fn canonical_scalar(bytes: &[u8]) -> Option<Scalar> {
    let arr: [u8; 32] = bytes.try_into().ok()?;
    Option::from(Scalar::from_canonical_bytes(arr))
}

/// [`canonical_scalar`], additionally rejecting the zero scalar. For
/// secret/ephemeral scalars that must be nonzero to mean anything (a
/// one-time private key, a shared-secret offset) — see [`crate::stealth_impl`]'s
/// own resampling discipline for the signer side of this same rule.
pub fn canonical_nonzero_scalar(bytes: &[u8]) -> Option<Scalar> {
    let scalar = canonical_scalar(bytes)?;
    if scalar == Scalar::ZERO {
        return None;
    }
    Some(scalar)
}

/// Decode a wire-supplied Ristretto point. `CompressedRistretto::decompress`
/// is already a canonical decoder (non-canonical encodings are rejected by
/// the underlying `curve25519-dalek` implementation) — this exists to give
/// every call site in this crate one shared helper instead of four
/// hand-copied ones.
pub fn canonical_point(bytes: &[u8]) -> Option<RistrettoPoint> {
    let arr: [u8; 32] = bytes.try_into().ok()?;
    CompressedRistretto(arr).decompress()
}

/// [`canonical_point`], additionally rejecting the identity element. Use
/// this for every point role the audit's Section 5.3 calls out as
/// semantically non-identity: spend/view public keys, transaction public
/// keys, one-time output keys, key images, output/pseudo commitments.
pub fn canonical_nonidentity_point(bytes: &[u8]) -> Option<RistrettoPoint> {
    let point = canonical_point(bytes)?;
    if point == RistrettoPoint::default() {
        return None;
    }
    Some(point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::basepoint;

    /// The group order `l`, little-endian, exactly as `curve25519-dalek`
    /// encodes `Scalar::ZERO - Scalar::ONE... ` would not give us this
    /// directly, so this is the well-known constant
    /// `2^252 + 27742317777372353535851937790883648493`.
    const GROUP_ORDER_LE: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x10,
    ];

    #[test]
    fn a_value_already_in_range_round_trips() {
        let scalar = crate::curve::random_scalar().unwrap();
        let bytes = scalar.to_bytes();
        assert_eq!(canonical_scalar(&bytes), Some(scalar));
    }

    #[test]
    fn zero_is_canonical_but_rejected_by_the_nonzero_variant() {
        let zero = [0u8; 32];
        assert_eq!(canonical_scalar(&zero), Some(Scalar::ZERO));
        assert_eq!(canonical_nonzero_scalar(&zero), None);
    }

    /// The exact malleability the audit's F72-01 describes: the group
    /// order itself, encoded as 32 bytes, is numerically `>= l` and must
    /// be rejected outright -- `from_bytes_mod_order` would have silently
    /// reduced it to zero, giving it the same accepted meaning as the
    /// canonical all-zero encoding. That is two different wire byte
    /// strings both claiming to be "the" encoding of the same value.
    #[test]
    fn the_group_order_itself_is_rejected_not_reduced_to_zero() {
        assert_eq!(canonical_scalar(&GROUP_ORDER_LE), None);
        // Confirms the two encodings really would have collided under the
        // old, non-canonical parser -- this is not a hypothetical.
        assert_eq!(
            Scalar::from_bytes_mod_order(GROUP_ORDER_LE),
            Scalar::from_bytes_mod_order([0u8; 32])
        );
    }

    #[test]
    fn one_past_the_group_order_is_also_rejected() {
        let mut bytes = GROUP_ORDER_LE;
        bytes[0] = bytes[0].wrapping_add(1);
        assert_eq!(canonical_scalar(&bytes), None);
    }

    #[test]
    fn wrong_length_is_rejected() {
        assert_eq!(canonical_scalar(&[0u8; 31]), None);
        assert_eq!(canonical_scalar(&[0u8; 33]), None);
    }

    #[test]
    fn the_identity_point_decodes_but_is_rejected_by_the_nonidentity_variant() {
        let identity = RistrettoPoint::default();
        let bytes = identity.compress().to_bytes();
        assert_eq!(canonical_point(&bytes), Some(identity));
        assert_eq!(canonical_nonidentity_point(&bytes), None);
    }

    #[test]
    fn a_real_point_round_trips_through_the_nonidentity_variant() {
        let point = basepoint();
        let bytes = point.compress().to_bytes();
        assert_eq!(canonical_nonidentity_point(&bytes), Some(point));
    }

    #[test]
    fn malformed_point_bytes_are_rejected() {
        assert_eq!(canonical_point(&[0xffu8; 32]), None);
    }
}
