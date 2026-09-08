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

#[test]
fn inferred_nested_records_preserve_the_complete_btop_contract_and_projection() {
    let original = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let original = original.checked_source_package().unwrap();
    let original = original.callables.iter().find(|c| c.designation == b"btop-module").unwrap();
    let inferred = include_bytes!("../../../test-vectors/authoring/foreign-construction-inferred.clause");
    let opened = ResidentSourceWorkbenchV1::open(inferred).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"btop-module").unwrap();
    assert_eq!(callable.result_kind, original.result_kind);
    assert_eq!(render_nix_callable_v1(callable).unwrap(), render_nix_callable_v1(original).unwrap());
    assert!(lower_javascript_v1(&checked).is_err());
    assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());

    let source = std::str::from_utf8(inferred).unwrap();
    for wrong in [
        source.replace("when-enabled(enabled(),", "when-enabled(true,"),
        source.replace("[btop()]", "[btop(), enable-option(\"wrong\")]"),
        source.replace("?condition: Delayed<nix,Bool>", "?condition: Delayed<other,Bool>"),
        source.replace("?body: Body", "?body: Sequence<Body>"),
        source.replace("failure: throw", "failure: ignore"),
        source.replace("construction: \"nix\"\n  get: \"btop\"", "construction: \"other\"\n  get: \"btop\""),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(wrong.as_bytes()).is_err(), "accepted wrong contract: {wrong}");
    }
    let strict = format!("{source}\nignore(?unused: Text)\n  btop-module()\n\nexport strict()\n  ignore(require(false, \"unused\", \"strict argument failed\"))\n");
    let opened = ResidentSourceWorkbenchV1::open(strict.as_bytes()).unwrap();
    let checked = opened.checked_source_package().unwrap();
    let callable = checked.callables.iter().find(|c| c.designation == b"strict").unwrap();
    assert_eq!(render_nix_callable_v1(callable).unwrap_err(), "strict argument failed");
}

const SHARED: &str = include_str!("../../../test-vectors/authoring/shared-foreign/nixpkgs.clause");
const BTOP: &str = include_str!("../../../test-vectors/authoring/shared-foreign/btop.clause");
const JQ: &str = include_str!("../../../test-vectors/authoring/shared-foreign/jq.clause");

fn imports(source: &str) -> CanonicalSourceImportsV1 {
    CanonicalSourceImportsV1::from([("nixpkgs.clause".into(), source.as_bytes().to_vec())])
}

#[test]
fn shared_foreign_declarations_preserve_both_exact_contracts_and_source_origins() {
    let inline = include_str!("../../../test-vectors/authoring/foreign-construction-inferred.clause");
    for (source, baseline, entry) in [
        (BTOP, inline.to_owned(), b"btop-module".as_slice()),
        (JQ, inline.replace("btop", "jq").replace("Enable jq system monitor", "jq command-line JSON processor"), b"jq-module".as_slice()),
    ] {
        let opened = ResidentSourceWorkbenchV1::open_with_imports(source.as_bytes(), imports(SHARED)).unwrap();
        let checked = opened.checked_source_package().unwrap();
        let callable = checked.callables.iter().find(|c| c.designation == entry).unwrap();
        let baseline = baseline
            .replace("{options: {myConfig: {modules: {btop: {enable: enable-option(\"Enable btop system monitor\")}}}},\n   config: when-enabled(enabled(), {environment: {systemPackages: [btop()]}})}", "module({myConfig: {modules: {btop: {enable: enable-option(\"Enable btop system monitor\")}}}}, enabled(), btop())")
            .replace("{options: {myConfig: {modules: {jq: {enable: enable-option(\"jq command-line JSON processor\")}}}},\n   config: when-enabled(enabled(), {environment: {systemPackages: [jq()]}})}", "module({myConfig: {modules: {jq: {enable: enable-option(\"jq command-line JSON processor\")}}}}, enabled(), jq())");
        let baseline = format!("{baseline}\nexport module<Options: Record>(?options: Options, ?enabled: Delayed<nix,Bool>, ?package: Package)\n  {{options: ?options, config: when-enabled(?enabled, {{environment: {{systemPackages: [?package]}}}})}}\n");
        let baseline = ResidentSourceWorkbenchV1::open(baseline.as_bytes()).unwrap().checked_source_package().unwrap();
        let baseline = baseline.callables.iter().find(|c| c.designation == entry).unwrap();
        assert_eq!(callable.result_kind, baseline.result_kind);
        assert_eq!(callable.expression, baseline.expression);
        assert_eq!(render_nix_callable_v1(callable).unwrap(), render_nix_callable_v1(baseline).unwrap());
        assert!(lower_javascript_v1(&checked).is_err());
        assert!(clause_runtime::lower_canonical_callable_v1(callable).is_err());
        let cst = read_canonical_source_with_imports_v1(source.as_bytes(), &imports(SHARED)).unwrap();
        let package = cst.declarations().find(|d| d.designation == b"Package").unwrap();
        assert_ne!(package.origin.artifact, cst.artifact());
        assert_eq!(cst.source_slice(package.origin).unwrap(), b"Package:\n  foreign: \"nixpkgs\"\n  type: \"Package\"");
        assert_eq!(print_canonical_source_v1(&cst).unwrap(), source.as_bytes());
    }
}

#[test]
fn shared_declarations_reject_missing_ambiguous_and_wrong_contracts() {
    assert!(ResidentSourceWorkbenchV1::open(BTOP.as_bytes()).is_err());
    for wrong in [
        BTOP.replace("when-enabled(enabled(),", "when-enabled(true,"),
        BTOP.replace("[btop()]", "[btop(), enable-option(\"wrong\")]"),
        format!("import \"nixpkgs.clause\"\n{BTOP}"),
        format!("{BTOP}\n{SHARED}"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open_with_imports(wrong.as_bytes(), imports(SHARED)).is_err(), "accepted {wrong}");
    }
    for wrong in [
        SHARED.replace("?condition: Delayed<nix,Bool>", "?condition: Delayed<other,Bool>"),
        SHARED.replace("?body: Body", "?body: Sequence<Body>"),
        SHARED.replace("failure: throw", "failure: ignore"),
        format!("{SHARED}\nexport extra(): Bool\n  true\n"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open_with_imports(BTOP.as_bytes(), imports(&wrong)).is_err(), "accepted {wrong}");
    }
}

#[test]
fn shared_foreign_sources_check_and_compile_through_the_file_commands() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/shared-foreign-proof");
    std::fs::create_dir_all(&output).unwrap();
    for name in ["btop", "jq"] {
        let source = root.join(format!("test-vectors/authoring/shared-foreign/{name}.clause"));
        let check = std::process::Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
            .arg("check-source").arg(&source).output().unwrap();
        assert!(check.status.success(), "{}", String::from_utf8_lossy(&check.stderr));
        let compile = std::process::Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
            .arg("compile-nix").arg(&source).arg(format!("{name}-module"))
            .arg(output.join(format!("{name}.nix"))).output().unwrap();
        assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    }
}
