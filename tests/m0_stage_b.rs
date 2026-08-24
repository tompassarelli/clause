//! Contracts for Stage-B structural classification, formatting, and editor input.

#[path = "../src/m0_stage_a.rs"]
mod m0_stage_a;
#[path = "../src/m0_stage_b.rs"]
mod m0_stage_b;

use m0_stage_b::{
    BlockClass, ChildForm, DiagnosticCode, StatementClass, classify, format,
    format_role_labelled_escape, rewrite_editor_input, warn_before_edit,
};

#[test]
fn classifies_binding_membership_focus_and_structural_families() {
    let source = concat!(
        "Game\n",
        "  Chess\n",
        "  Soccer\n",
        "\n",
        "Vec2\n",
        "  x: F32\n",
        "  y: F32\n",
        "\n",
        "iron-door\n",
        "  Door\n",
        "  connects Cellar to Armory\n",
        "  state: locked\n",
        "\n",
        "select ?person\n",
        "  ?person likes Chess\n",
        "\n",
        "next from base\n",
        "  - old relation value\n",
        "  + new relation value\n",
        "\n",
        "on frame ?dt\n",
        "  state: old ~>\n",
        "    state: new\n",
        "\n",
        "requires\n",
        "  game\n",
        "  three\n",
        "\n",
        "form connects\n",
        "  door: iron-door\n",
        "  origin: Cellar\n",
        "  destination: Armory\n",
        "\n",
        "any ?door connects Cellar to Armory\n",
        "render! scene\n",
    );
    let classification = classify(&m0_stage_a::read(source));

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
            BlockClass::FocusedProjection,
            BlockClass::FocusedProjection,
            BlockClass::Query,
            BlockClass::Delta,
            BlockClass::Transition,
            BlockClass::Requirement,
            BlockClass::StructuralEscape,
        ]
    );
    assert_eq!(
        classification.blocks[1].child_forms,
        vec![ChildForm::Binding, ChildForm::Binding]
    );
    assert_eq!(
        classification.blocks[2].child_forms,
        vec![
            ChildForm::Membership,
            ChildForm::UnresolvedStructuralForm,
            ChildForm::Binding,
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
}

#[test]
fn distinguishes_binding_membership_and_equality_content() {
    let classification = classify(&m0_stage_a::read(
        "gravity: 9.81\nChess ∈ Game\ngravity = 9.81\n",
    ));

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![
            StatementClass::Binding,
            StatementClass::Membership,
            StatementClass::RelationalContent,
        ]
    );
}

#[test]
fn recognizes_flat_ascii_operators_without_effect_or_slash_collisions() {
    let classification = classify(&m0_stage_a::read(concat!(
        "x > y\n", "x < y\n", "x >= y\n", "x <= y\n", "x != y\n", "x = y\n", "a + b\n", "a - b\n",
        "a * b\n", "a / b\n",
    )));

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![StatementClass::RelationalContent; 10]
    );
}

#[test]
fn preserves_contextual_delta_effect_contract_transition_and_qualified_name_forms() {
    let classification = classify(&m0_stage_a::read(concat!(
        "+ admitted content\n",
        "- withdrawn content\n",
        "render! scene\n",
        "position -> Vec2\n",
        "state: old ~>\n",
        "egress/route\n",
        "egress /route\n",
    )));

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![
            StatementClass::Delta,
            StatementClass::Delta,
            StatementClass::Effect,
            StatementClass::RelationContract,
            StatementClass::Transition,
            StatementClass::UnresolvedStructuralForm,
            StatementClass::UnresolvedStructuralForm,
        ]
    );
}

#[test]
fn keeps_law_rule_invariant_goal_and_judgment_modes_distinct() {
    let classification = classify(&m0_stage_a::read(concat!(
        "law universal reachability\n",
        "  premise\n",
        "?origin reaches ?destination if\n",
        "  premise\n",
        "invariant acyclic\n",
        "  premise\n",
        "goal safe egress\n",
        "  desired = safe\n",
        "observe build-host supports wasm\n",
        "require worker-pool safe\n",
        "assume target supports threads\n",
        "intend North materializes\n",
        "do reconcile state\n",
    )));

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
            StatementClass::Intention,
            StatementClass::Procedure,
        ]
    );
}

#[test]
fn rejects_raw_double_colon_and_word_membership_aliases() {
    let raw = classify(&m0_stage_a::read("iron-door :: Door\n"));
    assert_eq!(
        raw.diagnostics[0].code,
        DiagnosticCode::PersistedDoubleColon
    );
    assert_eq!(
        raw.diagnostics[0].repair,
        "replace `::` with `∈` before persistence or parsing"
    );

    for (source, code) in [
        ("Door 101 in Door\n", DiagnosticCode::MembershipInAlias),
        (
            "Door 101 member of Door\n",
            DiagnosticCode::MembershipMemberOfAlias,
        ),
    ] {
        let result = classify(&m0_stage_a::read(source));
        assert_eq!(result.diagnostics[0].code, code);
        assert_eq!(result.diagnostics[0].repair, "write membership with `∈`");
    }

    let legacy = classify(&m0_stage_a::read(
        "why all in egress:\n  target relation value\n",
    ));
    assert!(legacy.is_accepted());
    assert_eq!(legacy.blocks[0].class, BlockClass::Query);
}

#[test]
fn editor_normalization_precedes_reading_and_preserves_literals_and_bindings() {
    let rewritten = rewrite_editor_input("iron-door :: Door\nstate: locked\nlabel: \"a::b\"\n");
    assert_eq!(
        rewritten.source,
        "iron-door ∈ Door\nstate: locked\nlabel: \"a::b\"\n"
    );
    assert_eq!(rewritten.replaced.len(), 1);

    let document = m0_stage_a::read(&rewritten.source);
    assert!(document.is_accepted());
    assert!(classify(&document).is_accepted());
}

#[test]
fn formatter_emits_binding_membership_and_separates_enumeration_from_focus() {
    let source = concat!(
        "Thing\n",
        "  A\n",
        "  B\n",
        "Thing\n",
        "  relation -> Value\n",
        "member ∈ Category\n",
        "gravity: 9.81\n",
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
            "member ∈ Category\n",
            "gravity: 9.81\n",
        )
    );
    assert!(!formatted.contains("::"));
    assert!(classify(&m0_stage_a::read(&formatted)).is_accepted());
}

#[test]
fn explicit_role_labelled_escape_round_trips_with_binding_children() {
    let source = concat!(
        "form connects\n",
        "  door: iron-door\n",
        "  origin: Cellar\n",
        "  destination: Armory\n",
    );
    let document = m0_stage_a::read(source);
    let classification = classify(&document);
    let rendered = format_role_labelled_escape(&document, &classification.blocks[0])
        .expect("explicit structural form formats");

    assert_eq!(rendered, source);
    assert_eq!(
        classification.blocks[0].child_forms,
        vec![ChildForm::Binding, ChildForm::Binding, ChildForm::Binding]
    );
}

#[test]
fn focused_bare_child_is_membership_and_colon_child_is_binding() {
    let classification = classify(&m0_stage_a::read(concat!(
        "iron-door\n",
        "  Door\n",
        "  state: locked\n",
    )));

    assert!(classification.is_accepted());
    assert_eq!(
        classification.blocks[0].class,
        BlockClass::FocusedProjection
    );
    assert_eq!(
        classification.blocks[0].child_forms,
        vec![ChildForm::Membership, ChildForm::Binding]
    );
}

#[test]
fn leaves_grouped_nested_and_application_forms_unresolved() {
    let classification = classify(&m0_stage_a::read(concat!(
        "call(state: locked)\n",
        "contains(member ∈ Group)\n",
        "invoke(argument)\n",
    )));

    assert!(classification.is_accepted());
    assert_eq!(
        classification
            .statements
            .iter()
            .map(|statement| statement.class)
            .collect::<Vec<_>>(),
        vec![StatementClass::UnresolvedStructuralForm; 3]
    );
}

#[test]
fn rejects_tabs_and_noncanonical_layout_before_structural_classification() {
    let tabbed = classify(&m0_stage_a::read("root\n\tchild\n"));
    assert_eq!(
        tabbed.diagnostics[0].code,
        DiagnosticCode::StageATabIndentation
    );

    let skipped = classify(&m0_stage_a::read("root\n    child\n"));
    assert_eq!(
        skipped.diagnostics[0].code,
        DiagnosticCode::StageANoncanonicalIndentation
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
