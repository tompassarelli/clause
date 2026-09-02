use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use clause_package::{
    CandidateDeltaId, CanonicalHandlerTriggerV1, CanonicalKeyPhaseV1, CanonicalKeyboardBindingV1,
    CanonicalScalarInputBindingV1, CanonicalSourceContextV1, CanonicalUnsupportedProductionV1,
    LocalRoleRefV2, ProcessPackageId, ProgramChangeOccurrenceId, StateRevisionId, TermScope,
    check_process_package, decode_process_package, elaborate_canonical_source_package_v1,
    plan_independent_canonical_source_allocations_v1, read_canonical_source_v1,
};
use clause_runtime::{
    ExecutableCanonicalHandlerBindingV1, ExecutableInputBindingV1, ExecutableInputPlanV1,
    ExecutableInputSourceV1, ExecutableKeyPhaseV1, ExecutableOccurrenceV1,
    ExecutablePhysicalPlanV1, ExecutableTickBindingV1, ExecutableValueV1,
    WASM_SESSION_EVENT_LIMIT_V1, WasmPersistentSessionBoundaryV1, WasmProcessRequestV1,
    WasmProcessStatusV1, WasmSessionAdmissionScopeV1, WasmSessionAdmissionV1,
    WasmSessionAllocationV1, WasmSessionCommandV1, WasmSessionEventKindV1, WasmSessionHandleV1,
    WasmSessionLimitsV1, WasmSessionOpenV1, WasmSessionOperationV1, WasmSessionProjectionV1,
    decode_executable_occurrence_v1, decode_executable_physical_plan_v1,
    decode_wasm_process_request_v1, encode_executable_occurrence_v1,
    encode_executable_physical_plan_v1, encode_wasm_process_request_v1,
    encode_wasm_session_command_v1, encode_wasm_session_open_v1,
    lower_canonical_executable_program_v1,
};

const COHERENT_TEMPLATE_CWR1_HEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../browser/jump-arena-shell/fixtures/wasm-coherent-game-v1/coherent-game-v1.cwr1.hex"
));
const MAX_COMMANDS: u64 = 4_096;
const SOURCE_AUTHORITY_BUDGET_UNITS: u64 = 1_000_000;

#[derive(Debug)]
pub struct ResidentSourceWorkbenchErrorV1(String);

impl fmt::Display for ResidentSourceWorkbenchErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ResidentSourceWorkbenchErrorV1 {}

impl From<WasmProcessStatusV1> for ResidentSourceWorkbenchErrorV1 {
    fn from(error: WasmProcessStatusV1) -> Self {
        Self(format!(
            "resident source boundary rejected the request: {error}"
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidentSourceGenerationV1 {
    pub handle: WasmSessionHandleV1,
    pub source_package: ProcessPackageId,
    pub cpp1: Vec<u8>,
    pub cwr1: Vec<u8>,
    pub unsupported: Vec<CanonicalUnsupportedProductionV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResidentSourceCandidateV1 {
    pub handle: WasmSessionHandleV1,
    pub base: StateRevisionId,
    pub candidate: CandidateDeltaId,
    pub state_revision_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidentSourceAdmissionV1 {
    pub handle: WasmSessionHandleV1,
    pub predecessor: StateRevisionId,
    pub successor: StateRevisionId,
    pub state_revision_count: u32,
    pub projection: WasmSessionProjectionV1,
}

/// One process-resident canonical-source compiler and persistent execution
/// boundary. The checked CWR1 template owns physical input/projection data;
/// source elaboration replaces every executable rule and gameplay value.
pub struct ResidentSourceWorkbenchV1 {
    boundary: WasmPersistentSessionBoundaryV1,
    template: WasmProcessRequestV1,
    coherent_template: WasmProcessRequestV1,
    template_scope: TermScope,
    template_projection_roles: Vec<LocalRoleRefV2>,
    template_physical_plan: ExecutablePhysicalPlanV1,
    generation: ResidentSourceGenerationV1,
    package: ProcessPackageId,
    session: clause_package::RuntimeSessionId,
    sequence: u64,
    pending: Option<ResidentSourceCandidateV1>,
    last_projection: Option<WasmSessionProjectionV1>,
    next_change: u64,
    default_occurrences: Vec<Vec<u8>>,
    handlers: BTreeMap<Vec<u8>, Vec<ExecutableCanonicalHandlerBindingV1>>,
}

impl ResidentSourceWorkbenchV1 {
    pub fn open(exact_source: &[u8]) -> Result<Self, ResidentSourceWorkbenchErrorV1> {
        let coherent_template =
            decode_wasm_process_request_v1(&decode_hex(COHERENT_TEMPLATE_CWR1_HEX)?)?;
        let template = coherent_template.clone();
        let decoded = decode_process_package(&coherent_template.package_bytes)
            .map_err(|error| boxed_error("template package decode", error))?;
        let checked_template = check_process_package(decoded)
            .map_err(|error| boxed_error("template package check", error))?;
        let template_scope = TermScope {
            universe: checked_template.constitution().universe(),
            semantics: checked_template.constitution().semantics(),
        };
        let mut template_projection_roles = checked_template
            .constitution()
            .preimage()
            .schemas
            .iter()
            .flat_map(|schema| {
                schema.roles.iter().map(|role| LocalRoleRefV2 {
                    schema: schema.id,
                    role: role.id,
                })
            })
            .collect::<Vec<_>>();
        template_projection_roles.sort();
        let template_physical_plan =
            decode_executable_physical_plan_v1(&coherent_template.physical_plan_bytes)
                .map_err(|error| boxed_error("CPP1 template decode", error))?;
        let mut workbench = Self {
            boundary: WasmPersistentSessionBoundaryV1::new(),
            generation: ResidentSourceGenerationV1 {
                handle: WasmSessionHandleV1 {
                    slot: 0,
                    generation: 0,
                },
                source_package: ProcessPackageId::from_bytes([0; clause_package::IDENTITY_BYTES]),
                cpp1: Vec::new(),
                cwr1: Vec::new(),
                unsupported: Vec::new(),
            },
            package: ProcessPackageId::from_bytes([0; clause_package::IDENTITY_BYTES]),
            session: template.authority.session,
            template,
            coherent_template,
            template_scope,
            template_projection_roles,
            template_physical_plan,
            sequence: 0,
            pending: None,
            last_projection: None,
            next_change: 0,
            default_occurrences: Vec::new(),
            handlers: BTreeMap::new(),
        };
        workbench.install_source(exact_source)?;
        Ok(workbench)
    }

    #[must_use]
    pub fn generation(&self) -> &ResidentSourceGenerationV1 {
        &self.generation
    }

    #[must_use]
    pub const fn pending_candidate(&self) -> Option<ResidentSourceCandidateV1> {
        self.pending
    }

    #[must_use]
    pub fn last_projection(&self) -> Option<&WasmSessionProjectionV1> {
        self.last_projection.as_ref()
    }

    /// Encode one exact caller-selected handler occurrence by its checked
    /// source designation. Trigger metadata controls scheduling; the
    /// designation always selects source semantics, never a Rust
    /// implementation.
    pub fn handler_occurrence(
        &self,
        designation: &[u8],
        arguments: &[ExecutableValueV1],
    ) -> Result<Vec<u8>, ResidentSourceWorkbenchErrorV1> {
        let matching = self
            .handlers
            .get(designation)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let [binding] = matching.as_slice() else {
            return Err(ResidentSourceWorkbenchErrorV1(format!(
                "source handler designation is missing or ambiguous: {}",
                String::from_utf8_lossy(designation)
            )));
        };
        if usize::from(binding.argument_count) != arguments.len() {
            return Err(ResidentSourceWorkbenchErrorV1(format!(
                "source handler {} requires {} arguments, received {}",
                String::from_utf8_lossy(designation),
                binding.argument_count,
                arguments.len()
            )));
        }
        encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
            entry: binding.entry,
            arguments: arguments.to_vec(),
        })
        .map_err(|error| boxed_error("source handler occurrence encode", error))
    }

    /// Encode the exact checked fixed-tick chain. Source `on tick` roots
    /// precede automatic reactions; identities break ties within each class.
    pub fn fixed_tick_occurrences(
        &self,
        delta_seconds: f64,
    ) -> Result<Vec<Vec<u8>>, ResidentSourceWorkbenchErrorV1> {
        let delta = ExecutableValueV1::number(delta_seconds)
            .map_err(|error| boxed_error("fixed tick value", error))?;
        let mut bindings = self
            .handlers
            .values()
            .flatten()
            .filter(|binding| {
                matches!(
                    binding.trigger,
                    CanonicalHandlerTriggerV1::FixedTickRoot
                        | CanonicalHandlerTriggerV1::FixedTickDerived
                        | CanonicalHandlerTriggerV1::FixedTick
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        bindings.sort_by_key(|binding| (fixed_tick_rank(binding.trigger), binding.handler));
        bindings
            .into_iter()
            .map(|binding| {
                encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
                    entry: binding.entry,
                    arguments: (binding.argument_count == 1)
                        .then_some(delta.clone())
                        .into_iter()
                        .collect(),
                })
                .map_err(|error| boxed_error("fixed tick occurrence encode", error))
            })
            .collect()
    }

    /// Compile and atomically install a fresh generation in this same
    /// workbench process. The prior handle becomes stale only after the new
    /// checked package/CPP1 session opens successfully.
    pub fn hot_reload(
        &mut self,
        exact_source: &[u8],
    ) -> Result<ResidentSourceGenerationV1, ResidentSourceWorkbenchErrorV1> {
        self.install_source(exact_source)?;
        while self.boundary.reclaim_retired() {}
        Ok(self.generation.clone())
    }

    /// Run the template's opaque occurrence sequence and stop at a hidden
    /// CandidateDelta. This operation cannot create an Admission or successor.
    pub fn run_to_candidate(
        &mut self,
    ) -> Result<ResidentSourceCandidateV1, ResidentSourceWorkbenchErrorV1> {
        let occurrences = self.default_occurrences.clone();
        self.run_occurrences_to_candidate(&occurrences)
    }

    /// Run one caller-selected exact occurrence chain in the resident
    /// generation. Every prefix occurrence remains local; only the final
    /// occurrence emits the hidden CandidateDelta.
    pub fn run_occurrences_to_candidate(
        &mut self,
        occurrences: &[Vec<u8>],
    ) -> Result<ResidentSourceCandidateV1, ResidentSourceWorkbenchErrorV1> {
        if self.pending.is_some() {
            return Err(ResidentSourceWorkbenchErrorV1(
                "resident source generation already has a hidden candidate".into(),
            ));
        }
        let (last, prefix) = occurrences.split_last().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("resident run has no occurrence sequence".into())
        })?;
        for (index, occurrence) in prefix.iter().enumerate() {
            let event = self.command(WasmSessionOperationV1::Input(occurrence.clone()))?;
            if !matches!(event, WasmSessionEventKindV1::InputAccepted { .. }) {
                let entry = decode_executable_occurrence_v1(occurrence)
                    .map(|occurrence| occurrence.entry.to_string())
                    .unwrap_or_else(|_| "undecodable".into());
                let designation = entry
                    .parse::<u16>()
                    .ok()
                    .and_then(|entry| {
                        self.handlers.iter().find_map(|(designation, bindings)| {
                            bindings
                                .iter()
                                .any(|binding| binding.entry == entry)
                                .then_some(designation)
                        })
                    })
                    .map(|designation| String::from_utf8_lossy(designation).into_owned())
                    .unwrap_or_else(|| "unknown".into());
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "unexpected input event at prefix {index}, entry {entry} ({designation}): {event:?}"
                )));
            }
        }
        let event = self.command(WasmSessionOperationV1::Candidate(last.clone()))?;
        let WasmSessionEventKindV1::CandidateAccepted {
            candidate,
            base,
            state_revision_count,
            ..
        } = event
        else {
            return Err(unexpected_event("candidate", event));
        };
        let candidate = ResidentSourceCandidateV1 {
            handle: self.generation.handle,
            base,
            candidate,
            state_revision_count,
        };
        self.pending = Some(candidate);
        Ok(candidate)
    }

    /// Perform the separately commanded Admission for the exact hidden
    /// candidate and return only the admitted renderer projection.
    pub fn admit(&mut self) -> Result<ResidentSourceAdmissionV1, ResidentSourceWorkbenchErrorV1> {
        let candidate = self.pending.ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("resident source generation has no candidate".into())
        })?;
        let issued = self.command(WasmSessionOperationV1::IssueAdmission(
            WasmSessionAdmissionScopeV1 {
                package: self.package,
                session: self.session,
                base: candidate.base,
                candidate: candidate.candidate,
            },
        ))?;
        let WasmSessionEventKindV1::AdmissionAuthorizationIssued { occurrence, .. } = issued else {
            return Err(unexpected_event("Admission authorization", issued));
        };
        let admitted = self.command(WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
            package: self.package,
            session: self.session,
            base: candidate.base,
            candidate: candidate.candidate,
            authorization: occurrence,
        }))?;
        let WasmSessionEventKindV1::AdmissionAccepted {
            predecessor,
            successor,
            state_revision_count,
            projection: Some(projection),
            ..
        } = admitted
        else {
            return Err(unexpected_event("Admission", admitted));
        };
        self.pending = None;
        self.last_projection = Some(projection.clone());
        Ok(ResidentSourceAdmissionV1 {
            handle: self.generation.handle,
            predecessor,
            successor,
            state_revision_count,
            projection,
        })
    }

    /// Exercise the real boundary's stale-generation fence without mutating
    /// the live generation.
    pub fn rejects_stale_handle(
        &mut self,
        handle: WasmSessionHandleV1,
    ) -> Result<bool, ResidentSourceWorkbenchErrorV1> {
        let bytes = encode_wasm_session_command_v1(&WasmSessionCommandV1 {
            handle,
            expected_sequence: 0,
            operation: WasmSessionOperationV1::Dispose,
        })?;
        match self.boundary.command(&bytes) {
            Err(WasmProcessStatusV1::StaleSessionHandle) => Ok(true),
            Err(error) => Err(error.into()),
            Ok(_) => Ok(false),
        }
    }

    fn install_source(
        &mut self,
        exact_source: &[u8],
    ) -> Result<(), ResidentSourceWorkbenchErrorV1> {
        self.next_change = self.next_change.checked_add(1).ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("source change sequence exhausted".into())
        })?;
        let cst = read_canonical_source_v1(exact_source)
            .map_err(|error| debug_error("canonical source read", error))?;
        let allocation_plan = plan_independent_canonical_source_allocations_v1(
            &cst,
            ProgramChangeOccurrenceId::from_bytes(sequence_id(self.next_change)),
        )
        .map_err(|error| debug_error("canonical source allocation", error))?;
        let scope = self.template_scope;
        let compiled = elaborate_canonical_source_package_v1(
            &cst,
            CanonicalSourceContextV1 {
                universe: scope.universe,
                semantics: scope.semantics,
            },
            &allocation_plan,
        )
        .map_err(|error| debug_error("canonical source elaboration", error))?;
        self.template = self.coherent_template.clone();
        self.template.authority.budget_units = SOURCE_AUTHORITY_BUDGET_UNITS;
        let mut physical_plan = self.template_physical_plan.clone();
        let projection_roles = &self.template_projection_roles;
        let projected_state_count = compiled.state_cells.len();
        if projection_roles.len() < projected_state_count {
            return Err(ResidentSourceWorkbenchErrorV1(format!(
                "selected package has {} projection Roles for {} source state cells",
                projection_roles.len(),
                projected_state_count
            )));
        }
        let template_input = physical_plan.input.clone();
        let lowered = lower_canonical_executable_program_v1(
            scope,
            &compiled.state_cells,
            &compiled.executable_handlers,
            projection_roles,
        )
        .map_err(|error| boxed_error("generic canonical lowering", error))?;
        let semantic_handlers = compiled
            .executable_handlers
            .iter()
            .map(|handler| (handler.id, handler))
            .collect::<BTreeMap<_, _>>();
        self.handlers.clear();
        for binding in &lowered.handlers {
            let source = semantic_handlers.get(&binding.handler).ok_or_else(|| {
                ResidentSourceWorkbenchErrorV1(
                    "generic lowering returned an unknown handler identity".into(),
                )
            })?;
            self.handlers
                .entry(source.designation.clone())
                .or_default()
                .push(binding.clone());
        }
        let declarative_only = lowered.states.is_empty() && lowered.handlers.is_empty();
        if !declarative_only {
            physical_plan.program = lowered.program;
        }

        let has_tick = lowered.handlers.iter().any(|binding| {
            matches!(
                binding.trigger,
                CanonicalHandlerTriggerV1::FixedTickRoot
                    | CanonicalHandlerTriggerV1::FixedTickDerived
                    | CanonicalHandlerTriggerV1::FixedTick
            )
        });
        let template_tick = self.template.occurrences.last().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("CWR1 template has no tick occurrence".into())
        })?;
        let template_tick = decode_executable_occurrence_v1(template_tick)
            .map_err(|error| boxed_error("template tick decode", error))?;
        let mut default_occurrences = Vec::new();
        if declarative_only {
            default_occurrences.extend(self.template.occurrences.clone());
        } else if has_tick {
            let template_input = template_input.ok_or_else(|| {
                ResidentSourceWorkbenchErrorV1("CPP1 template has no physical input plan".into())
            })?;
            let events = if compiled.keyboard_bindings.is_empty()
                && compiled.scalar_input_bindings.is_empty()
            {
                bind_physical_events(
                    &template_input.events,
                    &lowered.handlers,
                    &semantic_handlers,
                )?
            } else {
                let mut events = bind_source_keyboard_events(
                    &compiled.keyboard_bindings,
                    &lowered.handlers,
                    &semantic_handlers,
                    projection_roles,
                    template_input.tick.role,
                )?;
                events.extend(bind_source_scalar_input_events(
                    &compiled.scalar_input_bindings,
                    compiled.keyboard_bindings.len(),
                    &lowered.handlers,
                    &semantic_handlers,
                    projection_roles,
                    template_input.tick.role,
                )?);
                events
            };
            if let Some(input) = events
                .iter()
                .find(|event| !event.occurrence.arguments.is_empty())
            {
                default_occurrences.push(
                    encode_executable_occurrence_v1(&input.occurrence)
                        .map_err(|error| boxed_error("default input occurrence encode", error))?,
                );
            }
            let tick_entries = ordered_tick_bindings(&lowered.handlers);
            for binding in &tick_entries {
                default_occurrences.push(
                    encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
                        entry: binding.entry,
                        arguments: (binding.argument_count == 1)
                            .then(|| template_tick.arguments.clone())
                            .unwrap_or_default(),
                    })
                    .map_err(|error| boxed_error("resident tick occurrence encode", error))?,
                );
            }
            physical_plan.input = Some(ExecutableInputPlanV1 {
                events,
                tick: ExecutableTickBindingV1 {
                    role: template_input.tick.role,
                    entries: tick_entries.iter().map(|binding| binding.entry).collect(),
                },
            });
        } else {
            physical_plan.input = None;
            let external = lowered
                .handlers
                .iter()
                .find(|binding| binding.trigger == CanonicalHandlerTriggerV1::External)
                .ok_or_else(|| {
                    ResidentSourceWorkbenchErrorV1(
                        "canonical source has no executable external handler".into(),
                    )
                })?;
            default_occurrences.push(
                encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
                    entry: external.entry,
                    arguments: Vec::new(),
                })
                .map_err(|error| boxed_error("default external occurrence encode", error))?,
            );
        }
        self.default_occurrences = default_occurrences;
        let cpp1 = encode_executable_physical_plan_v1(&physical_plan)
            .map_err(|error| boxed_error("CPP1 encode", error))?;
        let open = WasmSessionOpenV1 {
            package_bytes: self.template.package_bytes.clone(),
            application: self.template.application,
            physical_plan_bytes: cpp1.clone(),
            authority: self.template.authority.clone(),
            allocation: WasmSessionAllocationV1::New,
            limits: WasmSessionLimitsV1 {
                max_commands: MAX_COMMANDS,
                command_bytes: u32::try_from(clause_runtime::WASM_SESSION_COMMAND_LIMIT_V1)
                    .map_err(|_| ResidentSourceWorkbenchErrorV1("command limit overflow".into()))?,
                event_bytes: u32::try_from(WASM_SESSION_EVENT_LIMIT_V1)
                    .map_err(|_| ResidentSourceWorkbenchErrorV1("event limit overflow".into()))?,
                trace_retention: clause_runtime::WasmSessionTraceRetentionV1::FullUntilCommandLimit,
            },
        };
        let exact_open = encode_wasm_session_open_v1(&open)?;
        let opened = self.boundary.open(&exact_open)?;
        let WasmSessionEventKindV1::Opened {
            package: package_id,
            session,
            allocation,
            ..
        } = opened.kind
        else {
            return Err(unexpected_event("open", opened.kind));
        };
        let mut cwr1 = self.template.clone();
        cwr1.physical_plan_bytes = cpp1.clone();
        cwr1.allocation = allocation;
        cwr1.occurrences = self.default_occurrences.clone();
        let exact_cwr1 = encode_wasm_process_request_v1(&cwr1)?;
        self.generation = ResidentSourceGenerationV1 {
            handle: opened.handle,
            source_package: compiled.checked_package.id(),
            cpp1,
            cwr1: exact_cwr1,
            unsupported: compiled.unsupported.clone(),
        };
        self.package = package_id;
        self.session = session;
        self.sequence = 0;
        self.pending = None;
        self.last_projection = None;
        Ok(())
    }

    fn command(
        &mut self,
        operation: WasmSessionOperationV1,
    ) -> Result<WasmSessionEventKindV1, ResidentSourceWorkbenchErrorV1> {
        let bytes = encode_wasm_session_command_v1(&WasmSessionCommandV1 {
            handle: self.generation.handle,
            expected_sequence: self.sequence,
            operation,
        })?;
        let event = self.boundary.command(&bytes)?;
        self.sequence = event.accepted_sequence;
        Ok(event.kind)
    }
}

fn ordered_tick_bindings(
    handlers: &[ExecutableCanonicalHandlerBindingV1],
) -> Vec<ExecutableCanonicalHandlerBindingV1> {
    let mut tick = handlers
        .iter()
        .filter(|binding| {
            matches!(
                binding.trigger,
                CanonicalHandlerTriggerV1::FixedTickRoot
                    | CanonicalHandlerTriggerV1::FixedTickDerived
                    | CanonicalHandlerTriggerV1::FixedTick
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    tick.sort_by_key(|binding| (fixed_tick_rank(binding.trigger), binding.handler));
    tick
}

const fn fixed_tick_rank(trigger: CanonicalHandlerTriggerV1) -> u8 {
    match trigger {
        CanonicalHandlerTriggerV1::FixedTickRoot => 0,
        CanonicalHandlerTriggerV1::FixedTickDerived => 1,
        CanonicalHandlerTriggerV1::FixedTick => 2,
        CanonicalHandlerTriggerV1::External => 3,
    }
}

fn bind_physical_events(
    template: &[ExecutableInputBindingV1],
    bindings: &[ExecutableCanonicalHandlerBindingV1],
    semantic: &BTreeMap<
        clause_package::FormationLocalId,
        &clause_package::CanonicalExecutableHandlerV1,
    >,
) -> Result<Vec<ExecutableInputBindingV1>, ResidentSourceWorkbenchErrorV1> {
    let mut groups = BTreeMap::<u16, Vec<ExecutableInputBindingV1>>::new();
    for event in template {
        groups
            .entry(event.occurrence.entry)
            .or_default()
            .push(event.clone());
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    let mut external = bindings
        .iter()
        .filter(|binding| binding.trigger == CanonicalHandlerTriggerV1::External)
        .map(|binding| {
            let source = semantic.get(&binding.handler).ok_or_else(|| {
                ResidentSourceWorkbenchErrorV1(
                    "physical input binding names an unknown source handler".into(),
                )
            })?;
            Ok((binding, source.designation.as_slice()))
        })
        .collect::<Result<Vec<_>, ResidentSourceWorkbenchErrorV1>>()?;
    external.sort_by(|(left, left_name), (right, right_name)| {
        (left.argument_count, *left_name, left.handler).cmp(&(
            right.argument_count,
            *right_name,
            right.handler,
        ))
    });
    let mut events = Vec::new();
    for (binding, _) in external {
        let Some(group_index) = groups.iter().position(|(_, events)| {
            events.first().is_some_and(|event| {
                event.occurrence.arguments.len() == usize::from(binding.argument_count)
            }) && events.iter().all(|event| {
                event.occurrence.arguments.len() == usize::from(binding.argument_count)
            })
        }) else {
            continue;
        };
        let (_, group) = groups.remove(group_index);
        events.extend(group.into_iter().map(|mut event| {
            event.occurrence.entry = binding.entry;
            event
        }));
    }
    Ok(events)
}

fn bind_source_keyboard_events(
    source_bindings: &[CanonicalKeyboardBindingV1],
    bindings: &[ExecutableCanonicalHandlerBindingV1],
    semantic: &BTreeMap<
        clause_package::FormationLocalId,
        &clause_package::CanonicalExecutableHandlerV1,
    >,
    available_roles: &[LocalRoleRefV2],
    tick_role: LocalRoleRefV2,
) -> Result<Vec<ExecutableInputBindingV1>, ResidentSourceWorkbenchErrorV1> {
    let roles = available_roles
        .iter()
        .copied()
        .filter(|role| *role != tick_role)
        .take(source_bindings.len())
        .collect::<Vec<_>>();
    if roles.len() != source_bindings.len() {
        return Err(ResidentSourceWorkbenchErrorV1(format!(
            "physical adapter has {} event Roles for {} source keyboard bindings",
            roles.len(),
            source_bindings.len()
        )));
    }

    source_bindings
        .iter()
        .zip(roles)
        .map(|(source, role)| {
            let semantic_matches = semantic
                .iter()
                .filter(|(_, handler)| handler.designation == source.handler_designation)
                .collect::<Vec<_>>();
            let [(handler, meaning)] = semantic_matches.as_slice() else {
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "source keyboard binding target is missing or ambiguous: {}",
                    String::from_utf8_lossy(&source.handler_designation)
                )));
            };
            if meaning.trigger != CanonicalHandlerTriggerV1::External || meaning.argument_count != 0
            {
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "source keyboard binding target is not a zero-argument external handler: {}",
                    String::from_utf8_lossy(&source.handler_designation)
                )));
            }
            let lowered = bindings
                .iter()
                .find(|binding| binding.handler == **handler)
                .ok_or_else(|| {
                    ResidentSourceWorkbenchErrorV1(
                        "source keyboard binding target was not lowered".into(),
                    )
                })?;
            Ok(ExecutableInputBindingV1 {
                role,
                source: ExecutableInputSourceV1::Keyboard {
                    code: source.code.clone(),
                    phase: match source.phase {
                        CanonicalKeyPhaseV1::Down => ExecutableKeyPhaseV1::Down,
                        CanonicalKeyPhaseV1::Up => ExecutableKeyPhaseV1::Up,
                    },
                },
                occurrence: ExecutableOccurrenceV1 {
                    entry: lowered.entry,
                    arguments: Vec::new(),
                },
            })
        })
        .collect()
}

fn bind_source_scalar_input_events(
    source_bindings: &[CanonicalScalarInputBindingV1],
    role_offset: usize,
    bindings: &[ExecutableCanonicalHandlerBindingV1],
    semantic: &BTreeMap<
        clause_package::FormationLocalId,
        &clause_package::CanonicalExecutableHandlerV1,
    >,
    available_roles: &[LocalRoleRefV2],
    tick_role: LocalRoleRefV2,
) -> Result<Vec<ExecutableInputBindingV1>, ResidentSourceWorkbenchErrorV1> {
    let roles = available_roles
        .iter()
        .copied()
        .filter(|role| *role != tick_role)
        .skip(role_offset)
        .take(source_bindings.len())
        .collect::<Vec<_>>();
    if roles.len() != source_bindings.len() {
        return Err(ResidentSourceWorkbenchErrorV1(format!(
            "physical adapter has {} scalar event Roles after offset {} for {} source bindings",
            roles.len(),
            role_offset,
            source_bindings.len()
        )));
    }

    source_bindings
        .iter()
        .zip(roles)
        .map(|(source, role)| {
            let semantic_matches = semantic
                .iter()
                .filter(|(_, handler)| handler.designation == source.handler_designation)
                .collect::<Vec<_>>();
            let [(handler, meaning)] = semantic_matches.as_slice() else {
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "source scalar input target is missing or ambiguous: {}",
                    String::from_utf8_lossy(&source.handler_designation)
                )));
            };
            if meaning.trigger != CanonicalHandlerTriggerV1::External || meaning.argument_count != 1
            {
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "source scalar input target is not a one-argument external handler: {}",
                    String::from_utf8_lossy(&source.handler_designation)
                )));
            }
            let lowered = bindings
                .iter()
                .find(|binding| binding.handler == **handler)
                .ok_or_else(|| {
                    ResidentSourceWorkbenchErrorV1(
                        "source scalar input target was not lowered".into(),
                    )
                })?;
            Ok(ExecutableInputBindingV1 {
                role,
                source: ExecutableInputSourceV1::Scalar {
                    channel: source.channel.clone(),
                },
                occurrence: ExecutableOccurrenceV1 {
                    entry: lowered.entry,
                    arguments: vec![
                        ExecutableValueV1::number(0.0)
                            .map_err(|error| boxed_error("scalar input placeholder", error))?,
                    ],
                },
            })
        })
        .collect()
}

fn decode_hex(source: &str) -> Result<Vec<u8>, ResidentSourceWorkbenchErrorV1> {
    let digits = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if digits.len() % 2 != 0 {
        return Err(ResidentSourceWorkbenchErrorV1(
            "CWR1 template contains odd hex".into(),
        ));
    }
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = nibble(pair[0])?;
            let low = nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn nibble(byte: u8) -> Result<u8, ResidentSourceWorkbenchErrorV1> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(ResidentSourceWorkbenchErrorV1(
            "CWR1 template contains non-hex bytes".into(),
        )),
    }
}

fn sequence_id(sequence: u64) -> [u8; clause_package::IDENTITY_BYTES] {
    let mut identity = [0; clause_package::IDENTITY_BYTES];
    identity[..8].copy_from_slice(&sequence.to_be_bytes());
    identity[clause_package::IDENTITY_BYTES - 8..].copy_from_slice(&sequence.to_le_bytes());
    identity
}

fn boxed_error(stage: &str, error: impl Error) -> ResidentSourceWorkbenchErrorV1 {
    ResidentSourceWorkbenchErrorV1(format!("{stage} failed: {error}"))
}

fn debug_error(stage: &str, error: impl fmt::Debug) -> ResidentSourceWorkbenchErrorV1 {
    ResidentSourceWorkbenchErrorV1(format!("{stage} failed: {error:?}"))
}

fn unexpected_event(stage: &str, event: WasmSessionEventKindV1) -> ResidentSourceWorkbenchErrorV1 {
    ResidentSourceWorkbenchErrorV1(format!("unexpected {stage} event: {event:?}"))
}
