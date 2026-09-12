//! Integration tests for Gate #97 hardware-backed presence verification
//! (`verify_presence_v2`).
//!
//! Mirrors `tests/presence.rs`'s fixture shape (real delegation, real channel
//! handshake, real signatures) but exercises the V2 path: InProcess
//! transport rejection, evidence/session binding, evidence *authentication*
//! (a real device signature, not just a public-transcript hash), and
//! assurance-derived classification.

use did_mini::{Capabilities, Controller};
use mini_bearer::{Initiator, Responder};
use mini_presence::{
    kel_digest, verify_presence_v2, AttestationFields, HardwareCapabilityRegistryV1,
    InMemoryReplayGuard, MeasurementSidedness, Party, PresenceAssuranceV2, PresenceAttestation,
    PresenceError, RangePolicy, RangingEvidenceV2, RangingSecurityProfileV2, RangingTechnologyV2,
    SignedRangingEvidenceV2, TransportKind, VerifyContext, PRESENCE_VERSION,
};

fn test_nonce(seed: u8) -> [u8; 32] {
    mini_crypto::HashAlgorithm::Blake3.digest(&[seed])
}

fn human(
    root_c: [u8; 32],
    root_n: [u8; 32],
    dev_c: [u8; 32],
    dev_n: [u8; 32],
    caps: Capabilities,
) -> (Controller, Controller) {
    let mut root = Controller::incept_single_from_seeds(&root_c, &root_n).unwrap();
    let device = Controller::incept_device_single_from_seeds(&root.did(), &dev_c, &dev_n).unwrap();
    root.delegate_device(&device.did(), caps).unwrap();
    (root, device)
}

fn fresh_binding() -> [u8; 32] {
    let (initiator, hello1) = Initiator::start().unwrap();
    let (responder_channel, hello2) = Responder::respond(&hello1).unwrap();
    let initiator_channel = initiator.finish(&hello2).unwrap();
    assert_eq!(
        initiator_channel.channel_binding(),
        responder_channel.channel_binding()
    );
    initiator_channel.channel_binding()
}

fn policy() -> RangePolicy {
    RangePolicy::ble_default()
}

fn valid_attestation(
    init_device: &Controller,
    resp_device: &Controller,
    binding: [u8; 32],
    transport: TransportKind,
) -> PresenceAttestation {
    let fields = AttestationFields {
        version: PRESENCE_VERSION,
        channel_binding: binding,
        initiator: Party {
            device: init_device.did(),
            kel_digest: kel_digest(&init_device.kel()),
            nonce: test_nonce(1),
        },
        responder: Party {
            device: resp_device.did(),
            kel_digest: kel_digest(&resp_device.kel()),
            nonce: test_nonce(2),
        },
        started_at_ms: 1_000,
        finished_at_ms: 1_006,
        rtt_samples_ms: vec![10, 12, 9, 11],
        transport,
        location_commitment: None,
        uwb: None,
    };
    let init_sig = fields.sign(init_device);
    let resp_sig = fields.sign(resp_device);
    PresenceAttestation::new(fields, init_sig, resp_sig)
}

/// Raw, unsigned evidence for `att` — callers mutate fields before signing
/// with [`sign_evidence`] so the signature always covers the final,
/// possibly-deliberately-broken record under test.
fn good_evidence(
    att: &PresenceAttestation,
    registry: &HardwareCapabilityRegistryV1,
) -> RangingEvidenceV2 {
    RangingEvidenceV2 {
        technology: RangingTechnologyV2::Uwb,
        security_profile: RangingSecurityProfileV2::SecureSts,
        sidedness: MeasurementSidedness::TwoSided,
        capability_class_id: registry
            .id_for("UWB (FiRa-profile secure ranging, STS)")
            .unwrap(),
        oob_config_digest: [3u8; 32],
        session_binding_digest: RangingEvidenceV2::bind_to_transcript(&att.fields.transcript()),
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

fn sign_evidence(evidence: RangingEvidenceV2, signer: &Controller) -> SignedRangingEvidenceV2 {
    SignedRangingEvidenceV2::sign(evidence, signer.did(), signer)
}

struct Fixture {
    att: PresenceAttestation,
    a_root: Controller,
    b_root: Controller,
    a_dev: Controller,
    b_dev: Controller,
    binding: [u8; 32],
}

fn fixture(transport: TransportKind) -> Fixture {
    let (a_root, a_dev) = human([1; 32], [2; 32], [3; 32], [4; 32], Capabilities::primary());
    let (b_root, b_dev) = human([5; 32], [6; 32], [7; 32], [8; 32], Capabilities::primary());
    let binding = fresh_binding();
    let att = valid_attestation(&a_dev, &b_dev, binding, transport);
    Fixture {
        att,
        a_root,
        b_root,
        a_dev,
        b_dev,
        binding,
    }
}

#[test]
fn in_process_transport_is_always_rejected_by_v2() {
    let f = fixture(TransportKind::InProcess);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let err = verify_presence_v2(
        &f.att,
        None,
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::WeakSoftware,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InProcessTransportRejectedByV2);
}

#[test]
fn no_evidence_at_or_below_weak_software_minimum_succeeds() {
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let verdict = verify_presence_v2(
        &f.att,
        None,
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::WeakSoftware,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::WeakSoftware);
    assert_eq!(
        verdict.verdict.initiator_root.as_str(),
        f.a_root.did().as_str()
    );
}

#[test]
fn no_evidence_but_certified_minimum_required_is_refused() {
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let err = verify_presence_v2(
        &f.att,
        None,
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedMedium,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InsufficientAssurance);
}

#[test]
fn no_evidence_below_the_fixed_v2_rtt_sample_floor_is_rejected_even_with_a_loose_ctx_policy() {
    // ctx.policy is caller-configurable and deliberately set far looser than
    // PresencePolicyV2's fixed bounds -- V2's WeakSoftware floor must still
    // be enforced directly against PresencePolicyV2, not whatever ctx.policy
    // says.
    let f = fixture(TransportKind::Ble);
    let mut loose_att = f.att.clone();
    loose_att.fields.rtt_samples_ms = vec![10]; // below MIN_SOFTWARE_RTT_SAMPLES (4)
    let init_sig = loose_att.fields.sign(&f.a_dev);
    let resp_sig = loose_att.fields.sign(&f.b_dev);
    let loose_att = PresenceAttestation::new(loose_att.fields, init_sig, resp_sig);

    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let mut loose_policy = policy();
    loose_policy.min_rtt_samples = 0; // caller loosened the base policy
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &loose_policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let err = verify_presence_v2(
        &loose_att,
        None,
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::WeakSoftware,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::NotEnoughRangeSamples);
}

#[test]
fn a_rejected_v2_attempt_never_burns_replay_nonces_for_a_legitimate_retry() {
    // Regression test for a Codex finding: the base verify_presence (which
    // durably records nonces) must run last, after every V2-specific check,
    // so a V2 rejection (here: insufficient assurance with no evidence)
    // never consumes the nonces a legitimate follow-up attempt would need.
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();

    // First attempt: no evidence, but a certified minimum is required -- rejected.
    let err = verify_presence_v2(
        &f.att,
        None,
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InsufficientAssurance);

    // Retry with real, corroborated two-sided evidence over the SAME
    // attestation (same nonces): must succeed, proving the first attempt
    // never recorded them.
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    let counterpart = sign_evidence(good_evidence(&f.att, &registry), &f.b_dev);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
        Some(&counterpart),
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::CertifiedSecure);
}

#[test]
fn well_formed_corroborated_hardware_evidence_reaches_certified_secure() {
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    let counterpart = sign_evidence(good_evidence(&f.att, &registry), &f.b_dev);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
        Some(&counterpart),
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::CertifiedSecure);
    // `verdict.verdict.hardware_ranged` reflects only V1's `AttestationFields::uwb`
    // (unset in this fixture), not the V2 `RangingEvidenceV2` passed separately —
    // `verdict.assurance` above is the V2-accurate signal.
}

#[test]
fn a_lone_signer_claiming_two_sided_evidence_is_capped_at_certified_medium() {
    // The core Codex finding this test closes: `sidedness` lives inside the
    // evidence a single device signs, so nothing previously stopped one
    // compromised endpoint from setting `TwoSided`, signing alone, and
    // being classified `CertifiedSecure`. Without a genuine, independently
    // signed counterpart, the assurance must silently downgrade rather than
    // trust the self-reported sidedness.
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None, // no counterpart at all
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedMedium,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::CertifiedMedium);

    // And, being capped, it must not satisfy a CertifiedSecure minimum.
    let mut replay2 = InMemoryReplayGuard::new();
    let mut att2 = f.att.clone();
    att2.fields.initiator.nonce = test_nonce(3);
    att2.fields.responder.nonce = test_nonce(4);
    let init_sig = att2.fields.sign(&f.a_dev);
    let resp_sig = att2.fields.sign(&f.b_dev);
    let att2 = PresenceAttestation::new(att2.fields, init_sig, resp_sig);
    let evidence2 = sign_evidence(good_evidence(&att2, &registry), &f.a_dev);
    let err = verify_presence_v2(
        &att2,
        Some(&evidence2),
        None,
        &ctx,
        &mut replay2,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InsufficientAssurance);
}

#[test]
fn a_counterpart_signed_by_the_same_device_as_the_primary_does_not_corroborate() {
    // A second signature from the SAME device is not independent
    // corroboration -- it must be treated exactly like having no
    // counterpart at all.
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    let fake_counterpart = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
        Some(&fake_counterpart),
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedMedium,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::CertifiedMedium);
}

#[test]
fn a_counterpart_describing_a_different_technology_does_not_corroborate() {
    // A "counterpart" that doesn't actually describe the same physical
    // ranging exchange (different technology here) must not be accepted
    // as cross-checking the primary evidence.
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    let mut mismatched = good_evidence(&f.att, &registry);
    mismatched.technology = RangingTechnologyV2::BleChannelSounding;
    mismatched.capability_class_id = registry
        .id_for("Bluetooth Channel Sounding (HADM)")
        .unwrap();
    let mismatched_counterpart = sign_evidence(mismatched, &f.b_dev);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
        Some(&mismatched_counterpart),
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedMedium,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::CertifiedMedium);
}

#[test]
fn evidence_signed_by_the_responder_is_also_accepted() {
    // Either attested party may be the evidence's signer -- the ranging
    // exchange can legitimately be observed/reported by either side. Alone
    // (no counterpart), it is still capped at CertifiedMedium.
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &f.b_dev);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedMedium,
    )
    .unwrap();
    assert_eq!(verdict.assurance, PresenceAssuranceV2::CertifiedMedium);
}

#[test]
fn evidence_with_a_forged_signature_is_rejected_even_with_a_correct_binding() {
    // The core Codex finding this whole authentication mechanism closes:
    // an observer who merely saw a completed (even weak) attestation can
    // compute session_binding_digest = blake3(transcript) themselves --
    // that alone must never be enough. Here the binding is exactly right
    // but the "signature" is garbage nobody's key produced.
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let mut evidence = sign_evidence(good_evidence(&f.att, &registry), &f.a_dev);
    evidence.signature = Vec::new(); // no real signature at all
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::EvidenceSignatureInvalid);
}

#[test]
fn evidence_signed_by_someone_who_is_not_a_party_to_this_session_is_rejected() {
    let f = fixture(TransportKind::Ble);
    let (stranger_root, stranger_dev) = human(
        [9; 32],
        [10; 32],
        [11; 32],
        [12; 32],
        Capabilities::primary(),
    );
    let _ = stranger_root;
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let evidence = sign_evidence(good_evidence(&f.att, &registry), &stranger_dev);
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::EvidenceSignerNotAParty);
}

#[test]
fn evidence_bound_to_a_different_session_is_rejected() {
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let mut raw = good_evidence(&f.att, &registry);
    // Evidence produced for/replayed from an unrelated session -- signed
    // over this (wrong) binding, so the signature itself is genuine.
    raw.session_binding_digest = [0xEE; 32];
    let evidence = sign_evidence(raw, &f.a_dev);
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::EvidenceSessionBindingMismatch);
}

#[test]
fn unusable_evidence_is_rejected_even_though_no_evidence_would_have_passed() {
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let mut raw = good_evidence(&f.att, &registry);
    raw.capability_class_id = [0xAA; 32]; // unrecognized
    let evidence = sign_evidence(raw, &f.a_dev);
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::WeakSoftware,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::EvidenceUnusable);
}

#[test]
fn one_sided_evidence_does_not_satisfy_a_certified_secure_minimum() {
    let f = fixture(TransportKind::Ble);
    let (a_root_kel, b_root_kel) = (f.a_root.kel(), f.b_root.kel());
    let (a_dev_kel, b_dev_kel) = (f.a_dev.kel(), f.b_dev.kel());
    let policy = policy();
    let ctx = VerifyContext {
        initiator_root: &a_root_kel,
        responder_root: &b_root_kel,
        initiator_device: &a_dev_kel,
        responder_device: &b_dev_kel,
        policy: &policy,
        now_ms: Some(2_000),
        expected_binding: Some(f.binding),
    };
    let mut replay = InMemoryReplayGuard::new();
    let registry = HardwareCapabilityRegistryV1::builtin();
    let mut raw = good_evidence(&f.att, &registry);
    raw.sidedness = MeasurementSidedness::OneSided;
    let evidence = sign_evidence(raw, &f.a_dev);
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
        None,
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InsufficientAssurance);
}
