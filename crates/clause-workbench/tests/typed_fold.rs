use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{collections::BTreeMap, process::Command};

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/typed-fold.clause");
fn text(s: &str) -> V { V::text(s).unwrap() }
fn names(xs: &[&str]) -> V { V::Sequence(xs.iter().map(|s| text(s)).collect()) }
fn collected(xs: &[&str]) -> V {
    V::Record(BTreeMap::from([(b"names".to_vec(),names(xs)),(b"error".to_vec(),text(""))]))
}

#[test]
fn typed_fold_native_and_javascript_preserve_order_scope_and_effects() {
    let session = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    assert!(session.generation().unsupported.is_empty());
    for (input, expected) in [(&[][..],&[][..]),(&["z","skip","a","z"][..],&["z","a","z"][..])] {
        assert_eq!(session.invoke_callable(b"collect", &[names(input)]).unwrap(),collected(expected));
        assert_eq!(session.invoke_callable(b"empty-initial", &[names(input)]).unwrap(),names(input));
    }
    assert_eq!(session.invoke_callable(b"scope", &[text("outer"),names(&["a","b","a"])]).unwrap(),text("aba:outer"));
    for name in [b"empty-result".as_slice(),b"empty-argument"] {
        assert_eq!(session.invoke_callable(name,&[]).unwrap(),names(&[]));
    }
    assert_eq!(session.invoke_callable(b"empty-branch",&[V::Boolean(true)]).unwrap(),names(&[]));
    assert_eq!(session.invoke_callable(b"ordered",&[names(&["\u{e000}","\u{10000}","z","a","a"])]).unwrap(),names(&["a","a","z","\u{10000}","\u{e000}"]));
    assert_eq!(session.invoke_callable(b"body-failure",&[names(&[])]).unwrap(),V::number(7.0).unwrap());
    assert!(session.invoke_callable(b"body-failure",&[names(&["a"])]).is_err());
    assert!(session.invoke_callable(b"initial-failure",&[names(&[])]).is_err());
    let checked = session.checked_source_package().unwrap();
    let effects = checked.callables.iter().find(|c| c.designation == b"effects").unwrap();
    assert_eq!(clause_runtime::lower_canonical_callable_v1(effects).unwrap().invoke(&[]),Err(clause_runtime::ExecutableErrorV1::UnboundForeign));
    let mut pure = effects.clone();
    pure.mode = clause_package::CanonicalCallableModeV1::Function;
    assert!(clause_package::check_canonical_callable_v1(&pure).is_err());
    assert!(clause_package::render_nix_callable_v1(effects).is_err());
    let artifacts = clause_package::lower_javascript_v1(&checked).unwrap();
    let dir = std::env::temp_dir().join(format!("clause-typed-fold-{}",std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("fold.js"),artifacts.module).unwrap();
    std::fs::write(dir.join("fold-effects.mjs"),r#"
export const events=[];
export let values=['z','a','stop','later'];
export function setValues(xs){values=xs;events.length=0;}
export function source(){events.push('source');return values;}
export function initial(){events.push('initial');return {names:[],error:''};}
export function inspect(name){events.push(name);return name==='stop'?'failed':'';}
"#).unwrap();
    std::fs::write(dir.join("check.mjs"),r#"
import assert from 'node:assert/strict';
import * as f from './fold.js';
import {events,setValues} from './fold-effects.mjs';
for(const [input,expected] of [[[],[]],[['z','skip','a','z'],['z','a','z']]]) {
  assert.deepEqual(f.collect(input),{names:expected,error:''});
  assert.deepEqual(f['empty-initial'](input),input);
}
assert.equal(f.scope('outer',['a','b','a']),'aba:outer');
assert.deepEqual(f['empty-result'](),[]);
assert.deepEqual(f['empty-argument'](),[]);
assert.deepEqual(f['empty-branch'](true),[]);
const input=['\ue000','\u{10000}','z','a','a'];
assert.deepEqual(f.ordered(input),['a','a','z','\u{10000}','\ue000']);
assert.deepEqual(input,['\ue000','\u{10000}','z','a','a']);
assert(Object.isFrozen(f.ordered(input)));
assert.equal(f['body-failure']([]),7);
assert.throws(()=>f['body-failure'](['a']));
assert.throws(()=>f['initial-failure']([]));
assert.deepEqual(f.effects(),{names:['z','a'],error:'failed'});
assert.deepEqual(events,['source','initial','z','a','stop']);
setValues([]);
assert.deepEqual(f.effects(),{names:[],error:''});
assert.deepEqual(events,['source','initial']);
"#).unwrap();
    let output=Command::new(std::env::var_os("BUN").unwrap_or_else(||"bun".into())).arg(dir.join("check.mjs")).output().unwrap();
    assert!(output.status.success(),"{output:?}");
    for file in ["fold.js","fold-effects.mjs","check.mjs"] { std::fs::remove_file(dir.join(file)).unwrap(); }
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn typed_fold_rejects_wrong_contracts_bindings_and_pure_effects() {
    for source in [
        "export f(?xs: Sequence<Text>): Text\n  fold(?xs, \"\", ?a, ?x, 1)\n",
        "export f(?xs: Sequence<Text>): Sequence<Text>\n  append(?xs, 1)\n",
        "export f(?xs: Sequence<F64>): Sequence<F64>\n  sort(?xs)\n",
        "export f(?xs: Sequence<Text>): Text\n  fold(?xs, \"\", ?a, ?a, ?a)\n",
        "export f(?xs: Sequence<Text>): Text\n  fold(?xs, ?a, ?a, ?x, ?a)\n",
        "export f(?xs: Sequence<Text>): Text\n  fold(?xs, \"\", ?a, ?x, ?missing)\n",
        "export f(?xs: Sequence<Text>): Text\n  fold(?xs, \"\", ?a, ?x, p())\n\nforeign p(): Text\n  call: \"p\"\n  from: \"example\"\n  failure: throw\n",
        "export procedure f(?xs: Sequence<Text>): Sequence<Text>\n  [p() for ?x in ?xs]\n\nforeign p(): Text\n  call: \"p\"\n  from: \"example\"\n  failure: throw\n",
    ] { assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),"accepted {source}"); }
}
