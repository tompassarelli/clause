use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::projected_relation_table_v1;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/structured-value-copy.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current
            .as_triple()
            .unwrap_or_else(|| panic!("missing projected field {}", String::from_utf8_lossy(key)))
            .slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8]) -> Term {
    let occurrence = w.handler_occurrence(name, &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

#[test]
fn whole_structure_and_scalar_updates_share_one_prestate() {
    for change_position in [false, true] {
        let source = if change_position {
            SOURCE
                .replace("  withdraw\n", "  withdraw\n    ?item position ?position\n")
                .replace(
                    "  include\n",
                    "  include\n    ?item position Point { x: 12.0, y: 13.0 }\n",
                )
        } else {
            SOURCE.to_owned()
        };
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        let frame = run(&mut w, b"stop");
        let item = field(&frame, b"item");
        for (axis, prior) in [(b"x".as_slice(), 2_f64), (b"y", 3_f64)] {
            assert_eq!(
                field(field(item, b"destination"), axis)
                    .as_atom()
                    .unwrap()
                    .canonical_payload(),
                prior.to_bits().to_le_bytes()
            );
            let position = if change_position { prior + 10.0 } else { prior };
            assert_eq!(
                field(field(item, b"position"), axis)
                    .as_atom()
                    .unwrap()
                    .canonical_payload(),
                position.to_bits().to_le_bytes()
            );
        }
        assert_eq!(
            field(item, b"moving")
                .as_atom()
                .unwrap()
                .canonical_payload(),
            [0]
        );
    }
}

#[test]
fn whole_structure_copy_includes_runtime_created_rows() {
    let source = format!(
        "{SOURCE}\n{}",
        r#"on spawn ?item
  when
    ?item moving ?prior
  create
    ?new
      shape: Item
  include
    ?new position Point { x: 5.0, y: 6.0 }
    ?new destination Point { x: 0.0, y: 0.0 }
    ?new moving true
"#
    );
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"spawn");
    let frame = run(&mut w, b"stop");
    let relations = field(&frame, b"relations");
    for axis in [b"x".as_slice(), b"y"] {
        let position = projected_relation_table_v1(field(field(relations, b"position"), axis))
            .unwrap()
            .unwrap();
        let destination =
            projected_relation_table_v1(field(field(relations, b"destination"), axis))
                .unwrap()
                .unwrap();
        assert_eq!(position.rows().len(), 2);
        assert_eq!(position.rows(), destination.rows());
    }
    let moving = projected_relation_table_v1(field(relations, b"moving"))
        .unwrap()
        .unwrap();
    assert_eq!(moving.rows().len(), 2);
    assert!(
        moving
            .rows()
            .values()
            .flatten()
            .all(|value| *value == clause_runtime::ExecutableValueV1::Boolean(false))
    );
}

#[test]
fn whole_structure_copy_preserves_nominal_types() {
    let wrong_shape = SOURCE
        .replace(
            "relation position",
            "shape OtherPoint\n  x: F64\n  y: F64\n\nrelation position",
        )
        .replace(
            "destination {value: Point}",
            "destination {value: OtherPoint}",
        )
        .replace("item destination Point", "item destination OtherPoint");
    assert!(ResidentSourceWorkbenchV1::open(wrong_shape.as_bytes()).is_err());
    let numeric = SOURCE.replace(
        "?item destination ?position\n",
        "?item destination ?position + 1.0\n",
    );
    assert!(ResidentSourceWorkbenchV1::open(numeric.as_bytes()).is_err());
}
