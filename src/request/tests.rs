use super::{Request, RequestOutput, RunLimits, RunResult, Selection, resolve, run};
use crate::{elaborate, frontend};

const SOURCE: &str = "Item
link: RelationShape
  {left: Item} links {right: Item}
  mode left -> right: many
graph
  A ∈ Item
  B ∈ Item
  A links B
graph/add: Revision
  from: graph
  admit:
    B links A
find all ?right in graph:
  A links ?right
why in graph:
  A links B
diff graph -> graph/add
";

const INTERVENTIONS: &str = "Item
link: RelationShape
  {left: Item} links {right: Item}
  mode left -> right: many
graph
  A ∈ Item
  B ∈ Item
  A links B
prevent one minimal in graph:
  A links B
using:
  link
prevent all minimal in graph:
  A links B
using:
  link
achieve one minimal in graph:
  B links A
using:
  link
achieve all minimal in graph:
  B links A
using:
  link
";

fn program(source: &str) -> super::ResolvedProgram {
    resolve(&elaborate::compile(frontend::parse(source).unwrap()).unwrap()).unwrap()
}

#[test]
fn resolves_typed_requests_in_authored_order_and_encodes_one_aggregate() {
    let program = program(SOURCE);
    assert!(matches!(
        program.requests(),
        [
            Request::Find { .. },
            Request::Why { all: false, .. },
            Request::Diff { .. }
        ]
    ));
    let output = run(&program, RunLimits::default()).unwrap();
    assert!(matches!(
        output.results[0].revision().unwrap().output(),
        RequestOutput::Find(_)
    ));
    assert!(matches!(
        output.results[1].revision().unwrap().output(),
        RequestOutput::WhyOne(_)
    ));
    assert!(matches!(output.results[2], RunResult::Diff { .. }));
    assert_eq!(output.canonical_bytes().matches("[\"find\"").count(), 1);
    assert!(output.canonical_bytes().starts_with("[\"clause-run-v2\","));
}

#[test]
fn request_hole_renames_preserve_alpha_normal_pattern_identity() {
    let before = program(SOURCE);
    let after = program(&SOURCE.replace("?right", "?destination"));
    let Request::Find {
        pattern: before_pattern,
        sought: before_sought,
        ..
    } = &before.requests()[0]
    else {
        panic!("first request must be find");
    };
    let Request::Find {
        pattern: after_pattern,
        sought: after_sought,
        ..
    } = &after.requests()[0]
    else {
        panic!("first request must be find");
    };

    assert_eq!(before_pattern, after_pattern);
    assert_eq!(before_sought, after_sought);
    assert!(
        before_pattern
            .roles()
            .values()
            .any(|term| term.pattern_id() == Some(before_sought))
    );
}

#[test]
fn dispatches_one_and_all_intervention_contracts() {
    let program = program(INTERVENTIONS);
    assert!(matches!(
        program.requests(),
        [
            Request::Prevent {
                selection: Selection::OneMinimal,
                ..
            },
            Request::Prevent {
                selection: Selection::AllMinimal,
                ..
            },
            Request::Achieve {
                selection: Selection::OneMinimal,
                ..
            },
            Request::Achieve {
                selection: Selection::AllMinimal,
                ..
            },
        ]
    ));
    let output = run(&program, RunLimits::default()).unwrap();
    assert!(matches!(
        output.results[0].revision().unwrap().output(),
        RequestOutput::PreventOne(_)
    ));
    assert!(matches!(
        output.results[1].revision().unwrap().output(),
        RequestOutput::PreventAll(_)
    ));
    assert!(matches!(
        output.results[2].revision().unwrap().output(),
        RequestOutput::AchieveOne(_)
    ));
    assert!(matches!(
        output.results[3].revision().unwrap().output(),
        RequestOutput::AchieveAll(_)
    ));
}
