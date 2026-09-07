use super::*;

impl ResidentSourceWorkbenchV1 {
    /// Checkpoint the current admitted world without executing input or effects.
    /// The caller owns atomic storage and trusted file selection. Pending local
    /// work must be admitted first; the checkpoint retains no completed trace.
    pub fn checkpoint_admitted(&self) -> Result<Vec<u8>, ResidentSourceWorkbenchErrorV1> {
        if self.pending.is_some() {
            return Err(ResidentSourceWorkbenchErrorV1(
                "admit the pending candidate before checkpointing".into(),
            ));
        }
        let mut context = b"CRS1".to_vec();
        context.extend_from_slice(&self.next_change.to_le_bytes());
        context.push(self.trace_retention as u8);
        for value in [
            self.exact_source.as_slice(),
            self.declared_frontend.exact_source(),
            self.last_source_edit.as_deref().unwrap_or(&[]),
            self.last_projection
                .as_ref()
                .map(|projection| projection.observation.as_bytes().as_slice())
                .unwrap_or(&[]),
        ] {
            let length = u32::try_from(value.len()).map_err(|_| invalid())?;
            context.extend_from_slice(&length.to_le_bytes());
            context.extend_from_slice(value);
        }
        Ok(self
            .boundary
            .checkpoint_admitted(self.generation.handle, &context)?)
    }

    /// Reopen a checkpoint selected from the same trusted local Store. The
    /// digest detects corruption, not forgery. Exact source, declared frontend,
    /// package, allocation and authority bindings must agree. Rejection never
    /// discards stored data or silently opens a fresh world.
    pub fn reopen(
        exact_source: &[u8],
        checkpoint: &[u8],
    ) -> Result<Self, ResidentSourceWorkbenchErrorV1> {
        let mut context = clause_runtime::wasm_session_checkpoint_context_v1(checkpoint)?;
        if take(&mut context, 4)? != b"CRS1" {
            return Err(invalid());
        }
        let change = u64::from_le_bytes(take(&mut context, 8)?.try_into().map_err(|_| invalid())?);
        let retention = match take(&mut context, 1)?[0] {
            0 => clause_runtime::WasmSessionTraceRetentionV1::FullUntilCommandLimit,
            1 => clause_runtime::WasmSessionTraceRetentionV1::CurrentAdmission,
            _ => return Err(invalid()),
        };
        let source = blob(&mut context)?;
        let frontend = blob(&mut context)?;
        let edit = blob(&mut context)?;
        let observation = blob(&mut context)?;
        if !context.is_empty() || source != exact_source {
            return Err(invalid());
        }
        let mut workbench =
            Self::open_with_checkpoint(source, frontend, retention, Some((change, checkpoint)))?;
        if !edit.is_empty() {
            workbench.last_source_edit = Some(edit.to_vec());
        }
        if !observation.is_empty() {
            let observation = clause_package::ObservationId::from_bytes(
                observation.try_into().map_err(|_| invalid())?,
            );
            let term = workbench
                .boundary
                .current_accepted_projection_term(workbench.generation.handle)?;
            workbench.last_projection = Some(WasmSessionProjectionV1 {
                observation,
                term,
            });
        }
        Ok(workbench)
    }
}

fn invalid() -> ResidentSourceWorkbenchErrorV1 {
    ResidentSourceWorkbenchErrorV1("checkpoint is invalid or belongs to different source".into())
}
fn take<'a>(
    bytes: &mut &'a [u8],
    count: usize,
) -> Result<&'a [u8], ResidentSourceWorkbenchErrorV1> {
    let (value, rest) = bytes.split_at_checked(count).ok_or_else(invalid)?;
    *bytes = rest;
    Ok(value)
}
fn blob<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], ResidentSourceWorkbenchErrorV1> {
    let count = u32::from_le_bytes(take(bytes, 4)?.try_into().map_err(|_| invalid())?) as usize;
    take(bytes, count)
}
