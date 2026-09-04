use clause_substrate::canonical_package::{
    AuthorizationError, Certificate, CertificateError, CertificateNode, CertificateReason,
    DecodeError, GroundRule, Lineage, TaggedConstruct, authorize, basis_admission_claim,
    check_certificate, decode, encode,
};

const BOOTSTRAP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/canonical-package/positive/bootstrap.hex"
));
const SUCCESSOR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/canonical-package/positive/successor.hex"
));

#[test]
fn positive_closure_counts_values_not_supports_and_cannot_sustain_itself() {
    use clause_substrate::canonical_package::{
        DerivationBasis, GroundClosureError, derive_ground_closure,
    };
    let decoded = decode(&hex(BOOTSTRAP)).unwrap();
    let p = decoded.value().target.clone();
    let mut q = p.clone();
    q.term = clause_substrate::canonical_package::Term::Triple(
        Box::new(p.term.clone()),
        Box::new(p.term.clone()),
        Box::new(p.term.clone()),
    );
    let mut basis = DerivationBasis {
        roots: vec![p.clone(), p.clone()],
        rules: vec![
            GroundRule {
                premises: vec![p.clone()],
                conclusion: q.clone(),
            },
            GroundRule {
                premises: vec![q.clone()],
                conclusion: p.clone(),
            },
        ],
    };
    for root_count in [2, 1, 0] {
        basis.roots.truncate(root_count);
        let closure = derive_ground_closure(&basis, 10).unwrap();
        assert_eq!(closure.nodes.len(), if root_count == 0 { 0 } else { 2 });
        for (index, node) in closure.nodes.iter().enumerate() {
            check_certificate(
                &basis,
                &Certificate {
                    nodes: closure.nodes[..=index].to_vec(),
                },
                &node.claimed,
            )
            .unwrap();
        }
    }
    assert_eq!(
        derive_ground_closure(&basis, 0),
        Err(GroundClosureError::Exhausted)
    );
    basis.roots.push(p.clone());
    basis.rules.reverse();
    let closure = derive_ground_closure(&basis, 10).unwrap();
    assert_eq!(closure.nodes.len(), 2);
    assert!(closure.nodes.iter().any(|node| node.claimed == q));
    assert!(closure.nodes.iter().any(|node| node.claimed == p));
}

fn vector(path: &str) -> Vec<u8> {
    let transport = match path {
        "bad-magic" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/bad-magic.hex"
        )),
        "bad-version" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/bad-version.hex"
        )),
        "bad-frame-order" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/bad-frame-order.hex"
        )),
        "unknown-term-tag" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/unknown-term-tag.hex"
        )),
        "bad-length" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/bad-length.hex"
        )),
        "truncated" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/truncated.hex"
        )),
        "trailing-bytes" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/trailing-bytes.hex"
        )),
        "malformed-predecessor" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/malformed-predecessor.hex"
        )),
        "decoded-field-tamper" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/decoded-field-tamper.hex"
        )),
        "epoch-tamper" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/epoch-tamper.hex"
        )),
        "basis-tamper" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/basis-tamper.hex"
        )),
        "certificate-tamper" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/certificate-tamper.hex"
        )),
        "target-tamper" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/target-tamper.hex"
        )),
        "auxiliary-tamper" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/auxiliary-tamper.hex"
        )),
        "nonliteral-root" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/nonliteral-root.hex"
        )),
        "nullary-self-rule" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/nullary-self-rule.hex"
        )),
        "self-authorization" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/self-authorization.hex"
        )),
        "wrong-transplanted-predecessor" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/wrong-transplanted-predecessor.hex"
        )),
        "check-auth-under-successor-basis" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/canonical-package/negative/check-auth-under-successor-basis.hex"
        )),
        other => panic!("unknown test vector {other}"),
    };
    hex(transport)
}

fn hex(transport: &str) -> Vec<u8> {
    let digits: Vec<u8> = transport
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert_eq!(digits.len() % 2, 0);
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("hex digit") as u8;
            let low = (pair[1] as char).to_digit(16).expect("hex digit") as u8;
            (high << 4) | low
        })
        .collect()
}

#[test]
fn frozen_bootstrap_decodes_reencodes_checks_and_authorizes() {
    let bytes = hex(BOOTSTRAP);
    assert_eq!(bytes.len(), 334);
    let bootstrap = decode(&bytes).expect("frozen bootstrap decodes");
    assert_eq!(bootstrap.exact_bytes(), bytes);
    assert_eq!(bootstrap.exact_index_frame(), &bytes[5..20]);
    assert_eq!(bootstrap.exact_basis_frame(), &bytes[26..210]);
    assert_eq!(bootstrap.value().index.universe_id.0, [0x10]);
    assert_eq!(bootstrap.value().index.semantics_id.0, [0x11]);
    assert!(matches!(bootstrap.value().lineage, Lineage::Root));
    assert_eq!(bootstrap.value().basis.roots.len(), 2);
    assert!(bootstrap.value().basis.rules.is_empty());
    assert!(bootstrap.value().auxiliary.is_empty());
    assert_eq!(encode(bootstrap.value()).expect("encodes"), bytes);
    check_certificate(
        &bootstrap.value().basis,
        &bootstrap.value().certificate,
        &bootstrap.value().target,
    )
    .expect("bootstrap package certificate checks");
    assert_eq!(
        authorize(&bootstrap)
            .expect("literal bootstrap authorizes")
            .package(),
        &bootstrap
    );
}

#[test]
fn frozen_successor_uses_only_the_authorized_predecessor_basis() {
    let bootstrap = decode(&hex(BOOTSTRAP)).expect("bootstrap decodes");
    let bytes = hex(SUCCESSOR);
    assert_eq!(bytes.len(), 681);
    let successor = decode(&bytes).expect("successor decodes");
    assert_eq!(successor.exact_index_frame(), &bytes[5..20]);
    assert_eq!(successor.exact_basis_frame(), &bytes[496..557]);
    assert_eq!(encode(successor.value()).expect("encodes"), bytes);

    let Lineage::Successor {
        predecessor_package,
        authorization,
    } = &successor.value().lineage
    else {
        panic!("successor lineage expected")
    };
    assert_eq!(predecessor_package.0, bootstrap.exact_bytes());
    let admission = basis_admission_claim(&successor);
    assert_eq!(bootstrap.value().basis.roots[1], admission);
    check_certificate(&bootstrap.value().basis, authorization, &admission)
        .expect("predecessor basis authorizes the exact successor basis claim");
    assert_eq!(
        check_certificate(&successor.value().basis, authorization, &admission),
        Err(CertificateError::RootOutOfBounds {
            node: 0,
            root_ref: 1,
        })
    );
    assert_eq!(
        authorize(&successor)
            .expect("successor authorizes")
            .package(),
        &successor
    );
}

#[test]
fn malformed_envelopes_and_tags_are_rejected_precisely() {
    assert!(matches!(
        decode(&vector("bad-magic")),
        Err(DecodeError::WrongMagic { .. })
    ));
    assert_eq!(
        decode(&vector("bad-version")),
        Err(DecodeError::UnsupportedVersion {
            offset: 4,
            found: 2,
        })
    );
    assert!(matches!(
        decode(&vector("bad-frame-order")),
        Err(DecodeError::UnexpectedFrameTag {
            offset: 5,
            expected: 1,
            found: 2,
        })
    ));
    assert!(matches!(
        decode(&vector("unknown-term-tag")),
        Err(DecodeError::UnknownTag {
            construct: TaggedConstruct::Term,
            found: 2,
            ..
        })
    ));
    assert!(matches!(
        decode(&vector("bad-length")),
        Err(DecodeError::UnexpectedEof { .. })
    ));
    assert!(matches!(
        decode(&vector("truncated")),
        Err(DecodeError::UnexpectedEof { .. })
    ));
    assert!(matches!(
        decode(&vector("trailing-bytes")),
        Err(DecodeError::TrailingBytes {
            offset: 681,
            remaining: 1,
        })
    ));
    assert!(matches!(
        decode(&vector("malformed-predecessor")),
        Err(DecodeError::InvalidPredecessorPackage {
            offset: 30,
            error,
        }) if matches!(*error, DecodeError::WrongMagic { .. })
    ));

    let mut successor = hex(SUCCESSOR);
    successor[25] = 2;
    assert!(matches!(
        decode(&successor),
        Err(DecodeError::UnknownTag {
            offset: 25,
            construct: TaggedConstruct::Lineage,
            found: 2,
        })
    ));

    let mut successor = hex(SUCCESSOR);
    successor[614] = 2;
    assert!(matches!(
        decode(&successor),
        Err(DecodeError::UnknownTag {
            offset: 614,
            construct: TaggedConstruct::CertificateReason,
            found: 2,
        })
    ));

    let mut successor = hex(SUCCESSOR);
    successor[30] = b'D';
    assert!(matches!(
        decode(&successor),
        Err(DecodeError::InvalidPredecessorPackage {
            offset: 30,
            error,
        }) if matches!(*error, DecodeError::WrongMagic { .. })
    ));

    let mut bootstrap = hex(BOOTSTRAP);
    bootstrap[22..25].copy_from_slice(&[0, 0, 2]);
    assert!(matches!(
        decode(&bootstrap),
        Err(DecodeError::UnderconsumedFrame {
            tag: 2,
            remaining: 1,
            ..
        })
    ));
}

#[test]
fn tamper_vectors_separate_decoding_binding_certificates_and_authority() {
    let positive = decode(&hex(SUCCESSOR)).expect("successor decodes");

    let universe = decode(&vector("decoded-field-tamper")).expect("tamper decodes");
    assert!(!universe.is_exactly(&positive));
    assert_eq!(
        authorize(&universe).expect_err("universe mismatch rejects"),
        AuthorizationError::UniverseMismatch
    );

    let epoch = decode(&vector("epoch-tamper")).expect("tamper decodes");
    assert!(!epoch.is_exactly(&positive));
    assert_eq!(
        authorize(&epoch).expect_err("semantics mismatch rejects"),
        AuthorizationError::SemanticsMismatch
    );

    let basis = decode(&vector("basis-tamper")).expect("tamper decodes");
    assert!(!basis.is_exactly(&positive));
    assert_eq!(
        authorize(&basis).expect_err("basis tamper rejects"),
        AuthorizationError::LineageCertificate(CertificateError::TargetMismatch)
    );

    let certificate = decode(&vector("certificate-tamper")).expect("tamper decodes");
    assert_eq!(
        authorize(&certificate).expect_err("certificate tamper rejects"),
        AuthorizationError::PackageCertificate(CertificateError::RootOutOfBounds {
            node: 0,
            root_ref: 1,
        })
    );

    let target = decode(&vector("target-tamper")).expect("tamper decodes");
    assert_eq!(
        authorize(&target).expect_err("target tamper rejects"),
        AuthorizationError::PackageCertificate(CertificateError::TargetMismatch)
    );

    let auxiliary = decode(&vector("auxiliary-tamper")).expect("tamper decodes");
    assert!(!auxiliary.is_exactly(&positive));
    assert_eq!(auxiliary.value().auxiliary.len(), 1);
    authorize(&auxiliary).expect("opaque auxiliary bytes do not grant or remove v0 authority");
}

#[test]
fn self_root_self_authorization_and_transplanted_predecessor_do_not_authorize() {
    let nonliteral = decode(&vector("nonliteral-root")).expect("candidate decodes");
    check_certificate(
        &nonliteral.value().basis,
        &nonliteral.value().certificate,
        &nonliteral.value().target,
    )
    .expect("relative package certificate still checks");
    assert_eq!(
        authorize(&nonliteral).expect_err("nonliteral root rejects"),
        AuthorizationError::RootIsNotLiteralBootstrap
    );

    let self_authorized = decode(&vector("self-authorization")).expect("candidate decodes");
    let Lineage::Successor { authorization, .. } = &self_authorized.value().lineage else {
        panic!("successor lineage expected")
    };
    let self_claim = &authorization
        .nodes
        .last()
        .expect("self-authorization certificate is nonempty")
        .claimed;
    check_certificate(&self_authorized.value().basis, authorization, self_claim)
        .expect("self claim checks only under the unauthorized successor basis");
    assert!(matches!(
        authorize(&self_authorized),
        Err(AuthorizationError::LineageCertificate(_))
    ));

    let transplant = decode(&vector("wrong-transplanted-predecessor")).expect("candidate decodes");
    assert_eq!(
        authorize(&transplant).expect_err("transplanted predecessor rejects"),
        AuthorizationError::PredecessorUnauthorized(Box::new(
            AuthorizationError::RootIsNotLiteralBootstrap
        ))
    );
}

#[test]
fn nullary_relative_self_rule_cannot_create_root_authority() {
    let bootstrap = decode(&hex(BOOTSTRAP)).expect("bootstrap decodes");
    let mut candidate_value = bootstrap.value().clone();
    candidate_value.basis.roots.clear();
    candidate_value.basis.rules = vec![GroundRule {
        premises: Vec::new(),
        conclusion: candidate_value.target.clone(),
    }];
    candidate_value.certificate = Certificate {
        nodes: vec![CertificateNode {
            claimed: candidate_value.target.clone(),
            reason: CertificateReason::Apply {
                rule_ref: 0,
                premise_refs: Vec::new(),
            },
        }],
    };
    let bytes = encode(&candidate_value).expect("candidate encodes");
    let candidate = decode(&bytes).expect("candidate decodes");
    check_certificate(
        &candidate.value().basis,
        &candidate.value().certificate,
        &candidate.value().target,
    )
    .expect("nullary rule derives only relative to its candidate basis");
    assert_eq!(
        authorize(&candidate).expect_err("candidate root rejects"),
        AuthorizationError::RootIsNotLiteralBootstrap
    );
}

#[test]
fn frozen_nullary_successor_vector_uses_no_successor_authority() {
    let bytes = vector("nullary-self-rule");
    assert_eq!(bytes.len(), 662);
    let candidate = decode(&bytes).expect("nullary successor vector decodes");
    assert_eq!(encode(candidate.value()).expect("vector re-encodes"), bytes);

    let Lineage::Successor { authorization, .. } = &candidate.value().lineage else {
        panic!("successor lineage expected")
    };
    let self_claim = &authorization
        .nodes
        .last()
        .expect("nullary authorization is nonempty")
        .claimed;
    check_certificate(&candidate.value().basis, authorization, self_claim)
        .expect("nullary rule derives only under the successor basis");
    assert!(matches!(
        authorize(&candidate),
        Err(AuthorizationError::LineageCertificate(_))
    ));
}

#[test]
fn identical_wrong_basis_vector_cannot_change_the_selected_basis() {
    let bytes = vector("check-auth-under-successor-basis");
    assert_eq!(bytes, hex(SUCCESSOR));
    let successor = decode(&bytes).expect("vector decodes");
    let Lineage::Successor { authorization, .. } = &successor.value().lineage else {
        panic!("successor lineage expected")
    };
    assert!(
        check_certificate(
            &successor.value().basis,
            authorization,
            &basis_admission_claim(&successor)
        )
        .is_err()
    );
    authorize(&successor).expect("the normative predecessor-basis path authorizes");
}
