//! Contracts for Stage-B classification, formatting, and non-rewriting editor input.

#[path = "../src/m0_stage_a.rs"]
mod m0_stage_a;
#[path = "../src/m0_stage_b.rs"]
mod m0_stage_b;

use m0_stage_b::{
    BlockClass, ChildForm, DiagnosticCode, StatementClass, classify, format,
    format_role_labelled_escape, validate_editor_input, warn_before_edit,
};

#[test]
fn classifies_constitutional_families_without_primitive_binding_membership_split() {
    let source = concat!(
        "Game\n",
        "  Chess\n",
        "  Soccer\n",
        "\n",
        "Vec2\n",
        "  x : F32\n",
        "  y : F32\n",
        "\n",
        "iron-door\n",
        "  Door\n",
        "  connects Cellar to Armory\n",
        "  state := locked\n",
        "\n",
        "distance between ?a and ?b :=\n",
        "  length(position of ?a - position of ?b)\n",
        "\n",
        "select ?person\n",
        "  ?person likes Chess\n",
        "\n",
        "next from base\n",
        "  - old relation value\n",
        "  + new relation value\n",
        "\n",
        "on frame ?dt\n",
        "  state := old ~>\n",
        "    state := new\n",
        "\n",
        "requires\n",
        "  game\n",
        "  three\n",
        "\n",
        "claim connects\n",
        "  door := iron-door\n",
        "  origin := Cellar\n",
        "  destination := Armory\n",
        "\n",
        "any ?door connects Cellar to Armory\n",
        "render! scene\n",
    );
    let document = m0_stage_a::read(source);
    assert!(document.is_accepted());
    let classification = classify(&document);

    assert!(
        classification.is_accepted(),
        "{:#?}",
        classification.diagnostics
    );
    assert_eq!(
        classification
            .blocks
            .iter()
            .map(|block| block.class)
            .collect::<Vec<_>>(),
        vec![
            BlockClass::Enumeration,
            BlockClass::ClassificationOrDerivedShape,
            BlockClass::FocusedClaimOrContract,
            BlockClass::DefinitionOrLaw,
            BlockClass::Query,
            BlockClass::RevisionDelta,
            BlockClass::Transition,
            BlockClass::EpistemicOrEffect,
            BlockClass::StructuralEscape,
        ]
    );
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![StatementClass::Query, StatementClass::EpistemicOrEffect]
    );
    assert_eq!(
        classification.blocks[2].child_forms,
        vec![
            ChildForm::BareTerm,
            ChildForm::BareTerm,
            ChildForm::Definition
        ]
    );
}

#[test]
fn distinguishes_classification_definition_and_equality_claim() {
    let document = m0_stage_a::read("Chess : Game\ngravity := 9.81\ngravity = 9.81\n");
    let classification = classify(&document);
    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![
            StatementClass::ClassificationClaim,
            StatementClass::Definition,
            StatementClass::Claim,
        ]
    );
}

#[test]
fn rejects_retired_membership_spellings_with_classification_repairs() {
    let legacy = classify(&m0_stage_a::read(
        "why all in egress:\n  target relation value\n",
    ));
    assert!(legacy.is_accepted());
    assert_eq!(legacy.blocks[0].class, BlockClass::Query);

    for (source, code) in [
        ("Door 101 in Door\n", DiagnosticCode::MembershipInAlias),
        (
            "Door 101 member of Door\n",
            DiagnosticCode::MembershipMemberOfAlias,
        ),
        ("Door 101 :: Door\n", DiagnosticCode::RetiredDoubleColon),
        ("Door 101 ∈ Door\n", DiagnosticCode::RetiredMembershipSymbol),
    ] {
        let result = classify(&m0_stage_a::read(source));
        assert_eq!(result.diagnostics[0].code, code);
        assert_eq!(
            result.diagnostics[0].repair,
            "write classification with `:`"
        );
    }
}

#[test]
fn editor_validation_preserves_every_byte_and_never_manufactures_membership() {
    let source = "iron-door :: Door\nstate := locked\nlabel := \"a::b\"\n";
    let checked = validate_editor_input(source);
    assert_eq!(checked.source, source);
    assert_eq!(checked.diagnostics.len(), 1);
    assert_eq!(
        checked.diagnostics[0].code,
        DiagnosticCode::RetiredDoubleColon
    );
    assert!(!checked.source.contains('∈'));

    let accepted = validate_editor_input("iron-door : Door\nstate := locked\n");
    assert!(accepted.diagnostics.is_empty());
}

#[test]
fn formatter_preserves_classification_and_definition_and_separates_focus() {
    let source = concat!(
        "Thing\n",
        "  A\n",
        "  B\n",
        "Thing\n",
        "  relation -> Value\n",
        "member : Category\n",
        "gravity := 9.81\n",
    );
    let document = m0_stage_a::read(source);
    let classification = classify(&document);
    let formatted = format(&document, &classification).expect("accepted structure formats");

    assert_eq!(
        formatted,
        concat!(
            "Thing\n",
            "  A\n",
            "  B\n",
            "\n",
            "Thing\n",
            "  relation -> Value\n",
            "member : Category\n",
            "gravity := 9.81\n",
        )
    );
    assert!(!formatted.contains('∈'));
    assert!(!formatted.contains("::"));
    assert!(classify(&m0_stage_a::read(&formatted)).is_accepted());
}

#[test]
fn explicit_named_role_escape_round_trips_without_role_invention() {
    let source = concat!(
        "claim connects\n",
        "  door := iron-door\n",
        "  origin := Cellar\n",
        "  destination := Armory\n",
    );
    let document = m0_stage_a::read(source);
    let classification = classify(&document);
    let rendered = format_role_labelled_escape(&document, &classification.blocks[0])
        .expect("explicit structural form formats");
    assert_eq!(rendered, source);
    assert_eq!(
        classification.blocks[0].child_forms,
        vec![
            ChildForm::Definition,
            ChildForm::Definition,
            ChildForm::Definition
        ]
    );
}

#[test]
fn editor_warns_before_enumeration_becomes_focus() {
    let document = m0_stage_a::read("Game\n  Chess\n  Soccer\n");
    let classification = classify(&document);
    let block = &classification.blocks[0];

    assert!(warn_before_edit(&document, block, "Go").is_none());
    assert_eq!(
        warn_before_edit(&document, block, "position -> Vec2")
            .expect("contract child changes block class")
            .code,
        DiagnosticCode::WouldReclassifyEnumeration
    );
}

#[test]
fn focused_definition_is_not_classification() {
    let document = m0_stage_a::read("iron-door\n  state := locked\n");
    let classification = classify(&document);
    assert_eq!(classification.blocks[0].class, BlockClass::DefinitionOrLaw);
    assert_eq!(
        classification.blocks[0].child_forms,
        vec![ChildForm::Definition]
    );
}
