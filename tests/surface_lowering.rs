//! Frozen source-to-typed-lowering table for the singular Clause grammar.

use clause::{
    delta::RevisionDiff,
    elaborate, frontend,
    kernel::{self, Cardinality, EntityId, Name, RelationId, RoleId, SentencePart, TypeId},
    request::{self, Request, Selection},
    wire,
};

const SOURCE: &str = include_str!("../examples/impact.clause");

fn program(source: &str) -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(source).expect("native source parses"))
        .expect("native source lowers")
}

fn name(value: &str) -> Name {
    Name::new(value.to_owned()).expect("valid stable name")
}

fn type_id(value: &str) -> TypeId {
    TypeId::new(name(value)).expect("valid Type identity")
}

fn relation_id(value: &str) -> RelationId {
    RelationId::new(name(value)).expect("valid Relation identity")
}

fn role_id(value: &str) -> RoleId {
    RoleId::new(name(value)).expect("valid Role identity")
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

    // type
    assert!(base.model().types().contains_key(&type_id("Module")));

    // binary relation
    let imports = base
        .model()
        .relations()
        .get(&relation_id("impact/imports"))
        .expect("typed imports Relation exists");
    assert_eq!(imports.roles().len(), 2);
    assert_eq!(
        imports.roles()[&role_id("consumer")].typ(),
        &type_id("Module")
    );
    assert_eq!(
        imports.roles()[&role_id("dependency")].typ(),
        &type_id("Module")
    );
    assert!(matches!(
        imports.shape().parts(),
        [SentencePart::Role(left), SentencePart::Literal(word), SentencePart::Role(right)]
            if left == &role_id("consumer") && word == "imports" && right == &role_id("dependency")
    ));
    assert_eq!(imports.modes()[0].cardinality(), &Cardinality::Many);

    // n-ary relation
    let nary = "Route: Type\nModule: Type\nZone: Type\nrouting/carries: Relation\n    {route: Route} carries {module: Module} through {zone: Zone}\n    mode route -> module, zone: many\nrouting: Model\n    R1: Route\n    Core: Module\n    East: Zone\n    R1 carries Core through East\n";
    let routing = program(nary);
    let carries = routing
        .revision(&frontend::Name("routing".to_owned()))
        .expect("routing Revision")
        .model()
        .relations()
        .get(&relation_id("routing/carries"))
        .expect("n-ary relation");
    assert_eq!(carries.roles().len(), 3);
    assert_eq!(carries.shape().parts().len(), 5);

    // typed entity and role-labelled asserted clause
    assert!(
        base.model().entities().contains(
            &EntityId::new(
                kernel::ModelId::new(name("impact")).unwrap(),
                name("North"),
                type_id("Module"),
            )
            .unwrap()
        )
    );
    let north_store = base
        .model()
        .assertions()
        .iter()
        .find(|assertion| {
            assertion.relation() == &relation_id("impact/imports")
                && assertion.roles()[&role_id("consumer")]
                    == kernel::Term::entity(
                        EntityId::new(
                            kernel::ModelId::new(name("impact")).unwrap(),
                            name("North"),
                            type_id("Module"),
                        )
                        .unwrap(),
                    )
                && assertion.roles()[&role_id("dependency")]
                    == kernel::Term::entity(
                        EntityId::new(
                            kernel::ModelId::new(name("impact")).unwrap(),
                            name("Store"),
                            type_id("Module"),
                        )
                        .unwrap(),
                    )
        })
        .expect("role-labelled North imports Store assertion");
    assert_eq!(north_store.roles().len(), 2);

    // law
    assert!(base
        .model()
        .laws()
        .iter()
        .any(|law| law.id().as_str() == "impact/direct-dependency" && law.premises().len() == 1));

    // Revision/Delta outcome is one typed admission and names stay outside bytes.
    let added = RevisionDiff::between(base, successor)
        .expect("same declaration diff")
        .added()
        .to_vec();
    assert_eq!(added.len(), 1);
    assert_ne!(base.identity(), successor.identity());
    assert!(!wire::semantic_payload(successor.model()).contains("adopt-south"));

    // Reusable Delta and equivalent direct Revision seal to identical bytes.
    let reusable = SOURCE.replace(
        "impact/adopt-south: Revision\n    from: impact\n    admit:\n        South imports North",
        "impact/add-south: Delta\n    from: impact\n    admit:\n        South imports North\n\nimpact/adopt-south: Revision\n    from: impact\n    apply: impact/add-south",
    );
    let reusable = program(&reusable);
    let reusable_successor = reusable
        .revision(&frontend::Name("impact/adopt-south".to_owned()))
        .expect("reusable Delta successor");
    assert_eq!(
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
fn native_parser_and_lowerer_reject_the_frozen_negative_rows() {
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
        "impact/direct-dependency: Law\n    ?consumer depends on ?dependency\n    when:\n        ?consumer imports ?dependency",
        "impact/direct-dependency: Law\n    ?consumer depends on ?dependency\n    when:\n        ?consumer imports ?consumer",
        1,
    )).is_err());
    let duplicate_relation = SOURCE.replace(
        "impact/depends: Relation",
        "impact/imports-also: Relation\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact/depends: Relation",
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
    assert!(frontend::parse(&duplicate).is_err());
    let overlap = SOURCE.replace(
        "    admit:\n        South imports North",
        "    admit:\n        South imports North\n    withdraw:\n        South imports North",
    );
    assert!(frontend::parse(&overlap).is_err());
}
