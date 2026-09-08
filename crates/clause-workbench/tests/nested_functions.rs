use clause_package::{CanonicalSourceImportsV1, render_nix_callable_v1};
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

const SHARED: &str = include_str!("../../../test-vectors/authoring/nested-functions/nixpkgs.clause");
const FASTFETCH: &str = include_str!("../../../test-vectors/authoring/nested-functions/fastfetch.clause");
const TEALDEER: &str = include_str!("../../../test-vectors/authoring/nested-functions/tealdeer.clause");

fn open(source: &str) -> Result<ResidentSourceWorkbenchV1, clause_workbench::ResidentSourceWorkbenchErrorV1> {
    ResidentSourceWorkbenchV1::open_with_imports(source.as_bytes(), CanonicalSourceImportsV1::from([
        ("nixpkgs.clause".into(), SHARED.as_bytes().to_vec()),
    ]))
}

#[test]
fn nested_home_functions_preserve_actual_inner_scope() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/nested-functions/parity");
    fs::create_dir_all(&output).unwrap();
    for (name, source) in [("fastfetch", FASTFETCH), ("tealdeer", TEALDEER)] {
        let opened = open(source).unwrap();
        let checked = opened.checked_source_package().unwrap();
        let entry = format!("{name}-module");
        let callable = checked.callables.iter().find(|c| c.designation == entry.as_bytes()).unwrap();
        let rendered = render_nix_callable_v1(callable).unwrap();
        assert!(rendered.starts_with("{ config, lib, pkgs, ... }:\n"));
        fs::write(output.join(format!("{name}.nix")), rendered).unwrap();
        assert!(clause_package::lower_javascript_v1(&checked).is_err());
        assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());
    }
    let source = include_bytes!("../../../test-vectors/authoring/nested-functions/delayed-interpolation.clause");
    let opened = ResidentSourceWorkbenchV1::open(source).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"interpolate").unwrap();
    fs::write(output.join("interpolation.nix"), render_nix_callable_v1(callable).unwrap()).unwrap();
    let expression = format!(
        "import {} {{ generated = {}; nixpkgs = (builtins.getFlake {}).inputs.nixpkgs; }}",
        root.join("test-vectors/authoring/nested-functions/parity.nix").display(),
        output.display(),
        format!("\"git+file://{}\"", root.canonicalize().unwrap().display()),
    );
    let result = Command::new("nix").args(["eval", "--impure", "--json", "--expr", &expression]).output().unwrap();
    fs::write(output.join("parity.json"), &result.stdout).unwrap();
    fs::write(output.join("parity.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}

#[test]
fn target_functions_check_scope_argument_and_target_contracts() {
    for source in [
        FASTFETCH.replace("?home.config.lib.file.mkOutOfStoreSymlink(\n             \"{?home.config.home.homeDirectory}/code/nixos-config/dotfiles/fastfetch/config.jsonc\")", "?home.config.lib.file.mkOutOfStoreSymlink(true)"),
        FASTFETCH.replace("?home.config.home.homeDirectory", "?absent.config.home.homeDirectory"),
        FASTFETCH.replace("?home.config.home.homeDirectory", "?home.config.home.missing"),
        FASTFETCH.replace("Delayed<nix,HomeModuleArguments>", "HomeModuleArguments"),
        FASTFETCH.replace("Delayed<nix,HomeModuleArguments>", "Delayed<other,HomeModuleArguments>"),
    ] {
        assert!(open(&source).is_err(), "accepted wrong contract: {source}");
    }
}
