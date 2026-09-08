//! Executable convergence of discovery, CH1, identity, retry, and onion routing.
//!
//! The lower-level modules intentionally remain reusable primitives. This module
//! is the safety seam a normal caller should use when it needs a named peer: one
//! value owns the bearer, the exact CH1 channel, and the identity verified on
//! that channel. Discovery never becomes trust, and an authenticated identity is
//! not returned separately from the connection it authenticated.

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use did_mini::{Controller, Did, FreshnessPins, Kel};
use mini_bearer::{Bearer, BearerError, Channel, Initiator, TcpBearer};
use mini_crypto::AgreementPublicKey;
use mini_privacy_policy::{PrivacyRequest, PrivacyTier, ProtectionProperty};
use mini_relay::{build_onion, ConnectionId, OnionHop, OnionPacket, RelayRole};
use mini_transport_policy::{PayloadSizeClass, TransportRequest};

use crate::{
    diverse_dial_plan, executable_transport, AuthenticatedPeer, ExecutableTransport,
    PeerSelectionPolicy, ReplayCache, Result, SessionAuthClaim, SessionRole, TransportPurpose,
    TransportSecurityError, VerifiedPeerAdvertisement, MAX_DIAL_TIMEOUT_MS, MIN_DIAL_TIMEOUT_MS,
};

/// AEAD associated data for encrypted authentication claims on CH1.
pub const SESSION_AUTH_FRAME_AAD: &[u8] = b"MINI/TRANSPORT-AUTH1";

/// Local identity disclosed for one typed authenticated session.
#[derive(Debug, Clone)]
pub struct LocalSessionIdentity<'a> {
    pub root: Did,
    pub device: &'a Controller,
    pub routing_key: AgreementPublicKey,
}

impl<'a> LocalSessionIdentity<'a> {
    pub const fn new(root: Did, device: &'a Controller, routing_key: AgreementPublicKey) -> Self {
        Self {
            root,
            device,
            routing_key,
        }
    }
}

/// What the remote side must prove on this exact channel.
#[derive(Debug, Clone, Copy)]
pub enum PeerExpectation<'a> {
    /// Verify a known identity without making a discovery-address claim.
    Identity {
        root_kel: &'a Kel,
        device_kel: &'a Kel,
    },
    /// Verify the live peer against the exact signed record selected for dial.
    Advertised {
        advertisement: &'a VerifiedPeerAdvertisement,
        root_kel: &'a Kel,
        device_kel: &'a Kel,
    },
}

impl<'a> PeerExpectation<'a> {
    pub const fn identity(root_kel: &'a Kel, device_kel: &'a Kel) -> Self {
        Self::Identity {
            root_kel,
            device_kel,
        }
    }

    pub const fn advertised(
        advertisement: &'a VerifiedPeerAdvertisement,
        root_kel: &'a Kel,
        device_kel: &'a Kel,
    ) -> Self {
        Self::Advertised {
            advertisement,
            root_kel,
            device_kel,
        }
    }
}

/// One secure-discovery target and the KEL material needed to verify it live.
#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedDialTarget<'a> {
    pub advertisement: &'a VerifiedPeerAdvertisement,
    pub root_kel: &'a Kel,
    pub device_kel: &'a Kel,
}

impl<'a> AuthenticatedDialTarget<'a> {
    pub const fn new(
        advertisement: &'a VerifiedPeerAdvertisement,
        root_kel: &'a Kel,
        device_kel: &'a Kel,
    ) -> Self {
        Self {
            advertisement,
            root_kel,
            device_kel,
        }
    }
}

/// A transport connection whose peer identity is inseparable from the channel
/// on which it was proved.
#[derive(Debug)]
pub struct AuthenticatedConnection<B: Bearer> {
    bearer: B,
    channel: Channel,
    peer: AuthenticatedPeer,
    usable: bool,
}

impl<B: Bearer> AuthenticatedConnection<B> {
    pub fn peer(&self) -> &AuthenticatedPeer {
        &self.peer
    }

    pub fn channel_binding(&self) -> [u8; 32] {
        self.channel.channel_binding()
    }

    /// Encrypt and send one application frame on the authenticated channel.
    /// A bearer failure after sealing permanently poisons the connection because
    /// CH1's local send counter has already advanced and remote receipt is
    /// unknowable.
    pub fn send(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<()> {
        let endpoint = self.peer.endpoint_id;
        dispatch_transport(
            &TransportRequest {
                privacy: PrivacyRequest {
                    tier: PrivacyTier::Direct,
                    properties: Vec::new(),
                },
                payload_size_class: PayloadSizeClass::Small,
            },
            self,
            TransportTarget::Direct { endpoint },
            plaintext,
            aad,
        )?;
        Ok(())
    }

    // Only the checked dispatcher may submit application data. Handshake
    // messages above are separate protocol frames, not privacy-tier payloads.
    fn send_checked_frame(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<()> {
        self.ensure_usable()?;
        let ciphertext = self.channel.seal(plaintext, aad)?;
        if let Err(error) = self.bearer.send(&ciphertext) {
            self.usable = false;
            return Err(error.into());
        }
        Ok(())
    }

    /// Receive, authenticate, and decrypt one application frame. Any bearer or
    /// AEAD failure poisons the ordered connection rather than letting a caller
    /// continue from an uncertain stream position.
    pub fn recv(&mut self, aad: &[u8]) -> Result<Vec<u8>> {
        self.ensure_usable()?;
        let ciphertext = match self.bearer.recv() {
            Ok(ciphertext) => ciphertext,
            Err(error) => {
                self.usable = false;
                return Err(error.into());
            }
        };
        match self.channel.open(&ciphertext, aad) {
            Ok(plaintext) => Ok(plaintext),
            Err(error) => {
                self.usable = false;
                Err(error.into())
            }
        }
    }

    fn ensure_usable(&self) -> Result<()> {
        if self.usable {
            Ok(())
        } else {
            Err(TransportSecurityError::ConnectionPoisoned)
        }
    }
}

/// Exact route selected for this send. A tier/target mismatch is an error;
/// there is no fallback from a requested onion/mix route to direct delivery.
#[derive(Debug)]
pub enum TransportTarget<'a> {
    Direct {
        endpoint: crate::TransportEndpointId,
    },
    Onion {
        relays: [VerifiedRelay<'a>; 3],
        destination_connection_id: ConnectionId,
        destination_key: AgreementPublicKey,
        now_ms: u64,
        expires_at_ms: u64,
    },
}

/// Evidence of a successful local bearer submission, minted only by dispatch.
/// For Relayed this records submission to the authenticated entry relay. It
/// does not assert relay forwarding, destination receipt, payment, storage,
/// source anonymity against colluding relays, or resistance to correlation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedSendReceipt {
    tier: PrivacyTier,
    peer: crate::TransportEndpointId,
    payload_digest: [u8; 32],
    channel_binding: [u8; 32],
}

impl ObservedSendReceipt {
    pub fn tier(&self) -> PrivacyTier {
        self.tier
    }
    pub fn peer(&self) -> crate::TransportEndpointId {
        self.peer
    }
    pub fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub fn channel_binding(&self) -> [u8; 32] {
        self.channel_binding
    }
}

/// Mandatory application dispatch for authenticated transport. Checks the
/// executable tier and each requested property before any sealing or send.
/// Anonymous low-level CH1 remains available for protocol handshakes and
/// explicitly anonymous callers; it cannot mint this execution receipt.
pub fn dispatch_transport<B: Bearer>(
    request: &TransportRequest,
    connection: &mut AuthenticatedConnection<B>,
    target: TransportTarget<'_>,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<ObservedSendReceipt> {
    let executor = executable_transport(request.privacy.tier, true)?;
    // The policy vocabulary also names storage/personhood/payment properties.
    // A send cannot establish those just because route() prices their tier.
    for property in &request.privacy.properties {
        let available = match property {
            ProtectionProperty::ContentSecrecy => true,
            ProtectionProperty::CounterpartyIpHiding | ProtectionProperty::MetadataMinimization => {
                request.privacy.tier == PrivacyTier::Relayed
            }
            _ => false,
        };
        if !available {
            return Err(TransportSecurityError::UnimplementedProtection);
        }
    }
    let frame = match (executor, target) {
        (ExecutableTransport::AuthenticatedDirect(_), TransportTarget::Direct { endpoint }) => {
            if connection.peer.endpoint_id != endpoint {
                return Err(TransportSecurityError::EndpointMismatch);
            }
            plaintext.to_vec()
        }
        (
            ExecutableTransport::ThreeHopOnion(_),
            TransportTarget::Onion {
                relays,
                destination_connection_id,
                destination_key,
                now_ms,
                expires_at_ms,
            },
        ) => {
            if connection.peer.endpoint_id != relays[0].advertisement.endpoint_id() {
                return Err(TransportSecurityError::EndpointMismatch);
            }
            if connection.peer.purpose != TransportPurpose::Relay {
                return Err(TransportSecurityError::WrongPurpose);
            }
            build_verified_onion_route(
                relays,
                destination_connection_id,
                request.payload_size_class,
                destination_key,
                plaintext,
                now_ms,
                expires_at_ms,
            )?
            .to_bytes()?
        }
        _ => return Err(TransportSecurityError::TransportTargetMismatch),
    };
    connection.send_checked_frame(&frame, aad)?;
    let mut commitment = b"mininet/transport-submitted-payload/v1".to_vec();
    commitment.extend_from_slice(&(aad.len() as u64).to_be_bytes());
    commitment.extend_from_slice(aad);
    commitment.extend_from_slice(plaintext);
    Ok(ObservedSendReceipt {
        tier: request.privacy.tier,
        peer: connection.peer.endpoint_id,
        payload_digest: mini_crypto::HashAlgorithm::Blake3.digest(&commitment),
        channel_binding: connection.channel_binding(),
    })
}

/// Authenticate an already-established initiator-side channel. The responder
/// proves itself first, so a redirected endpoint cannot collect the initiator's
/// DID before matching the selected advertisement.
#[allow(clippy::too_many_arguments)]
pub fn authenticate_established_initiator<B: Bearer>(
    mut bearer: B,
    mut channel: Channel,
    local: LocalSessionIdentity<'_>,
    purpose: TransportPurpose,
    issued_at_ms: u64,
    expires_at_ms: u64,
    now_ms: u64,
    expected_peer: PeerExpectation<'_>,
    freshness: &mut FreshnessPins,
    replay: &mut ReplayCache,
) -> Result<AuthenticatedConnection<B>> {
    let encrypted_claim = bearer.recv()?;
    let claim_bytes = channel.open(&encrypted_claim, SESSION_AUTH_FRAME_AAD)?;
    let claim = SessionAuthClaim::from_bytes(&claim_bytes)?;

    let mut staged_freshness = freshness.clone();
    let mut staged_replay = replay.clone();
    let peer = verify_expected_claim(
        &claim,
        expected_peer,
        SessionRole::Responder,
        purpose,
        &channel.channel_binding(),
        now_ms,
        &mut staged_freshness,
        &mut staged_replay,
    )?;

    let local_claim = SessionAuthClaim::issue(
        &local.root,
        local.device,
        SessionRole::Initiator,
        purpose,
        local.routing_key,
        &channel.channel_binding(),
        issued_at_ms,
        expires_at_ms,
    )?;
    let encrypted_local = channel.seal(&local_claim.to_bytes()?, SESSION_AUTH_FRAME_AAD)?;
    bearer.send(&encrypted_local)?;

    *freshness = staged_freshness;
    *replay = staged_replay;
    Ok(AuthenticatedConnection {
        bearer,
        channel,
        peer,
        usable: true,
    })
}

/// Authenticate an already-established responder-side channel. The responder
/// sends its channel-bound proof first; it commits peer freshness/replay state
/// only after the initiator's complete proof verifies.
#[allow(clippy::too_many_arguments)]
pub fn authenticate_established_responder<B: Bearer>(
    mut bearer: B,
    mut channel: Channel,
    local: LocalSessionIdentity<'_>,
    purpose: TransportPurpose,
    issued_at_ms: u64,
    expires_at_ms: u64,
    now_ms: u64,
    expected_peer: PeerExpectation<'_>,
    freshness: &mut FreshnessPins,
    replay: &mut ReplayCache,
) -> Result<AuthenticatedConnection<B>> {
    let local_claim = SessionAuthClaim::issue(
        &local.root,
        local.device,
        SessionRole::Responder,
        purpose,
        local.routing_key,
        &channel.channel_binding(),
        issued_at_ms,
        expires_at_ms,
    )?;
    let encrypted_local = channel.seal(&local_claim.to_bytes()?, SESSION_AUTH_FRAME_AAD)?;
    bearer.send(&encrypted_local)?;

    let encrypted_claim = bearer.recv()?;
    let claim_bytes = channel.open(&encrypted_claim, SESSION_AUTH_FRAME_AAD)?;
    let claim = SessionAuthClaim::from_bytes(&claim_bytes)?;

    let mut staged_freshness = freshness.clone();
    let mut staged_replay = replay.clone();
    let peer = verify_expected_claim(
        &claim,
        expected_peer,
        SessionRole::Initiator,
        purpose,
        &channel.channel_binding(),
        now_ms,
        &mut staged_freshness,
        &mut staged_replay,
    )?;

    *freshness = staged_freshness;
    *replay = staged_replay;
    Ok(AuthenticatedConnection {
        bearer,
        channel,
        peer,
        usable: true,
    })
}

#[allow(clippy::too_many_arguments)]
fn verify_expected_claim(
    claim: &SessionAuthClaim,
    expected_peer: PeerExpectation<'_>,
    role: SessionRole,
    purpose: TransportPurpose,
    binding: &[u8; 32],
    now_ms: u64,
    freshness: &mut FreshnessPins,
    replay: &mut ReplayCache,
) -> Result<AuthenticatedPeer> {
    match expected_peer {
        PeerExpectation::Identity {
            root_kel,
            device_kel,
        } => claim.verify(
            role, purpose, binding, now_ms, root_kel, device_kel, freshness, replay,
        ),
        PeerExpectation::Advertised {
            advertisement,
            root_kel,
            device_kel,
        } => {
            ensure_advertisement_live(advertisement, now_ms)?;
            claim.verify_advertised(
                advertisement,
                role,
                purpose,
                binding,
                now_ms,
                root_kel,
                device_kel,
                freshness,
                replay,
            )
        }
    }
}

fn ensure_advertisement_live(advertisement: &VerifiedPeerAdvertisement, now_ms: u64) -> Result<()> {
    if now_ms > advertisement.expires_at_ms() {
        return Err(TransportSecurityError::Expired);
    }
    Ok(())
}

/// Dial one signed endpoint over TCP, establish CH1, and authenticate the live
/// responder against that exact endpoint record.
#[allow(clippy::too_many_arguments)]
pub fn connect_authenticated_tcp(
    local: LocalSessionIdentity<'_>,
    purpose: TransportPurpose,
    issued_at_ms: u64,
    expires_at_ms: u64,
    now_ms: u64,
    target: AuthenticatedDialTarget<'_>,
    timeout_ms: u64,
    freshness: &mut FreshnessPins,
    replay: &mut ReplayCache,
) -> Result<AuthenticatedConnection<TcpBearer>> {
    ensure_advertisement_live(target.advertisement, now_ms)?;
    let (bearer, channel) = establish_tcp_initiator(target.advertisement.address(), timeout_ms)?;
    authenticate_established_initiator(
        bearer,
        channel,
        local,
        purpose,
        issued_at_ms,
        expires_at_ms,
        now_ms,
        PeerExpectation::advertised(target.advertisement, target.root_kel, target.device_kel),
        freshness,
        replay,
    )
}

/// Try a locally seeded, prefix-diverse dial plan in bounded order. A failed
/// connection or identity proof is discarded whole; no failed attempt mutates
/// the caller's freshness pins or replay cache.
#[allow(clippy::too_many_arguments)]
pub fn connect_first_authenticated_tcp(
    local: LocalSessionIdentity<'_>,
    purpose: TransportPurpose,
    issued_at_ms: u64,
    expires_at_ms: u64,
    now_ms: u64,
    targets: &[AuthenticatedDialTarget<'_>],
    local_seed: [u8; 32],
    policy: PeerSelectionPolicy,
    freshness: &mut FreshnessPins,
    replay: &mut ReplayCache,
) -> Result<AuthenticatedConnection<TcpBearer>> {
    if targets.len() > crate::MAX_SELECTION_CANDIDATES {
        return Err(TransportSecurityError::LimitExceeded);
    }
    let expected_network = targets
        .first()
        .map(|target| target.advertisement.network_id());
    let mut records = Vec::with_capacity(targets.len());
    for target in targets {
        if Some(target.advertisement.network_id()) != expected_network {
            return Err(TransportSecurityError::WrongNetwork);
        }
        if now_ms <= target.advertisement.expires_at_ms() {
            records.push(target.advertisement.clone());
        }
    }
    let plan = diverse_dial_plan(&records, local_seed, policy)?;
    let mut attempted = 0usize;

    for attempt in plan {
        let Some(target) = targets.iter().copied().find(|target| {
            target.advertisement.endpoint_id() == attempt.endpoint_id
                && target.advertisement.address() == attempt.address
                && target.advertisement.routing_key() == attempt.routing_key
        }) else {
            continue;
        };
        attempted += 1;
        if let Ok(connection) = connect_authenticated_tcp(
            local.clone(),
            purpose,
            issued_at_ms,
            expires_at_ms,
            now_ms,
            target,
            attempt.timeout_ms,
            freshness,
            replay,
        ) {
            return Ok(connection);
        }
    }

    Err(TransportSecurityError::DialExhausted { attempted })
}

fn establish_tcp_initiator(address: SocketAddr, timeout_ms: u64) -> Result<(TcpBearer, Channel)> {
    if !(MIN_DIAL_TIMEOUT_MS..=MAX_DIAL_TIMEOUT_MS).contains(&timeout_ms) {
        return Err(TransportSecurityError::InvalidSelectionPolicy);
    }
    let timeout = Duration::from_millis(timeout_ms);
    let stream = TcpStream::connect_timeout(&address, timeout).map_err(BearerError::from)?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(BearerError::from)?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(BearerError::from)?;
    let mut bearer = TcpBearer::from_stream(stream)?;
    let (initiator, hello) = Initiator::start()?;
    bearer.send(&hello)?;
    let response = bearer.recv()?;
    let channel = initiator.finish(&response)?;
    Ok((bearer, channel))
}

/// One already-verified candidate for a fixed onion role.
#[derive(Debug, Clone, Copy)]
pub struct VerifiedRelay<'a> {
    pub advertisement: &'a VerifiedPeerAdvertisement,
    pub next_hop: &'a [u8],
}

impl<'a> VerifiedRelay<'a> {
    pub const fn new(advertisement: &'a VerifiedPeerAdvertisement, next_hop: &'a [u8]) -> Self {
        Self {
            advertisement,
            next_hop,
        }
    }
}

/// Build the executable three-hop onion only from three visibly distinct,
/// verified transport endpoints. Distinct pairwise roots can still belong to one
/// hidden operator; that residual operator-independence limit remains explicit.
#[allow(clippy::too_many_arguments)]
pub fn build_verified_onion_route(
    relays: [VerifiedRelay<'_>; 3],
    destination_connection_id: ConnectionId,
    size_class: PayloadSizeClass,
    destination_key: AgreementPublicKey,
    plaintext: &[u8],
    now_ms: u64,
    expires_at_ms: u64,
) -> Result<OnionPacket> {
    let expected_network = relays[0].advertisement.network_id();
    for relay in &relays {
        ensure_advertisement_live(relay.advertisement, now_ms)?;
        if relay.advertisement.network_id() != expected_network {
            return Err(TransportSecurityError::WrongNetwork);
        }
    }
    for left in 0..relays.len() {
        for right in left + 1..relays.len() {
            let a = relays[left].advertisement;
            let b = relays[right].advertisement;
            if a.endpoint_id() == b.endpoint_id()
                || a.routing_key() == b.routing_key()
                || a.root() == b.root()
                || a.device() == b.device()
            {
                return Err(TransportSecurityError::RouteEndpointReuse);
            }
        }
    }

    let roles = [RelayRole::Entry, RelayRole::Rendezvous, RelayRole::Delivery];
    let hops: Vec<_> = relays
        .iter()
        .zip(roles)
        .map(|(relay, role)| OnionHop {
            role,
            routing_key: relay.advertisement.routing_key(),
            next_hop: relay.next_hop.to_vec(),
        })
        .collect();
    Ok(build_onion(
        destination_connection_id,
        size_class,
        &hops,
        destination_key,
        plaintext,
        now_ms,
        expires_at_ms,
    )?)
}

#[cfg(test)]
mod tests {
    use did_mini::{Capabilities, Controller, FreshnessPins};
    use mini_crypto::AgreementSecretKey;

    use super::*;
    use crate::{PeerAdvertisement, ReplayCache};

    fn verified(seed: u8, address: &str) -> VerifiedPeerAdvertisement {
        let mut root = Controller::incept_single_from_seeds(&[seed; 32], &[seed + 1; 32]).unwrap();
        let device = Controller::incept_device_single_from_seeds(
            &root.did(),
            &[seed + 2; 32],
            &[seed + 3; 32],
        )
        .unwrap();
        root.delegate_device(&device.did(), Capabilities::primary())
            .unwrap();
        let routing = AgreementSecretKey::from_seed(&[seed + 4; 32]).public_key();
        let advertisement = PeerAdvertisement::issue(
            [7; 32],
            &root.did(),
            &device,
            routing,
            address.parse().unwrap(),
            1_000,
            2_000,
        )
        .unwrap();
        let mut freshness = FreshnessPins::new();
        let mut replay = ReplayCache::new(8).unwrap();
        advertisement
            .verify(
                [7; 32],
                1_500,
                &root.kel(),
                &device.kel(),
                &mut freshness,
                &mut replay,
            )
            .unwrap()
    }

    #[derive(Debug)]
    struct FailingBearer;

    impl Bearer for FailingBearer {
        fn send(&mut self, _frame: &[u8]) -> mini_bearer::Result<()> {
            Err(BearerError::Closed)
        }

        fn recv(&mut self) -> mini_bearer::Result<Vec<u8>> {
            Err(BearerError::Closed)
        }

        fn try_recv(&mut self) -> mini_bearer::Result<Option<Vec<u8>>> {
            Err(BearerError::Closed)
        }
    }

    #[test]
    fn bearer_send_failure_permanently_poisons_the_ordered_connection() {
        let (root, device) = {
            let mut root = Controller::incept_single_from_seeds(&[80; 32], &[81; 32]).unwrap();
            let device =
                Controller::incept_device_single_from_seeds(&root.did(), &[82; 32], &[83; 32])
                    .unwrap();
            root.delegate_device(&device.did(), Capabilities::primary())
                .unwrap();
            (root, device)
        };
        let routing = AgreementSecretKey::from_seed(&[84; 32]).public_key();
        let (initiator, hello) = mini_bearer::Initiator::start().unwrap();
        let (_responder, response) = mini_bearer::Responder::respond(&hello).unwrap();
        let channel = initiator.finish(&response).unwrap();
        let peer = AuthenticatedPeer {
            root: root.did(),
            device: device.did(),
            endpoint_id: crate::TransportEndpointId::derive(&device.did(), &routing),
            routing_key: routing,
            capabilities: Capabilities::primary(),
            purpose: TransportPurpose::PeerExchange,
        };
        let mut connection = AuthenticatedConnection {
            bearer: FailingBearer,
            channel,
            peer,
            usable: true,
        };

        assert_eq!(
            connection.send(b"first", b"aad"),
            Err(TransportSecurityError::Bearer(BearerError::Closed))
        );
        assert_eq!(
            connection.send(b"second", b"aad"),
            Err(TransportSecurityError::ConnectionPoisoned)
        );
        assert_eq!(
            connection.recv(b"aad"),
            Err(TransportSecurityError::ConnectionPoisoned)
        );
    }

    #[test]
    fn verified_route_rejects_reusing_one_endpoint_for_two_roles() {
        let a = verified(10, "10.0.0.1:9000");
        let b = verified(20, "10.0.1.1:9000");
        let destination = AgreementSecretKey::from_seed(&[99; 32]);
        let result = build_verified_onion_route(
            [
                VerifiedRelay::new(&a, b"rendezvous"),
                VerifiedRelay::new(&a, b"delivery"),
                VerifiedRelay::new(&b, b"destination"),
            ],
            ConnectionId::from_bytes([1; 16]),
            PayloadSizeClass::Small,
            destination.public_key(),
            b"payload",
            1_500,
            10_000,
        );
        assert_eq!(result, Err(TransportSecurityError::RouteEndpointReuse));
    }

    #[test]
    fn verified_route_rechecks_expiry_and_network_at_use_time() {
        let a = verified(10, "10.0.0.1:9000");
        let b = verified(20, "10.0.1.1:9000");
        let c = verified(30, "10.0.2.1:9000");
        let destination = AgreementSecretKey::from_seed(&[99; 32]);
        assert_eq!(
            build_verified_onion_route(
                [
                    VerifiedRelay::new(&a, b"rendezvous"),
                    VerifiedRelay::new(&b, b"delivery"),
                    VerifiedRelay::new(&c, b"destination"),
                ],
                ConnectionId::from_bytes([1; 16]),
                PayloadSizeClass::Small,
                destination.public_key(),
                b"payload",
                2_001,
                10_000,
            ),
            Err(TransportSecurityError::Expired)
        );

        let mut foreign_root = Controller::incept_single_from_seeds(&[60; 32], &[61; 32]).unwrap();
        let foreign_device =
            Controller::incept_device_single_from_seeds(&foreign_root.did(), &[62; 32], &[63; 32])
                .unwrap();
        foreign_root
            .delegate_device(&foreign_device.did(), Capabilities::primary())
            .unwrap();
        let foreign_routing = AgreementSecretKey::from_seed(&[64; 32]).public_key();
        let foreign = PeerAdvertisement::issue(
            [8; 32],
            &foreign_root.did(),
            &foreign_device,
            foreign_routing,
            "10.0.3.1:9000".parse().unwrap(),
            1_000,
            2_000,
        )
        .unwrap();
        let mut freshness = FreshnessPins::new();
        let mut replay = ReplayCache::new(8).unwrap();
        let foreign = foreign
            .verify(
                [8; 32],
                1_500,
                &foreign_root.kel(),
                &foreign_device.kel(),
                &mut freshness,
                &mut replay,
            )
            .unwrap();
        assert_eq!(
            build_verified_onion_route(
                [
                    VerifiedRelay::new(&a, b"rendezvous"),
                    VerifiedRelay::new(&b, b"delivery"),
                    VerifiedRelay::new(&foreign, b"destination"),
                ],
                ConnectionId::from_bytes([1; 16]),
                PayloadSizeClass::Small,
                destination.public_key(),
                b"payload",
                1_500,
                10_000,
            ),
            Err(TransportSecurityError::WrongNetwork)
        );
    }

    #[test]
    fn three_distinct_verified_endpoints_build_an_onion() {
        let a = verified(10, "10.0.0.1:9000");
        let b = verified(20, "10.0.1.1:9000");
        let c = verified(30, "10.0.2.1:9000");
        let destination = AgreementSecretKey::from_seed(&[99; 32]);
        let packet = build_verified_onion_route(
            [
                VerifiedRelay::new(&a, b"rendezvous"),
                VerifiedRelay::new(&b, b"delivery"),
                VerifiedRelay::new(&c, b"destination"),
            ],
            ConnectionId::from_bytes([1; 16]),
            PayloadSizeClass::Small,
            destination.public_key(),
            b"payload",
            1_500,
            10_000,
        )
        .unwrap();
        assert_eq!(packet.hop_index, 0);
    }
    #[derive(Debug, Clone, Default)]
    struct RecordingBearer(std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>);
    impl Bearer for RecordingBearer {
        fn send(&mut self, frame: &[u8]) -> mini_bearer::Result<()> {
            self.0.lock().unwrap().push(frame.to_vec());
            Ok(())
        }
        fn recv(&mut self) -> mini_bearer::Result<Vec<u8>> {
            Err(BearerError::Closed)
        }
        fn try_recv(&mut self) -> mini_bearer::Result<Option<Vec<u8>>> {
            Ok(None)
        }
    }

    fn recording_connection(
        ad: &VerifiedPeerAdvertisement,
    ) -> (AuthenticatedConnection<RecordingBearer>, Channel) {
        let (initiator, hello) = mini_bearer::Initiator::start().unwrap();
        let (remote, response) = mini_bearer::Responder::respond(&hello).unwrap();
        let channel = initiator.finish(&response).unwrap();
        let peer = AuthenticatedPeer {
            root: ad.root().clone(),
            device: ad.device().clone(),
            endpoint_id: ad.endpoint_id(),
            routing_key: ad.routing_key(),
            capabilities: Capabilities::primary(),
            purpose: TransportPurpose::Relay,
        };
        (
            AuthenticatedConnection {
                bearer: RecordingBearer::default(),
                channel,
                peer,
                usable: true,
            },
            remote,
        )
    }

    fn request(tier: PrivacyTier, properties: Vec<ProtectionProperty>) -> TransportRequest {
        TransportRequest {
            privacy: PrivacyRequest { tier, properties },
            payload_size_class: PayloadSizeClass::Small,
        }
    }

    #[test]
    fn actual_dispatch_refuses_missing_executor_fallback_wrong_peer_and_unimplemented_properties_without_send(
    ) {
        let ad = verified(10, "10.0.0.1:9000");
        let (mut connection, mut remote) = recording_connection(&ad);
        for tier in [PrivacyTier::Mixed, PrivacyTier::Burst, PrivacyTier::Relayed] {
            assert!(dispatch_transport(
                &request(tier, vec![]),
                &mut connection,
                TransportTarget::Direct {
                    endpoint: ad.endpoint_id()
                },
                b"secret",
                b"aad"
            )
            .is_err());
        }
        assert_eq!(
            dispatch_transport(
                &request(PrivacyTier::Direct, vec![]),
                &mut connection,
                TransportTarget::Direct {
                    endpoint: crate::TransportEndpointId::from_bytes([0; 32])
                },
                b"secret",
                b"aad"
            ),
            Err(TransportSecurityError::EndpointMismatch)
        );
        for property in [
            ProtectionProperty::CounterpartyIpHiding,
            ProtectionProperty::StorageAvailability,
            ProtectionProperty::HumanUniquenessSignal,
            ProtectionProperty::TimingCorrelationResistance,
        ] {
            assert_eq!(
                dispatch_transport(
                    &request(PrivacyTier::Direct, vec![property]),
                    &mut connection,
                    TransportTarget::Direct {
                        endpoint: ad.endpoint_id()
                    },
                    b"secret",
                    b"aad"
                ),
                Err(TransportSecurityError::UnimplementedProtection)
            );
        }
        assert!(connection.bearer.0.lock().unwrap().is_empty());
        let receipt = dispatch_transport(
            &request(
                PrivacyTier::Direct,
                vec![ProtectionProperty::ContentSecrecy],
            ),
            &mut connection,
            TransportTarget::Direct {
                endpoint: ad.endpoint_id(),
            },
            b"secret",
            b"aad",
        )
        .unwrap();
        assert_eq!(receipt.tier(), PrivacyTier::Direct);
        assert_eq!(receipt.peer(), ad.endpoint_id());
        // Refusals did not consume the channel nonce or leak any earlier frame.
        assert_eq!(
            remote
                .open(&connection.bearer.0.lock().unwrap()[0], b"aad")
                .unwrap(),
            b"secret"
        );
    }

    #[test]
    fn relayed_dispatch_submits_only_a_verified_onion_and_never_falls_back_on_route_failure() {
        let a = verified(10, "10.0.0.1:9000");
        let b = verified(20, "10.0.1.1:9000");
        let c = verified(30, "10.0.2.1:9000");
        let destination = AgreementSecretKey::from_seed(&[99; 32]);
        let (mut connection, mut remote) = recording_connection(&a);
        let req = request(
            PrivacyTier::Relayed,
            vec![ProtectionProperty::CounterpartyIpHiding],
        );
        for bad in [true, false] {
            let target = TransportTarget::Onion {
                relays: [
                    VerifiedRelay::new(&a, b"rendezvous"),
                    VerifiedRelay::new(if bad { &a } else { &b }, b"delivery"),
                    VerifiedRelay::new(&c, b"destination"),
                ],
                destination_connection_id: ConnectionId::from_bytes([1; 16]),
                destination_key: destination.public_key(),
                now_ms: 1_500,
                expires_at_ms: 10_000,
            };
            let result = dispatch_transport(&req, &mut connection, target, b"secret", b"onion");
            if bad {
                assert_eq!(result, Err(TransportSecurityError::RouteEndpointReuse));
                assert!(connection.bearer.0.lock().unwrap().is_empty());
            } else {
                assert_eq!(result.unwrap().tier(), PrivacyTier::Relayed);
            }
        }
        let bytes = remote
            .open(&connection.bearer.0.lock().unwrap()[0], b"onion")
            .unwrap();
        assert!(!bytes.windows(6).any(|window| window == b"secret"));
        let mut packet = OnionPacket::from_bytes(&bytes).unwrap();
        for seed in [14, 24, 34] {
            let mut replay = mini_relay::OnionReplayCache::new(8).unwrap();
            let peeled = packet
                .peel(
                    &AgreementSecretKey::from_seed(&[seed; 32]),
                    1_500,
                    &mut replay,
                )
                .unwrap();
            match peeled.forward {
                mini_relay::OnionForward::Next(next) => packet = next,
                mini_relay::OnionForward::Destination(opaque) => {
                    let mut replay = mini_relay::OnionReplayCache::new(8).unwrap();
                    assert_eq!(
                        mini_relay::open_onion_destination(
                            &opaque,
                            &destination,
                            1_500,
                            &mut replay
                        )
                        .unwrap(),
                        b"secret"
                    );
                    return;
                }
            }
        }
        panic!("destination was not reached");
    }
}
