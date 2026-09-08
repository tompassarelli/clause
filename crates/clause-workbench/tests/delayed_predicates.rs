use clause_package::{CanonicalSourceImportsV1, render_nix_callable_v1};
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn target_predicates_are_lazy_and_whole_gtk_matches_nix() {
    let source = include_str!("../../../test-vectors/authoring/delayed-predicates/predicates.clause");
    let opened = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let checked = opened.checked_source_package().unwrap();
    assert!(clause_package::lower_javascript_v1(&checked).is_err());
    assert!(opened.invoke_callable(b"mode", &[]).is_err());
    for wrong in [
        source.replace("polarity() = \"dark\"", "polarity() = 1"),
        source.replace("if(polarity() = \"dark\", \"1\", \"0\")", "if(polarity() = \"dark\", \"1\", 0)"),
        source.replace("foreign selected(): Text\n  construction: \"nix\"", "foreign selected(): Text\n  construction: \"other\""),
        source.replace("polarity() = \"dark\"", "polarity() = selected()").replace("foreign selected(): Text\n  construction: \"nix\"", "foreign selected(): Text\n  construction: \"other\""),
    ] { assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "{wrong}"); }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let output = root.join("target/delayed-predicates");
    fs::create_dir_all(&output).unwrap();
    for callable in &checked.callables {
        fs::write(output.join(format!("{}.nix", std::str::from_utf8(&callable.designation).unwrap())), render_nix_callable_v1(callable).unwrap()).unwrap();
    }
    let gtk = ResidentSourceWorkbenchV1::open_with_imports(
        include_bytes!("../../../test-vectors/authoring/delayed-predicates/gtk.clause"),
        CanonicalSourceImportsV1::from([("nixpkgs.clause".into(), include_bytes!("../../../test-vectors/authoring/delayed-predicates/nixpkgs.clause").to_vec())]),
    ).unwrap();
    let gtk = gtk.checked_source_package().unwrap();
    let gtk = gtk.callables.iter().find(|c| c.designation == b"gtk-module").unwrap();
    fs::write(output.join("gtk.nix"), render_nix_callable_v1(gtk).unwrap()).unwrap();
    let expression = format!(r#"let
      dark = import {out}/dark.nix;
      mode = import {out}/mode.nix;
      lazy = import {out}/lazy.nix;
      in assert dark {{ config.polarity = "dark"; }};
      assert !(dark {{ config.polarity = "light"; }});
      assert mode {{ config.polarity = "dark"; }} == "1";
      assert mode {{ config.polarity = "light"; }} == "0";
      assert lazy {{ config = {{ polarity = "dark"; selected = "yes"; unselected = throw "unselected forced"; }}; }} == "yes";
      assert lazy {{ config = {{ polarity = "light"; selected = throw "unselected forced"; unselected = "no"; }}; }} == "no";
      import {root}/test-vectors/authoring/delayed-predicates/gtk-parity.nix {{ generated = {out}/gtk.nix; nixpkgs = (builtins.getFlake "git+file://{root}").inputs.nixpkgs; }}
    "#, out = output.display(), root = root.display());
    fs::write(output.join("check.nix"), &expression).unwrap();
    let result = Command::new("nix").args(["eval", "--impure", "--json", "--expr", &expression]).output().unwrap();
    fs::write(output.join("parity.json"), &result.stdout).unwrap();
    fs::write(output.join("parity.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}
