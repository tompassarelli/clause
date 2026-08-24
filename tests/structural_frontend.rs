use clause::{
    elaborate::{self, CompileDiagnosticStatus, ModelContext},
    frontend,
    kernel::{
        ProposalPathSegment, ProposalSubject, ReferentId, StructuralFailureClass, StructuralForm,
        Term,
    },
    wire,
};
use std::collections::BTreeSet;

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

    type TermCheck = fn(&Term) -> bool;
    let checks: [(&str, TermCheck); 5] = [
        ("gravity", |term: &Term| matches!(term, Term::F32(_))),
        ("truth", |term: &Term| matches!(term, Term::Bool(true))),
        ("pair", |term: &Term| matches!(term, Term::Product { .. })),
        ("vectors", |term: &Term| {
            matches!(term, Term::Sequence { .. })
        }),
        ("labelled vector", |term: &Term| {
            matches!(term, Term::LabelledProduct { .. })
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

    let vec2 = program
        .designations()
        .global("Vec2")
        .expect("Vec2 shape resolves");
    let x = program
        .designations()
        .scoped(&vec2, "x")
        .expect("Vec2.x field resolves");
    let y = program
        .designations()
        .scoped(&vec2, "y")
        .expect("Vec2.y field resolves");
    assert_eq!(
        model
            .structural_contracts()
            .get(&vec2)
            .expect("Vec2 contract is sealed")
            .form(),
        &StructuralForm::Product(BTreeSet::from([x.clone(), y.clone()]))
    );
    let f32_domain = program
        .designations()
        .global("F32")
        .expect("F32 domain resolves");
    for field in [x, y] {
        assert_eq!(
            model
                .definition(&field)
                .expect("shape field has an exact binding definition")
                .denotation(),
            &Term::referent(f32_domain.clone())
        );
    }

    let reloaded =
        wire::reload(&wire::serialize(revision)).expect("structural semantic-v7 wire reloads");
    assert_eq!(&reloaded, revision);
}

#[test]
fn labelled_products_reject_undeclared_missing_and_wrong_domain_fields() {
    let cases = [
        ("Vec2 { x: 3.0, z: 4.0 }", "unknown designation 'z'"),
        (
            "Vec2 { x: 3.0 }",
            "labelled product must fill its exact structural contract",
        ),
        (
            "Vec2 { x: 3.0, y: true }",
            "labelled product field does not satisfy its bound domain",
        ),
    ];
    for (product, expected) in cases {
        let source = SOURCE.replace("Vec2 { x: 3.0, y: 4.0 }", product);
        let error = elaborate::compile_in(
            frontend::parse(&source).expect("malformed labelled product source parses"),
            ModelContext::new(model_id()),
        )
        .expect_err("malformed labelled product must fail closed");
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn nested_wrong_domain_reports_the_rank_one_authored_proposal_path() {
    const VALID: &str = r#"F32

Vec2
  x: F32
  y: F32

Pose
  position: Vec2

pose: Pose { position: Vec2 { x: 3.0, y: 4.0 } }
"#;
    let valid = elaborate::compile_in(
        frontend::parse(VALID).expect("valid nested product parses"),
        ModelContext::new(model_id()),
    )
    .expect("valid nested product lowers");
    let pose_definition = valid
        .designations()
        .scoped(&model_id(), "pose")
        .expect("pose definition resolves");
    let pose_shape = valid
        .designations()
        .global("Pose")
        .expect("Pose shape resolves");
    let position = valid
        .designations()
        .scoped(&pose_shape, "position")
        .expect("Pose.position field resolves");
    let vec2_shape = valid
        .designations()
        .global("Vec2")
        .expect("Vec2 shape resolves");
    let y = valid
        .designations()
        .scoped(&vec2_shape, "y")
        .expect("Vec2.y field resolves");

    let invalid = VALID.replace("y: 4.0", "y: true");
    let error = elaborate::compile_in(
        frontend::parse(&invalid).expect("wrong-domain nested product remains syntax"),
        ModelContext::new(model_id()),
    )
    .expect_err("kernel must reject the wrong-domain proposal");
    let diagnostic = error
        .diagnostic()
        .expect("authored kernel rejection has a compile diagnostic");
    assert_eq!(diagnostic.rank(), 1);
    assert_eq!(
        diagnostic.status(),
        CompileDiagnosticStatus::RejectedProposal
    );
    assert_eq!(diagnostic.class(), StructuralFailureClass::DomainMismatch);
    assert_eq!(
        diagnostic.path().subject(),
        &ProposalSubject::Definition(pose_definition)
    );
    assert_eq!(
        diagnostic.path().segments(),
        &[
            ProposalPathSegment::ProductField(position),
            ProposalPathSegment::ProductField(y),
        ]
    );
    assert_eq!(diagnostic.presentation(), ["Pose.position", "Vec2.y"]);
    let line = invalid.lines().nth(9).expect("nested proposal line");
    let expected = frontend::Span {
        line: 10,
        column: line.find("true").expect("authored true token") + 1,
        width: "true".len(),
    };
    assert_eq!(diagnostic.span(), expected);
    let valid_line = VALID.lines().nth(9).expect("valid nested proposal line");
    assert_eq!(
        valid.proposal_span(diagnostic.path()),
        Some(frontend::Span {
            line: 10,
            column: valid_line.find("4.0").expect("authored valid y token") + 1,
            width: "4.0".len(),
        })
    );
}

fn structural_term_json(term: &Term) -> String {
    match term {
        Term::F32(value) => format!("[\"f32\",\"{:08x}\"]", value.bits()),
        Term::Bool(value) => format!("[\"bool\",\"{value}\"]"),
        Term::Product { shape, fields } => format!(
            "[\"product\",\"{}\",[{}]]",
            shape.as_str(),
            fields
                .iter()
                .map(|(label, field)| format!(
                    "[\"{}\",\"{}\",{}]",
                    label.as_str(),
                    field.domain().as_str(),
                    structural_term_json(field.value())
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Term::LabelledProduct { shape, fields } => format!(
            "[\"labelled-product\",\"{}\",[{}]]",
            shape.as_str(),
            fields
                .iter()
                .map(|(field, value)| format!(
                    "[\"{}\",{}]",
                    field.as_str(),
                    structural_term_json(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Term::Sequence {
            shape,
            element,
            values,
        } => format!(
            "[\"sequence\",\"{}\",\"{}\",[{}]]",
            shape.as_str(),
            element.as_str(),
            values
                .iter()
                .map(structural_term_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => panic!("test renderer received a non-structural term: {term:?}"),
    }
}

fn rehashed_wire_error(semantic: &str, before: &str, after: &str) -> clause::kernel::KernelError {
    assert_eq!(
        semantic.matches(before).count(),
        1,
        "mutation must be exact"
    );
    let tampered = semantic.replacen(before, after, 1);
    let identity = wire::sha256_hex(tampered.as_bytes());
    let artifact = format!(
        "[\"{}\",\"rev-sha256-{identity}\",{tampered}]",
        wire::REVISION_TAG
    );
    wire::reload(&artifact).expect_err("rehashed structural tampering must fail admission")
}

#[test]
fn rehashed_structural_wire_tampering_fails_exact_admission() {
    let program = elaborate::compile_in(
        frontend::parse(SOURCE).expect("structural source parses"),
        ModelContext::new(model_id()),
    )
    .expect("structural source lowers");
    let revision = program.context_revision().expect("context Revision");
    let model = revision.model();
    let semantic = wire::semantic_payload(revision);
    let definition = |name: &str| {
        let id = program
            .designations()
            .scoped(&model_id(), name)
            .expect("definition resolves");
        model.definition(&id).expect("definition is sealed")
    };

    let labelled = definition("labelled vector");
    let Term::LabelledProduct { shape, fields } = labelled.denotation() else {
        panic!("labelled vector must remain a labelled product");
    };
    let original_labelled = structural_term_json(labelled.denotation());
    let mut missing_fields = fields.clone();
    missing_fields.pop_first();
    let missing =
        Term::labelled_product(shape.clone(), missing_fields).expect("one Vec2 field remains");
    let labelled_definition = format!(
        "[\"definition\",\"{}\",{original_labelled}]",
        labelled.id().as_str()
    );
    let missing_definition = format!(
        "[\"definition\",\"{}\",{}]",
        labelled.id().as_str(),
        structural_term_json(&missing)
    );
    assert_eq!(
        rehashed_wire_error(&semantic, &labelled_definition, &missing_definition).to_string(),
        "labelled product must fill its exact structural contract"
    );

    let mut wrong_fields = fields.clone();
    let wrong_field = wrong_fields
        .first_key_value()
        .expect("Vec2 has fields")
        .0
        .clone();
    *wrong_fields
        .first_entry()
        .expect("Vec2 has fields")
        .get_mut() = Term::boolean(true);
    let wrong = Term::labelled_product(shape.clone(), wrong_fields).expect("wrong Vec2 encodes");
    let wrong_definition = format!(
        "[\"definition\",\"{}\",{}]",
        labelled.id().as_str(),
        structural_term_json(&wrong)
    );
    let source_free_error = rehashed_wire_error(&semantic, &labelled_definition, &wrong_definition);
    assert_eq!(
        source_free_error.to_string(),
        "labelled product field does not satisfy its bound domain"
    );
    let source_free_failure = source_free_error
        .structural_failure()
        .expect("wire tamper retains typed source-free kernel evidence");
    assert_eq!(
        source_free_failure.class(),
        StructuralFailureClass::DomainMismatch
    );
    assert_eq!(
        source_free_failure.path().subject(),
        &ProposalSubject::Definition(labelled.id().clone())
    );
    assert_eq!(
        source_free_failure.path().segments(),
        &[ProposalPathSegment::ProductField(wrong_field)]
    );

    let (field, _) = fields.first_key_value().expect("Vec2 has fields");
    let field = field.clone();
    let f32_domain = model
        .definition(&field)
        .expect("field binding exists")
        .denotation()
        .referent_id()
        .expect("field binding denotes F32");
    let bool_domain = model
        .structural_contracts()
        .values()
        .find(|contract| contract.form() == &StructuralForm::Bool)
        .expect("Bool contract exists")
        .referent();
    let field_definition = format!(
        "[\"definition\",\"{}\",[\"referent\",\"{}\"]]",
        field.as_str(),
        f32_domain.as_str()
    );
    let altered_field_definition = format!(
        "[\"definition\",\"{}\",[\"referent\",\"{}\"]]",
        field.as_str(),
        bool_domain.as_str()
    );
    assert_eq!(
        rehashed_wire_error(&semantic, &field_definition, &altered_field_definition).to_string(),
        "labelled product field does not satisfy its bound domain"
    );

    let vectors = definition("vectors");
    let Term::Sequence {
        shape,
        element,
        values,
    } = vectors.denotation()
    else {
        panic!("vectors must remain a sequence");
    };
    let vectors_definition = format!(
        "[\"definition\",\"{}\",{}]",
        vectors.id().as_str(),
        structural_term_json(vectors.denotation())
    );
    let mut wrong_values = values.clone();
    wrong_values[0] = Term::boolean(true);
    let wrong_element = Term::sequence(shape.clone(), element.clone(), wrong_values)
        .expect("wrong sequence element encodes");
    let wrong_element_definition = format!(
        "[\"definition\",\"{}\",{}]",
        vectors.id().as_str(),
        structural_term_json(&wrong_element)
    );
    assert_eq!(
        rehashed_wire_error(&semantic, &vectors_definition, &wrong_element_definition).to_string(),
        "structural term does not match its expected domain"
    );

    let wrong_metadata = Term::sequence(shape.clone(), bool_domain.clone(), values.clone())
        .expect("wrong sequence metadata encodes");
    let wrong_metadata_definition = format!(
        "[\"definition\",\"{}\",{}]",
        vectors.id().as_str(),
        structural_term_json(&wrong_metadata)
    );
    assert_eq!(
        rehashed_wire_error(&semantic, &vectors_definition, &wrong_metadata_definition).to_string(),
        "structural term does not match its expected domain"
    );
}
