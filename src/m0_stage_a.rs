//! Semantic-free, lossless concrete reading for the M0 surface contract.
//!
//! This module deliberately has no dependency on the executable frontend.  Its
//! token categories are lexical only; later stages alone assign RelationalContent meaning.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceSpan {
    /// Zero-based, half-open UTF-8 byte offsets into `Document::source`.
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineBreak {
    pub kind: LineEnding,
    pub span: SourceSpan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TriviaKind {
    HorizontalWhitespace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: SourceSpan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delimiter {
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Punctuation {
    Colon,
    Comma,
    Dot,
    Question,
    Exclamation,
    Plus,
    Minus,
    Equals,
    Tilde,
    GreaterThan,
    LessThan,
    Star,
    Slash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Name,
    Literal,
    Delimiter(Delimiter),
    Punctuation(Punctuation),
    Symbol,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Line {
    /// The complete line, including its exact line break when present.
    pub span: SourceSpan,
    /// The source portion before the optional line break.
    pub content_span: SourceSpan,
    /// The raw leading spaces and tabs.  It is never expanded or normalized.
    pub indentation: SourceSpan,
    pub tokens: Vec<Token>,
    pub trivia: Vec<Trivia>,
    pub line_break: Option<LineBreak>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndentationGroup {
    /// Raw leading-byte width, retained even when it violates the layout rule.
    pub indentation: usize,
    pub lines: Vec<LayoutLine>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutLine {
    /// Index into `Document::lines`.
    pub line: usize,
    pub children: Vec<IndentationGroup>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticCode {
    TabIndentation,
    NoncanonicalIndentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: SourceSpan,
    /// A lexical repair suggestion.  It is not an elaboration decision.
    pub replacement: Option<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    /// The unchanged UTF-8 input, retained to make the output lossless.
    pub source: String,
    pub lines: Vec<Line>,
    /// The virtual zero-indent group; source order remains in every group.
    pub root: IndentationGroup,
    /// Source-ordered layout and persisted-source diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl Document {
    pub fn is_accepted(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Reads concrete source without choosing a RelationalContent syntactic or semantic form.
pub fn read(source: &str) -> Document {
    let mut lines = Vec::new();
    let mut diagnostics = Vec::new();
    let mut line_start = 0;

    while line_start < source.len() {
        let (content_end, line_end, line_break) = next_line(source, line_start);
        let content_span = SourceSpan {
            start: line_start,
            end: content_end,
        };
        let indentation_end = source[content_span.start..content_span.end]
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count()
            + line_start;
        let indentation = SourceSpan {
            start: line_start,
            end: indentation_end,
        };
        let (tokens, trivia) = lex_line(source, content_span);

        for offset in indentation.start..indentation.end {
            if source.as_bytes()[offset] == b'\t' {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::TabIndentation,
                    span: SourceSpan {
                        start: offset,
                        end: offset + 1,
                    },
                    replacement: None,
                });
            }
        }

        lines.push(Line {
            span: SourceSpan {
                start: line_start,
                end: line_end,
            },
            content_span,
            indentation,
            tokens,
            trivia,
            line_break,
        });
        line_start = line_end;
    }

    let root = build_layout(source, &lines, &mut diagnostics);
    diagnostics
        .sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.span.end, diagnostic.code));
    Document {
        source: source.to_owned(),
        lines,
        root,
        diagnostics,
    }
}

fn next_line(source: &str, start: usize) -> (usize, usize, Option<LineBreak>) {
    let bytes = source.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && !matches!(bytes[cursor], b'\n' | b'\r') {
        cursor += 1;
    }
    if cursor == bytes.len() {
        return (cursor, cursor, None);
    }
    let (end, kind) = if bytes[cursor] == b'\r' && bytes.get(cursor + 1) == Some(&b'\n') {
        (cursor + 2, LineEnding::Crlf)
    } else if bytes[cursor] == b'\r' {
        (cursor + 1, LineEnding::Cr)
    } else {
        (cursor + 1, LineEnding::Lf)
    };
    (
        cursor,
        end,
        Some(LineBreak {
            kind,
            span: SourceSpan { start: cursor, end },
        }),
    )
}

fn lex_line(source: &str, content: SourceSpan) -> (Vec<Token>, Vec<Trivia>) {
    let mut tokens = Vec::new();
    let mut trivia = Vec::new();
    let mut cursor = content.start;
    while cursor < content.end {
        let text = &source[cursor..content.end];
        let character = text.chars().next().expect("cursor remains in source");
        if character == ' ' || character == '\t' {
            let start = cursor;
            while cursor < content.end {
                let next = source[cursor..content.end]
                    .chars()
                    .next()
                    .expect("cursor remains in source");
                if next != ' ' && next != '\t' {
                    break;
                }
                cursor += next.len_utf8();
            }
            trivia.push(Trivia {
                kind: TriviaKind::HorizontalWhitespace,
                span: SourceSpan { start, end: cursor },
            });
            continue;
        }

        let start = cursor;
        let kind = if is_name_start(character) {
            cursor += character.len_utf8();
            while cursor < content.end {
                let next = source[cursor..content.end]
                    .chars()
                    .next()
                    .expect("cursor remains in source");
                if !is_name_continue(next) {
                    break;
                }
                cursor += next.len_utf8();
            }
            TokenKind::Name
        } else if character.is_ascii_digit() {
            cursor += character.len_utf8();
            while cursor < content.end {
                let next = source[cursor..content.end]
                    .chars()
                    .next()
                    .expect("cursor remains in source");
                if !(next.is_ascii_alphanumeric() || matches!(next, '_' | '.')) {
                    break;
                }
                cursor += next.len_utf8();
            }
            TokenKind::Literal
        } else if matches!(character, '\'' | '\"') {
            cursor = quoted_literal_end(source, cursor, content.end, character);
            TokenKind::Literal
        } else if let Some(delimiter) = delimiter(character) {
            cursor += character.len_utf8();
            TokenKind::Delimiter(delimiter)
        } else if let Some(punctuation) = punctuation(character) {
            cursor += character.len_utf8();
            TokenKind::Punctuation(punctuation)
        } else {
            cursor += character.len_utf8();
            TokenKind::Symbol
        };
        tokens.push(Token {
            kind,
            span: SourceSpan { start, end: cursor },
        });
    }

    (tokens, trivia)
}

fn is_name_start(character: char) -> bool {
    character.is_alphabetic() || character == '_'
}

fn is_name_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-')
}

fn quoted_literal_end(source: &str, start: usize, end: usize, quote: char) -> usize {
    let mut cursor = start + quote.len_utf8();
    let mut escaped = false;
    while cursor < end {
        let character = source[cursor..end]
            .chars()
            .next()
            .expect("cursor remains in source");
        cursor += character.len_utf8();
        if !escaped && character == quote {
            break;
        }
        escaped = !escaped && character == '\\';
        if character != '\\' {
            escaped = false;
        }
    }
    cursor
}

fn delimiter(character: char) -> Option<Delimiter> {
    Some(match character {
        '(' => Delimiter::OpenParen,
        ')' => Delimiter::CloseParen,
        '[' => Delimiter::OpenBracket,
        ']' => Delimiter::CloseBracket,
        '{' => Delimiter::OpenBrace,
        '}' => Delimiter::CloseBrace,
        _ => return None,
    })
}

fn punctuation(character: char) -> Option<Punctuation> {
    Some(match character {
        ':' => Punctuation::Colon,
        ',' => Punctuation::Comma,
        '.' => Punctuation::Dot,
        '?' => Punctuation::Question,
        '!' => Punctuation::Exclamation,
        '+' => Punctuation::Plus,
        '-' => Punctuation::Minus,
        '=' => Punctuation::Equals,
        '~' => Punctuation::Tilde,
        '>' => Punctuation::GreaterThan,
        '<' => Punctuation::LessThan,
        '*' => Punctuation::Star,
        '/' => Punctuation::Slash,
        _ => return None,
    })
}

fn build_layout(
    source: &str,
    lines: &[Line],
    diagnostics: &mut Vec<Diagnostic>,
) -> IndentationGroup {
    let mut parents = vec![None; lines.len()];
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        if line.content_span.is_empty() {
            continue;
        }
        let indent_text = &source[line.indentation.start..line.indentation.end];
        if indent_text.contains('\t') {
            continue;
        }
        let width = line.indentation.len();
        if width % 2 != 0 {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::NoncanonicalIndentation,
                span: line.indentation,
                replacement: None,
            });
        }
        while stack
            .last()
            .is_some_and(|(indentation, _)| *indentation >= width)
        {
            stack.pop();
        }
        if let Some((parent_width, _)) = stack.last()
            && width > parent_width + 2
        {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::NoncanonicalIndentation,
                span: line.indentation,
                replacement: None,
            });
        } else if stack.is_empty() && width != 0 {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::NoncanonicalIndentation,
                span: line.indentation,
                replacement: None,
            });
        }
        parents[line_index] = stack.last().map(|(_, index)| *index);
        stack.push((width, line_index));
    }

    IndentationGroup {
        indentation: 0,
        lines: build_lines(None, &parents, lines),
    }
}

fn build_lines(
    parent: Option<usize>,
    parents: &[Option<usize>],
    lines: &[Line],
) -> Vec<LayoutLine> {
    let mut result = Vec::new();
    for (line_index, observed_parent) in parents.iter().enumerate() {
        if *observed_parent != parent {
            continue;
        }
        let children = child_groups(line_index, parents, lines);
        result.push(LayoutLine {
            line: line_index,
            children,
        });
    }
    result
}

fn child_groups(parent: usize, parents: &[Option<usize>], lines: &[Line]) -> Vec<IndentationGroup> {
    let mut groups: Vec<IndentationGroup> = Vec::new();
    for (line_index, observed_parent) in parents.iter().enumerate() {
        if *observed_parent != Some(parent) {
            continue;
        }
        let indentation = lines[line_index].indentation.len();
        if groups
            .last()
            .is_none_or(|group| group.indentation != indentation)
        {
            groups.push(IndentationGroup {
                indentation,
                lines: Vec::new(),
            });
        }
        groups
            .last_mut()
            .expect("group was added")
            .lines
            .push(LayoutLine {
                line: line_index,
                children: child_groups(line_index, parents, lines),
            });
    }
    groups
}
