use clause_package::*;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/foreign-construction.clause");

#[test]
fn staged_foreign_construction_preserves_checked_contracts_and_strict_bindings() {
    let opened = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"btop-module").unwrap();
    let rendered = render_nix_callable_v1(callable).unwrap();
    assert!(rendered.contains("lib.\"mkIf\""));
    assert!(rendered.contains("config.\"myConfig\".\"modules\".\"btop\".\"enable\""));
    assert!(rendered.contains("pkgs.\"btop\""));
    assert!(lower_javascript_v1(&checked).is_err());
    assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());

    let source = std::str::from_utf8(SOURCE).unwrap();
    for wrong in [
        source.replace("when-enabled(enabled(),", "when-enabled(true,"),
        source.replace("Sequence<Delayed<nix,Package>>", "Sequence<Delayed<nix,Option>>"),
        source.replace("?condition: Delayed<nix,Bool>", "?condition: Delayed<other,Bool>"),
        source.replace("enable: Delayed<nix,Option>", "enable: Option"),
        source.replace("get: \"btop\"", "call: \"btop\"").replace("foreign btop(): Package", "foreign btop(?x: F64): Package"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "accepted wrong contract: {wrong}");
    }
    let strict = format!("{source}\nignore(?unused: Text): Module\n  btop-module()\n\nexport strict(): Module\n  ignore(require(false, \"unused\", \"strict argument failed\"))\n");
    let opened = ResidentSourceWorkbenchV1::open(strict.as_bytes()).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"strict").unwrap();
    assert_eq!(render_nix_callable_v1(callable).unwrap_err(), "strict argument failed");
}
