use clause_runtime::ExecutableValueV1;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/callable-outcomes.clause");
fn text(value: &str) -> ExecutableValueV1 { ExecutableValueV1::text(value).unwrap() }

#[test]
fn execution_and_diagnostic_alternatives_run_through_checked_consumers() {
    let source = [SOURCE, br#"
export render(?outcome: Execution | Diagnostic): Text
  match(?outcome, ?request: Execution => ?request.executable, ?error: Diagnostic => ?error.message)

export guarded(?value: Text | F64): Text
  match(?value, ?message: Text => ?message, ?number: F64 => require(false, "", "selected failure"))
"#].concat();
    let session = ResidentSourceWorkbenchV1::open(&source).unwrap();
    for (args, expected) in [
        (vec![text("module"), text("list")], "firn-inventory module list all"),
        (vec![text("unknown")], "1: Unknown command\n"),
    ] {
        assert_eq!(session.invoke_callable(b"describe", &[ExecutableValueV1::Sequence(args)]).unwrap(), text(expected));
    }
    assert_eq!(session.invoke_callable(b"guarded", &[text("kept")]).unwrap(), text("kept"));
    assert!(session.invoke_callable(b"guarded", &[ExecutableValueV1::number(1.0).unwrap()]).is_err());
    let request = session.invoke_callable(b"dispatch", &[ExecutableValueV1::Sequence(vec![text("module"), text("list")])]).unwrap();
    assert_eq!(session.invoke_callable(b"render", &[request]).unwrap(), text("firn-inventory"));
    assert!(session.invoke_callable(b"render", &[ExecutableValueV1::Boolean(false)]).is_err());
    let checked = session.checked_source_package().unwrap();
    let javascript = clause_package::lower_javascript_v1(&checked).unwrap();
    let directory = std::env::temp_dir().join(format!("clause-outcomes-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let module = directory.join("outcomes.mjs");
    std::fs::write(&module, javascript.module).unwrap();
    let test = directory.join("run.mjs");
    std::fs::write(&test, r#"
import {describe, dispatch, render, guarded} from './outcomes.mjs';
import assert from 'node:assert/strict';
assert.equal(describe(['module', 'list']), 'firn-inventory module list all');
assert.equal(describe(['unknown']), '1: Unknown command\n');
assert.deepEqual(dispatch(['module', 'list']), {executable:'firn-inventory',arguments:['module','list','all']});
assert.deepEqual(dispatch(['unknown']), {message:'Unknown command\n',status:1});
assert.equal(render(dispatch(['module','list'])), 'firn-inventory');
assert.equal(guarded('kept'), 'kept');
assert.throws(()=>guarded(1));
for(const invalid of [false, {message:'bad',status:'1'}, {message:'bad',status:1,extra:true}, {executable:'bad',arguments:[1]}]) assert.throws(()=>render(invalid));
"#).unwrap();
    let result = std::process::Command::new("bun").arg(&test).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    std::fs::remove_file(module).unwrap();
    std::fs::remove_file(test).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn alternatives_reject_missing_unreachable_mistyped_and_unbound_cases() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    let missing = source.replace(",\n    ?error: Diagnostic => \"{?error.status}: {?error.message}\"", "");
    let duplicate = source.replace("?error: Diagnostic", "?error: Execution");
    let unreachable = source.replace("?error: Diagnostic", "?error: Text");
    let field = source.replace("?request.executable} {", "?request.message} {");
    let payload = source.replace("status: 1}", "status: \"wrong\"}");
    let unbound = source.replace("{?error.status}", "{?request.executable}");
    let no_contract = source.replace(": Execution | Diagnostic\n", "\n");
    for invalid in [missing, duplicate, unreachable, field, payload, unbound, no_contract] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err(), "accepted {invalid}");
    }
    for invalid in [
        "export f(?value: Text | Text): Text\n  \"no\"\n",
        "export f(?value: Sequence<Text> | Sequence<F64>): Text\n  \"no\"\n",
        "export f(?value: Text): Text\n  match(?value, ?text: Text => ?text)\n",
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err(), "accepted {invalid}");
    }
}
