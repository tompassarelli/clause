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
        "form connects\n",
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
            BlockClass::ClassificationProjection,
            BlockClass::FocusedProjection,
            BlockClass::Definition,
            BlockClass::Query,
            BlockClass::Delta,
            BlockClass::Transition,
            BlockClass::Requirement,
            BlockClass::StructuralEscape,
        ]
    );
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![StatementClass::Query, StatementClass::Effect]
    );
    assert_eq!(
        classification.blocks[2].child_forms,
        vec![
            ChildForm::BareTerm,
            ChildForm::UnresolvedStructuralForm,
            ChildForm::Definition
        ]
    );
}

#[test]
fn distinguishes_classification_definition_and_equality_content() {
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
            StatementClass::ClassificationContent,
            StatementClass::Definition,
            StatementClass::RelationalContent,
        ]
    );
}

#[test]
fn keeps_law_rule_invariant_and_goal_modes_distinct() {
    let source = concat!(
        "law universal reachability\n",
        "  premise\n",
        "?origin reaches ?destination if\n",
        "  premise\n",
        "invariant acyclic\n",
        "  premise\n",
        "goal safe egress\n",
        "  desired = safe\n",
    );
    let classification = classify(&m0_stage_a::read(source));

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .blocks
            .iter()
            .map(|block| block.class)
            .collect::<Vec<_>>(),
        vec![
            BlockClass::UniversalLaw,
            BlockClass::DerivationRule,
            BlockClass::Invariant,
            BlockClass::Goal,
        ]
    );
}

#[test]
fn keeps_judgment_intention_effect_and_procedure_modes_distinct() {
    let document = m0_stage_a::read(concat!(
        "observe build-host supports wasm\n",
        "require worker-pool safe\n",
        "assume target supports threads\n",
        "hypothesis target stable\n",
        "intend North materializes\n",
        "render! scene\n",
        "do reconcile state\n",
        "procedure cleanup\n",
    ));
    let classification = classify(&document);

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![
            StatementClass::Observation,
            StatementClass::Requirement,
            StatementClass::Assumption,
            StatementClass::Assumption,
            StatementClass::Intention,
            StatementClass::Effect,
            StatementClass::Procedure,
            StatementClass::Procedure,
        ]
    );
}

#[test]
fn distinguishes_content_explicit_assertion_and_structural_contracts() {
    let source = concat!(
        "door = open\n",
        "assert door = open\n",
        "Alice = asserts\n",
        "Alice relates asserts to keyword\n",
        "relation contract connects\n",
        "position -> Vec2\n",
        "Alice asserts\n",
        "  door = open\n",
    );
    let classification = classify(&m0_stage_a::read(source));

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![
            StatementClass::RelationalContent,
            StatementClass::AssertionOccurrence,
            StatementClass::RelationalContent,
            StatementClass::UnresolvedStructuralForm,
            StatementClass::RelationContract,
            StatementClass::RelationContract,
        ]
    );
    assert_eq!(
        classification.blocks[0].class,
        BlockClass::AssertionOccurrence
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
        "form connects\n",
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
    assert_eq!(classification.blocks[0].class, BlockClass::Definition);
    assert_eq!(
        classification.blocks[0].child_forms,
        vec![ChildForm::Definition]
    );
}
