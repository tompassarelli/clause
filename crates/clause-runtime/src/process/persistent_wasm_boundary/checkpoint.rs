use super::super::MAX_ADMITTED_CHECKPOINT_BYTES_V1 as LIMIT;
use super::*;
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 4] = b"CWC1";

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
    if bytes.len() < 36 || bytes.len() > LIMIT {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let (body, digest) = bytes.split_at(bytes.len() - 32);
    if Sha256::digest(body).as_slice() != digest {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let mut d = Decoder::new(body);
    if d.take(4)? != MAGIC {
        return Err(WasmProcessStatusV1::MalformedRequest);
    }
    let record = RecordedBoundary {
        context: d.blob(LIMIT)?,
        exact_open: d.blob(LIMIT)?,
        runtime: d.blob(LIMIT)?,
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

/// Read the exact admitted opening recipe after checking the checkpoint's
/// corruption digest. Callers must still check the package, source binding,
/// and runtime frontier; this accessor grants no authority to import it.
pub fn wasm_session_checkpoint_open_v1(bytes: &[u8]) -> Result<&[u8], WasmProcessStatusV1> {
    Ok(decode(bytes)?.exact_open)
}

impl WasmPersistentSessionBoundaryV1 {
    /// Export an idle admitted frontier from this exact live handle. `context`
    /// is opaque caller metadata, covered by the same corruption digest.
    pub fn checkpoint_admitted(
        &self,
        handle: WasmSessionHandleV1,
        context: &[u8],
    ) -> Result<Vec<u8>, WasmProcessStatusV1> {
        let session = self.captured_session(handle)?;
        let live = self
            .live
            .as_ref()
            .ok_or(WasmProcessStatusV1::StaleSessionHandle)?;
        let runtime = session
            .checkpoint_admitted()
            .map_err(|_| WasmProcessStatusV1::ProcessRejected)?;
        let mut bytes = MAGIC.to_vec();
        for value in [context, live.exact_open.as_slice(), runtime.as_slice()] {
            if bytes.len().saturating_add(value.len()).saturating_add(4) > LIMIT {
                return Err(WasmProcessStatusV1::ResponseOutOfBounds);
            }
            put_blob(&mut bytes, value)?;
        }
        bytes.extend_from_slice(&handle.generation.to_le_bytes());
        bytes.extend_from_slice(&live.sequence.to_le_bytes());
        bytes.extend_from_slice(&live.command_window_start.to_le_bytes());
        bytes.push(u8::from(live.at_admitted_frontier));
        bytes.extend_from_slice(&live.last_input_sequence.to_le_bytes());
        bytes.extend_from_slice(&live.last_configuration_revision.to_le_bytes());
        let digest = Sha256::digest(&bytes);
        bytes.extend_from_slice(&digest);
        if bytes.len() > LIMIT {
            return Err(WasmProcessStatusV1::ResponseOutOfBounds);
        }
        Ok(bytes)
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
    fn checkpoint_envelope_rejects_corruption_and_valid_oversize_encoding() {
        let mut bytes = MAGIC.to_vec();
        // Three blobs, the fixed boundary fields, and the digest consume 85 bytes.
        let context = vec![0; LIMIT - 85];
        for value in [context.as_slice(), &[], &[]] {
            put_blob(&mut bytes, value).unwrap();
        }
        bytes.extend_from_slice(&[0; 37]);
        let digest = Sha256::digest(&bytes);
        bytes.extend_from_slice(&digest);
        assert_eq!(bytes.len(), LIMIT);
        assert_eq!(wasm_session_checkpoint_context_v1(&bytes).unwrap(), context);

        *bytes.last_mut().unwrap() ^= 1;
        assert_eq!(
            wasm_session_checkpoint_context_v1(&bytes),
            Err(WasmProcessStatusV1::MalformedRequest),
        );

        bytes.truncate(bytes.len() - 32);
        bytes.insert(8, 0);
        bytes[4..8].copy_from_slice(&((context.len() + 1) as u32).to_le_bytes());
        let digest = Sha256::digest(&bytes);
        bytes.extend_from_slice(&digest);
        assert_eq!(bytes.len(), LIMIT + 1);
        assert_eq!(
            wasm_session_checkpoint_context_v1(&bytes),
            Err(WasmProcessStatusV1::MalformedRequest),
        );
    }
}
