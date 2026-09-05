//! Declared projection views of runtime-owned relation rows; no mirrored state.
use super::*;

const ROW_SELECTOR: &[u8] = b"clause/js-relation-row-selector-v1";

fn fields(term: &Term) -> Result<BTreeMap<Vec<u8>, Term>, ExecutableErrorV1> {
    let mut result = BTreeMap::new();
    let mut node = term;
    while let Some(triple) = node.as_triple() {
        let [key, value, rest] = triple.slots();
        let key = key
            .as_atom()
            .filter(|atom| atom.kind() == b"clause/js-field-v1")
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        if result
            .insert(key.canonical_payload().to_vec(), value.clone())
            .is_some()
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        node = rest;
    }
    if node.as_atom().is_none_or(|atom| {
        atom.kind() != b"clause/js-object-end-v1" || !atom.canonical_payload().is_empty()
    }) {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(result)
}

impl ExecutablePhysicalPlanV1 {
    pub(super) fn project_source_rows(
        &mut self,
        scope: TermScope,
        package: &clause_package::CanonicalSourcePackageSliceV1,
    ) -> Result<(), ExecutableErrorV1> {
        if package.relational_projection.is_empty() {
            return Ok(());
        }
        let projection = self
            .program
            .projection
            .as_mut()
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        let roles = projection
            .bindings
            .iter()
            .map(|binding| binding.role)
            .collect::<Vec<_>>();
        let lowered = lower_canonical_executable_program_v1(
            scope,
            &package.state_cells,
            &package.executable_handlers,
            &roles,
        )?;
        let bindings = lowered
            .states
            .iter()
            .map(|binding| (&binding.state, binding))
            .collect::<BTreeMap<_, _>>();
        let mut subjects = fields(&projection.template)?;
        let mut views =
            BTreeMap::<Vec<u8>, Vec<&clause_package::CanonicalRelationalProjectionV1>>::new();
        for view in &package.relational_projection {
            views.entry(view.subject.clone()).or_default().push(view);
        }
        for (name, views) in views {
            let mut properties = subjects
                .get(&name)
                .map(fields)
                .transpose()?
                .unwrap_or_default();
            let mut facets = BTreeSet::new();
            if let Some(reference) = properties.remove(b"$referent".as_slice()) {
                if let Some(reference) = projected_referent_value_v1(&reference)? {
                    facets.insert(reference);
                }
            }
            if let Some(references) = properties.remove(b"$referents".as_slice()) {
                for reference in fields(&references)?.into_values() {
                    if let Some(reference) = projected_referent_value_v1(&reference)? {
                        facets.insert(reference);
                    }
                }
            }
            let mut rows = BTreeMap::<Vec<u8>, BTreeMap<Option<Vec<u8>>, Term>>::new();
            for view in views {
                let referent = ExecutableReferentV1::declared(
                    view.referent.domain.get(),
                    view.referent.identity.get(),
                );
                facets.insert(referent.clone());
                let binding = bindings
                    .get(&view.state)
                    .ok_or(ExecutableErrorV1::CanonicalLoweringUnknownState)?;
                let value = Term::triple([
                    projection_literal(scope, ROW_SELECTOR, &[])?,
                    executable_projection_role_term_v1(
                        scope,
                        binding.projection_role,
                        ExecutableValueKindV1::RelationTable,
                    )?,
                    projected_scalar_value_term(scope, &ExecutableValueV1::Referent(referent))?,
                ])
                .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
                let field = match &view.state.path {
                    CanonicalStatePathV1::Field { designation, .. } => Some(designation.clone()),
                    _ => None,
                };
                rows.entry(view.state.relation_designation.clone())
                    .or_default()
                    .insert(field, value);
            }
            for (relation, mut row_fields) in rows {
                let value = if let Some(value) = row_fields.remove(&None) {
                    if !row_fields.is_empty() {
                        return Err(ExecutableErrorV1::MalformedProgram);
                    }
                    value
                } else {
                    projection_object(
                        scope,
                        row_fields
                            .into_iter()
                            .map(|(key, value)| {
                                Ok((key.ok_or(ExecutableErrorV1::MalformedProgram)?, value))
                            })
                            .collect::<Result<_, ExecutableErrorV1>>()?,
                    )?
                };
                if properties.insert(relation, value).is_some() {
                    return Err(ExecutableErrorV1::MalformedProgram);
                }
            }
            if facets.len() == 1 {
                properties.insert(
                    b"$referent".to_vec(),
                    projected_scalar_value_term(
                        scope,
                        &ExecutableValueV1::Referent(facets.into_iter().next().unwrap()),
                    )?,
                );
            } else if !facets.is_empty() {
                properties.insert(
                    b"$referents".to_vec(),
                    projection_object(
                        scope,
                        facets
                            .into_iter()
                            .map(|reference| {
                                Ok((
                                    reference.domain.to_string().into_bytes(),
                                    projected_scalar_value_term(
                                        scope,
                                        &ExecutableValueV1::Referent(reference),
                                    )?,
                                ))
                            })
                            .collect::<Result<_, ExecutableErrorV1>>()?,
                    )?,
                );
            }
            subjects.insert(
                name,
                projection_object(scope, properties.into_iter().collect())?,
            );
        }
        projection.template = projection_object(scope, subjects.into_iter().collect())?;
        Ok(())
    }
}

pub(super) fn row_selection<'a>(term: &'a Term) -> Option<(&'a Term, &'a Term)> {
    let [header, table, subject] = term.as_triple()?.slots();
    (header.as_atom()?.kind() == ROW_SELECTOR).then_some((table, subject))
}

pub(super) fn validate_selector(term: &Term) -> Result<(), ExecutableErrorV1> {
    let Some((table, subject)) = row_selection(term) else {
        return Ok(());
    };
    let header = term.as_triple().unwrap().slots()[0].as_atom().unwrap();
    if !header.canonical_payload().is_empty()
        || header.equality_contract() != EqualityContract::ExactOctetsV1
        || projection_role(table.as_atom().ok_or(ExecutableErrorV1::MalformedProgram)?)?
            .is_none_or(|(_, kind)| kind != ExecutableValueKindV1::RelationTable)
        || projected_referent_value_v1(subject)?.is_none()
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(())
}

pub(super) fn selected_value(
    term: &Term,
    bindings: &BTreeMap<LocalRoleRefV2, ExecutableProjectionBindingV1>,
    configuration: &[ExecutableSlotV1],
) -> Result<Option<ExecutableValueV1>, ExecutableErrorV1> {
    let (table, subject) = row_selection(term).ok_or(ExecutableErrorV1::MalformedProgram)?;
    let (role, kind) =
        projection_role(table.as_atom().ok_or(ExecutableErrorV1::MalformedProgram)?)?
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
    if kind != ExecutableValueKindV1::RelationTable {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let binding = bindings
        .get(&role)
        .ok_or(ExecutableErrorV1::MalformedProgram)?;
    let Some(ExecutableValueV1::RelationTable(table)) =
        configuration[usize::from(binding.slot)].value()
    else {
        return Err(ExecutableErrorV1::TypeMismatch);
    };
    let subject =
        projected_referent_value_v1(subject)?.ok_or(ExecutableErrorV1::MalformedProgram)?;
    let subject = ExecutableValueV1::Referent(subject);
    if !table.present(&subject)? {
        return Ok(None);
    }
    Ok(Some(table.read(&subject)?))
}
