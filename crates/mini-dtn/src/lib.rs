//! # mini-dtn
//!
//! Deferred (store-carry-forward) transport for disruption-tolerant
//! operation -- disaster-mesh partition, satellite/lunar/Mars-class delay,
//! or any contact pattern where an end-to-end live session
//! (`mini_bearer::Bearer`/`mini_bearer::Channel`) cannot be assumed.
//! Gate #28 (roadmap issue #28), D-0513.
//!
//! ## What this crate is
//!
//! The priority/lifetime/admission model an external DTN/satellite
//! domain-expert design report (adopted D-0513) recommends for Mininet's
//! deferred transport layer, plus a bounded local scheduler
//! ([`InMemoryDeferredTransport`]) that enforces it — the
//! `mini-dtn::queue`/`mini-dtn::route` engineering pieces of that
//! report's own suggested `crates/mini-dtn` layout (D28-09).
//!
//! ## What this crate is NOT (yet)
//!
//! - **Not RFC 9171 BPv7 on the wire.** [`DeferredParcel`] is a
//!   Mininet-internal admission/scheduling record, not a CBOR-encoded
//!   BPv7 bundle. A real BPv7 codec needs byte-for-byte interoperability
//!   evidence against at least two independent BPv7 implementations
//!   before it can honestly claim standards compliance — exactly the
//!   discipline this tree already applies to Gate #97/#98's physical-
//!   hardware claims. Building an unverified from-scratch CBOR/BPv7 codec
//!   and calling it RFC 9171 would repeat the exact mistake
//!   `mini_bearer::discovery` was careful never to make about mDNS: that
//!   module's own docs say plainly it is not RFC 6762, rather than
//!   quietly hoping nobody checks.
//! - **Not a convergence layer.** No TCPCLv4, no LTP, no QUICCL, no real
//!   network I/O at all. [`InMemoryDeferredTransport`] is a
//!   single-process loopback scheduler for testing the admission/priority/
//!   expiry model in isolation.
//! - **Not durable.** [`InMemoryDeferredTransport`] holds everything in
//!   RAM. A real relay needs `mini-durable`-backed persistence before it
//!   can honestly report a "durably stored" status; this scaffold never
//!   claims that.
//! - **Not wired to any application crate.** `mini-objects`/
//!   `mini-messaging`/`mini-settlement`/`mini-forge` dispatch is future
//!   work — see `docs/gates/dtn-design-constraints.md` and the D-0513
//!   decision-log entry for the full remaining roadmap (P28-05 onward).
//!
//! No correctness property in this crate ever produces canonical monetary
//! or governance finality: a `mini-dtn` queue may drop, delay, duplicate,
//! or reorder a parcel; it can never forge, finalize, or authorize one.
//! See `mini_settlement::reconcile_v2` for the only place money can
//! actually become final, and [`crate::model::DeferredStatus`]'s docs for
//! exactly where this crate's authority ends.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

mod memory;
mod model;
mod transport;

pub use memory::{
    InMemoryDeferredTransport, DEFAULT_MAX_ENTRIES_PER_PRIORITY, DEFAULT_MAX_TOTAL_BYTES,
};
pub use model::{
    BundleAge, DeferredId, DeferredParcel, DeferredStatus, DeliveredParcel, DtnError, EndpointId,
    HopCount, LifetimeClass, Priority, Result, DEFAULT_HOP_LIMIT, MAX_CONTROL_PARCEL_BYTES,
    MAX_ENDPOINT_ID_BYTES, MAX_GENERATED_HOP_LIMIT, MAX_PARCEL_BYTES,
};
pub use transport::DeferredTransport;
