//! Bounded, construct-blind byte boundary for WebAssembly hosts.

use std::error::Error;
use std::fmt;

use clause_package::*;

use super::{
    ExecutableAuthorityFactsV1, ExecutableBoundaryFactV1, ExecutableProcessRuntimeV1,
    ExecutableValueV1, RuntimeAllocationEpochV1, decode_executable_occurrence_v1,
    decode_executable_physical_plan_v1, decode_runtime_allocation_epoch_v1,
    encode_runtime_allocation_epoch_v1,
};

const REQUEST_MAGIC: &[u8; 4] = b"CWR1";
const RESPONSE_MAGIC: &[u8; 4] = b"CWO1";
const MAX_OCCURRENCES: usize = 256;
const MAX_RENDER_SLOTS: usize = 256;
pub(super) const MAX_EVIDENCE_BYTES: usize = 64 * 1024;

pub const WASM_PROCESS_REQUEST_LIMIT_V1: usize = 4 * 1024 * 1024;
pub const WASM_PROCESS_RESPONSE_LIMIT_V1: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum WasmProcessStatusV1 {
    Ready = 0,
    RequestOutOfBounds = 1,
    ResponseOutOfBounds = 2,
    MalformedRequest = 3,
    PackageRejected = 4,
    AuthorityRejected = 5,
    ProcessRejected = 6,
    SessionOccupied = 7,
    StaleSessionHandle = 8,
    SequenceRejected = 9,
    SessionExhausted = 10,
    SessionLimitReached = 11,
}

impl fmt::Display for WasmProcessStatusV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for WasmProcessStatusV1 {}

/// Host-established identities and exact evidence. Decoding these bytes does
/// not grant authority; dispatch explicitly establishes the matching in-memory
/// anchors before `ProcessCarrier` sees an occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmAuthorityInputV1 {
    pub program: ProgramId,
    pub change: ProgramChangeOccurrenceId,
    pub session: RuntimeSessionId,
    pub policy: RuntimePolicyId,
    pub session_start: SessionStartOccurrenceId,
    pub root_policy: RootPolicyId,
    pub occurrence_boundary: BoundaryRef,
    pub state_boundary: BoundaryRef,
    pub occurrence_evidence: ExternalEvidenceRef,
    pub occurrence_evidence_bytes: Vec<u8>,
    pub judgment_evidence: ExternalEvidenceRef,
    pub judgment_evidence_bytes: Vec<u8>,
    pub admission_evidence: ExternalEvidenceRef,
    pub admission_evidence_bytes: Vec<u8>,
    pub budget_units: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmProcessRequestV1 {
    pub package_bytes: Vec<u8>,
    pub application: ApplicationLocalId,
    pub physical_plan_bytes: Vec<u8>,
    pub allocation: RuntimeAllocationEpochV1,
    pub authority: WasmAuthorityInputV1,
    pub occurrences: Vec<Vec<u8>>,
    pub render_slots: Vec<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmProcessObservationV1 {
    pub observation: ObservationId,
    pub state: StateRevisionId,
    pub exact_value_bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct WasmProcessBuffersV1 {
    request: Vec<u8>,
    request_len: usize,
    response: Vec<u8>,
    response_len: usize,
    status: WasmProcessStatusV1,
}

impl Default for WasmProcessBuffersV1 {
    fn default() -> Self {
        Self {
            request: vec![0; WASM_PROCESS_REQUEST_LIMIT_V1],
            request_len: 0,
            response: vec![0; WASM_PROCESS_RESPONSE_LIMIT_V1],
            response_len: 0,
            status: WasmProcessStatusV1::Ready,
        }
    }
}

impl WasmProcessBuffersV1 {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.request_len = 0;
        self.response_len = 0;
        self.status = WasmProcessStatusV1::Ready;
    }

    pub fn push_request_byte(&mut self, byte: u8) -> Result<(), WasmProcessStatusV1> {
        if self.request_len == WASM_PROCESS_REQUEST_LIMIT_V1 {
            self.status = WasmProcessStatusV1::RequestOutOfBounds;
            return Err(self.status);
        }
        self.request[self.request_len] = byte;
        self.request_len += 1;
        Ok(())
    }

    pub fn dispatch(&mut self) -> Result<(), WasmProcessStatusV1> {
        self.response_len = 0;
        let request = decode_wasm_process_request_v1(self.request()).inspect_err(|error| {
            self.status = *error;
        })?;
        let observation = run_wasm_process_request_v1(&request).inspect_err(|error| {
            self.status = *error;
        })?;
        let response = encode_wasm_process_observation_v1(&observation);
        if response.len() > WASM_PROCESS_RESPONSE_LIMIT_V1 {
            self.status = WasmProcessStatusV1::ResponseOutOfBounds;
            return Err(self.status);
        }
        self.response[..response.len()].copy_from_slice(&response);
        self.response_len = response.len();
        self.status = WasmProcessStatusV1::Ready;
        Ok(())
    }

    #[must_use]
    pub fn request(&self) -> &[u8] {
        &self.request[..self.request_len]
    }

    #[must_use]
    pub fn response(&self) -> &[u8] {
        &self.response[..self.response_len]
    }

    #[must_use]
    pub const fn status(&self) -> WasmProcessStatusV1 {
        self.status
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use std::cell::RefCell;

    use wasm_bindgen::prelude::wasm_bindgen;

    use super::{WasmProcessBuffersV1, WasmProcessStatusV1};

    thread_local! {
        static BOUNDARY: RefCell<WasmProcessBuffersV1> =
            RefCell::new(WasmProcessBuffersV1::new());
    }

    #[wasm_bindgen]
    pub fn clause_process_v1_reset() {
        BOUNDARY.with_borrow_mut(WasmProcessBuffersV1::reset);
    }

    #[wasm_bindgen]
    pub fn clause_process_v1_request_push(byte: u32) -> u32 {
        let Ok(byte) = u8::try_from(byte) else {
            return WasmProcessStatusV1::MalformedRequest as u32;
        };
        BOUNDARY.with_borrow_mut(|boundary| match boundary.push_request_byte(byte) {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_process_v1_dispatch() -> u32 {
        BOUNDARY.with_borrow_mut(|boundary| match boundary.dispatch() {
            Ok(()) => WasmProcessStatusV1::Ready as u32,
            Err(error) => error as u32,
        })
    }

    #[wasm_bindgen]
    pub fn clause_process_v1_response_len() -> u32 {
        BOUNDARY
            .with_borrow(|boundary| u32::try_from(boundary.response().len()).unwrap_or(u32::MAX))
    }

    /// Values 0..=255 are response bytes; 256 means an out-of-range index.
    #[wasm_bindgen]
    pub fn clause_process_v1_response_byte(index: u32) -> u32 {
        BOUNDARY.with_borrow(|boundary| {
            usize::try_from(index)
                .ok()
                .and_then(|index| boundary.response().get(index))
                .map_or(256, |byte| u32::from(*byte))
        })
    }
}

pub fn encode_wasm_process_request_v1(
    request: &WasmProcessRequestV1,
) -> Result<Vec<u8>, WasmProcessStatusV1> {
    validate_shape(request)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(REQUEST_MAGIC);
    put_blob(&mut bytes, &request.package_bytes)?;
    bytes.extend_from_slice(&request.application.get().to_le_bytes());
    put_blob(&mut bytes, &request.physical_plan_bytes)?;
    put_blob(
        &mut bytes,
        &encode_runtime_allocation_epoch_v1(request.allocation),
    )?;
    encode_wasm_authority_input_v1(&mut bytes, &request.authority)?;
    let a = &request.authority;
    bytes.extend_from_slice(&a.budget_units.to_le_bytes());
    put_count(&mut bytes, request.occurrences.len())?;
    for occurrence in &request.occurrences {
        put_blob(&mut bytes, occurrence)?;
    }
    put_count(&mut bytes, request.render_slots.len())?;
    for slot in &request.render_slots {
        bytes.extend_from_slice(&slot.to_le_bytes());
    }
    if bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(bytes)
}

pub(super) fn encode_wasm_authority_input_v1(
    bytes: &mut Vec<u8>,
    a: &WasmAuthorityInputV1,
) -> Result<(), WasmProcessStatusV1> {
    for id in [
        a.program.as_bytes(),
        a.change.as_bytes(),
        a.session.as_bytes(),
        a.policy.as_bytes(),
        a.session_start.as_bytes(),
        a.root_policy.as_bytes(),
        a.occurrence_boundary.as_bytes(),
        a.state_boundary.as_bytes(),
        a.occurrence_evidence.as_bytes(),
    ] {
        bytes.extend_from_slice(id);
    }
    put_blob(bytes, &a.occurrence_evidence_bytes)?;
    bytes.extend_from_slice(a.judgment_evidence.as_bytes());
    put_blob(bytes, &a.judgment_evidence_bytes)?;
    bytes.extend_from_slice(a.admission_evidence.as_bytes());
    put_blob(bytes, &a.admission_evidence_bytes)?;
    Ok(())
}

pub fn decode_wasm_process_request_v1(
    bytes: &[u8],
) -> Result<WasmProcessRequestV1, WasmProcessStatusV1> {
    if bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1 {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    let mut d = Decoder::new(bytes);
    if d.take(4)? != REQUEST_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let package_bytes = d.blob(WASM_PROCESS_REQUEST_LIMIT_V1)?.to_vec();
    let application = ApplicationLocalId::new(d.u32()?);
    let physical_plan_bytes = d.blob(WASM_PROCESS_REQUEST_LIMIT_V1)?.to_vec();
    let allocation = decode_runtime_allocation_epoch_v1(d.blob(WASM_PROCESS_REQUEST_LIMIT_V1)?)
        .map_err(|_| WasmProcessStatusV1::MalformedRequest)?;
    let mut authority = decode_wasm_authority_input_v1(&mut d)?;
    authority.budget_units = d.u64()?;
    let occurrence_count = d.count(MAX_OCCURRENCES)?;
    let occurrences = (0..occurrence_count)
        .map(|_| d.blob(WASM_PROCESS_REQUEST_LIMIT_V1).map(<[u8]>::to_vec))
        .collect::<Result<Vec<_>, _>>()?;
    let slot_count = d.count(MAX_RENDER_SLOTS)?;
    let render_slots = (0..slot_count)
        .map(|_| d.u16())
        .collect::<Result<Vec<_>, _>>()?;
    if !d.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let request = WasmProcessRequestV1 {
        package_bytes,
        application,
        physical_plan_bytes,
        allocation,
        authority,
        occurrences,
        render_slots,
    };
    validate_shape(&request)?;
    Ok(request)
}

pub(super) fn decode_wasm_authority_input_v1(
    d: &mut Decoder<'_>,
) -> Result<WasmAuthorityInputV1, WasmProcessStatusV1> {
    Ok(WasmAuthorityInputV1 {
        program: ProgramId::from_bytes(d.identity()?),
        change: ProgramChangeOccurrenceId::from_bytes(d.identity()?),
        session: RuntimeSessionId::from_bytes(d.identity()?),
        policy: RuntimePolicyId::from_bytes(d.identity()?),
        session_start: SessionStartOccurrenceId::from_bytes(d.identity()?),
        root_policy: RootPolicyId::from_bytes(d.identity()?),
        occurrence_boundary: BoundaryRef::from_bytes(d.identity()?),
        state_boundary: BoundaryRef::from_bytes(d.identity()?),
        occurrence_evidence: ExternalEvidenceRef::from_bytes(d.identity()?),
        occurrence_evidence_bytes: d.blob(MAX_EVIDENCE_BYTES)?.to_vec(),
        judgment_evidence: ExternalEvidenceRef::from_bytes(d.identity()?),
        judgment_evidence_bytes: d.blob(MAX_EVIDENCE_BYTES)?.to_vec(),
        admission_evidence: ExternalEvidenceRef::from_bytes(d.identity()?),
        admission_evidence_bytes: d.blob(MAX_EVIDENCE_BYTES)?.to_vec(),
        budget_units: 0,
    })
}

pub fn run_wasm_process_request_v1(
    request: &WasmProcessRequestV1,
) -> Result<WasmProcessObservationV1, WasmProcessStatusV1> {
    validate_shape(request)?;
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
    let (authority, facts, admission_authorization) = establish_authority(
        &package,
        &request.authority,
        request.allocation.candidate_id(0),
    )?;
    let mut runtime = ExecutableProcessRuntimeV1::instantiate_rematerialized(
        package,
        authority,
        application,
        physical_plan,
        facts,
        request.allocation,
    )
    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
    runtime
        .start_carrier_process(facts)
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
    let (last, prefix) = request
        .occurrences
        .split_last()
        .ok_or(WasmProcessStatusV1::MalformedRequest)?;
    for bytes in prefix {
        let occurrence = decode_executable_occurrence_v1(bytes)
            .map_err(|_| WasmProcessStatusV1::MalformedRequest)?;
        runtime
            .advance_carrier_occurrence(occurrence)
            .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
    }
    let occurrence =
        decode_executable_occurrence_v1(last).map_err(|_| WasmProcessStatusV1::MalformedRequest)?;
    runtime
        .advance_carrier_occurrence_and_emit_candidate(occurrence)
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
    runtime
        .settle_carrier_process(admission_authorization)
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
    let observation = runtime
        .observe_carrier_state(&request.render_slots)
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
    Ok(WasmProcessObservationV1 {
        observation: observation.id,
        state: observation.state,
        exact_value_bytes: encode_values(&observation.value)?,
    })
}

pub fn encode_wasm_process_observation_v1(observation: &WasmProcessObservationV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(RESPONSE_MAGIC);
    bytes.extend_from_slice(observation.observation.as_bytes());
    bytes.extend_from_slice(observation.state.as_bytes());
    bytes.extend_from_slice(&observation.exact_value_bytes);
    bytes
}

pub fn decode_wasm_process_observation_v1(
    bytes: &[u8],
) -> Result<WasmProcessObservationV1, WasmProcessStatusV1> {
    let mut d = Decoder::new(bytes);
    if d.take(4)? != RESPONSE_MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let observation = ObservationId::from_bytes(d.identity()?);
    let state = StateRevisionId::from_bytes(d.identity()?);
    let start = d.offset;
    d.skip_values()?;
    let exact_value_bytes = bytes[start..d.offset].to_vec();
    if !d.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(WasmProcessObservationV1 {
        observation,
        state,
        exact_value_bytes,
    })
}

fn validate_shape(request: &WasmProcessRequestV1) -> Result<(), WasmProcessStatusV1> {
    if request.package_bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1
        || request.physical_plan_bytes.len() > WASM_PROCESS_REQUEST_LIMIT_V1
        || request.occurrences.is_empty()
        || request.occurrences.len() > MAX_OCCURRENCES
        || request.render_slots.len() > MAX_RENDER_SLOTS
        || [
            request.authority.occurrence_evidence_bytes.len(),
            request.authority.judgment_evidence_bytes.len(),
            request.authority.admission_evidence_bytes.len(),
        ]
        .into_iter()
        .any(|length| length > MAX_EVIDENCE_BYTES)
    {
        return Err(WasmProcessStatusV1::RequestOutOfBounds);
    }
    Ok(())
}

fn establish_authority(
    package: &CheckedProcessPackage,
    input: &WasmAuthorityInputV1,
    candidate: CandidateDeltaId,
) -> Result<
    (
        AuthorityStore,
        ExecutableAuthorityFactsV1,
        AdmissionAuthorizationEvidence,
    ),
    WasmProcessStatusV1,
> {
    establish_authority_inner(package, input, Some(candidate))
}

pub(super) fn establish_persistent_authority(
    package: &CheckedProcessPackage,
    input: &WasmAuthorityInputV1,
) -> Result<(AuthorityStore, ExecutableAuthorityFactsV1), WasmProcessStatusV1> {
    let (authority, facts, _) = establish_authority_inner(package, input, None)?;
    Ok((authority, facts))
}

fn establish_authority_inner(
    package: &CheckedProcessPackage,
    input: &WasmAuthorityInputV1,
    initial_admission_candidate: Option<CandidateDeltaId>,
) -> Result<
    (
        AuthorityStore,
        ExecutableAuthorityFactsV1,
        AdmissionAuthorizationEvidence,
    ),
    WasmProcessStatusV1,
> {
    let semantics = package.constitution().semantics();
    let snapshot = package.constitution().snapshot();
    let revision = ProgramRevisionPreimage {
        semantics,
        program: input.program,
        predecessor: None,
        snapshot,
        change: input.change,
    }
    .derived_claim();
    let initial = package
        .initial_state_views()
        .first()
        .ok_or(WasmProcessStatusV1::AuthorityRejected)?;
    let session_anchor = RuntimeSessionAnchor::establish(
        input.session,
        revision.id,
        semantics,
        input.policy,
        input.session_start,
        initial.canonical_state_snapshot.to_vec(),
    );
    let initial_state = session_anchor.initial_state_id();
    let genesis = RootAdmissionAuthorizationRef {
        policy: input.root_policy,
        local: AdmissionAuthorizationLocalId::new(0),
    };
    let admission_authorization = RootAdmissionAuthorizationRef {
        policy: input.root_policy,
        local: AdmissionAuthorizationLocalId::new(1),
    };
    let judgment_authority = RootJudgmentAuthorityRef {
        policy: input.root_policy,
        local: JudgmentAuthorityLocalId::new(0),
    };
    let admission_authorization_issuer = RootAdmissionAuthorizationIssuerRef {
        policy: input.root_policy,
        local: AdmissionAuthorizationIssuerLocalId::new(0),
    };
    let state_admission_grants =
        initial_admission_candidate.map(|candidate| RootStateAdmissionGrant {
            authorization: admission_authorization,
            scope: CheckedStateAdmissionScope {
                package: package.id(),
                session: input.session,
                base: initial_state,
                delta: candidate,
            },
        });
    let root = RootPolicyAnchor::establish_with_governance(
        input.root_policy,
        vec![RootGenesisGrant {
            authorization: genesis,
            scope: RootGenesisScope {
                semantics,
                program: input.program,
                snapshot,
                change: input.change,
            },
        }],
        vec![],
        state_admission_grants.into_iter().collect(),
        vec![RootJudgmentAuthorityGrant {
            authority: judgment_authority,
            scope: JudgmentAuthorityScope {
                semantics,
                session: input.session,
                policy: input.policy,
            },
        }],
        vec![RootStateAdmissionIssuerGrant {
            issuer: admission_authorization_issuer,
            scope: StateAdmissionIssuerScope {
                revision: revision.id,
                package: package.id(),
                session: input.session,
                policy: input.policy,
            },
        }],
    )
    .map_err(|_| WasmProcessStatusV1::AuthorityRejected)?;
    let mut authority = AuthorityStore::new();
    authority
        .establish_root_policy(root)
        .and_then(|()| {
            authority.admit_genesis(
                revision,
                package.authority_input(),
                input.root_policy,
                genesis,
            )
        })
        .and_then(|()| authority.establish_runtime_session(session_anchor))
        .map_err(|_| WasmProcessStatusV1::AuthorityRejected)?;
    let boundary_target = package
        .constitution()
        .preimage()
        .formations
        .first()
        .ok_or(WasmProcessStatusV1::AuthorityRejected)?
        .target
        .clone();
    let admitted = CheckedConstitutionBinding::Admitted {
        revision: revision.id,
    };
    let occurrence_pins = BoundaryPins {
        semantics,
        snapshot,
        constitution: admitted,
        runtime_session: None,
        observed_state: None,
        runtime_policy: None,
    };
    let state_pins = BoundaryPins {
        semantics,
        snapshot,
        constitution: admitted,
        runtime_session: Some(input.session),
        observed_state: None,
        runtime_policy: Some(input.policy),
    };
    let at_most_one = CardinalityV2 {
        minimum: 0,
        maximum: Some(1),
    };
    let exactly_one = CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    };
    authority
        .establish_boundary(BoundaryAnchor {
            boundary: input.occurrence_boundary,
            permissions: vec![
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(0),
                    kind: EnteredOccurrenceKind::ExternalTrigger,
                    payload: boundary_target.clone(),
                    pins: occurrence_pins,
                    cause_schema: vec![],
                    support_schema: vec![],
                    replay: BoundaryReplayPolicyV2::OneShot,
                },
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(1),
                    kind: EnteredOccurrenceKind::Observation,
                    payload: boundary_target.clone(),
                    pins: occurrence_pins,
                    cause_schema: vec![
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::ExternalTrigger,
                            cardinality: at_most_one,
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Step,
                            cardinality: at_most_one,
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Admission,
                            cardinality: at_most_one,
                        },
                    ],
                    support_schema: vec![],
                    replay: BoundaryReplayPolicyV2::Repeatable {
                        maximum_occurrences: None,
                    },
                },
            ],
        })
        .and_then(|()| {
            authority.establish_boundary(BoundaryAnchor {
                boundary: input.state_boundary,
                permissions: vec![
                    BoundaryOccurrencePermissionV2 {
                        id: BoundaryPermissionLocalId::new(2),
                        kind: EnteredOccurrenceKind::Judgment,
                        payload: boundary_target.clone(),
                        pins: state_pins,
                        cause_schema: vec![BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::CandidateDelta,
                            cardinality: exactly_one,
                        }],
                        support_schema: vec![],
                        replay: BoundaryReplayPolicyV2::Repeatable {
                            maximum_occurrences: None,
                        },
                    },
                    BoundaryOccurrencePermissionV2 {
                        id: BoundaryPermissionLocalId::new(3),
                        kind: EnteredOccurrenceKind::AdmissionDecision,
                        payload: boundary_target.clone(),
                        pins: state_pins,
                        cause_schema: vec![
                            BoundaryCauseRequirementV2 {
                                kind: EnteredCauseKindV2::CandidateDelta,
                                cardinality: exactly_one,
                            },
                            BoundaryCauseRequirementV2 {
                                kind: EnteredCauseKindV2::Judgment,
                                cardinality: CardinalityV2 {
                                    minimum: 1,
                                    maximum: None,
                                },
                            },
                        ],
                        support_schema: vec![],
                        replay: BoundaryReplayPolicyV2::Repeatable {
                            maximum_occurrences: None,
                        },
                    },
                    BoundaryOccurrencePermissionV2 {
                        id: BoundaryPermissionLocalId::new(4),
                        kind: EnteredOccurrenceKind::AdmissionAuthorization,
                        payload: boundary_target.clone(),
                        pins: state_pins,
                        cause_schema: vec![BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::CandidateDelta,
                            cardinality: exactly_one,
                        }],
                        support_schema: vec![],
                        replay: BoundaryReplayPolicyV2::Repeatable {
                            maximum_occurrences: None,
                        },
                    },
                ],
            })
        })
        .map_err(|_| WasmProcessStatusV1::AuthorityRejected)?;
    for (evidence, boundary, permissions, exact) in [
        (
            input.occurrence_evidence,
            input.occurrence_boundary,
            vec![
                BoundaryPermissionLocalId::new(0),
                BoundaryPermissionLocalId::new(1),
            ],
            &input.occurrence_evidence_bytes,
        ),
        (
            input.judgment_evidence,
            input.state_boundary,
            vec![BoundaryPermissionLocalId::new(2)],
            &input.judgment_evidence_bytes,
        ),
        (
            input.admission_evidence,
            input.state_boundary,
            vec![
                BoundaryPermissionLocalId::new(3),
                BoundaryPermissionLocalId::new(4),
            ],
            &input.admission_evidence_bytes,
        ),
    ] {
        authority
            .establish_evidence(EvidenceAnchor {
                evidence,
                boundary,
                permissions,
                exact_evidence: exact.clone().into_boxed_slice(),
            })
            .map_err(|_| WasmProcessStatusV1::AuthorityRejected)?;
    }
    Ok((
        authority,
        ExecutableAuthorityFactsV1 {
            program_revision: revision.id,
            session: input.session,
            initial_state,
            policy: input.policy,
            session_start: input.session_start,
            root_policy: input.root_policy,
            judgment_authority,
            admission_authorization_issuer,
            trigger_ingress: ExecutableBoundaryFactV1 {
                boundary: input.occurrence_boundary,
                evidence: input.occurrence_evidence,
                permission: BoundaryPermissionLocalId::new(0),
            },
            occurrence_ingress: ExecutableBoundaryFactV1 {
                boundary: input.occurrence_boundary,
                evidence: input.occurrence_evidence,
                permission: BoundaryPermissionLocalId::new(1),
            },
            judgment_ingress: ExecutableBoundaryFactV1 {
                boundary: input.state_boundary,
                evidence: input.judgment_evidence,
                permission: BoundaryPermissionLocalId::new(2),
            },
            admission_issuance_ingress: ExecutableBoundaryFactV1 {
                boundary: input.state_boundary,
                evidence: input.admission_evidence,
                permission: BoundaryPermissionLocalId::new(4),
            },
            admission_ingress: ExecutableBoundaryFactV1 {
                boundary: input.state_boundary,
                evidence: input.admission_evidence,
                permission: BoundaryPermissionLocalId::new(3),
            },
            budget_units: input.budget_units,
        },
        AdmissionAuthorizationEvidence::IrreducibleRoot {
            policy: input.root_policy,
            authorization: admission_authorization,
        },
    ))
}

fn encode_values(values: &[ExecutableValueV1]) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let mut bytes = Vec::new();
    put_count(&mut bytes, values.len())?;
    for value in values {
        match value {
            ExecutableValueV1::Number(bits) => {
                bytes.push(0);
                bytes.extend_from_slice(&bits.to_le_bytes());
            }
            ExecutableValueV1::Boolean(value) => bytes.extend_from_slice(&[1, u8::from(*value)]),
        }
    }
    Ok(bytes)
}

pub(super) fn put_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), WasmProcessStatusV1> {
    let count = u16::try_from(count).map_err(|_| WasmProcessStatusV1::RequestOutOfBounds)?;
    bytes.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

pub(super) fn put_blob(bytes: &mut Vec<u8>, blob: &[u8]) -> Result<(), WasmProcessStatusV1> {
    let count = u32::try_from(blob.len()).map_err(|_| WasmProcessStatusV1::RequestOutOfBounds)?;
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes.extend_from_slice(blob);
    Ok(())
}

pub(super) struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(super) fn take(&mut self, count: usize) -> Result<&'a [u8], WasmProcessStatusV1> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(WasmProcessStatusV1::MalformedRequest)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(WasmProcessStatusV1::MalformedRequest)?;
        self.offset = end;
        Ok(value)
    }

    pub(super) fn u16(&mut self) -> Result<u16, WasmProcessStatusV1> {
        Ok(u16::from_le_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| WasmProcessStatusV1::MalformedRequest)?,
        ))
    }

    pub(super) fn u32(&mut self) -> Result<u32, WasmProcessStatusV1> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| WasmProcessStatusV1::MalformedRequest)?,
        ))
    }

    pub(super) fn u64(&mut self) -> Result<u64, WasmProcessStatusV1> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| WasmProcessStatusV1::MalformedRequest)?,
        ))
    }

    pub(super) fn identity(&mut self) -> Result<[u8; IDENTITY_BYTES], WasmProcessStatusV1> {
        self.take(IDENTITY_BYTES)?
            .try_into()
            .map_err(|_| WasmProcessStatusV1::MalformedRequest)
    }

    fn count(&mut self, maximum: usize) -> Result<usize, WasmProcessStatusV1> {
        let count = usize::from(self.u16()?);
        if count > maximum {
            return Err(WasmProcessStatusV1::RequestOutOfBounds);
        }
        Ok(count)
    }

    pub(super) fn blob(&mut self, maximum: usize) -> Result<&'a [u8], WasmProcessStatusV1> {
        let count =
            usize::try_from(self.u32()?).map_err(|_| WasmProcessStatusV1::RequestOutOfBounds)?;
        if count > maximum {
            return Err(WasmProcessStatusV1::RequestOutOfBounds);
        }
        self.take(count)
    }

    fn skip_values(&mut self) -> Result<(), WasmProcessStatusV1> {
        let count = self.count(MAX_RENDER_SLOTS)?;
        for _ in 0..count {
            match self.take(1)?[0] {
                0 => {
                    self.take(8)?;
                }
                1 if self.take(1)?[0] <= 1 => {}
                _ => return Err(WasmProcessStatusV1::MalformedRequest),
            }
        }
        Ok(())
    }

    pub(super) fn is_complete(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
