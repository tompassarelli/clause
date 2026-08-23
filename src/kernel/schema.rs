use std::collections::{BTreeMap, BTreeSet};

use super::{
    error::{KernelError, Result},
    identity::{RelationId, RoleId, TypeId},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    id: TypeId,
}

impl Type {
    pub fn new(id: TypeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &TypeId {
        &self.id
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Role {
    id: RoleId,
    typ: TypeId,
}

impl Role {
    pub fn new(id: RoleId, typ: TypeId) -> Self {
        Self { id, typ }
    }

    pub fn id(&self) -> &RoleId {
        &self.id
    }

    pub fn typ(&self) -> &TypeId {
        &self.typ
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cardinality {
    One,
    Maybe,
    Some,
    Many,
}

impl Cardinality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Maybe => "maybe",
            Self::Some => "some",
            Self::Many => "many",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mode {
    known: Vec<RoleId>,
    sought: Vec<RoleId>,
    cardinality: Cardinality,
}

impl Mode {
    pub fn finite(
        known: Vec<RoleId>,
        sought: Vec<RoleId>,
        cardinality: Cardinality,
    ) -> Result<Self> {
        let known = sorted_unique(known, "known role")?;
        let sought = sorted_unique(sought, "sought role")?;
        if known.is_empty()
            || sought.is_empty()
            || known.iter().any(|role| sought.binary_search(role).is_ok())
        {
            return Err(KernelError::new(
                "mode must have disjoint nonempty known and sought roles",
            ));
        }
        Ok(Self {
            known,
            sought,
            cardinality,
        })
    }

    pub fn known(&self) -> &[RoleId] {
        &self.known
    }

    pub fn sought(&self) -> &[RoleId] {
        &self.sought
    }

    pub fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SentencePart {
    Literal(String),
    Role(RoleId),
}

/// One inline shape. Role types travel with the shape until `Relation::new`
/// derives the Relation role map; the public parts remain the semantic n-ary
/// sentence pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceShape {
    parts: Vec<SentencePart>,
    inline_roles: BTreeMap<RoleId, Role>,
}

impl SentenceShape {
    pub fn new(parts: Vec<InlineSentencePart>) -> Result<Self> {
        if parts.len() < 3
            || !matches!(parts.first(), Some(InlineSentencePart::Role(_)))
            || !matches!(parts.last(), Some(InlineSentencePart::Role(_)))
        {
            return Err(KernelError::new(
                "sentence shape must begin and end with a role and contain a literal",
            ));
        }
        let mut inline_roles = BTreeMap::new();
        let mut canonical = Vec::with_capacity(parts.len());
        let mut role_count = 0;
        let mut previous_was_role = false;
        for part in parts {
            match part {
                InlineSentencePart::Role(role) => {
                    if previous_was_role {
                        return Err(KernelError::new(
                            "sentence roles need a literal between them",
                        ));
                    }
                    if inline_roles.insert(role.id.clone(), role.clone()).is_some() {
                        return Err(KernelError::new("duplicate inline relation role"));
                    }
                    canonical.push(SentencePart::Role(role.id));
                    role_count += 1;
                    previous_was_role = true;
                }
                InlineSentencePart::Literal(literal) => {
                    if !previous_was_role {
                        return Err(KernelError::new("sentence literals must follow a role"));
                    }
                    canonical.push(SentencePart::Literal(canonical_literal(literal)?));
                    previous_was_role = false;
                }
            }
        }
        if role_count < 2 {
            return Err(KernelError::new("relation needs at least two inline roles"));
        }
        Ok(Self {
            parts: canonical,
            inline_roles,
        })
    }

    pub fn parts(&self) -> &[SentencePart] {
        &self.parts
    }

    fn roles(&self) -> &BTreeMap<RoleId, Role> {
        &self.inline_roles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InlineSentencePart {
    Literal(String),
    Role(Role),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    id: RelationId,
    roles: BTreeMap<RoleId, Role>,
    shape: SentenceShape,
    modes: Vec<Mode>,
}

impl Relation {
    pub fn new(id: RelationId, shape: SentenceShape, mut modes: Vec<Mode>) -> Result<Self> {
        let roles = shape.roles().clone();
        for mode in &modes {
            let covered = mode
                .known()
                .iter()
                .chain(mode.sought())
                .cloned()
                .collect::<BTreeSet<_>>();
            if roles.keys().cloned().collect::<BTreeSet<_>>() != covered {
                return Err(KernelError::new("mode must classify every relation role"));
            }
        }
        modes.sort();
        modes.dedup();
        if modes.is_empty() {
            return Err(KernelError::new("relation needs a declared mode"));
        }
        Ok(Self {
            id,
            roles,
            shape,
            modes,
        })
    }

    pub fn id(&self) -> &RelationId {
        &self.id
    }

    pub fn roles(&self) -> &BTreeMap<RoleId, Role> {
        &self.roles
    }

    pub fn shape(&self) -> &SentenceShape {
        &self.shape
    }

    pub fn modes(&self) -> &[Mode] {
        &self.modes
    }
}

fn sorted_unique<T: Ord>(mut values: Vec<T>, where_: &str) -> Result<Vec<T>> {
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(KernelError::new(format!("duplicate {where_}")));
    }
    Ok(values)
}

fn canonical_literal(value: String) -> Result<String> {
    let literal = value.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    if literal.is_empty() {
        Err(KernelError::new("sentence literal cannot be empty"))
    } else {
        Ok(literal)
    }
}
