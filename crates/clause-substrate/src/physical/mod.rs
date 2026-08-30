//! Sealed compiler physical profile.
//!
//! Package data cannot implement this boundary or add an operation. The only
//! callable target is FIPS-180-4 SHA-256 over one exact byte value.

use std::fmt;

use sha2::{Digest, Sha256};

use crate::compiler_package_v3::{
    Id32, KValue, MAX_TERM_NODES, MAX_WIRE_BYTES, MAX_WIRE_ITEMS, Term, sha256_operation_id,
    try_copy_bytes,
};

const K_TAG: &[u8] = b"clause/core-abi/tag/v1";
const K_BYTES: &[u8] = b"clause/core-abi/bytes/v1";
const K_ID32: &[u8] = b"clause/core-abi/id32/v1";
const K_U64: &[u8] = b"clause/core-abi/u64/v1";
const K_EQ: &[u8] = b"clause/core/bytes-equal/v1";

#[derive(Debug, Eq, PartialEq)]
pub struct PhysicalObservation {
    pub index: u64,
    pub operation_id: Id32,
    pub arguments: Vec<KValue>,
    pub result: KValue,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct ObservationLog {
    items: Vec<PhysicalObservation>,
    retained_payload_bytes: usize,
}

impl ObservationLog {
    #[must_use]
    pub fn items(&self) -> &[PhysicalObservation] {
        &self.items
    }

    #[must_use]
    pub const fn retained_payload_bytes(&self) -> usize {
        self.retained_payload_bytes
    }

    pub fn try_to_term(&self) -> Result<Term, PhysicalError> {
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
        arguments: Vec<KValue>,
        observations: &mut ObservationLog,
    ) -> Result<KValue, PhysicalError> {
        if operation_id != sha256_operation_id() {
            return Err(PhysicalError::UnknownOperation(operation_id));
        }
        let [KValue::Bytes(input)] = arguments.as_slice() else {
            return Err(PhysicalError::SignatureMismatch);
        };
        let index = u64::try_from(observations.items.len())
            .map_err(|_| PhysicalError::ObservationIndexOverflow)?;
        if observations.items.len() >= MAX_WIRE_ITEMS {
            return Err(PhysicalError::ResourceExhausted);
        }
        let digest = Sha256::digest(input);
        let retained_payload_bytes = observations
            .retained_payload_bytes
            .checked_add(input.len())
            .and_then(|bytes| bytes.checked_add(digest.len()))
            .ok_or(PhysicalError::ResourceExhausted)?;
        if retained_payload_bytes > MAX_WIRE_BYTES {
            return Err(PhysicalError::ResourceExhausted);
        }
        let result =
            KValue::Bytes(try_copy_bytes(&digest).map_err(|_| PhysicalError::ResourceExhausted)?);
        let recorded_result = result
            .try_clone_resource()
            .map_err(|_| PhysicalError::ResourceExhausted)?;
        observations
            .items
            .try_reserve(1)
            .map_err(|_| PhysicalError::ResourceExhausted)?;
        observations.items.push(PhysicalObservation {
            index,
            operation_id,
            arguments,
            result: recorded_result,
        });
        observations.retained_payload_bytes = retained_payload_bytes;
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
    ResourceExhausted,
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
            Self::ResourceExhausted => {
                formatter.write_str("physical observation exhausted resources")
            }
        }
    }
}

impl std::error::Error for PhysicalError {}

fn atom(kind: &[u8], payload: Vec<u8>) -> Result<Term, PhysicalError> {
    Ok(Term::Atom {
        kind: try_copy_bytes(kind).map_err(|_| PhysicalError::ResourceExhausted)?,
        canonical_payload: payload,
        equality_contract: try_copy_bytes(K_EQ).map_err(|_| PhysicalError::ResourceExhausted)?,
    })
}

fn tag(value: u8) -> Result<Term, PhysicalError> {
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(1)
        .map_err(|_| PhysicalError::ResourceExhausted)?;
    payload.push(value);
    atom(K_TAG, payload)
}

fn bytes(value: &[u8]) -> Result<Term, PhysicalError> {
    atom(
        K_BYTES,
        try_copy_bytes(value).map_err(|_| PhysicalError::ResourceExhausted)?,
    )
}

fn id(value: Id32) -> Result<Term, PhysicalError> {
    atom(
        K_ID32,
        try_copy_bytes(value.as_bytes()).map_err(|_| PhysicalError::ResourceExhausted)?,
    )
}

fn nat64(value: u64) -> Result<Term, PhysicalError> {
    atom(
        K_U64,
        try_copy_bytes(&value.to_be_bytes()).map_err(|_| PhysicalError::ResourceExhausted)?,
    )
}

fn list(values: Vec<Term>) -> Result<Term, PhysicalError> {
    if values.len() > MAX_TERM_NODES {
        return Err(PhysicalError::ResourceExhausted);
    }
    let mut tail = tag(0x00)?;
    for head in values.into_iter().rev() {
        tail = Term::try_triple(tag(0x01)?, head, tail)
            .map_err(|_| PhysicalError::ResourceExhausted)?;
    }
    Ok(tail)
}

fn record(record_tag: u8, fields: Vec<Term>) -> Result<Term, PhysicalError> {
    Term::try_triple(tag(record_tag)?, list(fields)?, tag(0x00)?)
        .map_err(|_| PhysicalError::ResourceExhausted)
}

fn one_field(value: Term) -> Result<Vec<Term>, PhysicalError> {
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(1)
        .map_err(|_| PhysicalError::ResourceExhausted)?;
    fields.push(value);
    Ok(fields)
}

fn value_term(value: &KValue) -> Result<Term, PhysicalError> {
    match value {
        KValue::Bytes(value) => record(0x02, one_field(bytes(value)?)?),
        KValue::Term(value) => record(
            0x03,
            one_field(
                value
                    .try_clone_resource()
                    .map_err(|_| PhysicalError::ResourceExhausted)?,
            )?,
        ),
    }
}

/// Canonical fixed-Core-ABI observations for receipt commitments and
/// judgments.
pub fn observations_term(observations: &[PhysicalObservation]) -> Result<Term, PhysicalError> {
    if observations.len() > MAX_WIRE_ITEMS {
        return Err(PhysicalError::ResourceExhausted);
    }
    let mut items = Vec::new();
    items
        .try_reserve_exact(observations.len())
        .map_err(|_| PhysicalError::ResourceExhausted)?;
    for observation in observations {
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(observation.arguments.len())
            .map_err(|_| PhysicalError::ResourceExhausted)?;
        for argument in &observation.arguments {
            arguments.push(value_term(argument)?);
        }
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(4)
            .map_err(|_| PhysicalError::ResourceExhausted)?;
        fields.push(nat64(observation.index)?);
        fields.push(id(observation.operation_id)?);
        fields.push(list(arguments)?);
        fields.push(value_term(&observation.result)?);
        items.push(record(0x19, fields)?);
    }
    record(0x1a, one_field(list(items)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_payload_budget_is_aggregate_and_failure_atomic() {
        let physical = SealedPhysical::new();
        let mut observations = ObservationLog::default();
        let first_input = vec![0x5a; MAX_WIRE_BYTES - 32];

        physical
            .request(
                sha256_operation_id(),
                vec![KValue::Bytes(first_input)],
                &mut observations,
            )
            .expect("one exact-limit observation fits");
        assert_eq!(observations.retained_payload_bytes(), MAX_WIRE_BYTES);
        assert_eq!(observations.items().len(), 1);

        assert_eq!(
            physical.request(
                sha256_operation_id(),
                vec![KValue::Bytes(Vec::new())],
                &mut observations,
            ),
            Err(PhysicalError::ResourceExhausted)
        );
        assert_eq!(observations.retained_payload_bytes(), MAX_WIRE_BYTES);
        assert_eq!(observations.items().len(), 1);
    }
}
