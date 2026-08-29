use clause_substrate::compiler_package_v2::{
    Definition, Hash32, Id32, KExpr, KSort, KValue, Term, sha256_operation_id,
};
use clause_substrate::evaluator::{CertificateContext, Evaluator, StaticError};

fn id(value: u8) -> Id32 {
    Id32([value; 32])
}

fn atom(kind: &[u8], payload: &[u8], equality: &[u8]) -> Term {
    Term::Atom {
        kind: kind.to_vec(),
        canonical_payload: payload.to_vec(),
        equality_contract: equality.to_vec(),
    }
}

fn definitions() -> Vec<Definition> {
    vec![
        Definition {
            id: id(1),
            arguments: vec![KSort::Bytes],
            result: KSort::Bytes,
            body: KExpr::Var(0),
        },
        Definition {
            id: id(2),
            arguments: vec![KSort::Bytes],
            result: KSort::Bytes,
            body: KExpr::Request {
                physical_operation_id: sha256_operation_id(),
                arguments: vec![KExpr::Var(0)],
            },
        },
    ]
}

fn evaluate(expression: KExpr) -> KValue {
    Evaluator::new(&definitions())
        .expect("definitions check")
        .evaluate(&expression, &[], 128)
        .expect("expression evaluates")
        .value
}

#[test]
fn every_fixed_expression_form_executes_only_generic_data_mechanics() {
    assert_eq!(
        evaluate(KExpr::BytesLiteral(b"a".to_vec())),
        KValue::Bytes(b"a".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::TermLiteral(atom(b"k", b"p", b"e"))),
        KValue::Term(atom(b"k", b"p", b"e"))
    );
    assert_eq!(
        evaluate(KExpr::Let {
            value: Box::new(KExpr::BytesLiteral(b"bound".to_vec())),
            body: Box::new(KExpr::Var(0)),
        }),
        KValue::Bytes(b"bound".to_vec())
    );

    let made_atom = KExpr::MakeAtom {
        kind: Box::new(KExpr::BytesLiteral(b"kind".to_vec())),
        payload: Box::new(KExpr::BytesLiteral(b"payload".to_vec())),
        equality: Box::new(KExpr::BytesLiteral(b"eq".to_vec())),
    };
    assert_eq!(
        evaluate(made_atom.clone()),
        KValue::Term(atom(b"kind", b"payload", b"eq"))
    );

    let triple = KExpr::MakeTriple {
        first: Box::new(made_atom.clone()),
        second: Box::new(made_atom.clone()),
        third: Box::new(made_atom.clone()),
    };
    assert!(matches!(
        evaluate(triple.clone()),
        KValue::Term(Term::Triple(..))
    ));

    assert_eq!(
        evaluate(KExpr::CaseTerm {
            scrutinee: Box::new(made_atom),
            atom_body: Box::new(KExpr::Var(1)),
            triple_body: Box::new(KExpr::BytesLiteral(b"unselected".to_vec())),
        }),
        KValue::Bytes(b"payload".to_vec())
    );
    assert!(matches!(
        evaluate(KExpr::CaseTerm {
            scrutinee: Box::new(triple),
            atom_body: Box::new(KExpr::TermLiteral(atom(b"unused", b"unused", b"unused"))),
            triple_body: Box::new(KExpr::Var(2)),
        }),
        KValue::Term(Term::Atom { .. })
    ));

    assert_eq!(
        evaluate(KExpr::CaseBytes {
            scrutinee: Box::new(KExpr::BytesLiteral(Vec::new())),
            empty_body: Box::new(KExpr::BytesLiteral(b"empty".to_vec())),
            cons_body: Box::new(KExpr::Var(0)),
        }),
        KValue::Bytes(b"empty".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::CaseBytes {
            scrutinee: Box::new(KExpr::BytesLiteral(b"abc".to_vec())),
            empty_body: Box::new(KExpr::BytesLiteral(Vec::new())),
            cons_body: Box::new(KExpr::ConcatBytes(vec![KExpr::Var(0), KExpr::Var(1)])),
        }),
        KValue::Bytes(b"abc".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::CaseBytesEqual {
            left: Box::new(KExpr::BytesLiteral(b"x".to_vec())),
            right: Box::new(KExpr::BytesLiteral(b"x".to_vec())),
            equal_body: Box::new(KExpr::BytesLiteral(b"yes".to_vec())),
            unequal_body: Box::new(KExpr::BytesLiteral(b"no".to_vec())),
        }),
        KValue::Bytes(b"yes".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::Call {
            definition_id: id(1),
            arguments: vec![KExpr::BytesLiteral(b"opaque".to_vec())],
        }),
        KValue::Bytes(b"opaque".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::Request {
            physical_operation_id: sha256_operation_id(),
            arguments: vec![KExpr::BytesLiteral(b"abc".to_vec())],
        }),
        KValue::Bytes(hex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        ))
    );
}

#[test]
fn deterministic_certificate_is_postorder_and_binds_exact_execution_state() {
    let definitions = definitions();
    let evaluator = Evaluator::new(&definitions).expect("definitions check");
    let context = CertificateContext {
        exact_accepted_predecessor: b"exact accepted predecessor".to_vec(),
        core_contract_id: Hash32([3; 32]),
        physical_profile_id: Hash32([4; 32]),
        entrypoint: id(2),
        arguments: vec![KValue::Bytes(b"abc".to_vec())],
        fuel_limit: 20,
    };
    let first = evaluator
        .build_certificate(context.clone())
        .expect("certificate builds");
    let second = evaluator
        .build_certificate(context)
        .expect("certificate builds deterministically");
    assert_eq!(first, second);
    assert_eq!(
        first
            .nodes
            .iter()
            .map(|node| node.rule_tag)
            .collect::<Vec<_>>(),
        [0x30, 0x32, 0x3e, 0x3d]
    );
    for (index, node) in first.nodes.iter().enumerate() {
        assert!(
            node.premises
                .iter()
                .all(|premise| (*premise as usize) < index)
        );
    }
    assert_eq!(first.nodes.last().unwrap().premises, [0, 2]);
    assert_eq!(first.nodes.last().unwrap().conclusion.fuel_before, 20);
    assert_eq!(first.nodes.last().unwrap().conclusion.fuel_after, 16);
    assert_eq!(
        first.statement.expected,
        clause_substrate::compiler_package_v2::EvalOutcome::Returned {
            value: KValue::Bytes(hex(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            )),
            remaining_fuel: 16,
            observations: first
                .nodes
                .last()
                .unwrap()
                .conclusion
                .observations_after
                .clone(),
        }
    );
}

#[test]
fn package_ids_only_select_definition_data_and_never_a_host_callable() {
    let body = KExpr::BytesLiteral(b"same package data".to_vec());
    let left = vec![Definition {
        id: id(7),
        arguments: Vec::new(),
        result: KSort::Bytes,
        body: body.clone(),
    }];
    let right = vec![Definition {
        id: id(231),
        arguments: Vec::new(),
        result: KSort::Bytes,
        body,
    }];
    let left_result = Evaluator::new(&left)
        .unwrap()
        .evaluate(
            &KExpr::Call {
                definition_id: id(7),
                arguments: Vec::new(),
            },
            &[],
            4,
        )
        .unwrap();
    let right_result = Evaluator::new(&right)
        .unwrap()
        .evaluate(
            &KExpr::Call {
                definition_id: id(231),
                arguments: Vec::new(),
            },
            &[],
            4,
        )
        .unwrap();
    assert_eq!(left_result, right_result);

    let unknown = Id32([9; 32]);
    assert_eq!(
        Evaluator::new(&definitions()).unwrap().infer_sort(
            &KExpr::Request {
                physical_operation_id: unknown,
                arguments: vec![KExpr::BytesLiteral(Vec::new())],
            },
            &[],
        ),
        Err(StaticError::OperationOutsideSealedProfile(unknown))
    );
}

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap() as u8;
            let low = (pair[1] as char).to_digit(16).unwrap() as u8;
            (high << 4) | low
        })
        .collect()
}
