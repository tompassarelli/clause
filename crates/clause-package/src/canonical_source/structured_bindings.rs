//! Whole structured bindings use the same typed field states as explicit
//! record patterns. Expansion precedes both static and relational lowering.

use super::*;
use std::borrow::Cow;

struct Binding {
    shape: Vec<u8>,
    components: Vec<ScalarParameterSourceCst>,
}

fn components(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<Option<Binding>, CanonicalSourceErrorV1> {
    if source.field.is_some() {
        return Ok(None);
    }
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    let Some(fields) = cst.items.iter().find_map(|item| match &item.kind {
        CstKind::Shape {
            designation,
            fields,
        } if designation == relation.value_domain => Some(fields),
        _ => None,
    }) else {
        return Ok(None);
    };
    let shape = relation.value_domain.to_vec();
    Ok(Some(Binding {
        shape: shape.clone(),
        components: fields
            .iter()
            .map(|field| {
                let mut component = source.clone();
                // NUL cannot occur in a source variable, so field binders cannot
                // capture an authored name or another aggregate's components.
                component.parameter = [
                    b"\0structured-".as_slice(),
                    &source.parameter,
                    b"\0",
                    &field.name,
                ]
                .concat();
                component.shape = Some(shape.clone());
                component.field = Some(field.name.clone());
                component
            })
            .collect(),
    }))
}

fn assignments(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &[GeneralAssignmentCst],
    bindings: &BTreeMap<Vec<u8>, Binding>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<GeneralAssignmentCst>, CanonicalSourceErrorV1> {
    let mut result = Vec::new();
    for assignment in source {
        let Some(target) = components(cst, plan, &assignment.target, origin)? else {
            result.push(assignment.clone());
            continue;
        };
        let error = || CanonicalSourceErrorV1::MissingExecutableBinding { origin };
        let CanonicalScalarExpressionV1::Parameter(parameter) = &assignment.value else {
            return Err(error());
        };
        let binding = bindings.get(parameter).ok_or_else(error)?;
        if binding.shape != target.shape {
            return Err(error());
        }
        for (target, value) in target.components.into_iter().zip(&binding.components) {
            result.push(GeneralAssignmentCst {
                target,
                value: CanonicalScalarExpressionV1::Parameter(value.parameter.clone()),
            });
        }
    }
    Ok(result)
}

pub(super) fn expand<'a>(
    cst: &'a CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Cow<'a, CanonicalSourceCstV1>, CanonicalSourceErrorV1> {
    let mut result = Cow::Borrowed(cst);
    for (index, item) in cst.items.iter().enumerate() {
        let CstKind::GeneralHandler(handler) = &item.kind else {
            continue;
        };
        let mut bindings = BTreeMap::new();
        for source in &handler.parameter_sources {
            if let Some(binding) = components(cst, plan, source, handler.origin)? {
                bindings.insert(source.parameter.clone(), binding);
            }
        }
        if bindings.is_empty() {
            continue;
        }
        let mut expanded = handler.clone();
        expanded.parameter_sources = handler
            .parameter_sources
            .iter()
            .flat_map(|source| {
                bindings
                    .get(&source.parameter)
                    .map(|binding| binding.components.clone())
                    .unwrap_or_else(|| vec![source.clone()])
            })
            .collect();
        expanded.assignments =
            assignments(cst, plan, &handler.assignments, &bindings, handler.origin)?;
        expanded.insertions =
            assignments(cst, plan, &handler.insertions, &bindings, handler.origin)?;
        for sources in [&mut expanded.required_sources, &mut expanded.removals] {
            let mut fields = Vec::new();
            for source in sources.iter() {
                match components(cst, plan, source, handler.origin)? {
                    Some(binding) => fields.extend(binding.components),
                    None => fields.push(source.clone()),
                }
            }
            *sources = fields;
        }
        expanded.predicates.clear();
        for predicate in &handler.predicates {
            if let CanonicalScalarPredicateV1::Equal(
                CanonicalScalarExpressionV1::Parameter(left),
                CanonicalScalarExpressionV1::Parameter(right),
            ) = predicate
                && let Some(left) = bindings.get(left)
            {
                let right = bindings
                    .get(right)
                    .filter(|right| right.shape == left.shape)
                    .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding {
                        origin: handler.origin,
                    })?;
                expanded
                    .predicates
                    .extend(left.components.iter().zip(&right.components).map(|(a, b)| {
                        CanonicalScalarPredicateV1::Equal(
                            CanonicalScalarExpressionV1::Parameter(a.parameter.clone()),
                            CanonicalScalarExpressionV1::Parameter(b.parameter.clone()),
                        )
                    }));
            } else {
                expanded.predicates.push(predicate.clone());
            }
        }
        result.to_mut().items[index].kind = CstKind::GeneralHandler(expanded);
    }
    Ok(result)
}
