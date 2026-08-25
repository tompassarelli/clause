use super::{ParseError, Span};

/// One source decision made by the legacy-to-canonical migration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationInference {
    pub span: Span,
    pub before: String,
    pub after: String,
}

/// Deterministic canonical source and the complete ordered rewrite report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Migration {
    pub source: String,
    pub inferences: Vec<MigrationInference>,
}

/// Rewrite legacy Revision ceremony to exact ancestry and signed clauses.
///
/// Parsing both projections remains the authority for semantic validity. This
/// pass changes only syntax whose meaning is explicit in the legacy form and
/// reports every changed line in source order.
pub fn migrate(source: &str) -> Result<Migration, ParseError> {
    super::parse(source)?;
    let lines = source.split('\n').collect::<Vec<_>>();
    let mut output = Vec::with_capacity(lines.len());
    let mut inferences = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        if let Some(subject) = line.strip_suffix(": DerivationRule") {
            let start = index;
            let mut end = index + 1;
            while end < lines.len() && (lines[end].is_empty() || lines[end].starts_with(' ')) {
                end += 1;
            }
            let body = &lines[index + 1..end];
            let conclusion_index = body
                .iter()
                .position(|line| line.starts_with("  ") && !line.starts_with("    "))
                .expect("validated DerivationRule conclusion");
            let when_index = body
                .iter()
                .position(|line| *line == "  when:")
                .expect("validated DerivationRule when block");
            let header = format!("{subject}:");
            output.push(header.clone());
            inferences.push(MigrationInference {
                span: Span {
                    line: start + 1,
                    column: 1,
                    width: line.len(),
                },
                before: line.to_owned(),
                after: header,
            });
            for (offset, member) in body.iter().enumerate() {
                if offset == conclusion_index {
                    let rewritten = format!("{member} if");
                    output.push(rewritten.clone());
                    inferences.push(MigrationInference {
                        span: Span {
                            line: start + offset + 2,
                            column: 1,
                            width: member.len(),
                        },
                        before: (*member).to_owned(),
                        after: rewritten,
                    });
                } else if offset == when_index {
                    inferences.push(MigrationInference {
                        span: Span {
                            line: start + offset + 2,
                            column: 1,
                            width: member.len(),
                        },
                        before: (*member).to_owned(),
                        after: String::new(),
                    });
                } else {
                    output.push((*member).to_owned());
                }
            }
            index = end;
            continue;
        }
        let Some(subject) = line.strip_suffix(": Revision") else {
            output.push(line.to_owned());
            index += 1;
            continue;
        };
        let start = index;
        let mut end = index + 1;
        while end < lines.len() && (lines[end].is_empty() || lines[end].starts_with(' ')) {
            end += 1;
        }
        let body = &lines[index + 1..end];
        let Some(from_line) = body.iter().find(|line| line.starts_with("  from: ")) else {
            output.push(line.to_owned());
            index += 1;
            continue;
        };
        if body.iter().any(|line| line.starts_with("  apply: ")) {
            output.extend(lines[start..end].iter().map(|line| (*line).to_owned()));
            index = end;
            continue;
        }
        let base = &from_line["  from: ".len()..];
        let header = format!("{subject} from {base}");
        output.push(header.clone());
        inferences.push(MigrationInference {
            span: Span {
                line: start + 1,
                column: 1,
                width: line.len(),
            },
            before: line.to_owned(),
            after: header,
        });
        let mut sign = None;
        for (offset, member) in body.iter().enumerate() {
            match *member {
                value if value.starts_with("  from: ") => {
                    inferences.push(MigrationInference {
                        span: Span {
                            line: start + offset + 2,
                            column: 1,
                            width: value.len(),
                        },
                        before: value.to_owned(),
                        after: String::new(),
                    });
                }
                value @ "  admit:" => {
                    sign = Some('+');
                    inferences.push(MigrationInference {
                        span: Span {
                            line: start + offset + 2,
                            column: 1,
                            width: value.len(),
                        },
                        before: value.to_owned(),
                        after: String::new(),
                    });
                }
                value @ "  withdraw:" => {
                    sign = Some('-');
                    inferences.push(MigrationInference {
                        span: Span {
                            line: start + offset + 2,
                            column: 1,
                            width: value.len(),
                        },
                        before: value.to_owned(),
                        after: String::new(),
                    });
                }
                "" => output.push(String::new()),
                value if value.starts_with("    ") && sign.is_some() => {
                    let rewritten = format!("  {} {}", sign.expect("checked sign"), &value[4..]);
                    output.push(rewritten.clone());
                    inferences.push(MigrationInference {
                        span: Span {
                            line: start + offset + 2,
                            column: 1,
                            width: value.len(),
                        },
                        before: value.to_owned(),
                        after: rewritten,
                    });
                }
                _ => unreachable!("validated legacy Revision layout"),
            }
        }
        index = end;
    }
    let mut canonical = output.join("\n");
    if source.ends_with('\n') && !canonical.ends_with('\n') {
        canonical.push('\n');
    }
    super::parse(&canonical)?;
    Ok(Migration {
        source: canonical,
        inferences,
    })
}
