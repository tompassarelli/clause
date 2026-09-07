use std::collections::BTreeMap;

use clause_package::{CanonicalExecutableExpressionV1 as E, CanonicalScalarValueKindV1 as K, CanonicalValueTypeV1 as T};
use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;

#[test]
fn callable_results_infer_exact_nested_fields_through_forward_calls() {
    let source = b"export report(?enabled: Bool)\n  if(?enabled, description(), {nested: {message: \"off\", score: 2}})\n\ndescription()\n  {nested: {message: \"on\", score: 1}}\n";
    let opened = ResidentSourceWorkbenchV1::open(source).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let report = checked.callables.iter().find(|c| c.designation == b"report").unwrap();
    assert_eq!(report.result_kind, T::Record(BTreeMap::from([(
        b"nested".to_vec(), T::Record(BTreeMap::from([
            (b"message".to_vec(), K::Text.into()),
            (b"score".to_vec(), K::Number.into()),
        ])),
    )])));
    assert_eq!(opened.invoke_callable(b"report", &[V::Boolean(false)]).unwrap(), V::Record(BTreeMap::from([(
        b"nested".to_vec(), V::Record(BTreeMap::from([
            (b"message".to_vec(), V::text("off").unwrap()),
            (b"score".to_vec(), V::number(2.0).unwrap()),
        ])),
    )])));
    let javascript = clause_package::lower_javascript_v1(&checked).unwrap();
    assert!(javascript.declarations.contains("message"));
    assert!(javascript.declarations.contains("score"));

    for wrong in [
        "export f()\n  if(true, {nested: {x: 1}}, {nested: {x: false}})\n",
        "export f()\n  if(true, {nested: {x: 1}}, {nested: {y: 1}})\n",
        "export f()\n  {nested: {values: []}}\n",
        "export f()\n  {nested: {x: 1}}.missing\n",
        "export f(): Bool\n  {nested: {x: 1}}\n",
        "export f()\n  f()\n",
        "foreign f()\n  get: \"x\"\n  from: \"m\"\n  failure: throw\n",
    ] {
        assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "accepted {wrong}");
    }
}

#[test]
fn foreign_record_parameters_instantiate_every_field_and_result() {
    let source = "foreign echo<Body: Record>(?value: Body): Body\n  call: \"echo\"\n  from: \"records\"\n  failure: throw\n\nexport procedure first()\n  echo({nested: {answer: 42}})\n\nexport procedure second()\n  echo({message: \"hello\"})\n";
    let opened = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let checked = opened.checked_source_package().unwrap();
    assert_eq!(checked.callables.len(), 2);
    for callable in &checked.callables {
        let E::Foreign { binding, .. } = &callable.expression else { panic!("expected specialized foreign access") };
        assert_eq!(binding.arguments, vec![callable.result_kind.clone()]);
        assert_eq!(binding.result, callable.result_kind);
        binding.check().unwrap();
    }
    assert_ne!(checked.callables[0].result_kind, checked.callables[1].result_kind);
    clause_package::lower_javascript_v1(&checked).unwrap();

    let repeated = "foreign same<Body: Record>(?left: Body, ?right: Body): Body\n  call: \"same\"\n  from: \"records\"\n  failure: throw\n\nexport procedure run()\n  same({x: 1}, {x: 2})\n";
    ResidentSourceWorkbenchV1::open(repeated.as_bytes()).unwrap();
    for wrong in [
        source.replace("echo({message: \"hello\"})", "echo(\"hello\")"),
        source.replace("export procedure", "export"),
        source.replace("Body: Record", "Body: Any"),
        source.replace("?value: Body", "?value: Text"),
        source.replace("foreign echo", "export foreign echo"),
        repeated.replace("{x: 2}", "{x: false}"),
        repeated.replace("{x: 2}", "{y: 2}"),
        "foreign bad<Body: Record>(?x: Body): Delayed<nix,Bool>\n  call: \"bad\"\n  from: \"records\"\n  failure: throw\n".into(),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "accepted {wrong}");
    }
}
