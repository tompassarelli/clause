use clause_substrate::compiler_package_v3::{Term, decode_canonical_term};
use clause_workbench::{
    BASE_SOURCE, CHANGED_SOURCE, INVALID_SOURCE, WorkbenchService, encode_request, framed,
    response_payload, split_frames,
};

fn request(operation: &[u8], base: &[u8], source: &[u8]) -> Vec<u8> {
    encode_request(operation, base, source).expect("request is one bounded canonical Term")
}

fn text(frame: &[u8]) -> String {
    String::from_utf8(response_payload(frame).expect("response is package-owned"))
        .expect("fixture responses are UTF-8")
}

fn state_token(frame: &[u8]) -> Vec<u8> {
    let term = decode_canonical_term(frame).expect("response decodes");
    let Term::Triple(_, state, _) = term else {
        panic!("response is a transaction triple");
    };
    let Term::Atom {
        kind,
        canonical_payload,
        ..
    } = state.into_inner()
    else {
        panic!("next state is one opaque Atom");
    };
    assert_eq!(kind, b"clause/workbench-state/v1");
    canonical_payload
}

#[test]
fn one_long_lived_stdio_process_executes_the_complete_workbench_vocabulary() {
    let requests = vec![
        request(b"parse", b"", BASE_SOURCE),
        request(b"check", b"", INVALID_SOURCE),
        request(b"explain", b"", BASE_SOURCE),
        request(b"query", b"", b""),
        request(b"diff", b"", CHANGED_SOURCE),
        request(b"run", b"", b""),
        request(b"propose", b"revision-1", CHANGED_SOURCE),
        request(b"query", b"", b""),
        request(b"admit", b"", b""),
        request(b"hotReload", b"revision-1", CHANGED_SOURCE),
        request(b"hotReload", b"revision-2", CHANGED_SOURCE),
        request(b"run", b"", b""),
    ];
    let mut output = Vec::new();
    let mut service = WorkbenchService::open().expect("exact CLCP03 package is accepted once");
    let accepted_package = service.package_hash();
    let initial_carrier = service
        .carrier_snapshot()
        .expect("one real runtime carrier is open");
    let mut carrier_snapshots = Vec::new();
    for request in &requests {
        let input = framed(std::slice::from_ref(request)).expect("transcript frame is bounded");
        service
            .serve(input.as_slice(), &mut output)
            .expect("one service processes the entire ordered transcript");
        carrier_snapshots.push(
            service
                .carrier_snapshot()
                .expect("the same runtime carrier remains live"),
        );
    }
    let responses = split_frames(&output).expect("all response frames are exact");
    assert_eq!(responses.len(), requests.len());
    assert_eq!(service.request_count(), 12);
    assert_eq!(service.package_hash(), accepted_package);
    assert_eq!(initial_carrier.candidate_delta_count, 0);
    assert_eq!(initial_carrier.decision_count, 0);
    assert_eq!(initial_carrier.state_revision_count, 1);
    assert!(
        carrier_snapshots[..6]
            .iter()
            .all(|snapshot| *snapshot == initial_carrier)
    );

    assert!(text(&responses[0]).contains("reading=singleton-field+relation-schema+select-one"));
    let diagnostic = text(&responses[1]);
    assert!(diagnostic.contains("diagnostic=clause/syntax/removed-defined-as-v1"));
    assert!(diagnostic.contains("stage=formation"));
    assert!(diagnostic.contains("failedFormation=answer"));
    assert!(diagnostic.contains("origin=24..26"));
    assert!(diagnostic.contains("obligation=replace-:=with-:"));
    assert!(diagnostic.contains("dependencies=source:answer"));
    assert!(diagnostic.contains("boundariesUnchanged=program,state"));

    assert!(text(&responses[2]).contains("ApplicationForm->Application->Activation->42"));
    let initial_query = text(&responses[3]);
    assert!(initial_query.contains("binding.after=42"));
    assert!(initial_query.contains("why=successor(41,after)"));
    assert!(initial_query.contains("prevent=withdraw-successor-row"));
    assert!(initial_query.contains("achieve=include-successor-row"));
    assert!(
        text(&responses[4])
            .contains("affected=constraint:answer,row:successor,request-result,run-result")
    );

    let first_run = text(&responses[5]);
    assert!(first_run.contains("value=42"));
    assert!(first_run.contains("stateRevision=1"));
    assert!(first_run.contains("stateRevisionCreated=false"));
    assert_eq!(state_token(&responses[5]), b"base-1");

    let proposed = text(&responses[6]);
    assert!(proposed.contains("status=candidate"));
    assert!(proposed.contains("hidden=true"));
    assert!(proposed.contains("stateRevision=1"));
    assert_eq!(state_token(&responses[6]), b"candidate-2");
    let candidate_carrier = carrier_snapshots[6];
    assert_eq!(candidate_carrier.candidate_delta_count, 1);
    assert_eq!(candidate_carrier.decision_count, 0);
    assert_eq!(candidate_carrier.state_revision_count, 1);
    assert_eq!(candidate_carrier.world_base, initial_carrier.world_base);
    assert_eq!(candidate_carrier.run, initial_carrier.run);
    assert_eq!(candidate_carrier.activation, initial_carrier.activation);

    let hidden_query = text(&responses[7]);
    assert!(hidden_query.contains("binding.after=42"));
    assert!(hidden_query.contains("stateRevision=1"));
    assert_eq!(state_token(&responses[7]), b"candidate-2");
    assert_eq!(carrier_snapshots[7], candidate_carrier);

    let admitted = text(&responses[8]);
    assert!(admitted.contains("status=admitted"));
    assert!(admitted.contains("stateRevision=2"));
    assert_eq!(state_token(&responses[8]), b"admitted-2");
    let admitted_carrier = carrier_snapshots[8];
    assert_eq!(admitted_carrier.candidate_delta_count, 1);
    assert_eq!(admitted_carrier.decision_count, 1);
    assert_eq!(admitted_carrier.state_revision_count, 2);
    assert_ne!(admitted_carrier.world_base, initial_carrier.world_base);
    assert_ne!(admitted_carrier.run, initial_carrier.run);
    assert_ne!(admitted_carrier.activation, initial_carrier.activation);

    let stale_reload = text(&responses[9]);
    assert!(stale_reload.contains("diagnostic=clause/workbench/stale-base-v1"));
    assert!(stale_reload.contains("partialChange=false"));
    assert_eq!(state_token(&responses[9]), b"admitted-2");
    assert_eq!(carrier_snapshots[9], admitted_carrier);

    let reloaded = text(&responses[10]);
    assert!(reloaded.contains("status=reloaded"));
    assert!(reloaded.contains("oldActivationRetired=true"));
    assert_eq!(state_token(&responses[10]), b"reloaded-2");
    assert_eq!(carrier_snapshots[10], admitted_carrier);

    let changed_run = text(&responses[11]);
    assert!(changed_run.contains("value=43"));
    assert!(changed_run.contains("stateRevision=2"));
    assert!(changed_run.contains("stateRevisionCreated=false"));
    assert_eq!(state_token(&responses[11]), b"reloaded-2");
    assert_eq!(carrier_snapshots[11], admitted_carrier);
}
