use std::collections::{BTreeMap, BTreeSet};

use super::{
    ContentId, KernelError, LookupMode, Model, PatternId, ReferentId, RelationalContent, Result,
    RoleId, RolePredicate, Term,
};

/// One bounded single-clause relational selection plan.
///
/// Holes are scoped pattern identities, not referents in the Model. `columns`
/// fixes their projection order independently of the role map's canonical
/// identity order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    pattern: RelationalContent,
    dependencies: Vec<RelationalContent>,
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
        mut dependencies: Vec<RelationalContent>,
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

        dependencies.sort_by(|left, right| left.id().cmp(right.id()));
        let origins = pattern_origins(model, pattern, &dependencies)?;
        model.validate_query_content(pattern, &dependencies)?;

        let shape = model
            .relation_shapes()
            .get(pattern.relation())
            .expect("validated query relation has a shape");
        let mut discovered = BTreeSet::new();
        let mut known = Vec::new();
        let mut sought = Vec::new();
        for (role, term) in pattern.roles() {
            if term.pattern_id().is_none() {
                known.push(role.clone());
                continue;
            }
            sought.push(role.clone());
        }
        discovered.extend(origins.keys().cloned());
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
            dependencies,
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
        dependencies: Vec<RelationalContent>,
        mut binders: Vec<PatternId>,
    ) -> Result<Self> {
        let origins = pattern_origins(model, pattern, &dependencies)?;
        let mut seen = binders.iter().cloned().collect::<BTreeSet<_>>();
        for binder in origins.keys() {
            if seen.insert(binder.clone()) {
                binders.push(binder.clone());
            }
        }
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
        Self::new(model, pattern, dependencies, columns)
    }

    pub fn pattern(&self) -> &RelationalContent {
        &self.pattern
    }

    pub fn dependencies(&self) -> &[RelationalContent] {
        &self.dependencies
    }

    pub(crate) fn pattern_content<'a>(
        &'a self,
        model: &'a Model,
        id: &ContentId,
    ) -> Option<&'a RelationalContent> {
        self.dependencies
            .binary_search_by(|content| content.id().cmp(id))
            .ok()
            .map(|index| &self.dependencies[index])
            .or_else(|| model.content(id))
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
    dependencies: &[RelationalContent],
) -> Result<BTreeMap<PatternId, BTreeSet<RoleId>>> {
    let mut origins = BTreeMap::<PatternId, BTreeSet<RoleId>>::new();
    let mut requirements = BTreeMap::<PatternId, Vec<RolePredicate>>::new();
    for (role, term) in pattern.roles() {
        collect_patterns(
            model,
            dependencies,
            term,
            role,
            model.relation_shapes()[pattern.relation()].roles()[role].admissibility(),
            &mut BTreeSet::new(),
            &mut origins,
            &mut requirements,
        )?;
    }
    Ok(origins)
}

#[allow(clippy::too_many_arguments)]
fn collect_patterns(
    model: &Model,
    dependencies: &[RelationalContent],
    term: &Term,
    origin: &RoleId,
    current: &[RolePredicate],
    active: &mut BTreeSet<crate::kernel::ContentId>,
    origins: &mut BTreeMap<PatternId, BTreeSet<RoleId>>,
    requirements: &mut BTreeMap<PatternId, Vec<RolePredicate>>,
) -> Result<()> {
    match term {
        Term::Pattern(id) => {
            origins
                .entry(id.clone())
                .or_default()
                .insert(origin.clone());
            if requirements
                .insert(id.clone(), current.to_vec())
                .is_some_and(|previous| previous != current)
            {
                return Err(KernelError::new(
                    "query hole occurs under inconsistent role admissibility",
                ));
            }
        }
        Term::Application(id) => {
            if !active.insert(id.clone()) {
                return Err(KernelError::new(
                    "query pattern application graph contains a cycle",
                ));
            }
            let content = dependencies
                .iter()
                .find(|content| content.id() == id)
                .or_else(|| model.content(id))
                .ok_or_else(|| KernelError::new("query term names undeclared content"))?;
            let shape = &model.relation_shapes()[content.relation()];
            for (role, term) in content.roles() {
                collect_patterns(
                    model,
                    dependencies,
                    term,
                    role,
                    shape.roles()[role].admissibility(),
                    active,
                    origins,
                    requirements,
                )?;
            }
            active.remove(id);
        }
        Term::Product { fields, .. } => {
            for field in fields.values() {
                collect_patterns(
                    model,
                    dependencies,
                    field.value(),
                    origin,
                    current,
                    active,
                    origins,
                    requirements,
                )?;
            }
        }
        Term::LabelledProduct { fields, .. } => {
            for term in fields.values() {
                collect_patterns(
                    model,
                    dependencies,
                    term,
                    origin,
                    current,
                    active,
                    origins,
                    requirements,
                )?;
            }
        }
        Term::Sum { value, .. } => collect_patterns(
            model,
            dependencies,
            value,
            origin,
            current,
            active,
            origins,
            requirements,
        )?,
        Term::Sequence { values, .. } => {
            for term in values {
                collect_patterns(
                    model,
                    dependencies,
                    term,
                    origin,
                    current,
                    active,
                    origins,
                    requirements,
                )?;
            }
        }
        Term::Referent(_) | Term::F32(_) | Term::Int(_) | Term::Bool(_) => {}
    }
    Ok(())
}
