use clause_package::*;
use clause_workbench::ResidentSourceWorkbenchV1;
const SOURCE: &str = include_str!("../../../test-vectors/authoring/mixed-delayed-sequences/flags.clause");

#[test]
fn mixed_sequence_elements_preserve_types_order_and_target_boundaries() {
    use CanonicalValueTypeV1 as T;
    let opened = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let expected = T::Sequence(Box::new(T::Delayed {target: "nix".into(), value: Box::new(CanonicalScalarValueKindV1::Text.into())}));
    for (name, output) in [
        ("flags", "[ (\"--system\") (pkgs.\"name\") ]"),
        ("reversed", "[ (pkgs.\"name\") (\"--system\") ]"),
        ("repeated", "[ (\"--system\") (pkgs.\"name\") (\"--system\") ]"),
    ] {
        let callable = checked.callables.iter().find(|c| c.designation == name.as_bytes()).unwrap();
        assert_eq!(callable.result_kind, expected);
        assert_eq!(render_nix_callable_v1(callable).unwrap(), format!("{{ pkgs, ... }}:\n{output}\n"));
        assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());
    }
    let nested = checked.callables.iter().find(|c| c.designation == b"nested").unwrap();
    assert_eq!(nested.result_kind, T::Sequence(Box::new(T::Record(std::collections::BTreeMap::from([(b"flags".to_vec(), expected)])))));
    assert!(lower_javascript_v1(&checked).is_err());
    for wrong in [
        SOURCE.replace("\"--system\"", "2"),
        SOURCE.replace("Sequence<Delayed<nix,Text>>", "Sequence<Text>"),
        format!("{SOURCE}\nforeign other(): Text\n  construction: \"other\"\n  get: \"name\"\n  from: \"pkgs\"\n  failure: throw\nexport wrong()\n  [target-text(), other()]\n"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "accepted incompatible sequence");
    }
    let other = SOURCE.replace("\"nix\"", "\"other\"").replace("Delayed<nix,", "Delayed<other,");
    let checked = ResidentSourceWorkbenchV1::open(other.as_bytes()).unwrap().checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"flags").unwrap();
    assert!(render_nix_callable_v1(callable).is_err());
    assert!(lower_javascript_v1(&checked).is_err());
}

#[test]
fn ordinary_sequences_keep_native_and_javascript_execution() {
    let source = b"export flags(): Sequence<Text>\n  [\"--system\", \"kept\", \"--system\"]\n";
    let opened = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(opened.invoke_callable(b"flags", &[]).unwrap(), clause_runtime::ExecutableValueV1::Sequence(
        ["--system", "kept", "--system"].map(|s| clause_runtime::ExecutableValueV1::text(s).unwrap()).to_vec()));
    assert!(lower_javascript_v1(&opened.checked_source_package().unwrap()).is_ok());
}
