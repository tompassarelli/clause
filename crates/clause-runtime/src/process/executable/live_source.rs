//! Compiler-checked source transitions applied to runtime-owned live state.
use super::*;
use clause_package::{
    CheckedCanonicalSourceAnalysisV1, CanonicalAllocatedIdentityV1, CanonicalDeclaredFrontendV1, CanonicalSourceEditV1, CanonicalSourceContextV1,
    ProgramChangeOccurrenceId,
    plan_independent_canonical_source_allocations_v1, read_canonical_source_with_imports_and_frontend_v1, CanonicalSourceImportsV1,
};

/// Aggregate envelope for one compiler-produced source transition witness.
///
/// CET3 carries both independently bounded old/new session snapshots together
/// with the source operation that relates them. Constituent formats retain
/// their own limits; this is only the outer transport ceiling.
pub const EXECUTABLE_SOURCE_EDIT_LIMIT_V1: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableSourceEditV1 {
    pub old_source: Vec<u8>,
    pub declared_frontend: Vec<u8>,
    pub imports: CanonicalSourceImportsV1,
    pub old_root: ProgramChangeOccurrenceId,
    pub new_root: ProgramChangeOccurrenceId,
    pub operation: ExecutableSourceOperationV1,
    pub old_cpp1: Vec<u8>,
    pub new_cpp1: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableSourceOperationV1 {
    ScalarEffect {
        handler: FormationLocalId,
        effect: FormationLocalId,
        field_path: Vec<FormationLocalId>,
        expression: Vec<u8>,
    },
    ReplaceItems(Vec<ExecutableSourceItemReplacementV1>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableSourceItemReplacementV1 {
    pub identity: CanonicalAllocatedIdentityV1,
    pub replacement: Vec<u8>,
}

/// Exact old/new snapshot addresses of a continuing source occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableSourceContinuityV1 {
    pub old_snapshot: ProgramSnapshotId,
    pub new_snapshot: ProgramSnapshotId,
    pub identities: BTreeMap<CanonicalAllocatedIdentityV1, CanonicalAllocatedIdentityV1>,
    pub slots: Vec<(u16, u16)>,
    /// Old/new Formation addresses and the stable first-snapshot occurrence
    /// address retained through this explicit chain of source operations.
    pub occurrences: Vec<(
        FormationLocalId,
        FormationLocalId,
        ProgramSnapshotId,
        FormationLocalId,
        [u8; IDENTITY_BYTES],
    )>,
}

pub struct CheckedExecutableSourceEditV1 {
    pub(crate) old_plan: ExecutablePhysicalPlanIdV1,
    pub(crate) new_plan: ExecutablePhysicalPlanIdV1,
    edit: CanonicalSourceEditV1,
    continuity: ExecutableSourceContinuityV1,
    pub(crate) preparation: Arc<CheckedExecutableSourcePreparationV1>,
}

impl CheckedExecutableSourceEditV1 {
    pub fn continuity(&self) -> &ExecutableSourceContinuityV1 {
        &self.continuity
    }
}

impl ExecutablePhysicalPlanV1 {
    /// Bind source metadata using the state layout that produced this program,
    /// including retained slots when reopening or editing an admitted world.
    pub fn bind_source_snapshot(
        &mut self,
        scope: TermScope,
        package: &clause_package::CanonicalSourcePackageSliceV1,
        artifact: clause_package::CanonicalSourceArtifactIdV1,
        root: ProgramChangeOccurrenceId,
        states: &[ExecutableCanonicalStateBindingV1],
    ) -> Result<(), ExecutableErrorV1> {
        let _profile = source_profile_scope_v1(SourceProfilePhaseV1::SnapshotMetadata);
        self.project_source_rows(scope, package, states)?;
        let projection = self
            .program
            .projection
            .as_mut()
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        let hex = |bytes: &[u8]| {
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        let package_id = package.checked_package.id();
        let snapshot = package.checked_package.constitution().snapshot();
        let fields = [
            (b"package".as_slice(), package_id.as_bytes().as_slice()),
            (b"snapshot".as_slice(), snapshot.as_bytes().as_slice()),
            (b"artifact".as_slice(), artifact.as_bytes().as_slice()),
            (b"change".as_slice(), root.as_bytes().as_slice()),
        ];
        let metadata = projection_object(
            scope,
            fields
                .into_iter()
                .map(|(key, value)| {
                    Ok((
                        key.to_vec(),
                        projected_scalar_value_term(
                            scope,
                            &ExecutableValueV1::symbol(hex(value).as_bytes())?,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, ExecutableErrorV1>>()?,
        )?;
        projection.template = Term::triple([
            projection_literal(scope, b"clause/js-field-v1", b"$source-snapshot")?,
            metadata,
            projection.template.clone(),
        ])
        .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
        self.source_metadata = Some(source_metadata(scope, package, artifact, states)?);
        Ok(())
    }
}

fn source_metadata(
    scope: TermScope,
    package: &clause_package::CanonicalSourcePackageSliceV1,
    artifact: clause_package::CanonicalSourceArtifactIdV1,
    states: &[ExecutableCanonicalStateBindingV1],
) -> Result<Term, ExecutableErrorV1> {
    let mut handlers = package.executable_handlers.iter().collect::<Vec<_>>();
    handlers.sort_by_key(|handler| handler.id);
    let mut rules = Vec::new();
    for handler in handlers {
        let origin = package
            .emissions
            .iter()
            .find(|emission| {
                emission.allocations.iter().any(|allocation| {
                    allocation.identity == CanonicalAllocatedIdentityV1::Formation(handler.id)
                })
            })
            .map(|emission| emission.origin);
        for rule in &handler.rules {
            let mut fields = vec![
                (
                    b"handler".to_vec(),
                    diagnostic_number(scope, handler.id.get() as f64)?,
                ),
                (
                    b"designation".to_vec(),
                    diagnostic_text(scope, &String::from_utf8_lossy(&handler.designation))?,
                ),
                (
                    b"laws".to_vec(),
                    projection_object(
                        scope,
                        rule.law_origins
                            .iter()
                            .enumerate()
                            .map(|(index, origin)| {
                                Ok((index.to_string().into_bytes(), origin_term(scope, *origin)?))
                            })
                            .collect::<Result<_, ExecutableErrorV1>>()?,
                    )?,
                ),
            ];
            if let Some(origin) = origin {
                fields.push((b"origin".to_vec(), origin_term(scope, origin)?));
            }
            rules.push((
                rules.len().to_string().into_bytes(),
                projection_object(scope, fields)?,
            ));
        }
    }
    let states = states.iter().map(|binding| Ok((binding.slot.to_string().into_bytes(),
        source_state_metadata(scope, binding)?))).collect::<Result<_, ExecutableErrorV1>>()?;
    projection_object(
        scope,
        vec![
            (
                b"artifact".to_vec(),
                diagnostic_text(scope, &hex_identity(artifact.as_bytes()))?,
            ),
            (
                b"snapshot".to_vec(),
                diagnostic_text(
                    scope,
                    &hex_identity(package.checked_package.constitution().snapshot().as_bytes()),
                )?,
            ),
            (b"rules".to_vec(), diagnostic_index(scope, rules)?),
            (b"states".to_vec(), diagnostic_index(scope, states)?),
        ],
    )
}

fn source_state_metadata(scope: TermScope, binding: &ExecutableCanonicalStateBindingV1) -> Result<Term, ExecutableErrorV1> {
    let state = &binding.state;
    let mut fields = vec![
        (
            b"slot".to_vec(),
            diagnostic_number(scope, binding.slot as f64)?,
        ),
        (
            b"subject".to_vec(),
            diagnostic_text(scope, &String::from_utf8_lossy(&state.subject))?,
        ),
        (
            b"relation".to_vec(),
            diagnostic_text(scope, &String::from_utf8_lossy(&state.relation_designation))?,
        ),
        (
            b"assertion".to_vec(),
            diagnostic_number(scope, state.assertion.get() as f64)?,
        ),
        (
            b"schema".to_vec(),
            diagnostic_number(scope, state.relation.get() as f64)?,
        ),
        (
            b"subject-role".to_vec(),
            diagnostic_number(scope, state.subject_role.role.get() as f64)?,
        ),
        (
            b"value-role".to_vec(),
            diagnostic_number(scope, state.value_role.role.get() as f64)?,
        ),
    ];
    if let Some(referent) = state.subject_identity {
        fields.push((
            b"referent".to_vec(),
            projected_scalar_value_term(
                scope,
                &ExecutableValueV1::Referent(ExecutableReferentV1::declared(
                    referent.domain.get(),
                    referent.identity.get(),
                )),
            )?,
        ));
    }
    if let clause_package::CanonicalStatePathV1::Field {
        formation,
        designation,
    } = &state.path
    {
        fields.push((
            b"field".to_vec(),
            diagnostic_text(scope, &String::from_utf8_lossy(designation))?,
        ));
        fields.push((
            b"field-formation".to_vec(),
            diagnostic_number(scope, formation.get() as f64)?,
        ));
    }
    projection_object(scope, fields)
}

pub(super) fn origin_term(
    scope: TermScope,
    origin: clause_package::CanonicalSourceOriginV1,
) -> Result<Term, ExecutableErrorV1> {
    projection_object(
        scope,
        vec![
            (
                b"artifact".to_vec(),
                diagnostic_text(scope, &hex_identity(origin.artifact.as_bytes()))?,
            ),
            (
                b"start".to_vec(),
                diagnostic_number(scope, origin.start as f64)?,
            ),
            (
                b"end".to_vec(),
                diagnostic_number(scope, origin.end as f64)?,
            ),
        ],
    )
}

pub(super) fn hex_identity(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
pub(super) fn diagnostic_text(scope: TermScope, value: &str) -> Result<Term, ExecutableErrorV1> {
    projected_scalar_value_term(scope, &ExecutableValueV1::text(value)?)
}
pub(super) fn diagnostic_number(scope: TermScope, value: f64) -> Result<Term, ExecutableErrorV1> {
    projected_scalar_value_term(scope, &ExecutableValueV1::number(value)?)
}
pub(super) fn diagnostic_field<'a>(term: &'a Term, field: &[u8]) -> Option<&'a Term> {
    let mut current = term;
    while let Some(triple) = current.as_triple() {
        let [key, value, rest] = triple.slots();
        if key.as_atom().is_some_and(|atom| {
            atom.kind() == b"clause/js-field-v1" && atom.canonical_payload() == field
        }) {
            return Some(value);
        }
        current = rest;
    }
    None
}

pub(super) fn diagnostic_index(
    scope: TermScope,
    fields: Vec<(Vec<u8>, Term)>,
) -> Result<Term, ExecutableErrorV1> {
    let mut pages = BTreeMap::<usize, Vec<(Vec<u8>, Term)>>::new();
    for (key, value) in fields {
        let index = std::str::from_utf8(&key)
            .ok()
            .and_then(|key| key.parse::<usize>().ok())
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        pages
            .entry(index / 64)
            .or_default()
            .push(((index % 64).to_string().into_bytes(), value));
    }
    projection_object(
        scope,
        pages
            .into_iter()
            .map(|(page, fields)| {
                Ok((
                    page.to_string().into_bytes(),
                    projection_object(scope, fields)?,
                ))
            })
            .collect::<Result<_, ExecutableErrorV1>>()?,
    )
}

pub(super) fn diagnostic_index_field(term: &Term, index: u16) -> Option<&Term> {
    diagnostic_field(
        diagnostic_field(term, (index / 64).to_string().as_bytes())?,
        (index % 64).to_string().as_bytes(),
    )
}

/// Reconstruct a retained recipe's dispatch layout for the same exact checked
/// source snapshot. The compiler's rule-emission metadata binds every rule to
/// its handler; neither equal rule text nor physical ordinals create identity.
/// Every emitted instruction must match. This preserves an admitted schedule
/// while reopening; a source edit separately selects its successor schedule.
pub fn replay_canonical_executable_entry_layout_v1(
    scope: TermScope,
    package: &clause_package::CanonicalSourcePackageSliceV1,
    artifact: clause_package::CanonicalSourceArtifactIdV1,
    lowered: &mut ExecutableCanonicalProgramV1,
    recorded: &ExecutablePhysicalPlanV1,
) -> Result<(), ExecutableErrorV1> {
    let rejected =
        || ExecutableErrorV1::SourceContinuityRejected("recorded source dispatch layout");
    let recorded_states = recorded.source_metadata.as_ref()
        .and_then(|metadata| diagnostic_field(metadata, b"states")).ok_or_else(rejected)?;
    let mut by_state = BTreeMap::new();
    for slot in 0..lowered.states.len() {
        let slot = u16::try_from(slot).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        let record = diagnostic_index_field(recorded_states, slot).ok_or_else(rejected)?;
        let [key, value, state] = record.as_triple().ok_or_else(rejected)?.slots();
        if key.as_atom().is_none_or(|key| key.kind() != b"clause/js-field-v1" || key.canonical_payload() != b"slot")
            || *value != diagnostic_number(scope, f64::from(slot))?
            || by_state.insert(state.clone(), slot).is_some() {
            return Err(rejected());
        }
    }
    let mut slots = BTreeMap::new();
    for binding in &lowered.states {
        let record = source_state_metadata(scope, binding)?;
        let key = record.as_triple().ok_or_else(rejected)?.slots()[2];
        slots.insert(binding.state.clone(), *by_state.get(key).ok_or_else(rejected)?);
    }
    if lowered.states.iter().any(|binding| slots[&binding.state] != binding.slot) {
        let roles = lowered.states.iter().map(|binding| binding.projection_role).collect::<Vec<_>>();
        let retained = lower_canonical_executable_program_with_layout(scope, &package.state_cells,
            &package.executable_handlers, &roles, Some(&slots), None, None)?;
        validate_program(&retained.program)
            .map_err(|error| ExecutableErrorV1::CanonicalLoweringValidation(Box::new(error)))?;
        *lowered = retained;
    }
    if recorded.source_metadata.as_ref()
        != Some(&source_metadata(
            scope,
            package,
            artifact,
            &lowered.states,
        )?)
        || recorded.program.initial_configuration != lowered.program.initial_configuration
        || recorded.program.rules.len() < lowered.program.rules.len()
    {
        return Err(rejected());
    }
    let mut handlers = package.executable_handlers.iter().collect::<Vec<_>>();
    handlers.sort_by_key(|handler| handler.id);
    let mut cursor = 0;
    let mut entries = BTreeMap::new();
    let mut external_entries = BTreeMap::new();
    for handler in handlers {
        let end = cursor + handler.rules.len();
        let recorded_rules = recorded
            .program
            .rules
            .get(cursor..end)
            .ok_or_else(rejected)?;
        let entry = recorded_rules.first().ok_or_else(rejected)?.entry;
        let group = (
            handler.trigger,
            &handler.designation,
            (!matches!(
                handler.trigger,
                CanonicalHandlerTriggerV1::External | CanonicalHandlerTriggerV1::FixedTickRoot
            ))
            .then_some(handler.id),
        );
        if entries
            .insert(entry, group)
            .is_some_and(|prior| prior != group)
            || recorded_rules.iter().any(|rule| rule.entry != entry)
        {
            return Err(rejected());
        }
        if handler.trigger == CanonicalHandlerTriggerV1::External
            && external_entries
                .insert(&handler.designation, entry)
                .is_some_and(|prior| prior != entry)
        {
            return Err(rejected());
        }
        for (expected, recorded) in lowered.program.rules[cursor..end]
            .iter_mut()
            .zip(recorded_rules)
        {
            expected.entry = entry;
            if expected != recorded {
                return Err(rejected());
            }
        }
        let binding = lowered
            .handlers
            .iter_mut()
            .find(|binding| binding.handler == handler.id)
            .ok_or_else(rejected)?;
        if binding.invocation_entry == binding.entry {
            binding.invocation_entry = entry;
        }
        binding.entry = entry;
        cursor = end;
    }
    // Synthesized named invocation recipes are checked in full as well, and
    // cannot alias a retained scheduled entry.
    if lowered.program.rules[cursor..]
        .iter()
        .any(|rule| entries.contains_key(&rule.entry))
        || lowered.program.rules[cursor..]
            != recorded.program.rules[cursor..lowered.program.rules.len()]
    {
        return Err(rejected());
    }
    Ok(())
}

pub fn encode_executable_source_edit_v1(
    edit: &ExecutableSourceEditV1,
) -> Result<Vec<u8>, ExecutableErrorV1> {
    let mut bytes = match edit.operation {
        ExecutableSourceOperationV1::ScalarEffect { .. } => b"CET3".to_vec(),
        ExecutableSourceOperationV1::ReplaceItems(_) => b"CET4".to_vec(),
    };
    bytes.extend_from_slice(edit.old_root.as_bytes());
    bytes.extend_from_slice(edit.new_root.as_bytes());
    let expression = match &edit.operation {
        ExecutableSourceOperationV1::ScalarEffect {
            handler,
            effect,
            field_path,
            expression,
        } => {
            bytes.extend_from_slice(&handler.get().to_le_bytes());
            bytes.extend_from_slice(&effect.get().to_le_bytes());
            bytes.extend_from_slice(
                &u32::try_from(field_path.len())
                    .map_err(|_| ExecutableErrorV1::ResourceLimit)?
                    .to_le_bytes(),
            );
            for field in field_path {
                bytes.extend_from_slice(&field.get().to_le_bytes());
            }
            Some(expression)
        }
        ExecutableSourceOperationV1::ReplaceItems(items) => {
            bytes.extend_from_slice(
                &u32::try_from(items.len())
                    .map_err(|_| ExecutableErrorV1::ResourceLimit)?
                    .to_le_bytes(),
            );
            for item in items {
                match item.identity {
                    CanonicalAllocatedIdentityV1::Formation(id) => {
                        bytes.push(0);
                        bytes.extend_from_slice(&id.get().to_le_bytes());
                    }
                    CanonicalAllocatedIdentityV1::Mode(id) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&id.operator.get().to_le_bytes());
                        bytes.extend_from_slice(&id.mode.get().to_le_bytes());
                    }
                    _ => return Err(ExecutableErrorV1::MalformedProgram),
                }
                bytes.extend_from_slice(
                    &u32::try_from(item.replacement.len())
                        .map_err(|_| ExecutableErrorV1::ResourceLimit)?
                        .to_le_bytes(),
                );
                bytes.extend_from_slice(&item.replacement);
            }
            None
        }
    };
    for blob in [
        Some(&edit.old_source),
        Some(&edit.declared_frontend),
        expression,
        Some(&edit.old_cpp1),
        Some(&edit.new_cpp1),
    ]
    .into_iter()
    .flatten()
    {
        bytes.extend_from_slice(
            &u32::try_from(blob.len())
                .map_err(|_| ExecutableErrorV1::ResourceLimit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(blob);
    }
    encode_source_imports(&mut bytes, &edit.imports)?;
    if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    Ok(bytes)
}

pub fn decode_executable_source_edit_v1(
    bytes: &[u8],
) -> Result<ExecutableSourceEditV1, ExecutableErrorV1> {
    if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    let mut d = Decoder::new(bytes);
    let magic = d.take(4)?;
    if magic != b"CET3" && magic != b"CET4" {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let old_root = ProgramChangeOccurrenceId::from_bytes(d.identity()?);
    let new_root = ProgramChangeOccurrenceId::from_bytes(d.identity()?);
    let mut operation = if magic == b"CET3" {
        let handler = FormationLocalId::new(d.u32()?);
        let effect = FormationLocalId::new(d.u32()?);
        let field_count = d.u32()? as usize;
        if field_count > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 / size_of::<u32>() {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        let field_path = (0..field_count)
            .map(|_| d.u32().map(FormationLocalId::new))
            .collect::<Result<Vec<_>, _>>()?;
        ExecutableSourceOperationV1::ScalarEffect {
            handler,
            effect,
            field_path,
            expression: vec![],
        }
    } else {
        let count = d.u32()? as usize;
        if count > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 / 9 {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        let mut items = Vec::new();
        for _ in 0..count {
            let kind = d.take(1)?[0];
            let id = d.u32()?;
            let identity = match kind {
                0 => CanonicalAllocatedIdentityV1::Formation(FormationLocalId::new(id)),
                1 => CanonicalAllocatedIdentityV1::Mode(clause_package::LocalModeRefV2 {
                    operator: clause_package::OperatorLocalId::new(id),
                    mode: clause_package::ModeLocalId::new(d.u32()?),
                }),
                _ => return Err(ExecutableErrorV1::MalformedProgram),
            };
            let len = d.u32()? as usize;
            items.push(ExecutableSourceItemReplacementV1 {
                identity,
                replacement: d.take(len)?.to_vec(),
            });
        }
        ExecutableSourceOperationV1::ReplaceItems(items)
    };
    let mut blob = || {
        let len = d.u32()? as usize;
        Ok::<_, ExecutableErrorV1>(d.take(len)?.to_vec())
    };
    let old_source = blob()?;
    let declared_frontend = blob()?;
    if let ExecutableSourceOperationV1::ScalarEffect { expression, .. } = &mut operation {
        *expression = blob()?;
    }
    let result = ExecutableSourceEditV1 {
        old_root,
        new_root,
        operation,
        old_source,
        declared_frontend,
        old_cpp1: blob()?,
        new_cpp1: blob()?,
        imports: decode_source_imports(&mut d)?,
    };
    if !d.is_complete() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(result)
}

fn encode_source_imports(bytes: &mut Vec<u8>, imports: &CanonicalSourceImportsV1) -> Result<(), ExecutableErrorV1> {
    bytes.extend_from_slice(&u32::try_from(imports.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?.to_le_bytes());
    for (name, source) in imports {
        for blob in [name.as_bytes(), source.as_slice()] {
            bytes.extend_from_slice(&u32::try_from(blob.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?.to_le_bytes());
            bytes.extend_from_slice(blob);
            if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 { return Err(ExecutableErrorV1::ResourceLimit); }
        }
    }
    Ok(())
}

fn decode_source_imports(d: &mut Decoder<'_>) -> Result<CanonicalSourceImportsV1, ExecutableErrorV1> {
    let count = d.u32()? as usize;
    if count > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 / 8 { return Err(ExecutableErrorV1::ResourceLimit); }
    let mut imports = CanonicalSourceImportsV1::new();
    for _ in 0..count {
        let len = d.u32()? as usize;
        let name = std::str::from_utf8(d.take(len)?).map_err(|_| ExecutableErrorV1::MalformedProgram)?.to_owned();
        if imports.last_key_value().is_some_and(|(last, _)| last >= &name) {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let len = d.u32()? as usize;
        imports.insert(name, d.take(len)?.to_vec());
    }
    Ok(imports)
}

/// Source-only startup capsule, including exact declared imports. Executable state never crosses this interface.
pub fn encode_executable_source_preparation_v1(
    source: &[u8], root: ProgramChangeOccurrenceId, declared_frontend: &[u8], imports: &CanonicalSourceImportsV1,
) -> Result<Vec<u8>, ExecutableErrorV1> {
    let mut bytes = b"CPS2".to_vec();
    bytes.extend_from_slice(root.as_bytes());
    for blob in [source, declared_frontend] {
        bytes.extend_from_slice(&u32::try_from(blob.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?.to_le_bytes());
        bytes.extend_from_slice(blob);
    }
    encode_source_imports(&mut bytes, imports)?;
    if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 { return Err(ExecutableErrorV1::ResourceLimit); }
    Ok(bytes)
}

/// Privately checked analysis of one exact source and physical realization.
/// It is local to this compiler instance and cannot be deserialized from a host.
pub struct CheckedExecutableSourcePreparationV1 {
    pub(crate) analysis: Arc<CheckedCanonicalSourceAnalysisV1>,
    declared_frontend: Vec<u8>,
    scope: TermScope,
    pub(crate) exact_cpp1: Vec<u8>,
    pub(crate) identity: ExecutablePhysicalPlanIdV1,
    pub(crate) plan: ExecutablePhysicalPlanV1,
    pub(crate) lowered: ExecutableCanonicalProgramV1,
}

pub fn check_executable_source_preparation_v1(
    bytes: &[u8], scope: TermScope, exact_cpp1: &[u8],
) -> Result<CheckedExecutableSourcePreparationV1, ExecutableErrorV1> {
    if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 { return Err(ExecutableErrorV1::ResourceLimit); }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != b"CPS2" { return Err(ExecutableErrorV1::MalformedProgram); }
    let root = ProgramChangeOccurrenceId::from_bytes(d.identity()?);
    let len = d.u32()? as usize;
    let source = d.take(len)?;
    let len = d.u32()? as usize;
    let declared_frontend = d.take(len)?;
    let imports = decode_source_imports(&mut d)?;
    if !d.is_complete() { return Err(ExecutableErrorV1::MalformedProgram); }
    let rejected = |_| ExecutableErrorV1::MalformedProgram;
    let frontend = CanonicalDeclaredFrontendV1::read(declared_frontend).map_err(rejected)?;
    let cst = read_canonical_source_with_imports_and_frontend_v1(source, &imports, &frontend).map_err(rejected)?;
    let allocations = plan_independent_canonical_source_allocations_v1(&cst, root).map_err(rejected)?;
    let analysis = CheckedCanonicalSourceAnalysisV1::new(cst, allocations,
        CanonicalSourceContextV1 { universe: scope.universe, semantics: scope.semantics }).map_err(rejected)?;
    let plan = decode_executable_physical_plan_v1(exact_cpp1)?;
    let package = analysis.package();
    let roles = plan.program.projection.as_ref().ok_or(ExecutableErrorV1::MalformedProgram)?
        .bindings.iter().map(|binding| binding.role).collect::<Vec<_>>();
    let mut lowered = lower_canonical_executable_program_v1(scope, &package.state_cells, &package.executable_handlers, &roles)?;
    replay_canonical_executable_entry_layout_v1(scope, package, analysis.source().artifact(), &mut lowered, &plan)?;
    let mut expected = plan.clone();
    expected.program = lowered.program.clone();
    if plan.program.rules.len() == expected.program.rules.len() + 1 {
        let checkpoint = plan.program.rules.last().ok_or(ExecutableErrorV1::MalformedProgram)?;
        if !checkpoint.predicates.is_empty() || !checkpoint.required_present.is_empty()
            || !checkpoint.required_absent.is_empty() || !checkpoint.assignments.is_empty() || !checkpoint.removals.is_empty() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        expected.program.rules.push(checkpoint.clone());
    }
    expected.project_referent_input_domains(scope)?;
    expected.bind_source_snapshot(scope, package, analysis.source().artifact(), root, &lowered.states)?;
    if expected != plan { return Err(ExecutableErrorV1::SourceContinuityRejected("prepared source does not realize exact bound CPP1")); }
    Ok(CheckedExecutableSourcePreparationV1 { analysis: Arc::new(analysis), declared_frontend: declared_frontend.to_vec(), scope, exact_cpp1: exact_cpp1.to_vec(), identity: physical_plan_identity(exact_cpp1), plan, lowered })
}

pub fn check_executable_source_edit_v1(
    witness: &ExecutableSourceEditV1,
    scope: TermScope,
) -> Result<CheckedExecutableSourceEditV1, ExecutableErrorV1> {
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::OldElaboration);
    let capsule = encode_executable_source_preparation_v1(&witness.old_source, witness.old_root, &witness.declared_frontend, &witness.imports)?;
    let preparation = check_executable_source_preparation_v1(&capsule, scope, &witness.old_cpp1)?;
    drop(phase);
    check_prepared_executable_source_edit_v1(witness, scope, &preparation)
}

pub fn check_prepared_executable_source_edit_v1(
    witness: &ExecutableSourceEditV1, scope: TermScope, preparation: &CheckedExecutableSourcePreparationV1,
) -> Result<CheckedExecutableSourceEditV1, ExecutableErrorV1> {
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::WitnessCheck);
    if preparation.scope != scope || preparation.exact_cpp1 != witness.old_cpp1
        || preparation.analysis.source().exact_source() != witness.old_source
        || preparation.analysis.plan().root() != witness.old_root
        || preparation.declared_frontend != witness.declared_frontend {
        return Err(ExecutableErrorV1::SourceContinuityRejected("stale source preparation"));
    }
    if preparation.analysis.source().imports() != &witness.imports {
        return Err(ExecutableErrorV1::SourceContinuityRejected("stale source imports"));
    }
    let derived = derive_prepared_source_edit(preparation, &witness.operation, witness.new_root)?;
    if derived.preparation.exact_cpp1 != witness.new_cpp1 {
        return Err(ExecutableErrorV1::SourceContinuityRejected("edited source does not realize exact replacement CPP1"));
    }
    Ok(derived)
}

pub(crate) fn derive_prepared_scalar_edit_v1(
    preparation: &CheckedExecutableSourcePreparationV1, operation: &ExecutableSourceOperationV1,
    new_root: ProgramChangeOccurrenceId,
) -> Result<CheckedExecutableSourceEditV1, ExecutableErrorV1> {
    if !matches!(operation, ExecutableSourceOperationV1::ScalarEffect { .. }) { return Err(ExecutableErrorV1::MalformedProgram); }
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::WitnessCheck);
    derive_prepared_source_edit(preparation, operation, new_root)
}

fn derive_prepared_source_edit(
    preparation: &CheckedExecutableSourcePreparationV1, operation: &ExecutableSourceOperationV1,
    new_root: ProgramChangeOccurrenceId,
) -> Result<CheckedExecutableSourceEditV1, ExecutableErrorV1> {
    let scope = preparation.scope;
    let rejected = |_| ExecutableErrorV1::MalformedProgram;
    let old_cst = preparation.analysis.source();
    let old_allocations = preparation.analysis.plan();
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::OfferedEdit);
    let edit = match operation {
        ExecutableSourceOperationV1::ScalarEffect {
            handler,
            effect,
            field_path,
            expression,
        } => {
            preparation.analysis.replace_scalar_effect(*handler, *effect, field_path, expression, new_root)
                .map_err(rejected)?
        }
        ExecutableSourceOperationV1::ReplaceItems(items) => {
            let offered =
                clause_package::canonical_editable_source_items_v1(&old_cst, &old_allocations)
                    .map_err(rejected)?;
            let replacements = items
                .iter()
                .map(|item| {
                    Ok(clause_package::CanonicalSourceItemReplacementV1 {
                        selected: offered
                            .iter()
                            .find(|selected| selected.identity == item.identity)
                            .ok_or(ExecutableErrorV1::MalformedProgram)?
                            .clone(),
                        replacement: item.replacement.clone(),
                    })
                })
                .collect::<Result<Vec<_>, ExecutableErrorV1>>()?;
            clause_package::replace_canonical_source_items_v1(
                &old_cst,
                &old_allocations,
                &replacements,
                new_root,
            )
            .map_err(rejected)?
        }
    };
    drop(phase);
    let old = preparation.analysis.package();
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::NewElaboration);
    let next_analysis = preparation.analysis.advance(&edit).map_err(rejected)?;
    let new = next_analysis.package();
    drop(phase);
    let old_plan = &preparation.plan;
    let roles = old_plan.program.projection.as_ref().ok_or(ExecutableErrorV1::MalformedProgram)?
        .bindings.iter().map(|binding| binding.role).collect::<Vec<_>>();
    let old_lowered = &preparation.lowered;
    let retained_layout = matches!(operation, ExecutableSourceOperationV1::ScalarEffect { .. });
    let retained_slots = if retained_layout {
        Some(old_lowered.states.iter().map(|binding|
            edit.state(&binding.state).map(|state| (state, binding.slot)).map_err(rejected))
            .collect::<Result<BTreeMap<_, _>, _>>()?)
    } else { None };
    let retained_entries = if retained_layout {
        Some(old_lowered.handlers.iter().map(|binding|
            edit.formation(binding.handler).map(|handler| (handler, binding.entry)).map_err(rejected))
            .collect::<Result<BTreeMap<_, _>, _>>()?)
    } else { None };
    let retained_rules = retained_slots.as_ref().map(|slots|
        retain_lowered_source_rules(preparation, &next_analysis, &edit, slots)).transpose()?;
    let new_lowered = lower_canonical_executable_program_with_layout(
        scope,
        &new.state_cells,
        &new.executable_handlers,
        &roles,
        retained_slots.as_ref(),
        retained_entries.as_ref(),
        retained_rules,
    )?;
    let mut expected_new = ExecutablePhysicalPlanV1 {
        application_shape: old_plan.application_shape,
        mode: old_plan.mode,
        refinement: old_plan.refinement.clone(),
        target: old_plan.target,
        input: old_plan.input.clone(),
        program: new_lowered.program.clone(),
        source_metadata: None,
    };
    let mut entries = BTreeMap::new();
    for binding in &old_lowered.handlers {
        let new_id = edit.formation(binding.handler).map_err(rejected)?;
        let matching = new_lowered
            .handlers
            .iter()
            .find(|new| {
                new.handler == new_id
                    && new.trigger == binding.trigger
                    && new.argument_count == binding.argument_count
            })
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        entries.insert(binding.entry, matching.entry);
    }
    // The event-only publication checkpoint is effect-free physical data.
    if old_plan.program.rules.len() == old_lowered.program.rules.len() + 1 {
        let checkpoint = old_plan
            .program
            .rules
            .last()
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        if !checkpoint.predicates.is_empty()
            || !checkpoint.required_present.is_empty()
            || !checkpoint.required_absent.is_empty()
            || !checkpoint.assignments.is_empty()
            || !checkpoint.removals.is_empty()
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let new_entry = new_lowered
            .program
            .rules
            .iter()
            .map(|rule| rule.entry)
            .max()
            .and_then(|entry| entry.checked_add(1))
            .ok_or(ExecutableErrorV1::ResourceLimit)?;
        entries.insert(checkpoint.entry, new_entry);
        let mut replacement = checkpoint.clone();
        replacement.entry = new_entry;
        expected_new.program.rules.push(replacement);
    }
    if let Some(input) = &mut expected_new.input {
        for binding in &mut input.events {
            binding.occurrence.entry = *entries
                .get(&binding.occurrence.entry)
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
            if matches!(binding.source, ExecutableInputSourceV1::Referent { .. }) {
                let [ExecutableValueV1::Referent(placeholder)] =
                    binding.occurrence.arguments.as_mut_slice()
                else {
                    return Err(ExecutableErrorV1::SourceContinuityRejected(
                        "referent input shape",
                    ));
                };
                if placeholder.identity != ExecutableReferentIdentityV1::Declared(0) {
                    return Err(ExecutableErrorV1::SourceContinuityRejected(
                        "referent input placeholder",
                    ));
                }
                placeholder.domain = edit
                    .formation(FormationLocalId::new(placeholder.domain))
                    .map_err(rejected)?
                    .get();
            } else {
                for argument in &mut binding.occurrence.arguments {
                    *argument = migrate_value(argument, &edit)?;
                }
            }
        }
        for entry in &mut input.tick.entries {
            *entry = *entries
                .get(entry)
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
        }
        // Tick order is source trigger/checked handler identity, not old entry order.
        input.tick.entries.sort_by_key(|entry| {
            new_lowered
                .handlers
                .iter()
                .find(|binding| binding.entry == *entry)
                .map(|binding| (binding.trigger, binding.handler))
        });
        input.tick.entries.dedup();
    }
    expected_new.add_referent_input_projection(scope)?;
    expected_new.bind_source_snapshot(scope, &new, edit.source().artifact(), new_root, &new_lowered.states)?;
    let _compare = source_profile_scope_v1(SourceProfilePhaseV1::CompareAndMap);
    let exact_cpp1 = encode_executable_physical_plan_v1(&expected_new)?;
    let mut slots = Vec::new();
    let new_states = new_lowered
        .states
        .iter()
        .map(|binding| (&binding.state, binding.slot))
        .collect::<BTreeMap<_, _>>();
    for old in &old_lowered.states {
        let target = edit.state(&old.state).map_err(rejected)?;
        let new_slot = *new_states
            .get(&target)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        slots.push((old.slot, new_slot));
    }
    if slots.len() != new_states.len() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let new_plan = physical_plan_identity(&exact_cpp1);
    Ok(CheckedExecutableSourceEditV1 {
        old_plan: preparation.identity,
        new_plan,
        continuity: ExecutableSourceContinuityV1 {
            old_snapshot: old.checked_package.constitution().snapshot(),
            new_snapshot: new.checked_package.constitution().snapshot(),
            identities: edit.retained().clone(),
            slots,
            occurrences: vec![],
        },
        edit,
        preparation: Arc::new(CheckedExecutableSourcePreparationV1 {
            analysis: Arc::new(next_analysis), declared_frontend: preparation.declared_frontend.clone(), scope,
            exact_cpp1, identity: new_plan, plan: expected_new, lowered: new_lowered,
        }),
    })
}

/// Compact explicit scalar operation. Result identity compares independently
/// derived exact physical plans; it never substitutes for source checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableScalarEditTransactionV1 {
    pub old_plan: ExecutablePhysicalPlanIdV1,
    pub new_plan: ExecutablePhysicalPlanIdV1,
    pub new_root: ProgramChangeOccurrenceId,
    pub operation: ExecutableSourceOperationV1,
}

pub fn encode_executable_scalar_edit_transaction_v1(transaction: &ExecutableScalarEditTransactionV1) -> Result<Vec<u8>, ExecutableErrorV1> {
    let ExecutableSourceOperationV1::ScalarEffect { handler, effect, field_path, expression } = &transaction.operation
        else { return Err(ExecutableErrorV1::MalformedProgram); };
    let mut bytes = b"CEX1".to_vec();
    bytes.extend_from_slice(transaction.old_plan.as_bytes());
    bytes.extend_from_slice(transaction.new_plan.as_bytes());
    bytes.extend_from_slice(transaction.new_root.as_bytes());
    bytes.extend_from_slice(&handler.get().to_le_bytes());
    bytes.extend_from_slice(&effect.get().to_le_bytes());
    bytes.extend_from_slice(&u32::try_from(field_path.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?.to_le_bytes());
    for field in field_path { bytes.extend_from_slice(&field.get().to_le_bytes()); }
    bytes.extend_from_slice(&u32::try_from(expression.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?.to_le_bytes());
    bytes.extend_from_slice(expression);
    if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 { return Err(ExecutableErrorV1::ResourceLimit); }
    Ok(bytes)
}

pub fn decode_executable_scalar_edit_transaction_v1(bytes: &[u8]) -> Result<ExecutableScalarEditTransactionV1, ExecutableErrorV1> {
    if bytes.len() > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 { return Err(ExecutableErrorV1::ResourceLimit); }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != b"CEX1" { return Err(ExecutableErrorV1::MalformedProgram); }
    let old_plan = ExecutablePhysicalPlanIdV1::from_bytes(d.identity()?);
    let new_plan = ExecutablePhysicalPlanIdV1::from_bytes(d.identity()?);
    let new_root = ProgramChangeOccurrenceId::from_bytes(d.identity()?);
    let handler = FormationLocalId::new(d.u32()?);
    let effect = FormationLocalId::new(d.u32()?);
    let count = d.u32()? as usize;
    if count > EXECUTABLE_SOURCE_EDIT_LIMIT_V1 / 4 { return Err(ExecutableErrorV1::ResourceLimit); }
    let field_path = (0..count).map(|_| d.u32().map(FormationLocalId::new)).collect::<Result<Vec<_>, _>>()?;
    let length = d.u32()? as usize;
    let expression = d.take(length)?.to_vec();
    if !d.is_complete() { return Err(ExecutableErrorV1::MalformedProgram); }
    Ok(ExecutableScalarEditTransactionV1 { old_plan, new_plan, new_root,
        operation: ExecutableSourceOperationV1::ScalarEffect { handler, effect, field_path, expression } })
}

pub(crate) fn physical_plan_identity(bytes: &[u8]) -> ExecutablePhysicalPlanIdV1 {
    ExecutablePhysicalPlanIdV1::from_bytes(runtime_domain_hash(
        "clause/executable-physical-plan/v1",
        &[bytes],
    ))
}

fn retain_lowered_source_rules(
    preparation: &CheckedExecutableSourcePreparationV1,
    next: &CheckedCanonicalSourceAnalysisV1,
    edit: &CanonicalSourceEditV1,
    slots: &BTreeMap<CanonicalStateRefV1, u16>,
) -> Result<BTreeMap<(FormationLocalId, usize), ExecutableRuleV1>, ExecutableErrorV1> {
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::RetainedRuleRebinding);
    let mut handlers = preparation.analysis.package().executable_handlers.iter().collect::<Vec<_>>();
    handlers.sort_by_key(|handler| handler.id);
    let mut offsets = BTreeMap::new();
    let mut offset = 0;
    for handler in handlers {
        offsets.insert(handler.id, offset);
        offset += handler.rules.len();
    }
    let slot = |state: &CanonicalStateRefV1| slots.get(state).copied()
        .ok_or(ExecutableErrorV1::CanonicalLoweringUnknownState);
    let mut retained = BTreeMap::new();
    let mut member_sets = BTreeMap::new();
    for handler in &next.package().executable_handlers {
        for (index, source_rule) in handler.rules.iter().enumerate() {
            let Some((old_handler, old_index)) = next.retained_rule(preparation.analysis.plan().root(), handler.id, index) else { continue; };
            let old_offset = offsets.get(&old_handler).ok_or(ExecutableErrorV1::MalformedProgram)?;
            let mut rule = preparation.lowered.program.rules.get(old_offset + old_index)
                .ok_or(ExecutableErrorV1::MalformedProgram)?.clone();
            for predicate in &mut rule.predicates { rebind_lowered_expression(predicate, edit, &mut member_sets)?; }
            for (_, value) in &mut rule.assignments { rebind_lowered_expression(value, edit, &mut member_sets)?; }
            let mut assignments = rule.assignments.into_iter().collect::<BTreeMap<_, _>>();
            rule.assignments = source_rule.assignments.iter().map(|assignment| {
                let target = slot(&assignment.target)?;
                Ok((target, assignments.remove(&target).ok_or(ExecutableErrorV1::MalformedProgram)?))
            }).collect::<Result<_, ExecutableErrorV1>>()?;
            if !assignments.is_empty() { return Err(ExecutableErrorV1::MalformedProgram); }
            rule.required_present = source_rule.required_present.iter().map(slot).collect::<Result<_, _>>()?;
            rule.required_absent = source_rule.required_absent.iter().map(slot).collect::<Result<_, _>>()?;
            rule.removals = source_rule.removals.iter().map(slot).collect::<Result<_, _>>()?;
            retained.insert((handler.id, index), rule);
        }
    }
    Ok(retained)
}

// Physical slots remain fixed, while semantic constants still belong to the
// newly checked allocation root. Reuse never preserves an old semantic address.
fn rebind_lowered_expression(value: &mut ExecutableExpressionV1, edit: &CanonicalSourceEditV1, member_sets: &mut BTreeMap<Vec<u32>, Vec<u32>>) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    let formation = |old| edit.formation(FormationLocalId::new(old)).map(|new| new.get())
        .map_err(|_| ExecutableErrorV1::MalformedProgram);
    match value {
        E::Match { value, cases } => {
            rebind_lowered_expression(value, edit, member_sets)?;
            for (_, _, body) in cases { rebind_lowered_expression(body, edit, member_sets)?; }
        }
        E::Constant(value) => *value = migrate_value(value, edit)?,
        E::Slot(_) | E::Argument(_) | E::Binding(_) => {},
        E::Sequence(values) | E::Foreign { arguments: values, .. } => for value in values {
            rebind_lowered_expression(value, edit, member_sets)?;
        },
        E::Record(fields) => for value in fields.values_mut() { rebind_lowered_expression(value, edit, member_sets)?; },
        E::Let { value, body, .. } | E::SequenceMap { source: value, body, .. } => {
            rebind_lowered_expression(value, edit, member_sets)?; rebind_lowered_expression(body, edit, member_sets)?;
        },
        E::SequenceFold { source, initial, body, .. } => {
            rebind_lowered_expression(source, edit, member_sets)?;
            rebind_lowered_expression(initial, edit, member_sets)?;
            rebind_lowered_expression(body, edit, member_sets)?;
        },
        E::SequenceCount(value) | E::SequenceSort(value) | E::ScalarText(value) | E::Field(value, _) => rebind_lowered_expression(value, edit, member_sets)?,
        E::SequenceDrop(a, b) | E::SequenceJoin(a, b) | E::Dictionary(a, b) | E::SequenceAppend(a, b) => {
            rebind_lowered_expression(a, edit, member_sets)?; rebind_lowered_expression(b, edit, member_sets)?;
        },
        E::Require(a, b, c) => {
            rebind_lowered_expression(a, edit, member_sets)?; rebind_lowered_expression(b, edit, member_sets)?; rebind_lowered_expression(c, edit, member_sets)?;
        },
        E::FreshReferent { domain, .. } => *domain = formation(*domain)?,
        E::ReferentFacet { value, domain, members } => {
            rebind_lowered_expression(value, edit, member_sets)?;
            *domain = formation(*domain)?;
            // Reuse only an exact member vector successfully translated under
            // this checked edit; equal domains alone do not imply equal facets.
            if let Some(rebound) = member_sets.get(members.as_slice()) {
                members.clone_from(rebound);
            } else {
                let previous = members.clone();
                for member in members.iter_mut() { *member = formation(*member)?; }
                members.sort();
                member_sets.insert(previous, members.clone());
            }
        }
        E::Sum { inputs, predicates, value } => {
            for input in inputs.iter_mut().chain(predicates) { rebind_lowered_expression(input, edit, member_sets)?; }
            rebind_lowered_expression(value, edit, member_sets)?;
        }
        E::RelationEffects(effects) | E::DerivedRelation(effects) => for effect in effects {
            use ExecutableRelationEffectV1 as R;
            let (a, b) = match effect { R::Put(a,b) | R::Insert(a,b) | R::Remove(a,b) | R::Accumulate(a,b) => (a,b) };
            rebind_lowered_expression(a, edit, member_sets)?; rebind_lowered_expression(b, edit, member_sets)?;
        },
        E::TextTransform(_, a) | E::SquareRoot(a) | E::Accumulate(a) | E::Not(a) => rebind_lowered_expression(a, edit, member_sets)?,
        E::Conditional(a,b,c) | E::RelationPut(a,b,c) | E::RelationInsert(a,b,c) | E::RelationRemoveValue(a,b,c) | E::Clamp(a,b,c) => {
            rebind_lowered_expression(a, edit, member_sets)?; rebind_lowered_expression(b, edit, member_sets)?; rebind_lowered_expression(c, edit, member_sets)?;
        }
        E::ContainsText(a,b) | E::StartsWith(a,b) | E::RelationMatch(_,a,b) | E::RelationRead(a,b) | E::RelationPresent(a,b)
        | E::RelationRemoveRow(a,b) | E::Concatenate(a,b) | E::Add(a,b) | E::Subtract(a,b) | E::Multiply(a,b) | E::Divide(a,b)
        | E::GreaterThan(a,b) | E::LessThanOrEqual(a,b) | E::Equal(a,b) | E::And(a,b) | E::SetInsert(a,b) | E::SetContains(a,b) | E::SetRemove(a,b) => {
            rebind_lowered_expression(a, edit, member_sets)?; rebind_lowered_expression(b, edit, member_sets)?;
        }
    }
    Ok(())
}

fn migrate_value(
    value: &ExecutableValueV1,
    edit: &CanonicalSourceEditV1,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    let formation = |old| {
        edit.formation(FormationLocalId::new(old))
            .map(|new| new.get())
            .map_err(|_| ExecutableErrorV1::MalformedProgram)
    };
    let referent = |old: &ExecutableReferentV1| {
        Ok::<_, ExecutableErrorV1>(ExecutableReferentV1 {
            domain: formation(old.domain)?,
            identity: match old.identity {
                ExecutableReferentIdentityV1::Declared(id) => {
                    ExecutableReferentIdentityV1::Declared(formation(id)?)
                }
                // Runtime creation is a continuing occurrence, not a new source address.
                ExecutableReferentIdentityV1::Created(id) => {
                    ExecutableReferentIdentityV1::Created(id)
                }
            },
        })
    };
    Ok(match value {
        ExecutableValueV1::Referent(value) => ExecutableValueV1::Referent(referent(value)?),
        ExecutableValueV1::Sequence(values) => ExecutableValueV1::Sequence(values.iter()
            .map(|value| migrate_value(value, edit)).collect::<Result<_, _>>()?),
        ExecutableValueV1::Record(fields) => ExecutableValueV1::Record(fields.iter()
            .map(|(name, value)| Ok((name.clone(), migrate_value(value, edit)?)))
            .collect::<Result<_, ExecutableErrorV1>>()?),
        ExecutableValueV1::Set(set) => ExecutableValueV1::Set(ExecutableSetV1 {
            element_kind: set.element_kind,
            values: set
                .values
                .iter()
                .map(|value| migrate_value(value, edit))
                .collect::<Result<_, _>>()?,
        }),
        ExecutableValueV1::RelationTable(table) => {
            ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
                subject_domain: formation(table.subject_domain)?,
                value_kind: table.value_kind,
                value_domain: table.value_domain.map(formation).transpose()?,
                cardinality: table.cardinality,
                total: table.total,
                rows: Arc::new(table
                    .rows
                    .iter()
                    .map(|(key, values)| {
                        Ok((
                            referent(key)?,
                            values
                                .iter()
                                .map(|value| migrate_value(value, edit))
                                .collect::<Result<_, ExecutableErrorV1>>()?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, ExecutableErrorV1>>()?.into()),
            })
        }
        _ => value.clone(),
    })
}

impl ExecutableProcessRuntimeV1 {
    pub(crate) fn source_continuity(&self) -> Result<&ExecutableSourceContinuityV1, ExecutableErrorV1> {
        self.source_continuity.as_ref().ok_or(ExecutableErrorV1::SourceContinuityRejected(
            "no explicit source transition"))
    }

    pub fn source_continuity_term(&self) -> Result<Term, ExecutableErrorV1> {
        let continuity = self.source_continuity()?;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        projection_object(
            scope,
            vec![
                (
                    b"old-snapshot".to_vec(),
                    diagnostic_text(scope, &hex_identity(continuity.old_snapshot.as_bytes()))?,
                ),
                (
                    b"new-snapshot".to_vec(),
                    diagnostic_text(scope, &hex_identity(continuity.new_snapshot.as_bytes()))?,
                ),
                (
                    b"formations".to_vec(),
                    diagnostic_index(
                        scope,
                        continuity
                            .occurrences
                            .iter()
                            .enumerate()
                            .map(|(index, (old, new, first_snapshot, first, occurrence))| {
                                Ok((
                                    index.to_string().into_bytes(),
                                    projection_object(
                                        scope,
                                        vec![
                                            (
                                                b"old".to_vec(),
                                                diagnostic_number(scope, old.get() as f64)?,
                                            ),
                                            (
                                                b"new".to_vec(),
                                                diagnostic_number(scope, new.get() as f64)?,
                                            ),
                                            (
                                                b"occurrence-snapshot".to_vec(),
                                                diagnostic_text(
                                                    scope,
                                                    &hex_identity(first_snapshot.as_bytes()),
                                                )?,
                                            ),
                                            (
                                                b"occurrence-coordinate".to_vec(),
                                                diagnostic_number(scope, first.get() as f64)?,
                                            ),
                                            (
                                                b"occurrence".to_vec(),
                                                diagnostic_text(scope, &hex_identity(occurrence))?,
                                            ),
                                        ],
                                    )?,
                                ))
                            })
                            .collect::<Result<_, ExecutableErrorV1>>()?,
                    )?,
                ),
                (
                    b"slots".to_vec(),
                    diagnostic_index(
                        scope,
                        continuity
                            .slots
                            .iter()
                            .map(|(old, new)| {
                                Ok((
                                    old.to_string().into_bytes(),
                                    diagnostic_number(scope, *new as f64)?,
                                ))
                            })
                            .collect::<Result<_, ExecutableErrorV1>>()?,
                    )?,
                ),
            ],
        )
    }

    pub(crate) fn initialize_source_continuity(
        &mut self,
        previous: &Self,
        checked: &CheckedExecutableSourceEditV1,
    ) -> Result<(), ExecutableErrorV1> {
        let _profile = source_profile_scope_v1(SourceProfilePhaseV1::Migration);
        if previous.candidate.is_some() {
            return Err(ExecutableErrorV1::SourceContinuityRejected(
                "settle hidden candidate before changed source edit",
            ));
        }
        if previous.physical_plan != checked.old_plan
            || self.physical_plan != checked.new_plan
            || self.last_step.is_some()
            || self
                .carrier_execution
                .as_ref()
                .is_none_or(|execution| execution.state_started)
            || previous.suspended_continuation.is_some()
            || previous.pending_effect_intent.is_some()
            || previous.active_effect_attempt.is_some()
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let mut next = self.configuration.clone();
        if previous.configuration.len() != checked.continuity.slots.len()
            || next.len() != checked.continuity.slots.len()
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        for (old, new) in &checked.continuity.slots {
            let old = previous
                .configuration
                .get(usize::from(*old))
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
            let new = next
                .get_mut(usize::from(*new))
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
            if old.kind() != new.kind() {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
            *new = match old {
                ExecutableSlotV1::Absent(kind) => ExecutableSlotV1::Absent(*kind),
                ExecutableSlotV1::Present(value) => migrate_value(value, &checked.edit)?.into(),
            };
        }
        // This fresh execution generation has not entered its Activation yet.
        // The first real carrier ingress will assert this checked carried
        // configuration as its initial configuration. No host configuration
        // import, candidate admission, or speculative Step is manufactured.
        let mut continuity = checked.continuity.clone();
        for (old, new) in &continuity.identities {
            let (
                CanonicalAllocatedIdentityV1::Formation(old),
                CanonicalAllocatedIdentityV1::Formation(new),
            ) = (old, new)
            else {
                continue;
            };
            let (first_snapshot, first, occurrence) = previous
                .source_continuity
                .as_ref()
                .and_then(|prior| {
                    prior
                        .occurrences
                        .iter()
                        .find(|(_, current, _, _, _)| current == old)
                })
                .map(|(_, _, first_snapshot, first, occurrence)| {
                    (*first_snapshot, *first, *occurrence)
                })
                .unwrap_or_else(|| {
                    (
                        continuity.old_snapshot,
                        *old,
                        runtime_domain_hash(
                            "clause/continuing-source-occurrence/v1",
                            &[&previous.allocation.root, &old.get().to_be_bytes()],
                        ),
                    )
                });
            continuity
                .occurrences
                .push((*old, *new, first_snapshot, first, occurrence));
        }
        self.configuration = next;
        self.source_continuity = Some(continuity);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CET3_FIXED_BYTES: usize =
        4 + 2 * IDENTITY_BYTES + 3 * size_of::<u32>() + 6 * size_of::<u32>();

    fn source_edit_with_old_source(old_source: Vec<u8>) -> ExecutableSourceEditV1 {
        ExecutableSourceEditV1 {
            old_source,
            declared_frontend: Vec::new(),
            imports: CanonicalSourceImportsV1::new(),
            old_root: ProgramChangeOccurrenceId::from_bytes([1; IDENTITY_BYTES]),
            new_root: ProgramChangeOccurrenceId::from_bytes([2; IDENTITY_BYTES]),
            operation: ExecutableSourceOperationV1::ScalarEffect {
                handler: FormationLocalId::new(1),
                effect: FormationLocalId::new(2),
                field_path: vec![],
                expression: Vec::new(),
            },
            old_cpp1: Vec::new(),
            new_cpp1: Vec::new(),
        }
    }

    #[test]
    fn source_edit_codec_accepts_exact_aggregate_limit_and_rejects_limit_plus_one() {
        let exact = source_edit_with_old_source(vec![
            0;
            EXECUTABLE_SOURCE_EDIT_LIMIT_V1
                - CET3_FIXED_BYTES
        ]);
        let encoded = encode_executable_source_edit_v1(&exact).unwrap();
        assert_eq!(encoded.len(), EXECUTABLE_SOURCE_EDIT_LIMIT_V1);
        assert_eq!(decode_executable_source_edit_v1(&encoded).unwrap(), exact);

        let oversized =
            source_edit_with_old_source(vec![
                0;
                EXECUTABLE_SOURCE_EDIT_LIMIT_V1 - CET3_FIXED_BYTES + 1
            ]);
        assert_eq!(
            encode_executable_source_edit_v1(&oversized),
            Err(ExecutableErrorV1::ResourceLimit),
        );
        let mut encoded_oversized = encoded;
        encoded_oversized.push(0);
        assert_eq!(
            decode_executable_source_edit_v1(&encoded_oversized),
            Err(ExecutableErrorV1::ResourceLimit),
        );
    }
}
