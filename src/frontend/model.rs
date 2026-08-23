use super::clause::{focus_term, lex_clause};
use super::source::*;
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct EntityCatalog {
    pub(super) explicit: BTreeMap<Name, TypeName>,
    pub(super) groups: Vec<EntityGroupDecl>,
}

pub(super) fn entity_line(line: SourceLine<'_>) -> Option<Result<EntityDecl, ParseError>> {
    let text = content(line);
    let (local_text, typ) = text.split_once(": ")?;
    if local_text.contains(':') || typ.contains(':') {
        return None;
    }
    Some((|| {
        let local = if let Some((inside, close, tail)) = bracket_contents(line, local_text)? {
            if close != local_text.len() || !tail.is_empty() {
                return Err(error(line_span(line), "malformed bracketed entity"));
            }
            entity_name(line, 5, inside)?
        } else {
            qname(line, 4, local_text)?
        };
        Ok(EntityDecl {
            local,
            typ: type_name(line, 4 + local_text.len() + 2, typ)?,
            span: line_span(line),
        })
    })())
}

fn bracket_contents<'a>(
    line: SourceLine<'a>,
    text: &'a str,
) -> Result<Option<(&'a str, usize, &'a str)>, ParseError> {
    if !text.starts_with('[') {
        return Ok(None);
    }
    let close = text
        .find(']')
        .ok_or_else(|| error(line_span(line), "unterminated bracketed entity"))?;
    if text[1..close].contains('[') {
        return Err(error(line_span(line), "malformed bracketed entity"));
    }
    Ok(Some((&text[1..close], close + 1, &text[close + 1..])))
}

pub(super) fn entity_group_line(
    line: SourceLine<'_>,
) -> Option<Result<EntityGroupDecl, ParseError>> {
    let text = content(line);
    let (inside, close, tail) = match bracket_contents(line, text) {
        Ok(Some(contents)) => contents,
        Ok(None) => return None,
        Err(error) => return Some(Err(error)),
    };
    let typ = tail.strip_prefix(": ")?;
    if typ.contains(':') || inside.contains('{') || inside.contains('}') {
        return Some(Err(error(line_span(line), "malformed finite entity group")));
    }
    let (before_end, range_end) = inside.split_once("..")?;
    let start_offset = before_end
        .char_indices()
        .rev()
        .take_while(|(_, character)| character.is_ascii_digit())
        .last()
        .map_or(before_end.len(), |(offset, _)| offset);
    let end_digits = range_end
        .char_indices()
        .take_while(|(_, character)| character.is_ascii_digit())
        .map(|(offset, character)| offset + character.len_utf8())
        .last()
        .unwrap_or(0);
    if start_offset == before_end.len() || end_digits == 0 {
        return Some(Err(error(
            line_span(line),
            "finite entity group requires one integer range",
        )));
    }
    let range_text = &inside[start_offset..before_end.len() + 2 + end_digits];
    if range_end[end_digits..].contains("..") || before_end[..start_offset].contains("..") {
        return Some(Err(error(
            line_span(line),
            "finite entity group permits exactly one integer range",
        )));
    }
    let prefix = &inside[..start_offset];
    let suffix = &range_end[end_digits..];
    if entity_name(line, 4 + 1, &format!("{prefix}0{suffix}")).is_err() {
        return Some(Err(error(
            line_span(line),
            "finite entity group does not form valid bracketed entity names",
        )));
    }
    Some((|| {
        Ok(EntityGroupDecl {
            prefix: Spanned {
                value: prefix.to_owned(),
                span: child_span(line, 5, prefix.len()),
            },
            range: integer_range(line, 5 + start_offset, range_text)?,
            suffix: Spanned {
                value: suffix.to_owned(),
                span: child_span(line, 5 + before_end.len() + 2 + end_digits, suffix.len()),
            },
            typ: type_name(line, 4 + close + 2, typ)?,
            span: line_span(line),
        })
    })())
}

pub(super) fn focus_template(line: SourceLine<'_>) -> Option<Result<EntityTemplate, ParseError>> {
    let text = content(line);
    let (inside, _close, tail) = match bracket_contents(line, text) {
        Ok(Some(contents)) => contents,
        Ok(None) => return None,
        Err(error) => return Some(Err(error)),
    };
    if tail != ":" {
        return None;
    }
    let open = inside.find('{')?;
    let close = match inside[open + 1..].find('}') {
        Some(close) => open + 1 + close,
        None => {
            return Some(Err(error(
                line_span(line),
                "unterminated focus template variable",
            )));
        }
    };
    if inside[close + 1..].contains('{')
        || inside[..open].contains('}')
        || inside[close + 1..].contains('}')
    {
        return Some(Err(error(
            line_span(line),
            "focus head permits exactly one template variable",
        )));
    }
    let prefix = &inside[..open];
    let variable = &inside[open + 1..close];
    let suffix = &inside[close + 1..];
    if variable.is_empty() || entity_name(line, 5, &format!("{prefix}0{suffix}")).is_err() {
        return Some(Err(error(
            line_span(line),
            "malformed correlated focus template",
        )));
    }
    Some((|| {
        Ok(EntityTemplate {
            prefix: Spanned {
                value: prefix.to_owned(),
                span: child_span(line, 5, prefix.len()),
            },
            variable: variable_name(line, 5 + open + 1, variable)?,
            suffix: Spanned {
                value: suffix.to_owned(),
                span: child_span(line, 5 + close + 1, suffix.len()),
            },
            span: line_span(line),
        })
    })())
}

pub(super) fn focus_slot(line: SourceLine<'_>) -> Result<FocusSlot, ParseError> {
    if indent(line)? != 8 {
        return Err(error(
            line_span(line),
            "focus slots must use eight-space indentation",
        ));
    }
    let text = content(line);
    let (label, value) = text
        .split_once(": ")
        .ok_or_else(|| error(line_span(line), "focus slot requires 'label: value'"))?;
    if label.contains(':') || value.is_empty() {
        return Err(error(line_span(line), "focus slot requires 'label: value'"));
    }
    let label = label.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    if label.is_empty()
        || label
            .chars()
            .any(|character| matches!(character, '{' | '}' | '[' | ']' | '"' | '?'))
    {
        return Err(error(
            line_span(line),
            "focus slot requires a sentence literal prefix",
        ));
    }
    let mut tokens = lex_clause(SourceLine {
        number: line.number,
        text: value,
    })?;
    let value_offset = 8 + label.len() + 2;
    for token in &mut tokens {
        token.span.column += value_offset;
    }
    if tokens.len() != 1 {
        return Err(error(
            line_span(line),
            "focus slot value must be one surface term",
        ));
    }
    let label_width = label.len();
    Ok(FocusSlot {
        label: Spanned {
            value: label,
            span: child_span(line, 8, label_width),
        },
        value: focus_term(&tokens[0])?,
        span: line_span(line),
    })
}

pub(super) fn focus_binding(line: SourceLine<'_>) -> Result<FocusBinding, ParseError> {
    if indent(line)? != 4 {
        return Err(error(
            line_span(line),
            "focus binding must use four-space indentation",
        ));
    }
    let text = content(line);
    let rest = text.strip_prefix("for ").ok_or_else(|| {
        error(
            line_span(line),
            "focus block requires 'for name: start..end'",
        )
    })?;
    let (variable, range) = rest.split_once(": ").ok_or_else(|| {
        error(
            line_span(line),
            "focus block requires 'for name: start..end'",
        )
    })?;
    Ok(FocusBinding {
        variable: variable_name(line, 4 + "for ".len(), variable)?,
        range: integer_range(line, 4 + "for ".len() + variable.len() + 2, range)?,
        span: line_span(line),
    })
}

pub(super) fn model_entities(raw: &RawDecl<'_>) -> Result<EntityCatalog, ParseError> {
    let mut explicit = BTreeMap::new();
    let mut groups = Vec::new();
    for line in nonblank(raw.body.iter().copied()) {
        match indent(line)? {
            4 => {
                if let Some(group) = entity_group_line(line) {
                    let group = group?;
                    groups.push(group);
                } else if let Some(template) = focus_template(line) {
                    template?;
                } else if content(line).starts_with("for ") {
                    // The later Model pass verifies that it belongs to the
                    // immediately preceding focus block.
                } else if let Some(entity) = entity_line(line) {
                    let entity = entity?;
                    if entity.local.value.0.contains('/') {
                        return Err(error(
                            entity.local.span,
                            "model entity names cannot be qualified",
                        ));
                    }
                    if explicit
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
            8 => {}
            _ => {
                return Err(error(
                    line_span(line),
                    "Model members must use four or eight-space indentation",
                ));
            }
        }
    }
    Ok(EntityCatalog { explicit, groups })
}
