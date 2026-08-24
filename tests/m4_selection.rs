use clause::{
    elaborate, frontend, generated,
    kernel::{Name, QueryPlan, QueryPlanColumn, Term},
    request::{self, QueryColumn, Request, RequestOutput, ResolvedProgram},
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
            columns,
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

    let mut nested_roles = pattern.roles().clone();
    nested_roles.insert(
        role("a"),
        Term::Sum {
            tag: Name::new("nested".to_owned()).unwrap(),
            value: Box::new(Term::pattern(resolved_columns[0].binder().clone())),
        },
    );
    let nested = clause::kernel::RelationalContent::new(relation.clone(), nested_roles).unwrap();
    let error = QueryPlan::new(
        compiled
            .revision(&frontend::Name("selection".into()))
            .unwrap()
            .model(),
        &nested,
        resolved_columns
            .iter()
            .map(|column| QueryPlanColumn::new(column.binder().clone(), column.origins().to_vec()))
            .collect(),
    )
    .expect_err("nested holes remain outside M4/S1");
    assert_eq!(
        error.to_string(),
        "nested query holes are not admitted by M4/S1"
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
