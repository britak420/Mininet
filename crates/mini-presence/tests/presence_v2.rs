//! Integration tests for Gate #97 hardware-backed presence verification
//! (`verify_presence_v2`).
//!
//! Mirrors `tests/presence.rs`'s fixture shape (real delegation, real channel
//! handshake, real signatures) but exercises the V2 path: InProcess
//! transport rejection, evidence/session binding, and assurance-derived
//! classification.

use did_mini::{Capabilities, Controller};
use mini_bearer::{Initiator, Responder};
use mini_presence::{
    kel_digest, verify_presence_v2, AttestationFields, HardwareCapabilityRegistryV1,
    InMemoryReplayGuard, MeasurementSidedness, Party, PresenceAssuranceV2, PresenceAttestation,
    PresenceError, RangePolicy, RangingEvidenceV2, RangingSecurityProfileV2, RangingTechnologyV2,
    TransportKind, VerifyContext, PRESENCE_VERSION,
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

fn good_evidence_for(
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
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedMedium,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InsufficientAssurance);
}

#[test]
fn well_formed_hardware_evidence_reaches_certified_secure() {
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
    let evidence = good_evidence_for(&f.att, &registry);
    let verdict = verify_presence_v2(
        &f.att,
        Some(&evidence),
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
    let mut evidence = good_evidence_for(&f.att, &registry);
    // Evidence produced for/replayed from an unrelated session.
    evidence.session_binding_digest = [0xEE; 32];
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
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
    let mut evidence = good_evidence_for(&f.att, &registry);
    evidence.capability_class_id = [0xAA; 32]; // unrecognized
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
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
    let mut evidence = good_evidence_for(&f.att, &registry);
    evidence.sidedness = MeasurementSidedness::OneSided;
    let err = verify_presence_v2(
        &f.att,
        Some(&evidence),
        &ctx,
        &mut replay,
        &registry,
        PresenceAssuranceV2::CertifiedSecure,
    )
    .unwrap_err();
    assert_eq!(err, PresenceError::InsufficientAssurance);
}
