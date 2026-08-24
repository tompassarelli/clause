use std::collections::{BTreeMap, BTreeSet};

use super::{
    error::{KernelError, Result},
    identity::{ReferentId, RoleId},
};

/// One addressable semantic distinction. Designations and every fact about a
/// referent live outside this identity-bearing value.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Referent {
    id: ReferentId,
}

impl Referent {
    pub fn new(id: ReferentId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &ReferentId {
        &self.id
    }
}

/// One ordinary relational pattern used to decide whether a candidate may
/// occupy a role. Zero or more may apply to a role.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RolePredicate {
    relation: ReferentId,
    candidate_role: RoleId,
    fixed_roles: BTreeMap<RoleId, ReferentId>,
}

impl RolePredicate {
    pub fn new(
        relation: ReferentId,
        candidate_role: RoleId,
        fixed_roles: BTreeMap<RoleId, ReferentId>,
    ) -> Result<Self> {
        if fixed_roles.contains_key(&candidate_role) {
            return Err(KernelError::new(
                "role predicate cannot fix its candidate role",
            ));
        }
        Ok(Self {
            relation,
            candidate_role,
            fixed_roles,
        })
    }

    pub fn relation(&self) -> &ReferentId {
        &self.relation
    }
    pub fn candidate_role(&self) -> &RoleId {
        &self.candidate_role
    }
    pub fn fixed_roles(&self) -> &BTreeMap<RoleId, ReferentId> {
        &self.fixed_roles
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Role {
    id: RoleId,
    admissibility: Vec<RolePredicate>,
}

impl Role {
    pub fn new(id: RoleId, mut admissibility: Vec<RolePredicate>) -> Result<Self> {
        admissibility.sort();
        if admissibility.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(KernelError::new("duplicate role admissibility predicate"));
        }
        Ok(Self { id, admissibility })
    }

    pub fn id(&self) -> &RoleId {
        &self.id
    }

    pub fn admissibility(&self) -> &[RolePredicate] {
        &self.admissibility
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

/// A derived executable lookup contract. It is not judgment modality.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LookupMode {
    known: Vec<RoleId>,
    sought: Vec<RoleId>,
    cardinality: Cardinality,
}

impl LookupMode {
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
                "lookup contract must have disjoint nonempty known and sought roles",
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

/// A derived executable contract for a referent used in relational position.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationShape {
    referent: ReferentId,
    roles: BTreeMap<RoleId, Role>,
    lookup: Vec<LookupMode>,
}

impl RelationShape {
    pub fn new(
        referent: ReferentId,
        roles: BTreeMap<RoleId, Role>,
        mut lookup: Vec<LookupMode>,
    ) -> Result<Self> {
        if roles.is_empty() {
            return Err(KernelError::new("relation shape needs at least one role"));
        }
        if roles.iter().any(|(id, role)| id != role.id()) {
            return Err(KernelError::new(
                "relation shape role map key does not match identity",
            ));
        }
        for mode in &lookup {
            let covered = mode
                .known()
                .iter()
                .chain(mode.sought())
                .cloned()
                .collect::<BTreeSet<_>>();
            if roles.keys().cloned().collect::<BTreeSet<_>>() != covered {
                return Err(KernelError::new(
                    "lookup contract must classify every relation role",
                ));
            }
        }
        lookup.sort();
        lookup.dedup();
        Ok(Self {
            referent,
            roles,
            lookup,
        })
    }

    pub fn referent(&self) -> &ReferentId {
        &self.referent
    }

    pub fn roles(&self) -> &BTreeMap<RoleId, Role> {
        &self.roles
    }

    pub fn lookup(&self) -> &[LookupMode] {
        &self.lookup
    }
}

fn sorted_unique<T: Ord>(mut values: Vec<T>, where_: &str) -> Result<Vec<T>> {
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(KernelError::new(format!("duplicate {where_}")));
    }
    Ok(values)
}
