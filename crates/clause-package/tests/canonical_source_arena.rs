use std::collections::BTreeSet;

use clause_package::*;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));

const MULTILINE_TEXT_OUTPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/multiline-text-output.clause"
));

const MULTI_REFERENT_SCALAR_HANDLER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/multi-referent-scalar-handler.clause"
));

const STRUCTURED_RELATION_REPLACEMENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/structured-relation-replacement.clause"
));

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

fn nominal_value(compiled: &CanonicalSourcePackageSliceV1, name: &[u8], domain: &[u8]) -> CanonicalScalarValueV1 {
    let identity = |name: &[u8]| compiled.emissions.iter()
        .filter(|emission| emission.producer.production == CanonicalSourceProductionV1::Referent
            && emission.producer.semantic_key == name
            && emission.slot.production == CanonicalSourceProductionV1::Referent)
        .flat_map(|emission| &emission.allocations)
        .find_map(|allocation| match allocation.identity {
            CanonicalAllocatedIdentityV1::Formation(id) => Some(id),
            _ => None,
        }).expect("declared referent has its exact allocated identity");
    CanonicalScalarValueV1::Referent(CanonicalReferentV1 { domain: identity(domain), identity: identity(name) })
}

#[test]
fn scalar_handler_lowers_one_deterministic_rule_per_referent() {
    let compiled = compile_source(MULTI_REFERENT_SCALAR_HANDLER, 49)
        .expect("one typed scalar handler specializes every referent row");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"carry-loot")
        .expect("the multi-referent scalar handler is executable");
    assert_eq!(handler.trigger, CanonicalHandlerTriggerV1::FixedTick);
    assert_eq!(handler.rules.len(), 2);

    let targets = handler
        .rules
        .iter()
        .map(|rule| {
            let [assignment] = rule.assignments.as_slice() else {
                panic!("each referent case owns one scalar assignment")
            };
            let target = &assignment.target;
            assert!(rule.predicates.iter().any(|predicate| matches!(
                predicate,
                CanonicalExecutablePredicateV1::Equal(
                    CanonicalExecutableExpressionV1::State(state),
                    CanonicalExecutableExpressionV1::Constant(value),
                ) if state.subject == target.subject && *value == nominal_value(&compiled, b"acquired", b"LootState")
            )));
            target.clone()
        })
        .collect::<Vec<_>>();
    assert!(targets.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        targets
            .iter()
            .map(|target| target.subject.as_slice())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([b"ashen-key".as_slice(), b"cephorium-cache".as_slice()]),
    );

    let duplicate = MULTI_REFERENT_SCALAR_HANDLER.replacen(
        "cephorium-cache carried distance 2.0",
        "ashen-key carried distance 3.0",
        1,
    );
    assert!(matches!(
        compile_source(&duplicate, 50),
        Err(CanonicalSourceErrorV1::AmbiguousScalarInitialAssertion { .. })
    ));
}

#[test]
fn structured_handler_copies_one_typed_value_per_referent_atomically() {
    let compiled = compile_source(STRUCTURED_RELATION_REPLACEMENT, 51)
        .expect("one structured handler specializes every joined referent row");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"reset-loot-position")
        .expect("the structured replacement handler is executable");
    assert_eq!(handler.trigger, CanonicalHandlerTriggerV1::External);
    assert_eq!(handler.rules.len(), 2);

    let subjects = handler
        .rules
        .iter()
        .map(|rule| {
            assert_eq!(rule.assignments.len(), 3);
            let target_subjects = rule
                .assignments
                .iter()
                .map(|assignment| assignment.target.subject.as_slice())
                .collect::<BTreeSet<_>>();
            if target_subjects.len() != 1 {
                panic!("one atomic rule only mutates one joined referent")
            }
            let subject = target_subjects
                .iter()
                .next()
                .copied()
                .expect("one target subject was established");
            assert!(rule.assignments.iter().all(|assignment| matches!(
                &assignment.value,
                CanonicalExecutableExpressionV1::State(source)
                    if source.subject.as_slice() == subject
                        && source.relation_designation == b"loot-origin-position"
            )));
            subject
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        subjects,
        BTreeSet::from([b"ashen-key".as_slice(), b"cephorium-cache".as_slice()]),
    );
}

const SCALAR_PARAMETER_WORLD: &str = r#"F64
Bool
Player
Enemy
CombatState
alive
telegraph

Vec3:
  ?example:
    x: F64
    y: F64
    z: F64

score:
  ?example:
    player: Player
    score: F64
    ?player:
      score: ?score

mode score given player yields score: one

pressure-clock:
  ?example:
    enemy: Enemy
    pressure-clock: F64
    ?enemy:
      pressure clock: ?pressure-clock

mode pressure-clock given enemy yields pressure-clock: one

pressure-state:
  ?example:
    enemy: Enemy
    pressure-state: CombatState
    ?enemy:
      pressure state: ?pressure-state

mode pressure-state given enemy yields pressure-state: one

grounded:
  ?example:
    player: Player
    grounded: Bool
    ?player:
      grounded: ?grounded

mode grounded given player yields grounded: one

spawn-position:
  ?example:
    enemy: Enemy
    spawn-position: Vec3
    ?enemy:
      spawn position: ?spawn-position

mode spawn-position given enemy yields spawn-position: one

player-1
  member of: Player
cinder-wraith
  member of: Enemy
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

score:
  ?example:
    player: Player
    score: F64
    ?player:
      score: ?score

mode score given player yields score: one

reserve:
  ?example:
    player: Player
    reserve: F64
    ?player:
      reserve: ?reserve

mode reserve given player yields reserve: one

player-1
  member of: Player
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

phase:
  ?example:
    root: Root
    phase: Text
    ?root:
      phase: ?phase

mode phase given root yields phase: one

known-command:
  ?example:
    root: Root
    known-command: Command
    ?root:
      known command: ?known-command

mode known-command given root yields known-command: many

command-description:
  ?example:
    command: Command
    command-description: Text
    ?command:
      description: ?command-description

mode command-description given command yields command-description: maybe

root-main
  member of: Root
root-main phase "ready"

on initialize ?root
  when
    ?root phase ?phase
  create
    ?command
      member of: Command
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

balance:
  ?example:
    root: Root
    balance: F64
    ?root:
      balance: ?balance

mode balance given root yields balance: one

selected-policy:
  ?example:
    root: Root
    policy: Policy
    ?root:
      selected policy: ?policy

mode selected-policy given root yields policy: one

policy-adjustment:
  ?example:
    policy: Policy
    policy-adjustment: F64
    ?policy:
      policy adjustment: ?policy-adjustment

mode policy-adjustment given policy yields policy-adjustment: one

root-1
  member of: Root
policy-a
  member of: Policy
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

const MULTI_SUBJECT_BOOLEAN_LAW_WORLD: &str = r#"F64
Bool
Player
Projectile
ProjectileState
ProjectileFaction
Arena

Vec3:
  ?example:
    x: F64
    y: F64
    z: F64

player-position:
  ?example:
    player: Player
    player-position: Vec3
    ?player:
      player position: ?player-position

mode player-position given player yields player-position: one

projectile-position:
  ?example:
    projectile: Projectile
    projectile-position: Vec3
    ?projectile:
      projectile position: ?projectile-position

mode projectile-position given projectile yields projectile-position: one

projectile-state:
  ?example:
    projectile: Projectile
    projectile-state: ProjectileState
    ?projectile:
      projectile state: ?projectile-state

mode projectile-state given projectile yields projectile-state: one

projectile-faction:
  ?example:
    projectile: Projectile
    projectile-faction: ProjectileFaction
    ?projectile:
      projectile faction: ?projectile-faction

mode projectile-faction given projectile yields projectile-faction: one

contact-radius:
  ?example:
    arena: Arena
    contact-radius: F64
    ?arena:
      contact radius: ?contact-radius

mode contact-radius given arena yields contact-radius: one

hostile-contact:
  ?example:
    player: Player
    hostile-contact: Bool
    ?player:
      hostile contact: ?hostile-contact

mode hostile-contact given player yields hostile-contact: one

law projectile-contact
  if
    ?player player position Vec3 { x: ?player-x, y: ?player-y, z: ?player-z }
    ?projectile projectile position Vec3 { x: ?projectile-x, y: ?projectile-y, z: ?projectile-z }
    ?projectile projectile state ?projectile-state
    ?projectile projectile faction enemy-origin
    combat-arena contact radius ?contact-radius
    ?projectile-state = flight
    ((?player-x - ?projectile-x) * (?player-x - ?projectile-x)) + ((?player-z - ?projectile-z) * (?player-z - ?projectile-z)) <= ?contact-radius * ?contact-radius
  then
    ?player hostile contact true

derive projectile-contact

player-1
  member of: Player
cinder-bolt
  member of: Projectile
wayfarer-bolt
  member of: Projectile
dormant
  member of: ProjectileState
flight
  member of: ProjectileState
enemy-origin
  member of: ProjectileFaction
player-origin
  member of: ProjectileFaction
combat-arena
  member of: Arena

player-1 player position Vec3 { x: 0.0, y: 0.0, z: 0.0 }
cinder-bolt projectile position Vec3 { x: 1.0, y: 0.0, z: 0.0 }
cinder-bolt projectile state flight
cinder-bolt projectile faction enemy-origin
wayfarer-bolt projectile position Vec3 { x: 2.0, y: 0.0, z: 0.0 }
wayfarer-bolt projectile state dormant
wayfarer-bolt projectile faction player-origin
combat-arena contact radius 0.6
"#;

const MULTI_SHAPE_REFERENT_WORLD: &str = r#"Door
Lockable
iron-door
  member of: Door
  member of: Lockable
"#;

const EXPLICIT_APPLICATIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/explicit-applications.clause"
));

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
    compile_source(MULTI_SHAPE_REFERENT_WORLD, 39)
        .expect("an explicit referent and its memberships share one identity");
    compile_source(
        "Door\nLockable\n\niron-door\niron-door\n  member of: Door\n  member of: Lockable\n",
        40,
    )
    .expect("subject focus may reuse an explicitly declared referent");
}

#[test]
fn one_referent_retains_explicit_semantic_applications() {
    let compiled = compile_source(EXPLICIT_APPLICATIONS, 42)
        .expect("shape, numeric, and Text applications elaborate together");
    assert_eq!(
        compiled
            .applications
            .iter()
            .map(|application| (
                application.subject.clone(),
                application.role.clone(),
                application.object.clone(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                b"north".to_vec(),
                b"shape".to_vec(),
                CanonicalScalarValueV1::Symbol(b"Flake".to_vec())
            ),
            (
                b"north".to_vec(),
                b"priority".to_vec(),
                CanonicalScalarValueV1::Number(5.0_f64.to_bits())
            ),
            (
                b"north".to_vec(),
                b"greeting".to_vec(),
                CanonicalScalarValueV1::Text("hello".into())
            ),
        ]
    );
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
fn multiline_text_uses_the_closing_delimiter_as_its_explicit_margin() {
    let compiled = compile_source(MULTILINE_TEXT_OUTPUT, 42)
        .expect("the multiline Text source reaches executable lowering");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"render")
        .expect("the multiline Text handler is executable");
    let [rule] = handler.rules.as_slice() else {
        panic!("one multiline Text transition remains one executable rule")
    };
    let [assignment] = rule.assignments.as_slice() else {
        panic!("one multiline Text replacement remains one assignment")
    };
    assert_eq!(
        assignment.value,
        CanonicalExecutableExpressionV1::Constant(CanonicalScalarValueV1::Text(
            "{\n  title = \"North\";\n  outputs = { nixpkgs, ... }: \"Clause emits readable text\";\n}\n".into()
        ))
    );
}

#[test]
fn multiline_text_rejects_content_left_of_its_explicit_margin() {
    let source =
        MULTILINE_TEXT_OUTPUT.replacen("      {\n        title", "    {\n        title", 1);
    assert!(matches!(
        compile_source(&source, 43),
        Err(CanonicalSourceErrorV1::InvalidMultilineText { .. })
    ));
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
        "policy-a policy adjustment 2.0\npolicy-b\n  member of: Policy\npolicy-b policy adjustment 4.0",
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
                CanonicalExecutableExpressionV1::Constant(expected),
            ) if state.relation_designation == b"selected-policy" => Some(expected.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected,
        BTreeSet::from([nominal_value(&compiled, b"policy-a", b"Policy"), nominal_value(&compiled, b"policy-b", b"Policy")]),
    );
}

#[test]
fn boolean_law_lowers_typed_multi_subject_selector_cases() {
    let compiled = compile_source(MULTI_SUBJECT_BOOLEAN_LAW_WORLD, 42)
        .expect("a typed faction selector specializes every projectile contact case");
    let matching = compiled
        .executable_handlers
        .iter()
        .filter(|handler| handler.designation == b"projectile-contact")
        .collect::<Vec<_>>();
    let [handler] = matching.as_slice() else {
        panic!("one logical Boolean derivation owns all projectile cases")
    };
    assert_eq!(handler.trigger, CanonicalHandlerTriggerV1::FixedTickDerived);
    let [rule] = handler.rules.as_slice() else { panic!("one derived value assignment") };
    let [assignment] = rule.assignments.as_slice() else { panic!("one Boolean result") };
    assert_eq!(assignment.target.relation_designation, b"hostile-contact");
    assert!(rule.predicates.is_empty(), "false must also update the derived value");
    let CanonicalExecutableExpressionV1::MatchesAny(cases) = &assignment.value else {
        panic!("the derived result tests every case and is false when none match")
    };
    assert_eq!(cases.len(), 2, "both projectile cases survive lowering");
    let selected_projectiles = cases.iter().flatten()
        .filter_map(|predicate| match predicate {
            CanonicalExecutablePredicateV1::Equal(
                CanonicalExecutableExpressionV1::State(state),
                CanonicalExecutableExpressionV1::Constant(expected),
            ) if state.relation_designation == b"projectile-faction"
                && *expected == nominal_value(&compiled, b"enemy-origin", b"ProjectileFaction") =>
            {
                Some(state.subject.as_slice())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected_projectiles,
        BTreeSet::from([b"cinder-bolt".as_slice(), b"wayfarer-bolt".as_slice()]),
        "the runtime selector remains explicit for every typed projectile"
    );
    let derived = compiled
        .state_cells
        .iter()
        .filter(|cell| {
            cell.state.relation_designation == b"hostile-contact"
                && cell.state.subject == b"player-1"
        })
        .collect::<Vec<_>>();
    assert_eq!(
        derived.len(),
        1,
        "the cases share one derived Boolean state"
    );
    assert_eq!(
        derived[0].initial_value,
        Some(CanonicalScalarValueV1::Boolean(false))
    );
}

#[test]
fn boolean_law_selector_rejects_wrong_domain_missing_value_and_non_singleton_state() {
    let wrong_domain = MULTI_SUBJECT_BOOLEAN_LAW_WORLD.replacen(
        "?projectile projectile faction enemy-origin",
        "?projectile projectile faction flight",
        1,
    );
    assert!(matches!(
        compile_source(&wrong_domain, 43),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));

    let missing = MULTI_SUBJECT_BOOLEAN_LAW_WORLD.replacen(
        "?projectile projectile faction enemy-origin",
        "?projectile projectile faction unlisted-origin",
        1,
    );
    assert!(matches!(
        compile_source(&missing, 44),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));

    let nonsingleton = MULTI_SUBJECT_BOOLEAN_LAW_WORLD.replacen(
        "mode projectile-faction given projectile yields projectile-faction: one",
        "mode projectile-faction given projectile yields projectile-faction: many",
        1,
    );
    assert!(matches!(
        compile_source(&nonsingleton, 45),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));
}

#[test]
fn boolean_law_selector_preserves_ambiguous_relation_rejection() {
    let ambiguous = MULTI_SUBJECT_BOOLEAN_LAW_WORLD.replacen(
        "contact-radius:\n",
        "alternate-projectile-faction:\n  ?example:\n    projectile: Projectile\n    faction: ProjectileFaction\n    ?projectile:\n      projectile faction: ?faction\n\nmode alternate-projectile-faction given projectile yields faction: one\n\ncontact-radius:\n",
        1,
    );
    assert!(matches!(
        compile_source(&ambiguous, 46),
        Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { .. })
    ));
}

#[test]
fn general_handler_lowers_typed_constant_state_selector() {
    let source = format!(
        "{MULTI_SUBJECT_BOOLEAN_LAW_WORLD}\n\non reset-hostile-projectile ?projectile\n  when\n    ?projectile projectile state ?state\n    ?projectile projectile faction enemy-origin\n    ?state = flight\n  withdraw\n    ?projectile projectile state ?state\n  include\n    ?projectile projectile state dormant\n"
    );
    let compiled = compile_source(&source, 47)
        .expect("a typed constant state selector lowers in a general handler");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"reset-hostile-projectile")
        .expect("the selected general handler is executable");
    let selected_projectiles = handler
        .rules
        .iter()
        .flat_map(|rule| &rule.predicates)
        .filter_map(|predicate| match predicate {
            CanonicalExecutablePredicateV1::Equal(
                CanonicalExecutableExpressionV1::State(state),
                CanonicalExecutableExpressionV1::Constant(expected),
            ) if state.relation_designation == b"projectile-faction"
                && *expected == nominal_value(&compiled, b"enemy-origin", b"ProjectileFaction") =>
            {
                Some(state.subject.as_slice())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected_projectiles,
        BTreeSet::from([b"cinder-bolt".as_slice(), b"wayfarer-bolt".as_slice()]),
        "the selector remains an explicit typed predicate for each handler subject"
    );

    let wrong_domain = source.replacen(
        "?projectile projectile faction enemy-origin",
        "?projectile projectile faction flight",
        1,
    );
    assert!(matches!(
        compile_source(&wrong_domain, 48),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));
}

#[test]
fn scalar_law_result_updates_a_vector_on_the_selected_handler_subject() {
    let source = r#"F64
Bool
Player
Enemy

Vec3:
  ?example:
    x: F64
    y: F64
    z: F64

clamped-between:
  ?example:
    value: F64
    lower: F64
    upper: F64
    result: F64
    ?value clamped between ?lower and ?upper as ?result

mode clamped-between given value lower upper yields result: maybe

combat-target:
  ?example:
    player: Player
    enemy: Enemy
    ?player:
      combat target: ?enemy

mode combat-target given player yields enemy: one

target-active:
  ?example:
    player: Player
    target-active: Bool
    ?player:
      target active: ?target-active

mode target-active given player yields target-active: one

vitals:
  ?example:
    enemy: Enemy
    vitals: Vec3
    ?enemy:
      vitals: ?vitals

mode vitals given enemy yields vitals: one

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

player-1
  member of: Player
enemy-1
  member of: Enemy
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
    assert_eq!(
        handler.rules.len(),
        3,
        "each authored law supplies one guarded case"
    );
    for rule in &handler.rules {
        assert_eq!(
            rule.assignments.len(),
            1,
            "only the changed vitality field is assigned"
        );
        assert_eq!(
            rule.law_origins.len(),
            2,
            "law and derivation remain inspectable"
        );
        assert!(
            rule.law_origins
                .iter()
                .all(|origin| cst.source_slice(*origin).is_some())
        );
        assert!(!rule.predicates.is_empty());
    }
}

#[test]
fn transitive_referent_join_rejects_wrong_type_missing_cardinality_and_ambiguity() {
    let wrong_type = TRANSITIVE_REFERENT_WORLD.replacen(
        "    policy: Policy",
        "    policy: Root",
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
        "mode selected-policy given root yields policy: one",
        "mode selected-policy given root yields policy: many",
        1,
    );
    assert!(matches!(
        compile_source(&nonsingleton, 21),
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. })
    ));

    let ambiguous = TRANSITIVE_REFERENT_WORLD.replacen(
        "policy-a policy adjustment 2.0\n\non apply-selected-policy",
        "policy-a policy adjustment 2.0\nroot-2\n  member of: Root\nroot-2 balance 8.0\nroot-2 selected policy policy-a\n\non apply-selected-policy",
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
fn general_tick_handler_specializes_every_subject_and_receives_delta_time() {
    let source = r#"F64
Unit

clock:
  ?example:
    unit: Unit
    clock: F64
    ?unit:
      clock: ?clock

mode clock given unit yields clock: one

unit-a
  member of: Unit
unit-b
  member of: Unit
unit-a clock 0.0
unit-b clock 2.0

on tick ?unit ?dt
  when
    ?unit clock ?clock
  withdraw
    ?unit clock ?clock
  include
    ?unit clock ?clock + ?dt
"#;
    let compiled = compile_source(source, 50)
        .expect("a general tick handler lowers for every matching subject");
    let handler = compiled
        .executable_handlers
        .iter()
        .find(|handler| handler.designation == b"tick")
        .expect("the general tick handler is executable");
    assert_eq!(handler.trigger, CanonicalHandlerTriggerV1::FixedTick);
    assert_eq!(handler.argument_count, 1);
    assert_eq!(handler.rules.len(), 2);
    assert!(handler.rules.iter().all(|rule| matches!(
        &rule.assignments[0].value,
        CanonicalExecutableExpressionV1::Add(_, argument)
            if argument.as_ref() == &CanonicalExecutableExpressionV1::Argument(0)
    )));
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

recovery-clock:
  ?example:
    player: Player
    recovery-clock: F64
    ?player:
      recovery clock: ?recovery-clock

mode recovery-clock given player yields recovery-clock: one

recovery-rate:
  ?example:
    player: Player
    recovery-rate: F64
    ?player:
      recovery rate: ?recovery-rate

mode recovery-rate given player yields recovery-rate: one

heat:
  ?example:
    player: Player
    heat: F64
    ?player:
      heat: ?heat

mode heat given player yields heat: one

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
fn tick_rules_accept_typed_state_equality_guards() {
    let source = std::str::from_utf8(WORLD)
        .expect("canonical arena source is UTF-8")
        .replacen(
            "position:\n",
            "reset-gate:\n  ?example:\n    arena: Arena\n    gate: Vec3\n    ?arena:\n      reset gate: ?gate\n\nmode reset-gate given arena yields gate: one\n\nposition:\n",
            1,
        )
        .replacen(
            "jump-arena gravity -8.0",
            "jump-arena reset gate Vec3 { x: 0.0, y: 0.0, z: 0.0 }\njump-arena gravity -8.0",
            1,
        )
        .replace(
            "    ?dt > 0.0\n",
            "    ?dt > 0.0\n    jump-arena reset gate Vec3 { x: ?reset, y: ?reset-phase, z: ?reset-unused }\n    ?reset = 0.0\n",
        );
    assert_eq!(source.matches("    ?reset = 0.0\n").count(), 3);

    let compiled =
        compile_source(&source, 42).expect("fixed-tick rules may depend on a typed state equality");
    let tick = compiled
        .tick_program
        .expect("the guarded physics profile remains a checked tick program");
    assert!(tick.rules.iter().all(|rule| {
        rule.predicates.iter().any(|predicate| {
            matches!(
                predicate,
                CanonicalTickPredicateV1::EqualState {
                    relation,
                    field: Some(field),
                    expected: CanonicalScalarValueV1::Number(value),
                    ..
                } if relation == b"reset gate" && field == b"x" && *value == 0.0f64.to_bits()
            )
        })
    }));
    assert_eq!(
        compiled
            .executable_handlers
            .iter()
            .filter(|handler| {
                handler.designation == b"tick"
                    && handler.trigger == CanonicalHandlerTriggerV1::FixedTickRoot
            })
            .count(),
        3,
    );
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
    assert_eq!(
        compiled
            .applications
            .iter()
            .map(|application| (
                application.subject.clone(),
                application.role.clone(),
                application.object.clone(),
                cst.source_slice(application.origin)
                    .expect("owned application origin")
                    .to_vec(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                b"jump-arena".to_vec(),
                b"member of".to_vec(),
                CanonicalScalarValueV1::Symbol(b"Arena".to_vec()),
                b"  member of: Arena".to_vec(),
            ),
            (
                b"player-1".to_vec(),
                b"member of".to_vec(),
                CanonicalScalarValueV1::Symbol(b"Player".to_vec()),
                b"  member of: Player".to_vec(),
            ),
        ]
    );
    let application_origins = compiled
        .applications
        .iter()
        .map(|application| application.origin)
        .collect::<Vec<_>>();
    let application_emissions = compiled
        .emissions
        .iter()
        .filter(|emission| application_origins.contains(&emission.origin))
        .collect::<Vec<_>>();
    assert_eq!(application_emissions.len(), 2);
    assert!(
        application_emissions
            .iter()
            .all(
                |emission| emission.slot.production == CanonicalSourceProductionV1::Assertion
                    && emission.slot.repetition.is_none()
            )
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
    assert_eq!(
        tick.rules.len(),
        27,
        "three transitions each compose two three-case relations"
    );
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
            .chain(&tick.law_origins)
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
fn repeated_roles_preserve_application_order_and_origins() {
    let source = "Door\nLockable\niron-door\n  member of: Door\n  member of: Lockable\n";
    let cst = read_canonical_source_v1(source.as_bytes()).expect("application source reads");
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
    .expect("the applications reach bounded elaboration");

    assert!(compiled.unsupported.is_empty());
    assert_eq!(compiled.applications.len(), 2);
    assert_eq!(compiled.applications[0].role, b"member of".to_vec());
    assert_eq!(
        compiled.applications[0].object,
        CanonicalScalarValueV1::Symbol(b"Door".to_vec())
    );
    assert_eq!(
        compiled.applications[1].object,
        CanonicalScalarValueV1::Symbol(b"Lockable".to_vec())
    );
    let emissions = compiled
        .emissions
        .iter()
        .filter(|emission| emission.slot.production == CanonicalSourceProductionV1::Assertion)
        .collect::<Vec<_>>();
    assert_eq!(emissions.len(), 2);
    assert_eq!(emissions[0].producer, emissions[1].producer);
    assert_ne!(emissions[0].slot.local, emissions[1].slot.local);
    assert_eq!(emissions[0].slot.repetition, None);
    assert_eq!(emissions[1].slot.repetition, None);
    assert_eq!(
        cst.source_slice(emissions[0].origin),
        Some(b"  member of: Door".as_slice())
    );
    assert_eq!(
        cst.source_slice(emissions[1].origin),
        Some(b"  member of: Lockable".as_slice())
    );
    assert!(
        emissions
            .iter()
            .all(|emission| emission.allocations.is_empty())
    );
}

#[test]
fn repeated_application_is_not_deduplicated() {
    let source = "Door\niron-door\n  member of: Door\n  member of: Door\n";
    let cst = read_canonical_source_v1(source.as_bytes()).expect("repeated applications read");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(8)),
    )
    .expect("repeated applications do not collapse emission inputs");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("repeated applications reach bounded elaboration");
    assert_eq!(compiled.applications.len(), 2);
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
        Some(b"  member of: Door".as_slice())
    );
    assert_eq!(
        cst.source_slice(emissions[1].origin),
        Some(b"  member of: Door".as_slice())
    );
}

#[test]
fn duplicate_declared_readings_are_an_explicit_ambiguity() {
    let declared_frontend = br#"Role
Term

first-edge:
  ?example:
    relation: Role
    object: Term
    ?relation:
      : ?object

mode first-edge given relation yields object: one

second-edge:
  ?example:
    relation: Role
    object: Term
    ?relation:
      : ?object

mode second-edge given relation yields object: one
"#;
    let frontend = CanonicalDeclaredFrontendV1::read(declared_frontend)
        .expect("both Readings are individually valid");
    let source = b"Door\niron-door\n  member of: Door\n";
    assert!(matches!(
        read_canonical_source_with_declared_frontend_v1(source, &frontend),
        Err(CanonicalSourceErrorV1::AmbiguousDeclaredProduction { .. })
    ));
    let grouped = b"Door\niron-door\n  member of\n    Door\n";
    assert!(matches!(
        read_canonical_source_with_declared_frontend_v1(grouped, &frontend),
        Err(CanonicalSourceErrorV1::AmbiguousDeclaredProduction { .. })
    ));
}

#[test]
fn noncanonical_denotation_forms_reject() {
    for source in [
        "iron-door: Door,\n",
        "iron-door∈ Door\n",
        "iron-door ∈ Door\n",
        "iron-door:\n  Door\n  Lockable\n",
    ] {
        assert!(matches!(
            read_canonical_source_v1(source.as_bytes()),
            Err(CanonicalSourceErrorV1::InvalidDenotation { .. })
        ));
    }
}

#[test]
fn denotation_is_compositional_and_never_selected_by_focus_children() {
    let nested = compile_source("pair: (1, 2), (3, \"comma, literal\")\n", 31).unwrap();
    let CanonicalSourceDenotedValueV1::OrderedProduct(members) = &nested.denotations[0].value
    else {
        panic!("outer product")
    };
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|value| matches!(value, CanonicalSourceDenotedValueV1::OrderedProduct(items) if items.len() == 2)));
    let flat = compile_source("pair: 1, 2, 3, \"comma, literal\"\n", 31).unwrap();
    assert_ne!(nested.denotations[0].value, flat.denotations[0].value);
    assert_ne!(nested.emissions[0].slot, flat.emissions[0].slot);
    for source in ["five:5\n", "five : 5\n", "five:   (5)\n"] {
        let compiled = compile_source(source, 31).unwrap();
        assert_eq!(
            compiled.denotations[0].value,
            CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Number(
                5.0_f64.to_bits()
            ))
        );
    }
    for source in ["rgb\n  255, 0, 0\n", "pair: (1, 2]\n", "pair: (1,,2), 3\n"] {
        assert!(read_canonical_source_v1(source.as_bytes()).is_err());
    }
}

#[test]
fn scalar_and_ordered_product_denotations_remain_distinct() {
    let source = "five: 5\npair: 5, \"hello\"\nrgb: 255, 0, 0\n";
    let cst = read_canonical_source_v1(source.as_bytes()).expect("denotations read");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(9)),
    )
    .expect("denotations need no implicit semantic roles");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("denotations elaborate");

    assert_eq!(
        compiled.denotations,
        vec![
            CanonicalSourceDenotationV1 {
                name: b"five".to_vec(),
                value: CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Number(
                    5.0_f64.to_bits()
                ),),
                origin: compiled.denotations[0].origin,
            },
            CanonicalSourceDenotationV1 {
                name: b"pair".to_vec(),
                value: CanonicalSourceDenotedValueV1::OrderedProduct(vec![
                    CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Number(
                        5.0_f64.to_bits()
                    )),
                    CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Text(
                        "hello".into()
                    )),
                ]),
                origin: compiled.denotations[1].origin,
            },
            CanonicalSourceDenotationV1 {
                name: b"rgb".to_vec(),
                value: CanonicalSourceDenotedValueV1::OrderedProduct(vec![
                    CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Number(
                        255.0_f64.to_bits()
                    )),
                    CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Number(
                        0.0_f64.to_bits()
                    )),
                    CanonicalSourceDenotedValueV1::Scalar(CanonicalScalarValueV1::Number(
                        0.0_f64.to_bits()
                    )),
                ]),
                origin: compiled.denotations[2].origin,
            },
        ]
    );
    let origins = compiled
        .emissions
        .iter()
        .filter(|emission| emission.slot.production == CanonicalSourceProductionV1::Assertion)
        .map(|emission| {
            cst.source_slice(emission.origin)
                .expect("each denoted member retains its exact source")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        origins,
        vec![
            b"5".as_slice(),
            b"5".as_slice(),
            b"\"hello\"".as_slice(),
            b"255".as_slice(),
            b"0".as_slice(),
            b"0".as_slice(),
        ]
    );
}
