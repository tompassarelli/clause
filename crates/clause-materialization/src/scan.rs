//! Cold support scan over one already-admitted snapshot.

use std::collections::BTreeMap;
use std::mem;

use crate::model::{
    AdmittedSnapshot, ContractError, MaterializationContract, OpaqueRef, PhysicalBudget, ScanPlan,
    SupportRecord, usize_to_u64,
};
use crate::support::{MaterializedView, VisibleSupportStore};
use crate::work::{
    AllocationTracker, FailureKind, OperationError, OperationReceipt, RetainedCost, RetainedLedger,
    Schedule, WorkCounters,
};

#[derive(Debug)]
pub struct ScanMaterialization {
    snapshot_ref: OpaqueRef,
    supports: BTreeMap<OpaqueRef, SupportRecord>,
    visible: VisibleSupportStore,
    _retained: RetainedLedger,
    receipt: OperationReceipt,
}

impl ScanMaterialization {
    #[must_use]
    pub fn snapshot_ref(&self) -> &OpaqueRef {
        &self.snapshot_ref
    }

    #[must_use]
    pub fn view(&self) -> MaterializedView<'_> {
        self.visible.view(&self.supports)
    }

    #[must_use]
    pub fn receipt(&self) -> &OperationReceipt {
        &self.receipt
    }
}

/// Materialize every caller-bound support occurrence through a cold scan.
///
/// # Errors
///
/// Returns an [`OperationError`] when a contract, input, identity, index, or
/// physical-budget check fails. The error retains a complete unpublished
/// receipt when that receipt fits its declared ceiling; otherwise it reports
/// the attempted receipt weight without allocating the oversized receipt.
pub fn materialize_scan(
    contract: &MaterializationContract,
    plan: &ScanPlan,
    snapshot: AdmittedSnapshot,
    budget: PhysicalBudget,
) -> Result<ScanMaterialization, OperationError> {
    let mut counters = WorkCounters::default();
    let mut tracker = AllocationTracker::new(budget, 0);
    let base_receipt_bytes = OperationReceipt::base_retained_bytes_for(
        &plan.graph_ref,
        &plan.contract_ref,
        &plan.plan_ref,
        None,
        &snapshot.snapshot_ref,
    );
    if let Err(kind) = tracker.reserve_output(base_receipt_bytes, &mut counters) {
        return Err(OperationError::without_receipt(kind, base_receipt_bytes));
    }
    let mut receipt = OperationReceipt::new(
        plan.graph_ref.clone(),
        plan.contract_ref.clone(),
        plan.plan_ref.clone(),
        None,
        snapshot.snapshot_ref.clone(),
        Schedule::ColdScan,
        counters,
    );
    receipt.counters.contract_checks += 1;
    receipt.counters.graph_reads += 1;
    if let Err(error) = plan.validate(contract) {
        return Err(receipt.reject(FailureKind::Contract(error)));
    }
    receipt.counters.contract_checks += 1;
    receipt.counters.input_rows_read = usize_to_u64(snapshot.rows.len());
    receipt.counters.support_records_read = usize_to_u64(snapshot.supports.len());
    if snapshot.rows.len() > budget.maximum_rows {
        return Err(receipt.reject(FailureKind::RowLimitExceeded));
    }
    if snapshot.supports.len() > budget.maximum_supports {
        return Err(receipt.reject(FailureKind::SupportLimitExceeded));
    }
    if let Err(error) = contract.validate_snapshot(&snapshot) {
        return Err(receipt.reject(FailureKind::Contract(error)));
    }

    let retained_cost = scan_retained_cost(&snapshot.snapshot_ref, &snapshot.supports);
    let retained = match RetainedLedger::default().project(
        RetainedCost::default(),
        retained_cost,
        budget,
        &mut receipt.counters,
    ) {
        Ok(retained) => retained,
        Err(kind) => return Err(receipt.reject(kind)),
    };
    if let Err(kind) = tracker.reserve_retained(retained_cost.bytes, &mut receipt.counters) {
        return Err(receipt.reject(kind));
    }

    let mut supports = BTreeMap::new();
    let mut visible = VisibleSupportStore::default();
    for support in snapshot.supports {
        receipt.counters.support_entries_read += 1;
        receipt.counters.premise_occurrences_visited = receipt
            .counters
            .premise_occurrences_visited
            .saturating_add(usize_to_u64(support.premise_occurrence_refs.len()));
        visible.attach_scan(&support, &mut receipt.counters);
        supports.insert(support.support_occurrence_ref.clone(), support);
    }
    let snapshot_ref = snapshot.snapshot_ref;
    if let Err(kind) = tracker.require_temporary_drained() {
        return Err(receipt.reject(kind));
    }
    Ok(ScanMaterialization {
        snapshot_ref,
        supports,
        visible,
        _retained: retained,
        receipt,
    })
}

fn scan_retained_cost(snapshot_ref: &OpaqueRef, supports: &[SupportRecord]) -> RetainedCost {
    supports.iter().fold(
        RetainedCost::entry(snapshot_ref.retained_bytes()),
        |cost, support| {
            let support_ref = &support.support_occurrence_ref;
            let mut support_cost = RetainedCost::entry(
                support_ref
                    .retained_bytes()
                    .saturating_add(support.retained_bytes()),
            )
            .saturating_add(RetainedCost::entry(support_ref.retained_bytes()))
            .saturating_add(RetainedCost::entry(
                support
                    .output
                    .retained_bytes()
                    .saturating_add(support_ref.retained_bytes()),
            ));
            for (index, premise) in support.premise_occurrence_refs.iter().enumerate() {
                if support.premise_occurrence_refs[..index].contains(premise) {
                    continue;
                }
                support_cost = support_cost.saturating_add(RetainedCost::entry(
                    premise
                        .retained_bytes()
                        .saturating_add(support_ref.retained_bytes())
                        .saturating_add(usize_to_u64(mem::size_of::<u64>())),
                ));
            }
            cost.saturating_add(support_cost)
        },
    )
}

impl From<ContractError> for FailureKind {
    fn from(value: ContractError) -> Self {
        Self::Contract(value)
    }
}
