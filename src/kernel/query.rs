use std::collections::{BTreeMap, BTreeSet};

use super::{
    KernelError, LookupMode, Model, PatternId, ReferentId, RelationalContent, Result, RoleId,
    RolePredicate, Term,
};

/// One bounded single-clause relational selection plan.
///
/// Holes are scoped pattern identities, not referents in the Model. `columns`
/// fixes their projection order independently of the role map's canonical
/// identity order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    pattern: RelationalContent,
    columns: Vec<PatternId>,
    relation: ReferentId,
    known: Vec<RoleId>,
    sought: Vec<RoleId>,
    mode: LookupMode,
}

impl QueryPlan {
    pub fn new(
        model: &Model,
        pattern: &RelationalContent,
        columns: Vec<PatternId>,
    ) -> Result<Self> {
        model.validate_content(pattern, true)?;
        if columns.is_empty() {
            return Err(KernelError::new(
                "query projection requires at least one hole",
            ));
        }
        let projected = columns.iter().cloned().collect::<BTreeSet<_>>();
        if projected.len() != columns.len() {
            return Err(KernelError::new(
                "query projection cannot repeat a hole column",
            ));
        }

        let shape = model
            .relation_shapes()
            .get(pattern.relation())
            .expect("validated query relation has a shape");
        let mut discovered = BTreeSet::new();
        let mut requirements = BTreeMap::<PatternId, Vec<RolePredicate>>::new();
        let mut known = Vec::new();
        let mut sought = Vec::new();
        for (role, term) in pattern.roles() {
            let mut role_patterns = BTreeSet::new();
            collect_patterns(model, term, &mut BTreeSet::new(), &mut role_patterns)?;
            if role_patterns.is_empty() {
                known.push(role.clone());
            } else {
                sought.push(role.clone());
                for pattern in role_patterns {
                    discovered.insert(pattern.clone());
                    let requirement = shape.roles()[role].admissibility().to_vec();
                    if requirements
                        .insert(pattern, requirement.clone())
                        .is_some_and(|previous| previous != requirement)
                    {
                        return Err(KernelError::new(
                            "query hole occurs under inconsistent role admissibility",
                        ));
                    }
                }
            }
        }
        if discovered != projected {
            return Err(KernelError::new(
                "query projection must name every hole exactly once",
            ));
        }
        let mode = shape
            .lookup()
            .iter()
            .find(|mode| mode.known() == known && mode.sought() == sought)
            .cloned()
            .ok_or_else(|| KernelError::new("no declared mode admits this query orientation"))?;
        Ok(Self {
            pattern: pattern.clone(),
            columns,
            relation: pattern.relation().clone(),
            known,
            sought,
            mode,
        })
    }

    pub fn pattern(&self) -> &RelationalContent {
        &self.pattern
    }

    pub fn columns(&self) -> &[PatternId] {
        &self.columns
    }

    pub fn relation(&self) -> &ReferentId {
        &self.relation
    }

    pub fn known(&self) -> &[RoleId] {
        &self.known
    }

    pub fn sought(&self) -> &[RoleId] {
        &self.sought
    }

    pub fn mode(&self) -> &LookupMode {
        &self.mode
    }
}

fn collect_patterns(
    model: &Model,
    term: &Term,
    active: &mut BTreeSet<crate::kernel::ContentId>,
    patterns: &mut BTreeSet<PatternId>,
) -> Result<()> {
    match term {
        Term::Pattern(id) => {
            patterns.insert(id.clone());
        }
        Term::Application(id) => {
            if !active.insert(id.clone()) {
                return Err(KernelError::new(
                    "query pattern application graph contains a cycle",
                ));
            }
            let content = model
                .content(id)
                .ok_or_else(|| KernelError::new("query term names undeclared content"))?;
            for term in content.roles().values() {
                collect_patterns(model, term, active, patterns)?;
            }
            active.remove(id);
        }
        Term::Product { fields, .. } => {
            for field in fields.values() {
                collect_patterns(model, field.value(), active, patterns)?;
            }
        }
        Term::LabelledProduct { fields, .. } => {
            for term in fields.values() {
                collect_patterns(model, term, active, patterns)?;
            }
        }
        Term::Sum { value, .. } => collect_patterns(model, value, active, patterns)?,
        Term::Sequence { values, .. } => {
            for term in values {
                collect_patterns(model, term, active, patterns)?;
            }
        }
        Term::Referent(_) | Term::F32(_) | Term::Int(_) | Term::Bool(_) => {}
    }
    Ok(())
}
