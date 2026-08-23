//! Exact typed Revision/Delta wire contract for the native Clause surface.

#![allow(unexpected_cfgs)]

#[cfg(g3_standalone)]
#[path = "../src/wire/canonical.rs"]
mod canonical;
#[cfg(g3_standalone)]
#[path = "../src/wire/decode.rs"]
mod decode;
#[cfg(g3_standalone)]
#[path = "../src/delta.rs"]
mod delta;
#[cfg(g3_standalone)]
#[path = "../src/wire/json.rs"]
mod json;
#[cfg(g3_standalone)]
#[path = "../src/kernel.rs"]
mod kernel;
#[cfg(g3_standalone)]
#[path = "../src/wire/sha256.rs"]
mod sha256;

#[cfg(g3_standalone)]
mod wire {
    pub use super::canonical::{admit, semantic_payload, serialize};
    pub use super::decode::reload;
    pub use super::sha256::sha256_hex;
}

use std::collections::{BTreeMap, BTreeSet};

#[cfg(not(g3_standalone))]
use clause::{
    delta::RevisionDiff,
    kernel::{
        Cardinality, Clause, Delta, EntityId, InlineSentencePart, Law, LawId, Mode, Model, ModelId,
        Name, Relation, RelationId, Role, RoleId, SentenceShape, Term, Type, TypeId, VariableId,
    },
    wire,
};
#[cfg(g3_standalone)]
use delta::RevisionDiff;
#[cfg(g3_standalone)]
use kernel::{
    Cardinality, Clause, Delta, EntityId, InlineSentencePart, Law, LawId, Mode, Model, ModelId,
    Name, Relation, RelationId, Role, RoleId, SentenceShape, Term, Type, TypeId, VariableId,
};

fn name(value: &str) -> Name {
    Name::new(value.to_owned()).unwrap()
}

fn type_id(value: &str) -> TypeId {
    TypeId::new(name(value)).unwrap()
}

fn model_id(value: &str) -> ModelId {
    ModelId::new(name(value)).unwrap()
}

fn relation_id(value: &str) -> RelationId {
    RelationId::new(name(value)).unwrap()
}

fn role_id(value: &str) -> RoleId {
    RoleId::new(name(value)).unwrap()
}

fn variable_id(value: &str) -> VariableId {
    VariableId::new(name(value)).unwrap()
}

fn law_id(value: &str) -> LawId {
    LawId::new(name(value)).unwrap()
}

fn entity(local: &str, typ: &str) -> EntityId {
    EntityId::new(model_id("routing"), name(local), type_id(typ)).unwrap()
}

fn carries(label: &str) -> Clause {
    Clause::new(
        relation_id("routing/carries"),
        BTreeMap::from([
            (
                role_id("label"),
                Term::value(type_id("Text"), label.to_owned()).unwrap(),
            ),
            (role_id("module"), Term::entity(entity("Core", "Module"))),
            (role_id("route"), Term::entity(entity("R1", "Route"))),
        ]),
    )
    .unwrap()
}

fn pattern() -> Clause {
    Clause::new(
        relation_id("routing/carries"),
        BTreeMap::from([
            (
                role_id("label"),
                Term::variable(variable_id("label"), type_id("Text")),
            ),
            (
                role_id("module"),
                Term::variable(variable_id("module"), type_id("Module")),
            ),
            (
                role_id("route"),
                Term::variable(variable_id("route"), type_id("Route")),
            ),
        ]),
    )
    .unwrap()
}

fn fixture_model(shape_literal: &str, assertions: Vec<Clause>) -> Model {
    let module = Type::new(type_id("Module"));
    let route = Type::new(type_id("Route"));
    let text = Type::new(type_id("Text"));
    let types = BTreeMap::from([
        (module.id().clone(), module),
        (route.id().clone(), route),
        (text.id().clone(), text),
    ]);
    let entities = BTreeSet::from([entity("Core", "Module"), entity("R1", "Route")]);
    let label_role = Role::new(role_id("label"), type_id("Text"));
    let module_role = Role::new(role_id("module"), type_id("Module"));
    let route_role = Role::new(role_id("route"), type_id("Route"));
    let relation = Relation::new(
        relation_id("routing/carries"),
        SentenceShape::new(vec![
            InlineSentencePart::Role(route_role),
            InlineSentencePart::Literal(shape_literal.to_owned()),
            InlineSentencePart::Role(module_role),
            InlineSentencePart::Literal("through".to_owned()),
            InlineSentencePart::Role(label_role),
        ])
        .unwrap(),
        vec![
            Mode::finite(
                vec![role_id("route")],
                vec![role_id("module"), role_id("label")],
                Cardinality::Many,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let relations = BTreeMap::from([(relation.id().clone(), relation)]);
    let rule = pattern();
    let laws = vec![Law::new(law_id("routing/reflexive"), vec![rule.clone()], rule).unwrap()];
    Model::new(
        model_id("routing"),
        types,
        entities,
        relations,
        assertions,
        laws,
    )
    .unwrap()
}

const EXPECTED_SEMANTIC: &str = "[\"clause-semantic-\u{76}5\",[\"model\",\"routing\"],[\"types\",[[\"type\",\"Module\"],[\"type\",\"Route\"],[\"type\",\"Text\"]]],[\"entities\",[[\"entity\",\"routing\",\"Core\",\"Module\"],[\"entity\",\"routing\",\"R1\",\"Route\"]]],[\"relations\",[[\"relation\",\"routing/carries\",[\"roles\",[[\"role\",\"label\",\"Text\"],[\"role\",\"module\",\"Module\"],[\"role\",\"route\",\"Route\"]]],[\"shape\",[[\"role\",\"route\"],[\"literal\",\"carries\"],[\"role\",\"module\"],[\"literal\",\"through\"],[\"role\",\"label\"]]],[\"modes\",[[\"mode\",[\"known\",[\"route\"]],[\"sought\",[\"label\",\"module\"]],[\"cardinality\",\"many\"]]]]]]],[\"assertions\",[[\"assertion\",\"routing/carries\",[\"roles\",[[\"label\",[\"value\",\"Text\",\"east\"]],[\"module\",[\"entity\",\"routing\",\"Core\",\"Module\"]],[\"route\",[\"entity\",\"routing\",\"R1\",\"Route\"]]]]]]],[\"laws\",[[\"law\",\"routing/reflexive\",[\"premises\",[[\"premise\",\"routing/carries\",[\"roles\",[[\"label\",[\"variable\",\"label\",\"Text\"]],[\"module\",[\"variable\",\"module\",\"Module\"]],[\"route\",[\"variable\",\"route\",\"Route\"]]]]]]],[\"conclusion\",[\"conclusion\",\"routing/carries\",[\"roles\",[[\"label\",[\"variable\",\"label\",\"Text\"]],[\"module\",[\"variable\",\"module\",\"Module\"]],[\"route\",[\"variable\",\"route\",\"Route\"]]]]]]]]]]";

const EXPECTED_REVISION_ID: &str =
    "rev-sha256-0591a6c41e3e593745161357f0da7824289cc2f1dc3d82d6d092769025e6db18";

#[test]
fn exact_v5_bytes_hash_and_v3_roundtrip_are_frozen() {
    assert_eq!(
        wire::sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let revision = wire::admit(fixture_model("carries", vec![carries("east")]));
    assert_eq!(wire::semantic_payload(revision.model()), EXPECTED_SEMANTIC);
    assert_eq!(revision.identity().to_string(), EXPECTED_REVISION_ID);
    let expected_wire =
        format!("[\"clause-revision-\u{76}3\",\"{EXPECTED_REVISION_ID}\",{EXPECTED_SEMANTIC}]");
    assert_eq!(wire::serialize(&revision), expected_wire);
    assert_eq!(wire::reload(&expected_wire).unwrap(), revision);
}

#[test]
fn reload_rejects_old_tags_noncanonical_order_and_hash_mismatch() {
    let revision = wire::admit(fixture_model("carries", vec![carries("east")]));
    let canonical = wire::serialize(&revision);
    let retired_revision_tag = ["clause-revision-", "v2"].concat();
    let retired_semantic_tag = ["clause-semantic-", "v4"].concat();
    assert!(
        wire::reload(&canonical.replacen("clause-revision-\u{76}3", &retired_revision_tag, 1))
            .is_err()
    );
    assert!(
        wire::reload(&canonical.replacen("clause-semantic-\u{76}5", &retired_semantic_tag, 1))
            .is_err()
    );
    assert!(wire::reload(&(" ".to_owned() + &canonical)).is_err());
    assert!(wire::reload(&(canonical.clone() + " ")).is_err());
    assert!(wire::reload(&canonical.replacen("rev-sha256-", "rev-sha256-a", 1)).is_err());

    let noncanonical_semantic = EXPECTED_SEMANTIC.replacen(
        "[\"type\",\"Module\"],[\"type\",\"Route\"],[\"type\",\"Text\"]",
        "[\"type\",\"Text\"],[\"type\",\"Route\"],[\"type\",\"Module\"]",
        1,
    );
    let recomputed = wire::sha256_hex(noncanonical_semantic.as_bytes());
    let noncanonical_wire = format!(
        "[\"clause-revision-\u{76}3\",\"rev-sha256-{recomputed}\",{noncanonical_semantic}]"
    );
    assert!(wire::reload(&noncanonical_wire).is_err());

    let noncanonical_groups =
        EXPECTED_SEMANTIC.replacen("[\"model\",\"routing\"],[\"types\"", "[\"types\"", 1);
    let noncanonical_groups = noncanonical_groups.replacen(
        "],[\"entities\"",
        "],[\"model\",\"routing\"],[\"entities\"",
        1,
    );
    let recomputed = wire::sha256_hex(noncanonical_groups.as_bytes());
    let noncanonical_wire =
        format!("[\"clause-revision-\u{76}3\",\"rev-sha256-{recomputed}\",{noncanonical_groups}]");
    assert!(wire::reload(&noncanonical_wire).is_err());
}

#[test]
fn reload_revalidates_typed_entities_even_with_a_recomputed_hash() {
    let invalid_semantic = EXPECTED_SEMANTIC.replacen(
        "[\"entity\",\"routing\",\"Core\",\"Module\"]",
        "[\"entity\",\"other\",\"Core\",\"Module\"]",
        1,
    );
    let recomputed = wire::sha256_hex(invalid_semantic.as_bytes());
    let invalid_wire =
        format!("[\"clause-revision-\u{76}3\",\"rev-sha256-{recomputed}\",{invalid_semantic}]");
    assert!(wire::reload(&invalid_wire).is_err());
}

#[test]
fn shape_wording_is_revision_material_but_not_relation_identity() {
    let base = wire::admit(fixture_model("carries", vec![carries("east")]));
    let renamed = wire::admit(fixture_model("transports", vec![carries("east")]));
    assert_eq!(
        base.model().relations().keys().collect::<Vec<_>>(),
        renamed.model().relations().keys().collect::<Vec<_>>()
    );
    assert_eq!(base.model().assertions(), renamed.model().assertions());
    assert_ne!(base.identity(), renamed.identity());
    assert!(RevisionDiff::between(&base, &renamed).is_err());
}

#[test]
fn typed_delta_applies_canonically_and_diff_reports_authored_changes() {
    let base = wire::admit(fixture_model("carries", vec![carries("east")]));
    let original = base.clone();
    let successor = Delta::new(
        base.identity().clone(),
        vec![carries("west")],
        vec![carries("east")],
    )
    .unwrap()
    .apply(&base)
    .unwrap();

    assert_eq!(base, original);
    assert_eq!(successor.model().id(), base.model().id());
    assert_eq!(successor.model().types(), base.model().types());
    assert_eq!(successor.model().entities(), base.model().entities());
    assert_eq!(successor.model().relations(), base.model().relations());
    assert_eq!(successor.model().laws(), base.model().laws());
    assert_eq!(successor.model().assertions(), [carries("west")]);

    let direct = wire::admit(base.model().with_assertions(vec![carries("west")]).unwrap());
    assert_eq!(successor, direct);
    let diff = RevisionDiff::between(&base, &successor).unwrap();
    assert_eq!(diff.base_revision(), base.identity());
    assert_eq!(diff.successor_revision(), successor.identity());
    assert_eq!(diff.added(), [carries("west")]);
    assert_eq!(diff.removed(), [carries("east")]);
}

#[test]
fn typed_delta_rejects_invalid_base_and_assertion_changes() {
    let base = wire::admit(fixture_model("carries", vec![carries("east")]));
    let other = wire::admit(fixture_model(
        "carries",
        vec![carries("east"), carries("west")],
    ));
    assert!(
        Delta::new(other.identity().clone(), vec![carries("west")], Vec::new())
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(
        Delta::new(base.identity().clone(), Vec::new(), vec![carries("west")])
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(
        Delta::new(base.identity().clone(), vec![carries("east")], Vec::new())
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(Delta::new(base.identity().clone(), Vec::new(), Vec::new()).is_err());
    assert!(
        Delta::new(
            base.identity().clone(),
            vec![carries("west"), carries("west")],
            Vec::new(),
        )
        .is_err()
    );
    assert!(
        Delta::new(
            base.identity().clone(),
            vec![carries("west")],
            vec![carries("west")],
        )
        .is_err()
    );
}
