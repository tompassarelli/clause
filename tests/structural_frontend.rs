use clause::{
    elaborate::{self, ModelContext},
    frontend,
    kernel::{ReferentId, Term},
};

const SOURCE: &str = r#"F32

Vec2
  x: F32
  y: F32

gravity: 9.81
truth: true
pair: (3.0, 4.0)
vectors: [(3.0, 4.0), (5.0, 12.0)]
labelled vector: Vec2 { x: 3.0, y: 4.0 }

lengths:
  input: [(3.0, 4.0), (5.0, 12.0)]
  map length over input
frame velocity:
  direction: (1.0, 0.0)
  direction * 300.0
frame collision:
  coin: (160.0, 0.0)
  length (frame velocity - coin) <= 20.0
frame collected:
  if frame collision then true else false
"#;

fn model_id() -> ReferentId {
    ReferentId::new(format!("ref-sha256-{}", "8".repeat(64)))
        .expect("fixed caller-owned Model identity")
}

#[test]
fn checked_structures_and_intrinsics_lower_without_an_evaluator() {
    let program = elaborate::compile_in(
        frontend::parse(SOURCE).expect("structural source parses"),
        ModelContext::new(model_id()),
    )
    .expect("structural source lowers");
    let revision = program.context_revision().expect("context Revision");
    let model = revision.model();

    let checks: [(&str, fn(&Term) -> bool); 5] = [
        ("gravity", |term: &Term| matches!(term, Term::F32(_))),
        ("truth", |term: &Term| matches!(term, Term::Bool(true))),
        ("pair", |term: &Term| matches!(term, Term::Product(_))),
        ("vectors", |term: &Term| matches!(term, Term::Sequence(_))),
        ("labelled vector", |term: &Term| {
            matches!(term, Term::Product(_))
        }),
    ];
    for (name, check) in checks {
        let id = program
            .designations()
            .scoped(&model_id(), name)
            .unwrap_or_else(|error| panic!("definition '{name}' resolves: {error}"));
        let term = model
            .definitions()
            .iter()
            .find(|definition| definition.id() == &id)
            .expect("definition lowers")
            .denotation();
        assert!(check(term), "unexpected {name} denotation: {term:?}");
    }

    for name in [
        "lengths",
        "frame velocity",
        "frame collision",
        "frame collected",
    ] {
        let id = program
            .designations()
            .scoped(&model_id(), name)
            .unwrap_or_else(|error| panic!("definition '{name}' resolves: {error}"));
        let term = model
            .definitions()
            .iter()
            .find(|definition| definition.id() == &id)
            .expect("definition lowers")
            .denotation();
        let Term::Application(content) = term else {
            panic!("{name} must lower to an unasserted application DAG: {term:?}");
        };
        assert!(
            model.content(content).is_some(),
            "root application is registered"
        );
    }
    assert!(model.occurrences().is_empty());
    assert!(model.judgments().is_empty());
}
