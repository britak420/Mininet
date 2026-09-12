use crate::model::{DeferredId, DeferredParcel, DeferredStatus, DeliveredParcel, Result};

/// A deferred (store-carry-forward) transport for parcels that may take
/// minutes to months to reach their destination -- the API shape the
/// Gate #28 design report calls `DeferredTransport` (D28-10), with one
/// deliberate change: every method here takes an explicit `now_ms`
/// instead of reading a system clock internally, because this crate makes
/// no assumption that any node has a trustworthy wall clock (D28-29:
/// "Bundle Age becomes mandatory" for exactly this reason). Callers supply
/// whatever time source they trust, the same caller-supplied-time
/// discipline `mini_settlement::reconcile` already uses.
///
/// No method here can ever answer a canonical-finality question about
/// what it carries (D28-12/76: a BP agent is not a trust root). A
/// `DeferredTransport` may drop, delay, duplicate, or reorder a parcel; it
/// may never forge, finalize, or authorize one.
pub trait DeferredTransport {
    /// Admit `parcel` for eventual delivery. Returns the same
    /// [`DeferredId`] on a duplicate admission (D28-41) rather than
    /// erroring or double-storing.
    fn enqueue(&mut self, parcel: DeferredParcel, now_ms: u64) -> Result<DeferredId>;

    /// Drain up to `limit` parcels that are ready to leave local
    /// scheduling, highest [`crate::model::Priority`] weight first
    /// (D28-34's deficit round robin). Also reaps any parcel whose
    /// [`crate::model::LifetimeClass`] window has elapsed as of `now_ms`
    /// before scheduling anything (D28-113: expiry first).
    fn poll_delivered(&mut self, limit: usize, now_ms: u64) -> Result<Vec<DeliveredParcel>>;

    /// Where an admitted parcel currently stands.
    /// [`crate::model::DtnError::UnknownParcel`] if this id was never
    /// admitted.
    fn status(&self, id: &DeferredId) -> Result<DeferredStatus>;

    /// Remove a still-[`crate::model::DeferredStatus::QueuedLocal`] parcel
    /// from local scheduling. Only ever a local decision (D28-115: a
    /// user's queued original is not transit cache to be seized) --
    /// nothing here reaches into a remote relay's copy.
    fn cancel_local(&mut self, id: &DeferredId) -> Result<()>;
}
