//! Opaque caller-owned inputs and checked physical contracts.

use std::fmt;
use std::mem;

/// An opaque caller-owned reference with deterministic physical byte ordering.
///
/// Its bytes are an adapter token, not a definition of Clause identity,
/// equality, authority, or canonical serialization.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpaqueRef(Box<[u8]>);

impl OpaqueRef {
    #[must_use]
    pub fn new(bytes: impl Into<Box<[u8]>>) -> Self {
        Self(bytes.into())
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "used by the sealed package-admission bridge")
    )]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>()).saturating_add(usize_to_u64(self.0.len()))
    }
}

impl From<&str> for OpaqueRef {
    fn from(value: &str) -> Self {
        Self::new(value.as_bytes())
    }
}

impl From<String> for OpaqueRef {
    fn from(value: String) -> Self {
        Self::new(value.into_bytes())
    }
}

impl From<Vec<u8>> for OpaqueRef {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

/// Opaque physical bytes under an exact caller-owned representation contract.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OpaqueValue {
    pub representation_contract_ref: OpaqueRef,
    pub bytes: Box<[u8]>,
}

impl OpaqueValue {
    #[must_use]
    pub fn new(representation_contract_ref: OpaqueRef, bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            representation_contract_ref,
            bytes: bytes.into(),
        }
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.representation_contract_ref.retained_bytes())
            .saturating_add(usize_to_u64(self.bytes.len()))
    }
}

/// One exact opaque binding carried by an admitted input occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub binding_ref: OpaqueRef,
    pub value: OpaqueValue,
}

impl Binding {
    pub(crate) fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.binding_ref.retained_bytes())
            .saturating_add(self.value.retained_bytes())
    }
}

/// One admitted, independently identified input occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputRow {
    pub input_ref: OpaqueRef,
    pub occurrence_ref: OpaqueRef,
    pub bindings: Vec<Binding>,
}

impl InputRow {
    #[must_use]
    pub fn binding(&self, binding_ref: &OpaqueRef) -> Option<&OpaqueValue> {
        self.bindings
            .iter()
            .find(|binding| binding.binding_ref == *binding_ref)
            .map(|binding| &binding.value)
    }

    pub(crate) fn validate(&self) -> Result<(), ContractError> {
        for (position, binding) in self.bindings.iter().enumerate() {
            if self.bindings[..position]
                .iter()
                .any(|prior| prior.binding_ref == binding.binding_ref)
            {
                return Err(ContractError::DuplicateBindingRef);
            }
        }
        Ok(())
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        self.bindings.iter().fold(
            usize_to_u64(mem::size_of::<Self>())
                .saturating_add(self.input_ref.retained_bytes())
                .saturating_add(self.occurrence_ref.retained_bytes()),
            |total, binding| total.saturating_add(binding.retained_bytes()),
        )
    }
}

/// One occurrence-bearing premise slot in exact contract order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiseSlot {
    pub slot_ref: OpaqueRef,
    pub input_ref: OpaqueRef,
}

impl PremiseSlot {
    pub(crate) fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.slot_ref.retained_bytes())
            .saturating_add(self.input_ref.retained_bytes())
    }
}

/// One caller-bound support occurrence.
///
/// Premise occurrences are in exact contract slot order. Equal support content
/// remains distinct when `support_occurrence_ref` differs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportRecord {
    pub support_occurrence_ref: OpaqueRef,
    pub premise_occurrence_refs: Vec<OpaqueRef>,
    pub output: OpaqueRef,
    pub evidence_ref: OpaqueRef,
}

impl SupportRecord {
    pub(crate) fn retained_bytes(&self) -> u64 {
        let premise_bytes = self
            .premise_occurrence_refs
            .iter()
            .fold(0_u64, |total, occurrence| {
                total.saturating_add(occurrence.retained_bytes())
            });
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.support_occurrence_ref.retained_bytes())
            .saturating_add(premise_bytes)
            .saturating_add(self.output.retained_bytes())
            .saturating_add(self.evidence_ref.retained_bytes())
    }
}

/// The checked semantic-to-physical shape supplied by the caller.
///
/// The crate validates only the invariants required by its physical schedules.
/// Possession of this value grants no semantic or admission authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializationContract {
    pub graph_ref: OpaqueRef,
    pub contract_ref: OpaqueRef,
    pub premise_slots: Vec<PremiseSlot>,
}

impl MaterializationContract {
    /// Validate the physical shape required by materialization schedules.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError`] when premise slots are empty or repeated.
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.premise_slots.is_empty() {
            return Err(ContractError::EmptyPremiseSlots);
        }
        for (position, slot) in self.premise_slots.iter().enumerate() {
            if self.premise_slots[..position]
                .iter()
                .any(|prior| prior.slot_ref == slot.slot_ref)
            {
                return Err(ContractError::DuplicatePremiseSlot);
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn is_dependency(&self, input_ref: &OpaqueRef) -> bool {
        self.premise_slots
            .iter()
            .any(|slot| slot.input_ref == *input_ref)
    }

    pub(crate) fn slot(&self, slot_ref: &OpaqueRef) -> Option<&PremiseSlot> {
        self.premise_slots
            .iter()
            .find(|slot| slot.slot_ref == *slot_ref)
    }

    pub(crate) fn validate_snapshot(
        &self,
        snapshot: &AdmittedSnapshot,
    ) -> Result<(), ContractError> {
        self.validate()?;
        if snapshot.admitted_contract.graph_ref != self.graph_ref {
            return Err(ContractError::GraphIdentityMismatch);
        }
        if snapshot.admitted_contract != *self {
            return Err(ContractError::ContractIdentityMismatch);
        }
        snapshot
            .admitted_state
            .validate_snapshot_ref(&snapshot.snapshot_ref)?;
        validate_rows(&snapshot.rows)?;
        validate_supports(self, &snapshot.rows, &snapshot.supports)
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        self.premise_slots.iter().fold(
            usize_to_u64(mem::size_of::<Self>())
                .saturating_add(self.graph_ref.retained_bytes())
                .saturating_add(self.contract_ref.retained_bytes()),
            |total, slot| total.saturating_add(slot.retained_bytes()),
        )
    }
}

/// One exact big-endian signed-i32 physical binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct I32Binding {
    pub binding_ref: OpaqueRef,
    pub representation_contract_ref: OpaqueRef,
}

impl I32Binding {
    pub(crate) fn read(&self, row: &InputRow) -> Result<i32, ContractError> {
        let value = row
            .binding(&self.binding_ref)
            .ok_or(ContractError::MissingBinding)?;
        self.decode(value)
    }

    pub(crate) fn decode(&self, value: &OpaqueValue) -> Result<i32, ContractError> {
        if value.representation_contract_ref != self.representation_contract_ref
            || value.bytes.len() != 4
        {
            return Err(ContractError::ScalarContractMismatch);
        }
        let bytes: [u8; 4] = value
            .bytes
            .as_ref()
            .try_into()
            .map_err(|_| ContractError::ScalarContractMismatch)?;
        Ok(i32::from_be_bytes(bytes))
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.binding_ref.retained_bytes())
            .saturating_add(self.representation_contract_ref.retained_bytes())
    }
}

/// An exact cold-scan schedule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanPlan {
    pub graph_ref: OpaqueRef,
    pub contract_ref: OpaqueRef,
    pub plan_ref: OpaqueRef,
    pub premise_slot_order: Vec<OpaqueRef>,
}

impl ScanPlan {
    pub(crate) fn validate(&self, contract: &MaterializationContract) -> Result<(), ContractError> {
        contract.validate()?;
        validate_plan_identity(contract, &self.graph_ref, &self.contract_ref)?;
        if !self
            .premise_slot_order
            .iter()
            .eq(contract.premise_slots.iter().map(|slot| &slot.slot_ref))
        {
            return Err(ContractError::PremiseSlotMismatch);
        }
        Ok(())
    }
}

/// A dual-uniform-grid physical schedule over opaque bindings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridPlan {
    pub graph_ref: OpaqueRef,
    pub contract_ref: OpaqueRef,
    pub plan_ref: OpaqueRef,
    pub point_slot_ref: OpaqueRef,
    pub range_slot_refs: Vec<OpaqueRef>,
    pub anchor_binding_refs: Vec<OpaqueRef>,
    pub partition_binding_ref: OpaqueRef,
    pub point_x: I32Binding,
    pub point_y: I32Binding,
    pub center_x: I32Binding,
    pub center_y: I32Binding,
    pub extent: I32Binding,
    pub bucket_width: i64,
    pub maximum_buckets_per_range: u64,
}

impl GridPlan {
    pub(crate) fn validate(&self, contract: &MaterializationContract) -> Result<(), ContractError> {
        contract.validate()?;
        validate_plan_identity(contract, &self.graph_ref, &self.contract_ref)?;
        if self.bucket_width <= 0 || self.maximum_buckets_per_range == 0 {
            return Err(ContractError::InvalidPhysicalParameter);
        }
        if contract.slot(&self.point_slot_ref).is_none() || self.range_slot_refs.is_empty() {
            return Err(ContractError::PremiseSlotMismatch);
        }
        if self.range_slot_refs.len().saturating_add(1) != contract.premise_slots.len()
            || contract.premise_slots.iter().any(|slot| {
                usize::from(slot.slot_ref == self.point_slot_ref)
                    + self
                        .range_slot_refs
                        .iter()
                        .filter(|range_slot| **range_slot == slot.slot_ref)
                        .count()
                    != 1
            })
        {
            return Err(ContractError::PremiseSlotMismatch);
        }
        if !self
            .anchor_binding_refs
            .iter()
            .any(|binding| binding == &self.partition_binding_ref)
            || self
                .anchor_binding_refs
                .iter()
                .enumerate()
                .any(|(position, binding)| {
                    self.anchor_binding_refs[..position]
                        .iter()
                        .any(|prior| prior == binding)
                })
        {
            return Err(ContractError::InvalidAnchorBindings);
        }
        Ok(())
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        let range_slots = self.range_slot_refs.iter().fold(0_u64, |total, slot| {
            total.saturating_add(slot.retained_bytes())
        });
        let anchors = self
            .anchor_binding_refs
            .iter()
            .fold(0_u64, |total, binding| {
                total.saturating_add(binding.retained_bytes())
            });
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.graph_ref.retained_bytes())
            .saturating_add(self.contract_ref.retained_bytes())
            .saturating_add(self.plan_ref.retained_bytes())
            .saturating_add(self.point_slot_ref.retained_bytes())
            .saturating_add(range_slots)
            .saturating_add(anchors)
            .saturating_add(self.partition_binding_ref.retained_bytes())
            .saturating_add(self.point_x.retained_bytes())
            .saturating_add(self.point_y.retained_bytes())
            .saturating_add(self.center_x.retained_bytes())
            .saturating_add(self.center_y.retained_bytes())
            .saturating_add(self.extent.retained_bytes())
    }
}

/// Opaque pins for one already-admitted State boundary.
///
/// Construction is available only through [`AdmittedSnapshot::from_admission`]
/// after a [`SnapshotAdmission`] adapter accepts the exact payload. Opaque
/// bytes alone never acquire authority at this physical boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedStateBinding {
    clause_semantics: OpaqueRef,
    program_revision: OpaqueRef,
    runtime_session: OpaqueRef,
    state_revision: OpaqueRef,
}

/// Borrowed exact identities supplied by the semantic admission authority.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) struct StateAdmissionPins<'a> {
    pub clause_semantics: &'a OpaqueRef,
    pub program_revision: &'a OpaqueRef,
    pub runtime_session: &'a OpaqueRef,
    pub state_revision: &'a OpaqueRef,
}

/// Borrowed physical payload whose semantic admission must be established.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) struct SnapshotPayload<'a> {
    pub contract: &'a MaterializationContract,
    pub snapshot_ref: &'a OpaqueRef,
    pub rows: &'a [InputRow],
    pub supports: &'a [SupportRecord],
}

/// Semantic authority adapter for one exact admitted State payload.
///
/// `clause-materialization` deliberately cannot establish this authority. The
/// pending `clause-package` integration must replace this crate-local
/// bridge with package-owned checked inputs bound to canonical State content.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) trait SnapshotAdmission {
    fn state_pins(&self) -> StateAdmissionPins<'_>;

    fn admits_snapshot(&self, payload: SnapshotPayload<'_>) -> bool;
}

impl AdmittedStateBinding {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "used by the sealed package-admission bridge")
    )]
    pub(crate) fn checked(pins: StateAdmissionPins<'_>) -> Result<Self, ContractError> {
        if [
            pins.clause_semantics,
            pins.program_revision,
            pins.runtime_session,
            pins.state_revision,
        ]
        .into_iter()
        .any(OpaqueRef::is_empty)
        {
            return Err(ContractError::EmptyAdmissionPin);
        }
        Ok(Self {
            clause_semantics: pins.clause_semantics.clone(),
            program_revision: pins.program_revision.clone(),
            runtime_session: pins.runtime_session.clone(),
            state_revision: pins.state_revision.clone(),
        })
    }

    pub(crate) fn validate_snapshot_ref(
        &self,
        snapshot_ref: &OpaqueRef,
    ) -> Result<(), ContractError> {
        if &self.state_revision != snapshot_ref {
            return Err(ContractError::AdmittedStateIdentityMismatch);
        }
        Ok(())
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<Self>())
            .saturating_add(self.clause_semantics.retained_bytes())
            .saturating_add(self.program_revision.retained_bytes())
            .saturating_add(self.runtime_session.retained_bytes())
            .saturating_add(self.state_revision.retained_bytes())
    }
}

/// Opaque pins for one checked, already-admitted semantic State delta.
///
/// Construction is available only through
/// [`MaterializationUpdate::from_admission`] after a [`DeltaAdmission`] adapter
/// validates semantic identity, Program, `RuntimeSession`, predecessor/result
/// `StateRevisions`, producing Activation and Step, delta identity, and the
/// exact payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedDeltaBinding {
    clause_semantics: OpaqueRef,
    program_revision: OpaqueRef,
    runtime_session: OpaqueRef,
    predecessor_state_revision: OpaqueRef,
    result_state_revision: OpaqueRef,
    producing_activation: OpaqueRef,
    producing_step: OpaqueRef,
    semantic_delta: OpaqueRef,
}

/// Borrowed exact identities supplied for one admitted State delta.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) struct DeltaAdmissionPins<'a> {
    pub clause_semantics: &'a OpaqueRef,
    pub program_revision: &'a OpaqueRef,
    pub runtime_session: &'a OpaqueRef,
    pub predecessor_state_revision: &'a OpaqueRef,
    pub result_state_revision: &'a OpaqueRef,
    pub producing_activation: &'a OpaqueRef,
    pub producing_step: &'a OpaqueRef,
    pub semantic_delta: &'a OpaqueRef,
}

/// Borrowed physical delta payload whose semantic admission must be checked.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) struct DeltaPayload<'a> {
    pub base_snapshot_ref: &'a OpaqueRef,
    pub result_snapshot_ref: &'a OpaqueRef,
    pub withdraw_rows: &'a [InputRow],
    pub admit_rows: &'a [InputRow],
    pub withdraw_support_occurrence_refs: &'a [OpaqueRef],
    pub admit_supports: &'a [SupportRecord],
}

/// Semantic authority adapter for one exact admitted State delta payload.
/// The pending `clause-package` integration must replace this crate-local
/// bridge with package-owned checked delta inputs.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) trait DeltaAdmission {
    fn delta_pins(&self) -> DeltaAdmissionPins<'_>;

    fn admits_delta(&self, payload: DeltaPayload<'_>) -> bool;
}

impl AdmittedDeltaBinding {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "used by the sealed package-admission bridge")
    )]
    pub(crate) fn checked(pins: DeltaAdmissionPins<'_>) -> Result<Self, ContractError> {
        if [
            pins.clause_semantics,
            pins.program_revision,
            pins.runtime_session,
            pins.predecessor_state_revision,
            pins.result_state_revision,
            pins.producing_activation,
            pins.producing_step,
            pins.semantic_delta,
        ]
        .into_iter()
        .any(OpaqueRef::is_empty)
        {
            return Err(ContractError::EmptyAdmissionPin);
        }
        Ok(Self {
            clause_semantics: pins.clause_semantics.clone(),
            program_revision: pins.program_revision.clone(),
            runtime_session: pins.runtime_session.clone(),
            predecessor_state_revision: pins.predecessor_state_revision.clone(),
            result_state_revision: pins.result_state_revision.clone(),
            producing_activation: pins.producing_activation.clone(),
            producing_step: pins.producing_step.clone(),
            semantic_delta: pins.semantic_delta.clone(),
        })
    }

    fn validate_payload_refs(
        &self,
        base_snapshot_ref: &OpaqueRef,
        result_snapshot_ref: &OpaqueRef,
    ) -> Result<(), ContractError> {
        if &self.predecessor_state_revision != base_snapshot_ref {
            return Err(ContractError::AdmittedDeltaBaseMismatch);
        }
        if &self.result_state_revision != result_snapshot_ref {
            return Err(ContractError::AdmittedDeltaResultMismatch);
        }
        Ok(())
    }

    pub(crate) fn validate_update(
        &self,
        current: &AdmittedStateBinding,
        base_snapshot_ref: &OpaqueRef,
        result_snapshot_ref: &OpaqueRef,
    ) -> Result<(), ContractError> {
        if self.clause_semantics != current.clause_semantics {
            return Err(ContractError::ClauseSemanticsIdentityMismatch);
        }
        if self.program_revision != current.program_revision {
            return Err(ContractError::ProgramRevisionIdentityMismatch);
        }
        if self.runtime_session != current.runtime_session {
            return Err(ContractError::RuntimeSessionIdentityMismatch);
        }
        self.validate_payload_refs(base_snapshot_ref, result_snapshot_ref)
    }

    pub(crate) fn result_state_binding(&self) -> AdmittedStateBinding {
        AdmittedStateBinding {
            clause_semantics: self.clause_semantics.clone(),
            program_revision: self.program_revision.clone(),
            runtime_session: self.runtime_session.clone(),
            state_revision: self.result_state_revision.clone(),
        }
    }

    pub(crate) fn result_state_binding_retained_bytes(&self) -> u64 {
        usize_to_u64(mem::size_of::<AdmittedStateBinding>())
            .saturating_add(self.clause_semantics.retained_bytes())
            .saturating_add(self.program_revision.retained_bytes())
            .saturating_add(self.runtime_session.retained_bytes())
            .saturating_add(self.result_state_revision.retained_bytes())
    }
}

/// One already-admitted caller snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedSnapshot {
    pub(crate) admitted_state: AdmittedStateBinding,
    pub(crate) admitted_contract: MaterializationContract,
    pub(crate) snapshot_ref: OpaqueRef,
    pub(crate) rows: Vec<InputRow>,
    pub(crate) supports: Vec<SupportRecord>,
}

impl AdmittedSnapshot {
    /// Bind an exact physical payload to a caller-established State admission.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError`] when an identity pin is empty or inconsistent,
    /// or when the semantic authority rejects the exact row/support payload.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "awaits the clause-package integration seam")
    )]
    pub(crate) fn from_admission<A: SnapshotAdmission>(
        admission: &A,
        contract: &MaterializationContract,
        snapshot_ref: OpaqueRef,
        rows: Vec<InputRow>,
        supports: Vec<SupportRecord>,
    ) -> Result<Self, ContractError> {
        let admitted_state = AdmittedStateBinding::checked(admission.state_pins())?;
        admitted_state.validate_snapshot_ref(&snapshot_ref)?;
        if !admission.admits_snapshot(SnapshotPayload {
            contract,
            snapshot_ref: &snapshot_ref,
            rows: &rows,
            supports: &supports,
        }) {
            return Err(ContractError::SnapshotPayloadNotAdmitted);
        }
        Ok(Self {
            admitted_state,
            admitted_contract: contract.clone(),
            snapshot_ref,
            rows,
            supports,
        })
    }

    #[must_use]
    pub fn snapshot_ref(&self) -> &OpaqueRef {
        &self.snapshot_ref
    }

    #[must_use]
    pub fn graph_ref(&self) -> &OpaqueRef {
        &self.admitted_contract.graph_ref
    }

    #[must_use]
    pub fn contract_ref(&self) -> &OpaqueRef {
        &self.admitted_contract.contract_ref
    }

    #[must_use]
    pub fn rows(&self) -> &[InputRow] {
        &self.rows
    }

    #[must_use]
    pub fn supports(&self) -> &[SupportRecord] {
        &self.supports
    }
}

/// Candidate physical input presented to a semantic delta authority.
///
/// A successful [`MaterializationUpdate::from_admission`] binds this entire
/// payload. Materialization still never admits the delta or State revision.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "awaits the clause-package integration seam")
)]
pub(crate) struct MaterializationUpdateInput {
    pub graph_ref: OpaqueRef,
    pub contract_ref: OpaqueRef,
    pub plan_ref: OpaqueRef,
    pub base_snapshot_ref: OpaqueRef,
    pub result_snapshot_ref: OpaqueRef,
    pub withdraw_rows: Vec<InputRow>,
    pub admit_rows: Vec<InputRow>,
    pub withdraw_support_occurrence_refs: Vec<OpaqueRef>,
    pub admit_supports: Vec<SupportRecord>,
    pub budget: PhysicalBudget,
}

/// One caller-admitted delta bound to its exact physical input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializationUpdate {
    pub(crate) admitted_delta: AdmittedDeltaBinding,
    pub(crate) graph_ref: OpaqueRef,
    pub(crate) contract_ref: OpaqueRef,
    pub(crate) plan_ref: OpaqueRef,
    pub(crate) base_snapshot_ref: OpaqueRef,
    pub(crate) result_snapshot_ref: OpaqueRef,
    pub(crate) withdraw_rows: Vec<InputRow>,
    pub(crate) admit_rows: Vec<InputRow>,
    pub(crate) withdraw_support_occurrence_refs: Vec<OpaqueRef>,
    pub(crate) admit_supports: Vec<SupportRecord>,
    pub(crate) budget: PhysicalBudget,
}

impl MaterializationUpdate {
    /// Bind an exact physical delta payload to caller-established admission.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError`] when an identity pin is empty or inconsistent,
    /// or when the semantic authority rejects the exact row/support delta.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "awaits the clause-package integration seam")
    )]
    pub(crate) fn from_admission<A: DeltaAdmission>(
        admission: &A,
        input: MaterializationUpdateInput,
    ) -> Result<Self, ContractError> {
        let MaterializationUpdateInput {
            graph_ref,
            contract_ref,
            plan_ref,
            base_snapshot_ref,
            result_snapshot_ref,
            withdraw_rows,
            admit_rows,
            withdraw_support_occurrence_refs,
            admit_supports,
            budget,
        } = input;
        let admitted_delta = AdmittedDeltaBinding::checked(admission.delta_pins())?;
        admitted_delta.validate_payload_refs(&base_snapshot_ref, &result_snapshot_ref)?;
        if !admission.admits_delta(DeltaPayload {
            base_snapshot_ref: &base_snapshot_ref,
            result_snapshot_ref: &result_snapshot_ref,
            withdraw_rows: &withdraw_rows,
            admit_rows: &admit_rows,
            withdraw_support_occurrence_refs: &withdraw_support_occurrence_refs,
            admit_supports: &admit_supports,
        }) {
            return Err(ContractError::DeltaPayloadNotAdmitted);
        }
        Ok(Self {
            admitted_delta,
            graph_ref,
            contract_ref,
            plan_ref,
            base_snapshot_ref,
            result_snapshot_ref,
            withdraw_rows,
            admit_rows,
            withdraw_support_occurrence_refs,
            admit_supports,
            budget,
        })
    }
}

/// Per-operation physical ceilings and deterministic failure injection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalBudget {
    pub maximum_rows: usize,
    pub maximum_supports: usize,
    pub maximum_index_entries: usize,
    pub maximum_temporary_bytes: u64,
    pub maximum_retained_bytes: u64,
    pub maximum_combined_live_bytes: u64,
    pub maximum_receipt_bytes: u64,
    pub maximum_buckets_per_range: u64,
    pub maximum_environments_per_anchor: usize,
    /// Fail this one-based logical byte reservation, for exact tests.
    pub fail_reservation_call: Option<u64>,
}

impl Default for PhysicalBudget {
    fn default() -> Self {
        Self {
            maximum_rows: 1_000_000,
            maximum_supports: 1_000_000,
            maximum_index_entries: 4_000_000,
            maximum_temporary_bytes: 256 * 1024 * 1024,
            maximum_retained_bytes: 256 * 1024 * 1024,
            maximum_combined_live_bytes: 512 * 1024 * 1024,
            maximum_receipt_bytes: 16 * 1024 * 1024,
            maximum_buckets_per_range: 65_536,
            maximum_environments_per_anchor: 65_536,
            fail_reservation_call: None,
        }
    }
}

/// A physical contract/input rejection. None of these outcomes carries
/// semantic authority or creates a partial materialized view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    EmptyPremiseSlots,
    DuplicatePremiseSlot,
    GraphIdentityMismatch,
    ContractIdentityMismatch,
    PlanIdentityMismatch,
    PremiseSlotMismatch,
    InvalidAnchorBindings,
    InvalidPhysicalParameter,
    EmptyAdmissionPin,
    SnapshotPayloadNotAdmitted,
    DeltaPayloadNotAdmitted,
    ClauseSemanticsIdentityMismatch,
    ProgramRevisionIdentityMismatch,
    RuntimeSessionIdentityMismatch,
    AdmittedStateIdentityMismatch,
    AdmittedDeltaBaseMismatch,
    AdmittedDeltaResultMismatch,
    DuplicateOccurrence,
    DuplicateBindingRef,
    DuplicateSupportOccurrence,
    MissingPremiseOccurrence,
    PremiseInputMismatch,
    MissingBinding,
    ScalarContractMismatch,
    ExactBaseMismatch,
    DuplicateWithdrawal,
    WithdrawalMissing,
    WithdrawalContentMismatch,
    ReusedOccurrence,
    DuplicateSupportWithdrawal,
    SupportWithdrawalMissing,
    NegativeExtent,
    InconsistentAnchorBindings,
    SupportOutsidePhysicalSuperset,
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ContractError {}

pub(crate) fn validate_rows(rows: &[InputRow]) -> Result<(), ContractError> {
    for (position, row) in rows.iter().enumerate() {
        row.validate()?;
        if rows[..position]
            .iter()
            .any(|prior| prior.occurrence_ref == row.occurrence_ref)
        {
            return Err(ContractError::DuplicateOccurrence);
        }
    }
    Ok(())
}

pub(crate) fn validate_supports(
    contract: &MaterializationContract,
    rows: &[InputRow],
    supports: &[SupportRecord],
) -> Result<(), ContractError> {
    for (position, support) in supports.iter().enumerate() {
        if supports[..position]
            .iter()
            .any(|prior| prior.support_occurrence_ref == support.support_occurrence_ref)
        {
            return Err(ContractError::DuplicateSupportOccurrence);
        }
        if support.premise_occurrence_refs.len() != contract.premise_slots.len() {
            return Err(ContractError::PremiseSlotMismatch);
        }
        for (slot, occurrence) in contract
            .premise_slots
            .iter()
            .zip(&support.premise_occurrence_refs)
        {
            let row = rows
                .iter()
                .find(|row| row.occurrence_ref == *occurrence)
                .ok_or(ContractError::MissingPremiseOccurrence)?;
            if row.input_ref != slot.input_ref {
                return Err(ContractError::PremiseInputMismatch);
            }
        }
    }
    Ok(())
}

fn validate_plan_identity(
    contract: &MaterializationContract,
    graph_ref: &OpaqueRef,
    contract_ref: &OpaqueRef,
) -> Result<(), ContractError> {
    if graph_ref != &contract.graph_ref {
        return Err(ContractError::GraphIdentityMismatch);
    }
    if contract_ref != &contract.contract_ref {
        return Err(ContractError::ContractIdentityMismatch);
    }
    Ok(())
}

pub(crate) fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
