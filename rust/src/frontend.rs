//! The source-facing Clause reader for the first shared language fixture.
//!
//! This file deliberately has no dependency on the kernel.  It preserves
//! source spans and resolves only declarations which are exact and explicit:
//! a relation's sentence shape, its declared mode, bare model facts, closed
//! desired intents, and a query over a model.  Kernel code can consume the
//! resulting typed values without needing to retain the authoring source.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

fn error(span: Span, message: impl Into<String>) -> ParseError {
    ParseError {
        span,
        message: message.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDecl {
    pub name: String,
    pub ty: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapePart {
    Literal { text: String, span: Span },
    Role { name: String, span: Span },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceShape {
    pub parts: Vec<ShapePart>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cardinality {
    One,
    Maybe,
    Some,
    Many,
}

impl Cardinality {
    fn parse(text: &str, span: Span) -> Result<Self, ParseError> {
        match text {
            "one" => Ok(Self::One),
            "maybe" => Ok(Self::Maybe),
            "some" => Ok(Self::Some),
            "many" => Ok(Self::Many),
            _ => Err(error(span, format!("unknown mode cardinality '{text}'"))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mode {
    pub known: Vec<String>,
    pub sought: Vec<String>,
    pub cardinality: Cardinality,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationDecl {
    pub name: String,
    pub roles: Vec<RoleDecl>,
    pub sentence: SentenceShape,
    pub mode: Mode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TermKind {
    Text(String),
    Variable(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Term {
    pub kind: TermKind,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fact {
    pub relation: String,
    pub roles: BTreeMap<String, Term>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelDecl {
    pub name: String,
    pub facts: Vec<Fact>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query {
    pub model: String,
    pub relation: String,
    pub sought: String,
    pub roles: BTreeMap<String, Term>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationKind {
    Claim,
    Require,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Operation {
    pub kind: OperationKind,
    pub model: String,
    pub clause: Fact,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentDecl {
    pub name: String,
    pub desired: Fact,
    pub span: Span,
}
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Program {
    pub relations: Vec<RelationDecl>,
    pub models: Vec<ModelDecl>,
    pub queries: Vec<Query>,
    pub operations: Vec<Operation>,
    pub intents: Vec<IntentDecl>,
}

#[derive(Clone, Copy, Debug)]
struct SourceLine<'a> {
    number: usize,
    text: &'a str,
}

fn line_span(line: SourceLine<'_>) -> Span {
    Span {
        line: line.number,
        column: 1,
        width: line.text.len(),
    }
}

fn child_span(line: SourceLine<'_>, offset: usize, width: usize) -> Span {
    Span {
        line: line.number,
        column: offset + 1,
        width,
    }
}

fn is_name(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '/' || c == '-')
}

fn is_role_name(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn split_top_level<'a>(text: &'a str, separator: &str) -> Vec<(usize, &'a str)> {
    let mut out = Vec::new();
    let mut start = 0;
    while start <= text.len() {
        let rest = &text[start..];
        match rest.find(separator) {
            Some(relative) => {
                out.push((start, &text[start..start + relative]));
                start += relative + separator.len();
            }
            None => {
                out.push((start, rest));
                break;
            }
        }
    }
    out
}

fn parse_roles(text: &str, line: SourceLine<'_>, base: usize) -> Result<Vec<RoleDecl>, ParseError> {
    let open = text
        .find('(')
        .ok_or_else(|| error(child_span(line, base, text.len()), "expected role list '('"))?;
    let close = text.rfind(')').ok_or_else(|| {
        error(
            child_span(line, base + open, text.len() - open),
            "expected role list ')'",
        )
    })?;
    if open == 0 || close != text.len() - 1 {
        return Err(error(
            child_span(line, base, text.len()),
            "malformed role list",
        ));
    }
    let inside = &text[open + 1..close];
    if inside.is_empty() {
        return Err(error(
            child_span(line, base + open + 1, 0),
            "role list cannot be empty",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut roles = Vec::new();
    for (part_index, (offset, item)) in split_top_level(inside, ",").into_iter().enumerate() {
        if item.is_empty() || (part_index > 0 && !item.starts_with(' ')) {
            return Err(error(
                child_span(line, base + open + 1 + offset, item.len()),
                "roles must be separated by ', '",
            ));
        }
        let item = item.strip_prefix(' ').unwrap_or(item);
        let item_offset = offset + usize::from(inside.as_bytes().get(offset) == Some(&b' '));
        let Some((name, ty)) = item.split_once(": ") else {
            return Err(error(
                child_span(line, base + open + 1 + offset, item.len()),
                "expected named role 'name: Type'",
            ));
        };
        if !is_role_name(name) || ty.is_empty() || !is_name(ty) {
            return Err(error(
                child_span(line, base + open + 1 + item_offset, item.len()),
                "invalid role name or type",
            ));
        }
        if !seen.insert(name) {
            return Err(error(
                child_span(line, base + open + 1 + item_offset, name.len()),
                format!("duplicate role '{name}'"),
            ));
        }
        roles.push(RoleDecl {
            name: name.to_owned(),
            ty: ty.to_owned(),
            span: child_span(line, base + open + 1 + item_offset, item.len()),
        });
    }
    Ok(roles)
}

fn parse_shape(
    body: &str,
    line: SourceLine<'_>,
    base: usize,
    role_names: &BTreeSet<String>,
) -> Result<SentenceShape, ParseError> {
    if body.is_empty() {
        return Err(error(
            child_span(line, base, 0),
            "sentence shape cannot be empty",
        ));
    }
    let mut parts = Vec::new();
    let mut cursor = 0;
    let mut seen = BTreeSet::new();
    while cursor < body.len() {
        let Some(open_relative) = body[cursor..].find('{') else {
            parts.push(ShapePart::Literal {
                text: body[cursor..].to_owned(),
                span: child_span(line, base + cursor, body.len() - cursor),
            });
            break;
        };
        let open = cursor + open_relative;
        if open > cursor {
            parts.push(ShapePart::Literal {
                text: body[cursor..open].to_owned(),
                span: child_span(line, base + cursor, open - cursor),
            });
        }
        let Some(close_relative) = body[open + 1..].find('}') else {
            return Err(error(
                child_span(line, base + open, body.len() - open),
                "unterminated sentence role",
            ));
        };
        let close = open + 1 + close_relative;
        let name = &body[open + 1..close];
        if !is_role_name(name) || !role_names.contains(name) {
            return Err(error(
                child_span(line, base + open, close - open + 1),
                format!("unknown sentence role '{name}'"),
            ));
        }
        if !seen.insert(name) {
            return Err(error(
                child_span(line, base + open, close - open + 1),
                format!("duplicate sentence role '{name}'"),
            ));
        }
        parts.push(ShapePart::Role {
            name: name.to_owned(),
            span: child_span(line, base + open, close - open + 1),
        });
        cursor = close + 1;
    }
    if seen.len() != role_names.len() {
        return Err(error(
            child_span(line, base, body.len()),
            "sentence shape must mention every declared role exactly once",
        ));
    }
    Ok(SentenceShape {
        parts,
        span: child_span(line, base, body.len()),
    })
}

fn parse_role_list(
    text: &str,
    line: SourceLine<'_>,
    base: usize,
) -> Result<Vec<String>, ParseError> {
    if text.is_empty() {
        return Err(error(
            child_span(line, base, 0),
            "mode role list cannot be empty",
        ));
    }
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for (offset, item) in split_top_level(text, ",") {
        let item = item.strip_prefix(' ').unwrap_or(item);
        if !is_role_name(item) {
            return Err(error(
                child_span(line, base + offset, item.len()),
                format!("invalid mode role '{item}'"),
            ));
        }
        if !seen.insert(item) {
            return Err(error(
                child_span(line, base + offset, item.len()),
                format!("duplicate mode role '{item}'"),
            ));
        }
        result.push(item.to_owned());
    }
    Ok(result)
}

fn parse_mode(
    body: &str,
    line: SourceLine<'_>,
    base: usize,
    role_names: &BTreeSet<String>,
) -> Result<Mode, ParseError> {
    let Some((direction, cardinality_text)) = body.rsplit_once(": ") else {
        return Err(error(
            child_span(line, base, body.len()),
            "mode requires ': cardinality'",
        ));
    };
    let cardinality_base = base + direction.len() + 2;
    let cardinality = Cardinality::parse(
        cardinality_text,
        child_span(line, cardinality_base, cardinality_text.len()),
    )?;
    let Some((known_text, sought_text)) = direction.split_once(" -> ") else {
        return Err(error(
            child_span(line, base, direction.len()),
            "mode requires 'known -> sought'",
        ));
    };
    let known = parse_role_list(known_text, line, base)?;
    let sought = parse_role_list(sought_text, line, base + known_text.len() + 4)?;
    for role in known.iter().chain(sought.iter()) {
        if !role_names.contains(role) {
            return Err(error(
                child_span(line, base, body.len()),
                format!("mode refers to unknown role '{role}'"),
            ));
        }
    }
    if known.iter().any(|role| sought.contains(role)) {
        return Err(error(
            child_span(line, base, direction.len()),
            "a mode role cannot be both known and sought",
        ));
    }
    Ok(Mode {
        known,
        sought,
        cardinality,
        span: child_span(line, base, body.len()),
    })
}

fn parse_term(text: &str, line: SourceLine<'_>, base: usize) -> Result<Term, ParseError> {
    if let Some(name) = text.strip_prefix('?') {
        if !is_role_name(name) {
            return Err(error(
                child_span(line, base, text.len()),
                "invalid query variable",
            ));
        }
        return Ok(Term {
            kind: TermKind::Variable(name.to_owned()),
            span: child_span(line, base, text.len()),
        });
    }
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        let value = &text[1..text.len() - 1];
        if value.is_empty() || value.contains('"') || value.contains('\n') {
            return Err(error(
                child_span(line, base, text.len()),
                "invalid quoted Text term",
            ));
        }
        return Ok(Term {
            kind: TermKind::Text(value.to_owned()),
            span: child_span(line, base, text.len()),
        });
    }
    Err(error(
        child_span(line, base, text.len()),
        "sentence terms must be quoted Text or query variables",
    ))
}

fn match_shape(
    shape: &SentenceShape,
    input: &str,
    line: SourceLine<'_>,
    base: usize,
    query: bool,
) -> Result<BTreeMap<String, Term>, ParseError> {
    let mut values = BTreeMap::new();
    let mut cursor = 0;
    for (index, part) in shape.parts.iter().enumerate() {
        match part {
            ShapePart::Literal { text, .. } => {
                if !input[cursor..].starts_with(text) {
                    return Err(error(
                        child_span(line, base + cursor, input.len().saturating_sub(cursor)),
                        "clause does not match its declared sentence shape",
                    ));
                }
                cursor += text.len();
            }
            ShapePart::Role { name, .. } => {
                let next_literal = shape.parts[index + 1..].iter().find_map(|part| match part {
                    ShapePart::Literal { text, .. } if !text.is_empty() => Some(text),
                    _ => None,
                });
                let end = match next_literal {
                    Some(literal) => input[cursor..]
                        .find(literal)
                        .map(|offset| cursor + offset)
                        .ok_or_else(|| {
                            error(
                                child_span(line, base + cursor, input.len().saturating_sub(cursor)),
                                "clause does not match its declared sentence shape",
                            )
                        })?,
                    None => input.len(),
                };
                if end == cursor {
                    return Err(error(
                        child_span(line, base + cursor, 0),
                        "empty sentence role",
                    ));
                }
                let term_text = &input[cursor..end];
                let term = parse_term(term_text, line, base + cursor)?;
                if !query && matches!(term.kind, TermKind::Variable(_)) {
                    return Err(error(
                        term.span,
                        "model facts cannot contain query variables",
                    ));
                }
                if values.insert(name.clone(), term).is_some() {
                    return Err(error(
                        child_span(line, base + cursor, end - cursor),
                        format!("duplicate sentence role '{name}'"),
                    ));
                }
                cursor = end;
            }
        }
    }
    if cursor != input.len() {
        return Err(error(
            child_span(line, base + cursor, input.len() - cursor),
            "trailing text does not match sentence shape",
        ));
    }
    Ok(values)
}

fn relation_names(relations: &[RelationDecl]) -> BTreeSet<String> {
    relations
        .iter()
        .map(|relation| relation.name.clone())
        .collect()
}

fn find_relation<'a>(
    relations: &'a [RelationDecl],
    input: &str,
    line: SourceLine<'_>,
    base: usize,
    query: bool,
) -> Result<(&'a RelationDecl, BTreeMap<String, Term>), ParseError> {
    let mut matches = Vec::new();
    for relation in relations {
        if let Ok(values) = match_shape(&relation.sentence, input, line, base, query) {
            matches.push((relation, values));
        }
    }
    match matches.len() {
        1 => Ok(matches.pop().expect("one match")),
        0 => Err(error(
            child_span(line, base, input.len()),
            "no declared sentence shape matches this clause",
        )),
        _ => Err(error(
            child_span(line, base, input.len()),
            "clause matches more than one declared sentence shape",
        )),
    }
}

fn parse_relation(lines: &[SourceLine<'_>], index: &mut usize) -> Result<RelationDecl, ParseError> {
    let line = lines[*index];
    let body = line
        .text
        .strip_prefix("relation ")
        .ok_or_else(|| error(line_span(line), "expected relation declaration"))?;
    let body = body
        .strip_suffix(':')
        .ok_or_else(|| error(line_span(line), "relation declaration must end with ':'"))?;
    let open = body
        .find('(')
        .ok_or_else(|| error(line_span(line), "relation declaration requires roles"))?;
    let name = &body[..open];
    if !is_name(name) {
        return Err(error(
            child_span(line, 9, name.len()),
            "invalid relation name",
        ));
    }
    let roles = parse_roles(body, line, 9)?;
    let role_names = roles
        .iter()
        .map(|role| role.name.clone())
        .collect::<BTreeSet<_>>();
    *index += 1;
    let mut sentence = None;
    let mut mode = None;
    while *index < lines.len() && lines[*index].text.starts_with("    ") {
        let child = lines[*index];
        let child_body = &child.text[4..];
        if let Some(value) = child_body.strip_prefix("sentence: ") {
            if sentence.is_some() {
                return Err(error(line_span(child), "duplicate sentence declaration"));
            }
            sentence = Some(parse_shape(value, child, 14, &role_names)?);
        } else if let Some(value) = child_body.strip_prefix("mode ") {
            if mode.is_some() {
                return Err(error(line_span(child), "duplicate mode declaration"));
            }
            mode = Some(parse_mode(value, child, 9, &role_names)?);
        } else {
            return Err(error(
                line_span(child),
                "unknown relation declaration member",
            ));
        }
        *index += 1;
    }
    let sentence =
        sentence.ok_or_else(|| error(line_span(line), "relation requires sentence declaration"))?;
    let mode = mode.ok_or_else(|| error(line_span(line), "relation requires mode declaration"))?;
    Ok(RelationDecl {
        name: name.to_owned(),
        roles,
        sentence,
        mode,
        span: line_span(line),
    })
}

fn parse_model(
    lines: &[SourceLine<'_>],
    index: &mut usize,
    relations: &[RelationDecl],
) -> Result<ModelDecl, ParseError> {
    let line = lines[*index];
    let body = line
        .text
        .strip_prefix("model ")
        .and_then(|body| body.strip_suffix(':'))
        .ok_or_else(|| error(line_span(line), "model declaration must be 'model name:'"))?;
    if !is_name(body) {
        return Err(error(child_span(line, 7, body.len()), "invalid model name"));
    }
    *index += 1;
    let mut facts = Vec::new();
    while *index < lines.len() && lines[*index].text.starts_with("    ") {
        let child = lines[*index];
        let input = &child.text[4..];
        let (relation, roles) = find_relation(relations, input, child, 5, false)?;
        if facts.iter().any(|fact: &Fact| {
            fact.relation == relation.name
                && fact.roles.len() == roles.len()
                && fact.roles.iter().all(|(role, term)| {
                    roles.get(role).map(|candidate| &candidate.kind) == Some(&term.kind)
                })
        }) {
            return Err(error(line_span(child), "duplicate model fact"));
        }
        facts.push(Fact {
            relation: relation.name.clone(),
            roles,
            span: line_span(child),
        });
        *index += 1;
    }
    if facts.is_empty() {
        return Err(error(line_span(line), "model requires at least one fact"));
    }
    Ok(ModelDecl {
        name: body.to_owned(),
        facts,
        span: line_span(line),
    })
}

fn parse_query(
    lines: &[SourceLine<'_>],
    index: &mut usize,
    relations: &[RelationDecl],
    models: &[ModelDecl],
) -> Result<Query, ParseError> {
    let line = lines[*index];
    let model = line
        .text
        .strip_prefix("query ")
        .and_then(|body| body.strip_suffix(':'))
        .ok_or_else(|| error(line_span(line), "query declaration must be 'query model:'"))?;
    if !is_name(model) || !models.iter().any(|candidate| candidate.name == model) {
        return Err(error(
            child_span(line, 7, model.len()),
            "query refers to an unknown model",
        ));
    }
    *index += 1;
    let child = lines
        .get(*index)
        .copied()
        .ok_or_else(|| error(line_span(line), "query requires a body"))?;
    if !child.text.starts_with("    ") {
        return Err(error(
            line_span(child),
            "query body must be indented by four spaces",
        ));
    }
    let body = &child.text[4..];
    let Some((sought_text, input)) = body.split_once(" where ") else {
        return Err(error(
            line_span(child),
            "query requires '?name where sentence'",
        ));
    };
    if !sought_text.starts_with('?') || !is_role_name(&sought_text[1..]) {
        return Err(error(
            child_span(child, 5, sought_text.len()),
            "invalid query variable",
        ));
    }
    let sought = sought_text[1..].to_owned();
    let (relation, roles) =
        find_relation(relations, input, child, 5 + sought_text.len() + 7, true)?;
    if !roles
        .values()
        .any(|term| term.kind == TermKind::Variable(sought.clone()))
    {
        return Err(error(
            child_span(child, 5, sought_text.len()),
            "query variable must occur in its sentence",
        ));
    }
    if roles
        .values()
        .filter(|term| term.kind == TermKind::Variable(sought.clone()))
        .count()
        != 1
    {
        return Err(error(
            child_span(child, 5, sought_text.len()),
            "query variable must occur exactly once",
        ));
    }
    *index += 1;
    Ok(Query {
        model: model.to_owned(),
        relation: relation.name.clone(),
        sought,
        roles,
        span: line_span(child),
    })
}

fn parse_operation(
    lines: &[SourceLine<'_>],
    index: &mut usize,
    relations: &[RelationDecl],
    models: &[ModelDecl],
) -> Result<Operation, ParseError> {
    let line = lines[*index];
    let (kind, keyword) = if line.text.starts_with("claim ") {
        (OperationKind::Claim, "claim ")
    } else if line.text.starts_with("require ") {
        (OperationKind::Require, "require ")
    } else {
        return Err(error(line_span(line), "expected claim or require declaration"));
    };
    let model = line
        .text
        .strip_prefix(keyword)
        .and_then(|body| body.strip_suffix(':'))
        .ok_or_else(|| error(line_span(line), "operation declaration must be 'claim|require model:'"))?;
    if !is_name(model) || !models.iter().any(|candidate| candidate.name == model) {
        return Err(error(child_span(line, keyword.len(), model.len()), "operation refers to an unknown model"));
    }

    *index += 1;
    let child = lines
        .get(*index)
        .copied()
        .ok_or_else(|| error(line_span(line), "operation requires exactly one closed clause"))?;
    if !child.text.starts_with("    ") || child.text.len() == 4 {
        return Err(error(line_span(child), "operation body must be one indented closed clause"));
    }
    let input = &child.text[4..];
    let (relation, roles) = find_relation(relations, input, child, 5, false)?;
    let clause = Fact {
        relation: relation.name.clone(),
        roles,
        span: line_span(child),
    };
    *index += 1;
    if let Some(extra) = lines.get(*index).copied() {
        if extra.text.starts_with("    ") {
            return Err(error(line_span(extra), "operation requires exactly one closed clause"));
        }
    }
    Ok(Operation {
        kind,
        model: model.to_owned(),
        clause,
        span: line_span(line),
    })
}

fn parse_intent(
    lines: &[SourceLine<'_>],
    index: &mut usize,
    relations: &[RelationDecl],
    models: &[ModelDecl],
) -> Result<IntentDecl, ParseError> {
    let line = lines[*index];
    let name = line
        .text
        .strip_prefix("intent ")
        .and_then(|body| body.strip_suffix(':'))
        .ok_or_else(|| error(line_span(line), "intent declaration must be 'intent name:'"))?;
    if !is_name(name) {
        return Err(error(
            child_span(line, 7, name.len()),
            "invalid intent name",
        ));
    }
    if !models.iter().any(|model| {
        let namespace = format!("{}/", model.name);
        name.strip_prefix(&namespace)
            .is_some_and(|local_name| is_name(local_name))
    }) {
        return Err(error(
            child_span(line, 7, name.len()),
            "intent name must begin with a declared model namespace",
        ));
    }

    *index += 1;
    let child = lines
        .get(*index)
        .copied()
        .ok_or_else(|| error(line_span(line), "intent requires exactly one closed clause"))?;
    if !child.text.starts_with("    ") {
        return Err(error(
            line_span(child),
            "intent requires exactly one closed clause",
        ));
    }
    let input = &child.text[4..];
    let (relation, roles) = find_relation(relations, input, child, 5, false)?;
    let desired = Fact {
        relation: relation.name.clone(),
        roles,
        span: line_span(child),
    };
    *index += 1;
    if *index < lines.len() && lines[*index].text.starts_with("    ") {
        return Err(error(
            line_span(lines[*index]),
            "intent requires exactly one closed clause",
        ));
    }
    Ok(IntentDecl {
        name: name.to_owned(),
        desired,
        span: line_span(line),
    })
}

/// Parse the exact first Clause fixture into typed, source-spanned declarations.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let lines = source
        .lines()
        .enumerate()
        .map(|(index, text)| SourceLine {
            number: index + 1,
            text: text.strip_suffix('\r').unwrap_or(text),
        })
        .filter(|line| !line.text.is_empty())
        .collect::<Vec<_>>();
    let mut program = Program::default();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if line.text.starts_with("relation ") {
            let relation = parse_relation(&lines, &mut index)?;
            if program
                .relations
                .iter()
                .any(|candidate| candidate.name == relation.name)
            {
                return Err(error(
                    relation.span,
                    format!("duplicate relation '{}'", relation.name),
                ));
            }
            program.relations.push(relation);
        } else if line.text.starts_with("model ") {
            let model = parse_model(&lines, &mut index, &program.relations)?;
            if program
                .models
                .iter()
                .any(|candidate| candidate.name == model.name)
            {
                return Err(error(
                    model.span,
                    format!("duplicate model '{}'", model.name),
                ));
            }
            program.models.push(model);
        } else if line.text.starts_with("query ") {
            let query = parse_query(&lines, &mut index, &program.relations, &program.models)?;
            if program
                .queries
                .iter()
                .any(|candidate| candidate.model == query.model)
            {
                return Err(error(
                    query.span,
                    format!("duplicate query for model '{}'", query.model),
                ));
            }
            program.queries.push(query);
        } else if line.text.starts_with("claim ") || line.text.starts_with("require ") {
            let operation = parse_operation(&lines, &mut index, &program.relations, &program.models)?;
            program.operations.push(operation);
        } else if line.text.starts_with("intent ") {
            let intent = parse_intent(&lines, &mut index, &program.relations, &program.models)?;
            if program
                .intents
                .iter()
                .any(|candidate| candidate.name == intent.name)
            {
                return Err(error(
                    intent.span,
                    format!("duplicate intent name '{}'", intent.name),
                ));
            }
            program.intents.push(intent);
        } else {
            return Err(error(line_span(line), "unknown top-level declaration"));
        }
    }
    if program.relations.is_empty() || program.models.is_empty() || program.queries.is_empty() {
        return Err(error(
            Span {
                line: 1,
                column: 1,
                width: 0,
            },
            "program requires relation, model, and query declarations",
        ));
    }
    let names = relation_names(&program.relations);
    if program
        .models
        .iter()
        .flat_map(|model| model.facts.iter())
        .any(|fact| !names.contains(&fact.relation))
    {
        return Err(error(
            Span {
                line: 1,
                column: 1,
                width: 0,
            },
            "model contains an undeclared relation",
        ));
    }
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"relation catalog/contains(set: Text, member: Text):
    sentence: {set} contains {member}
    mode set -> member: many

model catalog:
    "letters" contains "a"
    "letters" contains "b"

query catalog:
    ?member where "letters" contains ?member
"#;

    fn m3_fixture() -> String {
        format!(
            "{FIXTURE}\nclaim catalog:\n    \"letters\" contains \"c\"\n\nrequire catalog:\n    \"letters\" contains \"c\"\n"
        )
    }

    const M4_FIXTURE: &str = r#"relation catalog/contains(set: Text, member: Text):
    sentence: {set} contains {member}
    mode set -> member: many

model catalog:
    "letters" contains "a"
    "letters" contains "b"

intent catalog/restock:
    "letters" contains "c"

query catalog:
    ?member where "letters" contains ?member
"#;

    #[test]
    fn parses_shared_fixture_with_spans_and_mode() {
        let program = parse(FIXTURE).expect("fixture parses");
        assert_eq!(program.relations.len(), 1);
        let relation = &program.relations[0];
        assert_eq!(relation.name, "catalog/contains");
        assert_eq!(
            relation
                .roles
                .iter()
                .map(|role| role.name.as_str())
                .collect::<Vec<_>>(),
            ["set", "member"]
        );
        assert_eq!(relation.mode.known, ["set"]);
        assert_eq!(relation.mode.sought, ["member"]);
        assert_eq!(relation.mode.cardinality, Cardinality::Many);
        assert_eq!(program.models[0].facts.len(), 2);
        assert_eq!(program.queries[0].sought, "member");
        assert_eq!(
            program.queries[0].roles["member"].kind,
            TermKind::Variable("member".to_owned())
        );
        assert_eq!(relation.sentence.span.line, 2);
    }

    #[test]
    fn rejects_unknown_sentence_role() {
        let source = FIXTURE.replace("{member}", "{unknown}");
        let error = parse(&source).expect_err("unknown role must fail");
        assert!(error.message.contains("unknown sentence role"));
        assert_eq!(error.span.line, 2);
    }

    #[test]
    fn rejects_duplicate_mode_role() {
        let source = FIXTURE.replace("mode set -> member: many", "mode set -> set: many");
        let error = parse(&source).expect_err("overlapping mode role must fail");
        assert!(error.message.contains("both known and sought"));
    }

    #[test]
    fn rejects_mismatched_bare_clause() {
        let source = FIXTURE.replace("\"letters\" contains \"b\"", "\"letters\" stores \"b\"");
        let error = parse(&source).expect_err("undeclared sentence must fail");
        assert!(error.message.contains("no declared sentence shape"));
    }

    #[test]
    fn rejects_duplicate_model_fact() {
        let source = FIXTURE.replace(
            "    \"letters\" contains \"b\"",
            "    \"letters\" contains \"a\"\n    \"letters\" contains \"b\"",
        );
        let error = parse(&source).expect_err("duplicate fact must fail");
        assert!(error.message.contains("duplicate model fact"));
    }
    #[test]
    fn parses_closed_claim_and_require_operations_in_source_order() {
        let program = parse(&m3_fixture()).expect("M3 fixture parses");
        assert_eq!(program.operations.len(), 2);
        assert_eq!(program.operations[0].kind, OperationKind::Claim);
        assert_eq!(program.operations[1].kind, OperationKind::Require);
        for operation in &program.operations {
            assert_eq!(operation.model, "catalog");
            assert_eq!(operation.clause.relation, "catalog/contains");
            assert_eq!(operation.clause.roles["set"].kind, TermKind::Text("letters".to_owned()));
        }
        assert_eq!(program.operations[0].clause.roles["member"].kind, TermKind::Text("c".to_owned()));
    }

    #[test]
    fn rejects_open_operation_clause() {
        let source = m3_fixture().replace("    \"letters\" contains \"c\"", "    \"letters\" contains ?member");
        let error = parse(&source).expect_err("open operation clause must fail");
        assert!(error.message.contains("no declared sentence shape"));
    }

    #[test]
    fn rejects_incomplete_operation_block() {
        let source = m3_fixture().replace("require catalog:\n    \"letters\" contains \"c\"", "require catalog:");
        let error = parse(&source).expect_err("operation without a body must fail");
        assert!(error.message.contains("exactly one closed clause"));
    }

    #[test]
    fn rejects_mismatched_operation_clause() {
        let source = m3_fixture().replace("require catalog:\n    \"letters\" contains \"c\"", "require catalog:\n    \"letters\" stores \"c\"");
        let error = parse(&source).expect_err("mismatched operation clause must fail");
        assert!(error.message.contains("no declared sentence shape"));
    }

    #[test]
    fn rejects_ambiguous_operation_clause() {
        let source = m3_fixture().replace(
            "model catalog:",
            "relation catalog/also(set: Text, member: Text):\n    sentence: {set} contains {member}\n    mode set -> member: many\n\nmodel catalog:",
        );
        let error = parse(&source).expect_err("ambiguous operation sentence must fail");
        assert!(error.message.contains("more than one declared sentence shape"));
    }

    #[test]
    fn rejects_extra_operation_clause() {
        let source = m3_fixture().replace(
            "require catalog:\n    \"letters\" contains \"c\"",
            "require catalog:\n    \"letters\" contains \"c\"\n    \"letters\" contains \"d\"",
        );
        let error = parse(&source).expect_err("operation with two clauses must fail");
        assert!(error.message.contains("exactly one closed clause"));
    }

    #[test]
    fn parses_closed_intent_resolved_by_declared_sentence() {
        let program = parse(M4_FIXTURE).expect("M4 fixture parses");
        assert_eq!(program.intents.len(), 1);
        let intent = &program.intents[0];
        assert_eq!(intent.name, "catalog/restock");
        assert_eq!(intent.desired.relation, "catalog/contains");
        assert_eq!(
            intent.desired.roles["set"].kind,
            TermKind::Text("letters".to_owned())
        );
        assert_eq!(
            intent.desired.roles["member"].kind,
            TermKind::Text("c".to_owned())
        );
        assert_eq!(intent.span.line, 9);
        assert_eq!(intent.desired.span.line, 10);
    }

    #[test]
    fn rejects_open_intent_clause() {
        let source = M4_FIXTURE.replace(
            "    \"letters\" contains \"c\"",
            "    \"letters\" contains ?member",
        );
        let error = parse(&source).expect_err("intent variables must fail");
        assert!(error.message.contains("no declared sentence shape"));
    }

    #[test]
    fn rejects_intent_with_unknown_sentence_shape() {
        let source = M4_FIXTURE.replace(
            "    \"letters\" contains \"c\"",
            "    \"letters\" stores \"c\"",
        );
        let error = parse(&source).expect_err("unknown intent sentence must fail");
        assert!(error.message.contains("no declared sentence shape"));
    }

    #[test]
    fn rejects_intent_outside_model_namespace() {
        let source = M4_FIXTURE.replace("intent catalog/restock:", "intent pantry/restock:");
        let error = parse(&source).expect_err("non-model intent namespace must fail");
        assert!(error.message.contains("declared model namespace"));
    }

    #[test]
    fn rejects_duplicate_intent_names() {
        let source = M4_FIXTURE.replace(
            "query catalog:",
            "intent catalog/restock:\n    \"letters\" contains \"c\"\n\nquery catalog:",
        );
        let error = parse(&source).expect_err("duplicate intents must fail");
        assert!(error.message.contains("duplicate intent name"));
    }

    #[test]
    fn rejects_multiple_intent_clauses() {
        let source = M4_FIXTURE.replace(
            "intent catalog/restock:\n    \"letters\" contains \"c\"",
            "intent catalog/restock:\n    \"letters\" contains \"c\"\n    \"letters\" contains \"d\"",
        );
        let error = parse(&source).expect_err("multiple intent clauses must fail");
        assert!(error.message.contains("exactly one closed clause"));
    }
}
