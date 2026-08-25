//! Closure query projection for `execution::find`.

use std::collections::BTreeMap;

use crate::{
    derive::{self, Limits},
    kernel::{
        FindPlan, KernelError, PatternId, QueryPlan, RelationalContent, Result, Revision, Term,
    },
};

use super::{QueryCell, QueryRow};

pub(super) fn find(revision: &Revision, plan: &FindPlan, limits: Limits) -> Result<Vec<Term>> {
    revision.model().validate_content(plan.pattern(), true)?;
    let sought = plan.sought();
    let sought_variable = plan
        .pattern()
        .roles()
        .get(sought)
        .and_then(Term::pattern_id)
        .ok_or_else(|| KernelError::new("find plan sought role is not a variable"))?;
    let closure = derive::saturate(revision, limits)?;
    let mut bindings = closure
        .contents()
        .iter()
        .filter(|candidate| matches_pattern(candidate, plan.pattern(), sought_variable))
        .map(|candidate| {
            candidate
                .roles()
                .get(sought)
                .cloned()
                .ok_or_else(|| KernelError::new("closure clause does not fill sought role"))
        })
        .collect::<Result<Vec<_>>>()?;
    bindings.sort();
    bindings.dedup();
    Ok(bindings)
}

pub(super) fn select(
    revision: &Revision,
    plan: &QueryPlan,
    limits: Limits,
) -> Result<Vec<QueryRow>> {
    select_projected(revision, plan, plan.columns().len(), limits)
}

pub(super) fn select_projected(
    revision: &Revision,
    plan: &QueryPlan,
    projected: usize,
    limits: Limits,
) -> Result<Vec<QueryRow>> {
    let columns = plan
        .columns()
        .get(..projected)
        .ok_or_else(|| KernelError::new("query projection exceeds the complete plan"))?;
    let closure = derive::saturate(revision, limits)?;
    let mut rows = closure
        .contents()
        .iter()
        .filter_map(|candidate| {
            crate::kernel::matching::unify(
                plan.pattern(),
                candidate,
                &BTreeMap::new(),
                true,
                |id| plan.pattern_content(revision.model(), id),
                |id| closure.content(revision.model(), id),
            )
        })
        .map(|substitution| {
            columns
                .iter()
                .map(|column| {
                    substitution
                        .get(column.binder())
                        .cloned()
                        .ok_or_else(|| KernelError::new("query result did not bind every column"))
                        .map(|value| QueryCell {
                            origins: column.origins().to_vec(),
                            value,
                        })
                })
                .collect::<Result<Vec<_>>>()
                .map(|cells| QueryRow { cells })
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort();
    rows.dedup();
    Ok(rows)
}

pub(super) fn any(revision: &Revision, plan: &QueryPlan, limits: Limits) -> Result<bool> {
    let closure = derive::saturate(revision, limits)?;
    Ok(closure.contents().iter().any(|candidate| {
        crate::kernel::matching::unify(
            plan.pattern(),
            candidate,
            &BTreeMap::new(),
            true,
            |id| plan.pattern_content(revision.model(), id),
            |id| closure.content(revision.model(), id),
        )
        .is_some()
    }))
}

fn matches_pattern(
    candidate: &RelationalContent,
    pattern: &RelationalContent,
    sought: &PatternId,
) -> bool {
    candidate.relation() == pattern.relation()
        && pattern.roles().iter().all(|(role, expected)| {
            let Some(actual) = candidate.roles().get(role) else {
                return false;
            };
            match expected.pattern_id() {
                Some(variable) => variable == sought,
                None => actual == expected,
            }
        })
}
