use std::path::PathBuf;
use std::process::Command;

use clause_workbench::{
    AUTHORING_EXAMPLES_V1, ResidentSourceWorkbenchV1, render_authoring_card_v1,
};

const COMMITTED_CARD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/authoring-card.md"
));

#[test]
fn authoring_catalog_and_cli_use_the_resident_compiler() {
    assert_eq!(
        render_authoring_card_v1(),
        COMMITTED_CARD,
        "the committed card is the deterministic catalog projection"
    );

    for example in AUTHORING_EXAMPLES_V1 {
        ResidentSourceWorkbenchV1::open(example.source.as_bytes()).unwrap_or_else(|error| {
            panic!(
                "authoring example {} did not open through the complete resident pipeline: {error}",
                example.slug
            )
        });
    }

    let executable = env!("CARGO_BIN_EXE_clause-workbench");
    let card = Command::new(executable)
        .arg("authoring-card")
        .output()
        .expect("the authoring-card command starts");
    assert!(card.status.success(), "authoring-card failed: {card:?}");
    assert_eq!(card.stdout, COMMITTED_CARD.as_bytes());

    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/ledger/ledger.clause");
    let checked = Command::new(executable)
        .arg("check-source")
        .arg(source)
        .output()
        .expect("the check-source command starts");
    assert!(checked.status.success(), "check-source failed: {checked:?}");
    assert!(
        checked.stdout.starts_with(b"checked generation="),
        "check-source did not report the opened resident generation: {checked:?}"
    );
}
