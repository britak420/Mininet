//! A bounded, single-process local scheduler exercising the admission/
//! priority/expiry model without any real network, convergence layer, or
//! durable storage. See the crate docs for exactly what this is not yet.

use std::collections::{HashMap, VecDeque};

use crate::model::{
    DeferredId, DeferredParcel, DeferredStatus, DeliveredParcel, DtnError, Priority, Result,
    MAX_CONTROL_PARCEL_BYTES, MAX_PARCEL_BYTES,
};
use crate::transport::DeferredTransport;

/// Default per-priority queue depth for [`InMemoryDeferredTransport`].
/// An in-process test/demo default, not the real durable-relay disk
/// budget the design report sizes for an actual mobile transit relay
/// (D28-37/38) -- that bound belongs to a future `mini-durable`-backed
/// implementation of this trait, not this in-memory scaffold.
pub const DEFAULT_MAX_ENTRIES_PER_PRIORITY: usize = 4_096;

/// Default total admitted-payload byte budget for
/// [`InMemoryDeferredTransport`]. Same caveat as
/// [`DEFAULT_MAX_ENTRIES_PER_PRIORITY`].
pub const DEFAULT_MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
struct Entry {
    parcel: DeferredParcel,
    id: DeferredId,
    deadline_ms: u64,
    status: DeferredStatus,
}

/// A single-node loopback [`DeferredTransport`]: `enqueue` admits and
/// schedules a parcel, `poll_delivered` hands it back out in
/// priority-weighted order once it is "ready" (immediately, in this
/// scaffold — there is no real contact/link to wait for). Useful for unit
/// testing the admission/priority/expiry/dedup model this crate defines
/// in isolation from any real transport.
#[derive(Debug)]
pub struct InMemoryDeferredTransport {
    queues: HashMap<Priority, VecDeque<DeferredId>>,
    entries: HashMap<DeferredId, Entry>,
    semantic_index: HashMap<[u8; 32], DeferredId>,
    total_bytes: usize,
    max_entries_per_priority: usize,
    max_total_bytes: usize,
    drr_deficit: HashMap<Priority, u32>,
}

impl Default for InMemoryDeferredTransport {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ENTRIES_PER_PRIORITY, DEFAULT_MAX_TOTAL_BYTES)
    }
}

impl InMemoryDeferredTransport {
    /// A new, empty scheduler with explicit admission bounds.
    pub fn new(max_entries_per_priority: usize, max_total_bytes: usize) -> Self {
        let mut queues = HashMap::new();
        let mut drr_deficit = HashMap::new();
        for p in Priority::all() {
            queues.insert(p, VecDeque::new());
            drr_deficit.insert(p, 0);
        }
        InMemoryDeferredTransport {
            queues,
            entries: HashMap::new(),
            semantic_index: HashMap::new(),
            total_bytes: 0,
            max_entries_per_priority,
            max_total_bytes,
            drr_deficit,
        }
    }

    /// Number of parcels still queued (not yet delivered, expired, or
    /// cancelled) for one priority class.
    pub fn queued_count(&self, priority: Priority) -> usize {
        self.queues.get(&priority).map_or(0, VecDeque::len)
    }

    fn admit_cheap_checks(parcel: &DeferredParcel) -> Result<()> {
        // Cheap bounds before anything else (D28-42: validate cheaply
        // before expensive work; here "expensive" is unbounded memory
        // growth, not cryptography, but the ordering principle is the
        // same -- check the size the caller already declared before
        // acting on it).
        if parcel.payload.len() > MAX_PARCEL_BYTES {
            return Err(DtnError::ParcelTooLarge);
        }
        if parcel.priority.is_control_class() && parcel.payload.len() > MAX_CONTROL_PARCEL_BYTES {
            return Err(DtnError::ControlParcelTooLarge);
        }
        Ok(())
    }

    fn reap_expired(&mut self, now_ms: u64) {
        for priority in Priority::all() {
            let queue = self
                .queues
                .get_mut(&priority)
                .expect("all priorities present");
            let mut still_queued = VecDeque::with_capacity(queue.len());
            for id in queue.drain(..) {
                let expired = self
                    .entries
                    .get(&id)
                    .map(|e| now_ms >= e.deadline_ms)
                    .unwrap_or(false);
                if expired {
                    if let Some(entry) = self.entries.get_mut(&id) {
                        self.total_bytes =
                            self.total_bytes.saturating_sub(entry.parcel.payload.len());
                        entry.status = DeferredStatus::ExpiredTransport;
                    }
                } else {
                    still_queued.push_back(id);
                }
            }
            *queue = still_queued;
        }
    }
}

impl DeferredTransport for InMemoryDeferredTransport {
    fn enqueue(&mut self, parcel: DeferredParcel, now_ms: u64) -> Result<DeferredId> {
        Self::admit_cheap_checks(&parcel)?;

        let id = parcel.transport_id();

        // Duplicate admission: refresh nothing, just hand back the
        // existing id (D28-41) -- a duplicate does not need a second copy
        // of the same payload taking up queue budget.
        if self.entries.contains_key(&id) {
            return Ok(id);
        }
        if let Some(semantic_id) = parcel.semantic_id {
            if let Some(existing) = self.semantic_index.get(&semantic_id) {
                return Ok(*existing);
            }
        }

        let queue = self
            .queues
            .get(&parcel.priority)
            .expect("all priorities present");
        if queue.len() >= self.max_entries_per_priority {
            return Err(DtnError::QueueFull);
        }
        if self.total_bytes.saturating_add(parcel.payload.len()) > self.max_total_bytes {
            return Err(DtnError::QueueFull);
        }

        let deadline_ms = now_ms.saturating_add(parcel.lifetime.duration_ms());
        self.total_bytes += parcel.payload.len();
        if let Some(semantic_id) = parcel.semantic_id {
            self.semantic_index.insert(semantic_id, id);
        }
        let priority = parcel.priority;
        self.entries.insert(
            id,
            Entry {
                parcel,
                id,
                deadline_ms,
                status: DeferredStatus::QueuedLocal,
            },
        );
        self.queues
            .get_mut(&priority)
            .expect("all priorities present")
            .push_back(id);
        Ok(id)
    }

    fn poll_delivered(&mut self, limit: usize, now_ms: u64) -> Result<Vec<DeliveredParcel>> {
        self.reap_expired(now_ms);

        let mut delivered = Vec::new();
        if limit == 0 {
            return Ok(delivered);
        }

        // Deficit round robin (D28-34): each round, every non-empty queue
        // earns its weight in deficit, then dequeues one item per unit of
        // deficit it holds. An empty queue neither earns nor spends
        // deficit, so its unused share is immediately available to
        // whichever queues are actually non-empty (D28-36: bulk still
        // gets served once P0-P2 drain, never starved forever).
        while delivered.len() < limit {
            let any_nonempty = Priority::all().iter().any(|p| !self.queues[p].is_empty());
            if !any_nonempty {
                break;
            }
            let mut made_progress = false;
            for priority in Priority::all() {
                if delivered.len() >= limit {
                    break;
                }
                if self.queues[&priority].is_empty() {
                    continue;
                }
                let deficit = self.drr_deficit.entry(priority).or_insert(0);
                *deficit += priority.drr_weight();
                while *deficit > 0 && delivered.len() < limit {
                    let Some(id) = self.queues.get_mut(&priority).and_then(VecDeque::pop_front)
                    else {
                        break;
                    };
                    *deficit -= 1;
                    made_progress = true;
                    let Some(entry) = self.entries.get_mut(&id) else {
                        continue;
                    };
                    entry.status = DeferredStatus::BundleDelivered;
                    self.total_bytes = self.total_bytes.saturating_sub(entry.parcel.payload.len());
                    delivered.push(DeliveredParcel {
                        id: entry.id,
                        destination: entry.parcel.destination.clone(),
                        payload: entry.parcel.payload.clone(),
                    });
                }
            }
            if !made_progress {
                break;
            }
        }
        Ok(delivered)
    }

    fn status(&self, id: &DeferredId) -> Result<DeferredStatus> {
        self.entries
            .get(id)
            .map(|e| e.status)
            .ok_or(DtnError::UnknownParcel)
    }

    fn cancel_local(&mut self, id: &DeferredId) -> Result<()> {
        let entry = self.entries.get_mut(id).ok_or(DtnError::UnknownParcel)?;
        if entry.status != DeferredStatus::QueuedLocal {
            return Err(DtnError::UnknownParcel);
        }
        entry.status = DeferredStatus::Cancelled;
        self.total_bytes = self.total_bytes.saturating_sub(entry.parcel.payload.len());
        let priority = entry.parcel.priority;
        if let Some(queue) = self.queues.get_mut(&priority) {
            queue.retain(|queued_id| queued_id != id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EndpointId, LifetimeClass};

    fn parcel(priority: Priority, payload: &[u8]) -> DeferredParcel {
        DeferredParcel {
            destination: EndpointId::new("dtn:peer-a").unwrap(),
            payload: payload.to_vec(),
            priority,
            lifetime: LifetimeClass::Short,
            semantic_id: None,
        }
    }

    #[test]
    fn an_admitted_parcel_is_queued_local_until_polled() {
        let mut t = InMemoryDeferredTransport::default();
        let id = t.enqueue(parcel(Priority::P2, b"hello"), 0).unwrap();
        assert_eq!(t.status(&id).unwrap(), DeferredStatus::QueuedLocal);
    }

    #[test]
    fn polling_moves_a_parcel_to_delivered() {
        let mut t = InMemoryDeferredTransport::default();
        let id = t.enqueue(parcel(Priority::P2, b"hello"), 0).unwrap();
        let delivered = t.poll_delivered(10, 0).unwrap();
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].id, id);
        assert_eq!(t.status(&id).unwrap(), DeferredStatus::BundleDelivered);
    }

    #[test]
    fn enqueuing_the_same_parcel_twice_is_deduplicated_by_transport_id() {
        let mut t = InMemoryDeferredTransport::default();
        let a = t.enqueue(parcel(Priority::P2, b"hello"), 0).unwrap();
        let b = t.enqueue(parcel(Priority::P2, b"hello"), 0).unwrap();
        assert_eq!(a, b);
        assert_eq!(
            t.queued_count(Priority::P2),
            1,
            "must not double-store an identical parcel"
        );
    }

    #[test]
    fn enqueuing_two_parcels_sharing_a_semantic_id_is_deduplicated() {
        let mut t = InMemoryDeferredTransport::default();
        let mut first = parcel(Priority::P2, b"version-1");
        first.semantic_id = Some([9u8; 32]);
        let mut second = parcel(Priority::P2, b"version-2-different-bytes-same-object");
        second.semantic_id = Some([9u8; 32]);

        let a = t.enqueue(first, 0).unwrap();
        let b = t.enqueue(second, 0).unwrap();
        assert_eq!(
            a, b,
            "same application semantic id must not be stored twice"
        );
        assert_eq!(t.queued_count(Priority::P2), 1);
    }

    #[test]
    fn an_oversized_payload_is_rejected_before_admission() {
        let mut t = InMemoryDeferredTransport::default();
        let big = vec![0u8; crate::model::MAX_PARCEL_BYTES + 1];
        assert_eq!(
            t.enqueue(parcel(Priority::P3, &big), 0).unwrap_err(),
            DtnError::ParcelTooLarge
        );
    }

    #[test]
    fn a_p0_payload_over_the_control_bound_is_rejected_even_though_it_fits_the_general_bound() {
        let mut t = InMemoryDeferredTransport::default();
        let over_control = vec![0u8; crate::model::MAX_CONTROL_PARCEL_BYTES + 1];
        assert_eq!(
            t.enqueue(parcel(Priority::P0, &over_control), 0)
                .unwrap_err(),
            DtnError::ControlParcelTooLarge
        );
        // The same size is fine for a bulk-class parcel.
        assert!(t.enqueue(parcel(Priority::P3, &over_control), 0).is_ok());
    }

    #[test]
    fn a_full_priority_queue_rejects_further_admission() {
        let mut t = InMemoryDeferredTransport::new(2, DEFAULT_MAX_TOTAL_BYTES);
        t.enqueue(parcel(Priority::P3, b"a"), 0).unwrap();
        t.enqueue(parcel(Priority::P3, b"b"), 0).unwrap();
        assert_eq!(
            t.enqueue(parcel(Priority::P3, b"c"), 0).unwrap_err(),
            DtnError::QueueFull
        );
    }

    #[test]
    fn a_parcel_past_its_lifetime_window_expires_and_is_never_delivered() {
        let mut t = InMemoryDeferredTransport::default();
        let mut short_lived = parcel(Priority::P2, b"hello");
        short_lived.lifetime = LifetimeClass::Ephemeral;
        let id = t.enqueue(short_lived, 0).unwrap();

        let past_deadline = LifetimeClass::Ephemeral.duration_ms();
        let delivered = t.poll_delivered(10, past_deadline).unwrap();
        assert!(delivered.is_empty());
        assert_eq!(t.status(&id).unwrap(), DeferredStatus::ExpiredTransport);
    }

    #[test]
    fn cancel_local_removes_a_still_queued_parcel() {
        let mut t = InMemoryDeferredTransport::default();
        let id = t.enqueue(parcel(Priority::P2, b"hello"), 0).unwrap();
        t.cancel_local(&id).unwrap();
        assert_eq!(t.status(&id).unwrap(), DeferredStatus::Cancelled);
        let delivered = t.poll_delivered(10, 0).unwrap();
        assert!(
            delivered.is_empty(),
            "a cancelled parcel must never be delivered"
        );
    }

    #[test]
    fn cancel_local_on_an_already_delivered_parcel_fails() {
        let mut t = InMemoryDeferredTransport::default();
        let id = t.enqueue(parcel(Priority::P2, b"hello"), 0).unwrap();
        t.poll_delivered(10, 0).unwrap();
        assert_eq!(t.cancel_local(&id).unwrap_err(), DtnError::UnknownParcel);
    }

    #[test]
    fn status_of_an_id_that_was_never_admitted_is_unknown() {
        let t = InMemoryDeferredTransport::default();
        let bogus = DeferredId([0u8; 32]);
        assert_eq!(t.status(&bogus).unwrap_err(), DtnError::UnknownParcel);
    }

    #[test]
    fn higher_priority_classes_are_scheduled_out_first_under_contention() {
        let mut t = InMemoryDeferredTransport::default();
        // Fill every class heavily so the DRR scheduler's weighting, not
        // arrival order, decides who gets served first.
        for i in 0..8u8 {
            t.enqueue(parcel(Priority::P3, &[i]), 0).unwrap();
            t.enqueue(parcel(Priority::P0, &[i]), 0).unwrap();
        }
        // One DRR round admits P0's full weight (4) before P3 gets its
        // weight (1) -- so among the first 4 delivered, none should be P3.
        let delivered = t.poll_delivered(4, 0).unwrap();
        assert_eq!(delivered.len(), 4);
        for d in &delivered {
            assert_eq!(d.payload.len(), 1);
        }
        // All four should have come from the P0 queue: confirm P0's queue
        // shrank by exactly 4 and P3's did not shrink at all yet.
        assert_eq!(t.queued_count(Priority::P0), 4);
        assert_eq!(t.queued_count(Priority::P3), 8);
    }

    #[test]
    fn bulk_traffic_is_never_starved_forever_once_higher_classes_drain() {
        let mut t = InMemoryDeferredTransport::default();
        t.enqueue(parcel(Priority::P0, b"critical"), 0).unwrap();
        for i in 0..5u8 {
            t.enqueue(parcel(Priority::P3, &[i]), 0).unwrap();
        }
        // Drain everything -- once P0 empties, P3 must still get served
        // rather than being starved indefinitely by an empty higher class.
        let delivered = t.poll_delivered(100, 0).unwrap();
        assert_eq!(
            delivered.len(),
            6,
            "every admitted parcel must eventually be delivered"
        );
    }
}
