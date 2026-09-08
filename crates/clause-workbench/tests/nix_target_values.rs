use clause_package::*;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::collections::BTreeSet;

const SHARED: &[u8] = include_bytes!("../../../test-vectors/authoring/nested-functions/nixpkgs.clause");
const GHOSTTY: &str = include_str!("../../../test-vectors/authoring/target-values/ghostty.clause");
const MICROPHONE: &str = include_str!("../../../test-vectors/authoring/target-values/framework13-mic.clause");

fn open(source: &str) -> ResidentSourceWorkbenchV1 {
    ResidentSourceWorkbenchV1::open_with_imports(source.as_bytes(), CanonicalSourceImportsV1::from([
        ("../nested-functions/nixpkgs.clause".into(), SHARED.to_vec()),
    ])).unwrap()
}

#[test]
fn target_alternatives_keep_foreign_identity_and_exact_contracts() {
    use CanonicalValueTypeV1 as T;
    let source = format!("{GHOSTTY}\nexport choice(): Delayed<nix,Null | Package>\n  if(darwin(), nix-null(), package(path(unstable.ghostty)))\n\nexport nested()\n  if(darwin(), choice(), nix-null())\n\nexport immediate()\n  if(true, nix-null(), package(path(unstable.ghostty)))\n");
    let opened = open(&source);
    let checked = opened.checked_source_package().unwrap();
    let expected = T::Delayed { target: "nix".into(), value: Box::new(T::Alternatives(BTreeSet::from([
        T::OpaqueForeign { module: "nix".into(), name: "Null".into() },
        T::OpaqueForeign { module: "nixpkgs".into(), name: "Package".into() },
    ]))) };
    for entry in [b"choice".as_slice(), b"nested", b"immediate"] {
        let callable = checked.callables.iter().find(|c| c.designation == entry).unwrap();
        assert_eq!(callable.result_kind, expected);
        assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());
        let rendered = render_nix_callable_v1(callable).unwrap();
        if entry == b"immediate" { assert_eq!(rendered, "{ ... }:\nbuiltins.\"null\"\n"); }
        else { assert!(rendered.contains("then builtins.\"null\" else pkgs.\"unstable\".\"ghostty\"")); }
    }
    assert!(lower_javascript_v1(&checked).is_err());
    for wrong in [
        source.replace("Delayed<nix,Null | Package>", "Delayed<nix,Package>"),
        source.replace("  construction: \"nix\"\n  get: \"null\"", "  construction: \"other\"\n  get: \"null\""),
        source.replace("Delayed<nix,Null | Package>", "Null | Package"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open_with_imports(wrong.as_bytes(), CanonicalSourceImportsV1::from([
            ("../nested-functions/nixpkgs.clause".into(), SHARED.to_vec()),
        ])).is_err(), "accepted invalid target alternatives");
    }
    let delayed_opaque = T::Delayed { target: "nix".into(), value: Box::new(T::OpaqueForeign { module: "nix".into(), name: "Null".into() }) };
    assert!(T::Alternatives(BTreeSet::from([delayed_opaque, CanonicalScalarValueKindV1::Text.into()])).check().is_err());
}

#[test]
fn whole_modules_construct_without_forcing_target_branches_or_rebasing_paths() {
    for (source, entry) in [(GHOSTTY, b"ghostty-module".as_slice()), (MICROPHONE, b"framework13-mic-module".as_slice())] {
        let checked = open(source).checked_source_package().unwrap();
        let callable = checked.callables.iter().find(|c| c.designation == entry).unwrap();
        let rendered = render_nix_callable_v1(callable).unwrap();
        assert!(rendered.starts_with("{ config, lib, pkgs, ... }:\n"));
        if entry == b"ghostty-module" {
            assert!(rendered.contains("then builtins.\"null\" else pkgs.\"unstable\".\"ghostty\""));
        } else {
            assert!(rendered.contains("builtins.\"readFile\" (./firn-mic)"));
        }
    }
}

#[test]
fn relative_roots_reject_expression_text_and_non_root_operations() {
    for path in ["../firn-mic", "./scripts/firn-mic"] {
        let checked = open(&MICROPHONE.replace("./firn-mic", path)).checked_source_package().unwrap();
        let callable = checked.callables.iter().find(|c| c.designation == b"framework13-mic-module").unwrap();
        assert!(render_nix_callable_v1(callable).unwrap().contains(&format!("builtins.\"readFile\" ({path})")));
    }
    for source in [
        MICROPHONE.replace("./firn-mic", "./firn-mic; builtins.abort"),
        MICROPHONE.replace("./firn-mic", "./"),
        MICROPHONE.replace("./firn-mic", "./firn//mic"),
        MICROPHONE.replace("./firn-mic", "/absolute/firn-mic"),
        MICROPHONE.replace("get: root", "get: \"name\""),
    ] {
        let checked = open(&source).checked_source_package().unwrap();
        let callable = checked.callables.iter().find(|c| c.designation == b"framework13-mic-module").unwrap();
        assert!(render_nix_callable_v1(callable).is_err(), "accepted invalid relative root");
    }
}
