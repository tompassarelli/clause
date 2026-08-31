//! Generation-tagged physical Wasm custody for one persistent process session.

use clause_package::*;

use super::wasm_boundary::{
    Decoder, MAX_EVIDENCE_BYTES, decode_wasm_authority_input_v1, encode_wasm_authority_input_v1,
    establish_persistent_authority, put_blob,
};
use super::{
    ExecutableCarrierErrorV1, PersistentProcessSessionErrorV1, PersistentProcessSessionV1,
    RuntimeAllocationEpochV1, WASM_PROCESS_REQUEST_LIMIT_V1, WASM_PROCESS_RESPONSE_LIMIT_V1,
    WasmAuthorityInputV1, WasmProcessStatusV1, decode_executable_physical_plan_v1,
    decode_runtime_allocation_epoch_v1, encode_runtime_allocation_epoch_v1,
};

const OPEN_MAGIC: &[u8; 4] = b"CWS1";
const COMMAND_MAGIC: &[u8; 4] = b"CWI1";
const EVENT_MAGIC: &[u8; 4] = b"CSE1";
const SLOT: u32 = 0;
const EVENT_HEADER_BYTES: usize = 4 + 4 + 4 + 8 + 1;
const ALLOCATION_EPOCH_BYTES_V1: usize = 304;

pub const WASM_SESSION_COMMAND_LIMIT_V1: usize = 1024 * 1024;
pub const WASM_SESSION_EVENT_LIMIT_V1: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmSessionHandleV1 {
    pub slot: u32,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmSessionLimitsV1 {
    pub max_commands: u64,
    pub command_bytes: u32,
    pub event_bytes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmSessionOpenV1 {
    pub package_bytes: Vec<u8>,
    pub application: ApplicationLocalId,
    pub physical_plan_bytes: Vec<u8>,
    pub authority: WasmAuthorityInputV1,
    pub allocation: WasmSessionAllocationV1,
    pub limits: WasmSessionLimitsV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmSessionAllocationV1 {
    New,
    Rematerialize(RuntimeAllocationEpochV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmSessionAdmissionV1 {
    pub package: ProcessPackageId,
    pub session: RuntimeSessionId,
    pub base: StateRevisionId,
    pub candidate: CandidateDeltaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmSessionOperationV1 {
    Input(Vec<u8>),
    Candidate(Vec<u8>),
    Admit(WasmSessionAdmissionV1),
    Dispose,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmSessionCommandV1 {
    pub handle: WasmSessionHandleV1,
    pub expected_sequence: u64,
    pub operation: WasmSessionOperationV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum WasmSessionRejectionV1 {
    InputRejected = 1,
    CandidateRejected = 2,
    AdmissionScopeRejected = 3,
    AuthorityRejected = 4,
    AdmissionRejected = 5,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmSessionEventKindV1 {
    Opened {
        package: ProcessPackageId,
        session: RuntimeSessionId,
        world: StateRevisionId,
        run: RunId,
        activation: ActivationId,
        allocation: RuntimeAllocationEpochV1,
        state_revision_count: u32,
    },
    InputAccepted {
        step: StepId,
        run: RunId,
        activation: ActivationId,
        before: ConfigurationId,
        after: ConfigurationId,
        state_revision_count: u32,
    },
    CandidateAccepted {
        step: StepId,
        candidate: CandidateDeltaId,
        base: StateRevisionId,
        run: RunId,
        activation: ActivationId,
        state_revision_count: u32,
    },
    AdmissionAccepted {
        predecessor: StateRevisionId,
        successor: StateRevisionId,
        run: RunId,
        activation: ActivationId,
        session: RuntimeSessionId,
        state_revision_count: u32,
        projection: Option<WasmSessionProjectionV1>,
    },
    Disposed,
    Rejected(WasmSessionRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmSessionProjectionV1 {
    pub observation: ObservationId,
    pub exact_term_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmSessionEventV1 {
    pub handle: WasmSessionHandleV1,
    pub accepted_sequence: u64,
    pub kind: WasmSessionEventKindV1,
}

struct LiveSessionV1 {
    session: PersistentProcessSessionV1,
    sequence: u64,
    limits: WasmSessionLimitsV1,
}

/// A bounded physical table with one live slot. Its handle never substitutes
/// for a Clause identity; generation exists only to reject stale host custody.
pub struct WasmPersistentSessionBoundaryV1 {
    generation: Option<u32>,
    exhausted: bool,
    live: Option<LiveSessionV1>,
    request: Vec<u8>,
    event: Vec<u8>,
    status: WasmProcessStatusV1,
}

impl Default for WasmPersistentSessionBoundaryV1 {
    fn default() -> Self {
        Self {
            generation: None,
            exhausted: false,
            live: None,
            request: Vec::with_capacity(WASM_SESSION_COMMAND_LIMIT_V1),
            event: Vec::with_capacity(WASM_SESSION_EVENT_LIMIT_V1),
            status: WasmProcessStatusV1::Ready,
        }
    }
}

impl WasmPersistentSessionBoundaryV1 {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_io(&mut self) {
        self.request.clear();
        self.event.clear();
        self.status = WasmProcessStatusV1::Ready;
    }

    pub fn push_request_byte(&mut self, byte: u8) -> Result<(), WasmProcessStatusV1> {
        if self.request.len() == WASM_PROCESS_REQUEST_LIMIT_V1 {
            self.status = WasmProcessStatusV1::RequestOutOfBounds;
            return Err(self.status);
        }
        self.request.push(byte);
        Ok(())
    }

    pub fn open_buffered(&mut self) -> Result<(), WasmProcessStatusV1> {
        self.event.clear();
        let mut bytes = Vec::new();
        std::mem::swap(&mut bytes, &mut self.request);
        let result = self.open(&bytes);
        bytes.clear();
        std::mem::swap(&mut bytes, &mut self.request);
        match result {
            Ok(event) => self.install_event(event),
            Err(error) => {
                self.status = error;
                Err(error)
            }
        }
    }

    pub fn command_buffered(&mut self) -> Result<(), WasmProcessStatusV1> {
        self.event.clear();
        let mut bytes = Vec::new();
        std::mem::swap(&mut bytes, &mut self.request);
        let result = self.command(&bytes);
        bytes.clear();
        std::mem::swap(&mut bytes, &mut self.request);
        match result {
            Ok(event) => self.install_event(event),
            Err(error) => {
                self.status = error;
                Err(error)
            }
        }
    }

    pub fn open(&mut self, bytes: &[u8]) -> Result<WasmSessionEventV1, WasmProcessStatusV1> {
        let request = decode_wasm_session_open_v1(bytes)?;
        validate_limits(request.limits)?;
        if self.live.is_some() {
            return self.fail(WasmProcessStatusV1::SessionOccupied);
        }
        if self.exhausted {
            return self.fail(WasmProcessStatusV1::SessionExhausted);
        }
        if usize::try_from(request.limits.event_bytes).unwrap_or(usize::MAX) < open_event_size() {
            return self.fail(WasmProcessStatusV1::ResponseOutOfBounds);
        }
        let generation = match self.generation {
            None => 1,
            Some(generation) => match generation.checked_add(1) {
                Some(next) => next,
                None => {
                    self.exhausted = true;
                    return self.fail(WasmProcessStatusV1::SessionExhausted);
                }
            },
        };
        let decoded = decode_process_package(&request.package_bytes)
            .map_err(|_| WasmProcessStatusV1::PackageRejected)?;
        let package =
            check_process_package(decoded).map_err(|_| WasmProcessStatusV1::PackageRejected)?;
        let application = ApplicationId {
            snapshot: package.constitution().snapshot(),
            local: request.application,
        };
        let physical_plan = decode_executable_physical_plan_v1(&request.physical_plan_bytes)
            .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let (authority, facts) = establish_persistent_authority(&package, &request.authority)?;
        let session = match request.allocation {
            WasmSessionAllocationV1::New => PersistentProcessSessionV1::open(
                package,
                authority,
                application,
                physical_plan,
                facts,
            ),
            WasmSessionAllocationV1::Rematerialize(allocation) => {
                PersistentProcessSessionV1::rematerialize(
                    package,
                    authority,
                    application,
                    physical_plan,
                    facts,
                    allocation,
                )
            }
        }
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let handle = WasmSessionHandleV1 {
            slot: SLOT,
            generation,
        };
        let event = WasmSessionEventV1 {
            handle,
            accepted_sequence: 0,
            kind: WasmSessionEventKindV1::Opened {
                package: session
                    .package()
                    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?,
                session: session.runtime_session(),
                world: session.world_base(),
                run: session
                    .run()
                    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?,
                activation: session
                    .activation()
                    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?,
                allocation: session.allocation(),
                state_revision_count: state_revision_count(&session)?,
            },
        };
        self.generation = Some(generation);
        self.live = Some(LiveSessionV1 {
            session,
            sequence: 0,
            limits: request.limits,
        });
        self.status = WasmProcessStatusV1::Ready;
        Ok(event)
    }

    pub fn command(&mut self, bytes: &[u8]) -> Result<WasmSessionEventV1, WasmProcessStatusV1> {
        if bytes.len() > WASM_SESSION_COMMAND_LIMIT_V1 {
            return self.fail(WasmProcessStatusV1::RequestOutOfBounds);
        }
        let command = decode_wasm_session_command_v1(bytes)?;
        let live = self
            .live
            .as_mut()
            .ok_or(WasmProcessStatusV1::StaleSessionHandle)?;
        if command.handle.slot != SLOT || self.generation != Some(command.handle.generation) {
            return self.fail(WasmProcessStatusV1::StaleSessionHandle);
        }
        if bytes.len() > usize::try_from(live.limits.command_bytes).unwrap_or(usize::MAX) {
            return self.fail(WasmProcessStatusV1::RequestOutOfBounds);
        }
        if command.expected_sequence != live.sequence {
            return self.fail(WasmProcessStatusV1::SequenceRejected);
        }
        if live.sequence == live.limits.max_commands {
            return self.fail(WasmProcessStatusV1::SessionLimitReached);
        }
        if usize::try_from(live.limits.event_bytes).unwrap_or(usize::MAX)
            < command_event_size(&command.operation)
        {
            return self.fail(WasmProcessStatusV1::ResponseOutOfBounds);
        }

        let kind = execute_operation(&mut live.session, command.operation);
        let accepted_sequence = live
            .sequence
            .checked_add(1)
            .ok_or(WasmProcessStatusV1::SessionLimitReached)?;
        live.sequence = accepted_sequence;
        self.status = WasmProcessStatusV1::Ready;
        let event = WasmSessionEventV1 {
            handle: command.handle,
            accepted_sequence,
            kind,
        };
        if matches!(event.kind, WasmSessionEventKindV1::Disposed) {
            self.live = None;
        }
        Ok(event)
    }

    fn install_event(&mut self, event: WasmSessionEventV1) -> Result<(), WasmProcessStatusV1> {
        let bytes = encode_wasm_session_event_v1(&event);
        if bytes.len() > WASM_PROCESS_RESPONSE_LIMIT_V1 {
            return self.fail(WasmProcessStatusV1::ResponseOutOfBounds);
        }
        self.event = bytes;
        self.status = WasmProcessStatusV1::Ready;
        Ok(())
    }

    fn fail<T>(&mut self, status: WasmProcessStatusV1) -> Result<T, WasmProcessStatusV1> {
        self.status = status;
        Err(status)
    }

    #[must_use]
    pub fn event(&self) -> &[u8] {
        &self.event
    }

    #[must_use]
    pub const fn status(&self) -> WasmProcessStatusV1 {
        self.status
    }
}

fn execute_operation(
    session: &mut PersistentProcessSessionV1,
    operation: WasmSessionOperationV1,
) -> WasmSessionEventKindV1 {
    match operation {
        WasmSessionOperationV1::Input(bytes) => match session.apply_opaque_input(&bytes) {
            Ok(step) => WasmSessionEventKindV1::InputAccepted {
                step: step.id,
                run: session.run().expect("accepted input retains its Run"),
                activation: session
                    .activation()
                    .expect("accepted input retains its Activation"),
                before: step.before,
                after: step.after,
                state_revision_count: state_revision_count(session)
                    .expect("accepted input retains its carrier"),
            },
            Err(_) => WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::InputRejected),
        },
        WasmSessionOperationV1::Candidate(bytes) => {
            match session.apply_opaque_input_and_emit_candidate(&bytes) {
                Ok(step) => {
                    let candidate = session
                        .candidate()
                        .expect("accepted candidate retains its runtime")
                        .expect("candidate command installs a candidate");
                    WasmSessionEventKindV1::CandidateAccepted {
                        step: step.id,
                        candidate: candidate.id,
                        base: candidate.base,
                        run: session.run().expect("accepted candidate retains its Run"),
                        activation: session
                            .activation()
                            .expect("accepted candidate retains its Activation"),
                        state_revision_count: state_revision_count(session)
                            .expect("accepted candidate retains its carrier"),
                    }
                }
                Err(_) => {
                    WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::CandidateRejected)
                }
            }
        }
        WasmSessionOperationV1::Admit(input) => admit(session, input),
        WasmSessionOperationV1::Dispose => {
            session.dispose();
            WasmSessionEventKindV1::Disposed
        }
    }
}

fn admit(
    session: &mut PersistentProcessSessionV1,
    input: WasmSessionAdmissionV1,
) -> WasmSessionEventKindV1 {
    let exact = session
        .package()
        .ok()
        .zip(session.candidate().ok().flatten())
        .is_some_and(|(package, candidate)| {
            package == input.package
                && session.runtime_session() == input.session
                && session.world_base() == input.base
                && candidate.id == input.candidate
                && candidate.base == input.base
        });
    if !exact {
        return WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::AdmissionScopeRejected);
    }
    let prior_run = session.run().expect("live Admission retains its prior Run");
    let prior_activation = session
        .activation()
        .expect("live Admission retains its prior Activation");
    match session.admit_constituted_candidate_with_projection() {
        Ok((successor, projection)) => {
            let run = session.run().expect("Admission installs a fresh Run");
            let activation = session
                .activation()
                .expect("Admission installs a fresh Activation");
            debug_assert_ne!(run, prior_run);
            debug_assert_ne!(activation, prior_activation);
            WasmSessionEventKindV1::AdmissionAccepted {
                predecessor: successor.predecessor,
                successor: successor.id,
                run,
                activation,
                session: session.runtime_session(),
                state_revision_count: state_revision_count(session)
                    .expect("Admission retains its carrier"),
                projection: projection.map(|projection| WasmSessionProjectionV1 {
                    observation: projection.id,
                    exact_term_bytes: canonical_term_bytes(&projection.term)
                        .expect("checked projection Term remains canonical"),
                }),
            }
        }
        Err(PersistentProcessSessionErrorV1::Carrier(
            ExecutableCarrierErrorV1::ConstitutiveAdmissionAuthorityUnavailable,
        )) => WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::AuthorityRejected),
        Err(_) => WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::AdmissionRejected),
    }
}

fn state_revision_count(session: &PersistentProcessSessionV1) -> Result<u32, WasmProcessStatusV1> {
    u32::try_from(
        session
            .carrier()
            .map_err(|_| WasmProcessStatusV1::ProcessRejected)?
            .state_revision_count(),
    )
    .map_err(|_| WasmProcessStatusV1::ResponseOutOfBounds)
}

fn validate_limits(limits: WasmSessionLimitsV1) -> Result<(), WasmProcessStatusV1> {
    if limits.max_commands == 0
        || limits.command_bytes == 0
        || usize::try_from(limits.command_bytes).unwrap_or(usize::MAX)
            > WASM_SESSION_COMMAND_LIMIT_V1
        || usize::try_from(limits.event_bytes).unwrap_or(usize::MAX) != WASM_SESSION_EVENT_LIMIT_V1
    {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(())
}

const fn open_event_size() -> usize {
    EVENT_HEADER_BYTES + 5 * IDENTITY_BYTES + 4 + 4 + ALLOCATION_EPOCH_BYTES_V1
}

fn command_event_size(operation: &WasmSessionOperationV1) -> usize {
    match operation {
        WasmSessionOperationV1::Dispose => EVENT_HEADER_BYTES,
        WasmSessionOperationV1::Input(_) | WasmSessionOperationV1::Candidate(_) => {
            EVENT_HEADER_BYTES + 5 * IDENTITY_BYTES + 4
        }
        WasmSessionOperationV1::Admit(_) => WASM_SESSION_EVENT_LIMIT_V1,
    }
}

pub fn encode_wasm_session_open_v1(
    request: &WasmSessionOpenV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    validate_limits(request.limits)?;
    if request.authority.occurrence_evidence_bytes.len() > MAX_EVIDENCE_BYTES
        || request.authority.judgment_evidence_bytes.len() > MAX_EVIDENCE_BYTES
        || request.authority.admission_evidence_bytes.len() > MAX_EVIDENCE_BYTES
    {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(OPEN_MAGIC);
    put_blob(&mut bytes, &request.package_bytes)?;
    bytes.extend_from_slice(&request.application.get().to_le_bytes());
    put_blob(&mut bytes, &request.physical_plan_bytes)?;
    encode_wasm_authority_input_v1(&mut bytes, &request.authority)?;
    bytes.extend_from_slice(&request.authority.budget_units.to_le_bytes());
    match request.allocation {
        WasmSessionAllocationV1::New => bytes.push(0),
        WasmSessionAllocationV1::Rematerialize(allocation) => {
            bytes.push(1);
            put_blob(&mut bytes, &encode_runtime_allocation_epoch_v1(allocation))?;
        }
    }
    bytes.extend_from_slice(&request.limits.max_commands.to_le_bytes());
    bytes.extend_from_slice(&request.limits.command_bytes.to_le_bytes());
    bytes.extend_from_slice(&request.limits.event_bytes.to_le_bytes());
    if bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_wasm_session_open_v1(bytes: &[u8]) -> Result<WasmSessionOpenV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != OPEN_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let package_bytes = d.blob(WASM_PROCESS_REQUEST_LIMIT_V1)?.to_vec();
    let application = ApplicationLocalId::new(d.u32()?);
    let physical_plan_bytes = d.blob(WASM_PROCESS_REQUEST_LIMIT_V1)?.to_vec();
    let mut authority = decode_wasm_authority_input_v1(&mut d)?;
    authority.budget_units = d.u64()?;
    let allocation = match d.take(1)?[0] {
        0 => WasmSessionAllocationV1::New,
        1 => WasmSessionAllocationV1::Rematerialize(
            decode_runtime_allocation_epoch_v1(d.blob(ALLOCATION_EPOCH_BYTES_V1)?)
                .map_err(|_| WasmProcessStatusV1::MalformedRequest)?,
        ),
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    };
    let limits = WasmSessionLimitsV1 {
        max_commands: d.u64()?,
        command_bytes: d.u32()?,
        event_bytes: d.u32()?,
    };
    if !d.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    validate_limits(limits)?;
    Ok(WasmSessionOpenV1 {
        package_bytes,
        application,
        physical_plan_bytes,
        authority,
        allocation,
        limits,
    })
}

pub fn encode_wasm_session_command_v1(
    command: &WasmSessionCommandV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(COMMAND_MAGIC);
    bytes.extend_from_slice(&command.handle.slot.to_le_bytes());
    bytes.extend_from_slice(&command.handle.generation.to_le_bytes());
    bytes.extend_from_slice(&command.expected_sequence.to_le_bytes());
    match &command.operation {
        WasmSessionOperationV1::Input(input) => {
            bytes.push(1);
            put_blob(&mut bytes, input)?;
        }
        WasmSessionOperationV1::Candidate(input) => {
            bytes.push(2);
            put_blob(&mut bytes, input)?;
        }
        WasmSessionOperationV1::Admit(input) => {
            bytes.push(3);
            for id in [
                input.package.as_bytes(),
                input.session.as_bytes(),
                input.base.as_bytes(),
                input.candidate.as_bytes(),
            ] {
                bytes.extend_from_slice(id);
            }
        }
        WasmSessionOperationV1::Dispose => bytes.push(4),
    }
    if bytes.len() > WASM_SESSION_COMMAND_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_wasm_session_command_v1(
    bytes: &[u8],
) -> Result<WasmSessionCommandV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_SESSION_COMMAND_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != COMMAND_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let handle = WasmSessionHandleV1 {
        slot: d.u32()?,
        generation: d.u32()?,
    };
    let expected_sequence = d.u64()?;
    let operation = match d.take(1)?[0] {
        1 => WasmSessionOperationV1::Input(d.blob(WASM_SESSION_COMMAND_LIMIT_V1)?.to_vec()),
        2 => WasmSessionOperationV1::Candidate(d.blob(WASM_SESSION_COMMAND_LIMIT_V1)?.to_vec()),
        3 => WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
            package: ProcessPackageId::from_bytes(d.identity()?),
            session: RuntimeSessionId::from_bytes(d.identity()?),
            base: StateRevisionId::from_bytes(d.identity()?),
            candidate: CandidateDeltaId::from_bytes(d.identity()?),
        }),
        4 => WasmSessionOperationV1::Dispose,
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    };
    if !d.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(WasmSessionCommandV1 {
        handle,
        expected_sequence,
        operation,
    })
}

pub fn encode_wasm_session_event_v1(event: &WasmSessionEventV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(EVENT_MAGIC);
    bytes.extend_from_slice(&event.handle.slot.to_le_bytes());
    bytes.extend_from_slice(&event.handle.generation.to_le_bytes());
    bytes.extend_from_slice(&event.accepted_sequence.to_le_bytes());
    match &event.kind {
        WasmSessionEventKindV1::Opened {
            package,
            session,
            world,
            run,
            activation,
            allocation,
            state_revision_count,
        } => {
            bytes.push(1);
            put_ids(
                &mut bytes,
                &[
                    package.as_bytes(),
                    session.as_bytes(),
                    world.as_bytes(),
                    run.as_bytes(),
                    activation.as_bytes(),
                ],
            );
            bytes.extend_from_slice(&state_revision_count.to_le_bytes());
            put_blob(&mut bytes, &encode_runtime_allocation_epoch_v1(*allocation))
                .expect("a runtime allocation record has one fixed bounded encoding");
        }
        WasmSessionEventKindV1::InputAccepted {
            step,
            run,
            activation,
            before,
            after,
            state_revision_count,
        } => {
            bytes.push(2);
            put_ids(
                &mut bytes,
                &[
                    step.as_bytes(),
                    run.as_bytes(),
                    activation.as_bytes(),
                    before.as_bytes(),
                    after.as_bytes(),
                ],
            );
            bytes.extend_from_slice(&state_revision_count.to_le_bytes());
        }
        WasmSessionEventKindV1::CandidateAccepted {
            step,
            candidate,
            base,
            run,
            activation,
            state_revision_count,
        } => {
            bytes.push(3);
            put_ids(
                &mut bytes,
                &[
                    step.as_bytes(),
                    candidate.as_bytes(),
                    base.as_bytes(),
                    run.as_bytes(),
                    activation.as_bytes(),
                ],
            );
            bytes.extend_from_slice(&state_revision_count.to_le_bytes());
        }
        WasmSessionEventKindV1::AdmissionAccepted {
            predecessor,
            successor,
            run,
            activation,
            session,
            state_revision_count,
            projection,
        } => {
            bytes.push(4);
            put_ids(
                &mut bytes,
                &[
                    predecessor.as_bytes(),
                    successor.as_bytes(),
                    run.as_bytes(),
                    activation.as_bytes(),
                    session.as_bytes(),
                ],
            );
            bytes.extend_from_slice(&state_revision_count.to_le_bytes());
            if let Some(projection) = projection {
                bytes.push(1);
                bytes.extend_from_slice(projection.observation.as_bytes());
                put_blob(&mut bytes, &projection.exact_term_bytes)
                    .expect("checked projection Term fits the CSE1 event bound");
            } else {
                bytes.push(0);
            }
        }
        WasmSessionEventKindV1::Disposed => bytes.push(5),
        WasmSessionEventKindV1::Rejected(rejection) => {
            bytes.push(6);
            bytes.extend_from_slice(&(*rejection as u32).to_le_bytes());
        }
    }
    bytes
}

pub fn decode_wasm_session_event_v1(
    bytes: &[u8],
) -> Result<WasmSessionEventV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_SESSION_EVENT_LIMIT_V1 {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != EVENT_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let handle = WasmSessionHandleV1 {
        slot: d.u32()?,
        generation: d.u32()?,
    };
    let accepted_sequence = d.u64()?;
    let kind = match d.take(1)?[0] {
        1 => WasmSessionEventKindV1::Opened {
            package: ProcessPackageId::from_bytes(d.identity()?),
            session: RuntimeSessionId::from_bytes(d.identity()?),
            world: StateRevisionId::from_bytes(d.identity()?),
            run: RunId::from_bytes(d.identity()?),
            activation: ActivationId::from_bytes(d.identity()?),
            state_revision_count: d.u32()?,
            allocation: decode_runtime_allocation_epoch_v1(d.blob(ALLOCATION_EPOCH_BYTES_V1)?)
                .map_err(|_| WasmProcessStatusV1::MalformedRequest)?,
        },
        2 => WasmSessionEventKindV1::InputAccepted {
            step: StepId::from_bytes(d.identity()?),
            run: RunId::from_bytes(d.identity()?),
            activation: ActivationId::from_bytes(d.identity()?),
            before: ConfigurationId::from_bytes(d.identity()?),
            after: ConfigurationId::from_bytes(d.identity()?),
            state_revision_count: d.u32()?,
        },
        3 => WasmSessionEventKindV1::CandidateAccepted {
            step: StepId::from_bytes(d.identity()?),
            candidate: CandidateDeltaId::from_bytes(d.identity()?),
            base: StateRevisionId::from_bytes(d.identity()?),
            run: RunId::from_bytes(d.identity()?),
            activation: ActivationId::from_bytes(d.identity()?),
            state_revision_count: d.u32()?,
        },
        4 => WasmSessionEventKindV1::AdmissionAccepted {
            predecessor: StateRevisionId::from_bytes(d.identity()?),
            successor: StateRevisionId::from_bytes(d.identity()?),
            run: RunId::from_bytes(d.identity()?),
            activation: ActivationId::from_bytes(d.identity()?),
            session: RuntimeSessionId::from_bytes(d.identity()?),
            state_revision_count: d.u32()?,
            projection: match d.take(1)?[0] {
                0 => None,
                1 => Some(WasmSessionProjectionV1 {
                    observation: ObservationId::from_bytes(d.identity()?),
                    exact_term_bytes: d.blob(WASM_SESSION_EVENT_LIMIT_V1)?.to_vec(),
                }),
                _ => return Err(WasmProcessStatusV1::MalformedRequest),
            },
        },
        5 => WasmSessionEventKindV1::Disposed,
        6 => WasmSessionEventKindV1::Rejected(match d.u32()? {
            1 => WasmSessionRejectionV1::InputRejected,
            2 => WasmSessionRejectionV1::CandidateRejected,
            3 => WasmSessionRejectionV1::AdmissionScopeRejected,
            4 => WasmSessionRejectionV1::AuthorityRejected,
            5 => WasmSessionRejectionV1::AdmissionRejected,
            _ => return Err(WasmProcessStatusV1::MalformedRequest),
        }),
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    };
    if !d.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(WasmSessionEventV1 {
        handle,
        accepted_sequence,
        kind,
    })
}

fn put_ids(bytes: &mut Vec<u8>, identities: &[&[u8; IDENTITY_BYTES]]) {
    for identity in identities {
        bytes.extend_from_slice(*identity);
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use std::cell::RefCell;

    use wasm_bindgen::prelude::wasm_bindgen;

    use super::{WasmPersistentSessionBoundaryV1, WasmProcessStatusV1};

    thread_local! {
        static SESSION_BOUNDARY: RefCell<WasmPersistentSessionBoundaryV1> =
            RefCell::new(WasmPersistentSessionBoundaryV1::new());
    }

    #[wasm_bindgen]
    pub fn clause_session_v1_io_reset() {
        SESSION_BOUNDARY.with_borrow_mut(WasmPersistentSessionBoundaryV1::clear_io);
    }

    #[wasm_bindgen]
    pub fn clause_session_v1_request_push(byte: u32) -> u32 {
        let Ok(byte) = u8::try_from(byte) else {
            return WasmProcessStatusV1::MalformedRequest as u32;
        };
        SESSION_BOUNDARY.with_borrow_mut(|boundary| match boundary.push_request_byte(byte) {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_session_v1_open() -> u32 {
        SESSION_BOUNDARY.with_borrow_mut(|boundary| match boundary.open_buffered() {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_session_v1_command() -> u32 {
        SESSION_BOUNDARY.with_borrow_mut(|boundary| match boundary.command_buffered() {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_session_v1_event_len() -> u32 {
        SESSION_BOUNDARY
            .with_borrow(|boundary| u32::try_from(boundary.event().len()).unwrap_or(u32::MAX))
    }

    /// Values 0..=255 are event bytes; 256 means an out-of-range index.
    #[wasm_bindgen]
    pub fn clause_session_v1_event_byte(index: u32) -> u32 {
        SESSION_BOUNDARY.with_borrow(|boundary| {
            usize::try_from(index)
                .ok()
                .and_then(|index| boundary.event().get(index))
                .map_or(256, |byte| u32::from(*byte))
        })
    }
}
