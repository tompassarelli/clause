use super::clause::{focus_term, lex_clause, relation_line_matches};
use super::relation::RelationSpec;
use super::source::*;
use super::syntax::{DefinitionDecl, MembershipDecl};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct MembershipCatalog {
    pub(super) explicit: BTreeMap<Name, BTreeSet<DomainName>>,
    pub(super) ranges: Vec<MembershipRangeDecl>,
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

pub(super) fn infer_bare_block_kind(
    raw: &RawDecl<'_>,
    relations: &BTreeMap<Name, RelationSpec>,
) -> Result<Kind, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    let flat = entries
        .iter()
        .all(|line| indent(*line).is_ok_and(|width| width == 2));
    if flat
        && entries
            .iter()
            .all(|line| definition_line(*line).is_some_and(|binding| binding.is_ok()))
    {
        return Ok(Kind::BindingShape);
    }
    if flat {
        let mut enumeration = true;
        for line in &entries {
            if semantic_name(*line, 2, content(*line)).is_err()
                || relation_line_matches(*line, relations)?
            {
                enumeration = false;
                break;
            }
        }
        if enumeration {
            return Ok(Kind::Enumeration);
        }
    }
    Ok(Kind::Model)
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
        .ok_or_else(|| error(line_span(line), "unterminated bracketed referent"))?;
    if text[1..close].contains('[') {
        return Err(error(line_span(line), "malformed bracketed referent"));
    }
    Ok(Some((&text[1..close], close + 1, &text[close + 1..])))
}

pub(super) fn membership_range_line(
    line: SourceLine<'_>,
) -> Option<Result<MembershipRangeDecl, ParseError>> {
    let text = content(line);
    let (inside, close, tail) = match bracket_contents(line, text) {
        Ok(Some(contents)) => contents,
        Ok(None) => return None,
        Err(error) => return Some(Err(error)),
    };
    let group = tail.strip_prefix(" ∈ ")?;
    if group.contains('∈') || inside.contains('{') || inside.contains('}') {
        return Some(Err(error(
            line_span(line),
            "malformed finite membership range",
        )));
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
            "finite membership requires one integer range",
        )));
    }
    let range_text = &inside[start_offset..before_end.len() + 2 + end_digits];
    if range_end[end_digits..].contains("..") || before_end[..start_offset].contains("..") {
        return Some(Err(error(
            line_span(line),
            "finite membership permits exactly one integer range",
        )));
    }
    let prefix = &inside[..start_offset];
    let suffix = &range_end[end_digits..];
    if referent_name(line, indent(line).ok()? + 1, &format!("{prefix}0{suffix}")).is_err() {
        return Some(Err(error(
            line_span(line),
            "finite membership does not form valid bracketed referent names",
        )));
    }
    Some((|| {
        let base = indent(line)?;
        Ok(MembershipRangeDecl {
            prefix: Spanned {
                value: prefix.to_owned(),
                span: child_span(line, base + 1, prefix.len()),
            },
            range: integer_range(line, base + 1 + start_offset, range_text)?,
            suffix: Spanned {
                value: suffix.to_owned(),
                span: child_span(
                    line,
                    base + 1 + before_end.len() + 2 + end_digits,
                    suffix.len(),
                ),
            },
            group: domain_name(line, base + close + " ∈ ".len(), group)?,
            span: line_span(line),
        })
    })())
}

pub(super) fn focus_template(line: SourceLine<'_>) -> Option<Result<ReferentTemplate, ParseError>> {
    let text = content(line);
    let (inside, _close, tail) = match bracket_contents(line, text) {
        Ok(Some(contents)) => contents,
        Ok(None) => return None,
        Err(error) => return Some(Err(error)),
    };
    if !tail.is_empty() {
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
    if variable.is_empty()
        || referent_name(line, indent(line).ok()? + 1, &format!("{prefix}0{suffix}")).is_err()
    {
        return Some(Err(error(
            line_span(line),
            "malformed correlated focus template",
        )));
    }
    Some((|| {
        let base = indent(line)?;
        Ok(ReferentTemplate {
            prefix: Spanned {
                value: prefix.to_owned(),
                span: child_span(line, base + 1, prefix.len()),
            },
            variable: variable_name(line, base + 1 + open + 1, variable)?,
            suffix: Spanned {
                value: suffix.to_owned(),
                span: child_span(line, base + 1 + close + 1, suffix.len()),
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
    let tokens = lex_clause(line)?;
    let value = tokens
        .last()
        .ok_or_else(|| error(line_span(line), "focus slot requires a relational phrase"))?;
    let value_start = value.span.column - 1 - indent(line)?;
    let label = text[..value_start].trim_end();
    if label.is_empty()
        || label.contains(':')
        || label
            .chars()
            .any(|character| matches!(character, '{' | '}' | '[' | ']' | '"' | '?'))
    {
        return Err(error(
            line_span(line),
            "focus slot requires a relational phrase before its participant",
        ));
    }
    let label = label.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    let label_width = label.len();
    Ok(FocusSlot {
        label: Spanned {
            value: label,
            span: child_span(line, 4, label_width),
        },
        value: focus_term(value)?,
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

pub(super) fn model_memberships(
    raw: &RawDecl<'_>,
    grounded: &BTreeSet<Name>,
) -> Result<MembershipCatalog, ParseError> {
    let mut explicit = BTreeMap::new();
    let mut ranges = Vec::new();
    let entries = nonblank(raw.body.iter().copied());
    if raw.kind == Kind::Enumeration {
        for line in entries {
            insert_membership(
                &mut explicit,
                &MembershipDecl {
                    member: semantic_name(line, 2, content(line))?,
                    group: raw.subject.clone(),
                    span: line_span(line),
                },
            );
        }
        return Ok(MembershipCatalog { explicit, ranges });
    }
    for (index, line) in entries.iter().copied().enumerate() {
        match indent(line)? {
            2 => {
                if let Some(range) = membership_range_line(line) {
                    let range = range?;
                    ranges.push(range);
                } else if let Some(template) = focus_template(line) {
                    template?;
                } else if content(line).starts_with("for ") {
                    // The later Model pass verifies that it belongs to the
                    // immediately preceding focus block.
                } else if let Some(membership) = membership_line(line) {
                    let membership = membership?;
                    insert_membership(&mut explicit, &membership);
                } else if definition_line(line).is_some() {
                    definition_line(line).expect("checked binding shape")?;
                } else if entries.get(index + 1).is_some_and(|next| {
                    indent(*next).expect("source indentation was validated") == 4
                }) {
                    let focus = focused_name(line)?;
                    let first_child = entries[index + 1];
                    if let Ok(group) = focused_name(first_child)
                        && grounded.contains(&group.value)
                    {
                        insert_membership(
                            &mut explicit,
                            &MembershipDecl {
                                member: focus,
                                group,
                                span: line_span(first_child),
                            },
                        );
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
    Ok(MembershipCatalog { explicit, ranges })
}

pub(super) fn insert_membership(
    explicit: &mut BTreeMap<Name, BTreeSet<DomainName>>,
    membership: &MembershipDecl,
) {
    explicit
        .entry(membership.member.value.clone())
        .or_default()
        .insert(DomainName(membership.group.value.0.clone()));
}
