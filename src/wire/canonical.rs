use crate::kernel::{
    Clause, EntityId, Law, Mode, Model, Relation, Revision, RevisionId, RoleId, SentencePart, Term,
    Type,
};

use super::json::escape;
use super::sha256::sha256_digest;

pub const SEMANTIC_TAG: &str = "clause-semantic-v5";
pub const REVISION_TAG: &str = "clause-revision-v3";

/// Serialize the exact semantic-v5 identity preimage for an admitted Model.
pub fn semantic_payload(model: &Model) -> String {
    let types = model
        .types()
        .values()
        .map(type_json)
        .collect::<Vec<_>>()
        .join(",");
    let entities = model
        .entities()
        .iter()
        .map(entity_json)
        .collect::<Vec<_>>()
        .join(",");
    let relations = model
        .relations()
        .values()
        .map(relation_json)
        .collect::<Vec<_>>()
        .join(",");
    let assertions = model
        .assertions()
        .iter()
        .map(|assertion| clause_json("assertion", assertion))
        .collect::<Vec<_>>()
        .join(",");
    let laws = model
        .laws()
        .iter()
        .map(law_json)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "[\"{SEMANTIC_TAG}\",[\"model\",\"{}\"],[\"types\",[{types}]],[\"entities\",[{entities}]],[\"relations\",[{relations}]],[\"assertions\",[{assertions}]],[\"laws\",[{laws}]]]",
        escape(model.id().as_str())
    )
}

/// Derive the typed revision identity from the exact semantic-v5 bytes.
pub fn revision_id(model: &Model) -> RevisionId {
    RevisionId::from_digest(sha256_digest(semantic_payload(model).as_bytes()))
}

/// Admit a Model as a content-addressed immutable Revision.
pub fn admit(model: Model) -> Revision {
    let identity = revision_id(&model);
    Revision::reloaded(identity, model)
}

/// Serialize the sole live persisted Revision envelope.
pub fn serialize(revision: &Revision) -> String {
    format!(
        "[\"{REVISION_TAG}\",\"{}\",{}]",
        revision.identity(),
        semantic_payload(revision.model())
    )
}

fn type_json(typ: &Type) -> String {
    format!("[\"type\",\"{}\"]", escape(typ.id().as_str()))
}

fn entity_json(entity: &EntityId) -> String {
    format!(
        "[\"entity\",\"{}\",\"{}\",\"{}\"]",
        escape(entity.model().as_str()),
        escape(entity.local().as_str()),
        escape(entity.typ().as_str())
    )
}

fn relation_json(relation: &Relation) -> String {
    let roles = relation
        .roles()
        .values()
        .map(|role| {
            format!(
                "[\"role\",\"{}\",\"{}\"]",
                escape(role.id().as_str()),
                escape(role.typ().as_str())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let shape = relation
        .shape()
        .parts()
        .iter()
        .map(shape_part_json)
        .collect::<Vec<_>>()
        .join(",");
    let modes = relation
        .modes()
        .iter()
        .map(mode_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"relation\",\"{}\",[\"roles\",[{roles}]],[\"shape\",[{shape}]],[\"modes\",[{modes}]]]",
        escape(relation.id().as_str())
    )
}

fn shape_part_json(part: &SentencePart) -> String {
    match part {
        SentencePart::Literal(literal) => {
            format!("[\"literal\",\"{}\"]", escape(literal))
        }
        SentencePart::Role(role) => format!("[\"role\",\"{}\"]", escape(role.as_str())),
    }
}

fn mode_json(mode: &Mode) -> String {
    let known = string_list(mode.known().iter().map(RoleId::as_str));
    let sought = string_list(mode.sought().iter().map(RoleId::as_str));
    format!(
        "[\"mode\",[\"known\",[{known}]],[\"sought\",[{sought}]],[\"cardinality\",\"{}\"]]",
        mode.cardinality().as_str()
    )
}

fn string_list<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values
        .map(|value| format!("\"{}\"", escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn clause_json(kind: &str, clause: &Clause) -> String {
    let roles = clause
        .roles()
        .iter()
        .map(|(role, term)| format!("[\"{}\",{}]", escape(role.as_str()), term_json(term)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"{kind}\",\"{}\",[\"roles\",[{roles}]]]",
        escape(clause.relation().as_str())
    )
}

fn law_json(law: &Law) -> String {
    let premises = law
        .premises()
        .iter()
        .map(|premise| clause_json("premise", premise))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"law\",\"{}\",[\"premises\",[{premises}]],[\"conclusion\",{}]]",
        escape(law.id().as_str()),
        clause_json("conclusion", law.conclusion())
    )
}

fn term_json(term: &Term) -> String {
    match term {
        Term::Entity(entity) => entity_json(entity),
        Term::Value { typ, canonical } => format!(
            "[\"value\",\"{}\",\"{}\"]",
            escape(typ.as_str()),
            escape(canonical)
        ),
        Term::Variable { id, typ } => format!(
            "[\"variable\",\"{}\",\"{}\"]",
            escape(id.as_str()),
            escape(typ.as_str())
        ),
    }
}
