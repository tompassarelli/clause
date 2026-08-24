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
        let name = referent_name(
            SourceLine {
                number: token.span.line,
                text: "",
            },
            token.span.column,
            &token.raw,
        )?;
        return Ok(SurfaceTerm::Referent(Spanned {
            value: name.value,
            span: token.span,
        }));
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
    current_model: &Name,
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
                let catalog = memberships
                    .get(current_model)
                    .expect("current model was declared before its clauses");
                if let Some(domains) = catalog.explicit.get(&referent.value) {
                    return Ok(Some(domains.clone()));
                }
                let matched = catalog
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

pub(super) fn relation_line_matches(
    line: SourceLine<'_>,
    relations: &BTreeMap<Name, RelationSpec>,
) -> Result<bool, ParseError> {
    let tokens = lex_clause(line)?;
    Ok(relations.values().any(|spec| {
        let pattern = shape_tokens(&spec.shape);
        pattern.len() == tokens.len()
            && pattern.iter().zip(&tokens).all(|(part, token)| match part {
                Some(word) => !token.quoted && token.raw == *word,
                None => true,
            })
    }))
}

pub(super) fn clause(
    line: SourceLine<'_>,
    current_model: &Name,
    relations: &BTreeMap<Name, RelationSpec>,
    memberships: &BTreeMap<Name, MembershipCatalog>,
    variable_domains: &mut BTreeMap<VariableName, DomainName>,
) -> Result<SurfaceClause, ParseError> {
    let tokens = lex_clause(line)?;
    let mut candidates = Vec::new();
    for (relation, spec) in relations {
        let pattern = shape_tokens(&spec.shape);
        if pattern.len() != tokens.len() {
            continue;
        }
        let mut terms = BTreeMap::new();
        let roles = spec
            .shape
            .parts
            .iter()
            .filter_map(|part| match part {
                ShapePartDecl::Role { id, .. } => Some(&id.value),
                ShapePartDecl::Literal(_) => None,
            })
            .collect::<Vec<_>>();
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
                    let role = (*roles[role_index]).clone();
                    role_index += 1;
                    let term = parse_term(token)?;
                    let expected = spec.roles.get(&role).expect("shape roles populate spec");
                    if let Some(actual) = term_domains(&term, current_model, memberships)?
                        && !actual.contains(expected)
                    {
                        matches = false;
                        break;
                    }
                    if let SurfaceTerm::Variable(variable) = &term
                        && let Some(previous) = variable_domains.get(&variable.value)
                        && previous != expected
                    {
                        matches = false;
                        break;
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
