//! Contextual field trees share declared edge syntax with ordinary clauses.
//! The role range selects their contract; no source constructor or field
//! Referent is invented during construction or matching.
use super::*;

pub(super) fn range<'a>(
    environment: &'a ScalarLawEnvironment,
    role: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<&'a [u8]>, CanonicalSourceErrorV1> {
    let mut shapes = Vec::new();
    for relation in environment
        .relations
        .iter()
        .filter(|relation| relation.surface == role)
    {
        let Some(subject) = &relation.subject else {
            continue;
        };
        let others = relation
            .roles
            .iter()
            .filter(|role| &role.name != subject)
            .collect::<Vec<_>>();
        let [value] = others.as_slice() else { continue };
        if fields(environment, &value.domain).is_some() {
            shapes.push(value.domain.as_slice());
        }
    }
    match shapes.as_slice() {
        [] => Ok(None),
        [shape] => Ok(Some(shape)),
        _ => Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin }),
    }
}

fn fields<'a>(environment: &'a ScalarLawEnvironment, shape: &[u8]) -> Option<&'a [ShapeField]> {
    environment
        .declarations
        .iter()
        .find_map(|item| match &item.kind {
            CstKind::Shape {
                designation,
                fields,
            } if designation == shape => Some(fields.as_slice()),
            _ => None,
        })
}

pub(super) fn read(
    environment: &ScalarLawEnvironment,
    shape: &[u8],
    lines: &[LogicalSourceLine],
    indent: usize,
    frontend: &CanonicalDeclaredFrontendV1,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalFocusedObjectV1, CanonicalSourceErrorV1> {
    let declared =
        fields(environment, shape).ok_or(CanonicalSourceErrorV1::InvalidShapeField { origin })?;
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < lines.len() {
        let line = &lines[cursor];
        if line.indent != indent {
            return Err(CanonicalSourceErrorV1::UnexpectedIndentation {
                origin: line.origin,
            });
        }
        let mut end = cursor + 1;
        while end < lines.len() && lines[end].indent > indent {
            end += 1;
        }
        let (name, value) = if end == cursor + 1 {
            let edge = frontend.edge(b"", &line.text, line.origin)?;
            (edge.relation, edge.object)
        } else {
            let (_, name, _) = frontend.prefix(&line.text, line.origin)?;
            let field = declared.iter().find(|field| field.name == name).ok_or(
                CanonicalSourceErrorV1::InvalidShapeField {
                    origin: line.origin,
                },
            )?;
            let value = read(
                environment,
                &field.domain,
                &lines[cursor + 1..end],
                indent + 2,
                frontend,
                line.origin,
            )?;
            (name, value)
        };
        if declared
            .get(result.len())
            .is_none_or(|field| field.name != name)
        {
            return Err(CanonicalSourceErrorV1::InvalidShapeField {
                origin: line.origin,
            });
        }
        result.push(CanonicalFocusedFieldV1 {
            name,
            value,
            source: line.text.as_bytes().to_vec(),
            origin: line.origin,
        });
        cursor = end;
    }
    if result.len() != declared.len() {
        return Err(CanonicalSourceErrorV1::InvalidShapeField { origin });
    }
    Ok(CanonicalFocusedObjectV1::Fields {
        shape: shape.to_vec(),
        fields: result,
    })
}

pub(super) fn assertion(
    edge: &CanonicalFocusedEdgeV1,
) -> Result<Option<ShapeAssertionCst>, CanonicalSourceErrorV1> {
    let CanonicalFocusedObjectV1::Fields { shape, fields } = &edge.object else {
        return Ok(None);
    };
    let fields = fields
        .iter()
        .map(|field| {
            let source = std::str::from_utf8(field.value.source(field.origin)?)
                .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
            Ok(ShapeAssertionFieldCst {
                name: field.name.clone(),
                value: parse_application_object(source, field.origin)?,
            })
        })
        .collect::<Result<_, CanonicalSourceErrorV1>>()?;
    Ok(Some(ShapeAssertionCst {
        origin: edge.origin,
        subject: edge.subject.clone(),
        relation: edge.relation.clone(),
        shape: shape.clone(),
        fields,
    }))
}

// A stable, origin-independent emission designation, not input to a source
// reader. Structured clauses reach checking and lowering through their CST.
pub(super) fn designation(edge: &CanonicalFocusedEdgeV1) -> String {
    fn object(value: &CanonicalFocusedObjectV1, output: &mut String, indent: usize) {
        match value {
            CanonicalFocusedObjectV1::Source(source) => {
                output.push_str(&String::from_utf8_lossy(source))
            }
            CanonicalFocusedObjectV1::Fields { shape, fields } => {
                output.push_str(&String::from_utf8_lossy(shape));
                for field in fields {
                    output.push('\n');
                    output.push_str(&" ".repeat(indent));
                    output.push_str(&String::from_utf8_lossy(&field.name));
                    output.push_str(": ");
                    object(&field.value, output, indent + 2);
                }
            }
        }
    }
    let mut output = format!(
        "{} {} ",
        String::from_utf8_lossy(&edge.subject),
        String::from_utf8_lossy(&edge.relation)
    );
    object(&edge.object, &mut output, 2);
    output
}

pub(super) fn premise_tokens(
    line: &LogicalSourceLine,
) -> Result<Vec<Vec<u8>>, CanonicalSourceErrorV1> {
    fn append(source: &[u8], tokens: &mut Vec<Vec<u8>>) -> Result<(), CanonicalSourceErrorV1> {
        let text = std::str::from_utf8(source).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
        tokens.extend(
            declared_frontend::input_tokens(text)?
                .into_iter()
                .map(|token| source[token.start..token.end].to_vec()),
        );
        Ok(())
    }
    fn fields(
        value: &CanonicalFocusedObjectV1,
        tokens: &mut Vec<Vec<u8>>,
    ) -> Result<(), CanonicalSourceErrorV1> {
        if let CanonicalFocusedObjectV1::Fields {
            fields: children, ..
        } = value
        {
            for field in children {
                append(&field.source, tokens)?;
                fields(&field.value, tokens)?;
            }
        }
        Ok(())
    }
    let mut tokens = Vec::new();
    if let Some(edge) = &line.structured {
        append(&edge.subject, &mut tokens)?;
        append(&edge.source, &mut tokens)?;
        fields(&edge.object, &mut tokens)?;
    } else {
        append(line.text.as_bytes(), &mut tokens)?;
    }
    Ok(tokens)
}

fn components(edge: &CanonicalFocusedEdgeV1) -> Option<Vec<(ScalarParameterSourceCst, &str)>> {
    let CanonicalFocusedObjectV1::Fields { shape, fields } = &edge.object else {
        return None;
    };
    fields
        .iter()
        .map(|field| {
            Some((
                ScalarParameterSourceCst {
                    parameter: Vec::new(),
                    subject: edge.subject.clone(),
                    relation: edge.relation.clone(),
                    shape: Some(shape.clone()),
                    field: Some(field.name.clone()),
                },
                std::str::from_utf8(field.value.source(field.origin).ok()?).ok()?,
            ))
        })
        .collect()
}

pub(super) fn state_declaration(
    line: &LogicalSourceLine,
    subject: &str,
) -> Option<Vec<ScalarParameterSourceCst>> {
    let Some(edge) = &line.structured else {
        return parse_general_state_declaration(&line.text, subject);
    };
    components(edge)?
        .into_iter()
        .map(|(mut target, value)| {
            match parse_scalar_expression(value, "")? {
                CanonicalScalarExpressionV1::Parameter(parameter) => target.parameter = parameter,
                CanonicalScalarExpressionV1::Number(_)
                | CanonicalScalarExpressionV1::Boolean(_)
                | CanonicalScalarExpressionV1::Text(_)
                | CanonicalScalarExpressionV1::Symbol(_) => {}
                _ => return None,
            }
            Some(target)
        })
        .collect()
}

pub(super) fn selectors(line: &LogicalSourceLine) -> Option<Vec<ScalarStateSelectorCst>> {
    let edge = line.structured.as_ref()?;
    components(edge)?
        .into_iter()
        .filter(|(_, value)| !value.starts_with('?'))
        .map(|(source, value)| {
            Some(ScalarStateSelectorCst {
                origin: line.origin,
                source,
                expected: parse_application_object(value, line.origin).ok()?,
            })
        })
        .collect()
}

pub(super) fn insertion(
    line: &LogicalSourceLine,
    subject: &str,
) -> Option<Vec<GeneralAssignmentCst>> {
    let Some(edge) = &line.structured else {
        return parse_general_insertion(&line.text, subject);
    };
    components(edge)?
        .into_iter()
        .map(|(target, value)| {
            Some(GeneralAssignmentCst {
                target,
                value: parse_scalar_expression(value, "")?,
            })
        })
        .collect()
}

pub(super) fn replacement(
    withdraw: &LogicalSourceLine,
    include: &LogicalSourceLine,
    subject: &str,
) -> Option<GeneralReplacementCst> {
    if withdraw.structured.is_none() && include.structured.is_none() {
        return parse_general_assignments(&withdraw.text, &include.text, subject);
    }
    let targets = state_declaration(withdraw, subject)?;
    let mut result = GeneralReplacementCst {
        assignments: vec![],
        aggregate_binding: None,
        aggregate_result: None,
        required_sources: vec![],
    };
    let next = insertion(include, subject)?;
    match (&withdraw.structured, &include.structured) {
        _ if targets.iter().all(|target| target.field.is_some())
            && next.iter().all(|assignment| assignment.target.field.is_some()) => {
            let current = insertion(withdraw, subject)?;
            if current.len() != next.len() {
                return None;
            }
            for ((target, old), new) in targets.into_iter().zip(current).zip(next) {
                if target.subject != new.target.subject
                    || target.relation != new.target.relation
                    || target.shape != new.target.shape
                    || target.field != new.target.field
                {
                    return None;
                }
                if old.value != new.value {
                    result.assignments.push(GeneralAssignmentCst {
                        target,
                        value: new.value,
                    });
                }
            }
        }
        (Some(old), None) => {
            let [new]: [ScalarParameterSourceCst; 1] =
                parse_general_state_declaration(&include.text, subject)?
                    .try_into()
                    .ok()?;
            if new.subject != old.subject || new.relation != old.relation {
                return None;
            }
            let CanonicalFocusedObjectV1::Fields { shape, fields } = &old.object else {
                return None;
            };
            let fields = fields
                .iter()
                .map(|field| {
                    (
                        field.name.clone(),
                        format!(
                            "\0general-aggregate-result-{}",
                            String::from_utf8_lossy(&field.name)
                        )
                        .into_bytes(),
                    )
                })
                .collect::<Vec<_>>();
            result.assignments = targets
                .into_iter()
                .zip(&fields)
                .map(|(target, (_, parameter))| GeneralAssignmentCst {
                    target,
                    value: CanonicalScalarExpressionV1::Parameter(parameter.clone()),
                })
                .collect();
            result.aggregate_result = Some(GeneralAggregateResultCst {
                parameter: new.parameter,
                shape: shape.clone(),
                fields,
            });
        }
        (None, Some(new)) => {
            let [old]: [ScalarParameterSourceCst; 1] = targets.try_into().ok()?;
            if old.subject != new.subject || old.relation != new.relation {
                return None;
            }
            result.assignments = insertion(include, subject)?;
            result.required_sources = result
                .assignments
                .iter()
                .map(|a| a.target.clone())
                .collect();
            result.aggregate_binding = Some(old.parameter);
        }
        _ => return None,
    }
    Some(result)
}
