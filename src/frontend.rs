//! The singular source-facing Clause grammar.
//!
//! This reader is intentionally independent from the kernel.  It preserves
//! authoring names and spans, resolves source structure after every declaration
//! has been collected, and rejects the retired prefix surface outright.

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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(pub String);

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleName(pub String);

impl RoleName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeName(pub String);

impl TypeName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VariableName(pub String);

impl VariableName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Type,
    Relation,
    Model,
    Law,
    Revision,
    Delta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub declarations: Vec<AscriptionDecl>,
    pub requests: Vec<RequestDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AscriptionDecl {
    pub subject: Spanned<Name>,
    pub kind: Kind,
    pub body: Vec<Member>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Member {
    Sentence(SentenceShapeDecl),
    Mode(ModeDecl),
    Entity(EntityDecl),
    Clause(SurfaceClause),
    When(Vec<SurfaceClause>),
    From(Name),
    Apply(Name),
    Admit(Vec<SurfaceClause>),
    Withdraw(Vec<SurfaceClause>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceShapeDecl {
    pub parts: Vec<ShapePartDecl>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapePartDecl {
    Literal(Spanned<String>),
    Role {
        id: Spanned<RoleName>,
        typ: Spanned<TypeName>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cardinality {
    One,
    Maybe,
    Some,
    Many,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDecl {
    pub known: Vec<Spanned<RoleName>>,
    pub sought: Vec<Spanned<RoleName>>,
    pub cardinality: Cardinality,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityDecl {
    pub local: Spanned<Name>,
    pub typ: Spanned<TypeName>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SurfaceTerm {
    Entity(Spanned<Name>),
    Variable(Spanned<VariableName>),
    String(Spanned<String>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceClause {
    pub relation: Spanned<Name>,
    pub roles: BTreeMap<RoleName, SurfaceTerm>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterventionSelection {
    OneMinimal,
    AllMinimal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestDecl {
    Find {
        revision: Spanned<Name>,
        pattern: SurfaceClause,
        sought: Spanned<VariableName>,
        span: Span,
    },
    Why {
        revision: Spanned<Name>,
        target: SurfaceClause,
        all: bool,
        span: Span,
    },
    Prevent {
        revision: Spanned<Name>,
        target: SurfaceClause,
        selection: InterventionSelection,
        using: Vec<Spanned<Name>>,
        span: Span,
    },
    Achieve {
        revision: Spanned<Name>,
        target: SurfaceClause,
        selection: InterventionSelection,
        using: Vec<Spanned<Name>>,
        span: Span,
    },
    Diff {
        base: Spanned<Name>,
        successor: Spanned<Name>,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug)]
struct SourceLine<'a> {
    number: usize,
    text: &'a str,
}

#[derive(Clone, Debug)]
struct RawDecl<'a> {
    subject: Spanned<Name>,
    kind: Kind,
    header: SourceLine<'a>,
    body: Vec<SourceLine<'a>>,
}

#[derive(Clone, Debug)]
enum RawRequest<'a> {
    Find {
        revision: Spanned<Name>,
        sought: Spanned<VariableName>,
        clause: SourceLine<'a>,
        header: SourceLine<'a>,
    },
    Why {
        revision: Spanned<Name>,
        all: bool,
        clause: SourceLine<'a>,
        header: SourceLine<'a>,
    },
    Intervention {
        verb: &'static str,
        revision: Spanned<Name>,
        selection: InterventionSelection,
        clause: SourceLine<'a>,
        using: Vec<Spanned<Name>>,
        header: SourceLine<'a>,
    },
    Diff {
        base: Spanned<Name>,
        successor: Spanned<Name>,
        header: SourceLine<'a>,
    },
}

#[derive(Clone, Debug)]
struct RelationSpec {
    shape: SentenceShapeDecl,
    modes: Vec<ModeDecl>,
    roles: BTreeMap<RoleName, TypeName>,
}

#[derive(Clone, Debug)]
struct ChangeLayout<'a> {
    from: Spanned<Name>,
    apply: Option<Spanned<Name>>,
    admit: Option<Vec<SourceLine<'a>>>,
    withdraw: Option<Vec<SourceLine<'a>>>,
}

#[derive(Clone, Debug)]
struct LawLayout<'a> {
    conclusion: SourceLine<'a>,
    premises: Vec<SourceLine<'a>>,
}

#[derive(Clone, Debug)]
struct Token {
    raw: String,
    quoted: bool,
    span: Span,
}

fn error(span: Span, message: impl Into<String>) -> ParseError {
    ParseError {
        span,
        message: message.into(),
    }
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

fn indent(line: SourceLine<'_>) -> Result<usize, ParseError> {
    if line.text.contains('\t') {
        return Err(error(line_span(line), "tabs are not permitted"));
    }
    if line.text.ends_with('\r') {
        return Err(error(line_span(line), "carriage returns are not permitted"));
    }
    if line.text.trim().is_empty() {
        if line.text.is_empty() {
            return Ok(0);
        }
        return Err(error(line_span(line), "blank lines must be empty"));
    }
    let width = line.text.bytes().take_while(|byte| *byte == b' ').count();
    if !matches!(width, 0 | 4 | 8) {
        return Err(error(
            child_span(line, 0, width.max(1)),
            "indentation must be exactly zero, four, or eight ASCII spaces",
        ));
    }
    Ok(width)
}

fn content(line: SourceLine<'_>) -> &str {
    &line.text[indent(line).expect("validated source lines")..]
}

fn is_local_name(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn is_qname(text: &str) -> bool {
    !text.is_empty() && text.split('/').all(is_local_name)
}

fn qname(line: SourceLine<'_>, offset: usize, text: &str) -> Result<Spanned<Name>, ParseError> {
    if !is_qname(text) {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected qualified name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: Name(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

fn role_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<RoleName>, ParseError> {
    if !is_local_name(text) {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected role name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: RoleName(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

fn variable_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<VariableName>, ParseError> {
    if !is_local_name(text) {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected variable name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: VariableName(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

fn type_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<TypeName>, ParseError> {
    let name = qname(line, offset, text)?;
    Ok(Spanned {
        value: TypeName(name.value.0),
        span: name.span,
    })
}

fn kind(text: &str) -> Option<Kind> {
    match text {
        "Type" => Some(Kind::Type),
        "Relation" => Some(Kind::Relation),
        "Model" => Some(Kind::Model),
        "Law" => Some(Kind::Law),
        "Revision" => Some(Kind::Revision),
        "Delta" => Some(Kind::Delta),
        _ => None,
    }
}

fn take_body<'a>(lines: &[SourceLine<'a>], index: &mut usize) -> Vec<SourceLine<'a>> {
    let mut body = Vec::new();
    while *index < lines.len() {
        let line = lines[*index];
        if !line.text.is_empty() && indent(line).expect("validated source lines") == 0 {
            break;
        }
        body.push(line);
        *index += 1;
    }
    body
}

fn nonblank<'a>(lines: impl IntoIterator<Item = SourceLine<'a>>) -> Vec<SourceLine<'a>> {
    lines
        .into_iter()
        .filter(|line| !line.text.is_empty())
        .collect()
}

fn read_clause_block<'a>(
    lines: &[SourceLine<'a>],
    index: &mut usize,
    description: &str,
) -> Result<SourceLine<'a>, ParseError> {
    let body = take_body(lines, index);
    let entries = nonblank(body);
    if entries.len() != 1
        || entries
            .first()
            .is_some_and(|line| indent(*line).unwrap() != 4)
    {
        let span = entries.first().map_or(
            Span {
                line: 1,
                column: 1,
                width: 0,
            },
            |line| line_span(*line),
        );
        return Err(error(
            span,
            format!("{description} requires exactly one four-space clause"),
        ));
    }
    Ok(entries[0])
}

fn parse_declaration<'a>(
    line: SourceLine<'a>,
    lines: &[SourceLine<'a>],
    index: &mut usize,
) -> Result<RawDecl<'a>, ParseError> {
    let text = content(line);
    let (subject, kind_text) = text
        .split_once(": ")
        .ok_or_else(|| error(line_span(line), "expected '<name>: <Kind>' declaration"))?;
    let declaration_kind =
        kind(kind_text).ok_or_else(|| error(line_span(line), "unknown declaration kind"))?;
    let subject = qname(line, 0, subject)?;
    *index += 1;
    Ok(RawDecl {
        subject,
        kind: declaration_kind,
        header: line,
        body: take_body(lines, index),
    })
}

fn parse_request<'a>(
    line: SourceLine<'a>,
    lines: &[SourceLine<'a>],
    index: &mut usize,
) -> Result<RawRequest<'a>, ParseError> {
    let text = content(line);
    *index += 1;
    if let Some(rest) = text.strip_prefix("find all ?") {
        let (sought_text, revision_text) = rest
            .split_once(" in ")
            .ok_or_else(|| error(line_span(line), "expected 'find all ?name in revision:'"))?;
        let revision_text = revision_text
            .strip_suffix(':')
            .ok_or_else(|| error(line_span(line), "find request requires ':'"))?;
        let sought = variable_name(line, "find all ?".len(), sought_text)?;
        let revision = qname(line, text.len() - revision_text.len() - 1, revision_text)?;
        let clause = read_clause_block(lines, index, "find request")?;
        return Ok(RawRequest::Find {
            revision,
            sought,
            clause,
            header: line,
        });
    }
    let (all, revision_text) = if let Some(rest) = text.strip_prefix("why all in ") {
        (true, rest)
    } else if let Some(rest) = text.strip_prefix("why in ") {
        (false, rest)
    } else {
        (false, "")
    };
    if !revision_text.is_empty() {
        let revision_text = revision_text
            .strip_suffix(':')
            .ok_or_else(|| error(line_span(line), "expected 'why [all] in revision:'"))?;
        let revision = qname(line, text.len() - revision_text.len() - 1, revision_text)?;
        let clause = read_clause_block(lines, index, "why request")?;
        return Ok(RawRequest::Why {
            revision,
            all,
            clause,
            header: line,
        });
    }
    for verb in ["prevent", "achieve"] {
        if let Some(rest) = text.strip_prefix(&format!("{verb} ")) {
            let (selection, revision_text) =
                if let Some(rest) = rest.strip_prefix("one minimal in ") {
                    (InterventionSelection::OneMinimal, rest)
                } else if let Some(rest) = rest.strip_prefix("all minimal in ") {
                    (InterventionSelection::AllMinimal, rest)
                } else {
                    return Err(error(
                        line_span(line),
                        "expected 'one minimal' or 'all minimal'",
                    ));
                };
            let revision_text = revision_text
                .strip_suffix(':')
                .ok_or_else(|| error(line_span(line), "intervention request requires ':'"))?;
            let revision = qname(line, text.len() - revision_text.len() - 1, revision_text)?;
            let clause = read_clause_block(lines, index, "intervention request")?;
            while *index < lines.len() && lines[*index].text.is_empty() {
                *index += 1;
            }
            let using_header = lines
                .get(*index)
                .copied()
                .ok_or_else(|| error(line_span(line), "intervention request requires 'using:'"))?;
            if indent(using_header)? != 0 || content(using_header) != "using:" {
                return Err(error(
                    line_span(using_header),
                    "intervention request requires 'using:'",
                ));
            }
            *index += 1;
            let using_lines = nonblank(take_body(lines, index));
            if using_lines.is_empty()
                || using_lines.iter().any(|entry| indent(*entry).unwrap() != 4)
            {
                return Err(error(
                    line_span(using_header),
                    "using requires one or more four-space relation references",
                ));
            }
            let using = using_lines
                .into_iter()
                .map(|entry| qname(entry, 4, content(entry)))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(RawRequest::Intervention {
                verb,
                revision,
                selection,
                clause,
                using,
                header: line,
            });
        }
    }
    if let Some(rest) = text.strip_prefix("diff ") {
        let (base, successor) = rest
            .split_once(" -> ")
            .ok_or_else(|| error(line_span(line), "expected 'diff base -> successor'"))?;
        return Ok(RawRequest::Diff {
            base: qname(line, "diff ".len(), base)?,
            successor: qname(line, "diff ".len() + base.len() + " -> ".len(), successor)?,
            header: line,
        });
    }
    Err(error(line_span(line), "unknown request"))
}

fn scan<'a>(source: &'a str) -> Result<(Vec<RawDecl<'a>>, Vec<RawRequest<'a>>), ParseError> {
    let lines = source
        .split('\n')
        .enumerate()
        .map(|(index, text)| SourceLine {
            number: index + 1,
            text,
        })
        .collect::<Vec<_>>();
    for line in &lines {
        indent(*line)?;
    }
    let mut declarations = Vec::new();
    let mut requests = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if line.text.is_empty() {
            index += 1;
            continue;
        }
        if indent(line)? != 0 {
            return Err(error(
                line_span(line),
                "unexpected indentation at top level",
            ));
        }
        let text = content(line);
        if text.starts_with("find ")
            || text.starts_with("why in ")
            || text.starts_with("why all in ")
            || text.starts_with("prevent ")
            || text.starts_with("achieve ")
            || text.starts_with("diff ")
        {
            requests.push(parse_request(line, &lines, &mut index)?);
        } else {
            declarations.push(parse_declaration(line, &lines, &mut index)?);
        }
    }
    Ok((declarations, requests))
}

fn parse_shape(line: SourceLine<'_>) -> Result<SentenceShapeDecl, ParseError> {
    let text = content(line);
    let mut parts = Vec::new();
    let offset = 4;
    let mut cursor = 0;
    while cursor < text.len() {
        let open = text[cursor..]
            .find('{')
            .map(|index| cursor + index)
            .ok_or_else(|| error(line_span(line), "sentence shape must end with a role"))?;
        let literal = &text[cursor..open];
        if !literal.trim().is_empty() {
            let trimmed = literal.trim();
            if trimmed.contains('}')
                || trimmed.contains('{')
                || trimmed.contains('"')
                || trimmed.contains('?')
            {
                return Err(error(
                    child_span(line, offset + cursor, literal.len()),
                    "invalid literal in sentence shape",
                ));
            }
            let canonical = trimmed
                .split_ascii_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let leading = literal.len() - literal.trim_start().len();
            parts.push(ShapePartDecl::Literal(Spanned {
                value: canonical,
                span: child_span(line, offset + cursor + leading, trimmed.len()),
            }));
        } else if !parts.is_empty() {
            return Err(error(
                child_span(line, offset + cursor, literal.len()),
                "roles require a nonempty literal between them",
            ));
        }
        let close = text[open + 1..]
            .find('}')
            .map(|index| open + 1 + index)
            .ok_or_else(|| {
                error(
                    child_span(line, offset + open, text.len() - open),
                    "unterminated typed role",
                )
            })?;
        let inside = &text[open + 1..close];
        let (role, typ) = inside.split_once(": ").ok_or_else(|| {
            error(
                child_span(line, offset + open, close - open + 1),
                "expected '{role: Type}'",
            )
        })?;
        if role.contains(':') || typ.contains(':') {
            return Err(error(
                child_span(line, offset + open, close - open + 1),
                "malformed typed role",
            ));
        }
        parts.push(ShapePartDecl::Role {
            id: role_name(line, offset + open + 1, role)?,
            typ: type_name(line, offset + open + 1 + role.len() + 2, typ)?,
        });
        cursor = close + 1;
    }
    if parts.len() < 3
        || !matches!(parts.first(), Some(ShapePartDecl::Role { .. }))
        || !matches!(parts.last(), Some(ShapePartDecl::Role { .. }))
    {
        return Err(error(
            line_span(line),
            "sentence shape must begin and end with roles and contain at least two roles",
        ));
    }
    let mut roles = BTreeSet::new();
    for part in &parts {
        if let ShapePartDecl::Role { id, .. } = part {
            if !roles.insert(id.value.clone()) {
                return Err(error(
                    id.span,
                    format!("duplicate inline role '{}'", id.value.as_str()),
                ));
            }
        }
    }
    Ok(SentenceShapeDecl {
        parts,
        span: line_span(line),
    })
}

fn parse_role_list(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Vec<Spanned<RoleName>>, ParseError> {
    if text.is_empty() {
        return Err(error(
            child_span(line, offset, 0),
            "role list cannot be empty",
        ));
    }
    let mut roles = Vec::new();
    let mut seen = BTreeSet::new();
    let mut position = 0;
    for item in text.split(", ") {
        if item.is_empty() || item.contains(',') {
            return Err(error(
                child_span(line, offset + position, item.len()),
                "roles must be separated by ', '",
            ));
        }
        let role = role_name(line, offset + position, item)?;
        if !seen.insert(role.value.clone()) {
            return Err(error(
                role.span,
                format!("duplicate mode role '{}'", role.value.as_str()),
            ));
        }
        position += item.len() + 2;
        roles.push(role);
    }
    Ok(roles)
}

fn parse_mode(
    line: SourceLine<'_>,
    roles: &BTreeMap<RoleName, TypeName>,
) -> Result<ModeDecl, ParseError> {
    let text = content(line);
    let rest = text.strip_prefix("mode ").ok_or_else(|| {
        error(
            line_span(line),
            "relation members after the shape must be mode declarations",
        )
    })?;
    let (sides, cardinality) = rest
        .rsplit_once(": ")
        .ok_or_else(|| error(line_span(line), "expected mode cardinality"))?;
    let (known, sought) = sides
        .split_once(" -> ")
        .ok_or_else(|| error(line_span(line), "expected 'known -> sought' mode"))?;
    let known = parse_role_list(line, 4 + "mode ".len(), known)?;
    let sought = parse_role_list(line, 4 + "mode ".len() + known_text_width(&known), sought)?;
    let mut every = BTreeSet::new();
    for role in known.iter().chain(&sought) {
        if !roles.contains_key(&role.value) {
            return Err(error(
                role.span,
                format!("unknown mode role '{}'", role.value.as_str()),
            ));
        }
        if !every.insert(role.value.clone()) {
            return Err(error(
                role.span,
                format!("role '{}' is both known and sought", role.value.as_str()),
            ));
        }
    }
    let cardinality = match cardinality {
        "one" => Cardinality::One,
        "maybe" => Cardinality::Maybe,
        "some" => Cardinality::Some,
        "many" => Cardinality::Many,
        _ => {
            return Err(error(
                line_span(line),
                format!("unknown mode cardinality '{cardinality}'"),
            ));
        }
    };
    Ok(ModeDecl {
        known,
        sought,
        cardinality,
        span: line_span(line),
    })
}

fn known_text_width(roles: &[Spanned<RoleName>]) -> usize {
    roles.iter().map(|role| role.value.0.len()).sum::<usize>()
        + roles.len().saturating_sub(1) * 2
        + " -> ".len()
}

fn relation_spec(raw: &RawDecl<'_>) -> Result<RelationSpec, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    if entries.is_empty() || entries.iter().any(|line| indent(*line).unwrap() != 4) {
        return Err(error(
            line_span(raw.header),
            "Relation requires four-space sentence and mode members",
        ));
    }
    let shape = parse_shape(entries[0])?;
    let mut roles = BTreeMap::new();
    for part in &shape.parts {
        if let ShapePartDecl::Role { id, typ } = part {
            roles.insert(id.value.clone(), typ.value.clone());
        }
    }
    let modes = entries[1..]
        .iter()
        .copied()
        .map(|line| parse_mode(line, &roles))
        .collect::<Result<Vec<_>, _>>()?;
    if modes.is_empty() {
        return Err(error(
            line_span(raw.header),
            "Relation requires one or more mode declarations",
        ));
    }
    Ok(RelationSpec {
        shape,
        modes,
        roles,
    })
}

fn entity_line(line: SourceLine<'_>) -> Option<Result<EntityDecl, ParseError>> {
    let text = content(line);
    let (local, typ) = text.split_once(": ")?;
    if local.contains(':') || typ.contains(':') {
        return None;
    }
    Some((|| {
        Ok(EntityDecl {
            local: qname(line, 4, local)?,
            typ: type_name(line, 4 + local.len() + 2, typ)?,
            span: line_span(line),
        })
    })())
}

fn model_entities(raw: &RawDecl<'_>) -> Result<BTreeMap<Name, TypeName>, ParseError> {
    let mut entities = BTreeMap::new();
    for line in nonblank(raw.body.iter().copied()) {
        if indent(line)? != 4 {
            return Err(error(
                line_span(line),
                "Model members must use four-space indentation",
            ));
        }
        if let Some(entity) = entity_line(line) {
            let entity = entity?;
            if entity.local.value.0.contains('/') {
                return Err(error(
                    entity.local.span,
                    "model entity names cannot be qualified",
                ));
            }
            if entities
                .insert(entity.local.value.clone(), entity.typ.value.clone())
                .is_some()
            {
                return Err(error(
                    entity.local.span,
                    format!("duplicate entity '{}'", entity.local.value.as_str()),
                ));
            }
        }
    }
    Ok(entities)
}

fn parse_law_layout<'a>(raw: &RawDecl<'a>) -> Result<LawLayout<'a>, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    if entries.len() < 3
        || indent(entries[0])? != 4
        || indent(entries[1])? != 4
        || content(entries[1]) != "when:"
    {
        return Err(error(
            line_span(raw.header),
            "Law requires one conclusion followed by 'when:' and premises",
        ));
    }
    let premises = entries[2..].to_vec();
    if premises.is_empty() || premises.iter().any(|line| indent(*line).unwrap() != 8) {
        return Err(error(
            line_span(raw.header),
            "when requires one or more eight-space clauses",
        ));
    }
    Ok(LawLayout {
        conclusion: entries[0],
        premises,
    })
}

fn parse_change_layout<'a>(raw: &RawDecl<'a>) -> Result<ChangeLayout<'a>, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    let first = entries
        .first()
        .copied()
        .ok_or_else(|| error(line_span(raw.header), "Revision and Delta require 'from:'"))?;
    if indent(first)? != 4 || !content(first).starts_with("from: ") {
        return Err(error(
            line_span(first),
            "first member must be 'from: revision'",
        ));
    }
    let from_text = &content(first)["from: ".len()..];
    let from = qname(first, 4 + "from: ".len(), from_text)?;
    let mut apply = None;
    let mut admit = None;
    let mut withdraw = None;
    let mut index = 1;
    while index < entries.len() {
        let member = entries[index];
        if indent(member)? != 4 {
            return Err(error(line_span(member), "unexpected nested member"));
        }
        match content(member) {
            text if text.starts_with("from:") => {
                return Err(error(line_span(member), "exactly one 'from:' is required"));
            }
            text if text.starts_with("apply: ") => {
                if apply.is_some() {
                    return Err(error(
                        line_span(member),
                        "exactly one 'apply:' is permitted",
                    ));
                }
                apply = Some(qname(
                    member,
                    4 + "apply: ".len(),
                    &text["apply: ".len()..],
                )?);
                index += 1;
            }
            "admit:" | "withdraw:" => {
                let is_admit = content(member) == "admit:";
                if (is_admit && admit.is_some()) || (!is_admit && withdraw.is_some()) {
                    return Err(error(line_span(member), "change blocks occur at most once"));
                }
                index += 1;
                let start = index;
                while index < entries.len() && indent(entries[index])? == 8 {
                    index += 1;
                }
                if start == index {
                    return Err(error(
                        line_span(member),
                        "change blocks require one or more clauses",
                    ));
                }
                let clauses = entries[start..index].to_vec();
                if is_admit {
                    if withdraw.is_some() {
                        return Err(error(line_span(member), "admit must precede withdraw"));
                    }
                    admit = Some(clauses);
                } else {
                    withdraw = Some(clauses);
                }
            }
            _ => return Err(error(line_span(member), "unknown Revision or Delta member")),
        }
    }
    if apply.is_some() && (admit.is_some() || withdraw.is_some()) {
        return Err(error(
            line_span(raw.header),
            "Revision has either apply or a local change set, not both",
        ));
    }
    if apply.is_none() && admit.is_none() && withdraw.is_none() {
        return Err(error(
            line_span(raw.header),
            "Revision and Delta require a nonempty change set or apply",
        ));
    }
    if raw.kind == Kind::Delta && apply.is_some() {
        return Err(error(
            line_span(raw.header),
            "Delta cannot apply another Delta",
        ));
    }
    Ok(ChangeLayout {
        from,
        apply,
        admit,
        withdraw,
    })
}

fn lex_clause(line: SourceLine<'_>) -> Result<Vec<Token>, ParseError> {
    let text = content(line);
    let base = indent(line)?;
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < text.len() {
        if text.as_bytes()[index] == b' ' {
            index += 1;
            continue;
        }
        if text.as_bytes()[index].is_ascii_whitespace() {
            return Err(error(
                child_span(line, base + index, 1),
                "clause words must be separated by ASCII spaces",
            ));
        }
        let start = index;
        if text.as_bytes()[index] == b'"' {
            index += 1;
            let mut value = String::new();
            let mut closed = false;
            while index < text.len() {
                match text.as_bytes()[index] {
                    b'"' => {
                        index += 1;
                        closed = true;
                        break;
                    }
                    b'\\' => {
                        index += 1;
                        let escaped = *text.as_bytes().get(index).ok_or_else(|| {
                            error(
                                child_span(line, base + start, text.len() - start),
                                "unterminated escape",
                            )
                        })?;
                        value.push(match escaped {
                            b'"' => '"',
                            b'\\' => '\\',
                            b'n' => '\n',
                            b't' => '\t',
                            _ => {
                                return Err(error(
                                    child_span(line, base + index - 1, 2),
                                    "unsupported string escape",
                                ));
                            }
                        });
                        index += 1;
                    }
                    byte => {
                        value.push(byte as char);
                        index += 1;
                    }
                }
            }
            if !closed {
                return Err(error(
                    child_span(line, base + start, text.len() - start),
                    "unterminated string",
                ));
            }
            if index < text.len() && text.as_bytes()[index] != b' ' {
                return Err(error(
                    child_span(line, base + index, 1),
                    "string must be followed by a space",
                ));
            }
            tokens.push(Token {
                raw: value,
                quoted: true,
                span: child_span(line, base + start, index - start),
            });
        } else {
            while index < text.len() && text.as_bytes()[index] != b' ' {
                if text.as_bytes()[index].is_ascii_whitespace() || text.as_bytes()[index] == b'"' {
                    return Err(error(
                        child_span(line, base + index, 1),
                        "invalid clause token",
                    ));
                }
                index += 1;
            }
            tokens.push(Token {
                raw: text[start..index].to_owned(),
                quoted: false,
                span: child_span(line, base + start, index - start),
            });
        }
    }
    if tokens.is_empty() {
        return Err(error(line_span(line), "clause cannot be empty"));
    }
    Ok(tokens)
}

fn parse_term(token: &Token) -> Result<SurfaceTerm, ParseError> {
    if token.quoted {
        return Ok(SurfaceTerm::String(Spanned {
            value: token.raw.clone(),
            span: token.span,
        }));
    }
    if let Some(name) = token.raw.strip_prefix('?') {
        return Ok(SurfaceTerm::Variable(variable_name(
            SourceLine {
                number: token.span.line,
                text: "",
            },
            token.span.column - 1,
            name,
        )?));
    }
    if !is_qname(&token.raw) {
        return Err(error(
            token.span,
            format!("expected entity name or variable, found '{}'", token.raw),
        ));
    }
    Ok(SurfaceTerm::Entity(Spanned {
        value: Name(token.raw.clone()),
        span: token.span,
    }))
}

fn entity_type(
    term: &SurfaceTerm,
    current_model: &Name,
    entities: &BTreeMap<Name, BTreeMap<Name, TypeName>>,
) -> Result<Option<TypeName>, ParseError> {
    match term {
        SurfaceTerm::String(_) => Ok(Some(TypeName("Text".to_owned()))),
        SurfaceTerm::Variable(_) => Ok(None),
        SurfaceTerm::Entity(entity) => {
            if !entity.value.0.contains('/') {
                return entities
                    .get(current_model)
                    .and_then(|model| model.get(&entity.value))
                    .cloned()
                    .map(Some)
                    .ok_or_else(|| {
                        error(
                            entity.span,
                            format!("unknown entity '{}'", entity.value.as_str()),
                        )
                    });
            }
            for (model, locals) in entities {
                for (local, typ) in locals {
                    if entity.value.0 == format!("{}/{}", model.as_str(), local.as_str()) {
                        return Ok(Some(typ.clone()));
                    }
                }
            }
            Err(error(
                entity.span,
                format!("unknown qualified entity '{}'", entity.value.as_str()),
            ))
        }
    }
}

fn shape_tokens(shape: &SentenceShapeDecl) -> Vec<Option<String>> {
    let mut tokens = Vec::new();
    for part in &shape.parts {
        match part {
            ShapePartDecl::Literal(value) => {
                tokens.extend(value.value.split(' ').map(|word| Some(word.to_owned())))
            }
            ShapePartDecl::Role { .. } => tokens.push(None),
        }
    }
    tokens
}

fn clause(
    line: SourceLine<'_>,
    current_model: &Name,
    relations: &BTreeMap<Name, RelationSpec>,
    entities: &BTreeMap<Name, BTreeMap<Name, TypeName>>,
    variable_types: &mut BTreeMap<VariableName, TypeName>,
) -> Result<SurfaceClause, ParseError> {
    let tokens = lex_clause(line)?;
    let mut candidates = Vec::new();
    for (relation, spec) in relations {
        let pattern = shape_tokens(&spec.shape);
        if pattern.len() != tokens.len() {
            continue;
        }
        let mut terms = BTreeMap::new();
        let mut role_index = 0;
        let mut matches = true;
        for (part, token) in pattern.iter().zip(&tokens) {
            match part {
                Some(word) if !token.quoted && token.raw == *word => {}
                Some(_) => {
                    matches = false;
                    break;
                }
                None => {
                    let role = match &spec
                        .shape
                        .parts
                        .iter()
                        .filter_map(|part| match part {
                            ShapePartDecl::Role { id, .. } => Some(id.value.clone()),
                            ShapePartDecl::Literal(_) => None,
                        })
                        .collect::<Vec<_>>()[role_index]
                    {
                        role => role.clone(),
                    };
                    role_index += 1;
                    let term = parse_term(token)?;
                    let expected = spec.roles.get(&role).expect("shape roles populate spec");
                    if let Some(actual) = entity_type(&term, current_model, entities)? {
                        if &actual != expected {
                            matches = false;
                            break;
                        }
                    }
                    if let SurfaceTerm::Variable(variable) = &term {
                        if let Some(previous) = variable_types.get(&variable.value) {
                            if previous != expected {
                                matches = false;
                                break;
                            }
                        }
                    }
                    terms.insert(role, term);
                }
            }
        }
        if matches {
            candidates.push((relation.clone(), terms));
        }
    }
    if candidates.is_empty() {
        return Err(error(
            line_span(line),
            "no declared sentence shape accepts this clause",
        ));
    }
    if candidates.len() > 1 {
        let names = candidates
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(error(
            line_span(line),
            format!("ambiguous clause; candidates: {names}"),
        ));
    }
    let (relation, roles) = candidates.pop().expect("nonempty candidates");
    let spec = relations.get(&relation).expect("candidate relation exists");
    for (role, term) in &roles {
        if let SurfaceTerm::Variable(variable) = term {
            variable_types.insert(variable.value.clone(), spec.roles[role].clone());
        }
    }
    Ok(SurfaceClause {
        relation: Spanned {
            value: relation,
            span: line_span(line),
        },
        roles,
        span: line_span(line),
    })
}

fn ground(clause: &SurfaceClause) -> bool {
    clause
        .roles
        .values()
        .all(|term| !matches!(term, SurfaceTerm::Variable(_)))
}

fn clause_key(clause: &SurfaceClause) -> String {
    let mut key = clause.relation.value.0.clone();
    for (role, term) in &clause.roles {
        key.push('|');
        key.push_str(role.as_str());
        key.push('=');
        match term {
            SurfaceTerm::Entity(value) => key.push_str(&format!("E:{}", value.value.0)),
            SurfaceTerm::Variable(value) => key.push_str(&format!("V:{}", value.value.0)),
            SurfaceTerm::String(value) => key.push_str(&format!("S:{:?}", value.value)),
        }
    }
    key
}

fn variables(clause: &SurfaceClause) -> BTreeSet<VariableName> {
    clause
        .roles
        .values()
        .filter_map(|term| match term {
            SurfaceTerm::Variable(value) => Some(value.value.clone()),
            _ => None,
        })
        .collect()
}

fn declared_model_for_law(
    name: &Name,
    models: &BTreeMap<Name, BTreeMap<Name, TypeName>>,
) -> Option<Name> {
    models
        .keys()
        .filter(|model| {
            name.0
                .strip_prefix(&format!("{}/", model.as_str()))
                .is_some()
        })
        .max_by_key(|model| model.0.len())
        .cloned()
}

fn reference_kind(
    name: &Spanned<Name>,
    kinds: &BTreeMap<Name, Kind>,
    allowed: &[Kind],
    description: &str,
) -> Result<(), ParseError> {
    match kinds.get(&name.value) {
        Some(kind) if allowed.contains(kind) => Ok(()),
        _ => Err(error(
            name.span,
            format!("{description} '{}' is not declared", name.value.as_str()),
        )),
    }
}

fn check_cycles(
    kinds: &BTreeMap<Name, Kind>,
    layouts: &BTreeMap<Name, ChangeLayout<'_>>,
) -> Result<(), ParseError> {
    fn visit(
        node: &Name,
        kinds: &BTreeMap<Name, Kind>,
        layouts: &BTreeMap<Name, ChangeLayout<'_>>,
        active: &mut BTreeSet<Name>,
        settled: &mut BTreeSet<Name>,
    ) -> Result<(), ParseError> {
        if settled.contains(node) {
            return Ok(());
        }
        if !active.insert(node.clone()) {
            return Err(error(
                Span {
                    line: 1,
                    column: 1,
                    width: 0,
                },
                format!("Revision/Delta dependency cycle at '{}'", node.as_str()),
            ));
        }
        let layout = layouts.get(node).expect("layout for revision or delta");
        let mut dependencies = vec![layout.from.value.clone()];
        if let Some(apply) = &layout.apply {
            dependencies.push(apply.value.clone());
        }
        for dependency in dependencies {
            if matches!(kinds.get(&dependency), Some(Kind::Revision | Kind::Delta)) {
                visit(&dependency, kinds, layouts, active, settled)?;
            }
        }
        active.remove(node);
        settled.insert(node.clone());
        Ok(())
    }
    let mut active = BTreeSet::new();
    let mut settled = BTreeSet::new();
    for node in layouts.keys() {
        visit(node, kinds, layouts, &mut active, &mut settled)?;
    }
    Ok(())
}

fn revision_model(
    name: &Name,
    kinds: &BTreeMap<Name, Kind>,
    layouts: &BTreeMap<Name, ChangeLayout<'_>>,
) -> Name {
    match kinds[name] {
        Kind::Model => name.clone(),
        Kind::Revision => revision_model(&layouts[name].from.value, kinds, layouts),
        _ => unreachable!("validated revision reference"),
    }
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let (raw_declarations, raw_requests) = scan(source)?;
    let mut kinds = BTreeMap::new();
    for declaration in &raw_declarations {
        if kinds
            .insert(declaration.subject.value.clone(), declaration.kind)
            .is_some()
        {
            return Err(error(
                declaration.subject.span,
                format!(
                    "duplicate declaration '{}'",
                    declaration.subject.value.as_str()
                ),
            ));
        }
    }
    let types = raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Type)
        .map(|declaration| declaration.subject.value.clone())
        .collect::<BTreeSet<_>>();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Type)
    {
        if nonblank(declaration.body.iter().copied()).is_empty() {
            continue;
        }
        return Err(error(
            line_span(declaration.header),
            "Type declarations cannot have members",
        ));
    }
    let mut relations = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Relation)
    {
        let spec = relation_spec(declaration)?;
        for typ in spec.roles.values() {
            if !types.contains(&Name(typ.0.clone())) {
                return Err(error(
                    line_span(declaration.header),
                    format!("unknown role type '{}'", typ.as_str()),
                ));
            }
        }
        relations.insert(declaration.subject.value.clone(), spec);
    }
    let mut entities = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Model)
    {
        let model_entities = model_entities(declaration)?;
        for typ in model_entities.values() {
            if !types.contains(&Name(typ.0.clone())) {
                return Err(error(
                    line_span(declaration.header),
                    format!("unknown entity type '{}'", typ.as_str()),
                ));
            }
        }
        entities.insert(declaration.subject.value.clone(), model_entities);
    }
    let mut layouts = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| matches!(declaration.kind, Kind::Revision | Kind::Delta))
    {
        layouts.insert(
            declaration.subject.value.clone(),
            parse_change_layout(declaration)?,
        );
    }
    for (name, layout) in &layouts {
        reference_kind(
            &layout.from,
            &kinds,
            &[Kind::Model, Kind::Revision],
            "revision base",
        )?;
        if let Some(apply) = &layout.apply {
            reference_kind(apply, &kinds, &[Kind::Delta], "applied Delta")?;
        }
        if kinds[name] == Kind::Delta && layout.apply.is_some() {
            return Err(error(
                layout.apply.as_ref().expect("some").span,
                "Delta cannot apply another Delta",
            ));
        }
    }
    check_cycles(&kinds, &layouts)?;

    let mut declarations = Vec::new();
    for raw in &raw_declarations {
        let body = match raw.kind {
            Kind::Type => Vec::new(),
            Kind::Relation => {
                let spec = relations
                    .get(&raw.subject.value)
                    .expect("relation spec exists");
                let mut members = vec![Member::Sentence(spec.shape.clone())];
                members.extend(spec.modes.iter().cloned().map(Member::Mode));
                members
            }
            Kind::Model => {
                let mut members = Vec::new();
                let mut variables = BTreeMap::new();
                for line in nonblank(raw.body.iter().copied()) {
                    if let Some(entity) = entity_line(line) {
                        members.push(Member::Entity(entity?));
                    } else {
                        let parsed = clause(
                            line,
                            &raw.subject.value,
                            &relations,
                            &entities,
                            &mut variables,
                        )?;
                        if !ground(&parsed) {
                            return Err(error(parsed.span, "model assertions must be closed"));
                        }
                        members.push(Member::Clause(parsed));
                    }
                }
                members
            }
            Kind::Law => {
                let layout = parse_law_layout(raw)?;
                let model =
                    declared_model_for_law(&raw.subject.value, &entities).ok_or_else(|| {
                        error(
                            raw.subject.span,
                            "Law name must be in a declared Model namespace",
                        )
                    })?;
                let mut variable_types = BTreeMap::new();
                let conclusion = clause(
                    layout.conclusion,
                    &model,
                    &relations,
                    &entities,
                    &mut variable_types,
                )?;
                let premises = layout
                    .premises
                    .iter()
                    .copied()
                    .map(|line| clause(line, &model, &relations, &entities, &mut variable_types))
                    .collect::<Result<Vec<_>, _>>()?;
                let premise_variables =
                    premises.iter().flat_map(variables).collect::<BTreeSet<_>>();
                if !variables(&conclusion).is_subset(&premise_variables) {
                    return Err(error(
                        conclusion.span,
                        "Law conclusion variables must be range-restricted by when",
                    ));
                }
                vec![Member::Clause(conclusion), Member::When(premises)]
            }
            Kind::Revision | Kind::Delta => {
                let layout = layouts
                    .get(&raw.subject.value)
                    .expect("change layout exists");
                let model = revision_model(&layout.from.value, &kinds, &layouts);
                let mut members = vec![Member::From(layout.from.value.clone())];
                if let Some(apply) = &layout.apply {
                    members.push(Member::Apply(apply.value.clone()));
                } else {
                    let mut admitted = BTreeSet::new();
                    let mut withdrawn = BTreeSet::new();
                    if let Some(lines) = &layout.admit {
                        let mut variables = BTreeMap::new();
                        let clauses = lines
                            .iter()
                            .copied()
                            .map(|line| clause(line, &model, &relations, &entities, &mut variables))
                            .collect::<Result<Vec<_>, _>>()?;
                        for parsed in &clauses {
                            if !ground(parsed) {
                                return Err(error(parsed.span, "changes must be closed"));
                            }
                            if !admitted.insert(clause_key(parsed)) {
                                return Err(error(parsed.span, "duplicate admission"));
                            }
                        }
                        members.push(Member::Admit(clauses));
                    }
                    if let Some(lines) = &layout.withdraw {
                        let mut variables = BTreeMap::new();
                        let clauses = lines
                            .iter()
                            .copied()
                            .map(|line| clause(line, &model, &relations, &entities, &mut variables))
                            .collect::<Result<Vec<_>, _>>()?;
                        for parsed in &clauses {
                            if !ground(parsed) {
                                return Err(error(parsed.span, "changes must be closed"));
                            }
                            let key = clause_key(parsed);
                            if !withdrawn.insert(key.clone()) {
                                return Err(error(parsed.span, "duplicate withdrawal"));
                            }
                            if admitted.contains(&key) {
                                return Err(error(
                                    parsed.span,
                                    "admit and withdraw cannot overlap",
                                ));
                            }
                        }
                        members.push(Member::Withdraw(clauses));
                    }
                }
                members
            }
        };
        declarations.push(AscriptionDecl {
            subject: raw.subject.clone(),
            kind: raw.kind,
            body,
            span: line_span(raw.header),
        });
    }

    let mut requests = Vec::new();
    for raw in raw_requests {
        match raw {
            RawRequest::Find {
                revision,
                sought,
                clause: line,
                header,
            } => {
                reference_kind(
                    &revision,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "request revision",
                )?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let pattern = clause(line, &model, &relations, &entities, &mut variables)?;
                if !variables.contains_key(&sought.value) {
                    return Err(error(sought.span, "find variable must occur in its clause"));
                }
                requests.push(RequestDecl::Find {
                    revision,
                    pattern,
                    sought,
                    span: line_span(header),
                });
            }
            RawRequest::Why {
                revision,
                all,
                clause: line,
                header,
            } => {
                reference_kind(
                    &revision,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "request revision",
                )?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let target = clause(line, &model, &relations, &entities, &mut variables)?;
                if !ground(&target) {
                    return Err(error(target.span, "why target must be closed"));
                }
                requests.push(RequestDecl::Why {
                    revision,
                    target,
                    all,
                    span: line_span(header),
                });
            }
            RawRequest::Intervention {
                verb,
                revision,
                selection,
                clause: line,
                using,
                header,
            } => {
                reference_kind(
                    &revision,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "request revision",
                )?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let target = clause(line, &model, &relations, &entities, &mut variables)?;
                if !ground(&target) {
                    return Err(error(target.span, "intervention target must be closed"));
                }
                let mut seen = BTreeSet::new();
                for relation in &using {
                    reference_kind(relation, &kinds, &[Kind::Relation], "using relation")?;
                    if !seen.insert(relation.value.clone()) {
                        return Err(error(relation.span, "using relations must be unique"));
                    }
                }
                let request = match verb {
                    "prevent" => RequestDecl::Prevent {
                        revision,
                        target,
                        selection,
                        using,
                        span: line_span(header),
                    },
                    "achieve" => RequestDecl::Achieve {
                        revision,
                        target,
                        selection,
                        using,
                        span: line_span(header),
                    },
                    _ => unreachable!("known intervention verb"),
                };
                requests.push(request);
            }
            RawRequest::Diff {
                base,
                successor,
                header,
            } => {
                reference_kind(&base, &kinds, &[Kind::Model, Kind::Revision], "diff base")?;
                reference_kind(
                    &successor,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "diff successor",
                )?;
                requests.push(RequestDecl::Diff {
                    base,
                    successor,
                    span: line_span(header),
                });
            }
        }
    }
    Ok(Program {
        declarations,
        requests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r#"Module: Type
Change: Type

impact/imports: Relation
    {consumer: Module} imports {dependency: Module}
    mode consumer -> dependency: many

impact/affects: Relation
    {change: Change} affects {consumer: Module}
    mode change -> consumer: many

impact: Model
    North: Module
    Store: Module
    compiler-change: Change
    North imports Store

impact/direct: Law
    ?consumer imports ?dependency
    when:
        ?consumer imports ?dependency

impact/adopt: Revision
    from: impact
    admit:
        Store imports North

find all ?consumer in impact:
    compiler-change affects ?consumer

why all in impact:
    North imports Store

prevent all minimal in impact:
    North imports Store
using:
    impact/imports

achieve one minimal in impact/adopt:
    Store imports North
using:
    impact/imports

diff impact -> impact/adopt
"#;

    #[test]
    fn parses_the_singular_surface_in_declaration_independent_order() {
        let source = format!(
            "find all ?consumer in impact:\n    compiler-change affects ?consumer\n\n{}\nModule: Type\nChange: Type\n",
            SOURCE
                .replace("Module: Type\nChange: Type\n\n", "")
                .replace(
                    "find all ?consumer in impact:\n    compiler-change affects ?consumer\n\n",
                    ""
                )
        );
        let program = parse(&source).expect("native source parses");
        assert_eq!(program.declarations.len(), 7);
        assert_eq!(program.requests.len(), 5);
        let relation = program
            .declarations
            .iter()
            .find(|declaration| declaration.subject.value.as_str() == "impact/imports")
            .expect("relation exists");
        assert_eq!(relation.subject.value.as_str(), "impact/imports");
        assert!(matches!(relation.body[0], Member::Sentence(_)));
        assert!(matches!(program.requests[0], RequestDecl::Find { .. }));
    }

    #[test]
    fn keeps_when_and_intervention_selection() {
        let program = parse(SOURCE).expect("native source parses");
        let law = program
            .declarations
            .iter()
            .find(|declaration| declaration.kind == Kind::Law)
            .expect("law exists");
        assert!(matches!(law.body[1], Member::When(_)));
        assert!(matches!(
            program.requests[2],
            RequestDecl::Prevent {
                selection: InterventionSelection::AllMinimal,
                ..
            }
        ));
        assert!(matches!(
            program.requests[3],
            RequestDecl::Achieve {
                selection: InterventionSelection::OneMinimal,
                ..
            }
        ));
    }

    #[test]
    fn parses_reusable_delta_and_revision_apply() {
        let source = SOURCE.replace(
            "impact/adopt: Revision\n    from: impact\n    admit:\n        Store imports North",
            "impact/remove: Delta\n    from: impact\n    withdraw:\n        North imports Store\n\nimpact/adopt: Revision\n    from: impact\n    apply: impact/remove",
        );
        let program = parse(&source).expect("Delta applies from the same base");
        assert!(
            program
                .declarations
                .iter()
                .any(|decl| decl.kind == Kind::Delta)
        );
    }

    #[test]
    fn rejects_retired_prefixes_and_sentence_members() {
        for (head, tail) in [
            ("rela", "tion"),
            ("mo", "del"),
            ("l", "aw"),
            ("in", "tent"),
            ("que", "ry"),
            ("cl", "aim"),
            ("requ", "ire"),
            ("fa", "ct"),
        ] {
            let prefix = [head, tail].concat();
            assert!(
                parse(&format!("{prefix} retired:")).is_err(),
                "{prefix} is retired"
            );
        }
        assert!(
            parse(&SOURCE.replace(
                "    {consumer: Module} imports {dependency: Module}",
                "    sentence: {consumer} imports {dependency}",
            ))
            .is_err()
        );
    }

    #[test]
    fn rejects_non_four_space_indentation_and_tabs() {
        assert!(parse(&SOURCE.replace("    North: Module", "  North: Module")).is_err());
        assert!(parse(&SOURCE.replace("    North: Module", "\tNorth: Module")).is_err());
    }

    #[test]
    fn rejects_invalid_shapes_and_modes() {
        assert!(
            parse(&SOURCE.replace(
                "{consumer: Module} imports {dependency: Module}",
                "{consumer: Module} {dependency: Module}"
            ))
            .is_err()
        );
        assert!(
            parse(&SOURCE.replace(
                "mode consumer -> dependency: many",
                "mode consumer -> consumer: many"
            ))
            .is_err()
        );
        assert!(
            parse(&SOURCE.replace(
                "{consumer: Module} imports {dependency: Module}",
                "{consumer: Module} imports {consumer: Module}"
            ))
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_or_wrongly_typed_entities_and_quoted_modules() {
        assert!(parse(&SOURCE.replace("North imports Store", "Missing imports Store")).is_err());
        assert!(
            parse(&SOURCE.replace("North imports Store", "compiler-change imports Store")).is_err()
        );
        assert!(parse(&SOURCE.replace("North imports Store", "\"North\" imports Store")).is_err());
    }

    #[test]
    fn rejects_inconsistent_law_variables() {
        let source = SOURCE.replace(
            "?consumer imports ?dependency\n    when:\n        ?consumer imports ?dependency",
            "?consumer imports ?dependency\n    when:\n        compiler-change affects ?consumer",
        );
        assert!(parse(&source).is_err());
    }

    #[test]
    fn rejects_change_duplicates_and_overlaps() {
        let duplicate = SOURCE.replace(
            "        Store imports North",
            "        Store imports North\n        Store imports North",
        );
        assert!(parse(&duplicate).is_err());
        let overlap = SOURCE.replace(
            "    admit:\n        Store imports North",
            "    admit:\n        Store imports North\n    withdraw:\n        Store imports North",
        );
        assert!(parse(&overlap).is_err());
    }

    #[test]
    fn accepts_apply_with_different_revision_aliases() {
        let source = SOURCE.replace(
            "impact/adopt: Revision\n    from: impact\n    admit:\n        Store imports North",
            "impact/alias-left: Revision\n    from: impact\n    admit:\n        Store imports North\n\nimpact/alias-right: Revision\n    from: impact\n    admit:\n        Store imports North\n\nimpact/remove: Delta\n    from: impact/alias-left\n    withdraw:\n        North imports Store\n\nimpact/adopt: Revision\n    from: impact/alias-right\n    apply: impact/remove",
        );
        assert!(parse(&source).is_ok());
    }

    #[test]
    fn rejects_cycles_and_bad_request_references() {
        let cycle = SOURCE.replace("impact/adopt: Revision\n    from: impact\n    admit:\n        Store imports North", "impact/first: Revision\n    from: impact/second\n    admit:\n        Store imports North\n\nimpact/second: Revision\n    from: impact/first\n    admit:\n        Store imports North");
        assert!(parse(&cycle).is_err());
        assert!(
            parse(&SOURCE.replace("using:\n    impact/imports", "using:\n    impact/missing"))
                .is_err()
        );
    }

    #[test]
    fn rejects_open_closed_requests_and_missing_find_variable() {
        assert!(
            parse(&SOURCE.replace(
                "    North imports Store\nusing:",
                "    ?north imports Store\nusing:"
            ))
            .is_err()
        );
        assert!(
            parse(&SOURCE.replace(
                "find all ?consumer in impact:",
                "find all ?missing in impact:"
            ))
            .is_err()
        );
    }

    #[test]
    fn parses_why_prefixed_qname_declarations() {
        let program = parse("Type: Type\nwhy: Type\nwhy-not: Type\n")
            .expect("why-prefixed names are declarations");
        assert!(
            program
                .declarations
                .iter()
                .any(|declaration| declaration.subject.value.as_str() == "why")
        );
        assert!(
            program
                .declarations
                .iter()
                .any(|declaration| declaration.subject.value.as_str() == "why-not")
        );
    }

    #[test]
    fn dispatches_only_exact_why_request_heads() {
        let source = SOURCE.replace(
            "why all in impact:\n    North imports Store",
            "why in impact:\n    North imports Store\n\nwhy all in impact:\n    North imports Store",
        );
        let program = parse(&source).expect("exact why request heads parse");
        assert!(matches!(
            program.requests[1],
            RequestDecl::Why { all: false, .. }
        ));
        assert!(matches!(
            program.requests[2],
            RequestDecl::Why { all: true, .. }
        ));
    }
}
