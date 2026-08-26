use clause::{
    elaborate::{self, ElaborationContext},
    execution, frontend, generated,
    kernel::{Delta, Model, ReferentId, Revision, SemanticAtom, Term},
    request, wire,
};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
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
expected lengths: [5.0, 13.0]

frame velocity:
  direction: (1.0, 0.0)
  direction * 300.0
expected velocity: (300.0, 0.0)

frame next position:
  position: (0.0, 0.0)
  dt: 0.5
  position + frame velocity * dt
expected next position: (150.0, 0.0)

frame collision:
  coin: (160.0, 0.0)
  player radius: 12.0
  coin radius: 8.0
  length (frame next position - coin) <= player radius + coin radius
expected collision: true

frame collected:
  if frame collision then true else false
expected collected: true

frame score:
  if frame collected then 10 else 0
expected score: 10
"#;

const DATA_NAMES: [&str; 5] = ["gravity", "truth", "pair", "vectors", "labelled vector"];
const RESULT_PAIRS: [(&str, &str); 6] = [
    ("lengths", "expected lengths"),
    ("frame velocity", "expected velocity"),
    ("frame next position", "expected next position"),
    ("frame collision", "expected collision"),
    ("frame collected", "expected collected"),
    ("frame score", "expected score"),
];
static SOURCE_NUMBER: AtomicUsize = AtomicUsize::new(0);

struct SealedFixture {
    revision: Revision,
    canonical: String,
    definitions: BTreeMap<&'static str, ReferentId>,
    source_path: PathBuf,
}

fn model_id() -> ReferentId {
    ReferentId::new(format!("ref-sha256-{}", "9".repeat(64)))
        .expect("fixed caller-owned Model identity")
}

fn sealed_fixture() -> SealedFixture {
    let source_path = std::env::temp_dir().join(format!(
        "clause-pure-computation-{}-{}.clause",
        std::process::id(),
        SOURCE_NUMBER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&source_path, SOURCE).expect("authoring source writes");
    let source = fs::read_to_string(&source_path).expect("authoring source reads");
    fs::remove_file(&source_path).expect("authoring source deletes before compilation");
    let program = elaborate::compile_in(
        frontend::parse(&source).expect("pure computation source parses"),
        ElaborationContext::new(model_id()),
    )
    .expect("pure computation source compiles in context");
    drop(source);

    for forbidden in ["Type", "Value", "Object", "Field"] {
        assert!(
            program.designations().global(forbidden).is_err()
                && program
                    .designations()
                    .scoped(&model_id(), forbidden)
                    .is_err(),
            "pure data must not synthesize a {forbidden} ontology"
        );
    }

    let definitions = DATA_NAMES
        .into_iter()
        .chain(
            RESULT_PAIRS
                .into_iter()
                .flat_map(|(actual, expected)| [actual, expected]),
        )
        .map(|name| {
            let id = program
                .designations()
                .scoped(&model_id(), name)
                .unwrap_or_else(|error| panic!("definition '{name}' resolves: {error}"));
            (name, id)
        })
        .collect();
    let revision = program
        .context_revision()
        .expect("caller-owned context Revision");
    assert_eq!(revision.model().id(), &model_id());
    let canonical = wire::serialize(revision);
    drop(program);

    SealedFixture {
        revision: wire::reload(&canonical).expect("source-deleted computation wire reloads"),
        canonical,
        definitions,
        source_path,
    }
}

fn denotation<'a>(model: &'a Model, id: &ReferentId) -> &'a Term {
    model
        .definitions()
        .iter()
        .find(|definition| definition.id() == id)
        .unwrap_or_else(|| panic!("definition {id:?} survives immutable wire reload"))
        .denotation()
}

fn evaluate(revision: &Revision, id: &ReferentId) -> Term {
    execution::evaluate(revision, denotation(revision.model(), id))
        .unwrap_or_else(|error| panic!("pure definition {id:?} evaluates: {error}"))
}

#[test]
fn checked_literals_and_structures_are_source_deleted_terms() {
    let fixture = sealed_fixture();
    let model = fixture.revision.model();

    for name in DATA_NAMES {
        let term = denotation(model, &fixture.definitions[name]);
        assert!(
            term.referent_id().is_none()
                && term.pattern_id().is_none()
                && term.content_id().is_none(),
            "{name} must remain a checked literal or structure, not a semantic referent, pattern, or relational application: {term:?}"
        );
    }
    assert!(model.occurrences().is_empty());
    assert!(model.judgments().is_empty());
    assert!(model.derivation_rules().is_empty());
    assert!(model.universal_laws().is_empty());
    assert!(model.invariants().is_empty());
    assert!(model.goals().is_empty());
    assert!(model.transitions().is_empty());
    assert_eq!(wire::serialize(&fixture.revision), fixture.canonical);
}

#[test]
fn reloaded_pure_computation_is_target_neutral_and_deterministic() {
    let fixture = sealed_fixture();
    let first =
        RESULT_PAIRS.map(|(actual, _)| evaluate(&fixture.revision, &fixture.definitions[actual]));
    let second =
        RESULT_PAIRS.map(|(actual, _)| evaluate(&fixture.revision, &fixture.definitions[actual]));
    let expected = RESULT_PAIRS.map(|(_, expected)| {
        denotation(fixture.revision.model(), &fixture.definitions[expected]).clone()
    });
    let repeat_reload =
        wire::reload(&fixture.canonical).expect("repeat immutable wire reload succeeds");
    let after_repeat_reload =
        RESULT_PAIRS.map(|(actual, _)| evaluate(&repeat_reload, &fixture.definitions[actual]));

    assert_eq!(first, expected);
    assert_eq!(second, expected);
    assert_eq!(after_repeat_reload, expected);
    assert_eq!(first, second);
    assert_eq!(wire::serialize(&fixture.revision), fixture.canonical);
    assert_eq!(wire::serialize(&repeat_reload), fixture.canonical);
}

#[test]
fn generated_evaluation_matches_source_deleted_interpreter_bytes() {
    let fixture = sealed_fixture();
    let selected = RESULT_PAIRS
        .map(|(actual, _)| fixture.definitions[actual].clone())
        .to_vec();
    let duplicate = [selected[0].clone(), selected[0].clone()];
    assert!(
        generated::emit_evaluation_rust(&fixture.revision, &duplicate)
            .expect_err("duplicate definition requests are ambiguous")
            .to_string()
            .contains("duplicate definition")
    );
    assert!(
        generated::emit_evaluation_rust(&fixture.revision, &[model_id()])
            .expect_err("a referent without a definition is not evaluable by definition ID")
            .to_string()
            .contains("missing definition")
    );
    let removed_definition = fixture.revision.model().definitions()[0].clone();
    let successor = Delta::new(
        fixture.revision.identity().clone(),
        Vec::new(),
        vec![SemanticAtom::Definition(removed_definition)],
    )
    .expect("one exact definition withdrawal is a valid Delta")
    .apply(&fixture.revision)
    .expect("definition withdrawal admits a successor");
    assert_eq!(
        generated::emit_evaluation_rust(&successor, &[])
            .expect_err("standalone evaluation requires a root Revision")
            .to_string(),
        "generated evaluation requires a root Revision"
    );
    let expected = request::EvaluationOutput::new(
        fixture.revision.identity().clone(),
        selected
            .iter()
            .map(|definition| (definition.clone(), evaluate(&fixture.revision, definition)))
            .collect(),
    )
    .expect("selected definitions are unique")
    .canonical_bytes();
    assert!(expected.starts_with("[\"clause-evaluate-v1\","));
    assert!(!expected.ends_with('\n'));
    let emitted = generated::emit_evaluation_rust(&fixture.revision, &selected)
        .expect("root Revision emits target-neutral evaluation Rust");
    assert!(!emitted.contains("mod frontend"));
    assert!(!emitted.contains("mod elaborate"));

    let artifact = std::env::temp_dir().join(format!(
        "clause-pure-evaluation-generated-{}-{}",
        std::process::id(),
        SOURCE_NUMBER.fetch_add(1, Ordering::Relaxed)
    ));
    let rust = artifact.with_extension("rs");
    let binary = artifact.with_extension("bin");
    fs::write(&rust, emitted).expect("generated evaluation Rust writes once");
    assert!(
        !fixture.source_path.exists(),
        "authoring source must be absent before generated rustc"
    );
    let compiled = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated evaluation rustc starts once");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("generated evaluation runs once");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.as_bytes());
    fs::remove_file(rust).expect("generated evaluation Rust cleans up");
    fs::remove_file(binary).expect("generated evaluation binary cleans up");
}
