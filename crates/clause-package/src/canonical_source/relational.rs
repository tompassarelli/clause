//! Checked finite row matching. Source variables are rule-local bindings;
//! declared and runtime-created referents traverse the same relation rows.
use super::*;

pub(super) fn projection_views(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    states: &[CanonicalStateCellV1],
) -> Result<Vec<CanonicalRelationalProjectionV1>, CanonicalSourceErrorV1> {
    let mut views = Vec::new();
    for state in states {
        let Some(CanonicalScalarValueV1::RelationTable(table)) = &state.initial_value else {
            continue;
        };
        let mut seen = BTreeSet::new();
        for item in &cst.items {
            let CstKind::Application(application) = &item.kind else {
                continue;
            };
            let CanonicalScalarValueV1::Symbol(domain) = &application.object else {
                continue;
            };
            if application.role != b"shape"
                || referent_type_id(cst, plan, domain, item.origin)? != table.subject_domain
            {
                continue;
            }
            if !seen.insert(application.subject.clone()) {
                continue;
            }
            views.push(CanonicalRelationalProjectionV1 {
                subject: application.subject.clone(),
                referent: declared_referent_value(
                    cst,
                    plan,
                    &application.subject,
                    domain,
                    item.origin,
                )?,
                state: state.state.clone(),
            });
        }
    }
    Ok(views)
}

pub(super) fn field_domain<'a>(
    cst: &'a CanonicalSourceCstV1,
    domain: &'a [u8],
    field: Option<&[u8]>,
    origin: CanonicalSourceOriginV1,
) -> Result<&'a [u8], CanonicalSourceErrorV1> {
    let Some(field) = field else {
        return Ok(domain);
    };
    cst.items
        .iter()
        .find_map(|item| match &item.kind {
            CstKind::Shape {
                designation,
                fields,
            } if designation == domain => fields
                .iter()
                .find(|candidate| candidate.name == field)
                .map(|field| field.domain.as_slice()),
            _ => None,
        })
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })
}

fn facet(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    value: CanonicalExecutableExpressionV1,
    domain: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    if matches!(domain, b"F64" | b"Bool" | b"Text") {
        return Ok(value);
    }
    let domain_id = referent_type_id(cst, plan, domain, origin)?;
    if matches!(&value, CanonicalExecutableExpressionV1::FreshReferent { domain, .. } if *domain != domain_id)
    {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    }
    let mut members = BTreeSet::new();
    for item in &cst.items {
        if let CstKind::Application(application) = &item.kind {
            if application.role == b"shape"
                && matches!(&application.object, CanonicalScalarValueV1::Symbol(shape) if shape == domain)
            {
                members.insert(
                    declared_referent_value(cst, plan, &application.subject, domain, origin)?
                        .identity,
                );
            }
        }
    }
    Ok(CanonicalExecutableExpressionV1::ReferentFacet {
        value: Box::new(value),
        domain: domain_id,
        members: members.into_iter().collect(),
    })
}

fn checked_conditions(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &GeneralHandlerCst,
) -> Result<(Vec<CanonicalExecutablePredicateV1>, BTreeMap<Vec<u8>, CanonicalExecutableExpressionV1>), CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalExecutablePredicateV1 as P;
    let error = || CanonicalSourceErrorV1::MissingExecutableBinding {
        origin: source.origin,
    };
    check_domains(cst, plan, source)?;
    let mut variables = source
        .arguments
        .iter()
        .map(|argument| (argument.designation.clone(), E::Argument(argument.ordinal)))
        .collect::<BTreeMap<_, _>>();
    for creation in &source.creations {
        variables.insert(
            creation.parameter.clone(),
            E::FreshReferent {
                domain: referent_type_id(cst, plan, &creation.domain, source.origin)?,
                binder: creation.binder,
            },
        );
    }
    let mut next_binding = 0_u16;
    let mut operand = |name: &[u8], domain: &[u8]| -> Result<E, CanonicalSourceErrorV1> {
        if !name.starts_with(b"?") && !name.starts_with(b"\0") {
            return Ok(constant_expression(CanonicalScalarValueV1::Referent(
                declared_referent_value(cst, plan, name, domain, source.origin)?,
            )));
        }
        if let Some(value) = variables.get(name) {
            return facet(cst, plan, value.clone(), domain, source.origin);
        }
        let binding = next_binding;
        next_binding = next_binding.checked_add(1).ok_or_else(error)?;
        let value = E::Binding(binding);
        variables.insert(name.to_vec(), value.clone());
        facet(cst, plan, value, domain, source.origin)
    };
    let mut predicates = Vec::new();
    for parameter in source
        .parameter_sources
        .iter()
        .chain(&source.membership_sources)
    {
        let relation = resolved_state_relation(cst, plan, &parameter.relation, source.origin)?;
        validate_source_shape(&relation, parameter, source.origin)?;
        let subject = operand(&parameter.subject, relation.subject_domain)?;
        let value = operand(
            &parameter.parameter,
            field_domain(
                cst,
                relation.value_domain,
                parameter.field.as_deref(),
                source.origin,
            )?,
        )?;
        predicates.push(P::RelationMatch(
            relation_table_field_state_ref(
                cst,
                plan,
                &parameter.relation,
                parameter.field.as_deref(),
                source.origin,
            )?,
            subject,
            value,
        ));
    }
    for selector in &source.selectors {
        validate_scalar_state_selector(cst, plan, selector)?;
        let relation =
            resolved_state_relation(cst, plan, &selector.source.relation, selector.origin)?;
        let subject = operand(&selector.source.subject, relation.subject_domain)?;
        let expected = match &selector.expected {
            CanonicalScalarValueV1::Symbol(name)
                if !matches!(relation.value_domain, b"F64" | b"Bool" | b"Text") =>
            {
                CanonicalScalarValueV1::Referent(declared_referent_value(
                    cst,
                    plan,
                    name,
                    relation.value_domain,
                    selector.origin,
                )?)
            }
            value => value.clone(),
        };
        predicates.push(P::RelationMatch(
            relation_table_state_ref(cst, plan, &selector.source.relation, selector.origin)?,
            subject,
            constant_expression(expected),
        ));
    }
    for condition in &source.boolean_conditions {
        let resolved = resolve_boolean_relation_use(cst, condition)?;
        let relation = resolved_state_relation_for(plan, resolved.relation, condition.origin)?;
        // A Boolean premise is an exact typed row match, not a skipped guard.
        // Multi-input derived relations must use their own general lowering.
        if relation.value_domain != b"Bool" || resolved.relation.roles.len() != 2 {
            return Err(error());
        }
        let name = resolved
            .bindings
            .get(relation.subject_designation)
            .ok_or_else(error)?;
        let subject = operand(name, relation.subject_domain)?;
        predicates.push(P::RelationMatch(
            relation_table_state_ref(cst, plan, &resolved.relation.surface, condition.origin)?,
            subject,
            constant_expression(CanonicalScalarValueV1::Boolean(resolved.value)),
        ));
    }
    drop(operand);
    for sum in &source.sums {
        // A closed query shares the ordinary row checker, but neither captures
        // the enclosing rule's bindings nor produces state effects.
        let query = GeneralHandlerCst {
            origin: sum.origin,
            producer: source.producer.clone(),
            designation: source.designation.clone(),
            subject: Vec::new(),
            arguments: vec![], creations: vec![],
            parameter_sources: sum.parameter_sources.clone(),
            membership_sources: vec![], required_sources: vec![],
            selectors: sum.selectors.clone(), scalar_bindings: vec![], sums: vec![],
            predicates: sum.predicates.clone(), boolean_conditions: vec![],
            assignments: vec![], accumulations: vec![], insertions: vec![],
            removals: vec![], includes: vec![],
        };
        let mut domains = check_domains(cst, plan, &query)?;
        check_expression(&sum.value, b"F64", &mut domains, sum.origin)?;
        let (predicates, bindings) = checked_conditions(cst, plan, &query)?;
        let value = relational_scalar_expression(cst, plan, &sum.value, &bindings,
            Some(b"F64"), sum.origin)?;
        variables.insert(sum.parameter.clone(), E::Sum { predicates, value: Box::new(value) });
    }
    for required in &source.required_sources {
        let subject = subject_expression(cst, plan, &variables, required, source.origin)?;
        let relation = resolved_state_relation(cst, plan, &required.relation, source.origin)?;
        let fields = if required.field.is_some() {
            vec![required.field.as_deref()]
        } else {
            cst.items
                .iter()
                .find_map(|item| match &item.kind {
                    CstKind::Shape {
                        designation,
                        fields,
                    } if designation == relation.value_domain => Some(
                        fields
                            .iter()
                            .map(|field| Some(field.name.as_slice()))
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                })
                .unwrap_or_else(|| vec![None])
        };
        for field in fields {
            predicates.push(P::Equal(
                E::RelationPresent(
                    Box::new(E::State(relation_table_field_state_ref(
                        cst,
                        plan,
                        &required.relation,
                        field,
                        source.origin,
                    )?)),
                    Box::new(subject.clone()),
                ),
                constant_expression(CanonicalScalarValueV1::Boolean(true)),
            ));
        }
    }
    for predicate in &source.predicates {
        let (left, right, constructor) = match predicate {
            CanonicalScalarPredicateV1::Equal(a, b) => (a, b, P::Equal as fn(_, _) -> _),
            CanonicalScalarPredicateV1::GreaterThan(a, b) => {
                (a, b, P::GreaterThan as fn(_, _) -> _)
            }
            CanonicalScalarPredicateV1::LessThanOrEqual(a, b) => {
                (a, b, P::LessThanOrEqual as fn(_, _) -> _)
            }
        };
        let expected = [left, right].into_iter().find_map(|expression| {
            let CanonicalScalarExpressionV1::Parameter(name) = expression else {
                return None;
            };
            source
                .parameter_sources
                .iter()
                .chain(&source.membership_sources)
                .find(|parameter| &parameter.parameter == name)
                .and_then(|parameter| {
                    resolved_state_relation(cst, plan, &parameter.relation, source.origin).ok()
                })
                .map(|relation| relation.value_domain)
        });
        predicates.push(constructor(
            relational_scalar_expression(cst, plan, left, &variables, expected, source.origin)?,
            relational_scalar_expression(cst, plan, right, &variables, expected, source.origin)?,
        ));
    }
    Ok((predicates, variables))
}

pub(super) fn checked_handler(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &GeneralHandlerCst,
) -> Result<CanonicalExecutableHandlerV1, CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalExecutablePredicateV1 as P;
    use CanonicalRelationEffectV1 as R;
    let error = || CanonicalSourceErrorV1::MissingExecutableBinding { origin: source.origin };
    let (mut predicates, variables) = checked_conditions(cst, plan, source)?;
    let mut effects = BTreeMap::<CanonicalStateRefV1, Vec<R>>::new();
    for (assignments, mode) in [
        (&source.assignments, 0),
        (&source.insertions, 1),
        (&source.accumulations, 2),
    ] {
        for assignment in assignments {
            let target = &assignment.target;
            let relation = resolved_state_relation(cst, plan, &target.relation, source.origin)?;
            validate_source_shape(&relation, target, source.origin)?;
            let domain = field_domain(
                cst,
                relation.value_domain,
                target.field.as_deref(),
                source.origin,
            )?;
            let cardinality = state_relation_cardinality(cst, plan, target, source.origin)?;
            if mode == 2 && (domain != b"F64" || cardinality == SourceCardinality::Many) {
                return Err(error());
            }
            let subject = subject_expression(cst, plan, &variables, target, source.origin)?;
            let value = relational_scalar_expression(
                cst,
                plan,
                &assignment.value,
                &variables,
                Some(domain),
                source.origin,
            )?;
            let value = facet(cst, plan, value, domain, source.origin)?;
            let state = relation_table_field_state_ref(
                cst,
                plan,
                &target.relation,
                target.field.as_deref(),
                source.origin,
            )?;
            if mode == 1 && cardinality != SourceCardinality::Many {
                predicates.push(P::Equal(
                    E::RelationPresent(
                        Box::new(E::State(state.clone())),
                        Box::new(subject.clone()),
                    ),
                    constant_expression(CanonicalScalarValueV1::Boolean(false)),
                ));
            }
            let effect = match mode {
                2 => R::Accumulate(subject, value),
                1 if cardinality == SourceCardinality::Many => R::Insert(subject, value),
                _ => R::Put(subject, value),
            };
            effects.entry(state).or_default().push(effect);
        }
    }
    for removal in &source.removals {
        let subject = subject_expression(cst, plan, &variables, removal, source.origin)?;
        let value = variables
            .get(&removal.parameter)
            .cloned()
            .ok_or_else(error)?;
        let relation = resolved_state_relation(cst, plan, &removal.relation, source.origin)?;
        let value = facet(
            cst,
            plan,
            value,
            field_domain(
                cst,
                relation.value_domain,
                removal.field.as_deref(),
                source.origin,
            )?,
            source.origin,
        )?;
        effects
            .entry(relation_table_field_state_ref(
                cst,
                plan,
                &removal.relation,
                removal.field.as_deref(),
                source.origin,
            )?)
            .or_default()
            .push(R::Remove(subject, value));
    }
    Ok(CanonicalExecutableHandlerV1 {
        id: formation_id(plan, &source.producer, &head_slot(CanonicalSourceProductionV1::Handler))?,
        designation: source.designation.clone(),
        trigger: if source.designation == b"tick" { CanonicalHandlerTriggerV1::FixedTick }
            else if !source.arguments.is_empty() || cst.items.iter().any(|item| matches!(&item.kind,
                CstKind::KeyboardBinding(binding) if binding.handler_designation == source.designation))
                || (source.predicates.is_empty() && source.boolean_conditions.is_empty()) { CanonicalHandlerTriggerV1::External }
            else { CanonicalHandlerTriggerV1::FixedTick },
        argument_count: u16::try_from(source.arguments.len()).map_err(|_| error())?,
        rules: vec![CanonicalExecutableRuleV1 { law_origins: vec![], predicates, required_present: vec![], required_absent: vec![],
            assignments: effects.into_iter().map(|(target, effects)| CanonicalExecutableAssignmentV1 { target, value: E::RelationEffects(effects) }).collect(), removals: vec![] }],
    })
}

fn constrain(
    domains: &mut BTreeMap<Vec<u8>, Vec<u8>>,
    name: &[u8],
    domain: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<(), CanonicalSourceErrorV1> {
    let scalar = |d: &[u8]| matches!(d, b"F64" | b"Bool" | b"Text");
    if let Some(prior) = domains.get(name) {
        if prior != domain && (scalar(prior) || scalar(domain)) {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
    } else {
        domains.insert(name.to_vec(), domain.to_vec());
    }
    Ok(())
}

fn expression_domain<'a>(
    expression: &'a CanonicalScalarExpressionV1,
    domains: &'a BTreeMap<Vec<u8>, Vec<u8>>,
) -> Option<&'a [u8]> {
    use CanonicalScalarExpressionV1 as S;
    match expression {
        S::Number(_) | S::Add(..) | S::Subtract(..) | S::Multiply(..) | S::Divide(..) => {
            Some(b"F64")
        }
        S::Boolean(_) => Some(b"Bool"),
        S::Text(_) | S::Concatenate(..) => Some(b"Text"),
        S::Parameter(name) => domains.get(name).map(Vec::as_slice),
        _ => None,
    }
}

fn check_expression(
    expression: &CanonicalScalarExpressionV1,
    expected: &[u8],
    domains: &mut BTreeMap<Vec<u8>, Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<(), CanonicalSourceErrorV1> {
    use CanonicalScalarExpressionV1 as S;
    if let Some(actual) = expression_domain(expression, domains) {
        let scalar = |d: &[u8]| matches!(d, b"F64" | b"Bool" | b"Text");
        if actual != expected && (scalar(actual) || scalar(expected)) {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
    }
    match expression {
        S::Parameter(name) => constrain(domains, name, expected, origin)?,
        S::Add(a, b) | S::Subtract(a, b) | S::Multiply(a, b) | S::Divide(a, b) => {
            check_expression(a, b"F64", domains, origin)?;
            check_expression(b, b"F64", domains, origin)?;
        }
        S::Concatenate(a, b) => {
            check_expression(a, b"Text", domains, origin)?;
            check_expression(b, b"Text", domains, origin)?;
        }
        S::Current => return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin }),
        S::Symbol(_) if matches!(expected, b"F64" | b"Bool" | b"Text") => {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
        _ => {}
    }
    Ok(())
}

fn check_domains(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &GeneralHandlerCst,
) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, CanonicalSourceErrorV1> {
    let mut domains = BTreeMap::new();
    for sum in &source.sums {
        constrain(&mut domains, &sum.parameter, b"F64", sum.origin)?;
    }
    for selector in &source.selectors {
        let relation = resolved_state_relation(cst, plan, &selector.source.relation, selector.origin)?;
        constrain(&mut domains, &selector.source.subject, relation.subject_domain, selector.origin)?;
    }
    for creation in &source.creations {
        constrain(
            &mut domains,
            &creation.parameter,
            &creation.domain,
            source.origin,
        )?;
    }
    for parameter in source
        .parameter_sources
        .iter()
        .chain(&source.membership_sources)
    {
        let relation = resolved_state_relation(cst, plan, &parameter.relation, source.origin)?;
        constrain(
            &mut domains,
            &parameter.subject,
            relation.subject_domain,
            source.origin,
        )?;
        constrain(
            &mut domains,
            &parameter.parameter,
            field_domain(
                cst,
                relation.value_domain,
                parameter.field.as_deref(),
                source.origin,
            )?,
            source.origin,
        )?;
    }
    for condition in &source.boolean_conditions {
        let resolved = resolve_boolean_relation_use(cst, condition)?;
        let relation = resolved_state_relation_for(plan, resolved.relation, condition.origin)?;
        let subject = resolved.bindings.get(relation.subject_designation).ok_or(
            CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: condition.origin,
            },
        )?;
        constrain(
            &mut domains,
            subject,
            relation.subject_domain,
            condition.origin,
        )?;
    }
    for assignment in source
        .assignments
        .iter()
        .chain(&source.insertions)
        .chain(&source.accumulations)
    {
        let relation =
            resolved_state_relation(cst, plan, &assignment.target.relation, source.origin)?;
        constrain(
            &mut domains,
            &assignment.target.subject,
            relation.subject_domain,
            source.origin,
        )?;
        check_expression(
            &assignment.value,
            field_domain(
                cst,
                relation.value_domain,
                assignment.target.field.as_deref(),
                source.origin,
            )?,
            &mut domains,
            source.origin,
        )?;
    }
    for predicate in &source.predicates {
        let (a, b, numeric) = match predicate {
            CanonicalScalarPredicateV1::Equal(a, b) => (a, b, false),
            CanonicalScalarPredicateV1::GreaterThan(a, b)
            | CanonicalScalarPredicateV1::LessThanOrEqual(a, b) => (a, b, true),
        };
        let domain = if numeric {
            b"F64".to_vec()
        } else {
            expression_domain(a, &domains)
                .or_else(|| expression_domain(b, &domains))
                .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding {
                    origin: source.origin,
                })?
                .to_vec()
        };
        check_expression(a, &domain, &mut domains, source.origin)?;
        check_expression(b, &domain, &mut domains, source.origin)?;
    }
    Ok(domains)
}

fn subject_expression(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    variables: &BTreeMap<Vec<u8>, CanonicalExecutableExpressionV1>,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    if source.subject.starts_with(b"?") {
        return facet(
            cst,
            plan,
            relational_subject_expression(variables, &source.subject, origin)?,
            relation.subject_domain,
            origin,
        );
    }
    Ok(constant_expression(CanonicalScalarValueV1::Referent(
        declared_referent_value(cst, plan, &source.subject, relation.subject_domain, origin)?,
    )))
}
