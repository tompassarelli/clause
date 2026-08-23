//! Revision-registry identity admission coverage.

use std::collections::BTreeMap;

use clause::{elaborate, frontend, request};

const SOURCE: &str = "Item: Type\nlink: Relation\n    {left: Item} links {right: Item}\n    mode left -> right: many\ngraph: Model\n    A: Item\n    B: Item\n    A links B\ngraph/add: Revision\n    from: graph\n    admit:\n        B links A\nfind all ?right in graph:\n    A links ?right\nwhy all in graph:\n    A links B\ndiff graph -> graph/add\n";
#[test]
fn rejects_a_revision_registry_key_for_a_different_sealed_revision() {
    let compiled = elaborate::compile(frontend::parse(SOURCE).expect("source parses"))
        .expect("source compiles");
    let base = compiled
        .revision(&frontend::Name("graph".into()))
        .expect("base Revision exists");
    let successor = compiled
        .revision(&frontend::Name("graph/add".into()))
        .expect("successor Revision exists");
    assert_ne!(base.identity(), successor.identity());

    let error = request::ResolvedProgram::new(
        BTreeMap::from([(base.identity().clone(), successor.clone())]),
        vec![],
    )
    .expect_err("registry key must authenticate the stored Revision");
    assert!(
        error
            .to_string()
            .contains("Revision registry key must match sealed Revision identity")
    );
}
