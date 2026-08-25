use clause::{
    elaborate::{self, ModelContext},
    frontend,
    kernel::{ContentId, JudgmentTarget, Model, ReferentId, Term},
    wire,
};
use std::collections::BTreeSet;

const SOURCE: &str = r#"Scalar

+
  result: Scalar is left: Scalar + right: Scalar
  left right -> result

=
  left: Scalar = right: Scalar
  left -> right*

9.81 ∈ Scalar
gravity ∈ Scalar
measured gravity ∈ Scalar
gravity: 9.81
gravity = measured gravity

energy:
  base: gravity + measured gravity
  base + base
"#;

fn model_id() -> ReferentId {
    ReferentId::new(format!("ref-sha256-{}", "8".repeat(64))).expect("fixed Model identity")
}

fn compile_in(source: &str, id: ReferentId) -> elaborate::CompiledProgram {
    elaborate::compile_in(
        frontend::parse(source).expect("pure definition source parses"),
        ModelContext::new(id),
    )
    .expect("pure definition source compiles in context")
}

fn application(term: &Term) -> &ContentId {
    let Term::Application(content) = term else {
        panic!("pure definition term must lower to an application: {term:?}");
    };
    content
}

fn has_judgment_for(model: &Model, content: &ContentId) -> bool {
    model
        .judgments()
        .iter()
        .any(|judgment| match judgment.target() {
            JudgmentTarget::Content(target) => target == content,
            JudgmentTarget::Occurrence(target) => model
                .occurrences()
                .iter()
                .find(|occurrence| occurrence.id() == target)
                .is_some_and(|occurrence| occurrence.content() == content),
        })
}

#[test]
fn closed_pure_definition_shares_its_local_application_without_exporting_it() {
    let id = model_id();
    let program = compile_in(SOURCE, id.clone());
    let revision = program
        .context_revision()
        .expect("caller-owned context Revision");
    let model = revision.model();
    assert_eq!(model.id(), &id);

    let gravity = program
        .designations()
        .scoped(&id, "gravity")
        .expect("gravity remains scoped to the caller Model");
    let energy = program
        .designations()
        .scoped(&id, "energy")
        .expect("energy remains scoped to the caller Model");
    let measured_gravity = program
        .designations()
        .scoped(&id, "measured gravity")
        .expect("measured gravity resolves in the caller Model");
    let scalar = program
        .designations()
        .scoped(&id, "9.81")
        .expect("bound scalar resolves in the caller Model");
    assert!(program.designations().global("gravity").is_err());
    assert!(program.designations().global("energy").is_err());

    let definition_ids = model
        .definitions()
        .iter()
        .map(|definition| definition.id().clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        definition_ids,
        BTreeSet::from([gravity.clone(), energy.clone()])
    );
    let gravity_definition = model
        .definitions()
        .iter()
        .find(|definition| definition.id() == &gravity)
        .expect("gravity Definition exists");
    assert_eq!(
        gravity_definition.denotation(),
        &Term::referent(scalar.clone())
    );
    let energy_definition = model
        .definitions()
        .iter()
        .find(|definition| definition.id() == &energy)
        .expect("energy Definition exists");

    let addition = program
        .designations()
        .global("+")
        .expect("addition relation resolves");
    let left = program
        .designations()
        .role(&addition, "left")
        .expect("addition left role resolves");
    let right = program
        .designations()
        .role(&addition, "right")
        .expect("addition right role resolves");
    let outer_id = application(energy_definition.denotation());
    let outer = model
        .content(outer_id)
        .expect("energy application is registered");
    assert_eq!(outer.relation(), &addition);
    assert_eq!(outer.roles().len(), 2);
    let left_inner_id = application(&outer.roles()[&left]);
    let right_inner_id = application(&outer.roles()[&right]);
    assert_eq!(left_inner_id, right_inner_id);

    let inner = model
        .content(left_inner_id)
        .expect("shared base application is registered");
    assert_eq!(inner.relation(), &addition);
    assert_eq!(inner.roles()[&left], Term::referent(gravity.clone()));
    assert_eq!(
        inner.roles()[&right],
        Term::referent(measured_gravity.clone())
    );

    for application in [outer_id, left_inner_id] {
        assert!(
            model
                .occurrences()
                .iter()
                .all(|occurrence| occurrence.content() != application),
            "definition applications are not assertion occurrences"
        );
        assert!(
            !has_judgment_for(model, application),
            "definition applications have no independent judgment"
        );
    }
    assert!(program.designations().scoped(&id, "base").is_err());
    assert!(program.designations().global("base").is_err());

    let equality = program
        .designations()
        .global("=")
        .expect("equality relation resolves");
    let equality_content = model
        .admitted_contents()
        .iter()
        .find(|content| content.relation() == &equality)
        .expect("equality remains admitted relational content");
    let equality_left = program
        .designations()
        .role(&equality, "left")
        .expect("equality left role resolves");
    let equality_right = program
        .designations()
        .role(&equality, "right")
        .expect("equality right role resolves");
    assert_eq!(
        equality_content.roles()[&equality_left],
        Term::referent(gravity)
    );
    assert_eq!(
        equality_content.roles()[&equality_right],
        Term::referent(measured_gravity)
    );

    let canonical = wire::serialize(revision);
    let reloaded = wire::reload(&canonical).expect("pure definition wire reloads");
    assert_eq!(reloaded, revision.clone());
    assert_eq!(wire::serialize(&reloaded), canonical);

    let renamed_program = compile_in(&SOURCE.replace("base", "subtotal"), id);
    assert_eq!(
        renamed_program
            .context_revision()
            .expect("renamed local compiles into the caller-owned Revision"),
        revision,
        "authoring-local spelling must not enter semantic identity"
    );

    let no_result = SOURCE.replace("  base + base\n", "");
    let error = frontend::parse(&no_result).expect_err("a local cannot double as the result");
    assert!(
        error
            .to_string()
            .contains("pure definition block requires one final result term"),
        "{error}"
    );

    let malformed = SOURCE.replace(
        "  base: gravity + measured gravity\n",
        "  gravity + measured gravity\n",
    );
    let error = frontend::parse(&malformed).expect_err("pre-result rows must be bindings");
    assert!(
        error.to_string().contains("':' only establishes a binding"),
        "{error}"
    );
}
