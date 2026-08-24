//! Frozen source-to-typed-lowering table for the singular RelationalContent grammar.

use clause::{
    delta::RevisionDiff,
    elaborate, frontend,
    kernel::{Cardinality, ReferentId, RoleId, Term},
    m0_stage_a, m0_stage_b,
    request::{self, Request, Selection},
    wire,
};

const SOURCE: &str = include_str!("../examples/impact.clause");

fn program(source: &str) -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(source).expect("native source parses"))
        .expect("native source lowers")
}

fn global(program: &elaborate::CompiledProgram, value: &str) -> ReferentId {
    program
        .designations()
        .global(value)
        .expect("global designation resolves")
}

fn scoped(program: &elaborate::CompiledProgram, model: &ReferentId, value: &str) -> ReferentId {
    program
        .designations()
        .scoped(model, value)
        .expect("scoped designation resolves")
}

fn role(program: &elaborate::CompiledProgram, relation: &ReferentId, value: &str) -> RoleId {
    program
        .designations()
        .role(relation, value)
        .expect("role designation resolves")
}

#[test]
fn canonical_binding_membership_focus_and_input_boundary() {
    const PREFIX: &str = "Door: Type\nPlace: Type\nState: Type\nGame: Type\n\nworld/connects: RelationShape\n  {door: Door} connects {origin: Place} to {destination: Place}\n  mode door -> origin, destination: many\n\nworld: Model\n  gravity: 9.81\n  Chess ∈ Game\n  Cellar ∈ Place\n  Armory ∈ Place\n  locked ∈ State\n";
    let focused_source = format!(
        "{PREFIX}  iron-door\n    Door\n    connects Cellar to Armory\n    state: locked\n"
    );
    let expanded_source = format!(
        "{PREFIX}  iron-door ∈ Door\n  iron-door connects Cellar to Armory\n  state of iron-door: locked\n"
    );

    let focused_program = program(&focused_source);
    let expanded_program = program(&expanded_source);
    let focused = focused_program
        .revision(&frontend::Name("world".to_owned()))
        .expect("focused world Revision");
    let expanded = expanded_program
        .revision(&frontend::Name("world".to_owned()))
        .expect("expanded world Revision");
    assert_eq!(focused.identity(), expanded.identity());
    assert_eq!(wire::serialize(focused), wire::serialize(expanded));
    assert_eq!(
        wire::reload(&wire::serialize(focused)).expect("canonical focused wire reloads"),
        focused.clone()
    );

    let model = global(&focused_program, "world");
    let gravity = scoped(&focused_program, &model, "gravity");
    let scalar = scoped(&focused_program, &model, "9.81");
    let state = scoped(&focused_program, &model, "state of iron-door");
    let locked = scoped(&focused_program, &model, "locked");
    assert!(focused.model().definitions().iter().any(|definition| {
        definition.id() == &gravity && definition.denotation() == &Term::referent(scalar.clone())
    }));
    assert!(focused.model().definitions().iter().any(|definition| {
        definition.id() == &state && definition.denotation() == &Term::referent(locked.clone())
    }));

    let chess = scoped(&focused_program, &model, "Chess");
    let game = global(&focused_program, "Game");
    let chess_game = focused
        .model()
        .admitted_contents()
        .iter()
        .find(|content| {
            let terms = content.roles().values().collect::<Vec<_>>();
            terms.contains(&&Term::referent(chess.clone()))
                && terms.contains(&&Term::referent(game.clone()))
        })
        .expect("Chess membership is ordinary admitted relational content");
    let membership_shape = &focused.model().relation_shapes()[chess_game.relation()];
    assert_eq!(membership_shape.roles().len(), 2);
    assert!(
        membership_shape
            .roles()
            .values()
            .all(|role| role.admissibility().is_empty())
    );
    assert!(
        focused
            .model()
            .definitions()
            .iter()
            .all(|definition| definition.id() != &chess)
    );

    let raw_editor_source = expanded_source.replace("Chess ∈ Game", "Chess :: Game");
    assert!(frontend::parse(&raw_editor_source).is_err());
    let rewritten = m0_stage_b::rewrite_editor_input(&raw_editor_source);
    assert_eq!(rewritten.source, expanded_source);
    assert_eq!(rewritten.replaced.len(), 1);
    assert_eq!(
        wire::serialize(
            program(&rewritten.source)
                .revision(&frontend::Name("world".to_owned()))
                .expect("editor-normalized world Revision")
        ),
        wire::serialize(expanded)
    );

    assert!(frontend::parse(&expanded_source.replacen("  gravity", "    gravity", 1)).is_err());
    assert!(frontend::parse(&expanded_source.replacen("  gravity", "\tgravity", 1)).is_err());
    let operators = m0_stage_b::classify(&m0_stage_a::read(
        "x != y\nrender! scene\na / b\negress/route\n",
    ));
    assert!(operators.is_accepted(), "{:#?}", operators.diagnostics);
    assert_eq!(
        operators
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![
            m0_stage_b::StatementClass::RelationalContent,
            m0_stage_b::StatementClass::Effect,
            m0_stage_b::StatementClass::RelationalContent,
            m0_stage_b::StatementClass::UnresolvedStructuralForm,
        ]
    );
}

#[test]
fn thirteen_native_rows_lower_to_typed_revisions_and_requests() {
    let compiled = program(SOURCE);
    let base = compiled
        .revision(&frontend::Name("impact".to_owned()))
        .expect("base revision exists");
    let successor = compiled
        .revision(&frontend::Name("impact/adopt-south".to_owned()))
        .expect("successor revision exists");

    let module = global(&compiled, "Module");
    assert!(base.model().referents().contains_key(&module));

    // binary relation
    let imports_id = global(&compiled, "impact/imports");
    let consumer = role(&compiled, &imports_id, "consumer");
    let dependency = role(&compiled, &imports_id, "dependency");
    let imports = base
        .model()
        .relation_shapes()
        .get(&imports_id)
        .expect("typed imports RelationShape exists");
    assert_eq!(imports.roles().len(), 2);
    assert!(imports.roles().contains_key(&consumer));
    assert!(imports.roles().contains_key(&dependency));
    assert_eq!(imports.lookup()[0].known(), std::slice::from_ref(&consumer));
    assert_eq!(
        imports.lookup()[0].sought(),
        std::slice::from_ref(&dependency)
    );
    assert_eq!(imports.lookup()[0].cardinality(), &Cardinality::Many);

    // n-ary relation
    let nary = "Route: Type\nModule: Type\nZone: Type\nrouting/carries: RelationShape\n    {route: Route} carries {module: Module} through {zone: Zone}\n    mode route -> module, zone: many\nrouting: Model\n    R1: Route\n    Core: Module\n    East: Zone\n    R1 carries Core through East\n";
    let routing = program(nary);
    let carries_id = global(&routing, "routing/carries");
    let carries = routing
        .revision(&frontend::Name("routing".to_owned()))
        .expect("routing Revision")
        .model()
        .relation_shapes()
        .get(&carries_id)
        .expect("n-ary relation");
    assert_eq!(carries.roles().len(), 3);
    assert_eq!(carries.lookup()[0].known().len(), 1);
    assert_eq!(carries.lookup()[0].sought().len(), 2);

    // scoped referent and role-labelled admitted relational content
    let impact_id = global(&compiled, "impact");
    let north = scoped(&compiled, &impact_id, "North");
    let store = scoped(&compiled, &impact_id, "Store");
    assert!(base.model().referents().contains_key(&north));
    let north_store = base
        .model()
        .admitted_contents()
        .iter()
        .find(|content| {
            content.relation() == &imports_id
                && content.roles()[&consumer] == Term::referent(north.clone())
                && content.roles()[&dependency] == Term::referent(store.clone())
        })
        .expect("role-labelled North imports Store assertion");
    assert_eq!(north_store.roles().len(), 2);

    // derivation rule
    let direct_dependency = global(&compiled, "impact/direct-dependency");
    assert!(
        base.model()
            .derivation_rules()
            .iter()
            .any(|rule| rule.id() == &direct_dependency && rule.premises().forms().len() == 1)
    );

    // Revision/Delta outcome is one typed admission and names stay outside bytes.
    let added = RevisionDiff::between(base, successor)
        .expect("same declaration diff")
        .added()
        .to_vec();
    assert_eq!(added.len(), 1);
    assert_ne!(base.identity(), successor.identity());
    assert!(!wire::semantic_payload(successor).contains("adopt-south"));

    // Equivalent content admitted by a reusable Delta remains semantically
    // equal without collapsing the two authored assertion occurrences.
    let reusable = SOURCE.replace(
        "impact/adopt-south: Revision\n    from: impact\n    admit:\n        South imports North",
        "impact/add-south: Delta\n    from: impact\n    admit:\n        South imports North\n\nimpact/adopt-south: Revision\n    from: impact\n    apply: impact/add-south",
    );
    let reusable = program(&reusable);
    let reusable_successor = reusable
        .revision(&frontend::Name("impact/adopt-south".to_owned()))
        .expect("reusable Delta successor");
    assert_eq!(
        successor.model().admitted_contents(),
        reusable_successor.model().admitted_contents()
    );
    assert_ne!(
        wire::serialize(successor),
        wire::serialize(reusable_successor)
    );

    // find, why, prevent, achieve, diff
    let resolved = request::resolve(&compiled).expect("requests resolve");
    assert!(matches!(
        resolved.requests(),
        [
            Request::Find { revision, .. },
            Request::Why { all: true, .. },
            Request::Prevent { selection: Selection::AllMinimal, .. },
            Request::Achieve { selection: Selection::OneMinimal, .. },
            Request::Diff { base: diff_base, successor: diff_successor }
        ] if revision == base.identity() && diff_base == base.identity() && diff_successor == successor.identity()
    ));
}

#[test]
fn native_parser_preserves_repeated_occurrences_and_rejects_invalid_rows() {
    for pieces in [
        ["rela", "tion"],
        ["mo", "del"],
        ["l", "aw"],
        ["in", "tent"],
        ["que", "ry"],
        ["cl", "aim"],
        ["requ", "ire"],
        ["fa", "ct"],
    ] {
        let retired = pieces.concat();
        assert!(frontend::parse(&format!("{retired} retired:")).is_err());
    }
    let sentence = ["sen", "tence:"].concat();
    assert!(
        frontend::parse(&SOURCE.replacen(
            "    {consumer: Module} imports {dependency: Module}",
            &format!("    {sentence} {{consumer}} imports {{dependency}}"),
            1,
        ))
        .is_err()
    );
    let quoted = ['\"', 'N', 'o', 'r', 't', 'h', '\"']
        .into_iter()
        .collect::<String>();
    assert!(
        frontend::parse(&SOURCE.replacen(
            "North imports Store",
            &format!("{quoted} imports Store"),
            1
        ))
        .is_err()
    );
    assert!(
        frontend::parse(&SOURCE.replacen("North imports Store", "Missing imports Store", 1))
            .is_err()
    );
    assert!(
        frontend::parse(&SOURCE.replacen(
            "North imports Store",
            "compiler-change imports Store",
            1
        ))
        .is_err()
    );
    assert!(
        frontend::parse(&SOURCE.replacen(
            "{consumer: Module} imports {dependency: Module}",
            "{consumer: Module} imports {consumer: Module}",
            1,
        ))
        .is_err()
    );
    assert!(frontend::parse(&SOURCE.replacen("North imports Store", "North imports", 1)).is_err());
    assert!(frontend::parse(&SOURCE.replacen(
        "impact/direct-dependency: DerivationRule\n    ?consumer depends on ?dependency\n    when:\n        ?consumer imports ?dependency",
        "impact/direct-dependency: DerivationRule\n    ?consumer depends on ?dependency\n    when:\n        ?consumer imports ?consumer",
        1,
    )).is_err());
    let duplicate_relation = SOURCE.replace(
        "impact/depends: RelationShape",
        "impact/imports-also: RelationShape\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact/depends: RelationShape",
    );
    assert!(frontend::parse(&duplicate_relation).is_err());
    assert!(frontend::parse(&SOURCE.replacen("    North: Module", "  North: Module", 1)).is_err());

    let cycle = SOURCE.replace(
        "impact/adopt-south: Revision\n    from: impact\n    admit:\n        South imports North",
        "impact/first: Revision\n    from: impact/second\n    admit:\n        South imports North\n\nimpact/second: Revision\n    from: impact/first\n    admit:\n        South imports North",
    );
    assert!(frontend::parse(&cycle).is_err());
    let base_mismatch = SOURCE.replace(
        "impact/adopt-south: Revision\n    from: impact\n    admit:\n        South imports North",
        "impact/remove: Delta\n    from: impact\n    withdraw:\n        North imports Store\n\nimpact/other: Revision\n    from: impact\n    admit:\n        South imports North\n\nimpact/adopt-south: Revision\n    from: impact/other\n    apply: impact/remove",
    );
    assert!(elaborate::compile(frontend::parse(&base_mismatch).unwrap()).is_err());
    let duplicate = SOURCE.replacen(
        "        South imports North",
        "        South imports North\n        South imports North",
        1,
    );
    let duplicate = program(&duplicate);
    let successor = duplicate
        .revision(&frontend::Name("impact/adopt-south".to_owned()))
        .expect("duplicate occurrence successor exists");
    let imports = global(&duplicate, "impact/imports");
    let model = global(&duplicate, "impact");
    let south = scoped(&duplicate, &model, "South");
    let north = scoped(&duplicate, &model, "North");
    let consumer = role(&duplicate, &imports, "consumer");
    let dependency = role(&duplicate, &imports, "dependency");
    let content = successor
        .model()
        .admitted_contents()
        .iter()
        .find(|content| {
            content.relation() == &imports
                && content.roles()[&consumer] == Term::referent(south.clone())
                && content.roles()[&dependency] == Term::referent(north.clone())
        })
        .expect("repeated content remains admitted once");
    assert_eq!(
        successor
            .model()
            .occurrences()
            .iter()
            .filter(|occurrence| occurrence.content() == content.id())
            .count(),
        2,
    );
    let overlap = SOURCE.replace(
        "    admit:\n        South imports North",
        "    admit:\n        South imports North\n    withdraw:\n        South imports North",
    );
    assert!(frontend::parse(&overlap).is_err());
}
