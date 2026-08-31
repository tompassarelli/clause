//! Generation-tagged Wasm custody for one retained non-authoritative branch
//! and its separate authoritative process session.

use clause_package::*;

use super::wasm_boundary::{Decoder, put_blob, put_count};
use super::{
    CheckedReconnectAdmissionPlanV1, ExecutablePhysicalPlanIdV1, ExecutableProjectedObservationV1,
    ExecutableResumptionV1, ExecutableSuspensionV1, ForkedProcessBranchV1, ProcessBranchAncestryV1,
    ProcessBranchErrorV1, ProcessBranchExplanationV1, ProcessBranchPinV1, ProcessBranchPinsV1,
    ProcessCausalRecordV1, ProcessCommandEvidenceV1, ProcessReconnectAdmissionV1,
    ProcessReconnectEvidenceV1, WASM_PROCESS_REQUEST_LIMIT_V1, WASM_PROCESS_RESPONSE_LIMIT_V1,
    WasmProcessStatusV1, open_fresh_persistent_process_session_v1,
};

const OPEN_MAGIC: &[u8; 4] = b"CBR1";
const COMMAND_MAGIC: &[u8; 4] = b"CBI1";
const EVENT_MAGIC: &[u8; 4] = b"CBE1";
const EVIDENCE_MAGIC: &[u8; 4] = b"CRE1";
const EXPLANATION_MAGIC: &[u8; 4] = b"CBX1";
const SLOT: u32 = 0;
const MAX_OCCURRENCES: usize = 256;
const MAX_CAUSAL_RECORDS: usize = 2048;

pub const WASM_BRANCH_COMMAND_LIMIT_V1: usize = 1024 * 1024;
pub const WASM_BRANCH_EVENT_LIMIT_V1: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmBranchHandleV1 {
    pub slot: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBranchOpenV1 {
    pub exact_cwr1: Vec<u8>,
    pub disconnect_tick: u64,
    pub disconnect_occurrence: Vec<u8>,
    pub max_commands: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBranchOperationV1 {
    /// Apply the supplied construct-blind occurrences to the authoritative
    /// session, emit a CandidateDelta on the last occurrence, and establish
    /// its successor only through issued authorization and Admission.
    AdmitAuthoritativeOccurrences(Vec<Vec<u8>>),
    /// Resume the retained branch and emit its non-authoritative reconnect
    /// CandidateDelta. The returned exact evidence must be submitted unchanged
    /// to adjudication.
    ProposeReconnect(Vec<Vec<u8>>),
    /// Execute one caller-selected, construct-blind consequence plan against
    /// the current authoritative base, then issue and consume Admission
    /// authority inside Clause.
    Adjudicate {
        reconnect_evidence: Vec<u8>,
        branch_candidate: CandidateDeltaId,
        authoritative_base: StateRevisionId,
        occurrences: Vec<Vec<u8>>,
    },
    Explain,
    Dispose,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBranchCommandV1 {
    pub handle: WasmBranchHandleV1,
    pub expected_sequence: u64,
    pub operation: WasmBranchOperationV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBranchRejectionV1 {
    MissingOccurrence,
    AuthoritativeAdmissionRejected,
    AlreadyProposed,
    MissingProposal,
    AlreadyAdjudicated,
    EvidenceMismatch,
    UnexpectedCandidate,
    PinMismatch(ProcessBranchPinV1),
    MissingCausalRecord,
    ExplanationUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBranchEventKindV1 {
    Opened {
        pins: ProcessBranchPinsV1,
        ancestry: ProcessBranchAncestryV1,
        suspension: ExecutableSuspensionV1,
    },
    AuthoritativeAdmissionAccepted {
        candidate: CandidateDeltaId,
        predecessor: StateRevisionId,
        successor: StateRevisionId,
        judgment: JudgmentOccurrenceId,
        admission: AdmissionOccurrenceId,
        run: RunId,
        activation: ActivationId,
    },
    ReconnectProposed {
        evidence: ProcessReconnectEvidenceV1,
        exact_evidence: Vec<u8>,
    },
    ReconnectAdmissionAccepted {
        predecessor: StateRevisionId,
        successor: StateRevisionId,
        branch_candidate: CandidateDeltaId,
        authoritative_candidate: CandidateDeltaId,
        judgment: JudgmentOccurrenceId,
        admission: AdmissionOccurrenceId,
        projection: Option<WasmBranchProjectionV1>,
        explanation: ProcessBranchExplanationV1,
        exact_explanation: Vec<u8>,
    },
    Explanation {
        explanation: ProcessBranchExplanationV1,
        exact_explanation: Vec<u8>,
    },
    Disposed,
    Rejected(WasmBranchRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBranchProjectionV1 {
    pub observation: ObservationId,
    pub exact_term_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBranchEventV1 {
    pub handle: WasmBranchHandleV1,
    pub accepted_sequence: u64,
    pub kind: WasmBranchEventKindV1,
}

struct LiveBranchV1 {
    authoritative: super::PersistentProcessSessionV1,
    branch: ForkedProcessBranchV1,
    sequence: u64,
    max_commands: u64,
}

/// One transactionally replaceable physical branch slot. The handle and
/// sequence are host-custody guards only; they never mint or replace a Clause
/// identity, evidence occurrence, CandidateDelta, or StateRevision.
pub struct WasmProcessBranchBoundaryV1 {
    generation: Option<u32>,
    exhausted: bool,
    live: Option<LiveBranchV1>,
    request: Vec<u8>,
    event: Vec<u8>,
    status: WasmProcessStatusV1,
}

impl Default for WasmProcessBranchBoundaryV1 {
    fn default() -> Self {
        Self {
            generation: None,
            exhausted: false,
            live: None,
            request: Vec::with_capacity(WASM_BRANCH_COMMAND_LIMIT_V1),
            event: Vec::with_capacity(WASM_BRANCH_EVENT_LIMIT_V1),
            status: WasmProcessStatusV1::Ready,
        }
    }
}

impl WasmProcessBranchBoundaryV1 {
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
        self.dispatch_buffered(true)
    }

    pub fn command_buffered(&mut self) -> Result<(), WasmProcessStatusV1> {
        self.dispatch_buffered(false)
    }

    fn dispatch_buffered(&mut self, open: bool) -> Result<(), WasmProcessStatusV1> {
        self.event.clear();
        let mut bytes = Vec::new();
        std::mem::swap(&mut bytes, &mut self.request);
        let result = if open {
            self.open(&bytes)
        } else {
            self.command(&bytes)
        };
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

    pub fn open(&mut self, bytes: &[u8]) -> Result<WasmBranchEventV1, WasmProcessStatusV1> {
        let request = decode_wasm_branch_open_v1(bytes)?;
        if self.exhausted {
            return self.fail(WasmProcessStatusV1::SessionExhausted);
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
        let authoritative = open_fresh_persistent_process_session_v1(&request.exact_cwr1)?;
        let branch_session = open_fresh_persistent_process_session_v1(&request.exact_cwr1)?;
        let branch = ForkedProcessBranchV1::fork(
            &authoritative,
            branch_session,
            request.disconnect_tick,
            &request.disconnect_occurrence,
        )
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let handle = WasmBranchHandleV1 {
            slot: SLOT,
            generation,
        };
        let event = WasmBranchEventV1 {
            handle,
            accepted_sequence: 0,
            kind: WasmBranchEventKindV1::Opened {
                pins: branch.pins(),
                ancestry: branch.ancestry(),
                suspension: branch.suspension(),
            },
        };
        self.live = Some(LiveBranchV1 {
            authoritative,
            branch,
            sequence: 0,
            max_commands: request.max_commands,
        });
        self.generation = Some(generation);
        self.status = WasmProcessStatusV1::Ready;
        Ok(event)
    }

    pub fn command(&mut self, bytes: &[u8]) -> Result<WasmBranchEventV1, WasmProcessStatusV1> {
        if bytes.len() > WASM_BRANCH_COMMAND_LIMIT_V1 {
            return self.fail(WasmProcessStatusV1::RequestOutOfBounds);
        }
        let command = decode_wasm_branch_command_v1(bytes)?;
        let live = self
            .live
            .as_mut()
            .ok_or(WasmProcessStatusV1::StaleSessionHandle)?;
        if command.handle.slot != SLOT || self.generation != Some(command.handle.generation) {
            return self.fail(WasmProcessStatusV1::StaleSessionHandle);
        }
        if command.expected_sequence != live.sequence {
            return self.fail(WasmProcessStatusV1::SequenceRejected);
        }
        if live.sequence == live.max_commands {
            return self.fail(WasmProcessStatusV1::SessionLimitReached);
        }
        let kind = execute_operation(live, command.operation);
        let accepted_sequence = live
            .sequence
            .checked_add(1)
            .ok_or(WasmProcessStatusV1::SessionLimitReached)?;
        live.sequence = accepted_sequence;
        let event = WasmBranchEventV1 {
            handle: command.handle,
            accepted_sequence,
            kind,
        };
        if matches!(event.kind, WasmBranchEventKindV1::Disposed) {
            self.live = None;
        }
        self.status = WasmProcessStatusV1::Ready;
        Ok(event)
    }

    fn install_event(&mut self, event: WasmBranchEventV1) -> Result<(), WasmProcessStatusV1> {
        let bytes = encode_wasm_branch_event_v1(&event)?;
        if bytes.len() > WASM_BRANCH_EVENT_LIMIT_V1 || bytes.len() > WASM_PROCESS_RESPONSE_LIMIT_V1
        {
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
    live: &mut LiveBranchV1,
    operation: WasmBranchOperationV1,
) -> WasmBranchEventKindV1 {
    match operation {
        WasmBranchOperationV1::AdmitAuthoritativeOccurrences(occurrences) => {
            admit_authoritative_occurrences(&mut live.authoritative, &occurrences)
        }
        WasmBranchOperationV1::ProposeReconnect(occurrences) => {
            match live.branch.resume_and_propose(&occurrences) {
                Ok(evidence) => match encode_process_reconnect_evidence_v1(&evidence) {
                    Ok(exact_evidence) => WasmBranchEventKindV1::ReconnectProposed {
                        evidence,
                        exact_evidence,
                    },
                    Err(_) => {
                        WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::UnexpectedCandidate)
                    }
                },
                Err(error) => WasmBranchEventKindV1::Rejected(map_branch_error(&error)),
            }
        }
        WasmBranchOperationV1::Adjudicate {
            reconnect_evidence,
            branch_candidate,
            authoritative_base,
            occurrences,
        } => {
            let Some(retained) = live.branch.proposal().cloned() else {
                return WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::MissingProposal);
            };
            if encode_process_reconnect_evidence_v1(&retained).as_deref()
                != Ok(reconnect_evidence.as_slice())
            {
                return WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::EvidenceMismatch);
            }
            let plan = CheckedReconnectAdmissionPlanV1 {
                branch_candidate,
                authoritative_base,
                occurrences,
            };
            match live
                .branch
                .adjudicate(&mut live.authoritative, &retained, &plan)
            {
                Ok(admitted) => reconnect_admitted_event(admitted),
                Err(error) => WasmBranchEventKindV1::Rejected(map_branch_error(&error)),
            }
        }
        WasmBranchOperationV1::Explain => match live.branch.explanation().cloned() {
            Some(explanation) => match encode_process_branch_explanation_v1(&explanation) {
                Ok(exact_explanation) => WasmBranchEventKindV1::Explanation {
                    explanation,
                    exact_explanation,
                },
                Err(_) => {
                    WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::MissingCausalRecord)
                }
            },
            None => WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::ExplanationUnavailable),
        },
        WasmBranchOperationV1::Dispose => {
            live.authoritative.dispose();
            WasmBranchEventKindV1::Disposed
        }
    }
}

fn admit_authoritative_occurrences(
    session: &mut super::PersistentProcessSessionV1,
    occurrences: &[Vec<u8>],
) -> WasmBranchEventKindV1 {
    let Some((last, prefix)) = occurrences.split_last() else {
        return WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::MissingOccurrence);
    };
    for occurrence in prefix {
        if session.apply_opaque_input(occurrence).is_err() {
            return WasmBranchEventKindV1::Rejected(
                WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
            );
        }
    }
    if session.apply_opaque_input_and_emit_candidate(last).is_err() {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    }
    let Some(candidate) = session
        .candidate()
        .ok()
        .flatten()
        .map(|candidate| candidate.id)
    else {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    };
    let Ok(authorization) = session.issue_candidate_admission_authorization() else {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    };
    let Ok((state, _)) = session.admit_issued_candidate_with_projection(authorization) else {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    };
    let Some(decision) = session
        .carrier()
        .ok()
        .and_then(|carrier| carrier.decision_by_occurrence(state.admission))
    else {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    };
    let Ok(run) = session.run() else {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    };
    let Ok(activation) = session.activation() else {
        return WasmBranchEventKindV1::Rejected(
            WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        );
    };
    WasmBranchEventKindV1::AuthoritativeAdmissionAccepted {
        candidate,
        predecessor: state.predecessor,
        successor: state.id,
        judgment: decision.verdict,
        admission: state.admission,
        run,
        activation,
    }
}

fn reconnect_admitted_event(admitted: ProcessReconnectAdmissionV1) -> WasmBranchEventKindV1 {
    let Ok(exact_explanation) = encode_process_branch_explanation_v1(&admitted.explanation) else {
        return WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::MissingCausalRecord);
    };
    WasmBranchEventKindV1::ReconnectAdmissionAccepted {
        predecessor: admitted.state.predecessor,
        successor: admitted.state.id,
        branch_candidate: admitted.explanation.branch_candidate,
        authoritative_candidate: admitted.explanation.authoritative_candidate,
        judgment: admitted.explanation.judgment,
        admission: admitted.explanation.admission,
        projection: admitted.projection.map(project_projection),
        explanation: admitted.explanation,
        exact_explanation,
    }
}

fn project_projection(projection: ExecutableProjectedObservationV1) -> WasmBranchProjectionV1 {
    WasmBranchProjectionV1 {
        observation: projection.id,
        exact_term_bytes: canonical_term_bytes(&projection.term)
            .expect("a checked projected Term remains canonical"),
    }
}

fn map_branch_error(error: &ProcessBranchErrorV1) -> WasmBranchRejectionV1 {
    match error {
        ProcessBranchErrorV1::PinMismatch(pin) => WasmBranchRejectionV1::PinMismatch(*pin),
        ProcessBranchErrorV1::MissingOccurrence => WasmBranchRejectionV1::MissingOccurrence,
        ProcessBranchErrorV1::AlreadyProposed => WasmBranchRejectionV1::AlreadyProposed,
        ProcessBranchErrorV1::MissingProposal => WasmBranchRejectionV1::MissingProposal,
        ProcessBranchErrorV1::AlreadyAdjudicated => WasmBranchRejectionV1::AlreadyAdjudicated,
        ProcessBranchErrorV1::UnexpectedCandidate => WasmBranchRejectionV1::UnexpectedCandidate,
        ProcessBranchErrorV1::MissingInputObservation(_) => {
            WasmBranchRejectionV1::UnexpectedCandidate
        }
        ProcessBranchErrorV1::MissingCausalRecord(_) => WasmBranchRejectionV1::MissingCausalRecord,
        ProcessBranchErrorV1::Session(_) => WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
    }
}

pub fn encode_wasm_branch_open_v1(
    request: &WasmBranchOpenV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    if request.max_commands == 0 || request.disconnect_occurrence.is_empty() {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(OPEN_MAGIC);
    put_blob(&mut bytes, &request.exact_cwr1)?;
    bytes.extend_from_slice(&request.disconnect_tick.to_le_bytes());
    put_blob(&mut bytes, &request.disconnect_occurrence)?;
    bytes.extend_from_slice(&request.max_commands.to_le_bytes());
    if bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_wasm_branch_open_v1(bytes: &[u8]) -> Result<WasmBranchOpenV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut decoder = Decoder::new(bytes);
    if decoder.take(4)? != OPEN_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let exact_cwr1 = decoder.blob(WASM_PROCESS_REQUEST_LIMIT_V1)?.to_vec();
    let disconnect_tick = decoder.u64()?;
    let disconnect_occurrence = decoder.blob(WASM_BRANCH_COMMAND_LIMIT_V1)?.to_vec();
    let max_commands = decoder.u64()?;
    if !decoder.is_complete() || max_commands == 0 || disconnect_occurrence.is_empty() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(WasmBranchOpenV1 {
        exact_cwr1,
        disconnect_tick,
        disconnect_occurrence,
        max_commands,
    })
}

pub fn encode_wasm_branch_command_v1(
    command: &WasmBranchCommandV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(COMMAND_MAGIC);
    put_handle(&mut bytes, command.handle);
    bytes.extend_from_slice(&command.expected_sequence.to_le_bytes());
    match &command.operation {
        WasmBranchOperationV1::AdmitAuthoritativeOccurrences(occurrences) => {
            bytes.push(1);
            put_occurrences(&mut bytes, occurrences)?;
        }
        WasmBranchOperationV1::ProposeReconnect(occurrences) => {
            bytes.push(2);
            put_occurrences(&mut bytes, occurrences)?;
        }
        WasmBranchOperationV1::Adjudicate {
            reconnect_evidence,
            branch_candidate,
            authoritative_base,
            occurrences,
        } => {
            bytes.push(3);
            put_blob(&mut bytes, reconnect_evidence)?;
            bytes.extend_from_slice(branch_candidate.as_bytes());
            bytes.extend_from_slice(authoritative_base.as_bytes());
            put_occurrences(&mut bytes, occurrences)?;
        }
        WasmBranchOperationV1::Explain => bytes.push(4),
        WasmBranchOperationV1::Dispose => bytes.push(5),
    }
    if bytes.len() > WASM_BRANCH_COMMAND_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_wasm_branch_command_v1(
    bytes: &[u8],
) -> Result<WasmBranchCommandV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_BRANCH_COMMAND_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut decoder = Decoder::new(bytes);
    if decoder.take(4)? != COMMAND_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let handle = get_handle(&mut decoder)?;
    let expected_sequence = decoder.u64()?;
    let operation = match decoder.take(1)?[0] {
        1 => WasmBranchOperationV1::AdmitAuthoritativeOccurrences(get_occurrences(&mut decoder)?),
        2 => WasmBranchOperationV1::ProposeReconnect(get_occurrences(&mut decoder)?),
        3 => WasmBranchOperationV1::Adjudicate {
            reconnect_evidence: decoder.blob(WASM_BRANCH_EVENT_LIMIT_V1)?.to_vec(),
            branch_candidate: CandidateDeltaId::from_bytes(decoder.identity()?),
            authoritative_base: StateRevisionId::from_bytes(decoder.identity()?),
            occurrences: get_occurrences(&mut decoder)?,
        },
        4 => WasmBranchOperationV1::Explain,
        5 => WasmBranchOperationV1::Dispose,
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    };
    if !decoder.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(WasmBranchCommandV1 {
        handle,
        expected_sequence,
        operation,
    })
}

pub fn encode_wasm_branch_event_v1(
    event: &WasmBranchEventV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(EVENT_MAGIC);
    put_handle(&mut bytes, event.handle);
    bytes.extend_from_slice(&event.accepted_sequence.to_le_bytes());
    match &event.kind {
        WasmBranchEventKindV1::Opened {
            pins,
            ancestry,
            suspension,
        } => {
            bytes.push(1);
            put_pins(&mut bytes, *pins);
            put_ancestry(&mut bytes, *ancestry);
            put_suspension(&mut bytes, *suspension);
        }
        WasmBranchEventKindV1::AuthoritativeAdmissionAccepted {
            candidate,
            predecessor,
            successor,
            judgment,
            admission,
            run,
            activation,
        } => {
            bytes.push(2);
            put_ids(
                &mut bytes,
                &[
                    candidate.as_bytes(),
                    predecessor.as_bytes(),
                    successor.as_bytes(),
                    judgment.as_bytes(),
                    admission.as_bytes(),
                    run.as_bytes(),
                    activation.as_bytes(),
                ],
            );
        }
        WasmBranchEventKindV1::ReconnectProposed {
            evidence: _,
            exact_evidence,
        } => {
            bytes.push(3);
            put_blob(&mut bytes, exact_evidence)?;
        }
        WasmBranchEventKindV1::ReconnectAdmissionAccepted {
            predecessor,
            successor,
            branch_candidate,
            authoritative_candidate,
            judgment,
            admission,
            projection,
            explanation: _,
            exact_explanation,
        } => {
            bytes.push(4);
            put_ids(
                &mut bytes,
                &[
                    predecessor.as_bytes(),
                    successor.as_bytes(),
                    branch_candidate.as_bytes(),
                    authoritative_candidate.as_bytes(),
                    judgment.as_bytes(),
                    admission.as_bytes(),
                ],
            );
            put_projection(&mut bytes, projection.as_ref())?;
            put_blob(&mut bytes, exact_explanation)?;
        }
        WasmBranchEventKindV1::Explanation {
            explanation: _,
            exact_explanation,
        } => {
            bytes.push(5);
            put_blob(&mut bytes, exact_explanation)?;
        }
        WasmBranchEventKindV1::Disposed => bytes.push(6),
        WasmBranchEventKindV1::Rejected(rejection) => {
            bytes.push(7);
            put_rejection(&mut bytes, *rejection);
        }
    }
    if bytes.len() > WASM_BRANCH_EVENT_LIMIT_V1 {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_wasm_branch_event_v1(bytes: &[u8]) -> Result<WasmBranchEventV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_BRANCH_EVENT_LIMIT_V1 {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    let mut decoder = Decoder::new(bytes);
    if decoder.take(4)? != EVENT_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let handle = get_handle(&mut decoder)?;
    let accepted_sequence = decoder.u64()?;
    let kind = match decoder.take(1)?[0] {
        1 => WasmBranchEventKindV1::Opened {
            pins: get_pins(&mut decoder)?,
            ancestry: get_ancestry(&mut decoder)?,
            suspension: get_suspension(&mut decoder)?,
        },
        2 => WasmBranchEventKindV1::AuthoritativeAdmissionAccepted {
            candidate: CandidateDeltaId::from_bytes(decoder.identity()?),
            predecessor: StateRevisionId::from_bytes(decoder.identity()?),
            successor: StateRevisionId::from_bytes(decoder.identity()?),
            judgment: JudgmentOccurrenceId::from_bytes(decoder.identity()?),
            admission: AdmissionOccurrenceId::from_bytes(decoder.identity()?),
            run: RunId::from_bytes(decoder.identity()?),
            activation: ActivationId::from_bytes(decoder.identity()?),
        },
        3 => {
            let exact_evidence = decoder.blob(WASM_BRANCH_EVENT_LIMIT_V1)?.to_vec();
            WasmBranchEventKindV1::ReconnectProposed {
                evidence: decode_process_reconnect_evidence_v1(&exact_evidence)?,
                exact_evidence,
            }
        }
        4 => {
            let predecessor = StateRevisionId::from_bytes(decoder.identity()?);
            let successor = StateRevisionId::from_bytes(decoder.identity()?);
            let branch_candidate = CandidateDeltaId::from_bytes(decoder.identity()?);
            let authoritative_candidate = CandidateDeltaId::from_bytes(decoder.identity()?);
            let judgment = JudgmentOccurrenceId::from_bytes(decoder.identity()?);
            let admission = AdmissionOccurrenceId::from_bytes(decoder.identity()?);
            let projection = get_projection(&mut decoder)?;
            let exact_explanation = decoder.blob(WASM_BRANCH_EVENT_LIMIT_V1)?.to_vec();
            let explanation = decode_process_branch_explanation_v1(&exact_explanation)?;
            WasmBranchEventKindV1::ReconnectAdmissionAccepted {
                predecessor,
                successor,
                branch_candidate,
                authoritative_candidate,
                judgment,
                admission,
                projection,
                explanation,
                exact_explanation,
            }
        }
        5 => {
            let exact_explanation = decoder.blob(WASM_BRANCH_EVENT_LIMIT_V1)?.to_vec();
            WasmBranchEventKindV1::Explanation {
                explanation: decode_process_branch_explanation_v1(&exact_explanation)?,
                exact_explanation,
            }
        }
        6 => WasmBranchEventKindV1::Disposed,
        7 => WasmBranchEventKindV1::Rejected(get_rejection(&mut decoder)?),
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    };
    if !decoder.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(WasmBranchEventV1 {
        handle,
        accepted_sequence,
        kind,
    })
}

pub fn encode_process_reconnect_evidence_v1(
    evidence: &ProcessReconnectEvidenceV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(EVIDENCE_MAGIC);
    put_evidence(&mut bytes, evidence)?;
    if bytes.len() > WASM_BRANCH_EVENT_LIMIT_V1 {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_process_reconnect_evidence_v1(
    bytes: &[u8],
) -> Result<ProcessReconnectEvidenceV1, WasmProcessStatusV1> {
    let mut decoder = Decoder::new(bytes);
    if decoder.take(4)? != EVIDENCE_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let evidence = get_evidence(&mut decoder)?;
    if !decoder.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(evidence)
}

pub fn encode_process_branch_explanation_v1(
    explanation: &ProcessBranchExplanationV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(EXPLANATION_MAGIC);
    put_explanation(&mut bytes, explanation)?;
    if bytes.len() > WASM_BRANCH_EVENT_LIMIT_V1 {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    Ok(bytes)
}

pub fn decode_process_branch_explanation_v1(
    bytes: &[u8],
) -> Result<ProcessBranchExplanationV1, WasmProcessStatusV1> {
    let mut decoder = Decoder::new(bytes);
    if decoder.take(4)? != EXPLANATION_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let explanation = get_explanation(&mut decoder)?;
    if !decoder.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(explanation)
}

fn put_handle(bytes: &mut Vec<u8>, handle: WasmBranchHandleV1) {
    bytes.extend_from_slice(&handle.slot.to_le_bytes());
    bytes.extend_from_slice(&handle.generation.to_le_bytes());
}

fn get_handle(decoder: &mut Decoder<'_>) -> Result<WasmBranchHandleV1, WasmProcessStatusV1> {
    Ok(WasmBranchHandleV1 {
        slot: decoder.u32()?,
        generation: decoder.u32()?,
    })
}

fn put_occurrences(
    bytes: &mut Vec<u8>,
    occurrences: &[Vec<u8>],
) -> Result<(), WasmProcessStatusV1> {
    if occurrences.is_empty() || occurrences.len() > MAX_OCCURRENCES {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    put_count(bytes, occurrences.len())?;
    for occurrence in occurrences {
        if occurrence.is_empty() {
            return Err(WasmProcessStatusV1::RequestOutOfBounds);
        }
        put_blob(bytes, occurrence)?;
    }
    Ok(())
}

fn get_occurrences(decoder: &mut Decoder<'_>) -> Result<Vec<Vec<u8>>, WasmProcessStatusV1> {
    let count = usize::from(decoder.u16()?);
    if count == 0 || count > MAX_OCCURRENCES {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    (0..count)
        .map(|_| {
            let occurrence = decoder.blob(WASM_BRANCH_COMMAND_LIMIT_V1)?.to_vec();
            if occurrence.is_empty() {
                Err(WasmProcessStatusV1::MalformedRequest)
            } else {
                Ok(occurrence)
            }
        })
        .collect()
}

fn put_pins(bytes: &mut Vec<u8>, pins: ProcessBranchPinsV1) {
    put_ids(
        bytes,
        &[
            pins.parent_state.as_bytes(),
            pins.program_revision.as_bytes(),
            pins.package.as_bytes(),
            pins.application.snapshot.as_bytes(),
        ],
    );
    bytes.extend_from_slice(&pins.application.local.get().to_le_bytes());
    put_ids(
        bytes,
        &[
            pins.session.as_bytes(),
            pins.runtime_policy.as_bytes(),
            pins.root_policy.as_bytes(),
            pins.input_evidence.as_bytes(),
            pins.physical_plan.as_bytes(),
        ],
    );
    bytes.extend_from_slice(&pins.budget_units.to_le_bytes());
    bytes.extend_from_slice(&pins.disconnect_tick.to_le_bytes());
}

fn get_pins(decoder: &mut Decoder<'_>) -> Result<ProcessBranchPinsV1, WasmProcessStatusV1> {
    Ok(ProcessBranchPinsV1 {
        parent_state: StateRevisionId::from_bytes(decoder.identity()?),
        program_revision: ProgramRevisionId::from_bytes(decoder.identity()?),
        package: ProcessPackageId::from_bytes(decoder.identity()?),
        application: ApplicationId {
            snapshot: ProgramSnapshotId::from_bytes(decoder.identity()?),
            local: ApplicationLocalId::new(decoder.u32()?),
        },
        session: RuntimeSessionId::from_bytes(decoder.identity()?),
        runtime_policy: RuntimePolicyId::from_bytes(decoder.identity()?),
        root_policy: RootPolicyId::from_bytes(decoder.identity()?),
        input_evidence: ExternalEvidenceRef::from_bytes(decoder.identity()?),
        physical_plan: ExecutablePhysicalPlanIdV1::from_bytes(decoder.identity()?),
        budget_units: decoder.u64()?,
        disconnect_tick: decoder.u64()?,
    })
}

fn put_ancestry(bytes: &mut Vec<u8>, ancestry: ProcessBranchAncestryV1) {
    put_ids(
        bytes,
        &[
            ancestry.parent_state.as_bytes(),
            ancestry.run.as_bytes(),
            ancestry.activation.as_bytes(),
            ancestry.disconnect_step.as_bytes(),
            ancestry.suspension_step.as_bytes(),
            ancestry.continuation.as_bytes(),
        ],
    );
}

fn get_ancestry(decoder: &mut Decoder<'_>) -> Result<ProcessBranchAncestryV1, WasmProcessStatusV1> {
    Ok(ProcessBranchAncestryV1 {
        parent_state: StateRevisionId::from_bytes(decoder.identity()?),
        run: RunId::from_bytes(decoder.identity()?),
        activation: ActivationId::from_bytes(decoder.identity()?),
        disconnect_step: StepId::from_bytes(decoder.identity()?),
        suspension_step: StepId::from_bytes(decoder.identity()?),
        continuation: ContinuationId::from_bytes(decoder.identity()?),
    })
}

fn put_suspension(bytes: &mut Vec<u8>, suspension: ExecutableSuspensionV1) {
    put_ids(
        bytes,
        &[
            suspension.step.as_bytes(),
            suspension.continuation.as_bytes(),
            suspension.run.as_bytes(),
            suspension.activation.as_bytes(),
            suspension.before.as_bytes(),
            suspension.after.as_bytes(),
        ],
    );
    bytes.extend_from_slice(&suspension.remaining_budget.to_le_bytes());
}

fn get_suspension(
    decoder: &mut Decoder<'_>,
) -> Result<ExecutableSuspensionV1, WasmProcessStatusV1> {
    Ok(ExecutableSuspensionV1 {
        step: StepId::from_bytes(decoder.identity()?),
        continuation: ContinuationId::from_bytes(decoder.identity()?),
        run: RunId::from_bytes(decoder.identity()?),
        activation: ActivationId::from_bytes(decoder.identity()?),
        before: ConfigurationId::from_bytes(decoder.identity()?),
        after: ConfigurationId::from_bytes(decoder.identity()?),
        remaining_budget: decoder.u64()?,
    })
}

fn put_resumption(bytes: &mut Vec<u8>, resumption: ExecutableResumptionV1) {
    put_ids(
        bytes,
        &[
            resumption.occurrence.as_bytes(),
            resumption.step.as_bytes(),
            resumption.continuation.as_bytes(),
            resumption.run.as_bytes(),
            resumption.activation.as_bytes(),
            resumption.before.as_bytes(),
            resumption.after.as_bytes(),
        ],
    );
    bytes.extend_from_slice(&resumption.remaining_budget.to_le_bytes());
}

fn get_resumption(
    decoder: &mut Decoder<'_>,
) -> Result<ExecutableResumptionV1, WasmProcessStatusV1> {
    Ok(ExecutableResumptionV1 {
        occurrence: ResumptionOccurrenceId::from_bytes(decoder.identity()?),
        step: StepId::from_bytes(decoder.identity()?),
        continuation: ContinuationId::from_bytes(decoder.identity()?),
        run: RunId::from_bytes(decoder.identity()?),
        activation: ActivationId::from_bytes(decoder.identity()?),
        before: ConfigurationId::from_bytes(decoder.identity()?),
        after: ConfigurationId::from_bytes(decoder.identity()?),
        remaining_budget: decoder.u64()?,
    })
}

fn put_evidence(
    bytes: &mut Vec<u8>,
    evidence: &ProcessReconnectEvidenceV1,
) -> Result<(), WasmProcessStatusV1> {
    put_pins(bytes, evidence.pins);
    put_ancestry(bytes, evidence.ancestry);
    put_resumption(bytes, evidence.resumption);
    put_command_evidence(bytes, &evidence.command_evidence)?;
    if evidence
        .command_evidence
        .last()
        .is_none_or(|command| command.step != evidence.candidate_step)
    {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    bytes.extend_from_slice(evidence.candidate.as_bytes());
    bytes.extend_from_slice(evidence.candidate_step.as_bytes());
    Ok(())
}

fn get_evidence(
    decoder: &mut Decoder<'_>,
) -> Result<ProcessReconnectEvidenceV1, WasmProcessStatusV1> {
    let pins = get_pins(decoder)?;
    let ancestry = get_ancestry(decoder)?;
    let resumption = get_resumption(decoder)?;
    let command_evidence = get_command_evidence(decoder)?;
    let candidate = CandidateDeltaId::from_bytes(decoder.identity()?);
    let candidate_step = StepId::from_bytes(decoder.identity()?);
    if command_evidence
        .last()
        .is_none_or(|command| command.step != candidate_step)
    {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(ProcessReconnectEvidenceV1 {
        pins,
        ancestry,
        resumption,
        command_evidence,
        candidate,
        candidate_step,
    })
}

fn put_explanation(
    bytes: &mut Vec<u8>,
    explanation: &ProcessBranchExplanationV1,
) -> Result<(), WasmProcessStatusV1> {
    put_pins(bytes, explanation.pins);
    put_ancestry(bytes, explanation.ancestry);
    put_resumption(bytes, explanation.resumption);
    put_command_evidence(bytes, &explanation.branch_command_evidence)?;
    put_ids(
        bytes,
        &[
            explanation.branch_candidate.as_bytes(),
            explanation.authoritative_base.as_bytes(),
            explanation.authoritative_run.as_bytes(),
            explanation.authoritative_activation.as_bytes(),
        ],
    );
    put_command_evidence(bytes, &explanation.authoritative_command_evidence)?;
    put_ids(
        bytes,
        &[
            explanation.authoritative_candidate.as_bytes(),
            explanation.authorization.as_bytes(),
            explanation.judgment.as_bytes(),
            explanation.admission.as_bytes(),
            explanation.successor.as_bytes(),
        ],
    );
    if explanation.causal_records.len() > MAX_CAUSAL_RECORDS {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    put_count(bytes, explanation.causal_records.len())?;
    for record in &explanation.causal_records {
        put_causal_ref(bytes, record.occurrence);
        if record.predecessors.len() > MAX_CAUSAL_RECORDS {
            return Err(WasmProcessStatusV1::ResponseOutOfBounds);
        }
        put_count(bytes, record.predecessors.len())?;
        for predecessor in &record.predecessors {
            put_causal_ref(bytes, *predecessor);
        }
    }
    Ok(())
}

fn get_explanation(
    decoder: &mut Decoder<'_>,
) -> Result<ProcessBranchExplanationV1, WasmProcessStatusV1> {
    let pins = get_pins(decoder)?;
    let ancestry = get_ancestry(decoder)?;
    let resumption = get_resumption(decoder)?;
    let branch_command_evidence = get_command_evidence(decoder)?;
    let branch_candidate = CandidateDeltaId::from_bytes(decoder.identity()?);
    let authoritative_base = StateRevisionId::from_bytes(decoder.identity()?);
    let authoritative_run = RunId::from_bytes(decoder.identity()?);
    let authoritative_activation = ActivationId::from_bytes(decoder.identity()?);
    let authoritative_command_evidence = get_command_evidence(decoder)?;
    let authoritative_candidate = CandidateDeltaId::from_bytes(decoder.identity()?);
    let authorization = IssuedAdmissionAuthorizationOccurrenceId::from_bytes(decoder.identity()?);
    let judgment = JudgmentOccurrenceId::from_bytes(decoder.identity()?);
    let admission = AdmissionOccurrenceId::from_bytes(decoder.identity()?);
    let successor = StateRevisionId::from_bytes(decoder.identity()?);
    let count = usize::from(decoder.u16()?);
    if count > MAX_CAUSAL_RECORDS {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    let mut causal_records = Vec::with_capacity(count);
    for _ in 0..count {
        let occurrence = get_causal_ref(decoder)?;
        let predecessor_count = usize::from(decoder.u16()?);
        if predecessor_count > MAX_CAUSAL_RECORDS {
            return Err(WasmProcessStatusV1::ResponseOutOfBounds);
        }
        let predecessors = (0..predecessor_count)
            .map(|_| get_causal_ref(decoder))
            .collect::<Result<Vec<_>, _>>()?;
        causal_records.push(ProcessCausalRecordV1 {
            occurrence,
            predecessors,
        });
    }
    Ok(ProcessBranchExplanationV1 {
        pins,
        ancestry,
        resumption,
        branch_command_evidence,
        branch_candidate,
        authoritative_base,
        authoritative_run,
        authoritative_activation,
        authoritative_command_evidence,
        authoritative_candidate,
        authorization,
        judgment,
        admission,
        successor,
        causal_records,
    })
}

fn put_projection(
    bytes: &mut Vec<u8>,
    projection: Option<&WasmBranchProjectionV1>,
) -> Result<(), WasmProcessStatusV1> {
    match projection {
        None => bytes.push(0),
        Some(projection) => {
            bytes.push(1);
            bytes.extend_from_slice(projection.observation.as_bytes());
            put_blob(bytes, &projection.exact_term_bytes)?;
        }
    }
    Ok(())
}

fn get_projection(
    decoder: &mut Decoder<'_>,
) -> Result<Option<WasmBranchProjectionV1>, WasmProcessStatusV1> {
    match decoder.take(1)?[0] {
        0 => Ok(None),
        1 => Ok(Some(WasmBranchProjectionV1 {
            observation: ObservationId::from_bytes(decoder.identity()?),
            exact_term_bytes: decoder.blob(WASM_BRANCH_EVENT_LIMIT_V1)?.to_vec(),
        })),
        _ => Err(WasmProcessStatusV1::MalformedRequest),
    }
}

fn put_command_evidence(
    bytes: &mut Vec<u8>,
    commands: &[ProcessCommandEvidenceV1],
) -> Result<(), WasmProcessStatusV1> {
    if commands.is_empty() || commands.len() > MAX_OCCURRENCES {
        return Err(WasmProcessStatusV1::ResponseOutOfBounds);
    }
    put_count(bytes, commands.len())?;
    for command in commands {
        if command.occurrence.is_empty() {
            return Err(WasmProcessStatusV1::MalformedRequest);
        }
        put_blob(bytes, &command.occurrence)?;
        put_ids(
            bytes,
            &[command.step.as_bytes(), command.observation.as_bytes()],
        );
    }
    Ok(())
}

fn get_command_evidence(
    decoder: &mut Decoder<'_>,
) -> Result<Vec<ProcessCommandEvidenceV1>, WasmProcessStatusV1> {
    let count = usize::from(decoder.u16()?);
    if count == 0 || count > MAX_OCCURRENCES {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    (0..count)
        .map(|_| {
            let occurrence = decoder.blob(WASM_BRANCH_COMMAND_LIMIT_V1)?.to_vec();
            if occurrence.is_empty() {
                return Err(WasmProcessStatusV1::MalformedRequest);
            }
            Ok(ProcessCommandEvidenceV1 {
                occurrence,
                step: StepId::from_bytes(decoder.identity()?),
                observation: ObservationId::from_bytes(decoder.identity()?),
            })
        })
        .collect()
}

fn put_causal_ref(bytes: &mut Vec<u8>, occurrence: CausalRef) {
    match occurrence {
        CausalRef::SessionStart(id) => put_causal_id(bytes, 0, id.as_bytes()),
        CausalRef::ExternalTrigger(id) => put_causal_id(bytes, 1, id.as_bytes()),
        CausalRef::Resumption(id) => put_causal_id(bytes, 2, id.as_bytes()),
        CausalRef::Handoff(id) => put_causal_id(bytes, 3, id.as_bytes()),
        CausalRef::Cancellation(id) => put_causal_id(bytes, 4, id.as_bytes()),
        CausalRef::Step(step) => {
            bytes.push(5);
            put_ids(
                bytes,
                &[
                    step.run.as_bytes(),
                    step.activation.as_bytes(),
                    step.step.as_bytes(),
                ],
            );
        }
        CausalRef::Observation(id) => put_causal_id(bytes, 6, id.as_bytes()),
        CausalRef::CandidateDelta(id) => put_causal_id(bytes, 7, id.as_bytes()),
        CausalRef::Judgment(id) => put_causal_id(bytes, 8, id.as_bytes()),
        CausalRef::Admission(id) => put_causal_id(bytes, 9, id.as_bytes()),
        CausalRef::EffectIntent(id) => put_causal_id(bytes, 10, id.as_bytes()),
        CausalRef::EffectAuthorization(id) => put_causal_id(bytes, 11, id.as_bytes()),
        CausalRef::EffectAttempt(id) => put_causal_id(bytes, 12, id.as_bytes()),
        CausalRef::EffectReceipt(id) => put_causal_id(bytes, 13, id.as_bytes()),
        CausalRef::EffectJudgment(id) => put_causal_id(bytes, 14, id.as_bytes()),
    }
}

fn put_causal_id(bytes: &mut Vec<u8>, tag: u8, identity: &[u8; IDENTITY_BYTES]) {
    bytes.push(tag);
    bytes.extend_from_slice(identity);
}

fn get_causal_ref(decoder: &mut Decoder<'_>) -> Result<CausalRef, WasmProcessStatusV1> {
    Ok(match decoder.take(1)?[0] {
        0 => CausalRef::SessionStart(SessionStartOccurrenceId::from_bytes(decoder.identity()?)),
        1 => {
            CausalRef::ExternalTrigger(ExternalTriggerOccurrenceId::from_bytes(decoder.identity()?))
        }
        2 => CausalRef::Resumption(ResumptionOccurrenceId::from_bytes(decoder.identity()?)),
        3 => CausalRef::Handoff(HandoffOccurrenceId::from_bytes(decoder.identity()?)),
        4 => CausalRef::Cancellation(CancellationOccurrenceId::from_bytes(decoder.identity()?)),
        5 => CausalRef::Step(StepRef {
            run: RunId::from_bytes(decoder.identity()?),
            activation: ActivationId::from_bytes(decoder.identity()?),
            step: StepId::from_bytes(decoder.identity()?),
        }),
        6 => CausalRef::Observation(ObservationId::from_bytes(decoder.identity()?)),
        7 => CausalRef::CandidateDelta(CandidateDeltaId::from_bytes(decoder.identity()?)),
        8 => CausalRef::Judgment(JudgmentOccurrenceId::from_bytes(decoder.identity()?)),
        9 => CausalRef::Admission(AdmissionOccurrenceId::from_bytes(decoder.identity()?)),
        10 => CausalRef::EffectIntent(EffectIntentId::from_bytes(decoder.identity()?)),
        11 => CausalRef::EffectAuthorization(IssuedEffectAuthorizationOccurrenceId::from_bytes(
            decoder.identity()?,
        )),
        12 => CausalRef::EffectAttempt(EffectAttemptId::from_bytes(decoder.identity()?)),
        13 => CausalRef::EffectReceipt(EffectReceiptId::from_bytes(decoder.identity()?)),
        14 => {
            CausalRef::EffectJudgment(EffectJudgmentOccurrenceId::from_bytes(decoder.identity()?))
        }
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    })
}

fn put_rejection(bytes: &mut Vec<u8>, rejection: WasmBranchRejectionV1) {
    match rejection {
        WasmBranchRejectionV1::MissingOccurrence => bytes.push(0),
        WasmBranchRejectionV1::AuthoritativeAdmissionRejected => bytes.push(1),
        WasmBranchRejectionV1::AlreadyProposed => bytes.push(2),
        WasmBranchRejectionV1::MissingProposal => bytes.push(3),
        WasmBranchRejectionV1::AlreadyAdjudicated => bytes.push(4),
        WasmBranchRejectionV1::EvidenceMismatch => bytes.push(5),
        WasmBranchRejectionV1::UnexpectedCandidate => bytes.push(6),
        WasmBranchRejectionV1::PinMismatch(pin) => {
            bytes.push(7);
            bytes.push(pin_tag(pin));
        }
        WasmBranchRejectionV1::MissingCausalRecord => bytes.push(8),
        WasmBranchRejectionV1::ExplanationUnavailable => bytes.push(9),
    }
}

fn get_rejection(decoder: &mut Decoder<'_>) -> Result<WasmBranchRejectionV1, WasmProcessStatusV1> {
    Ok(match decoder.take(1)?[0] {
        0 => WasmBranchRejectionV1::MissingOccurrence,
        1 => WasmBranchRejectionV1::AuthoritativeAdmissionRejected,
        2 => WasmBranchRejectionV1::AlreadyProposed,
        3 => WasmBranchRejectionV1::MissingProposal,
        4 => WasmBranchRejectionV1::AlreadyAdjudicated,
        5 => WasmBranchRejectionV1::EvidenceMismatch,
        6 => WasmBranchRejectionV1::UnexpectedCandidate,
        7 => WasmBranchRejectionV1::PinMismatch(pin_from_tag(decoder.take(1)?[0])?),
        8 => WasmBranchRejectionV1::MissingCausalRecord,
        9 => WasmBranchRejectionV1::ExplanationUnavailable,
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
    })
}

fn pin_tag(pin: ProcessBranchPinV1) -> u8 {
    match pin {
        ProcessBranchPinV1::ParentState => 0,
        ProcessBranchPinV1::ProgramRevision => 1,
        ProcessBranchPinV1::Package => 2,
        ProcessBranchPinV1::Application => 3,
        ProcessBranchPinV1::Session => 4,
        ProcessBranchPinV1::RuntimePolicy => 5,
        ProcessBranchPinV1::RootPolicy => 6,
        ProcessBranchPinV1::InputEvidence => 7,
        ProcessBranchPinV1::PhysicalPlan => 8,
        ProcessBranchPinV1::Budget => 9,
        ProcessBranchPinV1::Allocation => 10,
        ProcessBranchPinV1::BranchCandidate => 11,
        ProcessBranchPinV1::AuthoritativeBase => 12,
    }
}

fn pin_from_tag(tag: u8) -> Result<ProcessBranchPinV1, WasmProcessStatusV1> {
    Ok(match tag {
        0 => ProcessBranchPinV1::ParentState,
        1 => ProcessBranchPinV1::ProgramRevision,
        2 => ProcessBranchPinV1::Package,
        3 => ProcessBranchPinV1::Application,
        4 => ProcessBranchPinV1::Session,
        5 => ProcessBranchPinV1::RuntimePolicy,
        6 => ProcessBranchPinV1::RootPolicy,
        7 => ProcessBranchPinV1::InputEvidence,
        8 => ProcessBranchPinV1::PhysicalPlan,
        9 => ProcessBranchPinV1::Budget,
        10 => ProcessBranchPinV1::Allocation,
        11 => ProcessBranchPinV1::BranchCandidate,
        12 => ProcessBranchPinV1::AuthoritativeBase,
        _ => return Err(WasmProcessStatusV1::MalformedRequest),
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

    use super::{WasmProcessBranchBoundaryV1, WasmProcessStatusV1};

    thread_local! {
        static BRANCH_BOUNDARY: RefCell<WasmProcessBranchBoundaryV1> =
            RefCell::new(WasmProcessBranchBoundaryV1::new());
    }

    #[wasm_bindgen]
    pub fn clause_branch_v1_io_reset() {
        BRANCH_BOUNDARY.with_borrow_mut(WasmProcessBranchBoundaryV1::clear_io);
    }

    #[wasm_bindgen]
    pub fn clause_branch_v1_request_push(byte: u32) -> u32 {
        let Ok(byte) = u8::try_from(byte) else {
            return WasmProcessStatusV1::MalformedRequest as u32;
        };
        BRANCH_BOUNDARY.with_borrow_mut(|boundary| match boundary.push_request_byte(byte) {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_branch_v1_open() -> u32 {
        BRANCH_BOUNDARY.with_borrow_mut(|boundary| match boundary.open_buffered() {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_branch_v1_command() -> u32 {
        BRANCH_BOUNDARY.with_borrow_mut(|boundary| match boundary.command_buffered() {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_branch_v1_event_len() -> u32 {
        BRANCH_BOUNDARY
            .with_borrow(|boundary| u32::try_from(boundary.event().len()).unwrap_or(u32::MAX))
    }

    /// Values 0..=255 are event bytes; 256 means an out-of-range index.
    #[wasm_bindgen]
    pub fn clause_branch_v1_event_byte(index: u32) -> u32 {
        BRANCH_BOUNDARY.with_borrow(|boundary| {
            usize::try_from(index)
                .ok()
                .and_then(|index| boundary.event().get(index))
                .map_or(256, |byte| u32::from(*byte))
        })
    }
}
