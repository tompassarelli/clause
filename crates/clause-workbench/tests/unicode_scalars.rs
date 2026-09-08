use clause_runtime::{ExecutableValueV1 as V, ExecutableExpressionV1 as E, ExecutableProgramV1, ExecutableRuleV1, decode_executable_physical_plan_v1, encode_executable_physical_plan_v1};
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};
fn text(s: &str) -> V { V::text(s).unwrap() }
fn number(n: f64) -> V { V::Number(n.to_bits()) }
fn decoded(s: &str, end: f64) -> V { V::Record([(b"text".to_vec(),text(s)),(b"end".to_vec(),number(end))].into()) }
fn rejected(offset: f64, message: &str) -> V { V::Record([(b"offset".to_vec(),number(offset)),(b"message".to_vec(),text(message))].into()) }

#[test]
fn unicode_scalars_and_json_escape_offsets_agree_in_native_and_bun() {
    // This whole-source fixture needs the CLI's stack size, beyond libtest's default.
    std::thread::Builder::new().stack_size(8 * 1024 * 1024)
        .spawn(unicode_scalar_checks).unwrap().join().unwrap();
}

fn unicode_scalar_checks() {
    let scalars = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/unicode-scalars/scalars.clause")).unwrap();
    let json = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/unicode-scalars/json-escape.clause")).unwrap();
    let mut plan = decode_executable_physical_plan_v1(&scalars.generation().cpp1).unwrap();
    plan.input=None; plan.source_metadata=None;
    plan.program=ExecutableProgramV1 {initial_configuration:vec![number(0.0)], projection:None,
      rules:vec![ExecutableRuleV1 {entry:0,predicates:vec![],required_present:vec![],required_absent:vec![],removals:vec![],
        assignments:vec![(0,E::TextCodepoint(Box::new(E::TextFromCodepoint(Box::new(E::Constant(number(128512.0)))))))],}]};
    assert_eq!(decode_executable_physical_plan_v1(&encode_executable_physical_plan_v1(&plan).unwrap()).unwrap(),plan);
    let mut js=String::from("import assert from 'node:assert/strict';import * as s from './scalars.mjs';import * as j from './json.mjs';\n");
    for point in [0,0x41,0xe9,0x7ff,0x800,0xd7ff,0xe000,0xffff,0x10000,0x1f600,0x10ffff] {
        let value=char::from_u32(point).unwrap().to_string();
        assert_eq!(scalars.invoke_callable(b"character",&[number(point as f64)]).unwrap(),text(&value));
        assert_eq!(scalars.invoke_callable(b"scalar",&[text(&value)]).unwrap(),number(point as f64));
        assert_eq!(scalars.invoke_callable(b"roundtrip",&[text(&value)]).unwrap(),text(&value));
        js.push_str(&format!("assert.equal(s.character({point}),String.fromCodePoint({point}));assert.equal(s.scalar(String.fromCodePoint({point})),{point});assert.equal(s.roundtrip(String.fromCodePoint({point})),String.fromCodePoint({point}));\n"));
    }
    for value in [-1.0,0.5,55296.0,57343.0,1114112.0,f64::INFINITY,f64::NAN] {assert!(scalars.invoke_callable(b"character",&[number(value)]).is_err());}
    for value in ["","ab","e\u{301}","😀a"] {assert!(scalars.invoke_callable(b"scalar",&[text(value)]).is_err());}
    for (prefix,escape,value,end) in [("", "\\u0041","A",6.0),("é", "\\u00e9","é",8.0),("é😀", "\\uD83D\\uDE00","😀",18.0)] {
        assert_eq!(json.invoke_callable(b"after-prefix",&[text(prefix),text(escape)]).unwrap(),decoded(value,end));
        js.push_str(&format!("assert.deepEqual(j['after-prefix']({prefix:?},{escape:?}),{{text:{value:?},end:{end}}});\n"));
    }
    for (escape, offset, message) in [("\\u12x4",8.0,"invalid four-digit Unicode escape"),("\\u12",8.0,"invalid four-digit Unicode escape"),("\\uD800",6.0,"high surrogate is not followed by a low surrogate"),("\\uD800\\u0041",14.0,"high surrogate is not followed by a low surrogate"),("\\uD800\\u😀",14.0,"high surrogate is not followed by a low surrogate"),("\\uDC00",6.0,"unpaired low surrogate")] {
        assert_eq!(json.invoke_callable(b"after-prefix",&[text("é😀"),text(escape)]).unwrap(),rejected(offset,message));
        js.push_str(&format!("assert.deepEqual(j['after-prefix']('é😀',{escape:?}),{{offset:{offset},message:{message:?}}});\n"));
    }
    assert_eq!(json.invoke_callable(b"byte-length",&[text("aé€😀")]).unwrap(),number(10.0));
    for bad in ["export f()\n  codepoint(1)\n","export f()\n  from-codepoint(\"a\")\n","export f()\n  codepoint(\"a\",\"b\")\n"] {assert!(ResidentSourceWorkbenchV1::open(bad.as_bytes()).is_err());}
    js.push_str("assert.equal(j['byte-length']('aé€😀'),10);for(const n of [-1,.5,0xd800,0xdfff,0x110000,Infinity,NaN])assert.throws(()=>s.character(n));for(const t of ['', 'ab','e\\u0301','😀a','\\ud800','\\udfff'])assert.throws(()=>s.scalar(t));\n");
    let output=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/unicode-scalars");fs::create_dir_all(&output).unwrap();
    for (name,session) in [("scalars",&scalars),("json",&json)] {fs::write(output.join(format!("{name}.mjs")),clause_package::lower_javascript_v1(&session.checked_source_package().unwrap()).unwrap().module).unwrap();}
    fs::write(output.join("check.mjs"),js).unwrap();
    let result=Command::new(std::env::var("BUN").unwrap()).arg("check.mjs").current_dir(&output).output().unwrap();fs::write(output.join("bun.stderr"),&result.stderr).unwrap();
    assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
}
