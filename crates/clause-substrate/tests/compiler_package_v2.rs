use clause_substrate::compiler_package_v2::{
    CompilerEvidence, CompilerInterface, CompilerLineage, CompilerPackage, CompilerSubject,
    CoreManifest, DecodeCode, DecodeFailure, Definition, EncodeError, Hash32, Id32, KExpr, KSort,
    KValue, Term, decode, encode, exact_core_manifest_bytes, exact_physical_profile_bytes,
};
use clause_substrate::evaluator::{CertificateContext, Evaluator};

fn id(value: u8) -> Id32 {
    Id32([value; 32])
}

fn atom(payload: &[u8]) -> Term {
    Term::Atom {
        kind: b"opaque-kind".to_vec(),
        canonical_payload: payload.to_vec(),
        equality_contract: b"opaque-equality".to_vec(),
    }
}

fn sample_package() -> CompilerPackage {
    CompilerPackage {
        core_manifest: CoreManifest::canonical_v1(),
        subject: CompilerSubject {
            lineage: CompilerLineage::Genesis,
            nominal_declarations: Vec::new(),
            interface: CompilerInterface {
                compile: id(1),
                admit_propose: id(2),
            },
            program: vec![
                Definition {
                    id: id(1),
                    arguments: vec![KSort::Term],
                    result: KSort::Term,
                    body: KExpr::Var(0),
                },
                Definition {
                    id: id(2),
                    arguments: vec![KSort::Term],
                    result: KSort::Term,
                    body: KExpr::Var(0),
                },
            ],
            build_request: atom(b"opaque-build-request"),
        },
        evidence: CompilerEvidence::Genesis,
    }
}

fn rejection(bytes: &[u8]) -> (DecodeCode, u64) {
    match decode(bytes).expect_err("candidate must reject") {
        DecodeFailure::Rejected(rejection) => (rejection.code, rejection.offset),
        DecodeFailure::ResourceExhausted => panic!("fixture must not exhaust resources"),
    }
}

fn frame_one_payload_length(bytes: &[u8]) -> usize {
    u32::from_be_bytes(bytes[6..10].try_into().expect("frame length")) as usize
}

fn nested_concat(depth: usize) -> KExpr {
    (0..depth).fold(KExpr::BytesLiteral(Vec::new()), |expression, _| {
        KExpr::ConcatBytes(vec![expression])
    })
}

#[test]
fn canonical_v2_candidate_round_trips_and_retains_exact_frame_payloads() {
    let package = sample_package();
    let bytes = encode(&package).expect("sample encodes");
    let candidate = decode(&bytes).expect("sample decodes");
    assert_eq!(candidate.exact_input(), bytes);
    assert_eq!(candidate.package(), &package);
    assert_eq!(encode(candidate.package()).expect("re-encodes"), bytes);
    assert_eq!(
        candidate.exact_core_manifest(),
        exact_core_manifest_bytes().expect("manifest encodes")
    );
    assert_eq!(
        candidate.package().core_manifest.physical_profile,
        CoreManifest::canonical_v1().physical_profile
    );
    assert!(
        !exact_physical_profile_bytes()
            .expect("profile encodes")
            .is_empty()
    );
    assert!(matches!(
        candidate.package().evidence,
        CompilerEvidence::Genesis
    ));
}

#[test]
fn version_frame_sum_and_eof_failures_have_separate_canonical_codes() {
    let bytes = encode(&sample_package()).expect("sample encodes");

    let mut wrong_version = bytes.clone();
    wrong_version[4] = 0x01;
    assert_eq!(rejection(&wrong_version), (DecodeCode::UnknownVersion, 4));

    let mut wrong_frame = bytes.clone();
    wrong_frame[5] = 0x02;
    assert_eq!(
        rejection(&wrong_frame),
        (DecodeCode::FrameTagOrderOrCount, 5)
    );

    let manifest_length = frame_one_payload_length(&bytes);
    let subject_tag = 10 + manifest_length;
    let subject_payload = subject_tag + 5;
    let mut unknown_lineage = bytes.clone();
    unknown_lineage[subject_payload] = 0x02;
    assert_eq!(
        rejection(&unknown_lineage),
        (DecodeCode::UnknownSumTag, subject_payload as u64)
    );

    let mut trailing = bytes.clone();
    trailing.push(0xff);
    assert_eq!(
        rejection(&trailing),
        (DecodeCode::TrailingBytes, bytes.len() as u64)
    );

    assert_eq!(
        rejection(&bytes[..bytes.len() - 1]),
        (DecodeCode::Truncated, (bytes.len() - 1) as u64)
    );
}

#[test]
fn bounded_frames_report_under_and_over_consumption_without_fallback() {
    let bytes = encode(&sample_package()).expect("sample encodes");
    let length = frame_one_payload_length(&bytes);

    let mut under = bytes.clone();
    under[6..10].copy_from_slice(&u32::try_from(length + 1).unwrap().to_be_bytes());
    assert_eq!(
        rejection(&under),
        (
            DecodeCode::BoundedValueUnderConsumed,
            u64::try_from(10 + length + 1).unwrap()
        )
    );

    let mut over = bytes;
    over[6..10].copy_from_slice(&u32::try_from(length - 1).unwrap().to_be_bytes());
    assert_eq!(
        rejection(&over),
        (
            DecodeCode::BoundedValueOverConsumed,
            u64::try_from(10 + length - 1).unwrap()
        )
    );
}

#[test]
fn counted_sequence_exhaustion_uses_the_next_depth_first_read_verdict() {
    let bytes = encode(&sample_package()).expect("sample encodes");

    let mut bounded = bytes.clone();
    bounded[6..10].copy_from_slice(&5_u32.to_be_bytes());
    bounded[11..15].copy_from_slice(&1_u32.to_be_bytes());
    assert_eq!(
        rejection(&bounded),
        (DecodeCode::BoundedValueOverConsumed, 15)
    );

    let mut truncated = bytes[..15].to_vec();
    truncated[6..10].copy_from_slice(&5_u32.to_be_bytes());
    truncated[11..15].copy_from_slice(&1_u32.to_be_bytes());
    assert_eq!(rejection(&truncated), (DecodeCode::Truncated, 15));
}

#[test]
fn recursive_wire_values_fail_with_typed_resource_exhaustion() {
    let mut too_deep = sample_package();
    too_deep.subject.program[0].body = nested_concat(32);
    assert_eq!(encode(&too_deep), Err(EncodeError::ResourceExhausted));

    let mut at_limit = sample_package();
    at_limit.subject.program[0].body = nested_concat(31);
    let mut bytes = encode(&at_limit).expect("depth-limited expression encodes");
    assert!(decode(&bytes).is_ok(), "depth-limited expression decodes");

    let manifest_length = frame_one_payload_length(&bytes);
    let subject_tag = 10 + manifest_length;
    let subject_length = u32::from_be_bytes(
        bytes[subject_tag + 1..subject_tag + 5]
            .try_into()
            .expect("subject frame length"),
    );
    let first_definition_body = subject_tag + 5 + 111;
    let wire_wrapper = [0x08, 0x00, 0x00, 0x00, 0x01];
    assert_eq!(
        &bytes[first_definition_body..first_definition_body + wire_wrapper.len()],
        &wire_wrapper
    );
    bytes.splice(first_definition_body..first_definition_body, wire_wrapper);
    bytes[subject_tag + 1..subject_tag + 5].copy_from_slice(&(subject_length + 5).to_be_bytes());

    assert_eq!(decode(&bytes), Err(DecodeFailure::ResourceExhausted));
}

#[test]
fn successor_evidence_round_trips_as_inert_certificate_data() {
    let mut package = sample_package();
    package.subject.lineage = CompilerLineage::Successor {
        predecessor_locator: Hash32([9; 32]),
        change_occurrence_id: id(9),
    };
    let evaluator = Evaluator::new(&package.subject.program).expect("program checks generically");
    let certificate = evaluator
        .build_certificate(CertificateContext {
            exact_accepted_predecessor: b"accepted predecessor bytes".to_vec(),
            core_contract_id: Hash32([3; 32]),
            physical_profile_id: Hash32([4; 32]),
            entrypoint: id(1),
            arguments: vec![KValue::Term(atom(b"argument"))],
            fuel_limit: 8,
        })
        .expect("generic certificate builds");
    package.evidence = CompilerEvidence::Successor {
        compile_certificate: Box::new(certificate.clone()),
        admission_certificate: Box::new(certificate),
    };

    let bytes = encode(&package).expect("successor evidence encodes");
    let candidate = decode(&bytes).expect("successor evidence decodes");
    assert_eq!(candidate.package(), &package);
    assert_eq!(encode(candidate.package()).unwrap(), bytes);

    let CompilerEvidence::Successor {
        compile_certificate,
        ..
    } = &mut package.evidence
    else {
        panic!("successor evidence expected")
    };
    compile_certificate.format_version = 1;
    assert_eq!(
        encode(&package),
        Err(EncodeError::InvalidClosedTag {
            field: "certificate format version",
            tag: 1,
        })
    );
}
