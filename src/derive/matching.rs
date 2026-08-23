use crate::kernel::{Clause, Term, VariableId};
use std::collections::BTreeMap;

pub(super) fn unify(
    pattern: &Clause,
    assertion: &Clause,
    substitution: &BTreeMap<VariableId, Term>,
) -> Option<BTreeMap<VariableId, Term>> {
    if pattern.relation() != assertion.relation()
        || pattern.roles().len() != assertion.roles().len()
    {
        return None;
    }
    let mut substitution = substitution.clone();
    for (role, pattern_term) in pattern.roles() {
        let assertion_term = assertion.roles().get(role)?;
        match pattern_term {
            Term::Variable { id, typ } if typ == assertion_term.typ() => match substitution.get(id)
            {
                Some(bound) if bound != assertion_term => return None,
                Some(_) => {}
                None => {
                    substitution.insert(id.clone(), assertion_term.clone());
                }
            },
            Term::Variable { .. } => return None,
            _ if pattern_term != assertion_term => return None,
            _ => {}
        }
    }
    Some(substitution)
}

pub(super) fn instantiate(pattern: &Clause, substitution: &BTreeMap<VariableId, Term>) -> Clause {
    Clause::new(
        pattern.relation().clone(),
        pattern
            .roles()
            .iter()
            .map(|(role, term)| {
                let value = match term {
                    Term::Variable { id, .. } => substitution
                        .get(id)
                        .expect("admitted law conclusions are range-restricted")
                        .clone(),
                    _ => term.clone(),
                };
                (role.clone(), value)
            })
            .collect(),
    )
    .expect("instantiating an admitted conclusion preserves its complete role map")
}
