//! Exact acceptance oracle for the one-page hospital fire-egress program.

use clause::{
    elaborate, frontend,
    intervention::{AchieveAll, PreventAll},
    kernel::{ReferentId, RelationalContent, Revision, RoleId, Term},
    request::{self, Request, RequestOutput, RunOutput, RunResult, Selection},
    wire,
};
use std::{collections::BTreeMap, env, fs, path::PathBuf, process::Command};

const SOURCE: &str = include_str!("../examples/hospital.clause");

fn temporary(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "clause-hospital-program-{}.{}",
        std::process::id(),
        extension
    ))
}

fn program() -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(SOURCE).expect("hospital source parses"))
        .expect("hospital source lowers")
}

fn revision(program: &elaborate::CompiledProgram, name: &str) -> Revision {
    program
        .revision(&frontend::Name(name.to_owned()))
        .expect("named hospital Revision resolves")
        .clone()
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

fn route(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    origin: &str,
    destination: &str,
) -> RelationalContent {
    assertion(
        program,
        revision,
        "egress/route",
        &[("origin", origin), ("destination", destination)],
    )
}

fn canonical_sets(mut alternatives: Vec<Vec<RelationalContent>>) -> Vec<Vec<RelationalContent>> {
    for members in &mut alternatives {
        members.sort();
    }
    alternatives.sort();
    alternatives
}

fn revision_output(output: &RunOutput, index: usize) -> &RequestOutput {
    output.results[index]
        .revision()
        .expect("query result is Revision-scoped")
        .output()
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

#[test]
fn hospital_program_has_the_complete_six_request_semantic_and_materialization_journey() {
    let compiled = program();
    let base = revision(&compiled, "egress");
    let successor = revision(&compiled, "egress/door-101-withdrawn");

    assert_eq!(base.model().admitted_contents().len(), 22);
    assert_eq!(successor.model().admitted_contents().len(), 21);
    for door in [
        "Door 101", "Door 102", "Door 103", "Door 104", "Door 105", "Door 106",
    ] {
        assert!(
            base.model()
                .referents()
                .contains_key(&referent(&compiled, &base, door))
        );
    }

    let resolved = request::resolve(&compiled).expect("hospital requests resolve in source order");
    assert!(matches!(
        resolved.requests(),
        [
            Request::Find { .. },
            Request::Why { all: true, .. },
            Request::Prevent {
                selection: Selection::AllMinimal,
                ..
            },
            Request::Prevent {
                selection: Selection::AllMinimal,
                ..
            },
            Request::Achieve {
                selection: Selection::AllMinimal,
                ..
            },
            Request::Diff { .. },
        ]
    ));

    let output = request::run(&resolved, request::RunLimits::default())
        .expect("hospital requests execute through generic semantics");
    assert_eq!(output.results.len(), 6);

    let RequestOutput::Find(destinations) = revision_output(&output, 0) else {
        panic!("first request is recursive find");
    };
    let mut expected_destinations = vec![
        Term::referent(referent(&compiled, &base, "East-Corridor")),
        Term::referent(referent(&compiled, &base, "North-Exit")),
        Term::referent(referent(&compiled, &base, "West-Corridor")),
    ];
    expected_destinations.sort();
    assert_eq!(destinations, &expected_destinations);

    let RequestOutput::WhyAll(Some(why)) = revision_output(&output, 1) else {
        panic!("second request is complete why all");
    };
    assert!(why.is_complete());
    assert_eq!(
        canonical_sets(support_sets(why)),
        canonical_sets(vec![
            vec![
                connects(&compiled, &base, "Door 101", "ICU-A", "East-Corridor"),
                passed(&compiled, &base, "Door 101"),
                connects(&compiled, &base, "Door 102", "East-Corridor", "North-Exit",),
                passed(&compiled, &base, "Door 102"),
            ],
            vec![
                connects(&compiled, &base, "Door 103", "ICU-A", "West-Corridor"),
                passed(&compiled, &base, "Door 103"),
                connects(&compiled, &base, "Door 104", "West-Corridor", "North-Exit",),
                passed(&compiled, &base, "Door 104"),
            ],
        ]),
    );

    let RequestOutput::PreventAll(base_prevent) = revision_output(&output, 2) else {
        panic!("third request is base prevention");
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
        ]),
    );

    let RequestOutput::PreventAll(successor_prevent) = revision_output(&output, 3) else {
        panic!("fourth request is successor prevention");
    };
    assert_eq!(
        canonical_sets(withdrawals(successor_prevent)),
        canonical_sets(vec![
            vec![passed(&compiled, &successor, "Door 103")],
            vec![passed(&compiled, &successor, "Door 104")],
        ]),
    );

    let RequestOutput::AchieveAll(achieve) = revision_output(&output, 4) else {
        panic!("fifth request is complete achievement");
    };
    assert_eq!(
        canonical_sets(additions(achieve)),
        canonical_sets(vec![
            vec![passed(&compiled, &base, "Door 105")],
            vec![passed(&compiled, &base, "Door 106")],
        ]),
    );

    let RunResult::Diff {
        base: diff_base,
        successor: diff_successor,
        output: diff,
    } = &output.results[5]
    else {
        panic!("sixth request is semantic diff");
    };
    assert_eq!(diff_base, base.identity());
    assert_eq!(diff_successor, successor.identity());
    assert!(diff.authored().added().is_empty());
    assert_eq!(
        diff.authored().removed(),
        &[passed(&compiled, &base, "Door 101")]
    );
    assert!(diff.entailed_added().is_empty());
    assert_eq!(
        diff.entailed_removed(),
        &[route(&compiled, &base, "ICU-A", "East-Corridor")],
    );
    assert_eq!(diff.changed_supports().len(), 2);
    let east = diff
        .changed_supports()
        .iter()
        .find(|change| change.consequence() == &route(&compiled, &base, "ICU-A", "East-Corridor"))
        .expect("east route support disappears");
    assert!(east.added().is_empty());
    assert_eq!(
        canonical_sets(
            east.removed()
                .iter()
                .map(|support| support.assertions().to_vec())
                .collect(),
        ),
        canonical_sets(vec![vec![
            connects(&compiled, &base, "Door 101", "ICU-A", "East-Corridor"),
            passed(&compiled, &base, "Door 101"),
        ]]),
    );
    assert!(east.retained().is_empty());
    let north = diff
        .changed_supports()
        .iter()
        .find(|change| change.consequence() == &route(&compiled, &base, "ICU-A", "North-Exit"))
        .expect("north route retains its west support");
    assert!(north.added().is_empty());
    assert_eq!(
        canonical_sets(
            north
                .removed()
                .iter()
                .map(|support| support.assertions().to_vec())
                .collect(),
        ),
        canonical_sets(vec![vec![
            connects(&compiled, &base, "Door 101", "ICU-A", "East-Corridor"),
            passed(&compiled, &base, "Door 101"),
            connects(&compiled, &base, "Door 102", "East-Corridor", "North-Exit",),
            passed(&compiled, &base, "Door 102"),
        ]]),
    );
    assert_eq!(
        canonical_sets(
            north
                .retained()
                .iter()
                .map(|support| support.assertions().to_vec())
                .collect(),
        ),
        canonical_sets(vec![vec![
            connects(&compiled, &base, "Door 103", "ICU-A", "West-Corridor"),
            passed(&compiled, &base, "Door 103"),
            connects(&compiled, &base, "Door 104", "West-Corridor", "North-Exit",),
            passed(&compiled, &base, "Door 104"),
        ]]),
    );

    let expected = output.canonical_bytes();
    let source = temporary("clause");
    let revision = temporary("revision");
    let generated_source = temporary("rs");
    let generated_binary = temporary("bin");
    fs::write(&source, SOURCE).expect("hospital source writes");

    let seal = Command::new(env!("CARGO_BIN_EXE_clause"))
        .arg("seal")
        .arg(&source)
        .arg("egress")
        .arg(&revision)
        .output()
        .expect("seal command starts");
    assert!(
        seal.status.success(),
        "{}",
        String::from_utf8_lossy(&seal.stderr)
    );
    let sealed = wire::reload(&fs::read_to_string(&revision).expect("revision reads"))
        .expect("sealed hospital Revision reloads");
    assert_eq!(sealed.identity(), base.identity());

    let run = Command::new(env!("CARGO_BIN_EXE_clause"))
        .arg("run")
        .arg(&source)
        .output()
        .expect("run command starts");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8(run.stdout).expect("run output is UTF-8"),
        format!("{expected}\n"),
    );

    let materialize = Command::new(env!("CARGO_BIN_EXE_clause"))
        .arg("emit-rust")
        .arg(&source)
        .arg(&generated_source)
        .output()
        .expect("emit-rust command starts");
    assert!(
        materialize.status.success(),
        "{}",
        String::from_utf8_lossy(&materialize.stderr)
    );
    let emitted = fs::read_to_string(&generated_source).expect("generated Rust reads");
    assert!(!emitted.contains("mod frontend"));
    assert!(!emitted.contains("[Door 101..106]"));

    fs::remove_file(&source).expect("authoring source removes before generated compilation");
    assert!(!source.exists());
    let compile = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&generated_source)
        .arg("-o")
        .arg(&generated_binary)
        .output()
        .expect("generated Rust compiler starts");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let generated = Command::new(&generated_binary)
        .output()
        .expect("source-deleted generated program starts");
    assert!(generated.status.success());
    assert_eq!(generated.stdout, expected.as_bytes());

    fs::remove_file(&generated_source).expect("generated source cleans up");
    fs::remove_file(&generated_binary).expect("generated binary cleans up");
    fs::remove_file(&revision).expect("revision cleans up");
}
