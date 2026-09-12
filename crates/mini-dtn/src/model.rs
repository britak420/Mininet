//! Types shared by [`crate::transport::DeferredTransport`] and
//! [`crate::memory::InMemoryDeferredTransport`].

use core::fmt;

use mini_crypto::HashAlgorithm;

/// Maximum payload bytes for an ordinary deferred parcel (D28-20 of the
/// Gate #28 design report) -- stricter than `mini-objects`' 8 MiB global
/// object cap because a single interrupted transfer must stay survivable
/// on very low-rate links. Larger application data is expected to arrive
/// as a manifest/chunk reference, not a single oversized parcel.
pub const MAX_PARCEL_BYTES: usize = 1_048_576;

/// Maximum payload bytes for a [`Priority::P0`] or [`Priority::P1`] parcel
/// (D28-21) -- control/money/governance traffic stays small so admission
/// control can afford to check it cheaply and completely.
pub const MAX_CONTROL_PARCEL_BYTES: usize = 65_536;

/// Maximum bytes for an [`EndpointId`]. Endpoints are opaque route
/// capabilities (D28-52), never a directory entry, so there is no reason
/// for one to be large.
pub const MAX_ENDPOINT_ID_BYTES: usize = 256;

/// Default hop-count ceiling for an ordinary disaster/terrestrial bundle
/// (D28-03 of the design report).
pub const DEFAULT_HOP_LIMIT: u8 = 32;

/// Maximum hop-count ceiling this node will ever place on a
/// Mininet-generated bundle (D28-03) -- a receiving interoperable BP node
/// may still accept a larger externally-sourced limit subject to its own
/// local policy; this bound is about what *this* node generates.
pub const MAX_GENERATED_HOP_LIMIT: u8 = 64;

const DEFERRED_ID_DOMAIN: &[u8] = b"mini-dtn/deferred-id/v1";

/// Errors from admission, scheduling, or lookup in this crate. Never a
/// statement about whether a parcel's *application* content is valid --
/// see the module docs for the transport/application boundary this crate
/// stays on the transport side of.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DtnError {
    /// Payload exceeds [`MAX_PARCEL_BYTES`].
    ParcelTooLarge,
    /// A [`Priority::P0`]/[`Priority::P1`] payload exceeds
    /// [`MAX_CONTROL_PARCEL_BYTES`] -- D28-33: a caller cannot self-label
    /// arbitrary bytes P0/P1 and expect unlimited critical-class size.
    ControlParcelTooLarge,
    /// An [`EndpointId`] was empty or exceeded [`MAX_ENDPOINT_ID_BYTES`].
    InvalidEndpoint,
    /// The local queue is at its bounded capacity (D28-39: every table has
    /// a byte/entry limit) and cannot admit another distinct parcel.
    QueueFull,
    /// No admitted parcel is tracked under this [`DeferredId`] (never
    /// admitted, already expired-and-reaped, or already cancelled).
    UnknownParcel,
    /// A generated [`HopCount`] limit exceeded [`MAX_GENERATED_HOP_LIMIT`].
    HopLimitTooLarge,
}

impl fmt::Display for DtnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DtnError::ParcelTooLarge => write!(
                f,
                "parcel payload exceeds the deferred-transport size bound"
            ),
            DtnError::ControlParcelTooLarge => {
                write!(
                    f,
                    "P0/P1 parcel payload exceeds the control-class size bound"
                )
            }
            DtnError::InvalidEndpoint => write!(f, "endpoint id is empty or too large"),
            DtnError::QueueFull => write!(f, "deferred-transport local queue is at capacity"),
            DtnError::UnknownParcel => write!(f, "no admitted parcel is tracked under this id"),
            DtnError::HopLimitTooLarge => {
                write!(f, "generated hop-count limit exceeds the maximum")
            }
        }
    }
}

impl std::error::Error for DtnError {}

/// Result alias for this crate.
pub type Result<T> = core::result::Result<T, DtnError>;

/// An opaque, rotatable route capability naming a deferred-transport
/// destination -- never a stable `did:mini` root (D28-51/52): setting a
/// private user's public DTN endpoint to their root identity by default
/// would create cross-path metadata linkage this crate has no business
/// creating. Callers that want a stable *application* identity bind it
/// above this layer (an authenticated Mininet relationship, a signed
/// capability object), exactly as [`crate`]'s module docs describe.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EndpointId(String);

impl EndpointId {
    /// Wrap `id` as an endpoint capability, rejecting empty or oversized
    /// values before anything is stored.
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if id.is_empty() || id.len() > MAX_ENDPOINT_ID_BYTES {
            return Err(DtnError::InvalidEndpoint);
        }
        Ok(EndpointId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Local scheduling priority (D28-32) -- functional, never purchased
/// (D28-31): no balance, fee, or role can raise a parcel's class, only its
/// actual content type and admission-time policy can.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Protocol-critical bounded control (finality/header safety proofs,
    /// narrowly-defined safety objects).
    P0,
    /// Human-control / canonical-submission traffic (governance votes,
    /// canonical transaction submissions, `PaymentClaimV2` delivery).
    P1,
    /// Interactive traffic (direct messages, CRDT ops, ordinary posts).
    P2,
    /// Bulk (content chunks, archival sync, non-urgent replication).
    P3,
}

impl Priority {
    /// Deficit-round-robin weight (D28-34): nominal shares 40/30/20/10
    /// under continuous contention, with any unused share immediately
    /// borrowable by other non-empty queues (D28-36: bulk cannot starve
    /// forever).
    pub const fn drr_weight(self) -> u32 {
        match self {
            Priority::P0 => 4,
            Priority::P1 => 3,
            Priority::P2 => 2,
            Priority::P3 => 1,
        }
    }

    /// All four classes, highest priority first -- the fixed iteration
    /// order [`crate::memory::InMemoryDeferredTransport`]'s scheduler uses.
    pub const fn all() -> [Priority; 4] {
        [Priority::P0, Priority::P1, Priority::P2, Priority::P3]
    }

    /// Whether this class is subject to [`MAX_CONTROL_PARCEL_BYTES`]
    /// instead of the looser [`MAX_PARCEL_BYTES`] (D28-21).
    pub const fn is_control_class(self) -> bool {
        matches!(self, Priority::P0 | Priority::P1)
    }
}

/// Bounded transport-retention classes (D28-27) a source application
/// selects from -- never an arbitrary caller-chosen duration, so no
/// parcel can demand "retain forever."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifetimeClass {
    /// 1 hour.
    Ephemeral,
    /// 24 hours.
    Short,
    /// 30 days.
    Standard,
    /// 180 days.
    Archival,
    /// 365 days -- the longest class, sized for prolonged disaster/
    /// deep-space carriage (D28-27).
    DeepSpace,
}

impl LifetimeClass {
    /// This class's retention window, in milliseconds. A local relay may
    /// still shorten its own *effective* retention under resource pressure
    /// (D28-28) -- this is the class's nominal ceiling, not a promise any
    /// one relay keeps a copy that long.
    pub const fn duration_ms(self) -> u64 {
        const HOUR_MS: u64 = 3_600_000;
        const DAY_MS: u64 = 24 * HOUR_MS;
        match self {
            LifetimeClass::Ephemeral => HOUR_MS,
            LifetimeClass::Short => DAY_MS,
            LifetimeClass::Standard => 30 * DAY_MS,
            LifetimeClass::Archival => 180 * DAY_MS,
            LifetimeClass::DeepSpace => 365 * DAY_MS,
        }
    }
}

/// RFC 9171's Bundle Age extension value, in milliseconds since creation
/// -- how a node without a trustworthy wall clock still expires transport
/// state (D28-29/D28-119/D28-120: no NTP/GPS/cellular clock is ever a
/// correctness authority). This crate never reads the system clock
/// itself; callers supply `now`/age from whatever source they trust
/// (device clock, contact-derived counter, or a real Bundle Age chain),
/// exactly the same caller-supplied-time discipline
/// `mini_settlement::reconcile` already uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BundleAge(pub u64);

impl BundleAge {
    /// Whether this age has exceeded `lifetime`'s retention window.
    pub const fn is_expired(self, lifetime: LifetimeClass) -> bool {
        self.0 >= lifetime.duration_ms()
    }
}

/// RFC 9171's Hop Count extension: a bounded loop breaker, never a
/// distance/authority signal. `limit` is fixed at construction; `count`
/// increments once per hop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HopCount {
    limit: u8,
    count: u8,
}

impl HopCount {
    /// A fresh hop count generated by this node, `limit` capped at
    /// [`MAX_GENERATED_HOP_LIMIT`] (D28-03: route loops must not become
    /// infinite storage/bandwidth debt).
    pub fn new_generated(limit: u8) -> Result<Self> {
        if limit > MAX_GENERATED_HOP_LIMIT {
            return Err(DtnError::HopLimitTooLarge);
        }
        Ok(HopCount { limit, count: 0 })
    }

    pub const fn limit(self) -> u8 {
        self.limit
    }

    pub const fn count(self) -> u8 {
        self.count
    }

    /// One more hop. `None` once incrementing would meet or exceed
    /// `limit` -- the caller must drop the bundle rather than forward it
    /// further (D28-162's hop-count-loop defense).
    pub const fn increment(self) -> Option<Self> {
        if self.count >= self.limit {
            return None;
        }
        Some(HopCount {
            limit: self.limit,
            count: self.count + 1,
        })
    }
}

/// Content-addressed transport identity of an admitted
/// [`DeferredParcel`] -- distinct from any application semantic id
/// (D28-15): two parcels carrying the same Mininet application object can
/// have different [`DeferredId`]s if any transport metadata differs, and
/// the application still processes the object once via its own id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeferredId(pub [u8; 32]);

/// One unit of deferred (store-carry-forward) transport admission -- the
/// Mininet-internal record [`crate::transport::DeferredTransport::enqueue`]
/// accepts. Not a BPv7 bundle; see the crate docs for why that distinction
/// matters right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredParcel {
    pub destination: EndpointId,
    pub payload: Vec<u8>,
    pub priority: Priority,
    pub lifetime: LifetimeClass,
    /// Application semantic identity for de-dup (D28-15/41) -- e.g. a
    /// Mininet object id or transaction id. `None` when the caller has no
    /// cheaper identity than the parcel bytes themselves, in which case
    /// [`DeferredId`] alone is used for de-dup.
    pub semantic_id: Option<[u8; 32]>,
}

impl DeferredParcel {
    /// This parcel's transport identity (D28-15). Two structurally
    /// identical parcels always compute the same id; enqueuing the same
    /// parcel twice is therefore always de-duplicated, not merely a
    /// same-semantic-id de-dup coincidence.
    pub fn transport_id(&self) -> DeferredId {
        let mut msg = Vec::with_capacity(
            DEFERRED_ID_DOMAIN.len()
                + 4
                + self.destination.as_str().len()
                + 4
                + self.payload.len()
                + 1
                + 1
                + 33,
        );
        msg.extend_from_slice(DEFERRED_ID_DOMAIN);
        let dest = self.destination.as_str().as_bytes();
        msg.extend_from_slice(&(dest.len() as u32).to_be_bytes());
        msg.extend_from_slice(dest);
        msg.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        msg.extend_from_slice(&self.payload);
        msg.push(u8_from_priority(self.priority));
        msg.push(lifetime_tag(self.lifetime));
        match self.semantic_id {
            Some(id) => {
                msg.push(1);
                msg.extend_from_slice(&id);
            }
            None => msg.push(0),
        }
        DeferredId(HashAlgorithm::Blake3.digest(&msg))
    }
}

fn u8_from_priority(p: Priority) -> u8 {
    match p {
        Priority::P0 => 0,
        Priority::P1 => 1,
        Priority::P2 => 2,
        Priority::P3 => 3,
    }
}

fn lifetime_tag(l: LifetimeClass) -> u8 {
    match l {
        LifetimeClass::Ephemeral => 0,
        LifetimeClass::Short => 1,
        LifetimeClass::Standard => 2,
        LifetimeClass::Archival => 3,
        LifetimeClass::DeepSpace => 4,
    }
}

/// A parcel [`crate::transport::DeferredTransport::poll_delivered`]
/// returned -- ready to leave this node's local scheduling, never itself
/// a statement of confirmed remote delivery (D28-12: delivery receipt is
/// not application finality).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveredParcel {
    pub id: DeferredId,
    pub destination: EndpointId,
    pub payload: Vec<u8>,
}

/// Where a parcel stands inside local transport scheduling. Deliberately
/// smaller than the design report's full application-facing state list
/// (`ObjectAccepted`/`SubmittedCanonical`/`IncludedCanonical`/
/// `FinalizedCanonical`/`RejectedApplication`, D28-12) -- those are states
/// of whatever consumes a [`DeliveredParcel`] (`mini-settlement`,
/// `mini-forge`, ...), not of this transport-only crate, and this crate
/// does not claim to produce them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredStatus {
    /// Admitted and waiting in local priority scheduling.
    QueuedLocal,
    /// Returned by [`crate::transport::DeferredTransport::poll_delivered`]
    /// -- left local scheduling for onward handling.
    BundleDelivered,
    /// Aged out of its [`LifetimeClass`] window before being scheduled out
    /// (D28-113: expiry checked before scheduling on every poll).
    ExpiredTransport,
    /// Removed locally via
    /// [`crate::transport::DeferredTransport::cancel_local`] before being
    /// scheduled out.
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_endpoint_id_is_rejected() {
        assert_eq!(EndpointId::new("").unwrap_err(), DtnError::InvalidEndpoint);
    }

    #[test]
    fn an_oversized_endpoint_id_is_rejected() {
        let too_long = "x".repeat(MAX_ENDPOINT_ID_BYTES + 1);
        assert_eq!(
            EndpointId::new(too_long).unwrap_err(),
            DtnError::InvalidEndpoint
        );
    }

    #[test]
    fn hop_count_increments_up_to_but_not_past_its_limit() {
        let mut hop = HopCount::new_generated(2).unwrap();
        assert_eq!(hop.count(), 0);
        hop = hop.increment().unwrap();
        assert_eq!(hop.count(), 1);
        hop = hop.increment().unwrap();
        assert_eq!(hop.count(), 2);
        assert!(
            hop.increment().is_none(),
            "a bundle at its hop limit must not forward further"
        );
    }

    #[test]
    fn a_generated_hop_limit_above_the_maximum_is_rejected() {
        assert_eq!(
            HopCount::new_generated(MAX_GENERATED_HOP_LIMIT + 1).unwrap_err(),
            DtnError::HopLimitTooLarge
        );
        assert!(HopCount::new_generated(MAX_GENERATED_HOP_LIMIT).is_ok());
    }

    #[test]
    fn bundle_age_expiry_is_a_pure_threshold_comparison() {
        let lifetime = LifetimeClass::Ephemeral;
        assert!(!BundleAge(lifetime.duration_ms() - 1).is_expired(lifetime));
        assert!(BundleAge(lifetime.duration_ms()).is_expired(lifetime));
        assert!(BundleAge(lifetime.duration_ms() + 1).is_expired(lifetime));
    }

    #[test]
    fn two_structurally_identical_parcels_have_the_same_transport_id() {
        let a = DeferredParcel {
            destination: EndpointId::new("dtn:peer-a").unwrap(),
            payload: b"hello".to_vec(),
            priority: Priority::P2,
            lifetime: LifetimeClass::Short,
            semantic_id: None,
        };
        let b = DeferredParcel {
            destination: EndpointId::new("dtn:peer-a").unwrap(),
            payload: b"hello".to_vec(),
            priority: Priority::P2,
            lifetime: LifetimeClass::Short,
            semantic_id: None,
        };
        assert_eq!(a.transport_id(), b.transport_id());
    }

    #[test]
    fn changing_any_field_changes_the_transport_id() {
        let base = DeferredParcel {
            destination: EndpointId::new("dtn:peer-a").unwrap(),
            payload: b"hello".to_vec(),
            priority: Priority::P2,
            lifetime: LifetimeClass::Short,
            semantic_id: None,
        };
        let mut different_dest = base.clone();
        different_dest.destination = EndpointId::new("dtn:peer-b").unwrap();
        assert_ne!(base.transport_id(), different_dest.transport_id());

        let mut different_payload = base.clone();
        different_payload.payload = b"world".to_vec();
        assert_ne!(base.transport_id(), different_payload.transport_id());

        let mut different_priority = base.clone();
        different_priority.priority = Priority::P0;
        assert_ne!(base.transport_id(), different_priority.transport_id());

        let mut different_lifetime = base.clone();
        different_lifetime.lifetime = LifetimeClass::Archival;
        assert_ne!(base.transport_id(), different_lifetime.transport_id());

        let mut different_semantic = base.clone();
        different_semantic.semantic_id = Some([7u8; 32]);
        assert_ne!(base.transport_id(), different_semantic.transport_id());
    }

    #[test]
    fn drr_weights_match_the_design_reports_nominal_shares() {
        assert_eq!(Priority::P0.drr_weight(), 4);
        assert_eq!(Priority::P1.drr_weight(), 3);
        assert_eq!(Priority::P2.drr_weight(), 2);
        assert_eq!(Priority::P3.drr_weight(), 1);
    }

    #[test]
    fn only_p0_and_p1_are_the_control_class() {
        assert!(Priority::P0.is_control_class());
        assert!(Priority::P1.is_control_class());
        assert!(!Priority::P2.is_control_class());
        assert!(!Priority::P3.is_control_class());
    }
}
