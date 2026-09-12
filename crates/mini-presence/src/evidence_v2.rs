//! Hardware-backed ranging evidence, V2 (Gate #97 hardware/BLE/UWB/presence
//! architecture).
//!
//! [`crate::attestation::UwbRanging`] (V1) is a bare `{distance_cm,
//! sample_count}` pair with no security profile, no attack-detection signal,
//! and — critically — a `sample_count` the crate's own docs already admit is
//! "not independently checked." That is enough to *tighten* the software RTT
//! bound additively (D-0034 point 1) but not enough to ever be trusted as the
//! sole proximity evidence for a canonical personhood path: nothing stops a
//! caller from just writing `UwbRanging { distance_cm: 1, sample_count: 999
//! }` by hand.
//!
//! This module gives the verifier a richer evidence record plus a
//! **deterministic classification function** ([`classify_ranging_evidence`])
//! that derives an assurance level from the evidence's raw fields and a
//! versioned hardware capability registry — never from a self-reported
//! classification. There is no "claimed assurance" field on
//! [`RangingEvidenceV2`] for exactly that reason: a value nobody can set is a
//! value nobody can lie about. [`crate::verify::verify_presence_v2`] always
//! recomputes it fresh.
//!
//! ## Honest limits
//!
//! - This crate does not talk to ranging hardware. It defines the evidence
//!   shape and the policy a platform shell's real UWB/Channel-Sounding stack
//!   must produce, and verifies internal consistency (sample counts,
//!   distance-distribution ordering, technology/security-profile/registry
//!   agreement) — it cannot independently confirm the reported numbers came
//!   from a real ranging exchange rather than a compromised platform lying to
//!   this library. That trust boundary is the OS/secure-element attestation
//!   chain, which is out of scope here (see `platform_quality_flags`).
//! - [`HardwareCapabilityRegistryV1::builtin`] ships three generic,
//!   capability-class entries (UWB w/ FiRa-style secure ranging, Bluetooth
//!   Channel Sounding, plain software RTT) rather than a per-device
//!   allowlist, matching the gate document's preference for capability
//!   classes over device models. It is not a claim that any specific device
//!   in the field actually implements its class correctly — see
//!   `docs/gates/` for the open hardware-validation gate this feeds.
//! - No physical hardware validation happens in this repository. This module
//!   is the architecture the gate document asks for; closing the gate itself
//!   requires real-device evidence this sandboxed environment cannot produce.

use did_mini::{Controller, Did, IndexedSig, Kel};
use mini_crypto::HashAlgorithm;

fn blake3_256(data: &[u8]) -> [u8; 32] {
    HashAlgorithm::Blake3.digest(data)
}

/// The ranging technology that produced a [`RangingEvidenceV2`] record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RangingTechnologyV2 {
    /// Ultra-wideband ranging (e.g. FiRa-profile UWB with Secure Ranging /
    /// STS).
    Uwb,
    /// Bluetooth 6.0+ Channel Sounding (HADM) ranging.
    BleChannelSounding,
    /// Application-layer round-trip timing over a bearer channel — the same
    /// measurement [`crate::active_range`] produces. No dedicated ranging
    /// radio; always the weakest technology.
    SoftwareRtt,
}

/// Whether a ranging exchange was cryptographically bound to the session it
/// claims to measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RangingSecurityProfileV2 {
    /// The ranging exchange used a secure, session-bound timestamp sequence
    /// (e.g. UWB STS or BLE Channel Sounding's equivalent) — an attacker who
    /// does not hold the session key cannot forge or relay a plausible
    /// distance without breaking that binding.
    SecureSts,
    /// Plain, unauthenticated timing (e.g. legacy UWB ranging without STS, or
    /// software RTT). Never eligible for a "certified" assurance level
    /// regardless of how good the raw numbers look.
    Unauthenticated,
}

/// Whether both devices independently produced and cross-checked ranging
/// evidence, or only one side reports a measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasurementSidedness {
    /// Only one device's evidence is available.
    OneSided,
    /// Both devices independently measured and the results were
    /// cross-checked — much harder for one compromised endpoint to spoof
    /// unilaterally.
    TwoSided,
}

/// A platform's self-reported attack-detection signal for a ranging session,
/// carried at the same numeric scale as the industry Normalized Attack
/// Detector Metric (NADM): `0` = no attack detected, `1` = attack possible,
/// `2` = attack likely, `3` = not evaluated / unknown. [`PresencePolicyV2`]
/// rejects anything above [`PresencePolicyV2::SECURE_NADM_MAX`] for a
/// certified assurance level.
pub type AttackIndicatorV2 = u8;

/// The derived, deterministic outcome of [`classify_ranging_evidence`].
/// Never stored on the wire — always recomputed by the verifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PresenceAssuranceV2 {
    /// The evidence is internally inconsistent, fails a required bound, or
    /// names an unrecognized/uncertified capability class. Must not be
    /// treated as proximity evidence at all.
    Unusable,
    /// Software round-trip timing only — the same trust level as V1's
    /// always-enforced RTT bound, no hardware ranging corroboration.
    WeakSoftware,
    /// Hardware-ranged, secure-session-bound, registry-recognized evidence,
    /// but only one side reported it.
    CertifiedMedium,
    /// Hardware-ranged, secure-session-bound, registry-recognized evidence,
    /// cross-checked from both sides.
    CertifiedSecure,
}

/// One entry in the hardware capability registry: a generic capability
/// *class* (e.g. "UWB with FiRa-profile secure ranging"), not a specific
/// device model, per the gate document's preference for capability-based
/// entries over device allowlists.
#[derive(Debug, Clone, Copy)]
pub struct HardwareCapabilityClassV1 {
    /// Stable identifier: `blake3("mini-presence/hw-class/v1/" ++ name)`.
    /// [`RangingEvidenceV2::capability_class_id`] must match this exactly for
    /// the class to apply.
    pub id: [u8; 32],
    /// Human-readable name, for logs/diagnostics only — never compared.
    pub name: &'static str,
    /// The technology this class covers.
    pub technology: RangingTechnologyV2,
    /// Whether devices in this class are trusted, as a class, to perform
    /// secure (STS-bound) ranging correctly. `false` classes can still
    /// report [`RangingSecurityProfileV2::SecureSts`] but will never be
    /// classified above [`PresenceAssuranceV2::Unusable`] for a certified
    /// level — the registry, not the device's own self-report, is
    /// authoritative on what a class is trusted for.
    pub certified_secure_ranging: bool,
}

fn class_id(name: &str) -> [u8; 32] {
    let mut buf = Vec::with_capacity(32 + name.len());
    buf.extend_from_slice(b"mini-presence/hw-class/v1/");
    buf.extend_from_slice(name.as_bytes());
    blake3_256(&buf)
}

/// A versioned, source-controlled registry of hardware capability classes.
/// No online/remote lookup — a new class ships as a code change reviewed
/// through the same governance as everything else, matching the gate
/// document's "no online service" requirement.
#[derive(Debug, Clone)]
pub struct HardwareCapabilityRegistryV1 {
    classes: Vec<HardwareCapabilityClassV1>,
}

impl HardwareCapabilityRegistryV1 {
    /// The built-in registry: one generic class per technology this crate
    /// knows how to classify.
    pub fn builtin() -> Self {
        HardwareCapabilityRegistryV1 {
            classes: vec![
                HardwareCapabilityClassV1 {
                    id: class_id("uwb-fira-secure-ranging-1.0"),
                    name: "UWB (FiRa-profile secure ranging, STS)",
                    technology: RangingTechnologyV2::Uwb,
                    certified_secure_ranging: true,
                },
                HardwareCapabilityClassV1 {
                    id: class_id("ble-channel-sounding-1.0"),
                    name: "Bluetooth Channel Sounding (HADM)",
                    technology: RangingTechnologyV2::BleChannelSounding,
                    certified_secure_ranging: true,
                },
                HardwareCapabilityClassV1 {
                    id: class_id("software-rtt-generic-1.0"),
                    name: "Software round-trip timing (no ranging radio)",
                    technology: RangingTechnologyV2::SoftwareRtt,
                    certified_secure_ranging: false,
                },
            ],
        }
    }

    /// The stable class id for a built-in class name, for constructing test
    /// or real evidence. Returns `None` for a name not in this registry.
    pub fn id_for(&self, name: &str) -> Option<[u8; 32]> {
        self.classes.iter().find(|c| c.name == name).map(|c| c.id)
    }

    /// Look up a capability class by id.
    pub fn lookup(&self, id: &[u8; 32]) -> Option<&HardwareCapabilityClassV1> {
        self.classes.iter().find(|c| &c.id == id)
    }
}

/// Canonical, fixed policy bounds for V2 hardware-backed classification
/// (Gate #97 §28). Unlike [`crate::verify::RangePolicy`], this is not
/// caller-configurable: the gate document requires "the canonical personhood
/// path must use fixed/versioned policy," so these are associated constants,
/// not struct fields a caller could loosen.
#[derive(Debug, Clone, Copy)]
pub struct PresencePolicyV2;

impl PresencePolicyV2 {
    /// Maximum median hardware-ranged distance, in millimeters, for a
    /// certified assurance level.
    pub const MAX_SECURE_DISTANCE_MM: u32 = 2_000;
    /// Maximum p90 hardware-ranged distance, in millimeters — bounds the
    /// tail, not just the median, so a bimodal or noisy sample set can't
    /// hide an out-of-range outlier majority behind a tight median.
    pub const MAX_P90_DISTANCE_MM: u32 = 2_500;
    /// Minimum number of hardware ranging samples.
    pub const MIN_HARDWARE_SAMPLES: u32 = 12;
    /// Minimum session window, in ms, for hardware ranging (too-short a
    /// window can't have taken enough independent physical measurements).
    pub const MIN_HARDWARE_WINDOW_MS: u32 = 1_500;
    /// Maximum session window, in ms, for hardware ranging (bounds staleness
    /// the same way [`crate::verify::RangePolicy::max_session_ms`] does for
    /// the base attestation).
    pub const MAX_HARDWARE_WINDOW_MS: u32 = 10_000;
    /// Minimum software RTT samples — matches
    /// [`crate::verify::RangePolicy::ble_default`]'s `min_rtt_samples`.
    pub const MIN_SOFTWARE_RTT_SAMPLES: u32 = 4;
    /// Maximum software RTT bound, in ms — matches
    /// [`crate::verify::RangePolicy::ble_default`]'s `max_rtt_ms`.
    pub const MAX_SOFTWARE_RTT_MS: u32 = 50;
    /// The highest (least confident) NADM-scale [`AttackIndicatorV2`] value
    /// still accepted for a certified assurance level: `0` (no attack
    /// detected) or `1` (attack possible) only. `2` ("attack likely") and
    /// `3` ("not evaluated") are always rejected — an earlier revision set
    /// this to `2`, which combined with a `>` comparison let "attack
    /// likely" evidence itself pass as certified.
    pub const SECURE_NADM_MAX: AttackIndicatorV2 = 0x01;
}

/// Hardware-backed ranging evidence for a presence session (Gate #97). Every
/// field is a raw, independently-checkable measurement or identifier — there
/// is deliberately no "assurance" field to self-report; call
/// [`classify_ranging_evidence`] (or [`crate::verify::verify_presence_v2`],
/// which does this internally) to derive one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangingEvidenceV2 {
    /// The ranging technology used.
    pub technology: RangingTechnologyV2,
    /// Whether the exchange was session-bound/secure or plain timing.
    pub security_profile: RangingSecurityProfileV2,
    /// One or both sides reporting.
    pub sidedness: MeasurementSidedness,
    /// Digest identifying the registry [`HardwareCapabilityClassV1`] this
    /// evidence claims to come from. Looked up, never trusted at face value:
    /// [`classify_ranging_evidence`] rejects any id the registry doesn't
    /// recognize, and cross-checks the registry's `technology` against
    /// `technology` above.
    pub capability_class_id: [u8; 32],
    /// BLAKE3 digest of the out-of-band ranging session configuration (radio
    /// parameters, channel, slot schedule) the two devices agreed before
    /// ranging began. Carried through so a verifier that separately observed
    /// the OOB exchange can cross-check it; this crate does not itself
    /// witness that exchange, so it only checks this is non-zero (present),
    /// never that it matches some independently-known value.
    pub oob_config_digest: [u8; 32],
    /// BLAKE3 digest binding this evidence to one specific presence session.
    /// [`crate::verify::verify_presence_v2`] requires this to equal the
    /// digest of the attestation transcript it is being verified against —
    /// evidence from one session can never be replayed to back a different
    /// one.
    pub session_binding_digest: [u8; 32],
    /// Number of independent ranging measurements.
    pub sample_count: u32,
    /// Wall-clock duration of the ranging session, in ms.
    pub duration_ms: u32,
    /// Minimum measured distance, in millimeters.
    pub min_distance_mm: u32,
    /// 10th-percentile measured distance, in millimeters.
    pub p10_distance_mm: u32,
    /// Median measured distance, in millimeters.
    pub median_distance_mm: u32,
    /// 90th-percentile measured distance, in millimeters.
    pub p90_distance_mm: u32,
    /// Maximum measured distance, in millimeters.
    pub max_distance_mm: u32,
    /// Platform-reported attack-detection signal (NADM scale).
    pub attack_indicator: AttackIndicatorV2,
    /// Opaque, platform-defined quality bits carried through for forward
    /// extensibility (e.g. secure-element attestation available, calibrated
    /// antenna, etc). Not interpreted by [`classify_ranging_evidence`] today
    /// — a future policy revision may start checking specific bits, which is
    /// why the field exists on the wire type now rather than being added
    /// later as a breaking change.
    pub platform_quality_flags: u32,
}

impl RangingEvidenceV2 {
    /// The digest [`RangingEvidenceV2::session_binding_digest`] must equal to
    /// bind evidence to a specific attestation transcript.
    pub fn bind_to_transcript(transcript: &[u8]) -> [u8; 32] {
        blake3_256(transcript)
    }

    /// Whether the five distance percentiles are in non-decreasing order
    /// (`min <= p10 <= median <= p90 <= max`) — a self-inconsistent ordering
    /// means the numbers cannot have come from one honest sample set,
    /// regardless of technology.
    fn distances_are_ordered(&self) -> bool {
        self.min_distance_mm <= self.p10_distance_mm
            && self.p10_distance_mm <= self.median_distance_mm
            && self.median_distance_mm <= self.p90_distance_mm
            && self.p90_distance_mm <= self.max_distance_mm
    }

    /// Deterministic canonical byte encoding of every field, in declaration
    /// order, length-prefixed nowhere it doesn't need to be (every field is
    /// fixed-size) — what [`SignedRangingEvidenceV2`] actually signs, and
    /// what a caller can hash to compare two evidence records for exact
    /// equality without a `Hash` impl on the floating enums.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut w = Vec::with_capacity(128);
        w.push(match self.technology {
            RangingTechnologyV2::Uwb => 1,
            RangingTechnologyV2::BleChannelSounding => 2,
            RangingTechnologyV2::SoftwareRtt => 3,
        });
        w.push(match self.security_profile {
            RangingSecurityProfileV2::SecureSts => 1,
            RangingSecurityProfileV2::Unauthenticated => 2,
        });
        w.push(match self.sidedness {
            MeasurementSidedness::OneSided => 1,
            MeasurementSidedness::TwoSided => 2,
        });
        w.extend_from_slice(&self.capability_class_id);
        w.extend_from_slice(&self.oob_config_digest);
        w.extend_from_slice(&self.session_binding_digest);
        w.extend_from_slice(&self.sample_count.to_be_bytes());
        w.extend_from_slice(&self.duration_ms.to_be_bytes());
        w.extend_from_slice(&self.min_distance_mm.to_be_bytes());
        w.extend_from_slice(&self.p10_distance_mm.to_be_bytes());
        w.extend_from_slice(&self.median_distance_mm.to_be_bytes());
        w.extend_from_slice(&self.p90_distance_mm.to_be_bytes());
        w.extend_from_slice(&self.max_distance_mm.to_be_bytes());
        w.push(self.attack_indicator);
        w.extend_from_slice(&self.platform_quality_flags.to_be_bytes());
        w
    }
}

/// Domain-separated message a device signs to authenticate a
/// [`RangingEvidenceV2`] record — see [`SignedRangingEvidenceV2`].
const EVIDENCE_SIGNATURE_DOMAIN: &[u8] = b"mininet/presence/evidence-v2/v1";

/// [`RangingEvidenceV2`] plus a signature from one of the two attested
/// devices, over the evidence's own [`RangingEvidenceV2::canonical_bytes`].
///
/// Without this, evidence is just self-reported data: `session_binding_digest`
/// is a hash of *public* transcript bytes (visible to anyone who observed —
/// or was handed — a completed attestation), not something only a
/// legitimate participant could produce. Hashing public data is not
/// authentication. A device's signature over the evidence is: only the
/// device holding a real device key (already verified, in
/// [`crate::verify::verify_presence_v2`], to be a delegated `ATTEST`
/// device of one of the two identity roots) can produce one, so evidence
/// cannot be fabricated by anyone who merely observed a valid but weak
/// attestation elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedRangingEvidenceV2 {
    pub evidence: RangingEvidenceV2,
    /// Which of the attestation's two devices (`fields.initiator.device` or
    /// `fields.responder.device`) produced/attests to this evidence.
    pub signer_device: Did,
    pub signature: Vec<IndexedSig>,
}

impl SignedRangingEvidenceV2 {
    /// The exact bytes a device signs to authenticate `evidence`.
    pub fn message_to_sign(evidence: &RangingEvidenceV2) -> Vec<u8> {
        let mut w = Vec::with_capacity(EVIDENCE_SIGNATURE_DOMAIN.len() + 128);
        w.extend_from_slice(EVIDENCE_SIGNATURE_DOMAIN);
        w.extend_from_slice(&evidence.canonical_bytes());
        w
    }

    /// Sign `evidence` as `signer_device`, using that device's own
    /// controller. Encapsulates the exact message format so a real caller
    /// never has to reconstruct [`Self::message_to_sign`] by hand.
    pub fn sign(evidence: RangingEvidenceV2, signer_device: Did, device: &Controller) -> Self {
        let message = Self::message_to_sign(&evidence);
        let signature = device.sign_message(&message);
        SignedRangingEvidenceV2 {
            evidence,
            signer_device,
            signature,
        }
    }

    /// Verify the signature against `signer_kel` (the caller must already
    /// have confirmed this is really the KEL for `self.signer_device`, and
    /// that it names one of the attestation's two parties — see
    /// [`crate::verify::verify_presence_v2`]).
    pub fn verify(&self, signer_kel: &Kel) -> bool {
        let message = Self::message_to_sign(&self.evidence);
        signer_kel.verify_message(&message, &self.signature).is_ok()
    }
}

/// Deterministically derive a [`PresenceAssuranceV2`] level from raw
/// [`RangingEvidenceV2`] fields and a [`HardwareCapabilityRegistryV1`] (Gate
/// #97 §28's hardware classification algorithm). Pure and total: the same
/// evidence and registry always classify identically, and every input
/// (however hostile) produces some `PresenceAssuranceV2`, never a panic.
///
/// This function does not know about attestation transcripts, sessions, or
/// signatures — [`crate::verify::verify_presence_v2`] is the entry point
/// that also checks `session_binding_digest` against the actual session
/// being verified. Calling this directly is for tests and for
/// non-attestation uses (e.g. inspecting evidence before it's wrapped in a
/// signed session).
pub fn classify_ranging_evidence(
    evidence: &RangingEvidenceV2,
    registry: &HardwareCapabilityRegistryV1,
) -> PresenceAssuranceV2 {
    if evidence.sample_count == 0 || evidence.max_distance_mm == 0 {
        return PresenceAssuranceV2::Unusable;
    }
    if !evidence.distances_are_ordered() {
        return PresenceAssuranceV2::Unusable;
    }

    match evidence.technology {
        RangingTechnologyV2::SoftwareRtt => {
            if evidence.sample_count >= PresencePolicyV2::MIN_SOFTWARE_RTT_SAMPLES
                && evidence.duration_ms <= PresencePolicyV2::MAX_HARDWARE_WINDOW_MS
            {
                PresenceAssuranceV2::WeakSoftware
            } else {
                PresenceAssuranceV2::Unusable
            }
        }
        RangingTechnologyV2::Uwb | RangingTechnologyV2::BleChannelSounding => {
            // An all-zero digest means "no OOB config was ever agreed" per
            // this field's own doc comment, which the field contract treats
            // as absent -- hardware ranging evidence with no OOB
            // configuration binding at all must not be certifiable, even if
            // every other check passes.
            if evidence.oob_config_digest == [0u8; 32] {
                return PresenceAssuranceV2::Unusable;
            }
            let Some(class) = registry.lookup(&evidence.capability_class_id) else {
                return PresenceAssuranceV2::Unusable;
            };
            if class.technology != evidence.technology || !class.certified_secure_ranging {
                return PresenceAssuranceV2::Unusable;
            }
            if evidence.security_profile != RangingSecurityProfileV2::SecureSts {
                return PresenceAssuranceV2::Unusable;
            }
            if evidence.attack_indicator > PresencePolicyV2::SECURE_NADM_MAX {
                return PresenceAssuranceV2::Unusable;
            }
            if evidence.sample_count < PresencePolicyV2::MIN_HARDWARE_SAMPLES {
                return PresenceAssuranceV2::Unusable;
            }
            if evidence.duration_ms < PresencePolicyV2::MIN_HARDWARE_WINDOW_MS
                || evidence.duration_ms > PresencePolicyV2::MAX_HARDWARE_WINDOW_MS
            {
                return PresenceAssuranceV2::Unusable;
            }
            if evidence.median_distance_mm > PresencePolicyV2::MAX_SECURE_DISTANCE_MM
                || evidence.p90_distance_mm > PresencePolicyV2::MAX_P90_DISTANCE_MM
            {
                return PresenceAssuranceV2::Unusable;
            }
            match evidence.sidedness {
                MeasurementSidedness::TwoSided => PresenceAssuranceV2::CertifiedSecure,
                MeasurementSidedness::OneSided => PresenceAssuranceV2::CertifiedMedium,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uwb_class_id() -> [u8; 32] {
        HardwareCapabilityRegistryV1::builtin()
            .id_for("UWB (FiRa-profile secure ranging, STS)")
            .unwrap()
    }

    fn good_uwb_evidence() -> RangingEvidenceV2 {
        RangingEvidenceV2 {
            technology: RangingTechnologyV2::Uwb,
            security_profile: RangingSecurityProfileV2::SecureSts,
            sidedness: MeasurementSidedness::TwoSided,
            capability_class_id: uwb_class_id(),
            oob_config_digest: [7u8; 32],
            session_binding_digest: [9u8; 32],
            sample_count: 20,
            duration_ms: 3_000,
            min_distance_mm: 800,
            p10_distance_mm: 900,
            median_distance_mm: 1_000,
            p90_distance_mm: 1_200,
            max_distance_mm: 1_400,
            attack_indicator: 0,
            platform_quality_flags: 0,
        }
    }

    #[test]
    fn two_sided_certified_uwb_within_bounds_is_certified_secure() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let evidence = good_uwb_evidence();
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::CertifiedSecure
        );
    }

    #[test]
    fn one_sided_otherwise_identical_evidence_caps_at_certified_medium() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.sidedness = MeasurementSidedness::OneSided;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::CertifiedMedium
        );
    }

    #[test]
    fn unrecognized_capability_class_id_is_unusable_even_if_everything_else_is_perfect() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.capability_class_id = [0xAA; 32]; // not in the registry
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn a_device_cannot_self_report_secure_ranging_for_a_software_rtt_class() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let software_class_id = registry
            .id_for("Software round-trip timing (no ranging radio)")
            .unwrap();
        let mut evidence = good_uwb_evidence();
        // Claims UWB technology and cites the software-only class, which the
        // registry does not certify for secure ranging.
        evidence.capability_class_id = software_class_id;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn unauthenticated_security_profile_never_reaches_certified() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.security_profile = RangingSecurityProfileV2::Unauthenticated;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn attack_indicator_above_threshold_is_unusable() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.attack_indicator = PresencePolicyV2::SECURE_NADM_MAX + 1;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn nadm_attack_likely_is_never_certified() {
        // Regression test for a Codex finding: the documented NADM scale
        // names 2 "attack likely" -- that must never pass, regardless of
        // exactly where SECURE_NADM_MAX sits, and regardless of whether the
        // comparison is `>` or `>=`.
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.attack_indicator = 2;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn nadm_attack_possible_can_still_certify() {
        // The floor is not so strict that it rejects everything short of
        // a perfect "no attack detected" signal.
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.attack_indicator = 1;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::CertifiedSecure
        );
    }

    #[test]
    fn too_few_hardware_samples_is_unusable() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.sample_count = PresencePolicyV2::MIN_HARDWARE_SAMPLES - 1;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn distance_beyond_median_bound_is_unusable() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.median_distance_mm = PresencePolicyV2::MAX_SECURE_DISTANCE_MM + 1;
        evidence.p90_distance_mm = evidence.median_distance_mm + 100;
        evidence.max_distance_mm = evidence.p90_distance_mm + 100;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn a_tight_median_cannot_hide_an_out_of_range_p90_tail() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.p90_distance_mm = PresencePolicyV2::MAX_P90_DISTANCE_MM + 1;
        evidence.max_distance_mm = evidence.p90_distance_mm + 100;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn self_inconsistent_distance_ordering_is_unusable() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.median_distance_mm = evidence.p90_distance_mm + 1; // median > p90
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn a_reported_max_below_p90_is_unusable() {
        // Regression test for a Codex finding: distances_are_ordered used to
        // omit the final p90 <= max comparison, so a corrupted/fabricated
        // record whose max is below its own p90 passed the consistency
        // gate.
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.max_distance_mm = evidence.p90_distance_mm - 1;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn a_zero_oob_config_digest_is_unusable_for_hardware_ranging() {
        // Regression test for a Codex finding: the field contract says an
        // all-zero digest means "no OOB configuration was ever agreed,"
        // but classification never actually checked for it.
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.oob_config_digest = [0u8; 32];
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn zero_samples_is_unusable_for_any_technology() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.sample_count = 0;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn plain_software_rtt_with_enough_samples_is_weak_software() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.technology = RangingTechnologyV2::SoftwareRtt;
        evidence.security_profile = RangingSecurityProfileV2::Unauthenticated;
        evidence.capability_class_id = registry
            .id_for("Software round-trip timing (no ranging radio)")
            .unwrap();
        evidence.sample_count = PresencePolicyV2::MIN_SOFTWARE_RTT_SAMPLES;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::WeakSoftware
        );
    }

    #[test]
    fn software_rtt_below_minimum_samples_is_unusable() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.technology = RangingTechnologyV2::SoftwareRtt;
        evidence.sample_count = PresencePolicyV2::MIN_SOFTWARE_RTT_SAMPLES - 1;
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::Unusable
        );
    }

    #[test]
    fn ble_channel_sounding_classifies_the_same_way_as_uwb() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let mut evidence = good_uwb_evidence();
        evidence.technology = RangingTechnologyV2::BleChannelSounding;
        evidence.capability_class_id = registry
            .id_for("Bluetooth Channel Sounding (HADM)")
            .unwrap();
        assert_eq!(
            classify_ranging_evidence(&evidence, &registry),
            PresenceAssuranceV2::CertifiedSecure
        );
    }

    #[test]
    fn assurance_levels_order_unusable_below_weak_below_certified() {
        assert!(PresenceAssuranceV2::Unusable < PresenceAssuranceV2::WeakSoftware);
        assert!(PresenceAssuranceV2::WeakSoftware < PresenceAssuranceV2::CertifiedMedium);
        assert!(PresenceAssuranceV2::CertifiedMedium < PresenceAssuranceV2::CertifiedSecure);
    }

    #[test]
    fn signed_evidence_verifies_against_the_signer_and_rejects_tampering() {
        let mut root = Controller::incept_single_from_seeds(&[1u8; 32], &[2u8; 32]).unwrap();
        let device =
            Controller::incept_device_single_from_seeds(&root.did(), &[3u8; 32], &[4u8; 32])
                .unwrap();
        root.delegate_device(&device.did(), did_mini::Capabilities::primary())
            .unwrap();
        let device_kel = device.kel();

        let evidence = good_uwb_evidence();
        let signed = SignedRangingEvidenceV2::sign(evidence, device.did(), &device);
        assert!(signed.verify(&device_kel));

        let mut tampered = signed.clone();
        tampered.evidence.median_distance_mm += 1;
        assert!(!tampered.verify(&device_kel));

        let mut wrong_sig = signed;
        wrong_sig.signature.clear();
        assert!(!wrong_sig.verify(&device_kel));
    }

    #[test]
    fn registry_ids_are_stable_and_distinct() {
        let registry = HardwareCapabilityRegistryV1::builtin();
        let uwb = registry
            .id_for("UWB (FiRa-profile secure ranging, STS)")
            .unwrap();
        let ble = registry
            .id_for("Bluetooth Channel Sounding (HADM)")
            .unwrap();
        let sw = registry
            .id_for("Software round-trip timing (no ranging radio)")
            .unwrap();
        assert_ne!(uwb, ble);
        assert_ne!(ble, sw);
        assert_ne!(uwb, sw);
        // Recomputing must be deterministic across calls.
        assert_eq!(
            registry.id_for("UWB (FiRa-profile secure ranging, STS)"),
            Some(uwb)
        );
    }
}
