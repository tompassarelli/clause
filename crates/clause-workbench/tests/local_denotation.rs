use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn local_denotations_preserve_marks_strictness_and_effect_order() {
    let source = include_bytes!("../../../test-vectors/authoring/local-denotation/local.clause");
    let local = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(local.invoke_callable(b"label", &[V::text("ABC").unwrap()]).unwrap(), V::text("abc/abc").unwrap());
    let strict = local.invoke_callable(b"strict", &[]).unwrap_err().to_string();
    assert!(strict.contains("first binding"), "{strict}");
    for wrong in [
        "export f()\n  ?x: ?missing\n  ?x\n",
        "export f()\n  ?x: ?x\n  ?x\n",
        "export f()\n  ?x: 1\n  ?x: 2\n  ?x\n",
        "export f(?x: Text)\n  ?x: 1\n  ?x\n",
        "export f()\n  ?x: 1\n",
        "export f()\n  ?x: 1 ?x\n",
        "private()\n  ?x: 1\n  ?x\nexport f()\n  ?x\n",
    ] { assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "{wrong}"); }
    let effects_source = include_bytes!("../../../test-vectors/authoring/local-denotation/effects.clause");
    let effects = ResidentSourceWorkbenchV1::open(effects_source).unwrap();
    assert!(ResidentSourceWorkbenchV1::open(std::str::from_utf8(effects_source).unwrap().replace("export procedure", "export").as_bytes()).is_err());
    let marks = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/local-denotation/mark-append.clause")).unwrap();
    let window = V::Record([(b"id".to_vec(), V::Number(1.0f64.to_bits())), (b"app-id".to_vec(), V::text("app").unwrap()), (b"title".to_vec(), V::text("title").unwrap())].into());
    let mark = |history, letter, timestamp| marks.invoke_callable(b"mark-window", &[history, V::text(letter).unwrap(), window.clone(), V::text(timestamp).unwrap()]).unwrap();
    let a = mark(V::Sequence(vec![]), "A", "first");
    let ab = mark(a, "B", "second");
    let updated = mark(ab, "a", "replacement");
    let expected = V::Sequence([("@mark-a", "replacement"), ("@mark-b", "second")].map(|(subject, timestamp)| V::Record([(b"subject".to_vec(), V::text(subject).unwrap()), (b"window".to_vec(), window.clone()), (b"marked-at".to_vec(), V::text(timestamp).unwrap())].into())).to_vec());
    assert_eq!(updated, expected);
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/local-denotation");
    fs::create_dir_all(&output).unwrap();
    for (name, opened) in [("local", &local), ("ordered", &effects), ("marks", &marks)] {
        fs::write(output.join(format!("{name}.mjs")), clause_package::lower_javascript_v1(&opened.checked_source_package().unwrap()).unwrap().module).unwrap();
    }
    fs::write(output.join("effects.mjs"), "export const calls=[]; export function step(label){calls.push(label);return label;}").unwrap();
    fs::write(output.join("check.mjs"), r#"import assert from 'node:assert/strict';
import {label,strict} from './local.mjs';
import {ordered} from './ordered.mjs';
import {calls} from './effects.mjs';
import {'mark-window' as mark} from './marks.mjs';
assert.equal(label('ABC'),'abc/abc'); assert.throws(strict,/first binding/);
assert.equal(ordered(),'first/first'); assert.deepEqual(calls,['first','second']);
const w={id:1,'app-id':'app',title:'title'};
assert.deepEqual(mark(mark(mark([],'A',w,'first'),'B',w,'second'),'a',w,'replacement'),[
{subject:'@mark-a',window:w,'marked-at':'replacement'}, {subject:'@mark-b',window:w,'marked-at':'second'}]);
"#).unwrap();
    let result = Command::new(std::env::var("BUN").unwrap()).arg("check.mjs").current_dir(output).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}
