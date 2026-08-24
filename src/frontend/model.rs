use super::clause::{focus_term, lex_clause};
use super::source::*;
use super::syntax::{DefinitionDecl, MembershipDecl};
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct EntityCatalog {
    pub(super) explicit: BTreeMap<Name, TypeName>,
    pub(super) groups: Vec<EntityGroupDecl>,
}

fn semantic_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<Name>, ParseError> {
    if text.is_empty()
        || text.trim() != text
        || text.chars().any(char::is_control)
        || text.contains([':', '∈'])
    {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected semantic name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: Name(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

pub(super) fn definition_line(line: SourceLine<'_>) -> Option<Result<DefinitionDecl, ParseError>> {
    let text = content(line);
    if text.contains("::") {
        return Some(Err(error(line_span(line), "raw '::' is not Clause syntax")));
    }
    let (name, denotation) = text.split_once(": ")?;
    Some((|| {
        if name.contains(':') || denotation.contains(':') {
            return Err(error(line_span(line), "binding requires one ':'"));
        }
        let base = indent(line)?;
        Ok(DefinitionDecl {
            name: semantic_name(line, base, name)?,
            denotation: semantic_name(line, base + name.len() + 2, denotation)?,
            span: line_span(line),
        })
    })())
}

pub(super) fn membership_line(line: SourceLine<'_>) -> Option<Result<MembershipDecl, ParseError>> {
    let text = content(line);
    let (member, group) = text.split_once(" ∈ ")?;
    Some((|| {
        if member.contains('∈') || group.contains('∈') {
            return Err(error(line_span(line), "membership requires one '∈'"));
        }
        let base = indent(line)?;
        Ok(MembershipDecl {
            member: semantic_name(line, base, member)?,
            group: semantic_name(line, base + member.len() + " ∈ ".len(), group)?,
            span: line_span(line),
        })
    })())
}

pub(super) fn focused_name(line: SourceLine<'_>) -> Result<Spanned<Name>, ParseError> {
    semantic_name(line, indent(line)?, content(line))
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
    if entity_name(line, 2 + 1, &format!("{prefix}0{suffix}")).is_err() {
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
            typ: type_name(line, 2 + close + 2, typ)?,
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
    if indent(line)? != 4 {
        return Err(error(
            line_span(line),
            "focus slots must use four-space indentation",
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
    let value_offset = 4 + label.len() + 2;
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
            span: child_span(line, 4, label_width),
        },
        value: focus_term(&tokens[0])?,
        span: line_span(line),
    })
}

pub(super) fn focus_binding(line: SourceLine<'_>) -> Result<FocusBinding, ParseError> {
    if indent(line)? != 2 {
        return Err(error(
            line_span(line),
            "focus binding must use two-space indentation",
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
        variable: variable_name(line, 2 + "for ".len(), variable)?,
        range: integer_range(line, 2 + "for ".len() + variable.len() + 2, range)?,
        span: line_span(line),
    })
}

pub(super) fn model_entities(raw: &RawDecl<'_>) -> Result<EntityCatalog, ParseError> {
    let mut explicit = BTreeMap::new();
    let mut groups = Vec::new();
    let entries = nonblank(raw.body.iter().copied());
    for (index, line) in entries.iter().copied().enumerate() {
        match indent(line)? {
            2 => {
                if let Some(group) = entity_group_line(line) {
                    let group = group?;
                    groups.push(group);
                } else if let Some(template) = focus_template(line) {
                    template?;
                } else if content(line).starts_with("for ") {
                    // The later Model pass verifies that it belongs to the
                    // immediately preceding focus block.
                } else if let Some(membership) = membership_line(line) {
                    let membership = membership?;
                    insert_membership_type(&mut explicit, &membership)?;
                } else if definition_line(line).is_some() {
                    definition_line(line).expect("checked binding shape")?;
                } else if entries.get(index + 1).is_some_and(|next| {
                    indent(*next).expect("source indentation was validated") == 4
                }) {
                    let focus = focused_name(line)?;
                    let first_child = entries[index + 1];
                    if content(first_child).split_ascii_whitespace().count() == 1 {
                        insert_membership_type(
                            &mut explicit,
                            &MembershipDecl {
                                member: focus,
                                group: focused_name(first_child)?,
                                span: line_span(first_child),
                            },
                        )?;
                    }
                }
            }
            4 => {}
            _ => {
                return Err(error(
                    line_span(line),
                    "Model members must use two or four-space indentation",
                ));
            }
        }
    }
    Ok(EntityCatalog { explicit, groups })
}

fn insert_membership_type(
    explicit: &mut BTreeMap<Name, TypeName>,
    membership: &MembershipDecl,
) -> Result<(), ParseError> {
    let typ = TypeName(membership.group.value.0.clone());
    if let Some(previous) = explicit.insert(membership.member.value.clone(), typ.clone())
        && previous != typ
    {
        return Err(error(
            membership.member.span,
            format!(
                "referent '{}' has conflicting memberships",
                membership.member.value.as_str()
            ),
        ));
    }
    Ok(())
}
