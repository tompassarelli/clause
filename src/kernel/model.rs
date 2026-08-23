use std::collections::{BTreeMap, BTreeSet};

use super::{
    clause::{Clause, Law, Term},
    error::{KernelError, Result},
    identity::{EntityId, ModelId, RelationId, TypeId, VariableId},
    schema::{Relation, Type},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    id: ModelId,
    types: BTreeMap<TypeId, Type>,
    entities: BTreeSet<EntityId>,
    relations: BTreeMap<RelationId, Relation>,
    assertions: Vec<Clause>,
    laws: Vec<Law>,
}

impl Model {
    pub fn new(
        id: ModelId,
        types: BTreeMap<TypeId, Type>,
        entities: BTreeSet<EntityId>,
        relations: BTreeMap<RelationId, Relation>,
        mut assertions: Vec<Clause>,
        mut laws: Vec<Law>,
    ) -> Result<Self> {
        if types.iter().any(|(identity, typ)| typ.id() != identity) {
            return Err(KernelError::new(
                "type map key must match its Type identity",
            ));
        }
        if entities
            .iter()
            .any(|entity| entity.model() != &id || !types.contains_key(entity.typ()))
        {
            return Err(KernelError::new(
                "entity must belong to this model and declare an admitted type",
            ));
        }
        if relations
            .iter()
            .any(|(identity, relation)| relation.id() != identity)
        {
            return Err(KernelError::new(
                "relation map key must match its Relation identity",
            ));
        }
        for relation in relations.values() {
            if relation
                .roles()
                .values()
                .any(|role| !types.contains_key(role.typ()))
            {
                return Err(KernelError::new("relation role has an undeclared type"));
            }
        }
        for assertion in &assertions {
            validate_clause(&id, &types, &entities, &relations, assertion, false)?;
        }
        assertions.sort();
        assertions.dedup();
        let mut law_ids = BTreeSet::new();
        for law in &laws {
            if !law_ids.insert(law.id().clone()) {
                return Err(KernelError::new("duplicate law identity"));
            }
            validate_law(&id, &types, &entities, &relations, law)?;
        }
        laws.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(Self {
            id,
            types,
            entities,
            relations,
            assertions,
            laws,
        })
    }

    pub fn id(&self) -> &ModelId {
        &self.id
    }

    pub fn types(&self) -> &BTreeMap<TypeId, Type> {
        &self.types
    }

    pub fn entities(&self) -> &BTreeSet<EntityId> {
        &self.entities
    }

    pub fn relations(&self) -> &BTreeMap<RelationId, Relation> {
        &self.relations
    }

    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }

    pub fn laws(&self) -> &[Law] {
        &self.laws
    }

    pub fn validate_clause(&self, clause: &Clause, allow_variables: bool) -> Result<()> {
        validate_clause(
            &self.id,
            &self.types,
            &self.entities,
            &self.relations,
            clause,
            allow_variables,
        )
    }

    /// Rebuild this semantic model with a replacement asserted-clause set.
    /// Delta application uses this named operation so the remaining model
    /// fields cannot be accidentally reordered during a breaking migration.
    pub fn with_assertions(&self, assertions: Vec<Clause>) -> Result<Self> {
        Self::new(
            self.id.clone(),
            self.types.clone(),
            self.entities.clone(),
            self.relations.clone(),
            assertions,
            self.laws.clone(),
        )
    }
}

fn validate_clause(
    model: &ModelId,
    types: &BTreeMap<TypeId, Type>,
    entities: &BTreeSet<EntityId>,
    relations: &BTreeMap<RelationId, Relation>,
    clause: &Clause,
    allow_variables: bool,
) -> Result<()> {
    let relation = relations
        .get(clause.relation())
        .ok_or_else(|| KernelError::new("clause relation is undeclared"))?;
    if clause.roles().keys().ne(relation.roles().keys()) {
        return Err(KernelError::new(
            "clause must fill the complete named role map",
        ));
    }
    for (role_id, term) in clause.roles() {
        let role = relation
            .roles()
            .get(role_id)
            .expect("complete role map was checked");
        if term.typ() != role.typ() || !types.contains_key(term.typ()) {
            return Err(KernelError::new(
                "clause term type does not match its role type",
            ));
        }
        match term {
            Term::Entity(entity) if entity.model() != model || !entities.contains(entity) => {
                return Err(KernelError::new(
                    "clause entity is not admitted by this model",
                ));
            }
            Term::Value { typ, canonical }
                if typ.as_str() != "Text"
                    || canonical.is_empty()
                    || canonical.chars().any(char::is_control) =>
            {
                return Err(KernelError::new(
                    "clause scalar values must be canonical Text",
                ));
            }
            Term::Variable { .. } if !allow_variables => {
                return Err(KernelError::new(
                    "assertions and delta changes must be ground",
                ));
            }
            Term::Entity(_) | Term::Value { .. } | Term::Variable { .. } => {}
        }
    }
    Ok(())
}

fn validate_law(
    model: &ModelId,
    types: &BTreeMap<TypeId, Type>,
    entities: &BTreeSet<EntityId>,
    relations: &BTreeMap<RelationId, Relation>,
    law: &Law,
) -> Result<()> {
    let mut premise_variables = BTreeSet::new();
    let mut variable_types = BTreeMap::new();
    for premise in law.premises() {
        validate_clause(model, types, entities, relations, premise, true)?;
        record_variables(premise, &mut variable_types, Some(&mut premise_variables))?;
    }
    validate_clause(model, types, entities, relations, law.conclusion(), true)?;
    record_variables(law.conclusion(), &mut variable_types, None)?;
    if law
        .conclusion()
        .roles()
        .values()
        .filter_map(Term::variable_id)
        .any(|variable| !premise_variables.contains(variable))
    {
        return Err(KernelError::new(
            "every conclusion variable must occur in a premise",
        ));
    }
    Ok(())
}

fn record_variables(
    clause: &Clause,
    variable_types: &mut BTreeMap<VariableId, TypeId>,
    mut variables: Option<&mut BTreeSet<VariableId>>,
) -> Result<()> {
    for term in clause.roles().values() {
        let Some(variable) = term.variable_id() else {
            continue;
        };
        if variable_types
            .insert(variable.clone(), term.typ().clone())
            .is_some_and(|previous| previous != *term.typ())
        {
            return Err(KernelError::new(
                "law variable occurs at inconsistent declared role types",
            ));
        }
        if let Some(variables) = variables.as_deref_mut() {
            variables.insert(variable.clone());
        }
    }
    Ok(())
}
