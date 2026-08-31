//! Deterministic CLCP-v3 package checking.
//!
//! This module returns exact compiler-machine checker evidence. `Authorized`
//! is not Clause `Admission`, does not create a `ProgramRevision`, and does not
//! model an `Activation`, `Step`, or `Run`.

use std::fmt;

use crate::evaluator::{EvalError, Evaluation, Evaluator, StaticError};
use crate::physical::PhysicalError;

use super::codec::{canonical_evidence_bytes, canonical_subject_bytes};
use super::{
    CompilerEvidence, CompilerLineage, CompilerSubject, DecodeFailure, DecodedCompilerPackage,
    EncodeError, EvalReceipt, Hash32, Id32, KExpr, KSort, KValue, NominalDeclaration,
    NominalWireRef, Term, compiler_package_hash, core_contract_id, decode, domain_hash, encode,
    exact_core_manifest_bytes, physical_profile_id, source_artifact_id, try_copy_bytes,
};

const K_TAG: &[u8] = b"clause/core-abi/tag/v1";
const K_BYTES: &[u8] = b"clause/core-abi/bytes/v1";
const K_ID32: &[u8] = b"clause/core-abi/id32/v1";
const K_U64: &[u8] = b"clause/core-abi/u64/v1";
const K_EQ: &[u8] = b"clause/core/bytes-equal/v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AuthorizationStage {
    CoreManifest = 0x40,
    CoreWellFormedness = 0x41,
    GenesisAnchor = 0x42,
    ExactPredecessor = 0x43,
    BuildRequest = 0x44,
    CompileEvaluation = 0x45,
    AdmissionEvaluation = 0x46,
    EvidenceAttachment = 0x47,
    FinalAuthorization = 0x48,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AuthorizationCode {
    ManifestMismatch = 0x60,
    SubjectStructure = 0x61,
    NominalTable = 0x62,
    DefinitionOrderOrDuplicate = 0x63,
    EntrypointResolution = 0x64,
    EntrypointAliased = 0x65,
    EntrypointSignature = 0x66,
    StaticRule = 0x67,
    PhysicalRequestSignature = 0x68,
    GenesisWrongLineage = 0x69,
    GenesisEvidenceNotEmpty = 0x6a,
    MissingAnchor = 0x6b,
    AnchorBytesMismatch = 0x6c,
    SuccessorWrongLineage = 0x6d,
    PredecessorNotAccepted = 0x6e,
    CandidateOrSelfPredecessor = 0x6f,
    LocatorMismatch = 0x70,
    PredecessorBytesMismatch = 0x71,
    BuildRequestShape = 0x72,
    DetachedBuildRequest = 0x73,
    BaseMismatch = 0x74,
    CoreContractMismatch = 0x75,
    PhysicalProfileMismatch = 0x76,
    SourceOrderOrDuplicate = 0x77,
    SourceArtifactMismatch = 0x78,
    IdentityPlanMismatch = 0x79,
    ChangeOccurrenceMismatch = 0x7a,
    PhysicalInputsNonempty = 0x7b,
    FuelInvalid = 0x7c,
    EvidenceShapeMismatch = 0x7d,
    ReceiptValueMismatch = 0x7e,
    ReceiptFuelMismatch = 0x7f,
    EvaluationFault = 0x80,
    UnexpectedResult = 0x81,
    SubjectMismatch = 0x82,
    ObservationMismatch = 0x83,
    EvidenceDetached = 0x84,
    SubjectChangedAfterCompile = 0x85,
    PackageChangedAfterEvidence = 0x86,
    FinalIdentityMismatch = 0x87,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationFailure {
    pub stage: AuthorizationStage,
    pub code: AuthorizationCode,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AuthorizationVerdict {
    /// Exact successful checker evidence only. Outer Clause governance must
    /// separately decide whether to admit any compiler or Program successor.
    Authorized(Vec<u8>),
    Unauthorized(AuthorizationFailure),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizationCheckError {
    Decode(DecodeFailure),
    ResourceExhausted,
}

impl fmt::Display for AuthorizationCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "strict CLCP-v3 decode failed: {error}"),
            Self::ResourceExhausted => {
                formatter.write_str("CLCP-v3 checking exhausted physical resources")
            }
        }
    }
}

impl std::error::Error for AuthorizationCheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::ResourceExhausted => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalPackageIdentityInput<'a> {
    pub package_hash: Hash32,
    pub exact_package_bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerAnchorObservation<'a> {
    pub exact_selected_bytes: &'a [u8],
    pub selected_byte_length: u64,
    pub selected_package_hash: Hash32,
}

/// Opaque non-wire capability. Only a crate-owned irreducible owner boundary
/// may construct one; the checker can inspect but cannot mint it from package
/// bytes, hashes, decoding, or execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerAnchorWitness<'a> {
    observation: OwnerAnchorObservation<'a>,
}

impl<'a> OwnerAnchorWitness<'a> {
    /// Establish the irreducible owner observation supplied at the one genesis
    /// boundary. This constructor records external selection; it does not
    /// infer authority from package bytes or hashes.
    pub const fn from_external_selection(observation: OwnerAnchorObservation<'a>) -> Self {
        Self { observation }
    }

    const fn observation(self) -> OwnerAnchorObservation<'a> {
        self.observation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerAnchorInput<'a> {
    Missing,
    Supplied(OwnerAnchorWitness<'a>),
}

pub struct GenesisAuthorizationRequest<'a> {
    pub owner_anchor: OwnerAnchorInput<'a>,
    pub build_request: &'a Term,
    pub evidence: &'a CompilerEvidence,
    pub compile_fuel_limit: u64,
    pub admission_fuel_limit: u64,
    pub final_identity: FinalPackageIdentityInput<'a>,
}

/// Opaque premise that outer governance has already admitted these exact
/// predecessor bytes. A hash match or successful replay cannot construct it.
#[derive(Clone, Copy, Debug)]
pub struct AcceptedExact<'a> {
    exact_bytes: &'a [u8],
}

impl<'a> AcceptedExact<'a> {
    #[allow(dead_code)]
    pub(crate) const fn from_outer_admission(exact_bytes: &'a [u8]) -> Self {
        Self { exact_bytes }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PredecessorInput<'a> {
    Absent {
        offered_bytes: &'a [u8],
    },
    Accepted {
        exact_bytes: &'a [u8],
        acceptance: AcceptedExact<'a>,
        offered_bytes: &'a [u8],
    },
}

pub struct SuccessorAuthorizationRequest<'a> {
    pub predecessor: PredecessorInput<'a>,
    pub build_request: &'a Term,
    pub evidence: &'a CompilerEvidence,
    pub final_identity: FinalPackageIdentityInput<'a>,
}

fn deny(stage: AuthorizationStage, code: AuthorizationCode) -> AuthorizationVerdict {
    AuthorizationVerdict::Unauthorized(AuthorizationFailure { stage, code })
}

fn failure(stage: AuthorizationStage, code: AuthorizationCode) -> AuthorizationFailure {
    AuthorizationFailure { stage, code }
}

fn resource<T>() -> Result<T, AuthorizationCheckError> {
    Err(AuthorizationCheckError::ResourceExhausted)
}

fn map_encode<T>(result: Result<T, EncodeError>) -> Result<T, AuthorizationCheckError> {
    result.map_err(|_| AuthorizationCheckError::ResourceExhausted)
}

fn copy_authorized(input: &[u8]) -> Result<AuthorizationVerdict, AuthorizationCheckError> {
    let bytes = try_copy_bytes(input).map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    Ok(AuthorizationVerdict::Authorized(bytes))
}

fn as_tag(term: &Term) -> Option<u8> {
    let Term::Atom {
        kind,
        canonical_payload,
        equality_contract,
    } = term
    else {
        return None;
    };
    (kind.as_slice() == K_TAG
        && equality_contract.as_slice() == K_EQ
        && canonical_payload.len() == 1)
        .then(|| canonical_payload[0])
}

fn as_bytes(term: &Term) -> Option<&[u8]> {
    let Term::Atom {
        kind,
        canonical_payload,
        equality_contract,
    } = term
    else {
        return None;
    };
    (kind.as_slice() == K_BYTES && equality_contract.as_slice() == K_EQ)
        .then_some(canonical_payload)
}

fn as_id(term: &Term) -> Option<Id32> {
    let Term::Atom {
        kind,
        canonical_payload,
        equality_contract,
    } = term
    else {
        return None;
    };
    if kind.as_slice() != K_ID32
        || equality_contract.as_slice() != K_EQ
        || canonical_payload.len() != 32
    {
        return None;
    }
    Some(Id32(canonical_payload.as_slice().try_into().ok()?))
}

fn as_hash(term: &Term) -> Option<Hash32> {
    as_id(term).map(|value| Hash32(value.0))
}

fn as_u64(term: &Term) -> Option<u64> {
    let Term::Atom {
        kind,
        canonical_payload,
        equality_contract,
    } = term
    else {
        return None;
    };
    if kind.as_slice() != K_U64
        || equality_contract.as_slice() != K_EQ
        || canonical_payload.len() != 8
    {
        return None;
    }
    Some(u64::from_be_bytes(
        canonical_payload.as_slice().try_into().ok()?,
    ))
}

fn list_items(term: &Term) -> Result<Option<Vec<&Term>>, AuthorizationCheckError> {
    let mut items = Vec::new();
    let mut tail = term;
    loop {
        if as_tag(tail) == Some(0x00) {
            return Ok(Some(items));
        }
        let Term::Triple(marker, head, next) = tail else {
            return Ok(None);
        };
        if as_tag(marker) != Some(0x01) {
            return Ok(None);
        }
        items
            .try_reserve(1)
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
        items.push(&**head);
        tail = next;
    }
}

fn record_fields(
    term: &Term,
    expected_tag: u8,
) -> Result<Option<Vec<&Term>>, AuthorizationCheckError> {
    let Term::Triple(marker, fields, trailer) = term else {
        return Ok(None);
    };
    if as_tag(marker) != Some(expected_tag) || as_tag(trailer) != Some(0x00) {
        return Ok(None);
    }
    list_items(fields)
}

fn exactly<const N: usize>(values: Vec<&Term>) -> Option<[&Term; N]> {
    values.try_into().ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NominalRefView {
    domain: Id32,
    id: Id32,
}

fn decode_nominal_ref(term: &Term) -> Result<Option<NominalRefView>, AuthorizationCheckError> {
    let Some(fields) = record_fields(term, 0x04)? else {
        return Ok(None);
    };
    let Some([domain, id]) = exactly(fields) else {
        return Ok(None);
    };
    Ok(Some(NominalRefView {
        domain: match as_id(domain) {
            Some(value) => value,
            None => return Ok(None),
        },
        id: match as_id(id) {
            Some(value) => value,
            None => return Ok(None),
        },
    }))
}

fn decode_ref_wrapper(
    term: &Term,
    tag: u8,
) -> Result<Option<NominalRefView>, AuthorizationCheckError> {
    let Some(fields) = record_fields(term, tag)? else {
        return Ok(None);
    };
    let Some([reference]) = exactly(fields) else {
        return Ok(None);
    };
    decode_nominal_ref(reference)
}

#[derive(Clone, Copy)]
enum BaseView {
    Genesis,
    Accepted {
        package_hash: Hash32,
        revision_id: Id32,
    },
}

struct SourceUnitView<'a> {
    unit_id: Id32,
    artifact_id: Hash32,
    bytes: &'a [u8],
}

struct IdentityPlanView {
    retained: Vec<NominalRefView>,
    seed_inputs: Vec<NominalRefView>,
}

struct BuildRequestView<'a> {
    base: BaseView,
    core_contract_id: Hash32,
    physical_profile_id: Hash32,
    source_units: Vec<SourceUnitView<'a>>,
    identity_plan: IdentityPlanView,
    change_occurrence_id: Id32,
    compile_fuel: u64,
    admission_fuel: u64,
    declared_physical_inputs: Vec<&'a Term>,
}

fn decode_base(term: &Term) -> Result<Option<BaseView>, AuthorizationCheckError> {
    if let Some(fields) = record_fields(term, 0x10)? {
        return Ok(fields.is_empty().then_some(BaseView::Genesis));
    }
    let Some(fields) = record_fields(term, 0x11)? else {
        return Ok(None);
    };
    let Some([package_hash, revision_id]) = exactly(fields) else {
        return Ok(None);
    };
    Ok(Some(BaseView::Accepted {
        package_hash: match as_hash(package_hash) {
            Some(value) => value,
            None => return Ok(None),
        },
        revision_id: match as_id(revision_id) {
            Some(value) => value,
            None => return Ok(None),
        },
    }))
}

fn decode_source_unit<'a>(
    term: &'a Term,
) -> Result<Option<SourceUnitView<'a>>, AuthorizationCheckError> {
    let Some(fields) = record_fields(term, 0x12)? else {
        return Ok(None);
    };
    let Some([unit_id, artifact_id, bytes]) = exactly(fields) else {
        return Ok(None);
    };
    Ok(Some(SourceUnitView {
        unit_id: match as_id(unit_id) {
            Some(value) => value,
            None => return Ok(None),
        },
        artifact_id: match as_hash(artifact_id) {
            Some(value) => value,
            None => return Ok(None),
        },
        bytes: match as_bytes(bytes) {
            Some(value) => value,
            None => return Ok(None),
        },
    }))
}

fn decode_ref_list(
    term: &Term,
    wrapper_tag: u8,
) -> Result<Option<Vec<NominalRefView>>, AuthorizationCheckError> {
    let Some(items) = list_items(term)? else {
        return Ok(None);
    };
    let mut references = Vec::new();
    references
        .try_reserve_exact(items.len())
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    for item in items {
        let Some(reference) = decode_ref_wrapper(item, wrapper_tag)? else {
            return Ok(None);
        };
        references.push(reference);
    }
    Ok(Some(references))
}

fn decode_identity_plan(term: &Term) -> Result<Option<IdentityPlanView>, AuthorizationCheckError> {
    let Some(fields) = record_fields(term, 0x08)? else {
        return Ok(None);
    };
    let Some([retained, seed_inputs]) = exactly(fields) else {
        return Ok(None);
    };
    let Some(retained) = decode_ref_list(retained, 0x09)? else {
        return Ok(None);
    };
    let Some(seed_inputs) = decode_ref_list(seed_inputs, 0x0a)? else {
        return Ok(None);
    };
    Ok(Some(IdentityPlanView {
        retained,
        seed_inputs,
    }))
}

fn decode_build_request<'a>(
    term: &'a Term,
) -> Result<Option<BuildRequestView<'a>>, AuthorizationCheckError> {
    let Some(fields) = record_fields(term, 0x13)? else {
        return Ok(None);
    };
    let Some(
        [
            base,
            core_id,
            profile_id,
            _target,
            sources,
            _inputs,
            identities,
            change,
            _options,
            compile_fuel,
            admission_fuel,
            physical_inputs,
        ],
    ) = exactly(fields)
    else {
        return Ok(None);
    };
    let Some(source_items) = list_items(sources)? else {
        return Ok(None);
    };
    let mut source_units = Vec::new();
    source_units
        .try_reserve_exact(source_items.len())
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    for item in source_items {
        let Some(source) = decode_source_unit(item)? else {
            return Ok(None);
        };
        source_units.push(source);
    }
    let Some(declared_physical_inputs) = list_items(physical_inputs)? else {
        return Ok(None);
    };
    Ok(Some(BuildRequestView {
        base: match decode_base(base)? {
            Some(value) => value,
            None => return Ok(None),
        },
        core_contract_id: match as_hash(core_id) {
            Some(value) => value,
            None => return Ok(None),
        },
        physical_profile_id: match as_hash(profile_id) {
            Some(value) => value,
            None => return Ok(None),
        },
        source_units,
        identity_plan: match decode_identity_plan(identities)? {
            Some(value) => value,
            None => return Ok(None),
        },
        change_occurrence_id: match as_id(change) {
            Some(value) => value,
            None => return Ok(None),
        },
        compile_fuel: match as_u64(compile_fuel) {
            Some(value) => value,
            None => return Ok(None),
        },
        admission_fuel: match as_u64(admission_fuel) {
            Some(value) => value,
            None => return Ok(None),
        },
        declared_physical_inputs,
    }))
}

fn nominal_parts(declaration: &NominalDeclaration) -> (Id32, Id32) {
    match declaration {
        NominalDeclaration::Seed { domain, id }
        | NominalDeclaration::RetainedSeed { domain, id, .. }
        | NominalDeclaration::Allocated { domain, id, .. } => (*domain, *id),
    }
}

fn nominal_reference(declaration: &NominalDeclaration) -> NominalRefView {
    let (domain, id) = nominal_parts(declaration);
    NominalRefView { domain, id }
}

fn find_nominal(
    declarations: &[NominalDeclaration],
    reference: NominalRefView,
) -> Option<&NominalDeclaration> {
    declarations
        .iter()
        .find(|declaration| nominal_parts(declaration) == (reference.domain, reference.id))
}

fn find_nominal_index(
    declarations: &[NominalDeclaration],
    reference: &NominalWireRef,
) -> Option<usize> {
    declarations
        .iter()
        .position(|declaration| nominal_parts(declaration) == (reference.domain, reference.id))
}

fn nominal_exists(declarations: &[NominalDeclaration], domain: Id32, id: Id32) -> bool {
    find_nominal(declarations, NominalRefView { domain, id }).is_some()
}

fn nominal_domain(component: &str) -> Id32 {
    Id32(domain_hash("clause/nominal-domain/v1", &[component.as_bytes()]).0)
}

fn definition_domain() -> Id32 {
    nominal_domain("definition")
}

fn source_unit_domain() -> Id32 {
    nominal_domain("source-unit")
}

fn change_occurrence_domain() -> Id32 {
    nominal_domain("change-occurrence")
}

fn wire_reference(reference: &NominalWireRef) -> [u8; 64] {
    let mut bytes = [0_u8; 64];
    bytes[..32].copy_from_slice(reference.domain.as_bytes());
    bytes[32..].copy_from_slice(reference.id.as_bytes());
    bytes
}

fn new_nominal_id(
    domain: Id32,
    change_input: &NominalWireRef,
    producer_input: &NominalWireRef,
    local_slot: u64,
) -> Id32 {
    let change = wire_reference(change_input);
    let producer = wire_reference(producer_input);
    Id32(
        domain_hash(
            "clause/new-nominal/v1",
            &[
                domain.as_bytes(),
                &change,
                &producer,
                &local_slot.to_be_bytes(),
            ],
        )
        .0,
    )
}

fn allocations_are_acyclic(
    declarations: &[NominalDeclaration],
) -> Result<bool, AuthorizationCheckError> {
    let mut state = Vec::new();
    state
        .try_reserve_exact(declarations.len())
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    state.resize(declarations.len(), 0_u8);

    for start in 0..declarations.len() {
        if matches!(state.get(start), Some(2)) {
            continue;
        }
        let mut stack = Vec::new();
        stack
            .try_reserve(1)
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
        stack.push((start, false));
        while let Some((index, exiting)) = stack.pop() {
            if exiting {
                let Some(slot) = state.get_mut(index) else {
                    return Ok(false);
                };
                *slot = 2;
                continue;
            }
            match state.get(index) {
                None => return Ok(false),
                Some(1) => return Ok(false),
                Some(2) => continue,
                Some(_) => {}
            }
            let Some(slot) = state.get_mut(index) else {
                return Ok(false);
            };
            *slot = 1;
            stack
                .try_reserve(3)
                .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
            stack.push((index, true));
            let Some(declaration) = declarations.get(index) else {
                return Ok(false);
            };
            if let NominalDeclaration::Allocated {
                change_input,
                producer_input,
                ..
            } = declaration
            {
                let Some(producer) = find_nominal_index(declarations, producer_input) else {
                    return Ok(false);
                };
                let Some(change) = find_nominal_index(declarations, change_input) else {
                    return Ok(false);
                };
                stack.push((producer, false));
                stack.push((change, false));
            }
        }
    }
    Ok(true)
}

fn term_nominal_references_valid(
    declarations: &[NominalDeclaration],
    term: &Term,
) -> Result<bool, AuthorizationCheckError> {
    let mut stack = Vec::new();
    stack
        .try_reserve(1)
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    stack.push(term);
    while let Some(value) = stack.pop() {
        let Term::Triple(first, second, third) = value else {
            continue;
        };
        if as_tag(first) == Some(0x04) {
            let Some(reference) = decode_nominal_ref(value)? else {
                return Ok(false);
            };
            if find_nominal(declarations, reference).is_none() {
                return Ok(false);
            }
        }
        stack
            .try_reserve(3)
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
        stack.push(third);
        stack.push(second);
        stack.push(first);
    }
    Ok(true)
}

fn expression_nominal_references_valid(
    declarations: &[NominalDeclaration],
    expression: &KExpr,
) -> Result<bool, AuthorizationCheckError> {
    let mut stack = Vec::new();
    stack
        .try_reserve(1)
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    stack.push(expression);
    while let Some(value) = stack.pop() {
        match value {
            KExpr::BytesLiteral(_) | KExpr::Var(_) => {}
            KExpr::TermLiteral(term) => {
                if !term_nominal_references_valid(declarations, term)? {
                    return Ok(false);
                }
            }
            KExpr::MakeAtom {
                kind,
                payload,
                equality,
            }
            | KExpr::MakeTriple {
                first: kind,
                second: payload,
                third: equality,
            }
            | KExpr::CaseTerm {
                scrutinee: kind,
                atom_body: payload,
                triple_body: equality,
            }
            | KExpr::CaseBytes {
                scrutinee: kind,
                empty_body: payload,
                cons_body: equality,
            } => {
                stack
                    .try_reserve(3)
                    .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
                stack.push(equality);
                stack.push(payload);
                stack.push(kind);
            }
            KExpr::Let { value, body } => {
                stack
                    .try_reserve(2)
                    .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
                stack.push(body);
                stack.push(value);
            }
            KExpr::ConcatBytes(parts)
            | KExpr::Call {
                arguments: parts, ..
            }
            | KExpr::Request {
                arguments: parts, ..
            } => {
                stack
                    .try_reserve(parts.len())
                    .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
                stack.extend(parts.iter().rev());
            }
            KExpr::CaseBytesEqual {
                left,
                right,
                equal_body,
                unequal_body,
            } => {
                stack
                    .try_reserve(4)
                    .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
                stack.push(unequal_body);
                stack.push(equal_body);
                stack.push(right);
                stack.push(left);
            }
        }
    }
    Ok(true)
}

fn nominal_table_valid(subject: &CompilerSubject) -> Result<bool, AuthorizationCheckError> {
    for pair in subject.nominal_declarations.windows(2) {
        let Some([left, right]) = <&[NominalDeclaration; 2]>::try_from(pair).ok() else {
            return resource();
        };
        if nominal_parts(left) >= nominal_parts(right) {
            return Ok(false);
        }
    }
    for declaration in &subject.nominal_declarations {
        if let NominalDeclaration::Allocated {
            domain,
            id,
            change_input,
            producer_input,
            local_slot,
        } = declaration
            && (find_nominal_index(&subject.nominal_declarations, change_input).is_none()
                || find_nominal_index(&subject.nominal_declarations, producer_input).is_none()
                || *id != new_nominal_id(*domain, change_input, producer_input, *local_slot))
        {
            return Ok(false);
        }
    }
    if !allocations_are_acyclic(&subject.nominal_declarations)? {
        return Ok(false);
    }
    let definition_domain = definition_domain();
    for definition in &subject.program {
        if !nominal_exists(
            &subject.nominal_declarations,
            definition_domain,
            definition.id,
        ) || !expression_nominal_references_valid(
            &subject.nominal_declarations,
            &definition.body,
        )? {
            return Ok(false);
        }
    }
    if !nominal_exists(
        &subject.nominal_declarations,
        definition_domain,
        subject.interface.compile,
    ) || !nominal_exists(
        &subject.nominal_declarations,
        definition_domain,
        subject.interface.admit_propose,
    ) || !term_nominal_references_valid(&subject.nominal_declarations, &subject.build_request)?
    {
        return Ok(false);
    }
    Ok(true)
}

fn program_strictly_sorted(subject: &CompilerSubject) -> bool {
    let mut prior: Option<Id32> = None;
    for definition in &subject.program {
        if let Some(prior_id) = prior
            && prior_id >= definition.id
        {
            return false;
        }
        prior = Some(definition.id);
    }
    true
}

fn core_failure(
    candidate: &DecodedCompilerPackage,
) -> Result<Option<AuthorizationFailure>, AuthorizationCheckError> {
    let subject = &candidate.package().subject;
    if map_encode(canonical_subject_bytes(subject))? != candidate.exact_subject() {
        return Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::SubjectStructure,
        )));
    }
    if !nominal_table_valid(subject)? {
        return Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::NominalTable,
        )));
    }
    if !program_strictly_sorted(subject) {
        return Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::DefinitionOrderOrDuplicate,
        )));
    }
    let compile = subject
        .program
        .iter()
        .find(|definition| definition.id == subject.interface.compile);
    let admit = subject
        .program
        .iter()
        .find(|definition| definition.id == subject.interface.admit_propose);
    let (Some(compile), Some(admit)) = (compile, admit) else {
        return Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::EntrypointResolution,
        )));
    };
    if subject.interface.compile == subject.interface.admit_propose {
        return Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::EntrypointAliased,
        )));
    }
    if compile.arguments.as_slice() != [KSort::Term]
        || compile.result != KSort::Term
        || admit.arguments.as_slice() != [KSort::Term]
        || admit.result != KSort::Term
    {
        return Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::EntrypointSignature,
        )));
    }
    let evaluator = match Evaluator::new_unprofiled(&subject.program) {
        Ok(evaluator) => evaluator,
        Err(StaticError::ResourceExhausted) => return resource(),
        Err(_) => {
            return Ok(Some(failure(
                AuthorizationStage::CoreWellFormedness,
                AuthorizationCode::StaticRule,
            )));
        }
    };
    match evaluator.check_physical_profile() {
        Ok(()) => Ok(None),
        Err(StaticError::ResourceExhausted) => resource(),
        Err(_) => Ok(Some(failure(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::PhysicalRequestSignature,
        ))),
    }
}

fn common_failure(
    candidate: &DecodedCompilerPackage,
) -> Result<Option<AuthorizationFailure>, AuthorizationCheckError> {
    let exact_manifest = map_encode(exact_core_manifest_bytes())?;
    if candidate.exact_core_manifest() != exact_manifest {
        return Ok(Some(failure(
            AuthorizationStage::CoreManifest,
            AuthorizationCode::ManifestMismatch,
        )));
    }
    core_failure(candidate)
}

fn compiler_revision_id(exact_subject: &[u8]) -> Id32 {
    Id32(domain_hash("clause/compiler-revision/v1", &[exact_subject]).0)
}

fn references_strictly_sorted(references: &[NominalRefView]) -> bool {
    let mut prior: Option<&NominalRefView> = None;
    for reference in references {
        if let Some(prior_reference) = prior
            && (prior_reference.domain, prior_reference.id) >= (reference.domain, reference.id)
        {
            return false;
        }
        prior = Some(reference);
    }
    true
}

fn plan_valid(
    subject: &CompilerSubject,
    plan: &IdentityPlanView,
    predecessor: Option<&DecodedCompilerPackage>,
) -> bool {
    if !references_strictly_sorted(&plan.retained)
        || !references_strictly_sorted(&plan.seed_inputs)
        || plan
            .retained
            .iter()
            .any(|reference| plan.seed_inputs.contains(reference))
        || plan.retained.iter().any(|reference| {
            !matches!(
                find_nominal(&subject.nominal_declarations, *reference),
                Some(NominalDeclaration::RetainedSeed { .. })
            )
        })
        || plan.seed_inputs.iter().any(|reference| {
            !matches!(
                find_nominal(&subject.nominal_declarations, *reference),
                Some(NominalDeclaration::Seed { .. })
            )
        })
    {
        return false;
    }

    match (&subject.lineage, predecessor) {
        (CompilerLineage::Genesis, None) => {
            plan.retained.is_empty()
                && subject
                    .nominal_declarations
                    .iter()
                    .all(|declaration| match declaration {
                        NominalDeclaration::Seed { .. } => {
                            plan.seed_inputs.contains(&nominal_reference(declaration))
                        }
                        NominalDeclaration::RetainedSeed { .. } => false,
                        NominalDeclaration::Allocated { .. } => true,
                    })
        }
        (CompilerLineage::Successor { .. }, Some(prior)) => {
            let prior_revision = compiler_revision_id(prior.exact_subject());
            subject.nominal_declarations.iter().all(|declaration| {
                let reference = nominal_reference(declaration);
                match declaration {
                    NominalDeclaration::RetainedSeed {
                        predecessor_revision_id,
                        ..
                    } => {
                        *predecessor_revision_id == prior_revision
                            && matches!(
                                find_nominal(
                                    &prior.package().subject.nominal_declarations,
                                    reference,
                                ),
                                Some(NominalDeclaration::Seed { .. })
                                    | Some(NominalDeclaration::RetainedSeed { .. })
                            )
                            && plan.retained.contains(&reference)
                    }
                    NominalDeclaration::Seed { .. } => {
                        find_nominal(&prior.package().subject.nominal_declarations, reference)
                            .is_none()
                            && plan.seed_inputs.contains(&reference)
                    }
                    NominalDeclaration::Allocated { .. } => {
                        match find_nominal(&prior.package().subject.nominal_declarations, reference)
                        {
                            None => true,
                            Some(prior_declaration) => prior_declaration == declaration,
                        }
                    }
                }
            })
        }
        _ => false,
    }
}

fn build_failure(
    candidate: &DecodedCompilerPackage,
    request_term: &Term,
    genesis_fuels: Option<(u64, u64)>,
    predecessor: Option<&DecodedCompilerPackage>,
) -> Result<Option<AuthorizationFailure>, AuthorizationCheckError> {
    let fail = |code| Some(failure(AuthorizationStage::BuildRequest, code));
    let Some(request) = decode_build_request(request_term)? else {
        return Ok(fail(AuthorizationCode::BuildRequestShape));
    };
    if request_term != &candidate.package().subject.build_request {
        return Ok(fail(AuthorizationCode::DetachedBuildRequest));
    }
    let route_matches = match (
        &candidate.package().subject.lineage,
        request.base,
        predecessor,
    ) {
        (CompilerLineage::Genesis, BaseView::Genesis, None) => true,
        (
            CompilerLineage::Successor { .. },
            BaseView::Accepted {
                package_hash,
                revision_id,
            },
            Some(prior),
        ) => {
            package_hash == compiler_package_hash(prior.exact_input())
                && revision_id == compiler_revision_id(prior.exact_subject())
        }
        _ => false,
    };
    if !route_matches {
        return Ok(fail(AuthorizationCode::BaseMismatch));
    }
    if request.core_contract_id != map_encode(core_contract_id())? {
        return Ok(fail(AuthorizationCode::CoreContractMismatch));
    }
    if request.physical_profile_id != map_encode(physical_profile_id())? {
        return Ok(fail(AuthorizationCode::PhysicalProfileMismatch));
    }
    let mut prior_unit_id = None;
    for source in &request.source_units {
        if let Some(prior) = prior_unit_id
            && prior >= source.unit_id
        {
            return Ok(fail(AuthorizationCode::SourceOrderOrDuplicate));
        }
        prior_unit_id = Some(source.unit_id);
    }
    if request.source_units.iter().any(|source| {
        !nominal_exists(
            &candidate.package().subject.nominal_declarations,
            source_unit_domain(),
            source.unit_id,
        )
    }) {
        return Ok(fail(AuthorizationCode::SourceOrderOrDuplicate));
    }
    if request
        .source_units
        .iter()
        .any(|source| source.artifact_id != source_artifact_id(source.bytes))
    {
        return Ok(fail(AuthorizationCode::SourceArtifactMismatch));
    }
    if !plan_valid(
        &candidate.package().subject,
        &request.identity_plan,
        predecessor,
    ) {
        return Ok(fail(AuthorizationCode::IdentityPlanMismatch));
    }
    let change_matches = match &candidate.package().subject.lineage {
        CompilerLineage::Genesis => nominal_exists(
            &candidate.package().subject.nominal_declarations,
            change_occurrence_domain(),
            request.change_occurrence_id,
        ),
        CompilerLineage::Successor {
            change_occurrence_id,
            ..
        } => {
            request.change_occurrence_id == *change_occurrence_id
                && nominal_exists(
                    &candidate.package().subject.nominal_declarations,
                    change_occurrence_domain(),
                    *change_occurrence_id,
                )
        }
    };
    if !change_matches {
        return Ok(fail(AuthorizationCode::ChangeOccurrenceMismatch));
    }
    if !request.declared_physical_inputs.is_empty() {
        return Ok(fail(AuthorizationCode::PhysicalInputsNonempty));
    }
    match genesis_fuels {
        Some((compile, admission))
            if compile == 0
                || admission == 0
                || request.compile_fuel != compile
                || request.admission_fuel != admission =>
        {
            Ok(fail(AuthorizationCode::FuelInvalid))
        }
        None if request.compile_fuel == 0 || request.admission_fuel == 0 => {
            Ok(fail(AuthorizationCode::FuelInvalid))
        }
        _ => Ok(None),
    }
}

fn final_failure(
    candidate: &DecodedCompilerPackage,
    identity: FinalPackageIdentityInput<'_>,
) -> Option<AuthorizationFailure> {
    (identity.exact_package_bytes != candidate.exact_input()
        || identity.package_hash != compiler_package_hash(identity.exact_package_bytes))
    .then(|| {
        failure(
            AuthorizationStage::FinalAuthorization,
            AuthorizationCode::FinalIdentityMismatch,
        )
    })
}

fn atom(kind: &[u8], payload: Vec<u8>) -> Result<Term, AuthorizationCheckError> {
    Ok(Term::Atom {
        kind: try_copy_bytes(kind).map_err(|_| AuthorizationCheckError::ResourceExhausted)?,
        canonical_payload: payload,
        equality_contract: try_copy_bytes(K_EQ)
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?,
    })
}

fn tag(value: u8) -> Result<Term, AuthorizationCheckError> {
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(1)
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    payload.push(value);
    atom(K_TAG, payload)
}

fn bytes_term(value: &[u8]) -> Result<Term, AuthorizationCheckError> {
    atom(
        K_BYTES,
        try_copy_bytes(value).map_err(|_| AuthorizationCheckError::ResourceExhausted)?,
    )
}

fn list_term(values: Vec<Term>) -> Result<Term, AuthorizationCheckError> {
    let mut tail = tag(0x00)?;
    for head in values.into_iter().rev() {
        tail = Term::try_triple(tag(0x01)?, head, tail)
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    }
    Ok(tail)
}

fn record_term(tag_value: u8, fields: Vec<Term>) -> Result<Term, AuthorizationCheckError> {
    Term::try_triple(tag(tag_value)?, list_term(fields)?, tag(0x00)?)
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)
}

fn admission_request_term(
    build_request: &Term,
    subject_bytes: &[u8],
    observations: &Term,
) -> Result<Term, AuthorizationCheckError> {
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(3)
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    fields.push(
        build_request
            .try_clone_resource()
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?,
    );
    fields.push(bytes_term(subject_bytes)?);
    fields.push(
        observations
            .try_clone_resource()
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?,
    );
    record_term(0x16, fields)
}

fn result_bytes(
    value: &KValue,
    expected_tag: u8,
) -> Result<Option<&[u8]>, AuthorizationCheckError> {
    let KValue::Term(term) = value else {
        return Ok(None);
    };
    let Some(fields) = record_fields(term, expected_tag)? else {
        return Ok(None);
    };
    let Some([subject]) = exactly(fields) else {
        return Ok(None);
    };
    Ok(as_bytes(subject))
}

fn receipt_shape_valid(receipt: &EvalReceipt) -> bool {
    receipt.format_version == 0x00
}

struct AcceptedPredecessor<'a> {
    exact_bytes: &'a [u8],
    offered_bytes: &'a [u8],
    decoded: DecodedCompilerPackage,
}

fn resolve_predecessor<'a>(
    input: PredecessorInput<'a>,
) -> Result<Option<AcceptedPredecessor<'a>>, AuthorizationCheckError> {
    let PredecessorInput::Accepted {
        exact_bytes,
        acceptance,
        offered_bytes,
    } = input
    else {
        return Ok(None);
    };
    if acceptance.exact_bytes != exact_bytes {
        return Ok(None);
    }
    let decoded = decode(exact_bytes).map_err(AuthorizationCheckError::Decode)?;
    Ok(Some(AcceptedPredecessor {
        exact_bytes,
        offered_bytes,
        decoded,
    }))
}

fn predecessor_evaluator<'a>(
    predecessor: &'a AcceptedPredecessor<'_>,
) -> Result<Option<Evaluator<'a>>, AuthorizationCheckError> {
    if predecessor.decoded.exact_input() != predecessor.exact_bytes
        || common_failure(&predecessor.decoded)?.is_some()
    {
        return Ok(None);
    }
    match Evaluator::new(&predecessor.decoded.package().subject.program) {
        Ok(evaluator) => Ok(Some(evaluator)),
        Err(StaticError::RecursionLimit | StaticError::ResourceExhausted) => resource(),
        Err(_) => Ok(None),
    }
}

fn replay_entrypoint(
    predecessor: &AcceptedPredecessor<'_>,
    entrypoint: Id32,
    argument: &Term,
    fuel: u64,
) -> Result<Option<Evaluation>, AuthorizationCheckError> {
    if fuel == 0 {
        return Ok(None);
    }
    let Some(evaluator) = predecessor_evaluator(predecessor)? else {
        return Ok(None);
    };
    let definition = predecessor
        .decoded
        .package()
        .subject
        .program
        .iter()
        .find(|definition| definition.id == entrypoint);
    if !matches!(
        definition,
        Some(definition)
            if definition.arguments.as_slice() == [KSort::Term]
                && definition.result == KSort::Term
    ) {
        return Ok(None);
    }
    let argument = KValue::Term(
        argument
            .try_clone_resource()
            .map_err(|_| AuthorizationCheckError::ResourceExhausted)?,
    );
    match evaluator.replay_entrypoint(entrypoint, &[argument], fuel) {
        Ok(evaluation) => Ok(Some(evaluation)),
        Err(error) if replay_error_is_resource_exhausted(&error) => resource(),
        Err(_) => Ok(None),
    }
}

fn replay_error_is_resource_exhausted(error: &EvalError) -> bool {
    matches!(
        error,
        EvalError::RecursionLimit
            | EvalError::ResourceExhausted
            | EvalError::Static(StaticError::RecursionLimit | StaticError::ResourceExhausted)
            | EvalError::Physical(PhysicalError::ResourceExhausted)
    )
}

fn compile_replay(
    candidate: &DecodedCompilerPackage,
    request: &BuildRequestView<'_>,
    predecessor: &AcceptedPredecessor<'_>,
    receipt: &EvalReceipt,
) -> Result<Result<Term, AuthorizationFailure>, AuthorizationCheckError> {
    let fail = |code| Err(failure(AuthorizationStage::CompileEvaluation, code));
    if !receipt_shape_valid(receipt) {
        return Ok(fail(AuthorizationCode::EvidenceShapeMismatch));
    }
    let Some(result) = replay_entrypoint(
        predecessor,
        predecessor.decoded.package().subject.interface.compile,
        &candidate.package().subject.build_request,
        request.compile_fuel,
    )?
    else {
        return Ok(fail(AuthorizationCode::EvaluationFault));
    };
    let value_hash = match super::eval_receipt_value_hash(&result.value) {
        Ok(value) => value,
        Err(EncodeError::ResourceExhausted) => return resource(),
        Err(_) => return Ok(fail(AuthorizationCode::EvaluationFault)),
    };
    if value_hash != receipt.expected_value_hash {
        return Ok(fail(AuthorizationCode::ReceiptValueMismatch));
    }
    if result.remaining_fuel != receipt.expected_remaining_fuel {
        return Ok(fail(AuthorizationCode::ReceiptFuelMismatch));
    }
    let Some(subject_bytes) = result_bytes(&result.value, 0x14)? else {
        return Ok(fail(AuthorizationCode::UnexpectedResult));
    };
    if subject_bytes != candidate.exact_subject() {
        return Ok(fail(AuthorizationCode::SubjectMismatch));
    }
    let observations = match result.observations.try_to_term() {
        Ok(value) => value,
        Err(PhysicalError::ResourceExhausted) => return resource(),
        Err(_) => return Ok(fail(AuthorizationCode::ObservationMismatch)),
    };
    let observations_hash = match super::eval_receipt_observations_hash(&observations) {
        Ok(value) => value,
        Err(EncodeError::ResourceExhausted) => return resource(),
        Err(_) => return Ok(fail(AuthorizationCode::ObservationMismatch)),
    };
    if observations_hash != receipt.expected_observations_hash {
        return Ok(fail(AuthorizationCode::ObservationMismatch));
    }
    Ok(Ok(observations))
}

fn admission_replay(
    candidate: &DecodedCompilerPackage,
    request: &BuildRequestView<'_>,
    predecessor: &AcceptedPredecessor<'_>,
    compile_observations: &Term,
    receipt: &EvalReceipt,
) -> Result<Result<(), AuthorizationFailure>, AuthorizationCheckError> {
    let fail = |code| Err(failure(AuthorizationStage::AdmissionEvaluation, code));
    if !receipt_shape_valid(receipt) {
        return Ok(fail(AuthorizationCode::EvidenceShapeMismatch));
    }
    let argument = admission_request_term(
        &candidate.package().subject.build_request,
        candidate.exact_subject(),
        compile_observations,
    )?;
    let Some(result) = replay_entrypoint(
        predecessor,
        predecessor
            .decoded
            .package()
            .subject
            .interface
            .admit_propose,
        &argument,
        request.admission_fuel,
    )?
    else {
        return Ok(fail(AuthorizationCode::EvaluationFault));
    };
    let value_hash = match super::eval_receipt_value_hash(&result.value) {
        Ok(value) => value,
        Err(EncodeError::ResourceExhausted) => return resource(),
        Err(_) => return Ok(fail(AuthorizationCode::EvaluationFault)),
    };
    if value_hash != receipt.expected_value_hash {
        return Ok(fail(AuthorizationCode::ReceiptValueMismatch));
    }
    if result.remaining_fuel != receipt.expected_remaining_fuel {
        return Ok(fail(AuthorizationCode::ReceiptFuelMismatch));
    }
    let Some(subject_bytes) = result_bytes(&result.value, 0x17)? else {
        return Ok(fail(AuthorizationCode::UnexpectedResult));
    };
    if subject_bytes != candidate.exact_subject() {
        return Ok(fail(AuthorizationCode::SubjectMismatch));
    }
    let observations = match result.observations.try_to_term() {
        Ok(value) => value,
        Err(PhysicalError::ResourceExhausted) => return resource(),
        Err(_) => return Ok(fail(AuthorizationCode::ObservationMismatch)),
    };
    let observations_hash = match super::eval_receipt_observations_hash(&observations) {
        Ok(value) => value,
        Err(EncodeError::ResourceExhausted) => return resource(),
        Err(_) => return Ok(fail(AuthorizationCode::ObservationMismatch)),
    };
    if observations_hash != receipt.expected_observations_hash {
        return Ok(fail(AuthorizationCode::ObservationMismatch));
    }
    Ok(Ok(()))
}

fn package_is_exactly_canonical(
    candidate: &DecodedCompilerPackage,
) -> Result<bool, AuthorizationCheckError> {
    Ok(map_encode(encode(candidate.package()))? == candidate.exact_input())
}

/// Strictly decode and check a genesis package. This route deliberately does
/// not invoke either compiler entrypoint; genesis has no evaluation receipt.
pub fn authorize_genesis(
    input: &[u8],
    request: GenesisAuthorizationRequest<'_>,
) -> Result<AuthorizationVerdict, AuthorizationCheckError> {
    let candidate = decode(input).map_err(AuthorizationCheckError::Decode)?;
    if candidate.exact_input() != input || !package_is_exactly_canonical(&candidate)? {
        return Ok(deny(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::SubjectStructure,
        ));
    }
    if let Some(failure) = common_failure(&candidate)? {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    if !matches!(
        candidate.package().subject.lineage,
        CompilerLineage::Genesis
    ) {
        return Ok(deny(
            AuthorizationStage::GenesisAnchor,
            AuthorizationCode::GenesisWrongLineage,
        ));
    }
    if request.evidence != &candidate.package().evidence
        || !matches!(request.evidence, CompilerEvidence::Genesis)
    {
        return Ok(deny(
            AuthorizationStage::GenesisAnchor,
            AuthorizationCode::GenesisEvidenceNotEmpty,
        ));
    }
    let OwnerAnchorInput::Supplied(witness) = request.owner_anchor else {
        return Ok(deny(
            AuthorizationStage::GenesisAnchor,
            AuthorizationCode::MissingAnchor,
        ));
    };
    let observation = witness.observation();
    let selected_length = u64::try_from(observation.exact_selected_bytes.len())
        .map_err(|_| AuthorizationCheckError::ResourceExhausted)?;
    if observation.selected_byte_length != selected_length
        || observation.selected_package_hash
            != compiler_package_hash(observation.exact_selected_bytes)
        || observation.exact_selected_bytes != input
    {
        return Ok(deny(
            AuthorizationStage::GenesisAnchor,
            AuthorizationCode::AnchorBytesMismatch,
        ));
    }
    if let Some(failure) = build_failure(
        &candidate,
        request.build_request,
        Some((request.compile_fuel_limit, request.admission_fuel_limit)),
        None,
    )? {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    if let Some(failure) = final_failure(&candidate, request.final_identity) {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    copy_authorized(input)
}

/// Strictly decode and check one exact-predecessor successor. Both receipts
/// are verified by complete replay; receipt fields never construct a replay.
pub fn authorize_successor(
    input: &[u8],
    request: SuccessorAuthorizationRequest<'_>,
) -> Result<AuthorizationVerdict, AuthorizationCheckError> {
    let candidate = decode(input).map_err(AuthorizationCheckError::Decode)?;
    if candidate.exact_input() != input || !package_is_exactly_canonical(&candidate)? {
        return Ok(deny(
            AuthorizationStage::CoreWellFormedness,
            AuthorizationCode::SubjectStructure,
        ));
    }
    if let Some(failure) = common_failure(&candidate)? {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    let CompilerLineage::Successor {
        predecessor_locator,
        ..
    } = candidate.package().subject.lineage
    else {
        return Ok(deny(
            AuthorizationStage::ExactPredecessor,
            AuthorizationCode::SuccessorWrongLineage,
        ));
    };
    let (offered, accepted_candidate) = match request.predecessor {
        PredecessorInput::Absent { offered_bytes } => (offered_bytes, false),
        PredecessorInput::Accepted {
            exact_bytes,
            offered_bytes,
            ..
        } => (offered_bytes, exact_bytes == candidate.exact_input()),
    };
    if offered == candidate.exact_input() || accepted_candidate {
        return Ok(deny(
            AuthorizationStage::ExactPredecessor,
            AuthorizationCode::CandidateOrSelfPredecessor,
        ));
    }
    let Some(predecessor) = resolve_predecessor(request.predecessor)? else {
        return Ok(deny(
            AuthorizationStage::ExactPredecessor,
            AuthorizationCode::PredecessorNotAccepted,
        ));
    };
    if predecessor_locator != compiler_package_hash(predecessor.exact_bytes) {
        return Ok(deny(
            AuthorizationStage::ExactPredecessor,
            AuthorizationCode::LocatorMismatch,
        ));
    }
    if predecessor.offered_bytes != predecessor.exact_bytes
        || predecessor.decoded.exact_input() != predecessor.exact_bytes
    {
        return Ok(deny(
            AuthorizationStage::ExactPredecessor,
            AuthorizationCode::PredecessorBytesMismatch,
        ));
    }
    if let Some(failure) = build_failure(
        &candidate,
        request.build_request,
        None,
        Some(&predecessor.decoded),
    )? {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    let Some(build_request) = decode_build_request(request.build_request)? else {
        return Ok(deny(
            AuthorizationStage::BuildRequest,
            AuthorizationCode::BuildRequestShape,
        ));
    };
    let CompilerEvidence::Successor {
        compile_receipt,
        admission_receipt,
    } = request.evidence
    else {
        return Ok(deny(
            AuthorizationStage::CompileEvaluation,
            AuthorizationCode::EvidenceShapeMismatch,
        ));
    };
    let compile_observations =
        match compile_replay(&candidate, &build_request, &predecessor, compile_receipt)? {
            Ok(observations) => observations,
            Err(failure) => return Ok(AuthorizationVerdict::Unauthorized(failure)),
        };
    if let Err(failure) = admission_replay(
        &candidate,
        &build_request,
        &predecessor,
        &compile_observations,
        admission_receipt,
    )? {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    if request.evidence != &candidate.package().evidence
        || map_encode(canonical_evidence_bytes(request.evidence))? != candidate.exact_evidence()
    {
        return Ok(deny(
            AuthorizationStage::EvidenceAttachment,
            AuthorizationCode::EvidenceDetached,
        ));
    }
    if map_encode(canonical_subject_bytes(&candidate.package().subject))?
        != candidate.exact_subject()
    {
        return Ok(deny(
            AuthorizationStage::EvidenceAttachment,
            AuthorizationCode::SubjectChangedAfterCompile,
        ));
    }
    if !package_is_exactly_canonical(&candidate)? {
        return Ok(deny(
            AuthorizationStage::EvidenceAttachment,
            AuthorizationCode::PackageChangedAfterEvidence,
        ));
    }
    if let Some(failure) = final_failure(&candidate, request.final_identity) {
        return Ok(AuthorizationVerdict::Unauthorized(failure));
    }
    copy_authorized(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::compiler_package_v3::{CompilerInterface, CoreManifest, Definition, FallibleBox};

    fn id(value: u8) -> Id32 {
        Id32([value; 32])
    }

    fn id_term(value: Id32) -> Term {
        atom(K_ID32, value.0.to_vec()).expect("test ID allocation")
    }

    fn u64_term(value: u64) -> Term {
        atom(K_U64, value.to_be_bytes().to_vec()).expect("test U64 allocation")
    }

    fn opaque(payload: &[u8]) -> Term {
        Term::Atom {
            kind: b"opaque-kind".to_vec(),
            canonical_payload: payload.to_vec(),
            equality_contract: b"opaque-equality".to_vec(),
        }
    }

    fn boxed_expression(value: KExpr) -> FallibleBox<KExpr> {
        FallibleBox::try_new(value).expect("test expression allocation")
    }

    fn nominal_ref_term(reference: NominalRefView) -> Term {
        record_term(0x04, vec![id_term(reference.domain), id_term(reference.id)])
            .expect("test nominal reference allocation")
    }

    fn wrapped_ref(tag_value: u8, reference: NominalRefView) -> Term {
        record_term(tag_value, vec![nominal_ref_term(reference)])
            .expect("test reference wrapper allocation")
    }

    fn build_request_term_with_base(
        declarations: &[NominalDeclaration],
        base: Term,
        change: Id32,
        compile_fuel: u64,
        admission_fuel: u64,
    ) -> Term {
        let seed_inputs = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                NominalDeclaration::Seed { .. } => {
                    Some(wrapped_ref(0x0a, nominal_reference(declaration)))
                }
                _ => None,
            })
            .collect();
        let identity_plan = record_term(
            0x08,
            vec![
                list_term(Vec::new()).expect("empty retained list"),
                list_term(seed_inputs).expect("seed input list"),
            ],
        )
        .expect("identity plan");
        record_term(
            0x13,
            vec![
                base,
                id_term(Id32(core_contract_id().unwrap().0)),
                id_term(Id32(physical_profile_id().unwrap().0)),
                opaque(b"target"),
                list_term(Vec::new()).expect("source list"),
                opaque(b"base inputs"),
                identity_plan,
                id_term(change),
                opaque(b"options"),
                u64_term(compile_fuel),
                u64_term(admission_fuel),
                list_term(Vec::new()).expect("physical input list"),
            ],
        )
        .expect("build request")
    }

    fn build_request_term(
        declarations: &[NominalDeclaration],
        change: Id32,
        compile_fuel: u64,
        admission_fuel: u64,
    ) -> Term {
        build_request_term_with_base(
            declarations,
            record_term(0x10, Vec::new()).expect("genesis base"),
            change,
            compile_fuel,
            admission_fuel,
        )
    }

    fn case_term_bytes(scrutinee: KExpr, triple_body: KExpr) -> KExpr {
        KExpr::CaseTerm {
            scrutinee: boxed_expression(scrutinee),
            atom_body: boxed_expression(KExpr::BytesLiteral(Vec::new())),
            triple_body: boxed_expression(triple_body),
        }
    }

    fn atom_payload(scrutinee: KExpr) -> KExpr {
        KExpr::CaseTerm {
            scrutinee: boxed_expression(scrutinee),
            atom_body: boxed_expression(KExpr::Var(1)),
            triple_body: boxed_expression(KExpr::BytesLiteral(Vec::new())),
        }
    }

    fn list_field_atom_payload(list: KExpr, field: usize) -> KExpr {
        let selected = if field == 0 {
            atom_payload(KExpr::Var(1))
        } else {
            list_field_atom_payload(KExpr::Var(2), field - 1)
        };
        case_term_bytes(list, selected)
    }

    fn build_base_field_payload(field: usize) -> KExpr {
        case_term_bytes(
            KExpr::Var(0),
            case_term_bytes(
                KExpr::Var(1),
                case_term_bytes(KExpr::Var(1), list_field_atom_payload(KExpr::Var(1), field)),
            ),
        )
    }

    fn admission_subject_payload() -> KExpr {
        case_term_bytes(KExpr::Var(0), list_field_atom_payload(KExpr::Var(1), 1))
    }

    fn result_expression(tag_value: u8, payload: KExpr) -> KExpr {
        let subject = KExpr::MakeAtom {
            kind: boxed_expression(KExpr::BytesLiteral(K_BYTES.to_vec())),
            payload: boxed_expression(payload),
            equality: boxed_expression(KExpr::BytesLiteral(K_EQ.to_vec())),
        };
        let fields = KExpr::MakeTriple {
            first: boxed_expression(KExpr::TermLiteral(
                tag(0x01).expect("test list marker allocation"),
            )),
            second: boxed_expression(subject),
            third: boxed_expression(KExpr::TermLiteral(
                tag(0x00).expect("test list terminator allocation"),
            )),
        };
        KExpr::MakeTriple {
            first: boxed_expression(KExpr::TermLiteral(
                tag(tag_value).expect("test result marker allocation"),
            )),
            second: boxed_expression(fields),
            third: boxed_expression(KExpr::TermLiteral(
                tag(0x00).expect("test record terminator allocation"),
            )),
        }
    }

    const PREDECESSOR_HASH_MARKER: [u8; 32] = [0xa5; 32];
    const PREDECESSOR_REVISION_MARKER: [u8; 32] = [0x5a; 32];

    fn dynamic_subject_bytes(template: &[u8]) -> KExpr {
        let mut parts = Vec::new();
        let mut start = 0_usize;
        let mut hash_fields = 0_usize;
        let mut revision_fields = 0_usize;
        while start < template.len() {
            let hash = template[start..]
                .windows(PREDECESSOR_HASH_MARKER.len())
                .position(|window| window == PREDECESSOR_HASH_MARKER)
                .map(|offset| start + offset);
            let revision = template[start..]
                .windows(PREDECESSOR_REVISION_MARKER.len())
                .position(|window| window == PREDECESSOR_REVISION_MARKER)
                .map(|offset| start + offset);
            let next = match (hash, revision) {
                (Some(hash), Some(revision)) if hash <= revision => Some((hash, true)),
                (Some(_), Some(revision)) => Some((revision, false)),
                (Some(hash), None) => Some((hash, true)),
                (None, Some(revision)) => Some((revision, false)),
                (None, None) => None,
            };
            let Some((position, is_hash)) = next else {
                parts.push(KExpr::BytesLiteral(template[start..].to_vec()));
                break;
            };
            if position > start {
                parts.push(KExpr::BytesLiteral(template[start..position].to_vec()));
            }
            if is_hash {
                parts.push(build_base_field_payload(0));
                hash_fields += 1;
            } else {
                parts.push(build_base_field_payload(1));
                revision_fields += 1;
            }
            start = position + PREDECESSOR_HASH_MARKER.len();
        }
        assert_eq!(hash_fields, 2, "subject has lineage and base hash fields");
        assert_eq!(revision_fields, 1, "subject has one base revision field");
        KExpr::ConcatBytes(parts)
    }

    fn successor_package(
        predecessor_hash: Hash32,
        predecessor_revision: Id32,
        compile_fuel: u64,
        admission_fuel: u64,
    ) -> super::super::CompilerPackage {
        let compile = id(11);
        let admit = id(12);
        let change = id(13);
        let mut declarations = vec![
            NominalDeclaration::Seed {
                domain: definition_domain(),
                id: compile,
            },
            NominalDeclaration::Seed {
                domain: definition_domain(),
                id: admit,
            },
            NominalDeclaration::Seed {
                domain: change_occurrence_domain(),
                id: change,
            },
        ];
        declarations.sort_by_key(nominal_parts);
        let base = record_term(
            0x11,
            vec![
                id_term(Id32(predecessor_hash.0)),
                id_term(predecessor_revision),
            ],
        )
        .expect("accepted predecessor base");
        let build_request =
            build_request_term_with_base(&declarations, base, change, compile_fuel, admission_fuel);
        let empty_receipt = EvalReceipt {
            format_version: 0x00,
            expected_value_hash: Hash32([0; 32]),
            expected_remaining_fuel: 0,
            expected_observations_hash: Hash32([0; 32]),
        };
        super::super::CompilerPackage {
            core_manifest: CoreManifest::canonical_v1(),
            subject: CompilerSubject {
                lineage: CompilerLineage::Successor {
                    predecessor_locator: predecessor_hash,
                    change_occurrence_id: change,
                },
                nominal_declarations: declarations,
                interface: CompilerInterface {
                    compile,
                    admit_propose: admit,
                },
                program: vec![
                    Definition {
                        id: compile,
                        arguments: vec![KSort::Term],
                        result: KSort::Term,
                        body: KExpr::Var(0),
                    },
                    Definition {
                        id: admit,
                        arguments: vec![KSort::Term],
                        result: KSort::Term,
                        body: KExpr::Var(0),
                    },
                ],
                build_request,
            },
            evidence: CompilerEvidence::Successor {
                compile_receipt: empty_receipt,
                admission_receipt: empty_receipt,
            },
        }
    }

    fn predecessor_package(
        compile_body: KExpr,
        admission_body: KExpr,
    ) -> super::super::CompilerPackage {
        let compile = id(1);
        let admit = id(2);
        let change = id(3);
        let mut declarations = vec![
            NominalDeclaration::Seed {
                domain: definition_domain(),
                id: compile,
            },
            NominalDeclaration::Seed {
                domain: definition_domain(),
                id: admit,
            },
            NominalDeclaration::Seed {
                domain: change_occurrence_domain(),
                id: change,
            },
        ];
        declarations.sort_by_key(nominal_parts);
        let build_request = build_request_term(&declarations, change, 1, 1);
        super::super::CompilerPackage {
            core_manifest: CoreManifest::canonical_v1(),
            subject: CompilerSubject {
                lineage: CompilerLineage::Genesis,
                nominal_declarations: declarations,
                interface: CompilerInterface {
                    compile,
                    admit_propose: admit,
                },
                program: vec![
                    Definition {
                        id: compile,
                        arguments: vec![KSort::Term],
                        result: KSort::Term,
                        body: compile_body,
                    },
                    Definition {
                        id: admit,
                        arguments: vec![KSort::Term],
                        result: KSort::Term,
                        body: admission_body,
                    },
                ],
                build_request,
            },
            evidence: CompilerEvidence::Genesis,
        }
    }

    fn successor_authorization_with_compile_body(
        compile_body: KExpr,
        compile_fuel: u64,
    ) -> Result<AuthorizationVerdict, AuthorizationCheckError> {
        let predecessor = predecessor_package(compile_body, KExpr::Var(0));
        let predecessor_bytes = encode(&predecessor).expect("predecessor encodes");
        let predecessor_decoded = decode(&predecessor_bytes).expect("predecessor decodes");
        let candidate = successor_package(
            compiler_package_hash(&predecessor_bytes),
            compiler_revision_id(predecessor_decoded.exact_subject()),
            compile_fuel,
            1,
        );
        let candidate_bytes = encode(&candidate).expect("candidate encodes");
        let acceptance = AcceptedExact::from_outer_admission(&predecessor_bytes);
        authorize_successor(
            &candidate_bytes,
            SuccessorAuthorizationRequest {
                predecessor: PredecessorInput::Accepted {
                    exact_bytes: &predecessor_bytes,
                    acceptance,
                    offered_bytes: &predecessor_bytes,
                },
                build_request: &candidate.subject.build_request,
                evidence: &candidate.evidence,
                final_identity: FinalPackageIdentityInput {
                    package_hash: compiler_package_hash(&candidate_bytes),
                    exact_package_bytes: &candidate_bytes,
                },
            },
        )
    }

    fn doubling_bytes(rounds: usize) -> KExpr {
        let mut expression = KExpr::BytesLiteral(vec![0x5a]);
        for _ in 0..rounds {
            expression = KExpr::Let {
                value: boxed_expression(expression),
                body: boxed_expression(KExpr::ConcatBytes(vec![KExpr::Var(0), KExpr::Var(0)])),
            };
        }
        expression
    }

    fn genesis_package() -> super::super::CompilerPackage {
        let compile = id(1);
        let admit = id(2);
        let change = id(3);
        let mut declarations = vec![
            NominalDeclaration::Seed {
                domain: definition_domain(),
                id: compile,
            },
            NominalDeclaration::Seed {
                domain: definition_domain(),
                id: admit,
            },
            NominalDeclaration::Seed {
                domain: change_occurrence_domain(),
                id: change,
            },
        ];
        declarations.sort_by_key(nominal_parts);
        let build_request = build_request_term(&declarations, change, 10, 11);
        super::super::CompilerPackage {
            core_manifest: CoreManifest::canonical_v1(),
            subject: CompilerSubject {
                lineage: CompilerLineage::Genesis,
                nominal_declarations: declarations,
                interface: CompilerInterface {
                    compile,
                    admit_propose: admit,
                },
                program: vec![
                    Definition {
                        id: compile,
                        arguments: vec![KSort::Term],
                        result: KSort::Term,
                        body: KExpr::Var(0),
                    },
                    Definition {
                        id: admit,
                        arguments: vec![KSort::Term],
                        result: KSort::Term,
                        body: KExpr::Var(0),
                    },
                ],
                build_request,
            },
            evidence: CompilerEvidence::Genesis,
        }
    }

    fn genesis_request<'a>(
        package: &'a super::super::CompilerPackage,
        bytes: &'a [u8],
        owner_anchor: OwnerAnchorInput<'a>,
    ) -> GenesisAuthorizationRequest<'a> {
        GenesisAuthorizationRequest {
            owner_anchor,
            build_request: &package.subject.build_request,
            evidence: &package.evidence,
            compile_fuel_limit: 10,
            admission_fuel_limit: 11,
            final_identity: FinalPackageIdentityInput {
                package_hash: compiler_package_hash(bytes),
                exact_package_bytes: bytes,
            },
        }
    }

    #[test]
    fn stage_and_code_tags_are_the_frozen_wire_values() {
        assert_eq!(AuthorizationStage::CoreManifest as u8, 0x40);
        assert_eq!(AuthorizationStage::FinalAuthorization as u8, 0x48);
        assert_eq!(AuthorizationCode::ManifestMismatch as u8, 0x60);
        assert_eq!(AuthorizationCode::FinalIdentityMismatch as u8, 0x87);
    }

    #[test]
    fn opaque_authority_premises_bind_exact_bytes() {
        let bytes = b"externally selected bytes";
        let observation = OwnerAnchorObservation {
            exact_selected_bytes: bytes,
            selected_byte_length: bytes.len() as u64,
            selected_package_hash: compiler_package_hash(bytes),
        };
        let witness = OwnerAnchorWitness::from_external_selection(observation);
        assert_eq!(witness.observation(), observation);
        let accepted = AcceptedExact::from_outer_admission(bytes);
        assert_eq!(accepted.exact_bytes, bytes);
    }

    #[test]
    fn genesis_skips_both_entrypoint_evaluations() {
        let package = genesis_package();
        let bytes = encode(&package).expect("genesis encodes");
        let observation = OwnerAnchorObservation {
            exact_selected_bytes: &bytes,
            selected_byte_length: bytes.len() as u64,
            selected_package_hash: compiler_package_hash(&bytes),
        };
        let witness = OwnerAnchorWitness::from_external_selection(observation);
        assert_eq!(
            authorize_genesis(
                &bytes,
                genesis_request(&package, &bytes, OwnerAnchorInput::Supplied(witness)),
            )
            .expect("genesis checks"),
            AuthorizationVerdict::Authorized(bytes)
        );
    }

    #[test]
    fn missing_anchor_has_exact_precedence_after_common_checks() {
        let package = genesis_package();
        let bytes = encode(&package).expect("genesis encodes");
        assert_eq!(
            authorize_genesis(
                &bytes,
                genesis_request(&package, &bytes, OwnerAnchorInput::Missing),
            )
            .expect("genesis checks"),
            deny(
                AuthorizationStage::GenesisAnchor,
                AuthorizationCode::MissingAnchor,
            )
        );
    }

    #[test]
    fn replay_resource_exhaustion_is_not_an_evaluation_fault() {
        assert!(replay_error_is_resource_exhausted(
            &EvalError::RecursionLimit
        ));
        assert!(replay_error_is_resource_exhausted(
            &EvalError::ResourceExhausted
        ));
        assert!(replay_error_is_resource_exhausted(&EvalError::Static(
            StaticError::RecursionLimit,
        )));
        assert!(replay_error_is_resource_exhausted(&EvalError::Static(
            StaticError::ResourceExhausted,
        )));
        assert!(replay_error_is_resource_exhausted(&EvalError::Physical(
            PhysicalError::ResourceExhausted,
        )));
        assert!(!replay_error_is_resource_exhausted(
            &EvalError::ByteLengthOverflow
        ));
        assert!(!replay_error_is_resource_exhausted(&EvalError::OutOfFuel));
    }

    #[test]
    fn successor_authorization_propagates_the_concat_byte_ceiling() {
        let compile_body = KExpr::Let {
            value: boxed_expression(doubling_bytes(24)),
            body: boxed_expression(KExpr::Var(1)),
        };

        assert_eq!(
            successor_authorization_with_compile_body(compile_body, 1_000),
            Err(AuthorizationCheckError::ResourceExhausted),
        );
    }

    #[test]
    fn successor_authorization_propagates_the_runtime_recursion_ceiling() {
        let compile_body = KExpr::Let {
            value: boxed_expression(KExpr::Call {
                definition_id: id(1),
                arguments: vec![KExpr::Var(0)],
            }),
            body: boxed_expression(KExpr::Var(0)),
        };

        assert_eq!(
            successor_authorization_with_compile_body(compile_body, 2_000_000),
            Err(AuthorizationCheckError::ResourceExhausted),
        );
    }

    #[test]
    fn exact_predecessor_replay_authorizes_a_complete_successor() {
        const COMPILE_FUEL: u64 = 10_000;
        const ADMISSION_FUEL: u64 = 10_000;

        let template = successor_package(
            Hash32(PREDECESSOR_HASH_MARKER),
            Id32(PREDECESSOR_REVISION_MARKER),
            COMPILE_FUEL,
            ADMISSION_FUEL,
        );
        let template_bytes = encode(&template).expect("successor template encodes");
        let template = decode(&template_bytes).expect("successor template decodes");
        let compile_body = result_expression(0x14, dynamic_subject_bytes(template.exact_subject()));
        let admission_body = result_expression(0x17, admission_subject_payload());
        let predecessor = predecessor_package(compile_body, admission_body);
        let predecessor_bytes = encode(&predecessor).expect("predecessor encodes");
        let predecessor_decoded = decode(&predecessor_bytes).expect("predecessor decodes");
        let predecessor_hash = compiler_package_hash(&predecessor_bytes);
        let predecessor_revision = compiler_revision_id(predecessor_decoded.exact_subject());

        let mut candidate = successor_package(
            predecessor_hash,
            predecessor_revision,
            COMPILE_FUEL,
            ADMISSION_FUEL,
        );
        let candidate_without_receipts =
            encode(&candidate).expect("candidate subject encodes before receipts");
        let candidate_decoded =
            decode(&candidate_without_receipts).expect("candidate subject decodes");
        let evaluator = Evaluator::new(&predecessor.subject.program)
            .expect("accepted predecessor program checks");
        let compile_argument = KValue::Term(
            candidate
                .subject
                .build_request
                .try_clone_resource()
                .expect("compile argument clones"),
        );
        let compile_evaluation = evaluator
            .replay_entrypoint(
                predecessor.subject.interface.compile,
                &[compile_argument],
                COMPILE_FUEL,
            )
            .expect("predecessor compiles the candidate subject");
        assert_eq!(
            result_bytes(&compile_evaluation.value, 0x14)
                .expect("compile result shape checks")
                .expect("compile result is a subject"),
            candidate_decoded.exact_subject(),
        );
        let compile_observations = compile_evaluation
            .observations
            .try_to_term()
            .expect("compile observations canonicalize");
        let compile_receipt = evaluator
            .build_receipt(
                predecessor.subject.interface.compile,
                &[KValue::Term(
                    candidate
                        .subject
                        .build_request
                        .try_clone_resource()
                        .expect("receipt argument clones"),
                )],
                COMPILE_FUEL,
            )
            .expect("compile receipt builds");
        let admission_argument = admission_request_term(
            &candidate.subject.build_request,
            candidate_decoded.exact_subject(),
            &compile_observations,
        )
        .expect("admission argument builds");
        let admission_receipt = evaluator
            .build_receipt(
                predecessor.subject.interface.admit_propose,
                &[KValue::Term(admission_argument)],
                ADMISSION_FUEL,
            )
            .expect("admission receipt builds");
        candidate.evidence = CompilerEvidence::Successor {
            compile_receipt,
            admission_receipt,
        };

        let candidate_bytes = encode(&candidate).expect("complete successor encodes");
        let final_candidate = decode(&candidate_bytes).expect("complete successor decodes");
        assert_eq!(
            final_candidate.exact_subject(),
            candidate_decoded.exact_subject(),
            "receipt attachment cannot alter the compiled subject",
        );
        let acceptance = AcceptedExact::from_outer_admission(&predecessor_bytes);
        let request = SuccessorAuthorizationRequest {
            predecessor: PredecessorInput::Accepted {
                exact_bytes: &predecessor_bytes,
                acceptance,
                offered_bytes: &predecessor_bytes,
            },
            build_request: &candidate.subject.build_request,
            evidence: &candidate.evidence,
            final_identity: FinalPackageIdentityInput {
                package_hash: compiler_package_hash(&candidate_bytes),
                exact_package_bytes: &candidate_bytes,
            },
        };

        assert_eq!(
            authorize_successor(&candidate_bytes, request)
                .expect("complete successor authorization checks"),
            AuthorizationVerdict::Authorized(candidate_bytes),
        );
    }

    #[test]
    fn successor_route_does_not_let_candidate_lineage_select_genesis() {
        let package = genesis_package();
        let bytes = encode(&package).expect("genesis encodes");
        let request = SuccessorAuthorizationRequest {
            predecessor: PredecessorInput::Absent {
                offered_bytes: b"absent",
            },
            build_request: &package.subject.build_request,
            evidence: &package.evidence,
            final_identity: FinalPackageIdentityInput {
                package_hash: compiler_package_hash(&bytes),
                exact_package_bytes: &bytes,
            },
        };
        assert_eq!(
            authorize_successor(&bytes, request).expect("successor route checks"),
            deny(
                AuthorizationStage::ExactPredecessor,
                AuthorizationCode::SuccessorWrongLineage,
            )
        );
    }
}
