use std::collections::BTreeMap;

use super::{
    error::{KernelError, Result},
    identity::{EntityId, LawId, RelationId, RoleId, TypeId, VariableId},
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Term {
    Entity(EntityId),
    Value { typ: TypeId, canonical: String },
    Variable { id: VariableId, typ: TypeId },
}

impl Term {
    pub fn entity(entity: EntityId) -> Self {
        Self::Entity(entity)
    }

    pub fn value(typ: TypeId, canonical: String) -> Result<Self> {
        if typ.as_str() != "Text" {
            return Err(KernelError::new(
                "only the admitted Text type may carry scalar values",
            ));
        }
        if canonical.is_empty() || canonical.chars().any(char::is_control) {
            return Err(KernelError::new("invalid canonical Text value"));
        }
        Ok(Self::Value { typ, canonical })
    }

    pub fn variable(id: VariableId, typ: TypeId) -> Self {
        Self::Variable { id, typ }
    }

    pub fn typ(&self) -> &TypeId {
        match self {
            Self::Entity(entity) => entity.typ(),
            Self::Value { typ, .. } | Self::Variable { typ, .. } => typ,
        }
    }

    pub fn variable_id(&self) -> Option<&VariableId> {
        match self {
            Self::Variable { id, .. } => Some(id),
            Self::Entity(_) | Self::Value { .. } => None,
        }
    }

    pub fn is_ground(&self) -> bool {
        self.variable_id().is_none()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Clause {
    relation: RelationId,
    roles: BTreeMap<RoleId, Term>,
}

impl Clause {
    pub fn new(relation: RelationId, roles: BTreeMap<RoleId, Term>) -> Result<Self> {
        if roles.is_empty() {
            return Err(KernelError::new("clause has no roles"));
        }
        Ok(Self { relation, roles })
    }

    pub fn relation(&self) -> &RelationId {
        &self.relation
    }

    pub fn roles(&self) -> &BTreeMap<RoleId, Term> {
        &self.roles
    }

    pub fn is_ground(&self) -> bool {
        self.roles.values().all(Term::is_ground)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Law {
    id: LawId,
    premises: Vec<Clause>,
    conclusion: Clause,
}

impl Law {
    pub fn new(id: LawId, premises: Vec<Clause>, conclusion: Clause) -> Result<Self> {
        if premises.is_empty() {
            return Err(KernelError::new("law needs at least one premise"));
        }
        Ok(Self {
            id,
            premises,
            conclusion,
        })
    }

    pub fn id(&self) -> &LawId {
        &self.id
    }

    pub fn premises(&self) -> &[Clause] {
        &self.premises
    }

    pub fn conclusion(&self) -> &Clause {
        &self.conclusion
    }
}
