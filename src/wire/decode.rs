use std::collections::{BTreeMap, BTreeSet};

use crate::kernel::{
    Cardinality, Clause, EntityId, InlineSentencePart, KernelError, Law, LawId, Mode, Model,
    ModelId, Name, Relation, RelationId, Result, Revision, RevisionId, Role, RoleId, SentenceShape,
    Term, Type, TypeId, VariableId,
};

use super::canonical::{REVISION_TAG, SEMANTIC_TAG, revision_id, semantic_payload, serialize};
use super::json::{Json, JsonParser, array, json, list, require_string, string};

/// Strictly reload one canonical v3/v5 Revision.
pub fn reload(bytes: &str) -> Result<Revision> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new("revision wire is not canonical JSON"));
    }

    let envelope = list(&value, 3, "revision envelope")?;
    require_string(&envelope[0], REVISION_TAG, "revision envelope tag")?;
    let claimed = decode_revision_id(string(&envelope[1], "revision identity")?)?;
    let model = decode_model(&envelope[2])?;
    let expected = revision_id(&model);
    if claimed != expected {
        return Err(KernelError::new(
            "revision identity does not match canonical semantic payload",
        ));
    }

    let revision = Revision::reloaded(claimed, model);
    if serialize(&revision) != bytes {
        return Err(KernelError::new("revision payload is not canonical"));
    }
    Ok(revision)
}

fn decode_model(value: &Json) -> Result<Model> {
    let root = list(value, 7, "semantic payload")?;
    require_string(&root[0], SEMANTIC_TAG, "semantic tag")?;

    let model_group = tagged_group(&root[1], "model", "model")?;
    let model = decode_model_id(model_group)?;

    let type_values = array(tagged_group(&root[2], "types", "types")?, "types body")?;
    let mut types = BTreeMap::new();
    for value in type_values {
        let typ = decode_type(value)?;
        if types.insert(typ.id().clone(), typ).is_some() {
            return Err(KernelError::new("duplicate type identity"));
        }
    }

    let entity_values = array(
        tagged_group(&root[3], "entities", "entities")?,
        "entities body",
    )?;
    let mut entities = BTreeSet::new();
    for value in entity_values {
        if !entities.insert(decode_entity(value)?) {
            return Err(KernelError::new("duplicate entity identity"));
        }
    }

    let relation_values = array(
        tagged_group(&root[4], "relations", "relations")?,
        "relations body",
    )?;
    let mut relations = BTreeMap::new();
    for value in relation_values {
        let relation = decode_relation(value)?;
        if relations.insert(relation.id().clone(), relation).is_some() {
            return Err(KernelError::new("duplicate relation identity"));
        }
    }

    let assertion_values = array(
        tagged_group(&root[5], "assertions", "assertions")?,
        "assertions body",
    )?;
    let assertions = assertion_values
        .iter()
        .map(|value| decode_clause(value, "assertion"))
        .collect::<Result<Vec<_>>>()?;

    let law_values = array(tagged_group(&root[6], "laws", "laws")?, "laws body")?;
    let laws = law_values
        .iter()
        .map(decode_law)
        .collect::<Result<Vec<_>>>()?;

    let model = Model::new(model, types, entities, relations, assertions, laws)?;
    if semantic_payload(&model) != json(value) {
        return Err(KernelError::new("semantic payload is not canonical"));
    }
    Ok(model)
}

fn decode_type(value: &Json) -> Result<Type> {
    let item = list(value, 2, "type")?;
    require_string(&item[0], "type", "type tag")?;
    Ok(Type::new(decode_type_id(&item[1])?))
}

fn decode_entity(value: &Json) -> Result<EntityId> {
    let item = list(value, 4, "entity")?;
    require_string(&item[0], "entity", "entity tag")?;
    EntityId::new(
        decode_model_id(&item[1])?,
        Name::entity_local(string(&item[2], "entity local")?.to_owned())?,
        decode_type_id(&item[3])?,
    )
}

fn decode_relation(value: &Json) -> Result<Relation> {
    let item = list(value, 5, "relation")?;
    require_string(&item[0], "relation", "relation tag")?;
    let identity = decode_relation_id(&item[1])?;

    let role_values = array(
        tagged_group(&item[2], "roles", "relation roles")?,
        "relation roles body",
    )?;
    let mut roles = BTreeMap::new();
    for value in role_values {
        let role = decode_role(value)?;
        if roles.insert(role.id().clone(), role).is_some() {
            return Err(KernelError::new("duplicate relation role"));
        }
    }

    let shape_values = array(
        tagged_group(&item[3], "shape", "relation shape")?,
        "relation shape body",
    )?;
    let mut parts = Vec::new();
    for value in shape_values {
        let part = array(value, "sentence shape part")?;
        let tag = part
            .first()
            .ok_or_else(|| KernelError::new("invalid sentence shape part"))?;
        match string(tag, "sentence shape part tag")? {
            "literal" => {
                let item = list(value, 2, "literal shape part")?;
                parts.push(InlineSentencePart::Literal(
                    string(&item[1], "sentence literal")?.to_owned(),
                ));
            }
            "role" => {
                let item = list(value, 2, "role shape part")?;
                let id = decode_role_id(&item[1])?;
                let role = roles
                    .get(&id)
                    .ok_or_else(|| KernelError::new("sentence shape names an unknown role"))?;
                parts.push(InlineSentencePart::Role(role.clone()));
            }
            _ => return Err(KernelError::new("invalid sentence shape part tag")),
        }
    }
    let shape = SentenceShape::new(parts)?;

    let mode_values = array(
        tagged_group(&item[4], "modes", "relation modes")?,
        "relation modes body",
    )?;
    let modes = mode_values
        .iter()
        .map(decode_mode)
        .collect::<Result<Vec<_>>>()?;
    let relation = Relation::new(identity, shape, modes)?;
    if relation.roles() != &roles {
        return Err(KernelError::new(
            "relation roles must exactly match its sentence shape",
        ));
    }
    Ok(relation)
}

fn decode_role(value: &Json) -> Result<Role> {
    let item = list(value, 3, "role")?;
    require_string(&item[0], "role", "role tag")?;
    Ok(Role::new(
        decode_role_id(&item[1])?,
        decode_type_id(&item[2])?,
    ))
}

fn decode_mode(value: &Json) -> Result<Mode> {
    let item = list(value, 4, "mode")?;
    require_string(&item[0], "mode", "mode tag")?;
    let known = decode_role_id_list(tagged_group(&item[1], "known", "mode known")?)?;
    let sought = decode_role_id_list(tagged_group(&item[2], "sought", "mode sought")?)?;
    let cardinality = match string(
        tagged_group(&item[3], "cardinality", "mode cardinality")?,
        "mode cardinality value",
    )? {
        "one" => Cardinality::One,
        "maybe" => Cardinality::Maybe,
        "some" => Cardinality::Some,
        "many" => Cardinality::Many,
        _ => return Err(KernelError::new("invalid mode cardinality")),
    };
    Mode::finite(known, sought, cardinality)
}

fn decode_role_id_list(value: &Json) -> Result<Vec<RoleId>> {
    array(value, "role identity list")?
        .iter()
        .map(decode_role_id)
        .collect()
}

fn decode_law(value: &Json) -> Result<Law> {
    let item = list(value, 4, "law")?;
    require_string(&item[0], "law", "law tag")?;
    let premises = array(
        tagged_group(&item[2], "premises", "law premises")?,
        "law premises body",
    )?
    .iter()
    .map(|value| decode_clause(value, "premise"))
    .collect::<Result<Vec<_>>>()?;
    let conclusion = decode_clause(
        tagged_group(&item[3], "conclusion", "law conclusion")?,
        "conclusion",
    )?;
    Law::new(decode_law_id(&item[1])?, premises, conclusion)
}

fn decode_clause(value: &Json, expected_kind: &str) -> Result<Clause> {
    let item = list(value, 3, "clause")?;
    require_string(&item[0], expected_kind, "clause tag")?;
    let relation = decode_relation_id(&item[1])?;
    let role_values = array(
        tagged_group(&item[2], "roles", "clause roles")?,
        "clause roles body",
    )?;
    let mut roles = BTreeMap::new();
    for value in role_values {
        let pair = list(value, 2, "clause role")?;
        let role = decode_role_id(&pair[0])?;
        if roles.insert(role, decode_term(&pair[1])?).is_some() {
            return Err(KernelError::new("duplicate clause role"));
        }
    }
    Clause::new(relation, roles)
}

fn decode_term(value: &Json) -> Result<Term> {
    let item = array(value, "term")?;
    let tag = item
        .first()
        .ok_or_else(|| KernelError::new("invalid term"))?;
    match string(tag, "term tag")? {
        "entity" => Ok(Term::entity(decode_entity(value)?)),
        "value" => {
            let item = list(value, 3, "value term")?;
            Term::value(
                decode_type_id(&item[1])?,
                string(&item[2], "canonical value")?.to_owned(),
            )
        }
        "variable" => {
            let item = list(value, 3, "variable term")?;
            Ok(Term::variable(
                decode_variable_id(&item[1])?,
                decode_type_id(&item[2])?,
            ))
        }
        _ => Err(KernelError::new("invalid term kind")),
    }
}

fn tagged_group<'a>(value: &'a Json, expected: &str, where_: &str) -> Result<&'a Json> {
    let item = list(value, 2, where_)?;
    require_string(&item[0], expected, &format!("{where_} tag"))?;
    Ok(&item[1])
}

fn decode_name(value: &Json, where_: &str) -> Result<Name> {
    Name::new(string(value, where_)?.to_owned())
}

fn decode_type_id(value: &Json) -> Result<TypeId> {
    TypeId::new(decode_name(value, "type identity")?)
}

fn decode_model_id(value: &Json) -> Result<ModelId> {
    ModelId::new(decode_name(value, "model identity")?)
}

fn decode_relation_id(value: &Json) -> Result<RelationId> {
    RelationId::new(decode_name(value, "relation identity")?)
}

fn decode_law_id(value: &Json) -> Result<LawId> {
    LawId::new(decode_name(value, "law identity")?)
}

fn decode_role_id(value: &Json) -> Result<RoleId> {
    RoleId::new(decode_name(value, "role identity")?)
}

fn decode_variable_id(value: &Json) -> Result<VariableId> {
    VariableId::new(decode_name(value, "variable identity")?)
}

fn decode_revision_id(value: &str) -> Result<RevisionId> {
    let hex = value
        .strip_prefix("rev-sha256-")
        .ok_or_else(|| KernelError::new("invalid revision identity"))?;
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(KernelError::new("invalid revision identity"));
    }
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| KernelError::new("invalid revision identity"))?;
    }
    Ok(RevisionId::from_digest(digest))
}
