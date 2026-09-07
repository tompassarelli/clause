use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/derived-capacity.clause");
const NETWORK: &str = include_str!("../../../test-vectors/authoring/guarded-powered-network.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn number(term: &Term) -> f64 {
    f64::from_le_bytes(term.as_atom().unwrap().canonical_payload().try_into().unwrap())
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) -> Term {
    let event = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[event]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap()
}

#[test]
fn structured_aggregate_recomputes_after_changes_and_empty_queries() {
    let source = format!("{SOURCE}\n{}", r#"on resize ?component ?mass
  when
    ?component mass ?prior
    ?component = core
  withdraw
    ?component mass ?prior
  include
    ?component mass ?mass

on clear ?component
  when
    ?component mass ?mass
  withdraw
    ?component mass ?mass
"#);
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    for (handler, arguments, expected) in [
        (b"inspect".as_slice(), vec![], 15.0),
        (b"resize".as_slice(), vec![V::number(10.0).unwrap()], 17.0),
        (b"clear".as_slice(), vec![], 0.0),
    ] {
        let frame = run(&mut w, handler, &arguments);
        assert_eq!(number(field(field(field(&frame, b"workshop"), b"capacity"), b"mass")), expected);
    }
}

const CAPACITY: &str = r#"
Capacity:
  health: F64
  power: F64
capacity
  domain: Report
  range: Capacity
  cardinality: maybe

law network-capacity
  if
    ?report total ?prior
    sum ?health where { ?component powered true; ?component health ?health } as ?health
    sum ?generation where { ?component powered true; ?component generation ?generation } as ?power
  then
    ?report capacity Capacity { health: ?health, power: ?power }
derive network-capacity

on use-capacity ?report
  when
    ?report total ?prior
    ?report capacity Capacity { health: ?health, power: ?power }
  withdraw
    ?report total ?prior
  include
    ?report total ?health + ?power
"#;

#[test]
fn aggregates_wait_for_recursive_support_and_share_the_structured_result_with_execution() {
    for source in [format!("{NETWORK}{CAPACITY}"), format!("{CAPACITY}{NETWORK}")] {
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        for (mounted, expected_health, expected_power) in [(true, 300.0, 12.0), (false, 0.0, 0.0), (true, 300.0, 12.0)] {
            run(&mut w, b"set-mounted", &[V::Boolean(mounted)]);
            let frame = run(&mut w, b"use-capacity", &[]);
            let report = field(&frame, b"report");
            let capacity = field(report, b"capacity");
            assert_eq!(number(field(capacity, b"health")), expected_health);
            assert_eq!(number(field(capacity, b"power")), expected_power);
            assert_eq!(number(field(report, b"total")), expected_health + expected_power);
        }
    }
}

#[test]
fn aggregate_dependency_cycles_reject_instead_of_publishing_partial_totals() {
    let direct = SOURCE.replace("sum ?mass where { ?component mass ?mass } as ?total",
        "sum ?mass where { ?workshop capacity Capacity { mass: ?mass } } as ?total");
    let indirect = format!("{NETWORK}{CAPACITY}\n{}", r#"law capacity-support
  if
    ?report capacity Capacity { health: ?health, power: ?power }
    ?component generation ?generation
  then
    ?component powered true
derive capacity-support
"#);
    for source in [direct, indirect] {
        let error = ResidentSourceWorkbenchV1::open(source.as_bytes()).err().unwrap();
        assert!(format!("{error:?}").contains("UnstratifiedDerivation"), "{error:?}");
    }
}
