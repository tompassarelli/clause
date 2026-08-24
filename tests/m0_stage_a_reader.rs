//! Contracts for the semantic-free, lossless M0 Stage-A reader.

#[path = "../src/m0_stage_a.rs"]
mod stage_a;

use stage_a::{Delimiter, DiagnosticCode, LineEnding, Punctuation, TokenKind, read};

#[test]
fn preserves_classification_definition_bytes_tokens_and_trivia() {
    let source = "title := \"red door\"\r\n  child : Group\n";
    let document = read(source);

    assert_eq!(document.source, source);
    assert_eq!(document.lines.len(), 2);
    assert_eq!(document.lines[0].line_break.unwrap().kind, LineEnding::Crlf);
    assert_eq!(
        document.lines[0].tokens[1].kind,
        TokenKind::Punctuation(Punctuation::Colon)
    );
    assert_eq!(
        document.lines[0].tokens[2].kind,
        TokenKind::Punctuation(Punctuation::Equals)
    );
    assert_eq!(document.lines[0].tokens[3].kind, TokenKind::Literal);
    assert_eq!(
        document.lines[1].tokens[1].kind,
        TokenKind::Punctuation(Punctuation::Colon)
    );
    assert_eq!(document.root.lines[0].children[0].indentation, 2);
    assert_eq!(document.root.lines[0].children[0].lines[0].line, 1);
    assert!(document.diagnostics.is_empty());
}

#[test]
fn retains_every_retired_spelling_for_later_structural_diagnostics() {
    let source = "a :: B\na ∈ B\na in B\na member of B\n";
    let document = read(source);

    assert_eq!(document.source, source);
    assert!(document.is_accepted());
    assert_eq!(document.diagnostics, []);
    assert_eq!(
        &document.source
            [document.lines[0].tokens[1].span.start..document.lines[0].tokens[1].span.end],
        ":"
    );
    assert_eq!(
        &document.source
            [document.lines[1].tokens[1].span.start..document.lines[1].tokens[1].span.end],
        "∈"
    );
    assert_eq!(
        &document.source
            [document.lines[2].tokens[1].span.start..document.lines[2].tokens[1].span.end],
        "in"
    );
    assert_eq!(
        &document.source
            [document.lines[3].tokens[1].span.start..document.lines[3].tokens[1].span.end],
        "member"
    );
}

#[test]
fn retains_delimiters_and_reports_only_layout_diagnostics() {
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
    assert!(
        document.lines[1]
            .tokens
            .iter()
            .any(|token| token.kind == TokenKind::Delimiter(Delimiter::CloseBracket))
    );
    assert_eq!(
        document
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            DiagnosticCode::TabIndentation,
            DiagnosticCode::NoncanonicalIndentation
        ]
    );
    assert!(!document.is_accepted());
}

#[test]
fn diagnoses_an_orphan_indented_root_without_normalizing_layout() {
    let orphan = read("  orphan\n");
    let root = read("root\n");

    assert_eq!(orphan.source, "  orphan\n");
    assert_eq!(orphan.lines[0].indentation.len(), 2);
    assert_eq!(orphan.root.lines[0].line, 0);
    assert_eq!(orphan.root.lines[0].children, []);
    assert_eq!(
        orphan
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![DiagnosticCode::NoncanonicalIndentation]
    );
    assert_eq!(root.lines[0].indentation.len(), 0);
    assert!(root.diagnostics.is_empty());
}
