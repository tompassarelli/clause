use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/escort-fixed-tick.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected object").slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}

fn tick(source: &str) -> Term {
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let occurrences = workbench.fixed_tick_occurrences(0.016).unwrap();
    workbench.run_occurrences_to_candidate(&occurrences).unwrap();
    decode_canonical_term_bytes(
        &workbench.admit().unwrap().projection.exact_term_bytes(),
    ).unwrap()
}

fn sources() -> [String; 2] {
    [SOURCE.to_owned(), format!("{SOURCE}\n{}", r#"on spawn ?actor
  when
    ?actor journey speed ?speed
  create
    ?new
      member of: Actor
  include
    ?new actor position Vec3 { x: 0.0, y: 0.0, z: -5.0 }
    ?new journey destination Vec3 { x: 0.0, y: 0.0, z: 27.0 }
    ?new journey speed ?speed
"#)]
}

#[test]
fn fixed_tick_matches_exact_declared_subject() {
    for source in sources() {
        for equality in ["?actor = ilyra", "ilyra = ?actor"] {
            let source = source.replace("?actor = ilyra", equality);
            let frame = tick(&source);
            let position = field(field(&frame, b"ilyra"), b"actor-position");
            assert_eq!(field(position, b"z").as_atom().unwrap().canonical_payload(),
                (-5.0_f64 + 1.75 * 0.016).to_bits().to_le_bytes(), "{equality}");
        }
    }
}

#[test]
fn declared_subject_equality_does_not_match_a_different_actor() {
    for source in sources() {
        let source = format!("{source}\n{}", r#"other
  member of: Actor
other actor position Vec3 { x: 0.0, y: 0.0, z: -5.0 }
other journey destination Vec3 { x: 0.0, y: 0.0, z: 27.0 }
other journey speed 1.75
"#);
        let frame = tick(&source);
        let position = field(field(&frame, b"other"), b"actor-position");
        assert_eq!(field(position, b"z").as_atom().unwrap().canonical_payload(),
            (-5.0_f64).to_bits().to_le_bytes());
    }
}

#[test]
fn declared_subject_literal_requires_its_checked_domain() {
    for source in sources() {
        let source = source.replace("?actor = ilyra", "?actor = dawnroad-crossing");
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    }
}
