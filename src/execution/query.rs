//! Closure query projection for `execution::find`.

use crate::{
    derive::{self, Limits},
    kernel::{Clause, FindPlan, KernelError, Result, Revision, Term, VariableId},
};

pub(super) fn find(revision: &Revision, plan: &FindPlan, limits: Limits) -> Result<Vec<Term>> {
    revision.model().validate_clause(plan.pattern(), true)?;
    let sought = plan.sought();
    let sought_variable = plan
        .pattern()
        .roles()
        .get(sought)
        .and_then(Term::variable_id)
        .ok_or_else(|| KernelError::new("find plan sought role is not a variable"))?;
    let closure = derive::saturate(revision, limits)?;
    let mut bindings = closure
        .assertions()
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

fn matches_pattern(candidate: &Clause, pattern: &Clause, sought: &VariableId) -> bool {
    candidate.relation() == pattern.relation()
        && pattern.roles().iter().all(|(role, expected)| {
            let Some(actual) = candidate.roles().get(role) else {
                return false;
            };
            match expected.variable_id() {
                Some(variable) => variable == sought,
                None => actual == expected,
            }
        })
}
