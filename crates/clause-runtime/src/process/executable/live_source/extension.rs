//! Checked additive source transitions, including new rows in existing roles.
use super::*;

pub(super) fn check_prepared_source_extension(
    previous: &CheckedExecutableSourcePreparationV1,
    appended: &[u8],
    root: ProgramChangeOccurrenceId,
    cpp1: &[u8],
) -> Result<CheckedExecutableSourceEditV1, ExecutableErrorV1> {
    let reject = |_| ExecutableErrorV1::SourceContinuityRejected("invalid additive source operation");
    let edit = clause_package::append_canonical_source_items_v1(
        previous.analysis.source(), previous.analysis.plan(), appended, root).map_err(reject)?;
    let capsule = encode_executable_source_preparation_v1(edit.source().exact_source(), root,
        &previous.declared_frontend, edit.source().imports())?;
    // The complete new program is lowered and compared, including its initial
    // world and projection. Its carrier package is checked when instantiated;
    // physical projection Roles may grow without changing source identities.
    let next = check_executable_source_preparation_v1(&capsule, previous.scope, cpp1)?;
    if previous.plan.target != next.plan.target || previous.plan.refinement != next.plan.refinement
        || previous.plan.mode.operator.local != next.plan.mode.operator.local
        || previous.plan.mode.local != next.plan.mode.local {
        return Err(ExecutableErrorV1::SourceContinuityRejected("changed physical execution contract"));
    }
    check_extension_inputs(&next)?;
    let old_cells = &previous.analysis.package().state_cells;
    let new_cells = next.analysis.package().state_cells.iter().map(|cell| (&cell.state, cell)).collect::<BTreeMap<_, _>>();
    let new_slots = next.lowered.states.iter().map(|binding| (&binding.state, binding.slot)).collect::<BTreeMap<_, _>>();
    let old_slots = previous.lowered.states.iter().map(|binding| (&binding.state, binding.slot)).collect::<BTreeMap<_, _>>();
    let retained_states = old_cells.iter().map(|cell| edit.state(&cell.state).map_err(reject)).collect::<Result<BTreeSet<_>, _>>()?;
    for state in new_cells.keys().filter(|state| !retained_states.contains(**state)) {
        if retained_states.iter().any(|old| old.relation == state.relation
            && old.subject_role == state.subject_role && old.value_role == state.value_role
            && old.subject_identity == state.subject_identity && old.subject == state.subject
            && old.path == state.path) {
            return Err(ExecutableErrorV1::SourceContinuityRejected("additional initial assertion redefines an old state binding"));
        }
    }
    let mut slots = Vec::new();
    let mut added_rows = BTreeMap::new();
    for old in old_cells {
        let state = edit.state(&old.state).map_err(reject)?;
        let new = new_cells.get(&state).ok_or(ExecutableErrorV1::SourceContinuityRejected("removed old state"))?;
        if old.value_kind != new.value_kind { return Err(ExecutableErrorV1::TypeMismatch); }
        let slot = *new_slots.get(&state).ok_or(ExecutableErrorV1::MalformedProgram)?;
        slots.push((*old_slots.get(&old.state).ok_or(ExecutableErrorV1::MalformedProgram)?, slot));
        let old_value = old.initial_value.as_ref().map(lower_scalar_value).transpose()?
            .as_ref().map(|value| migrate_value(value, &edit)).transpose()?;
        let new_value = new.initial_value.as_ref().map(lower_scalar_value).transpose()?;
        if old_value == new_value { continue; }
        let (Some(ExecutableValueV1::RelationTable(old_table)), Some(ExecutableValueV1::RelationTable(mut new_table))) = (old_value, new_value) else {
            return Err(ExecutableErrorV1::SourceContinuityRejected("changed old initial state"));
        };
        if old_table.subject_domain != new_table.subject_domain || old_table.value_kind != new_table.value_kind
            || old_table.value_domain != new_table.value_domain || old_table.cardinality != new_table.cardinality
            || old_table.total != new_table.total {
            return Err(ExecutableErrorV1::SourceContinuityRejected("changed old relation contract"));
        }
        let rows = Arc::make_mut(&mut new_table.rows);
        for (subject, values) in old_table.rows.iter() {
            if rows.remove(subject).as_ref() != Some(values) {
                return Err(ExecutableErrorV1::SourceContinuityRejected("changed old initial row"));
            }
        }
        for subject in rows.keys() {
            let ExecutableReferentIdentityV1::Declared(identity) = subject.identity else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            if edit.retained().values().any(|retained| *retained == CanonicalAllocatedIdentityV1::Formation(FormationLocalId::new(identity))) {
                return Err(ExecutableErrorV1::SourceContinuityRejected("new initial row requires a new subject"));
            }
        }
        added_rows.insert(slot, new_table);
    }
    if slots.len() != previous.lowered.states.len()
        || slots.iter().map(|(_, new)| new).collect::<BTreeSet<_>>().len() != slots.len() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(CheckedExecutableSourceEditV1 {
        old_plan: previous.identity, new_plan: next.identity,
        continuity: ExecutableSourceContinuityV1 {
            old_snapshot: previous.analysis.package().checked_package.constitution().snapshot(),
            new_snapshot: next.analysis.package().checked_package.constitution().snapshot(),
            identities: edit.retained().clone(), slots, occurrences: vec![],
        }, edit, preparation: Arc::new(next), added_rows,
    })
}

fn check_extension_inputs(next: &CheckedExecutableSourcePreparationV1) -> Result<(), ExecutableErrorV1> {
    let package = next.analysis.package();
    let reject = || ExecutableErrorV1::SourceContinuityRejected("input plan does not realize appended source");
    let mut tick = next.lowered.handlers.iter().filter(|binding| matches!(binding.trigger,
        CanonicalHandlerTriggerV1::FixedTickRoot | CanonicalHandlerTriggerV1::FixedTickDerived | CanonicalHandlerTriggerV1::FixedTick)).collect::<Vec<_>>();
    tick.sort_by_key(|binding| (binding.trigger, binding.handler));
    let mut entries = BTreeSet::new();
    let mut tick = tick.into_iter().filter_map(|binding| entries.insert(binding.entry).then_some(binding.entry)).collect::<Vec<_>>();
    let count = package.keyboard_bindings.len() + package.scalar_input_bindings.len() + package.referent_input_bindings.len();
    if count == 0 && tick.is_empty() {
        if next.plan.input.is_some() { return Err(reject()); }
        return Ok(());
    }
    let input = next.plan.input.as_ref().ok_or_else(reject)?;
    if input.events.len() != count { return Err(reject()); }
    let entry = |designation: &[u8], arguments: usize| -> Result<u16, ExecutableErrorV1> {
        let handlers = package.executable_handlers.iter().filter(|handler| handler.designation == designation).collect::<Vec<_>>();
        if handlers.is_empty() || handlers.iter().any(|handler|
            handler.trigger != CanonicalHandlerTriggerV1::External || usize::from(handler.argument_count) != arguments) { return Err(reject()); }
        let entries = handlers.iter().map(|handler| next.lowered.handlers.iter().find(|binding| binding.handler == handler.id)
            .map(|binding| binding.entry).ok_or_else(reject)).collect::<Result<BTreeSet<_>, _>>()?;
        if entries.len() != 1 { return Err(reject()); }
        Ok(*entries.first().ok_or_else(reject)?)
    };
    let mut expected = Vec::new();
    for source in &package.keyboard_bindings {
        expected.push((ExecutableInputSourceV1::Keyboard { code: source.code.clone(), phase: match source.phase {
            CanonicalKeyPhaseV1::Down => ExecutableKeyPhaseV1::Down, CanonicalKeyPhaseV1::Up => ExecutableKeyPhaseV1::Up,
        } }, ExecutableOccurrenceV1 { entry: entry(&source.handler_designation, source.arguments.len())?,
            arguments: source.arguments.iter().copied().map(ExecutableValueV1::Number).collect() }));
    }
    for source in &package.scalar_input_bindings {
        expected.push((ExecutableInputSourceV1::Scalar { channel: source.channel.clone() },
            ExecutableOccurrenceV1 { entry: entry(&source.handler_designation, 1)?, arguments: vec![ExecutableValueV1::number(0.0)?] }));
    }
    for source in &package.referent_input_bindings {
        expected.push((ExecutableInputSourceV1::Referent { channel: source.channel.clone() },
            ExecutableOccurrenceV1 { entry: entry(&source.handler_designation, 1)?,
                arguments: vec![ExecutableValueV1::Referent(ExecutableReferentV1::declared(source.domain.get(), 0))] }));
    }
    for (binding, (source, occurrence)) in input.events.iter().zip(expected) {
        if binding.source != source || binding.occurrence != occurrence { return Err(reject()); }
    }
    if tick.is_empty() {
        let rule = next.plan.program.rules.last().ok_or_else(reject)?;
        if next.plan.program.rules.len() != next.lowered.program.rules.len() + 1
            || !rule.predicates.is_empty() || !rule.required_present.is_empty() || !rule.required_absent.is_empty()
            || !rule.assignments.is_empty() || !rule.removals.is_empty() { return Err(reject()); }
        tick.push(rule.entry);
    }
    if input.tick.entries != tick { return Err(reject()); }
    Ok(())
}
