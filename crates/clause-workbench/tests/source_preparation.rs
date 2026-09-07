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
        let request = open_bytes(&producer.generation().cwr1);
        let mut tampered = decode_executable_source_edit_v1(witness).unwrap();
        tampered.old_source.push(b' ');
        assert!(consumer.open_source_edit(current.handle, 0, &request, &encode_executable_source_edit_v1(&tampered).unwrap()).is_err());
        let next = consumer.open_source_edit(current.handle, 0, &request, witness).unwrap();
        assert_eq!(consumer.prepare_source(current.handle, 0, &capsule), Err(WasmProcessStatusV1::StaleSessionHandle));
        assert!(consumer.prepare_source(next.handle, 0, &capsule).is_err());
        assert!(consumer.source_continuity_bytes(next.handle).is_ok());
        while consumer.reclaim_retired() {}
        current = next;
    }
}
