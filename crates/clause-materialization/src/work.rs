//! Total logical work receipts for physical materialization operations.

use std::fmt;
use std::mem;

use crate::model::{OpaqueRef, PhysicalBudget};

/// Exact logical operation counts produced by this crate.
///
/// Reservation bytes are conservative logical weights for records retained or
/// temporarily owned by this crate, including opaque payload bytes. Recursive
/// weights may count inline shells more than once and intentionally exclude
/// allocator metadata and spare collection capacity. They are portable budget
/// units, not a measurement of heap usage. A reservation call is a budget
/// boundary; fallible collection reservations report
/// [`FallbackReason::AllocatorFailure`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorkCounters {
    pub contract_checks: u64,
    pub input_rows_read: u64,
    pub support_records_read: u64,
    pub graph_reads: u64,
    pub index_bucket_probes: u64,
    pub premise_occurrences_visited: u64,
    pub candidate_bindings: u64,
    pub support_entries_read: u64,
    pub support_entries_written: u64,
    pub index_membership_writes: u64,
    pub retained_cost_records_read: u64,
    pub dependency_misses: u64,
    pub whole_state_clones: u64,
    pub whole_view_rebuilds: u64,
    pub support_set_clones: u64,
    pub disconnected_rows_visited: u64,
    pub reservation_calls: u64,
    pub reserved_bytes: u64,
    pub peak_live_bytes: u64,
    pub receipt_bytes: u64,
    pub retained_bytes_before: u64,
    pub retained_bytes_after: u64,
    pub retained_bytes_added: u64,
    pub retained_bytes_released: u64,
    pub index_entries_before: u64,
    pub index_entries_after: u64,
    pub fallback_point_visits: u64,
    pub view_admits: u64,
    pub view_withdraws: u64,
}

impl WorkCounters {
    #[must_use]
    pub fn locality(self) -> LocalityCounters {
        LocalityCounters {
            contract_checks: self.contract_checks,
            input_rows_read: self.input_rows_read,
            support_records_read: self.support_records_read,
            graph_reads: self.graph_reads,
            index_bucket_probes: self.index_bucket_probes,
            premise_occurrences_visited: self.premise_occurrences_visited,
            candidate_bindings: self.candidate_bindings,
            support_entries_read: self.support_entries_read,
            support_entries_written: self.support_entries_written,
            index_membership_writes: self.index_membership_writes,
            retained_cost_records_read: self.retained_cost_records_read,
            dependency_misses: self.dependency_misses,
            whole_state_clones: self.whole_state_clones,
            whole_view_rebuilds: self.whole_view_rebuilds,
            support_set_clones: self.support_set_clones,
            disconnected_rows_visited: self.disconnected_rows_visited,
            reservation_calls: self.reservation_calls,
            reserved_bytes: self.reserved_bytes,
            receipt_bytes: self.receipt_bytes,
            retained_bytes_added: self.retained_bytes_added,
            retained_bytes_released: self.retained_bytes_released,
            fallback_point_visits: self.fallback_point_visits,
            view_admits: self.view_admits,
            view_withdraws: self.view_withdraws,
        }
    }
}

/// The update receipt fields that must remain invariant under disconnected
/// population growth for the same ordinary local update.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LocalityCounters {
    pub contract_checks: u64,
    pub input_rows_read: u64,
    pub support_records_read: u64,
    pub graph_reads: u64,
    pub index_bucket_probes: u64,
    pub premise_occurrences_visited: u64,
    pub candidate_bindings: u64,
    pub support_entries_read: u64,
    pub support_entries_written: u64,
    pub index_membership_writes: u64,
    pub retained_cost_records_read: u64,
    pub dependency_misses: u64,
    pub whole_state_clones: u64,
    pub whole_view_rebuilds: u64,
    pub support_set_clones: u64,
    pub disconnected_rows_visited: u64,
    pub reservation_calls: u64,
    pub reserved_bytes: u64,
    pub receipt_bytes: u64,
    pub retained_bytes_added: u64,
    pub retained_bytes_released: u64,
    pub fallback_point_visits: u64,
    pub view_admits: u64,
    pub view_withdraws: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Schedule {
    ColdScan,
    PartitionScan,
    UniformGrid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackReason {
    BucketLimit,
    TemporaryByteLimit,
    EnvironmentLimit,
    ForcedReservationFailure,
    AllocatorFailure,
    CombinedLiveByteLimit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackReceipt {
    pub anchor_key: Vec<(OpaqueRef, crate::model::OpaqueValue)>,
    pub range_occurrence_refs: Option<Vec<OpaqueRef>>,
    pub reason: FallbackReason,
    pub selected_schedule: Schedule,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureKind {
    Contract(crate::model::ContractError),
    RowLimitExceeded,
    SupportLimitExceeded,
    IndexLimitExceeded,
    TemporaryAllocationExhausted(FallbackReason),
    RetainedAllocationExhausted,
    CombinedLiveAllocationExhausted,
    ReceiptLimitExceeded,
    ReceiptAllocationExhausted(FallbackReason),
    InternalInvariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptOutcome {
    Published,
    Rejected(FailureKind),
}

/// One complete API-entry-to-return physical receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationReceipt {
    pub graph_ref: OpaqueRef,
    pub contract_ref: OpaqueRef,
    pub plan_ref: OpaqueRef,
    pub base_snapshot_ref: Option<OpaqueRef>,
    pub result_snapshot_ref: OpaqueRef,
    pub schedule: Schedule,
    pub fallbacks: Vec<FallbackReceipt>,
    pub counters: WorkCounters,
    pub outcome: ReceiptOutcome,
    incomplete_receipt_bytes: Option<u64>,
}

impl OperationReceipt {
    pub(crate) fn new(
        graph_ref: OpaqueRef,
        contract_ref: OpaqueRef,
        plan_ref: OpaqueRef,
        base_snapshot_ref: Option<OpaqueRef>,
        result_snapshot_ref: OpaqueRef,
        schedule: Schedule,
        counters: WorkCounters,
    ) -> Self {
        Self {
            graph_ref,
            contract_ref,
            plan_ref,
            base_snapshot_ref,
            result_snapshot_ref,
            schedule,
            fallbacks: Vec::new(),
            counters,
            outcome: ReceiptOutcome::Published,
            incomplete_receipt_bytes: None,
        }
    }

    pub(crate) fn reject(mut self, kind: FailureKind) -> OperationError {
        if let Some(attempted_receipt_bytes) = self.incomplete_receipt_bytes {
            return OperationError::without_receipt(kind, attempted_receipt_bytes);
        }
        self.outcome = ReceiptOutcome::Rejected(kind);
        OperationError {
            kind,
            attempted_receipt_bytes: self.counters.receipt_bytes,
            receipt: Some(Box::new(self)),
        }
    }

    pub(crate) fn mark_incomplete(&mut self, attempted_receipt_bytes: u64) {
        self.incomplete_receipt_bytes = Some(attempted_receipt_bytes);
    }

    pub(crate) fn base_retained_bytes_for(
        graph_ref: &OpaqueRef,
        contract_ref: &OpaqueRef,
        plan_ref: &OpaqueRef,
        base_snapshot_ref: Option<&OpaqueRef>,
        result_snapshot_ref: &OpaqueRef,
    ) -> u64 {
        crate::model::usize_to_u64(mem::size_of::<Self>())
            .saturating_add(graph_ref.retained_bytes())
            .saturating_add(contract_ref.retained_bytes())
            .saturating_add(plan_ref.retained_bytes())
            .saturating_add(base_snapshot_ref.map_or(0, OpaqueRef::retained_bytes))
            .saturating_add(result_snapshot_ref.retained_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationError {
    pub kind: FailureKind,
    pub attempted_receipt_bytes: u64,
    pub receipt: Option<Box<OperationReceipt>>,
}

impl OperationError {
    pub(crate) fn without_receipt(kind: FailureKind, attempted_receipt_bytes: u64) -> Self {
        Self {
            kind,
            attempted_receipt_bytes,
            receipt: None,
        }
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "physical materialization failed: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for OperationError {}

pub(crate) struct AllocationTracker {
    maximum_temporary_bytes: u64,
    maximum_retained_bytes: u64,
    maximum_combined_live_bytes: u64,
    fail_reservation_call: Option<u64>,
    maximum_receipt_bytes: u64,
    retained_base_bytes: u64,
    current_temporary_bytes: u64,
    promoted_retained_bytes: u64,
    output_retained_bytes: u64,
}

impl AllocationTracker {
    pub(crate) fn new(budget: PhysicalBudget, retained_base_bytes: u64) -> Self {
        Self {
            maximum_temporary_bytes: budget.maximum_temporary_bytes,
            maximum_retained_bytes: budget.maximum_retained_bytes,
            maximum_combined_live_bytes: budget.maximum_combined_live_bytes,
            fail_reservation_call: budget.fail_reservation_call,
            maximum_receipt_bytes: budget.maximum_receipt_bytes,
            retained_base_bytes,
            current_temporary_bytes: 0,
            promoted_retained_bytes: 0,
            output_retained_bytes: 0,
        }
    }

    pub(crate) fn reserve(
        &mut self,
        bytes: u64,
        counters: &mut WorkCounters,
    ) -> Result<(), FallbackReason> {
        counters.reservation_calls = counters.reservation_calls.saturating_add(1);
        if self.fail_reservation_call == Some(counters.reservation_calls) {
            return Err(FallbackReason::ForcedReservationFailure);
        }
        let next = self
            .current_temporary_bytes
            .checked_add(bytes)
            .ok_or(FallbackReason::TemporaryByteLimit)?;
        if next > self.maximum_temporary_bytes {
            return Err(FallbackReason::TemporaryByteLimit);
        }
        if !self.within_combined_live(
            next,
            self.promoted_retained_bytes,
            self.output_retained_bytes,
        ) {
            return Err(FallbackReason::CombinedLiveByteLimit);
        }
        self.current_temporary_bytes = next;
        counters.reserved_bytes = counters.reserved_bytes.saturating_add(bytes);
        self.record_peak(counters);
        Ok(())
    }

    pub(crate) fn reserve_retained(
        &mut self,
        bytes: u64,
        counters: &mut WorkCounters,
    ) -> Result<(), FailureKind> {
        counters.reservation_calls = counters.reservation_calls.saturating_add(1);
        if self.fail_reservation_call == Some(counters.reservation_calls) {
            return Err(FailureKind::RetainedAllocationExhausted);
        }
        let next_retained = self
            .promoted_retained_bytes
            .checked_add(bytes)
            .ok_or(FailureKind::RetainedAllocationExhausted)?;
        if next_retained > self.maximum_retained_bytes {
            return Err(FailureKind::RetainedAllocationExhausted);
        }
        if !self.within_combined_live(
            self.current_temporary_bytes,
            next_retained,
            self.output_retained_bytes,
        ) {
            return Err(FailureKind::CombinedLiveAllocationExhausted);
        }
        self.promoted_retained_bytes = next_retained;
        counters.reserved_bytes = counters.reserved_bytes.saturating_add(bytes);
        self.record_peak(counters);
        Ok(())
    }

    pub(crate) fn reserve_output(
        &mut self,
        bytes: u64,
        counters: &mut WorkCounters,
    ) -> Result<(), FailureKind> {
        counters.reservation_calls = counters.reservation_calls.saturating_add(1);
        if self.fail_reservation_call == Some(counters.reservation_calls) {
            return Err(FailureKind::ReceiptAllocationExhausted(
                FallbackReason::ForcedReservationFailure,
            ));
        }
        let next_output_bytes = self
            .output_retained_bytes
            .checked_add(bytes)
            .ok_or(FailureKind::ReceiptLimitExceeded)?;
        if next_output_bytes > self.maximum_receipt_bytes {
            return Err(FailureKind::ReceiptLimitExceeded);
        }
        if !self.within_combined_live(
            self.current_temporary_bytes,
            self.promoted_retained_bytes,
            next_output_bytes,
        ) {
            return Err(FailureKind::CombinedLiveAllocationExhausted);
        }
        self.output_retained_bytes = next_output_bytes;
        counters.reserved_bytes = counters.reserved_bytes.saturating_add(bytes);
        counters.receipt_bytes = counters.receipt_bytes.saturating_add(bytes);
        self.record_peak(counters);
        Ok(())
    }

    pub(crate) fn release(&mut self, bytes: u64) -> Result<(), FailureKind> {
        self.current_temporary_bytes = self
            .current_temporary_bytes
            .checked_sub(bytes)
            .ok_or(FailureKind::InternalInvariant)?;
        Ok(())
    }

    pub(crate) fn require_temporary_drained(&self) -> Result<(), FailureKind> {
        if self.current_temporary_bytes == 0 {
            Ok(())
        } else {
            Err(FailureKind::InternalInvariant)
        }
    }

    pub(crate) fn current_temporary_bytes(&self) -> u64 {
        self.current_temporary_bytes
    }

    pub(crate) fn promote(&mut self, bytes: u64) -> Result<(), FailureKind> {
        let next_temporary = self
            .current_temporary_bytes
            .checked_sub(bytes)
            .ok_or(FailureKind::InternalInvariant)?;
        let next_retained = self
            .promoted_retained_bytes
            .checked_add(bytes)
            .ok_or(FailureKind::RetainedAllocationExhausted)?;
        if next_retained > self.maximum_retained_bytes {
            return Err(FailureKind::RetainedAllocationExhausted);
        }
        if !self.within_combined_live(next_temporary, next_retained, self.output_retained_bytes) {
            return Err(FailureKind::CombinedLiveAllocationExhausted);
        }
        self.current_temporary_bytes = next_temporary;
        self.promoted_retained_bytes = next_retained;
        Ok(())
    }

    pub(crate) fn promoted_retained_bytes(&self) -> u64 {
        self.promoted_retained_bytes
    }

    fn record_peak(&self, counters: &mut WorkCounters) {
        counters.peak_live_bytes = counters.peak_live_bytes.max(
            self.retained_base_bytes
                .saturating_add(self.promoted_retained_bytes)
                .saturating_add(self.output_retained_bytes)
                .saturating_add(self.current_temporary_bytes),
        );
    }

    fn within_combined_live(
        &self,
        temporary_bytes: u64,
        promoted_retained_bytes: u64,
        output_retained_bytes: u64,
    ) -> bool {
        self.retained_base_bytes
            .checked_add(promoted_retained_bytes)
            .and_then(|value| value.checked_add(output_retained_bytes))
            .and_then(|value| value.checked_add(temporary_bytes))
            .is_some_and(|live| live <= self.maximum_combined_live_bytes)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RetainedCost {
    pub(crate) entries: u64,
    pub(crate) bytes: u64,
}

impl RetainedCost {
    pub(crate) fn entry(bytes: u64) -> Self {
        Self { entries: 1, bytes }
    }

    pub(crate) fn saturating_add(self, other: Self) -> Self {
        Self {
            entries: self.entries.saturating_add(other.entries),
            bytes: self.bytes.saturating_add(other.bytes),
        }
    }

    pub(crate) fn validate(self, budget: PhysicalBudget) -> Result<(), FailureKind> {
        if self.entries > crate::model::usize_to_u64(budget.maximum_index_entries) {
            return Err(FailureKind::IndexLimitExceeded);
        }
        if self.bytes > budget.maximum_retained_bytes {
            return Err(FailureKind::RetainedAllocationExhausted);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RetainedLedger {
    cost: RetainedCost,
}

impl RetainedLedger {
    pub(crate) fn cost(self) -> RetainedCost {
        self.cost
    }

    pub(crate) fn project(
        self,
        removed: RetainedCost,
        added: RetainedCost,
        budget: PhysicalBudget,
        counters: &mut WorkCounters,
    ) -> Result<Self, FailureKind> {
        let entries = self
            .cost
            .entries
            .checked_sub(removed.entries)
            .and_then(|value| value.checked_add(added.entries))
            .ok_or(FailureKind::InternalInvariant)?;
        let bytes = self
            .cost
            .bytes
            .checked_sub(removed.bytes)
            .and_then(|value| value.checked_add(added.bytes))
            .ok_or(FailureKind::InternalInvariant)?;
        counters.index_entries_before = self.cost.entries;
        counters.index_entries_after = entries;
        counters.retained_bytes_before = self.cost.bytes;
        counters.retained_bytes_after = bytes;
        counters.retained_bytes_added = counters.retained_bytes_added.saturating_add(added.bytes);
        counters.retained_bytes_released = counters
            .retained_bytes_released
            .saturating_add(removed.bytes);
        RetainedCost { entries, bytes }.validate(budget)?;
        Ok(Self {
            cost: RetainedCost { entries, bytes },
        })
    }
}
