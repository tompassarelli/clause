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
        assert!(consumer.source_continuity_bytes(next.handle).is_ok());
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
