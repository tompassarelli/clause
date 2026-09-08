use std::collections::BTreeMap;
use clause_package::{CanonicalScalarValueKindV1 as K, CanonicalValueTypeV1 as T, CanonicalScalarValueV1 as C, check_canonical_callable_v1, lower_javascript_v1, render_nix_callable_v1};
use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/dictionary.clause");
fn text(value: &str) -> V { V::text(value).unwrap() }

#[test]
fn dictionary_preserves_exact_values_in_native_and_javascript() {
    let session = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let checked = session.checked_source_package().unwrap();
    let settings = T::Record(BTreeMap::from([
        (b"enabled".to_vec(), K::Boolean.into()),
        (b"labels".to_vec(), T::Sequence(Box::new(K::Text.into()))),
    ]));
    let dictionary = T::Dictionary(Box::new(settings));
    assert_eq!(checked.callables.iter().find(|c| c.designation == b"settings").unwrap().result_kind, dictionary);
    for name in ["tom", "a.b-\"${user}", "", "__proto__", "café"] {
        let expected = V::Record(BTreeMap::from([(name.as_bytes().to_vec(), V::Record(BTreeMap::from([
            (b"enabled".to_vec(), V::Boolean(true)),
            (b"labels".to_vec(), V::Sequence(vec![text("kept")])),
        ])))]));
        assert_eq!(session.invoke_callable(b"settings", &[text(name)]).unwrap(), expected);
        assert_eq!(session.invoke_callable(b"preserve", &[expected.clone()]).unwrap(), expected);
    }
    let wrong = V::Record(BTreeMap::from([(b"tom".to_vec(), V::Record(BTreeMap::from([(b"enabled".to_vec(), text("wrong"))])))]));
    assert!(session.invoke_callable(b"preserve", &[wrong]).is_err());
    assert!(!dictionary.accepts(&C::Record(BTreeMap::from([(b"tom".to_vec(), C::Boolean(true))]))));
    let artifacts = lower_javascript_v1(&checked).unwrap();
    assert!(artifacts.declarations.contains("readonly [key: string]"));
    let directory = std::env::temp_dir().join(format!("clause-dictionary-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("dictionary.mjs"), artifacts.module).unwrap();
    std::fs::write(directory.join("run.mjs"), r#"
import assert from 'node:assert/strict';
import {settings, preserve} from './dictionary.mjs';
for (const name of ['tom', 'a.b-"${user}', '', '__proto__', 'café']) {
  const result = settings(name);
  assert.deepEqual(Object.keys(result), [name]);
  assert.deepEqual(result[name], {enabled:true,labels:['kept']});
  assert.deepEqual(preserve(result), result);
  assert.ok(Object.isFrozen(result));
}
assert.deepEqual(preserve({}), {});
assert.deepEqual(preserve({tom:{enabled:true,labels:[]},other:{enabled:false,labels:['x']}}), {tom:{enabled:true,labels:[]},other:{enabled:false,labels:['x']}});
for (const wrong of [true, [], {tom:true}, {tom:{enabled:'wrong',labels:[]}}, {tom:{enabled:true,labels:[],extra:1}}]) assert.throws(()=>preserve(wrong));
assert.throws(()=>preserve(Object.fromEntries([['\ud800',{enabled:true,labels:[]}]])));
assert.throws(()=>settings(true));
"#).unwrap();
    let result = std::process::Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into())).arg(directory.join("run.mjs")).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    for file in ["dictionary.mjs", "run.mjs"] { std::fs::remove_file(directory.join(file)).unwrap(); }
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn dictionary_rejects_wrong_keys_values_and_static_field_claims() {
    for source in [
        SOURCE.replace("dictionary(?name,", "dictionary(true,"),
        SOURCE.replace("enabled: true", "enabled: 1"),
        SOURCE.replace("labels: [\"kept\"]", "labels: [true]"),
        "export f(?name: Text)\n  field-at(dictionary(?name, true), path(tom))\n".into(),
        "export f(?name: Text): Dictionary<Bool>\n  {tom: true}\n".into(),
        "export f(?value: Dictionary<Bool> | Dictionary<Text>)\n  true\n".into(),
        "Settings:\n  enabled: Bool\n\nexport f(?value: Dictionary<Bool> | Settings)\n  true\n".into(),
    ] { assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "accepted {source}"); }
    let session = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let mut callable = session.checked_source_package().unwrap().callables.into_iter().find(|c| c.designation == b"settings").unwrap();
    callable.arguments[0].value_kind = K::Boolean.into();
    assert!(check_canonical_callable_v1(&callable).is_err());
}

#[test]
fn delayed_dictionary_keeps_nix_keys_and_nested_contracts() {
    let source = "foreign username(): Text\n  construction: \"nix\"\n  get: \"username\"\n  from: \"config\"\n  failure: throw\n\nforeign enabled(): Bool\n  construction: \"nix\"\n  get: \"enabled\"\n  from: \"config\"\n  failure: throw\n\nexport settings()\n  dictionary(username(), {enabled: enabled(), labels: [\"kept\"]})\n";
    let session = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let checked = session.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"settings").unwrap();
    assert_eq!(callable.result_kind, T::Delayed { target: "nix".into(), value: Box::new(T::Dictionary(Box::new(T::Record(BTreeMap::from([
        (b"enabled".to_vec(), K::Boolean.into()),
        (b"labels".to_vec(), T::Sequence(Box::new(K::Text.into()))),
    ]))))) });
    assert!(render_nix_callable_v1(callable).unwrap().contains("${config.\"username\"}"));
    assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());
    assert!(lower_javascript_v1(&checked).is_err());
    let wrong = source.replacen("construction: \"nix\"", "construction: \"other\"", 1);
    assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err());
}

#[test]
fn callable_unicode_escapes_remain_literal_before_interpolation() {
    let source = br#"export rules(?suffix: Text): Text
  "ATTRS\u{7b}idVendor}=\"c2ab\" {?suffix} \u{1f680}"
"#;
    let session = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(session.invoke_callable(b"rules", &[text("kept")]).unwrap(), text("ATTRS{idVendor}=\"c2ab\" kept 🚀"));
    for escape in ["\\u{}", "\\u{d800}", "\\u{110000}", "\\u{xx}", "\\u{1234567}"] {
        let source = format!("export f(): Text\n  \"{escape}\"\n");
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "accepted {escape}");
    }
}
