use clause_runtime::*;
use clause_workbench::ResidentSourceWorkbenchV1;

fn open_bytes(cwr1: &[u8]) -> Vec<u8> {
    let request = decode_wasm_process_request_v1(cwr1).unwrap();
    encode_wasm_session_open_v1(&WasmSessionOpenV1 {
        package_bytes: request.package_bytes,
        application: request.application,
        physical_plan_bytes: request.physical_plan_bytes,
        authority: request.authority,
        allocation: WasmSessionAllocationV1::New,
        limits: WasmSessionLimitsV1 {
            max_commands: 4096,
            command_bytes: WASM_SESSION_COMMAND_LIMIT_V1 as u32,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1 as u32,
            trace_retention: WasmSessionTraceRetentionV1::CurrentAdmission,
        },
    }).unwrap()
}

#[test]
fn independently_prepared_boundary_preserves_custody_and_rejects_stale_source() {
    let source = include_bytes!("../../../test-vectors/authoring/live-encounter.clause");
    let mut producer = ResidentSourceWorkbenchV1::open_continuous(source).unwrap();
    let capsule = producer.source_preparation().unwrap();
    let mut consumer = WasmPersistentSessionBoundaryV1::new();
    let initial = consumer.open(&open_bytes(&producer.generation().cwr1)).unwrap();
    let before = consumer.current_accepted_projection_bytes(initial.handle).unwrap();
    assert_eq!(consumer.prepare_source(initial.handle, 1, &capsule), Err(WasmProcessStatusV1::SequenceRejected));
    consumer.prepare_source(initial.handle, 0, &capsule).unwrap();
    assert_eq!(consumer.current_accepted_projection_bytes(initial.handle).unwrap(), before);
    let mut corrupt = capsule.clone();
    corrupt[4] ^= 1;
    assert!(consumer.prepare_source(initial.handle, 0, &corrupt).is_err());
    assert_eq!(consumer.current_accepted_projection_bytes(initial.handle).unwrap(), before);
    let mut current = initial;
    for (old, new) in [(b"0.0 - ?damage".as_slice(), b"0.0 - (?damage * 2.0)".as_slice()), (b"0.0 - (?damage * 2.0)", b"0.0 - ?damage")] {
        let effect = producer.scalar_effects().unwrap().into_iter().find(|effect| effect.expression == old).unwrap();
        producer.edit_scalar_effect(producer.generation().handle, &effect, new).unwrap();
        let witness = producer.last_source_edit().unwrap();
        let before = consumer.current_accepted_projection_bytes(current.handle).unwrap();
        let mut tampered = decode_executable_scalar_edit_transaction_v1(witness).unwrap();
        tampered.new_root = clause_package::ProgramChangeOccurrenceId::from_bytes([99;32]);
        assert!(consumer.scalar_edit(current.handle, 0, &encode_executable_scalar_edit_transaction_v1(&tampered).unwrap()).is_err());
        assert_eq!(consumer.current_accepted_projection_bytes(current.handle).unwrap(), before);
        let mut wrong_identity = decode_executable_scalar_edit_transaction_v1(witness).unwrap();
        wrong_identity.new_plan = wrong_identity.old_plan;
        assert!(consumer.scalar_edit(current.handle, 0, &encode_executable_scalar_edit_transaction_v1(&wrong_identity).unwrap()).is_err());
        assert!(consumer.scalar_edit(current.handle, 0, &witness[..witness.len()-1]).is_err());
        let next = consumer.scalar_edit(current.handle, 0, witness).unwrap();
        assert_eq!(consumer.prepare_source(current.handle, 0, &capsule), Err(WasmProcessStatusV1::StaleSessionHandle));
        assert!(consumer.prepare_source(next.handle, 0, &capsule).is_err());
        let compact = consumer.source_continuity_bytes(next.handle).unwrap();
        let diagnostic = consumer.source_continuity_term(next.handle).unwrap();
        assert_complete_continuity_projection(&compact, &diagnostic);
        assert_eq!(consumer.source_continuity_bytes(current.handle), Err(WasmProcessStatusV1::StaleSessionHandle));
        while consumer.reclaim_retired() {}
        current = next;
    }
}

#[test]
fn native_prepared_token_rejects_changed_custody_and_other_boundary() {
    let source = include_bytes!("../../../test-vectors/authoring/live-encounter.clause");
    let producer = ResidentSourceWorkbenchV1::open_continuous(source).unwrap();
    let capsule = producer.source_preparation().unwrap();
    let open = open_bytes(&producer.generation().cwr1);
    let mut a = WasmPersistentSessionBoundaryV1::new();
    let initial = a.open_prepared(&open, &capsule).unwrap();
    let effect = producer.scalar_effects().unwrap().into_iter().find(|e| e.expression == b"0.0 - ?damage").unwrap();
    let operation = ExecutableSourceOperationV1::ScalarEffect { handler: effect.handler, effect: effect.effect,
        field_path: effect.field_path, expression: b"0.0 - (?damage * 2.0)".to_vec() };
    let root = clause_package::ProgramChangeOccurrenceId::from_bytes([91;32]);
    let prepared = a.prepare_scalar_edit(initial.handle, 0, root, &operation).unwrap();
    let mut b = WasmPersistentSessionBoundaryV1::new();
    let other = b.open_prepared(&open, &capsule).unwrap();
    let before = b.current_accepted_projection_bytes(other.handle).unwrap();
    assert!(b.commit_scalar_edit(prepared).is_err());
    assert_eq!(b.current_accepted_projection_bytes(other.handle).unwrap(), before);
    let prepared = a.prepare_scalar_edit(initial.handle, 0, root, &operation).unwrap();
    a.prepare_source(initial.handle, 0, &capsule).unwrap();
    assert!(a.commit_scalar_edit(prepared).is_err(), "replaced preparation invalidates a captured token");
    let prepared = a.prepare_scalar_edit(initial.handle, 0, root, &operation).unwrap();
    let command = encode_wasm_session_command_v1(&WasmSessionCommandV1 { handle: initial.handle, expected_sequence: 0,
        operation: WasmSessionOperationV1::PhysicalInput(WasmSessionPhysicalInputV1 { input_sequence: 1,
            source: ExecutableInputSourceV1::Keyboard { code: b"BeginEncounter".to_vec(), phase: ExecutableKeyPhaseV1::Down }, value: None }) }).unwrap();
    a.command(&command).unwrap();
    assert_eq!(a.commit_scalar_edit(prepared).unwrap_err(), WasmProcessStatusV1::SequenceRejected);
}

#[test]
fn imported_declarations_survive_independent_preparation_and_live_scalar_edits() {
    let source = [b"import \"shared.clause\"\n".as_slice(), include_bytes!("../../../test-vectors/authoring/live-encounter.clause").as_slice(),
        b"\nexport local-label(): Text\n  shared-label()\n"].concat();
    let imports = clause_package::CanonicalSourceImportsV1::from([("shared.clause".into(), b"export shared-label(): Text\n  \"retained\"\n".to_vec())]);
    let mut producer = ResidentSourceWorkbenchV1::open_with_imports(&source, imports).unwrap();
    let expected = producer.invoke_callable(b"local-label", &[]).unwrap();
    let capsule = producer.source_preparation().unwrap();
    let mut consumer = WasmPersistentSessionBoundaryV1::new();
    let opened = consumer.open_prepared(&open_bytes(&producer.generation().cwr1), &capsule).unwrap();
    let mut substituted = capsule.clone();
    let offset = substituted.windows(8).position(|bytes| bytes == b"retained").unwrap();
    substituted[offset..offset + 8].copy_from_slice(b"tampered");
    assert!(WasmPersistentSessionBoundaryV1::new().open_prepared(&open_bytes(&producer.generation().cwr1), &substituted).is_err());
    let effect = producer.scalar_effects().unwrap().into_iter().find(|effect| effect.expression == b"0.0 - ?damage").unwrap();
    producer.edit_scalar_effect(producer.generation().handle, &effect, b"0.0 - (?damage * 2.0)").unwrap();
    let next = consumer.scalar_edit(opened.handle, 0, producer.last_source_edit().unwrap()).unwrap();
    assert_eq!(producer.invoke_callable(b"local-label", &[]).unwrap(), expected);
    let producer_before = producer.source_continuity_bytes().unwrap();
    let consumer_before = consumer.source_continuity_bytes(next.handle).unwrap();
    assert_complete_continuity_projection(&producer_before, &producer.source_continuity().unwrap());
    assert_complete_continuity_projection(&consumer_before, &consumer.source_continuity_term(next.handle).unwrap());
    while consumer.reclaim_retired() {}
    let effect = producer.scalar_effects().unwrap().into_iter().find(|effect| effect.expression == b"0.0 - (?damage * 2.0)").unwrap();
    producer.edit_scalar_effect(producer.generation().handle, &effect, b"0.0 - ?damage").unwrap();
    let next = consumer.scalar_edit(next.handle, 0, producer.last_source_edit().unwrap()).unwrap();
    assert_eq!(producer.invoke_callable(b"local-label", &[]).unwrap(), expected);
    let producer_after = producer.source_continuity_bytes().unwrap();
    let consumer_after = consumer.source_continuity_bytes(next.handle).unwrap();
    assert_complete_continuity_projection(&producer_after, &producer.source_continuity().unwrap());
    assert_complete_continuity_projection(&consumer_after, &consumer.source_continuity_term(next.handle).unwrap());
    assert_retained_occurrences(&producer_before, &producer_after);
    assert_retained_occurrences(&consumer_before, &consumer_after);
    let mappings = |bytes: &[u8]| {
        let count = u32::from_le_bytes(bytes[68..72].try_into().unwrap()) as usize;
        bytes[76..76 + count * 76].chunks_exact(76)
            .map(|entry| <[u8; 44]>::try_from(&entry[..44]).unwrap()).collect::<Vec<_>>()
    };
    assert!(mappings(&producer_after) == mappings(&consumer_after), "independent runtimes must agree on source address mappings");
}

fn assert_retained_occurrences(before: &[u8], after: &[u8]) {
    use std::collections::BTreeMap;
    let entries = |bytes: &[u8], coordinate_offset: usize| {
        assert_eq!(&bytes[..4], b"CSC1");
        let count = u32::from_le_bytes(bytes[68..72].try_into().unwrap()) as usize;
        let result = bytes[76..76 + count * 76].chunks_exact(76).map(|entry| {
            let coordinate = u32::from_le_bytes(entry[coordinate_offset..coordinate_offset + 4].try_into().unwrap());
            (coordinate, <[u8; 68]>::try_from(&entry[8..76]).unwrap())
        }).collect::<BTreeMap<_, _>>();
        assert_eq!(result.len(), count, "source coordinates must be unique");
        result
    };
    let previous = entries(before, 4);
    let continued = entries(after, 0);
    assert!(!previous.is_empty());
    assert_eq!(previous.len(), continued.len(), "scalar edits retain every occurrence");
    for (coordinate, identity) in previous {
        assert!(continued.get(&coordinate) == Some(&identity), "runtime occurrence changed at source coordinate {coordinate}");
    }
}

#[test]
fn callable_only_source_opens_without_invented_game_state_or_input() {
    let source = b"export value(): Bool\n  true\n";
    let workbench = ResidentSourceWorkbenchV1::open(source).unwrap();
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1).unwrap();
    assert!(plan.program.initial_configuration.is_empty());
    assert!(plan.input.is_none());
    assert!(plan.program.rules.iter().all(|rule| rule.predicates.is_empty() && rule.assignments.is_empty() && rule.removals.is_empty()));
    assert_eq!(workbench.invoke_callable(b"value", &[]).unwrap(), ExecutableValueV1::Boolean(true));
}

fn assert_complete_continuity_projection(bytes: &[u8], diagnostic: &clause_package::Term) {
    use clause_package::Term;
    use std::collections::BTreeMap;
    fn fields(term: &Term) -> BTreeMap<&[u8], &Term> {
        let mut result = BTreeMap::new();
        let mut current = term;
        while let Some(triple) = current.as_triple() {
            let [key, value, rest] = triple.slots();
            assert!(result.insert(key.as_atom().unwrap().canonical_payload(), value).is_none());
            current = rest;
        }
        result
    }
    fn index(term: &Term) -> BTreeMap<u32, &Term> {
        let coordinate = |key| std::str::from_utf8(key).unwrap().parse::<u32>().unwrap();
        fields(term).into_iter().flat_map(|(page, values)| {
            fields(values).into_iter().map(move |(key, value)| (coordinate(page) * 64 + coordinate(key), value))
        }).collect()
    }
    fn number(term: &Term) -> f64 {
        f64::from_le_bytes(term.as_atom().unwrap().canonical_payload().try_into().unwrap())
    }
    fn hex(bytes: &[u8]) -> Vec<u8> {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>().into_bytes()
    }
    let u32_at = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    let u16_at = |offset| u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
    assert_eq!(&bytes[..4], b"CSC1");
    let root = fields(diagnostic);
    assert_eq!(root.len(), 4);
    assert_eq!(root[b"old-snapshot".as_slice()].as_atom().unwrap().canonical_payload(), hex(&bytes[4..36]));
    assert_eq!(root[b"new-snapshot".as_slice()].as_atom().unwrap().canonical_payload(), hex(&bytes[36..68]));
    let occurrences = u32_at(68) as usize;
    let slots = u32_at(72) as usize;
    assert_eq!(bytes.len(), 76 + occurrences * 76 + slots * 4);
    let formations = index(root[b"formations".as_slice()]);
    assert_eq!(formations.len(), occurrences);
    for i in 0..occurrences {
        let offset = 76 + i * 76;
        let fields = fields(formations[&(i as u32)]);
        assert_eq!(fields.len(), 5);
        assert_eq!(number(fields[b"old".as_slice()]), f64::from(u32_at(offset)));
        assert_eq!(number(fields[b"new".as_slice()]), f64::from(u32_at(offset + 4)));
        assert_eq!(fields[b"occurrence-snapshot".as_slice()].as_atom().unwrap().canonical_payload(), hex(&bytes[offset + 8..offset + 40]));
        assert_eq!(number(fields[b"occurrence-coordinate".as_slice()]), f64::from(u32_at(offset + 40)));
        assert_eq!(fields[b"occurrence".as_slice()].as_atom().unwrap().canonical_payload(), hex(&bytes[offset + 44..offset + 76]));
    }
    let expected_slots = index(root[b"slots".as_slice()]);
    let actual_slots = (0..slots).map(|i| {
        let offset = 76 + occurrences * 76 + i * 4;
        (u32::from(u16_at(offset)), f64::from(u16_at(offset + 2)))
    }).collect::<BTreeMap<_, _>>();
    assert_eq!(actual_slots.len(), slots);
    assert_eq!(expected_slots.into_iter().map(|(slot, value)| (slot, number(value))).collect::<BTreeMap<_, _>>(), actual_slots);
}
