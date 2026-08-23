use super::model::EntityCatalog;
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
                        "bracketed entity words must be separated by ASCII spaces",
                    ));
                }
                if matches!(text.as_bytes()[index], b'[' | b'"') {
                    return Err(error(
                        child_span(line, base + index, 1),
                        "malformed bracketed entity",
                    ));
                }
                index += 1;
            }
            if index == text.len() {
                return Err(error(
                    child_span(line, base + start, text.len() - start),
                    "unterminated bracketed entity",
                ));
            }
            let value = &text[value_start..index];
            index += 1;
            if index < text.len() && text.as_bytes()[index] != b' ' {
                return Err(error(
                    child_span(line, base + index, 1),
                    "bracketed entity must be followed by a space",
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
            return Err(error(token.span, "variables cannot be bracketed entities"));
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
        let name = entity_name(
            SourceLine {
                number: token.span.line,
                text: "",
            },
            token.span.column,
            &token.raw,
        )?;
        return Ok(SurfaceTerm::Entity(Spanned {
            value: name.value,
            span: token.span,
        }));
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

pub(super) fn focus_term(token: &Token) -> Result<SurfaceTerm, ParseError> {
    if !token.bracketed || !token.raw.contains('{') {
        return parse_term(token);
    }
    let open = token.raw.find('{').expect("checked focus template marker");
    let close = token.raw[open + 1..]
        .find('}')
        .map(|offset| open + 1 + offset)
        .ok_or_else(|| error(token.span, "unterminated correlated entity template"))?;
    if token.raw[..open].contains('}')
        || token.raw[close + 1..].contains(['{', '}'])
        || token.raw[open + 1..close].is_empty()
    {
        return Err(error(
            token.span,
            "correlated entity template permits exactly one variable",
        ));
    }
    let prefix = &token.raw[..open];
    let variable = &token.raw[open + 1..close];
    let suffix = &token.raw[close + 1..];
    let source = SourceLine {
        number: token.span.line,
        text: "",
    };
    entity_name(source, token.span.column, &format!("{prefix}0{suffix}"))?;
    Ok(SurfaceTerm::Template(EntityTemplate {
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

fn entity_type(
    term: &SurfaceTerm,
    current_model: &Name,
    entities: &BTreeMap<Name, EntityCatalog>,
) -> Result<Option<TypeName>, ParseError> {
    match term {
        SurfaceTerm::String(_) => Ok(Some(TypeName("Text".to_owned()))),
        SurfaceTerm::Variable(_) => Ok(None),
        SurfaceTerm::Template(template) => Err(error(
            template.span,
            "correlated entity templates are only valid inside a focus block",
        )),
        SurfaceTerm::Entity(entity) => {
            if !entity.value.0.contains('/') {
                let catalog = entities
                    .get(current_model)
                    .expect("current model was declared before its clauses");
                if let Some(typ) = catalog.explicit.get(&entity.value) {
                    return Ok(Some(typ.clone()));
                }
                let mut matched = catalog.groups.iter().filter_map(|group| {
                    let name = entity.value.as_str();
                    let prefix = group.prefix.value.as_str();
                    let suffix = group.suffix.value.as_str();
                    let number = name
                        .strip_prefix(prefix)?
                        .strip_suffix(suffix)?
                        .parse::<u64>()
                        .ok()?;
                    (group.range.start <= number && number <= group.range.end)
                        .then(|| group.typ.value.clone())
                });
                let Some(typ) = matched.next() else {
                    return Err(error(
                        entity.span,
                        format!("unknown entity '{}'", entity.value.as_str()),
                    ));
                };
                if matched.any(|other| other != typ) {
                    return Err(error(
                        entity.span,
                        format!("ambiguous grouped entity '{}'", entity.value.as_str()),
                    ));
                }
                return Ok(Some(typ));
            }
            for (model, catalog) in entities {
                for (local, typ) in &catalog.explicit {
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

pub(super) fn clause(
    line: SourceLine<'_>,
    current_model: &Name,
    relations: &BTreeMap<Name, RelationSpec>,
    entities: &BTreeMap<Name, EntityCatalog>,
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
                    if let Some(actual) = entity_type(&term, current_model, entities)?
                        && &actual != expected
                    {
                        matches = false;
                        break;
                    }
                    if let SurfaceTerm::Variable(variable) = &term
                        && let Some(previous) = variable_types.get(&variable.value)
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
            SurfaceTerm::Entity(value) => key.push_str(&format!("E:{}", value.value.0)),
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
