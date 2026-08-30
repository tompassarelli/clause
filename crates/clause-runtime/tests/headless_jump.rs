use clause_package::*;
use clause_runtime::*;

fn decode_hex(source: &str) -> Vec<u8> {
    let digits = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    digits
        .chunks_exact(2)
        .map(|pair| (nibble(pair[0]) << 4) | nibble(pair[1]))
        .collect()
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => panic!("fixture contains a non-hex byte"),
    }
}

fn number(value: f64) -> ExecutableValueV1 {
    ExecutableValueV1::number(value).expect("scenario numbers are finite")
}

fn n(value: f64) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Constant(number(value))
}

fn b(value: bool) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(value))
}

fn s(slot: u16) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Slot(slot)
}

fn a(argument: u16) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Argument(argument)
}

fn boxed(expression: ExecutableExpressionV1) -> Box<ExecutableExpressionV1> {
    Box::new(expression)
}

fn add(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Add(boxed(left), boxed(right))
}

fn sub(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Subtract(boxed(left), boxed(right))
}

fn mul(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Multiply(boxed(left), boxed(right))
}

fn div(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Divide(boxed(left), boxed(right))
}

fn clamp(
    value: ExecutableExpressionV1,
    lower: ExecutableExpressionV1,
    upper: ExecutableExpressionV1,
) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Clamp(boxed(value), boxed(lower), boxed(upper))
}

fn eq(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Equal(boxed(left), boxed(right))
}

fn gt(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::GreaterThan(boxed(left), boxed(right))
}

fn le(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::LessThanOrEqual(boxed(left), boxed(right))
}

fn and(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::And(boxed(left), boxed(right))
}

fn next_x() -> ExecutableExpressionV1 {
    clamp(
        add(s(0), mul(mul(s(4), s(8)), a(0))),
        s(10),
        s(11),
    )
}

fn next_vertical_velocity() -> ExecutableExpressionV1 {
    add(s(3), mul(s(6), a(0)))
}

fn next_y() -> ExecutableExpressionV1 {
    add(s(1), mul(next_vertical_velocity(), a(0)))
}

fn headless_program() -> ExecutableProgramV1 {
    let horizontal_assignments = || {
        vec![
            (0, next_x()),
            (2, div(sub(next_x(), s(0)), a(0))),
        ]
    };
    let mut grounded_tick = horizontal_assignments();
    grounded_tick.extend([(1, s(9)), (3, n(0.0))]);
    let mut airborne_tick = horizontal_assignments();
    airborne_tick.extend([(1, next_y()), (3, next_vertical_velocity())]);
    let mut landing_tick = horizontal_assignments();
    landing_tick.extend([(1, s(9)), (3, n(0.0)), (5, b(true))]);

    ExecutableProgramV1 {
        // x, y, vx, vy, horizontal intent, grounded, and six package constants.
        initial_configuration: vec![
            number(9.5),
            number(0.0),
            number(0.0),
            number(0.0),
            number(0.0),
            ExecutableValueV1::Boolean(true),
            number(-8.0),
            number(8.0),
            number(5.0),
            number(0.0),
            number(-10.0),
            number(10.0),
        ],
        rules: vec![
            ExecutableRuleV1 {
                entry: 0,
                predicates: vec![],
                assignments: vec![(4, a(0))],
            },
            ExecutableRuleV1 {
                entry: 1,
                predicates: vec![eq(s(5), b(true))],
                assignments: vec![(3, s(7)), (5, b(false))],
            },
            ExecutableRuleV1 {
                entry: 2,
                predicates: vec![eq(s(5), b(true))],
                assignments: grounded_tick,
            },
            ExecutableRuleV1 {
                entry: 2,
                predicates: vec![and(eq(s(5), b(false)), gt(next_y(), s(9)))],
                assignments: airborne_tick,
            },
            ExecutableRuleV1 {
                entry: 2,
                predicates: vec![and(eq(s(5), b(false)), le(next_y(), s(9)))],
                assignments: landing_tick,
            },
        ],
    }
}

fn checked_program_package() -> CheckedProcessPackage {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/positive/process-v2-core.hex"
    ));
    let decoded = decode_process_package(&decode_hex(source)).expect("base package decodes");
    let mut candidate = decoded.candidate().clone();
    candidate.initial_state_views.clear();
    candidate.records.clear();

    let scope = candidate.snapshot.constitution.semantics;
    let term = headless_program()
        .encode_term(TermScope {
            universe: candidate.snapshot.constitution.universe,
            semantics: scope,
        })
        .expect("closed program encodes as a Term");
    let dependency = LocalSemanticDependencyV2::ExternalReference(term);
    candidate.snapshot.constitution.formations[0]
        .direct_dependencies
        .push(dependency.clone());
    candidate.snapshot.constitution.formations[0]
        .direct_dependencies
        .sort();
    for application in &mut candidate.snapshot.constitution.applications {
        application.form.dependency_closure.push(dependency.clone());
        application.form.dependency_closure.sort();
    }
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("program snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("program package encodes");
    check_process_package(decode_process_package(&bytes).expect("program package decodes"))
        .expect("program package checks")
}

fn occurrence(entry: u16, arguments: &[f64]) -> ExecutableOccurrenceV1 {
    ExecutableOccurrenceV1 {
        entry,
        arguments: arguments.iter().copied().map(number).collect(),
    }
}

fn value(configuration: &[ExecutableValueV1], slot: usize) -> f64 {
    configuration[slot]
        .as_number()
        .expect("selected slot is numeric")
}

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

#[test]
fn package_owned_headless_scenario_reaches_one_admitted_render_state() {
    let package = checked_program_package();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let authority = AuthorityStore::new();
    let mut runtime = ExecutableProcessRuntimeV1::instantiate(&package, &authority, application)
        .expect("checked package executable instantiates");

    runtime.advance(occurrence(0, &[1.0])).expect("input applies");
    runtime.advance(occurrence(2, &[0.25])).expect("ground tick applies");
    assert_eq!(value(runtime.configuration(), 0), 10.0);
    assert_eq!(value(runtime.configuration(), 2), 2.0);

    runtime.advance(occurrence(1, &[])).expect("grounded impulse applies");
    assert_eq!(value(runtime.configuration(), 3), 8.0);
    assert_eq!(runtime.configuration()[5].as_boolean(), Some(false));

    let before_rejected = runtime.configuration().to_vec();
    let rejected = runtime
        .advance(occurrence(1, &[]))
        .expect("unmatched occurrence still advances Configuration custody");
    assert!(!rejected.rule_applied);
    assert_eq!(runtime.configuration(), before_rejected);

    for _ in 0..7 {
        runtime.advance(occurrence(2, &[0.25])).expect("airborne tick applies");
    }
    assert_eq!(value(runtime.configuration(), 1), 0.0);
    assert_eq!(value(runtime.configuration(), 3), 0.0);
    assert_eq!(runtime.configuration()[5].as_boolean(), Some(true));
    assert!(runtime.candidate().is_none());
    assert!(runtime.judgment().is_none());
    assert!(runtime.admission().is_none());

    let base = StateRevisionId::from_bytes(raw_id(120));
    let candidate = runtime.emit_candidate(base).expect("candidate is emitted").clone();
    assert_eq!(candidate.base, base);
    assert!(runtime.judgment().is_none());
    assert!(runtime.admission().is_none());

    let judgment = runtime.judge(true).expect("candidate is judged").clone();
    assert_eq!(judgment.candidate, candidate.id);
    assert!(runtime.admission().is_none());

    let successor = runtime.admit().expect("accepted candidate is admitted").clone();
    assert_eq!(successor.predecessor, base);
    assert_ne!(successor.id, base);
    let observation = runtime.observe(&[0, 1, 3, 5]).expect("render projection exists");
    assert_eq!(observation.state, successor.id);
    assert_eq!(observation.value[0].as_number(), Some(10.0));
    assert_eq!(observation.value[1].as_number(), Some(0.0));
    assert_eq!(observation.value[2].as_number(), Some(0.0));
    assert_eq!(observation.value[3].as_boolean(), Some(true));

    assert_eq!(runtime.package(), package.id());
    assert_eq!(runtime.application(), application);
    assert_eq!(runtime.carrier().carrier().accepted_ingress_record_count(), 0);
}
