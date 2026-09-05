//! Focused clauses share the fact reader's edge grammar. The surrounding
//! section supplies stance; focus neither asserts a fact nor binds a new name.
use super::*;

fn subject(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    if let Some(name) = source.strip_prefix('?') {
        denotation_designation_bytes(name, origin)?;
        Ok(source.as_bytes().to_vec())
    } else {
        application_designation_bytes(source, origin)
    }
}

fn clauses(
    lines: &[LogicalSourceLine],
    indent: usize,
) -> Result<Vec<LogicalSourceLine>, CanonicalSourceErrorV1> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < lines.len() {
        let head = &lines[cursor];
        if head.indent != indent {
            return Err(CanonicalSourceErrorV1::UnexpectedIndentation {
                origin: head.origin,
            });
        }
        let mut end = cursor + 1;
        while end < lines.len() && lines[end].indent > indent {
            end += 1;
        }
        if end == cursor + 1 {
            if head.text.ends_with(':') {
                return Err(CanonicalSourceErrorV1::InvalidApplication {
                    origin: head.origin,
                });
            }
            result.push(head.clone());
        } else {
            let designation = head.text.strip_suffix(':').unwrap_or(&head.text);
            let focus = subject(designation, head.origin)?;
            for edge in parse_focused_edges(&lines[cursor + 1..end], indent, &focus, subject)? {
                let source = std::str::from_utf8(&edge.source)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
                let focused = if let Some((role, object)) = source.split_once(": ") {
                    application_role_bytes(role, edge.origin)?;
                    if object.is_empty() {
                        return Err(CanonicalSourceErrorV1::InvalidApplication {
                            origin: edge.origin,
                        });
                    }
                    format!("{role} {object}")
                } else {
                    source.to_owned()
                };
                let subject = std::str::from_utf8(&edge.subject)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
                result.push(LogicalSourceLine {
                    text: format!("{subject} {focused}"),
                    origin: edge.origin,
                    indent,
                });
            }
        }
        cursor = end;
    }
    Ok(result)
}

pub(super) fn handler_lines(
    lines: Vec<LogicalSourceLine>,
) -> Result<Vec<LogicalSourceLine>, CanonicalSourceErrorV1> {
    let lines = lines
        .into_iter()
        .filter(|line| !line.text.is_empty())
        .collect::<Vec<_>>();
    let Some(header) = lines.first() else {
        return Ok(lines);
    };
    let mut result = vec![header.clone()];
    let mut cursor = 1;
    while cursor < lines.len() {
        let section = &lines[cursor];
        if section.indent != 2 {
            return Err(CanonicalSourceErrorV1::UnexpectedIndentation {
                origin: section.origin,
            });
        }
        let mut end = cursor + 1;
        while end < lines.len() && lines[end].indent > 2 {
            end += 1;
        }
        result.push(section.clone());
        let body = &lines[cursor + 1..end];
        if matches!(
            section.text.as_str(),
            "if" | "then" | "when" | "withdraw" | "include" | "accumulate"
        ) {
            result.extend(clauses(body, 4)?);
        } else {
            result.extend_from_slice(body);
        }
        cursor = end;
    }
    Ok(result)
}
