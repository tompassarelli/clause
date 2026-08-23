//! Exact acceptance oracle for the one-page hospital fire-egress program.

use clause::{
    elaborate, frontend, generated,
    intervention::{AchieveAll, PreventAll},
    kernel::{Clause, EntityId, Name, RelationId, Revision, RoleId, Term},
    request::{self, Request, RequestOutput, Selection},
    wire,
};
use std::{collections::BTreeMap, env, fs, path::PathBuf, process::Command};

const SOURCE: &str = include_str!("../examples/hospital.clause");

fn temporary(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "clause-hospital-golden-{}.{}",
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

fn name(value: &str) -> Name {
    Name::new(value.to_owned()).expect("valid stable semantic name")
}

fn relation(value: &str) -> RelationId {
    RelationId::new(name(value)).expect("valid Relation identity")
}

fn role(value: &str) -> RoleId {
    RoleId::new(name(value)).expect("valid role identity")
}

fn entity(revision: &Revision, local: &str) -> EntityId {
    revision
        .model()
        .entities()
        .iter()
        .find(|candidate| candidate.local().as_str() == local)
        .expect("hospital entity is admitted")
        .clone()
}

fn assertion(revision: &Revision, relation_name: &str, roles: &[(&str, &str)]) -> Clause {
    Clause::new(
        relation(relation_name),
        roles
            .iter()
            .map(|(role_name, local)| (role(role_name), Term::entity(entity(revision, local))))
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("typed hospital assertion")
}

fn connects(revision: &Revision, door: &str, origin: &str, destination: &str) -> Clause {
    assertion(
        revision,
        "egress/connects",
        &[
            ("door", door),
            ("origin", origin),
            ("destination", destination),
        ],
    )
}

fn passed(revision: &Revision, door: &str) -> Clause {
    assertion(
        revision,
        "egress/passed",
        &[("door", door), ("inspection", "Fire-Marshal-Inspection")],
    )
}

fn route(revision: &Revision, origin: &str, destination: &str) -> Clause {
    assertion(
        revision,
        "egress/route",
        &[("origin", origin), ("destination", destination)],
    )
}

fn support_sets(why: &clause::execution::WhyAll) -> Vec<Vec<Clause>> {
    why.alternatives
        .iter()
        .map(|alternative| alternative.assertions.clone())
        .collect()
}

fn withdrawals(result: &PreventAll) -> Vec<Vec<Clause>> {
    let PreventAll::Complete(items) = result else {
        panic!("prevent all must exhaust its finite frontier: {result:?}");
    };
    items
        .iter()
        .map(|item| item.delta().withdrawals().to_vec())
        .collect()
}

fn additions(result: &AchieveAll) -> Vec<Vec<Clause>> {
    let AchieveAll::Complete(items) = result else {
        panic!("achieve all must exhaust its finite frontier: {result:?}");
    };
    items
        .iter()
        .map(|item| item.delta().admissions().to_vec())
        .collect()
}

#[test]
fn hospital_program_has_the_complete_six_request_semantic_and_materialization_journey() {
    let compiled = program();
    let base = revision(&compiled, "egress");
    let successor = revision(&compiled, "egress/door-101-withdrawn");

    assert_eq!(base.model().assertions().len(), 10);
    assert_eq!(
        base.model()
            .entities()
            .iter()
            .filter(|candidate| candidate.typ().as_str() == "Door")
            .count(),
        6,
    );

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

    let RequestOutput::Find(destinations) = &output.results[0] else {
        panic!("first request is recursive find");
    };
    assert_eq!(
        destinations,
        &[
            Term::entity(entity(&base, "East-Corridor")),
            Term::entity(entity(&base, "North-Exit")),
            Term::entity(entity(&base, "West-Corridor")),
        ],
    );

    let RequestOutput::WhyAll(Some(why)) = &output.results[1] else {
        panic!("second request is complete why all");
    };
    assert!(why.is_complete());
    assert_eq!(
        support_sets(why),
        vec![
            vec![
                connects(&base, "Door 101", "ICU-A", "East-Corridor"),
                passed(&base, "Door 101"),
                connects(&base, "Door 102", "East-Corridor", "North-Exit"),
                passed(&base, "Door 102"),
            ],
            vec![
                connects(&base, "Door 103", "ICU-A", "West-Corridor"),
                passed(&base, "Door 103"),
                connects(&base, "Door 104", "West-Corridor", "North-Exit"),
                passed(&base, "Door 104"),
            ],
        ],
    );

    let RequestOutput::PreventAll(base_prevent) = &output.results[2] else {
        panic!("third request is base prevention");
    };
    assert_eq!(
        withdrawals(base_prevent),
        vec![
            vec![passed(&base, "Door 101"), passed(&base, "Door 103")],
            vec![passed(&base, "Door 101"), passed(&base, "Door 104")],
            vec![passed(&base, "Door 102"), passed(&base, "Door 103")],
            vec![passed(&base, "Door 102"), passed(&base, "Door 104")],
        ],
    );

    let RequestOutput::PreventAll(successor_prevent) = &output.results[3] else {
        panic!("fourth request is successor prevention");
    };
    assert_eq!(
        withdrawals(successor_prevent),
        vec![
            vec![passed(&successor, "Door 103")],
            vec![passed(&successor, "Door 104")],
        ],
    );

    let RequestOutput::AchieveAll(achieve) = &output.results[4] else {
        panic!("fifth request is complete achievement");
    };
    assert_eq!(
        additions(achieve),
        vec![
            vec![passed(&base, "Door 105")],
            vec![passed(&base, "Door 106")],
        ],
    );

    let RequestOutput::Diff(diff) = &output.results[5] else {
        panic!("sixth request is semantic diff");
    };
    assert!(diff.authored().added().is_empty());
    assert_eq!(diff.authored().removed(), &[passed(&base, "Door 101")]);
    assert!(diff.entailed_added().is_empty());
    assert_eq!(
        diff.entailed_removed(),
        &[route(&base, "ICU-A", "East-Corridor")],
    );
    assert_eq!(diff.changed_supports().len(), 2);
    let east = diff
        .changed_supports()
        .iter()
        .find(|change| change.consequence() == &route(&base, "ICU-A", "East-Corridor"))
        .expect("east route support disappears");
    assert!(east.added().is_empty());
    assert_eq!(
        east.removed()
            .iter()
            .map(|support| support.assertions().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![
            connects(&base, "Door 101", "ICU-A", "East-Corridor"),
            passed(&base, "Door 101"),
        ]],
    );
    assert!(east.retained().is_empty());
    let north = diff
        .changed_supports()
        .iter()
        .find(|change| change.consequence() == &route(&base, "ICU-A", "North-Exit"))
        .expect("north route retains its west support");
    assert!(north.added().is_empty());
    assert_eq!(
        north
            .removed()
            .iter()
            .map(|support| support.assertions().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![
            connects(&base, "Door 101", "ICU-A", "East-Corridor"),
            passed(&base, "Door 101"),
            connects(&base, "Door 102", "East-Corridor", "North-Exit"),
            passed(&base, "Door 102"),
        ]],
    );
    assert_eq!(
        north
            .retained()
            .iter()
            .map(|support| support.assertions().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![
            connects(&base, "Door 103", "ICU-A", "West-Corridor"),
            passed(&base, "Door 103"),
            connects(&base, "Door 104", "West-Corridor", "North-Exit"),
            passed(&base, "Door 104"),
        ]],
    );

    let expected = output.canonical_bytes();
    let source = temporary("clause");
    let revision = temporary("revision");
    let generated_source = temporary("rs");
    let generated_binary = temporary("bin");
    fs::write(&source, SOURCE).expect("hospital source writes");

    let seal = Command::new(env!("CARGO_BIN_EXE_clause"))
        .args([
            "seal",
            source.to_str().expect("UTF-8 source path"),
            "egress",
            revision.to_str().expect("UTF-8 revision path"),
        ])
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
        .args(["run", source.to_str().expect("UTF-8 source path")])
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

    fs::remove_file(&source).expect("authoring source removes before generation");
    assert!(!source.exists());
    fs::write(
        &generated_source,
        generated::emit_rust(&resolved).expect("hospital requests emit Rust"),
    )
    .expect("generated source writes");
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
