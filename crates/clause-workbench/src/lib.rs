//! One long-lived, binary-framed agent workbench whose operation meaning is
//! owned by an accepted CLCP03 package.
//!
//! The host reads and writes exact neutral Terms, invokes one opaque package
//! `DefId`, and atomically retains the state Term returned by that definition.
//! It contains no Clause source parser, checker, query engine, diagnostic
//! table, or operation switch.

#![forbid(unsafe_code)]

mod carrier;
mod source_session;

use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};

use clause_substrate::compiler_package_v3::{
    AuthorizationCheckError, AuthorizationVerdict, CompilerPackage, DecodeFailure, EncodeError,
    FallibleBox, FinalPackageIdentityInput, Hash32, Id32, KValue, OwnerAnchorInput,
    OwnerAnchorObservation, OwnerAnchorWitness, Term, authorize_genesis, compiler_package_hash,
    decode, decode_canonical_term, encode_canonical_term,
};
use clause_substrate::evaluator::{EvalError, Evaluator, StaticError};

use carrier::{CarrierActionV1, WorkbenchCarrier};
pub use carrier::{WorkbenchCarrierError, WorkbenchCarrierSnapshot};
pub use source_session::{
    ResidentSourceAdmissionV1, ResidentSourceCandidateV1, ResidentSourceGenerationV1,
    ResidentSourceWorkbenchErrorV1, ResidentSourceWorkbenchV1,
};

pub const BASE_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/workbench-v1/program.clause"
));
pub const CHANGED_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/workbench-v1/program-changed.clause"
));
pub const INVALID_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/workbench-v1/program-invalid.clause"
));
const EXACT_WORKBENCH_PACKAGE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/workbench-v1/workbench.clcp03"
));

const MAX_FRAME_BYTES: usize = 1024 * 1024;
const EXACT_EQ: &[u8] = b"clause/core/bytes-equal/v1";
const RESPONSE_KIND: &[u8] = b"clause/workbench-response/v1";
const STATE_KIND: &[u8] = b"clause/workbench-state/v1";
const TRANSITION_KIND: &[u8] = b"clause/workbench-transition/v1";
pub const WORKBENCH_ENTRYPOINT: Id32 = Id32([3; 32]);
const BASE_STATE: &[u8] = b"base-1";

#[derive(Debug)]
pub enum WorkbenchError {
    Encode(EncodeError),
    Decode(DecodeFailure),
    Static(StaticError),
    Evaluate(EvalError),
    Authorization(AuthorizationCheckError),
    Carrier(WorkbenchCarrierError),
    Unauthorized,
    MalformedPackageResult,
    FrameTooLarge(usize),
    Io(io::Error),
}

impl fmt::Display for WorkbenchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "CLCP03 encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "CLCP03 decode failed: {error}"),
            Self::Static(error) => {
                write!(formatter, "accepted definition table is invalid: {error}")
            }
            Self::Evaluate(error) => {
                write!(formatter, "accepted definition execution failed: {error}")
            }
            Self::Authorization(error) => write!(formatter, "CLCP03 authorization failed: {error}"),
            Self::Carrier(error) => write!(formatter, "runtime carrier failed: {error}"),
            Self::Unauthorized => formatter.write_str("CLCP03 rejected the workbench package"),
            Self::MalformedPackageResult => formatter
                .write_str("accepted workbench definition returned a malformed transaction"),
            Self::FrameTooLarge(length) => {
                write!(formatter, "workbench frame is too large: {length}")
            }
            Self::Io(error) => write!(formatter, "workbench transport failed: {error}"),
        }
    }
}

impl Error for WorkbenchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::Static(error) => Some(error),
            Self::Evaluate(error) => Some(error),
            Self::Authorization(error) => Some(error),
            Self::Carrier(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EncodeError> for WorkbenchError {
    fn from(value: EncodeError) -> Self {
        Self::Encode(value)
    }
}

impl From<DecodeFailure> for WorkbenchError {
    fn from(value: DecodeFailure) -> Self {
        Self::Decode(value)
    }
}

impl From<StaticError> for WorkbenchError {
    fn from(value: StaticError) -> Self {
        Self::Static(value)
    }
}

impl From<EvalError> for WorkbenchError {
    fn from(value: EvalError) -> Self {
        Self::Evaluate(value)
    }
}

impl From<AuthorizationCheckError> for WorkbenchError {
    fn from(value: AuthorizationCheckError) -> Self {
        Self::Authorization(value)
    }
}

impl From<WorkbenchCarrierError> for WorkbenchError {
    fn from(value: WorkbenchCarrierError) -> Self {
        Self::Carrier(value)
    }
}

impl From<io::Error> for WorkbenchError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// The host-owned mechanics for one long-lived accepted package. `state` is
/// opaque: only the package can interpret or replace it.
pub struct WorkbenchService {
    package: CompilerPackage,
    package_hash: Hash32,
    state: Option<Term>,
    carrier: WorkbenchCarrier,
    requests: u64,
}

impl WorkbenchService {
    pub fn open() -> Result<Self, WorkbenchError> {
        let package = decode(EXACT_WORKBENCH_PACKAGE)?.into_package();
        let bytes = EXACT_WORKBENCH_PACKAGE;
        let package_hash = compiler_package_hash(bytes);
        let observation = OwnerAnchorObservation {
            exact_selected_bytes: bytes,
            selected_byte_length: u64::try_from(bytes.len())
                .map_err(|_| WorkbenchError::FrameTooLarge(bytes.len()))?,
            selected_package_hash: package_hash,
        };
        let witness = OwnerAnchorWitness::from_external_selection(observation);
        let verdict = authorize_genesis(
            bytes,
            clause_substrate::compiler_package_v3::GenesisAuthorizationRequest {
                owner_anchor: OwnerAnchorInput::Supplied(witness),
                build_request: &package.subject.build_request,
                evidence: &package.evidence,
                compile_fuel_limit: 64,
                admission_fuel_limit: 64,
                final_identity: FinalPackageIdentityInput {
                    package_hash,
                    exact_package_bytes: bytes,
                },
            },
        )?;
        if !matches!(verdict, AuthorizationVerdict::Authorized(authorized) if authorized == bytes) {
            return Err(WorkbenchError::Unauthorized);
        }
        Evaluator::new(&package.subject.program)?;
        let carrier = WorkbenchCarrier::open()?;
        Ok(Self {
            package,
            package_hash,
            state: Some(state_term(BASE_STATE)),
            carrier,
            requests: 0,
        })
    }

    #[must_use]
    pub const fn package_hash(&self) -> Hash32 {
        self.package_hash
    }

    #[must_use]
    pub const fn request_count(&self) -> u64 {
        self.requests
    }

    pub fn exact_state(&self) -> Result<Vec<u8>, WorkbenchError> {
        encode_canonical_term(
            self.state
                .as_ref()
                .ok_or(WorkbenchError::MalformedPackageResult)?,
        )
        .map_err(Into::into)
    }

    pub fn carrier_snapshot(&self) -> Result<WorkbenchCarrierSnapshot, WorkbenchError> {
        self.carrier.snapshot().map_err(Into::into)
    }

    /// Execute one exact request through the accepted package and atomically
    /// install the opaque next-state Term returned by it.
    pub fn process_term(&mut self, request: Term) -> Result<Vec<u8>, WorkbenchError> {
        let old_state = self
            .state
            .take()
            .ok_or(WorkbenchError::MalformedPackageResult)?;
        let mut arguments = vec![KValue::Term(request), KValue::Term(old_state)];
        let evaluation = Evaluator::new(&self.package.subject.program)?.invoke_entrypoint(
            WORKBENCH_ENTRYPOINT,
            &arguments,
            16_384,
        );
        let evaluation = match evaluation {
            Ok(value) => value,
            Err(error) => {
                let state = arguments
                    .pop()
                    .ok_or(WorkbenchError::MalformedPackageResult)?;
                let KValue::Term(state) = state else {
                    return Err(WorkbenchError::MalformedPackageResult);
                };
                self.state = Some(state);
                return Err(error.into());
            }
        };
        let KValue::Term(result) = evaluation.value else {
            let state = arguments
                .pop()
                .ok_or(WorkbenchError::MalformedPackageResult)?;
            let KValue::Term(state) = state else {
                return Err(WorkbenchError::MalformedPackageResult);
            };
            self.state = Some(state);
            return Err(WorkbenchError::MalformedPackageResult);
        };
        let encoded = match encode_canonical_term(&result) {
            Ok(encoded) => encoded,
            Err(error) => {
                let state = arguments
                    .pop()
                    .ok_or(WorkbenchError::MalformedPackageResult)?;
                let KValue::Term(state) = state else {
                    return Err(WorkbenchError::MalformedPackageResult);
                };
                self.state = Some(state);
                return Err(error.into());
            }
        };
        let Term::Triple(_, next_state, transition) = result else {
            let state = arguments
                .pop()
                .ok_or(WorkbenchError::MalformedPackageResult)?;
            let KValue::Term(state) = state else {
                return Err(WorkbenchError::MalformedPackageResult);
            };
            self.state = Some(state);
            return Err(WorkbenchError::MalformedPackageResult);
        };
        let old_state = arguments
            .pop()
            .ok_or(WorkbenchError::MalformedPackageResult)?;
        let KValue::Term(old_state) = old_state else {
            return Err(WorkbenchError::MalformedPackageResult);
        };
        let action = match carrier_action(&transition) {
            Ok(action) => action,
            Err(error) => {
                self.state = Some(old_state);
                return Err(error);
            }
        };
        if let Err(error) = self.carrier.apply(action) {
            self.state = Some(old_state);
            return Err(error.into());
        }
        self.state = Some(next_state.into_inner());
        self.requests = self
            .requests
            .checked_add(1)
            .ok_or(WorkbenchError::MalformedPackageResult)?;
        Ok(encoded)
    }

    pub fn process_frame(&mut self, exact_request: &[u8]) -> Result<Vec<u8>, WorkbenchError> {
        self.process_term(decode_canonical_term(exact_request)?)
    }

    pub fn serve<R: Read, W: Write>(
        &mut self,
        mut input: R,
        mut output: W,
    ) -> Result<(), WorkbenchError> {
        while let Some(frame) = read_frame(&mut input)? {
            let response = self.process_frame(&frame)?;
            write_frame(&mut output, &response)?;
            output.flush()?;
        }
        Ok(())
    }
}

fn carrier_action(transition: &Term) -> Result<CarrierActionV1, WorkbenchError> {
    let Term::Atom {
        kind,
        canonical_payload,
        equality_contract,
    } = transition
    else {
        return Err(WorkbenchError::MalformedPackageResult);
    };
    if kind != TRANSITION_KIND || equality_contract != EXACT_EQ {
        return Err(WorkbenchError::MalformedPackageResult);
    }
    match canonical_payload.as_slice() {
        b"unchanged" => Ok(CarrierActionV1::Unchanged),
        b"candidate" => Ok(CarrierActionV1::Candidate),
        b"admission" => Ok(CarrierActionV1::Admission),
        b"hot-reload" => Ok(CarrierActionV1::HotReload),
        _ => Err(WorkbenchError::MalformedPackageResult),
    }
}

pub fn request_term(operation: &[u8], base: &[u8], source: &[u8]) -> Term {
    triple(
        atom(operation, b""),
        atom(b"clause/workbench-base/v1", base),
        atom(b"clause/workbench-source/v1", source),
    )
}

pub fn encode_request(
    operation: &[u8],
    base: &[u8],
    source: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    encode_canonical_term(&request_term(operation, base, source))
}

pub fn response_payload(exact_response: &[u8]) -> Result<Vec<u8>, WorkbenchError> {
    let response = decode_canonical_term(exact_response)?;
    let Term::Triple(response, _, _) = response else {
        return Err(WorkbenchError::MalformedPackageResult);
    };
    let Term::Atom {
        kind,
        canonical_payload,
        equality_contract,
    } = response.into_inner()
    else {
        return Err(WorkbenchError::MalformedPackageResult);
    };
    if kind != RESPONSE_KIND || equality_contract != EXACT_EQ {
        return Err(WorkbenchError::MalformedPackageResult);
    }
    Ok(canonical_payload)
}

pub fn framed(exact_terms: &[Vec<u8>]) -> Result<Vec<u8>, WorkbenchError> {
    let mut bytes = Vec::new();
    for term in exact_terms {
        write_frame(&mut bytes, term)?;
    }
    Ok(bytes)
}

pub fn split_frames(mut bytes: &[u8]) -> Result<Vec<Vec<u8>>, WorkbenchError> {
    let mut frames = Vec::new();
    while let Some(frame) = read_frame(&mut bytes)? {
        frames.push(frame);
    }
    Ok(frames)
}

fn read_frame(input: &mut impl Read) -> Result<Option<Vec<u8>>, WorkbenchError> {
    let mut length = [0_u8; 4];
    let mut read = 0;
    while read < length.len() {
        let count = input.read(&mut length[read..])?;
        if count == 0 {
            if read == 0 {
                return Ok(None);
            }
            return Err(
                io::Error::new(io::ErrorKind::UnexpectedEof, "partial frame length").into(),
            );
        }
        read += count;
    }
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(WorkbenchError::FrameTooLarge(length));
    }
    let mut frame = vec![0_u8; length];
    input.read_exact(&mut frame)?;
    Ok(Some(frame))
}

fn write_frame(output: &mut impl Write, frame: &[u8]) -> Result<(), WorkbenchError> {
    if frame.len() > MAX_FRAME_BYTES {
        return Err(WorkbenchError::FrameTooLarge(frame.len()));
    }
    let length =
        u32::try_from(frame.len()).map_err(|_| WorkbenchError::FrameTooLarge(frame.len()))?;
    output.write_all(&length.to_be_bytes())?;
    output.write_all(frame)?;
    Ok(())
}

fn boxed<T>(value: T) -> FallibleBox<T> {
    FallibleBox::try_new(value).expect("bounded protocol Term allocation")
}

fn atom(kind: &[u8], payload: &[u8]) -> Term {
    Term::Atom {
        kind: kind.to_vec(),
        canonical_payload: payload.to_vec(),
        equality_contract: EXACT_EQ.to_vec(),
    }
}

fn state_term(token: &[u8]) -> Term {
    atom(STATE_KIND, token)
}

fn triple(first: Term, second: Term, third: Term) -> Term {
    Term::Triple(boxed(first), boxed(second), boxed(third))
}
