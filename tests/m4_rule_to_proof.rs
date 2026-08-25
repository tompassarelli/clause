//! Canonical M4/S2 positive-rule surface through existing proof operations.

use clause::{
    elaborate, frontend, generated,
    intervention::{AchieveAll, PreventAll},
    kernel::{ReferentId, RelationalContent, Revision, RoleId, Term},
    request::{self, Request, RequestOutput},
};
use std::{collections::BTreeMap, env, fs, path::PathBuf, process::Command};

const LEGACY_SOURCE: &str = include_str!("../examples/hospital.clause");

const LEGACY_RULES: &str = "egress/direct-route: DerivationRule
  ?origin has a usable egress path to ?destination
  when:
    ?door connects ?origin to ?destination
    ?door passed Fire-Marshal-Inspection

egress/recursive-route: DerivationRule
  ?origin has a usable egress path to ?destination
  when:
    ?door connects ?origin to ?intermediate
    ?door passed Fire-Marshal-Inspection
    ?intermediate has a usable egress path to ?destination";

const LEGACY_FIND: &str = "find all ?destination in egress:
  ICU-A has a usable egress path to ?destination";

const RULE_SCOPE_SHAPES: &str = "Entity
Edge

graph/connects: RelationShape
  {edge: Edge} connects {origin: Entity} to {destination: Entity}
  mode edge, origin -> destination: many

graph/reaches: RelationShape
  {origin: Entity} reaches {destination: Entity}
  mode origin -> destination: many";

const SCOPE_RULE: &str = "?origin reaches ?destination if
  ?edge connects ?origin to ?destination";

const RULE_LITERAL_SOURCE: &str = "Text
Entity

tags/has: RelationShape
  {entity: Entity} has tag {tag: Text}
  mode entity -> tag: many

catalog
  Item ∈ Entity

?entity has tag \"canonical\" if
  ?entity has tag \"premise\"
";

const RULE_IDENTITY_BOUND_SOURCE: &str = "Thing

wide/links: RelationShape
  {a: Thing} links {b: Thing} through-c {c: Thing} through-d {d: Thing} through-e {e: Thing} through-f {f: Thing} through-g {g: Thing} through-h {h: Thing} through-i {i: Thing}
  mode a -> b, c, d, e, f, g, h, i: many

wide/reaches: RelationShape
  {a: Thing} reaches {b: Thing}
  mode a -> b: many

world
  item ∈ Thing

?a reaches ?b if
  ?a links ?b through-c ?c through-d ?d through-e ?e through-f ?f through-g ?g through-h ?h through-i ?i
";

fn canonical_source(label: Option<&str>) -> String {
    let recursive = match label {
        Some(label) => format!(
            "{label}:
  ?origin has a usable egress path to ?destination if
    ?door connects ?origin to ?intermediate
    ?door passed Fire-Marshal-Inspection
    ?intermediate has a usable egress path to ?destination"
        ),
        None => "?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?intermediate
  ?door passed Fire-Marshal-Inspection
  ?intermediate has a usable egress path to ?destination"
            .to_owned(),
    };
    let rules = format!(
        "?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?destination
  ?door passed Fire-Marshal-Inspection

{recursive}"
    );
    LEGACY_SOURCE.replace(LEGACY_RULES, &rules).replace(
        LEGACY_FIND,
        "ICU-A has a usable egress path to ?destination",
    )
}

fn compile(source: &str) -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(source).expect("M4/S2 source parses"))
        .expect("M4/S2 source compiles")
}

fn revision(program: &elaborate::CompiledProgram, name: &str) -> Revision {
    program
        .revision(&frontend::Name(name.to_owned()))
        .expect("named Revision resolves")
        .clone()
}

fn rule_ids(revision: &Revision) -> Vec<ReferentId> {
    revision
        .model()
        .derivation_rules()
        .iter()
        .map(|rule| rule.id().clone())
        .collect()
}

fn relation(program: &elaborate::CompiledProgram, value: &str) -> ReferentId {
    program
        .designations()
        .global(value)
        .expect("relation designation resolves")
}

fn role(program: &elaborate::CompiledProgram, relation: &ReferentId, value: &str) -> RoleId {
    program
        .designations()
        .role(relation, value)
        .expect("role designation resolves")
}

fn referent(program: &elaborate::CompiledProgram, revision: &Revision, local: &str) -> ReferentId {
    program
        .designations()
        .scoped(revision.model().id(), local)
        .expect("hospital referent designation resolves")
}

fn assertion(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    relation_name: &str,
    roles: &[(&str, &str)],
) -> RelationalContent {
    let relation = relation(program, relation_name);
    RelationalContent::new(
        relation.clone(),
        roles
            .iter()
            .map(|(role_name, local)| {
                (
                    role(program, &relation, role_name),
                    Term::referent(referent(program, revision, local)),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("typed hospital assertion")
}

fn connects(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    door: &str,
    origin: &str,
    destination: &str,
) -> RelationalContent {
    assertion(
        program,
        revision,
        "egress/connects",
        &[
            ("door", door),
            ("origin", origin),
            ("destination", destination),
        ],
    )
}

fn passed(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    door: &str,
) -> RelationalContent {
    assertion(
        program,
        revision,
        "egress/passed",
        &[("door", door), ("inspection", "Fire-Marshal-Inspection")],
    )
}

fn canonical_sets(mut alternatives: Vec<Vec<RelationalContent>>) -> Vec<Vec<RelationalContent>> {
    for members in &mut alternatives {
        members.sort();
    }
    alternatives.sort();
    alternatives
}

fn support_sets(why: &clause::execution::WhyAll) -> Vec<Vec<RelationalContent>> {
    why.alternatives
        .iter()
        .map(|alternative| alternative.assertions.clone())
        .collect()
}

fn withdrawals(result: &PreventAll) -> Vec<Vec<RelationalContent>> {
    let PreventAll::Complete(items) = result else {
        panic!("prevent all must exhaust its finite frontier: {result:?}");
    };
    items
        .iter()
        .map(|item| item.withdrawals().to_vec())
        .collect()
}

fn additions(result: &AchieveAll) -> Vec<Vec<RelationalContent>> {
    let AchieveAll::Complete(items) = result else {
        panic!("achieve all must exhaust its finite frontier: {result:?}");
    };
    items
        .iter()
        .map(|item| item.admissions().to_vec())
        .collect()
}

fn temporary(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "clause-m4-rule-to-proof-{}.{}",
        std::process::id(),
        extension
    ))
}

#[test]
fn canonical_positive_if_rules_drive_existing_proof_and_intervention_semantics() {
    let source = canonical_source(Some("recursive route"));
    let compiled = compile(&source);
    let base = revision(&compiled, "egress");
    let successor = revision(&compiled, "egress/door-101-withdrawn");

    assert_eq!(base.model().derivation_rules().len(), 2);
    assert_eq!(base.model().universal_laws().len(), 2);
    assert!(base.model().derivation_rules().iter().all(|rule| {
        base.model().universal_laws().iter().any(|law| {
            law.id() == rule.governing_law()
                && law.scope() == rule.scope()
                && law.premises() == rule.premises()
                && law.conclusion() == rule.conclusion()
        })
    }));
    assert!(base.model().invariants().is_empty());
    assert!(base.model().goals().is_empty());

    let recursive_rule = base
        .model()
        .derivation_rules()
        .iter()
        .find(|rule| rule.premises().forms().len() == 3)
        .expect("recursive canonical rule is distinct");
    let label = compiled
        .designations()
        .scoped(base.model().id(), "recursive route")
        .expect("optional rule label is a scoped referent");
    assert_eq!(
        base.model()
            .definition(&label)
            .expect("optional rule label lowers as a definition")
            .denotation(),
        &Term::referent(recursive_rule.id().clone())
    );

    let resolved = request::resolve(&compiled).expect("canonical requests resolve");
    assert!(matches!(resolved.requests()[0], Request::Select { .. }));
    let output = request::run(&resolved, request::RunLimits::default())
        .expect("canonical requests execute through existing engines");
    assert_eq!(output.results.len(), 6);

    let RequestOutput::Select { columns, rows } = &output.results[0] else {
        panic!("naked canonical query remains Select");
    };
    assert_eq!(columns.len(), 1);
    assert_eq!(columns[0].label(), Some("destination"));
    let mut destinations = rows
        .iter()
        .map(|row| row.cells()[0].value().clone())
        .collect::<Vec<_>>();
    destinations.sort();
    let mut expected_destinations = vec![
        Term::referent(referent(&compiled, &base, "East-Corridor")),
        Term::referent(referent(&compiled, &base, "North-Exit")),
        Term::referent(referent(&compiled, &base, "West-Corridor")),
    ];
    expected_destinations.sort();
    assert_eq!(destinations, expected_destinations);

    let RequestOutput::WhyAll(Some(why)) = &output.results[1] else {
        panic!("canonical rule proof is complete");
    };
    assert!(why.is_complete());
    let expected_supports = canonical_sets(vec![
        vec![
            connects(&compiled, &base, "Door 101", "ICU-A", "East-Corridor"),
            passed(&compiled, &base, "Door 101"),
            connects(&compiled, &base, "Door 102", "East-Corridor", "North-Exit"),
            passed(&compiled, &base, "Door 102"),
        ],
        vec![
            connects(&compiled, &base, "Door 103", "ICU-A", "West-Corridor"),
            passed(&compiled, &base, "Door 103"),
            connects(&compiled, &base, "Door 104", "West-Corridor", "North-Exit"),
            passed(&compiled, &base, "Door 104"),
        ],
    ]);
    assert_eq!(canonical_sets(support_sets(why)), expected_supports);

    let RequestOutput::PreventAll(base_prevent) = &output.results[2] else {
        panic!("canonical base prevention is complete");
    };
    assert_eq!(
        canonical_sets(withdrawals(base_prevent)),
        canonical_sets(vec![
            vec![
                passed(&compiled, &base, "Door 101"),
                passed(&compiled, &base, "Door 103"),
            ],
            vec![
                passed(&compiled, &base, "Door 101"),
                passed(&compiled, &base, "Door 104"),
            ],
            vec![
                passed(&compiled, &base, "Door 102"),
                passed(&compiled, &base, "Door 103"),
            ],
            vec![
                passed(&compiled, &base, "Door 102"),
                passed(&compiled, &base, "Door 104"),
            ],
        ])
    );

    let RequestOutput::PreventAll(successor_prevent) = &output.results[3] else {
        panic!("canonical successor prevention is complete");
    };
    assert_eq!(
        canonical_sets(withdrawals(successor_prevent)),
        canonical_sets(vec![
            vec![passed(&compiled, &successor, "Door 103")],
            vec![passed(&compiled, &successor, "Door 104")],
        ])
    );

    let RequestOutput::AchieveAll(achieve) = &output.results[4] else {
        panic!("canonical achievement is complete");
    };
    assert_eq!(
        canonical_sets(additions(achieve)),
        canonical_sets(vec![
            vec![passed(&compiled, &base, "Door 105")],
            vec![passed(&compiled, &base, "Door 106")],
        ])
    );

    let legacy = compile(LEGACY_SOURCE);
    let legacy_base = revision(&legacy, "egress");
    let legacy_output = request::run(
        &request::resolve(&legacy).expect("retained legacy requests resolve"),
        request::RunLimits::default(),
    )
    .expect("retained legacy requests execute");
    let RequestOutput::Find(legacy_destinations) = &legacy_output.results[0] else {
        panic!("retained legacy fixture keeps Find until M5");
    };
    assert_eq!(&destinations, legacy_destinations);
    let RequestOutput::WhyAll(Some(legacy_why)) = &legacy_output.results[1] else {
        panic!("retained legacy proof remains complete");
    };
    assert_eq!(
        canonical_sets(support_sets(why)),
        canonical_sets(support_sets(legacy_why))
    );
    let RequestOutput::PreventAll(legacy_base_prevent) = &legacy_output.results[2] else {
        panic!("retained legacy base prevention remains complete");
    };
    assert_eq!(
        canonical_sets(withdrawals(base_prevent)),
        canonical_sets(withdrawals(legacy_base_prevent))
    );
    let RequestOutput::PreventAll(legacy_successor_prevent) = &legacy_output.results[3] else {
        panic!("retained legacy successor prevention remains complete");
    };
    assert_eq!(
        canonical_sets(withdrawals(successor_prevent)),
        canonical_sets(withdrawals(legacy_successor_prevent))
    );
    let RequestOutput::AchieveAll(legacy_achieve) = &legacy_output.results[4] else {
        panic!("retained legacy achievement remains complete");
    };
    assert_eq!(
        canonical_sets(additions(achieve)),
        canonical_sets(additions(legacy_achieve))
    );

    let canonical_ids = rule_ids(&base);
    let legacy_ids = rule_ids(&legacy_base);
    assert!(
        canonical_ids.iter().all(|id| !legacy_ids.contains(id)),
        "hidden canonical rule identities remain distinct from retained legacy names"
    );

    let unlabelled = compile(&canonical_source(None));
    let unlabelled_base = revision(&unlabelled, "egress");
    let renamed_label = compile(&canonical_source(Some("renamed route")));
    let renamed_label_base = revision(&renamed_label, "egress");
    assert_eq!(
        canonical_ids,
        rule_ids(&unlabelled_base),
        "adding or removing an optional label does not change hidden rule identity"
    );
    assert_eq!(
        canonical_ids,
        rule_ids(&renamed_label_base),
        "renaming an optional label does not change hidden rule identity"
    );

    let reordered_source = source.replace(
        "    ?door connects ?origin to ?intermediate
    ?door passed Fire-Marshal-Inspection
    ?intermediate has a usable egress path to ?destination",
        "    ?intermediate has a usable egress path to ?destination
    ?door passed Fire-Marshal-Inspection
    ?door connects ?origin to ?intermediate",
    );
    assert_ne!(
        reordered_source, source,
        "recursive premises were reordered"
    );
    let reordered = compile(&reordered_source);
    let reordered_base = revision(&reordered, "egress");
    assert_eq!(
        rule_ids(&base),
        rule_ids(&reordered_base),
        "unordered premise authoring does not change hidden rule identity"
    );
    assert_eq!(
        base.identity(),
        reordered_base.identity(),
        "unordered premise authoring preserves the complete admitted Revision"
    );

    let alpha_source = canonical_source(None)
        .replace("?origin", "?from")
        .replace("?destination", "?to")
        .replace("?door", "?portal")
        .replace("?intermediate", "?via");
    let alpha = compile(&alpha_source);
    let alpha_base = revision(&alpha, "egress");
    assert_eq!(
        rule_ids(&unlabelled_base),
        rule_ids(&alpha_base),
        "rule identity is independent of binder spelling"
    );

    let unrestricted = source.replacen(
        "?origin has a usable egress path to ?destination if",
        "?origin has a usable egress path to ?unbound if",
        1,
    );
    let error = frontend::parse(&unrestricted).expect_err("rule conclusions are range-restricted");
    assert_eq!(
        error.message,
        "derivation rule conclusion variables must be range-restricted by if"
    );

    let anonymous = source.replacen(
        "?origin has a usable egress path to ?destination if",
        "? has a usable egress path to ?destination if",
        1,
    );
    let error = frontend::parse(&anonymous).expect_err("rule holes must be named");
    assert_eq!(
        error.message,
        "anonymous holes are only valid in naked queries"
    );

    let qualified_label = canonical_source(Some("egress/recursive route"));
    let error = frontend::parse(&qualified_label).expect_err("rule labels are Model-local");
    assert_eq!(error.message, "derivation rule label must be unqualified");

    let no_model = format!("{RULE_SCOPE_SHAPES}\n\n{SCOPE_RULE}\n");
    let error = frontend::parse(&no_model).expect_err("rule scope cannot be implicit");
    assert_eq!(
        error.message,
        "derivation rule matches no declared Model; candidates: <none>"
    );
    let ambiguous_models = format!(
        "{RULE_SCOPE_SHAPES}\n\nfirst\n  A ∈ Entity\n\nsecond\n  B ∈ Entity\n\n{SCOPE_RULE}\n"
    );
    let error = frontend::parse(&ambiguous_models).expect_err("rule scope must be unique");
    assert_eq!(
        error.message,
        "derivation rule is ambiguous across Models: first, second"
    );

    let literal_rule = compile(RULE_LITERAL_SOURCE);
    let literal_model = revision(&literal_rule, "catalog");
    for value in ["canonical", "premise"] {
        let literal = literal_rule
            .designations()
            .literal(value)
            .expect("rule-only literal is designated");
        assert!(
            literal_model.model().referents().contains_key(&literal),
            "rule-only literal is admitted to the inferred Model"
        );
    }
    let mut forged_scope = frontend::parse(RULE_LITERAL_SOURCE).expect("rule fixture parses");
    forged_scope.rules[0].model.value = frontend::Name("Text".to_owned());
    let error = elaborate::compile(forged_scope).expect_err("public AST cannot forge rule scope");
    assert_eq!(
        error.to_string(),
        "canonical derivation rule requires a declared Model"
    );

    let identity_bound = frontend::parse(RULE_IDENTITY_BOUND_SOURCE)
        .expect("over-bound canonical rule still has valid syntax");
    let error = elaborate::compile(identity_bound).expect_err("rule identity search is bounded");
    assert_eq!(
        error.to_string(),
        "canonical derivation rule exceeds 40320-candidate identity bound"
    );

    let expected = output.canonical_bytes();
    let authoring = temporary("clause");
    let rust = temporary("rs");
    let binary = temporary("bin");
    fs::write(&authoring, &source).expect("canonical authoring source writes");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("canonical requests emit Rust"),
    )
    .expect("generated Rust writes");
    fs::remove_file(&authoring).expect("authoring source deletes before generated compile");
    let generated = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated Rust compiler starts");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("source-deleted generated executable starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.as_bytes());
    fs::remove_file(rust).expect("generated Rust cleans up");
    fs::remove_file(binary).expect("generated executable cleans up");
}

#[test]
fn authored_law_is_ground_until_separately_authorized_and_retains_exact_provenance() {
    const LAW: &str = "Text
Entity

tags/has: RelationShape
  {entity: Entity} has tag {tag: Text}
  mode entity -> tag: many

catalog
  Item ∈ Entity
  Item has tag \"premise\"

law canonical tagging
  ?entity has tag \"derived\" if
    ?entity has tag \"premise\"

why in catalog:
  Item has tag \"derived\"
";
    let ground_only = compile(LAW);
    let ground_revision = revision(&ground_only, "catalog");
    assert_eq!(ground_revision.model().universal_laws().len(), 1);
    assert!(ground_revision.model().derivation_rules().is_empty());

    let relation = relation(&ground_only, "tags/has");
    let item = referent(&ground_only, &ground_revision, "Item");
    let derived = ground_only
        .designations()
        .literal("derived")
        .expect("law-only literal remains designated");
    let target = RelationalContent::new(
        relation.clone(),
        BTreeMap::from([
            (
                role(&ground_only, &relation, "entity"),
                Term::referent(item.clone()),
            ),
            (
                role(&ground_only, &relation, "tag"),
                Term::referent(derived.clone()),
            ),
        ]),
    )
    .expect("law conclusion target is typed");
    assert!(
        clause::derive::saturate(&ground_revision, clause::derive::Limits::new(16, 4, 64),)
            .expect("plain law closure is bounded")
            .proof(&target)
            .is_none(),
        "an authored universal law does not execute by itself"
    );

    let authorized_source = format!("{LAW}\nderive canonical tagging\n");
    let authorized = compile(&authorized_source);
    let authorized_revision = revision(&authorized, "catalog");
    let law = &authorized_revision.model().universal_laws()[0];
    let rule = &authorized_revision.model().derivation_rules()[0];
    assert_eq!(rule.governing_law(), law.id());
    assert_eq!(rule.authority(), authorized_revision.model().id());
    assert_eq!(rule.scope(), authorized_revision.model().id());
    assert_eq!(rule.premises(), law.premises());
    assert_eq!(rule.conclusion(), law.conclusion());
    let label = authorized
        .designations()
        .scoped(authorized_revision.model().id(), "canonical tagging")
        .expect("law label is a scoped source binding");
    assert_eq!(
        authorized_revision
            .model()
            .definition(&label)
            .expect("law label lowers as a definition")
            .denotation(),
        &Term::referent(law.id().clone())
    );

    let alpha = compile(
        &authorized_source
            .replace("?entity", "?subject")
            .replace("canonical tagging", "renamed tagging"),
    );
    let alpha_revision = revision(&alpha, "catalog");
    assert_eq!(
        law.id(),
        alpha_revision.model().universal_laws()[0].id(),
        "law identity is independent of binder and source-label spelling"
    );

    let proof = clause::execution::why(
        &authorized_revision,
        &target,
        clause::derive::Limits::new(16, 4, 64),
    )
    .expect("authorized law proof executes")
    .expect("authorized law entails its conclusion");
    let clause::execution::Witness::Derived {
        rule: witnessed_rule,
        governing_law,
        authority,
        scope,
        ..
    } = &proof.why.witnesses[proof.why.root].witness
    else {
        panic!("authorized conclusion has a derived witness");
    };
    assert_eq!(witnessed_rule, rule.id());
    assert_eq!(governing_law, law.id());
    assert_eq!(authority, rule.authority());
    assert_eq!(scope, rule.scope());

    let reloaded = clause::wire::reload(&clause::wire::serialize(&authorized_revision))
        .expect("semantic-v9 law/rule authority reloads");
    assert_eq!(reloaded, authorized_revision);

    let resolved = request::resolve(&authorized).expect("authorized law request resolves");
    let expected = request::run(&resolved, request::RunLimits::default())
        .expect("authorized law request executes")
        .canonical_bytes();
    let authoring = temporary("law_clause");
    let rust = temporary("law_rs");
    let binary = temporary("law_bin");
    fs::write(&authoring, &authorized_source).expect("law authoring source writes");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("authorized law emits Rust"),
    )
    .expect("generated law Rust writes");
    fs::remove_file(&authoring).expect("law source deletes before generated compile");
    let generated = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated law Rust compiler starts");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("generated law executable starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.as_bytes());
    fs::remove_file(rust).expect("generated law Rust cleans up");
    fs::remove_file(binary).expect("generated law executable cleans up");
}
