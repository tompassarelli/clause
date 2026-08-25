use clause::{
    elaborate, frontend, generated,
    kernel::Term,
    request::{self, QueryColumn, QuerySelection, Request, RequestOutput, ResolvedProgram},
    wire,
};
use std::{env, fs, path::PathBuf, process::Command};

const SOURCE: &str = "Entity

selection/related: RelationShape
  {scope: Entity} relates {a: Entity} through {b: Entity} and {c: Entity} to {d: Entity}
  mode scope -> a, b, c, d: many
  mode scope, a, b, c -> d: many

selection
  World ∈ Entity
  A ∈ Entity
  B ∈ Entity
  C ∈ Entity
  D ∈ Entity
  World relates A through B and B to C
  World relates A through B and C to D
  World relates C through B and B to A

why in selection:
  World relates A through B and B to C

World relates ? through ?same and ?same to ?

find all ?d in selection:
  World relates A through B and B to ?d
";

const ANY_SOURCE: &str = "Entity

selection/related: RelationShape
  {scope: Entity} relates {a: Entity} through {b: Entity} and {c: Entity} to {d: Entity}
  mode scope -> a, b, c, d: many

selection
  World ∈ Entity
  A ∈ Entity
  B ∈ Entity
  C ∈ Entity
  D ∈ Entity
  World relates A through B and B to C
  World relates A through B and C to D
  World relates C through B and B to A

any World relates ?person through ?same and ?same to ?
any World relates ?same through ?same and ?same to ?same
";

const EXPLICIT_PROJECTION_SOURCE: &str = "Entity

selection/related: RelationShape
  {scope: Entity} relates {a: Entity} through {b: Entity} and {c: Entity} to {d: Entity}
  mode scope -> a, b, c, d: many

selection
  World ∈ Entity
  A ∈ Entity
  B ∈ Entity
  C ∈ Entity
  D ∈ Entity
  World relates A through B and B to C
  World relates A through B and C to D
  World relates C through B and B to A

select ?person
  World relates ?person through ?same and ?same to ?
";

const CARDINALITY_SOURCE: &str = "Entity

selection/related: RelationShape
  {scope: Entity} relates {a: Entity} through {b: Entity} and {c: Entity} to {d: Entity}
  mode scope -> a, b, c, d: many
  mode scope, b, c -> a, d: many
  mode scope, b, c, d -> a: many

selection
  World ∈ Entity
  A ∈ Entity
  B ∈ Entity
  C ∈ Entity
  D ∈ Entity
  World relates A through B and B to C
  World relates C through B and B to A

select ?person
  World relates ?person through B and B to ?destination

select one ?person
  World relates ?person through B and B to C

select first ?person
  World relates ?person through B and B to ?destination

select first ?person
  World relates ?person through C and C to D
";

const NESTED_APPLICATION_SOURCE: &str = "Body
Scalar

distance
  result: Scalar distance between left: Body and right: Body
  left right -> result

radius
  result: Scalar radius of subject: Body
  subject -> result

+
  result: Scalar is left: Scalar + right: Scalar
  left right -> result

collision
  subject: Body collides with other: Body at separation: Scalar within reach: Scalar
  other separation reach -> subject*

overlap
  subject: Body overlaps other: Body
  other -> subject*

scene
  player ∈ Body
  coin ∈ Body
  coin collides with player at distance between coin and player within radius of coin + radius of player

law collision overlap
  ?body overlaps ?other if
    ?body collides with ?other at (distance between ?body and ?other) within (radius of ?body + radius of ?other)

derive collision overlap

select one ?body
  ?body collides with player at (distance between ?body and player) within (radius of ?body + radius of player)

any ?body overlaps player
";

fn temporary(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "clause-m4-selection-{}.{}",
        std::process::id(),
        extension
    ))
}

#[test]
fn naked_selection_requires_one_exact_declared_model() {
    let schema = "Entity

selection/related: RelationShape
  {scope: Entity} relates {a: Entity} through {b: Entity} and {c: Entity} to {d: Entity}
  mode scope -> a, b, c, d: many
";
    let missing = format!(
        "{schema}
only
  Present ∈ Entity

World relates ? through ?same and ?same to ?
"
    );
    let error = frontend::parse(&missing).expect_err("query has no eligible Model");
    assert_eq!(
        error.message,
        "naked query matches no declared Model; candidates: only"
    );

    let ambiguous = format!(
        "{schema}
left
  World ∈ Entity

right
  World ∈ Entity

World relates ? through ?same and ?same to ?
"
    );
    let error = frontend::parse(&ambiguous).expect_err("query has two eligible Models");
    assert_eq!(
        error.message,
        "naked query is ambiguous across Models: left, right"
    );

    let referent_hole = "Entity

only
  ? ∈ Entity
";
    let error = frontend::parse(referent_hole).expect_err("a hole cannot become a referent");
    assert_eq!(error.message, "expected semantic name, found '?'");

    let rule_hole = format!(
        "{schema}
only
  World ∈ Entity
  A ∈ Entity
  B ∈ Entity
  C ∈ Entity
  World relates A through B and B to C

only/legacy: DerivationRule
  World relates ? through B and B to C
  when:
    World relates A through B and B to C
"
    );
    let error = frontend::parse(&rule_hole).expect_err("bare rule holes are not in this slice");
    assert_eq!(
        error.message,
        "anonymous holes are only valid in naked queries"
    );
}

#[test]
fn naked_selection_preserves_freshness_labels_identity_and_generated_parity() {
    let compiled = elaborate::compile(frontend::parse(SOURCE).expect("M4 source parses"))
        .expect("M4 source compiles");
    let resolved = request::resolve(&compiled).expect("M4 requests resolve");
    assert!(matches!(
        resolved.requests(),
        [
            Request::Why { .. },
            Request::Select { .. },
            Request::Find { .. }
        ]
    ));

    let Request::Select {
        pattern,
        dependencies,
        columns: resolved_columns,
        ..
    } = &resolved.requests()[1]
    else {
        panic!("source-middle request must resolve as a selection");
    };

    let relation = compiled.designations().global("selection/related").unwrap();
    let role = |label: &str| compiled.designations().role(&relation, label).unwrap();
    let mut correlated_origins = vec![role("b"), role("c")];
    correlated_origins.sort();
    let expected_origins = vec![vec![role("a")], correlated_origins, vec![role("d")]];
    assert_eq!(
        resolved_columns
            .iter()
            .map(|column| column.origins().to_vec())
            .collect::<Vec<_>>(),
        expected_origins,
        "each column retains every exact stable role origin"
    );

    let rejected_origins = |origins: Vec<_>| {
        let mut requests = resolved.requests().to_vec();
        let mut columns = resolved_columns.clone();
        columns[0] = QueryColumn::new(
            columns[0].label().map(str::to_owned),
            columns[0].binder().clone(),
            origins,
        );
        requests[1] = Request::Select {
            revision: compiled
                .revision(&frontend::Name("selection".into()))
                .unwrap()
                .identity()
                .clone(),
            pattern: pattern.clone(),
            dependencies: dependencies.clone(),
            columns,
            selection: QuerySelection::All,
        };
        ResolvedProgram::new(resolved.revisions().clone(), requests)
            .expect_err("invalid role provenance must fail closed")
            .to_string()
    };
    assert_eq!(
        rejected_origins(Vec::new()),
        "query column requires at least one role origin"
    );
    assert_eq!(
        rejected_origins(vec![role("d")]),
        "query column role origins do not match the pattern"
    );

    let output =
        request::run(&resolved, request::RunLimits::default()).expect("M4 requests execute");
    let RequestOutput::Select { columns, rows } = &output.results[1] else {
        panic!("source-middle request must remain a selection");
    };
    assert_eq!(
        columns, resolved_columns,
        "binder, role origins, and label survive the execution boundary"
    );
    assert_eq!(
        columns
            .iter()
            .map(|column| column.label().map(str::to_owned))
            .collect::<Vec<_>>(),
        vec![None, Some("same".to_owned()), None],
        "bare holes are unlabelled and a repeated named hole projects once"
    );

    let model = compiled.designations().global("selection").unwrap();
    let term = |name: &str| {
        Term::referent(
            compiled
                .designations()
                .scoped(&model, name)
                .expect("fixture referent resolves"),
        )
    };
    let mut expected = vec![
        vec![term("A"), term("B"), term("C")],
        vec![term("C"), term("B"), term("A")],
    ];
    expected.sort();
    assert_eq!(
        rows.iter()
            .map(|row| {
                assert_eq!(
                    row.cells()
                        .iter()
                        .map(|cell| cell.origins().to_vec())
                        .collect::<Vec<_>>(),
                    expected_origins
                );
                row.cells()
                    .iter()
                    .map(|cell| cell.value().clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        expected,
        "anonymous holes stay fresh while the named hole correlates"
    );

    let renamed_source = SOURCE.replace("?same", "?opening");
    let renamed = elaborate::compile(
        frontend::parse(&renamed_source).expect("alpha-renamed M4 source parses"),
    )
    .expect("alpha-renamed M4 source compiles");
    let renamed_resolved = request::resolve(&renamed).expect("renamed requests resolve");
    assert_eq!(
        wire::serialize(
            compiled
                .revision(&frontend::Name("selection".into()))
                .unwrap()
        ),
        wire::serialize(
            renamed
                .revision(&frontend::Name("selection".into()))
                .unwrap()
        ),
        "query labels do not enter Model or Revision identity"
    );
    let renamed_output = request::run(&renamed_resolved, request::RunLimits::default())
        .expect("renamed requests execute");
    let RequestOutput::Select {
        columns: renamed_columns,
        rows: renamed_rows,
    } = &renamed_output.results[1]
    else {
        panic!("renamed source-middle request must remain a selection");
    };
    assert_eq!(
        renamed_columns
            .iter()
            .map(|column| column.label().map(str::to_owned))
            .collect::<Vec<_>>(),
        vec![None, Some("opening".to_owned()), None]
    );
    assert_eq!(
        renamed_columns
            .iter()
            .map(|column| (column.binder(), column.origins()))
            .collect::<Vec<_>>(),
        columns
            .iter()
            .map(|column| (column.binder(), column.origins()))
            .collect::<Vec<_>>(),
        "alpha-renaming changes only presentation labels"
    );
    assert_eq!(
        renamed_rows, rows,
        "alpha-renaming preserves matched values"
    );

    let expected_bytes = output.canonical_bytes();
    let header = format!(
        "[\"select\",[[\"{}\",[\"{}\"],null],[\"{}\",[\"{}\",\"{}\"],\"same\"],[\"{}\",[\"{}\"],null]],",
        resolved_columns[0].binder().as_str(),
        expected_origins[0][0].as_str(),
        resolved_columns[1].binder().as_str(),
        expected_origins[1][0].as_str(),
        expected_origins[1][1].as_str(),
        resolved_columns[2].binder().as_str(),
        expected_origins[2][0].as_str(),
    );
    assert!(
        expected_bytes.contains(&header),
        "canonical columns retain binder, exact role origins, and optional label"
    );
    let authoring = temporary("clause");
    let rust = temporary("rs");
    let binary = temporary("bin");
    fs::write(&authoring, SOURCE).expect("authoring source writes");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("resolved M4 requests emit Rust"),
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
    assert_eq!(actual.stdout, expected_bytes.as_bytes());
    fs::remove_file(rust).expect("generated Rust cleans up");
    fs::remove_file(binary).expect("generated executable cleans up");
}

#[test]
fn law_backed_nested_one_coin_closes_the_m4_acceptance_seam() {
    let compiled = elaborate::compile(
        frontend::parse(NESTED_APPLICATION_SOURCE).expect("nested query source parses"),
    )
    .expect("nested query source compiles");
    let resolved = request::resolve(&compiled).expect("nested query resolves");
    let [
        Request::Select {
            dependencies,
            columns,
            selection: QuerySelection::ExactlyOne,
            ..
        },
        Request::Any { .. },
    ] = resolved.requests()
    else {
        panic!("nested source resolves to exact-one selection and existence query");
    };
    assert!(!dependencies.is_empty());
    let revision = compiled.revision(&frontend::Name("scene".into())).unwrap();
    let hole_bearing = dependencies
        .iter()
        .filter(|dependency| {
            let mut found = false;
            for term in dependency.roles().values() {
                term.walk(&mut |term| found |= term.pattern_id().is_some());
            }
            found
        })
        .collect::<Vec<_>>();
    assert!(!hole_bearing.is_empty());
    assert!(
        hole_bearing
            .iter()
            .all(|dependency| revision.model().content(dependency.id()).is_none()),
        "hole-bearing application dependencies remain request-local"
    );
    assert_eq!(columns.len(), 1);
    assert_eq!(columns[0].label(), Some("body"));

    let law = &revision.model().universal_laws()[0];
    let rule = &revision.model().derivation_rules()[0];
    assert_ne!(rule.id(), law.id(), "law and operational rule stay distinct");
    assert_eq!(rule.governing_law(), law.id());
    assert_eq!(rule.authority(), revision.model().id());
    assert_eq!(rule.scope(), revision.model().id());

    let closure_limits = clause::derive::Limits::new(16, 4, 64);
    let closure = clause::derive::saturate(revision, closure_limits)
        .expect("authorized collision law saturates within its explicit bound");
    let overlap = compiled.designations().global("overlap").unwrap();
    let overlaps = closure
        .contents()
        .iter()
        .filter(|content| content.relation() == &overlap)
        .collect::<Vec<_>>();
    assert_eq!(overlaps.len(), 1, "one authorized overlap is derived");
    let proof = closure
        .proof(overlaps[0])
        .expect("derived overlap retains an exact proof");
    let clause::derive::Witness::Derived {
        rule: witnessed_rule,
        governing_law,
        authority,
        scope,
        ..
    } = proof.witness()
    else {
        panic!("overlap must be produced by the authorized law projection");
    };
    assert_eq!(witnessed_rule, rule.id());
    assert_eq!(governing_law, law.id());
    assert_eq!(authority, rule.authority());
    assert_eq!(scope, rule.scope());

    let mut limits = request::RunLimits::default();
    limits.closure = closure_limits;
    let output = request::run(&resolved, limits)
        .expect("nested law-backed queries execute within their explicit bound");
    assert_eq!(
        request::run(&resolved, limits).expect("bounded execution repeats deterministically"),
        output
    );
    let [RequestOutput::SelectOne { rows, .. }, RequestOutput::Any(true)] =
        output.results.as_slice()
    else {
        panic!("the acceptance query returns exactly one row and true");
    };
    let scene = compiled.designations().global("scene").unwrap();
    let coin = Term::referent(compiled.designations().scoped(&scene, "coin").unwrap());
    assert_eq!(
        rows.iter()
            .map(|row| row.cells()[0].value().clone())
            .collect::<Vec<_>>(),
        vec![coin],
        "one named binder correlates the direct subject and every nested application leaf"
    );

    let renamed_source = NESTED_APPLICATION_SOURCE
        .replace("?body", "?candidate")
        .replace("?other", "?counterpart");
    let renamed = elaborate::compile(
        frontend::parse(&renamed_source).expect("alpha-renamed acceptance source parses"),
    )
    .expect("alpha-renamed acceptance source compiles");
    assert_eq!(
        wire::serialize(revision),
        wire::serialize(renamed.revision(&frontend::Name("scene".into())).unwrap()),
        "law and request hole labels do not enter semantic-v9 bytes"
    );

    let canonical_revision = wire::serialize(revision);
    let reloaded = wire::reload(&canonical_revision).expect("canonical semantic-v9 reloads");
    assert_eq!(&reloaded, revision);
    assert_eq!(wire::serialize(&reloaded), canonical_revision);
    let governing = format!(
        "[\"governing-law\",\"{}\"]",
        rule.governing_law().as_str()
    );
    let forged = format!("[\"governing-law\",\"{}\"]", rule.authority().as_str());
    let tampered = canonical_revision.replacen(&governing, &forged, 1);
    assert_ne!(tampered, canonical_revision);
    assert!(
        wire::reload(&tampered).is_err(),
        "tampered governing-law identity fails strict admission"
    );

    let expected = output.canonical_bytes();
    let authoring = temporary("closure.clause");
    let rust = temporary("nested.rs");
    let binary = temporary("nested.bin");
    fs::write(&authoring, NESTED_APPLICATION_SOURCE).expect("acceptance source writes");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("nested request emits Rust"),
    )
    .expect("generated nested Rust writes");
    fs::remove_file(&authoring).expect("acceptance source deletes before generated compile");
    let generated = Command::new("rustc")
        .args([
            "--edition=2024",
            "--cfg",
            "clause_generated",
            "--crate-name",
            "clause_m4_nested",
        ])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated nested Rust compiler starts");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("generated nested executable starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.as_bytes());
    fs::remove_file(rust).expect("generated nested Rust cleans up");
    fs::remove_file(binary).expect("generated nested executable cleans up");
}

#[test]
fn any_returns_only_bool_with_alpha_and_generated_parity() {
    let compiled = elaborate::compile(frontend::parse(ANY_SOURCE).expect("Any source parses"))
        .expect("Any source compiles");
    let resolved = request::resolve(&compiled).expect("Any requests resolve");
    let [
        Request::Any {
            pattern: matching_pattern,
            ..
        },
        Request::Any {
            pattern: rejecting_pattern,
            ..
        },
    ] = resolved.requests()
    else {
        panic!("source must resolve to exactly two Any requests");
    };

    let relation = compiled.designations().global("selection/related").unwrap();
    let role = |label: &str| compiled.designations().role(&relation, label).unwrap();
    let pattern_origins = |pattern: &clause::kernel::RelationalContent| {
        let mut origins = std::collections::BTreeMap::new();
        for (role, term) in pattern.roles() {
            if let Some(binder) = term.pattern_id() {
                origins
                    .entry(binder.clone())
                    .or_insert_with(Vec::new)
                    .push(role.clone());
            }
        }
        let mut origins = origins.into_values().collect::<Vec<_>>();
        origins.sort();
        origins
    };
    let mut correlated_origins = vec![role("b"), role("c")];
    correlated_origins.sort();
    let mut expected_matching_origins = vec![vec![role("a")], correlated_origins, vec![role("d")]];
    expected_matching_origins.sort();
    assert_eq!(
        pattern_origins(matching_pattern),
        expected_matching_origins,
        "Any pattern retains fresh anonymous and correlated named-hole role groupings"
    );
    let mut all_hole_origins = vec![role("a"), role("b"), role("c"), role("d")];
    all_hole_origins.sort();
    assert_eq!(
        pattern_origins(rejecting_pattern),
        vec![all_hole_origins],
        "one repeated named hole retains every correlated role in the pattern"
    );

    let output =
        request::run(&resolved, request::RunLimits::default()).expect("Any requests execute");
    assert_eq!(
        output.results,
        vec![RequestOutput::Any(true), RequestOutput::Any(false)],
        "Any exposes only one Boolean per authored request"
    );
    assert_eq!(
        output.canonical_bytes(),
        "[\"clause-run-v1\",[[\"any\",true],[\"any\",false]]]"
    );

    let renamed_source = ANY_SOURCE.replace("?same", "?opening");
    let renamed = elaborate::compile(
        frontend::parse(&renamed_source).expect("alpha-renamed Any source parses"),
    )
    .expect("alpha-renamed Any source compiles");
    let model = frontend::Name("selection".into());
    assert_eq!(
        wire::serialize(compiled.revision(&model).unwrap()),
        wire::serialize(renamed.revision(&model).unwrap()),
        "Any labels do not enter Model or Revision identity"
    );
    let renamed_resolved = request::resolve(&renamed).expect("renamed Any requests resolve");
    assert_eq!(
        request::run(&renamed_resolved, request::RunLimits::default())
            .expect("renamed Any requests execute")
            .canonical_bytes(),
        output.canonical_bytes(),
        "alpha-renaming is Boolean and canonical-byte neutral"
    );

    let authoring = temporary("any.clause");
    let rust = temporary("any.rs");
    let binary = temporary("any.bin");
    fs::write(&authoring, ANY_SOURCE).expect("Any authoring source writes");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("resolved Any requests emit Rust"),
    )
    .expect("generated Any Rust writes");
    fs::remove_file(&authoring).expect("Any authoring source deletes before generated compile");
    let generated = Command::new("rustc")
        .args([
            "--edition=2024",
            "--cfg",
            "clause_generated",
            "--crate-name",
            "clause_m4_any",
        ])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated Any Rust compiler starts");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("source-deleted generated Any executable starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, output.canonical_bytes().as_bytes());
    fs::remove_file(rust).expect("generated Any Rust cleans up");
    fs::remove_file(binary).expect("generated Any executable cleans up");
}

#[test]
fn explicit_projection_keeps_hidden_binders_private_and_matches_with_them() {
    let compiled = elaborate::compile(
        frontend::parse(EXPLICIT_PROJECTION_SOURCE).expect("explicit projection source parses"),
    )
    .expect("explicit projection source compiles");
    let resolved = request::resolve(&compiled).expect("explicit projection resolves");
    let [
        Request::Select {
            pattern,
            columns: resolved_columns,
            ..
        },
    ] = resolved.requests()
    else {
        panic!("source must resolve to exactly one explicit selection");
    };

    let relation = compiled.designations().global("selection/related").unwrap();
    let role = |label: &str| compiled.designations().role(&relation, label).unwrap();
    assert_eq!(resolved_columns.len(), 1, "only ?person is projected");
    assert_eq!(resolved_columns[0].label(), Some("person"));
    assert_eq!(resolved_columns[0].origins(), &[role("a")]);

    let mut pattern_origins = std::collections::BTreeMap::new();
    for (role, term) in pattern.roles() {
        if let Some(binder) = term.pattern_id() {
            pattern_origins
                .entry(binder.clone())
                .or_insert_with(Vec::new)
                .push(role.clone());
        }
    }
    for origins in pattern_origins.values_mut() {
        origins.sort();
    }
    let mut correlated_origins = vec![role("b"), role("c")];
    correlated_origins.sort();
    assert_eq!(
        pattern_origins.get(resolved_columns[0].binder()),
        Some(&vec![role("a")]),
        "the projected column retains its exact role origin"
    );
    assert_eq!(
        pattern_origins.len(),
        3,
        "two hidden binders remain in the pattern"
    );
    assert!(
        pattern_origins
            .values()
            .any(|origins| origins == &correlated_origins),
        "hidden ?same still correlates roles b and c"
    );
    assert!(
        pattern_origins
            .values()
            .any(|origins| origins == &vec![role("d")]),
        "the anonymous hidden hole remains a distinct fresh binder"
    );

    let output =
        request::run(&resolved, request::RunLimits::default()).expect("selection executes");
    let [RequestOutput::Select { columns, rows }] = output.results.as_slice() else {
        panic!("source must produce exactly one explicit selection result");
    };
    assert_eq!(columns, resolved_columns);
    let model = compiled.designations().global("selection").unwrap();
    let term = |name: &str| {
        Term::referent(
            compiled
                .designations()
                .scoped(&model, name)
                .expect("fixture referent resolves"),
        )
    };
    let mut expected = vec![vec![term("A")], vec![term("C")]];
    expected.sort();
    assert_eq!(
        rows.iter()
            .map(|row| {
                assert_eq!(row.cells().len(), 1);
                assert_eq!(row.cells()[0].origins(), &[role("a")]);
                vec![row.cells()[0].value().clone()]
            })
            .collect::<Vec<_>>(),
        expected,
        "hidden correlation filters the mismatched fact while the fresh hole matches freely"
    );

    let renamed_source = EXPLICIT_PROJECTION_SOURCE.replace("?same", "?opening");
    let renamed = elaborate::compile(
        frontend::parse(&renamed_source).expect("hidden-alpha-renamed source parses"),
    )
    .expect("hidden-alpha-renamed source compiles");
    let revision = frontend::Name("selection".into());
    assert_eq!(
        wire::serialize(compiled.revision(&revision).unwrap()),
        wire::serialize(renamed.revision(&revision).unwrap()),
        "hidden binder labels do not enter Model or Revision identity"
    );
    let renamed_resolved = request::resolve(&renamed).expect("renamed projection resolves");
    let renamed_output = request::run(&renamed_resolved, request::RunLimits::default())
        .expect("renamed projection executes");
    assert_eq!(
        renamed_output.canonical_bytes(),
        output.canonical_bytes(),
        "hidden alpha-renaming changes no semantic or canonical output"
    );

    let expected_bytes = output.canonical_bytes();
    let authoring = temporary("explicit.clause");
    let rust = temporary("explicit.rs");
    let binary = temporary("explicit.bin");
    fs::write(&authoring, EXPLICIT_PROJECTION_SOURCE).expect("authoring source writes");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("explicit projection emits Rust"),
    )
    .expect("generated Rust writes");
    fs::remove_file(&authoring).expect("authoring source deletes before generated compile");
    let generated = Command::new("rustc")
        .args([
            "--edition=2024",
            "--cfg",
            "clause_generated",
            "--crate-name",
            "clause_m4_explicit_projection",
        ])
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
    assert_eq!(actual.stdout, expected_bytes.as_bytes());
    fs::remove_file(rust).expect("generated Rust cleans up");
    fs::remove_file(binary).expect("generated executable cleans up");
}

#[test]
fn selection_cardinalities_preserve_canonical_rows_and_generated_parity() {
    let compiled =
        elaborate::compile(frontend::parse(CARDINALITY_SOURCE).expect("cardinality source parses"))
            .expect("cardinality source compiles");
    let resolved = request::resolve(&compiled).expect("cardinality requests resolve");
    assert!(matches!(
        resolved.requests(),
        [
            Request::Select {
                selection: QuerySelection::All,
                ..
            },
            Request::Select {
                selection: QuerySelection::ExactlyOne,
                ..
            },
            Request::Select {
                selection: QuerySelection::CanonicalFirst,
                ..
            },
            Request::Select {
                selection: QuerySelection::CanonicalFirst,
                ..
            },
        ]
    ));

    let output = request::run(&resolved, request::RunLimits::default())
        .expect("all cardinality contracts are satisfied");
    let [
        RequestOutput::Select { rows: all, .. },
        RequestOutput::SelectOne { rows: one, .. },
        RequestOutput::SelectFirst { rows: first, .. },
        RequestOutput::SelectFirst {
            rows: empty_first, ..
        },
    ] = output.results.as_slice()
    else {
        panic!("cardinality requests retain their exact output contracts");
    };
    assert_eq!(all.len(), 2);
    assert_eq!(one.len(), 1);
    assert_eq!(first, &all[..1]);
    assert!(empty_first.is_empty());

    let canonical = output.canonical_bytes();
    assert_eq!(canonical.matches("[\"select\",").count(), 1);
    assert_eq!(canonical.matches("[\"select-one\",").count(), 1);
    assert_eq!(canonical.matches("[\"select-first\",").count(), 2);

    let rust = temporary("cardinality.rs");
    let binary = temporary("cardinality.bin");
    fs::write(
        &rust,
        generated::emit_rust(&resolved).expect("cardinality requests emit Rust"),
    )
    .expect("generated cardinality Rust writes");
    let generated = Command::new("rustc")
        .args([
            "--edition=2024",
            "--cfg",
            "clause_generated",
            "--crate-name",
            "clause_m4_cardinality",
        ])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated cardinality Rust compiler starts");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("generated cardinality executable starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, canonical.as_bytes());
    fs::remove_file(rust).expect("generated cardinality Rust cleans up");
    fs::remove_file(binary).expect("generated cardinality executable cleans up");
}

#[test]
fn select_one_rejects_empty_and_multiple_complete_rows() {
    let compile = |source: &str| {
        let compiled = elaborate::compile(frontend::parse(source).expect("source parses"))
            .expect("source compiles");
        request::resolve(&compiled).expect("request resolves")
    };
    let multiple = CARDINALITY_SOURCE
        .split("select one ?person")
        .next()
        .expect("fixture has a query prefix")
        .to_owned()
        + "select one ?person\n  World relates ?person through B and B to ?destination\n";
    assert_eq!(
        request::run(&compile(&multiple), request::RunLimits::default())
            .expect_err("two distinct rows violate select one")
            .to_string(),
        "select one requires exactly one row, found 2"
    );

    let empty = multiple.replace(
        "World relates ?person through B and B to ?destination",
        "World relates ?person through C and C to D",
    );
    assert_eq!(
        request::run(&compile(&empty), request::RunLimits::default())
            .expect_err("no rows violate select one")
            .to_string(),
        "select one requires exactly one row, found 0"
    );
}
