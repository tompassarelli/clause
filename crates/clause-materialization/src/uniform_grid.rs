//! Incremental construct-blind dual-uniform-grid projection.

use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use crate::model::{
    AdmittedSnapshot, AdmittedStateBinding, ContractError, GridPlan, InputRow,
    MaterializationContract, MaterializationUpdate, OpaqueRef, OpaqueValue, PhysicalBudget,
    SupportRecord, usize_to_u64,
};
use crate::support::{AnchorKey, MaterializedView, ReverseIndexSizes, VisibleSupportStore};
use crate::work::{
    AllocationTracker, FailureKind, FallbackReason, FallbackReceipt, OperationError,
    OperationReceipt, RetainedCost, RetainedLedger, Schedule, WorkCounters,
};

#[derive(Debug)]
pub struct UniformGridMaterializer {
    contract: MaterializationContract,
    plan: GridPlan,
    admitted_state: AdmittedStateBinding,
    snapshot_ref: OpaqueRef,
    point_input_ref: OpaqueRef,
    rows: BTreeMap<OpaqueRef, InputRow>,
    points: BTreeMap<OpaqueRef, PointRecord>,
    point_buckets: BTreeMap<BucketKey, BTreeSet<OpaqueRef>>,
    points_by_partition: BTreeMap<OpaqueValue, BTreeSet<OpaqueRef>>,
    range_slot_rows: BTreeMap<OpaqueRef, BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>>,
    anchors: BTreeMap<AnchorKey, AnchorRecord>,
    range_buckets: BTreeMap<BucketKey, BTreeSet<AnchorKey>>,
    partition_fallback_ranges: BTreeMap<OpaqueValue, BTreeSet<AnchorKey>>,
    supports: BTreeMap<OpaqueRef, SupportRecord>,
    supports_by_tuple: BTreeMap<Vec<OpaqueRef>, BTreeSet<OpaqueRef>>,
    supports_by_premise: BTreeMap<OpaqueRef, BTreeMap<OpaqueRef, u64>>,
    support_locations: BTreeMap<OpaqueRef, SupportLocation>,
    supports_by_anchor: BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>,
    supports_by_range: BTreeMap<RangeKey, BTreeSet<OpaqueRef>>,
    visible: VisibleSupportStore,
    retained: RetainedLedger,
}

impl UniformGridMaterializer {
    /// Build a fresh replaceable view from one caller-admitted snapshot.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationError`] when contract, identity, input, index, or
    /// physical-budget validation fails. The error retains an unpublished
    /// receipt only when the complete receipt fits its declared ceiling.
    pub fn build(
        contract: &MaterializationContract,
        plan: &GridPlan,
        snapshot: AdmittedSnapshot,
        budget: PhysicalBudget,
    ) -> Result<(Self, OperationReceipt), OperationError> {
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
            Schedule::UniformGrid,
            counters,
        );
        if let Err(kind) =
            Self::validate_build_input(contract, plan, &snapshot, budget, &mut receipt)
        {
            return Err(receipt.reject(kind));
        }

        let Some(point_input_ref) = contract
            .slot(&plan.point_slot_ref)
            .map(|slot| &slot.input_ref)
        else {
            return Err(receipt.reject(FailureKind::InternalInvariant));
        };
        let AdmittedSnapshot {
            admitted_state,
            admitted_contract: _,
            snapshot_ref,
            rows,
            supports,
        } = snapshot;
        let mut preflight_cost = Self::core_retained_cost(
            contract,
            plan,
            &admitted_state,
            &snapshot_ref,
            point_input_ref,
        );
        if let Err(kind) = preflight_cost.validate(budget) {
            return Err(receipt.reject(kind));
        }
        if let Err(kind) = tracker.reserve_retained(preflight_cost.bytes, &mut receipt.counters) {
            return Err(receipt.reject(kind));
        }
        let mut materializer = Self::unindexed(
            contract,
            plan,
            admitted_state,
            snapshot_ref,
            point_input_ref.clone(),
        );

        if let Err(kind) = materializer.index_initial_payload(
            rows,
            supports,
            budget,
            &mut preflight_cost,
            &mut tracker,
            &mut receipt,
        ) {
            return Err(receipt.reject(kind));
        }

        if let Err(kind) = materializer.prepare_initial_anchors(
            budget,
            &mut preflight_cost,
            &mut tracker,
            &mut receipt,
        ) {
            return Err(receipt.reject(kind));
        }
        if let Err(kind) =
            materializer.finish_initial_build(preflight_cost, budget, &tracker, &mut receipt)
        {
            return Err(receipt.reject(kind));
        }
        Ok((materializer, receipt))
    }

    fn core_retained_cost(
        contract: &MaterializationContract,
        plan: &GridPlan,
        admitted_state: &AdmittedStateBinding,
        snapshot_ref: &OpaqueRef,
        point_input_ref: &OpaqueRef,
    ) -> RetainedCost {
        RetainedCost::entry(contract.retained_bytes())
            .saturating_add(RetainedCost::entry(plan.retained_bytes()))
            .saturating_add(RetainedCost::entry(admitted_state.retained_bytes()))
            .saturating_add(RetainedCost::entry(snapshot_ref.retained_bytes()))
            .saturating_add(RetainedCost::entry(point_input_ref.retained_bytes()))
    }

    fn validate_build_input(
        contract: &MaterializationContract,
        plan: &GridPlan,
        snapshot: &AdmittedSnapshot,
        budget: PhysicalBudget,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        receipt.counters.contract_checks += 1;
        receipt.counters.graph_reads += 1;
        plan.validate(contract).map_err(FailureKind::Contract)?;
        receipt.counters.contract_checks += 1;
        receipt.counters.input_rows_read = usize_to_u64(snapshot.rows.len());
        receipt.counters.support_records_read = usize_to_u64(snapshot.supports.len());
        if snapshot.rows.len() > budget.maximum_rows {
            return Err(FailureKind::RowLimitExceeded);
        }
        if snapshot.supports.len() > budget.maximum_supports {
            return Err(FailureKind::SupportLimitExceeded);
        }
        contract
            .validate_snapshot(snapshot)
            .map_err(FailureKind::Contract)
    }

    #[allow(clippy::too_many_arguments)]
    fn index_initial_payload(
        &mut self,
        rows: Vec<InputRow>,
        supports: Vec<SupportRecord>,
        budget: PhysicalBudget,
        preflight_cost: &mut RetainedCost,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        for row in rows {
            let row_cost = self.row_retained_cost(&row)?;
            let next_cost = preflight_cost.saturating_add(row_cost);
            next_cost.validate(budget)?;
            tracker.reserve_retained(row_cost.bytes, &mut receipt.counters)?;
            if self.contract.is_dependency(&row.input_ref) {
                self.index_row(&row, &mut receipt.counters)
                    .map_err(FailureKind::Contract)?;
            } else {
                receipt.counters.dependency_misses += 1;
            }
            self.rows.insert(row.occurrence_ref.clone(), row);
            *preflight_cost = next_cost;
        }
        for support in supports {
            let location_shape = self
                .support_location_shape_current(&support)
                .map_err(FailureKind::Contract)?;
            let support_cost = Self::support_retained_cost_for_shape(&support, location_shape);
            let next_cost = preflight_cost.saturating_add(support_cost);
            next_cost.validate(budget)?;
            tracker.reserve_retained(support_cost.bytes, &mut receipt.counters)?;
            let location = self
                .support_location_current(&support)
                .map_err(FailureKind::Contract)?;
            self.index_support_record_at(support, location, &mut receipt.counters);
            *preflight_cost = next_cost;
        }
        Ok(())
    }

    fn prepare_initial_anchors(
        &mut self,
        budget: PhysicalBudget,
        preflight_cost: &mut RetainedCost,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        let mut anchors = BTreeSet::new();
        for anchor in self.range_slot_rows.values().flat_map(BTreeMap::keys) {
            if !anchors.contains(anchor) {
                tracker
                    .reserve(anchor_retained_bytes(anchor), &mut receipt.counters)
                    .map_err(FailureKind::TemporaryAllocationExhausted)?;
                anchors.insert(anchor.clone());
            }
        }
        for anchor in anchors {
            let promoted_before = tracker.promoted_retained_bytes();
            match self.prepare_anchor_current(&anchor, budget, tracker, receipt)? {
                Some(prepared) => {
                    tracker.promote(anchor_retained_bytes(&anchor))?;
                    let anchor_cost = self.prepared_anchor_retained_cost(
                        &anchor,
                        &prepared,
                        &mut receipt.counters,
                    )?;
                    let next_cost = preflight_cost.saturating_add(anchor_cost);
                    next_cost.validate(budget)?;
                    let promoted_for_anchor = tracker
                        .promoted_retained_bytes()
                        .checked_sub(promoted_before)
                        .ok_or(FailureKind::InternalInvariant)?;
                    let missing_retained_bytes = anchor_cost
                        .bytes
                        .checked_sub(promoted_for_anchor)
                        .ok_or(FailureKind::InternalInvariant)?;
                    tracker.reserve_retained(missing_retained_bytes, &mut receipt.counters)?;
                    self.install_prepared_anchor(anchor, prepared, &mut receipt.counters);
                    *preflight_cost = next_cost;
                }
                None => tracker.release(anchor_retained_bytes(&anchor))?,
            }
        }
        Ok(())
    }

    fn finish_initial_build(
        &mut self,
        preflight_cost: RetainedCost,
        budget: PhysicalBudget,
        tracker: &AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        tracker.require_temporary_drained()?;
        if !self.visible.exactly_matches_records(&self.supports) {
            return Err(FailureKind::Contract(
                ContractError::SupportOutsidePhysicalSuperset,
            ));
        }
        let retained_cost = self.retained_cost(&mut receipt.counters)?;
        if retained_cost != preflight_cost
            || retained_cost.bytes != tracker.promoted_retained_bytes()
        {
            return Err(FailureKind::InternalInvariant);
        }
        self.retained = RetainedLedger::default().project(
            RetainedCost::default(),
            retained_cost,
            budget,
            &mut receipt.counters,
        )?;
        Ok(())
    }

    fn unindexed(
        contract: &MaterializationContract,
        plan: &GridPlan,
        admitted_state: AdmittedStateBinding,
        snapshot_ref: OpaqueRef,
        point_input_ref: OpaqueRef,
    ) -> Self {
        Self {
            contract: contract.clone(),
            plan: plan.clone(),
            admitted_state,
            snapshot_ref,
            point_input_ref,
            rows: BTreeMap::new(),
            points: BTreeMap::new(),
            point_buckets: BTreeMap::new(),
            points_by_partition: BTreeMap::new(),
            range_slot_rows: BTreeMap::new(),
            anchors: BTreeMap::new(),
            range_buckets: BTreeMap::new(),
            partition_fallback_ranges: BTreeMap::new(),
            supports: BTreeMap::new(),
            supports_by_tuple: BTreeMap::new(),
            supports_by_premise: BTreeMap::new(),
            support_locations: BTreeMap::new(),
            supports_by_anchor: BTreeMap::new(),
            supports_by_range: BTreeMap::new(),
            visible: VisibleSupportStore::default(),
            retained: RetainedLedger::default(),
        }
    }

    #[must_use]
    pub fn snapshot_ref(&self) -> &OpaqueRef {
        &self.snapshot_ref
    }

    #[must_use]
    pub fn graph_ref(&self) -> &OpaqueRef {
        &self.contract.graph_ref
    }

    #[must_use]
    pub fn contract_ref(&self) -> &OpaqueRef {
        &self.contract.contract_ref
    }

    #[must_use]
    pub fn plan_ref(&self) -> &OpaqueRef {
        &self.plan.plan_ref
    }

    #[must_use]
    pub fn view(&self) -> MaterializedView<'_> {
        self.visible.view(&self.supports)
    }

    #[must_use]
    pub fn reverse_index_sizes(&self) -> ReverseIndexSizes {
        self.visible.sizes()
    }

    /// Apply one exact-base caller-admitted delta without creating semantic
    /// history. Every modeled `Result` failure completes before the previous
    /// physical view is changed. Host allocator abort/OOM is outside this typed
    /// failure contract because standard B-tree mutation is not fallible.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationError`] when any exact identity, contract, input,
    /// index, or physical-budget check fails. The error retains an unpublished
    /// receipt only when the complete receipt fits its declared ceiling.
    pub fn advance(
        &mut self,
        update: MaterializationUpdate,
    ) -> Result<OperationReceipt, OperationError> {
        let mut counters = WorkCounters::default();
        let mut tracker = AllocationTracker::new(update.budget, self.retained.cost().bytes);
        let base_receipt_bytes = OperationReceipt::base_retained_bytes_for(
            &update.graph_ref,
            &update.contract_ref,
            &update.plan_ref,
            Some(&update.base_snapshot_ref),
            &update.result_snapshot_ref,
        );
        if let Err(kind) = tracker.reserve_output(base_receipt_bytes, &mut counters) {
            return Err(OperationError::without_receipt(kind, base_receipt_bytes));
        }
        let mut receipt = OperationReceipt::new(
            update.graph_ref.clone(),
            update.contract_ref.clone(),
            update.plan_ref.clone(),
            Some(update.base_snapshot_ref.clone()),
            update.result_snapshot_ref.clone(),
            Schedule::UniformGrid,
            counters,
        );
        match self.prepare_update(&update, &mut tracker, &mut receipt) {
            Ok(prepared) => {
                if let Err(kind) = tracker
                    .release(prepared.temporary_bytes)
                    .and_then(|()| tracker.require_temporary_drained())
                {
                    return Err(receipt.reject(kind));
                }
                self.commit_update(update, prepared, &mut receipt.counters);
                Ok(receipt)
            }
            Err(kind) => Err(receipt.reject(kind)),
        }
    }

    fn point_input_ref(&self) -> &OpaqueRef {
        &self.point_input_ref
    }

    fn range_slots_for_input<'a>(
        &'a self,
        input_ref: &'a OpaqueRef,
    ) -> impl Iterator<Item = &'a OpaqueRef> + 'a {
        self.plan.range_slot_refs.iter().filter(move |slot_ref| {
            self.contract
                .slot(slot_ref)
                .is_some_and(|slot| slot.input_ref == *input_ref)
        })
    }

    fn index_row(
        &mut self,
        row: &InputRow,
        counters: &mut WorkCounters,
    ) -> Result<(), ContractError> {
        if row.input_ref == *self.point_input_ref() {
            let point = self.point_record(row)?;
            self.point_buckets
                .entry(point.bucket.clone())
                .or_default()
                .insert(row.occurrence_ref.clone());
            self.points_by_partition
                .entry(point.partition.clone())
                .or_default()
                .insert(row.occurrence_ref.clone());
            self.points.insert(row.occurrence_ref.clone(), point);
            counters.index_membership_writes += 3;
        }
        if self.range_slots_for_input(&row.input_ref).next().is_some() {
            let anchor = self.anchor_for_row(row)?;
            let contract = &self.contract;
            let range_slot_refs = &self.plan.range_slot_refs;
            let range_slot_rows = &mut self.range_slot_rows;
            for slot_ref in range_slot_refs {
                if contract
                    .slot(slot_ref)
                    .is_none_or(|slot| slot.input_ref != row.input_ref)
                {
                    continue;
                }
                range_slot_rows
                    .entry(slot_ref.clone())
                    .or_default()
                    .entry(anchor.clone())
                    .or_default()
                    .insert(row.occurrence_ref.clone());
                counters.index_membership_writes += 1;
            }
        }
        Ok(())
    }

    fn unindex_row(&mut self, row: &InputRow, counters: &mut WorkCounters) {
        if row.input_ref == *self.point_input_ref()
            && let Some(point) = self.points.remove(&row.occurrence_ref)
        {
            remove_set_member(&mut self.point_buckets, &point.bucket, &row.occurrence_ref);
            remove_set_member(
                &mut self.points_by_partition,
                &point.partition,
                &row.occurrence_ref,
            );
            counters.index_membership_writes += 3;
        }
        if self.range_slots_for_input(&row.input_ref).next().is_none() {
            return;
        }
        let Ok(anchor) = self.anchor_for_row(row) else {
            return;
        };
        let contract = &self.contract;
        let range_slot_refs = &self.plan.range_slot_refs;
        let range_slot_rows = &mut self.range_slot_rows;
        for slot_ref in range_slot_refs {
            if contract
                .slot(slot_ref)
                .is_none_or(|slot| slot.input_ref != row.input_ref)
            {
                continue;
            }
            let remove_slot = range_slot_rows.get_mut(slot_ref).is_some_and(|by_anchor| {
                let remove_anchor = by_anchor.get_mut(&anchor).is_some_and(|occurrences| {
                    occurrences.remove(&row.occurrence_ref);
                    occurrences.is_empty()
                });
                if remove_anchor {
                    by_anchor.remove(&anchor);
                }
                by_anchor.is_empty()
            });
            if remove_slot {
                range_slot_rows.remove(slot_ref);
            }
            counters.index_membership_writes += 1;
        }
    }

    fn point_record(&self, row: &InputRow) -> Result<PointRecord, ContractError> {
        let partition = row
            .binding(&self.plan.partition_binding_ref)
            .cloned()
            .ok_or(ContractError::MissingBinding)?;
        let x = i64::from(self.plan.point_x.read(row)?);
        let y = i64::from(self.plan.point_y.read(row)?);
        Ok(PointRecord {
            partition: partition.clone(),
            bucket: BucketKey {
                partition,
                x: bucket_coordinate(x, self.plan.bucket_width),
                y: bucket_coordinate(y, self.plan.bucket_width),
            },
        })
    }

    fn anchor_for_row(&self, row: &InputRow) -> Result<AnchorKey, ContractError> {
        self.plan
            .anchor_binding_refs
            .iter()
            .map(|binding_ref| {
                row.binding(binding_ref)
                    .cloned()
                    .map(|value| (binding_ref.clone(), value))
                    .ok_or(ContractError::MissingBinding)
            })
            .collect()
    }

    fn index_support_record_at(
        &mut self,
        support: SupportRecord,
        location: SupportLocation,
        counters: &mut WorkCounters,
    ) {
        let writes = 5_u64.saturating_add(usize_to_u64(support.premise_occurrence_refs.len()));
        let support_ref = support.support_occurrence_ref.clone();
        self.supports_by_tuple
            .entry(support.premise_occurrence_refs.clone())
            .or_default()
            .insert(support_ref.clone());
        for premise in &support.premise_occurrence_refs {
            *self
                .supports_by_premise
                .entry(premise.clone())
                .or_default()
                .entry(support_ref.clone())
                .or_default() += 1;
        }
        self.supports_by_anchor
            .entry(location.anchor.clone())
            .or_default()
            .insert(support_ref.clone());
        self.supports_by_range
            .entry(location.range.clone())
            .or_default()
            .insert(support_ref.clone());
        self.support_locations.insert(support_ref.clone(), location);
        self.supports.insert(support_ref, support);
        counters.index_membership_writes = counters.index_membership_writes.saturating_add(writes);
    }

    fn unindex_support_record(&mut self, support_ref: &OpaqueRef, counters: &mut WorkCounters) {
        let Some(support) = self.supports.remove(support_ref) else {
            return;
        };
        let writes = 5_u64.saturating_add(usize_to_u64(support.premise_occurrence_refs.len()));
        let remove_tuple = self
            .supports_by_tuple
            .get_mut(&support.premise_occurrence_refs)
            .is_some_and(|supports| {
                supports.remove(support_ref);
                supports.is_empty()
            });
        if remove_tuple {
            self.supports_by_tuple
                .remove(&support.premise_occurrence_refs);
        }
        for premise in &support.premise_occurrence_refs {
            let remove_premise =
                self.supports_by_premise
                    .get_mut(premise)
                    .is_some_and(|supports| {
                        if let Some(count) = supports.get_mut(support_ref) {
                            *count -= 1;
                            if *count == 0 {
                                supports.remove(support_ref);
                            }
                        }
                        supports.is_empty()
                    });
            if remove_premise {
                self.supports_by_premise.remove(premise);
            }
        }
        if let Some(location) = self.support_locations.remove(support_ref) {
            remove_set_member(&mut self.supports_by_anchor, &location.anchor, support_ref);
            remove_set_member(&mut self.supports_by_range, &location.range, support_ref);
        }
        counters.index_membership_writes = counters.index_membership_writes.saturating_add(writes);
    }

    fn support_location_current(
        &self,
        support: &SupportRecord,
    ) -> Result<SupportLocation, ContractError> {
        self.support_location_with(support, |occurrence| self.rows.get(occurrence))
    }

    fn support_location_shape_current(
        &self,
        support: &SupportRecord,
    ) -> Result<SupportLocationShape, ContractError> {
        self.support_location_shape_with(support, |occurrence| self.rows.get(occurrence))
    }

    fn support_location_shape_with<'a>(
        &'a self,
        support: &SupportRecord,
        mut row_for: impl FnMut(&OpaqueRef) -> Option<&'a InputRow>,
    ) -> Result<SupportLocationShape, ContractError> {
        let mut anchor_row = None;
        let mut range_bytes = usize_to_u64(mem::size_of::<RangeKey>());
        for slot_ref in &self.plan.range_slot_refs {
            let position = self
                .contract
                .premise_slots
                .iter()
                .position(|slot| slot.slot_ref == *slot_ref)
                .ok_or(ContractError::PremiseSlotMismatch)?;
            let occurrence = support
                .premise_occurrence_refs
                .get(position)
                .ok_or(ContractError::PremiseSlotMismatch)?;
            let row = row_for(occurrence).ok_or(ContractError::MissingPremiseOccurrence)?;
            if let Some(expected) = anchor_row
                && !self.rows_share_anchor(expected, row)?
            {
                return Err(ContractError::InconsistentAnchorBindings);
            }
            anchor_row = Some(row);
            range_bytes = range_bytes.saturating_add(occurrence.retained_bytes());
        }
        let anchor_row = anchor_row.ok_or(ContractError::PremiseSlotMismatch)?;
        Ok(SupportLocationShape {
            anchor_bytes: self.anchor_retained_bytes_for_row(anchor_row)?,
            range_bytes,
        })
    }

    fn rows_share_anchor(&self, left: &InputRow, right: &InputRow) -> Result<bool, ContractError> {
        for binding_ref in &self.plan.anchor_binding_refs {
            let left_value = left
                .binding(binding_ref)
                .ok_or(ContractError::MissingBinding)?;
            let right_value = right
                .binding(binding_ref)
                .ok_or(ContractError::MissingBinding)?;
            if left_value != right_value {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn support_location_with<'a>(
        &'a self,
        support: &SupportRecord,
        mut row_for: impl FnMut(&OpaqueRef) -> Option<&'a InputRow>,
    ) -> Result<SupportLocation, ContractError> {
        let mut anchor = None;
        let mut range = Vec::with_capacity(self.plan.range_slot_refs.len());
        for slot_ref in &self.plan.range_slot_refs {
            let position = self
                .contract
                .premise_slots
                .iter()
                .position(|slot| slot.slot_ref == *slot_ref)
                .ok_or(ContractError::PremiseSlotMismatch)?;
            let occurrence = support
                .premise_occurrence_refs
                .get(position)
                .ok_or(ContractError::PremiseSlotMismatch)?;
            let row = row_for(occurrence).ok_or(ContractError::MissingPremiseOccurrence)?;
            let row_anchor = self.anchor_for_row(row)?;
            if anchor
                .as_ref()
                .is_some_and(|expected| expected != &row_anchor)
            {
                return Err(ContractError::InconsistentAnchorBindings);
            }
            anchor = Some(row_anchor);
            range.push(occurrence.clone());
        }
        Ok(SupportLocation {
            anchor: anchor.ok_or(ContractError::PremiseSlotMismatch)?,
            range,
        })
    }

    fn prepare_anchor_current(
        &self,
        anchor: &AnchorKey,
        budget: PhysicalBudget,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<Option<PreparedAnchor>, FailureKind> {
        match self.join_environments_current(anchor, budget, tracker, receipt)? {
            RangePreparation::Absent => Ok(None),
            RangePreparation::Ready(batch) => {
                let PromotedEnvironments {
                    environments,
                    vector_bytes,
                } = batch.into_promoted(tracker)?;
                let mut ranges = BTreeMap::new();
                let mut visible_supports = BTreeSet::new();
                for environment in environments {
                    let range = self.range_record(anchor, environment, budget, tracker, receipt)?;
                    self.visible_supports_current(&range, &mut visible_supports, tracker, receipt)?;
                    let range_key_bytes = range_key_retained_bytes(&range.key);
                    tracker
                        .reserve(range_key_bytes, &mut receipt.counters)
                        .map_err(FailureKind::TemporaryAllocationExhausted)?;
                    if ranges.insert(range.key.clone(), range).is_some() {
                        return Err(FailureKind::InternalInvariant);
                    }
                    tracker.promote(range_key_bytes)?;
                }
                tracker.release(vector_bytes)?;
                self.require_exact_supports_current(anchor, &visible_supports, receipt)?;
                Ok(Some(Self::prepared_anchor(
                    ranges,
                    visible_supports,
                    tracker,
                    receipt,
                )?))
            }
        }
    }

    fn join_environments_current(
        &self,
        anchor: &AnchorKey,
        budget: PhysicalBudget,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<RangePreparation, FailureKind> {
        let mut batch = EnvironmentBatch::initial(tracker, &mut receipt.counters)?;
        for slot_ref in &self.plan.range_slot_refs {
            let Some(rows) = self
                .range_slot_rows
                .get(slot_ref)
                .and_then(|by_anchor| by_anchor.get(anchor))
            else {
                tracker.release(batch.temporary_bytes)?;
                return Ok(RangePreparation::Absent);
            };
            let occurrences = rows.iter();
            let joined = Self::join_environment_slot(
                &batch,
                slot_ref,
                &occurrences,
                budget,
                tracker,
                receipt,
                |occurrence| self.rows.get(occurrence),
            );
            tracker.release(batch.temporary_bytes)?;
            match joined {
                Ok(next) if next.environments.is_empty() => {
                    tracker.release(next.temporary_bytes)?;
                    return Ok(RangePreparation::Absent);
                }
                Ok(next) => batch = next,
                Err(kind) => return Err(kind),
            }
        }
        Ok(RangePreparation::Ready(batch))
    }

    #[allow(clippy::too_many_arguments)]
    fn join_environment_slot<'a, I, F>(
        batch: &EnvironmentBatch,
        slot_ref: &OpaqueRef,
        occurrences: &I,
        budget: PhysicalBudget,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
        mut row_for: F,
    ) -> Result<EnvironmentBatch, FailureKind>
    where
        I: ExactSizeIterator<Item = &'a OpaqueRef> + Clone,
        F: FnMut(&OpaqueRef) -> Option<&'a InputRow>,
    {
        let Some(capacity) = batch.environments.len().checked_mul(occurrences.len()) else {
            return Err(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::EnvironmentLimit,
            ));
        };
        if capacity > budget.maximum_environments_per_anchor {
            return Err(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::EnvironmentLimit,
            ));
        }
        let vector_bytes =
            usize_to_u64(capacity).saturating_mul(usize_to_u64(mem::size_of::<Environment>()));
        tracker
            .reserve(vector_bytes, &mut receipt.counters)
            .map_err(FailureKind::TemporaryAllocationExhausted)?;
        let mut joined = Vec::new();
        if joined.try_reserve_exact(capacity).is_err() {
            tracker.release(vector_bytes)?;
            return Err(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::AllocatorFailure,
            ));
        }
        let mut joined_bytes = vector_bytes;
        for environment in &batch.environments {
            for occurrence in (*occurrences).clone() {
                receipt.counters.input_rows_read += 1;
                let Some(row) = row_for(occurrence) else {
                    tracker.release(joined_bytes)?;
                    return Err(FailureKind::InternalInvariant);
                };
                receipt.counters.candidate_bindings += 1;
                if !environment.can_merge(row) {
                    continue;
                }
                let deep_bytes = environment.merged_deep_bytes(slot_ref, row);
                if let Err(reason) = tracker.reserve(deep_bytes, &mut receipt.counters) {
                    tracker.release(joined_bytes)?;
                    return Err(FailureKind::TemporaryAllocationExhausted(reason));
                }
                joined_bytes = joined_bytes.saturating_add(deep_bytes);
                joined.push(environment.merge_unchecked(slot_ref, row));
            }
        }
        Ok(EnvironmentBatch {
            environments: joined,
            temporary_bytes: joined_bytes,
        })
    }

    fn require_exact_supports_current(
        &self,
        anchor: &AnchorKey,
        visible: &BTreeSet<OpaqueRef>,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        let mut expected_count = 0_usize;
        if let Some(expected) = self.supports_by_anchor.get(anchor) {
            for support in expected {
                receipt.counters.support_entries_read += 1;
                expected_count += 1;
                if !visible.contains(support) {
                    return Err(FailureKind::Contract(
                        ContractError::SupportOutsidePhysicalSuperset,
                    ));
                }
            }
        }
        if visible.len() != expected_count {
            return Err(FailureKind::Contract(
                ContractError::SupportOutsidePhysicalSuperset,
            ));
        }
        Ok(())
    }

    fn prepared_anchor(
        ranges: BTreeMap<RangeKey, RangeRecord>,
        visible_supports: BTreeSet<OpaqueRef>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<PreparedAnchor, FailureKind> {
        let mut indexed_buckets = BTreeSet::new();
        let mut partition_scans = BTreeSet::new();
        for range in ranges.values() {
            match &range.coverage {
                Coverage::Buckets(buckets) => {
                    for bucket in buckets {
                        if !indexed_buckets.contains(bucket) {
                            let bytes = bucket.retained_bytes();
                            tracker
                                .reserve(bytes, &mut receipt.counters)
                                .map_err(FailureKind::TemporaryAllocationExhausted)?;
                            indexed_buckets.insert(bucket.clone());
                            tracker.promote(bytes)?;
                        }
                    }
                }
                Coverage::PartitionScan => {
                    if !partition_scans.contains(&range.geometry.partition) {
                        let bytes = range.geometry.partition.retained_bytes();
                        tracker
                            .reserve(bytes, &mut receipt.counters)
                            .map_err(FailureKind::TemporaryAllocationExhausted)?;
                        partition_scans.insert(range.geometry.partition.clone());
                        tracker.promote(bytes)?;
                    }
                }
            }
        }
        let visible_support_bytes = visible_supports.iter().fold(0_u64, |total, support_ref| {
            total.saturating_add(support_ref.retained_bytes())
        });
        tracker.promote(visible_support_bytes)?;
        Ok(PreparedAnchor {
            anchor_record: AnchorRecord {
                ranges,
                indexed_buckets,
                partition_scans,
            },
            visible_supports,
        })
    }

    fn range_record(
        &self,
        anchor: &AnchorKey,
        environment: Environment,
        budget: PhysicalBudget,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<RangeRecord, FailureKind> {
        let range_key_bytes = self.plan.range_slot_refs.iter().try_fold(
            usize_to_u64(mem::size_of::<RangeKey>()),
            |bytes, slot_ref| {
                let occurrence = environment
                    .occurrences
                    .get(slot_ref)
                    .ok_or(FailureKind::InternalInvariant)?;
                Ok::<u64, FailureKind>(bytes.saturating_add(occurrence.retained_bytes()))
            },
        )?;
        let partition_bytes = environment
            .binding(&self.plan.partition_binding_ref)
            .ok_or(FailureKind::Contract(ContractError::MissingBinding))?
            .retained_bytes();
        let shell_bytes = usize_to_u64(mem::size_of::<RangeRecord>())
            .saturating_add(range_key_bytes)
            .saturating_add(usize_to_u64(mem::size_of::<Environment>()))
            .saturating_add(usize_to_u64(mem::size_of::<RangeGeometry>()))
            .saturating_add(partition_bytes)
            .saturating_add(usize_to_u64(mem::size_of::<Coverage>()));
        tracker
            .reserve(shell_bytes, &mut receipt.counters)
            .map_err(FailureKind::TemporaryAllocationExhausted)?;
        let key = environment.range_key(&self.plan)?;
        let partition = environment
            .binding(&self.plan.partition_binding_ref)
            .cloned()
            .ok_or(FailureKind::Contract(ContractError::MissingBinding))?;
        let center_x = Self::decode_environment(&self.plan.center_x, &environment)?;
        let center_y = Self::decode_environment(&self.plan.center_y, &environment)?;
        let extent = Self::decode_environment(&self.plan.extent, &environment)?;
        if extent < 0 {
            return Err(FailureKind::Contract(ContractError::NegativeExtent));
        }
        let extent = i64::from(extent);
        let minimum_x = bucket_coordinate(i64::from(center_x) - extent, self.plan.bucket_width);
        let maximum_x = bucket_coordinate(i64::from(center_x) + extent, self.plan.bucket_width);
        let minimum_y = bucket_coordinate(i64::from(center_y) - extent, self.plan.bucket_width);
        let maximum_y = bucket_coordinate(i64::from(center_y) + extent, self.plan.bucket_width);
        let envelope = BucketEnvelope {
            minimum_x,
            maximum_x,
            minimum_y,
            maximum_y,
        };
        let coverage = self.range_coverage(
            RangeCoverageInput {
                anchor,
                key: &key,
                partition: &partition,
                envelope,
                budget,
            },
            tracker,
            receipt,
        )?;
        tracker.promote(shell_bytes)?;
        Ok(RangeRecord {
            key,
            environment,
            geometry: RangeGeometry {
                partition,
                minimum_bucket_x: minimum_x,
                maximum_bucket_x: maximum_x,
                minimum_bucket_y: minimum_y,
                maximum_bucket_y: maximum_y,
            },
            coverage,
        })
    }

    fn range_coverage(
        &self,
        input: RangeCoverageInput<'_>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<Coverage, FailureKind> {
        let RangeCoverageInput {
            anchor,
            key,
            partition,
            envelope,
            budget,
        } = input;
        let bucket_count = envelope.bucket_count()?;
        let bucket_limit = self
            .plan
            .maximum_buckets_per_range
            .min(budget.maximum_buckets_per_range);
        if bucket_count > u128::from(bucket_limit) {
            Self::record_fallback(
                anchor,
                Some(key),
                FallbackReason::BucketLimit,
                Schedule::PartitionScan,
                tracker,
                receipt,
            )?;
            return Ok(Coverage::PartitionScan);
        }
        let capacity = usize::try_from(bucket_count)
            .map_err(|_| FailureKind::TemporaryAllocationExhausted(FallbackReason::BucketLimit))?;
        let allocation_bytes = usize_to_u64(capacity).saturating_mul(
            usize_to_u64(mem::size_of::<BucketKey>()).saturating_add(partition.retained_bytes()),
        );
        let reservation = tracker.reserve(allocation_bytes, &mut receipt.counters);
        if let Err(reason) = reservation {
            Self::record_fallback(
                anchor,
                Some(key),
                reason,
                Schedule::PartitionScan,
                tracker,
                receipt,
            )?;
            return Ok(Coverage::PartitionScan);
        }
        let mut buckets = Vec::new();
        if buckets.try_reserve_exact(capacity).is_err() {
            tracker.release(allocation_bytes)?;
            Self::record_fallback(
                anchor,
                Some(key),
                FallbackReason::AllocatorFailure,
                Schedule::PartitionScan,
                tracker,
                receipt,
            )?;
            return Ok(Coverage::PartitionScan);
        }
        for x in envelope.minimum_x..=envelope.maximum_x {
            for y in envelope.minimum_y..=envelope.maximum_y {
                buckets.push(BucketKey {
                    partition: partition.clone(),
                    x,
                    y,
                });
            }
        }
        tracker.promote(allocation_bytes)?;
        Ok(Coverage::Buckets(buckets))
    }

    fn record_fallback(
        anchor: &AnchorKey,
        range: Option<&RangeKey>,
        reason: FallbackReason,
        selected_schedule: Schedule,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        let bytes = usize_to_u64(mem::size_of::<FallbackReceipt>())
            .saturating_add(anchor_retained_bytes(anchor))
            .saturating_add(range.map_or(0, range_key_retained_bytes));
        let attempted_receipt_bytes = receipt.counters.receipt_bytes.saturating_add(bytes);
        if let Err(kind) = tracker.reserve_output(bytes, &mut receipt.counters) {
            receipt.mark_incomplete(attempted_receipt_bytes);
            return Err(kind);
        }
        if receipt.fallbacks.try_reserve(1).is_err() {
            receipt.mark_incomplete(attempted_receipt_bytes);
            return Err(FailureKind::ReceiptAllocationExhausted(
                FallbackReason::AllocatorFailure,
            ));
        }
        receipt.fallbacks.push(FallbackReceipt {
            anchor_key: anchor.clone(),
            range_occurrence_refs: range.cloned(),
            reason,
            selected_schedule,
        });
        Ok(())
    }

    fn decode_environment(
        binding: &crate::model::I32Binding,
        environment: &Environment,
    ) -> Result<i32, FailureKind> {
        let value = environment
            .binding(&binding.binding_ref)
            .ok_or(FailureKind::Contract(ContractError::MissingBinding))?;
        binding.decode(value).map_err(FailureKind::Contract)
    }

    fn visible_supports_current(
        &self,
        range: &RangeRecord,
        visible: &mut BTreeSet<OpaqueRef>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        match &range.coverage {
            Coverage::Buckets(buckets) => {
                for bucket in buckets {
                    receipt.counters.index_bucket_probes += 1;
                    if let Some(occurrences) = self.point_buckets.get(bucket) {
                        for occurrence in occurrences {
                            self.visible_supports_for_point_current(
                                range, occurrence, visible, tracker, receipt,
                            )?;
                        }
                    }
                }
            }
            Coverage::PartitionScan => {
                if let Some(occurrences) = self.points_by_partition.get(&range.geometry.partition) {
                    receipt.counters.fallback_point_visits = receipt
                        .counters
                        .fallback_point_visits
                        .saturating_add(usize_to_u64(occurrences.len()));
                    for occurrence in occurrences {
                        if self
                            .points
                            .get(occurrence)
                            .is_some_and(|point| range.geometry.contains(point))
                        {
                            self.visible_supports_for_point_current(
                                range, occurrence, visible, tracker, receipt,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn visible_supports_for_point_current(
        &self,
        range: &RangeRecord,
        point_occurrence: &OpaqueRef,
        visible: &mut BTreeSet<OpaqueRef>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        receipt.counters.candidate_bindings += 1;
        let point_row = self
            .rows
            .get(point_occurrence)
            .ok_or(FailureKind::InternalInvariant)?;
        let Some((tuple, tuple_bytes)) = range.environment.premise_tuple_with_point(
            &self.plan.point_slot_ref,
            point_row,
            &self.contract,
            tracker,
            &mut receipt.counters,
        )?
        else {
            return Ok(());
        };
        receipt.counters.premise_occurrences_visited = receipt
            .counters
            .premise_occurrences_visited
            .saturating_add(usize_to_u64(tuple.len()));
        if let Some(supports) = self.supports_by_tuple.get(&tuple) {
            receipt.counters.support_entries_read = receipt
                .counters
                .support_entries_read
                .saturating_add(usize_to_u64(supports.len()));
            for support in supports {
                if !visible.contains(support) {
                    tracker
                        .reserve(support.retained_bytes(), &mut receipt.counters)
                        .map_err(FailureKind::TemporaryAllocationExhausted)?;
                    visible.insert(support.clone());
                }
            }
        }
        tracker.release(tuple_bytes)?;
        Ok(())
    }

    fn install_prepared_anchor(
        &mut self,
        anchor: AnchorKey,
        prepared: PreparedAnchor,
        counters: &mut WorkCounters,
    ) {
        let PreparedAnchor {
            anchor_record,
            visible_supports,
        } = prepared;
        self.install_range_indexes(&anchor, &anchor_record, counters);
        for support_ref in visible_supports {
            if let Some(support) = self.supports.get(&support_ref) {
                self.visible
                    .attach_anchor(anchor.clone(), support, counters);
            }
        }
        self.anchors.insert(anchor, anchor_record);
    }

    fn install_range_indexes(
        &mut self,
        anchor: &AnchorKey,
        record: &AnchorRecord,
        counters: &mut WorkCounters,
    ) {
        for bucket in &record.indexed_buckets {
            self.range_buckets
                .entry(bucket.clone())
                .or_default()
                .insert(anchor.clone());
            counters.index_membership_writes += 1;
        }
        for partition in &record.partition_scans {
            self.partition_fallback_ranges
                .entry(partition.clone())
                .or_default()
                .insert(anchor.clone());
            counters.index_membership_writes += 1;
        }
    }

    fn remove_range_indexes(&mut self, anchor: &AnchorKey, counters: &mut WorkCounters) {
        let Some(record) = self.anchors.remove(anchor) else {
            return;
        };
        for bucket in record.indexed_buckets {
            remove_set_member(&mut self.range_buckets, &bucket, anchor);
            counters.index_membership_writes += 1;
        }
        for partition in record.partition_scans {
            remove_set_member(&mut self.partition_fallback_ranges, &partition, anchor);
            counters.index_membership_writes += 1;
        }
    }

    fn retained_cost(&self, counters: &mut WorkCounters) -> Result<RetainedCost, FailureKind> {
        counters.retained_cost_records_read = counters.retained_cost_records_read.saturating_add(5);
        let mut cost = RetainedCost::default()
            .saturating_add(RetainedCost::entry(self.contract.retained_bytes()))
            .saturating_add(RetainedCost::entry(self.plan.retained_bytes()))
            .saturating_add(RetainedCost::entry(self.admitted_state.retained_bytes()))
            .saturating_add(RetainedCost::entry(self.snapshot_ref.retained_bytes()))
            .saturating_add(RetainedCost::entry(self.point_input_ref.retained_bytes()));
        for row in self.rows.values() {
            counters.retained_cost_records_read += 1;
            cost = cost.saturating_add(self.row_retained_cost(row)?);
        }
        for (support_ref, support) in &self.supports {
            counters.retained_cost_records_read += 1;
            let location = self
                .support_locations
                .get(support_ref)
                .ok_or(FailureKind::InternalInvariant)?;
            cost = cost.saturating_add(Self::support_retained_cost(support, location));
        }
        for (anchor, record) in &self.anchors {
            counters.retained_cost_records_read = counters
                .retained_cost_records_read
                .saturating_add(1)
                .saturating_add(usize_to_u64(record.ranges.len()))
                .saturating_add(usize_to_u64(record.indexed_buckets.len()))
                .saturating_add(usize_to_u64(record.partition_scans.len()));
            cost = cost.saturating_add(Self::anchor_retained_cost(anchor, record));
            if let Some(supports) = self.visible.visible_supports_for_anchor(anchor) {
                for support_ref in supports {
                    counters.retained_cost_records_read += 1;
                    let support = self
                        .supports
                        .get(support_ref)
                        .ok_or(FailureKind::InternalInvariant)?;
                    cost = cost.saturating_add(Self::visible_retained_cost(anchor, support));
                }
            }
        }
        Ok(cost)
    }

    fn prepared_anchor_retained_cost(
        &self,
        anchor: &AnchorKey,
        prepared: &PreparedAnchor,
        counters: &mut WorkCounters,
    ) -> Result<RetainedCost, FailureKind> {
        counters.retained_cost_records_read = counters
            .retained_cost_records_read
            .saturating_add(1)
            .saturating_add(usize_to_u64(prepared.anchor_record.ranges.len()))
            .saturating_add(usize_to_u64(prepared.anchor_record.indexed_buckets.len()))
            .saturating_add(usize_to_u64(prepared.anchor_record.partition_scans.len()));
        let mut cost = Self::anchor_retained_cost(anchor, &prepared.anchor_record);
        for support_ref in &prepared.visible_supports {
            counters.retained_cost_records_read += 1;
            let support = self
                .supports
                .get(support_ref)
                .ok_or(FailureKind::InternalInvariant)?;
            cost = cost.saturating_add(Self::visible_retained_cost(anchor, support));
        }
        Ok(cost)
    }

    fn row_retained_cost(&self, row: &InputRow) -> Result<RetainedCost, FailureKind> {
        let mut cost = RetainedCost::entry(
            row.occurrence_ref
                .retained_bytes()
                .saturating_add(row.retained_bytes()),
        );
        if row.input_ref == *self.point_input_ref() {
            let partition = row
                .binding(&self.plan.partition_binding_ref)
                .ok_or(FailureKind::Contract(ContractError::MissingBinding))?;
            let bucket_bytes = usize_to_u64(mem::size_of::<BucketKey>())
                .saturating_add(partition.retained_bytes());
            let point_bytes = usize_to_u64(mem::size_of::<PointRecord>())
                .saturating_add(partition.retained_bytes())
                .saturating_add(bucket_bytes);
            cost = cost
                .saturating_add(RetainedCost::entry(
                    row.occurrence_ref
                        .retained_bytes()
                        .saturating_add(point_bytes),
                ))
                .saturating_add(RetainedCost::entry(
                    bucket_bytes.saturating_add(row.occurrence_ref.retained_bytes()),
                ))
                .saturating_add(RetainedCost::entry(
                    partition
                        .retained_bytes()
                        .saturating_add(row.occurrence_ref.retained_bytes()),
                ));
        }
        let mut range_slots = self.range_slots_for_input(&row.input_ref).peekable();
        if range_slots.peek().is_some() {
            let anchor_bytes = self
                .anchor_retained_bytes_for_row(row)
                .map_err(FailureKind::Contract)?;
            for slot in range_slots {
                cost = cost.saturating_add(RetainedCost::entry(
                    slot.retained_bytes()
                        .saturating_add(anchor_bytes)
                        .saturating_add(row.occurrence_ref.retained_bytes()),
                ));
            }
        }
        Ok(cost)
    }

    fn anchor_retained_bytes_for_row(&self, row: &InputRow) -> Result<u64, ContractError> {
        self.plan.anchor_binding_refs.iter().try_fold(
            usize_to_u64(mem::size_of::<AnchorKey>()),
            |bytes, binding_ref| {
                let value = row
                    .binding(binding_ref)
                    .ok_or(ContractError::MissingBinding)?;
                Ok(bytes
                    .saturating_add(binding_ref.retained_bytes())
                    .saturating_add(value.retained_bytes()))
            },
        )
    }

    fn support_retained_cost(support: &SupportRecord, location: &SupportLocation) -> RetainedCost {
        Self::support_retained_cost_for_shape(
            support,
            SupportLocationShape {
                anchor_bytes: anchor_retained_bytes(&location.anchor),
                range_bytes: range_key_retained_bytes(&location.range),
            },
        )
    }

    fn support_retained_cost_for_shape(
        support: &SupportRecord,
        location: SupportLocationShape,
    ) -> RetainedCost {
        let support_ref = &support.support_occurrence_ref;
        let tuple_bytes = support.premise_occurrence_refs.iter().fold(
            usize_to_u64(mem::size_of::<Vec<OpaqueRef>>()),
            |total, occurrence| total.saturating_add(occurrence.retained_bytes()),
        );
        let mut cost = RetainedCost::entry(
            support_ref
                .retained_bytes()
                .saturating_add(support.retained_bytes()),
        )
        .saturating_add(RetainedCost::entry(
            tuple_bytes.saturating_add(support_ref.retained_bytes()),
        ))
        .saturating_add(RetainedCost::entry(
            support_ref
                .retained_bytes()
                .saturating_add(location.retained_bytes()),
        ))
        .saturating_add(RetainedCost::entry(
            location
                .anchor_bytes
                .saturating_add(support_ref.retained_bytes()),
        ))
        .saturating_add(RetainedCost::entry(
            location
                .range_bytes
                .saturating_add(support_ref.retained_bytes()),
        ));
        for (index, premise) in support.premise_occurrence_refs.iter().enumerate() {
            if support.premise_occurrence_refs[..index].contains(premise) {
                continue;
            }
            cost = cost.saturating_add(RetainedCost::entry(
                premise
                    .retained_bytes()
                    .saturating_add(support_ref.retained_bytes())
                    .saturating_add(usize_to_u64(mem::size_of::<u64>())),
            ));
        }
        cost
    }

    fn visible_retained_cost(anchor: &AnchorKey, support: &SupportRecord) -> RetainedCost {
        let support_ref = &support.support_occurrence_ref;
        let mut cost = RetainedCost::entry(support_ref.retained_bytes())
            .saturating_add(RetainedCost::entry(
                support
                    .output
                    .retained_bytes()
                    .saturating_add(support_ref.retained_bytes()),
            ))
            .saturating_add(RetainedCost::entry(
                anchor_retained_bytes(anchor).saturating_add(support_ref.retained_bytes()),
            ))
            .saturating_add(RetainedCost::entry(
                support_ref
                    .retained_bytes()
                    .saturating_add(anchor_retained_bytes(anchor)),
            ));
        for (index, premise) in support.premise_occurrence_refs.iter().enumerate() {
            if support.premise_occurrence_refs[..index].contains(premise) {
                continue;
            }
            cost = cost.saturating_add(RetainedCost::entry(
                premise
                    .retained_bytes()
                    .saturating_add(support_ref.retained_bytes())
                    .saturating_add(usize_to_u64(mem::size_of::<u64>())),
            ));
        }
        cost
    }

    fn anchor_retained_cost(anchor: &AnchorKey, record: &AnchorRecord) -> RetainedCost {
        let mut cost = RetainedCost::entry(
            anchor_retained_bytes(anchor)
                .saturating_add(usize_to_u64(mem::size_of::<AnchorRecord>())),
        );
        for (range_key, range) in &record.ranges {
            cost = cost.saturating_add(RetainedCost::entry(
                range_key_retained_bytes(range_key).saturating_add(range.retained_bytes()),
            ));
        }
        for bucket in &record.indexed_buckets {
            cost = cost
                .saturating_add(RetainedCost::entry(bucket.retained_bytes()))
                .saturating_add(RetainedCost::entry(
                    bucket
                        .retained_bytes()
                        .saturating_add(anchor_retained_bytes(anchor)),
                ));
        }
        for partition in &record.partition_scans {
            cost = cost
                .saturating_add(RetainedCost::entry(partition.retained_bytes()))
                .saturating_add(RetainedCost::entry(
                    partition
                        .retained_bytes()
                        .saturating_add(anchor_retained_bytes(anchor)),
                ));
        }
        cost
    }

    fn removed_retained_cost(
        &self,
        update: &MaterializationUpdate,
        affected_anchors: &BTreeSet<AnchorKey>,
        counters: &mut WorkCounters,
    ) -> Result<RetainedCost, FailureKind> {
        counters.retained_cost_records_read = counters.retained_cost_records_read.saturating_add(2);
        let mut cost = RetainedCost::entry(self.admitted_state.retained_bytes())
            .saturating_add(RetainedCost::entry(self.snapshot_ref.retained_bytes()));
        for row in &update.withdraw_rows {
            counters.retained_cost_records_read += 1;
            cost = cost.saturating_add(self.row_retained_cost(row)?);
        }
        for support_ref in &update.withdraw_support_occurrence_refs {
            counters.retained_cost_records_read += 1;
            let support = self
                .supports
                .get(support_ref)
                .ok_or(FailureKind::InternalInvariant)?;
            let location = self
                .support_locations
                .get(support_ref)
                .ok_or(FailureKind::InternalInvariant)?;
            cost = cost.saturating_add(Self::support_retained_cost(support, location));
        }
        for anchor in affected_anchors {
            if let Some(record) = self.anchors.get(anchor) {
                counters.retained_cost_records_read = counters
                    .retained_cost_records_read
                    .saturating_add(1)
                    .saturating_add(usize_to_u64(record.ranges.len()))
                    .saturating_add(usize_to_u64(record.indexed_buckets.len()))
                    .saturating_add(usize_to_u64(record.partition_scans.len()));
                cost = cost.saturating_add(Self::anchor_retained_cost(anchor, record));
            }
            if let Some(supports) = self.visible.visible_supports_for_anchor(anchor) {
                for support_ref in supports {
                    counters.retained_cost_records_read += 1;
                    let support = self
                        .supports
                        .get(support_ref)
                        .ok_or(FailureKind::InternalInvariant)?;
                    cost = cost.saturating_add(Self::visible_retained_cost(anchor, support));
                }
            }
        }
        Ok(cost)
    }

    fn added_retained_cost(
        &self,
        update: &MaterializationUpdate,
        successor_state: &AdmittedStateBinding,
        admitted_supports: &BTreeMap<OpaqueRef, &SupportRecord>,
        admitted_support_locations: &BTreeMap<OpaqueRef, SupportLocation>,
        prepared_anchors: &BTreeMap<AnchorKey, Option<PreparedAnchor>>,
        counters: &mut WorkCounters,
    ) -> Result<RetainedCost, FailureKind> {
        counters.retained_cost_records_read = counters.retained_cost_records_read.saturating_add(2);
        let mut cost = RetainedCost::entry(successor_state.retained_bytes()).saturating_add(
            RetainedCost::entry(update.result_snapshot_ref.retained_bytes()),
        );
        for row in &update.admit_rows {
            counters.retained_cost_records_read += 1;
            cost = cost.saturating_add(self.row_retained_cost(row)?);
        }
        for support in &update.admit_supports {
            counters.retained_cost_records_read += 1;
            let location = admitted_support_locations
                .get(&support.support_occurrence_ref)
                .ok_or(FailureKind::InternalInvariant)?;
            cost = cost.saturating_add(Self::support_retained_cost(support, location));
        }
        for (anchor, prepared) in prepared_anchors {
            let Some(prepared) = prepared else {
                continue;
            };
            counters.retained_cost_records_read = counters
                .retained_cost_records_read
                .saturating_add(1)
                .saturating_add(usize_to_u64(prepared.anchor_record.ranges.len()))
                .saturating_add(usize_to_u64(prepared.anchor_record.indexed_buckets.len()))
                .saturating_add(usize_to_u64(prepared.anchor_record.partition_scans.len()));
            cost = cost.saturating_add(Self::anchor_retained_cost(anchor, &prepared.anchor_record));
            for support_ref in &prepared.visible_supports {
                counters.retained_cost_records_read += 1;
                let support = admitted_supports
                    .get(support_ref)
                    .copied()
                    .or_else(|| self.supports.get(support_ref))
                    .ok_or(FailureKind::InternalInvariant)?;
                cost = cost.saturating_add(Self::visible_retained_cost(anchor, support));
            }
        }
        Ok(cost)
    }
}

#[derive(Clone, Copy)]
struct RangeCoverageInput<'a> {
    anchor: &'a AnchorKey,
    key: &'a RangeKey,
    partition: &'a OpaqueValue,
    envelope: BucketEnvelope,
    budget: PhysicalBudget,
}

#[derive(Clone, Copy)]
struct BucketEnvelope {
    minimum_x: i64,
    maximum_x: i64,
    minimum_y: i64,
    maximum_y: i64,
}

impl BucketEnvelope {
    fn bucket_count(self) -> Result<u128, FailureKind> {
        let x_count = u128::try_from(self.maximum_x - self.minimum_x + 1)
            .map_err(|_| FailureKind::TemporaryAllocationExhausted(FallbackReason::BucketLimit))?;
        let y_count = u128::try_from(self.maximum_y - self.minimum_y + 1)
            .map_err(|_| FailureKind::TemporaryAllocationExhausted(FallbackReason::BucketLimit))?;
        x_count
            .checked_mul(y_count)
            .ok_or(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::BucketLimit,
            ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct BucketKey {
    partition: OpaqueValue,
    x: i64,
    y: i64,
}

impl BucketKey {
    fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>()).saturating_add(self.partition.retained_bytes())
    }
}

#[derive(Clone, Debug)]
struct PointRecord {
    partition: OpaqueValue,
    bucket: BucketKey,
}

type RangeKey = Vec<OpaqueRef>;

#[derive(Clone, Copy)]
struct SupportLocationShape {
    anchor_bytes: u64,
    range_bytes: u64,
}

impl SupportLocationShape {
    fn retained_bytes(self) -> u64 {
        usize_to_u64(mem::size_of::<SupportLocation>())
            .saturating_add(self.anchor_bytes)
            .saturating_add(self.range_bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SupportLocation {
    anchor: AnchorKey,
    range: RangeKey,
}

#[derive(Clone, Debug, Default)]
struct Environment {
    bindings: BTreeMap<OpaqueRef, OpaqueValue>,
    occurrences: BTreeMap<OpaqueRef, OpaqueRef>,
}

impl Environment {
    fn binding(&self, binding_ref: &OpaqueRef) -> Option<&OpaqueValue> {
        self.bindings.get(binding_ref)
    }

    fn can_merge(&self, row: &InputRow) -> bool {
        !row.bindings.iter().any(|binding| {
            self.bindings
                .get(&binding.binding_ref)
                .is_some_and(|existing| existing != &binding.value)
        })
    }

    fn merge_unchecked(&self, slot_ref: &OpaqueRef, row: &InputRow) -> Self {
        let mut merged = self.clone();
        for binding in &row.bindings {
            merged
                .bindings
                .entry(binding.binding_ref.clone())
                .or_insert_with(|| binding.value.clone());
        }
        merged
            .occurrences
            .insert(slot_ref.clone(), row.occurrence_ref.clone());
        merged
    }

    fn merged_deep_bytes(&self, slot_ref: &OpaqueRef, row: &InputRow) -> u64 {
        let mut bytes = self.deep_bytes();
        for binding in &row.bindings {
            if !self.bindings.contains_key(&binding.binding_ref) {
                bytes = bytes
                    .saturating_add(binding.binding_ref.retained_bytes())
                    .saturating_add(binding.value.retained_bytes());
            }
        }
        if !self.occurrences.contains_key(slot_ref) {
            bytes = bytes
                .saturating_add(slot_ref.retained_bytes())
                .saturating_add(row.occurrence_ref.retained_bytes());
        }
        bytes
    }

    fn deep_bytes(&self) -> u64 {
        let binding_bytes = self.bindings.iter().fold(0_u64, |total, (key, value)| {
            total
                .saturating_add(key.retained_bytes())
                .saturating_add(value.retained_bytes())
        });
        self.occurrences
            .iter()
            .fold(binding_bytes, |total, (slot, occurrence)| {
                total
                    .saturating_add(slot.retained_bytes())
                    .saturating_add(occurrence.retained_bytes())
            })
    }

    fn range_key(&self, plan: &GridPlan) -> Result<RangeKey, FailureKind> {
        let mut key = Vec::new();
        if key.try_reserve_exact(plan.range_slot_refs.len()).is_err() {
            return Err(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::AllocatorFailure,
            ));
        }
        for slot_ref in &plan.range_slot_refs {
            key.push(
                self.occurrences
                    .get(slot_ref)
                    .cloned()
                    .ok_or(FailureKind::InternalInvariant)?,
            );
        }
        Ok(key)
    }

    fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>()).saturating_add(self.deep_bytes())
    }

    fn premise_tuple_with_point(
        &self,
        point_slot_ref: &OpaqueRef,
        point_row: &InputRow,
        contract: &MaterializationContract,
        tracker: &mut AllocationTracker,
        counters: &mut WorkCounters,
    ) -> Result<Option<(Vec<OpaqueRef>, u64)>, FailureKind> {
        if !self.can_merge(point_row) {
            return Ok(None);
        }
        let mut tuple_bytes = usize_to_u64(mem::size_of::<Vec<OpaqueRef>>());
        for slot in &contract.premise_slots {
            let occurrence = if slot.slot_ref == *point_slot_ref {
                &point_row.occurrence_ref
            } else {
                self.occurrences
                    .get(&slot.slot_ref)
                    .ok_or(FailureKind::InternalInvariant)?
            };
            tuple_bytes = tuple_bytes.saturating_add(occurrence.retained_bytes());
        }
        tracker
            .reserve(tuple_bytes, counters)
            .map_err(FailureKind::TemporaryAllocationExhausted)?;
        let mut tuple = Vec::new();
        if tuple
            .try_reserve_exact(contract.premise_slots.len())
            .is_err()
        {
            tracker.release(tuple_bytes)?;
            return Err(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::AllocatorFailure,
            ));
        }
        for slot in &contract.premise_slots {
            let occurrence = if slot.slot_ref == *point_slot_ref {
                &point_row.occurrence_ref
            } else {
                let Some(occurrence) = self.occurrences.get(&slot.slot_ref) else {
                    tracker.release(tuple_bytes)?;
                    return Err(FailureKind::InternalInvariant);
                };
                occurrence
            };
            tuple.push(occurrence.clone());
        }
        Ok(Some((tuple, tuple_bytes)))
    }
}

#[derive(Debug)]
struct EnvironmentBatch {
    environments: Vec<Environment>,
    temporary_bytes: u64,
}

struct PromotedEnvironments {
    environments: Vec<Environment>,
    vector_bytes: u64,
}

impl EnvironmentBatch {
    fn initial(
        tracker: &mut AllocationTracker,
        counters: &mut WorkCounters,
    ) -> Result<Self, FailureKind> {
        let bytes = usize_to_u64(mem::size_of::<Environment>());
        tracker
            .reserve(bytes, counters)
            .map_err(FailureKind::TemporaryAllocationExhausted)?;
        let mut environments = Vec::new();
        if environments.try_reserve_exact(1).is_err() {
            tracker.release(bytes)?;
            return Err(FailureKind::TemporaryAllocationExhausted(
                FallbackReason::AllocatorFailure,
            ));
        }
        environments.push(Environment::default());
        Ok(Self {
            environments,
            temporary_bytes: bytes,
        })
    }

    fn into_promoted(
        self,
        tracker: &mut AllocationTracker,
    ) -> Result<PromotedEnvironments, FailureKind> {
        let retained_deep_bytes = self.environments.iter().fold(0_u64, |total, environment| {
            total.saturating_add(environment.deep_bytes())
        });
        let vector_bytes = self
            .temporary_bytes
            .checked_sub(retained_deep_bytes)
            .ok_or(FailureKind::InternalInvariant)?;
        tracker.promote(retained_deep_bytes)?;
        Ok(PromotedEnvironments {
            environments: self.environments,
            vector_bytes,
        })
    }
}

#[derive(Debug)]
enum RangePreparation {
    Absent,
    Ready(EnvironmentBatch),
}

#[derive(Clone, Debug)]
struct RangeRecord {
    key: RangeKey,
    environment: Environment,
    geometry: RangeGeometry,
    coverage: Coverage,
}

#[derive(Clone, Debug)]
struct RangeGeometry {
    partition: OpaqueValue,
    minimum_bucket_x: i64,
    maximum_bucket_x: i64,
    minimum_bucket_y: i64,
    maximum_bucket_y: i64,
}

impl RangeGeometry {
    fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>()).saturating_add(self.partition.retained_bytes())
    }

    fn contains(&self, point: &PointRecord) -> bool {
        self.partition == point.partition
            && (self.minimum_bucket_x..=self.maximum_bucket_x).contains(&point.bucket.x)
            && (self.minimum_bucket_y..=self.maximum_bucket_y).contains(&point.bucket.y)
    }
}

#[derive(Clone, Debug)]
enum Coverage {
    Buckets(Vec<BucketKey>),
    PartitionScan,
}

impl Coverage {
    fn retained_bytes(&self) -> u64 {
        match self {
            Self::Buckets(buckets) => buckets
                .iter()
                .fold(usize_to_u64(mem::size_of::<Self>()), |total, bucket| {
                    total.saturating_add(bucket.retained_bytes())
                }),
            Self::PartitionScan => usize_to_u64(mem::size_of::<Self>()),
        }
    }
}

impl RangeRecord {
    fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(range_key_retained_bytes(&self.key))
            .saturating_add(self.environment.retained_bytes())
            .saturating_add(self.geometry.retained_bytes())
            .saturating_add(self.coverage.retained_bytes())
    }
}

#[derive(Debug)]
struct PreparedAnchor {
    anchor_record: AnchorRecord,
    visible_supports: BTreeSet<OpaqueRef>,
}

#[derive(Debug)]
struct AnchorRecord {
    ranges: BTreeMap<RangeKey, RangeRecord>,
    indexed_buckets: BTreeSet<BucketKey>,
    partition_scans: BTreeSet<OpaqueValue>,
}

fn bucket_coordinate(raw: i64, width: i64) -> i64 {
    raw.div_euclid(width)
}

fn anchor_retained_bytes(anchor: &AnchorKey) -> u64 {
    anchor.iter().fold(
        usize_to_u64(mem::size_of::<AnchorKey>()),
        |total, (binding, value)| {
            total
                .saturating_add(binding.retained_bytes())
                .saturating_add(value.retained_bytes())
        },
    )
}

fn range_key_retained_bytes(range: &RangeKey) -> u64 {
    range.iter().fold(
        usize_to_u64(mem::size_of::<RangeKey>()),
        |total, occurrence| total.saturating_add(occurrence.retained_bytes()),
    )
}

fn insert_temporary_ref(
    values: &mut BTreeSet<OpaqueRef>,
    value: &OpaqueRef,
    tracker: &mut AllocationTracker,
    counters: &mut WorkCounters,
) -> Result<bool, FailureKind> {
    if values.contains(value) {
        return Ok(false);
    }
    tracker
        .reserve(value.retained_bytes(), counters)
        .map_err(FailureKind::TemporaryAllocationExhausted)?;
    values.insert(value.clone());
    Ok(true)
}

fn insert_temporary_anchor(
    values: &mut BTreeSet<AnchorKey>,
    value: &AnchorKey,
    tracker: &mut AllocationTracker,
    counters: &mut WorkCounters,
) -> Result<bool, FailureKind> {
    if values.contains(value) {
        return Ok(false);
    }
    tracker
        .reserve(anchor_retained_bytes(value), counters)
        .map_err(FailureKind::TemporaryAllocationExhausted)?;
    values.insert(value.clone());
    Ok(true)
}

fn insert_temporary_range_occurrence(
    index: &mut BTreeMap<OpaqueRef, BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>>,
    slot_ref: &OpaqueRef,
    anchor: &AnchorKey,
    occurrence_ref: &OpaqueRef,
    tracker: &mut AllocationTracker,
    counters: &mut WorkCounters,
) -> Result<(), FailureKind> {
    let existing_anchor = index
        .get(slot_ref)
        .and_then(|by_anchor| by_anchor.get(anchor));
    if existing_anchor.is_some_and(|occurrences| occurrences.contains(occurrence_ref)) {
        return Ok(());
    }
    let bytes = occurrence_ref
        .retained_bytes()
        .saturating_add(if index.contains_key(slot_ref) {
            0
        } else {
            slot_ref.retained_bytes()
        })
        .saturating_add(if existing_anchor.is_some() {
            0
        } else {
            anchor_retained_bytes(anchor)
        });
    tracker
        .reserve(bytes, counters)
        .map_err(FailureKind::TemporaryAllocationExhausted)?;
    index
        .entry(slot_ref.clone())
        .or_default()
        .entry(anchor.clone())
        .or_default()
        .insert(occurrence_ref.clone());
    Ok(())
}

fn insert_temporary_bucket_occurrence(
    index: &mut BTreeMap<BucketKey, BTreeSet<OpaqueRef>>,
    bucket: &BucketKey,
    occurrence_ref: &OpaqueRef,
    tracker: &mut AllocationTracker,
    counters: &mut WorkCounters,
) -> Result<(), FailureKind> {
    if index
        .get(bucket)
        .is_some_and(|occurrences| occurrences.contains(occurrence_ref))
    {
        return Ok(());
    }
    let bytes = occurrence_ref
        .retained_bytes()
        .saturating_add(if index.contains_key(bucket) {
            0
        } else {
            bucket.retained_bytes()
        });
    tracker
        .reserve(bytes, counters)
        .map_err(FailureKind::TemporaryAllocationExhausted)?;
    index
        .entry(bucket.clone())
        .or_default()
        .insert(occurrence_ref.clone());
    Ok(())
}

fn insert_temporary_partition_occurrence(
    index: &mut BTreeMap<OpaqueValue, BTreeSet<OpaqueRef>>,
    partition: &OpaqueValue,
    occurrence_ref: &OpaqueRef,
    tracker: &mut AllocationTracker,
    counters: &mut WorkCounters,
) -> Result<(), FailureKind> {
    if index
        .get(partition)
        .is_some_and(|occurrences| occurrences.contains(occurrence_ref))
    {
        return Ok(());
    }
    let bytes = occurrence_ref
        .retained_bytes()
        .saturating_add(if index.contains_key(partition) {
            0
        } else {
            partition.retained_bytes()
        });
    tracker
        .reserve(bytes, counters)
        .map_err(FailureKind::TemporaryAllocationExhausted)?;
    index
        .entry(partition.clone())
        .or_default()
        .insert(occurrence_ref.clone());
    Ok(())
}

fn insert_temporary_anchor_occurrence(
    index: &mut BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>,
    anchor: &AnchorKey,
    occurrence_ref: &OpaqueRef,
    tracker: &mut AllocationTracker,
    counters: &mut WorkCounters,
) -> Result<(), FailureKind> {
    if index
        .get(anchor)
        .is_some_and(|occurrences| occurrences.contains(occurrence_ref))
    {
        return Ok(());
    }
    let bytes = occurrence_ref
        .retained_bytes()
        .saturating_add(if index.contains_key(anchor) {
            0
        } else {
            anchor_retained_bytes(anchor)
        });
    tracker
        .reserve(bytes, counters)
        .map_err(FailureKind::TemporaryAllocationExhausted)?;
    index
        .entry(anchor.clone())
        .or_default()
        .insert(occurrence_ref.clone());
    Ok(())
}

fn remove_set_member<K, V>(index: &mut BTreeMap<K, BTreeSet<V>>, key: &K, value: &V)
where
    K: Ord,
    V: Ord,
{
    let remove_entry = index.get_mut(key).is_some_and(|values| {
        values.remove(value);
        values.is_empty()
    });
    if remove_entry {
        index.remove(key);
    }
}

impl UniformGridMaterializer {
    #[allow(clippy::too_many_lines)]
    fn prepare_update(
        &self,
        update: &MaterializationUpdate,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<PreparedUpdate, FailureKind> {
        receipt.counters.contract_checks += 1;
        receipt.counters.graph_reads += 1;
        if update.graph_ref != self.contract.graph_ref {
            return Err(FailureKind::Contract(ContractError::GraphIdentityMismatch));
        }
        if update.contract_ref != self.contract.contract_ref {
            return Err(FailureKind::Contract(
                ContractError::ContractIdentityMismatch,
            ));
        }
        if update.plan_ref != self.plan.plan_ref {
            return Err(FailureKind::Contract(ContractError::PlanIdentityMismatch));
        }
        if update.base_snapshot_ref != self.snapshot_ref {
            return Err(FailureKind::Contract(ContractError::ExactBaseMismatch));
        }
        update
            .admitted_delta
            .validate_update(
                &self.admitted_state,
                &update.base_snapshot_ref,
                &update.result_snapshot_ref,
            )
            .map_err(FailureKind::Contract)?;
        receipt.counters.input_rows_read = usize_to_u64(
            update
                .withdraw_rows
                .len()
                .saturating_add(update.admit_rows.len()),
        );
        receipt.counters.support_records_read = usize_to_u64(
            update
                .withdraw_support_occurrence_refs
                .len()
                .saturating_add(update.admit_supports.len()),
        );

        let mut withdrawn_rows = BTreeSet::new();
        for row in &update.withdraw_rows {
            row.validate().map_err(FailureKind::Contract)?;
            if withdrawn_rows.contains(&row.occurrence_ref) {
                return Err(FailureKind::Contract(ContractError::DuplicateWithdrawal));
            }
            insert_temporary_ref(
                &mut withdrawn_rows,
                &row.occurrence_ref,
                tracker,
                &mut receipt.counters,
            )?;
            let existing = self
                .rows
                .get(&row.occurrence_ref)
                .ok_or(FailureKind::Contract(ContractError::WithdrawalMissing))?;
            if existing != row {
                return Err(FailureKind::Contract(
                    ContractError::WithdrawalContentMismatch,
                ));
            }
        }

        let mut admitted_rows = BTreeMap::new();
        for row in &update.admit_rows {
            row.validate().map_err(FailureKind::Contract)?;
            if self.rows.contains_key(&row.occurrence_ref)
                || withdrawn_rows.contains(&row.occurrence_ref)
            {
                return Err(FailureKind::Contract(ContractError::ReusedOccurrence));
            }
            if admitted_rows.contains_key(&row.occurrence_ref) {
                return Err(FailureKind::Contract(ContractError::DuplicateOccurrence));
            }
            tracker
                .reserve(
                    row.occurrence_ref
                        .retained_bytes()
                        .saturating_add(usize_to_u64(mem::size_of::<&InputRow>())),
                    &mut receipt.counters,
                )
                .map_err(FailureKind::TemporaryAllocationExhausted)?;
            admitted_rows.insert(row.occurrence_ref.clone(), row);
        }
        let projected_rows = self
            .rows
            .len()
            .saturating_sub(withdrawn_rows.len())
            .saturating_add(admitted_rows.len());
        if projected_rows > update.budget.maximum_rows {
            return Err(FailureKind::RowLimitExceeded);
        }

        let mut withdrawn_supports = BTreeSet::new();
        for support_ref in &update.withdraw_support_occurrence_refs {
            if withdrawn_supports.contains(support_ref) {
                return Err(FailureKind::Contract(
                    ContractError::DuplicateSupportWithdrawal,
                ));
            }
            insert_temporary_ref(
                &mut withdrawn_supports,
                support_ref,
                tracker,
                &mut receipt.counters,
            )?;
            if !self.supports.contains_key(support_ref) {
                return Err(FailureKind::Contract(
                    ContractError::SupportWithdrawalMissing,
                ));
            }
        }
        let mut admitted_supports = BTreeMap::new();
        for support in &update.admit_supports {
            if self.supports.contains_key(&support.support_occurrence_ref)
                || withdrawn_supports.contains(&support.support_occurrence_ref)
            {
                return Err(FailureKind::Contract(ContractError::ReusedOccurrence));
            }
            if admitted_supports.contains_key(&support.support_occurrence_ref) {
                return Err(FailureKind::Contract(
                    ContractError::DuplicateSupportOccurrence,
                ));
            }
            tracker
                .reserve(
                    support
                        .support_occurrence_ref
                        .retained_bytes()
                        .saturating_add(usize_to_u64(mem::size_of::<&SupportRecord>())),
                    &mut receipt.counters,
                )
                .map_err(FailureKind::TemporaryAllocationExhausted)?;
            admitted_supports.insert(support.support_occurrence_ref.clone(), support);
            self.validate_support_after(support, &withdrawn_rows, &admitted_rows)?;
        }
        let projected_supports = self
            .supports
            .len()
            .saturating_sub(withdrawn_supports.len())
            .saturating_add(admitted_supports.len());
        if projected_supports > update.budget.maximum_supports {
            return Err(FailureKind::SupportLimitExceeded);
        }
        for occurrence in &withdrawn_rows {
            if let Some(supports) = self.supports_by_premise.get(occurrence) {
                for support in supports.keys() {
                    receipt.counters.support_entries_read += 1;
                    if !withdrawn_supports.contains(support) {
                        return Err(FailureKind::Contract(
                            ContractError::MissingPremiseOccurrence,
                        ));
                    }
                }
            }
        }

        let mut admitted_points = BTreeMap::new();
        let mut admitted_point_buckets = BTreeMap::new();
        let mut admitted_points_by_partition = BTreeMap::new();
        let mut admitted_range_slot_rows = BTreeMap::new();
        let mut affected_anchors = BTreeSet::new();
        for row in &update.withdraw_rows {
            if !self.contract.is_dependency(&row.input_ref) {
                receipt.counters.dependency_misses += 1;
                continue;
            }
            if row.input_ref == *self.point_input_ref() {
                let point = self
                    .points
                    .get(&row.occurrence_ref)
                    .ok_or(FailureKind::InternalInvariant)?;
                self.record_anchors_for_point(point, &mut affected_anchors, tracker, receipt)?;
            }
            if self.range_slots_for_input(&row.input_ref).next().is_some() {
                let anchor_bytes = self
                    .anchor_retained_bytes_for_row(row)
                    .map_err(FailureKind::Contract)?;
                tracker
                    .reserve(anchor_bytes, &mut receipt.counters)
                    .map_err(FailureKind::TemporaryAllocationExhausted)?;
                let anchor = self.anchor_for_row(row).map_err(FailureKind::Contract)?;
                insert_temporary_anchor(
                    &mut affected_anchors,
                    &anchor,
                    tracker,
                    &mut receipt.counters,
                )?;
                drop(anchor);
                tracker.release(anchor_bytes)?;
            }
        }
        for row in &update.admit_rows {
            if !self.contract.is_dependency(&row.input_ref) {
                receipt.counters.dependency_misses += 1;
                continue;
            }
            if row.input_ref == *self.point_input_ref() {
                let partition = row
                    .binding(&self.plan.partition_binding_ref)
                    .ok_or(FailureKind::Contract(ContractError::MissingBinding))?;
                let point_bytes = usize_to_u64(mem::size_of::<PointRecord>())
                    .saturating_add(partition.retained_bytes())
                    .saturating_add(
                        usize_to_u64(mem::size_of::<BucketKey>())
                            .saturating_add(partition.retained_bytes()),
                    );
                tracker
                    .reserve(
                        row.occurrence_ref
                            .retained_bytes()
                            .saturating_add(point_bytes),
                        &mut receipt.counters,
                    )
                    .map_err(FailureKind::TemporaryAllocationExhausted)?;
                let point = self.point_record(row).map_err(FailureKind::Contract)?;
                self.record_anchors_for_point(&point, &mut affected_anchors, tracker, receipt)?;
                insert_temporary_bucket_occurrence(
                    &mut admitted_point_buckets,
                    &point.bucket,
                    &row.occurrence_ref,
                    tracker,
                    &mut receipt.counters,
                )?;
                insert_temporary_partition_occurrence(
                    &mut admitted_points_by_partition,
                    &point.partition,
                    &row.occurrence_ref,
                    tracker,
                    &mut receipt.counters,
                )?;
                admitted_points.insert(row.occurrence_ref.clone(), point);
            }
            if self.range_slots_for_input(&row.input_ref).next().is_some() {
                let anchor_bytes = self
                    .anchor_retained_bytes_for_row(row)
                    .map_err(FailureKind::Contract)?;
                tracker
                    .reserve(anchor_bytes, &mut receipt.counters)
                    .map_err(FailureKind::TemporaryAllocationExhausted)?;
                let anchor = self.anchor_for_row(row).map_err(FailureKind::Contract)?;
                insert_temporary_anchor(
                    &mut affected_anchors,
                    &anchor,
                    tracker,
                    &mut receipt.counters,
                )?;
                for slot_ref in self.range_slots_for_input(&row.input_ref) {
                    insert_temporary_range_occurrence(
                        &mut admitted_range_slot_rows,
                        slot_ref,
                        &anchor,
                        &row.occurrence_ref,
                        tracker,
                        &mut receipt.counters,
                    )?;
                }
                drop(anchor);
                tracker.release(anchor_bytes)?;
            }
        }

        for support_ref in &withdrawn_supports {
            let anchor = self
                .support_locations
                .get(support_ref)
                .map(|location| &location.anchor)
                .ok_or(FailureKind::InternalInvariant)?;
            insert_temporary_anchor(
                &mut affected_anchors,
                anchor,
                tracker,
                &mut receipt.counters,
            )?;
        }
        let mut admitted_support_locations = BTreeMap::new();
        let mut admitted_supports_by_anchor = BTreeMap::new();
        for support in admitted_supports.values() {
            let location_shape = self
                .support_location_shape_with(support, |occurrence| {
                    self.row_after(occurrence, &withdrawn_rows, &admitted_rows)
                })
                .map_err(FailureKind::Contract)?;
            tracker
                .reserve(
                    support
                        .support_occurrence_ref
                        .retained_bytes()
                        .saturating_add(location_shape.retained_bytes()),
                    &mut receipt.counters,
                )
                .map_err(FailureKind::TemporaryAllocationExhausted)?;
            let location = self.support_location_after(support, &withdrawn_rows, &admitted_rows)?;
            insert_temporary_anchor(
                &mut affected_anchors,
                &location.anchor,
                tracker,
                &mut receipt.counters,
            )?;
            insert_temporary_anchor_occurrence(
                &mut admitted_supports_by_anchor,
                &location.anchor,
                &support.support_occurrence_ref,
                tracker,
                &mut receipt.counters,
            )?;
            admitted_support_locations.insert(support.support_occurrence_ref.clone(), location);
        }

        let mut visibility_before = BTreeMap::new();
        for support_ref in &withdrawn_supports {
            if let Some(support) = self.supports.get(support_ref) {
                self.record_visibility_before(
                    &support.output,
                    &mut visibility_before,
                    tracker,
                    &mut receipt.counters,
                )?;
            }
        }
        for support in admitted_supports.values() {
            self.record_visibility_before(
                &support.output,
                &mut visibility_before,
                tracker,
                &mut receipt.counters,
            )?;
        }
        for anchor in &affected_anchors {
            if let Some(support_refs) = self.visible.visible_supports_for_anchor(anchor) {
                for support_ref in support_refs {
                    receipt.counters.support_entries_read += 1;
                    if let Some(support) = self.supports.get(support_ref) {
                        self.record_visibility_before(
                            &support.output,
                            &mut visibility_before,
                            tracker,
                            &mut receipt.counters,
                        )?;
                    }
                }
            }
        }

        let mut admitted_supports_by_tuple = BTreeMap::<Vec<OpaqueRef>, BTreeSet<OpaqueRef>>::new();
        for support in admitted_supports.values() {
            let mut bytes = support.support_occurrence_ref.retained_bytes();
            if !admitted_supports_by_tuple.contains_key(&support.premise_occurrence_refs) {
                bytes = bytes
                    .saturating_add(range_key_retained_bytes(&support.premise_occurrence_refs));
            }
            tracker
                .reserve(bytes, &mut receipt.counters)
                .map_err(FailureKind::TemporaryAllocationExhausted)?;
            admitted_supports_by_tuple
                .entry(support.premise_occurrence_refs.clone())
                .or_default()
                .insert(support.support_occurrence_ref.clone());
        }

        let mut prepared_anchors = BTreeMap::new();
        for anchor in &affected_anchors {
            let prepared = self.prepare_anchor_after(
                anchor,
                &withdrawn_rows,
                &admitted_rows,
                &admitted_range_slot_rows,
                &admitted_points,
                &admitted_point_buckets,
                &admitted_points_by_partition,
                &withdrawn_supports,
                &admitted_supports_by_anchor,
                &admitted_supports_by_tuple,
                update.budget,
                tracker,
                receipt,
            )?;
            tracker
                .reserve(
                    anchor_retained_bytes(anchor)
                        .saturating_add(usize_to_u64(mem::size_of::<Option<PreparedAnchor>>())),
                    &mut receipt.counters,
                )
                .map_err(FailureKind::TemporaryAllocationExhausted)?;
            prepared_anchors.insert(anchor.clone(), prepared);
        }

        let removed_cost =
            self.removed_retained_cost(update, &affected_anchors, &mut receipt.counters)?;
        let successor_state_bytes = update.admitted_delta.result_state_binding_retained_bytes();
        tracker
            .reserve(successor_state_bytes, &mut receipt.counters)
            .map_err(FailureKind::TemporaryAllocationExhausted)?;
        let successor_state = update.admitted_delta.result_state_binding();
        tracker.promote(successor_state_bytes)?;
        let added_cost = self.added_retained_cost(
            update,
            &successor_state,
            &admitted_supports,
            &admitted_support_locations,
            &prepared_anchors,
            &mut receipt.counters,
        )?;
        let remaining_retained_bytes = added_cost
            .bytes
            .checked_sub(tracker.promoted_retained_bytes())
            .ok_or(FailureKind::InternalInvariant)?;
        tracker.reserve_retained(remaining_retained_bytes, &mut receipt.counters)?;
        let projected_retained = self.retained.project(
            removed_cost,
            added_cost,
            update.budget,
            &mut receipt.counters,
        )?;
        let temporary_bytes = tracker.current_temporary_bytes();

        Ok(PreparedUpdate {
            affected_anchors,
            prepared_anchors,
            visibility_before,
            admitted_support_locations,
            successor_state,
            projected_retained,
            temporary_bytes,
        })
    }

    fn validate_support_after(
        &self,
        support: &SupportRecord,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &BTreeMap<OpaqueRef, &InputRow>,
    ) -> Result<(), FailureKind> {
        if support.premise_occurrence_refs.len() != self.contract.premise_slots.len() {
            return Err(FailureKind::Contract(ContractError::PremiseSlotMismatch));
        }
        for (slot, occurrence) in self
            .contract
            .premise_slots
            .iter()
            .zip(&support.premise_occurrence_refs)
        {
            let row = self
                .row_after(occurrence, withdrawn_rows, admitted_rows)
                .ok_or(FailureKind::Contract(
                    ContractError::MissingPremiseOccurrence,
                ))?;
            if row.input_ref != slot.input_ref {
                return Err(FailureKind::Contract(ContractError::PremiseInputMismatch));
            }
        }
        Ok(())
    }

    fn row_after<'a>(
        &'a self,
        occurrence: &OpaqueRef,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &'a BTreeMap<OpaqueRef, &InputRow>,
    ) -> Option<&'a InputRow> {
        if withdrawn_rows.contains(occurrence) {
            None
        } else {
            admitted_rows
                .get(occurrence)
                .copied()
                .or_else(|| self.rows.get(occurrence))
        }
    }

    fn record_anchors_for_point(
        &self,
        point: &PointRecord,
        anchors: &mut BTreeSet<AnchorKey>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        receipt.counters.index_bucket_probes += 1;
        if let Some(indexed) = self.range_buckets.get(&point.bucket) {
            for anchor in indexed {
                insert_temporary_anchor(anchors, anchor, tracker, &mut receipt.counters)?;
            }
        }
        if let Some(fallback) = self.partition_fallback_ranges.get(&point.partition) {
            receipt.counters.fallback_point_visits = receipt
                .counters
                .fallback_point_visits
                .saturating_add(usize_to_u64(fallback.len()));
            for anchor in fallback {
                insert_temporary_anchor(anchors, anchor, tracker, &mut receipt.counters)?;
            }
        }
        Ok(())
    }

    fn record_visibility_before(
        &self,
        output: &OpaqueRef,
        visibility: &mut BTreeMap<OpaqueRef, bool>,
        tracker: &mut AllocationTracker,
        counters: &mut WorkCounters,
    ) -> Result<(), FailureKind> {
        if visibility.contains_key(output) {
            return Ok(());
        }
        tracker
            .reserve(
                output
                    .retained_bytes()
                    .saturating_add(usize_to_u64(mem::size_of::<bool>())),
                counters,
            )
            .map_err(FailureKind::TemporaryAllocationExhausted)?;
        visibility.insert(output.clone(), self.visible.output_visibility(output));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_anchor_after(
        &self,
        anchor: &AnchorKey,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &BTreeMap<OpaqueRef, &InputRow>,
        admitted_range_slot_rows: &BTreeMap<OpaqueRef, BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>>,
        admitted_points: &BTreeMap<OpaqueRef, PointRecord>,
        admitted_point_buckets: &BTreeMap<BucketKey, BTreeSet<OpaqueRef>>,
        admitted_points_by_partition: &BTreeMap<OpaqueValue, BTreeSet<OpaqueRef>>,
        withdrawn_supports: &BTreeSet<OpaqueRef>,
        admitted_supports_by_anchor: &BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>,
        admitted_supports_by_tuple: &BTreeMap<Vec<OpaqueRef>, BTreeSet<OpaqueRef>>,
        budget: PhysicalBudget,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<Option<PreparedAnchor>, FailureKind> {
        match self.join_environments_after(
            anchor,
            withdrawn_rows,
            admitted_rows,
            admitted_range_slot_rows,
            budget,
            tracker,
            receipt,
        )? {
            RangePreparation::Absent => {
                self.require_exact_supports_after(
                    anchor,
                    &BTreeSet::new(),
                    withdrawn_supports,
                    admitted_supports_by_anchor,
                    receipt,
                )?;
                Ok(None)
            }
            RangePreparation::Ready(batch) => {
                let PromotedEnvironments {
                    environments,
                    vector_bytes,
                } = batch.into_promoted(tracker)?;
                let mut ranges = BTreeMap::new();
                let mut visible_supports = BTreeSet::new();
                for environment in environments {
                    let range = self.range_record(anchor, environment, budget, tracker, receipt)?;
                    self.visible_supports_after(
                        &range,
                        withdrawn_rows,
                        admitted_rows,
                        admitted_points,
                        admitted_point_buckets,
                        admitted_points_by_partition,
                        withdrawn_supports,
                        admitted_supports_by_tuple,
                        &mut visible_supports,
                        tracker,
                        receipt,
                    )?;
                    let range_key_bytes = range_key_retained_bytes(&range.key);
                    tracker
                        .reserve(range_key_bytes, &mut receipt.counters)
                        .map_err(FailureKind::TemporaryAllocationExhausted)?;
                    if ranges.insert(range.key.clone(), range).is_some() {
                        return Err(FailureKind::InternalInvariant);
                    }
                    tracker.promote(range_key_bytes)?;
                }
                tracker.release(vector_bytes)?;
                self.require_exact_supports_after(
                    anchor,
                    &visible_supports,
                    withdrawn_supports,
                    admitted_supports_by_anchor,
                    receipt,
                )?;
                Ok(Some(Self::prepared_anchor(
                    ranges,
                    visible_supports,
                    tracker,
                    receipt,
                )?))
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the delta-local indexes and accounting remain separate semantic inputs"
    )]
    fn join_environments_after(
        &self,
        anchor: &AnchorKey,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &BTreeMap<OpaqueRef, &InputRow>,
        admitted_range_slot_rows: &BTreeMap<OpaqueRef, BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>>,
        budget: PhysicalBudget,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<RangePreparation, FailureKind> {
        let mut batch = EnvironmentBatch::initial(tracker, &mut receipt.counters)?;
        for slot_ref in &self.plan.range_slot_refs {
            let mut occurrences = BTreeSet::new();
            let mut occurrence_bytes = 0_u64;
            if let Some(current) = self
                .range_slot_rows
                .get(slot_ref)
                .and_then(|by_anchor| by_anchor.get(anchor))
            {
                for occurrence in current {
                    if !withdrawn_rows.contains(occurrence)
                        && insert_temporary_ref(
                            &mut occurrences,
                            occurrence,
                            tracker,
                            &mut receipt.counters,
                        )?
                    {
                        occurrence_bytes =
                            occurrence_bytes.saturating_add(occurrence.retained_bytes());
                    }
                }
            }
            if let Some(admitted) = admitted_range_slot_rows
                .get(slot_ref)
                .and_then(|by_anchor| by_anchor.get(anchor))
            {
                for occurrence in admitted {
                    if insert_temporary_ref(
                        &mut occurrences,
                        occurrence,
                        tracker,
                        &mut receipt.counters,
                    )? {
                        occurrence_bytes =
                            occurrence_bytes.saturating_add(occurrence.retained_bytes());
                    }
                }
            }
            if occurrences.is_empty() {
                tracker.release(occurrence_bytes.saturating_add(batch.temporary_bytes))?;
                return Ok(RangePreparation::Absent);
            }
            let occurrence_iter = occurrences.iter();
            let joined = Self::join_environment_slot(
                &batch,
                slot_ref,
                &occurrence_iter,
                budget,
                tracker,
                receipt,
                |occurrence| self.row_after(occurrence, withdrawn_rows, admitted_rows),
            );
            tracker.release(occurrence_bytes.saturating_add(batch.temporary_bytes))?;
            match joined {
                Ok(next) if next.environments.is_empty() => {
                    tracker.release(next.temporary_bytes)?;
                    return Ok(RangePreparation::Absent);
                }
                Ok(next) => batch = next,
                Err(kind) => return Err(kind),
            }
        }
        Ok(RangePreparation::Ready(batch))
    }

    #[allow(clippy::too_many_arguments)]
    fn visible_supports_after(
        &self,
        range: &RangeRecord,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &BTreeMap<OpaqueRef, &InputRow>,
        admitted_points: &BTreeMap<OpaqueRef, PointRecord>,
        admitted_point_buckets: &BTreeMap<BucketKey, BTreeSet<OpaqueRef>>,
        admitted_points_by_partition: &BTreeMap<OpaqueValue, BTreeSet<OpaqueRef>>,
        withdrawn_supports: &BTreeSet<OpaqueRef>,
        admitted_supports_by_tuple: &BTreeMap<Vec<OpaqueRef>, BTreeSet<OpaqueRef>>,
        visible: &mut BTreeSet<OpaqueRef>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        match &range.coverage {
            Coverage::Buckets(buckets) => {
                for bucket in buckets {
                    receipt.counters.index_bucket_probes += 1;
                    if let Some(occurrences) = self.point_buckets.get(bucket) {
                        for occurrence in occurrences {
                            if !withdrawn_rows.contains(occurrence) {
                                self.visible_supports_for_point_after(
                                    range,
                                    occurrence,
                                    withdrawn_rows,
                                    admitted_rows,
                                    withdrawn_supports,
                                    admitted_supports_by_tuple,
                                    visible,
                                    tracker,
                                    receipt,
                                )?;
                            }
                        }
                    }
                    if let Some(occurrences) = admitted_point_buckets.get(bucket) {
                        for occurrence in occurrences {
                            self.visible_supports_for_point_after(
                                range,
                                occurrence,
                                withdrawn_rows,
                                admitted_rows,
                                withdrawn_supports,
                                admitted_supports_by_tuple,
                                visible,
                                tracker,
                                receipt,
                            )?;
                        }
                    }
                }
            }
            Coverage::PartitionScan => {
                if let Some(occurrences) = self.points_by_partition.get(&range.geometry.partition) {
                    receipt.counters.fallback_point_visits = receipt
                        .counters
                        .fallback_point_visits
                        .saturating_add(usize_to_u64(occurrences.len()));
                    for occurrence in occurrences {
                        if !withdrawn_rows.contains(occurrence)
                            && self
                                .points
                                .get(occurrence)
                                .is_some_and(|point| range.geometry.contains(point))
                        {
                            self.visible_supports_for_point_after(
                                range,
                                occurrence,
                                withdrawn_rows,
                                admitted_rows,
                                withdrawn_supports,
                                admitted_supports_by_tuple,
                                visible,
                                tracker,
                                receipt,
                            )?;
                        }
                    }
                }
                if let Some(occurrences) =
                    admitted_points_by_partition.get(&range.geometry.partition)
                {
                    receipt.counters.fallback_point_visits = receipt
                        .counters
                        .fallback_point_visits
                        .saturating_add(usize_to_u64(occurrences.len()));
                    for occurrence in occurrences {
                        if admitted_points
                            .get(occurrence)
                            .is_some_and(|point| range.geometry.contains(point))
                        {
                            self.visible_supports_for_point_after(
                                range,
                                occurrence,
                                withdrawn_rows,
                                admitted_rows,
                                withdrawn_supports,
                                admitted_supports_by_tuple,
                                visible,
                                tracker,
                                receipt,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn visible_supports_for_point_after(
        &self,
        range: &RangeRecord,
        point_occurrence: &OpaqueRef,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &BTreeMap<OpaqueRef, &InputRow>,
        withdrawn_supports: &BTreeSet<OpaqueRef>,
        admitted_supports_by_tuple: &BTreeMap<Vec<OpaqueRef>, BTreeSet<OpaqueRef>>,
        visible: &mut BTreeSet<OpaqueRef>,
        tracker: &mut AllocationTracker,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        receipt.counters.candidate_bindings += 1;
        let point_row = self
            .row_after(point_occurrence, withdrawn_rows, admitted_rows)
            .ok_or(FailureKind::InternalInvariant)?;
        let Some((tuple, tuple_bytes)) = range.environment.premise_tuple_with_point(
            &self.plan.point_slot_ref,
            point_row,
            &self.contract,
            tracker,
            &mut receipt.counters,
        )?
        else {
            return Ok(());
        };
        receipt.counters.premise_occurrences_visited = receipt
            .counters
            .premise_occurrences_visited
            .saturating_add(usize_to_u64(tuple.len()));
        if let Some(supports) = self.supports_by_tuple.get(&tuple) {
            for support in supports {
                if !withdrawn_supports.contains(support) {
                    receipt.counters.support_entries_read += 1;
                    if !visible.contains(support) {
                        tracker
                            .reserve(support.retained_bytes(), &mut receipt.counters)
                            .map_err(FailureKind::TemporaryAllocationExhausted)?;
                        visible.insert(support.clone());
                    }
                }
            }
        }
        if let Some(supports) = admitted_supports_by_tuple.get(&tuple) {
            receipt.counters.support_entries_read = receipt
                .counters
                .support_entries_read
                .saturating_add(usize_to_u64(supports.len()));
            for support in supports {
                if !visible.contains(support) {
                    tracker
                        .reserve(support.retained_bytes(), &mut receipt.counters)
                        .map_err(FailureKind::TemporaryAllocationExhausted)?;
                    visible.insert(support.clone());
                }
            }
        }
        tracker.release(tuple_bytes)?;
        Ok(())
    }

    fn require_exact_supports_after(
        &self,
        anchor: &AnchorKey,
        visible: &BTreeSet<OpaqueRef>,
        withdrawn_supports: &BTreeSet<OpaqueRef>,
        admitted_supports_by_anchor: &BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>,
        receipt: &mut OperationReceipt,
    ) -> Result<(), FailureKind> {
        let mut expected_count = 0_usize;
        if let Some(supports) = self.supports_by_anchor.get(anchor) {
            for support in supports {
                receipt.counters.support_entries_read += 1;
                if withdrawn_supports.contains(support) {
                    continue;
                }
                expected_count += 1;
                if !visible.contains(support) {
                    return Err(FailureKind::Contract(
                        ContractError::SupportOutsidePhysicalSuperset,
                    ));
                }
            }
        }
        if let Some(supports) = admitted_supports_by_anchor.get(anchor) {
            for support in supports {
                receipt.counters.support_entries_read += 1;
                expected_count += 1;
                if !visible.contains(support) {
                    return Err(FailureKind::Contract(
                        ContractError::SupportOutsidePhysicalSuperset,
                    ));
                }
            }
        }
        if visible.len() != expected_count {
            return Err(FailureKind::Contract(
                ContractError::SupportOutsidePhysicalSuperset,
            ));
        }
        Ok(())
    }

    fn support_location_after(
        &self,
        support: &SupportRecord,
        withdrawn_rows: &BTreeSet<OpaqueRef>,
        admitted_rows: &BTreeMap<OpaqueRef, &InputRow>,
    ) -> Result<SupportLocation, FailureKind> {
        let mut anchor = None;
        let mut range = Vec::with_capacity(self.plan.range_slot_refs.len());
        for slot_ref in &self.plan.range_slot_refs {
            let position = self
                .contract
                .premise_slots
                .iter()
                .position(|slot| slot.slot_ref == *slot_ref)
                .ok_or(FailureKind::InternalInvariant)?;
            let occurrence = support
                .premise_occurrence_refs
                .get(position)
                .ok_or(FailureKind::InternalInvariant)?;
            let row = self
                .row_after(occurrence, withdrawn_rows, admitted_rows)
                .ok_or(FailureKind::Contract(
                    ContractError::MissingPremiseOccurrence,
                ))?;
            let row_anchor = self.anchor_for_row(row).map_err(FailureKind::Contract)?;
            if anchor
                .as_ref()
                .is_some_and(|expected| expected != &row_anchor)
            {
                return Err(FailureKind::Contract(
                    ContractError::InconsistentAnchorBindings,
                ));
            }
            anchor = Some(row_anchor);
            range.push(occurrence.clone());
        }
        Ok(SupportLocation {
            anchor: anchor.ok_or(FailureKind::InternalInvariant)?,
            range,
        })
    }

    fn commit_update(
        &mut self,
        update: MaterializationUpdate,
        prepared: PreparedUpdate,
        counters: &mut WorkCounters,
    ) {
        let PreparedUpdate {
            affected_anchors,
            prepared_anchors,
            visibility_before,
            mut admitted_support_locations,
            successor_state,
            projected_retained,
            temporary_bytes: _,
        } = prepared;
        for anchor in &affected_anchors {
            self.visible.detach_anchor(anchor, &self.supports, counters);
            self.remove_range_indexes(anchor, counters);
        }

        for row in &update.withdraw_rows {
            if self.contract.is_dependency(&row.input_ref) {
                self.unindex_row(row, counters);
            }
            self.rows.remove(&row.occurrence_ref);
        }
        for support_ref in &update.withdraw_support_occurrence_refs {
            self.unindex_support_record(support_ref, counters);
        }

        for row in update.admit_rows {
            if self.contract.is_dependency(&row.input_ref) {
                let indexed = self.index_row(&row, counters);
                debug_assert!(indexed.is_ok());
            }
            self.rows.insert(row.occurrence_ref.clone(), row);
        }
        for support in update.admit_supports {
            let location = admitted_support_locations
                .remove(&support.support_occurrence_ref)
                .expect("prepared support location must remain exact");
            self.index_support_record_at(support, location, counters);
        }

        for (anchor, prepared_anchor) in prepared_anchors {
            if let Some(prepared_anchor) = prepared_anchor {
                self.install_prepared_anchor(anchor, prepared_anchor, counters);
            }
        }

        for (output, was_visible) in visibility_before {
            match (was_visible, self.visible.output_visibility(&output)) {
                (false, true) => counters.view_admits += 1,
                (true, false) => counters.view_withdraws += 1,
                (false, false) | (true, true) => {}
            }
        }
        self.snapshot_ref = update.result_snapshot_ref;
        self.admitted_state = successor_state;
        self.retained = projected_retained;
    }
}

#[derive(Debug)]
struct PreparedUpdate {
    affected_anchors: BTreeSet<AnchorKey>,
    prepared_anchors: BTreeMap<AnchorKey, Option<PreparedAnchor>>,
    visibility_before: BTreeMap<OpaqueRef, bool>,
    admitted_support_locations: BTreeMap<OpaqueRef, SupportLocation>,
    successor_state: AdmittedStateBinding,
    projected_retained: RetainedLedger,
    /// Logical ownership consumed atomically at the commit boundary.
    temporary_bytes: u64,
}
