use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

fn text(value: &str) -> V { V::text(value).unwrap() }
fn number(value: f64) -> V { V::Number(value.to_bits()) }

#[test]
fn finite_collections_execute_padding_and_title_distance_in_native_and_bun() {
    let policy = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/finite-iteration/window-text.clause")).unwrap();
    let operations = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/finite-iteration/operations.clause")).unwrap();
    let effects = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/finite-iteration/effects.clause")).unwrap();
    use clause_runtime::{ExecutableExpressionV1 as E, ExecutableProgramV1, ExecutableRuleV1, decode_executable_physical_plan_v1, encode_executable_physical_plan_v1};
    let mut plan = decode_executable_physical_plan_v1(&operations.generation().cpp1).unwrap();
    plan.input = None;
    plan.source_metadata = None;
    plan.program = ExecutableProgramV1 {
        initial_configuration: vec![number(0.0)],
        rules: vec![ExecutableRuleV1 { entry: 0, predicates: vec![], required_present: vec![], required_absent: vec![], removals: vec![],
            assignments: vec![(0, E::SequenceAt(Box::new(E::SequenceRange(Box::new(E::Constant(number(4.0))))), Box::new(E::Constant(number(2.0)))))],
        }].into(),
        projection: None,
    };
    assert_eq!(decode_executable_physical_plan_v1(&encode_executable_physical_plan_v1(&plan).unwrap()).unwrap(), plan);
    let mut js = String::from("import assert from 'node:assert/strict';\nimport * as p from './policy.mjs';\nimport * as o from './operations.mjs';\nimport * as e from './ordered.mjs';\nimport {calls} from './effects.mjs';\n");
    for (value, width, expected) in [("", 3.0, "   "), ("app", 5.0, "app  "), ("already long", 3.0, "already long"), ("x", -1.0, "x"), ("é", 3.0, "é  "), ("😀", 3.0, "😀  ")] {
        assert_eq!(policy.invoke_callable(b"pad-right", &[text(value), number(width)]).unwrap(), text(expected));
        js.push_str(&format!("assert.equal(p['pad-right']({value:?},{width}),{expected:?});\n"));
    }
    for (left, right, expected) in [("", "", 0.0), ("", "abc", 3.0), ("abc", "", 3.0), ("kitten", "sitting", 3.0), ("Window title", "window title", 1.0), ("same", "same", 0.0), ("é", "e", 1.0), ("😀a", "😀b", 1.0)] {
        assert_eq!(policy.invoke_callable(b"title-distance", &[text(left), text(right)]).unwrap(), number(expected));
        js.push_str(&format!("assert.equal(p['title-distance']({left:?},{right:?}),{expected});\n"));
    }
    for (left, right, expected) in [("", "", 1.0), ("TITLE", "title", 1.0), ("abc", "axc", 1.0 - 1.0 / 3.0)] {
        assert_eq!(policy.invoke_callable(b"similarity", &[text(left), text(right)]).unwrap(), number(expected));
        js.push_str(&format!("assert.equal(p.similarity({left:?},{right:?}),{expected});\n"));
    }
    assert_eq!(operations.invoke_callable(b"integers", &[number(0.0)]).unwrap(), V::Sequence(vec![]));
    assert_eq!(operations.invoke_callable(b"integers", &[number(4.0)]).unwrap(), V::Sequence((0..4).map(|i| number(i as f64)).collect()));
    assert_eq!(operations.invoke_callable(b"indexed", &[V::Sequence(vec![text("a"), text("b")]), number(1.0)]).unwrap(), text("b"));
    let record = V::Record([(b"label".to_vec(), text("kept"))].into());
    assert_eq!(operations.invoke_callable(b"entry", &[V::Sequence(vec![record.clone()]), number(0.0)]).unwrap(), record);
    for invalid in [-1.0, 1.5, f64::INFINITY, f64::NAN, 65536.0] {
        assert!(operations.invoke_callable(b"integers", &[number(invalid)]).is_err());
    }
    for invalid in [-1.0, 0.5, 1.0, f64::INFINITY, f64::NAN] {
        assert!(operations.invoke_callable(b"indexed", &[V::Sequence(vec![text("a")]), number(invalid)]).is_err());
    }
    for (entry, message) in [("strict-range", "range end evaluated"), ("strict-index", "sequence evaluated first")] {
        let error = operations.invoke_callable(entry.as_bytes(), &[]).unwrap_err().to_string();
        assert!(error.contains(message), "{error}");
    }
    for source in ["export f()\n  range(true)\n", "export f()\n  range(1, 2)\n", "export f()\n  at(\"text\", 0)\n", "export f()\n  at([1], true)\n", "export f(): Text\n  at([1], 0)\n"] {
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{source}");
    }
    js.push_str("assert.deepEqual(o.integers(0),[]); assert.deepEqual(o.integers(4),[0,1,2,3]); assert.equal(o.indexed(['a','b'],1),'b'); assert.deepEqual(o.entry([{label:'kept'}],0),{label:'kept'});\nfor(const n of [-1,1.5,Infinity,NaN,65536])assert.throws(()=>o.integers(n));\nfor(const n of [-1,.5,1,Infinity,NaN])assert.throws(()=>o.indexed(['a'],n));\nassert.throws(()=>o['strict-range'](),/range end evaluated/); assert.throws(()=>o['strict-index'](),/sequence evaluated first/);\nassert.deepEqual(e.counted(),[0,1,2]); assert.equal(e.selected(),'b'); assert.deepEqual(calls,['end','values','index']);\n");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/finite-iteration");
    fs::create_dir_all(&output).unwrap();
    for (name, opened) in [("policy", &policy), ("operations", &operations), ("ordered", &effects)] {
        fs::write(output.join(format!("{name}.mjs")), clause_package::lower_javascript_v1(&opened.checked_source_package().unwrap()).unwrap().module).unwrap();
    }
    fs::write(output.join("effects.mjs"), "export const calls=[];export function end(){calls.push('end');return 3;}export function values(){calls.push('values');return ['a','b'];}export function index(){calls.push('index');return 1;}").unwrap();
    fs::write(output.join("check.mjs"), js).unwrap();
    let result = Command::new(std::env::var("BUN").unwrap()).arg("check.mjs").current_dir(&output).output().unwrap();
    fs::write(output.join("bun.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}
