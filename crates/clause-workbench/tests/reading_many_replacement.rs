use std::process::Command;

use clause_package::{Term, decode_canonical_term_bytes, lower_javascript_v1};
use clause_runtime::ExecutableValueV1;
use clause_workbench::ResidentSourceWorkbenchV1;

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}

#[test]
fn declared_reading_replaces_one_member_in_native_and_javascript() {
    let source = format!(
        "{}\nbtop is included by tag \"utilities\"\n",
        include_str!("../../../test-vectors/authoring/reading-many-replacement.clause")
    );
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let artifacts = lower_javascript_v1(&workbench.checked_source_package().unwrap()).unwrap();
    let args = ["cli-tools", "monitoring"].map(|text| ExecutableValueV1::text(text).unwrap());
    let occurrence = workbench.handler_occurrence(b"retag", &args).unwrap();
    workbench
        .run_occurrences_to_candidate(&[occurrence])
        .unwrap();
    let projection = workbench.admit().unwrap().projection;
    let frame = decode_canonical_term_bytes(&projection.exact_term_bytes()).unwrap();
    let tags = clause_runtime::projected_relation_table_v1(field(
        field(&frame, b"relations"),
        b"included-by",
    ))
    .unwrap()
    .unwrap();
    let mut values = tags
        .rows()
        .values()
        .flatten()
        .map(|value| value.as_text().unwrap())
        .collect::<Vec<_>>();
    values.sort();
    assert_eq!(values, ["monitoring", "utilities"]);
    let wrong = workbench
        .handler_occurrence(
            b"retag",
            &[args[1].clone(), ExecutableValueV1::Boolean(true)],
        )
        .unwrap();
    assert!(workbench.run_occurrences_to_candidate(&[wrong]).is_err());
    assert!(workbench.pending_candidate().is_none());

    let directory =
        std::env::temp_dir().join(format!("clause-reading-many-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("reading.js"), artifacts.module).unwrap();
    std::fs::write(
        directory.join("check.mjs"),
        r#"
import assert from 'node:assert/strict';
import { createSession } from './reading.js';
const session = createSession();
assert.deepEqual([...session.read('btop', 'included-by')].sort(), ['cli-tools', 'utilities']);
session.handlers.retag('cli-tools', 'monitoring');
assert.deepEqual([...session.read('btop', 'included-by')].sort(), ['monitoring', 'utilities']);
assert.throws(() => session.handlers.retag('monitoring', true));
assert.deepEqual([...session.read('btop', 'included-by')].sort(), ['monitoring', 'utilities']);
"#,
    )
    .unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(directory.join("check.mjs"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    for file in ["reading.js", "check.mjs"] {
        std::fs::remove_file(directory.join(file)).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}
