use super::source::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct RelationSpec {
    pub(super) shape: SentenceShapeDecl,
    pub(super) modes: Vec<ModeDecl>,
    pub(super) roles: BTreeMap<RoleName, TypeName>,
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

pub(super) fn relation_spec(raw: &RawDecl<'_>) -> Result<RelationSpec, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    if entries.is_empty()
        || entries.iter().any(|line| {
            indent(*line).expect("source indentation was validated before parsing") != 4
        })
    {
        return Err(error(
            line_span(raw.header),
            "RelationShape requires four-space sentence and mode members",
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
            "RelationShape requires one or more mode declarations",
        ));
    }
    Ok(RelationSpec {
        shape,
        modes,
        roles,
    })
}
