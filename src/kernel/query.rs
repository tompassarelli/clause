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
    columns: Vec<QueryPlanColumn>,
    relation: ReferentId,
    known: Vec<RoleId>,
    sought: Vec<RoleId>,
    mode: LookupMode,
}

/// One projected binder and every stable relation role from which it arose.
///
/// Projection order is authored order. Role identities are canonical and do
/// not depend on an optional presentation label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlanColumn {
    binder: PatternId,
    origins: Vec<RoleId>,
}

impl QueryPlanColumn {
    pub fn new(binder: PatternId, origins: Vec<RoleId>) -> Self {
        Self { binder, origins }
    }

    pub fn binder(&self) -> &PatternId {
        &self.binder
    }

    pub fn origins(&self) -> &[RoleId] {
        &self.origins
    }
}

impl QueryPlan {
    pub fn new(
        model: &Model,
        pattern: &RelationalContent,
        columns: Vec<QueryPlanColumn>,
    ) -> Result<Self> {
        if columns.is_empty() {
            return Err(KernelError::new(
                "query projection requires at least one hole",
            ));
        }
        let projected = columns
            .iter()
            .map(|column| column.binder().clone())
            .collect::<BTreeSet<_>>();
        if projected.len() != columns.len() {
            return Err(KernelError::new(
                "query projection cannot repeat a hole column",
            ));
        }
        for column in &columns {
            if column.origins().is_empty() {
                return Err(KernelError::new(
                    "query column requires at least one role origin",
                ));
            }
            if column.origins().windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(KernelError::new(
                    "query column role origins must be sorted and deduplicated",
                ));
            }
        }

        let origins = pattern_origins(model, pattern)?;
        model.validate_content(pattern, true)?;

        let shape = model
            .relation_shapes()
            .get(pattern.relation())
            .expect("validated query relation has a shape");
        let mut discovered = BTreeSet::new();
        let mut requirements = BTreeMap::<PatternId, Vec<RolePredicate>>::new();
        let mut known = Vec::new();
        let mut sought = Vec::new();
        for (role, term) in pattern.roles() {
            let Some(pattern) = term.pattern_id() else {
                known.push(role.clone());
                continue;
            };
            sought.push(role.clone());
            discovered.insert(pattern.clone());
            let requirement = shape.roles()[role].admissibility().to_vec();
            if requirements
                .insert(pattern.clone(), requirement.clone())
                .is_some_and(|previous| previous != requirement)
            {
                return Err(KernelError::new(
                    "query hole occurs under inconsistent role admissibility",
                ));
            }
        }
        if discovered != projected {
            return Err(KernelError::new(
                "query projection must name every hole exactly once",
            ));
        }
        for column in &columns {
            let expected = origins
                .get(column.binder())
                .expect("every projected binder was discovered");
            if !expected.iter().eq(column.origins()) {
                return Err(KernelError::new(
                    "query column role origins do not match the pattern",
                ));
            }
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

    pub fn derive(
        model: &Model,
        pattern: &RelationalContent,
        binders: Vec<PatternId>,
    ) -> Result<Self> {
        let origins = pattern_origins(model, pattern)?;
        let columns = binders
            .into_iter()
            .map(|binder| {
                let origins = origins
                    .get(&binder)
                    .ok_or_else(|| {
                        KernelError::new("query projection must name every hole exactly once")
                    })?
                    .iter()
                    .cloned()
                    .collect();
                Ok(QueryPlanColumn::new(binder, origins))
            })
            .collect::<Result<Vec<_>>>()?;
        Self::new(model, pattern, columns)
    }

    pub fn pattern(&self) -> &RelationalContent {
        &self.pattern
    }

    pub fn columns(&self) -> &[QueryPlanColumn] {
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

fn pattern_origins(
    model: &Model,
    pattern: &RelationalContent,
) -> Result<BTreeMap<PatternId, BTreeSet<RoleId>>> {
    let mut origins = BTreeMap::<PatternId, BTreeSet<RoleId>>::new();
    for (role, term) in pattern.roles() {
        if let Some(pattern) = term.pattern_id() {
            origins
                .entry(pattern.clone())
                .or_default()
                .insert(role.clone());
            continue;
        }
        let mut nested = BTreeSet::new();
        collect_patterns(model, term, &mut BTreeSet::new(), &mut nested)?;
        if !nested.is_empty() {
            return Err(KernelError::new(
                "nested query holes are not admitted by M4/S1",
            ));
        }
    }
    Ok(origins)
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
