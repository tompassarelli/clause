use crate::model::{
    AdmittedDeltaBinding, DeltaAdmission, DeltaAdmissionPins, DeltaPayload,
    MaterializationUpdateInput, SnapshotAdmission, SnapshotPayload, StateAdmissionPins,
};
use crate::work::AllocationTracker;
use crate::{
    AdmittedSnapshot, Binding, ContractError, FailureKind, FallbackReason, GridPlan, I32Binding,
    InputRow, MaterializationContract, MaterializationUpdate, MaterializedView, OpaqueRef,
    OpaqueValue, PhysicalBudget, PremiseSlot, ReceiptOutcome, ScanPlan, Schedule, SupportRecord,
    UniformGridMaterializer, WorkCounters, materialize_scan,
};

fn r(value: impl AsRef<str>) -> OpaqueRef {
    OpaqueRef::from(value.as_ref())
}

#[allow(clippy::too_many_arguments)]
fn admitted_delta_with(
    semantics: &str,
    program: &str,
    session: &str,
    predecessor: &str,
    result: &str,
    activation: &str,
    step: &str,
    delta: &str,
) -> AdmittedDeltaBinding {
    checked_delta_with(
        semantics,
        program,
        session,
        predecessor,
        result,
        activation,
        step,
        delta,
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
fn checked_delta_with(
    semantics: &str,
    program: &str,
    session: &str,
    predecessor: &str,
    result: &str,
    activation: &str,
    step: &str,
    delta: &str,
) -> Result<AdmittedDeltaBinding, ContractError> {
    let clause_semantics_ref = r(semantics);
    let program_revision_ref = r(program);
    let runtime_session_ref = r(session);
    let predecessor_state_revision_ref = r(predecessor);
    let result_state_revision_ref = r(result);
    let producing_activation_ref = r(activation);
    let producing_step_ref = r(step);
    let semantic_delta_ref = r(delta);
    AdmittedDeltaBinding::checked(DeltaAdmissionPins {
        clause_semantics: &clause_semantics_ref,
        program_revision: &program_revision_ref,
        runtime_session: &runtime_session_ref,
        predecessor_state_revision: &predecessor_state_revision_ref,
        result_state_revision: &result_state_revision_ref,
        producing_activation: &producing_activation_ref,
        producing_step: &producing_step_ref,
        semantic_delta: &semantic_delta_ref,
    })
}

#[derive(Clone)]
struct ExactSnapshotAdmission {
    clause_semantics_ref: OpaqueRef,
    program_revision_ref: OpaqueRef,
    runtime_session_ref: OpaqueRef,
    state_revision_ref: OpaqueRef,
    contract: MaterializationContract,
    rows: Vec<InputRow>,
    supports: Vec<SupportRecord>,
}

impl SnapshotAdmission for ExactSnapshotAdmission {
    fn state_pins(&self) -> StateAdmissionPins<'_> {
        StateAdmissionPins {
            clause_semantics: &self.clause_semantics_ref,
            program_revision: &self.program_revision_ref,
            runtime_session: &self.runtime_session_ref,
            state_revision: &self.state_revision_ref,
        }
    }

    fn admits_snapshot(&self, payload: SnapshotPayload<'_>) -> bool {
        payload.contract == &self.contract
            && payload.snapshot_ref == &self.state_revision_ref
            && payload.rows == self.rows
            && payload.supports == self.supports
    }
}

fn snapshot_with_contract(
    contract: &MaterializationContract,
    state: &str,
    rows: Vec<InputRow>,
    supports: Vec<SupportRecord>,
) -> AdmittedSnapshot {
    let admission = ExactSnapshotAdmission {
        clause_semantics_ref: r("semantics/process-v1"),
        program_revision_ref: r("program/revision-a"),
        runtime_session_ref: r("session/a"),
        state_revision_ref: r(state),
        contract: contract.clone(),
        rows: rows.clone(),
        supports: supports.clone(),
    };
    AdmittedSnapshot::from_admission(&admission, contract, r(state), rows, supports).unwrap()
}

struct ExactDeltaAdmission {
    clause_semantics_ref: OpaqueRef,
    program_revision_ref: OpaqueRef,
    runtime_session_ref: OpaqueRef,
    predecessor_state_revision_ref: OpaqueRef,
    result_state_revision_ref: OpaqueRef,
    producing_activation_ref: OpaqueRef,
    producing_step_ref: OpaqueRef,
    semantic_delta_ref: OpaqueRef,
    withdraw_rows: Vec<InputRow>,
    admit_rows: Vec<InputRow>,
    withdraw_support_occurrence_refs: Vec<OpaqueRef>,
    admit_supports: Vec<SupportRecord>,
}

impl DeltaAdmission for ExactDeltaAdmission {
    fn delta_pins(&self) -> DeltaAdmissionPins<'_> {
        DeltaAdmissionPins {
            clause_semantics: &self.clause_semantics_ref,
            program_revision: &self.program_revision_ref,
            runtime_session: &self.runtime_session_ref,
            predecessor_state_revision: &self.predecessor_state_revision_ref,
            result_state_revision: &self.result_state_revision_ref,
            producing_activation: &self.producing_activation_ref,
            producing_step: &self.producing_step_ref,
            semantic_delta: &self.semantic_delta_ref,
        }
    }

    fn admits_delta(&self, payload: DeltaPayload<'_>) -> bool {
        payload.base_snapshot_ref == &self.predecessor_state_revision_ref
            && payload.result_snapshot_ref == &self.result_state_revision_ref
            && payload.withdraw_rows == self.withdraw_rows
            && payload.admit_rows == self.admit_rows
            && payload.withdraw_support_occurrence_refs == self.withdraw_support_occurrence_refs
            && payload.admit_supports == self.admit_supports
    }
}

fn scalar(value: i32) -> OpaqueValue {
    OpaqueValue::new(r("repr/i32-be"), value.to_be_bytes().to_vec())
}

fn symbol(value: impl AsRef<str>) -> OpaqueValue {
    OpaqueValue::new(r("repr/symbol"), value.as_ref().as_bytes().to_vec())
}

fn binding(name: &str, value: OpaqueValue) -> Binding {
    Binding {
        binding_ref: r(name),
        value,
    }
}

fn row(input: &str, occurrence: &str, bindings: Vec<Binding>) -> InputRow {
    InputRow {
        input_ref: r(input),
        occurrence_ref: r(occurrence),
        bindings,
    }
}

fn range_x(occurrence: &str, x: i32) -> InputRow {
    range_x_in(occurrence, "world/a", "observer/a", x)
}

fn range_x_in(occurrence: &str, world: &str, anchor: &str, x: i32) -> InputRow {
    row(
        "input/range-x",
        occurrence,
        vec![
            binding("binding/world", symbol(world)),
            binding("binding/anchor", symbol(anchor)),
            binding("binding/center-x", scalar(x)),
        ],
    )
}

fn range_y(occurrence: &str, y: i32) -> InputRow {
    range_y_in(occurrence, "world/a", "observer/a", y)
}

fn range_y_in(occurrence: &str, world: &str, anchor: &str, y: i32) -> InputRow {
    row(
        "input/range-y",
        occurrence,
        vec![
            binding("binding/world", symbol(world)),
            binding("binding/anchor", symbol(anchor)),
            binding("binding/center-y", scalar(y)),
        ],
    )
}

fn range_extent(occurrence: &str, extent: i32) -> InputRow {
    range_extent_in(occurrence, "world/a", "observer/a", extent)
}

fn range_extent_in(occurrence: &str, world: &str, anchor: &str, extent: i32) -> InputRow {
    row(
        "input/range-extent",
        occurrence,
        vec![
            binding("binding/world", symbol(world)),
            binding("binding/anchor", symbol(anchor)),
            binding("binding/extent", scalar(extent)),
        ],
    )
}

fn point(occurrence: &str, target: &str, x: i32, y: i32) -> InputRow {
    point_in(occurrence, "world/a", target, x, y)
}

fn point_in(occurrence: &str, world: &str, target: &str, x: i32, y: i32) -> InputRow {
    row(
        "input/point",
        occurrence,
        vec![
            binding("binding/world", symbol(world)),
            binding("binding/target", symbol(target)),
            binding("binding/point-x", scalar(x)),
            binding("binding/point-y", scalar(y)),
        ],
    )
}

fn support(
    occurrence: &str,
    range_x: &str,
    range_y: &str,
    extent: &str,
    point: &str,
    output: &str,
) -> SupportRecord {
    SupportRecord {
        support_occurrence_ref: r(occurrence),
        premise_occurrence_refs: vec![r(range_x), r(range_y), r(extent), r(point)],
        output: r(output),
        evidence_ref: r("evidence/exact-bound-filter"),
    }
}

#[derive(Clone)]
struct Fixture {
    contract: MaterializationContract,
    scan_plan: ScanPlan,
    grid_plan: GridPlan,
    snapshot: AdmittedSnapshot,
}

impl Fixture {
    fn basic() -> Self {
        let contract = MaterializationContract {
            graph_ref: r("graph/a"),
            contract_ref: r("contract/range-v1"),
            premise_slots: vec![
                PremiseSlot {
                    slot_ref: r("slot/range-x"),
                    input_ref: r("input/range-x"),
                },
                PremiseSlot {
                    slot_ref: r("slot/range-y"),
                    input_ref: r("input/range-y"),
                },
                PremiseSlot {
                    slot_ref: r("slot/range-extent"),
                    input_ref: r("input/range-extent"),
                },
                PremiseSlot {
                    slot_ref: r("slot/point"),
                    input_ref: r("input/point"),
                },
            ],
        };
        let scan_plan = ScanPlan {
            graph_ref: contract.graph_ref.clone(),
            contract_ref: contract.contract_ref.clone(),
            plan_ref: r("plan/scan-v1"),
            premise_slot_order: contract
                .premise_slots
                .iter()
                .map(|slot| slot.slot_ref.clone())
                .collect(),
        };
        let grid_plan = GridPlan {
            graph_ref: contract.graph_ref.clone(),
            contract_ref: contract.contract_ref.clone(),
            plan_ref: r("plan/grid-v1"),
            point_slot_ref: r("slot/point"),
            range_slot_refs: vec![r("slot/range-x"), r("slot/range-y"), r("slot/range-extent")],
            anchor_binding_refs: vec![r("binding/world"), r("binding/anchor")],
            partition_binding_ref: r("binding/world"),
            point_x: I32Binding {
                binding_ref: r("binding/point-x"),
                representation_contract_ref: r("repr/i32-be"),
            },
            point_y: I32Binding {
                binding_ref: r("binding/point-y"),
                representation_contract_ref: r("repr/i32-be"),
            },
            center_x: I32Binding {
                binding_ref: r("binding/center-x"),
                representation_contract_ref: r("repr/i32-be"),
            },
            center_y: I32Binding {
                binding_ref: r("binding/center-y"),
                representation_contract_ref: r("repr/i32-be"),
            },
            extent: I32Binding {
                binding_ref: r("binding/extent"),
                representation_contract_ref: r("repr/i32-be"),
            },
            bucket_width: 10,
            maximum_buckets_per_range: 1_024,
        };
        let snapshot = snapshot_with_contract(
            &contract,
            "state/root",
            vec![
                range_x("row/rx", 0),
                range_y("row/ry", 0),
                range_extent("row/re", 15),
                point("row/p1", "target/a", 5, 0),
                point("row/p2", "target/b", 50, 0),
            ],
            vec![support(
                "support/s1",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p1",
                "output/visible-a",
            )],
        );
        Self {
            contract,
            scan_plan,
            grid_plan,
            snapshot,
        }
    }
}

fn support_snapshot(view: MaterializedView<'_>) -> Vec<SupportRecord> {
    view.supports().cloned().collect()
}

fn update(
    fixture: &Fixture,
    base: &str,
    result: &str,
    withdraw_rows: Vec<InputRow>,
    admit_rows: Vec<InputRow>,
    withdraw_supports: Vec<&str>,
    admit_supports: Vec<SupportRecord>,
) -> MaterializationUpdate {
    let withdraw_support_occurrence_refs = withdraw_supports.into_iter().map(r).collect::<Vec<_>>();
    let admission = ExactDeltaAdmission {
        clause_semantics_ref: r("semantics/process-v1"),
        program_revision_ref: r("program/revision-a"),
        runtime_session_ref: r("session/a"),
        predecessor_state_revision_ref: r(base),
        result_state_revision_ref: r(result),
        producing_activation_ref: r(format!("activation/{result}")),
        producing_step_ref: r(format!("step/{result}")),
        semantic_delta_ref: r(format!("delta/{result}")),
        withdraw_rows: withdraw_rows.clone(),
        admit_rows: admit_rows.clone(),
        withdraw_support_occurrence_refs: withdraw_support_occurrence_refs.clone(),
        admit_supports: admit_supports.clone(),
    };
    MaterializationUpdate::from_admission(
        &admission,
        MaterializationUpdateInput {
            graph_ref: fixture.contract.graph_ref.clone(),
            contract_ref: fixture.contract.contract_ref.clone(),
            plan_ref: fixture.grid_plan.plan_ref.clone(),
            base_snapshot_ref: r(base),
            result_snapshot_ref: r(result),
            withdraw_rows,
            admit_rows,
            withdraw_support_occurrence_refs,
            admit_supports,
            budget: PhysicalBudget::default(),
        },
    )
    .unwrap()
}

#[test]
fn semantic_admission_is_bound_to_exact_snapshot_and_delta_payloads() {
    let fixture = Fixture::basic();
    let snapshot_admission = ExactSnapshotAdmission {
        clause_semantics_ref: r("semantics/process-v1"),
        program_revision_ref: r("program/revision-a"),
        runtime_session_ref: r("session/a"),
        state_revision_ref: r("state/root"),
        contract: fixture.contract.clone(),
        rows: fixture.snapshot.rows.clone(),
        supports: fixture.snapshot.supports.clone(),
    };
    let snapshot_error = AdmittedSnapshot::from_admission(
        &snapshot_admission,
        &fixture.contract,
        r("state/root"),
        fixture.snapshot.rows.clone(),
        vec![],
    )
    .unwrap_err();
    assert_eq!(snapshot_error, ContractError::SnapshotPayloadNotAdmitted);

    let mut substituted_contract = fixture.contract.clone();
    substituted_contract.premise_slots[0].input_ref = r("input/substituted");
    let contract_error = AdmittedSnapshot::from_admission(
        &snapshot_admission,
        &substituted_contract,
        r("state/root"),
        fixture.snapshot.rows.clone(),
        fixture.snapshot.supports.clone(),
    )
    .unwrap_err();
    assert_eq!(contract_error, ContractError::SnapshotPayloadNotAdmitted);

    let admitted_support = support(
        "support/new",
        "row/rx",
        "row/ry",
        "row/re",
        "row/p1",
        "output/new",
    );
    let delta_admission = ExactDeltaAdmission {
        clause_semantics_ref: r("semantics/process-v1"),
        program_revision_ref: r("program/revision-a"),
        runtime_session_ref: r("session/a"),
        predecessor_state_revision_ref: r("state/root"),
        result_state_revision_ref: r("state/next"),
        producing_activation_ref: r("activation/next"),
        producing_step_ref: r("step/next"),
        semantic_delta_ref: r("delta/next"),
        withdraw_rows: vec![],
        admit_rows: vec![],
        withdraw_support_occurrence_refs: vec![],
        admit_supports: vec![admitted_support],
    };
    let delta_error = MaterializationUpdate::from_admission(
        &delta_admission,
        MaterializationUpdateInput {
            graph_ref: fixture.contract.graph_ref.clone(),
            contract_ref: fixture.contract.contract_ref.clone(),
            plan_ref: fixture.grid_plan.plan_ref.clone(),
            base_snapshot_ref: r("state/root"),
            result_snapshot_ref: r("state/next"),
            withdraw_rows: vec![],
            admit_rows: vec![],
            withdraw_support_occurrence_refs: vec![],
            admit_supports: vec![],
            budget: PhysicalBudget::default(),
        },
    )
    .unwrap_err();
    assert_eq!(delta_error, ContractError::DeltaPayloadNotAdmitted);
}

#[test]
fn cold_scan_and_indexed_plan_preserve_exact_support_multiset() {
    let fixture = Fixture::basic();
    let scan = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let (grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(support_snapshot(scan.view()), support_snapshot(grid.view()));
}

#[test]
fn atomic_point_move_and_range_change_preserve_scan_grid_parity() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let moved_point = point("row/p1-moved", "target/a", 25, 0);
    let move_update = update(
        &fixture,
        "state/root",
        "state/moved",
        vec![point("row/p1", "target/a", 5, 0)],
        vec![moved_point.clone()],
        vec!["support/s1"],
        vec![],
    );
    grid.advance(move_update).unwrap();
    assert!(grid.view().supports().next().is_none());

    let widened = range_extent("row/re-wide", 30);
    let widened_support = support(
        "support/s-wide",
        "row/rx",
        "row/ry",
        "row/re-wide",
        "row/p1-moved",
        "output/visible-a",
    );
    grid.advance(update(
        &fixture,
        "state/moved",
        "state/wide",
        vec![range_extent("row/re", 15)],
        vec![widened.clone()],
        vec![],
        vec![widened_support.clone()],
    ))
    .unwrap();

    let expected = snapshot_with_contract(
        &fixture.contract,
        "state/wide",
        vec![
            range_x("row/rx", 0),
            range_y("row/ry", 0),
            widened,
            moved_point,
            point("row/p2", "target/b", 50, 0),
        ],
        vec![widened_support],
    );
    let scan = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        expected,
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(support_snapshot(scan.view()), support_snapshot(grid.view()));
}

#[test]
fn equal_support_content_with_distinct_occurrences_retracts_independently() {
    let mut fixture = Fixture::basic();
    let mut second = fixture.snapshot.supports[0].clone();
    second.support_occurrence_ref = r("support/s2");
    fixture.snapshot.supports.push(second);
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(grid.view().supports().count(), 2);
    grid.advance(update(
        &fixture,
        "state/root",
        "state/one-support",
        vec![],
        vec![],
        vec!["support/s1"],
        vec![],
    ))
    .unwrap();
    let remaining = grid.view().supports().collect::<Vec<_>>();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].support_occurrence_ref, r("support/s2"));
}

#[test]
fn repeated_premise_slots_and_self_join_preserve_multiplicity() {
    let contract = MaterializationContract {
        graph_ref: r("graph/self"),
        contract_ref: r("contract/self"),
        premise_slots: vec![
            PremiseSlot {
                slot_ref: r("slot/range-a"),
                input_ref: r("input/range"),
            },
            PremiseSlot {
                slot_ref: r("slot/range-b"),
                input_ref: r("input/range"),
            },
            PremiseSlot {
                slot_ref: r("slot/point"),
                input_ref: r("input/point"),
            },
        ],
    };
    let grid_plan = GridPlan {
        graph_ref: contract.graph_ref.clone(),
        contract_ref: contract.contract_ref.clone(),
        plan_ref: r("plan/self-grid"),
        point_slot_ref: r("slot/point"),
        range_slot_refs: vec![r("slot/range-a"), r("slot/range-b")],
        anchor_binding_refs: vec![r("binding/world"), r("binding/anchor")],
        partition_binding_ref: r("binding/world"),
        point_x: I32Binding {
            binding_ref: r("binding/point-x"),
            representation_contract_ref: r("repr/i32-be"),
        },
        point_y: I32Binding {
            binding_ref: r("binding/point-y"),
            representation_contract_ref: r("repr/i32-be"),
        },
        center_x: I32Binding {
            binding_ref: r("binding/center-x"),
            representation_contract_ref: r("repr/i32-be"),
        },
        center_y: I32Binding {
            binding_ref: r("binding/center-y"),
            representation_contract_ref: r("repr/i32-be"),
        },
        extent: I32Binding {
            binding_ref: r("binding/extent"),
            representation_contract_ref: r("repr/i32-be"),
        },
        bucket_width: 10,
        maximum_buckets_per_range: 100,
    };
    let range = row(
        "input/range",
        "row/range-1",
        vec![
            binding("binding/world", symbol("world/a")),
            binding("binding/anchor", symbol("anchor/a")),
            binding("binding/center-x", scalar(0)),
            binding("binding/center-y", scalar(0)),
            binding("binding/extent", scalar(10)),
        ],
    );
    let snapshot = snapshot_with_contract(
        &contract,
        "state/self",
        vec![range, point("row/p1", "target/a", 0, 0)],
        vec![SupportRecord {
            support_occurrence_ref: r("support/self"),
            premise_occurrence_refs: vec![r("row/range-1"), r("row/range-1"), r("row/p1")],
            output: r("output/self"),
            evidence_ref: r("evidence/self"),
        }],
    );
    let (grid, _) =
        UniformGridMaterializer::build(&contract, &grid_plan, snapshot, PhysicalBudget::default())
            .unwrap();
    assert_eq!(grid.view().supports().count(), 1);
    assert_eq!(
        grid.view()
            .premise_multiplicity(&r("row/range-1"), &r("support/self")),
        2
    );
    assert_eq!(grid.reverse_index_sizes().premise_edges, 3);
}

#[test]
fn rejected_update_publishes_no_prefix() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    let error = grid
        .advance(update(
            &fixture,
            "state/root",
            "state/invalid",
            vec![point("row/p1", "target/a", 5, 0)],
            vec![point("row/p1-new", "target/a", 6, 0)],
            vec![],
            vec![],
        ))
        .unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::Contract(ContractError::MissingPremiseOccurrence)
    );
    assert_eq!(grid.snapshot_ref(), &r("state/root"));
    assert_eq!(support_snapshot(grid.view()), before);
    assert!(!matches!(
        error
            .receipt
            .as_ref()
            .expect("a post-preflight rejection retains its receipt")
            .outcome,
        ReceiptOutcome::Published
    ));
}

#[test]
fn oversized_extent_and_bucket_allocation_use_visible_typed_fallbacks() {
    let mut fixture = Fixture::basic();
    // Extent 15 at width 10 covers exactly 16 buckets: declared limit + 1.
    fixture.grid_plan.maximum_buckets_per_range = 15;
    let (grid, receipt) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(grid.view().supports().count(), 1);
    assert_eq!(receipt.fallbacks[0].reason, FallbackReason::BucketLimit);
    assert_eq!(
        receipt.fallbacks[0].selected_schedule,
        Schedule::PartitionScan
    );
    assert_eq!(
        receipt.fallbacks[0].range_occurrence_refs,
        Some(vec![r("row/rx"), r("row/ry"), r("row/re")])
    );
    let scan = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(support_snapshot(scan.view()), support_snapshot(grid.view()));

    fixture.grid_plan.maximum_buckets_per_range = 1_024;
    let budget = PhysicalBudget {
        // Admission, payload, anchor, and range-shell reservations precede
        // this exact bounded bucket-vector reservation.
        fail_reservation_call: Some(18),
        ..PhysicalBudget::default()
    };
    let (grid, receipt) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        budget,
    )
    .unwrap();
    assert_eq!(grid.view().supports().count(), 1);
    assert!(
        receipt
            .fallbacks
            .iter()
            .any(|fallback| fallback.reason == FallbackReason::ForcedReservationFailure)
    );
}

#[test]
fn disconnected_growth_does_not_change_local_indexed_advance_receipt() {
    let fixture = Fixture::basic();
    let mut large_rows = fixture.snapshot.rows.clone();
    for index in 0..256 {
        large_rows.push(row(
            "input/unrelated",
            &format!("row/unrelated-{index}"),
            vec![binding("binding/noise", symbol(format!("noise/{index}")))],
        ));
    }
    let large_snapshot = snapshot_with_contract(
        &fixture.contract,
        "state/root",
        large_rows,
        fixture.snapshot.supports.clone(),
    );
    let (mut small, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let (mut large, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        large_snapshot,
        PhysicalBudget::default(),
    )
    .unwrap();
    let local = || {
        update(
            &fixture,
            "state/root",
            "state/local",
            vec![point("row/p1", "target/a", 5, 0)],
            vec![point("row/p1-new", "target/a", 6, 0)],
            vec!["support/s1"],
            vec![support(
                "support/s-new",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p1-new",
                "output/visible-a",
            )],
        )
    };
    let small_receipt = small.advance(local()).unwrap();
    let large_receipt = large.advance(local()).unwrap();
    assert_eq!(
        small_receipt.counters.locality(),
        large_receipt.counters.locality()
    );
    assert_eq!(small_receipt.counters.whole_state_clones, 0);
    assert_eq!(small_receipt.counters.whole_view_rebuilds, 0);
    assert_eq!(small_receipt.counters.support_set_clones, 0);
    assert_eq!(small_receipt.counters.disconnected_rows_visited, 0);
    assert_eq!(
        support_snapshot(small.view()),
        support_snapshot(large.view())
    );
}

#[test]
fn nominal_ref_renaming_does_not_select_meaning_in_rust() {
    let fixture = Fixture::basic();
    let mut renamed = fixture.clone();
    renamed.grid_plan.plan_ref = r("completely-different-opaque-plan-token");
    let (grid, _) = UniformGridMaterializer::build(
        &renamed.contract,
        &renamed.grid_plan,
        renamed.snapshot,
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(grid.view().supports().count(), 1);
    assert_eq!(grid.snapshot_ref(), &r("state/root"));
}

#[test]
fn unrelated_delta_is_an_observable_dependency_miss_only() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    let receipt = grid
        .advance(update(
            &fixture,
            "state/root",
            "state/unrelated",
            vec![],
            vec![row(
                "input/unrelated",
                "row/unrelated",
                vec![binding("binding/noise", symbol("noise"))],
            )],
            vec![],
            vec![],
        ))
        .unwrap();
    assert_eq!(receipt.counters.dependency_misses, 1);
    assert_eq!(receipt.counters.whole_view_rebuilds, 0);
    assert_eq!(support_snapshot(grid.view()), before);
}

#[test]
fn equal_row_content_with_distinct_occurrences_remains_distinct() {
    let mut fixture = Fixture::basic();
    fixture
        .snapshot
        .rows
        .push(point("row/p1-equal", "target/a", 5, 0));
    fixture.snapshot.supports.push(support(
        "support/equal-row",
        "row/rx",
        "row/ry",
        "row/re",
        "row/p1-equal",
        "output/visible-a",
    ));
    let (grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let occurrences = grid
        .view()
        .supports()
        .map(|support| support.premise_occurrence_refs[3].clone())
        .collect::<Vec<_>>();
    assert_eq!(occurrences, vec![r("row/p1-equal"), r("row/p1")]);
}

#[test]
fn duplicate_values_under_distinct_bindings_survive_but_duplicate_refs_reject() {
    let mut fixture = Fixture::basic();
    fixture.snapshot.rows[0]
        .bindings
        .push(binding("binding/alias", scalar(0)));
    UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();

    let duplicate = fixture.snapshot.rows[0].bindings[0].clone();
    fixture.snapshot.rows[0].bindings.push(duplicate);
    let error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot,
        PhysicalBudget::default(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::Contract(ContractError::DuplicateBindingRef)
    );
}

#[test]
fn stale_graph_contract_plan_and_base_are_rejected_without_mutation() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    for (field, expected) in [
        ("graph", ContractError::GraphIdentityMismatch),
        ("contract", ContractError::ContractIdentityMismatch),
        ("plan", ContractError::PlanIdentityMismatch),
        ("base", ContractError::ExactBaseMismatch),
    ] {
        let mut candidate = update(
            &fixture,
            "state/root",
            "state/stale",
            vec![],
            vec![],
            vec![],
            vec![],
        );
        match field {
            "graph" => candidate.graph_ref = r("stale/graph"),
            "contract" => candidate.contract_ref = r("stale/contract"),
            "plan" => candidate.plan_ref = r("stale/plan"),
            "base" => candidate.base_snapshot_ref = r("stale/base"),
            _ => unreachable!(),
        }
        let error = grid.advance(candidate).unwrap_err();
        assert_eq!(error.kind, FailureKind::Contract(expected));
        assert_eq!(grid.snapshot_ref(), &r("state/root"));
        assert_eq!(support_snapshot(grid.view()), before);
    }
}

#[test]
fn physical_plan_change_never_changes_snapshot_identity() {
    let fixture = Fixture::basic();
    let mut alternate = fixture.grid_plan.clone();
    alternate.plan_ref = r("plan/grid-alternate");
    alternate.bucket_width = 7;
    let (first, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let (second, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &alternate,
        fixture.snapshot,
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_ne!(first.plan_ref(), second.plan_ref());
    assert_eq!(first.snapshot_ref(), second.snapshot_ref());
    assert_eq!(
        support_snapshot(first.view()),
        support_snapshot(second.view())
    );
}

fn delta_pin_mismatch_cases() -> [(AdmittedDeltaBinding, ContractError); 5] {
    [
        (
            admitted_delta_with(
                "semantics/other",
                "program/revision-a",
                "session/a",
                "state/root",
                "state/next",
                "activation/next",
                "step/next",
                "delta/next",
            ),
            ContractError::ClauseSemanticsIdentityMismatch,
        ),
        (
            admitted_delta_with(
                "semantics/process-v1",
                "program/other",
                "session/a",
                "state/root",
                "state/next",
                "activation/next",
                "step/next",
                "delta/next",
            ),
            ContractError::ProgramRevisionIdentityMismatch,
        ),
        (
            admitted_delta_with(
                "semantics/process-v1",
                "program/revision-a",
                "session/other",
                "state/root",
                "state/next",
                "activation/next",
                "step/next",
                "delta/next",
            ),
            ContractError::RuntimeSessionIdentityMismatch,
        ),
        (
            admitted_delta_with(
                "semantics/process-v1",
                "program/revision-a",
                "session/a",
                "state/other",
                "state/next",
                "activation/next",
                "step/next",
                "delta/next",
            ),
            ContractError::AdmittedDeltaBaseMismatch,
        ),
        (
            admitted_delta_with(
                "semantics/process-v1",
                "program/revision-a",
                "session/a",
                "state/root",
                "state/other",
                "activation/next",
                "step/next",
                "delta/next",
            ),
            ContractError::AdmittedDeltaResultMismatch,
        ),
    ]
}

#[test]
fn admitted_delta_pins_are_exact_and_rejection_is_atomic() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    for (binding, expected) in delta_pin_mismatch_cases() {
        let mut candidate = update(
            &fixture,
            "state/root",
            "state/next",
            vec![],
            vec![],
            vec![],
            vec![],
        );
        candidate.admitted_delta = binding;
        let error = grid.advance(candidate).unwrap_err();
        assert_eq!(error.kind, FailureKind::Contract(expected));
        assert_eq!(grid.snapshot_ref(), &r("state/root"));
        assert_eq!(support_snapshot(grid.view()), before);
    }

    for (activation, step, delta) in [
        ("", "step/next", "delta/next"),
        ("activation/next", "", "delta/next"),
        ("activation/next", "step/next", ""),
    ] {
        let error = checked_delta_with(
            "semantics/process-v1",
            "program/revision-a",
            "session/a",
            "state/root",
            "state/next",
            activation,
            step,
            delta,
        )
        .unwrap_err();
        assert_eq!(error, ContractError::EmptyAdmissionPin);
    }
}

#[test]
fn independent_range_occurrence_combinations_share_one_anchor() {
    let mut fixture = Fixture::basic();
    fixture.snapshot.rows.push(range_x("row/rx-second", 40));
    fixture.snapshot.supports.push(support(
        "support/s2",
        "row/rx-second",
        "row/ry",
        "row/re",
        "row/p2",
        "output/visible-b",
    ));
    let (grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let scan = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        fixture.snapshot,
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(grid.view().supports().count(), 2);
    assert_eq!(support_snapshot(scan.view()), support_snapshot(grid.view()));
}

#[test]
fn support_outside_indexed_physical_superset_rejects_without_publication() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    let error = grid
        .advance(update(
            &fixture,
            "state/root",
            "state/outside",
            vec![],
            vec![],
            vec![],
            vec![support(
                "support/outside",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p2",
                "output/outside",
            )],
        ))
        .unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::Contract(ContractError::SupportOutsidePhysicalSuperset)
    );
    assert_eq!(grid.snapshot_ref(), &r("state/root"));
    assert_eq!(support_snapshot(grid.view()), before);
}

#[test]
fn partition_fallback_preserves_the_exact_bucket_envelope() {
    let mut fixture = Fixture::basic();
    fixture.grid_plan.maximum_buckets_per_range = 15;
    fixture.snapshot.supports.push(support(
        "support/outside",
        "row/rx",
        "row/ry",
        "row/re",
        "row/p2",
        "output/outside",
    ));
    let error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot,
        PhysicalBudget::default(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::Contract(ContractError::SupportOutsidePhysicalSuperset)
    );

    let mut fixture = Fixture::basic();
    fixture.grid_plan.maximum_buckets_per_range = 15;
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let error = grid
        .advance(update(
            &fixture,
            "state/root",
            "state/outside",
            vec![],
            vec![],
            vec![],
            vec![support(
                "support/outside",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p2",
                "output/outside",
            )],
        ))
        .unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::Contract(ContractError::SupportOutsidePhysicalSuperset)
    );
}

#[test]
fn environment_exhaustion_is_typed_and_never_support_scans() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    let mut candidate = update(
        &fixture,
        "state/root",
        "state/environment-limit",
        vec![point("row/p1", "target/a", 5, 0)],
        vec![point("row/p1-new", "target/a", 6, 0)],
        vec!["support/s1"],
        vec![support(
            "support/s-new",
            "row/rx",
            "row/ry",
            "row/re",
            "row/p1-new",
            "output/visible-a",
        )],
    );
    candidate.budget.maximum_environments_per_anchor = 0;
    let error = grid.advance(candidate).unwrap_err();
    assert_eq!(
        error.kind,
        FailureKind::TemporaryAllocationExhausted(FallbackReason::EnvironmentLimit)
    );
    assert!(
        error
            .receipt
            .as_ref()
            .expect("a post-preflight rejection retains its receipt")
            .fallbacks
            .is_empty()
    );
    assert_eq!(grid.snapshot_ref(), &r("state/root"));
    assert_eq!(support_snapshot(grid.view()), before);
}

#[test]
fn retained_ceilings_and_injected_retained_failure_precede_publication() {
    let fixture = Fixture::basic();
    let retained_limit = PhysicalBudget {
        maximum_retained_bytes: 1,
        ..PhysicalBudget::default()
    };
    let scan_error = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        fixture.snapshot.clone(),
        retained_limit,
    )
    .unwrap_err();
    assert_eq!(scan_error.kind, FailureKind::RetainedAllocationExhausted);
    let grid_error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        retained_limit,
    )
    .unwrap_err();
    assert_eq!(grid_error.kind, FailureKind::RetainedAllocationExhausted);

    let index_limit = PhysicalBudget {
        maximum_index_entries: 1,
        ..PhysicalBudget::default()
    };
    let scan_error = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        fixture.snapshot.clone(),
        index_limit,
    )
    .unwrap_err();
    assert_eq!(scan_error.kind, FailureKind::IndexLimitExceeded);
    let grid_error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        index_limit,
    )
    .unwrap_err();
    assert_eq!(grid_error.kind, FailureKind::IndexLimitExceeded);

    let (_, successful_receipt) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let failed_call = successful_receipt.counters.reservation_calls;
    let injected = PhysicalBudget {
        fail_reservation_call: Some(failed_call),
        ..PhysicalBudget::default()
    };
    let injected_error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot,
        injected,
    )
    .unwrap_err();
    assert_eq!(
        injected_error.kind,
        FailureKind::RetainedAllocationExhausted
    );
    assert_eq!(
        injected_error
            .receipt
            .as_ref()
            .expect("a post-preflight rejection retains its receipt")
            .counters
            .reservation_calls,
        failed_call
    );
}

#[test]
fn receipt_payloads_have_an_explicit_byte_ceiling() {
    let fixture = Fixture::basic();
    let budget = PhysicalBudget {
        maximum_receipt_bytes: 1,
        ..PhysicalBudget::default()
    };
    let scan_error = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        fixture.snapshot.clone(),
        budget,
    )
    .unwrap_err();
    assert_eq!(scan_error.kind, FailureKind::ReceiptLimitExceeded);
    assert!(scan_error.receipt.is_none());
    assert!(scan_error.attempted_receipt_bytes > budget.maximum_receipt_bytes);
    let grid_error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        budget,
    )
    .unwrap_err();
    assert_eq!(grid_error.kind, FailureKind::ReceiptLimitExceeded);
    assert!(grid_error.receipt.is_none());
    assert!(grid_error.attempted_receipt_bytes > budget.maximum_receipt_bytes);

    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    let mut candidate = update(
        &fixture,
        "state/root",
        "state/receipt-limited",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    candidate.budget = budget;
    let update_error = grid.advance(candidate).unwrap_err();
    assert_eq!(update_error.kind, FailureKind::ReceiptLimitExceeded);
    assert!(update_error.receipt.is_none());
    assert!(update_error.attempted_receipt_bytes > budget.maximum_receipt_bytes);
    assert_eq!(grid.snapshot_ref(), &r("state/root"));
    assert_eq!(support_snapshot(grid.view()), before);
}

#[test]
fn fallback_receipt_preflight_returns_compact_exact_failure() {
    let mut fixture = Fixture::basic();
    fixture.grid_plan.maximum_buckets_per_range = 15;
    let (_, full_receipt) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    assert_eq!(full_receipt.fallbacks.len(), 1);
    let full_bytes = full_receipt.counters.receipt_bytes;
    let budget = PhysicalBudget {
        maximum_receipt_bytes: full_bytes - 1,
        ..PhysicalBudget::default()
    };
    let error = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot,
        budget,
    )
    .unwrap_err();
    assert_eq!(error.kind, FailureKind::ReceiptLimitExceeded);
    assert!(error.receipt.is_none());
    assert_eq!(error.attempted_receipt_bytes, full_bytes);
}

#[test]
fn combined_live_preflight_is_compact_and_update_atomic() {
    let fixture = Fixture::basic();
    let budget = PhysicalBudget {
        maximum_combined_live_bytes: 1,
        ..PhysicalBudget::default()
    };
    for error in [
        materialize_scan(
            &fixture.contract,
            &fixture.scan_plan,
            fixture.snapshot.clone(),
            budget,
        )
        .unwrap_err(),
        UniformGridMaterializer::build(
            &fixture.contract,
            &fixture.grid_plan,
            fixture.snapshot.clone(),
            budget,
        )
        .unwrap_err(),
    ] {
        assert_eq!(error.kind, FailureKind::CombinedLiveAllocationExhausted);
        assert!(error.receipt.is_none());
        assert!(error.attempted_receipt_bytes > budget.maximum_combined_live_bytes);
    }

    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let before = support_snapshot(grid.view());
    let mut candidate = update(
        &fixture,
        "state/root",
        "state/combined-limited",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    candidate.budget = budget;
    let error = grid.advance(candidate).unwrap_err();
    assert_eq!(error.kind, FailureKind::CombinedLiveAllocationExhausted);
    assert!(error.receipt.is_none());
    assert_eq!(grid.snapshot_ref(), &r("state/root"));
    assert_eq!(support_snapshot(grid.view()), before);
}

#[test]
fn live_budget_reservations_fail_transactionally() {
    let budget = PhysicalBudget {
        maximum_temporary_bytes: 8,
        maximum_retained_bytes: 8,
        maximum_combined_live_bytes: 10,
        maximum_receipt_bytes: 8,
        ..PhysicalBudget::default()
    };
    let mut tracker = AllocationTracker::new(budget, 0);
    let mut counters = WorkCounters::default();
    tracker.reserve_output(3, &mut counters).unwrap();
    tracker.reserve_retained(4, &mut counters).unwrap();
    assert_eq!(
        tracker.reserve(4, &mut counters),
        Err(FallbackReason::CombinedLiveByteLimit)
    );
    tracker.reserve(3, &mut counters).unwrap();
    tracker.release(3).unwrap();
    assert_eq!(
        tracker.reserve_retained(5, &mut counters),
        Err(FailureKind::RetainedAllocationExhausted)
    );
    assert_eq!(
        tracker.reserve_retained(4, &mut counters),
        Err(FailureKind::CombinedLiveAllocationExhausted)
    );
}

#[test]
fn allocation_free_validation_preserves_contract_diagnostics() {
    let fixture = Fixture::basic();
    let invalid = snapshot_with_contract(
        &fixture.contract,
        "state/duplicate-binding",
        vec![row(
            "input/range-x",
            "row/duplicate-binding",
            vec![
                binding("binding/world", symbol("world/a")),
                binding("binding/world", symbol("world/a")),
            ],
        )],
        vec![],
    );
    let budget = PhysicalBudget {
        maximum_temporary_bytes: 0,
        ..PhysicalBudget::default()
    };
    let scan_error = materialize_scan(
        &fixture.contract,
        &fixture.scan_plan,
        invalid.clone(),
        budget,
    )
    .unwrap_err();
    assert_eq!(
        scan_error.kind,
        FailureKind::Contract(ContractError::DuplicateBindingRef)
    );
    let grid_error =
        UniformGridMaterializer::build(&fixture.contract, &fixture.grid_plan, invalid, budget)
            .unwrap_err();
    assert_eq!(
        grid_error.kind,
        FailureKind::Contract(ContractError::DuplicateBindingRef)
    );
}

#[test]
fn retained_add_release_receipt_balances_exactly() {
    let fixture = Fixture::basic();
    let (mut grid, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let receipt = grid
        .advance(update(
            &fixture,
            "state/root",
            "state/replaced",
            vec![point("row/p1", "target/a", 5, 0)],
            vec![point("row/p1-replaced", "target/a", 6, 0)],
            vec!["support/s1"],
            vec![support(
                "support/s-replaced",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p1-replaced",
                "output/visible-a",
            )],
        ))
        .unwrap();
    assert!(receipt.counters.retained_bytes_before > 0);
    assert!(receipt.counters.retained_bytes_added > 0);
    assert!(receipt.counters.retained_bytes_released > 0);
    assert_eq!(
        receipt.counters.retained_bytes_after,
        receipt
            .counters
            .retained_bytes_before
            .checked_sub(receipt.counters.retained_bytes_released)
            .unwrap()
            .checked_add(receipt.counters.retained_bytes_added)
            .unwrap()
    );
}

#[test]
fn dependency_bearing_disconnected_partitions_do_not_change_local_work() {
    let fixture = Fixture::basic();
    let mut expanded_rows = fixture.snapshot.rows.clone();
    let mut expanded_supports = fixture.snapshot.supports.clone();
    for index in 0..64 {
        let world = format!("world/disconnected-{index}");
        let anchor = format!("observer/disconnected-{index}");
        let rx = format!("row/disconnected-rx-{index}");
        let ry = format!("row/disconnected-ry-{index}");
        let re = format!("row/disconnected-re-{index}");
        let point_ref = format!("row/disconnected-point-{index}");
        expanded_rows.push(range_x_in(&rx, &world, &anchor, 0));
        expanded_rows.push(range_y_in(&ry, &world, &anchor, 0));
        expanded_rows.push(range_extent_in(&re, &world, &anchor, 10));
        expanded_rows.push(point_in(
            &point_ref,
            &world,
            &format!("target/disconnected-{index}"),
            0,
            0,
        ));
        expanded_supports.push(support(
            &format!("support/disconnected-{index}"),
            &rx,
            &ry,
            &re,
            &point_ref,
            &format!("output/disconnected-{index}"),
        ));
    }
    let expanded = snapshot_with_contract(
        &fixture.contract,
        "state/root",
        expanded_rows,
        expanded_supports,
    );
    let (mut small, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let (mut large, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        expanded,
        PhysicalBudget::default(),
    )
    .unwrap();
    let local = || {
        update(
            &fixture,
            "state/root",
            "state/local-dependent",
            vec![point("row/p1", "target/a", 5, 0)],
            vec![point("row/p1-local", "target/a", 6, 0)],
            vec!["support/s1"],
            vec![support(
                "support/s-local",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p1-local",
                "output/visible-a",
            )],
        )
    };
    let small_receipt = small.advance(local()).unwrap();
    let large_receipt = large.advance(local()).unwrap();
    assert_eq!(
        small_receipt.counters.locality(),
        large_receipt.counters.locality()
    );
    assert_eq!(large_receipt.counters.whole_state_clones, 0);
    assert_eq!(large_receipt.counters.whole_view_rebuilds, 0);
    assert_eq!(large_receipt.counters.support_set_clones, 0);
    assert_eq!(large_receipt.counters.disconnected_rows_visited, 0);
    assert_eq!(small.view().supports().count(), 1);
    assert_eq!(large.view().supports().count(), 65);
    assert!(
        large
            .view()
            .supports()
            .any(|support| support.support_occurrence_ref == r("support/s-local"))
    );
}

#[test]
fn same_partition_disjoint_buckets_do_not_change_local_work() {
    let fixture = Fixture::basic();
    let mut expanded_rows = fixture.snapshot.rows.clone();
    let mut expanded_supports = fixture.snapshot.supports.clone();
    for index in 0..32 {
        let anchor = format!("observer/far-{index}");
        let rx = format!("row/far-rx-{index}");
        let ry = format!("row/far-ry-{index}");
        let re = format!("row/far-re-{index}");
        let point_ref = format!("row/far-point-{index}");
        let x = 1_000 + index * 100;
        expanded_rows.push(range_x_in(&rx, "world/a", &anchor, x));
        expanded_rows.push(range_y_in(&ry, "world/a", &anchor, 0));
        expanded_rows.push(range_extent_in(&re, "world/a", &anchor, 10));
        expanded_rows.push(point_in(
            &point_ref,
            "world/a",
            &format!("target/far-{index}"),
            x,
            0,
        ));
        expanded_supports.push(support(
            &format!("support/far-{index}"),
            &rx,
            &ry,
            &re,
            &point_ref,
            &format!("output/far-{index}"),
        ));
    }
    let expanded = snapshot_with_contract(
        &fixture.contract,
        "state/root",
        expanded_rows,
        expanded_supports,
    );
    let (mut small, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let (mut large, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        expanded,
        PhysicalBudget::default(),
    )
    .unwrap();
    let local = || {
        update(
            &fixture,
            "state/root",
            "state/same-partition-local",
            vec![point("row/p1", "target/a", 5, 0)],
            vec![point("row/p1-same-partition", "target/a", 6, 0)],
            vec!["support/s1"],
            vec![support(
                "support/same-partition-local",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p1-same-partition",
                "output/visible-a",
            )],
        )
    };
    let small_receipt = small.advance(local()).unwrap();
    let large_receipt = large.advance(local()).unwrap();
    assert_eq!(
        small_receipt.counters.locality(),
        large_receipt.counters.locality()
    );
    assert_eq!(small.view().supports().count(), 1);
    assert_eq!(large.view().supports().count(), 33);
    assert!(
        large
            .view()
            .supports()
            .any(|support| support.support_occurrence_ref == r("support/same-partition-local"))
    );
}

#[test]
fn same_partition_partition_scan_growth_is_visible_and_exact() {
    let mut fixture = Fixture::basic();
    fixture.grid_plan.maximum_buckets_per_range = 1;
    let mut expanded_rows = fixture.snapshot.rows.clone();
    let mut expanded_supports = fixture.snapshot.supports.clone();
    for index in 0..12 {
        let anchor = format!("observer/fallback-{index}");
        let rx = format!("row/fallback-rx-{index}");
        let ry = format!("row/fallback-ry-{index}");
        let re = format!("row/fallback-re-{index}");
        let point_ref = format!("row/fallback-point-{index}");
        let x = 1_000 + index * 100;
        expanded_rows.push(range_x_in(&rx, "world/a", &anchor, x));
        expanded_rows.push(range_y_in(&ry, "world/a", &anchor, 0));
        expanded_rows.push(range_extent_in(&re, "world/a", &anchor, 10));
        expanded_rows.push(point_in(
            &point_ref,
            "world/a",
            &format!("target/fallback-{index}"),
            x,
            0,
        ));
        expanded_supports.push(support(
            &format!("support/fallback-{index}"),
            &rx,
            &ry,
            &re,
            &point_ref,
            &format!("output/fallback-{index}"),
        ));
    }
    let expanded = snapshot_with_contract(
        &fixture.contract,
        "state/root",
        expanded_rows,
        expanded_supports,
    );
    let (mut small, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        fixture.snapshot.clone(),
        PhysicalBudget::default(),
    )
    .unwrap();
    let (mut large, _) = UniformGridMaterializer::build(
        &fixture.contract,
        &fixture.grid_plan,
        expanded,
        PhysicalBudget::default(),
    )
    .unwrap();
    let local = || {
        update(
            &fixture,
            "state/root",
            "state/fallback-local",
            vec![point("row/p1", "target/a", 5, 0)],
            vec![point("row/p1-fallback", "target/a", 6, 0)],
            vec!["support/s1"],
            vec![support(
                "support/fallback-local",
                "row/rx",
                "row/ry",
                "row/re",
                "row/p1-fallback",
                "output/visible-a",
            )],
        )
    };
    let small_receipt = small.advance(local()).unwrap();
    let large_receipt = large.advance(local()).unwrap();
    assert!(
        large_receipt.counters.fallback_point_visits > small_receipt.counters.fallback_point_visits
    );
    assert_eq!(small.view().supports().count(), 1);
    assert_eq!(large.view().supports().count(), 13);
    assert!(
        large
            .view()
            .supports()
            .any(|support| support.support_occurrence_ref == r("support/fallback-local"))
    );
}

const POSITION_RADIUS_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/materialization/position-radius-v1/program.clause"
));
const POSITION_RADIUS_SOURCE_CONTEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/materialization/position-radius-v1/source-context.json"
));
const POSITION_RADIUS_NORMALIZED_GRAPH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/materialization/position-radius-v1/normalized-graph.json"
));

#[derive(Debug, Eq, PartialEq)]
struct SourceProjection {
    relations: Vec<String>,
    laws: Vec<String>,
    derivations: Vec<String>,
    type_uses: Vec<String>,
    operator_uses: Vec<String>,
}

struct ExpectedSourceBinding {
    source_kind: &'static str,
    local_designation: &'static str,
    opaque_ref: &'static str,
    graph_paths: &'static [&'static str],
}

const SOURCE_BINDINGS: [ExpectedSourceBinding; 4] = [
    ExpectedSourceBinding {
        source_kind: "relation",
        local_designation: "observer-position-v1",
        opaque_ref: "world/observer-position-v1",
        graph_paths: &[
            "/physical_profile/premise_slots/0/input_ref",
            "/physical_profile/center_input_ref",
            "/relations/0/relation_ref",
            "/materialized_law/semantic_dependencies/0",
            "/scan_plan/join_order/0",
            "/invalidation/0/relation_ref",
        ],
    },
    ExpectedSourceBinding {
        source_kind: "relation",
        local_designation: "target-position-v1",
        opaque_ref: "world/target-position-v1",
        graph_paths: &[
            "/physical_profile/premise_slots/2/input_ref",
            "/physical_profile/point_input_ref",
            "/relations/2/relation_ref",
            "/materialized_law/semantic_dependencies/2",
            "/scan_plan/join_order/2",
            "/invalidation/2/relation_ref",
        ],
    },
    ExpectedSourceBinding {
        source_kind: "relation",
        local_designation: "proximity-radius-v1",
        opaque_ref: "world/proximity-radius-v1",
        graph_paths: &[
            "/physical_profile/premise_slots/1/input_ref",
            "/physical_profile/extent_input_ref",
            "/relations/1/relation_ref",
            "/materialized_law/semantic_dependencies/1",
            "/scan_plan/join_order/1",
            "/invalidation/1/relation_ref",
        ],
    },
    ExpectedSourceBinding {
        source_kind: "law",
        local_designation: "radius-proximity-law-v1",
        opaque_ref: "world/radius-proximity-law-v1",
        graph_paths: &["/materialized_law/law_ref"],
    },
];

const UNBOUND_SOURCE_DESIGNATIONS: [(&str, &str); 3] = [
    ("type-use", "point2-v1"),
    ("type-use", "nonnegative-q16-16-v1"),
    ("operator-use", "within-radius-v1"),
];

fn is_local_designation(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn one_local_designation(value: &str, line_number: usize) -> Result<String, String> {
    let mut words = value.split_whitespace();
    let designation = words
        .next()
        .ok_or_else(|| format!("missing Designation at line {line_number}"))?;
    if words.next().is_some() || !is_local_designation(designation) {
        return Err(format!("invalid local Designation at line {line_number}"));
    }
    Ok(designation.to_owned())
}

fn parse_source_projection(source: &str) -> Result<SourceProjection, String> {
    let mut projection = SourceProjection {
        relations: Vec::new(),
        laws: Vec::new(),
        derivations: Vec::new(),
        type_uses: Vec::new(),
        operator_uses: Vec::new(),
    };
    let mut inside_law_premises = false;
    for (line_index, line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let authored = line.split_once('#').map_or(line, |(code, _)| code);
        if authored.contains('/') {
            return Err(format!("authored slash at line {line_number}"));
        }
        let trimmed = authored.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("relation ") {
            projection
                .relations
                .push(one_local_designation(value, line_number)?);
        } else if let Some(value) = trimmed.strip_prefix("law ") {
            projection
                .laws
                .push(one_local_designation(value, line_number)?);
        } else if let Some(value) = trimmed.strip_prefix("derive ") {
            projection
                .derivations
                .push(one_local_designation(value, line_number)?);
        } else if let Some(value) = trimmed.strip_prefix("has {") {
            let field = value
                .strip_suffix('}')
                .ok_or_else(|| format!("unclosed role declaration at line {line_number}"))?;
            let (_, value_type) = field
                .split_once(':')
                .ok_or_else(|| format!("missing role type at line {line_number}"))?;
            projection
                .type_uses
                .push(one_local_designation(value_type, line_number)?);
        } else if trimmed == "if" {
            inside_law_premises = true;
        } else if trimmed == "then" {
            inside_law_premises = false;
        } else if inside_law_premises && !trimmed.starts_with('?') {
            let operator = trimmed
                .split_whitespace()
                .next()
                .ok_or_else(|| format!("missing operator at line {line_number}"))?;
            if !is_local_designation(operator) {
                return Err(format!("invalid local operator at line {line_number}"));
            }
            projection.operator_uses.push(operator.to_owned());
        }
    }
    Ok(projection)
}

fn json_string_field<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {field}"))
}

fn validate_source_context(source: &str, context: &str, graph: &str) -> Result<(), String> {
    let projection = parse_source_projection(source)?;
    if projection.relations
        != [
            "observer-position-v1",
            "target-position-v1",
            "proximity-radius-v1",
        ]
        || projection.laws != ["radius-proximity-law-v1"]
        || projection.derivations != ["radius-proximity-law-v1"]
        || projection.type_uses != ["point2-v1", "point2-v1", "nonnegative-q16-16-v1"]
        || projection.operator_uses != ["within-radius-v1"]
    {
        return Err("source declaration/use closure changed".to_owned());
    }
    if projection
        .derivations
        .iter()
        .any(|derivation| !projection.laws.contains(derivation))
    {
        return Err("derive target is not a declared law".to_owned());
    }

    let context: serde_json::Value =
        serde_json::from_str(context).map_err(|error| error.to_string())?;
    let context = context
        .as_object()
        .ok_or_else(|| "source context must be an object".to_owned())?;
    if context.len() != 3
        || json_string_field(context, "format")? != "position-radius-source-context-v1"
    {
        return Err("source context root mismatch".to_owned());
    }
    let bindings = context
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "missing bindings".to_owned())?;
    if bindings.len() != SOURCE_BINDINGS.len() {
        return Err("source binding count mismatch".to_owned());
    }
    let graph: serde_json::Value =
        serde_json::from_str(graph).map_err(|error| error.to_string())?;
    for (binding, expected) in bindings.iter().zip(&SOURCE_BINDINGS) {
        let binding = binding
            .as_object()
            .ok_or_else(|| "source binding must be an object".to_owned())?;
        if binding.len() != 4
            || json_string_field(binding, "source_kind")? != expected.source_kind
            || json_string_field(binding, "local_designation")? != expected.local_designation
            || json_string_field(binding, "opaque_ref")? != expected.opaque_ref
        {
            return Err("source binding mismatch".to_owned());
        }
        let paths = binding
            .get("normalized_graph_paths")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "missing normalized graph paths".to_owned())?;
        if paths.len() != expected.graph_paths.len() {
            return Err("normalized graph path count mismatch".to_owned());
        }
        for (path, expected_path) in paths.iter().zip(expected.graph_paths) {
            if path.as_str() != Some(expected_path)
                || graph
                    .pointer(expected_path)
                    .and_then(serde_json::Value::as_str)
                    != Some(expected.opaque_ref)
            {
                return Err(format!("typed graph binding mismatch at {expected_path}"));
            }
        }
    }

    let unbound = context
        .get("unbound_local_designations")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "missing unbound local Designations".to_owned())?;
    if unbound.len() != UNBOUND_SOURCE_DESIGNATIONS.len() {
        return Err("unbound local Designation count mismatch".to_owned());
    }
    for (entry, (expected_kind, expected_designation)) in
        unbound.iter().zip(UNBOUND_SOURCE_DESIGNATIONS)
    {
        let entry = entry
            .as_object()
            .ok_or_else(|| "unbound entry must be an object".to_owned())?;
        if entry.len() != 3
            || json_string_field(entry, "source_kind")? != expected_kind
            || json_string_field(entry, "local_designation")? != expected_designation
            || json_string_field(entry, "status")? != "unbound-unknown"
        {
            return Err("unbound local Designation mismatch".to_owned());
        }
    }
    Ok(())
}

#[test]
fn source_context_binds_exact_typed_graph_paths() {
    validate_source_context(
        POSITION_RADIUS_SOURCE,
        POSITION_RADIUS_SOURCE_CONTEXT,
        POSITION_RADIUS_NORMALIZED_GRAPH,
    )
    .unwrap();
}

#[test]
fn unrelated_derive_target_rejects_source_context() {
    let altered_source =
        POSITION_RADIUS_SOURCE.replace("derive radius-proximity-law-v1", "derive unrelated-law");
    assert!(
        validate_source_context(
            &altered_source,
            POSITION_RADIUS_SOURCE_CONTEXT,
            POSITION_RADIUS_NORMALIZED_GRAPH,
        )
        .is_err()
    );
}

#[test]
fn decoy_json_strings_cannot_satisfy_typed_graph_paths() {
    let mut graph: serde_json::Value =
        serde_json::from_str(POSITION_RADIUS_NORMALIZED_GRAPH).unwrap();
    for binding in &SOURCE_BINDINGS {
        for path in binding.graph_paths {
            *graph.pointer_mut(path).unwrap() = serde_json::Value::String("wrong/ref".to_owned());
        }
    }
    graph.as_object_mut().unwrap().insert(
        "decoy_refs".to_owned(),
        serde_json::Value::Array(
            SOURCE_BINDINGS
                .iter()
                .map(|binding| serde_json::Value::String(binding.opaque_ref.to_owned()))
                .collect(),
        ),
    );
    assert!(
        validate_source_context(
            POSITION_RADIUS_SOURCE,
            POSITION_RADIUS_SOURCE_CONTEXT,
            &serde_json::to_string(&graph).unwrap(),
        )
        .is_err()
    );
}
