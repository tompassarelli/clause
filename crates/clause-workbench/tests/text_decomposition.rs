use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/text-decomposition.clause");
fn text(value: &str) -> V { V::text(value).unwrap() }
fn texts(values: &[&str]) -> V { V::Sequence(values.iter().map(|v| text(v)).collect()) }

// The same authored cases drive native invocation and generated JavaScript.
fn cases() -> Vec<(&'static str, Vec<V>, V)> {
    let mut cases = vec![
        ("letters", vec![text("a🚀e\u{301}\0")], texts(&["a", "🚀", "e", "\u{301}", "\0"])),
        ("letters", vec![text("")], texts(&[])),
        ("parts", vec![text("/a//"), text("/")], texts(&["", "a", "", ""])),
        ("parts", vec![text("a.*b.*"), text(".*")], texts(&["a", "b", ""])),
        ("parts", vec![text(""), text("/")], texts(&[""])),
        ("parts", vec![text("🚀a"), text("")], texts(&["🚀", "a"])),
        ("short-host", vec![text("whiterabbit.example.test")], text("whiterabbit")),
        ("short-host", vec![text(".example")], text("")),
        ("short-host", vec![text("whiterabbit")], text("whiterabbit")),
        ("main-candidates", vec![text("/srv/firn/worktrees/prewarm"), text("/srv/firn/main/.git")], texts(&["/srv/firn/main", "/srv/firn/worktrees/prewarm"])),
        ("main-candidates", vec![text("/srv/firn/pins/rev"), text("/other/.git")], texts(&["/srv/firn/main", "/other", "/srv/firn/pins/rev"])),
        ("main-candidates", vec![text("/srv/main"), text("/srv/main/.git")], texts(&["/srv/main"])),
        ("layout-main", vec![text("/a/pins/b/worktrees/c/worktrees/d")], text("/a/pins/b/main")),
        ("common-main", vec![text("/a/.git/b/.git")], text("/a/.git/b")),
        ("common-main", vec![text("/a/.git/")], text("")),
        ("delimited", vec![text("a\n\nb\n"), text("\n")], texts(&["a", "", "b"])),
        ("delimited", vec![text("a\n\n"), text("\n")], texts(&["a", ""])),
        ("delimited", vec![text(""), text("\n")], texts(&[])),
        ("command-argv", vec![text("firn\0\0host\0rebuild\0")], texts(&["firn", "", "host", "rebuild"])),
        ("command-argv", vec![text("a|b||")], texts(&["a", "b", ""])),
        ("command-argv", vec![text("a🚀b🚀")], texts(&["a", "b"])),
        ("command-argv", vec![text("")], texts(&[])),
        ("trim-line-ending", vec![text(" key \r\n\r")], text(" key ")),
        ("trim-line-ending", vec![text("a\r\nb\t \n")], text("a\r\nb\t ")),
        ("trim-line-ending", vec![text("\r\n")], text("")),
        ("trim-line-ending", vec![text("a\n ")], text("a\n ")),
    ];
    for (input, expected) in [("42", Some(42.0)), (" \t+0042tail", Some(42.0)),
        ("-12rest", Some(-12.0)), ("0x12", Some(0.0)), ("3.5", Some(3.0)),
        ("\u{feff}7", Some(7.0)), ("\u{85}7", None), ("", None), ("+", None),
        ("not a pid", None), ("-0", Some(0.0))] {
        cases.push(("integer", vec![text(input)], expected.map(|n| V::number(n).unwrap()).unwrap_or_else(|| text(input))));
    }
    for (input, expected) in [("12\n34\nkey\n", true), (" +12pid\n34\nkey", true),
        ("12\n\nkey\n", false), ("12\n34\n\n", false), ("0\n34\nkey\n", false),
        ("-1\n34\nkey\n", false), ("x\n34\nkey\n", false), ("12\n34\nkey\n\n", false)] {
        cases.push(("lease-valid", vec![text(input)], V::Boolean(expected)));
    }
    for (input, expected) in [("aaa bbb refs/heads/main", true), ("aaa bbb refs/heads/main ", true),
        ("aaa  bbb refs/heads/main", false), ("aaa 000000 refs/heads/main", false),
        ("aaa aaa refs/heads/main", false), ("aaa bbb refs/heads/side", false),
        ("aaa\tbbb refs/heads/main", false)] {
        cases.push(("main-update", vec![text(input)], V::Boolean(expected)));
    }
    cases
}

fn javascript(value: &V) -> String {
    match value {
        V::Text(value) => format!("{:?}", value.as_str()),
        V::Number(bits) => f64::from_bits(*bits).to_string(),
        V::Boolean(value) => value.to_string(),
        V::Sequence(values) => format!("[{}]", values.iter().map(javascript).collect::<Vec<_>>().join(",")),
        _ => panic!("unexpected fixture value"),
    }
}

#[test]
fn text_decomposition_preserves_prewarms_exact_fields_in_native_and_bun() {
    let session = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let mut assertions = String::from("import * as p from './text.mjs';\nimport assert from 'node:assert/strict';\n");
    for (name, arguments, expected) in cases() {
        assert_eq!(session.invoke_callable(name.as_bytes(), &arguments).unwrap(), expected, "{name}({arguments:?})");
        assertions.push_str(&format!("assert.deepEqual(p[{name:?}]({}),{});\n",
            arguments.iter().map(javascript).collect::<Vec<_>>().join(","), javascript(&expected)));
    }
    let compiled = clause_package::lower_javascript_v1(&session.checked_source_package().unwrap()).unwrap();
    let directory = std::env::temp_dir().join(format!("clause-text-decomposition-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let module = directory.join("text.mjs");
    let runner = directory.join("run.mjs");
    std::fs::write(&module, compiled.module).unwrap();
    std::fs::write(&runner, assertions).unwrap();
    let bun = std::env::var_os("BUN").unwrap_or_else(|| "bun".into());
    let result = std::process::Command::new(bun).arg(&runner).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    std::fs::remove_file(module).unwrap();
    std::fs::remove_file(runner).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn text_decomposition_rejects_wrong_types_arities_and_unhandled_parse_results() {
    for source in [
        "export f(): Sequence<Text>\n  characters(true)\n",
        "export f(): Sequence<Text>\n  characters(\"a\", \"b\")\n",
        "export f(): Sequence<Text>\n  split-text(\"a\", 1)\n",
        "export f(): Sequence<Text>\n  split-text(\"a\")\n",
        "export f(): F64 | Text\n  parse-integer-prefix(false)\n",
        "export f(): F64\n  parse-integer-prefix(\"12\")\n",
        "export f(): Bool\n  parse-integer-prefix(\"12\") > 0\n",
        "export f(?text: Text): F64\n  count(?text)\n",
    ] {
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{source}");
    }
}
