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

/// The delimited edge head selects this grammar before children are read.
/// Each subject token owns an emission, including repeated equal subjects.
pub(super) fn edge_focus(
    lines: &[LogicalSourceLine],
    frontend: &CanonicalDeclaredFrontendV1,
) -> Result<Option<Vec<CanonicalFocusedEdgeV1>>, CanonicalSourceErrorV1> {
    let Some(head) = lines.first() else { return Ok(None) };
    let Some(source) = head.text.strip_prefix('(').and_then(|s| s.strip_suffix("):")) else {
        return Ok(None);
    };
    let edge = frontend.edge(b"", source, head.origin)?;
    let mut edges = Vec::new();
    for line in &lines[1..] {
        if line.indent != head.indent + 2 {
            return Err(CanonicalSourceErrorV1::UnexpectedIndentation { origin: line.origin });
        }
        for token in declared_frontend::input_tokens(&line.text)? {
            let origin = CanonicalSourceOriginV1 {
                artifact: line.origin.artifact,
                start: line.origin.start + line.indent as u64 + token.start as u64,
                end: line.origin.start + line.indent as u64 + token.end as u64,
            };
            let mut emission = edge.clone();
            emission.subject = subject(&line.text[token.start..token.end], origin)?;
            emission.origin = origin;
            edges.push(emission);
        }
    }
    if edges.is_empty() {
        return Err(CanonicalSourceErrorV1::InvalidApplication { origin: head.origin });
    }
    Ok(Some(edges))
}

pub(super) fn clauses(
    lines: &[LogicalSourceLine],
    indent: usize,
    frontend: &CanonicalDeclaredFrontendV1,
    environment: &ScalarLawEnvironment,
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
        if let Some(edges) = edge_focus(&lines[cursor..end], frontend)? {
            for edge in edges {
                result.push(LogicalSourceLine {
                    text: format!("{} {} {}",
                        std::str::from_utf8(&edge.subject).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                        std::str::from_utf8(&edge.relation).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                        std::str::from_utf8(edge.object.source(edge.origin)?).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                    ),
                    origin: edge.origin,
                    indent,
                    structured: None,
                });
            }
        } else if end == cursor + 1 {
            if head.text.ends_with(':') {
                return Err(CanonicalSourceErrorV1::InvalidApplication {
                    origin: head.origin,
                });
            }
            result.push(head.clone());
        } else {
            let designation = head.text.strip_suffix(':').unwrap_or(&head.text);
            let focus = subject(designation, head.origin)?;
            for edge in parse_focused_edges(
                &lines[cursor + 1..end],
                indent,
                &focus,
                subject,
                frontend,
                environment,
            )? {
                if matches!(edge.object, CanonicalFocusedObjectV1::Fields { .. }) {
                    result.push(LogicalSourceLine {
                        text: structured_values::designation(&edge),
                        origin: edge.origin,
                        indent,
                        structured: Some(edge),
                    });
                    continue;
                }
                let relation = std::str::from_utf8(&edge.relation)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
                let object = std::str::from_utf8(edge.object.source(edge.origin)?)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
                let subject = std::str::from_utf8(&edge.subject)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
                result.push(LogicalSourceLine {
                    text: format!("{subject} {relation} {object}"),
                    origin: edge.origin,
                    indent,
                    structured: None,
                });
            }
        }
        cursor = end;
    }
    Ok(result)
}

pub(super) fn handler_lines(
    lines: Vec<LogicalSourceLine>,
    frontend: &CanonicalDeclaredFrontendV1,
    environment: &ScalarLawEnvironment,
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
            result.extend(clauses(body, 4, frontend, environment)?);
        } else {
            result.extend_from_slice(body);
        }
        cursor = end;
    }
    Ok(result)
}
