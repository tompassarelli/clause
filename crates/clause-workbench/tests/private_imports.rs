use clause_package::{CanonicalSourceImportsV1, read_canonical_source_with_imports_v1, print_canonical_source_v1};
use clause_runtime::ExecutableValueV1;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::path::Path;
use std::process::Command;

const LIBRARY: &[u8] = include_bytes!("../../../test-vectors/authoring/private-imports/library.clause");
const CONSUMER: &[u8] = include_bytes!("../../../test-vectors/authoring/private-imports/consumer.clause");

fn imports() -> CanonicalSourceImportsV1 {
    CanonicalSourceImportsV1::from([("library.clause".into(), LIBRARY.to_vec())])
}

#[test]
fn private_helpers_keep_module_scope_and_do_not_leak_or_collide() {
    let opened = ResidentSourceWorkbenchV1::open_with_imports(CONSUMER, imports()).unwrap();
    assert_eq!(opened.invoke_callable(b"result", &[]).unwrap(), ExecutableValueV1::text("library:kept/consumer:local").unwrap());
    let cst = read_canonical_source_with_imports_v1(CONSUMER, &imports()).unwrap();
    assert_eq!(print_canonical_source_v1(&cst).unwrap(), CONSUMER);
    let checked = opened.checked_source_package().unwrap();
    assert_eq!(checked.callables.iter().filter(|c| c.designation == b"finish").count(), 1);
    let imported = checked.callables.iter().find(|c| c.designation == b"imported").unwrap();
    assert!(cst.source_slice(imported.origin).unwrap().starts_with(b"export imported"));
    assert!(!checked.callables.iter().any(|c| c.designation == b"wrap"));

    let sibling = b"finish(?value: Text): Text\n  \"sibling:{?value}\"\n\nexport sibling(): Text\n  finish(\"kept\")\n";
    let mut supplied = imports();
    supplied.insert("sibling.clause".into(), sibling.to_vec());
    let source = b"import \"library.clause\"\nimport \"sibling.clause\"\n\nexport result(): Text\n  imported(\"kept\").value ++ \"/\" ++ sibling()\n";
    let opened = ResidentSourceWorkbenchV1::open_with_imports(source, supplied).unwrap();
    assert_eq!(opened.invoke_callable(b"result", &[]).unwrap(), ExecutableValueV1::text("library:kept/sibling:kept").unwrap());
    assert!(!opened.checked_source_package().unwrap().callables.iter().any(|c| c.designation == b"finish"));

    for expression in ["finish(\"escaped\")", "wrap({text: \"escaped\"}).value"] {
        let source = format!("import \"library.clause\"\nexport escaped(): Text\n  {expression}\n");
        assert!(ResidentSourceWorkbenchV1::open_with_imports(source.as_bytes(), imports()).is_err());
    }
    let wrong = String::from_utf8(LIBRARY.to_vec()).unwrap().replace("finish(?value: Text): Text\n  \"library:{?value}\"", "finish(?value: Text): Text\n  consumer-only(?value)");
    let supplied = CanonicalSourceImportsV1::from([("library.clause".into(), wrong.into_bytes())]);
    let source = b"import \"library.clause\"\nconsumer-only(?value: Text): Text\n  ?value\nexport result(): Text\n  imported(\"kept\").value\n";
    assert!(ResidentSourceWorkbenchV1::open_with_imports(source, supplied).is_err());
}

#[test]
fn imported_private_dependencies_compile_through_file_commands_and_execute_in_bun() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/private-import-proof");
    std::fs::create_dir_all(&output).unwrap();
    let source = root.join("test-vectors/authoring/private-imports/consumer.clause");
    let module = output.join("consumer.js");
    for arguments in [vec!["check-source".as_ref(), source.as_os_str()],
        vec!["compile-js".as_ref(), source.as_os_str(), module.as_os_str()]] {
        let result = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
            .args(arguments).current_dir(&output).output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    let runner = output.join("run.mjs");
    std::fs::write(&runner, r#"
import assert from 'node:assert/strict';
import * as module from './consumer.js';
assert.equal(module.result(), 'library:kept/consumer:local');
assert.equal(module.finish, undefined);
assert.equal(module.wrap, undefined);
assert.deepEqual(module.imported('direct'), {value: 'library:direct'});
"#).unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(&runner).output().unwrap();
    assert!(result.status.success(), "{result:?}");
}
