use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct SourceLine<'a> {
    pub(super) number: usize,
    pub(super) column: usize,
    pub(super) text: &'a str,
}

#[derive(Clone, Debug)]
pub(super) struct RawDecl<'a> {
    pub(super) subject: Spanned<Name>,
    pub(super) kind: Kind,
    pub(super) bare_block: bool,
    pub(super) header: SourceLine<'a>,
    pub(super) body: Vec<SourceLine<'a>>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RawTopLevel<'a> {
    pub(super) line: SourceLine<'a>,
}

#[derive(Clone, Debug)]
pub(super) struct RawRule<'a> {
    pub(super) label: Option<Spanned<Name>>,
    pub(super) conclusion: SourceLine<'a>,
    pub(super) premises: Vec<SourceLine<'a>>,
    pub(super) header: SourceLine<'a>,
}

#[derive(Clone, Debug)]
enum RawItem<'a> {
    Declaration(RawDecl<'a>),
    TopLevel(RawTopLevel<'a>),
    Rule(RawRule<'a>),
}

#[derive(Clone, Debug)]
pub(super) enum RawRequest<'a> {
    Select {
        projected: Spanned<VariableName>,
        clause: SourceLine<'a>,
        header: SourceLine<'a>,
    },
    Any {
        clause: SourceLine<'a>,
        header: SourceLine<'a>,
    },
    Find {
        revision: Spanned<Name>,
        sought: Spanned<VariableName>,
        clause: SourceLine<'a>,
        header: SourceLine<'a>,
    },
    Why {
        revision: Spanned<Name>,
        all: bool,
        clause: SourceLine<'a>,
        header: SourceLine<'a>,
    },
    Intervention {
        verb: &'static str,
        revision: Spanned<Name>,
        selection: InterventionSelection,
        clause: SourceLine<'a>,
        using: Vec<Spanned<Name>>,
        header: SourceLine<'a>,
    },
    Diff {
        base: Spanned<Name>,
        successor: Spanned<Name>,
        header: SourceLine<'a>,
    },
}

type ScanOutput<'a> = (
    Vec<RawDecl<'a>>,
    Vec<RawTopLevel<'a>>,
    Vec<RawRule<'a>>,
    Vec<RawRequest<'a>>,
);

pub(super) fn error(span: Span, message: impl Into<String>) -> ParseError {
    ParseError {
        span,
        message: message.into(),
    }
}

pub(super) fn line_span(line: SourceLine<'_>) -> Span {
    Span {
        line: line.number,
        column: line.column,
        width: line.text.len(),
    }
}

pub(super) fn child_span(line: SourceLine<'_>, offset: usize, width: usize) -> Span {
    Span {
        line: line.number,
        column: line.column + offset,
        width,
    }
}

pub(super) fn indent(line: SourceLine<'_>) -> Result<usize, ParseError> {
    if line.text.contains('\t') {
        return Err(error(line_span(line), "tabs are not permitted"));
    }
    if line.text.ends_with('\r') {
        return Err(error(line_span(line), "carriage returns are not permitted"));
    }
    if line.text.trim().is_empty() {
        if line.text.is_empty() {
            return Ok(0);
        }
        return Err(error(line_span(line), "blank lines must be empty"));
    }
    let width = line.text.bytes().take_while(|byte| *byte == b' ').count();
    if !matches!(width, 0 | 2 | 4) {
        return Err(error(
            child_span(line, 0, width.max(1)),
            "indentation must be exactly zero, two, or four ASCII spaces",
        ));
    }
    Ok(width)
}

pub(super) fn content(line: SourceLine<'_>) -> &str {
    &line.text[indent(line).expect("validated source lines")..]
}

fn is_local_name(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub(super) fn is_qname(text: &str) -> bool {
    !text.is_empty() && text.split('/').all(is_local_name)
}

pub(super) fn qname(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<Name>, ParseError> {
    if !is_qname(text) {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected qualified name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: Name(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

pub(super) fn semantic_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<Name>, ParseError> {
    if text.contains(['[', ']']) {
        return Err(error(
            child_span(line, offset, text.len()),
            "bracketed concrete referents are retired; brackets are reserved for ranges and templates",
        ));
    }
    if text.is_empty()
        || text.trim() != text
        || text.chars().any(char::is_control)
        || text.contains([':', '∈', '?'])
        || text.split(' ').any(str::is_empty)
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

pub(super) fn role_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<RoleName>, ParseError> {
    if !is_local_name(text) {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected role name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: RoleName(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

pub(super) fn variable_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<VariableName>, ParseError> {
    if !is_local_name(text) {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected variable name, found '{text}'"),
        ));
    }
    Ok(Spanned {
        value: VariableName(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

pub(super) fn domain_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<DomainName>, ParseError> {
    let name = semantic_name(line, offset, text)?;
    Ok(Spanned {
        value: DomainName(name.value.0),
        span: name.span,
    })
}

pub(super) fn referent_name(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<Spanned<Name>, ParseError> {
    if text.is_empty()
        || text.starts_with(' ')
        || text.ends_with(' ')
        || text.split(' ').any(|part| {
            part.is_empty()
                || !part.chars().all(|character| {
                    character.is_ascii_alphanumeric()
                        || character == '_'
                        || character == '-'
                        || character == '/'
                })
        })
    {
        return Err(error(
            child_span(line, offset, text.len()),
            format!("expected bracketed referent name, found '[{text}]'"),
        ));
    }
    Ok(Spanned {
        value: Name(text.to_owned()),
        span: child_span(line, offset, text.len()),
    })
}

pub(super) fn integer_range(
    line: SourceLine<'_>,
    offset: usize,
    text: &str,
) -> Result<IntegerRange, ParseError> {
    let (start, end) = text.split_once("..").ok_or_else(|| {
        error(
            child_span(line, offset, text.len()),
            "expected inclusive integer range 'start..end'",
        )
    })?;
    if start.is_empty() || end.is_empty() || end.contains("..") {
        return Err(error(
            child_span(line, offset, text.len()),
            "expected inclusive integer range 'start..end'",
        ));
    }
    let start = start.parse::<u64>().map_err(|_| {
        error(
            child_span(line, offset, text.len()),
            "range bounds must be unsigned integers",
        )
    })?;
    let end = end.parse::<u64>().map_err(|_| {
        error(
            child_span(line, offset, text.len()),
            "range bounds must be unsigned integers",
        )
    })?;
    if start > end {
        return Err(error(
            child_span(line, offset, text.len()),
            "range must be nonempty and ascending",
        ));
    }
    Ok(IntegerRange {
        start,
        end,
        span: child_span(line, offset, text.len()),
    })
}

fn kind(text: &str) -> Option<Kind> {
    match text {
        "RelationShape" => Some(Kind::RelationShape),
        "DerivationRule" => Some(Kind::DerivationRule),
        "Revision" => Some(Kind::Revision),
        "Delta" => Some(Kind::Delta),
        _ => None,
    }
}

fn take_body<'a>(lines: &[SourceLine<'a>], index: &mut usize) -> Vec<SourceLine<'a>> {
    let mut body = Vec::new();
    while *index < lines.len() {
        let line = lines[*index];
        if !line.text.is_empty() && indent(line).expect("validated source lines") == 0 {
            break;
        }
        body.push(line);
        *index += 1;
    }
    body
}

pub(super) fn nonblank<'a>(lines: impl IntoIterator<Item = SourceLine<'a>>) -> Vec<SourceLine<'a>> {
    lines
        .into_iter()
        .filter(|line| !line.text.is_empty())
        .collect()
}

fn read_clause_block<'a>(
    lines: &[SourceLine<'a>],
    index: &mut usize,
    description: &str,
) -> Result<SourceLine<'a>, ParseError> {
    let body = take_body(lines, index);
    let entries = nonblank(body);
    if entries.len() != 1
        || entries.first().is_some_and(|line| {
            indent(*line).expect("source indentation was validated before parsing") != 2
        })
    {
        let span = entries.first().map_or(
            Span {
                line: 1,
                column: 1,
                width: 0,
            },
            |line| line_span(*line),
        );
        return Err(error(
            span,
            format!("{description} requires exactly one two-space clause"),
        ));
    }
    Ok(entries[0])
}

fn parse_declaration<'a>(
    line: SourceLine<'a>,
    lines: &[SourceLine<'a>],
    index: &mut usize,
) -> Result<RawItem<'a>, ParseError> {
    let text = content(line);
    *index += 1;
    let body = take_body(lines, index);
    let entries = nonblank(body.iter().copied());
    if text.ends_with(" if") {
        if entries.is_empty()
            || entries.iter().any(|premise| {
                indent(*premise).expect("source indentation was validated before parsing") != 2
            })
        {
            return Err(error(
                line_span(line),
                "derivation rule requires one or more two-space premises after if",
            ));
        }
        return Ok(RawItem::Rule(RawRule {
            label: None,
            conclusion: SourceLine {
                number: line.number,
                column: line.column,
                text: &line.text[..line.text.len() - " if".len()],
            },
            premises: entries,
            header: line,
        }));
    }
    if let Some(label) = text.strip_suffix(':')
        && entries
            .first()
            .is_some_and(|conclusion| content(*conclusion).ends_with(" if"))
    {
        let conclusion = entries[0];
        if indent(conclusion)? != 2
            || entries.len() < 2
            || entries[1..].iter().any(|premise| {
                indent(*premise).expect("source indentation was validated before parsing") != 4
            })
        {
            return Err(error(
                line_span(line),
                "labelled derivation rule requires one two-space conclusion and one or more four-space premises",
            ));
        }
        let label = semantic_name(line, 0, label)?;
        if label.value.as_str().contains('/') {
            return Err(error(
                label.span,
                "derivation rule label must be unqualified",
            ));
        }
        return Ok(RawItem::Rule(RawRule {
            label: Some(label),
            conclusion: SourceLine {
                number: conclusion.number,
                column: conclusion.column,
                text: &conclusion.text[..conclusion.text.len() - " if".len()],
            },
            premises: entries[1..].to_vec(),
            header: line,
        }));
    }
    if entries.is_empty() && super::clause::clause_has_hole(line)? {
        return Ok(RawItem::TopLevel(RawTopLevel { line }));
    }
    if text.contains(" ∈ ") {
        if entries.is_empty() {
            return Ok(RawItem::TopLevel(RawTopLevel { line }));
        }
        return Err(error(
            line_span(line),
            "top-level membership cannot have an indented body",
        ));
    }
    let (subject, declaration_kind, bare_block) =
        if let Some((subject, kind_text)) = text.split_once(": ") {
            if matches!(kind_text, "Type" | "Model") {
                return Err(error(
                    line_span(line),
                    format!(
                        "'{kind_text}' is inferred from bare form and must not be declared with ':'"
                    ),
                ));
            }
            let Some(declaration_kind) = kind(kind_text) else {
                if nonblank(body.iter().copied()).is_empty() {
                    return Ok(RawItem::TopLevel(RawTopLevel { line }));
                }
                return Err(error(line_span(line), "unknown declaration kind"));
            };
            (qname(line, 0, subject)?, declaration_kind, false)
        } else if let Some(subject) = text.strip_suffix(':') {
            if entries.is_empty() {
                return Err(error(
                    line_span(line),
                    "binding block requires an indented body",
                ));
            }
            (semantic_name(line, 0, subject)?, Kind::Model, true)
        } else {
            let subject = semantic_name(line, 0, text)?;
            if entries.is_empty() {
                (subject, Kind::Grounding, false)
            } else {
                (subject, Kind::Model, true)
            }
        };
    Ok(RawItem::Declaration(RawDecl {
        subject,
        kind: declaration_kind,
        bare_block,
        header: line,
        body,
    }))
}

fn parse_request<'a>(
    line: SourceLine<'a>,
    lines: &[SourceLine<'a>],
    index: &mut usize,
) -> Result<RawRequest<'a>, ParseError> {
    let text = content(line);
    *index += 1;
    if let Some(projected) = text.strip_prefix("select ?") {
        let projected = variable_name(line, "select ?".len(), projected)?;
        let clause = read_clause_block(lines, index, "select request")?;
        return Ok(RawRequest::Select {
            projected,
            clause,
            header: line,
        });
    }
    if let Some(clause) = text.strip_prefix("any ") {
        if clause.is_empty() {
            return Err(error(line_span(line), "any requires one relational clause"));
        }
        return Ok(RawRequest::Any {
            clause: SourceLine {
                number: line.number,
                column: line.column + "any ".len(),
                text: clause,
            },
            header: line,
        });
    }
    if let Some(rest) = text.strip_prefix("find all ?") {
        let (sought_text, revision_text) = rest
            .split_once(" in ")
            .ok_or_else(|| error(line_span(line), "expected 'find all ?name in revision:'"))?;
        let revision_text = revision_text
            .strip_suffix(':')
            .ok_or_else(|| error(line_span(line), "find request requires ':'"))?;
        let sought = variable_name(line, "find all ?".len(), sought_text)?;
        let revision = qname(line, text.len() - revision_text.len() - 1, revision_text)?;
        let clause = read_clause_block(lines, index, "find request")?;
        return Ok(RawRequest::Find {
            revision,
            sought,
            clause,
            header: line,
        });
    }
    let (all, revision_text) = if let Some(rest) = text.strip_prefix("why all in ") {
        (true, rest)
    } else if let Some(rest) = text.strip_prefix("why in ") {
        (false, rest)
    } else {
        (false, "")
    };
    if !revision_text.is_empty() {
        let revision_text = revision_text
            .strip_suffix(':')
            .ok_or_else(|| error(line_span(line), "expected 'why [all] in revision:'"))?;
        let revision = qname(line, text.len() - revision_text.len() - 1, revision_text)?;
        let clause = read_clause_block(lines, index, "why request")?;
        return Ok(RawRequest::Why {
            revision,
            all,
            clause,
            header: line,
        });
    }
    for verb in ["prevent", "achieve"] {
        if let Some(rest) = text.strip_prefix(&format!("{verb} ")) {
            let (selection, revision_text) =
                if let Some(rest) = rest.strip_prefix("one minimal in ") {
                    (InterventionSelection::OneMinimal, rest)
                } else if let Some(rest) = rest.strip_prefix("all minimal in ") {
                    (InterventionSelection::AllMinimal, rest)
                } else {
                    return Err(error(
                        line_span(line),
                        "expected 'one minimal' or 'all minimal'",
                    ));
                };
            let revision_text = revision_text
                .strip_suffix(':')
                .ok_or_else(|| error(line_span(line), "intervention request requires ':'"))?;
            let revision = qname(line, text.len() - revision_text.len() - 1, revision_text)?;
            let clause = read_clause_block(lines, index, "intervention request")?;
            while *index < lines.len() && lines[*index].text.is_empty() {
                *index += 1;
            }
            let using_header = lines
                .get(*index)
                .copied()
                .ok_or_else(|| error(line_span(line), "intervention request requires 'using:'"))?;
            if indent(using_header)? != 0 || content(using_header) != "using:" {
                return Err(error(
                    line_span(using_header),
                    "intervention request requires 'using:'",
                ));
            }
            *index += 1;
            let using_lines = nonblank(take_body(lines, index));
            if using_lines.is_empty()
                || using_lines.iter().any(|entry| {
                    indent(*entry).expect("source indentation was validated before parsing") != 2
                })
            {
                return Err(error(
                    line_span(using_header),
                    "using requires one or more two-space relation references",
                ));
            }
            let using = using_lines
                .into_iter()
                .map(|entry| qname(entry, 2, content(entry)))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(RawRequest::Intervention {
                verb,
                revision,
                selection,
                clause,
                using,
                header: line,
            });
        }
    }
    if let Some(rest) = text.strip_prefix("diff ") {
        let (base, successor) = rest
            .split_once(" -> ")
            .ok_or_else(|| error(line_span(line), "expected 'diff base -> successor'"))?;
        return Ok(RawRequest::Diff {
            base: qname(line, "diff ".len(), base)?,
            successor: qname(line, "diff ".len() + base.len() + " -> ".len(), successor)?,
            header: line,
        });
    }
    Err(error(line_span(line), "unknown request"))
}

pub(super) fn scan(source: &str) -> Result<ScanOutput<'_>, ParseError> {
    let lines = source
        .split('\n')
        .enumerate()
        .map(|(index, text)| SourceLine {
            number: index + 1,
            column: 1,
            text,
        })
        .collect::<Vec<_>>();
    for line in &lines {
        indent(*line)?;
    }
    let mut declarations = Vec::new();
    let mut top_level = Vec::new();
    let mut rules = Vec::new();
    let mut requests = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if line.text.is_empty() {
            index += 1;
            continue;
        }
        if indent(line)? != 0 {
            return Err(error(
                line_span(line),
                "unexpected indentation at top level",
            ));
        }
        let text = content(line);
        if text.starts_with("select ")
            || text.starts_with("any ")
            || text.starts_with("find ")
            || text.starts_with("why in ")
            || text.starts_with("why all in ")
            || text.starts_with("prevent ")
            || text.starts_with("achieve ")
            || text.starts_with("diff ")
        {
            requests.push(parse_request(line, &lines, &mut index)?);
        } else {
            match parse_declaration(line, &lines, &mut index)? {
                RawItem::Declaration(declaration) => declarations.push(declaration),
                RawItem::TopLevel(fragment) => top_level.push(fragment),
                RawItem::Rule(rule) => rules.push(rule),
            }
        }
    }
    Ok((declarations, top_level, rules, requests))
}
