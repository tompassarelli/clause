//! Passive CSC1 projection of every source occurrence and physical slot pair.
use super::{WasmProcessStatusV1, super::ExecutableSourceContinuityV1};

const HEADER_BYTES: usize = 76;
const OCCURRENCE_BYTES: usize = 76;
const SLOT_BYTES: usize = 4;

pub(super) fn encode(value: &ExecutableSourceContinuityV1, limit: usize) -> Result<Vec<u8>, WasmProcessStatusV1> {
    let too_large = || WasmProcessStatusV1::ResponseOutOfBounds;
    let occurrences = u32::try_from(value.occurrences.len()).map_err(|_| too_large())?;
    let slots = u32::try_from(value.slots.len()).map_err(|_| too_large())?;
    let length = value.occurrences.len().checked_mul(OCCURRENCE_BYTES)
        .and_then(|length| value.slots.len().checked_mul(SLOT_BYTES).and_then(|slots| length.checked_add(slots)))
        .and_then(|length| length.checked_add(HEADER_BYTES))
        .filter(|length| *length <= limit).ok_or_else(too_large)?;
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(b"CSC1");
    bytes.extend_from_slice(value.old_snapshot.as_bytes());
    bytes.extend_from_slice(value.new_snapshot.as_bytes());
    bytes.extend_from_slice(&occurrences.to_le_bytes());
    bytes.extend_from_slice(&slots.to_le_bytes());
    for (old, new, first_snapshot, first, occurrence) in &value.occurrences {
        bytes.extend_from_slice(&old.get().to_le_bytes());
        bytes.extend_from_slice(&new.get().to_le_bytes());
        bytes.extend_from_slice(first_snapshot.as_bytes());
        bytes.extend_from_slice(&first.get().to_le_bytes());
        bytes.extend_from_slice(occurrence);
    }
    for (old, new) in &value.slots {
        bytes.extend_from_slice(&old.to_le_bytes());
        bytes.extend_from_slice(&new.to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clause_package::ProgramSnapshotId;
    use std::collections::BTreeMap;

    #[test]
    fn source_continuity_aggregate_accepts_exact_limit_and_rejects_limit_plus_one() {
        let limit = super::super::WASM_SOURCE_CONTINUITY_LIMIT_V1;
        let mut value = ExecutableSourceContinuityV1 {
            old_snapshot: ProgramSnapshotId::from_bytes([1; 32]),
            new_snapshot: ProgramSnapshotId::from_bytes([2; 32]),
            identities: BTreeMap::new(),
            slots: vec![(0, u16::MAX); (limit - HEADER_BYTES) / SLOT_BYTES],
            occurrences: vec![],
        };
        let exact = encode(&value, limit).unwrap();
        assert_eq!(exact.len(), limit);
        assert_eq!(encode(&value, limit - 1), Err(WasmProcessStatusV1::ResponseOutOfBounds));
        value.slots.push((u16::MAX, 0));
        assert_eq!(encode(&value, limit), Err(WasmProcessStatusV1::ResponseOutOfBounds));
    }
}
