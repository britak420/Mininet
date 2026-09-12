//! Platform-neutral local-discovery record/route types (Gate #98, D-0514).
//!
//! An external Wi-Fi/local-discovery design report (adopted D-0514)
//! recommended a small set of Rust-facing types that a production
//! Android/iOS discovery adapter (`NsdManager`/Bonjour, still unbuilt in
//! this environment — see the crate-level docs) would build local service
//! records and route hints from. This module defines exactly those types
//! and *closes* them: a Wi-Fi (or Wi-Fi Direct/Aware, or any future local
//! link) discovery advertisement can never carry more than what these
//! structs' fixed fields allow.
//!
//! ## The one rule this module exists to enforce
//!
//! **Local network context contributes zero personhood, zero
//! physical-presence, and zero human-continuity score.** A shared SSID/
//! BSSID/subnet/hotspot/VPN path proves nothing about who is on the other
//! end — it is easier to fake than [`mini_presence`]'s UWB/BLE Channel
//! Sounding ranging evidence (Gate #97), and this crate's docs already
//! say so. [`LocalServiceRecord`] and [`LocalRouteHint`] therefore have no
//! field for a DID, a display name, a balance, a governance weight, a
//! router fingerprint, or any personhood/presence status — not because a
//! caller promises not to misuse an opaque byte field, but because no
//! such field exists on either type. A future change that adds one is a
//! Gate #98/personhood-boundary regression, not a routine feature add —
//! see `docs/gates/wifi-bearer-test-protocol.md`.
//!
//! Both types are pure route/connectivity metadata: where to try
//! connecting and what bounded, versioned capability bits it advertises.
//! Discovering a [`LocalServiceRecord`] proves only "an advertisement was
//! observed"; nothing here is trusted before [`crate::Channel`]
//! establishment and application-layer verification, exactly as this
//! crate's other discovery primitive ([`crate::discovery`]) already
//! documents.

use crate::error::{BearerError, Result};

/// Maximum encoded bytes for a [`LocalServiceRecord`]'s TXT-record form
/// (D98-023): Mininet's discovery capability data stays minimal and
/// bounded regardless of how many capability bits a future version adds.
pub const MAX_LOCAL_SERVICE_RECORD_TXT_BYTES: usize = 1024;

const TXT_MAGIC: [u8; 4] = *b"MSR1";
const TXT_LEN: usize = TXT_MAGIC.len() + 1 + 4 + 2; // magic + version + capabilities + port

/// A bounded, versioned local-service advertisement — the Rust-facing
/// analogue of a DNS-SD TXT record (RFC 6763) for a production discovery
/// adapter to publish, without this crate taking any position on the
/// actual mDNS/DNS-SD wire encoding a platform API (`NsdManager`/Bonjour)
/// already owns.
///
/// Deliberately closed to exactly these three fields — see the module
/// docs for why nothing else may ever be added here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalServiceRecord {
    /// This discovery profile's protocol version, so a future
    /// incompatible revision can coexist during rollout.
    pub protocol_version: u8,
    /// A bounded capability bitset. Never a personhood/role/balance
    /// signal — only what optional protocol features this listener
    /// supports (e.g. which transports it also offers).
    pub capabilities: u32,
    /// The dynamic local TCP port this listener is reachable on
    /// (D98-026: never a fixed/privileged port).
    pub port: u16,
}

impl LocalServiceRecord {
    /// Encode as bounded TXT-record bytes (D98-023/T98-021).
    pub fn to_txt_bytes(self) -> Vec<u8> {
        let mut w = Vec::with_capacity(TXT_LEN);
        w.extend_from_slice(&TXT_MAGIC);
        w.push(self.protocol_version);
        w.extend_from_slice(&self.capabilities.to_be_bytes());
        w.extend_from_slice(&self.port.to_be_bytes());
        debug_assert!(w.len() <= MAX_LOCAL_SERVICE_RECORD_TXT_BYTES);
        w
    }

    /// Decode TXT-record bytes discovered from an untrusted local peer.
    /// Bounds are checked before any field is interpreted
    /// (T98-022: an oversized or malformed Mininet-profile TXT value is
    /// rejected, never partially trusted).
    pub fn from_txt_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_LOCAL_SERVICE_RECORD_TXT_BYTES || bytes.len() != TXT_LEN {
            return Err(BearerError::MalformedLocalServiceRecord);
        }
        if bytes[..TXT_MAGIC.len()] != TXT_MAGIC {
            return Err(BearerError::MalformedLocalServiceRecord);
        }
        let protocol_version = bytes[TXT_MAGIC.len()];
        let capabilities = u32::from_be_bytes(
            bytes[TXT_MAGIC.len() + 1..TXT_MAGIC.len() + 5]
                .try_into()
                .expect("slice is exactly 4 bytes"),
        );
        let port = u16::from_be_bytes(
            bytes[TXT_MAGIC.len() + 5..TXT_MAGIC.len() + 7]
                .try_into()
                .expect("slice is exactly 2 bytes"),
        );
        Ok(LocalServiceRecord {
            protocol_version,
            capabilities,
            port,
        })
    }
}

/// An unauthenticated local route hint — where a discovered
/// [`LocalServiceRecord`] might currently be reachable. Route hints are
/// transient: a DHCP change, network handoff, or roam invalidates one
/// without changing any cryptographic peer identity above this layer
/// (D98-030/T98-040..044).
///
/// Deliberately closed to exactly these two fields, for the same reason
/// as [`LocalServiceRecord`] — see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRouteHint {
    /// Candidate addresses to try, in the order the platform discovery
    /// API returned them. IPv4 and IPv6 are both first-class (D98-027) —
    /// this type never assumes or prefers either.
    pub addresses: Vec<std::net::IpAddr>,
    /// The port [`LocalServiceRecord::port`] advertised.
    pub port: u16,
}

impl LocalRouteHint {
    /// The candidate socket addresses this hint currently offers, in
    /// order — hand these to [`crate::TcpBearer::connect`] one at a time
    /// until one succeeds.
    pub fn candidates(&self) -> Vec<std::net::SocketAddr> {
        self.addresses
            .iter()
            .map(|ip| std::net::SocketAddr::new(*ip, self.port))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn a_service_record_round_trips_through_txt_bytes() {
        let record = LocalServiceRecord {
            protocol_version: 1,
            capabilities: 0x0000_0007,
            port: 47812,
        };
        let bytes = record.to_txt_bytes();
        assert!(bytes.len() <= MAX_LOCAL_SERVICE_RECORD_TXT_BYTES);
        assert_eq!(LocalServiceRecord::from_txt_bytes(&bytes).unwrap(), record);
    }

    #[test]
    fn truncated_txt_bytes_are_rejected() {
        let record = LocalServiceRecord {
            protocol_version: 1,
            capabilities: 0,
            port: 1,
        };
        let bytes = record.to_txt_bytes();
        for cut in 0..bytes.len() {
            assert_eq!(
                LocalServiceRecord::from_txt_bytes(&bytes[..cut]).unwrap_err(),
                BearerError::MalformedLocalServiceRecord
            );
        }
    }

    #[test]
    fn trailing_bytes_are_rejected_not_silently_ignored() {
        let record = LocalServiceRecord {
            protocol_version: 1,
            capabilities: 0,
            port: 1,
        };
        let mut bytes = record.to_txt_bytes();
        bytes.push(0);
        assert_eq!(
            LocalServiceRecord::from_txt_bytes(&bytes).unwrap_err(),
            BearerError::MalformedLocalServiceRecord
        );
    }

    #[test]
    fn wrong_magic_is_rejected() {
        let mut bytes = LocalServiceRecord {
            protocol_version: 1,
            capabilities: 0,
            port: 1,
        }
        .to_txt_bytes();
        bytes[0] ^= 0xff;
        assert_eq!(
            LocalServiceRecord::from_txt_bytes(&bytes).unwrap_err(),
            BearerError::MalformedLocalServiceRecord
        );
    }

    #[test]
    fn an_oversized_declared_input_is_rejected_before_any_field_is_read() {
        let oversized = vec![0u8; MAX_LOCAL_SERVICE_RECORD_TXT_BYTES + 1];
        assert_eq!(
            LocalServiceRecord::from_txt_bytes(&oversized).unwrap_err(),
            BearerError::MalformedLocalServiceRecord
        );
    }

    #[test]
    fn a_route_hint_builds_candidates_for_both_address_families() {
        let hint = LocalRouteHint {
            addresses: vec![
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42)),
                IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
            ],
            port: 9443,
        };
        let candidates = hint.candidates();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|c| c.port() == 9443));
        assert!(candidates[0].is_ipv4());
        assert!(candidates[1].is_ipv6());
    }
}
