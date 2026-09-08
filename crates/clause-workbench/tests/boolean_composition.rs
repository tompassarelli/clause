use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::process::Command;

const COUNTEREXAMPLE: &str =
    include_str!("../../../test-vectors/authoring/boolean-composition.clause");
const COMPOSITION: &str = r#"
export precedence(): Bool
  true or false and false
export grouped(): Bool
  (true or false) and false
export comparison(?x: F64): Bool
  ?x + 1 > 2 and ?x <= 4 or ?x = 0
export nested(?a: Bool, ?b: Bool): Text
  "{if(?a and ?b, true or false, false)}"
export skip-and(?left: Bool): Bool
  ?left and require(false, true, "right evaluated")
export skip-or(?left: Bool): Bool
  ?left or require(false, false, "right evaluated")
export sequence(): Sequence<Bool>
  [true and false, false or true]
export record(): Bool
  {value: false or true}.value
export chained(): Bool
  true and true and true or false or false
and-gate(): Bool
  true
or-gate(): Bool
  false
export names(): Bool
  and-gate() and (true or or-gate())
"#;

#[test]
fn boolean_composition_native_and_javascript_preserve_types_precedence_and_short_circuiting() {
    let source = format!(
        r#"{COUNTEREXAMPLE}
{COMPOSITION}
foreign trace(?name: Text, ?value: Bool): Bool
  call: "trace"
  from: "./boolean-effects.mjs"
  failure: throw
export procedure effect-and(?left: Bool): Bool
  trace("left", ?left) and trace("right", true)
export procedure effect-or(?left: Bool): Bool
  trace("left", ?left) or trace("right", false)
"#
    );
    let session = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    for a in [false, true] {
        for b in [false, true] {
            assert_eq!(
                session
                    .invoke_callable(b"either-pressure", &[V::Boolean(a), V::Boolean(b)])
                    .unwrap(),
                V::Boolean(a || b)
            );
            assert_eq!(
                session
                    .invoke_callable(b"exclusive-conflict", &[V::Boolean(a), V::Boolean(b)])
                    .unwrap(),
                V::Boolean(a && b)
            );
        }
    }
    for (name, expected) in [
        (b"precedence".as_slice(), true),
        (b"grouped", false),
        (b"record", true),
        (b"chained", true),
        (b"names", true),
    ] {
        assert_eq!(
            session.invoke_callable(name, &[]).unwrap(),
            V::Boolean(expected)
        );
    }
    for (x, expected) in [(0.0, true), (1.0, false), (2.0, true), (5.0, false)] {
        assert_eq!(
            session
                .invoke_callable(b"comparison", &[V::number(x).unwrap()])
                .unwrap(),
            V::Boolean(expected)
        );
    }
    assert_eq!(
        session
            .invoke_callable(b"nested", &[V::Boolean(true), V::Boolean(false)])
            .unwrap(),
        V::text("false").unwrap()
    );
    for (name, left) in [(b"skip-and".as_slice(), false), (b"skip-or", true)] {
        assert_eq!(
            session.invoke_callable(name, &[V::Boolean(left)]).unwrap(),
            V::Boolean(left)
        );
        assert!(session.invoke_callable(name, &[V::Boolean(!left)]).is_err());
    }
    let checked = session.checked_source_package().unwrap();
    let artifacts = clause_package::lower_javascript_v1(&checked).unwrap();
    let dir = std::env::temp_dir().join(format!("clause-boolean-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("boolean.js"), artifacts.module).unwrap();
    std::fs::write(dir.join("boolean-effects.mjs"), "export const events=[]; export function trace(name,value){events.push(name);return value;}\n").unwrap();
    std::fs::write(dir.join("check.mjs"), r#"
import assert from 'node:assert/strict';
import * as f from './boolean.js';
import {events} from './boolean-effects.mjs';
for (const a of [false,true]) for (const b of [false,true]) {
  assert.equal(f['either-pressure'](a,b), a || b);
  assert.equal(f['exclusive-conflict'](a,b), a && b);
}
assert.equal(f.precedence(),true);
assert.equal(f.grouped(),false);
assert.equal(f.record(),true);
assert.equal(f.chained(),true);
assert.equal(f.names(),true);
for (const [x,expected] of [[0,true],[1,false],[2,true],[5,false]]) assert.equal(f.comparison(x),expected);
assert.equal(f.nested(true,false),'false');
assert.deepEqual(f.sequence(),[false,true]);
assert.equal(f['skip-and'](false),false);
assert.equal(f['skip-or'](true),true);
assert.throws(()=>f['skip-and'](true));
assert.throws(()=>f['skip-or'](false));
for (const [name,left,expected] of [['effect-and',false,['left']],['effect-and',true,['left','right']],['effect-or',true,['left']],['effect-or',false,['left','right']]]) {
  events.length=0;
  assert.equal(f[name](left),left);
  assert.deepEqual(events,expected);
}
"#).unwrap();
    let output = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(dir.join("check.mjs"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    for name in ["boolean.js", "boolean-effects.mjs", "check.mjs"] {
        std::fs::remove_file(dir.join(name)).unwrap();
    }
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn boolean_composition_rejects_non_boolean_operands_even_when_skipped() {
    for expression in [
        "1 and true",
        "true and 1",
        "false and 1",
        "1 or false",
        "false or 1",
        "true or 1",
        "true and",
        "false or",
        "true android()",
        "false origin()",
    ] {
        let source = format!("export invalid()\n  {expression}\n");
        assert!(
            ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),
            "accepted {expression}"
        );
    }
}

#[test]
fn boolean_composition_nix_construction_preserves_short_circuiting() {
    let source = format!(
        r#"{COUNTEREXAMPLE}
{COMPOSITION}
foreign target(?value: Bool): Bool
  construction: "nix"
  call: "id"
  from: "lib"
  failure: throw
export nix-check(): Sequence<Delayed<nix,Bool>>
  [target(either-pressure(false, true)), target(exclusive-conflict(true, false)), target(precedence()), target(grouped()), target(skip-and(false)), target(skip-or(true))]
"#
    );
    let session = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let checked = session.checked_source_package().unwrap();
    let callable = checked
        .callables
        .iter()
        .find(|c| c.designation == b"nix-check")
        .unwrap();
    let rendered = clause_package::render_nix_callable_v1(callable).unwrap();
    let output = Command::new("nix")
        .args([
            "eval",
            "--json",
            "--expr",
            &format!("({rendered}) {{ lib.id = x: x; }}"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "[true,false,true,false,false,true]"
    );
}
