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

Pair
  first: F32
  second: F32

Pose
  position: Vec2

gravity: 9.81
truth: true
pair: (3.0, 4.0)
vectors: [(3.0, 4.0), (5.0, 12.0)]
labelled vector: Vec2 { x: 3.0, y: 4.0 }
labelled pair: Pair { first: 3.0, second: 4.0 }
pose: Pose { position: Vec2 { x: 3.0, y: 4.0 } }

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

const RELATIONAL_VALID: &str = r#"F32
Marker

Vec2
  x: F32
  y: F32

Pose
  position: Vec2

check: RelationShape
  {pose: Pose} checks {marker: Marker}
  mode pose -> marker: many

authored pose: Pose { position: Vec2 { x: 3.0, y: 4.0 } }

scene
  sample ∈ Pose
  flag ∈ Marker
  sample checks flag
"#;

const RELATIONAL_CONTEXT_VALID: &str = r#"F32
Marker

Vec2
  x: F32
  y: F32

Pose
  position: Vec2

check: RelationShape
  {pose: Pose} checks {marker: Marker}
  mode pose -> marker: many

sample ∈ Pose
flag ∈ Marker
authored pose: Pose { position: Vec2 { x: 3.0, y: 4.0 } }
sample checks flag
"#;

fn take_authored_binding(
    members: &mut Vec<frontend::Member>,
    binding: &str,
) -> frontend::SurfaceTerm {
    let binding_index = members
        .iter()
        .position(|member| {
            matches!(
                member,
                frontend::Member::PureDefinition(definition)
                    if definition.name.value.as_str() == binding
            )
        })
        .expect("authored structural binding exists");
    let frontend::Member::PureDefinition(definition) = members.remove(binding_index) else {
        unreachable!("located member is a pure definition");
    };
    definition.result
}

fn replace_relational_role(
    members: &mut [frontend::Member],
    role: &str,
    term: frontend::SurfaceTerm,
) {
    let clause = members
        .iter_mut()
        .find_map(|member| match member {
            frontend::Member::RelationalContent(clause) => Some(clause),
            _ => None,
        })
        .expect("relational content exists");
    clause
        .roles
        .insert(frontend::RoleName(role.to_owned()), term);
}

fn relational_program(source: &str, named: bool) -> frontend::Program {
    let mut program = frontend::parse(source).expect("relational source parses");
    let term = take_authored_binding(&mut program.top_level, "authored pose");
    if named {
        let model = program
            .declarations
            .iter_mut()
            .find(|declaration| declaration.subject.value.as_str() == "scene")
            .expect("scene Model exists");
        replace_relational_role(&mut model.body, "pose", term);
    } else {
        replace_relational_role(&mut program.top_level, "pose", term);
    }
    program
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
    let pair = program
        .designations()
        .scoped(&model_id(), "pair")
        .expect("pair definition resolves");
    let Term::Product {
        shape: pair_domain, ..
    } = model.definition(&pair).expect("pair lowers").denotation()
    else {
        panic!("pair lowers as a tuple");
    };
    assert_eq!(
        model
            .structural_contracts()
            .get(pair_domain)
            .expect("tuple contract is sealed")
            .form(),
        &StructuralForm::Tuple(vec![f32_domain.clone(), f32_domain.clone()])
    );
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
        wire::reload(&wire::serialize(revision)).expect("structural semantic-v10 wire reloads");
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
        let source = SOURCE.replacen("Vec2 { x: 3.0, y: 4.0 }", product, 1);
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

#[test]
fn relational_failures_keep_authored_paths_in_named_and_context_models() {
    fn assert_diagnostic(
        valid: &elaborate::CompiledProgram,
        error: &elaborate::CompileError,
        invalid: &str,
    ) {
        let relation = valid
            .designations()
            .global("check")
            .expect("check relation resolves");
        let pose_role = valid
            .designations()
            .role(&relation, "pose")
            .expect("check.pose resolves");
        let pose = valid.designations().global("Pose").expect("Pose resolves");
        let position = valid
            .designations()
            .scoped(&pose, "position")
            .expect("Pose.position resolves");
        let vec2 = valid.designations().global("Vec2").expect("Vec2 resolves");
        let y = valid
            .designations()
            .scoped(&vec2, "y")
            .expect("Vec2.y resolves");
        let diagnostic = error
            .diagnostic()
            .expect("authored relational rejection has a source diagnostic");
        assert_eq!(diagnostic.class(), StructuralFailureClass::DomainMismatch);
        assert!(matches!(
            diagnostic.path().subject(),
            ProposalSubject::Content(_)
        ));
        assert_eq!(
            diagnostic.path().segments(),
            &[
                ProposalPathSegment::Role(pose_role),
                ProposalPathSegment::ProductField(position),
                ProposalPathSegment::ProductField(y),
            ]
        );
        let line = invalid
            .lines()
            .find(|line| line.contains("true"))
            .expect("invalid relational line");
        assert_eq!(
            diagnostic.span(),
            frontend::Span {
                line: invalid
                    .lines()
                    .position(|candidate| candidate == line)
                    .expect("invalid line has an index")
                    + 1,
                column: line.find("true").expect("true token") + 1,
                width: "true".len(),
            }
        );
    }

    let valid_named = elaborate::compile(relational_program(RELATIONAL_VALID, true))
        .expect("valid named Model source lowers");
    let invalid_named = RELATIONAL_VALID.replace("y: 4.0", "y: true");
    let named_error = elaborate::compile(relational_program(&invalid_named, true))
        .expect_err("named Model structural proposal is rejected");
    assert_diagnostic(&valid_named, &named_error, &invalid_named);

    let valid_context = elaborate::compile_in(
        relational_program(RELATIONAL_CONTEXT_VALID, false),
        ModelContext::new(model_id()),
    )
    .expect("valid context source lowers");
    let invalid_context = RELATIONAL_CONTEXT_VALID.replace("y: 4.0", "y: true");
    let context_error = elaborate::compile_in(
        relational_program(&invalid_context, false),
        ModelContext::new(model_id()),
    )
    .expect_err("context structural proposal is rejected");
    assert_diagnostic(&valid_context, &context_error, &invalid_context);
}

#[test]
fn pure_definition_local_trace_survives_application_substitution() {
    const VALID: &str = r#"F32

Vec2
  x: F32
  y: F32

authored vector: Vec2 { x: 3.0, y: 4.0 }

magnitude:
  vector: 3.0
  if true then vector else vector
"#;
    fn program(source: &str) -> frontend::Program {
        let mut program = frontend::parse(source).expect("local source parses");
        let source_index = program
            .top_level
            .iter()
            .position(|member| {
                matches!(
                    member,
                    frontend::Member::PureDefinition(definition)
                        if definition.name.value.as_str() == "authored vector"
                )
            })
            .expect("authored vector binding exists");
        let frontend::Member::PureDefinition(source) = program.top_level.remove(source_index)
        else {
            unreachable!("located member is a pure definition");
        };
        let target = program
            .top_level
            .iter_mut()
            .find_map(|member| match member {
                frontend::Member::PureDefinition(definition)
                    if definition.name.value.as_str() == "magnitude" =>
                {
                    Some(definition)
                }
                _ => None,
            })
            .expect("magnitude definition exists");
        target
            .locals
            .first_mut()
            .expect("magnitude has one local")
            .denotation = source.result;
        program
    }

    let valid = elaborate::compile_in(program(VALID), ModelContext::new(model_id()))
        .expect("valid local source lowers");
    let magnitude = valid
        .designations()
        .scoped(&model_id(), "magnitude")
        .expect("magnitude definition resolves");
    let definition = valid
        .context_revision()
        .expect("valid context Revision")
        .model()
        .definition(&magnitude)
        .expect("magnitude definition is sealed");
    let Term::Application(application) = definition.denotation() else {
        panic!("magnitude result is an application");
    };
    let input_role = valid
        .context_revision()
        .expect("valid context Revision")
        .model()
        .content(application)
        .expect("length application is registered")
        .roles()
        .iter()
        .filter_map(|(role, term)| {
            matches!(term, Term::LabelledProduct { .. }).then_some(role.clone())
        })
        .min()
        .expect("conditional has a structural branch role");
    let vec2 = valid.designations().global("Vec2").expect("Vec2 resolves");
    let y = valid
        .designations()
        .scoped(&vec2, "y")
        .expect("Vec2.y resolves");

    let invalid = VALID.replace("y: 4.0", "y: true");
    let error = elaborate::compile_in(program(&invalid), ModelContext::new(model_id()))
        .expect_err("substituted local proposal is rejected");
    let diagnostic = error
        .diagnostic()
        .expect("substituted local retains its authored source anchor");
    assert_eq!(diagnostic.class(), StructuralFailureClass::DomainMismatch);
    assert!(matches!(
        diagnostic.path().subject(),
        ProposalSubject::Content(_)
    ));
    assert_eq!(
        diagnostic.path().segments(),
        &[
            ProposalPathSegment::Role(input_role),
            ProposalPathSegment::ProductField(y),
        ]
    );
    let line = invalid
        .lines()
        .find(|line| line.contains("true"))
        .expect("invalid local line");
    assert_eq!(diagnostic.span().column, line.find("true").unwrap() + 1);
    assert_eq!(diagnostic.span().width, "true".len());
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
        Term::Application(content) => {
            format!("[\"application\",\"{}\"]", content.as_str())
        }
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

    let pose = definition("pose");
    let Term::LabelledProduct {
        shape: pose_shape,
        fields: pose_fields,
    } = pose.denotation()
    else {
        panic!("pose must remain a labelled product");
    };
    let position = pose_fields
        .first_key_value()
        .expect("Pose has one position field")
        .0
        .clone();
    let labelled_pair = definition("labelled pair");
    let Term::LabelledProduct {
        shape: pair_shape,
        fields: pair_fields,
    } = labelled_pair.denotation()
    else {
        panic!("labelled pair must remain a labelled product");
    };
    let pose_definition = format!(
        "[\"definition\",\"{}\",{}]",
        pose.id().as_str(),
        structural_term_json(pose.denotation())
    );
    let mut wrong_shape_fields = pose_fields.clone();
    wrong_shape_fields.insert(
        position.clone(),
        Term::labelled_product(pair_shape.clone(), pair_fields.clone())
            .expect("complete Pair term encodes"),
    );
    let wrong_shape = Term::labelled_product(pose_shape.clone(), wrong_shape_fields)
        .expect("Pose with a complete wrong-shape field encodes");
    let wrong_shape_definition = format!(
        "[\"definition\",\"{}\",{}]",
        pose.id().as_str(),
        structural_term_json(&wrong_shape)
    );
    let wrong_shape_error =
        rehashed_wire_error(&semantic, &pose_definition, &wrong_shape_definition);
    assert_eq!(
        wrong_shape_error
            .structural_failure()
            .expect("wrong complete shape retains typed evidence")
            .class(),
        StructuralFailureClass::DomainMismatch
    );
    assert_eq!(
        wrong_shape_error
            .structural_failure()
            .expect("wrong complete shape retains typed evidence")
            .path()
            .segments(),
        &[ProposalPathSegment::ProductField(position.clone())]
    );

    let frame_velocity = definition("frame velocity");
    let Term::Application(application) = frame_velocity.denotation() else {
        panic!("frame velocity remains an application");
    };
    let mut application_fields = pose_fields.clone();
    application_fields.insert(position.clone(), Term::application(application.clone()));
    let application_pose = Term::labelled_product(pose_shape.clone(), application_fields)
        .expect("Pose with an application field encodes");
    let application_definition = format!(
        "[\"definition\",\"{}\",{}]",
        pose.id().as_str(),
        structural_term_json(&application_pose)
    );
    let application_error =
        rehashed_wire_error(&semantic, &pose_definition, &application_definition);
    let application_failure = application_error
        .structural_failure()
        .expect("mismatched application retains typed evidence");
    assert_eq!(
        application_failure.class(),
        StructuralFailureClass::DomainMismatch
    );
    assert_eq!(
        application_failure.path().segments(),
        &[
            ProposalPathSegment::ProductField(position),
            ProposalPathSegment::Application(application.clone()),
        ]
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
