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
    arguments: Vec<CallableArgumentCst>,
    result_kind: Option<value_type::Pattern>,
    type_parameters: BTreeSet<Vec<u8>>,
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
            E::Lambda { body, .. } => collect(body, contracts),
            E::Apply(function, argument) => { collect(function, contracts); collect(argument, contracts); }
            E::Widen { value, .. } => collect(value, contracts),
            E::Match { value, cases } => {
                collect(value, contracts);
                for (_, _, body) in cases { collect(body, contracts); }
            }
            E::Foreign { binding, arguments } => {
                contracts.insert(binding.as_ref().clone());
                for argument in arguments {
                    collect(argument, contracts);
                }
            }
            E::Let { value, body, .. }
            | E::SequenceMap {
                source: value,
                body,
                ..
            } => {
                collect(value, contracts);
                collect(body, contracts);
            }
            E::SequenceFold { source, initial, body, .. } => {
                collect(source, contracts); collect(initial, contracts); collect(body, contracts);
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
            E::Dictionary(a, b)
            | E::SequenceJoin(a, b)
            | E::SequenceAppend(a, b)
            | E::SequenceDrop(a, b)
            | E::Concatenate(a, b)
            | E::Equal(a, b)
            | E::GreaterThan(a, b)
            | E::LessThanOrEqual(a, b)
            | E::Add(a, b)
            | E::Subtract(a, b)
            | E::Multiply(a, b)
            | E::Divide(a, b)
            | E::ContainsText(a, b)
            | E::TextSplit(a, b)
            | E::StartsWith(a, b) => {
                collect(a, contracts);
                collect(b, contracts);
            }
            E::Require(a, b, c) | E::Conditional(a, b, c) => {
                collect(a, contracts);
                collect(b, contracts);
                collect(c, contracts);
            }
            E::SequenceSort(value)
            | E::TextCharacters(value)
            | E::ParseIntegerPrefix(value)
            | E::SequenceCount(value)
            | E::ScalarText(value)
            | E::Field(value, _)
            | E::SquareRoot(value)
            | E::TextTransform(_, value) => collect(value, contracts),
            _ => {}
        }
    }
    let mut contracts = BTreeSet::new();
    collect(expression, &mut contracts);
    contracts.into_iter().collect()
}

#[derive(Clone, Debug)]
struct CallableArgumentCst {
    designation: Vec<u8>,
    contract: CallableArgumentContract,
}

#[derive(Clone, Debug)]
enum CallableArgumentContract {
    Value(value_type::Pattern),
    StaticFieldPath,
}

impl CallableArgumentCst {
    fn value_kind(&self) -> Option<&value_type::Pattern> {
        match &self.contract {
            CallableArgumentContract::Value(kind) => Some(kind),
            CallableArgumentContract::StaticFieldPath => None,
        }
    }
}

#[derive(Clone, Debug)]
enum CallableBodyCst {
    Expressions(Vec<(Option<Vec<u8>>, CanonicalScalarExpressionV1)>),
    Foreign {
        evaluation: CanonicalForeignEvaluationV1,
        operation: CanonicalForeignOperationV1,
        failure: CanonicalForeignFailureV1,
        module: String,
        member: foreign::MemberCst,
    },
}

pub(super) fn read(
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
    declarations: &[CstItem],
) -> Result<Option<(CallableCst, Option<RelationCst>)>, CanonicalSourceErrorV1> {
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
    if !exported && !(head.contains('(') && (head.contains("):") || head.ends_with(')'))) {
        return Ok(None);
    }
    let error = |reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason };
    let (name, signature) = head
        .split_once('(')
        .ok_or_else(|| error("expected a named typed callable"))?;
    let (name, type_parameters) = if let Some((name, parameters)) = name.trim().split_once('<') {
        if foreign && exported {
            return Err(error("foreign record type parameters must remain private"));
        }
        let parameters = parameters.strip_suffix('>').ok_or_else(|| error("invalid type parameters"))?;
        let mut names = BTreeSet::new();
        for parameter in parameters.split(',') {
            let (name, bound) = parameter.split_once(':').ok_or_else(|| error("type parameter requires a Record bound"))?;
            let name = denotation_designation_bytes(name.trim(), origin)?;
            if bound.trim() != "Record" || !names.insert(name) {
                return Err(error("expected distinct Record type parameters"));
            }
        }
        (name, names)
    } else {
        (name, BTreeSet::new())
    };
    let designation = denotation_designation_bytes(name.trim(), origin)?;
    if [
        b"drop".as_slice(),
        b"count",
        b"join",
        b"require",
        b"if".as_slice(),
        b"match",
        b"sqrt",
        b"trim",
        b"characters",
        b"split-text",
        b"parse-integer-prefix",
        b"lowercase",
        b"first-word",
        b"remaining-words",
        b"contains-text",
        b"starts-with",
        b"path",
        b"record-at",
        b"dictionary",
        b"field-at",
    ]
    .contains(&designation.as_slice())
    {
        return Err(error("callable name conflicts with a primitive expression"));
    }
    let (parameters, suffix) = signature.split_once(')').ok_or_else(|| error("invalid callable signature"))?;
    let result = if suffix.trim().is_empty() { "" } else {
        suffix.trim().strip_prefix(':').ok_or_else(|| error("expected a result type"))?.trim()
    };
    let mut result_kind = if result.is_empty() {
        if foreign { return Err(error("foreign declarations require an explicit result contract")); }
        None
    } else {
        Some(value_type::Pattern::read(result.as_bytes(), declarations, &type_parameters).map_err(error)?)
    };
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
        let contract = if domain.trim() == "FieldPath" {
            CallableArgumentContract::StaticFieldPath
        } else {
            CallableArgumentContract::Value(value_type::Pattern::read(domain.trim().as_bytes(), declarations, &type_parameters)
                .map_err(error)?)
        };
        roles.push(RelationRoleCst {
            name: name.clone(),
            domain: domain.trim().as_bytes().to_vec(),
            origin,
        });
        arguments.push(CallableArgumentCst {
            designation: name,
            contract,
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
            result_kind = Some(value_type::Pattern::Delayed {
                target: target.clone(), value: Box::new(result_kind.take().expect("foreign result contract")),
            });
            roles.last_mut().expect("callable result role").domain = format!("Delayed<{target},{}>", result.trim()).into_bytes();
        }
        if module.is_empty() || module.contains('\0') {
            return Err(error("foreign module and member must be nonempty identifiers"));
        }
        match &member {
            foreign::MemberCst::Exact(member) if (member.is_empty() && operation != CanonicalForeignOperationV1::Root) || member.contains('\0') =>
                return Err(error("foreign module and member must be nonempty identifiers")),
            foreign::MemberCst::StaticFieldPath(name) => {
                if !matches!(evaluation, CanonicalForeignEvaluationV1::Construct { .. }) {
                    return Err(error("static foreign field paths require delayed construction"));
                }
                if !arguments.iter().any(|argument| argument.designation == *name && argument.value_kind().is_none()) {
                    return Err(error("foreign member requires a declared FieldPath argument"));
                }
            }
            _ => {}
        }
        if operation != CanonicalForeignOperationV1::Call && arguments.iter().any(|a| a.value_kind().is_some()) {
            return Err(error("foreign property access takes no arguments"));
        }
        CallableBodyCst::Foreign { evaluation, operation, failure, module, member }
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
            parser.skip_spaces();
            let mut binding = None;
            let line = source[parser.cursor..].split('\n').next().unwrap_or("");
            if let Some((name, _)) = line.strip_prefix('?').and_then(|line| line.split_once(':')) {
                if !name.contains(|ch: char| ch.is_whitespace() || matches!(ch, '(' | ')' | '{' | '}')) {
                    application_designation_bytes(name, expression_origin)?;
                    binding = Some(format!("?{name}").into_bytes());
                    parser.cursor += name.len() + 2;
                }
            }
            let expression = parser.disjunction().ok_or_else(|| error("unsupported callable expression"))?;
            let denotation = binding.is_some();
            expressions.push((binding, expression));
            parser.skip_spaces();
            if parser.cursor == parser.source.len() {
                if denotation { return Err(error("local denotation requires a following result expression")); }
                break;
            }
            let trailing = &source[..parser.cursor];
            let gap = trailing.trim_end_matches(char::is_whitespace).len();
            if (!denotation && mode != CanonicalCallableModeV1::Procedure) || !trailing[gap..].contains('\n') {
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
        type_parameters,
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
    for parameter in &callable.type_parameters {
        if !callable.arguments.iter().filter_map(CallableArgumentCst::value_kind).any(|kind| kind.contains(parameter)) {
            return Err(error("type parameter must be inferred from an argument"));
        }
    }
    if foreign {
        let result = callable.result_kind.as_ref().expect("foreign result contract");
        for pattern in callable.arguments.iter().filter_map(CallableArgumentCst::value_kind).chain(std::iter::once(result)) {
            pattern.check().map_err(error)?;
            match &callable.body {
                CallableBodyCst::Foreign { evaluation: CanonicalForeignEvaluationV1::Attempt, .. }
                    if pattern.contains_delayed() => return Err(error("runtime foreign crossing cannot carry delayed values")),
                CallableBodyCst::Foreign { evaluation: CanonicalForeignEvaluationV1::Construct { target }, .. }
                    if !pattern.in_target(target) => return Err(error("foreign construction target mismatch")),
                _ => {}
            }
        }
    }
    let relation = (!callable.requires_specialization()).then_some(relation);
    Ok(Some((callable, relation)))
}

impl CallableCst {
    fn requires_specialization(&self) -> bool {
        !self.type_parameters.is_empty() || self.arguments.iter().any(|a| a.value_kind().is_none())
    }

    fn foreign_binding(
        &self,
        substitutions: &BTreeMap<Vec<u8>, CanonicalValueTypeV1>,
        static_paths: &BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
        origin: CanonicalSourceOriginV1,
    ) -> Result<CanonicalForeignBindingV1, CanonicalSourceErrorV1> {
        let error = |reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason };
        let CallableBodyCst::Foreign { evaluation, operation, failure, module, member } = &self.body else {
            return Err(error("expected a foreign declaration"));
        };
        let binding = CanonicalForeignBindingV1 {
            evaluation: evaluation.clone(), operation: *operation, failure: *failure,
            module: module.clone(), member: match member {
                foreign::MemberCst::Exact(member) => member.clone(),
                foreign::MemberCst::StaticFieldPath(name) => static_paths.get(name)
                    .ok_or_else(|| error("unresolved static field path"))?.iter()
                    .map(|field| std::str::from_utf8(field).map_err(|_| error("invalid field path segment")))
                    .collect::<Result<Vec<_>, _>>()?.join("."),
            },
            arguments: self.arguments.iter().filter_map(CallableArgumentCst::value_kind).map(|kind| kind.instantiate(substitutions)).collect::<Result<_, _>>().map_err(error)?,
            result: self.result_kind.as_ref().ok_or_else(|| error("foreign result contract is missing"))?
                .instantiate(substitutions).map_err(error)?,
        };
        binding.check().map_err(error)?;
        Ok(binding)
    }
}

pub(super) fn complete_inferred_results(items: &mut [std::sync::Arc<CstItem>], callables: &[CanonicalCallableV1]) -> Result<(), CanonicalSourceErrorV1> {
    for item in items {
        if let CstKind::Relation(relation) = &mut std::sync::Arc::make_mut(item).kind
            && let Some(callable) = callables.iter().find(|c| c.designation == relation.designation)
            && let Some(role) = relation.roles.last_mut()
            && role.domain.is_empty()
        {
            role.domain = crate::canonical::encode_value_type_v1(&callable.result_kind)
                .map_err(CanonicalSourceErrorV1::Encode)?;
        }
    }
    Ok(())
}

impl CallableCst {
    pub(super) fn rebase_origins(&mut self, edit: &incremental_read::OriginEdit<'_>) -> Result<(), CanonicalSourceErrorV1> {
        edit.origin(&mut self.origin)?;
        edit.origin(&mut self.expression_origin)
    }
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
                if bytes.get(cursor + 1..cursor + 3) == Some(b"u{") {
                    cursor += 3;
                    while *bytes.get(cursor)? != b'}' { cursor += 1; }
                    cursor += 1;
                } else {
                    cursor += 2;
                }
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
                parts.push(E::ScalarText(Box::new(parser.disjunction()?)));
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
    declarations: &[std::sync::Arc<CstItem>],
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
        declarations,
        indices,
        checked: BTreeMap::new(),
        active: BTreeSet::new(),
        remaining: MAX_EXPANDED_NODES,
        next_binding: 0,
    };
    for (index, definition) in definitions.iter().enumerate() {
        if !definition.requires_specialization() {
            expansion.compile(index, &BTreeMap::new(), &BTreeMap::new(), definition.origin)?;
        }
    }
    Ok((0..definitions.len())
        .filter(|index| !definitions[*index].requires_specialization())
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
    declarations: &'a [std::sync::Arc<CstItem>],
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
        substitutions: &BTreeMap<Vec<u8>, CanonicalValueTypeV1>,
        static_paths: &BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
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
        let arguments = definition.arguments.iter().filter_map(|argument| argument.value_kind().map(|kind| (argument, kind))).map(|(argument, kind)| Ok(CanonicalCallableArgumentV1 {
            designation: argument.designation.clone(),
            value_kind: kind.instantiate(substitutions)
                .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?,
        })).collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
        let expression = match &definition.body {
            CallableBodyCst::Expressions(expressions) => {
                let mut locals = BTreeMap::new();
                let mut values = Vec::new();
                let types = arguments.iter().map(|a| a.value_kind.clone()).collect::<Vec<_>>();
                for (position, (name, expression)) in expressions.iter().enumerate() {
                    if name.as_ref().is_some_and(|name| locals.contains_key(name) || definition.arguments.iter().any(|a| name.strip_prefix(b"?") == Some(a.designation.as_slice()))) {
                        return Err(CanonicalSourceErrorV1::InvalidCallable { origin, reason: "duplicate local binding" });
                    }
                    let expected = if position + 1 == expressions.len() { definition.result_kind.as_ref().map(|kind| kind.instantiate(substitutions)).transpose()
                        .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })? } else { None };
                    let value = lower(expression, &arguments, &locals, static_paths, definition.expression_origin, self, 0, definition.mode, expected.as_ref())?;
                    let binding = self.binding(definition.expression_origin)?;
                    if let Some(name) = name {
                        let kind = expression_kind(&value, &types, &locals.values().cloned().collect(), 0, definition.mode)
                            .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
                        locals.insert(name.clone(), (binding, kind));
                    }
                    values.push((binding, value));
                }
                let (_, mut body) = values.pop().expect("parsed nonempty callable body");
                for (binding, value) in values.into_iter().rev() {
                    body = CanonicalExecutableExpressionV1::Let {
                        binding,
                        value: Box::new(value),
                        body: Box::new(body),
                    };
                }
                body
            }
            CallableBodyCst::Foreign { .. } => CanonicalExecutableExpressionV1::Foreign {
                binding: Box::new(definition.foreign_binding(substitutions, static_paths, origin)?),
                arguments: (0..arguments.len())
                    .map(|i| CanonicalExecutableExpressionV1::Argument(i as u16))
                    .collect(),
            },
        };
        let result_kind = match &definition.result_kind {
            Some(kind) => kind.instantiate(substitutions),
            None => expression_kind(&expression, &arguments.iter().map(|a| a.value_kind.clone()).collect::<Vec<_>>(),
                &BTreeMap::new(), 0, definition.mode),
        }.map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
        let callable = CanonicalCallableV1 {
            designation: definition.designation.clone(),
            exported: definition.exported,
            mode: definition.mode,
            arguments,
            result_kind,
            expression,
            origin: definition.origin,
        };
        check_canonical_callable_v1(&callable)?;
        self.active.remove(&index);
        if !definition.requires_specialization() {
            self.checked.insert(index, callable.clone());
        }
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
        E::Lambda { binding, kind, body } => {
            let fresh = expansion.binding(origin)?;
            let mut nested = locals.clone(); nested.insert(*binding, fresh);
            E::Lambda { binding: fresh, kind: kind.clone(), body: Box::new(bind_body(body, actual, &nested, expansion, origin, depth + 1)?) }
        }
        E::Apply(function, argument) => E::Apply(recur(function)?, recur(argument)?),
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
        E::Widen { value, kind } => E::Widen { value: recur(value)?, kind: kind.clone() },
        E::Match { value, cases } => {
            let value = recur(value)?;
            let mut bound_cases = Vec::new();
            for (kind, binding, body) in cases {
                let fresh = expansion.binding(origin)?;
                let mut nested = locals.clone(); nested.insert(*binding, fresh);
                bound_cases.push((kind.clone(), fresh, bind_body(body, actual, &nested, expansion, origin, depth + 1)?));
            }
            E::Match { value, cases: bound_cases }
        }
        E::Dictionary(key, value) => E::Dictionary(recur(key)?, recur(value)?),
        E::EmptySequence(kind) => E::EmptySequence(kind.clone()),
        E::SequenceSort(a) => E::SequenceSort(recur(a)?),
        E::SequenceAppend(a,b) => E::SequenceAppend(recur(a)?, recur(b)?),
        E::SequenceFold { accumulator, item, source, initial, body } => {
            let source = recur(source)?;
            let initial = recur(initial)?;
            let fresh_accumulator = expansion.binding(origin)?;
            let fresh_item = expansion.binding(origin)?;
            let mut nested = locals.clone();
            nested.insert(*accumulator, fresh_accumulator); nested.insert(*item, fresh_item);
            let body = Box::new(bind_body(body, actual, &nested, expansion, origin, depth + 1)?);
            E::SequenceFold { accumulator: fresh_accumulator, item: fresh_item, source, initial, body }
        }
        E::TextCharacters(a) => E::TextCharacters(recur(a)?),
        E::ParseIntegerPrefix(a) => E::ParseIntegerPrefix(recur(a)?),
        E::TextSplit(a, b) => E::TextSplit(recur(a)?, recur(b)?),
        E::SequenceCount(a) => E::SequenceCount(recur(a)?),
        E::ScalarText(a) => E::ScalarText(recur(a)?),
        E::SequenceJoin(a, b) => E::SequenceJoin(recur(a)?, recur(b)?),
        E::SequenceMap {
            binding,
            source,
            body,
        } => {
            let source = recur(source)?;
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
            E::SequenceMap {
                binding: fresh,
                source,
                body,
            }
        }
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

fn static_field_path(
    expression: &CanonicalScalarExpressionV1,
    paths: &BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
    locals: &BTreeMap<Vec<u8>, (u16, CanonicalValueTypeV1)>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<Vec<u8>>, CanonicalSourceErrorV1> {
    let path = match expression {
        CanonicalScalarExpressionV1::StaticFieldPath(fields) => Some(fields),
        CanonicalScalarExpressionV1::Parameter(name) if !locals.contains_key(name) =>
            name.strip_prefix(b"?").and_then(|name| paths.get(name)),
        _ => None,
    };
    path.filter(|fields| !fields.is_empty() && fields.len() < MAX_CALL_DEPTH
        && fields.iter().all(|field| !field.is_empty()
            && field.iter().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))))
        .cloned().ok_or(CanonicalSourceErrorV1::InvalidCallable {
            origin, reason: "expected a FieldPath literal or parameter",
        })
}

fn lower(
    expression: &CanonicalScalarExpressionV1,
    arguments: &[CanonicalCallableArgumentV1],
    locals: &BTreeMap<Vec<u8>, (u16, CanonicalValueTypeV1)>,
    static_paths: &BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
    origin: CanonicalSourceOriginV1,
    expansion: &mut Expansion<'_>,
    depth: usize,
    mode: CanonicalCallableModeV1,
    expected: Option<&CanonicalValueTypeV1>,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    use CanonicalScalarExpressionV1 as S;
    let error = || CanonicalSourceErrorV1::InvalidCallable {
        origin,
        reason: "unresolved or unsupported pure expression",
    };
    expansion.consume(origin, depth)?;
    let mut recur =
        |e| lower(e, arguments, locals, static_paths, origin, expansion, depth + 1, mode, None).map(Box::new);
    let expected_record = if let (S::Record(fields), Some(CanonicalValueTypeV1::Alternatives(types))) = (expression, expected) {
        let mut candidates = types.iter().filter(|kind| matches!(kind, CanonicalValueTypeV1::Record(wanted) if wanted.keys().eq(fields.keys())));
        let first = candidates.next();
        if candidates.next().is_none() { first } else { None }
    } else { None };
    let record_expected = expected_record.or(expected);
    let lowered = match expression {
        S::Lambda { binding, kind, body } => {
            let kind = value_type::resolve(kind, expansion.declarations, &mut BTreeSet::new())
                .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
            kind.check().map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
            let fresh = expansion.binding(origin)?;
            let mut nested = locals.clone(); nested.insert(binding.clone(), (fresh, kind.clone()));
            E::Lambda { binding: fresh, kind, body: Box::new(lower(body, arguments, &nested, static_paths, origin, expansion, depth + 1, mode, None)?) }
        }
        S::Apply(function, argument) => E::Apply(recur(function)?, recur(argument)?),
        S::Match { value, cases } => {
            let value = recur(value)?;
            let mut lowered_cases = Vec::new();
            for (name, binding, body) in cases {
                let kind = value_type::resolve(name, expansion.declarations, &mut BTreeSet::new())
                    .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
                let fresh = expansion.binding(origin)?;
                let mut nested = locals.clone(); nested.insert(binding.clone(), (fresh, kind.clone()));
                let body = lower(body, arguments, &nested, static_paths, origin, expansion, depth + 1, mode, expected)?;
                lowered_cases.push((kind, fresh, body));
            }
            E::Match { value, cases: lowered_cases }
        }
        S::StaticFieldPath(_) => return Err(CanonicalSourceErrorV1::InvalidCallable {
            origin, reason: "FieldPath cannot become a runtime value",
        }),
        S::Dictionary(key, value) => {
            let key = recur(key)?;
            let element = match expected {
                Some(CanonicalValueTypeV1::Dictionary(element)) => Some(element.as_ref()),
                _ => None,
            };
            let value = lower(value, arguments, locals, static_paths, origin, expansion, depth + 1, mode, element)?;
            E::Dictionary(key, Box::new(value))
        }
        S::RecordAt(path, value) => {
            let path = static_field_path(path, static_paths, locals, origin)?;
            let mut wanted = expected;
            for field in &path {
                wanted = match wanted {
                    Some(CanonicalValueTypeV1::Record(fields)) => fields.get(field),
                    _ => None,
                };
            }
            let value = lower(value, arguments, locals, static_paths, origin, expansion, depth + path.len(), mode, wanted)?;
            path.into_iter().rev().fold(value, |value, field| E::Record(BTreeMap::from([(field, value)])))
        }
        S::FieldAt(value, path) => {
            let path = static_field_path(path, static_paths, locals, origin)?;
            let value = lower(value, arguments, locals, static_paths, origin, expansion, depth + path.len(), mode, None)?;
            path.into_iter().fold(value, |value, field| E::Field(Box::new(value), field))
        }
        S::Sequence(values) => {
            let element = match expected { Some(CanonicalValueTypeV1::Sequence(element)) => Some(element.as_ref()), _ => None };
            if values.is_empty() {
                E::EmptySequence(element.ok_or(CanonicalSourceErrorV1::InvalidCallable { origin, reason: "empty sequence literal needs an element contract" })?.clone())
            } else {
                E::Sequence(values.iter().map(|v| lower(v, arguments, locals, static_paths, origin, expansion, depth + 1, mode, element)).collect::<Result<_, _>>()?)
            }
        },
        S::Record(fields) => E::Record(
            fields
                .iter()
                .map(|(k, v)| {
                    let wanted = match record_expected { Some(CanonicalValueTypeV1::Record(fields)) => fields.get(k), _ => None };
                    Ok((k.clone(), lower(v, arguments, locals, static_paths, origin, expansion, depth + 1, mode, wanted)?))
                })
                .collect::<Result<_, CanonicalSourceErrorV1>>()?,
        ),
        S::SequenceSort(a) => E::SequenceSort(Box::new(lower(a, arguments, locals, static_paths, origin, expansion, depth + 1, mode,
            Some(&CanonicalValueTypeV1::Sequence(Box::new(CanonicalScalarValueKindV1::Text.into()))))?)),
        S::SequenceAppend(a,b) => {
            let a = Box::new(lower(a, arguments, locals, static_paths, origin, expansion, depth + 1, mode, expected)?);
            let types = arguments.iter().map(|a| a.value_kind.clone()).collect::<Vec<_>>();
            let bindings = locals.values().cloned().collect();
            let CanonicalValueTypeV1::Sequence(element) = expression_kind(&a, &types, &bindings, 0, mode)
                .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })? else { return Err(error()); };
            let b = Box::new(lower(b, arguments, locals, static_paths, origin, expansion, depth + 1, mode, Some(&element))?);
            E::SequenceAppend(a,b)
        }
        S::SequenceFold { accumulator, item, source, initial, body } => {
            if accumulator == item { return Err(CanonicalSourceErrorV1::InvalidCallable { origin, reason: "duplicate fold binding" }); }
            let source = recur(source)?;
            let initial = Box::new(lower(initial, arguments, locals, static_paths, origin, expansion, depth + 1, mode, expected)?);
            let types = arguments.iter().map(|a| a.value_kind.clone()).collect::<Vec<_>>();
            let bindings = locals.values().cloned().collect();
            let kind = |e| expression_kind(e, &types, &bindings, 0, mode)
                .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason });
            let CanonicalValueTypeV1::Sequence(element) = kind(&source)? else { return Err(error()); };
            let accumulator_kind = kind(&initial)?;
            let fresh_accumulator = expansion.binding(origin)?;
            let fresh_item = expansion.binding(origin)?;
            let mut nested = locals.clone();
            nested.insert(accumulator.clone(), (fresh_accumulator, accumulator_kind.clone()));
            nested.insert(item.clone(), (fresh_item, *element));
            let body = Box::new(lower(body, arguments, &nested, static_paths, origin, expansion, depth + 1, mode, Some(&accumulator_kind))?);
            E::SequenceFold { accumulator: fresh_accumulator, item: fresh_item, source, initial, body }
        }
        S::TextCharacters(a) => E::TextCharacters(recur(a)?),
        S::ParseIntegerPrefix(a) => E::ParseIntegerPrefix(recur(a)?),
        S::TextSplit(a, b) => E::TextSplit(recur(a)?, recur(b)?),
        S::SequenceCount(a) => E::SequenceCount(recur(a)?),
        S::ScalarText(a) => E::ScalarText(recur(a)?),
        S::SequenceJoin(a, b) => E::SequenceJoin(recur(a)?, recur(b)?),
        S::SequenceMap {
            binding,
            source,
            body,
        } => {
            let source = recur(source)?;
            let argument_types = arguments
                .iter()
                .map(|a| a.value_kind.clone())
                .collect::<Vec<_>>();
            let binding_types = locals.values().cloned().collect();
            let CanonicalValueTypeV1::Sequence(element) =
                expression_kind(&source, &argument_types, &binding_types, 0, mode)
                    .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?
            else {
                return Err(CanonicalSourceErrorV1::InvalidCallable {
                    origin,
                    reason: "mapping requires an ordered sequence",
                });
            };
            let fresh = expansion.binding(origin)?;
            let mut nested = locals.clone();
            nested.insert(binding.clone(), (fresh, *element));
            let body = Box::new(lower(
                body,
                arguments,
                &nested,
                static_paths,
                origin,
                expansion,
                depth + 1,
                CanonicalCallableModeV1::Function,
                match expected { Some(CanonicalValueTypeV1::Sequence(element)) => Some(element.as_ref()), _ => None },
            )?);
            E::SequenceMap {
                binding: fresh,
                source,
                body,
            }
        }
        S::SequenceDrop(a, b) => E::SequenceDrop(recur(a)?, recur(b)?),
        S::Field(a, k) => E::Field(recur(a)?, k.clone()),
        S::Require(a, b, c) => E::Require(recur(a)?, recur(b)?, recur(c)?),
        S::Call {
            designation,
            arguments: actual,
        } => {
            let index = *expansion.indices.get(designation).ok_or(
                CanonicalSourceErrorV1::InvalidCallable {
                    origin,
                    reason: "unresolved pure callable",
                },
            )?;
            let definition = &expansion.definitions[index];
            if actual.len() != definition.arguments.len() {
                return Err(CanonicalSourceErrorV1::InvalidCallable { origin, reason: "callable argument count mismatch" });
            }
            let mut values = Vec::new();
            let mut substitutions = BTreeMap::new();
            let mut callee_paths = BTreeMap::new();
            for (actual, parameter) in actual.iter().zip(&definition.arguments) {
                let Some(pattern) = parameter.value_kind() else {
                    callee_paths.insert(parameter.designation.clone(), static_field_path(actual, static_paths, locals, origin)?);
                    continue;
                };
                let wanted = pattern.instantiate(&substitutions).ok();
                let value = lower(actual, arguments, locals, static_paths, origin, expansion, depth + 1, mode, wanted.as_ref())?;
                let argument_types = arguments.iter().map(|a| a.value_kind.clone()).collect::<Vec<_>>();
                let binding_types = locals.values().cloned().collect();
                let kind = expression_kind(&value, &argument_types, &binding_types, 0, mode)
                    .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
                pattern.unify(&kind, &mut substitutions)
                    .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
                values.push(value);
            }
            if definition.requires_specialization() && matches!(definition.body, CallableBodyCst::Foreign { .. }) {
                let binding = definition.foreign_binding(&substitutions, &callee_paths, origin)?;
                return Ok(E::Foreign { binding: Box::new(binding), arguments: values });
            }
            let callee = expansion.compile(index, &substitutions, &callee_paths, origin)?;
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
                    &locals.values().cloned().collect(),
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
        S::Parameter(name) if locals.contains_key(name) => E::Binding(locals[name].0),
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
        S::Conditional(a, b, c) => E::Conditional(recur(a)?,
            Box::new(lower(b, arguments, locals, static_paths, origin, expansion, depth + 1, mode, expected)?),
            Box::new(lower(c, arguments, locals, static_paths, origin, expansion, depth + 1, mode, expected)?)),
        S::Current | S::Symbol(_) => return Err(error()),
    };
    if let Some(kind @ CanonicalValueTypeV1::Alternatives(_)) = expected {
        let argument_types = arguments.iter().map(|a| a.value_kind.clone()).collect::<Vec<_>>();
        let binding_types = locals.values().cloned().collect();
        let actual = expression_kind(&lowered, &argument_types, &binding_types, 0, mode)
            .map_err(|reason| CanonicalSourceErrorV1::InvalidCallable { origin, reason })?;
        if &actual != kind && kind.includes_alternatives(&actual) { return Ok(E::Widen { value: Box::new(lowered), kind: kind.clone() }); }
    }
    Ok(lowered)
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
            CanonicalValueTypeV1::Sequence(element) | CanonicalValueTypeV1::Dictionary(element) => supported(element),
            CanonicalValueTypeV1::Record(fields) => fields.values().all(supported),
            CanonicalValueTypeV1::Alternatives(types) => kind.check().is_ok() && types.iter().all(supported),
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
        E::Lambda { binding, kind, body } => {
            kind.check()?;
            let T::Delayed { target, value } = kind else { return Err("target lambda requires a delayed argument contract"); };
            let mut nested = bindings.clone();
            if nested.insert(*binding, kind.clone()).is_some() { return Err("duplicate lexical binding"); }
            let result = expression_kind(body, arguments, &nested, depth + 1, CanonicalCallableModeV1::Function)?;
            let result = result.constructed_value(target)?;
            T::Delayed { target: target.clone(), value: Box::new(T::Function { argument: value.clone(), result: Box::new(result) }) }
        }
        E::Apply(function, argument) => {
            let T::Delayed { target, value } = recur(function)? else { return Err("application requires a delayed function"); };
            let T::Function { argument: expected, result } = *value else { return Err("application requires a function contract"); };
            if recur(argument)?.constructed_value(&target)? != *expected { return Err("function argument type mismatch"); }
            T::Delayed { target, value: result }
        }
        E::Widen { value, kind } => {
            kind.check()?;
            let T::Alternatives(_) = kind else { return Err("inclusion requires an alternative contract"); };
            if !kind.includes_alternatives(&recur(value)?) { return Err("value is not a declared alternative"); }
            kind.clone()
        }
        E::Match { value, cases } => {
            let T::Alternatives(types) = recur(value)? else { return Err("match requires alternatives"); };
            types.iter().try_for_each(T::check)?;
            let mut remaining = types;
            let mut result = None;
            for (kind, binding, body) in cases {
                if !remaining.remove(kind) { return Err("unreachable or duplicate alternative"); }
                let mut nested = bindings.clone();
                if nested.insert(*binding, kind.clone()).is_some() { return Err("duplicate lexical binding"); }
                let actual = expression_kind(body, arguments, &nested, depth + 1, mode)?;
                if result.as_ref().is_some_and(|expected| expected != &actual) { return Err("match result type mismatch"); }
                result = Some(actual);
            }
            if !remaining.is_empty() { return Err("missing alternative"); }
            result.ok_or("empty match")?
        }
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
        E::EmptySequence(element) => { element.check()?; T::Sequence(Box::new(element.clone())) }
        E::SequenceSort(value) => {
            let kind = T::Sequence(Box::new(K::Text.into())); require(value, &kind)?; kind
        }
        E::SequenceAppend(sequence, item) => {
            let kind = recur(sequence)?;
            let T::Sequence(element) = &kind else { return Err("append requires an ordered sequence"); };
            require(item, element)?; kind
        }
        E::SequenceFold { accumulator, item, source, initial, body } => {
            let T::Sequence(element) = recur(source)? else { return Err("fold requires an ordered sequence"); };
            let kind = recur(initial)?;
            let mut nested = bindings.clone();
            if nested.insert(*accumulator, kind.clone()).is_some() || nested.insert(*item, *element).is_some() { return Err("duplicate lexical binding"); }
            if expression_kind(body, arguments, &nested, depth + 1, mode)? != kind { return Err("fold body must preserve accumulator contract"); }
            kind
        }
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
        E::Dictionary(key, value) => {
            let element = recur(value)?;
            match recur(key)? {
                T::Scalar(K::Text) => T::Dictionary(Box::new(element)),
                T::Delayed { target, value } if *value == T::Scalar(K::Text) => T::Delayed {
                    value: Box::new(T::Dictionary(Box::new(element.constructed_value(&target)?))), target,
                },
                _ => return Err("dictionary key requires Text or delayed Text"),
            }
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
        E::SequenceMap {
            binding,
            source,
            body,
        } => {
            let T::Sequence(element) = recur(source)? else {
                return Err("mapping requires an ordered sequence");
            };
            let mut nested = bindings.clone();
            if nested.insert(*binding, *element).is_some() {
                return Err("duplicate lexical binding");
            }
            T::Sequence(Box::new(expression_kind(
                body,
                arguments,
                &nested,
                depth + 1,
                CanonicalCallableModeV1::Function,
            )?))
        }
        E::SequenceCount(value) => {
            if !matches!(recur(value)?, T::Sequence(_)) {
                return Err("count requires an ordered sequence");
            }
            K::Number.into()
        }
        E::SequenceJoin(value, separator) => {
            require(value, &T::Sequence(Box::new(K::Text.into())))?;
            require(separator, &K::Text.into())?;
            K::Text.into()
        }
        E::ScalarText(value) => {
            match recur(value)? {
                T::Scalar(K::Text | K::Boolean | K::Number) => K::Text.into(),
                T::Delayed { target, value } if *value == T::Scalar(K::Text) => T::Delayed { target, value },
                _ => return Err("interpolation requires Text, Bool or F64"),
            }
        }
        E::SequenceDrop(value, count) => {
            require(count, &K::Number.into())?;
            let kind = recur(value)?;
            if !matches!(kind, T::Sequence(_)) {
                return Err("drop requires an ordered sequence");
            }
            kind
        }
        E::Field(value, field) => {
            match recur(value)? {
                T::Record(fields) => fields.get(field).ok_or("unknown record field")?.clone(),
                T::Delayed { target, value } => {
                    let T::Record(fields) = *value else { return Err("field requires a record"); };
                    T::Delayed { target, value: Box::new(fields.get(field).ok_or("unknown record field")?.clone()) }
                }
                _ => return Err("field requires a record"),
            }
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
        E::Concatenate(a, b) => {
            let a = recur(a)?;
            let b = recur(b)?;
            let target = match (&a, &b) {
                (T::Delayed { target, .. }, _) | (_, T::Delayed { target, .. }) => Some(target.clone()),
                _ => None,
            };
            if let Some(target) = target {
                if a.constructed_value(&target)? != T::Scalar(K::Text) || b.constructed_value(&target)? != T::Scalar(K::Text) { return Err("concatenation requires Text"); }
                T::Delayed { target, value: Box::new(K::Text.into()) }
            } else {
                if a != T::Scalar(K::Text) || b != T::Scalar(K::Text) { return Err("concatenation requires Text"); }
                K::Text.into()
            }
        }
        E::ContainsText(a, b) | E::StartsWith(a, b) => {
            require(a, &K::Text.into())?;
            require(b, &K::Text.into())?;
            K::Boolean.into()
        }
        E::TextCharacters(a) => {
            require(a, &K::Text.into())?;
            T::Sequence(Box::new(K::Text.into()))
        }
        E::TextSplit(a, b) => {
            require(a, &K::Text.into())?;
            require(b, &K::Text.into())?;
            T::Sequence(Box::new(K::Text.into()))
        }
        E::ParseIntegerPrefix(a) => {
            require(a, &K::Text.into())?;
            T::Alternatives(BTreeSet::from([K::Number.into(), K::Text.into()]))
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
            let a = recur(a)?;
            let b = recur(b)?;
            match (&a, &b) {
                (T::Delayed { target, .. }, _) | (_, T::Delayed { target, .. }) => {
                    if !a.in_target(target) || !b.in_target(target) { return Err("equality construction target mismatch"); }
                    if a.constructed_value(target)? != b.constructed_value(target)? { return Err("equality value type mismatch"); }
                    T::Delayed { target: target.clone(), value: Box::new(K::Boolean.into()) }
                }
                _ => {
                    if a.contains_delayed() || b.contains_delayed() { return Err("delayed values cannot be compared during construction"); }
                    if a != b { return Err("equality value type mismatch"); }
                    K::Boolean.into()
                }
            }
        }
        E::Conditional(a, b, c) => {
            let join = |target: String, b: T, c: T| -> Result<T, &'static str> {
                if !b.in_target(&target) || !c.in_target(&target) { return Err("conditional construction target mismatch"); }
                let b = b.constructed_value(&target)?;
                let c = c.constructed_value(&target)?;
                let value = if b == c { b } else {
                    let mut types = BTreeSet::new();
                    for kind in [b, c] {
                        match kind {
                            T::Alternatives(alternatives) => types.extend(alternatives),
                            kind => { types.insert(kind); }
                        }
                    }
                    T::Alternatives(types)
                };
                let kind = T::Delayed { target, value: Box::new(value) };
                kind.check()?;
                Ok(kind)
            };
            match recur(a)? {
                T::Delayed { target, value } if *value == T::Scalar(K::Boolean) => {
                    join(target, recur(b)?, recur(c)?)?
                }
                T::Scalar(K::Boolean) => {
                    let kind = recur(b)?;
                    let other = recur(c)?;
                    match (&kind, &other) {
                        (T::Delayed { target, .. }, T::Delayed { .. }) => join(target.clone(), kind, other)?,
                        _ if kind == other => kind,
                        _ => return Err("callable expression type mismatch"),
                    }
                }
                _ => return Err("conditional requires Bool"),
            }
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
