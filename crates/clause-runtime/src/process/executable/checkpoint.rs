//! Physical reconstitution of a trusted, already-admitted world frontier.
use super::*;

const MAGIC: &[u8; 4] = b"CRF1";

macro_rules! ordinal_wire {
    ($($field:ident),+ $(,)?) => {
        impl RuntimeIdentityOrdinalsV1 {
            fn encode(&self, bytes: &mut impl ExecutableBytes) { $(bytes.extend_from_slice(&self.$field.to_le_bytes());)+ }
            fn decode(d: &mut Decoder<'_>) -> Result<Self, ExecutableErrorV1> {
                Ok(Self { $($field: d.u64()?,)+ })
            }
        }
    };
}
ordinal_wire!(
    next_run,
    next_activation,
    next_configuration,
    next_step,
    next_input_observation,
    next_state_observation,
    next_checker,
    next_candidate,
    next_admission_authorization,
    next_continuation,
    next_resumption,
    next_effect_intent,
    next_effect_authorization,
    next_effect_attempt,
    next_effect_receipt,
    next_effect_judgment,
    next_effect_observation
);

fn blob(bytes: &mut SegmentedBytes, value: &[u8]) -> Result<(), ExecutableErrorV1> {
    let length = u32::try_from(value.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
    bytes.current.try_reserve(value.len().checked_add(4).ok_or(ExecutableErrorV1::ResourceLimit)?)
        .map_err(|_| ExecutableErrorV1::ResourceLimit)?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value);
    Ok(())
}
fn read_blob<'a>(d: &mut Decoder<'a>) -> Result<&'a [u8], ExecutableErrorV1> {
    let count = d.u32()? as usize;
    d.take(count)
}

impl ExecutableProcessRuntimeV1 {
    pub(crate) fn checkpoint_admitted(&self) -> Result<Vec<u8>, ExecutableCarrierErrorV1> {
        let mut bytes = Vec::new();
        for segment in self.checkpoint_admitted_segments()? {
            bytes.try_reserve(segment.as_bytes().len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
            bytes.extend_from_slice(segment.as_bytes());
        }
        Ok(bytes)
    }

    pub(crate) fn checkpoint_admitted_segments(&self) -> Result<Vec<AtomPayloadSegment>, ExecutableCarrierErrorV1> {
        let execution = self
            .carrier_execution
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
        if execution.state_started
            || execution.prior_step.is_some()
            || self.candidate.is_some()
            || self.suspended_continuation.is_some()
            || self.pending_effect_intent.is_some()
            || self.active_effect_attempt.is_some()
        {
            return Err(ExecutableCarrierErrorV1::HistoryCompactionUnavailable);
        }
        let frontier = self
            .carrier
            .carrier()
            .record_admitted_frontier(execution.facts.initial_state)
            .map_err(|_| ExecutableCarrierErrorV1::HistoryCompactionUnavailable)?;
        let encoded = encode_recorded_admitted_frontier_segments_v1(&frontier)
            .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
        let mut bytes = SegmentedBytes::default();
        bytes.extend_from_slice(MAGIC);
        blob(
            &mut bytes,
            &encode_runtime_allocation_epoch_v1(self.allocation),
        )?;
        let length = encoded.iter().try_fold(0usize, |length, segment| length.checked_add(segment.as_bytes().len()))
            .and_then(|length| u32::try_from(length).ok()).ok_or(ExecutableErrorV1::ResourceLimit)?;
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.flush();
        bytes.segments.extend(encoded);
        bytes.extend_from_slice(self.run.as_bytes());
        bytes.extend_from_slice(self.activation.as_bytes());
        bytes.extend_from_slice(self.configuration_id.as_bytes());
        bytes.extend_from_slice(&execution.remaining_budget.to_le_bytes());
        self.identity_ordinals.encode(&mut bytes);
        encode_slots(&mut bytes, &self.configuration)?;
        encode_continuity(&mut bytes, self.source_continuity.as_ref())?;
        Ok(bytes.finish())
    }

    pub(crate) fn reopen_admitted(
        package: CheckedProcessPackage,
        authority: AuthorityStore,
        application: ApplicationId,
        physical_plan: ExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
        bytes: &[u8],
    ) -> Result<Self, ExecutableCarrierErrorV1> {
        let mut d = Decoder::new(bytes);
        if d.take(4)? != MAGIC {
            return Err(ExecutableErrorV1::MalformedProgram.into());
        }
        let allocation = decode_runtime_allocation_epoch_v1(read_blob(&mut d)?)?;
        let frontier = decode_recorded_admitted_frontier_v1(read_blob(&mut d)?)
            .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
        let state = frontier.state().clone();
        if state.session != facts.session || state.policy != facts.policy {
            return Err(ExecutableErrorV1::MalformedProgram.into());
        }
        let mut runtime = Self::instantiate_rematerialized(
            package,
            authority,
            application,
            physical_plan,
            facts,
            allocation,
        )?;
        runtime.start_carrier_process(facts)?;
        runtime.run = RunId::from_bytes(d.identity()?);
        runtime.activation = ActivationId::from_bytes(d.identity()?);
        runtime.configuration_id = ConfigurationId::from_bytes(d.identity()?);
        let remaining_budget = d.u64()?;
        if remaining_budget > facts.budget_units {
            return Err(ExecutableErrorV1::MalformedProgram.into());
        }
        runtime.identity_ordinals = RuntimeIdentityOrdinalsV1::decode(&mut d)?;
        let count = d.count()?;
        let mut configuration = Vec::with_capacity(count);
        for _ in 0..count {
            configuration.push(match d.byte()? {
                0 => ExecutableSlotV1::Absent(match d.byte()? {
                    0 => ExecutableValueKindV1::Number,
                    1 => ExecutableValueKindV1::Boolean,
                    2 => ExecutableValueKindV1::Symbol,
                    3 => ExecutableValueKindV1::NumberSet,
                    4 => ExecutableValueKindV1::BooleanSet,
                    5 => ExecutableValueKindV1::SymbolSet,
                    6 => ExecutableValueKindV1::Text,
                    7 => ExecutableValueKindV1::TextSet,
                    8 => ExecutableValueKindV1::Referent,
                    9 => ExecutableValueKindV1::ReferentSet,
                    10 => ExecutableValueKindV1::RelationTable,
                    _ => return Err(ExecutableErrorV1::MalformedProgram.into()),
                }),
                1 => ExecutableSlotV1::Present(d.value()?),
                _ => return Err(ExecutableErrorV1::MalformedProgram.into()),
            });
        }
        runtime.source_continuity = decode_continuity(&mut d)?;
        if !d.is_complete()
            || configuration.len() != runtime.configuration.len()
            || configuration
                .iter()
                .zip(&runtime.configuration)
                .any(|(a, b)| a.kind() != b.kind())
        {
            return Err(ExecutableErrorV1::MalformedProgram.into());
        }
        relational::validate_contracts(&configuration)?;
        if matches!(state.cause, StateRevisionCause::Admission { .. }) {
            let scope = TermScope {
                universe: runtime.carrier.carrier().constitution().universe(),
                semantics: runtime.carrier.carrier().constitution().semantics(),
            };
            if executable_configuration_term_v1(scope, &configuration)? != state.payload {
                return Err(ExecutableErrorV1::MalformedProgram.into());
            }
        }
        runtime
            .carrier
            .restore_admitted_frontier(frontier)
            .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        runtime.configuration = configuration;
        let execution = runtime
            .carrier_execution
            .as_mut()
            .expect("reopen started execution");
        execution.facts.initial_state = state.id;
        execution.facts.budget_units = remaining_budget;
        execution.remaining_budget = remaining_budget;
        if let StateRevisionCause::Admission { occurrence, .. } = state.cause {
            execution.epoch_origin = CausalRef::Admission(occurrence);
            execution.state_base_support = SupportSource::Admission(occurrence);
        }
        // Exact re-encoding rejects alternate encodings of maps, sets and scalars.
        if runtime.checkpoint_admitted()? != bytes {
            return Err(ExecutableErrorV1::MalformedProgram.into());
        }
        Ok(runtime)
    }
}

fn encode_identity(bytes: &mut impl ExecutableBytes, value: CanonicalAllocatedIdentityV1) {
    use CanonicalAllocatedIdentityV1 as I;
    let (tag, a, b) = match value {
        I::Formation(a) => (0, a.get(), 0),
        I::Capability(a) => (1, a.get(), 0),
        I::RelationSchema(a) => (2, a.get(), 0),
        I::Role(a) => (3, a.schema.get(), a.role.get()),
        I::Operator(a) => (4, a.get(), 0),
        I::Mode(a) => (5, a.operator.get(), a.mode.get()),
    };
    bytes.push(tag);
    bytes.extend_from_slice(&a.to_le_bytes());
    bytes.extend_from_slice(&b.to_le_bytes());
}
fn decode_identity(d: &mut Decoder<'_>) -> Result<CanonicalAllocatedIdentityV1, ExecutableErrorV1> {
    let tag = d.byte()?;
    let a = d.u32()?;
    let b = d.u32()?;
    use CanonicalAllocatedIdentityV1 as I;
    Ok(match (tag, b) {
        (0, 0) => I::Formation(FormationLocalId::new(a)),
        (1, 0) => I::Capability(CapabilityLocalId::new(a)),
        (2, 0) => I::RelationSchema(RelationSchemaLocalId::new(a)),
        (3, _) => I::Role(LocalRoleRefV2 {
            schema: RelationSchemaLocalId::new(a),
            role: RoleLocalId::new(b),
        }),
        (4, 0) => I::Operator(OperatorLocalId::new(a)),
        (5, _) => I::Mode(LocalModeRefV2 {
            operator: OperatorLocalId::new(a),
            mode: ModeLocalId::new(b),
        }),
        _ => return Err(ExecutableErrorV1::MalformedProgram),
    })
}
fn encode_continuity(
    bytes: &mut impl ExecutableBytes,
    value: Option<&ExecutableSourceContinuityV1>,
) -> Result<(), ExecutableErrorV1> {
    bytes.push(u8::from(value.is_some()));
    let Some(value) = value else {
        return Ok(());
    };
    bytes.extend_from_slice(value.old_snapshot.as_bytes());
    bytes.extend_from_slice(value.new_snapshot.as_bytes());
    encode_count(bytes, value.identities.len())?;
    for (a, b) in &value.identities {
        encode_identity(bytes, *a);
        encode_identity(bytes, *b);
    }
    encode_count(bytes, value.slots.len())?;
    for (a, b) in &value.slots {
        bytes.extend_from_slice(&a.to_le_bytes());
        bytes.extend_from_slice(&b.to_le_bytes());
    }
    encode_count(bytes, value.occurrences.len())?;
    for (a, b, snapshot, first, occurrence) in &value.occurrences {
        bytes.extend_from_slice(&a.get().to_le_bytes());
        bytes.extend_from_slice(&b.get().to_le_bytes());
        bytes.extend_from_slice(snapshot.as_bytes());
        bytes.extend_from_slice(&first.get().to_le_bytes());
        bytes.extend_from_slice(occurrence);
    }
    Ok(())
}
fn decode_continuity(
    d: &mut Decoder<'_>,
) -> Result<Option<ExecutableSourceContinuityV1>, ExecutableErrorV1> {
    match d.byte()? {
        0 => return Ok(None),
        1 => {}
        _ => return Err(ExecutableErrorV1::MalformedProgram),
    }
    let old_snapshot = ProgramSnapshotId::from_bytes(d.identity()?);
    let new_snapshot = ProgramSnapshotId::from_bytes(d.identity()?);
    let mut identities = BTreeMap::new();
    for _ in 0..d.count()? {
        let a = decode_identity(d)?;
        let b = decode_identity(d)?;
        if identities.insert(a, b).is_some() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    let mut slots = Vec::new();
    for _ in 0..d.count()? {
        slots.push((d.u16()?, d.u16()?));
    }
    let mut occurrences = Vec::new();
    for _ in 0..d.count()? {
        occurrences.push((
            FormationLocalId::new(d.u32()?),
            FormationLocalId::new(d.u32()?),
            ProgramSnapshotId::from_bytes(d.identity()?),
            FormationLocalId::new(d.u32()?),
            d.identity()?,
        ));
    }
    Ok(Some(ExecutableSourceContinuityV1 {
        old_snapshot,
        new_snapshot,
        identities,
        slots,
        occurrences,
    }))
}
