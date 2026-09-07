use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCallableArgumentV1 {
    pub designation: Vec<u8>,
    pub value_kind: CanonicalScalarValueKindV1,
}

/// One pure, deterministic, single-result direction of a named relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCallableV1 {
    pub designation: Vec<u8>,
    pub exported: bool,
    pub arguments: Vec<CanonicalCallableArgumentV1>,
    pub result_kind: CanonicalScalarValueKindV1,
    pub expression: CanonicalExecutableExpressionV1,
    pub origin: CanonicalSourceOriginV1,
}

pub(super) fn productions(
    callable: &CanonicalCallableV1,
) -> impl Iterator<Item = CanonicalSourceProductionV1> {
    std::iter::once(CanonicalSourceProductionV1::CallableDefinition).chain(
        callable
            .exported
            .then_some(CanonicalSourceProductionV1::CallableExport),
    )
}

fn kind(name: &str) -> Option<CanonicalScalarValueKindV1> {
    Some(match name.trim() {
        "Text" => CanonicalScalarValueKindV1::Text,
        "F64" => CanonicalScalarValueKindV1::Number,
        "Bool" => CanonicalScalarValueKindV1::Boolean,
        _ => return None,
    })
}

pub(super) fn read(
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<(CanonicalCallableV1, RelationCst)>, CanonicalSourceErrorV1> {
    let head = block[0].text;
    let exported = head.starts_with("export ");
    let head = head.strip_prefix("export ").unwrap_or(head);
    if !exported && !(head.contains('(') && head.contains("):")) {
        return Ok(None);
    }
    let error = |reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason };
    let (name, signature) = head
        .split_once('(')
        .ok_or_else(|| error("expected a named typed callable"))?;
    let designation = denotation_designation_bytes(name.trim(), origin)?;
    let (parameters, result) = signature
        .split_once("):")
        .ok_or_else(|| error("expected a result type"))?;
    let result_kind = kind(result).ok_or_else(|| error("unsupported result type"))?;
    let mut arguments = Vec::new();
    let mut roles = Vec::new();
    let mut names = BTreeSet::new();
    for parameter in parameters.split(',').filter(|s| !s.trim().is_empty()) {
        let (name, domain) = parameter
            .trim()
            .split_once(':')
            .ok_or_else(|| error("expected a named typed argument"))?;
        let name = name
            .trim()
            .strip_prefix('?')
            .ok_or_else(|| error("argument bindings start with ?"))?;
        let name = denotation_designation_bytes(name, origin)?;
        if !names.insert(name.clone()) {
            return Err(error("duplicate argument binding"));
        }
        let value_kind = kind(domain).ok_or_else(|| error("unsupported argument type"))?;
        roles.push(RelationRoleCst {
            name: name.clone(),
            domain: domain.trim().as_bytes().to_vec(),
            origin,
        });
        arguments.push(CanonicalCallableArgumentV1 {
            designation: name,
            value_kind,
        });
    }
    if arguments.len() > usize::from(u16::MAX) {
        return Err(error("argument limit"));
    }
    // The produced role is compiler-owned and cannot collide with an author binder.
    let result_role = b"$result".to_vec();
    roles.push(RelationRoleCst {
        name: result_role.clone(),
        domain: result.trim().as_bytes().to_vec(),
        origin,
    });
    let body = block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
        .collect::<Vec<_>>();
    let [body] = body.as_slice() else {
        return Err(error("a pure callable requires one expression"));
    };
    let expression_origin = line_origin(origin.artifact, **body);
    let mut parser = ScalarExpressionParser {
        source: body.text.as_bytes(),
        cursor: 0,
        current: "",
        interpolate: true,
    };
    let expression = parser
        .comparison()
        .ok_or(CanonicalSourceErrorV1::InvalidCallable {
            origin: expression_origin,
            reason: "unsupported pure expression",
        })?;
    parser.skip_spaces();
    if parser.cursor != parser.source.len() {
        return Err(error("unsupported pure expression"));
    }
    let expression = lower(&expression, &arguments, expression_origin)?;
    let callable = CanonicalCallableV1 {
        designation: designation.clone(),
        exported,
        arguments,
        result_kind,
        expression,
        origin,
    };
    check_canonical_callable_v1(&callable)?;
    let known = callable
        .arguments
        .iter()
        .map(|a| a.designation.clone())
        .collect::<Vec<_>>();
    let mut canonical = b"pure given".to_vec();
    for name in &known {
        canonical.push(b' ');
        canonical.extend(name);
    }
    canonical.extend(b" yields $result: one");
    let reading = std::iter::once(RelationReadingPartCst::Literal(designation.clone()))
        .chain(
            roles
                .iter()
                .map(|r| RelationReadingPartCst::Role(r.name.clone())),
        )
        .collect();
    let relation = RelationCst {
        contract_origin: None,
        designation: designation.clone(),
        surface: designation,
        reading,
        subject: None,
        roles,
        modes: vec![RelationModeCst {
            known,
            produced: vec![result_role],
            cardinality: SourceCardinality::One,
            reactive_obligation: None,
            continues_linearly: false,
            effect: None,
            canonical,
            origin,
        }],
    };
    Ok(Some((callable, relation)))
}

/// Reads literal segments and embedded Clause expressions without reinterpreting
/// literal escape sequences as source. Existing non-callable literals stay literal.
pub(super) fn text_template(source: &str) -> Option<(CanonicalScalarExpressionV1, usize)> {
    use CanonicalScalarExpressionV1 as E;
    let bytes = source.as_bytes();
    (bytes.first() == Some(&b'"')).then_some(())?;
    let mut cursor = 1;
    let mut start = cursor;
    let mut parts = Vec::new();
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'\\' => {
                cursor += 2;
            }
            b'"' | b'{' => {
                let literal = format!("\"{}\"", source.get(start..cursor)?);
                parts.push(E::Text(parse_text_literal(&literal)?));
                if byte == b'"' {
                    let mut iter = parts.into_iter();
                    let result = iter.next()?;
                    return Some((
                        iter.fold(result, |a, b| E::Concatenate(Box::new(a), Box::new(b))),
                        cursor + 1,
                    ));
                }
                let expression_start = cursor + 1;
                cursor += 1;
                let mut quoted = false;
                let mut escaped = false;
                let mut depth = 1usize;
                while let Some(&byte) = bytes.get(cursor) {
                    if escaped {
                        escaped = false;
                    } else if quoted && byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        quoted = !quoted;
                    } else if !quoted && byte == b'{' {
                        depth += 1;
                    } else if !quoted && byte == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    cursor += 1;
                }
                if depth != 0 {
                    return None;
                }
                parts.push(parse_scalar_expression(
                    source.get(expression_start..cursor)?,
                    "",
                )?);
                cursor += 1;
                start = cursor;
            }
            _ => cursor += 1,
        }
    }
    None
}

fn lower(
    expression: &CanonicalScalarExpressionV1,
    arguments: &[CanonicalCallableArgumentV1],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalScalarExpressionV1 as S;
    let error = || CanonicalSourceErrorV1::InvalidCallable {
        origin,
        reason: "unresolved or unsupported pure expression",
    };
    let recur = |e| lower(e, arguments, origin).map(Box::new);
    Ok(match expression {
        S::Number(n) => E::Constant(CanonicalScalarValueV1::Number(*n)),
        S::Boolean(b) => E::Constant(CanonicalScalarValueV1::Boolean(*b)),
        S::Text(t) => E::Constant(CanonicalScalarValueV1::Text(t.clone())),
        S::Parameter(name) => E::Argument(
            u16::try_from(
                arguments
                    .iter()
                    .position(|a| name.strip_prefix(b"?") == Some(a.designation.as_slice()))
                    .ok_or_else(error)?,
            )
            .map_err(|_| error())?,
        ),
        S::ContainsText(a, b) => E::ContainsText(recur(a)?, recur(b)?),
        S::StartsWith(a, b) => E::StartsWith(recur(a)?, recur(b)?),
        S::Equal(a, b) => E::Equal(recur(a)?, recur(b)?),
        S::GreaterThan(a, b) => E::GreaterThan(recur(a)?, recur(b)?),
        S::LessThanOrEqual(a, b) => E::LessThanOrEqual(recur(a)?, recur(b)?),
        S::Concatenate(a, b) => E::Concatenate(recur(a)?, recur(b)?),
        S::Add(a, b) => E::Add(recur(a)?, recur(b)?),
        S::Subtract(a, b) => E::Subtract(recur(a)?, recur(b)?),
        S::Multiply(a, b) => E::Multiply(recur(a)?, recur(b)?),
        S::Divide(a, b) => E::Divide(recur(a)?, recur(b)?),
        S::SquareRoot(a) => E::SquareRoot(recur(a)?),
        S::TextTransform(op, a) => E::TextTransform(*op, recur(a)?),
        S::Conditional(a, b, c) => E::Conditional(recur(a)?, recur(b)?, recur(c)?),
        S::Current | S::Symbol(_) => return Err(error()),
    })
}

/// Checks the pure scalar subset and declared argument/result contracts.
/// Effects, state access and unresolved arguments reject even in hand-built IR.
pub fn check_canonical_callable_v1(
    callable: &CanonicalCallableV1,
) -> Result<(), CanonicalSourceErrorV1> {
    let error = |reason| CanonicalSourceErrorV1::InvalidCallable {
        origin: callable.origin,
        reason,
    };
    let supported = |kind| {
        matches!(
            kind,
            CanonicalScalarValueKindV1::Text
                | CanonicalScalarValueKindV1::Number
                | CanonicalScalarValueKindV1::Boolean
        )
    };
    let mut names = BTreeSet::new();
    if !supported(callable.result_kind)
        || callable
            .arguments
            .iter()
            .any(|a| !supported(a.value_kind) || !names.insert(&a.designation))
    {
        return Err(error("unsupported or duplicate argument/result contract"));
    }
    if expression_kind(&callable.expression, &callable.arguments, 0).map_err(error)?
        != callable.result_kind
    {
        return Err(error("callable result type mismatch"));
    }
    Ok(())
}

fn expression_kind(
    expression: &CanonicalExecutableExpressionV1,
    arguments: &[CanonicalCallableArgumentV1],
    depth: usize,
) -> Result<CanonicalScalarValueKindV1, &'static str> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalScalarValueKindV1 as K;
    if depth >= 64 {
        return Err("pure expression depth limit");
    }
    let recur = |e| expression_kind(e, arguments, depth + 1);
    let require = |e, kind| {
        if recur(e)? == kind {
            Ok(())
        } else {
            Err("pure expression type mismatch")
        }
    };
    Ok(match expression {
        E::Constant(CanonicalScalarValueV1::Number(_)) => K::Number,
        E::Constant(CanonicalScalarValueV1::Boolean(_)) => K::Boolean,
        E::Constant(CanonicalScalarValueV1::Text(_)) => K::Text,
        E::Argument(i) => {
            arguments
                .get(usize::from(*i))
                .ok_or("unresolved argument")?
                .value_kind
        }
        E::Concatenate(a, b) | E::ContainsText(a, b) | E::StartsWith(a, b) => {
            require(a, K::Text)?;
            require(b, K::Text)?;
            if matches!(expression, E::Concatenate(..)) {
                K::Text
            } else {
                K::Boolean
            }
        }
        E::TextTransform(_, a) => {
            require(a, K::Text)?;
            K::Text
        }
        E::Add(a, b)
        | E::Subtract(a, b)
        | E::Multiply(a, b)
        | E::Divide(a, b)
        | E::GreaterThan(a, b)
        | E::LessThanOrEqual(a, b) => {
            require(a, K::Number)?;
            require(b, K::Number)?;
            if matches!(expression, E::GreaterThan(..) | E::LessThanOrEqual(..)) {
                K::Boolean
            } else {
                K::Number
            }
        }
        E::SquareRoot(a) => {
            require(a, K::Number)?;
            K::Number
        }
        E::Equal(a, b) => {
            if recur(a)? != recur(b)? {
                return Err("equality operand type mismatch");
            }
            K::Boolean
        }
        E::Conditional(a, b, c) => {
            require(a, K::Boolean)?;
            let k = recur(b)?;
            require(c, k)?;
            k
        }
        _ => return Err("effects and state access are forbidden in a pure callable"),
    })
}
