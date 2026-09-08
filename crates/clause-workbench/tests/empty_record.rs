use clause_package::{CanonicalSourceImportsV1, render_nix_callable_v1};
use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

fn output() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/empty-record");
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn empty_record_contracts_agree_in_native_and_bun() {
    let source = include_bytes!("../../../test-vectors/authoring/empty-record/value.clause");
    let opened = ResidentSourceWorkbenchV1::open(source).unwrap();
    assert_eq!(opened.invoke_callable(b"specimen", &[]).unwrap(), V::Record(Default::default()));
    assert!(opened.invoke_callable(b"empty", &[V::text("wrong").unwrap()]).is_err());
    let checked = opened.checked_source_package().unwrap();
    let generated = clause_package::lower_javascript_v1(&checked).unwrap();
    let output = output();
    fs::write(output.join("value.mjs"), generated.module).unwrap();
    let bun = std::env::var("BUN").unwrap_or_else(|_| "bun".into());
    let result = Command::new(bun).args(["-e", "import {specimen,empty} from './value.mjs'; if (JSON.stringify(specimen()) !== '{}') throw Error('empty record mismatch'); let rejected=false; try { empty('wrong'); } catch { rejected=true; } if (!rejected) throw Error('invalid argument accepted');"]).current_dir(output).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}

#[test]
fn clipboard_preserves_inner_function_and_outer_packages() {
    let source = include_bytes!("../../../test-vectors/authoring/empty-record/clipboard-tools.clause");
    let shared = include_bytes!("../../../test-vectors/authoring/empty-record/nixpkgs.clause");
    let opened = ResidentSourceWorkbenchV1::open_with_imports(source, CanonicalSourceImportsV1::from([("nixpkgs.clause".into(), shared.to_vec())])).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"clipboard-tools-module").unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let output = output().canonicalize().unwrap();
    fs::write(output.join("clipboard.nix"), render_nix_callable_v1(callable).unwrap()).unwrap();
    let expression = format!("import {} {{ generated = {}; nixpkgs = (builtins.getFlake \"git+file://{}\").inputs.nixpkgs; }}", root.join("test-vectors/authoring/empty-record/clipboard-parity.nix").display(), output.join("clipboard.nix").display(), root.display());
    let result = Command::new("nix").args(["eval", "--impure", "--json", "--expr", &expression]).output().unwrap();
    fs::write(output.join("clipboard-parity.json"), &result.stdout).unwrap();
    fs::write(output.join("clipboard-parity.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}
