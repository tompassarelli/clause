use std::collections::BTreeMap;

use clause::{
    elaborate::{self, ElaborationContext},
    frontend,
    kernel::{ReferentId, RelationalContent, RoleId, SemanticAtom, Term},
    wire,
};

fn scope_id(byte: &str) -> ReferentId {
    ReferentId::new(format!("ref-sha256-{}", byte.repeat(64))).expect("fixed scope identity")
}

#[test]
fn invalid_snapshot_candidates_fail_closed() {
    let scope = scope_id("a");
    let duplicate = SemanticAtom::Referent(clause::kernel::Referent::new(scope.clone()));
    let candidate = elaborate::ProgramSnapshotCandidate::new(
        clause::kernel::ClauseSemanticsId::current(),
        scope.clone(),
        scope.clone(),
        vec![duplicate.clone(), duplicate],
    );
    assert!(elaborate::validate(candidate).is_err());

    let relation = scope_id("c");
    let role = RoleId::from_digest([0xef; 32]);
    let content = RelationalContent::new(
        relation,
        BTreeMap::from([(role, Term::referent(scope_id("0")))]),
    )
    .expect("content is structurally formed");
    let candidate = elaborate::ProgramSnapshotCandidate::new(
        clause::kernel::ClauseSemanticsId::current(),
        scope.clone(),
        scope,
        vec![SemanticAtom::RelationalContent(content)],
    );
    assert!(elaborate::validate(candidate).is_err());
}

fn compile_in(source: &str, id: ReferentId) -> elaborate::CompiledProgram {
    elaborate::compile_in(
        frontend::parse(source).expect("source parses"),
        ElaborationContext::new(id),
    )
    .expect("source compiles in context")
}

fn scoped(program: &elaborate::CompiledProgram, scope: &ReferentId, name: &str) -> ReferentId {
    program
        .designations()
        .scoped(scope, name)
        .expect("scoped designation resolves")
}

#[test]
fn direct_forms_require_an_explicit_elaboration_context() {
    let program = frontend::parse("Game\n\nChess ∈ Game\ngravity: 9.81\n").expect("forms parse");
    let error = elaborate::compile(program).expect_err("ambient content needs its caller");
    assert!(error.to_string().contains("explicit ElaborationContext"));
}

#[test]
fn top_level_membership_uses_the_exact_caller_owned_scope() {
    let id = scope_id("1");
    let program = compile_in("Game\n\nChess ∈ Game\n", id.clone());
    let revision = program.context_revision().expect("context Revision");
    assert_eq!(revision.model().id(), &id);

    let chess = scoped(&program, &id, "Chess");
    let game = program
        .designations()
        .global("Game")
        .expect("Game resolves");
    let membership = revision
        .model()
        .admitted_contents()
        .iter()
        .find(|content| {
            let terms = content.roles().values().collect::<Vec<_>>();
            terms.contains(&&Term::referent(chess.clone()))
                && terms.contains(&&Term::referent(game.clone()))
        })
        .expect("ordinary membership content");
    assert_eq!(
        revision.model().relation_shapes()[membership.relation()]
            .roles()
            .len(),
        2
    );
    assert!(
        revision.model().relation_shapes()[membership.relation()]
            .roles()
            .values()
            .all(|role| role.admissibility().is_empty())
    );
    assert!(
        revision
            .model()
            .definitions()
            .iter()
            .all(|item| item.id() != &chess)
    );
    assert!(
        revision
            .model()
            .occurrences()
            .iter()
            .all(|occurrence| { occurrence.source() == &id && occurrence.scope() == &id })
    );
}

#[test]
fn top_level_binding_is_a_stable_definition_in_the_same_model() {
    let id = scope_id("2");
    let program = compile_in("gravity: 9.81\n", id.clone());
    let model = program
        .context_revision()
        .expect("context Revision")
        .model();
    let gravity = scoped(&program, &id, "gravity");
    assert!(model.definitions().iter().any(|definition| {
        definition.id() == &gravity
            && definition.denotation() == &Term::f32(9.81).expect("9.81 is finite")
    }));
    assert!(model.admitted_contents().is_empty());
    assert!(model.occurrences().is_empty());
}

const RELATIONAL_PREFIX: &str = "Door\nPlace\n\nworld/connects: RelationShape\n  {door: Door} connects {origin: Place} to {destination: Place}\n  mode door -> origin, destination: many\n\nCellar ∈ Place\nArmory ∈ Place\niron-door ∈ Door\n";

#[test]
fn flattened_top_level_relation_uses_only_resolved_context_referents() {
    let id = scope_id("3");
    let program = compile_in(
        &format!("{RELATIONAL_PREFIX}iron-door connects Cellar to Armory\n"),
        id.clone(),
    );
    let model = program
        .context_revision()
        .expect("context Revision")
        .model();
    let relation = program
        .designations()
        .global("world/connects")
        .expect("relation resolves");
    let content = model
        .admitted_contents()
        .iter()
        .find(|content| content.relation() == &relation)
        .expect("role-labelled relational content");
    for (role, name) in [
        ("door", "iron-door"),
        ("origin", "Cellar"),
        ("destination", "Armory"),
    ] {
        let role = program
            .designations()
            .role(&relation, role)
            .expect("role resolves");
        assert_eq!(
            content.roles()[&role],
            Term::referent(scoped(&program, &id, name))
        );
    }
}

#[test]
fn focused_and_flattened_top_level_forms_have_identical_canonical_identity() {
    const PREFIX: &str = "Door\nPlace\nState\n\nworld/connects: RelationShape\n  {door: Door} connects {origin: Place} to {destination: Place}\n  mode door -> origin, destination: many\n\nCellar ∈ Place\nArmory ∈ Place\nlocked ∈ State\n";
    let focused =
        format!("{PREFIX}iron-door\n  Door\n  connects Cellar to Armory\n  state: locked\n");
    let flattened = format!(
        "{PREFIX}iron-door ∈ Door\niron-door connects Cellar to Armory\nstate of iron-door: locked\n"
    );
    let id = scope_id("4");
    let focused = compile_in(&focused, id.clone());
    let flattened = compile_in(&flattened, id.clone());
    let focused_revision = focused.context_revision().expect("focused Revision");
    let flattened_revision = flattened.context_revision().expect("flattened Revision");
    assert_eq!(focused_revision.model().id(), &id);
    assert_eq!(focused_revision.identity(), flattened_revision.identity());
    assert_eq!(
        wire::serialize(focused_revision),
        wire::serialize(flattened_revision)
    );
    assert_eq!(
        wire::reload(&wire::serialize(focused_revision)).expect("canonical wire reloads"),
        focused_revision.clone()
    );
    assert_eq!(focused_revision.model().definitions().len(), 1);
    assert_eq!(focused_revision.model().admitted_contents().len(), 5);
}

#[test]
fn semantic_source_changes_do_not_replace_the_supplied_scope_identity() {
    let id = scope_id("5");
    let first = compile_in("Game\n\nChess ∈ Game\n", id.clone());
    let second = compile_in("Game\n\nChess ∈ Game\nSoccer ∈ Game\n", id.clone());
    assert_eq!(first.context_revision().unwrap().model().id(), &id);
    assert_eq!(second.context_revision().unwrap().model().id(), &id);
    assert_ne!(
        first.context_revision().unwrap().identity(),
        second.context_revision().unwrap().identity()
    );
}

#[test]
fn contextual_designation_renames_preserve_binding_identity() {
    let id = scope_id("6");
    let before = compile_in("gravity: 9.81\n", id.clone());
    let mut designations = before.designations().clone();
    designations
        .retain_scoped(&id, "gravity", "weight")
        .expect("explicit scoped rename");
    let after = elaborate::compile_in(
        frontend::parse("weight: 9.81\n").expect("renamed source parses"),
        ElaborationContext::with_designations(id, designations),
    )
    .expect("renamed source compiles in the same context");
    assert_eq!(
        wire::serialize(before.context_revision().unwrap()),
        wire::serialize(after.context_revision().unwrap())
    );
}

#[test]
fn source_map_span_changes_do_not_change_frozen_revision_identity() {
    let id = scope_id("7");
    let compact = compile_in("Game\n\nChess ∈ Game\n", id.clone());
    let shifted = compile_in("Game\n\n\nChess ∈ Game\n", id);
    let compact_revision = compact.context_revision().expect("compact Revision");
    let shifted_revision = shifted.context_revision().expect("shifted Revision");
    let compact_occurrence = compact_revision
        .model()
        .occurrences()
        .first()
        .expect("compact membership occurrence");
    let shifted_occurrence = shifted_revision
        .model()
        .occurrences()
        .first()
        .expect("shifted membership occurrence");

    assert_eq!(compact_occurrence.id(), shifted_occurrence.id());
    assert_ne!(
        compact.source_span(compact_occurrence.id()),
        shifted.source_span(shifted_occurrence.id())
    );
    assert_ne!(compact.source_map(), shifted.source_map());
    assert_eq!(compact_revision.identity(), shifted_revision.identity());
    assert_eq!(
        wire::serialize(compact_revision),
        wire::serialize(shifted_revision)
    );
}

#[test]
fn unresolved_top_level_uses_fail_and_raw_editor_alias_is_not_grammar() {
    let unresolved = format!("{RELATIONAL_PREFIX}Missing connects Cellar to Armory\n");
    assert!(frontend::parse(&unresolved).is_err());
    assert!(frontend::parse("Game\n\nChess :: Game\n").is_err());
}

#[test]
fn compile_remains_compatible_for_self_contained_models() {
    let program = elaborate::compile(
        frontend::parse("Game\n\ncatalog\n  Chess ∈ Game\n").expect("source parses"),
    )
    .expect("self-contained Model compiles");
    let revision = program
        .revision(&frontend::Name("catalog".to_owned()))
        .expect("named Revision remains available");
    assert_eq!(
        wire::reload(&wire::serialize(revision)).expect("canonical wire reloads"),
        revision.clone()
    );
}
