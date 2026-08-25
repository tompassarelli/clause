use std::collections::{BTreeMap, BTreeSet};

use super::{ContentId, KernelError, PatternId, RelationalContent, Result, Term};

pub(crate) struct InstantiatedContent {
    pub(crate) root: RelationalContent,
    pub(crate) dependencies: BTreeMap<ContentId, RelationalContent>,
}

pub(crate) fn unify<'p, 'a>(
    pattern: &RelationalContent,
    actual: &RelationalContent,
    substitution: &BTreeMap<PatternId, Term>,
    allow_new_bindings: bool,
    mut pattern_content: impl FnMut(&ContentId) -> Option<&'p RelationalContent>,
    mut actual_content: impl FnMut(&ContentId) -> Option<&'a RelationalContent>,
) -> Option<BTreeMap<PatternId, Term>> {
    let mut substitution = substitution.clone();
    unify_content(
        pattern,
        actual,
        &mut substitution,
        allow_new_bindings,
        &mut pattern_content,
        &mut actual_content,
        &mut BTreeSet::new(),
    )?;
    Some(substitution)
}

#[allow(clippy::too_many_arguments)]
fn unify_content<'p, 'a, P, A>(
    pattern: &RelationalContent,
    actual: &RelationalContent,
    substitution: &mut BTreeMap<PatternId, Term>,
    allow_new_bindings: bool,
    pattern_content: &mut P,
    actual_content: &mut A,
    active: &mut BTreeSet<(ContentId, ContentId)>,
) -> Option<()>
where
    P: FnMut(&ContentId) -> Option<&'p RelationalContent>,
    A: FnMut(&ContentId) -> Option<&'a RelationalContent>,
{
    if pattern.relation() != actual.relation() || pattern.roles().keys().ne(actual.roles().keys()) {
        return None;
    }
    for (role, pattern) in pattern.roles() {
        unify_term(
            pattern,
            &actual.roles()[role],
            substitution,
            allow_new_bindings,
            pattern_content,
            actual_content,
            active,
        )?;
    }
    Some(())
}

#[allow(clippy::too_many_arguments)]
fn unify_term<'p, 'a, P, A>(
    pattern: &Term,
    actual: &Term,
    substitution: &mut BTreeMap<PatternId, Term>,
    allow_new_bindings: bool,
    pattern_content: &mut P,
    actual_content: &mut A,
    active: &mut BTreeSet<(ContentId, ContentId)>,
) -> Option<()>
where
    P: FnMut(&ContentId) -> Option<&'p RelationalContent>,
    A: FnMut(&ContentId) -> Option<&'a RelationalContent>,
{
    match (pattern, actual) {
        (Term::Pattern(id), value) => match substitution.get(id) {
            Some(bound) if bound != value => None,
            Some(_) => Some(()),
            None if allow_new_bindings => {
                substitution.insert(id.clone(), value.clone());
                Some(())
            }
            None => None,
        },
        (Term::Application(pattern_id), Term::Application(actual_id)) => {
            let pair = (pattern_id.clone(), actual_id.clone());
            if !active.insert(pair.clone()) {
                return None;
            }
            let result = (|| {
                let pattern = pattern_content(pattern_id)?;
                let actual = actual_content(actual_id)?;
                unify_content(
                    pattern,
                    actual,
                    substitution,
                    allow_new_bindings,
                    pattern_content,
                    actual_content,
                    active,
                )
            })();
            active.remove(&pair);
            result
        }
        (
            Term::Product {
                shape: pattern_shape,
                fields: pattern_fields,
            },
            Term::Product {
                shape: actual_shape,
                fields: actual_fields,
            },
        ) if pattern_shape == actual_shape && pattern_fields.keys().eq(actual_fields.keys()) => {
            for (label, pattern) in pattern_fields {
                let actual = &actual_fields[label];
                if pattern.domain() != actual.domain() {
                    return None;
                }
                unify_term(
                    pattern.value(),
                    actual.value(),
                    substitution,
                    allow_new_bindings,
                    pattern_content,
                    actual_content,
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
                shape: actual_shape,
                fields: actual_fields,
            },
        ) if pattern_shape == actual_shape && pattern_fields.keys().eq(actual_fields.keys()) => {
            for (field, pattern) in pattern_fields {
                unify_term(
                    pattern,
                    &actual_fields[field],
                    substitution,
                    allow_new_bindings,
                    pattern_content,
                    actual_content,
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
                tag: actual_tag,
                value: actual_value,
            },
        ) if pattern_tag == actual_tag => unify_term(
            pattern_value,
            actual_value,
            substitution,
            allow_new_bindings,
            pattern_content,
            actual_content,
            active,
        ),
        (
            Term::Sequence {
                shape: pattern_shape,
                element: pattern_element,
                values: pattern_values,
            },
            Term::Sequence {
                shape: actual_shape,
                element: actual_element,
                values: actual_values,
            },
        ) if pattern_shape == actual_shape
            && pattern_element == actual_element
            && pattern_values.len() == actual_values.len() =>
        {
            for (pattern, actual) in pattern_values.iter().zip(actual_values) {
                unify_term(
                    pattern,
                    actual,
                    substitution,
                    allow_new_bindings,
                    pattern_content,
                    actual_content,
                    active,
                )?;
            }
            Some(())
        }
        _ if pattern == actual => Some(()),
        _ => None,
    }
}

pub(crate) fn instantiate<'a>(
    pattern: &RelationalContent,
    substitution: &BTreeMap<PatternId, Term>,
    mut content: impl FnMut(&ContentId) -> Option<&'a RelationalContent>,
) -> Result<InstantiatedContent> {
    let mut dependencies = BTreeMap::new();
    let root = instantiate_content(
        pattern,
        substitution,
        &mut content,
        &mut BTreeSet::new(),
        &mut dependencies,
    )?;
    Ok(InstantiatedContent { root, dependencies })
}

fn instantiate_content<'a, C>(
    pattern: &RelationalContent,
    substitution: &BTreeMap<PatternId, Term>,
    content: &mut C,
    active: &mut BTreeSet<ContentId>,
    dependencies: &mut BTreeMap<ContentId, RelationalContent>,
) -> Result<RelationalContent>
where
    C: FnMut(&ContentId) -> Option<&'a RelationalContent>,
{
    RelationalContent::new(
        pattern.relation().clone(),
        pattern
            .roles()
            .iter()
            .map(|(role, term)| {
                Ok((
                    role.clone(),
                    instantiate_term(term, substitution, content, active, dependencies)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    )
}

fn instantiate_term<'a, C>(
    pattern: &Term,
    substitution: &BTreeMap<PatternId, Term>,
    content: &mut C,
    active: &mut BTreeSet<ContentId>,
    dependencies: &mut BTreeMap<ContentId, RelationalContent>,
) -> Result<Term>
where
    C: FnMut(&ContentId) -> Option<&'a RelationalContent>,
{
    Ok(match pattern {
        Term::Pattern(id) => substitution
            .get(id)
            .ok_or_else(|| KernelError::new("instantiated pattern has no binding"))?
            .clone(),
        Term::Application(id) => {
            if !active.insert(id.clone()) {
                return Err(KernelError::new(
                    "recursive term application graph contains a cycle",
                ));
            }
            let instantiated = (|| {
                let pattern = content(id)
                    .ok_or_else(|| KernelError::new("recursive term names undeclared content"))?;
                instantiate_content(pattern, substitution, content, active, dependencies)
            })();
            active.remove(id);
            let instantiated = instantiated?;
            if let Some(existing) =
                dependencies.insert(instantiated.id().clone(), instantiated.clone())
                && existing != instantiated
            {
                return Err(KernelError::new(
                    "instantiated recursive term has conflicting content identity",
                ));
            }
            Term::application(instantiated.id().clone())
        }
        Term::Product { shape, fields } => Term::product(
            shape.clone(),
            fields
                .iter()
                .map(|(label, field)| {
                    Ok((
                        label.clone(),
                        super::ProductField::new(
                            field.domain().clone(),
                            instantiate_term(
                                field.value(),
                                substitution,
                                content,
                                active,
                                dependencies,
                            )?,
                        ),
                    ))
                })
                .collect::<Result<_>>()?,
        )?,
        Term::LabelledProduct { shape, fields } => Term::labelled_product(
            shape.clone(),
            fields
                .iter()
                .map(|(field, value)| {
                    Ok((
                        field.clone(),
                        instantiate_term(value, substitution, content, active, dependencies)?,
                    ))
                })
                .collect::<Result<_>>()?,
        )?,
        Term::Sum { tag, value } => Term::sum(
            tag.clone(),
            instantiate_term(value, substitution, content, active, dependencies)?,
        )?,
        Term::Sequence {
            shape,
            element,
            values,
        } => Term::sequence(
            shape.clone(),
            element.clone(),
            values
                .iter()
                .map(|value| instantiate_term(value, substitution, content, active, dependencies))
                .collect::<Result<_>>()?,
        )?,
        Term::Referent(_) | Term::F32(_) | Term::Int(_) | Term::Bool(_) => pattern.clone(),
    })
}
