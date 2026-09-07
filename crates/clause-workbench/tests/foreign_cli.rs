use clause_package::{CanonicalCallableModeV1, CanonicalExecutableExpressionV1 as E};
use clause_runtime::{ExecutableErrorV1, ExecutableValueV1 as V, lower_canonical_callable_v1};
use clause_workbench::ResidentSourceWorkbenchV1;
use std::collections::BTreeMap;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/foreign-cli.clause");
fn text(value: &str) -> V {
    V::text(value).unwrap()
}
fn args(values: &[&str]) -> V {
    V::Sequence(values.iter().map(|v| text(v)).collect())
}

#[test]
fn ordered_argv_forms_one_typed_outcome_and_retains_foreign_obligations() {
    let session = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    assert!(session.generation().unsupported.is_empty());
    let outcome = session
        .invoke_callable(b"dispatch", &[args(&["module", "add"])])
        .unwrap();
    assert_eq!(
        outcome,
        V::Record(BTreeMap::from([
            (
                b"message".to_vec(),
                text(
                    "firn: 'module add' requires a leaf node\nUsage: firn module add <name>\n  scaffold a minimal module (.bnix + .nix)\n"
                )
            ),
            (b"status".to_vec(), V::number(1.0).unwrap()),
        ]))
    );
    for values in [
        &[][..],
        &["add", "module"],
        &["module", "add", "name"],
        &["module", "module"],
        &["module", "add", "add"],
    ] {
        assert!(
            session
                .invoke_callable(b"dispatch", &[args(values)])
                .is_err(),
            "{values:?}"
        );
    }
    assert!(session.invoke_callable(b"dispatch", &[]).is_err());
    assert!(
        session
            .invoke_callable(b"dispatch", &[V::Sequence(vec![V::Boolean(true)])])
            .is_err()
    );
    let package = session.checked_source_package().unwrap();
    let contracts = package
        .checked_package
        .constitution()
        .preimage()
        .operators
        .iter()
        .flat_map(|operator| &operator.modes)
        .map(|mode| &mode.contract)
        .collect::<Vec<_>>();
    assert_eq!(
        contracts
            .iter()
            .filter(|contract| contract.is_function())
            .count(),
        2
    );
    let mut access_counts = contracts
        .iter()
        .map(|contract| contract.foreign_accesses.len())
        .collect::<Vec<_>>();
    access_counts.sort();
    assert_eq!(access_counts, vec![0, 0, 1, 1, 1, 2]);
    for contract in contracts
        .iter()
        .filter(|contract| !contract.foreign_accesses.is_empty())
    {
        assert!(!contract.is_pure() && !contract.is_function());
        assert!(contract.effect_intents.is_empty() && contract.capability_requirements.is_empty());
        for access in &contract.foreign_accesses {
            access.check().unwrap();
        }
    }
    let run = package
        .callables
        .iter()
        .find(|c| c.designation == b"run")
        .unwrap();
    assert_eq!(run.mode, CanonicalCallableModeV1::Procedure);
    assert_eq!(
        lower_canonical_callable_v1(run).unwrap().invoke(&[]),
        Err(ExecutableErrorV1::UnboundForeign)
    );
    let mut invalid = run.clone();
    invalid.mode = CanonicalCallableModeV1::Function;
    assert!(lower_canonical_callable_v1(&invalid).is_err());
    let property = package
        .callables
        .iter()
        .find(|c| c.designation == b"arguments")
        .unwrap();
    let mut wrong = property.clone();
    if let E::Foreign { arguments, .. } = &mut wrong.expression {
        arguments.push(E::Constant(
            clause_package::CanonicalScalarValueV1::Boolean(true),
        ));
    } else {
        panic!("foreign declaration was lost");
    }
    assert!(lower_canonical_callable_v1(&wrong).is_err());
}

#[test]
fn recursive_types_arity_and_pure_effect_use_reject_at_source_boundary() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    for rejected in [
        source.replace("export procedure run()", "export run()"),
        source.replace("write-output(2,", "write-output(true,"),
        source.replace("write-output(2, ?outcome.message)", "write-output(2)"),
        source.replace("status: 1", "status: true"),
        source.replace("?outcome.status", "?outcome.missing"),
        source.replace("foreign arguments()", "foreign arguments(?input: Text)"),
        source.replace("  failure: throw\n", ""),
        source.replace("procedure deliver", "deliver"),
    ] {
        assert!(
            ResidentSourceWorkbenchV1::open(rejected.as_bytes()).is_err(),
            "accepted {rejected}"
        );
    }
}
