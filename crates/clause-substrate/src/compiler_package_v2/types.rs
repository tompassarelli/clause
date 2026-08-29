use std::fmt;
use std::ops::Range;

/// CLCP-v2 admits the exact 279,620-byte Compiler0 package while bounding all
/// retained wire data and collections well below the U32 format ceiling.
pub(crate) const MAX_WIRE_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_WIRE_ITEMS: usize = 262_144;

/// The exact Compiler0 package measures 51 Term levels, 266 KExpr levels, 906
/// Term nodes, and 11,637 KExpr nodes. Separate limits preserve that real
/// shape without exposing evaluator stack depth as a wire-format constraint.
pub(crate) const MAX_TERM_DEPTH: usize = 64;
pub(crate) const MAX_EXPRESSION_DEPTH: usize = 512;
pub(crate) const MAX_TERM_NODES: usize = 262_144;
pub(crate) const MAX_EXPRESSION_NODES: usize = 262_144;

/// Runtime recursion is represented on a fallibly allocated machine stack.
pub(crate) const MAX_EVALUATION_FRAMES: usize = 262_144;
pub(crate) const MAX_CERTIFICATE_NODES: usize = 1_000_000;

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

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceLimit;

pub(crate) fn try_copy_bytes(value: &[u8]) -> Result<Vec<u8>, ResourceLimit> {
    if value.len() > MAX_WIRE_BYTES {
        return Err(ResourceLimit);
    }
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| ResourceLimit)?;
    copied.extend_from_slice(value);
    Ok(copied)
}

impl Term {
    pub(crate) fn validate_resource_bounds(&self) -> Result<(), ResourceLimit> {
        let mut stack = Vec::new();
        stack.try_reserve(1).map_err(|_| ResourceLimit)?;
        stack.push((self, 1_usize));
        let mut nodes = 0_usize;
        let mut bytes = 0_usize;
        while let Some((term, depth)) = stack.pop() {
            if depth > MAX_TERM_DEPTH {
                return Err(ResourceLimit);
            }
            nodes = nodes.checked_add(1).ok_or(ResourceLimit)?;
            if nodes > MAX_TERM_NODES {
                return Err(ResourceLimit);
            }
            match term {
                Self::Atom {
                    kind,
                    canonical_payload,
                    equality_contract,
                } => {
                    bytes = bytes
                        .checked_add(kind.len())
                        .and_then(|value| value.checked_add(canonical_payload.len()))
                        .and_then(|value| value.checked_add(equality_contract.len()))
                        .ok_or(ResourceLimit)?;
                    if bytes > MAX_WIRE_BYTES {
                        return Err(ResourceLimit);
                    }
                }
                Self::Triple(first, second, third) => {
                    stack.try_reserve(3).map_err(|_| ResourceLimit)?;
                    stack.push((third, depth + 1));
                    stack.push((second, depth + 1));
                    stack.push((first, depth + 1));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn try_clone_resource(&self) -> Result<Self, ResourceLimit> {
        self.validate_resource_bounds()?;
        clone_term(self, 1)
    }

    pub(crate) fn try_triple(
        first: Self,
        second: Self,
        third: Self,
    ) -> Result<Self, ResourceLimit> {
        let value = Self::Triple(Box::new(first), Box::new(second), Box::new(third));
        value.validate_resource_bounds()?;
        Ok(value)
    }
}

fn clone_term(term: &Term, depth: usize) -> Result<Term, ResourceLimit> {
    enum Task<'a> {
        Read(&'a Term, usize),
        Triple,
    }

    let mut tasks = Vec::new();
    tasks.try_reserve(1).map_err(|_| ResourceLimit)?;
    tasks.push(Task::Read(term, depth));
    let mut results = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Read(term, depth) => {
                if depth > MAX_TERM_DEPTH {
                    return Err(ResourceLimit);
                }
                match term {
                    Term::Atom {
                        kind,
                        canonical_payload,
                        equality_contract,
                    } => push_term(
                        &mut results,
                        Term::Atom {
                            kind: try_copy_bytes(kind)?,
                            canonical_payload: try_copy_bytes(canonical_payload)?,
                            equality_contract: try_copy_bytes(equality_contract)?,
                        },
                    )?,
                    Term::Triple(first, second, third) => {
                        tasks.try_reserve(4).map_err(|_| ResourceLimit)?;
                        tasks.push(Task::Triple);
                        tasks.push(Task::Read(third, depth + 1));
                        tasks.push(Task::Read(second, depth + 1));
                        tasks.push(Task::Read(first, depth + 1));
                    }
                }
            }
            Task::Triple => {
                let third = results.pop().ok_or(ResourceLimit)?;
                let second = results.pop().ok_or(ResourceLimit)?;
                let first = results.pop().ok_or(ResourceLimit)?;
                push_term(
                    &mut results,
                    Term::Triple(Box::new(first), Box::new(second), Box::new(third)),
                )?;
            }
        }
    }
    if results.len() != 1 {
        return Err(ResourceLimit);
    }
    let cloned = results.pop().ok_or(ResourceLimit)?;
    Ok(cloned)
}

fn push_term(results: &mut Vec<Term>, term: Term) -> Result<(), ResourceLimit> {
    results.try_reserve(1).map_err(|_| ResourceLimit)?;
    results.push(term);
    Ok(())
}

impl KExpr {
    pub(crate) fn validate_resource_bounds(&self) -> Result<(), ResourceLimit> {
        let mut stack = Vec::new();
        stack.try_reserve(1).map_err(|_| ResourceLimit)?;
        stack.push((self, 1_usize));
        let mut nodes = 0_usize;
        while let Some((expression, depth)) = stack.pop() {
            if depth > MAX_EXPRESSION_DEPTH {
                return Err(ResourceLimit);
            }
            nodes = nodes.checked_add(1).ok_or(ResourceLimit)?;
            if nodes > MAX_EXPRESSION_NODES {
                return Err(ResourceLimit);
            }
            let next = depth + 1;
            match expression {
                Self::BytesLiteral(bytes) => {
                    if bytes.len() > MAX_WIRE_BYTES {
                        return Err(ResourceLimit);
                    }
                }
                Self::TermLiteral(term) => term.validate_resource_bounds()?,
                Self::Var(_) => {}
                Self::MakeAtom {
                    kind,
                    payload,
                    equality,
                }
                | Self::MakeTriple {
                    first: kind,
                    second: payload,
                    third: equality,
                }
                | Self::CaseTerm {
                    scrutinee: kind,
                    atom_body: payload,
                    triple_body: equality,
                }
                | Self::CaseBytes {
                    scrutinee: kind,
                    empty_body: payload,
                    cons_body: equality,
                } => {
                    stack.try_reserve(3).map_err(|_| ResourceLimit)?;
                    stack.push((equality, next));
                    stack.push((payload, next));
                    stack.push((kind, next));
                }
                Self::Let { value, body } => {
                    stack.try_reserve(2).map_err(|_| ResourceLimit)?;
                    stack.push((body, next));
                    stack.push((value, next));
                }
                Self::ConcatBytes(parts) => {
                    stack.try_reserve(parts.len()).map_err(|_| ResourceLimit)?;
                    stack.extend(parts.iter().rev().map(|part| (part, next)));
                }
                Self::CaseBytesEqual {
                    left,
                    right,
                    equal_body,
                    unequal_body,
                } => {
                    stack.try_reserve(4).map_err(|_| ResourceLimit)?;
                    stack.push((unequal_body, next));
                    stack.push((equal_body, next));
                    stack.push((right, next));
                    stack.push((left, next));
                }
                Self::Call { arguments, .. } | Self::Request { arguments, .. } => {
                    stack
                        .try_reserve(arguments.len())
                        .map_err(|_| ResourceLimit)?;
                    stack.extend(arguments.iter().rev().map(|argument| (argument, next)));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn try_clone_resource(&self) -> Result<Self, ResourceLimit> {
        self.validate_resource_bounds()?;
        clone_expression(self, 1)
    }
}

fn clone_expression(expression: &KExpr, depth: usize) -> Result<KExpr, ResourceLimit> {
    #[derive(Clone, Copy)]
    enum Fixed {
        MakeAtom,
        MakeTriple,
        Let,
        CaseTerm,
        CaseBytes,
        CaseBytesEqual,
    }

    #[derive(Clone, Copy)]
    enum Sequence {
        ConcatBytes,
        Call(Id32),
        Request(Id32),
    }

    enum Task<'a> {
        Read(&'a KExpr, usize),
        Fixed(Fixed),
        Sequence(Sequence, usize),
    }

    let mut tasks = Vec::new();
    tasks.try_reserve(1).map_err(|_| ResourceLimit)?;
    tasks.push(Task::Read(expression, depth));
    let mut results = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Read(expression, depth) => {
                if depth > MAX_EXPRESSION_DEPTH {
                    return Err(ResourceLimit);
                }
                let next = depth + 1;
                match expression {
                    KExpr::BytesLiteral(bytes) => {
                        push_expression(&mut results, KExpr::BytesLiteral(try_copy_bytes(bytes)?))?
                    }
                    KExpr::TermLiteral(term) => push_expression(
                        &mut results,
                        KExpr::TermLiteral(term.try_clone_resource()?),
                    )?,
                    KExpr::Var(index) => push_expression(&mut results, KExpr::Var(*index))?,
                    KExpr::MakeAtom {
                        kind,
                        payload,
                        equality,
                    } => schedule_clone_fixed(
                        &mut tasks,
                        Fixed::MakeAtom,
                        &[kind, payload, equality],
                        next,
                    )?,
                    KExpr::MakeTriple {
                        first,
                        second,
                        third,
                    } => schedule_clone_fixed(
                        &mut tasks,
                        Fixed::MakeTriple,
                        &[first, second, third],
                        next,
                    )?,
                    KExpr::Let { value, body } => {
                        schedule_clone_fixed(&mut tasks, Fixed::Let, &[value, body], next)?;
                    }
                    KExpr::CaseTerm {
                        scrutinee,
                        atom_body,
                        triple_body,
                    } => schedule_clone_fixed(
                        &mut tasks,
                        Fixed::CaseTerm,
                        &[scrutinee, atom_body, triple_body],
                        next,
                    )?,
                    KExpr::CaseBytes {
                        scrutinee,
                        empty_body,
                        cons_body,
                    } => schedule_clone_fixed(
                        &mut tasks,
                        Fixed::CaseBytes,
                        &[scrutinee, empty_body, cons_body],
                        next,
                    )?,
                    KExpr::ConcatBytes(parts) => {
                        schedule_clone_sequence(&mut tasks, Sequence::ConcatBytes, parts, next)?
                    }
                    KExpr::CaseBytesEqual {
                        left,
                        right,
                        equal_body,
                        unequal_body,
                    } => schedule_clone_fixed(
                        &mut tasks,
                        Fixed::CaseBytesEqual,
                        &[left, right, equal_body, unequal_body],
                        next,
                    )?,
                    KExpr::Call {
                        definition_id,
                        arguments,
                    } => schedule_clone_sequence(
                        &mut tasks,
                        Sequence::Call(*definition_id),
                        arguments,
                        next,
                    )?,
                    KExpr::Request {
                        physical_operation_id,
                        arguments,
                    } => schedule_clone_sequence(
                        &mut tasks,
                        Sequence::Request(*physical_operation_id),
                        arguments,
                        next,
                    )?,
                }
            }
            Task::Fixed(kind) => {
                let expression = match kind {
                    Fixed::MakeAtom => {
                        let equality = pop_expression(&mut results)?;
                        let payload = pop_expression(&mut results)?;
                        let kind = pop_expression(&mut results)?;
                        KExpr::MakeAtom {
                            kind: Box::new(kind),
                            payload: Box::new(payload),
                            equality: Box::new(equality),
                        }
                    }
                    Fixed::MakeTriple => {
                        let third = pop_expression(&mut results)?;
                        let second = pop_expression(&mut results)?;
                        let first = pop_expression(&mut results)?;
                        KExpr::MakeTriple {
                            first: Box::new(first),
                            second: Box::new(second),
                            third: Box::new(third),
                        }
                    }
                    Fixed::Let => {
                        let body = pop_expression(&mut results)?;
                        let value = pop_expression(&mut results)?;
                        KExpr::Let {
                            value: Box::new(value),
                            body: Box::new(body),
                        }
                    }
                    Fixed::CaseTerm => {
                        let triple_body = pop_expression(&mut results)?;
                        let atom_body = pop_expression(&mut results)?;
                        let scrutinee = pop_expression(&mut results)?;
                        KExpr::CaseTerm {
                            scrutinee: Box::new(scrutinee),
                            atom_body: Box::new(atom_body),
                            triple_body: Box::new(triple_body),
                        }
                    }
                    Fixed::CaseBytes => {
                        let cons_body = pop_expression(&mut results)?;
                        let empty_body = pop_expression(&mut results)?;
                        let scrutinee = pop_expression(&mut results)?;
                        KExpr::CaseBytes {
                            scrutinee: Box::new(scrutinee),
                            empty_body: Box::new(empty_body),
                            cons_body: Box::new(cons_body),
                        }
                    }
                    Fixed::CaseBytesEqual => {
                        let unequal_body = pop_expression(&mut results)?;
                        let equal_body = pop_expression(&mut results)?;
                        let right = pop_expression(&mut results)?;
                        let left = pop_expression(&mut results)?;
                        KExpr::CaseBytesEqual {
                            left: Box::new(left),
                            right: Box::new(right),
                            equal_body: Box::new(equal_body),
                            unequal_body: Box::new(unequal_body),
                        }
                    }
                };
                push_expression(&mut results, expression)?;
            }
            Task::Sequence(kind, count) => {
                let values = take_expressions(&mut results, count)?;
                let expression = match kind {
                    Sequence::ConcatBytes => KExpr::ConcatBytes(values),
                    Sequence::Call(definition_id) => KExpr::Call {
                        definition_id,
                        arguments: values,
                    },
                    Sequence::Request(physical_operation_id) => KExpr::Request {
                        physical_operation_id,
                        arguments: values,
                    },
                };
                push_expression(&mut results, expression)?;
            }
        }
    }
    if results.len() != 1 {
        return Err(ResourceLimit);
    }
    let cloned = results.pop().ok_or(ResourceLimit)?;

    fn schedule_clone_fixed<'a>(
        tasks: &mut Vec<Task<'a>>,
        kind: Fixed,
        children: &[&'a KExpr],
        depth: usize,
    ) -> Result<(), ResourceLimit> {
        tasks
            .try_reserve(children.len() + 1)
            .map_err(|_| ResourceLimit)?;
        tasks.push(Task::Fixed(kind));
        for child in children.iter().rev() {
            tasks.push(Task::Read(child, depth));
        }
        Ok(())
    }

    fn schedule_clone_sequence<'a>(
        tasks: &mut Vec<Task<'a>>,
        kind: Sequence,
        children: &'a [KExpr],
        depth: usize,
    ) -> Result<(), ResourceLimit> {
        tasks
            .try_reserve(children.len() + 1)
            .map_err(|_| ResourceLimit)?;
        tasks.push(Task::Sequence(kind, children.len()));
        for child in children.iter().rev() {
            tasks.push(Task::Read(child, depth));
        }
        Ok(())
    }

    Ok(cloned)
}

fn push_expression(results: &mut Vec<KExpr>, expression: KExpr) -> Result<(), ResourceLimit> {
    results.try_reserve(1).map_err(|_| ResourceLimit)?;
    results.push(expression);
    Ok(())
}

fn pop_expression(results: &mut Vec<KExpr>) -> Result<KExpr, ResourceLimit> {
    results.pop().ok_or(ResourceLimit)
}

fn take_expressions(results: &mut Vec<KExpr>, count: usize) -> Result<Vec<KExpr>, ResourceLimit> {
    if results.len() < count {
        return Err(ResourceLimit);
    }
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| ResourceLimit)?;
    for _ in 0..count {
        values.push(pop_expression(results)?);
    }
    values.reverse();
    Ok(values)
}

impl KValue {
    pub(crate) fn validate_resource_bounds(&self) -> Result<(), ResourceLimit> {
        match self {
            Self::Bytes(bytes) if bytes.len() <= MAX_WIRE_BYTES => Ok(()),
            Self::Bytes(_) => Err(ResourceLimit),
            Self::Term(term) => term.validate_resource_bounds(),
        }
    }

    pub(crate) fn try_clone_resource(&self) -> Result<Self, ResourceLimit> {
        self.validate_resource_bounds()?;
        match self {
            Self::Bytes(bytes) => Ok(Self::Bytes(try_copy_bytes(bytes)?)),
            Self::Term(term) => Ok(Self::Term(term.try_clone_resource()?)),
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
#[derive(Debug, Eq, PartialEq)]
pub struct DecodedCompilerPackage {
    exact_input: Vec<u8>,
    exact_core_manifest: Range<usize>,
    exact_subject: Range<usize>,
    exact_evidence: Range<usize>,
    package: CompilerPackage,
}

impl DecodedCompilerPackage {
    pub(crate) fn new(
        exact_input: Vec<u8>,
        exact_core_manifest: Range<usize>,
        exact_subject: Range<usize>,
        exact_evidence: Range<usize>,
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
        &self.exact_input[self.exact_core_manifest.clone()]
    }

    #[must_use]
    pub fn exact_subject(&self) -> &[u8] {
        &self.exact_input[self.exact_subject.clone()]
    }

    #[must_use]
    pub fn exact_evidence(&self) -> &[u8] {
        &self.exact_input[self.exact_evidence.clone()]
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
    ResourceExhausted,
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
            Self::ResourceExhausted => {
                formatter.write_str("CLCP-v2 encode exhausted physical resources")
            }
        }
    }
}

impl std::error::Error for EncodeError {}
