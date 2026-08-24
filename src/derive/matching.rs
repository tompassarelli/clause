use crate::kernel::{PatternId, RelationalContent, Term};
use std::collections::BTreeMap;

pub(super) fn unify(
    pattern: &RelationalContent,
    assertion: &RelationalContent,
    substitution: &BTreeMap<PatternId, Term>,
) -> Option<BTreeMap<PatternId, Term>> {
    if pattern.relation() != assertion.relation()
        || pattern.roles().len() != assertion.roles().len()
    {
        return None;
    }
    let mut substitution = substitution.clone();
    for (role, pattern_term) in pattern.roles() {
        let assertion_term = assertion.roles().get(role)?;
        match pattern_term {
            Term::Pattern(id) => match substitution.get(id) {
                Some(bound) if bound != assertion_term => return None,
                Some(_) => {}
                None => {
                    substitution.insert(id.clone(), assertion_term.clone());
                }
            },
            _ if pattern_term != assertion_term => return None,
            _ => {}
        }
    }
    Some(substitution)
}

pub(super) fn instantiate(
    pattern: &RelationalContent,
    substitution: &BTreeMap<PatternId, Term>,
) -> RelationalContent {
    RelationalContent::new(
        pattern.relation().clone(),
        pattern
            .roles()
            .iter()
            .map(|(role, term)| {
                let value = match term {
                    Term::Pattern(id) => substitution
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
