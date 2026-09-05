use std::error::Error;
use std::fmt;

use clause_package::*;
use clause_runtime::*;

const PROCESS_PACKAGE_HEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/process-v2/positive/process-v2-core.hex"
));

macro_rules! id {
    ($kind:ident, $tag:expr) => {
        $kind::from_bytes(raw_id($tag))
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CarrierActionV1 {
    Unchanged,
    Candidate,
    Admission,
    HotReload,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkbenchCarrierSnapshot {
    pub candidate_delta_count: usize,
    pub decision_count: usize,
    pub state_revision_count: usize,
    pub world_base: StateRevisionId,
    pub run: RunId,
    pub activation: ActivationId,
}

#[derive(Debug)]
pub enum WorkbenchCarrierError {
    InvalidHexFixture,
    MissingInitialState,
    MissingApplicationShape,
    MissingBoundaryTarget,
    CanonicalDecode(CanonicalDecodeError),
    CanonicalEncode(CanonicalEncodeError),
    PackageCheck(ProcessPackageCheckError),
    Authority(AuthorityError),
    Session(PersistentProcessSessionErrorV1),
}

impl fmt::Display for WorkbenchCarrierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexFixture => formatter.write_str("transaction package hex is invalid"),
            Self::MissingInitialState => {
                formatter.write_str("transaction package has no initial State view")
            }
            Self::MissingApplicationShape => {
                formatter.write_str("transaction package has no Application shape")
            }
            Self::MissingBoundaryTarget => {
                formatter.write_str("transaction package has no boundary target")
            }
            Self::CanonicalDecode(error) => {
                write!(formatter, "transaction package decode failed: {error}")
            }
            Self::CanonicalEncode(error) => {
                write!(formatter, "transaction package encode failed: {error}")
            }
            Self::PackageCheck(error) => {
                write!(formatter, "transaction package check failed: {error}")
            }
            Self::Authority(error) => write!(formatter, "transaction authority failed: {error}"),
            Self::Session(error) => write!(formatter, "transaction session failed: {error}"),
        }
    }
}

impl Error for WorkbenchCarrierError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalDecode(error) => Some(error),
            Self::CanonicalEncode(error) => Some(error),
            Self::PackageCheck(error) => Some(error),
            Self::Authority(error) => Some(error),
            Self::Session(error) => Some(error),
            Self::InvalidHexFixture
            | Self::MissingInitialState
            | Self::MissingApplicationShape
            | Self::MissingBoundaryTarget => None,
        }
    }
}

impl From<CanonicalDecodeError> for WorkbenchCarrierError {
    fn from(value: CanonicalDecodeError) -> Self {
        Self::CanonicalDecode(value)
    }
}

impl From<CanonicalEncodeError> for WorkbenchCarrierError {
    fn from(value: CanonicalEncodeError) -> Self {
        Self::CanonicalEncode(value)
    }
}

impl From<ProcessPackageCheckError> for WorkbenchCarrierError {
    fn from(value: ProcessPackageCheckError) -> Self {
        Self::PackageCheck(value)
    }
}

impl From<AuthorityError> for WorkbenchCarrierError {
    fn from(value: AuthorityError) -> Self {
        Self::Authority(value)
    }
}

impl From<PersistentProcessSessionErrorV1> for WorkbenchCarrierError {
    fn from(value: PersistentProcessSessionErrorV1) -> Self {
        Self::Session(value)
    }
}

pub(crate) struct WorkbenchCarrier {
    session: PersistentProcessSessionV1,
}

impl WorkbenchCarrier {
    pub(crate) fn open() -> Result<Self, WorkbenchCarrierError> {
        let package = checked_transaction_package()?;
        let application = application(&package);
        let physical_plan = physical_plan(&package)?;
        let (authority, facts) = carrier_authority(&package)?;
        let session = PersistentProcessSessionV1::open(
            package,
            authority,
            application,
            physical_plan,
            facts,
        )?;
        Ok(Self { session })
    }

    pub(crate) fn apply(&mut self, action: CarrierActionV1) -> Result<(), WorkbenchCarrierError> {
        match action {
            CarrierActionV1::Unchanged | CarrierActionV1::HotReload => Ok(()),
            CarrierActionV1::Candidate => {
                let occurrence = encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
                    entry: 0,
                    arguments: vec![number(1.0)],
                })
                .map_err(PersistentProcessSessionErrorV1::from)?;
                self.session
                    .apply_opaque_input_and_emit_candidate(&occurrence)?;
                Ok(())
            }
            CarrierActionV1::Admission => {
                let authorization = self.session.issue_candidate_admission_authorization()?;
                self.session
                    .admit_issued_candidate_with_projection(authorization)?;
                Ok(())
            }
        }
    }

    pub(crate) fn snapshot(&self) -> Result<WorkbenchCarrierSnapshot, WorkbenchCarrierError> {
        let carrier = self.session.carrier()?;
        Ok(WorkbenchCarrierSnapshot {
            candidate_delta_count: carrier.candidate_delta_count(),
            decision_count: carrier.decision_count(),
            state_revision_count: carrier.state_revision_count(),
            world_base: self.session.world_base(),
            run: self.session.run()?,
            activation: self.session.activation()?,
        })
    }
}

fn checked_transaction_package() -> Result<CheckedProcessPackage, WorkbenchCarrierError> {
    let decoded = decode_process_package(&decode_hex(PROCESS_PACKAGE_HEX)?)?;
    let mut candidate = decoded.candidate().clone();
    candidate.records.clear();
    let bytes = encode_process_package(&candidate)?;
    Ok(check_process_package(decode_process_package(&bytes)?)?)
}

fn physical_plan(
    package: &CheckedProcessPackage,
) -> Result<ExecutablePhysicalPlanV1, WorkbenchCarrierError> {
    let snapshot = package.constitution().snapshot();
    let application = ApplicationLocalId::new(1);
    let application_shape = package
        .constitution()
        .application_shape(application)
        .ok_or(WorkbenchCarrierError::MissingApplicationShape)?;
    Ok(ExecutablePhysicalPlanV1 {
        source_metadata: None,
        application_shape,
        mode: ModeId {
            operator: OperatorRef {
                snapshot,
                local: OperatorLocalId::new(1),
            },
            local: ModeLocalId::new(2),
        },
        refinement: ExecutableRefinementV1::ClosedApplicationRuleMachineV1,
        target: ExecutablePhysicalTargetV1::PortableScalarInterpreterV1,
        input: None,
        program: ExecutableProgramV1 {
            initial_configuration: vec![number(0.0)],
            rules: vec![ExecutableRuleV1 {
                entry: 0,
                predicates: vec![],
                required_present: vec![],
                required_absent: vec![],
                assignments: vec![(
                    0,
                    ExecutableExpressionV1::Add(
                        Box::new(ExecutableExpressionV1::Slot(0)),
                        Box::new(ExecutableExpressionV1::Argument(0)),
                    ),
                )],
                removals: vec![],
            }],
            projection: None,
        },
    })
}

fn carrier_authority(
    checked: &CheckedProcessPackage,
) -> Result<(AuthorityStore, ExecutableAuthorityFactsV1), WorkbenchCarrierError> {
    let semantics = checked.constitution().semantics();
    let snapshot = checked.constitution().snapshot();
    let session = id!(RuntimeSessionId, 120);
    let policy = id!(RuntimePolicyId, 121);
    let session_start = id!(SessionStartOccurrenceId, 122);
    let revision = ProgramRevisionPreimage {
        semantics,
        program: id!(ProgramId, 123),
        predecessor: None,
        snapshot,
        change: id!(ProgramChangeOccurrenceId, 124),
    }
    .derived_claim();
    let initial_view = checked
        .initial_state_views()
        .first()
        .ok_or(WorkbenchCarrierError::MissingInitialState)?;
    let session_anchor = RuntimeSessionAnchor::establish(
        session,
        revision.id,
        semantics,
        policy,
        session_start,
        initial_view.canonical_state_snapshot.to_vec(),
    );
    let initial_state = session_anchor.initial_state_id();
    let root_policy = id!(RootPolicyId, 125);
    let root_genesis = RootAdmissionAuthorizationRef {
        policy: root_policy,
        local: AdmissionAuthorizationLocalId::new(0),
    };
    let judgment_authority = RootJudgmentAuthorityRef {
        policy: root_policy,
        local: JudgmentAuthorityLocalId::new(0),
    };
    let admission_authorization_issuer = RootAdmissionAuthorizationIssuerRef {
        policy: root_policy,
        local: AdmissionAuthorizationIssuerLocalId::new(0),
    };
    let mut authority = AuthorityStore::new();
    authority.establish_root_policy(RootPolicyAnchor::establish_with_governance(
        root_policy,
        vec![RootGenesisGrant {
            authorization: root_genesis,
            scope: RootGenesisScope {
                semantics,
                program: revision.preimage.program,
                snapshot,
                change: revision.preimage.change,
            },
        }],
        vec![],
        vec![],
        vec![RootJudgmentAuthorityGrant {
            authority: judgment_authority,
            scope: JudgmentAuthorityScope {
                semantics,
                session,
                policy,
            },
        }],
        vec![RootStateAdmissionIssuerGrant {
            issuer: admission_authorization_issuer,
            scope: StateAdmissionIssuerScope {
                revision: revision.id,
                package: checked.id(),
                session,
                policy,
            },
        }],
    )?)?;
    authority.admit_genesis(
        revision,
        checked.authority_input(),
        root_policy,
        root_genesis,
    )?;
    authority.establish_runtime_session(session_anchor)?;

    let occurrence_boundary = id!(BoundaryRef, 126);
    let state_boundary = id!(BoundaryRef, 127);
    let boundary_target = checked
        .constitution()
        .preimage()
        .formations
        .first()
        .ok_or(WorkbenchCarrierError::MissingBoundaryTarget)?
        .target
        .clone();
    let admitted = CheckedConstitutionBinding::Admitted {
        revision: revision.id,
    };
    authority.establish_boundary(executable_occurrence_boundary_anchor_v1(
        occurrence_boundary,
        boundary_target.clone(),
        BoundaryPins {
            semantics,
            snapshot,
            constitution: admitted,
            runtime_session: None,
            observed_state: None,
            runtime_policy: None,
        },
    ))?;
    authority.establish_boundary(executable_state_boundary_anchor_v1(
        state_boundary,
        boundary_target,
        BoundaryPins {
            semantics,
            snapshot,
            constitution: admitted,
            runtime_session: Some(session),
            observed_state: None,
            runtime_policy: Some(policy),
        },
    ))?;

    let occurrence_evidence = id!(ExternalEvidenceRef, 181);
    let judgment_evidence = id!(ExternalEvidenceRef, 186);
    let admission_evidence = id!(ExternalEvidenceRef, 190);
    for (evidence, boundary, permissions, bytes) in [
        (
            occurrence_evidence,
            occurrence_boundary,
            vec![
                EXECUTABLE_TRIGGER_PERMISSION_V1,
                EXECUTABLE_OBSERVATION_PERMISSION_V1,
            ],
            vec![181],
        ),
        (
            judgment_evidence,
            state_boundary,
            vec![EXECUTABLE_JUDGMENT_PERMISSION_V1],
            vec![186],
        ),
        (
            admission_evidence,
            state_boundary,
            vec![
                EXECUTABLE_ADMISSION_PERMISSION_V1,
                EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1,
                EXECUTABLE_RESUMPTION_PERMISSION_V1,
            ],
            vec![190],
        ),
    ] {
        authority.establish_evidence(EvidenceAnchor {
            evidence,
            boundary,
            permissions,
            exact_evidence: bytes.into_boxed_slice(),
        })?;
    }

    Ok((
        authority,
        ExecutableAuthorityFactsV1 {
            program_revision: revision.id,
            session,
            initial_state,
            policy,
            session_start,
            root_policy,
            judgment_authority,
            admission_authorization_issuer,
            trigger_ingress: ExecutableBoundaryFactV1 {
                boundary: occurrence_boundary,
                evidence: occurrence_evidence,
                permission: EXECUTABLE_TRIGGER_PERMISSION_V1,
            },
            occurrence_ingress: ExecutableBoundaryFactV1 {
                boundary: occurrence_boundary,
                evidence: occurrence_evidence,
                permission: EXECUTABLE_OBSERVATION_PERMISSION_V1,
            },
            resumption_ingress: ExecutableBoundaryFactV1 {
                boundary: state_boundary,
                evidence: admission_evidence,
                permission: EXECUTABLE_RESUMPTION_PERMISSION_V1,
            },
            judgment_ingress: ExecutableBoundaryFactV1 {
                boundary: state_boundary,
                evidence: judgment_evidence,
                permission: EXECUTABLE_JUDGMENT_PERMISSION_V1,
            },
            admission_issuance_ingress: ExecutableBoundaryFactV1 {
                boundary: state_boundary,
                evidence: admission_evidence,
                permission: EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1,
            },
            admission_ingress: ExecutableBoundaryFactV1 {
                boundary: state_boundary,
                evidence: admission_evidence,
                permission: EXECUTABLE_ADMISSION_PERMISSION_V1,
            },
            budget_units: 100,
        },
    ))
}

fn application(package: &CheckedProcessPackage) -> ApplicationId {
    ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    }
}

fn number(value: f64) -> ExecutableValueV1 {
    ExecutableValueV1::number(value).expect("finite internal transaction value")
}

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

fn decode_hex(source: &str) -> Result<Vec<u8>, WorkbenchCarrierError> {
    let digits = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if digits.len() % 2 != 0 {
        return Err(WorkbenchCarrierError::InvalidHexFixture);
    }
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = nibble(pair[0]).ok_or(WorkbenchCarrierError::InvalidHexFixture)?;
            let low = nibble(pair[1]).ok_or(WorkbenchCarrierError::InvalidHexFixture)?;
            Ok((high << 4) | low)
        })
        .collect()
}

const fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
