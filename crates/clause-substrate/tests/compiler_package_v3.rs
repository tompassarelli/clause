use clause_substrate::compiler_package_v3::{
    CompilerEvidence, CompilerInterface, CompilerLineage, CompilerPackage, CompilerSubject,
    CoreManifest, DecodeCode, DecodeFailure, Definition, EncodeError, EvalReceipt, FallibleBox,
    Hash32, Id32, KExpr, KSort, Term, core_contract_id, decode, encode, exact_core_manifest_bytes,
    exact_physical_profile_bytes,
};
use sha2::{Digest, Sha256};

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

fn boxed<T>(value: T) -> FallibleBox<T> {
    FallibleBox::try_new(value).expect("test value allocation")
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

fn nested_term(depth: usize) -> Term {
    assert!(depth > 0);
    (1..depth).fold(atom(b"leaf"), |term, index| {
        Term::Triple(
            boxed(atom(&[u8::try_from(index % 251).unwrap()])),
            boxed(atom(b"sibling")),
            boxed(term),
        )
    })
}

fn hexadecimal(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[test]
fn canonical_manifest_matches_the_fixed_lean_authority() {
    let manifest = CoreManifest::canonical_v1();
    let bytes = exact_core_manifest_bytes().expect("fixed manifest encodes");

    assert_eq!(manifest.contract_clauses.len(), 35);
    assert_eq!(bytes.len(), 19_601);
    assert_eq!(
        hexadecimal(&Sha256::digest(&bytes)),
        "88b804c3c67f58fad7a823ca92a2beed829e75404334c625e0b0659f5187b09c"
    );
    assert_eq!(
        hexadecimal(
            core_contract_id()
                .expect("fixed CoreContractId derives")
                .as_bytes()
        ),
        "d5289138ebff540f6acd51f707f557509666c19042b88164794183fbf940b1f0"
    );
    assert_eq!(manifest.receipt_signature.len(), 364);
    assert_eq!(
        hexadecimal(&Sha256::digest(&manifest.receipt_signature)),
        "c8186e543228bc5c4494ea4b1ca5c1b227f0012e593de9249f15c0a610a4c918"
    );
}

#[test]
fn aggregate_wire_item_budget_is_shared_across_frames() {
    const HALF_WIRE_ITEM_LIMIT: usize = 262_144 / 2;

    let mut manifest_only = sample_package();
    manifest_only.core_manifest.contract_clauses = vec![Vec::new(); HALF_WIRE_ITEM_LIMIT];
    assert!(
        encode(&manifest_only).is_ok(),
        "manifest frame remains below every package budget"
    );

    let mut subject_only = sample_package();
    subject_only.subject.program[0].arguments = vec![KSort::Term; HALF_WIRE_ITEM_LIMIT];
    assert!(
        encode(&subject_only).is_ok(),
        "subject frame remains below every package budget"
    );

    let mut package = sample_package();
    package.core_manifest.contract_clauses = vec![Vec::new(); HALF_WIRE_ITEM_LIMIT];
    package.subject.program[0].arguments = vec![KSort::Term; HALF_WIRE_ITEM_LIMIT];

    assert_eq!(encode(&package), Err(EncodeError::ResourceExhausted));
}

#[test]
fn canonical_v3_candidate_round_trips_and_retains_exact_frame_payloads() {
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
fn independent_term_and_expression_depth_limits_match_the_wire_contract() {
    let mut too_deep = sample_package();
    too_deep.subject.program[0].body = nested_concat(512);
    assert_eq!(encode(&too_deep), Err(EncodeError::ResourceExhausted));

    let mut at_limit = sample_package();
    at_limit.subject.program[0].body = nested_concat(511);
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

    let mut term_at_limit = sample_package();
    term_at_limit.subject.build_request = nested_term(128);
    let bytes = encode(&term_at_limit).expect("128-level Term encodes");
    assert!(decode(&bytes).is_ok(), "128-level Term decodes");

    let mut term_too_deep = sample_package();
    term_too_deep.subject.build_request = nested_term(129);
    assert_eq!(encode(&term_too_deep), Err(EncodeError::ResourceExhausted));
}

#[test]
fn successor_evidence_is_exactly_two_fixed_size_inert_receipts() {
    let mut package = sample_package();
    package.subject.lineage = CompilerLineage::Successor {
        predecessor_locator: Hash32([9; 32]),
        change_occurrence_id: id(9),
    };
    let compile_receipt = EvalReceipt {
        format_version: 0x00,
        expected_value_hash: Hash32([0x11; 32]),
        expected_remaining_fuel: 0x0102_0304_0506_0708,
        expected_observations_hash: Hash32([0x22; 32]),
    };
    let admission_receipt = EvalReceipt {
        format_version: 0x00,
        expected_value_hash: Hash32([0x33; 32]),
        expected_remaining_fuel: 0x1112_1314_1516_1718,
        expected_observations_hash: Hash32([0x44; 32]),
    };
    package.evidence = CompilerEvidence::Successor {
        compile_receipt,
        admission_receipt,
    };

    let bytes = encode(&package).expect("successor evidence encodes");
    let candidate = decode(&bytes).expect("successor evidence decodes");
    assert_eq!(candidate.package(), &package);
    assert_eq!(encode(candidate.package()).unwrap(), bytes);
    assert_eq!(candidate.exact_evidence().len(), 147);

    let mut expected_evidence = Vec::with_capacity(147);
    expected_evidence.push(0x01);
    for receipt in [compile_receipt, admission_receipt] {
        expected_evidence.push(0x00);
        expected_evidence.extend_from_slice(receipt.expected_value_hash.as_bytes());
        expected_evidence.extend_from_slice(&receipt.expected_remaining_fuel.to_be_bytes());
        expected_evidence.extend_from_slice(receipt.expected_observations_hash.as_bytes());
    }
    assert_eq!(expected_evidence.len(), 147);
    assert_eq!(candidate.exact_evidence(), expected_evidence);

    let evidence_offset = bytes
        .len()
        .checked_sub(147)
        .expect("successor evidence payload is present");
    let compile_format_offset = evidence_offset + 1;
    let mut unknown_receipt_format = bytes.clone();
    unknown_receipt_format[compile_format_offset] = 0x01;
    assert_eq!(
        rejection(&unknown_receipt_format),
        (
            DecodeCode::UnknownSumTag,
            u64::try_from(compile_format_offset).unwrap()
        )
    );

    let CompilerEvidence::Successor {
        compile_receipt, ..
    } = &mut package.evidence
    else {
        panic!("successor evidence expected")
    };
    compile_receipt.format_version = 1;
    assert_eq!(
        encode(&package),
        Err(EncodeError::InvalidClosedTag {
            field: "receipt format version",
            tag: 1,
        })
    );
}
