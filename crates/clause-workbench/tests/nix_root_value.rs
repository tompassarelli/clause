use clause_package::{CanonicalSourceImportsV1, render_nix_callable_v1};
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

const ROOT: &[u8] = include_bytes!("../../../test-vectors/authoring/foreign-root/root.clause");

#[test]
fn checked_root_values_and_quoted_root_members_remain_distinct() {
    let opened = ResidentSourceWorkbenchV1::open(ROOT).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let root = checked.callables.iter().find(|c| c.designation == b"root-path").unwrap();
    let member = checked.callables.iter().find(|c| c.designation == b"member-path").unwrap();
    let root_nix = render_nix_callable_v1(root).unwrap();
    let member_nix = render_nix_callable_v1(member).unwrap();
    assert!(root_nix.starts_with("{ flakeRoot, ... }:\n"));
    assert!(!root_nix.contains("flakeRoot."));
    assert!(member_nix.starts_with("{ config, ... }:\n"));
    assert!(member_nix.contains("config.\"root\""));
    let source = std::str::from_utf8(ROOT).unwrap();
    for wrong in [
        source.replace("get: root", "get: \"\""),
        source.replace("location(): Text", "location(?value: Text): Text"),
        source.replace("  construction: \"nix\"\n", ""),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err());
    }
}

#[test]
fn whole_aws_module_preserves_string_and_path_roots() {
    let source = include_bytes!("../../../test-vectors/authoring/foreign-root/awscli.clause");
    let shared = include_bytes!("../../../test-vectors/authoring/foreign-root/nixpkgs.clause");
    let opened = ResidentSourceWorkbenchV1::open_with_imports(source, CanonicalSourceImportsV1::from([("nixpkgs.clause".into(), shared.to_vec())])).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"awscli-module").unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let output = root.join("target/nix-root-value");
    let path_root = output.join("path-fixture");
    fs::create_dir_all(&path_root).unwrap();
    fs::write(path_root.join("fixture.txt"), "invented fixture content\n").unwrap();
    fs::write(output.join("awscli.nix"), render_nix_callable_v1(callable).unwrap()).unwrap();
    let expression = format!("import {} {{ generated = {}; pathRoot = {}; nixpkgs = (builtins.getFlake \"git+file://{}\").inputs.nixpkgs; }}", root.join("test-vectors/authoring/foreign-root/aws-parity.nix").display(), output.display(), path_root.display(), root.display());
    let result = Command::new("nix").args(["eval", "--impure", "--json", "--expr", &expression]).output().unwrap();
    fs::write(output.join("aws-parity.json"), &result.stdout).unwrap();
    fs::write(output.join("aws-parity.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}
