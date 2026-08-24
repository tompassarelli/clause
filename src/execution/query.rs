//! Closure query projection for `execution::find`.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    derive::{self, Limits},
    kernel::{
        FindPlan, KernelError, Model, PatternId, QueryPlan, RelationalContent, Result, Revision,
        Term,
    },
};

use super::QueryRow;

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
    revision.model().validate_content(plan.pattern(), true)?;
    let closure = derive::saturate(revision, limits)?;
    let mut rows = closure
        .contents()
        .iter()
        .filter_map(|candidate| {
            unify_content(
                revision.model(),
                plan.pattern(),
                candidate,
                &BTreeMap::new(),
                &mut BTreeSet::new(),
            )
        })
        .map(|substitution| {
            plan.columns()
                .iter()
                .map(|column| {
                    substitution
                        .get(column)
                        .cloned()
                        .ok_or_else(|| KernelError::new("query result did not bind every column"))
                })
                .collect::<Result<Vec<_>>>()
                .map(|values| QueryRow { values })
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort();
    rows.dedup();
    Ok(rows)
}

fn unify_content(
    model: &Model,
    pattern: &RelationalContent,
    candidate: &RelationalContent,
    substitution: &BTreeMap<PatternId, Term>,
    active: &mut BTreeSet<(crate::kernel::ContentId, crate::kernel::ContentId)>,
) -> Option<BTreeMap<PatternId, Term>> {
    if pattern.relation() != candidate.relation()
        || pattern.roles().keys().ne(candidate.roles().keys())
    {
        return None;
    }
    let mut substitution = substitution.clone();
    for (role, expected) in pattern.roles() {
        unify_term(
            model,
            expected,
            &candidate.roles()[role],
            &mut substitution,
            active,
        )?;
    }
    Some(substitution)
}

fn unify_term(
    model: &Model,
    pattern: &Term,
    candidate: &Term,
    substitution: &mut BTreeMap<PatternId, Term>,
    active: &mut BTreeSet<(crate::kernel::ContentId, crate::kernel::ContentId)>,
) -> Option<()> {
    match (pattern, candidate) {
        (Term::Pattern(id), value) => match substitution.get(id) {
            Some(bound) if bound != value => None,
            Some(_) => Some(()),
            None => {
                substitution.insert(id.clone(), value.clone());
                Some(())
            }
        },
        (Term::Application(pattern), Term::Application(candidate)) => {
            let pair = (pattern.clone(), candidate.clone());
            if !active.insert(pair.clone()) {
                return None;
            }
            let pattern = model.content(pattern)?;
            let candidate = model.content(candidate)?;
            let next = unify_content(model, pattern, candidate, substitution, active)?;
            *substitution = next;
            active.remove(&pair);
            Some(())
        }
        (
            Term::Product {
                shape: pattern_shape,
                fields: pattern_fields,
            },
            Term::Product {
                shape: candidate_shape,
                fields: candidate_fields,
            },
        ) if pattern_shape == candidate_shape
            && pattern_fields.keys().eq(candidate_fields.keys()) =>
        {
            for (label, pattern) in pattern_fields {
                let candidate = &candidate_fields[label];
                if pattern.domain() != candidate.domain() {
                    return None;
                }
                unify_term(
                    model,
                    pattern.value(),
                    candidate.value(),
                    substitution,
                    active,
                )?;
            }
            Some(())
        }
        (
            Term::LabelledProduct {
                shape: pattern_shape,
                fields: pattern_fields,
            },
            Term::LabelledProduct {
                shape: candidate_shape,
                fields: candidate_fields,
            },
        ) if pattern_shape == candidate_shape
            && pattern_fields.keys().eq(candidate_fields.keys()) =>
        {
            for (field, pattern) in pattern_fields {
                unify_term(
                    model,
                    pattern,
                    &candidate_fields[field],
                    substitution,
                    active,
                )?;
            }
            Some(())
        }
        (
            Term::Sum {
                tag: pattern_tag,
                value: pattern_value,
            },
            Term::Sum {
                tag: candidate_tag,
                value: candidate_value,
            },
        ) if pattern_tag == candidate_tag => {
            unify_term(model, pattern_value, candidate_value, substitution, active)
        }
        (
            Term::Sequence {
                shape: pattern_shape,
                element: pattern_element,
                values: pattern_values,
            },
            Term::Sequence {
                shape: candidate_shape,
                element: candidate_element,
                values: candidate_values,
            },
        ) if pattern_shape == candidate_shape
            && pattern_element == candidate_element
            && pattern_values.len() == candidate_values.len() =>
        {
            for (pattern, candidate) in pattern_values.iter().zip(candidate_values) {
                unify_term(model, pattern, candidate, substitution, active)?;
            }
            Some(())
        }
        _ if pattern == candidate => Some(()),
        _ => None,
    }
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
