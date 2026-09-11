use std::net::TcpListener;
use std::thread;

use did_mini::{Capabilities, Controller, FreshnessPins};
use mini_bearer::{Bearer, Initiator, Responder, TcpBearer};
use mini_crypto::AgreementSecretKey;
use mini_privacy_policy::{PrivacyTier, ProtectionProperty};
use mini_publication_policy::{
    publication_routing_plan_for, Attribution, Persistence, PublicationProfile, Visibility,
};
use mini_resource_pricing::PriceVector;
use mini_transport_policy::PayloadSizeClass;
use mini_transport_security::{
    authenticate_established_initiator, authenticate_established_responder, LocalSessionIdentity,
    PeerExpectation, ReplayCache, TransportPurpose, TransportTarget,
};

fn identity(seed: u8) -> (Controller, Controller) {
    let mut root = Controller::incept_single_from_seeds(&[seed; 32], &[seed + 1; 32]).unwrap();
    let device =
        Controller::incept_device_single_from_seeds(&root.did(), &[seed + 2; 32], &[seed + 3; 32])
            .unwrap();
    root.delegate_device(&device.did(), Capabilities::primary())
        .unwrap();
    (root, device)
}

#[test]
fn plan_dispatch_checks_immutable_request_before_any_tcp_application_frame() {
    let (client, client_device) = identity(10);
    let (server, server_device) = identity(40);
    let client_kel = client.kel();
    let client_device_kel = client_device.kel();
    let server_kel = server.kel();
    let server_device_kel = server_device.kel();
    let server_key = AgreementSecretKey::from_seed(&[80; 32]).public_key();
    let client_key = AgreementSecretKey::from_seed(&[90; 32]).public_key();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut bearer = TcpBearer::from_stream(stream).unwrap();
        let hello = bearer.recv().unwrap();
        let (channel, reply) = Responder::respond(&hello).unwrap();
        bearer.send(&reply).unwrap();
        let mut connection = authenticate_established_responder(
            bearer,
            channel,
            LocalSessionIdentity::new(server.did(), &server_device, server_key),
            TransportPurpose::Messaging,
            1_000,
            2_000,
            1_500,
            PeerExpectation::identity(&client_kel, &client_device_kel),
            &mut FreshnessPins::new(),
            &mut ReplayCache::new(8).unwrap(),
        )
        .unwrap();
        connection.recv(b"publication").unwrap()
    });
    let mut bearer = TcpBearer::connect(address).unwrap();
    let (initiator, hello) = Initiator::start().unwrap();
    bearer.send(&hello).unwrap();
    let channel = initiator.finish(&bearer.recv().unwrap()).unwrap();
    let mut connection = authenticate_established_initiator(
        bearer,
        channel,
        LocalSessionIdentity::new(client.did(), &client_device, client_key),
        TransportPurpose::Messaging,
        1_000,
        2_000,
        1_500,
        PeerExpectation::identity(&server_kel, &server_device_kel),
        &mut FreshnessPins::new(),
        &mut ReplayCache::new(8).unwrap(),
    )
    .unwrap();
    let endpoint = connection.peer().endpoint_id;
    let prices = PriceVector {
        bandwidth_micro_mini_per_mb: 1,
        storage_micro_mini_per_mb_day: 1,
    };
    let plan = |tier, properties| {
        publication_routing_plan_for(
            PublicationProfile {
                visibility: Visibility::Public,
                attribution: Attribution::Anonymous,
                transport: tier,
                persistence: Persistence::Durable,
            },
            properties,
            PayloadSizeClass::Small,
            &prices,
            1,
            1,
        )
        .unwrap()
    };
    let target = || TransportTarget::Direct { endpoint };
    let mut mixed = plan(PrivacyTier::Mixed, vec![ProtectionProperty::ContentSecrecy]);
    assert!(mixed
        .dispatch(
            &mut connection,
            target(),
            b"must not send mix",
            b"publication"
        )
        .is_err());
    mixed.profile.transport = PrivacyTier::Direct;
    assert!(mixed
        .dispatch(
            &mut connection,
            target(),
            b"must not downgrade",
            b"publication"
        )
        .is_err());
    let mut storage = plan(
        PrivacyTier::Direct,
        vec![ProtectionProperty::StorageAvailability],
    );
    storage.achievable.mechanisms.clear();
    assert!(storage
        .dispatch(
            &mut connection,
            target(),
            b"must not erase requested property",
            b"publication"
        )
        .is_err());
    let direct = plan(
        PrivacyTier::Direct,
        vec![ProtectionProperty::ContentSecrecy],
    );
    let receipt = direct
        .dispatch(
            &mut connection,
            target(),
            b"authorized publication",
            b"publication",
        )
        .unwrap();
    assert_eq!(receipt.tier(), PrivacyTier::Direct);
    assert_eq!(receipt.peer(), endpoint);
    assert_eq!(server_thread.join().unwrap(), b"authorized publication");
}
