use std::error::Error;
use std::fmt;

use clause_package::{
    CandidateDeltaId, CanonicalSourceContextV1, ProcessPackageId, ProgramChangeOccurrenceId,
    StateRevisionId, TermScope, check_process_package, decode_process_package,
    elaborate_canonical_source_package_v1, plan_independent_canonical_source_allocations_v1,
    read_canonical_source_v1,
};
use clause_runtime::{
    ExecutableCanonicalInputBindingV1, ExecutableCanonicalJumpBindingV1,
    ExecutableCanonicalScalarBindingV1, ExecutableCanonicalScalarParameterBindingV1,
    ExecutableCanonicalTickBindingV1, ExecutableOccurrenceV1, WASM_SESSION_EVENT_LIMIT_V1,
    WasmPersistentSessionBoundaryV1, WasmProcessRequestV1, WasmProcessStatusV1,
    WasmSessionAdmissionScopeV1, WasmSessionAdmissionV1, WasmSessionAllocationV1,
    WasmSessionCommandV1, WasmSessionEventKindV1, WasmSessionHandleV1, WasmSessionLimitsV1,
    WasmSessionOpenV1, WasmSessionOperationV1, WasmSessionProjectionV1,
    decode_executable_occurrence_v1, decode_executable_physical_plan_v1,
    decode_wasm_process_request_v1, encode_executable_occurrence_v1,
    encode_executable_physical_plan_v1, encode_wasm_process_request_v1,
    encode_wasm_session_command_v1, encode_wasm_session_open_v1, lower_canonical_input_handler_v1,
    lower_canonical_jump_handler_v1, lower_canonical_scalar_handler_v1,
    lower_canonical_tick_program_v1,
};

const BASE_TEMPLATE_CWR1_HEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../browser/jump-arena-shell/fixtures/wasm-jump-v1/jump-v1.cwr1.hex"
));
const COHERENT_TEMPLATE_CWR1_HEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../browser/jump-arena-shell/fixtures/wasm-coherent-game-v1/coherent-game-v1.cwr1.hex"
));
const MAX_COMMANDS: u64 = 64;

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
    base_template: WasmProcessRequestV1,
    coherent_template: WasmProcessRequestV1,
    generation: ResidentSourceGenerationV1,
    package: ProcessPackageId,
    session: clause_package::RuntimeSessionId,
    sequence: u64,
    pending: Option<ResidentSourceCandidateV1>,
    last_projection: Option<WasmSessionProjectionV1>,
    next_change: u64,
    default_occurrences: Vec<Vec<u8>>,
}

impl ResidentSourceWorkbenchV1 {
    pub fn open(exact_source: &[u8]) -> Result<Self, ResidentSourceWorkbenchErrorV1> {
        let base_template = decode_wasm_process_request_v1(&decode_hex(BASE_TEMPLATE_CWR1_HEX)?)?;
        let coherent_template =
            decode_wasm_process_request_v1(&decode_hex(COHERENT_TEMPLATE_CWR1_HEX)?)?;
        let template = base_template.clone();
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
            },
            package: ProcessPackageId::from_bytes([0; clause_package::IDENTITY_BYTES]),
            session: template.authority.session,
            template,
            base_template,
            coherent_template,
            sequence: 0,
            pending: None,
            last_projection: None,
            next_change: 0,
            default_occurrences: Vec::new(),
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

    /// Compile and atomically install a fresh generation in this same
    /// workbench process. The prior handle becomes stale only after the new
    /// checked package/CPP1 session opens successfully.
    pub fn hot_reload(
        &mut self,
        exact_source: &[u8],
    ) -> Result<ResidentSourceGenerationV1, ResidentSourceWorkbenchErrorV1> {
        self.install_source(exact_source)?;
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
        for occurrence in prefix {
            let event = self.command(WasmSessionOperationV1::Input(occurrence.clone()))?;
            if !matches!(event, WasmSessionEventKindV1::InputAccepted { .. }) {
                return Err(unexpected_event("input", event));
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
        let decoded = decode_process_package(&self.template.package_bytes)
            .map_err(|error| boxed_error("template package decode", error))?;
        let template_package = check_process_package(decoded)
            .map_err(|error| boxed_error("template package check", error))?;
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
        let scope = TermScope {
            universe: template_package.constitution().universe(),
            semantics: template_package.constitution().semantics(),
        };
        let compiled = elaborate_canonical_source_package_v1(
            &cst,
            CanonicalSourceContextV1 {
                universe: scope.universe,
                semantics: scope.semantics,
            },
            &allocation_plan,
        )
        .map_err(|error| debug_error("canonical source elaboration", error))?;
        self.template = match compiled.scalar_handlers.len() {
            0 => self.base_template.clone(),
            7 => self.coherent_template.clone(),
            count => {
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "resident physical profile does not bind {count} scalar handlers"
                )));
            }
        };
        let decoded = decode_process_package(&self.template.package_bytes)
            .map_err(|error| boxed_error("selected template package decode", error))?;
        let _package = check_process_package(decoded)
            .map_err(|error| boxed_error("selected template package check", error))?;
        let mut physical_plan =
            decode_executable_physical_plan_v1(&self.template.physical_plan_bytes)
                .map_err(|error| boxed_error("CPP1 template decode", error))?;
        physical_plan.program.rules.clear();
        let input = compiled.input_handler.as_ref().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("canonical source has no input handler".into())
        })?;
        let jump = compiled.jump_handler.as_ref().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("canonical source has no jump handler".into())
        })?;
        let tick = compiled.tick_program.as_ref().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("canonical source has no tick program".into())
        })?;
        lower_canonical_input_handler_v1(
            &mut physical_plan.program,
            input,
            ExecutableCanonicalInputBindingV1 {
                entry: 0,
                x_slot: 4,
                z_slot: 21,
            },
        )
        .map_err(|error| boxed_error("canonical input lowering", error))?;
        lower_canonical_jump_handler_v1(
            &mut physical_plan.program,
            jump,
            ExecutableCanonicalJumpBindingV1 {
                entry: 1,
                velocity_slots: [2, 3, 13],
                grounded_slot: 5,
                jump_speed_slot: 7,
            },
        )
        .map_err(|error| boxed_error("canonical jump lowering", error))?;
        lower_canonical_tick_program_v1(
            &mut physical_plan.program,
            tick,
            ExecutableCanonicalTickBindingV1 {
                entry: 2,
                delta_time_argument: 0,
                position_slots: [0, 1, 12],
                velocity_slots: [2, 3, 13],
                intent_slots: [4, 22, 21],
                grounded_slot: 5,
                gravity_slot: 6,
                move_speed_slot: 8,
                floor_height_slot: 9,
                minimum_x_slot: 10,
                maximum_x_slot: 11,
                minimum_z_slot: 23,
                maximum_z_slot: 24,
            },
        )
        .map_err(|error| boxed_error("canonical tick lowering", error))?;
        for (index, handler) in compiled.scalar_handlers.iter().enumerate() {
            let (entry, state_slot, parameter_slots): (u16, u16, &[u16]) = match index {
                0 => (3, 28, &[0, 12]),
                1 => (4, 3, &[0, 12]),
                2 => (5, 5, &[0, 12]),
                3 => (6, 29, &[0, 12]),
                4 => (7, 29, &[0, 12]),
                5 => (8, 29, &[]),
                6 => (9, 29, &[0, 12]),
                _ => unreachable!("resident scalar profile was checked above"),
            };
            if handler.parameters.len() != parameter_slots.len() {
                return Err(ResidentSourceWorkbenchErrorV1(format!(
                    "resident scalar handler {index} has {} parameters; profile requires {}",
                    handler.parameters.len(),
                    parameter_slots.len()
                )));
            }
            lower_canonical_scalar_handler_v1(
                &mut physical_plan.program,
                handler,
                ExecutableCanonicalScalarBindingV1 {
                    entry,
                    state_slot,
                    parameters: handler
                        .parameters
                        .iter()
                        .zip(parameter_slots.iter().copied())
                        .map(
                            |(parameter, slot)| ExecutableCanonicalScalarParameterBindingV1 {
                                parameter: parameter.clone(),
                                slot,
                            },
                        )
                        .collect(),
                },
            )
            .map_err(|error| boxed_error("canonical scalar lowering", error))?;
        }
        let template_input = self.template.occurrences.first().cloned().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("CWR1 template has no input occurrence".into())
        })?;
        let template_tick = self.template.occurrences.last().ok_or_else(|| {
            ResidentSourceWorkbenchErrorV1("CWR1 template has no tick occurrence".into())
        })?;
        let template_tick = decode_executable_occurrence_v1(template_tick)
            .map_err(|error| boxed_error("template tick decode", error))?;
        let tick_entries = physical_plan
            .input
            .as_ref()
            .ok_or_else(|| {
                ResidentSourceWorkbenchErrorV1("CPP1 template has no input plan".into())
            })?
            .tick
            .entries
            .clone();
        let mut default_occurrences = Vec::with_capacity(tick_entries.len() + 1);
        default_occurrences.push(template_input);
        for (ordinal, entry) in tick_entries.into_iter().enumerate() {
            default_occurrences.push(
                encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
                    entry,
                    arguments: if ordinal == 0 {
                        template_tick.arguments.clone()
                    } else {
                        Vec::new()
                    },
                })
                .map_err(|error| boxed_error("resident occurrence encode", error))?,
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
