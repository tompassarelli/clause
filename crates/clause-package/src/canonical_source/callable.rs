use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCallableArgumentV1 {
    pub designation: Vec<u8>,
    pub value_kind: CanonicalValueTypeV1,
}

/// One typed single-result direction, with a checked function/procedure allowance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCallableV1 {
    pub designation: Vec<u8>,
    pub exported: bool,
    pub mode: CanonicalCallableModeV1,
    pub arguments: Vec<CanonicalCallableArgumentV1>,
    pub result_kind: CanonicalValueTypeV1,
    pub expression: CanonicalExecutableExpressionV1,
    pub origin: CanonicalSourceOriginV1,
}

#[derive(Clone, Debug)]
pub(super) struct CallableCst {
    designation: Vec<u8>,
    exported: bool,
    mode: CanonicalCallableModeV1,
    arguments: Vec<CanonicalCallableArgumentV1>,
    result_kind: CanonicalValueTypeV1,
    body: CallableBodyCst,
    origin: CanonicalSourceOriginV1,
    expression_origin: CanonicalSourceOriginV1,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalCallableModeV1 {
    Function,
    Procedure,
}

/// Reachable foreign contracts are static obligations, not attempt occurrences
/// or an execution order. Repeated uses retain one obligation here.
pub(super) fn foreign_accesses(
    expression: &CanonicalExecutableExpressionV1,
) -> Vec<CanonicalForeignBindingV1> {
    use CanonicalExecutableExpressionV1 as E;
    fn collect(expression: &E, contracts: &mut BTreeSet<CanonicalForeignBindingV1>) {
        match expression {
            E::Foreign { binding, arguments } => {
                contracts.insert(binding.as_ref().clone());
                for argument in arguments {
                    collect(argument, contracts);
                }
            }
            E::Let { value, body, .. } => {
                collect(value, contracts);
                collect(body, contracts);
            }
            E::Sequence(values) => {
                for value in values {
                    collect(value, contracts);
                }
            }
            E::Record(fields) => {
                for value in fields.values() {
                    collect(value, contracts);
                }
            }
            E::SequenceDrop(a, b)
            | E::Concatenate(a, b)
            | E::Equal(a, b)
            | E::GreaterThan(a, b)
            | E::LessThanOrEqual(a, b)
            | E::Add(a, b)
            | E::Subtract(a, b)
            | E::Multiply(a, b)
            | E::Divide(a, b)
            | E::ContainsText(a, b)
            | E::StartsWith(a, b) => {
                collect(a, contracts);
                collect(b, contracts);
            }
            E::Require(a, b, c) | E::Conditional(a, b, c) => {
                collect(a, contracts);
                collect(b, contracts);
                collect(c, contracts);
            }
            E::Field(value, _) | E::SquareRoot(value) | E::TextTransform(_, value) => {
                collect(value, contracts)
            }
            _ => {}
        }
    }
    let mut contracts = BTreeSet::new();
    collect(expression, &mut contracts);
    contracts.into_iter().collect()
}

#[derive(Clone, Debug)]
enum CallableBodyCst {
    Expressions(Vec<CanonicalScalarExpressionV1>),
    Foreign(CanonicalForeignBindingV1),
}

pub(super) fn read(
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
    declarations: &[CstItem],
) -> Result<Option<(CallableCst, RelationCst)>, CanonicalSourceErrorV1> {
    let head = block[0].text;
    let exported = head.starts_with("export ");
    let head = head.strip_prefix("export ").unwrap_or(head);
    let foreign = head.starts_with("foreign ");
    let mut mode = if foreign || head.starts_with("procedure ") {
        CanonicalCallableModeV1::Procedure
    } else {
        CanonicalCallableModeV1::Function
    };
    let head = head
        .strip_prefix("foreign ")
        .or_else(|| head.strip_prefix("procedure "))
        .unwrap_or(head);
    if !exported && !(head.contains('(') && head.contains("):")) {
        return Ok(None);
    }
    let error = |reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason };
    let (name, signature) = head
        .split_once('(')
        .ok_or_else(|| error("expected a named typed callable"))?;
    let designation = denotation_designation_bytes(name.trim(), origin)?;
    if [
        b"drop".as_slice(),
        b"require",
        b"if".as_slice(),
        b"sqrt",
        b"trim",
        b"lowercase",
        b"first-word",
        b"remaining-words",
        b"contains-text",
        b"starts-with",
    ]
    .contains(&designation.as_slice())
    {
        return Err(error("callable name conflicts with a primitive expression"));
    }
    let (parameters, result) = signature
        .split_once("):")
        .ok_or_else(|| error("expected a result type"))?;
    let mut result_kind =
        value_type::resolve(result.trim().as_bytes(), declarations, &mut BTreeSet::new())
            .map_err(error)?;
    let mut arguments = Vec::new();
    let mut roles = Vec::new();
    let mut names = BTreeSet::new();
    for parameter in split_parameters(parameters).ok_or_else(|| error("invalid parameter contract"))? {
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
        let value_kind =
            value_type::resolve(domain.trim().as_bytes(), declarations, &mut BTreeSet::new())
                .map_err(error)?;
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
    let expression_origin = CanonicalSourceOriginV1 {
        start: block.get(1).map_or(origin.start, |line| line.start as u64),
        ..origin
    };
    let body = if foreign {
        let (evaluation, operation, failure, module, member) = foreign::read_abi(&block[1..])
            .ok_or_else(|| error("invalid foreign ABI declaration"))?;
        if let CanonicalForeignEvaluationV1::Construct { target } = &evaluation {
            mode = CanonicalCallableModeV1::Function;
            result_kind = CanonicalValueTypeV1::Delayed { target: target.clone(), value: Box::new(result_kind) };
            roles.last_mut().expect("callable result role").domain = format!("Delayed<{target},{}>", result.trim()).into_bytes();
        }
        let binding = CanonicalForeignBindingV1 {
            evaluation,
            operation,
            failure,
            module,
            member,
            arguments: arguments.iter().map(|a| a.value_kind.clone()).collect(),
            result: result_kind.clone(),
        };
        binding.check().map_err(error)?;
        CallableBodyCst::Foreign(binding)
    } else {
        let source = block
            .iter()
            .skip(1)
            .map(|line| line.text)
            .collect::<Vec<_>>()
            .join("\n");
        let mut parser = ScalarExpressionParser {
            source: source.as_bytes(),
            cursor: 0,
            current: "",
            interpolate: true,
        };
        let mut expressions = Vec::new();
        loop {
            expressions.push(
                parser
                    .comparison()
                    .ok_or_else(|| error("unsupported callable expression"))?,
            );
            parser.skip_spaces();
            if parser.cursor == parser.source.len() {
                break;
            }
            let trailing = &source[..parser.cursor];
            let gap = trailing.trim_end_matches(char::is_whitespace).len();
            if mode != CanonicalCallableModeV1::Procedure || !trailing[gap..].contains('\n') {
                return Err(error(
                    "multiple expressions require separate procedure lines",
                ));
            }
        }
        CallableBodyCst::Expressions(expressions)
    };
    let callable = CallableCst {
        expression_origin,
        designation: designation.clone(),
        exported,
        mode,
        arguments,
        result_kind,
        body,
        origin,
    };
    let known = callable
        .arguments
        .iter()
        .map(|a| a.designation.clone())
        .collect::<Vec<_>>();
    let mut canonical = if mode == CanonicalCallableModeV1::Function {
        b"pure given".to_vec()
    } else {
        b"procedure given".to_vec()
    };
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
                let mut parser = ScalarExpressionParser {
                    source: source.get(expression_start..cursor)?.as_bytes(),
                    cursor: 0,
                    current: "",
                    interpolate: true,
                };
                parts.push(parser.comparison()?);
                parser.skip_spaces();
                if parser.cursor != parser.source.len() {
                    return None;
                }
                cursor += 1;
                start = cursor;
            }
            _ => cursor += 1,
        }
    }
    None
}

const MAX_EXPANDED_NODES: usize = 65_536;
const MAX_CALL_DEPTH: usize = 64;

pub(super) fn check_definitions(
    definitions: &[CallableCst],
) -> Result<Vec<CanonicalCallableV1>, CanonicalSourceErrorV1> {
    let mut indices = BTreeMap::new();
    for (index, definition) in definitions.iter().enumerate() {
        if indices
            .insert(definition.designation.clone(), index)
            .is_some()
        {
            return Err(CanonicalSourceErrorV1::DuplicateDesignation {
                designation: definition.designation.clone(),
            });
        }
    }
    let mut expansion = Expansion {
        definitions,
        indices,
        checked: BTreeMap::new(),
        active: BTreeSet::new(),
        remaining: MAX_EXPANDED_NODES,
        next_binding: 0,
    };
    for (index, definition) in definitions.iter().enumerate() {
        expansion.compile(index, definition.origin)?;
    }
    Ok((0..definitions.len())
        .map(|index| {
            expansion
                .checked
                .remove(&index)
                .expect("each definition was checked")
        })
        .collect())
}

struct Expansion<'a> {
    definitions: &'a [CallableCst],
    indices: BTreeMap<Vec<u8>, usize>,
    checked: BTreeMap<usize, CanonicalCallableV1>,
    active: BTreeSet<usize>,
    remaining: usize,
    next_binding: u32,
}

impl Expansion<'_> {
    fn binding(&mut self, origin: CanonicalSourceOriginV1) -> Result<u16, CanonicalSourceErrorV1> {
        let binding = u16::try_from(self.next_binding).map_err(|_| {
            CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "pure callable binding limit",
            }
        })?;
        self.next_binding += 1;
        Ok(binding)
    }

    fn consume(
        &mut self,
        origin: CanonicalSourceOriginV1,
        depth: usize,
    ) -> Result<(), CanonicalSourceErrorV1> {
        if depth >= MAX_CALL_DEPTH || self.remaining == 0 {
            return Err(CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "pure callable expansion limit",
            });
        }
        self.remaining -= 1;
        Ok(())
    }

    fn compile(
        &mut self,
        index: usize,
        origin: CanonicalSourceOriginV1,
    ) -> Result<CanonicalCallableV1, CanonicalSourceErrorV1> {
        if let Some(checked) = self.checked.get(&index) {
            return Ok(checked.clone());
        }
        if !self.active.insert(index) {
            return Err(CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "recursive pure callables are unsupported",
            });
        }
        if self.active.len() > MAX_CALL_DEPTH {
            return Err(CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "pure callable dependency depth limit",
            });
        }
        let definitions = self.definitions;
        let definition = &definitions[index];
        let expression = match &definition.body {
            CallableBodyCst::Expressions(expressions) => {
                let mut expressions = expressions
                    .iter()
                    .map(|expression| {
                        lower(
                            expression,
                            &definition.arguments,
                            definition.expression_origin,
                            self,
                            0,
                            definition.mode,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut body = expressions.pop().expect("parsed nonempty procedure body");
                for value in expressions.into_iter().rev() {
                    body = CanonicalExecutableExpressionV1::Let {
                        binding: self.binding(definition.expression_origin)?,
                        value: Box::new(value),
                        body: Box::new(body),
                    };
                }
                body
            }
            CallableBodyCst::Foreign(binding) => CanonicalExecutableExpressionV1::Foreign {
                binding: Box::new(binding.clone()),
                arguments: (0..definition.arguments.len())
                    .map(|i| CanonicalExecutableExpressionV1::Argument(i as u16))
                    .collect(),
            },
        };
        let callable = CanonicalCallableV1 {
            designation: definition.designation.clone(),
            exported: definition.exported,
            mode: definition.mode,
            arguments: definition.arguments.clone(),
            result_kind: definition.result_kind.clone(),
            expression,
            origin: definition.origin,
        };
        check_canonical_callable_v1(&callable)?;
        self.active.remove(&index);
        self.checked.insert(index, callable.clone());
        Ok(callable)
    }
}

// Fresh binders keep independently expanded callee scopes disjoint. Caller
// actuals are bound outside this walk and are never substituted into again.
fn bind_body(
    expression: &CanonicalExecutableExpressionV1,
    actual: &[u16],
    locals: &BTreeMap<u16, u16>,
    expansion: &mut Expansion<'_>,
    origin: CanonicalSourceOriginV1,
    depth: usize,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    expansion.consume(origin, depth)?;
    let mut recur = |e| bind_body(e, actual, locals, expansion, origin, depth + 1).map(Box::new);
    Ok(match expression {
        E::Sequence(values) => E::Sequence(
            values
                .iter()
                .map(|v| recur(v).map(|v| *v))
                .collect::<Result<_, _>>()?,
        ),
        E::Record(fields) => E::Record(
            fields
                .iter()
                .map(|(k, v)| Ok((k.clone(), *recur(v)?)))
                .collect::<Result<_, CanonicalSourceErrorV1>>()?,
        ),
        E::SequenceDrop(a, b) => E::SequenceDrop(recur(a)?, recur(b)?),
        E::Field(a, k) => E::Field(recur(a)?, k.clone()),
        E::Require(a, b, c) => E::Require(recur(a)?, recur(b)?, recur(c)?),
        E::Foreign { binding, arguments } => E::Foreign {
            binding: binding.clone(),
            arguments: arguments
                .iter()
                .map(|v| recur(v).map(|v| *v))
                .collect::<Result<_, _>>()?,
        },
        E::Let {
            binding,
            value,
            body,
        } => {
            let value = recur(value)?;
            let fresh = expansion.binding(origin)?;
            let mut nested = locals.clone();
            nested.insert(*binding, fresh);
            let body = Box::new(bind_body(
                body,
                actual,
                &nested,
                expansion,
                origin,
                depth + 1,
            )?);
            E::Let {
                binding: fresh,
                value,
                body,
            }
        }
        E::Argument(i) => E::Binding(*actual.get(usize::from(*i)).ok_or(
            CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "unresolved call argument",
            },
        )?),
        E::Binding(i) => E::Binding(*locals.get(i).ok_or(
            CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "unresolved lexical binding",
            },
        )?),
        E::Constant(value) => E::Constant(value.clone()),
        E::ContainsText(a, b) => E::ContainsText(recur(a)?, recur(b)?),
        E::StartsWith(a, b) => E::StartsWith(recur(a)?, recur(b)?),
        E::Equal(a, b) => E::Equal(recur(a)?, recur(b)?),
        E::GreaterThan(a, b) => E::GreaterThan(recur(a)?, recur(b)?),
        E::LessThanOrEqual(a, b) => E::LessThanOrEqual(recur(a)?, recur(b)?),
        E::Concatenate(a, b) => E::Concatenate(recur(a)?, recur(b)?),
        E::Add(a, b) => E::Add(recur(a)?, recur(b)?),
        E::Subtract(a, b) => E::Subtract(recur(a)?, recur(b)?),
        E::Multiply(a, b) => E::Multiply(recur(a)?, recur(b)?),
        E::Divide(a, b) => E::Divide(recur(a)?, recur(b)?),
        E::SquareRoot(a) => E::SquareRoot(recur(a)?),
        E::TextTransform(op, a) => E::TextTransform(*op, recur(a)?),
        E::Conditional(a, b, c) => E::Conditional(recur(a)?, recur(b)?, recur(c)?),
        _ => {
            return Err(CanonicalSourceErrorV1::InvalidCallable {
                origin,
                reason: "unsupported pure callable expression",
            });
        }
    })
}

fn lower(
    expression: &CanonicalScalarExpressionV1,
    arguments: &[CanonicalCallableArgumentV1],
    origin: CanonicalSourceOriginV1,
    expansion: &mut Expansion<'_>,
    depth: usize,
    mode: CanonicalCallableModeV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalScalarExpressionV1 as S;
    let error = || CanonicalSourceErrorV1::InvalidCallable {
        origin,
        reason: "unresolved or unsupported pure expression",
    };
    expansion.consume(origin, depth)?;
    let mut recur = |e| lower(e, arguments, origin, expansion, depth + 1, mode).map(Box::new);
    Ok(match expression {
        S::Sequence(values) => E::Sequence(
            values
                .iter()
                .map(|v| recur(v).map(|v| *v))
                .collect::<Result<_, _>>()?,
        ),
        S::Record(fields) => E::Record(
            fields
                .iter()
                .map(|(k, v)| Ok((k.clone(), *recur(v)?)))
                .collect::<Result<_, CanonicalSourceErrorV1>>()?,
        ),
        S::SequenceDrop(a, b) => E::SequenceDrop(recur(a)?, recur(b)?),
        S::Field(a, k) => E::Field(recur(a)?, k.clone()),
        S::Require(a, b, c) => E::Require(recur(a)?, recur(b)?, recur(c)?),
        S::Call {
            designation,
            arguments: actual,
        } => {
            let values = actual
                .iter()
                .map(|a| lower(a, arguments, origin, expansion, depth + 1, mode))
                .collect::<Result<Vec<_>, _>>()?;
            let index = *expansion.indices.get(designation).ok_or(
                CanonicalSourceErrorV1::InvalidCallable {
                    origin,
                    reason: "unresolved pure callable",
                },
            )?;
            let callee = expansion.compile(index, origin)?;
            if mode == CanonicalCallableModeV1::Function
                && callee.mode == CanonicalCallableModeV1::Procedure
            {
                return Err(CanonicalSourceErrorV1::InvalidCallable {
                    origin,
                    reason: "procedure call is forbidden in a pure callable",
                });
            }
            if values.len() != callee.arguments.len() {
                return Err(CanonicalSourceErrorV1::InvalidCallable {
                    origin,
                    reason: "callable argument count mismatch",
                });
            }
            for (value, parameter) in values.iter().zip(&callee.arguments) {
                if expression_kind(
                    value,
                    &arguments
                        .iter()
                        .map(|a| a.value_kind.clone())
                        .collect::<Vec<_>>(),
                    &BTreeMap::new(),
                    0,
                    mode,
                )
                .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?
                    != parameter.value_kind
                {
                    return Err(CanonicalSourceErrorV1::InvalidCallable {
                        origin,
                        reason: "callable argument type mismatch",
                    });
                }
            }
            let bindings = values
                .iter()
                .map(|_| expansion.binding(origin))
                .collect::<Result<Vec<_>, _>>()?;
            let body = bind_body(
                &callee.expression,
                &bindings,
                &BTreeMap::new(),
                expansion,
                origin,
                depth + values.len(),
            )?;
            values
                .into_iter()
                .zip(bindings)
                .rev()
                .fold(body, |body, (value, binding)| E::Let {
                    binding,
                    value: Box::new(value),
                    body: Box::new(body),
                })
        }
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

/// Checks the declared recursive contracts and the selected Mode's effect allowance.
pub fn check_canonical_callable_v1(
    callable: &CanonicalCallableV1,
) -> Result<(), CanonicalSourceErrorV1> {
    let error = |reason| CanonicalSourceErrorV1::InvalidCallable {
        origin: callable.origin,
        reason,
    };
    fn supported(kind: &CanonicalValueTypeV1) -> bool {
        use CanonicalScalarValueKindV1 as K;
        match kind {
            CanonicalValueTypeV1::Scalar(K::Text | K::Number | K::Boolean) => true,
            CanonicalValueTypeV1::Delayed { .. } => kind.check().is_ok(),
            CanonicalValueTypeV1::Sequence(element) => supported(element),
            CanonicalValueTypeV1::Record(fields) => fields.values().all(supported),
            _ => false,
        }
    }
    let mut names = BTreeSet::new();
    if !supported(&callable.result_kind)
        || callable
            .arguments
            .iter()
            .any(|a| !supported(&a.value_kind) || !names.insert(&a.designation))
    {
        return Err(error("unsupported or duplicate argument/result contract"));
    }
    let arguments = callable
        .arguments
        .iter()
        .map(|a| a.value_kind.clone())
        .collect::<Vec<_>>();
    if expression_kind(
        &callable.expression,
        &arguments,
        &BTreeMap::new(),
        0,
        callable.mode,
    )
    .map_err(error)?
        != callable.result_kind
    {
        return Err(error("callable result type mismatch"));
    }
    Ok(())
}

fn expression_kind(
    expression: &CanonicalExecutableExpressionV1,
    arguments: &[CanonicalValueTypeV1],
    bindings: &BTreeMap<u16, CanonicalValueTypeV1>,
    depth: usize,
    mode: CanonicalCallableModeV1,
) -> Result<CanonicalValueTypeV1, &'static str> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalScalarValueKindV1 as K;
    use CanonicalValueTypeV1 as T;
    if depth >= 64 {
        return Err("callable expression depth limit");
    }
    let recur = |e| expression_kind(e, arguments, bindings, depth + 1, mode);
    let require = |e, kind: &T| {
        if recur(e)? == *kind {
            Ok(())
        } else {
            Err("callable expression type mismatch")
        }
    };
    Ok(match expression {
        E::Let {
            binding,
            value,
            body,
        } => {
            let kind = recur(value)?;
            let mut nested = bindings.clone();
            if nested.insert(*binding, kind).is_some() {
                return Err("duplicate lexical binding");
            }
            expression_kind(body, arguments, &nested, depth + 1, mode)?
        }
        E::Binding(i) => bindings.get(i).ok_or("unresolved lexical binding")?.clone(),
        E::Argument(i) => arguments
            .get(usize::from(*i))
            .ok_or("unresolved argument")?
            .clone(),
        E::Constant(CanonicalScalarValueV1::Number(bits)) if f64::from_bits(*bits).is_finite() => {
            K::Number.into()
        }
        E::Constant(CanonicalScalarValueV1::Boolean(_)) => K::Boolean.into(),
        E::Constant(CanonicalScalarValueV1::Text(_)) => K::Text.into(),
        E::Sequence(values) => {
            let first = values
                .first()
                .ok_or("empty sequence literal needs an element contract")?;
            let kind = recur(first)?;
            for value in &values[1..] {
                require(value, &kind)?;
            }
            T::Sequence(Box::new(kind))
        }
        E::Record(fields) => T::Record(
            fields
                .iter()
                .map(|(k, v)| {
                    Ok((
                        k.clone(),
                        expression_kind(
                            v,
                            arguments,
                            bindings,
                            depth + 1,
                            CanonicalCallableModeV1::Function,
                        )?,
                    ))
                })
                .collect::<Result<_, &'static str>>()?,
        ),
        E::SequenceDrop(value, count) => {
            require(count, &K::Number.into())?;
            let kind = recur(value)?;
            if !matches!(kind, T::Sequence(_)) {
                return Err("drop requires an ordered sequence");
            }
            kind
        }
        E::Field(value, field) => {
            let T::Record(fields) = recur(value)? else {
                return Err("field requires a record");
            };
            fields.get(field).ok_or("unknown record field")?.clone()
        }
        E::Require(condition, value, message) => {
            require(condition, &K::Boolean.into())?;
            require(message, &K::Text.into())?;
            recur(value)?
        }
        E::Foreign {
            binding,
            arguments: actual,
        } => {
            if mode != CanonicalCallableModeV1::Procedure && binding.evaluation == CanonicalForeignEvaluationV1::Attempt {
                return Err("foreign effects are forbidden in a pure callable");
            }
            binding.check()?;
            if actual.len() != binding.arguments.len() {
                return Err("foreign argument count mismatch");
            }
            for (value, kind) in actual.iter().zip(&binding.arguments) {
                require(value, kind)?;
            }
            binding.result.clone()
        }
        E::Concatenate(a, b) | E::ContainsText(a, b) | E::StartsWith(a, b) => {
            require(a, &K::Text.into())?;
            require(b, &K::Text.into())?;
            if matches!(expression, E::Concatenate(..)) {
                K::Text.into()
            } else {
                K::Boolean.into()
            }
        }
        E::TextTransform(_, a) => {
            require(a, &K::Text.into())?;
            K::Text.into()
        }
        E::Add(a, b)
        | E::Subtract(a, b)
        | E::Multiply(a, b)
        | E::Divide(a, b)
        | E::GreaterThan(a, b)
        | E::LessThanOrEqual(a, b) => {
            require(a, &K::Number.into())?;
            require(b, &K::Number.into())?;
            if matches!(expression, E::GreaterThan(..) | E::LessThanOrEqual(..)) {
                K::Boolean.into()
            } else {
                K::Number.into()
            }
        }
        E::SquareRoot(a) => {
            require(a, &K::Number.into())?;
            K::Number.into()
        }
        E::Equal(a, b) => {
            let kind = recur(a)?;
            if kind.contains_delayed() { return Err("delayed values cannot be compared during construction"); }
            require(b, &kind)?;
            K::Boolean.into()
        }
        E::Conditional(a, b, c) => {
            require(a, &K::Boolean.into())?;
            let kind = recur(b)?;
            require(c, &kind)?;
            kind
        }
        _ => return Err("unsupported callable expression or state access"),
    })
}

fn split_parameters(source: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, byte) in source.bytes().enumerate() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => { parts.push(source[start..index].trim()); start = index + 1; }
            _ => {}
        }
    }
    if depth != 0 { return None; }
    if !source[start..].trim().is_empty() { parts.push(source[start..].trim()); }
    Some(parts)
}
