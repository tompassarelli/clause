use clause_substrate::compiler_package_v3::{
    Definition, FallibleBox, Id32, KExpr, KSort, KValue, Term, eval_receipt_observations_hash,
    eval_receipt_value_hash, sha256_operation_id,
};
use clause_substrate::evaluator::{Evaluator, StaticError};

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

fn boxed<T>(value: T) -> FallibleBox<T> {
    FallibleBox::try_new(value).expect("test value allocation")
}

fn made_atom() -> KExpr {
    KExpr::MakeAtom {
        kind: boxed(KExpr::BytesLiteral(b"kind".to_vec())),
        payload: boxed(KExpr::BytesLiteral(b"payload".to_vec())),
        equality: boxed(KExpr::BytesLiteral(b"eq".to_vec())),
    }
}

fn triple() -> KExpr {
    KExpr::MakeTriple {
        first: boxed(made_atom()),
        second: boxed(made_atom()),
        third: boxed(made_atom()),
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

fn nested_concat(depth: usize) -> KExpr {
    (0..depth).fold(KExpr::BytesLiteral(Vec::new()), |expression, _| {
        KExpr::ConcatBytes(vec![expression])
    })
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
            value: boxed(KExpr::BytesLiteral(b"bound".to_vec())),
            body: boxed(KExpr::Var(0)),
        }),
        KValue::Bytes(b"bound".to_vec())
    );

    assert_eq!(
        evaluate(made_atom()),
        KValue::Term(atom(b"kind", b"payload", b"eq"))
    );

    assert!(matches!(evaluate(triple()), KValue::Term(Term::Triple(..))));

    assert_eq!(
        evaluate(KExpr::CaseTerm {
            scrutinee: boxed(made_atom()),
            atom_body: boxed(KExpr::Var(1)),
            triple_body: boxed(KExpr::BytesLiteral(b"unselected".to_vec())),
        }),
        KValue::Bytes(b"payload".to_vec())
    );
    assert!(matches!(
        evaluate(KExpr::CaseTerm {
            scrutinee: boxed(triple()),
            atom_body: boxed(KExpr::TermLiteral(atom(b"unused", b"unused", b"unused"))),
            triple_body: boxed(KExpr::Var(2)),
        }),
        KValue::Term(Term::Atom { .. })
    ));

    assert_eq!(
        evaluate(KExpr::CaseBytes {
            scrutinee: boxed(KExpr::BytesLiteral(Vec::new())),
            empty_body: boxed(KExpr::BytesLiteral(b"empty".to_vec())),
            cons_body: boxed(KExpr::Var(0)),
        }),
        KValue::Bytes(b"empty".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::CaseBytes {
            scrutinee: boxed(KExpr::BytesLiteral(b"abc".to_vec())),
            empty_body: boxed(KExpr::BytesLiteral(Vec::new())),
            cons_body: boxed(KExpr::ConcatBytes(vec![KExpr::Var(0), KExpr::Var(1)])),
        }),
        KValue::Bytes(b"abc".to_vec())
    );
    assert_eq!(
        evaluate(KExpr::CaseBytesEqual {
            left: boxed(KExpr::BytesLiteral(b"x".to_vec())),
            right: boxed(KExpr::BytesLiteral(b"x".to_vec())),
            equal_body: boxed(KExpr::BytesLiteral(b"yes".to_vec())),
            unequal_body: boxed(KExpr::BytesLiteral(b"no".to_vec())),
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
fn deterministic_receipt_commits_to_the_complete_actual_execution() {
    let definitions = definitions();
    let evaluator = Evaluator::new(&definitions).expect("definitions check");
    let arguments = [KValue::Bytes(b"abc".to_vec())];
    let first = evaluator
        .build_receipt(id(2), &arguments, 20)
        .expect("receipt builds");
    let second = evaluator
        .build_receipt(id(2), &arguments, 20)
        .expect("receipt builds deterministically");
    assert_eq!(first, second);

    let actual = evaluator
        .evaluate(
            &KExpr::Call {
                definition_id: id(2),
                arguments: vec![KExpr::BytesLiteral(b"abc".to_vec())],
            },
            &[],
            20,
        )
        .expect("independent replay succeeds");
    let actual_observations = actual
        .observations
        .try_to_term()
        .expect("actual observations canonicalize");

    assert_eq!(first.format_version, 0x00);
    assert_eq!(first.expected_remaining_fuel, 16);
    assert_eq!(
        first.expected_value_hash,
        eval_receipt_value_hash(&actual.value).expect("actual value canonicalizes")
    );
    assert_eq!(
        actual.value,
        KValue::Bytes(hex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        ))
    );
    assert_eq!(
        first.expected_observations_hash,
        eval_receipt_observations_hash(&actual_observations)
            .expect("actual observations canonicalize for hashing")
    );
}

#[test]
fn package_ids_only_select_definition_data_and_never_a_host_callable() {
    let body = || KExpr::BytesLiteral(b"same package data".to_vec());
    let left = vec![Definition {
        id: id(7),
        arguments: Vec::new(),
        result: KSort::Bytes,
        body: body(),
    }];
    let right = vec![Definition {
        id: id(231),
        arguments: Vec::new(),
        result: KSort::Bytes,
        body: body(),
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

#[test]
fn bounded_expression_depth_uses_the_machine_stack() {
    let expression = nested_concat(300);
    let expression = std::thread::Builder::new()
        .name("bounded-stack-evaluator".to_owned())
        .stack_size(128 * 1024)
        .spawn(move || {
            let evaluator = Evaluator::new(&[]).expect("empty definition table checks");
            assert_eq!(evaluator.infer_sort(&expression, &[]), Ok(KSort::Bytes));
            assert_eq!(
                evaluator
                    .evaluate(&expression, &[], 512)
                    .expect("deep expression evaluates")
                    .value,
                KValue::Bytes(Vec::new())
            );
            expression
        })
        .expect("small-stack evaluator thread starts")
        .join()
        .expect("explicit evaluator does not overflow the host stack");
    drop(expression);

    let evaluator = Evaluator::new(&[]).expect("empty definition table checks");
    assert_eq!(
        evaluator.infer_sort(&nested_concat(512), &[]),
        Err(StaticError::ResourceExhausted)
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
