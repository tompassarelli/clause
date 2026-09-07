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

#[test]
fn pure_composition_resolves_forward_calls_and_preserves_binding_scope() {
    let session = ResidentSourceWorkbenchV1::open(include_bytes!(
        "../../../test-vectors/authoring/pure-composition.clause"
    ))
    .unwrap();
    assert_eq!(
        session
            .invoke_callable(b"missing-command", &[text("module"), text("add")])
            .unwrap(),
        text(
            "firn: 'module add' requires a leaf node\nUsage: firn module add <name>\n  Add a module\n"
        )
    );
    assert_eq!(
        session
            .invoke_callable(b"missing-command", &[text("module"), text("list")])
            .unwrap(),
        text("")
    );
    assert!(session.invoke_callable(b"missing-leaf", &[]).is_err());
    let source = b"export swap(?left: Text, ?right: Text): Text\n  pair(?right, pair(?left, ?right))\n\npair(?left: Text, ?right: Text): Text\n  \"{?left}/{?right}\"\n";
    let session = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(
        session
            .invoke_callable(b"swap", &[text("a"), text("b")])
            .unwrap(),
        text("b/a/b")
    );
    let source = b"pair(?left: Text, ?right: Text): Text\n  \"{?left}/{?right}\"\n\nexport swap(?left: Text, ?right: Text): Text\n  \"{pair(?right, pair(?left, ?right))}\"\n";
    let session = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(
        session
            .invoke_callable(b"swap", &[text("a"), text("b")])
            .unwrap(),
        text("b/a/b")
    );
}

#[test]
fn pure_composition_rejects_wrong_arguments_results_effects_and_cycles() {
    for source in [
        "export f(?x: Text): Text\n  g()\n\ng(?x: Text): Text\n  ?x\n",
        "export f(?x: Text): Text\n  g(true)\n\ng(?x: Text): Text\n  ?x\n",
        "export f(?x: Text): Bool\n  g(?x)\n\ng(?x: Text): Text\n  ?x\n",
        "export f(?x: Text): Text\n  g(?x)\n\ng(?x: Text): Text\n  ?missing\n",
        "export f(?x: Text): Text\n  g(?x)\n\ng(?x: Text): Text\n  include\n    ?x output ?x\n",
        "export f(?x: Text): Text\n  f(?x)\n",
        "export f(?x: Text): Text\n  g(?x)\n\ng(?x: Text): Text\n  f(?x)\n",
        "export f(?x: Text): Text\n  unknown(?x)\n",
        "export trim(?x: Text): Text\n  ?x\n",
    ] {
        assert!(
            ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),
            "accepted {source}"
        );
    }
    let cycle =
        clause_package::read_canonical_source_v1(b"f(?x: Text): Text\n  f(?x)\n").unwrap_err();
    assert!(
        matches!(cycle, clause_package::CanonicalSourceErrorV1::InvalidCallable { reason: "recursive pure callables are unsupported", origin } if origin.start > 0)
    );
}

#[test]
fn pure_composition_bounds_expanded_definitions() {
    let mut source = String::from("export f0(?x: Text): Text\n  f1(?x) ++ f1(?x)\n");
    for index in 1..18 {
        source.push_str(&format!(
            "\nf{index}(?x: Text): Text\n  f{}(?x) ++ f{}(?x)\n",
            index + 1,
            index + 1
        ));
    }
    source.push_str("\nf18(?x: Text): Text\n  ?x\n");
    let error = clause_package::read_canonical_source_v1(source.as_bytes()).unwrap_err();
    assert!(matches!(
        error,
        clause_package::CanonicalSourceErrorV1::InvalidCallable {
            reason: "pure callable expansion limit",
            ..
        }
    ));
}

#[test]
fn pure_composition_preserves_unused_argument_failure_and_branch_laziness() {
    let session = ResidentSourceWorkbenchV1::open(include_bytes!(
        "../../../test-vectors/authoring/pure-call-failure.clause"
    ))
    .unwrap();
    assert!(
        session
            .invoke_callable(
                b"unused-failure",
                &[ExecutableValueV1::number(0.0).unwrap()]
            )
            .is_err()
    );
    assert_eq!(
        session
            .invoke_callable(
                b"unused-failure",
                &[ExecutableValueV1::number(2.0).unwrap()]
            )
            .unwrap(),
        text("unused")
    );
    let session = ResidentSourceWorkbenchV1::open(b"export f(?yes: Bool): Text\n  if(?yes, ignore(1.0 / 0.0), \"safe\")\n\nignore(?unused: F64): Text\n  \"unused\"\n").unwrap();
    assert_eq!(
        session
            .invoke_callable(b"f", &[ExecutableValueV1::Boolean(false)])
            .unwrap(),
        text("safe")
    );
    assert!(
        session
            .invoke_callable(b"f", &[ExecutableValueV1::Boolean(true)])
            .is_err()
    );
    let compiled = session.checked_source_package().unwrap();
    let mut invalid = compiled.callables[0].clone();
    invalid.expression = CanonicalExecutableExpressionV1::Binding(0);
    assert!(clause_package::check_canonical_callable_v1(&invalid).is_err());
}
