//! Canonical Clause semantic wire v5.
//!
//! The semantic payload is an ordered JSON array whose exact UTF-8 bytes are
//! the revision identity preimage. Reload admits only the v3 envelope and v5
//! payload and accepts no alternate ordering or JSON spelling.

use std::collections::{BTreeMap, BTreeSet};

use crate::kernel::{
    Cardinality, Clause, EntityId, InlineSentencePart, KernelError, Law, LawId, Mode, Model,
    ModelId, Name, Relation, RelationId, Result, Revision, RevisionId, Role, RoleId, SentencePart,
    SentenceShape, Term, Type, TypeId, VariableId,
};

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

fn escape(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{1f}' => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}

#[derive(Clone, Debug)]
enum Json {
    String(String),
    Array(Vec<Json>),
}

fn json(value: &Json) -> String {
    match value {
        Json::String(text) => format!("\"{}\"", escape(text)),
        Json::Array(values) => format!(
            "[{}]",
            values.iter().map(json).collect::<Vec<_>>().join(",")
        ),
    }
}

fn array<'a>(value: &'a Json, where_: &str) -> Result<&'a [Json]> {
    match value {
        Json::Array(values) => Ok(values),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
}

fn list<'a>(value: &'a Json, count: usize, where_: &str) -> Result<&'a [Json]> {
    let values = array(value, where_)?;
    if values.len() == count {
        Ok(values)
    } else {
        Err(KernelError::new(format!("invalid {where_}")))
    }
}

fn string<'a>(value: &'a Json, where_: &str) -> Result<&'a str> {
    match value {
        Json::String(text) => Ok(text),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
}

fn require_string(value: &Json, expected: &str, where_: &str) -> Result<()> {
    if string(value, where_)? == expected {
        Ok(())
    } else {
        Err(KernelError::new(format!("invalid {where_}")))
    }
}

struct JsonParser<'a> {
    input: &'a [u8],
    at: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            at: 0,
        }
    }

    fn parse(mut self) -> Result<Json> {
        let value = self.value()?;
        if self.at == self.input.len() {
            Ok(value)
        } else {
            Err(KernelError::new("trailing data in revision wire"))
        }
    }

    fn value(&mut self) -> Result<Json> {
        match self.peek() {
            Some(b'\"') => self.string().map(Json::String),
            Some(b'[') => self.array(),
            _ => Err(KernelError::new(
                "revision wire admits only arrays and strings",
            )),
        }
    }

    fn array(&mut self) -> Result<Json> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.value()?);
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Array(values));
                }
                _ => return Err(KernelError::new("invalid JSON array")),
            }
        }
    }

    fn string(&mut self) -> Result<String> {
        self.expect(b'\"')?;
        let mut output = String::new();
        loop {
            let byte = self
                .next()
                .ok_or_else(|| KernelError::new("unterminated JSON string"))?;
            match byte {
                b'\"' => return Ok(output),
                b'\\' => match self
                    .next()
                    .ok_or_else(|| KernelError::new("truncated JSON escape"))?
                {
                    b'\"' => output.push('"'),
                    b'\\' => output.push('\\'),
                    b'n' => output.push('\n'),
                    b'r' => output.push('\r'),
                    b't' => output.push('\t'),
                    b'u' => {
                        let hex = self.take(4)?;
                        let text = std::str::from_utf8(hex)
                            .map_err(|_| KernelError::new("invalid JSON unicode escape"))?;
                        let scalar = u32::from_str_radix(text, 16)
                            .map_err(|_| KernelError::new("invalid JSON unicode escape"))?;
                        output.push(
                            char::from_u32(scalar)
                                .ok_or_else(|| KernelError::new("invalid JSON unicode escape"))?,
                        );
                    }
                    _ => return Err(KernelError::new("invalid JSON escape")),
                },
                0..=0x1f => return Err(KernelError::new("control character in JSON string")),
                _ if byte < 0x80 => output.push(byte as char),
                _ => {
                    let length = utf8_width(byte)
                        .ok_or_else(|| KernelError::new("invalid UTF-8 in JSON string"))?;
                    let mut encoded = vec![byte];
                    encoded.extend_from_slice(self.take(length - 1)?);
                    output.push_str(
                        std::str::from_utf8(&encoded)
                            .map_err(|_| KernelError::new("invalid UTF-8 in JSON string"))?,
                    );
                }
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.at).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.at += 1;
        Some(byte)
    }

    fn expect(&mut self, expected: u8) -> Result<()> {
        if self.next() == Some(expected) {
            Ok(())
        } else {
            Err(KernelError::new("invalid JSON"))
        }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| KernelError::new("truncated JSON"))?;
        let result = self
            .input
            .get(self.at..end)
            .ok_or_else(|| KernelError::new("truncated JSON"))?;
        self.at = end;
        Ok(result)
    }
}

fn utf8_width(first: u8) -> Option<usize> {
    match first {
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}

fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut data = bytes.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    for block in data.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, chunk) in block.chunks_exact(4).enumerate().take(16) {
            words[index] = u32::from_be_bytes(chunk.try_into().expect("four bytes"));
        }
        for index in 16..64 {
            words[index] = words[index - 16]
                .wrapping_add(
                    words[index - 15].rotate_right(7)
                        ^ words[index - 15].rotate_right(18)
                        ^ (words[index - 15] >> 3),
                )
                .wrapping_add(words[index - 7])
                .wrapping_add(
                    words[index - 2].rotate_right(17)
                        ^ words[index - 2].rotate_right(19)
                        ^ (words[index - 2] >> 10),
                );
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
            state[0], state[1], state[2], state[3], state[4], state[5], state[6], state[7],
        );
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut digest = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// Lowercase SHA-256 hex, retained as a small public deterministic utility.
pub fn sha256_hex(bytes: &[u8]) -> String {
    sha256_digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
