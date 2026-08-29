use std::fmt;

/// One exact 32-octet identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id32(pub [u8; 32]);

impl Id32 {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// One exact 32-octet domain-separated hash.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A half-open source span. Bounds are checked against an exact artifact by
/// artifact storage rather than by the context-free wire decoder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub source_artifact_id: Id32,
    pub start: u64,
    pub end: u64,
}

/// The complete fixed neutral Term carrier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Term {
    Atom {
        kind: Vec<u8>,
        canonical_payload: Vec<u8>,
        equality_contract: Vec<u8>,
    },
    Triple(Box<Term>, Box<Term>, Box<Term>),
}

/// The two fixed evaluator sorts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KSort {
    Bytes,
    Term,
}

/// The twelve fixed construct-blind evaluator forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KExpr {
    BytesLiteral(Vec<u8>),
    TermLiteral(Term),
    Var(u32),
    MakeAtom {
        kind: Box<KExpr>,
        payload: Box<KExpr>,
        equality: Box<KExpr>,
    },
    MakeTriple {
        first: Box<KExpr>,
        second: Box<KExpr>,
        third: Box<KExpr>,
    },
    Let {
        value: Box<KExpr>,
        body: Box<KExpr>,
    },
    CaseTerm {
        scrutinee: Box<KExpr>,
        atom_body: Box<KExpr>,
        triple_body: Box<KExpr>,
    },
    CaseBytes {
        scrutinee: Box<KExpr>,
        empty_body: Box<KExpr>,
        cons_body: Box<KExpr>,
    },
    ConcatBytes(Vec<KExpr>),
    CaseBytesEqual {
        left: Box<KExpr>,
        right: Box<KExpr>,
        equal_body: Box<KExpr>,
        unequal_body: Box<KExpr>,
    },
    Call {
        definition_id: Id32,
        arguments: Vec<KExpr>,
    },
    Request {
        physical_operation_id: Id32,
        arguments: Vec<KExpr>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedSignature {
    pub tag: u8,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleSignature {
    pub tag: u8,
    pub premise_policy: u8,
    pub clause: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalOperation {
    pub operation_id: Id32,
    pub arguments: Vec<KSort>,
    pub result: KSort,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalProfile {
    pub profile_version: u8,
    pub observation_policy: u8,
    pub operations: Vec<PhysicalOperation>,
}

/// Frame 01. Text fields are inert exact bytes, never executable rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreManifest {
    pub manifest_version: u8,
    pub frame_tags: Vec<u8>,
    pub term_tags: Vec<u8>,
    pub sort_tags: Vec<u8>,
    pub expression_forms: Vec<NamedSignature>,
    pub abi_forms: Vec<NamedSignature>,
    pub premise_policy_tags: Vec<u8>,
    pub lineage_tags: Vec<u8>,
    pub nominal_declaration_tags: Vec<u8>,
    pub compiler_evidence_tags: Vec<u8>,
    pub value_tags: Vec<u8>,
    pub eval_outcome_tags: Vec<u8>,
    pub decode_verdict_tags: Vec<u8>,
    pub decode_code_tags: Vec<u8>,
    pub authorization_stage_tags: Vec<u8>,
    pub authorization_code_tags: Vec<u8>,
    pub static_rules: Vec<RuleSignature>,
    pub evaluation_rules: Vec<RuleSignature>,
    pub certificate_format_version: u8,
    pub certificate_signature: Vec<u8>,
    pub contract_clauses: Vec<Vec<u8>>,
    pub physical_profile: PhysicalProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerLineage {
    Genesis,
    Successor {
        predecessor_locator: Hash32,
        change_occurrence_id: Id32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NominalWireRef {
    pub domain: Id32,
    pub id: Id32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NominalDeclaration {
    Seed {
        domain: Id32,
        id: Id32,
    },
    RetainedSeed {
        domain: Id32,
        id: Id32,
        predecessor_revision_id: Id32,
    },
    Allocated {
        domain: Id32,
        id: Id32,
        change_input: NominalWireRef,
        producer_input: NominalWireRef,
        local_slot: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerInterface {
    pub compile: Id32,
    pub admit_propose: Id32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub id: Id32,
    pub arguments: Vec<KSort>,
    pub result: KSort,
    pub body: KExpr,
}

/// Frame 02. Only `interface` and `program` are executable compiler data;
/// Rust still treats both through fixed generic lookup and KExpr mechanics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerSubject {
    pub lineage: CompilerLineage,
    pub nominal_declarations: Vec<NominalDeclaration>,
    pub interface: CompilerInterface,
    pub program: Vec<Definition>,
    pub build_request: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KValue {
    Bytes(Vec<u8>),
    Term(Term),
}

impl KValue {
    #[must_use]
    pub const fn sort(&self) -> KSort {
        match self {
            Self::Bytes(_) => KSort::Bytes,
            Self::Term(_) => KSort::Term,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalOutcome {
    Returned {
        value: KValue,
        remaining_fuel: u64,
        observations: Term,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalStatement {
    pub exact_accepted_predecessor: Vec<u8>,
    pub core_contract_id: Hash32,
    pub physical_profile_id: Hash32,
    pub entrypoint: Id32,
    pub arguments: Vec<KValue>,
    pub fuel_limit: u64,
    pub expected: EvalOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalJudgment {
    pub expression: KExpr,
    pub environment: Vec<KValue>,
    pub fuel_before: u64,
    pub observations_before: Term,
    pub value: KValue,
    pub fuel_after: u64,
    pub observations_after: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalNode {
    pub rule_tag: u8,
    pub premises: Vec<u32>,
    pub conclusion: EvalJudgment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalCertificate {
    pub format_version: u8,
    pub statement: EvalStatement,
    pub nodes: Vec<EvalNode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerEvidence {
    Genesis,
    Successor {
        compile_certificate: Box<EvalCertificate>,
        admission_certificate: Box<EvalCertificate>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerPackage {
    pub core_manifest: CoreManifest,
    pub subject: CompilerSubject,
    pub evidence: CompilerEvidence,
}

/// A strictly decoded candidate. Exact input retention is deliberately
/// inseparable from the decoded fields, but confers no compiler authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedCompilerPackage {
    exact_input: Box<[u8]>,
    exact_core_manifest: Box<[u8]>,
    exact_subject: Box<[u8]>,
    exact_evidence: Box<[u8]>,
    package: CompilerPackage,
}

impl DecodedCompilerPackage {
    pub(crate) fn new(
        exact_input: Box<[u8]>,
        exact_core_manifest: Box<[u8]>,
        exact_subject: Box<[u8]>,
        exact_evidence: Box<[u8]>,
        package: CompilerPackage,
    ) -> Self {
        Self {
            exact_input,
            exact_core_manifest,
            exact_subject,
            exact_evidence,
            package,
        }
    }

    #[must_use]
    pub fn exact_input(&self) -> &[u8] {
        &self.exact_input
    }

    #[must_use]
    pub fn exact_core_manifest(&self) -> &[u8] {
        &self.exact_core_manifest
    }

    #[must_use]
    pub fn exact_subject(&self) -> &[u8] {
        &self.exact_subject
    }

    #[must_use]
    pub fn exact_evidence(&self) -> &[u8] {
        &self.exact_evidence
    }

    #[must_use]
    pub const fn package(&self) -> &CompilerPackage {
        &self.package
    }
}

/// The ten canonical strict-decode rejection codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DecodeCode {
    WrongMagic = 0x00,
    UnknownVersion = 0x01,
    FrameTagOrderOrCount = 0x02,
    Truncated = 0x03,
    LengthOrCountOverflow = 0x04,
    InvalidFixedWidth = 0x05,
    UnknownSumTag = 0x06,
    BoundedValueUnderConsumed = 0x07,
    BoundedValueOverConsumed = 0x08,
    TrailingBytes = 0x09,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeRejection {
    pub code: DecodeCode,
    pub offset: u64,
}

/// Resource limits are not converted into a different canonical verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeFailure {
    Rejected(DecodeRejection),
    ResourceExhausted,
}

impl fmt::Display for DecodeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(rejection) => write!(
                formatter,
                "CLCP-v2 decode rejected with code {:#04x} at byte {}",
                rejection.code as u8, rejection.offset
            ),
            Self::ResourceExhausted => {
                formatter.write_str("CLCP-v2 decode exhausted physical resources")
            }
        }
    }
}

impl std::error::Error for DecodeFailure {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthExceedsU32 { field: &'static str, length: usize },
    InvalidClosedTag { field: &'static str, tag: u8 },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthExceedsU32 { field, length } => {
                write!(formatter, "{field} length {length} exceeds u32")
            }
            Self::InvalidClosedTag { field, tag } => {
                write!(formatter, "{field} has invalid closed tag {tag:#04x}")
            }
        }
    }
}

impl std::error::Error for EncodeError {}
