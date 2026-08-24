use super::{
    clause::RelationalContent,
    error::{KernelError, Result},
    identity::{PatternId, ReferentId, RoleId},
    model::Model,
    schema::LookupMode,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindPlan {
    pattern: RelationalContent,
    relation: ReferentId,
    known: Vec<RoleId>,
    sought: RoleId,
    mode: LookupMode,
}

impl FindPlan {
    pub fn new(model: &Model, pattern: &RelationalContent, sought: PatternId) -> Result<Self> {
        model.validate_content(pattern, true)?;
        let relation = model
            .relation_shapes()
            .get(pattern.relation())
            .expect("validated clause relation is declared");
        let mut sought_roles = pattern
            .roles()
            .iter()
            .filter(|(_, term)| term.pattern_id() == Some(&sought))
            .map(|(role, _)| role.clone())
            .collect::<Vec<_>>();
        if sought_roles.len() != 1
            || pattern
                .roles()
                .values()
                .any(|term| term.pattern_id().is_some_and(|id| id != &sought))
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
            .lookup()
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

    pub fn pattern(&self) -> &RelationalContent {
        &self.pattern
    }

    pub fn relation(&self) -> &ReferentId {
        &self.relation
    }

    pub fn known(&self) -> &[RoleId] {
        &self.known
    }

    pub fn sought(&self) -> &RoleId {
        &self.sought
    }

    pub fn mode(&self) -> &LookupMode {
        &self.mode
    }
}
