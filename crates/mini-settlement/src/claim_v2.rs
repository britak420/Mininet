//! A height-anchored payment claim for long-delay/disruption-tolerant
//! settlement (Gate #28, D-0513).
//!
//! [`crate::PaymentClaim`] (V1) expires against `valid_until_ms` — a wall/
//! device clock. That is the wrong validity boundary for a claim that may
//! spend days or months inside a store-carry-forward network (disaster
//! mesh, satellite, or longer): a device clock is not a canonical fact,
//! two devices' clocks can disagree by exactly the outage duration that
//! matters most, and a clock is not present at all on a node with no
//! trustworthy time source (`mini-dtn`'s Bundle Age mode, RFC 9171 §4.2.6).
//!
//! [`PaymentClaimV2`] replaces the wall-clock boundary with a canonical
//! chain anchor: the claim names the exact block it was signed against
//! ([`ChainAnchorV2`]) and a height past which it is no longer eligible
//! for canonical inclusion (`valid_through_height`). Nothing about this
//! changes M1/M2/M3 (`docs/INVARIANTS.md` §4): only a real
//! [`crate::CanonicalLedgerView`] can ever answer [`crate::SettlementState::Finalized`],
//! exactly as for V1.
//!
//! `PaymentClaim` (V1) is not reinterpreted or deprecated by this module —
//! old signed bytes keep meaning exactly what they always meant.
//!
//! ## A note on what this module deliberately does *not* implement
//!
//! An external DTN/satellite design report proposed anchoring
//! `valid_through_economic_epoch` to "Gate #6's twelve deterministic
//! Economic Epochs per Economic Year." No such canonical economic-epoch
//! concept exists anywhere in this repository today — `mini-execution`
//! has monetary issuance epochs (`ScalableEpochPlan`), a different concept
//! entirely, and there is no roadmap issue #6 defining calendar epochs.
//! Rather than invent a new canonical economic-time concept unilaterally
//! on an external document's unverified say-so (exactly the kind of claim
//! this tree's own discipline requires checking against real code before
//! acting on), this module anchors to chain **height** instead — a
//! primitive `mini-settlement` already models via
//! [`crate::CanonicalLedgerView`]. The report's actual underlying
//! engineering point (replace wall-clock expiry with a canonical,
//! chain-anchored boundary; bound the window so no claim is an
//! indefinitely reusable spend authorization) is fully preserved; only the
//! specific "economic epoch" vocabulary, which does not correspond to
//! anything real in this codebase, is not.

use mini_crypto::{HashAlgorithm, Signature, SignatureSuite, SigningKey, VerifyingKey};

use crate::claim::{MAX_CLAIM_FIELD_BYTES, MININET_NETWORK_ID};
use crate::error::{Result, SettlementError};

const CLAIM_V2_DOMAIN: &[u8] = b"mini-settlement/payment-claim-v2/v1";
const CLAIM_V2_WIRE_DOMAIN: &[u8] = b"mini-settlement/payment-claim-v2-wire/v1";

/// Maximum encoded size of one standalone `PaymentClaimV2`. Same bound as
/// V1's [`crate::MAX_PAYMENT_CLAIM_BYTES`]; kept as its own constant so the
/// two formats' bounds can diverge later without a silent coupling.
pub const MAX_PAYMENT_CLAIM_V2_BYTES: usize = 16 * 1024;

/// The canonical chain state a [`PaymentClaimV2`] was signed against: an
/// exact block height and its id. A real [`crate::CanonicalLedgerView`]
/// decides whether this is still a recognized ancestor of canonical chain
/// state (`CanonicalLedgerView::is_recognized_anchor`) — this type is only
/// the claim's own assertion of what it saw at signing time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainAnchorV2 {
    pub height: u64,
    pub block_id: [u8; 32],
}

/// A signed promise to pay, anchored to canonical chain height rather than
/// a device clock. See the module docs for why. Never final by itself —
/// exactly like [`crate::PaymentClaim`], only
/// [`crate::reconcile_v2`] reading a real [`crate::CanonicalLedgerView`]
/// can ever produce [`crate::SettlementState::Finalized`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentClaimV2 {
    /// Exact settlement network this promise may be executed on — same
    /// role as [`crate::PaymentClaim::network_id`].
    pub network_id: [u8; 32],
    pub payer: Vec<u8>,
    pub payee: Vec<u8>,
    pub amount_micro: u64,
    pub sequence: u64,
    /// The canonical chain state the payer observed when signing.
    pub anchor: ChainAnchorV2,
    /// The claim is no longer eligible for canonical inclusion once the
    /// canonical chain height strictly exceeds this value. Always strictly
    /// greater than `anchor.height` (checked at signing) and bounded to at
    /// most the signer-supplied `max_validity_height_span` — see
    /// [`sign_claim_v2_for_network`].
    pub valid_through_height: u64,
    /// An opaque, application-chosen 32-byte binding context (e.g. an
    /// invoice reference or a `mini-dtn` parcel binding) carried inside
    /// the signed bytes. This crate never inspects its contents.
    pub claim_context: [u8; 32],
    pub signature: Signature,
}

#[allow(clippy::too_many_arguments)]
fn claim_v2_message(
    network_id: &[u8; 32],
    payer: &[u8],
    payee: &[u8],
    amount_micro: u64,
    sequence: u64,
    anchor: &ChainAnchorV2,
    valid_through_height: u64,
    claim_context: &[u8; 32],
) -> Vec<u8> {
    let mut msg = Vec::with_capacity(
        CLAIM_V2_DOMAIN.len() + 32 + 4 + payer.len() + 4 + payee.len() + 8 + 8 + 8 + 32 + 8 + 32,
    );
    msg.extend_from_slice(CLAIM_V2_DOMAIN);
    msg.extend_from_slice(network_id);
    msg.extend_from_slice(&(payer.len() as u32).to_be_bytes());
    msg.extend_from_slice(payer);
    msg.extend_from_slice(&(payee.len() as u32).to_be_bytes());
    msg.extend_from_slice(payee);
    msg.extend_from_slice(&amount_micro.to_be_bytes());
    msg.extend_from_slice(&sequence.to_be_bytes());
    msg.extend_from_slice(&anchor.height.to_be_bytes());
    msg.extend_from_slice(&anchor.block_id);
    msg.extend_from_slice(&valid_through_height.to_be_bytes());
    msg.extend_from_slice(claim_context);
    msg
}

/// Sign a new `PaymentClaimV2` on the canonical public Mininet network.
/// `max_validity_height_span` bounds how far past `anchor.height`
/// `valid_through_height` may be set — this crate deliberately takes no
/// position on the right number of blocks (that depends on real block
/// cadence, which is a chain-execution concern this crate stays decoupled
/// from; see [`crate::ledger`]'s module docs), only that *some* finite
/// bound must exist so a claim is never an indefinitely reusable spend
/// authorization.
#[allow(clippy::too_many_arguments)]
pub fn sign_claim_v2(
    payer: &SigningKey,
    payee: &[u8],
    amount_micro: u64,
    sequence: u64,
    anchor: ChainAnchorV2,
    valid_through_height: u64,
    max_validity_height_span: u64,
    claim_context: [u8; 32],
) -> Result<PaymentClaimV2> {
    sign_claim_v2_for_network(
        payer,
        payee,
        amount_micro,
        sequence,
        anchor,
        valid_through_height,
        max_validity_height_span,
        &MININET_NETWORK_ID,
        claim_context,
    )
}

/// Sign a `PaymentClaimV2` for one exact settlement network.
#[allow(clippy::too_many_arguments)]
pub fn sign_claim_v2_for_network(
    payer: &SigningKey,
    payee: &[u8],
    amount_micro: u64,
    sequence: u64,
    anchor: ChainAnchorV2,
    valid_through_height: u64,
    max_validity_height_span: u64,
    network_id: &[u8; 32],
    claim_context: [u8; 32],
) -> Result<PaymentClaimV2> {
    if payer.suite() != SignatureSuite::DEFAULT {
        return Err(SettlementError::UnsupportedSignatureSuite);
    }
    if amount_micro == 0 {
        return Err(SettlementError::ZeroAmount);
    }
    if valid_through_height <= anchor.height {
        return Err(SettlementError::BadValidityWindow);
    }
    if valid_through_height - anchor.height > max_validity_height_span {
        return Err(SettlementError::BadValidityWindow);
    }
    let payer_bytes = payer.verifying_key().to_bytes().to_vec();
    let message = claim_v2_message(
        network_id,
        &payer_bytes,
        payee,
        amount_micro,
        sequence,
        &anchor,
        valid_through_height,
        &claim_context,
    );
    let signature = payer.sign(&message);
    Ok(PaymentClaimV2 {
        network_id: *network_id,
        payer: payer_bytes,
        payee: payee.to_vec(),
        amount_micro,
        sequence,
        anchor,
        valid_through_height,
        claim_context,
        signature,
    })
}

/// Verify a claim's signature against its own claimed payer key. Purely
/// structural, exactly like [`crate::verify_claim_signature`] — it says
/// nothing about canonical eligibility (see [`crate::reconcile_v2`]).
pub fn verify_claim_v2_signature(claim: &PaymentClaimV2) -> Result<()> {
    if claim.amount_micro == 0 {
        return Err(SettlementError::ZeroAmount);
    }
    let payer_key = VerifyingKey::from_suite_bytes(SignatureSuite::DEFAULT, &claim.payer)
        .map_err(|_| SettlementError::BadKey)?;
    let message = claim_v2_message(
        &claim.network_id,
        &claim.payer,
        &claim.payee,
        claim.amount_micro,
        claim.sequence,
        &claim.anchor,
        claim.valid_through_height,
        &claim.claim_context,
    );
    payer_key
        .verify(&message, &claim.signature)
        .map_err(|_| SettlementError::BadSignature)
}

/// A content digest of the claim's signed bytes, the V2 analogue of
/// [`crate::claim_digest`].
pub fn claim_v2_digest(claim: &PaymentClaimV2) -> [u8; 32] {
    let message = claim_v2_message(
        &claim.network_id,
        &claim.payer,
        &claim.payee,
        claim.amount_micro,
        claim.sequence,
        &claim.anchor,
        claim.valid_through_height,
        &claim.claim_context,
    );
    HashAlgorithm::Blake3.digest(&message)
}

fn put_bytes(w: &mut Vec<u8>, bytes: &[u8]) {
    w.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    w.extend_from_slice(bytes);
}

struct ClaimV2Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ClaimV2Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(SettlementError::MalformedClaim)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(SettlementError::MalformedClaim)?;
        self.position = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| SettlementError::MalformedClaim)?,
        ))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| SettlementError::MalformedClaim)?,
        ))
    }

    fn bytes(&mut self, maximum: usize) -> Result<&'a [u8]> {
        let length = usize::try_from(self.u32()?).map_err(|_| SettlementError::MalformedClaim)?;
        if length > maximum {
            return Err(SettlementError::ClaimTooLarge);
        }
        self.take(length)
    }

    fn finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

impl PaymentClaimV2 {
    /// Canonical bounded bytes for wallet-to-validator/DTN-parcel
    /// submission.
    pub fn to_wire_bytes(&self) -> Result<Vec<u8>> {
        if self.payer.len() > MAX_CLAIM_FIELD_BYTES || self.payee.len() > MAX_CLAIM_FIELD_BYTES {
            return Err(SettlementError::ClaimTooLarge);
        }
        let mut w = Vec::new();
        w.extend_from_slice(CLAIM_V2_WIRE_DOMAIN);
        w.extend_from_slice(&self.network_id);
        put_bytes(&mut w, &self.payer);
        put_bytes(&mut w, &self.payee);
        w.extend_from_slice(&self.amount_micro.to_be_bytes());
        w.extend_from_slice(&self.sequence.to_be_bytes());
        w.extend_from_slice(&self.anchor.height.to_be_bytes());
        w.extend_from_slice(&self.anchor.block_id);
        w.extend_from_slice(&self.valid_through_height.to_be_bytes());
        w.extend_from_slice(&self.claim_context);
        w.push(self.signature.suite().tag());
        w.extend_from_slice(&self.signature.to_bytes());
        if w.len() > MAX_PAYMENT_CLAIM_V2_BYTES {
            return Err(SettlementError::ClaimTooLarge);
        }
        Ok(w)
    }

    /// Decode one standalone claim with allocation bounds checked first —
    /// same discipline as [`crate::PaymentClaim::from_wire_bytes`], load-
    /// bearing for a claim arriving over an untrusted `mini-dtn` relay.
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_PAYMENT_CLAIM_V2_BYTES {
            return Err(SettlementError::ClaimTooLarge);
        }
        let mut r = ClaimV2Reader::new(bytes);
        if r.take(CLAIM_V2_WIRE_DOMAIN.len())? != CLAIM_V2_WIRE_DOMAIN {
            return Err(SettlementError::MalformedClaim);
        }
        let mut network_id = [0u8; 32];
        network_id.copy_from_slice(r.take(32)?);
        let payer = r.bytes(MAX_CLAIM_FIELD_BYTES)?.to_vec();
        let payee = r.bytes(MAX_CLAIM_FIELD_BYTES)?.to_vec();
        let amount_micro = r.u64()?;
        let sequence = r.u64()?;
        let height = r.u64()?;
        let mut block_id = [0u8; 32];
        block_id.copy_from_slice(r.take(32)?);
        let valid_through_height = r.u64()?;
        let mut claim_context = [0u8; 32];
        claim_context.copy_from_slice(r.take(32)?);
        let suite =
            SignatureSuite::from_tag(r.u8()?).map_err(|_| SettlementError::MalformedClaim)?;
        let signature = Signature::from_suite_bytes(suite, r.take(suite.signature_len())?)
            .map_err(|_| SettlementError::MalformedClaim)?;
        if !r.finished() {
            return Err(SettlementError::MalformedClaim);
        }
        Ok(Self {
            network_id,
            payer,
            payee,
            amount_micro,
            sequence,
            anchor: ChainAnchorV2 { height, block_id },
            valid_through_height,
            claim_context,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payer_key() -> SigningKey {
        SigningKey::from_seed(&[0x11; 32])
    }

    fn anchor(height: u64) -> ChainAnchorV2 {
        ChainAnchorV2 {
            height,
            block_id: [height as u8; 32],
        }
    }

    #[test]
    fn a_validly_signed_claim_v2_verifies() {
        let claim = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            0,
            anchor(100),
            112,
            100,
            [0u8; 32],
        )
        .unwrap();
        assert!(verify_claim_v2_signature(&claim).is_ok());
    }

    #[test]
    fn a_window_at_or_below_the_anchor_height_is_rejected() {
        assert_eq!(
            sign_claim_v2(
                &payer_key(),
                b"payee-a",
                1_000,
                0,
                anchor(100),
                100,
                100,
                [0u8; 32],
            )
            .unwrap_err(),
            SettlementError::BadValidityWindow
        );
        assert_eq!(
            sign_claim_v2(
                &payer_key(),
                b"payee-a",
                1_000,
                0,
                anchor(100),
                99,
                100,
                [0u8; 32],
            )
            .unwrap_err(),
            SettlementError::BadValidityWindow
        );
    }

    #[test]
    fn a_window_wider_than_the_caller_supplied_span_is_rejected() {
        assert_eq!(
            sign_claim_v2(
                &payer_key(),
                b"payee-a",
                1_000,
                0,
                anchor(100),
                201,
                100,
                [0u8; 32],
            )
            .unwrap_err(),
            SettlementError::BadValidityWindow
        );
        // Exactly at the span boundary is fine.
        assert!(sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            0,
            anchor(100),
            200,
            100,
            [0u8; 32],
        )
        .is_ok());
    }

    #[test]
    fn tampering_any_signed_field_breaks_verification() {
        let claim = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            0,
            anchor(100),
            112,
            100,
            [0u8; 32],
        )
        .unwrap();

        let mut tampered_amount = claim.clone();
        tampered_amount.amount_micro = 999_999;
        assert_eq!(
            verify_claim_v2_signature(&tampered_amount).unwrap_err(),
            SettlementError::BadSignature
        );

        let mut tampered_anchor = claim.clone();
        tampered_anchor.anchor.height = 50;
        assert_eq!(
            verify_claim_v2_signature(&tampered_anchor).unwrap_err(),
            SettlementError::BadSignature
        );

        let mut tampered_anchor_id = claim.clone();
        tampered_anchor_id.anchor.block_id = [0xff; 32];
        assert_eq!(
            verify_claim_v2_signature(&tampered_anchor_id).unwrap_err(),
            SettlementError::BadSignature
        );

        let mut tampered_valid_through = claim.clone();
        tampered_valid_through.valid_through_height = 999;
        assert_eq!(
            verify_claim_v2_signature(&tampered_valid_through).unwrap_err(),
            SettlementError::BadSignature
        );

        let mut tampered_context = claim;
        tampered_context.claim_context = [0x42; 32];
        assert_eq!(
            verify_claim_v2_signature(&tampered_context).unwrap_err(),
            SettlementError::BadSignature
        );
    }

    #[test]
    fn standalone_wire_round_trip_preserves_digest_and_signature() {
        let claim = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            7,
            anchor(100),
            112,
            100,
            [0x9u8; 32],
        )
        .unwrap();
        let decoded = PaymentClaimV2::from_wire_bytes(&claim.to_wire_bytes().unwrap()).unwrap();
        assert_eq!(decoded, claim);
        assert_eq!(claim_v2_digest(&decoded), claim_v2_digest(&claim));
        verify_claim_v2_signature(&decoded).unwrap();
    }

    #[test]
    fn standalone_wire_rejects_every_truncation_and_trailing_bytes() {
        let claim = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            7,
            anchor(100),
            112,
            100,
            [0u8; 32],
        )
        .unwrap();
        let bytes = claim.to_wire_bytes().unwrap();
        for cut in 0..bytes.len() {
            assert!(PaymentClaimV2::from_wire_bytes(&bytes[..cut]).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert_eq!(
            PaymentClaimV2::from_wire_bytes(&trailing).unwrap_err(),
            SettlementError::MalformedClaim
        );
    }

    #[test]
    fn a_declared_oversized_field_length_is_rejected_before_allocating() {
        // A hostile DTN relay-delivered claim declares an enormous payer
        // length up front; the reader must reject it from the length
        // prefix alone, never by first trying to allocate/copy that many
        // bytes out of a much shorter buffer.
        let claim = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            7,
            anchor(100),
            112,
            100,
            [0u8; 32],
        )
        .unwrap();
        let mut bytes = claim.to_wire_bytes().unwrap();
        let payer_len_offset = CLAIM_V2_WIRE_DOMAIN.len() + 32;
        bytes[payer_len_offset..payer_len_offset + 4]
            .copy_from_slice(&(MAX_CLAIM_FIELD_BYTES as u32 + 1).to_be_bytes());
        assert_eq!(
            PaymentClaimV2::from_wire_bytes(&bytes).unwrap_err(),
            SettlementError::ClaimTooLarge
        );
    }

    #[test]
    fn two_claims_differing_only_by_claim_context_have_different_digests() {
        let a = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            0,
            anchor(100),
            112,
            100,
            [1u8; 32],
        )
        .unwrap();
        let b = sign_claim_v2(
            &payer_key(),
            b"payee-a",
            1_000,
            0,
            anchor(100),
            112,
            100,
            [2u8; 32],
        )
        .unwrap();
        assert_ne!(claim_v2_digest(&a), claim_v2_digest(&b));
    }
}
