use std::time::Instant;

use clause_package::{
    ApplicationId, ProgramRevisionPreimage, Term, check_process_package,
    decode_canonical_term_bytes, decode_process_package,
};
use clause_runtime::{
    ExecutableInputSourceV1, ExecutableKeyPhaseV1, ExecutableValueV1, ForkedProcessBranchV1,
    decode_executable_occurrence_v1, decode_executable_physical_plan_v1,
    decode_wasm_process_request_v1, open_fresh_persistent_process_session_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));
const DASH_WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world-dash-jump.clause"
));
const COLLECT_CONTACT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/collect-contact.clause"
));
const SPRING_PAD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/spring-pad.clause"
));
const OBJECTIVE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/objective.clause"
));
const LEDGER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/ledger/ledger.clause"
));
const SOURCE_ONLY_AUTOMATIC_EXTENSION: &[u8] = br#"
relation pulse-count
  reads {objective: Objective} pulse count {value: F64}
  subject objective
  mode given objective yields value: one

relation pulse-radius
  reads {objective: Objective} pulse radius {value: F64}
  subject objective
  mode given objective yields value: one

relation pulse-echo
  reads {player: Player} pulse echo {value: F64}
  subject player
  mode given player yields value: one

relation pulse-contact
  reads {objective: Objective} has pulse contact with {player: Player} as {value: Bool}
  subject objective
  mode given objective player yields value: one

game-objective pulse count 0.0
game-objective pulse radius 0.6
player-1 pulse echo 0.0

law pulse-contact-within-radius
  if
    ?player position Vec3 { x: ?player-x, y: ?player-y, z: ?player-z }
    game-objective pulse radius ?radius
    ((?player-x - 0.5) * (?player-x - 0.5)) + ((?player-z - 0.0) * (?player-z - 0.0)) <= ?radius * ?radius
  then
    game-objective has pulse contact with ?player as true

derive pulse-contact-within-radius

on count-pulse ?objective
  when
    ?objective pulse count ?count
    game-objective has pulse contact with player-1 as true
  withdraw
    ?objective pulse count ?count
  include
    ?objective pulse count ?count + 1.0

on echo-pulse ?player
  when
    ?player pulse echo ?echo
    game-objective has pulse contact with ?player as true
  withdraw
    ?player pulse echo ?echo
  include
    ?player pulse echo ?echo + 1.0
"#;
const SOURCE_KEYBOARD_BURST_EXTENSION: &[u8] = br#"
bind keyboard KeyQ down to planar-burst

on planar-burst ?player
  when
    ?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }
  withdraw
    ?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }
  include
    ?player velocity Vec3 { x: ?velocity-x + 3.0, y: ?velocity-y, z: ?velocity-z - 2.0 }
"#;

fn coherent_source(objective: &[u8]) -> Vec<u8> {
    let mut source = Vec::with_capacity(
        DASH_WORLD.len() + COLLECT_CONTACT.len() + SPRING_PAD.len() + objective.len() + 3,
    );
    for part in [DASH_WORLD, COLLECT_CONTACT, SPRING_PAD, objective] {
        if !source.is_empty() {
            source.push(b'\n');
        }
        source.extend_from_slice(part);
    }
    source
}

fn coherent_source_with_automatic_extension() -> Vec<u8> {
    let mut source = coherent_source(OBJECTIVE);
    source.extend_from_slice(SOURCE_ONLY_AUTOMATIC_EXTENSION);
    source
}

fn arguments(values: &[f64]) -> Vec<ExecutableValueV1> {
    values
        .iter()
        .copied()
        .map(|value| ExecutableValueV1::number(value).expect("finite test number"))
        .collect()
}

fn decode_hex(source: &str) -> Vec<u8> {
    let digits = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert_eq!(digits.len() % 2, 0, "fixture hex is complete");
    digits
        .chunks_exact(2)
        .map(|pair| {
            let digit = |value: u8| match value {
                b'0'..=b'9' => value - b'0',
                b'a'..=b'f' => value - b'a' + 10,
                _ => panic!("fixture hex is lowercase"),
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect()
}

fn lowercase_hex_lines(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2 + bytes.len() / 64 + 1);
    for line in bytes.chunks(64) {
        for byte in line {
            text.push(char::from(DIGITS[usize::from(byte >> 4)]));
            text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
        }
        text.push('\n');
    }
    text
}

fn tick_chain(workbench: &ResidentSourceWorkbenchV1, mut prefix: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    prefix.extend(
        workbench
            .fixed_tick_occurrences(0.016)
            .expect("checked source owns the exact tick chain"),
    );
    prefix
}

fn projected_object_field<'a>(term: &'a Term, expected: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        if current.as_atom().is_some() {
            panic!("projected object lacks field {expected:?}");
        }
        let [field, value, rest] = current
            .as_triple()
            .expect("projected object is an entry chain")
            .slots();
        let field = field.as_atom().expect("projected field is an Atom");
        if field.canonical_payload() == expected {
            return value;
        }
        current = rest;
    }
}

fn projected_symbol(term: &Term) -> &[u8] {
    let atom = term.as_atom().expect("projected symbol is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-symbol-v1");
    atom.canonical_payload()
}

fn projected_number(term: &Term) -> f64 {
    let atom = term.as_atom().expect("projected number is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-f64-v1");
    f64::from_bits(u64::from_le_bytes(
        atom.canonical_payload()
            .try_into()
            .expect("projected F64 is exact"),
    ))
}

fn projected_boolean(term: &Term) -> bool {
    let atom = term.as_atom().expect("projected Boolean is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-bool-v1");
    match atom.canonical_payload() {
        [0] => false,
        [1] => true,
        _ => panic!("projected Boolean payload is canonical"),
    }
}

fn objective_state(exact_term_bytes: &[u8]) -> Vec<u8> {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let objective = projected_object_field(&term, b"game-objective");
    projected_symbol(projected_object_field(objective, b"objective-state")).to_vec()
}

fn player_launch_state(exact_term_bytes: &[u8]) -> (f64, bool) {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let player = projected_object_field(&term, b"player-1");
    let velocity = projected_object_field(player, b"velocity");
    (
        projected_number(projected_object_field(velocity, b"y")),
        projected_boolean(projected_object_field(player, b"grounded")),
    )
}

fn ledger_balance(exact_term_bytes: &[u8]) -> f64 {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let account = projected_object_field(&term, b"operating-account");
    projected_number(projected_object_field(account, b"balance"))
}

fn pulse_count(exact_term_bytes: &[u8]) -> f64 {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let objective = projected_object_field(&term, b"game-objective");
    projected_number(projected_object_field(objective, b"pulse-count"))
}

fn pulse_echo(exact_term_bytes: &[u8]) -> f64 {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let player = projected_object_field(&term, b"player-1");
    projected_number(projected_object_field(player, b"pulse-echo"))
}

fn player_planar_velocity(exact_term_bytes: &[u8]) -> (f64, f64) {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let player = projected_object_field(&term, b"player-1");
    let velocity = projected_object_field(player, b"velocity");
    (
        projected_number(projected_object_field(velocity, b"x")),
        projected_number(projected_object_field(velocity, b"z")),
    )
}

#[test]
fn source_edit_hot_reloads_in_one_workbench_without_admission_custody_leak() {
    let mut workbench =
        ResidentSourceWorkbenchV1::open(WORLD).expect("base source opens in one workbench");
    let base_generation = workbench.generation().clone();
    let base_candidate = workbench
        .run_to_candidate()
        .expect("base source produces one hidden candidate");
    assert_eq!(base_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let base_admission = workbench
        .admit()
        .expect("separate base Admission returns the rendered frame");
    assert_eq!(base_admission.state_revision_count, 2);
    assert_ne!(base_admission.predecessor, base_admission.successor);

    let changed = std::str::from_utf8(WORLD)
        .expect("world source is UTF-8")
        .replacen("jump-arena move speed 5.0", "jump-arena move speed 7.0", 1);
    assert_ne!(changed.as_bytes(), WORLD);
    let changed_generation = workbench
        .hot_reload(changed.as_bytes())
        .expect("source-only edit hot reloads in the resident process");
    assert_eq!(
        changed_generation.handle.generation,
        base_generation.handle.generation + 1
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    assert_ne!(changed_generation.cwr1, base_generation.cwr1);
    assert!(
        workbench
            .rejects_stale_handle(base_generation.handle)
            .expect("stale-handle probe reaches the live boundary")
    );

    let changed_candidate = workbench
        .run_to_candidate()
        .expect("changed source reruns without restarting Rust");
    assert_eq!(changed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let changed_admission = workbench
        .admit()
        .expect("separate changed-source Admission returns its frame");
    assert_eq!(changed_admission.state_revision_count, 2);
    assert_ne!(
        changed_admission.projection.exact_term_bytes, base_admission.projection.exact_term_bytes,
        "the source edit changes the admitted rendered frame"
    );

    let changed_again =
        changed.replacen("jump-arena move speed 7.0", "jump-arena move speed 9.0", 1);
    let changed_again_generation = workbench
        .hot_reload(changed_again.as_bytes())
        .expect("a second source-only edit reclaims the prior resident generation");
    assert_eq!(
        changed_again_generation.handle.generation,
        changed_generation.handle.generation + 1
    );
    assert!(
        workbench
            .rejects_stale_handle(changed_generation.handle)
            .expect("the second source edit keeps the prior handle stale")
    );
}

#[test]
fn resident_generation_opens_exact_fresh_branch_sessions() {
    let workbench =
        ResidentSourceWorkbenchV1::open(WORLD).expect("source opens one resident generation");
    let generation = workbench.generation();
    let request = decode_wasm_process_request_v1(&generation.cwr1)
        .expect("resident generation retains one exact CWR1");
    let package = check_process_package(
        decode_process_package(&request.package_bytes).expect("CWR1 package decodes"),
    )
    .expect("CWR1 package remains checked");
    let expected_application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: request.application,
    };
    let expected_revision = ProgramRevisionPreimage {
        semantics: package.constitution().semantics(),
        program: request.authority.program,
        predecessor: None,
        snapshot: package.constitution().snapshot(),
        change: request.authority.change,
    }
    .derived_claim()
    .id;

    let authoritative = open_fresh_persistent_process_session_v1(&generation.cwr1)
        .expect("resident CWR1 opens one fresh authoritative session");
    let branch_session = open_fresh_persistent_process_session_v1(&generation.cwr1)
        .expect("the same resident CWR1 opens one distinct fresh branch session");
    assert_eq!(authoritative.package().unwrap(), package.id());
    assert_eq!(authoritative.application().unwrap(), expected_application);
    assert_eq!(authoritative.program_revision(), expected_revision);
    assert_eq!(branch_session.package().unwrap(), package.id());
    assert_eq!(branch_session.application().unwrap(), expected_application);
    assert_eq!(branch_session.program_revision(), expected_revision);
    assert_eq!(
        branch_session.runtime_session(),
        authoritative.runtime_session()
    );
    assert_ne!(
        branch_session.allocation().root(),
        authoritative.allocation().root(),
        "each fresh session owns a distinct runtime allocation root"
    );

    let disconnect = request
        .occurrences
        .first()
        .expect("resident CWR1 retains one construct-blind occurrence");
    let branch = ForkedProcessBranchV1::fork(&authoritative, branch_session, 1, disconnect)
        .expect("fresh CWR1 sessions enter the exact branch path");
    assert_eq!(branch.pins().package, package.id());
    assert_eq!(branch.pins().application, expected_application);
    assert_eq!(branch.pins().program_revision, expected_revision);
}

#[test]
fn coherent_source_fails_resets_completes_and_hot_reloads_in_one_workbench() {
    let source = coherent_source(OBJECTIVE);
    let mut workbench =
        ResidentSourceWorkbenchV1::open(&source).expect("coherent source opens resident workbench");
    let base_generation = workbench.generation().clone();

    let failure_input = workbench
        .handler_occurrence(b"input", &arguments(&[0.0, -1.0]))
        .expect("checked input handler accepts southward intent");
    let failure = tick_chain(&workbench, vec![failure_input]);
    let failed_candidate = workbench
        .run_occurrences_to_candidate(&failure)
        .expect("hazard produces one hidden candidate");
    assert_eq!(failed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let failed = workbench
        .admit()
        .expect("separate Admission exposes the failed frame");
    assert_eq!(
        objective_state(&failed.projection.exact_term_bytes),
        b"failed"
    );

    let north = workbench
        .handler_occurrence(b"input", &arguments(&[0.0, 1.0]))
        .expect("checked input handler accepts northward intent");
    let reset_handler = workbench
        .handler_occurrence(b"reset-objective", &[])
        .expect("checked reset handler accepts its external occurrence");
    let reset = tick_chain(&workbench, vec![north, reset_handler]);
    let reset_candidate = workbench
        .run_occurrences_to_candidate(&reset)
        .expect("reset produces one hidden candidate");
    assert_eq!(reset_candidate.state_revision_count, 2);
    assert_eq!(
        objective_state(&workbench.last_projection().unwrap().exact_term_bytes),
        b"failed",
        "the admitted renderer remains on failure before reset Admission"
    );
    let reset = workbench
        .admit()
        .expect("separate Admission exposes the reset frame");
    assert_eq!(
        objective_state(&reset.projection.exact_term_bytes),
        b"playing"
    );

    let east = workbench
        .handler_occurrence(b"input", &arguments(&[1.0, 0.0]))
        .expect("checked input handler accepts eastward intent");
    let completion = tick_chain(&workbench, vec![east]);
    workbench
        .run_occurrences_to_candidate(&completion)
        .expect("movement and collection produce hidden completion");
    assert_eq!(
        objective_state(&workbench.last_projection().unwrap().exact_term_bytes),
        b"playing",
        "completion is invisible before Admission"
    );
    let completed = workbench
        .admit()
        .expect("separate Admission exposes completion");
    assert_eq!(
        objective_state(&completed.projection.exact_term_bytes),
        b"completed"
    );

    let spring_input = workbench
        .handler_occurrence(b"input", &arguments(&[1.0, 0.0]))
        .expect("checked input handler advances onto the spring");
    let launch = tick_chain(&workbench, vec![spring_input]);
    workbench
        .run_occurrences_to_candidate(&launch)
        .expect("source-owned spring transition remains hidden");
    assert_eq!(
        player_launch_state(&workbench.last_projection().unwrap().exact_term_bytes),
        (0.0, true),
        "spring velocity and airborne state remain invisible before Admission"
    );
    let launched = workbench
        .admit()
        .expect("separate Admission exposes the source-owned spring transition");
    assert_eq!(
        player_launch_state(&launched.projection.exact_term_bytes),
        (12.0, false)
    );

    let changed_objective = std::str::from_utf8(OBJECTIVE)
        .expect("objective source is UTF-8")
        .replacen("?player-x = 0.08", "?player-x = 0.16", 1);
    let changed_source = coherent_source(changed_objective.as_bytes());
    let reload_started = Instant::now();
    let changed_generation = workbench
        .hot_reload(&changed_source)
        .expect("Clause-only objective threshold hot reloads in-process");
    let reload_elapsed = reload_started.elapsed();
    eprintln!(
        "resident coherent source hot reload: {:.3} ms",
        reload_elapsed.as_secs_f64() * 1000.0
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    workbench
        .run_to_candidate()
        .expect("changed source reruns without rebuilding Rust");
    let changed = workbench
        .admit()
        .expect("changed source reaches separate Admission");
    assert_eq!(
        objective_state(&changed.projection.exact_term_bytes),
        b"playing",
        "the edited completion threshold defers the objective by one tick"
    );
}

#[test]
fn ledger_uses_the_same_checked_resident_binding_path() {
    let mut workbench =
        ResidentSourceWorkbenchV1::open(LEDGER).expect("ledger opens in the generic workbench");
    workbench
        .run_to_candidate()
        .expect("source-owned deposit produces one hidden candidate");
    assert!(workbench.last_projection().is_none());
    let deposited = workbench
        .admit()
        .expect("separate Admission exposes the deposited balance");
    assert_eq!(
        ledger_balance(&deposited.projection.exact_term_bytes),
        125.0
    );

    let changed = std::str::from_utf8(LEDGER)
        .expect("ledger source is UTF-8")
        .replacen(
            "?account balance ?balance + 25.0",
            "?account balance ?balance + 40.0",
            1,
        );
    workbench
        .hot_reload(changed.as_bytes())
        .expect("Clause-only deposit edit hot reloads without a Rust binding change");
    workbench
        .run_to_candidate()
        .expect("edited deposit produces one hidden candidate");
    let changed = workbench
        .admit()
        .expect("separate Admission exposes the edited balance");
    assert_eq!(ledger_balance(&changed.projection.exact_term_bytes), 140.0);
}

#[test]
fn source_only_state_and_bounded_automatic_handler_need_no_host_binding_edit() {
    let source = coherent_source_with_automatic_extension();
    let mut workbench = ResidentSourceWorkbenchV1::open(&source)
        .expect("source-only state and automatic handler allocate generically");
    workbench
        .run_to_candidate()
        .expect("the new automatic handler participates in the checked tick chain");
    assert!(workbench.last_projection().is_none());
    let admitted = workbench
        .admit()
        .expect("separate Admission exposes the source-only state");
    assert_eq!(pulse_count(&admitted.projection.exact_term_bytes), 1.0);
    assert_eq!(pulse_echo(&admitted.projection.exact_term_bytes), 1.0);
}

#[test]
fn source_keyboard_binding_reaches_one_atomic_multi_assignment_candidate() {
    let mut source = WORLD.to_vec();
    source.extend_from_slice(SOURCE_KEYBOARD_BURST_EXTENSION);
    let mut workbench = ResidentSourceWorkbenchV1::open(&source)
        .expect("source keyboard binding and general handler open");
    let burst = workbench
        .handler_occurrence(b"planar-burst", &[])
        .expect("checked burst handler has one occurrence");
    let burst_occurrence =
        decode_executable_occurrence_v1(&burst).expect("burst occurrence decodes");
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1)
        .expect("generated physical plan decodes");
    let input = plan
        .input
        .expect("source keyboard binding creates input plan");
    let key = input
        .events
        .iter()
        .find(|event| {
            event.source
                == ExecutableInputSourceV1::Keyboard {
                    code: b"KeyQ".to_vec(),
                    phase: ExecutableKeyPhaseV1::Down,
                }
        })
        .expect("KeyQ down is present in the physical plan");
    assert_eq!(key.occurrence, burst_occurrence);

    workbench
        .run_occurrences_to_candidate(&[burst])
        .expect("burst produces one hidden candidate");
    assert!(workbench.last_projection().is_none());
    let admitted = workbench
        .admit()
        .expect("separate Admission exposes both burst assignments");
    assert_eq!(
        player_planar_velocity(&admitted.projection.exact_term_bytes),
        (3.0, -2.0)
    );
}

#[test]
fn tracked_browser_carrier_uses_the_generic_source_plan() {
    let source = std::str::from_utf8(WORLD)
        .expect("world source is UTF-8")
        .replace("player-1", "player");
    let workbench = ResidentSourceWorkbenchV1::open(source.as_bytes())
        .expect("browser fixture compiles through generic source bindings");
    let current = decode_wasm_process_request_v1(&workbench.generation().cwr1)
        .expect("generated generic CWR1 decodes");
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../browser/jump-arena-shell/fixtures/wasm-generic-source-v1");
    let fixture_path = fixture_root.join("generic-source-v1.cwr1.hex");
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_GENERIC_SOURCE_CWR1").is_some() {
        std::fs::create_dir_all(&fixture_root).expect("generic browser fixture directory exists");
        std::fs::write(
            &fixture_path,
            lowercase_hex_lines(&workbench.generation().cwr1),
        )
        .expect("generic browser fixture updates");
    }
    let tracked = std::fs::read_to_string(&fixture_path)
        .expect("tracked generic browser CWR1 fixture exists");
    let tracked = decode_wasm_process_request_v1(&decode_hex(&tracked))
        .expect("tracked generic browser CWR1 decodes");
    assert_eq!(tracked.package_bytes, current.package_bytes);
    assert_eq!(tracked.application, current.application);
    assert_eq!(tracked.physical_plan_bytes, current.physical_plan_bytes);
    assert_eq!(tracked.authority, current.authority);
    assert_eq!(tracked.occurrences, current.occurrences);
    assert_eq!(tracked.render_slots, current.render_slots);
}
