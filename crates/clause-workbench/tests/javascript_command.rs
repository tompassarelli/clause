use std::path::PathBuf;
use std::process::Command;

#[test]
fn compiled_imports_resolve_relative_to_the_source_and_execute_in_bun() {
    let output_dir =
        std::env::temp_dir().join(format!("clause-js-imports-{}", std::process::id()));
    std::fs::create_dir(&output_dir).unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-vectors/authoring/javascript-imports/consumer.clause");
    let module = output_dir.join("consumer.js");
    let compiled = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
        .args(["compile-js".as_ref(), source.as_os_str(), module.as_os_str()])
        .current_dir(&output_dir)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    assert!(module.with_extension("d.ts").is_file());
    let runner = output_dir.join("run.mjs");
    std::fs::write(
        &runner,
        "import assert from 'node:assert/strict';\nimport { result } from './consumer.js';\nassert.equal(result(), 'kept');\n",
    )
    .unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(&runner)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    for path in [&module, &module.with_extension("d.ts"), &runner] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(output_dir).unwrap();
}

#[test]
fn compiled_callable_runs_in_bun_with_its_declared_public_name() {
    let output_dir = std::env::temp_dir().join(format!("clause-js-command-{}", std::process::id()));
    std::fs::create_dir(&output_dir).unwrap();
    let module = output_dir.join("missing-leaf.js");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-vectors/authoring/pure-callable.clause");
    let compiled = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
        .arg("compile-js")
        .arg(&source)
        .arg(&module)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    assert!(module.with_extension("d.ts").is_file());
    let runner = output_dir.join("compare.mjs");
    std::fs::write(
        &runner,
        r#"
import assert from 'node:assert/strict';
import { 'missing-leaf' as missingLeaf } from './missing-leaf.js';
function handwritten(node, edge, leaf, summary) {
  return `firn: '${node} ${edge}' requires a leaf node\n`
    + `Usage: firn ${node} ${edge} ${leaf}\n  ${summary}\n`;
}
const inputs = ['module', 'add', '<name>', 'Add a module'];
assert.equal(missingLeaf(...inputs), handwritten(...inputs));
assert.throws(() => missingLeaf(17, ...inputs.slice(1)));
"#,
    )
    .unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(&runner)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    for path in [&module, &module.with_extension("d.ts"), &runner] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(output_dir).unwrap();
}

#[test]
fn compiled_composition_preserves_strict_arguments_and_private_bindings() {
    let output_dir =
        std::env::temp_dir().join(format!("clause-js-composition-{}", std::process::id()));
    std::fs::create_dir(&output_dir).unwrap();
    for fixture in ["pure-composition", "pure-call-failure"] {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../test-vectors/authoring/{fixture}.clause"));
        let result = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
            .arg("compile-js")
            .arg(source)
            .arg(output_dir.join(format!("{fixture}.js")))
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    let runner = output_dir.join("compare.mjs");
    std::fs::write(&runner, r#"
import assert from 'node:assert/strict';
import * as composed from './pure-composition.js';
import { 'unused-failure' as unusedFailure } from './pure-call-failure.js';
assert.equal(composed['missing-command']('module', 'add'), "firn: 'module add' requires a leaf node\nUsage: firn module add <name>\n  Add a module\n");
assert.equal(composed['missing-command']('module', 'list'), '');
assert.equal(composed['missing-leaf'], undefined);
assert.throws(() => unusedFailure(0));
assert.equal(unusedFailure(2), 'unused');
"#).unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(&runner)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    for fixture in ["pure-composition", "pure-call-failure"] {
        std::fs::remove_file(output_dir.join(format!("{fixture}.js"))).unwrap();
        std::fs::remove_file(output_dir.join(format!("{fixture}.d.ts"))).unwrap();
    }
    std::fs::remove_file(runner).unwrap();
    std::fs::remove_dir(output_dir).unwrap();
}

#[test]
fn compiled_foreign_procedure_reads_real_argv_and_writes_stderr() {
    let output_dir =
        std::env::temp_dir().join(format!("clause-js-foreign-command-{}", std::process::id()));
    std::fs::create_dir(&output_dir).unwrap();
    let module = output_dir.join("foreign-cli.js");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-vectors/authoring/foreign-cli.clause");
    let compiled = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
        .arg("compile-js")
        .arg(source)
        .arg(&module)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let runner = output_dir.join("run.mjs");
    std::fs::write(
        &runner,
        "import { run } from './foreign-cli.js';\nprocess.exitCode = run();\n",
    )
    .unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(&runner)
        .args(["module", "add"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert_eq!(
        result.stderr,
        b"firn: 'module add' requires a leaf node\nUsage: firn module add <name>\n  scaffold a minimal module (.bnix + .nix)\n"
    );
    for path in [&module, &module.with_extension("d.ts"), &runner] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(output_dir).unwrap();
}
