//! Compiler-checked source transitions applied to runtime-owned live state.
use super::*;
use clause_package::{
    CanonicalAllocatedIdentityV1, CanonicalScalarEditV1, CanonicalSourceContextV1,
    ProgramChangeOccurrenceId, canonical_scalar_effects_v1, elaborate_canonical_source_package_v1,
    plan_independent_canonical_source_allocations_v1, read_canonical_source_v1,
    replace_canonical_scalar_effect_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableSourceEditV1 {
    pub old_source: Vec<u8>,
    pub old_root: ProgramChangeOccurrenceId,
    pub new_root: ProgramChangeOccurrenceId,
    pub handler: FormationLocalId,
    pub effect: FormationLocalId,
    pub expression: Vec<u8>,
    pub old_cpp1: Vec<u8>,
    pub new_cpp1: Vec<u8>,
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
    old_plan: ExecutablePhysicalPlanIdV1,
    new_plan: ExecutablePhysicalPlanIdV1,
    edit: CanonicalScalarEditV1,
    continuity: ExecutableSourceContinuityV1,
}

impl CheckedExecutableSourceEditV1 {
    pub fn continuity(&self) -> &ExecutableSourceContinuityV1 {
        &self.continuity
    }
}

impl ExecutablePhysicalPlanV1 {
    pub fn bind_source_snapshot(
        &mut self,
        scope: TermScope,
        package: &clause_package::CanonicalSourcePackageSliceV1,
        artifact: clause_package::CanonicalSourceArtifactIdV1,
        root: ProgramChangeOccurrenceId,
    ) -> Result<(), ExecutableErrorV1> {
        let _profile = source_profile_scope_v1(SourceProfilePhaseV1::SnapshotMetadata);
        self.project_source_rows(scope, package)?;
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
        self.source_metadata = Some(source_metadata(scope, package, artifact, &self.program)?);
        Ok(())
    }
}

fn source_metadata(
    scope: TermScope,
    package: &clause_package::CanonicalSourcePackageSliceV1,
    artifact: clause_package::CanonicalSourceArtifactIdV1,
    program: &ExecutableProgramV1,
) -> Result<Term, ExecutableErrorV1> {
    let roles = program
        .projection
        .as_ref()
        .ok_or(ExecutableErrorV1::MalformedProgram)?
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
    let states = lowered
        .states
        .iter()
        .map(|binding| {
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
            Ok((
                binding.slot.to_string().into_bytes(),
                projection_object(scope, fields)?,
            ))
        })
        .collect::<Result<_, ExecutableErrorV1>>()?;
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

pub fn encode_executable_source_edit_v1(
    edit: &ExecutableSourceEditV1,
) -> Result<Vec<u8>, ExecutableErrorV1> {
    let mut bytes = b"CET1".to_vec();
    bytes.extend_from_slice(edit.old_root.as_bytes());
    bytes.extend_from_slice(edit.new_root.as_bytes());
    bytes.extend_from_slice(&edit.handler.get().to_le_bytes());
    bytes.extend_from_slice(&edit.effect.get().to_le_bytes());
    for blob in [
        &edit.old_source,
        &edit.expression,
        &edit.old_cpp1,
        &edit.new_cpp1,
    ] {
        bytes.extend_from_slice(
            &u32::try_from(blob.len())
                .map_err(|_| ExecutableErrorV1::ResourceLimit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(blob);
    }
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    Ok(bytes)
}

pub fn decode_executable_source_edit_v1(
    bytes: &[u8],
) -> Result<ExecutableSourceEditV1, ExecutableErrorV1> {
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != b"CET1" {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let old_root = ProgramChangeOccurrenceId::from_bytes(d.identity()?);
    let new_root = ProgramChangeOccurrenceId::from_bytes(d.identity()?);
    let handler = FormationLocalId::new(d.u32()?);
    let effect = FormationLocalId::new(d.u32()?);
    let mut blob = || {
        let len = d.u32()? as usize;
        Ok::<_, ExecutableErrorV1>(d.take(len)?.to_vec())
    };
    let result = ExecutableSourceEditV1 {
        old_root,
        new_root,
        handler,
        effect,
        old_source: blob()?,
        expression: blob()?,
        old_cpp1: blob()?,
        new_cpp1: blob()?,
    };
    if !d.is_complete() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(result)
}

pub fn check_executable_source_edit_v1(
    witness: &ExecutableSourceEditV1,
    scope: TermScope,
) -> Result<CheckedExecutableSourceEditV1, ExecutableErrorV1> {
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::WitnessCheck);
    let rejected = |_| ExecutableErrorV1::MalformedProgram;
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::SourceRead);
    let old_cst = read_canonical_source_v1(&witness.old_source).map_err(rejected)?;
    drop(phase);
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::Allocation);
    let old_allocations =
        plan_independent_canonical_source_allocations_v1(&old_cst, witness.old_root)
            .map_err(rejected)?;
    drop(phase);
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::OfferedEdit);
    let offered = canonical_scalar_effects_v1(&old_cst, &old_allocations).map_err(rejected)?;
    let selected = offered
        .iter()
        .find(|effect| effect.handler == witness.handler && effect.effect == witness.effect)
        .ok_or(ExecutableErrorV1::MalformedProgram)?;
    if selected.expression == witness.expression {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let edit = replace_canonical_scalar_effect_v1(
        &old_cst,
        &old_allocations,
        selected,
        &witness.expression,
        witness.new_root,
    )
    .map_err(rejected)?;
    drop(phase);
    let context = CanonicalSourceContextV1 {
        universe: scope.universe,
        semantics: scope.semantics,
    };
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::OldElaboration);
    let old = elaborate_canonical_source_package_v1(&old_cst, context, &old_allocations)
        .map_err(rejected)?;
    drop(phase);
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::NewElaboration);
    let new = elaborate_canonical_source_package_v1(edit.source(), context, edit.plan())
        .map_err(rejected)?;
    drop(phase);
    let phase = source_profile_scope_v1(SourceProfilePhaseV1::Cpp1Decode);
    let old_plan = decode_executable_physical_plan_v1(&witness.old_cpp1)?;
    let new_plan = decode_executable_physical_plan_v1(&witness.new_cpp1)?;
    drop(phase);
    let roles = old_plan
        .program
        .projection
        .as_ref()
        .ok_or(ExecutableErrorV1::MalformedProgram)?
        .bindings
        .iter()
        .map(|binding| binding.role)
        .collect::<Vec<_>>();
    let old_lowered = lower_canonical_executable_program_v1(
        scope,
        &old.state_cells,
        &old.executable_handlers,
        &roles,
    )?;
    let new_lowered = lower_canonical_executable_program_v1(
        scope,
        &new.state_cells,
        &new.executable_handlers,
        &roles,
    )?;
    let mut expected_old = old_plan.clone();
    expected_old.program = old_lowered.program.clone();
    let mut expected_new = old_plan.clone();
    expected_new.program = new_lowered.program.clone();
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
    if old_plan.program.rules.len() == expected_old.program.rules.len() + 1 {
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
        let new_entry = u16::try_from(new_lowered.handlers.len())
            .map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        entries.insert(checkpoint.entry, new_entry);
        expected_old.program.rules.push(checkpoint.clone());
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
    }
    expected_old.project_referent_input_domains(scope)?;
    expected_new.project_referent_input_domains(scope)?;
    expected_old.bind_source_snapshot(scope, &old, old_cst.artifact(), witness.old_root)?;
    expected_new.bind_source_snapshot(scope, &new, edit.source().artifact(), witness.new_root)?;
    let _compare = source_profile_scope_v1(SourceProfilePhaseV1::CompareAndMap);
    if expected_old != old_plan {
        return Err(ExecutableErrorV1::SourceContinuityRejected(
            "old source does not realize exact bound CPP1",
        ));
    }
    if expected_new != new_plan {
        return Err(ExecutableErrorV1::SourceContinuityRejected(
            "edited source does not realize exact replacement CPP1",
        ));
    }
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
    Ok(CheckedExecutableSourceEditV1 {
        old_plan: physical_plan_identity(&witness.old_cpp1),
        new_plan: physical_plan_identity(&witness.new_cpp1),
        continuity: ExecutableSourceContinuityV1 {
            old_snapshot: old.checked_package.constitution().snapshot(),
            new_snapshot: new.checked_package.constitution().snapshot(),
            identities: edit.retained().clone(),
            slots,
            occurrences: vec![],
        },
        edit,
    })
}

fn physical_plan_identity(bytes: &[u8]) -> ExecutablePhysicalPlanIdV1 {
    ExecutablePhysicalPlanIdV1::from_bytes(runtime_domain_hash(
        "clause/executable-physical-plan/v1",
        &[bytes],
    ))
}

fn migrate_value(
    value: &ExecutableValueV1,
    edit: &CanonicalScalarEditV1,
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
                rows: table
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
                    .collect::<Result<_, ExecutableErrorV1>>()?,
            })
        }
        _ => value.clone(),
    })
}

impl ExecutableProcessRuntimeV1 {
    pub fn source_continuity_term(&self) -> Result<Term, ExecutableErrorV1> {
        let continuity =
            self.source_continuity
                .as_ref()
                .ok_or(ExecutableErrorV1::SourceContinuityRejected(
                    "no explicit source transition",
                ))?;
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
