use super::*;
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 4] = b"CWC1";
const NATIVE_MAGIC: &[u8; 4] = b"CWC2";

// Retaining the immutable allocation prevents address reuse from aliasing a
// cached digest. Each seal replaces this map with only its current segments.
#[derive(Default)]
pub(super) struct NativeCheckpointDigests {
    segments: std::collections::BTreeMap<(usize, usize), (clause_package::AtomPayloadSegment, [u8; 32])>,
}

impl NativeCheckpointDigests {
    fn seal(&mut self, segments: Vec<clause_package::AtomPayloadSegment>) -> Result<Vec<clause_package::AtomPayloadSegment>, WasmProcessStatusV1> {
        use clause_package::AtomPayloadSegment;
        let count = u32::try_from(segments.len()).map_err(|_| WasmProcessStatusV1::ResponseOutOfBounds)?;
        let mut metadata = NATIVE_MAGIC.to_vec();
        metadata.extend_from_slice(&count.to_le_bytes());
        let size = segments.len().checked_mul(40).and_then(|n| n.checked_add(32))
            .ok_or(WasmProcessStatusV1::ResponseOutOfBounds)?;
        metadata.try_reserve(size).map_err(|_| WasmProcessStatusV1::ResponseOutOfBounds)?;
        let mut retained = std::collections::BTreeMap::new();
        for segment in &segments {
            let bytes = segment.as_bytes();
            let key = (bytes.as_ptr() as usize, bytes.len());
            let digest = self.segments.get(&key).map(|(_, digest)| *digest)
                .unwrap_or_else(|| Sha256::digest(bytes).into());
            metadata.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            metadata.extend_from_slice(&digest);
            retained.insert(key, (segment.clone(), digest));
        }
        let root = Sha256::digest(&metadata);
        metadata.extend_from_slice(&root);
        self.segments = retained;
        let mut output = Vec::with_capacity(segments.len() + 1);
        output.push(AtomPayloadSegment::Bytes(metadata.into()));
        output.extend(segments);
        Ok(output)
    }
}

fn native_body(bytes: &[u8]) -> Result<&[u8], WasmProcessStatusV1> {
    let mut d = Decoder::new(bytes);
    if d.take(4)? != NATIVE_MAGIC { return Err(WasmProcessStatusV1::MalformedRequest); }
    let count = d.u32()? as usize;
    let metadata_length = count.checked_mul(40).and_then(|n| n.checked_add(8))
        .ok_or(WasmProcessStatusV1::MalformedRequest)?;
    let entries = d.take(metadata_length - 8)?;
    let root = d.take(32)?;
    if Sha256::digest(&bytes[..metadata_length]).as_slice() != root {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let body = &bytes[metadata_length + 32..];
    for entry in entries.chunks_exact(40) {
        let length = usize::try_from(u64::from_le_bytes(entry[..8].try_into().unwrap()))
            .map_err(|_| WasmProcessStatusV1::MalformedRequest)?;
        if Sha256::digest(d.take(length)?).as_slice() != &entry[8..] {
            return Err(WasmProcessStatusV1::MalformedRequest);
        }
    }
    if !d.is_complete() { return Err(WasmProcessStatusV1::MalformedRequest); }
    Ok(body)
}

struct RecordedBoundary<'a> {
    context: &'a [u8],
    exact_open: &'a [u8],
    runtime: &'a [u8],
    generation: u32,
    sequence: u64,
    command_window_start: u64,
    at_admitted_frontier: bool,
    last_input_sequence: u64,
    last_configuration_revision: u64,
}

fn decode(bytes: &[u8]) -> Result<RecordedBoundary<'_>, WasmProcessStatusV1> {
    let body = if bytes.starts_with(NATIVE_MAGIC) {
        native_body(bytes)?
    } else {
        if bytes.len() < 36 || !bytes.starts_with(MAGIC) {
            return Err(WasmProcessStatusV1::MalformedRequest);
        }
        let (body, digest) = bytes.split_at(bytes.len() - 32);
        if Sha256::digest(body).as_slice() != digest {
            return Err(WasmProcessStatusV1::MalformedRequest);
        }
        &body[4..]
    };
    decode_fields(body)
}

fn decode_fields(body: &[u8]) -> Result<RecordedBoundary<'_>, WasmProcessStatusV1> {
    let mut d = Decoder::new(body);
    let record = RecordedBoundary {
        context: d.blob(body.len())?,
        exact_open: d.blob(body.len())?,
        runtime: d.blob(body.len())?,
        generation: d.u32()?,
        sequence: d.u64()?,
        command_window_start: d.u64()?,
        at_admitted_frontier: match d.take(1)?[0] {
            0 => false,
            1 => true,
            _ => return Err(WasmProcessStatusV1::MalformedRequest),
        },
        last_input_sequence: d.u64()?,
        last_configuration_revision: d.u64()?,
    };
    if !d.is_complete() {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    Ok(record)
}

/// Inspect the exact caller context after checking the checkpoint's corruption
/// digest. This does not authenticate bytes or authorize their import.
pub fn wasm_session_checkpoint_context_v1(bytes: &[u8]) -> Result<&[u8], WasmProcessStatusV1> {
    Ok(decode(bytes)?.context)
}

impl WasmPersistentSessionBoundaryV1 {
    /// Export an idle admitted frontier from this exact live handle. `context`
    /// is opaque caller metadata, covered by the same corruption digest.
    pub fn checkpoint_admitted(
        &self,
        handle: WasmSessionHandleV1,
        context: &[u8],
    ) -> Result<Vec<u8>, WasmProcessStatusV1> {
        let mut bytes = Vec::new();
        for segment in self.checkpoint_admitted_segments(handle, context)? {
            bytes.try_reserve(segment.as_bytes().len()).map_err(|_| WasmProcessStatusV1::ResponseOutOfBounds)?;
            bytes.extend_from_slice(segment.as_bytes());
        }
        Ok(bytes)
    }

    /// Exact checkpoint bytes without copying immutable payloads. Segment
    /// boundaries are physical only; concatenation is `checkpoint_admitted`.
    pub fn checkpoint_admitted_segments(
        &self,
        handle: WasmSessionHandleV1,
        context: &[u8],
    ) -> Result<Vec<clause_package::AtomPayloadSegment>, WasmProcessStatusV1> {
        use clause_package::AtomPayloadSegment;
        let mut segments = vec![AtomPayloadSegment::Bytes(MAGIC.as_slice().into())];
        segments.extend(self.checkpoint_body_segments(handle, context)?);
        let mut digest = Sha256::new();
        for segment in &segments { digest.update(segment.as_bytes()); }
        segments.push(AtomPayloadSegment::Bytes(digest.finalize().to_vec().into()));
        Ok(segments)
    }

    /// Export a corruption-sealed native checkpoint, reusing immutable segment
    /// digests across saves. Concatenation is accepted by `reopen_admitted`.
    pub fn checkpoint_native_segments(
        &self,
        handle: WasmSessionHandleV1,
        context: &[u8],
    ) -> Result<Vec<clause_package::AtomPayloadSegment>, WasmProcessStatusV1> {
        let segments = self.checkpoint_body_segments(handle, context)?;
        self.checkpoint_digests.lock().map_err(|_| WasmProcessStatusV1::ProcessRejected)?.seal(segments)
    }

    fn checkpoint_body_segments(
        &self,
        handle: WasmSessionHandleV1,
        context: &[u8],
    ) -> Result<Vec<clause_package::AtomPayloadSegment>, WasmProcessStatusV1> {
        use clause_package::AtomPayloadSegment;
        let session = self.captured_session(handle)?;
        let live = self.live.as_ref().ok_or(WasmProcessStatusV1::StaleSessionHandle)?;
        let runtime = session.checkpoint_admitted_segments()
            .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let mut bytes = Vec::new();
        for value in [context, live.exact_open.as_slice()] {
            bytes.try_reserve(value.len().checked_add(4).ok_or(WasmProcessStatusV1::ResponseOutOfBounds)?)
                .map_err(|_| WasmProcessStatusV1::ResponseOutOfBounds)?;
            put_blob(&mut bytes, value)?;
        }
        let length = runtime.iter().try_fold(0usize, |length, segment| length.checked_add(segment.as_bytes().len()))
            .and_then(|length| u32::try_from(length).ok()).ok_or(WasmProcessStatusV1::ResponseOutOfBounds)?;
        bytes.extend_from_slice(&length.to_le_bytes());
        let mut segments = vec![AtomPayloadSegment::Bytes(bytes.into())];
        segments.extend(runtime);
        let mut tail = handle.generation.to_le_bytes().to_vec();
        tail.extend_from_slice(&live.sequence.to_le_bytes());
        tail.extend_from_slice(&live.command_window_start.to_le_bytes());
        tail.push(u8::from(live.at_admitted_frontier));
        tail.extend_from_slice(&live.last_input_sequence.to_le_bytes());
        tail.extend_from_slice(&live.last_configuration_revision.to_le_bytes());
        segments.push(AtomPayloadSegment::Bytes(tail.into()));
        Ok(segments)
    }

    /// Reconstitute the recorded world in an empty boundary, from the caller's
    /// trusted local Store. A mismatched program or corrupt checkpoint rejects;
    /// this never falls back to opening the initial world.
    pub fn reopen_admitted(
        &mut self,
        exact_open: &[u8],
        checkpoint: &[u8],
    ) -> Result<WasmSessionEventV1, WasmProcessStatusV1> {
        if self.live.is_some() || self.generation.is_some() || self.retired.is_some() {
            return Err(WasmProcessStatusV1::SessionOccupied);
        }
        let saved = decode(checkpoint)?;
        if saved.exact_open != exact_open || saved.generation == 0 {
            return Err(WasmProcessStatusV1::PackageRejected);
        }
        let request = decode_wasm_session_open_v1(exact_open)?;
        validate_limits(request.limits)?;
        if saved.command_window_start > saved.sequence
            || saved.sequence - saved.command_window_start > request.limits.max_commands
        {
            return Err(WasmProcessStatusV1::SequenceRejected);
        }
        let package = check_process_package(
            decode_process_package(&request.package_bytes)
                .map_err(|_| WasmProcessStatusV1::PackageRejected)?,
        )
        .map_err(|_| WasmProcessStatusV1::PackageRejected)?;
        let application = ApplicationId {
            snapshot: package.constitution().snapshot(),
            local: request.application,
        };
        let physical_plan = decode_executable_physical_plan_v1(&request.physical_plan_bytes)
            .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let (authority, facts) = establish_persistent_authority(&package, &request.authority)?;
        let session = PersistentProcessSessionV1::reopen_admitted(
            package,
            authority,
            application,
            physical_plan,
            facts,
            saved.runtime,
        )
        .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let handle = WasmSessionHandleV1 {
            slot: SLOT,
            generation: saved.generation,
        };
        let event = WasmSessionEventV1 {
            handle,
            accepted_sequence: saved.sequence,
            kind: WasmSessionEventKindV1::Opened {
                package: session
                    .package()
                    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?,
                session: session.runtime_session(),
                world: session.world_base(),
                run: session
                    .run()
                    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?,
                activation: session
                    .activation()
                    .map_err(|_| WasmProcessStatusV1::ProcessRejected)?,
                allocation: session.allocation(),
                state_revision_count: state_revision_count(&session)?,
            },
        };
        self.live = Some(LiveSessionV1 {
            exact_open: exact_open.to_vec(),
            session,
            sequence: saved.sequence,
            command_window_start: saved.command_window_start,
            at_admitted_frontier: saved.at_admitted_frontier,
            limits: request.limits,
            last_input_sequence: saved.last_input_sequence,
            last_configuration_revision: saved.last_configuration_revision,
        });
        self.generation = Some(saved.generation);
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_seal_checks_segments_even_with_recomputed_metadata_root() {
        use clause_package::AtomPayloadSegment;
        let first = AtomPayloadSegment::Bytes(b"first".as_slice().into());
        let second = AtomPayloadSegment::Bytes(b"other".as_slice().into());
        let mut cache = NativeCheckpointDigests::default();
        let segments = cache.seal(vec![first.clone(), second.clone()]).unwrap();
        let bytes: Vec<u8> = segments.iter().flat_map(|s| s.as_bytes().iter().copied()).collect();
        assert_eq!(native_body(&bytes).unwrap(), b"firstother");
        for (start, end) in [(8, 48), (16, 48)] {
            let mut corrupt = bytes.clone();
            if start == 8 {
                let other = corrupt[48..88].to_vec();
                let original = corrupt[start..end].to_vec();
                corrupt[start..end].copy_from_slice(&other);
                corrupt[48..88].copy_from_slice(&original);
            } else {
                corrupt[start] ^= 1;
            }
            let root = Sha256::digest(&corrupt[..88]);
            corrupt[88..120].copy_from_slice(&root);
            assert!(native_body(&corrupt).is_err());
        }
        cache.seal(vec![second.clone()]).unwrap();
        assert_eq!(cache.segments.len(), 1);
        let retained = &cache.segments.values().next().unwrap().0;
        assert!(std::ptr::eq(retained.as_bytes(), second.as_bytes()));
    }

    #[test]
    fn checkpoint_envelope_rejects_corruption_and_truncated_blob() {
        let mut bytes = MAGIC.to_vec();
        let context = vec![0; 1024];
        for value in [context.as_slice(), &[], &[]] {
            put_blob(&mut bytes, value).unwrap();
        }
        bytes.extend_from_slice(&[0; 37]);
        let digest = Sha256::digest(&bytes);
        bytes.extend_from_slice(&digest);
        assert_eq!(wasm_session_checkpoint_context_v1(&bytes).unwrap(), context);

        *bytes.last_mut().unwrap() ^= 1;
        assert_eq!(
            wasm_session_checkpoint_context_v1(&bytes),
            Err(WasmProcessStatusV1::MalformedRequest),
        );

        bytes.truncate(bytes.len() - 32);
        bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        let digest = Sha256::digest(&bytes);
        bytes.extend_from_slice(&digest);
        assert_eq!(
            wasm_session_checkpoint_context_v1(&bytes),
            Err(WasmProcessStatusV1::RequestOutOfBounds),
        );
    }
}
