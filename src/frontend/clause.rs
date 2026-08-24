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

fn recursive_clause_tokens(line: SourceLine<'_>) -> Result<Vec<Token>, ParseError> {
    let mut grouped = Vec::new();
    for token in lex_clause(line)? {
        if token.quoted || token.bracketed {
            grouped.push(token);
            continue;
        }
        let mut start = 0;
        for (offset, byte) in token.raw.bytes().enumerate() {
            if !matches!(byte, b'(' | b')') {
                continue;
            }
            if start < offset {
                grouped.push(Token {
                    raw: token.raw[start..offset].to_owned(),
                    quoted: false,
                    bracketed: false,
                    span: Span {
                        line: token.span.line,
                        column: token.span.column + start,
                        width: offset - start,
                    },
                });
            }
            grouped.push(Token {
                raw: token.raw[offset..offset + 1].to_owned(),
                quoted: false,
                bracketed: false,
                span: Span {
                    line: token.span.line,
                    column: token.span.column + offset,
                    width: 1,
                },
            });
            start = offset + 1;
        }
        if start < token.raw.len() {
            grouped.push(Token {
                raw: token.raw[start..].to_owned(),
                quoted: false,
                bracketed: false,
                span: Span {
                    line: token.span.line,
                    column: token.span.column + start,
                    width: token.raw.len() - start,
                },
            });
        }
    }

    let mut opens = Vec::new();
    for token in &grouped {
        if is_open_parenthesis(token) {
            opens.push(token.span);
        } else if is_close_parenthesis(token) && opens.pop().is_none() {
            return Err(error(token.span, "unmatched closing parenthesis"));
        }
    }
    if let Some(span) = opens.first() {
        return Err(error(*span, "unterminated parenthesized term"));
    }
    Ok(grouped)
}

fn balanced_tokens(tokens: &[Token]) -> bool {
    let mut depth = 0usize;
    for token in tokens {
        if is_open_parenthesis(token) {
            depth += 1;
        } else if is_close_parenthesis(token) {
            let Some(next) = depth.checked_sub(1) else {
                return false;
            };
            depth = next;
        }
    }
    depth == 0
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
        SurfaceTerm::Application(_) => Ok(None),
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

fn token_span(tokens: &[Token]) -> Span {
    let first = tokens.first().expect("term tokens are nonempty");
    let last = tokens.last().expect("term tokens are nonempty");
    Span {
        line: first.span.line,
        column: first.span.column,
        width: last.span.column + last.span.width - first.span.column,
    }
}

fn term_is_ground(term: &SurfaceTerm) -> bool {
    match term {
        SurfaceTerm::Referent(_) | SurfaceTerm::String(_) => true,
        SurfaceTerm::Application(application) => application.roles.values().all(term_is_ground),
        SurfaceTerm::Template(_) | SurfaceTerm::Variable(_) => false,
    }
}

fn term_key(term: &SurfaceTerm) -> String {
    match term {
        SurfaceTerm::Referent(value) => format!("R:{}", value.value.0),
        SurfaceTerm::Template(value) => format!(
            "T:{}{{{}}}{}",
            value.prefix.value, value.variable.value.0, value.suffix.value
        ),
        SurfaceTerm::Variable(value) => format!("V:{}", value.value.0),
        SurfaceTerm::String(value) => format!("S:{:?}", value.value),
        SurfaceTerm::Application(value) => {
            let mut key = format!("A:{}->{}", value.relation.value.0, value.result.value.0);
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
            if !roles.values().all(term_is_ground) {
                continue;
            }
            push_unique_term(
                &mut candidates,
                SurfaceTerm::Application(Box::new(SurfaceApplication {
                    relation: Spanned {
                        value: relation.clone(),
                        span,
                    },
                    roles,
                    result: result.clone(),
                    span,
                })),
            );
        }
    }
    chart.insert(key, Some(candidates.clone()));
    candidates
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
    reject_bracketed_clause_terms(&tokens)?;
    let ungrouped = tokens
        .into_iter()
        .filter(|token| !is_open_parenthesis(token) && !is_close_parenthesis(token))
        .collect::<Vec<_>>();
    Ok(relations
        .values()
        .any(|spec| !shape_matches(&spec.shape, &ungrouped).is_empty()))
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
    reject_bracketed_clause_terms(&tokens)?;
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
            SurfaceTerm::Referent(_) | SurfaceTerm::Template(_) | SurfaceTerm::String(_) => {}
        }
    }

    let mut variables = BTreeSet::new();
    for term in clause.roles.values() {
        collect(term, &mut variables);
    }
    variables
}
