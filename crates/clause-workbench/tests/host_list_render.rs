use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{path::PathBuf, process::Command};

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/host-list.clause");
fn text(value: &str) -> V {
    V::text(value).unwrap()
}
fn hosts(values: &[&str]) -> V {
    V::Sequence(values.iter().map(|v| text(v)).collect())
}

const SCOPES: &str = r#"
export scope(?host: Text, ?hosts: Sequence<Text>): Text
  "{join([pair(?host, join([?host for ?host in ?hosts], "/")) for ?host in ?hosts], ",")}:{?host}"

pair(?left: Text, ?right: Text): Text
  "{?left}={?right}"

export composed(?hosts: Sequence<Text>): Text
  join(render(?hosts), ",")

render(?hosts: Sequence<Text>): Sequence<Text>
  [pair(?host, ?host) for ?host in ?hosts]

export scalars(?number: F64, ?flag: Bool): Text
  "{?number}/{?flag}"

export strict(?hosts: Sequence<Text>): Sequence<Text>
  [ignore(1 / 0) for ?host in ?hosts]

ignore(?unused: F64): Text
  "unused"

export source-failure(?hosts: Sequence<Text>): Sequence<Text>
  ["unused" for ?host in require(false, ?hosts, "source failed")]

export numbers(?values: Sequence<F64>): Text
  "{count([?value + 1 for ?value in ?values])}:{join(["{?value + 1}" for ?value in ?values], "/")}"
"#;

#[test]
fn actual_host_list_and_lexical_mapping_execute_natively_and_in_bun() {
    let session = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    assert!(session.generation().unsupported.is_empty());
    for (input, expected) in [
        (&[][..], "Hosts (0):\n"),
        (&["birch"][..], "Hosts (1):\n  birch\n"),
        (
            &["willow", "birch", "willow"][..],
            "Hosts (3):\n  willow\n  birch\n  willow\n",
        ),
    ] {
        assert_eq!(
            session
                .invoke_callable(b"render-host-list", &[hosts(input)])
                .unwrap(),
            text(expected)
        );
    }
    assert!(
        session
            .invoke_callable(b"render-host-list", &[V::Sequence(vec![V::Boolean(true)])])
            .is_err()
    );
    let scopes = ResidentSourceWorkbenchV1::open(SCOPES.as_bytes()).unwrap();
    assert_eq!(
        scopes
            .invoke_callable(b"scope", &[text("outer"), hosts(&["a", "b", "a"])])
            .unwrap(),
        text("a=a/b/a,b=a/b/a,a=a/b/a:outer")
    );
    assert_eq!(
        scopes
            .invoke_callable(b"composed", &[hosts(&["a", "b"])])
            .unwrap(),
        text("a=a,b=b")
    );
    for (number, expected) in [
        (12.5, "12.5/true"),
        (-0.0, "0/true"),
        (0.0000001, "0.0000001/true"),
        (1e21, "1000000000000000000000/true"),
    ] {
        assert_eq!(
            scopes
                .invoke_callable(b"scalars", &[V::number(number).unwrap(), V::Boolean(true)])
                .unwrap(),
            text(expected)
        );
    }
    assert_eq!(
        scopes
            .invoke_callable(
                b"numbers",
                &[V::Sequence(vec![
                    V::number(2.0).unwrap(),
                    V::number(2.0).unwrap()
                ])]
            )
            .unwrap(),
        text("2:3/3")
    );
    assert_eq!(
        scopes
            .invoke_callable(b"numbers", &[V::Sequence(vec![])])
            .unwrap(),
        text("0:")
    );
    assert_eq!(
        scopes.invoke_callable(b"strict", &[hosts(&[])]).unwrap(),
        V::Sequence(vec![])
    );
    assert!(scopes.invoke_callable(b"strict", &[hosts(&["a"])]).is_err());
    assert!(
        scopes
            .invoke_callable(b"source-failure", &[hosts(&[])])
            .is_err()
    );

    let directory = std::env::temp_dir().join(format!("clause-host-list-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-vectors/authoring/host-list.clause");
    std::fs::write(directory.join("scopes.clause"), SCOPES).unwrap();
    std::fs::write(
        directory.join("handwritten.ts"),
        include_str!("../../../test-vectors/authoring/host-list.ts"),
    )
    .unwrap();
    for (source, output) in [
        (source, "host-list.js"),
        (directory.join("scopes.clause"), "scopes.js"),
    ] {
        let checked = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
            .arg("check-source")
            .arg(&source)
            .output()
            .unwrap();
        assert!(checked.status.success(), "{checked:?}");
        let compiled = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
            .arg("compile-js")
            .arg(&source)
            .arg(directory.join(output))
            .output()
            .unwrap();
        assert!(compiled.status.success(), "{compiled:?}");
    }
    std::fs::write(directory.join("check.ts"), r#"
import assert from 'node:assert/strict';
import { 'render-host-list' as renderHostList } from './host-list.js';
import { renderHostList as handwritten } from './handwritten.ts';
import * as s from './scopes.js';
for (const hosts of [[], ['birch'], ['willow', 'birch', 'willow'], ['λ', 'a"b', '{x}']]) assert.equal(renderHostList(hosts), handwritten(hosts));
assert.throws(() => renderHostList([true]));
assert.equal(s.scope('outer', ['a', 'b', 'a']), 'a=a/b/a,b=a/b/a,a=a/b/a:outer');
assert.equal(s.composed(['a', 'b']), 'a=a,b=b');
for (const [number, expected] of [[12.5, '12.5/true'], [-0, '0/true'], [1e-7, '0.0000001/true'], [1e21, '1000000000000000000000/true']]) assert.equal(s.scalars(number, true), expected);
assert.equal(s.numbers([2, 2]), '2:3/3');
assert.equal(s.numbers([]), '0:');
assert.deepEqual(s.strict([]), []);
assert.throws(() => s.strict(['a']));
assert.throws(() => s['source-failure']([]));
"#).unwrap();
    let result = Command::new(std::env::var_os("BUN").unwrap_or_else(|| "bun".into()))
        .arg(directory.join("check.ts"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    for name in [
        "host-list.js",
        "host-list.d.ts",
        "scopes.clause",
        "scopes.js",
        "scopes.d.ts",
        "handwritten.ts",
        "check.ts",
    ] {
        std::fs::remove_file(directory.join(name)).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn mapping_and_interpolation_reject_invalid_bindings_types_and_effects() {
    for source in [
        "export f(?xs: Sequence<Text>): Sequence<Text>\n  [?missing for ?x in ?xs]\n",
        "export f(?xs: Sequence<Text>): Text\n  ?x\n",
        "export f(?xs: Text): Sequence<Text>\n  [?x for ?x in ?xs]\n",
        "export f(?xs: Sequence<F64>): Text\n  join(?xs, \",\")\n",
        "export f(?xs: Text): F64\n  count(?xs)\n",
        "export f(?xs: Sequence<Text>): Text\n  \"{?xs}\"\n",
        "export f(): Text\n  \"{{value: true}}\"\n",
        "export f(?xs: Sequence<Text>): Sequence<F64>\n  [?x + 1 for ?x in ?xs]\n",
        "export f(?xs: Sequence<Text>): Sequence<Text>\n  [p() for ?x in ?xs]\n\nforeign p(): Text\n  get: \"name\"\n  from: \"example\"\n  failure: throw\n",
    ] {
        assert!(
            ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),
            "accepted {source}"
        );
    }
}
