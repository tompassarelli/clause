use super::source::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct RelationSpec {
    pub(super) shape: SentenceShapeDecl,
    pub(super) modes: Vec<ModeDecl>,
    pub(super) roles: BTreeMap<RoleName, DomainName>,
}

fn parse_shape(line: SourceLine<'_>) -> Result<SentenceShapeDecl, ParseError> {
    let text = content(line);
    let mut parts = Vec::new();
    let offset = 2;
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
                    "unterminated role domain",
                )
            })?;
        let inside = &text[open + 1..close];
        let (role, domain) = inside.split_once(": ").ok_or_else(|| {
            error(
                child_span(line, offset + open, close - open + 1),
                "expected '{role: domain}'",
            )
        })?;
        if role.contains(':') || domain.contains(':') {
            return Err(error(
                child_span(line, offset + open, close - open + 1),
                "malformed role domain",
            ));
        }
        parts.push(ShapePartDecl::Role {
            id: role_name(line, offset + open + 1, role)?,
            domain: domain_name(line, offset + open + 1 + role.len() + 2, domain)?,
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
        if let ShapePartDecl::Role { id, .. } = part
            && !roles.insert(id.value.clone())
        {
            return Err(error(
                id.span,
                format!("duplicate inline role '{}'", id.value.as_str()),
            ));
        }
    }
    Ok(SentenceShapeDecl {
        parts,
        span: line_span(line),
    })
}

fn parse_compact_shape(line: SourceLine<'_>) -> Result<SentenceShapeDecl, ParseError> {
    let text = content(line);
    let base = indent(line)?;
    let mut offset = base;
    let tokens = text
        .split(' ')
        .map(|token| {
            let current = offset;
            offset += token.len() + 1;
            (token, current)
        })
        .collect::<Vec<_>>();
    if tokens.iter().any(|(token, _)| token.is_empty()) {
        return Err(error(
            line_span(line),
            "compact relation roles require single spaces",
        ));
    }
    let markers = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, (token, _))| token.ends_with(':').then_some(index))
        .collect::<Vec<_>>();
    if markers.len() < 2 || markers.first() != Some(&0) {
        return Err(error(
            line_span(line),
            "compact relation phrase must begin with a role and contain at least two roles",
        ));
    }

    let mut parts = Vec::new();
    let mut roles = BTreeSet::new();
    for (ordinal, marker) in markers.iter().copied().enumerate() {
        let (role_token, role_offset) = tokens[marker];
        let role_text = role_token
            .strip_suffix(':')
            .expect("role marker ends with ':'");
        let domain_index = marker + 1;
        let Some((domain_text, domain_offset)) = tokens.get(domain_index).copied() else {
            return Err(error(
                line_span(line),
                "compact relation role needs a domain",
            ));
        };
        if domain_text.ends_with(':') {
            return Err(error(
                child_span(line, domain_offset, domain_text.len()),
                "compact relation role needs a domain before the next role",
            ));
        }
        let role = role_name(line, role_offset, role_text)?;
        if !roles.insert(role.value.clone()) {
            return Err(error(
                role.span,
                format!("duplicate inline role '{}'", role.value.as_str()),
            ));
        }
        parts.push(ShapePartDecl::Role {
            id: role,
            domain: domain_name(line, domain_offset, domain_text)?,
        });

        if let Some(next_marker) = markers.get(ordinal + 1).copied() {
            let literal_tokens = &tokens[domain_index + 1..next_marker];
            if literal_tokens.is_empty() {
                return Err(error(
                    line_span(line),
                    "roles require a nonempty literal between them",
                ));
            }
            let literal = literal_tokens
                .iter()
                .map(|(token, _)| *token)
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(ShapePartDecl::Literal(Spanned {
                value: literal.clone(),
                span: child_span(line, literal_tokens[0].1, literal.len()),
            }));
        } else if domain_index + 1 != tokens.len() {
            return Err(error(
                line_span(line),
                "compact relation phrase must end with a role domain",
            ));
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

fn parse_compact_role_list(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Vec<Spanned<RoleName>>, ParseError> {
    if text.is_empty() || text.split(' ').any(str::is_empty) {
        return Err(error(
            child_span(line, offset, text.len()),
            "compact projection roles require single spaces",
        ));
    }
    let mut position = 0;
    let mut seen = BTreeSet::new();
    text.split(' ')
        .map(|item| {
            let role = role_name(line, offset + position, item)?;
            position += item.len() + 1;
            if !seen.insert(role.value.clone()) {
                return Err(error(
                    role.span,
                    format!("duplicate mode role '{}'", role.value.as_str()),
                ));
            }
            Ok(role)
        })
        .collect()
}

fn validate_mode_roles(
    known: &[Spanned<RoleName>],
    sought: &[Spanned<RoleName>],
    roles: &BTreeMap<RoleName, DomainName>,
) -> Result<(), ParseError> {
    let mut every = BTreeSet::new();
    for role in known.iter().chain(sought) {
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
    Ok(())
}

fn parse_mode(
    line: SourceLine<'_>,
    roles: &BTreeMap<RoleName, DomainName>,
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
    let known = parse_role_list(line, 2 + "mode ".len(), known)?;
    let sought = parse_role_list(line, 2 + "mode ".len() + known_text_width(&known), sought)?;
    validate_mode_roles(&known, &sought, roles)?;
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

fn parse_compact_mode(
    line: SourceLine<'_>,
    roles: &BTreeMap<RoleName, DomainName>,
) -> Result<ModeDecl, ParseError> {
    let text = content(line);
    let (known_text, sought_with_cardinality) = text
        .split_once(" -> ")
        .ok_or_else(|| error(line_span(line), "expected 'known -> sought' projection"))?;
    let (sought_text, cardinality) = if let Some(sought) = sought_with_cardinality.strip_suffix('*')
    {
        (sought, Cardinality::Many)
    } else if let Some(sought) = sought_with_cardinality.strip_suffix('+') {
        (sought, Cardinality::Some)
    } else {
        (sought_with_cardinality, Cardinality::One)
    };
    let base = indent(line)?;
    let known = parse_compact_role_list(line, base, known_text)?;
    let sought =
        parse_compact_role_list(line, base + known_text.len() + " -> ".len(), sought_text)?;
    validate_mode_roles(&known, &sought, roles)?;
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

pub(super) fn relation_spec(raw: &RawDecl<'_>) -> Result<RelationSpec, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    if entries.is_empty()
        || entries.iter().any(|line| {
            indent(*line).expect("source indentation was validated before parsing") != 2
        })
    {
        return Err(error(
            line_span(raw.header),
            "RelationShape requires two-space sentence and mode members",
        ));
    }
    let compact = content(raw.header).ends_with(':');
    let shape = if compact {
        parse_compact_shape(entries[0])?
    } else {
        parse_shape(entries[0])?
    };
    let mut roles = BTreeMap::new();
    for part in &shape.parts {
        if let ShapePartDecl::Role { id, domain } = part {
            roles.insert(id.value.clone(), domain.value.clone());
        }
    }
    let modes = entries[1..]
        .iter()
        .copied()
        .map(|line| {
            if compact {
                parse_compact_mode(line, &roles)
            } else {
                parse_mode(line, &roles)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if modes.is_empty() {
        return Err(error(
            line_span(raw.header),
            "RelationShape requires one or more mode declarations",
        ));
    }
    Ok(RelationSpec {
        shape,
        modes,
        roles,
    })
}

pub(super) fn compact_relation_candidate(raw: &RawDecl<'_>) -> bool {
    let entries = nonblank(raw.body.iter().copied());
    content(raw.header).ends_with(':')
        && entries.len() == 2
        && content(entries[0]).matches(": ").count() >= 2
        && content(entries[1]).contains(" -> ")
}
