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
        return Ok(SurfaceTerm::Variable(variable_name(
            SourceLine {
                number: token.span.line,
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
        SurfaceTerm::Variable(_) => Ok(None),
        SurfaceTerm::Template(template) => Err(error(
            template.span,
            "correlated referent templates are only valid inside a focus block",
        )),
        SurfaceTerm::Referent(referent) => {
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

fn reject_bracketed_clause_terms(tokens: &[Token]) -> Result<(), ParseError> {
    if let Some(token) = tokens.iter().find(|token| token.bracketed) {
        return Err(error(
            token.span,
            format!(
                "bracketed concrete referents are retired; write '{}'",
                token.raw
            ),
        ));
    }
    Ok(())
}

pub(super) fn relation_line_matches(
    line: SourceLine<'_>,
    relations: &BTreeMap<Name, RelationSpec>,
) -> Result<bool, ParseError> {
    let tokens = lex_clause(line)?;
    reject_bracketed_clause_terms(&tokens)?;
    Ok(relations
        .values()
        .any(|spec| !shape_matches(&spec.shape, &tokens).is_empty()))
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
    let tokens = lex_clause(line)?;
    reject_bracketed_clause_terms(&tokens)?;
    let mut candidates = Vec::new();
    let mut first_term_error = None;
    for (relation, spec) in relations {
        for terms in shape_matches(&spec.shape, &tokens) {
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
        if let Some(error) = first_term_error {
            return Err(error);
        }
        return Err(error(
            line_span(line),
            "no declared sentence shape accepts this clause",
        ));
    }
    if candidates.len() > 1 {
        let names = candidates
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
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
    clause
        .roles
        .values()
        .all(|term| !matches!(term, SurfaceTerm::Variable(_)))
}

pub(super) fn clause_key(clause: &SurfaceClause) -> String {
    let mut key = clause.relation.value.0.clone();
    for (role, term) in &clause.roles {
        key.push('|');
        key.push_str(role.as_str());
        key.push('=');
        match term {
            SurfaceTerm::Referent(value) => key.push_str(&format!("R:{}", value.value.0)),
            SurfaceTerm::Template(value) => key.push_str(&format!(
                "T:{}{{{}}}{}",
                value.prefix.value, value.variable.value.0, value.suffix.value
            )),
            SurfaceTerm::Variable(value) => key.push_str(&format!("V:{}", value.value.0)),
            SurfaceTerm::String(value) => key.push_str(&format!("S:{:?}", value.value)),
        }
    }
    key
}

pub(super) fn variables(clause: &SurfaceClause) -> BTreeSet<VariableName> {
    clause
        .roles
        .values()
        .filter_map(|term| match term {
            SurfaceTerm::Variable(value) => Some(value.value.clone()),
            _ => None,
        })
        .collect()
}
