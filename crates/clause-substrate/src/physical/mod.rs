//! Sealed compiler physical profile.
//!
//! Package data cannot implement this boundary or add an operation. The only
//! callable target is FIPS-180-4 SHA-256 over one exact byte value.

use std::fmt;

use sha2::{Digest, Sha256};

use crate::compiler_package_v2::{Id32, KValue, Term, sha256_operation_id};

const K_TAG: &[u8] = b"clause/core-abi/tag/v1";
const K_BYTES: &[u8] = b"clause/core-abi/bytes/v1";
const K_ID32: &[u8] = b"clause/core-abi/id32/v1";
const K_U64: &[u8] = b"clause/core-abi/u64/v1";
const K_EQ: &[u8] = b"clause/core/bytes-equal/v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalObservation {
    pub index: u64,
    pub operation_id: Id32,
    pub arguments: Vec<KValue>,
    pub result: KValue,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObservationLog {
    items: Vec<PhysicalObservation>,
}

impl ObservationLog {
    #[must_use]
    pub fn items(&self) -> &[PhysicalObservation] {
        &self.items
    }

    #[must_use]
    pub fn to_term(&self) -> Term {
        observations_term(&self.items)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SealedPhysical {
    _private: Private,
}

#[derive(Clone, Copy, Debug)]
struct Private;

impl SealedPhysical {
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: Private }
    }

    pub(crate) fn request(
        self,
        operation_id: Id32,
        arguments: &[KValue],
        observations: &mut ObservationLog,
    ) -> Result<KValue, PhysicalError> {
        if operation_id != sha256_operation_id() {
            return Err(PhysicalError::UnknownOperation(operation_id));
        }
        let [KValue::Bytes(input)] = arguments else {
            return Err(PhysicalError::SignatureMismatch);
        };
        let index = u64::try_from(observations.items.len())
            .map_err(|_| PhysicalError::ObservationIndexOverflow)?;
        let digest = Sha256::digest(input).to_vec();
        let result = KValue::Bytes(digest);
        observations.items.push(PhysicalObservation {
            index,
            operation_id,
            arguments: arguments.to_vec(),
            result: result.clone(),
        });
        Ok(result)
    }
}

impl Default for SealedPhysical {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicalError {
    UnknownOperation(Id32),
    SignatureMismatch,
    ObservationIndexOverflow,
}

impl fmt::Display for PhysicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOperation(_) => {
                formatter.write_str("operation is outside the sealed profile")
            }
            Self::SignatureMismatch => {
                formatter.write_str("operation arguments do not match [Bytes] -> Bytes")
            }
            Self::ObservationIndexOverflow => formatter.write_str("observation index exceeds U64"),
        }
    }
}

impl std::error::Error for PhysicalError {}

fn atom(kind: &[u8], payload: Vec<u8>) -> Term {
    Term::Atom {
        kind: kind.to_vec(),
        canonical_payload: payload,
        equality_contract: K_EQ.to_vec(),
    }
}

fn tag(value: u8) -> Term {
    atom(K_TAG, vec![value])
}

fn bytes(value: Vec<u8>) -> Term {
    atom(K_BYTES, value)
}

fn id(value: Id32) -> Term {
    atom(K_ID32, value.0.to_vec())
}

fn nat64(value: u64) -> Term {
    atom(K_U64, value.to_be_bytes().to_vec())
}

fn list(values: impl IntoIterator<Item = Term>) -> Term {
    let values: Vec<Term> = values.into_iter().collect();
    values.into_iter().rev().fold(tag(0x00), |tail, head| {
        Term::Triple(Box::new(tag(0x01)), Box::new(head), Box::new(tail))
    })
}

fn record(record_tag: u8, fields: Vec<Term>) -> Term {
    Term::Triple(
        Box::new(tag(record_tag)),
        Box::new(list(fields)),
        Box::new(tag(0x00)),
    )
}

fn value_term(value: &KValue) -> Term {
    match value {
        KValue::Bytes(value) => record(0x02, vec![bytes(value.clone())]),
        KValue::Term(value) => record(0x03, vec![value.clone()]),
    }
}

/// Canonical fixed-Core-ABI observations for certificate statements and
/// judgments.
#[must_use]
pub fn observations_term(observations: &[PhysicalObservation]) -> Term {
    let items = observations.iter().map(|observation| {
        record(
            0x19,
            vec![
                nat64(observation.index),
                id(observation.operation_id),
                list(observation.arguments.iter().map(value_term)),
                value_term(&observation.result),
            ],
        )
    });
    record(0x1a, vec![list(items)])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostMechanicClass {
    WireCodec,
    CoreAbi,
    ByteMachine,
    DefinitionTable,
    KernelStep,
    CertificateStep,
    PhysicalDispatch,
}

impl HostMechanicClass {
    const fn name(self) -> &'static str {
        match self {
            Self::WireCodec => "WireCodec",
            Self::CoreAbi => "CoreABI",
            Self::ByteMachine => "ByteMachine",
            Self::DefinitionTable => "DefinitionTable",
            Self::KernelStep => "KernelStep",
            Self::CertificateStep => "CertificateStep",
            Self::PhysicalDispatch => "PhysicalDispatch",
        }
    }
}

/// One typed, stable host branch family in the trusted v2 mechanics closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostMechanicSite {
    pub site: &'static str,
    pub class: HostMechanicClass,
    pub controls: &'static str,
    pub fixed_tags: &'static str,
    pub code_target: &'static str,
}

/// This registry is consumed as structured data by the audit test. Its rows
/// name fixed mechanics and callable targets; no Clause semantic identifier or
/// Atom field can add a row or change a target.
pub const HOST_MECHANIC_SITES: &[HostMechanicSite] = &[
    HostMechanicSite {
        site: "compiler_package_v2::codec::decode",
        class: HostMechanicClass::WireCodec,
        controls: "octet,bound,length",
        fixed_tags: "CLCP-v2/frame/closed-sum",
        code_target: "closed-wire-decoders",
    },
    HostMechanicSite {
        site: "compiler_package_v2::codec::encode",
        class: HostMechanicClass::WireCodec,
        controls: "closed Rust wire value",
        fixed_tags: "CLCP-v2/frame/closed-sum",
        code_target: "closed-wire-encoders",
    },
    HostMechanicSite {
        site: "physical::observations_term",
        class: HostMechanicClass::CoreAbi,
        controls: "fixed ABI tag,arity,field",
        fixed_tags: "00,01,02,03,19,1a",
        code_target: "fixed-core-abi-constructor",
    },
    HostMechanicSite {
        site: "evaluator::Evaluator::step/bytes",
        class: HostMechanicClass::ByteMachine,
        controls: "empty,head-tail,equality",
        fixed_tags: "07,08,09",
        code_target: "selected-child-kexpr",
    },
    HostMechanicSite {
        site: "evaluator::DefinitionTable::resolve",
        class: HostMechanicClass::DefinitionTable,
        controls: "opaque-Id32-order,hit-miss",
        fixed_tags: "0a",
        code_target: "selected-package-kexpr-data",
    },
    HostMechanicSite {
        site: "evaluator::Evaluator::step",
        class: HostMechanicClass::KernelStep,
        controls: "KExpr-tag,value-shape,fuel",
        fixed_tags: "00..0b",
        code_target: "fixed-kexpr-mechanic",
    },
    HostMechanicSite {
        site: "evaluator::Evaluator::step/certificate-node",
        class: HostMechanicClass::CertificateStep,
        controls: "fixed-rule-tag,premise-index",
        fixed_tags: "30..3e",
        code_target: "fixed-eval-node-constructor",
    },
    HostMechanicSite {
        site: "physical::SealedPhysical::request",
        class: HostMechanicClass::PhysicalDispatch,
        controls: "fixed-PhysicalOpId",
        fixed_tags: "Sha256OpId",
        code_target: "sha2::Sha256::digest",
    },
];

/// Deterministic machine-readable TSV audit evidence.
#[must_use]
pub fn host_mechanics_evidence() -> String {
    let mut output = String::from("site\tclass\tcontrols\tfixed_tags\tcode_target\n");
    for site in HOST_MECHANIC_SITES {
        output.push_str(site.site);
        output.push('\t');
        output.push_str(site.class.name());
        output.push('\t');
        output.push_str(site.controls);
        output.push('\t');
        output.push_str(site.fixed_tags);
        output.push('\t');
        output.push_str(site.code_target);
        output.push('\n');
    }
    output
}
