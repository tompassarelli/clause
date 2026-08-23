use super::{
    clause::Clause,
    error::{KernelError, Result},
    identity::{RelationId, RoleId, VariableId},
    model::Model,
    schema::Mode,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindPlan {
    pattern: Clause,
    relation: RelationId,
    known: Vec<RoleId>,
    sought: RoleId,
    mode: Mode,
}

impl FindPlan {
    pub fn new(model: &Model, pattern: &Clause, sought: VariableId) -> Result<Self> {
        model.validate_clause(pattern, true)?;
        let relation = model
            .relations()
            .get(pattern.relation())
            .expect("validated clause relation is declared");
        let mut sought_roles = pattern
            .roles()
            .iter()
            .filter(|(_, term)| term.variable_id() == Some(&sought))
            .map(|(role, _)| role.clone())
            .collect::<Vec<_>>();
        if sought_roles.len() != 1
            || pattern
                .roles()
                .values()
                .any(|term| term.variable_id().is_some_and(|id| id != &sought))
        {
            return Err(KernelError::new(
                "find pattern must contain exactly one sought variable",
            ));
        }
        let known = pattern
            .roles()
            .iter()
            .filter(|(_, term)| term.is_ground())
            .map(|(role, _)| role.clone())
            .collect::<Vec<_>>();
        let sought_role = sought_roles.remove(0);
        let mode = relation
            .modes()
            .iter()
            .find(|mode| mode.known() == known && mode.sought() == [sought_role.clone()])
            .cloned()
            .ok_or_else(|| KernelError::new("no declared mode admits this find orientation"))?;
        Ok(Self {
            pattern: pattern.clone(),
            relation: pattern.relation().clone(),
            known,
            sought: sought_role,
            mode,
        })
    }

    pub fn pattern(&self) -> &Clause {
        &self.pattern
    }

    pub fn relation(&self) -> &RelationId {
        &self.relation
    }

    pub fn known(&self) -> &[RoleId] {
        &self.known
    }

    pub fn sought(&self) -> &RoleId {
        &self.sought
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }
}
