use std::collections::BTreeSet;

use clause_package::*;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

const SCALAR_PARAMETER_WORLD: &str = r#"F64
Bool
Player
Enemy
CombatState
alive
telegraph

shape Vec3
  x: F64
  y: F64
  z: F64

relation score
  reads {player: Player} score {value: F64}
  subject player
  mode given player yields value: one

relation pressure-clock
  reads {enemy: Enemy} pressure clock {value: F64}
  subject enemy
  mode given enemy yields value: one

relation pressure-state
  reads {enemy: Enemy} pressure state {value: CombatState}
  subject enemy
  mode given enemy yields value: one

relation grounded
  reads {player: Player} grounded {value: Bool}
  subject player
  mode given player yields value: one

relation spawn-position
  reads {enemy: Enemy} spawn position {value: Vec3}
  subject enemy
  mode given enemy yields value: one

player-1 ∈ Player
cinder-wraith ∈ Enemy
player-1 score 0.0
cinder-wraith pressure clock 3.0
cinder-wraith pressure state telegraph
player-1 grounded true
cinder-wraith spawn position Vec3 { x: 2.0, y: 0.0, z: 0.0 }

on advance ?player
  when
    ?player score ?score
    cinder-wraith pressure clock ?clock
    cinder-wraith pressure state ?pressure
    player-1 grounded ?grounded
    ?pressure = telegraph
    ?grounded = true
  withdraw
    ?player score ?score
  include
    ?player score ?score + ?clock
"#;

const GENERAL_HANDLER_ARGUMENT_WORLD: &str = r#"F64
Player

relation score
  reads {player: Player} score {value: F64}
  subject player
  mode given player yields value: one

relation reserve
  reads {player: Player} reserve {value: F64}
  subject player
  mode given player yields value: one

player-1 ∈ Player
player-1 score 4.0
player-1 reserve 6.0

on adjust ?player ?amount
  when
    ?player score ?score
    ?player reserve ?reserve
  withdraw
    ?player score ?score
    ?player reserve ?reserve
  include
    ?player score ?score + ?amount
    ?player reserve ?reserve - ?amount
"#;

const SPACED_TEXT_INSERTION_WORLD: &str = r#"Root
Command
Text

relation phase
  reads {root: Root} phase {value: Text}
  subject root
  mode given root yields value: one

relation known-command
  reads {root: Root} known command {value: Command}
  subject root
  mode given root yields value: many

relation command-description
  reads {command: Command} description {value: Text}
  subject command
  mode given command yields value: maybe

root-main ∈ Root
root-main phase "ready"

on initialize ?root
  when
    ?root phase ?phase
  create
    ?command ∈ Command
  withdraw
    ?root phase ?phase
  include
    ?root phase ?phase
    ?root known command ?command
    ?command description "start a new conversation"
"#;

const TRANSITIVE_REFERENT_WORLD: &str = r#"F64
Root
Policy

relation balance
  reads {root: Root} balance {value: F64}
  subject root
  mode given root yields value: one

relation selected-policy
  reads {root: Root} selected policy {policy: Policy}
  subject root
  mode given root yields policy: one

relation policy-adjustment
  reads {policy: Policy} policy adjustment {value: F64}
  subject policy
  mode given policy yields value: one

root-1 ∈ Root
policy-a ∈ Policy
root-1 balance 10.0
root-1 selected policy policy-a
policy-a policy adjustment 2.0

on apply-selected-policy ?root
  when
    ?root balance ?prior
    ?root selected policy ?policy
    ?policy policy adjustment ?adjustment
  withdraw
    ?root balance ?prior
  include
    ?root balance ?prior - ?adjustment
"#;

const MULTI_MEMBERSHIP_REFERENT_WORLD: &str = r#"Door
Lockable
iron-door

iron-door ∈ Door
iron-door ∈ Lockable
"#;

fn compile_source(
    source: &str,
    root: u8,
) -> Result<CanonicalSourcePackageSliceV1, CanonicalSourceErrorV1> {
    let cst = read_canonical_source_v1(source.as_bytes())?;
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(root)),
    )?;
    elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
}

#[test]
fn one_referent_may_declare_multiple_memberships() {
    compile_source(MULTI_MEMBERSHIP_REFERENT_WORLD, 39)
        .expect("an explicit referent and its memberships share one allocated identity");
    compile_source(
        "Door\nLockable\n\niron-door ∈ Door\niron-door ∈ Lockable\niron-door\n",
        40,
    )
    .expect("referent identity sharing does not depend on declaration order");
}

#[test]
fn repeated_explicit_referent_declaration_remains_rejected() {
    let error = compile_source("iron-door\niron-door\n", 41)
        .expect_err("two explicit declarations still compete for one designation");
    assert!(matches!(
        error,
        CanonicalSourceErrorV1::DuplicateDesignation { designation }
            if designation == b"iron-door"
    ));
}

#[test]
fn general_handler_inserts_spaced_text_without_absorbing_it_into_the_relation() {
    let compiled = compile_source(SPACED_TEXT_INSERTION_WORLD, 38)
        .expect("a spaced Text literal reaches relational insertion lowering");
    assert!(compiled.unsupported.is_empty());
    assert!(
        compiled
            .executable_handlers
            .iter()
            .any(|handler| handler.designation == b"initialize")
    );
}

#[test]
fn general_handler_joins_a_typed_referent_selected_by_prior_state() {
    let compiled = compile_source(TRANSITIVE_REFERENT_WORLD, 18)
        .expect("the selected typed referent reaches executable lowering");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"apply-selected-policy")
        .expect("the transitive referent handler is executable");
    let [rule] = handler.rules.as_slice() else {
        panic!("one source transition remains one executable rule")
    };
    assert_eq!(rule.required_present.len(), 3);
    assert_eq!(
        rule.required_present
            .iter()
            .map(|state| (
                state.relation_designation.as_slice(),
                state.subject.as_slice(),
            ))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (b"balance".as_slice(), b"root-1".as_slice()),
            (b"policy-adjustment".as_slice(), b"policy-a".as_slice()),
            (b"selected-policy".as_slice(), b"root-1".as_slice()),
        ]),
        "every edge in the selected-referent join remains an exact execution support"
    );
    assert_eq!(
        rule.required_present
            .iter()
            .map(|state| state.assertion)
            .collect::<BTreeSet<_>>()
            .len(),
        3,
        "the join retains three distinct nominal assertion identities"
    );
    let [assignment] = rule.assignments.as_slice() else {
        panic!("the source replacement remains one atomic assignment")
    };
    assert_eq!(assignment.target.relation_designation, b"balance");
    assert_eq!(assignment.target.subject, b"root-1");
    assert!(matches!(
        &assignment.value,
        CanonicalExecutableExpressionV1::Subtract(prior, adjustment)
            if matches!(prior.as_ref(), CanonicalExecutableExpressionV1::State(state)
                if state.relation_designation == b"balance" && state.subject == b"root-1")
                && matches!(adjustment.as_ref(), CanonicalExecutableExpressionV1::State(state)
                    if state.relation_designation == b"policy-adjustment"
                        && state.subject == b"policy-a")
    ));
}

#[test]
fn transitive_referent_join_lowers_each_runtime_selectable_target() {
    let source = TRANSITIVE_REFERENT_WORLD.replacen(
        "policy-a policy adjustment 2.0",
        "policy-a policy adjustment 2.0\npolicy-b ∈ Policy\npolicy-b policy adjustment 4.0",
        1,
    );
    let compiled = compile_source(&source, 23)
        .expect("every typed selected policy reaches executable lowering");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"apply-selected-policy")
        .expect("the runtime-selected policy handler is executable");
    assert_eq!(handler.rules.len(), 2);
    let selected = handler
        .rules
        .iter()
        .flat_map(|rule| &rule.predicates)
        .filter_map(|predicate| match predicate {
            CanonicalExecutablePredicateV1::Equal(
                CanonicalExecutableExpressionV1::State(state),
                CanonicalExecutableExpressionV1::Constant(CanonicalScalarValueV1::Symbol(expected)),
            ) if state.relation_designation == b"selected-policy" => Some(expected.as_slice()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected,
        BTreeSet::from([b"policy-a".as_slice(), b"policy-b".as_slice()]),
    );
}

#[test]
fn scalar_law_result_updates_a_vector_on_the_selected_handler_subject() {
    let source = r#"F64
Bool
Player
Enemy

shape Vec3
  x: F64
  y: F64
  z: F64

relation clamped-between
  reads {value: F64} clamped between {lower: F64} and {upper: F64} as {result: F64}
  mode given value lower upper yields result: maybe

relation combat-target
  reads {player: Player} combat target {enemy: Enemy}
  subject player
  mode given player yields enemy: one

relation target-active
  reads {player: Player} target active {value: Bool}
  subject player
  mode given player yields value: one

relation vitals
  reads {enemy: Enemy} vitals {value: Vec3}
  subject enemy
  mode given enemy yields value: one

law clamp-lower
  if
    ?lower <= ?upper
    ?value < ?lower
  then
    ?value clamped between ?lower and ?upper as ?lower

law clamp-interior
  if
    ?lower <= ?value
    ?value <= ?upper
  then
    ?value clamped between ?lower and ?upper as ?value

law clamp-upper
  if
    ?lower <= ?upper
    ?value > ?upper
  then
    ?value clamped between ?lower and ?upper as ?upper

derive clamp-lower
derive clamp-interior
derive clamp-upper

player-1 ∈ Player
enemy-1 ∈ Enemy
player-1 combat target enemy-1
player-1 target active true
enemy-1 vitals Vec3 { x: 6.0, y: 6.0, z: 1.0 }

on targeted-hit ?enemy
  when
    player-1 combat target ?enemy
    player-1 target active ?active
    ?enemy vitals Vec3 { x: ?vitality, y: ?maximum, z: ?alive }
    (?vitality - 2.0) clamped between 0.0 and ?maximum as ?next
    ?active = true
  withdraw
    ?enemy vitals Vec3 { x: ?vitality, y: ?maximum, z: ?alive }
  include
    ?enemy vitals Vec3 { x: ?next, y: ?maximum, z: ?alive }
"#;
    let cst = read_canonical_source_v1(source.as_bytes())
        .expect("the selected-subject scalar-law source reads");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(24)),
    )
    .expect("the selected handler subject receives exact allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("a derived scalar result updates the selected subject vector");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"targeted-hit")
        .expect("the selected-subject handler is executable");
    let [rule] = handler.rules.as_slice() else {
        panic!("one source transition remains one executable rule")
    };
    let [assignment] = rule.assignments.as_slice() else {
        panic!("only the changed vitality field is assigned")
    };
    assert!(matches!(
        assignment.value,
        CanonicalExecutableExpressionV1::Clamp(_, _, _)
    ));
}

#[test]
fn transitive_referent_join_rejects_wrong_type_missing_cardinality_and_ambiguity() {
    let wrong_type = TRANSITIVE_REFERENT_WORLD.replacen(
        "reads {policy: Policy} policy adjustment",
        "reads {policy: Root} policy adjustment",
        1,
    );
    assert!(matches!(
        compile_source(&wrong_type, 19),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));

    let missing = TRANSITIVE_REFERENT_WORLD.replacen("policy-a policy adjustment 2.0\n", "", 1);
    assert!(matches!(
        compile_source(&missing, 20),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));

    let nonsingleton = TRANSITIVE_REFERENT_WORLD.replacen(
        "mode given root yields policy: one",
        "mode given root yields policy: many",
        1,
    );
    assert!(matches!(
        compile_source(&nonsingleton, 21),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));

    let ambiguous = TRANSITIVE_REFERENT_WORLD.replacen(
        "policy-a policy adjustment 2.0\n\non apply-selected-policy",
        "policy-a policy adjustment 2.0\nroot-2 ∈ Root\nroot-2 balance 8.0\nroot-2 selected policy policy-a\n\non apply-selected-policy",
        1,
    );
    assert!(matches!(
        compile_source(&ambiguous, 22),
        Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { .. })
    ));
}

#[test]
fn general_handler_arguments_lower_by_declared_header_ordinal() {
    let cst = read_canonical_source_v1(GENERAL_HANDLER_ARGUMENT_WORLD.as_bytes())
        .expect("the argument-bearing general handler reads canonically");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(17)),
    )
    .expect("the argument-bearing general handler receives rooted allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("the declared general-handler argument reaches executable lowering");

    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"adjust")
        .expect("the argument-bearing general handler is executable");
    assert_eq!(handler.trigger, CanonicalHandlerTriggerV1::External);
    assert_eq!(handler.argument_count, 1);
    assert!(matches!(
        &handler.rules[0].assignments[0].value,
        CanonicalExecutableExpressionV1::Add(_, argument)
            if argument.as_ref() == &CanonicalExecutableExpressionV1::Argument(0)
    ));
    assert!(matches!(
        &handler.rules[0].assignments[1].value,
        CanonicalExecutableExpressionV1::Subtract(_, argument)
            if argument.as_ref() == &CanonicalExecutableExpressionV1::Argument(0)
    ));
}

#[test]
fn scalar_handlers_bind_number_symbol_and_boolean_state_parameters() {
    let cst = read_canonical_source_v1(SCALAR_PARAMETER_WORLD.as_bytes())
        .expect("canonical scalar-parameter source reads losslessly");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(12)),
    )
    .expect("scalar state dependencies receive rooted allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("number, symbol, and Boolean state parameters reach executable lowering");

    assert_eq!(compiled.scalar_handlers.len(), 1);
    assert_eq!(
        compiled.scalar_handlers[0].parameters,
        [
            b"?clock".to_vec(),
            b"?grounded".to_vec(),
            b"?pressure".to_vec(),
        ]
    );
    assert_eq!(compiled.state_cells.len(), 7);
    let executable = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"advance")
        .expect("the scalar handler is executable rather than unsupported");
    assert_eq!(executable.trigger, CanonicalHandlerTriggerV1::FixedTick);
    assert_eq!(executable.rules.len(), 1);
    assert_eq!(executable.rules[0].predicates.len(), 2);
    assert_eq!(executable.rules[0].assignments.len(), 1);
}

#[test]
fn source_keyboard_bindings_leave_unbound_actor_relative_scalar_handlers_on_fixed_tick() {
    let mut source = WORLD.to_vec();
    source.extend_from_slice(
        br#"

relation recovery-clock
  reads {player: Player} recovery clock {value: F64}
  subject player
  mode given player yields value: one

relation recovery-rate
  reads {player: Player} recovery rate {value: F64}
  subject player
  mode given player yields value: one

relation heat
  reads {player: Player} heat {value: F64}
  subject player
  mode given player yields value: one

player-1 recovery clock 8.0
player-1 recovery rate 1.0
player-1 heat 3.0

bind keyboard Space down to jump

on count-recovery ?player
  when
    ?player recovery clock ?clock
    ?player recovery rate ?rate
    ?clock > 0.0
  withdraw
    ?player recovery clock ?clock
  include
    ?player recovery clock ?clock - ?rate

on cool-heat ?player
  when
    ?player heat ?heat
    ?heat > 0.0
  withdraw
    ?player heat ?heat
  include
    ?player heat ?heat - 1.0
"#,
    );
    let cst = read_canonical_source_v1(&source)
        .expect("actor-relative scalar handlers read with source keyboard bindings");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(16)),
    )
    .expect("actor-relative scalar handlers receive rooted allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("actor-relative scalar handlers reach executable lowering");

    for designation in [b"count-recovery".as_slice(), b"cool-heat".as_slice()] {
        let handler = compiled
            .executable_handlers
            .iter()
            .find(|handler| handler.designation == designation)
            .expect("the automatic scalar handler is executable");
        assert_eq!(handler.trigger, CanonicalHandlerTriggerV1::FixedTick);
    }
    let jump = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"jump")
        .expect("the source-bound jump handler is executable");
    assert_eq!(jump.trigger, CanonicalHandlerTriggerV1::External);
}

#[test]
fn jump_shaped_handlers_retain_their_source_designation() {
    let source = std::str::from_utf8(WORLD)
        .expect("canonical arena source is UTF-8")
        .replacen("on jump ?player", "on dash ?player", 1);
    let cst = read_canonical_source_v1(source.as_bytes())
        .expect("a source-named jump-shaped handler reads canonically");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(13)),
    )
    .expect("the source-named handler receives rooted allocation");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("the jump-shaped handler reaches executable lowering");

    assert!(compiled.jump_handler.is_some());
    assert!(compiled.executable_handlers.iter().any(|handler| {
        handler.designation == b"dash"
            && handler.trigger == CanonicalHandlerTriggerV1::External
            && handler.argument_count == 0
    }));
    assert!(
        !compiled
            .executable_handlers
            .iter()
            .any(|handler| handler.designation == b"jump")
    );
}

#[test]
fn canonical_world_declarations_reach_the_checked_package_with_exact_remainder() {
    let cst = read_canonical_source_v1(WORLD).expect("canonical world source reads losslessly");
    assert_eq!(cst.exact_source(), WORLD);

    let root = ProgramChangeOccurrenceId::from_bytes(raw_id(3));
    let plan = plan_independent_canonical_source_allocations_v1(&cst, root)
        .expect("the declaration slice has an explicit independent allocation plan");
    assert_eq!(plan.artifact(), cst.artifact());
    assert_eq!(plan.root(), root);
    assert!(plan.allocations().iter().all(|allocation| {
        let nonzero = match allocation.identity {
            CanonicalAllocatedIdentityV1::Formation(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::Capability(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::RelationSchema(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::Role(id) => id.role.get() != 0,
            CanonicalAllocatedIdentityV1::Operator(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::Mode(id) => id.mode.get() != 0,
        };
        nonzero
            && matches!(
                &allocation.judgment,
                CanonicalAllocationJudgmentV1::Fresh {
                    basis: CanonicalFreshBasisV1::ConstitutedProgramChange(actual_root),
                    producer,
                    slot: CanonicalAllocationSlotV1::Emission(slot),
                    collision:
                        CanonicalAllocationCollisionDispositionV1::RejectTypedCollision,
                    cycle: CanonicalAllocationCycleDispositionV1::RejectDependencyCycle,
                } if *actual_root == root
                    && !producer.semantic_key.is_empty()
                    && !slot.local.is_empty()
            )
    }));
    assert_eq!(
        plan.allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<BTreeSet<_>>()
            .len(),
        plan.allocations().len(),
        "every nominal product receives one distinct typed local identity"
    );
    let rematerialized = rematerialize_canonical_source_allocation_plan_v1(&cst, &plan)
        .expect("the recorded plan rematerializes without allocating again");
    assert_eq!(rematerialized, plan);

    let other_root = ProgramChangeOccurrenceId::from_bytes(raw_id(4));
    let other_plan = plan_independent_canonical_source_allocations_v1(&cst, other_root)
        .expect("a distinct constituted change root has an independent plan");
    assert_ne!(other_plan, plan);
    assert_ne!(
        other_plan
            .allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<Vec<_>>(),
        plan.allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<Vec<_>>(),
        "equal source under a distinct constituted root has distinct nominal coordinates"
    );
    assert!(
        other_plan
            .allocations()
            .iter()
            .zip(plan.allocations())
            .all(|(other, original)| other.identity != original.identity),
        "every corresponding fixture allocation changes under a distinct root"
    );

    let reordered_source = std::str::from_utf8(WORLD)
        .expect("fixture is UTF-8")
        .replacen("F64\nBool", "Bool\nF64", 1);
    let reordered = read_canonical_source_v1(reordered_source.as_bytes())
        .expect("reordered unrelated declarations remain canonical source");
    assert_eq!(
        rematerialize_canonical_source_allocation_plan_v1(&reordered, &plan),
        Err(CanonicalSourceErrorV1::AllocationArtifactMismatch),
        "source reorder cannot masquerade as rematerialization or retention"
    );
    let reordered_plan = plan_independent_canonical_source_allocations_v1(
        &reordered,
        ProgramChangeOccurrenceId::from_bytes(raw_id(5)),
    )
    .expect("reordered source requires a fresh constituted plan");
    assert!(
        reordered_plan
            .allocations()
            .iter()
            .all(|allocation| matches!(
                allocation.judgment,
                CanonicalAllocationJudgmentV1::Fresh { .. }
            ))
    );

    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("the supported declaration slice encodes, decodes, and checks");
    let compiled_again = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &rematerialized,
    )
    .expect("the exact recorded plan rematerializes the same checked package");
    assert_eq!(
        compiled_again.checked_package.exact_bytes(),
        compiled.checked_package.exact_bytes()
    );
    assert_eq!(compiled_again.emissions, compiled.emissions);

    let constitution = compiled.checked_package.constitution().preimage();
    assert_eq!(constitution.formations.len(), 56);
    assert_eq!(constitution.schemas.len(), 13);
    assert_eq!(constitution.operators.len(), 13);
    assert!(constitution.applications.is_empty());
    assert_eq!(
        constitution
            .schemas
            .iter()
            .map(|schema| schema.roles.len())
            .sum::<usize>(),
        28
    );
    assert_eq!(
        constitution
            .schemas
            .iter()
            .filter(|schema| schema.roles.len() == 4)
            .count(),
        1,
        "the four-role clamped-between declaration remains structurally distinct"
    );
    assert_eq!(compiled.emissions.len(), 99);
    assert!(compiled.emissions.iter().all(|emission| {
        cst.source_slice(emission.origin)
            .is_some_and(|source| !source.is_empty())
    }));

    let unsupported_counts =
        compiled
            .unsupported
            .iter()
            .fold([0_usize; 4], |mut counts, unsupported| {
                let index = match unsupported.production {
                    CanonicalSourceProductionV1::Law => 0,
                    CanonicalSourceProductionV1::Derive => 1,
                    CanonicalSourceProductionV1::Assertion => 2,
                    CanonicalSourceProductionV1::Handler => 3,
                    other => panic!("unexpected unsupported production: {other:?}"),
                };
                counts[index] += 1;
                counts
            });
    assert_eq!(unsupported_counts, [0, 0, 0, 0]);
    let membership_emissions = compiled
        .emissions
        .iter()
        .filter(|emission| {
            emission.slot.production == CanonicalSourceProductionV1::Assertion
                && matches!(emission.slot.local.as_slice(), b"Arena" | b"Player")
        })
        .collect::<Vec<_>>();
    assert_eq!(membership_emissions.len(), 2);
    assert_eq!(
        membership_emissions
            .iter()
            .map(|emission| cst
                .source_slice(emission.origin)
                .expect("owned target origin"))
            .collect::<Vec<_>>(),
        [b"Arena".as_slice(), b"Player".as_slice()]
    );
    assert!(
        membership_emissions
            .iter()
            .all(|emission| emission.slot.repetition.is_none())
    );

    let input = compiled
        .input_handler
        .expect("the bounded source profile lowers the actual on-input handler");
    assert_eq!(input.artifact, cst.artifact());
    assert_eq!(input.initial_x, 0.0_f64.to_bits());
    assert_eq!(input.initial_z, 0.0_f64.to_bits());
    assert_eq!(input.result_x, CanonicalInputScalarV1::Parameter(0));
    assert_eq!(input.result_z, CanonicalInputScalarV1::Parameter(1));
    assert!(cst.source_slice(input.handler_origin).is_some());
    assert!(cst.source_slice(input.initial_assertion_origin).is_some());

    let jump = compiled
        .jump_handler
        .expect("the bounded source profile lowers the actual on-jump handler");
    assert_eq!(jump.initial_velocity, [0.0_f64.to_bits(); 3]);
    assert!(jump.initial_grounded);
    assert_eq!(jump.jump_speed, 8.0_f64.to_bits());
    assert!(jump.required_grounded);
    assert_eq!(
        jump.result_velocity,
        [
            CanonicalJumpScalarV1::VelocityComponent(0),
            CanonicalJumpScalarV1::JumpSpeed,
            CanonicalJumpScalarV1::VelocityComponent(2),
        ]
    );
    assert!(!jump.result_grounded);
    for origin in [
        jump.handler_origin,
        jump.velocity_assertion_origin,
        jump.grounded_assertion_origin,
        jump.jump_speed_assertion_origin,
    ] {
        assert!(cst.source_slice(origin).is_some());
    }

    let tick = compiled
        .tick_program
        .expect("the bounded source profile lowers all three on-tick branches");
    assert_eq!(tick.rules.len(), 3);
    assert_eq!(tick.initial_position, [0.0_f64.to_bits(); 3]);
    assert_eq!(tick.initial_velocity, [0.0_f64.to_bits(); 3]);
    assert_eq!(tick.initial_intent, [0.0_f64.to_bits(); 3]);
    assert!(tick.initial_grounded);
    assert_eq!(tick.gravity, (-8.0_f64).to_bits());
    assert_eq!(tick.move_speed, 5.0_f64.to_bits());
    assert_eq!(tick.floor_height, 0.0_f64.to_bits());
    assert_eq!(tick.minimum_x, (-10.0_f64).to_bits());
    assert_eq!(tick.maximum_x, 10.0_f64.to_bits());
    assert_eq!(tick.minimum_z, (-10.0_f64).to_bits());
    assert_eq!(tick.maximum_z, 10.0_f64.to_bits());
    for origin in
        tick.assertion_origins
            .iter()
            .chain(&tick.clamp_law_origins)
            .chain(&tick.derive_origins)
            .chain(tick.rules.iter().flat_map(|rule| {
                std::iter::once(&rule.handler_origin).chain(&rule.include_origins)
            }))
    {
        assert!(cst.source_slice(*origin).is_some());
    }

    let carrier = ProcessCarrier::replay(&compiled.checked_package, &AuthorityStore::new())
        .expect("the existing package carrier consumes the checked declaration package");
    assert_eq!(carrier.application_count(), 0);
}

#[test]
fn canonical_input_preserves_negative_zero_bits() {
    let source = std::str::from_utf8(WORLD)
        .expect("fixture is UTF-8")
        .replacen(
            "player-1 horizontal intent Vec3 { x: 0.0, y: 0.0, z: 0.0 }",
            "player-1 horizontal intent Vec3 { x: -0.0, y: 0.0, z: 0.0 }",
            1,
        )
        .replacen(
            "include\n    ?player horizontal intent Vec3 { x: ?intent-x, y: 0.0, z: ?intent-z }",
            "include\n    ?player horizontal intent Vec3 { x: -0.0, y: 0.0, z: ?intent-z }",
            1,
        );
    let cst = read_canonical_source_v1(source.as_bytes()).expect("negative zero source reads");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(6)),
    )
    .expect("negative zero source receives rooted allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("negative zero source reaches the checked package");
    let input = compiled.input_handler.expect("input handler lowers");
    assert_eq!(input.initial_x, (-0.0_f64).to_bits());
    assert_eq!(
        input.result_x,
        CanonicalInputScalarV1::Number((-0.0_f64).to_bits())
    );
}

#[test]
fn standalone_membership_group_preserves_order_and_item_origins() {
    let source = "Door\nLockable\niron-door ∈ Door, Lockable\n";
    let cst = read_canonical_source_v1(source.as_bytes()).expect("grouped membership source reads");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(7)),
    )
    .expect("supported declarations retain one explicit allocation plan");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("the membership group reaches bounded elaboration");

    assert!(compiled.unsupported.is_empty());
    let emissions = compiled
        .emissions
        .iter()
        .filter(|emission| emission.slot.production == CanonicalSourceProductionV1::Assertion)
        .collect::<Vec<_>>();
    assert_eq!(emissions.len(), 2);
    assert_eq!(emissions[0].producer, emissions[1].producer);
    assert_eq!(emissions[0].slot.local, b"Door");
    assert_eq!(emissions[1].slot.local, b"Lockable");
    assert_eq!(emissions[0].slot.repetition, None);
    assert_eq!(emissions[1].slot.repetition, None);
    assert_eq!(
        cst.source_slice(emissions[0].origin),
        Some(b"Door".as_slice())
    );
    assert_eq!(
        cst.source_slice(emissions[1].origin),
        Some(b"Lockable".as_slice())
    );
    assert!(
        emissions
            .iter()
            .all(|emission| emission.allocations.is_empty())
    );
}

#[test]
fn repeated_membership_target_is_not_deduplicated() {
    let source = "Door\niron-door ∈ Door, Door\n";
    let cst = read_canonical_source_v1(source.as_bytes())
        .expect("repeated targets are valid grouping sugar");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(8)),
    )
    .expect("the repeated membership group does not collapse allocation inputs");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("the repeated membership group reaches bounded elaboration");
    let emissions = compiled
        .emissions
        .iter()
        .filter(|emission| emission.slot.production == CanonicalSourceProductionV1::Assertion)
        .collect::<Vec<_>>();
    assert_eq!(emissions.len(), 2);
    assert_eq!(emissions[0].slot.local, emissions[1].slot.local);
    assert_eq!(emissions[0].slot.repetition, None);
    assert_eq!(emissions[1].slot.repetition, Some(1));
    assert_ne!(emissions[0].origin, emissions[1].origin);
    assert_eq!(
        cst.source_slice(emissions[0].origin),
        Some(b"Door".as_slice())
    );
    assert_eq!(
        cst.source_slice(emissions[1].origin),
        Some(b"Door".as_slice())
    );
}

#[test]
fn malformed_or_competing_membership_group_forms_reject() {
    for source in [
        "iron-door ∈ Door,\n",
        "iron-door∈ Door\n",
        "iron-door ∈ [Door, Lockable]\n",
    ] {
        assert!(matches!(
            read_canonical_source_v1(source.as_bytes()),
            Err(CanonicalSourceErrorV1::InvalidMembershipGroup { .. })
        ));
    }
}
