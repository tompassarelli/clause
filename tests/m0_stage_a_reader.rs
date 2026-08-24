//! Contracts for the semantic-free, lossless M0 Stage-A reader.

#[path = "../src/m0_stage_a.rs"]
mod stage_a;

use stage_a::{Delimiter, DiagnosticCode, LineEnding, Punctuation, TokenKind, read};

#[test]
fn preserves_binding_membership_bytes_tokens_and_trivia() {
    let source = "title: \"red door\"\r\n  child ∈ Group\n";
    let document = read(source);

    assert_eq!(document.source, source);
    assert_eq!(document.lines.len(), 2);
    assert_eq!(document.lines[0].line_break.unwrap().kind, LineEnding::Crlf);
    assert_eq!(
        document.lines[0].tokens[1].kind,
        TokenKind::Punctuation(Punctuation::Colon)
    );
    assert_eq!(document.lines[0].tokens[2].kind, TokenKind::Literal);
    assert_eq!(document.lines[1].tokens[1].kind, TokenKind::Symbol);
    let membership = document.lines[1].tokens[1].span;
    assert_eq!(&document.source[membership.start..membership.end], "∈");
    assert_eq!(document.root.lines[0].children[0].indentation, 2);
    assert_eq!(document.root.lines[0].children[0].lines[0].line, 1);
    assert!(document.is_accepted());
}

#[test]
fn rejects_persisted_double_colon_without_rewriting_source_or_literals() {
    let source = "member :: Group\nlabel: \"a::b\"\n";
    let document = read(source);

    assert_eq!(document.source, source);
    assert_eq!(document.diagnostics.len(), 1);
    assert_eq!(
        document.diagnostics[0].code,
        DiagnosticCode::PersistedDoubleColon
    );
    let span = document.diagnostics[0].span;
    assert_eq!(&document.source[span.start..span.end], "::");
    assert_eq!(document.diagnostics[0].replacement, Some("∈"));
    assert!(!document.is_accepted());
}

#[test]
fn preserves_flat_symbolic_operator_tokens_and_qualified_slashes() {
    let source = concat!(
        "x > y\n",
        "x < y\n",
        "x >= y\n",
        "x <= y\n",
        "x != y\n",
        "x = y\n",
        "a + b\n",
        "a - b\n",
        "a * b\n",
        "a / b\n",
        "egress/route\n",
        "+ admitted\n",
        "- withdrawn\n",
        "render! scene\n",
        "position -> Vec2\n",
        "state ~> next\n",
    );
    let document = read(source);

    assert_eq!(document.source, source);
    assert!(document.is_accepted());
    assert_eq!(
        document.lines[4]
            .tokens
            .iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>(),
        vec![
            TokenKind::Name,
            TokenKind::Punctuation(Punctuation::Exclamation),
            TokenKind::Punctuation(Punctuation::Equals),
            TokenKind::Name,
        ]
    );
    assert_eq!(
        document.lines[9].tokens[1].kind,
        TokenKind::Punctuation(Punctuation::Slash)
    );
    assert_eq!(
        document.lines[10].tokens[1].kind,
        TokenKind::Punctuation(Punctuation::Slash)
    );
}

#[test]
fn retains_delimiters_and_rejects_tabs_and_noncanonical_indentation() {
    let source = "root\n\tchild(a, [b])\n    skipped\n";
    let document = read(source);

    assert!(
        document.lines[1]
            .tokens
            .iter()
            .any(|token| token.kind == TokenKind::Delimiter(Delimiter::OpenParen))
    );
    assert!(
        document.lines[1]
            .tokens
            .iter()
            .any(|token| token.kind == TokenKind::Delimiter(Delimiter::OpenBracket))
    );
    assert_eq!(
        document
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            DiagnosticCode::TabIndentation,
            DiagnosticCode::NoncanonicalIndentation,
        ]
    );
    assert!(!document.is_accepted());
}

#[test]
fn accepts_exact_two_space_layout_and_rejects_an_indented_root() {
    let canonical = read("root\n  child\n    leaf\n");
    assert!(canonical.is_accepted());
    assert_eq!(canonical.root.lines[0].children[0].indentation, 2);
    assert_eq!(
        canonical.root.lines[0].children[0].lines[0].children[0].indentation,
        4
    );

    let orphan = read("  orphan\n");
    assert_eq!(orphan.source, "  orphan\n");
    assert_eq!(orphan.root.lines[0].line, 0);
    assert_eq!(
        orphan
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![DiagnosticCode::NoncanonicalIndentation]
    );
}
