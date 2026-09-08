use clause_workbench::ResidentSourceWorkbenchV1;
use clause_runtime::ExecutableValueV1;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn multiline_callable_keeps_literal_layout_in_native_and_bun() {
    let source = include_bytes!("../../../test-vectors/authoring/multiline-callable.clause");
    let opened = ResidentSourceWorkbenchV1::open(source).unwrap();
    let expected = "echo \"ready\" # retained\n  printf '%s\\n' \"${HOME:-}\" \"{?literal}\" \"$@\"\n\ncase \"$1\" in *) echo '\\path' ;; esac\n";
    assert_eq!(opened.invoke_callable(b"script", &[]).unwrap(), ExecutableValueV1::text(expected).unwrap());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/multiline-callable");
    fs::create_dir_all(&output).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let generated = clause_package::lower_javascript_v1(&checked).unwrap();
    fs::write(output.join("literal.mjs"), generated.module).unwrap();
    let bun = std::env::var("BUN").unwrap_or_else(|_| "bun".into());
    let result = Command::new(bun).args(["-e", "import {script,wrapped} from './literal.mjs'; if (wrapped().script !== script()) throw Error('composition mismatch'); process.stdout.write(script());"]).current_dir(&output).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.stdout, expected.as_bytes());
}
