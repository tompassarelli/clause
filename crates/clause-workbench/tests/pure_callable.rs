use clause_package::{CanonicalExecutableExpressionV1, CanonicalSourceProductionV1};
use clause_runtime::ExecutableValueV1;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/pure-callable.clause");
fn text(s: &str) -> ExecutableValueV1 {
    ExecutableValueV1::text(s).unwrap()
}

#[test]
fn pure_callable_opens_lowers_and_returns_actual_missing_leaf_text() {
    let session = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    assert!(session.generation().unsupported.is_empty());
    let compiled = session.checked_source_package().unwrap();
    assert!(compiled.state_cells.is_empty());
    assert!(compiled.executable_handlers.is_empty());
    assert_eq!(compiled.callables.len(), 1);
    for production in [
        CanonicalSourceProductionV1::Relation,
        CanonicalSourceProductionV1::RelationMode,
        CanonicalSourceProductionV1::CallableDefinition,
        CanonicalSourceProductionV1::CallableExport,
    ] {
        assert!(
            compiled
                .emissions
                .iter()
                .any(|e| e.slot.production == production),
            "missing {production:?}"
        );
    }
    let result = session
        .invoke_callable(
            b"missing-leaf",
            &[
                text("module"),
                text("add"),
                text("<name>"),
                text("create a module"),
            ],
        )
        .unwrap();
    assert_eq!(
        result,
        text(
            "firn: 'module add' requires a leaf node\nUsage: firn module add <name>\n  create a module\n"
        )
    );
    assert!(session.invoke_callable(b"missing-leaf", &[]).is_err());
    assert!(
        session
            .invoke_callable(
                b"missing-leaf",
                &[
                    text("module"),
                    text("add"),
                    text("<name>"),
                    ExecutableValueV1::Boolean(false)
                ]
            )
            .is_err()
    );
    let mut invalid = compiled.callables[0].clone();
    invalid.expression = CanonicalExecutableExpressionV1::Accumulate(Box::new(invalid.expression));
    assert!(clause_runtime::lower_canonical_callable_v1(&invalid).is_err());
}

#[test]
fn pure_callable_checks_bindings_types_and_effects() {
    for source in [
        "export f(?x: Text): Bool\n  ?x\n",
        "export f(?x: Text): Text\n  ?missing\n",
        "export f(?x: F64): Text\n  \"{?x}\"\n",
        "export f(?x: Text): Text\n  include\n    ?x output ?x\n",
        "export f(?x: Text, ?x: Text): Text\n  ?x\n",
    ] {
        assert!(
            ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn pure_callable_interpolates_typed_expressions_and_preserves_literal_escapes() {
    let source = b"export f(?x: Text): Text\n  \"prefix {trim(?x)}: \\\"yes\\\"\\n\"\n";
    let session = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(
        session.invoke_callable(b"f", &[text("  world  ")]).unwrap(),
        text("prefix world: \"yes\"\n")
    );
    let private = ResidentSourceWorkbenchV1::open(b"f(?x: Text): Text\n  ?x\n").unwrap();
    assert!(private.invoke_callable(b"f", &[text("world")]).is_err());
}
