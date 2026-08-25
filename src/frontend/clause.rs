use super::model::MembershipCatalog;
use super::relation::RelationSpec;
use super::source::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct Token {
    pub(super) raw: String,
    pub(super) quoted: bool,
    pub(super) bracketed: bool,
    pub(super) span: Span,
}

pub(super) fn lex_clause(line: SourceLine<'_>) -> Result<Vec<Token>, ParseError> {
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
                bracketed: false,
                span: child_span(line, base + start, index - start),
            });
        } else if text.as_bytes()[index] == b'[' {
            index += 1;
            let value_start = index;
            while index < text.len() && text.as_bytes()[index] != b']' {
                if text.as_bytes()[index].is_ascii_whitespace() && text.as_bytes()[index] != b' ' {
                    return Err(error(
                        child_span(line, base + index, 1),
                        "bracketed referent words must be separated by ASCII spaces",
                    ));
                }
                if matches!(text.as_bytes()[index], b'[' | b'"') {
                    return Err(error(
                        child_span(line, base + index, 1),
                        "malformed bracketed referent",
                    ));
                }
                index += 1;
            }
            if index == text.len() {
                return Err(error(
                    child_span(line, base + start, text.len() - start),
                    "unterminated bracketed referent",
                ));
            }
            let value = &text[value_start..index];
            index += 1;
            if index < text.len() && text.as_bytes()[index] != b' ' {
                return Err(error(
                    child_span(line, base + index, 1),
                    "bracketed referent must be followed by a space",
                ));
            }
            tokens.push(Token {
                raw: value.to_owned(),
                quoted: false,
                bracketed: true,
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
                bracketed: false,
                span: child_span(line, base + start, index - start),
            });
        }
    }
    if tokens.is_empty() {
        return Err(error(line_span(line), "clause cannot be empty"));
    }
    Ok(tokens)
}

fn is_open_parenthesis(token: &Token) -> bool {
    !token.quoted && !token.bracketed && token.raw == "("
}

fn is_close_parenthesis(token: &Token) -> bool {
    !token.quoted && !token.bracketed && token.raw == ")"
}

fn is_open_delimiter(token: &Token) -> bool {
    !token.quoted && matches!(token.raw.as_str(), "(" | "[" | "{")
}

fn is_close_delimiter(token: &Token) -> bool {
    !token.quoted && matches!(token.raw.as_str(), ")" | "]" | "}")
}

fn matching_delimiters(open: &str, close: &str) -> bool {
    matches!((open, close), ("(", ")") | ("[", "]") | ("{", "}"))
}

fn push_structural_tokens(grouped: &mut Vec<Token>, token: &Token, raw: &str, column: usize) {
    let mut start = 0;
    for (offset, character) in raw.char_indices() {
        if character == ' ' || matches!(character, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':') {
            if start < offset {
                grouped.push(Token {
                    raw: raw[start..offset].to_owned(),
                    quoted: false,
                    bracketed: false,
                    span: Span {
                        line: token.span.line,
                        column: column + start,
                        width: offset - start,
                    },
                });
            }
            if character != ' ' {
                grouped.push(Token {
                    raw: character.to_string(),
                    quoted: false,
                    bracketed: false,
                    span: Span {
                        line: token.span.line,
                        column: column + offset,
                        width: character.len_utf8(),
                    },
                });
            }
            start = offset + character.len_utf8();
        }
    }
    if start < raw.len() {
        grouped.push(Token {
            raw: raw[start..].to_owned(),
            quoted: false,
            bracketed: false,
            span: Span {
                line: token.span.line,
                column: column + start,
                width: raw.len() - start,
            },
        });
    }
}

fn recursive_clause_tokens(line: SourceLine<'_>) -> Result<Vec<Token>, ParseError> {
    let mut grouped = Vec::new();
    for token in lex_clause(line)? {
        if token.quoted {
            grouped.push(token);
            continue;
        }
        if token.bracketed {
            grouped.push(Token {
                raw: "[".to_owned(),
                quoted: false,
                bracketed: false,
                span: Span {
                    line: token.span.line,
                    column: token.span.column,
                    width: 1,
                },
            });
            push_structural_tokens(&mut grouped, &token, &token.raw, token.span.column + 1);
            grouped.push(Token {
                raw: "]".to_owned(),
                quoted: false,
                bracketed: false,
                span: Span {
                    line: token.span.line,
                    column: token.span.column + token.span.width - 1,
                    width: 1,
                },
            });
        } else {
            push_structural_tokens(&mut grouped, &token, &token.raw, token.span.column);
        }
    }

    let mut opens: Vec<&Token> = Vec::new();
    for token in &grouped {
        if is_open_delimiter(token) {
            opens.push(token);
        } else if is_close_delimiter(token) {
            let Some(open) = opens.pop() else {
                let message = if token.raw == ")" {
                    "unmatched closing parenthesis"
                } else {
                    "unmatched closing term delimiter"
                };
                return Err(error(token.span, message));
            };
            if !matching_delimiters(&open.raw, &token.raw) {
                return Err(error(token.span, "mismatched term delimiters"));
            }
        }
    }
    if let Some(open) = opens.first() {
        let message = if open.raw == "(" {
            "unterminated parenthesized term"
        } else {
            "unterminated term delimiter"
        };
        return Err(error(open.span, message));
    }
    Ok(grouped)
}

fn balanced_tokens(tokens: &[Token]) -> bool {
    let mut opens = Vec::new();
    for token in tokens {
        if is_open_delimiter(token) {
            opens.push(token.raw.as_str());
        } else if is_close_delimiter(token) {
            let Some(open) = opens.pop() else {
                return false;
            };
            if !matching_delimiters(open, &token.raw) {
                return false;
            }
        }
    }
    opens.is_empty()
}

fn parenthesized_tokens(tokens: &[Token]) -> Option<&[Token]> {
    if !tokens.first().is_some_and(is_open_parenthesis)
        || !tokens.last().is_some_and(is_close_parenthesis)
    {
        return None;
    }
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if is_open_parenthesis(token) {
            depth += 1;
        } else if is_close_parenthesis(token) {
            depth = depth.checked_sub(1)?;
            if depth == 0 && index + 1 != tokens.len() {
                return None;
            }
        }
    }
    (depth == 0).then(|| &tokens[1..tokens.len() - 1])
}

fn parse_term(token: &Token) -> Result<SurfaceTerm, ParseError> {
    if token.quoted {
        return Ok(SurfaceTerm::String(Spanned {
            value: token.raw.clone(),
            span: token.span,
        }));
    }
    if let Some(name) = token.raw.strip_prefix('?') {
        if token.bracketed {
            return Err(error(token.span, "variables cannot be bracketed referents"));
        }
        if name.is_empty() {
            return Ok(SurfaceTerm::AnonymousHole(token.span));
        }
        return Ok(SurfaceTerm::Variable(variable_name(
            SourceLine {
                number: token.span.line,
                column: 1,
                text: "",
            },
            token.span.column - 1,
            name,
        )?));
    }
    if token.bracketed {
        return Err(error(
            token.span,
            format!(
                "bracketed concrete referents are retired; write '{}'",
                token.raw
            ),
        ));
    }
    if !is_qname(&token.raw) {
        return Err(error(
            token.span,
            format!("expected referent name or variable, found '{}'", token.raw),
        ));
    }
    Ok(SurfaceTerm::Referent(Spanned {
        value: Name(token.raw.clone()),
        span: token.span,
    }))
}

pub(super) fn focus_term(token: &Token) -> Result<SurfaceTerm, ParseError> {
    if !token.bracketed || !token.raw.contains('{') {
        return parse_term(token);
    }
    let open = token.raw.find('{').expect("checked focus template marker");
    let close = token.raw[open + 1..]
        .find('}')
        .map(|offset| open + 1 + offset)
        .ok_or_else(|| error(token.span, "unterminated correlated referent template"))?;
    if token.raw[..open].contains('}')
        || token.raw[close + 1..].contains(['{', '}'])
        || token.raw[open + 1..close].is_empty()
    {
        return Err(error(
            token.span,
            "correlated referent template permits exactly one variable",
        ));
    }
    let prefix = &token.raw[..open];
    let variable = &token.raw[open + 1..close];
    let suffix = &token.raw[close + 1..];
    let source = SourceLine {
        number: token.span.line,
        column: 1,
        text: "",
    };
    referent_name(source, token.span.column, &format!("{prefix}0{suffix}"))?;
    Ok(SurfaceTerm::Template(ReferentTemplate {
        prefix: Spanned {
            value: prefix.to_owned(),
            span: child_span(source, token.span.column, prefix.len()),
        },
        variable: variable_name(source, token.span.column + open + 1, variable)?,
        suffix: Spanned {
            value: suffix.to_owned(),
            span: child_span(source, token.span.column + close + 1, suffix.len()),
        },
        span: token.span,
    }))
}

fn term_domains(
    term: &SurfaceTerm,
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
) -> Result<Option<BTreeSet<DomainName>>, ParseError> {
    match term {
        SurfaceTerm::String(_) => Ok(Some(BTreeSet::from([DomainName("Text".to_owned())]))),
        SurfaceTerm::F32(_) => Ok(Some(BTreeSet::from([DomainName("F32".to_owned())]))),
        SurfaceTerm::Int(_) => Ok(Some(BTreeSet::from([DomainName("Int".to_owned())]))),
        SurfaceTerm::Bool(_) => Ok(Some(BTreeSet::from([DomainName("Bool".to_owned())]))),
        SurfaceTerm::Variable(_) | SurfaceTerm::AnonymousHole(_) => Ok(None),
        SurfaceTerm::Application(application) => {
            Ok(Some(BTreeSet::from([application.domain.clone()])))
        }
        SurfaceTerm::Tuple { .. }
        | SurfaceTerm::Product { .. }
        | SurfaceTerm::Sequence { .. }
        | SurfaceTerm::Intrinsic(_) => Ok(None),
        SurfaceTerm::Template(template) => Err(error(
            template.span,
            "correlated referent templates are only valid inside a focus block",
        )),
        SurfaceTerm::Referent(referent) | SurfaceTerm::Local(referent) => {
            if !referent.value.0.contains('/') {
                if let Some(domains) = current_memberships.explicit.get(&referent.value) {
                    return Ok(Some(domains.clone()));
                }
                let matched = current_memberships
                    .ranges
                    .iter()
                    .filter_map(|range| {
                        let name = referent.value.as_str();
                        let prefix = range.prefix.value.as_str();
                        let suffix = range.suffix.value.as_str();
                        let number = name
                            .strip_prefix(prefix)?
                            .strip_suffix(suffix)?
                            .parse::<u64>()
                            .ok()?;
                        (range.range.start <= number && number <= range.range.end)
                            .then(|| range.group.value.clone())
                    })
                    .collect::<BTreeSet<_>>();
                if matched.is_empty() {
                    return Err(error(
                        referent.span,
                        format!("unknown referent '{}'", referent.value.as_str()),
                    ));
                }
                return Ok(Some(matched));
            }
            for (model, catalog) in memberships {
                for (local, domains) in &catalog.explicit {
                    if referent.value.0 == format!("{}/{}", model.as_str(), local.as_str()) {
                        return Ok(Some(domains.clone()));
                    }
                }
            }
            Err(error(
                referent.span,
                format!("unknown qualified referent '{}'", referent.value.as_str()),
            ))
        }
    }
}

fn parse_role_term(tokens: &[Token]) -> Result<SurfaceTerm, ParseError> {
    if tokens.len() == 1 {
        return parse_term(&tokens[0]);
    }
    let first = tokens.first().expect("role capture is nonempty");
    let last = tokens.last().expect("role capture is nonempty");
    if tokens
        .iter()
        .any(|token| token.quoted || token.bracketed || token.raw.starts_with('?'))
    {
        return Err(error(
            Span {
                line: first.span.line,
                column: first.span.column,
                width: last.span.column + last.span.width - first.span.column,
            },
            "a multiword participant must be one semantic name",
        ));
    }
    let value = tokens
        .iter()
        .map(|token| token.raw.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let span = Span {
        line: first.span.line,
        column: first.span.column,
        width: last.span.column + last.span.width - first.span.column,
    };
    semantic_name(
        SourceLine {
            number: span.line,
            column: 1,
            text: "",
        },
        span.column - 1,
        &value,
    )?;
    Ok(SurfaceTerm::Referent(Spanned {
        value: Name(value),
        span,
    }))
}

fn literal_matches(tokens: &[Token], start: usize, literal: &str) -> Option<usize> {
    let words = literal.split(' ').collect::<Vec<_>>();
    let end = start.checked_add(words.len())?;
    let supplied = tokens.get(start..end)?;
    supplied
        .iter()
        .zip(words)
        .all(|(token, word)| !token.quoted && !token.bracketed && token.raw == word)
        .then_some(end)
}

fn collect_shape_matches(
    parts: &[ShapePartDecl],
    tokens: &[Token],
    part_index: usize,
    token_index: usize,
    roles: &mut BTreeMap<RoleName, SurfaceTerm>,
    matches: &mut Vec<BTreeMap<RoleName, SurfaceTerm>>,
) {
    let Some(part) = parts.get(part_index) else {
        if token_index == tokens.len() {
            matches.push(roles.clone());
        }
        return;
    };
    match part {
        ShapePartDecl::Literal(literal) => {
            if let Some(next) = literal_matches(tokens, token_index, &literal.value) {
                collect_shape_matches(parts, tokens, part_index + 1, next, roles, matches);
            }
        }
        ShapePartDecl::Role { id, .. } => {
            if part_index + 1 == parts.len() {
                if token_index < tokens.len()
                    && let Ok(term) = parse_role_term(&tokens[token_index..])
                {
                    roles.insert(id.value.clone(), term);
                    collect_shape_matches(
                        parts,
                        tokens,
                        part_index + 1,
                        tokens.len(),
                        roles,
                        matches,
                    );
                    roles.remove(&id.value);
                }
                return;
            }
            let ShapePartDecl::Literal(next_literal) = &parts[part_index + 1] else {
                unreachable!("sentence-shape roles have literal separators");
            };
            for end in token_index + 1..tokens.len() {
                if literal_matches(tokens, end, &next_literal.value).is_none() {
                    continue;
                }
                let Ok(term) = parse_role_term(&tokens[token_index..end]) else {
                    continue;
                };
                roles.insert(id.value.clone(), term);
                collect_shape_matches(parts, tokens, part_index + 1, end, roles, matches);
                roles.remove(&id.value);
            }
        }
    }
}

fn shape_matches(
    shape: &SentenceShapeDecl,
    tokens: &[Token],
) -> Vec<BTreeMap<RoleName, SurfaceTerm>> {
    let mut matches = Vec::new();
    collect_shape_matches(
        &shape.parts,
        tokens,
        0,
        0,
        &mut BTreeMap::new(),
        &mut matches,
    );
    matches
}

fn token_span(tokens: &[Token]) -> Span {
    let first = tokens.first().expect("term tokens are nonempty");
    let last = tokens.last().expect("term tokens are nonempty");
    Span {
        line: first.span.line,
        column: first.span.column,
        width: last.span.column + last.span.width - first.span.column,
    }
}

fn domain_signature(name: &str, members: &[DomainName]) -> DomainName {
    let mut signature = String::from("@clause/");
    signature.push_str(name);
    signature.push('(');
    signature.push_str(
        &members
            .iter()
            .map(DomainName::as_str)
            .collect::<Vec<_>>()
            .join(","),
    );
    signature.push(')');
    DomainName(signature)
}

fn delimited_body<'a>(tokens: &'a [Token], open: &str, close: &str) -> Option<&'a [Token]> {
    (tokens.first()?.raw == open && tokens.last()?.raw == close).then(|| {
        let mut depth = 0usize;
        for (index, token) in tokens.iter().enumerate() {
            if is_open_delimiter(token) {
                depth += 1;
            } else if is_close_delimiter(token) {
                depth -= 1;
                if depth == 0 && index + 1 != tokens.len() {
                    return &tokens[0..0];
                }
            }
        }
        &tokens[1..tokens.len() - 1]
    })
}

fn split_top_level<'a>(tokens: &'a [Token], separator: &str) -> Vec<&'a [Token]> {
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut parts = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if is_open_delimiter(token) {
            depth += 1;
        } else if is_close_delimiter(token) {
            depth -= 1;
        } else if depth == 0 && token.raw == separator {
            parts.push(&tokens[start..index]);
            start = index + 1;
        }
    }
    parts.push(&tokens[start..]);
    parts
}

fn numeric_spelling(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "nan" | "inf" | "+inf" | "-inf" | "infinity" | "+infinity" | "-infinity"
    ) || value
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-'))
        && value.bytes().all(|byte| {
            byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.' | b'e' | b'E' | b'_')
        })
}

fn structural_term(
    tokens: &[Token],
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
) -> Result<Option<(SurfaceTerm, DomainName)>, ParseError> {
    if tokens.is_empty() {
        return Ok(None);
    }
    let span = token_span(tokens);
    if tokens.len() == 1 && !tokens[0].quoted {
        let token = &tokens[0];
        if token.raw == "true" || token.raw == "false" {
            return Ok(Some((
                SurfaceTerm::Bool(Spanned {
                    value: token.raw == "true",
                    span: token.span,
                }),
                DomainName("Bool".to_owned()),
            )));
        }
        if numeric_spelling(&token.raw) {
            if token.raw.contains(['.', 'e', 'E'])
                || matches!(
                    token.raw.to_ascii_lowercase().as_str(),
                    "nan" | "inf" | "+inf" | "-inf" | "infinity" | "+infinity" | "-infinity"
                )
            {
                let value = token.raw.parse::<f32>().map_err(|_| {
                    error(token.span, "malformed or overflowing decimal F32 literal")
                })?;
                if !value.is_finite() {
                    return Err(error(token.span, "F32 literal must be finite"));
                }
                let bits = if value == 0.0 {
                    0.0_f32.to_bits()
                } else {
                    value.to_bits()
                };
                return Ok(Some((
                    SurfaceTerm::F32(Spanned {
                        value: bits,
                        span: token.span,
                    }),
                    DomainName("F32".to_owned()),
                )));
            }
            let value = token
                .raw
                .parse::<i64>()
                .map_err(|_| error(token.span, "malformed or overflowing Int literal"))?;
            return Ok(Some((
                SurfaceTerm::Int(Spanned {
                    value,
                    span: token.span,
                }),
                DomainName("Int".to_owned()),
            )));
        }
    }

    if let Some(body) = delimited_body(tokens, "(", ")")
        && !body.is_empty()
    {
        let parts = split_top_level(body, ",");
        if parts.len() > 1 {
            if parts.iter().any(|part| part.is_empty()) {
                return Err(error(span, "invalid tuple delimiter"));
            }
            let mut values = Vec::new();
            let mut domains = Vec::new();
            for part in parts {
                let Some((value, domain)) =
                    structural_term(part, current_memberships, memberships)?
                else {
                    return Err(error(token_span(part), "invalid tuple member term"));
                };
                values.push(value);
                domains.push(domain);
            }
            return Ok(Some((
                SurfaceTerm::Tuple { values, span },
                domain_signature("tuple", &domains),
            )));
        }
    }

    if let Some(body) = delimited_body(tokens, "[", "]") {
        if body.is_empty() {
            return Err(error(span, "empty sequence has no element shape"));
        }
        let parts = split_top_level(body, ",");
        if parts.iter().any(|part| part.is_empty()) {
            return Err(error(span, "invalid sequence delimiter"));
        }
        let mut values = Vec::new();
        let mut element_domain = None;
        for part in parts {
            let Some((value, domain)) = structural_term(part, current_memberships, memberships)?
            else {
                return Err(error(token_span(part), "invalid sequence member term"));
            };
            if element_domain
                .as_ref()
                .is_some_and(|actual| actual != &domain)
            {
                return Err(error(
                    token_span(part),
                    "heterogeneous sequence member shape",
                ));
            }
            element_domain.get_or_insert(domain);
            values.push(value);
        }
        let element_domain = element_domain.expect("nonempty sequence");
        return Ok(Some((
            SurfaceTerm::Sequence { values, span },
            domain_signature("sequence", &[element_domain]),
        )));
    }

    if let Some(open) = tokens
        .iter()
        .position(|token| token.raw == "{" && !token.quoted)
        && open > 0
        && tokens.last().is_some_and(|token| token.raw == "}")
    {
        let shape_tokens = &tokens[..open];
        let body = &tokens[open + 1..tokens.len() - 1];
        let SurfaceTerm::Referent(shape) = parse_role_term(shape_tokens)? else {
            return Err(error(
                token_span(shape_tokens),
                "invalid product shape name",
            ));
        };
        if body.is_empty() {
            return Err(error(span, "labelled product requires at least one field"));
        }
        let mut fields = BTreeMap::new();
        for field in split_top_level(body, ",") {
            if field.is_empty() {
                return Err(error(span, "invalid product field delimiter"));
            }
            let parts = split_top_level(field, ":");
            let [label_tokens, value_tokens] = parts.as_slice() else {
                return Err(error(token_span(field), "product field requires one ':'"));
            };
            let SurfaceTerm::Referent(label) = parse_role_term(label_tokens)? else {
                return Err(error(
                    token_span(label_tokens),
                    "invalid product field label",
                ));
            };
            let Some((value, _)) = structural_term(value_tokens, current_memberships, memberships)?
            else {
                return Err(error(
                    token_span(value_tokens),
                    "invalid product field term",
                ));
            };
            if fields.insert(label.value.clone(), value).is_some() {
                return Err(error(label.span, "duplicate product field"));
            }
        }
        let domain = DomainName(shape.value.0.clone());
        return Ok(Some((
            SurfaceTerm::Product {
                shape,
                fields,
                span,
            },
            domain,
        )));
    }

    if let Ok(term) = parse_role_term(tokens)
        && let Ok(Some(domains)) = term_domains(&term, current_memberships, memberships)
        && domains.len() == 1
    {
        return Ok(Some((
            term,
            domains.into_iter().next().expect("one domain"),
        )));
    }
    Ok(None)
}

fn term_is_ground(term: &SurfaceTerm) -> bool {
    match term {
        SurfaceTerm::Referent(_)
        | SurfaceTerm::Local(_)
        | SurfaceTerm::String(_)
        | SurfaceTerm::F32(_)
        | SurfaceTerm::Int(_)
        | SurfaceTerm::Bool(_)
        | SurfaceTerm::Intrinsic(_) => true,
        SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
            values.iter().all(term_is_ground)
        }
        SurfaceTerm::Product { fields, .. } => fields.values().all(term_is_ground),
        SurfaceTerm::Application(application) => application.roles.values().all(term_is_ground),
        SurfaceTerm::Template(_) | SurfaceTerm::Variable(_) | SurfaceTerm::AnonymousHole(_) => {
            false
        }
    }
}

fn term_key(term: &SurfaceTerm) -> String {
    match term {
        SurfaceTerm::Referent(value) => format!("R:{}", value.value.0),
        SurfaceTerm::Local(value) => format!("L:{}", value.value.0),
        SurfaceTerm::Template(value) => format!(
            "T:{}{{{}}}{}",
            value.prefix.value, value.variable.value.0, value.suffix.value
        ),
        SurfaceTerm::Variable(value) => format!("V:{}", value.value.0),
        SurfaceTerm::AnonymousHole(span) => format!("H:{}:{}", span.line, span.column),
        SurfaceTerm::String(value) => format!("S:{:?}", value.value),
        SurfaceTerm::F32(value) => format!("F:{:08x}", value.value),
        SurfaceTerm::Int(value) => format!("I:{}", value.value),
        SurfaceTerm::Bool(value) => format!("B:{}", value.value),
        SurfaceTerm::Tuple { values, .. } => format!(
            "T({})",
            values.iter().map(term_key).collect::<Vec<_>>().join(",")
        ),
        SurfaceTerm::Product { shape, fields, .. } => {
            let mut key = format!("P:{}", shape.value.as_str());
            for (field, value) in fields {
                key.push('|');
                key.push_str(field.as_str());
                key.push('=');
                key.push_str(&term_key(value));
            }
            key
        }
        SurfaceTerm::Sequence { values, .. } => format!(
            "Q[{}]",
            values.iter().map(term_key).collect::<Vec<_>>().join(",")
        ),
        SurfaceTerm::Intrinsic(value) => format!("N:{}", value.value.as_str()),
        SurfaceTerm::Application(value) => {
            let mut key = format!(
                "A:{}->{}:{}",
                value.relation.value.0, value.result.value.0, value.domain.0
            );
            for (role, term) in &value.roles {
                key.push('|');
                key.push_str(role.as_str());
                key.push('=');
                key.push_str(&term_key(term));
            }
            key
        }
    }
}

fn push_unique_term(terms: &mut Vec<SurfaceTerm>, term: SurfaceTerm) {
    let key = term_key(&term);
    if !terms.iter().any(|candidate| term_key(candidate) == key) {
        terms.push(term);
    }
}

fn single_result_projection(spec: &RelationSpec) -> Option<(&Spanned<RoleName>, &[ShapePartDecl])> {
    let ShapePartDecl::Role { id: result, .. } = spec.shape.parts.first()? else {
        return None;
    };
    let mut modes = spec.modes.iter().filter(|mode| {
        mode.cardinality == Cardinality::One
            && mode.sought.len() == 1
            && mode.sought[0].value == result.value
            && mode.known.len() + mode.sought.len() == spec.roles.len()
    });
    modes.next()?;
    if modes.next().is_some() {
        return None;
    }
    let mut start = 1;
    if matches!(
        spec.shape.parts.get(start),
        Some(ShapePartDecl::Literal(literal)) if literal.value == "is"
    ) {
        start += 1;
    }
    let projected = spec.shape.parts.get(start..)?;
    (!projected.is_empty()).then_some((result, projected))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TermChartKey {
    line: usize,
    column: usize,
    width: usize,
    expected: DomainName,
    minimum_precedence: u8,
    binding: bool,
}

type TermChart = BTreeMap<TermChartKey, Option<Vec<SurfaceTerm>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Association {
    Left,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OperatorPrior {
    precedence: u8,
    association: Association,
}

fn operator_prior(operator: &str) -> Option<OperatorPrior> {
    match operator {
        "*" | "/" => Some(OperatorPrior {
            precedence: 30,
            association: Association::Left,
        }),
        "+" | "-" => Some(OperatorPrior {
            precedence: 20,
            association: Association::Left,
        }),
        "<" | "<=" | ">" | ">=" | "=" | "!=" => Some(OperatorPrior {
            precedence: 10,
            association: Association::None,
        }),
        _ => None,
    }
}

fn binary_operator_projection(
    parts: &[ShapePartDecl],
) -> Option<(&Spanned<RoleName>, &str, &Spanned<RoleName>, OperatorPrior)> {
    let [
        ShapePartDecl::Role { id: left, .. },
        ShapePartDecl::Literal(operator),
        ShapePartDecl::Role { id: right, .. },
    ] = parts
    else {
        return None;
    };
    Some((
        left,
        operator.value.as_str(),
        right,
        operator_prior(&operator.value)?,
    ))
}

#[allow(clippy::too_many_arguments)]
fn collect_recursive_matches(
    parts: &[ShapePartDecl],
    tokens: &[Token],
    part_index: usize,
    token_index: usize,
    role_domains: &BTreeMap<RoleName, DomainName>,
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    chart: &mut TermChart,
    roles: &mut BTreeMap<RoleName, SurfaceTerm>,
    matches: &mut Vec<BTreeMap<RoleName, SurfaceTerm>>,
) {
    let Some(part) = parts.get(part_index) else {
        if token_index == tokens.len() && !matches.contains(roles) {
            matches.push(roles.clone());
        }
        return;
    };
    match part {
        ShapePartDecl::Literal(literal) => {
            if let Some(next) = literal_matches(tokens, token_index, &literal.value) {
                collect_recursive_matches(
                    parts,
                    tokens,
                    part_index + 1,
                    next,
                    role_domains,
                    current_memberships,
                    memberships,
                    relations,
                    chart,
                    roles,
                    matches,
                );
            }
        }
        ShapePartDecl::Role { id, .. } => {
            let expected = role_domains
                .get(&id.value)
                .expect("relation role has a declared domain");
            let mut capture = |end: usize| {
                for term in term_candidates(
                    &tokens[token_index..end],
                    expected,
                    current_memberships,
                    memberships,
                    relations,
                    0,
                    chart,
                ) {
                    roles.insert(id.value.clone(), term);
                    collect_recursive_matches(
                        parts,
                        tokens,
                        part_index + 1,
                        end,
                        role_domains,
                        current_memberships,
                        memberships,
                        relations,
                        chart,
                        roles,
                        matches,
                    );
                    roles.remove(&id.value);
                }
            };
            if part_index + 1 == parts.len() {
                if token_index < tokens.len() {
                    capture(tokens.len());
                }
                return;
            }
            let ShapePartDecl::Literal(next_literal) = &parts[part_index + 1] else {
                unreachable!("sentence-shape roles have literal separators");
            };
            for end in token_index + 1..tokens.len() {
                if balanced_tokens(&tokens[token_index..end])
                    && literal_matches(tokens, end, &next_literal.value).is_some()
                {
                    capture(end);
                }
            }
        }
    }
}

fn recursive_shape_matches(
    spec: &RelationSpec,
    tokens: &[Token],
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    chart: &mut TermChart,
) -> Vec<BTreeMap<RoleName, SurfaceTerm>> {
    let mut matches = Vec::new();
    collect_recursive_matches(
        &spec.shape.parts,
        tokens,
        0,
        0,
        &spec.roles,
        current_memberships,
        memberships,
        relations,
        chart,
        &mut BTreeMap::new(),
        &mut matches,
    );
    matches
}

#[allow(clippy::too_many_arguments)]
fn binary_operator_matches(
    tokens: &[Token],
    left: &Spanned<RoleName>,
    operator: &str,
    right: &Spanned<RoleName>,
    prior: OperatorPrior,
    role_domains: &BTreeMap<RoleName, DomainName>,
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    minimum_precedence: u8,
    chart: &mut TermChart,
) -> Vec<BTreeMap<RoleName, SurfaceTerm>> {
    if prior.precedence < minimum_precedence {
        return Vec::new();
    }
    let left_minimum = match prior.association {
        Association::Left => prior.precedence,
        Association::None => prior.precedence + 1,
    };
    let right_minimum = prior.precedence + 1;
    let mut matches = Vec::new();
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if is_open_parenthesis(token) {
            depth += 1;
            continue;
        }
        if is_close_parenthesis(token) {
            depth -= 1;
            continue;
        }
        if depth != 0
            || token.quoted
            || token.bracketed
            || token.raw != operator
            || index == 0
            || index + 1 == tokens.len()
        {
            continue;
        }
        let left_terms = term_candidates(
            &tokens[..index],
            &role_domains[&left.value],
            current_memberships,
            memberships,
            relations,
            left_minimum,
            chart,
        );
        let right_terms = term_candidates(
            &tokens[index + 1..],
            &role_domains[&right.value],
            current_memberships,
            memberships,
            relations,
            right_minimum,
            chart,
        );
        for left_term in &left_terms {
            for right_term in &right_terms {
                let roles = BTreeMap::from([
                    (left.value.clone(), left_term.clone()),
                    (right.value.clone(), right_term.clone()),
                ]);
                if !matches.contains(&roles) {
                    matches.push(roles);
                }
            }
        }
    }
    matches
}

fn term_candidates(
    tokens: &[Token],
    expected: &DomainName,
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    minimum_precedence: u8,
    chart: &mut TermChart,
) -> Vec<SurfaceTerm> {
    if tokens.is_empty() || !balanced_tokens(tokens) {
        return Vec::new();
    }
    let span = token_span(tokens);
    let key = TermChartKey {
        line: span.line,
        column: span.column,
        width: span.width,
        expected: expected.clone(),
        minimum_precedence,
        binding: false,
    };
    if let Some(entry) = chart.get(&key) {
        return entry.clone().unwrap_or_default();
    }
    chart.insert(key.clone(), None);

    if let Some(inner) = parenthesized_tokens(tokens) {
        let candidates = term_candidates(
            inner,
            expected,
            current_memberships,
            memberships,
            relations,
            0,
            chart,
        );
        chart.insert(key, Some(candidates.clone()));
        return candidates;
    }

    let mut candidates = Vec::new();
    if let Ok(term) = parse_role_term(tokens)
        && match term_domains(&term, current_memberships, memberships) {
            Ok(Some(actual)) => actual.contains(expected),
            Ok(None) => true,
            Err(_) => false,
        }
    {
        push_unique_term(&mut candidates, term);
    }
    for (relation, spec) in relations {
        let Some((result, projected)) = single_result_projection(spec) else {
            continue;
        };
        if spec.roles.get(&result.value) != Some(expected) {
            continue;
        }
        let matches =
            if let Some((left, operator, right, prior)) = binary_operator_projection(projected) {
                binary_operator_matches(
                    tokens,
                    left,
                    operator,
                    right,
                    prior,
                    &spec.roles,
                    current_memberships,
                    memberships,
                    relations,
                    minimum_precedence,
                    chart,
                )
            } else {
                let mut matches = Vec::new();
                collect_recursive_matches(
                    projected,
                    tokens,
                    0,
                    0,
                    &spec.roles,
                    current_memberships,
                    memberships,
                    relations,
                    chart,
                    &mut BTreeMap::new(),
                    &mut matches,
                );
                matches
            };
        for roles in matches {
            push_unique_term(
                &mut candidates,
                SurfaceTerm::Application(Box::new(SurfaceApplication {
                    relation: Spanned {
                        value: relation.clone(),
                        span,
                    },
                    roles,
                    result: result.clone(),
                    domain: expected.clone(),
                    span,
                })),
            );
        }
    }
    chart.insert(key, Some(candidates.clone()));
    candidates
}

const INTRINSIC_PREFIX: &str = "@clause/intrinsic/";

fn intrinsic_application(
    name: &str,
    roles: BTreeMap<RoleName, SurfaceTerm>,
    domain: DomainName,
    span: Span,
) -> SurfaceTerm {
    SurfaceTerm::Application(Box::new(SurfaceApplication {
        relation: Spanned {
            value: Name(format!("{INTRINSIC_PREFIX}{name}")),
            span,
        },
        roles,
        result: Spanned {
            value: RoleName("result".to_owned()),
            span,
        },
        domain,
        span,
    }))
}

fn top_level_token(tokens: &[Token], raw: &str) -> Vec<usize> {
    let mut depth = 0usize;
    let mut indices = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if is_open_delimiter(token) {
            depth += 1;
        } else if is_close_delimiter(token) {
            depth -= 1;
        } else if depth == 0 && !token.quoted && token.raw == raw {
            indices.push(index);
        }
    }
    indices
}

fn candidate_domains(
    tokens: &[Token],
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
) -> Result<BTreeSet<DomainName>, ParseError> {
    let mut domains = BTreeSet::from([
        DomainName("F32".to_owned()),
        DomainName("Int".to_owned()),
        DomainName("Bool".to_owned()),
        domain_signature("sequence", &[DomainName("F32".to_owned())]),
    ]);
    domains.extend(current_memberships.explicit.values().flatten().cloned());
    domains.extend(
        relations
            .values()
            .flat_map(|relation| relation.roles.values().cloned()),
    );
    if let Some((_, domain)) = structural_term(tokens, current_memberships, memberships)? {
        domains.insert(domain);
    }
    Ok(domains)
}

#[allow(clippy::too_many_arguments)]
fn any_binding_candidates(
    tokens: &[Token],
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    minimum_precedence: u8,
    chart: &mut TermChart,
) -> Vec<(DomainName, SurfaceTerm)> {
    let Ok(domains) = candidate_domains(tokens, current_memberships, memberships, relations) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for domain in domains {
        for term in binding_term_candidates(
            tokens,
            &domain,
            current_memberships,
            memberships,
            relations,
            minimum_precedence,
            chart,
        ) {
            if !candidates.iter().any(|(other, candidate)| {
                other == &domain && term_key(candidate) == term_key(&term)
            }) {
                candidates.push((domain.clone(), term));
            }
        }
    }
    candidates
}

fn arithmetic_result(operator: &str, left: &DomainName, right: &DomainName) -> Option<DomainName> {
    let scalar = |domain: &DomainName| matches!(domain.as_str(), "F32" | "Int");
    let tuple = |domain: &DomainName| domain.as_str().starts_with("@clause/tuple(");
    match operator {
        "+" | "-" if left == right && (scalar(left) || tuple(left)) => Some(left.clone()),
        "*" if left == right && scalar(left) => Some(left.clone()),
        "*" | "/" if tuple(left) && scalar(right) => Some(left.clone()),
        "/" if left == right && scalar(left) => Some(left.clone()),
        "<" | "<=" | ">" | ">=" if left == right && scalar(left) => {
            Some(DomainName("Bool".to_owned()))
        }
        "=" | "!=" if left == right => Some(DomainName("Bool".to_owned())),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn intrinsic_candidates(
    tokens: &[Token],
    expected: &DomainName,
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    minimum_precedence: u8,
    chart: &mut TermChart,
) -> Vec<SurfaceTerm> {
    let span = token_span(tokens);
    let mut candidates = Vec::new();

    if minimum_precedence == 0
        && tokens.first().is_some_and(|token| token.raw == "if")
        && let ([then_index], [else_index]) = (
            top_level_token(tokens, "then").as_slice(),
            top_level_token(tokens, "else").as_slice(),
        )
        && 1 < *then_index
        && *then_index + 1 < *else_index
        && *else_index + 1 < tokens.len()
    {
        let conditions = binding_term_candidates(
            &tokens[1..*then_index],
            &DomainName("Bool".to_owned()),
            current_memberships,
            memberships,
            relations,
            0,
            chart,
        );
        let then_terms = binding_term_candidates(
            &tokens[*then_index + 1..*else_index],
            expected,
            current_memberships,
            memberships,
            relations,
            0,
            chart,
        );
        let else_terms = binding_term_candidates(
            &tokens[*else_index + 1..],
            expected,
            current_memberships,
            memberships,
            relations,
            0,
            chart,
        );
        for condition in &conditions {
            for then_term in &then_terms {
                for else_term in &else_terms {
                    push_unique_term(
                        &mut candidates,
                        intrinsic_application(
                            "conditional",
                            BTreeMap::from([
                                (RoleName("condition".to_owned()), condition.clone()),
                                (RoleName("then".to_owned()), then_term.clone()),
                                (RoleName("else".to_owned()), else_term.clone()),
                            ]),
                            expected.clone(),
                            span,
                        ),
                    );
                }
            }
        }
    }

    if minimum_precedence <= 40
        && tokens.first().is_some_and(|token| token.raw == "length")
        && tokens.len() > 1
        && expected.as_str() == "F32"
    {
        for (domain, input) in any_binding_candidates(
            &tokens[1..],
            current_memberships,
            memberships,
            relations,
            40,
            chart,
        ) {
            if domain.as_str().starts_with("@clause/tuple(") {
                push_unique_term(
                    &mut candidates,
                    intrinsic_application(
                        "length",
                        BTreeMap::from([(RoleName("input".to_owned()), input)]),
                        expected.clone(),
                        span,
                    ),
                );
            }
        }
    }

    if minimum_precedence == 0
        && tokens.first().is_some_and(|token| token.raw == "map")
        && tokens.get(1).is_some_and(|token| token.raw == "length")
        && tokens.get(2).is_some_and(|token| token.raw == "over")
        && tokens.len() > 3
        && expected == &domain_signature("sequence", &[DomainName("F32".to_owned())])
    {
        for (domain, sequence) in any_binding_candidates(
            &tokens[3..],
            current_memberships,
            memberships,
            relations,
            0,
            chart,
        ) {
            if domain
                .as_str()
                .starts_with("@clause/sequence(@clause/tuple(")
            {
                push_unique_term(
                    &mut candidates,
                    intrinsic_application(
                        "map",
                        BTreeMap::from([
                            (
                                RoleName("mapper".to_owned()),
                                SurfaceTerm::Intrinsic(Spanned {
                                    value: Name(format!("{INTRINSIC_PREFIX}length")),
                                    span: tokens[1].span,
                                }),
                            ),
                            (RoleName("sequence".to_owned()), sequence),
                        ]),
                        expected.clone(),
                        span,
                    ),
                );
            }
        }
    }

    for operator in ["<", "<=", ">", ">=", "=", "!=", "+", "-", "*", "/"] {
        let Some(prior) = operator_prior(operator) else {
            continue;
        };
        if prior.precedence < minimum_precedence {
            continue;
        }
        for index in top_level_token(tokens, operator) {
            if index == 0 || index + 1 == tokens.len() {
                continue;
            }
            let left_minimum = match prior.association {
                Association::Left => prior.precedence,
                Association::None => prior.precedence + 1,
            };
            let left_terms = any_binding_candidates(
                &tokens[..index],
                current_memberships,
                memberships,
                relations,
                left_minimum,
                chart,
            );
            let right_terms = any_binding_candidates(
                &tokens[index + 1..],
                current_memberships,
                memberships,
                relations,
                prior.precedence + 1,
                chart,
            );
            for (left_domain, left) in &left_terms {
                for (right_domain, right) in &right_terms {
                    if arithmetic_result(operator, left_domain, right_domain).as_ref()
                        != Some(expected)
                    {
                        continue;
                    }
                    let name = match operator {
                        "+" => "add",
                        "-" => "subtract",
                        "*" => "multiply",
                        "/" => "divide",
                        "<" => "less-than",
                        "<=" => "less-or-equal",
                        ">" => "greater-than",
                        ">=" => "greater-or-equal",
                        "=" => "equal",
                        "!=" => "not-equal",
                        _ => unreachable!(),
                    };
                    push_unique_term(
                        &mut candidates,
                        intrinsic_application(
                            name,
                            BTreeMap::from([
                                (RoleName("left".to_owned()), left.clone()),
                                (RoleName("right".to_owned()), right.clone()),
                            ]),
                            expected.clone(),
                            span,
                        ),
                    );
                }
            }
        }
    }
    candidates
}

#[allow(clippy::too_many_arguments)]
fn binding_term_candidates(
    tokens: &[Token],
    expected: &DomainName,
    current_memberships: &MembershipCatalog,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    relations: &BTreeMap<Name, RelationSpec>,
    minimum_precedence: u8,
    chart: &mut TermChart,
) -> Vec<SurfaceTerm> {
    if tokens.is_empty() || !balanced_tokens(tokens) {
        return Vec::new();
    }
    let span = token_span(tokens);
    let key = TermChartKey {
        line: span.line,
        column: span.column,
        width: span.width,
        expected: expected.clone(),
        minimum_precedence,
        binding: true,
    };
    if let Some(entry) = chart.get(&key) {
        return entry.clone().unwrap_or_default();
    }
    chart.insert(key.clone(), None);
    let mut candidates = Vec::new();
    if let Ok(Some((term, domain))) = structural_term(tokens, current_memberships, memberships)
        && &domain == expected
    {
        push_unique_term(&mut candidates, term);
    }
    if let Some(inner) = parenthesized_tokens(tokens)
        && split_top_level(inner, ",").len() == 1
    {
        for term in binding_term_candidates(
            inner,
            expected,
            current_memberships,
            memberships,
            relations,
            0,
            chart,
        ) {
            push_unique_term(&mut candidates, term);
        }
    }
    for term in term_candidates(
        tokens,
        expected,
        current_memberships,
        memberships,
        relations,
        minimum_precedence,
        chart,
    ) {
        push_unique_term(&mut candidates, term);
    }
    for term in intrinsic_candidates(
        tokens,
        expected,
        current_memberships,
        memberships,
        relations,
        minimum_precedence,
        chart,
    ) {
        push_unique_term(&mut candidates, term);
    }
    chart.insert(key, Some(candidates.clone()));
    candidates
}

pub(super) fn closed_term_with_catalog(
    line: SourceLine<'_>,
    current_memberships: &MembershipCatalog,
    relations: &BTreeMap<Name, RelationSpec>,
    memberships: &BTreeMap<Name, MembershipCatalog>,
) -> Result<(SurfaceTerm, DomainName), ParseError> {
    let tokens = recursive_clause_tokens(line)?;
    closed_term_candidates(
        tokens,
        line_span(line),
        current_memberships,
        relations,
        memberships,
    )
}

pub(super) fn closed_term_text_with_catalog(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
    current_memberships: &MembershipCatalog,
    relations: &BTreeMap<Name, RelationSpec>,
    memberships: &BTreeMap<Name, MembershipCatalog>,
) -> Result<(SurfaceTerm, DomainName), ParseError> {
    let synthetic = SourceLine {
        number: line.number,
        column: 1,
        text,
    };
    let mut tokens = recursive_clause_tokens(synthetic)?;
    for token in &mut tokens {
        token.span.column += offset;
    }
    closed_term_candidates(
        tokens,
        child_span(line, offset, text.len()),
        current_memberships,
        relations,
        memberships,
    )
}

fn closed_term_candidates(
    tokens: Vec<Token>,
    span: Span,
    current_memberships: &MembershipCatalog,
    relations: &BTreeMap<Name, RelationSpec>,
    memberships: &BTreeMap<Name, MembershipCatalog>,
) -> Result<(SurfaceTerm, DomainName), ParseError> {
    let mut domains = current_memberships
        .explicit
        .values()
        .flatten()
        .chain(
            relations
                .values()
                .flat_map(|relation| relation.roles.values()),
        )
        .cloned()
        .collect::<BTreeSet<_>>();
    domains.extend(candidate_domains(
        &tokens,
        current_memberships,
        memberships,
        relations,
    )?);
    // Preserve precise checked-literal and delimiter diagnostics even when no
    // candidate survives later domain-directed ambiguity filtering.
    let _ = structural_term(&tokens, current_memberships, memberships)?;
    let mut chart = TermChart::new();
    let mut candidates = Vec::new();
    for domain in domains {
        for term in binding_term_candidates(
            &tokens,
            &domain,
            current_memberships,
            memberships,
            relations,
            0,
            &mut chart,
        ) {
            if term_is_ground(&term)
                && !candidates.iter().any(|(other_domain, other_term)| {
                    other_domain == &domain && term_key(other_term) == term_key(&term)
                })
            {
                candidates.push((domain.clone(), term));
            }
        }
    }
    match candidates.as_slice() {
        [(domain, term)] => Ok((term.clone(), domain.clone())),
        [] => {
            reject_bracketed_clause_terms(&tokens)?;
            Err(error(
                span,
                "no declared recursive term accepts this definition expression",
            ))
        }
        _ => Err(error(
            span,
            "ambiguous definition expression; expected one recursive term",
        )),
    }
}

pub(super) fn bind_local_references(term: &mut SurfaceTerm, locals: &BTreeSet<Name>) {
    match term {
        SurfaceTerm::Referent(value) if locals.contains(&value.value) => {
            *term = SurfaceTerm::Local(value.clone());
        }
        SurfaceTerm::Application(application) => {
            for nested in application.roles.values_mut() {
                bind_local_references(nested, locals);
            }
        }
        SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
            for nested in values {
                bind_local_references(nested, locals);
            }
        }
        SurfaceTerm::Product { fields, .. } => {
            for nested in fields.values_mut() {
                bind_local_references(nested, locals);
            }
        }
        SurfaceTerm::Referent(_)
        | SurfaceTerm::Local(_)
        | SurfaceTerm::Template(_)
        | SurfaceTerm::Variable(_)
        | SurfaceTerm::AnonymousHole(_)
        | SurfaceTerm::String(_)
        | SurfaceTerm::F32(_)
        | SurfaceTerm::Int(_)
        | SurfaceTerm::Bool(_)
        | SurfaceTerm::Intrinsic(_) => {}
    }
}

fn reject_bracketed_clause_terms(tokens: &[Token]) -> Result<(), ParseError> {
    for (open, token) in tokens.iter().enumerate() {
        if token.quoted || token.raw != "[" {
            continue;
        }
        let mut depth = 0usize;
        let mut close = None;
        for (index, candidate) in tokens.iter().enumerate().skip(open) {
            if !candidate.quoted && candidate.raw == "[" {
                depth += 1;
            } else if !candidate.quoted && candidate.raw == "]" {
                depth -= 1;
                if depth == 0 {
                    close = Some(index);
                    break;
                }
            }
        }
        let Some(close) = close else {
            continue;
        };
        let inner = &tokens[open + 1..close];
        if split_top_level(inner, ",").len() != 1 {
            continue;
        }
        let Ok(SurfaceTerm::Referent(referent)) = parse_role_term(inner) else {
            continue;
        };
        let close_token = &tokens[close];
        return Err(error(
            Span {
                line: token.span.line,
                column: token.span.column,
                width: close_token.span.column + close_token.span.width - token.span.column,
            },
            format!(
                "bracketed concrete referents are retired; write '{}'",
                referent.value.as_str()
            ),
        ));
    }
    Ok(())
}

fn collect_application_paths(path: &str, term: &SurfaceTerm, paths: &mut BTreeSet<String>) {
    let SurfaceTerm::Application(application) = term else {
        return;
    };
    let known = application
        .roles
        .keys()
        .map(RoleName::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    paths.insert(format!(
        "{path} -> {} [{known} -> {}]",
        application.relation.value.as_str(),
        application.result.value.as_str()
    ));
    for (role, nested) in &application.roles {
        collect_application_paths(
            &format!(
                "{path}/{}.{}",
                application.relation.value.as_str(),
                role.as_str()
            ),
            nested,
            paths,
        );
    }
}

fn application_candidate_paths(
    candidates: &[(Name, BTreeMap<RoleName, SurfaceTerm>)],
) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for (relation, roles) in candidates {
        for (role, term) in roles {
            collect_application_paths(
                &format!("{}.{}", relation.as_str(), role.as_str()),
                term,
                &mut paths,
            );
        }
    }
    paths
}

pub(super) fn relation_line_matches(
    line: SourceLine<'_>,
    relations: &BTreeMap<Name, RelationSpec>,
) -> Result<bool, ParseError> {
    let tokens = recursive_clause_tokens(line)?;
    let ungrouped = tokens
        .iter()
        .filter(|token| !is_open_parenthesis(token) && !is_close_parenthesis(token))
        .cloned()
        .collect::<Vec<_>>();
    let matched = relations
        .values()
        .any(|spec| !shape_matches(&spec.shape, &ungrouped).is_empty());
    if !matched {
        reject_bracketed_clause_terms(&tokens)?;
    }
    Ok(matched)
}

pub(super) fn clause(
    line: SourceLine<'_>,
    current_model: &Name,
    relations: &BTreeMap<Name, RelationSpec>,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    variable_domains: &mut BTreeMap<VariableName, DomainName>,
) -> Result<SurfaceClause, ParseError> {
    let current_memberships = memberships
        .get(current_model)
        .expect("current model was declared before its clauses");
    clause_with_catalog(
        line,
        current_memberships,
        relations,
        memberships,
        variable_domains,
    )
}

pub(super) fn clause_with_catalog(
    line: SourceLine<'_>,
    current_memberships: &MembershipCatalog,
    relations: &BTreeMap<Name, RelationSpec>,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    variable_domains: &mut BTreeMap<VariableName, DomainName>,
) -> Result<SurfaceClause, ParseError> {
    let tokens = recursive_clause_tokens(line)?;
    let mut candidates = Vec::new();
    let mut first_term_error = None;
    let mut chart = TermChart::new();
    for (relation, spec) in relations {
        for terms in recursive_shape_matches(
            spec,
            &tokens,
            current_memberships,
            memberships,
            relations,
            &mut chart,
        ) {
            let mut accepted = true;
            for (role, term) in &terms {
                let expected = spec.roles.get(role).expect("shape roles populate spec");
                match term_domains(term, current_memberships, memberships) {
                    Ok(Some(actual)) if !actual.contains(expected) => {
                        accepted = false;
                        break;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        first_term_error.get_or_insert(error);
                        accepted = false;
                        break;
                    }
                }
                if let SurfaceTerm::Variable(variable) = term
                    && let Some(previous) = variable_domains.get(&variable.value)
                    && previous != expected
                {
                    accepted = false;
                    break;
                }
            }
            if accepted {
                candidates.push((relation.clone(), terms));
            }
        }
    }
    if candidates.is_empty() {
        for spec in relations.values() {
            for terms in shape_matches(&spec.shape, &tokens) {
                for term in terms.values() {
                    if let Err(error) = term_domains(term, current_memberships, memberships) {
                        first_term_error.get_or_insert(error);
                    }
                }
            }
        }
    }
    if candidates.is_empty() {
        structural_term(&tokens, current_memberships, memberships)?;
        reject_bracketed_clause_terms(&tokens)?;
        if let Some(error) = first_term_error {
            return Err(error);
        }
        return Err(error(
            line_span(line),
            "no declared sentence shape accepts this clause",
        ));
    }
    if candidates.len() > 1 {
        let application_paths = application_candidate_paths(&candidates);
        if !application_paths.is_empty() {
            return Err(error(
                line_span(line),
                format!(
                    "ambiguous clause; conflicting candidate paths: {}",
                    application_paths.into_iter().collect::<Vec<_>>().join("; ")
                ),
            ));
        }
        let descriptions = candidates
            .iter()
            .map(|(name, _)| {
                let roles = relations[name]
                    .roles
                    .keys()
                    .map(RoleName::as_str)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} [{roles}]", name.as_str())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join("; ");
        return Err(error(
            line_span(line),
            format!("ambiguous clause; conflicting schemas and roles: {descriptions}"),
        ));
    }
    let (relation, roles) = candidates.pop().expect("nonempty candidates");
    let spec = relations.get(&relation).expect("candidate relation exists");
    for (role, term) in &roles {
        if let SurfaceTerm::Variable(variable) = term {
            variable_domains.insert(variable.value.clone(), spec.roles[role].clone());
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

pub(super) fn ground(clause: &SurfaceClause) -> bool {
    clause.roles.values().all(term_is_ground)
}

pub(super) fn clause_key(clause: &SurfaceClause) -> String {
    let mut key = clause.relation.value.0.clone();
    for (role, term) in &clause.roles {
        key.push('|');
        key.push_str(role.as_str());
        key.push('=');
        key.push_str(&term_key(term));
    }
    key
}

pub(super) fn variables(clause: &SurfaceClause) -> BTreeSet<VariableName> {
    fn collect(term: &SurfaceTerm, variables: &mut BTreeSet<VariableName>) {
        match term {
            SurfaceTerm::Variable(value) => {
                variables.insert(value.value.clone());
            }
            SurfaceTerm::Application(value) => {
                for term in value.roles.values() {
                    collect(term, variables);
                }
            }
            SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
                for term in values {
                    collect(term, variables);
                }
            }
            SurfaceTerm::Product { fields, .. } => {
                for term in fields.values() {
                    collect(term, variables);
                }
            }
            SurfaceTerm::Referent(_)
            | SurfaceTerm::Local(_)
            | SurfaceTerm::Template(_)
            | SurfaceTerm::AnonymousHole(_)
            | SurfaceTerm::String(_)
            | SurfaceTerm::F32(_)
            | SurfaceTerm::Int(_)
            | SurfaceTerm::Bool(_)
            | SurfaceTerm::Intrinsic(_) => {}
        }
    }

    let mut variables = BTreeSet::new();
    for term in clause.roles.values() {
        collect(term, &mut variables);
    }
    variables
}

pub(super) fn clause_has_hole(line: SourceLine<'_>) -> Result<bool, ParseError> {
    Ok(recursive_clause_tokens(line)?
        .iter()
        .any(|token| !token.quoted && token.raw.starts_with('?')))
}

pub(super) fn query_columns(clause: &SurfaceClause) -> Vec<QueryColumnDecl> {
    fn collect(term: &SurfaceTerm, columns: &mut Vec<QueryColumnDecl>) {
        match term {
            SurfaceTerm::Variable(value) => columns.push(QueryColumnDecl {
                label: Some(value.value.clone()),
                span: value.span,
            }),
            SurfaceTerm::AnonymousHole(span) => columns.push(QueryColumnDecl {
                label: None,
                span: *span,
            }),
            SurfaceTerm::Application(value) => {
                for term in value.roles.values() {
                    collect(term, columns);
                }
            }
            SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
                for term in values {
                    collect(term, columns);
                }
            }
            SurfaceTerm::Product { fields, .. } => {
                for term in fields.values() {
                    collect(term, columns);
                }
            }
            SurfaceTerm::Referent(_)
            | SurfaceTerm::Local(_)
            | SurfaceTerm::Template(_)
            | SurfaceTerm::String(_)
            | SurfaceTerm::F32(_)
            | SurfaceTerm::Int(_)
            | SurfaceTerm::Bool(_)
            | SurfaceTerm::Intrinsic(_) => {}
        }
    }

    let mut columns = Vec::new();
    for term in clause.roles.values() {
        collect(term, &mut columns);
    }
    columns.sort_by_key(|column| (column.span.line, column.span.column));
    let mut named = BTreeSet::new();
    columns.retain(|column| {
        column
            .label
            .as_ref()
            .is_none_or(|label| named.insert(label.clone()))
    });
    columns
}
