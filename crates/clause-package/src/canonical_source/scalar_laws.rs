use super::*;

#[derive(Clone, Debug)]
pub(super) struct ScalarLawCst {
    pub origin: CanonicalSourceOriginV1,
    pub designation: Vec<u8>,
    relation: Vec<u8>,
    roles: BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
    predicates: Vec<CanonicalScalarPredicateV1>,
}

#[derive(Clone, Debug)]
pub(super) struct ScalarDeriveCst {
    pub origin: CanonicalSourceOriginV1,
    pub designation: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ScalarLawEnvironment {
    pub relations: Vec<RelationCst>,
    pub laws: Vec<ScalarLawCst>,
    pub derives: Vec<ScalarDeriveCst>,
}

#[derive(Clone, Debug)]
pub(super) struct ScalarLawCase {
    pub value: CanonicalScalarExpressionV1,
    pub predicates: Vec<CanonicalScalarPredicateV1>,
    pub law_origin: CanonicalSourceOriginV1,
    pub derive_origin: CanonicalSourceOriginV1,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ScalarBindingCase {
    pub bindings: BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
    pub predicates: Vec<CanonicalScalarPredicateV1>,
    pub origins: Vec<CanonicalSourceOriginV1>,
}

impl ScalarLawEnvironment {
    pub fn read(
        artifact: CanonicalSourceArtifactIdV1,
        lines: &[SourceLine<'_>],
    ) -> Result<Self, CanonicalSourceErrorV1> {
        let starts = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.indent == 0 && !line.text.trim().is_empty())
            .map(|(index, _)| index)
            .chain(std::iter::once(lines.len()))
            .collect::<Vec<_>>();
        let blocks = starts
            .windows(2)
            .map(|pair| &lines[pair[0]..pair[1]])
            .collect::<Vec<_>>();
        let mut environment = Self::default();
        for block in &blocks {
            if let Some(name) = block[0].text.strip_prefix("relation ") {
                let origin = line_origin(artifact, block[0]);
                environment.relations.push(parse_relation(
                    artifact,
                    block,
                    designation_bytes(name, origin)?,
                )?);
            }
        }
        for block in &blocks {
            let Some(name) = block[0].text.strip_prefix("law ") else {
                continue;
            };
            let origin = CanonicalSourceOriginV1 {
                artifact,
                start: block[0].start as u64,
                end: block
                    .iter()
                    .rfind(|line| !line.text.trim().is_empty())
                    .unwrap()
                    .end as u64,
            };
            let mut section = "";
            let mut predicates = Vec::new();
            let mut result = None;
            let mut supported = true;
            let mut sections = BTreeSet::new();
            for line in block
                .iter()
                .skip(1)
                .filter(|line| !line.text.trim().is_empty())
            {
                let text = line.text.trim();
                if line.indent == 2 && matches!(text, "if" | "then") && sections.insert(text) {
                    section = text;
                } else if line.indent == 4 && section == "if" {
                    if let Some(predicate) = parse_scalar_predicate(text, "") {
                        predicates.push(predicate);
                    } else {
                        supported = false;
                    }
                } else if line.indent == 4 && section == "then" && result.is_none() {
                    result = Some(text);
                } else {
                    supported = false;
                }
            }
            if !supported {
                continue;
            }
            let Some(result) = result else { continue };
            let Some((relation, roles)) = environment.application(result, origin)? else {
                continue;
            };
            let relation = relation.designation.clone();
            environment.laws.push(ScalarLawCst {
                origin,
                designation: designation_bytes(name, origin)?,
                relation,
                roles,
                predicates,
            });
        }
        for block in &blocks {
            if let Some(name) = block[0].text.strip_prefix("derive ") {
                let designation = designation_bytes(name, line_origin(artifact, block[0]))?;
                if environment
                    .laws
                    .iter()
                    .any(|law| law.designation == designation)
                {
                    require_leaf(block, artifact)?;
                    environment.derives.push(ScalarDeriveCst {
                        origin: line_origin(artifact, block[0]),
                        designation,
                    });
                }
            }
        }
        for law in &environment.laws {
            if environment
                .laws
                .iter()
                .filter(|other| other.designation == law.designation)
                .count()
                != 1
                || environment
                    .derives
                    .iter()
                    .filter(|derive| derive.designation == law.designation)
                    .count()
                    > 1
            {
                return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                    origin: law.origin,
                });
            }
        }
        Ok(environment)
    }

    fn application(
        &self,
        source: &str,
        origin: CanonicalSourceOriginV1,
    ) -> Result<
        Option<(&RelationCst, BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>)>,
        CanonicalSourceErrorV1,
    > {
        let mut matches = Vec::new();
        for relation in &self.relations {
            if relation.subject.is_some() || relation.roles.iter().any(|role| role.domain != b"F64")
            {
                continue;
            }
            for bindings in reading_matches(source, &relation.reading) {
                matches.push((relation, bindings));
            }
        }
        if matches.len() > 1 {
            return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin });
        }
        Ok(matches.pop())
    }

    pub fn binding(
        &self,
        source: &str,
        origin: CanonicalSourceOriginV1,
    ) -> Result<Option<ScalarLawBindingCst>, CanonicalSourceErrorV1> {
        let Some((relation, roles)) = self.application(source, origin)? else {
            return Ok(None);
        };
        let modes = relation
            .modes
            .iter()
            .filter(|mode| {
                mode.produced.len() == 1
                    && mode.cardinality == SourceCardinality::Maybe
                    && mode.effect.is_none()
                    && mode.reactive_obligation.is_none()
                    && !mode.continues_linearly
                    && mode.known.len() + 1 == relation.roles.len()
                    && mode
                        .known
                        .iter()
                        .chain(&mode.produced)
                        .collect::<BTreeSet<_>>()
                        == relation
                            .roles
                            .iter()
                            .map(|role| &role.name)
                            .collect::<BTreeSet<_>>()
            })
            .collect::<Vec<_>>();
        let [mode] = modes.as_slice() else {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        };
        let output = &mode.produced[0];
        let Some(CanonicalScalarExpressionV1::Parameter(parameter)) = roles.get(output) else {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        };
        let mut cases = Vec::new();
        for law in self
            .laws
            .iter()
            .filter(|law| law.relation == relation.designation)
        {
            let Some(derive) = self
                .derives
                .iter()
                .find(|derive| derive.designation == law.designation)
            else {
                continue;
            };
            let mut substitutions = BTreeMap::new();
            let mut predicates = Vec::new();
            for role in &mode.known {
                let given = &roles[role];
                match &law.roles[role] {
                    CanonicalScalarExpressionV1::Parameter(variable) => {
                        if let Some(previous) =
                            substitutions.insert(variable.clone(), given.clone())
                        {
                            predicates
                                .push(CanonicalScalarPredicateV1::Equal(previous, given.clone()));
                        }
                    }
                    CanonicalScalarExpressionV1::Number(_) => {
                        predicates.push(CanonicalScalarPredicateV1::Equal(
                            law.roles[role].clone(),
                            given.clone(),
                        ));
                    }
                    _ => {
                        return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                            origin: law.origin,
                        });
                    }
                }
            }
            let mut used = BTreeSet::new();
            collect_scalar_expression_parameters(&law.roles[output], &mut used);
            for predicate in &law.predicates {
                collect_predicate_parameters(predicate, &mut used);
            }
            if used
                .iter()
                .any(|variable| !substitutions.contains_key(variable))
            {
                return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                    origin: law.origin,
                });
            }
            // Substitution is simultaneous: a caller variable with the same spelling
            // as a law binder is not another occurrence of that binder.
            let value = substitute(&law.roles[output], &substitutions);
            predicates.extend(
                law.predicates
                    .iter()
                    .map(|predicate| substitute_predicate(predicate, &substitutions)),
            );
            cases.push(ScalarLawCase {
                value,
                predicates,
                law_origin: law.origin,
                derive_origin: derive.origin,
            });
        }
        if cases.is_empty() {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
        for (index, left) in cases.iter().enumerate() {
            for right in &cases[index + 1..] {
                if left.value != right.value
                    && !contradictory(left.predicates.iter().chain(&right.predicates))
                {
                    return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin });
                }
            }
        }
        Ok(Some(ScalarLawBindingCst {
            origin,
            parameter: parameter.clone(),
            cases,
        }))
    }
}

fn reading_matches(
    source: &str,
    reading: &[RelationReadingPartCst],
) -> Vec<BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>> {
    fn walk(
        source: &str,
        reading: &[RelationReadingPartCst],
        bindings: BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
        found: &mut Vec<BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>>,
    ) {
        let source = source.trim();
        let Some((part, rest)) = reading.split_first() else {
            if source.is_empty() {
                found.push(bindings);
            }
            return;
        };
        if found.len() > 1 {
            return;
        }
        match part {
            RelationReadingPartCst::Literal(literal) => {
                let literal = std::str::from_utf8(literal).unwrap();
                if let Some(tail) = source.strip_prefix(literal)
                    && (tail.is_empty() || tail.starts_with(char::is_whitespace))
                {
                    walk(tail, rest, bindings, found);
                }
            }
            RelationReadingPartCst::Role(role) => {
                let mut depth = 0_i32;
                for (end, character) in source
                    .char_indices()
                    .chain(std::iter::once((source.len(), ' ')))
                {
                    match character {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    if depth != 0 || !character.is_whitespace() || end == 0 {
                        continue;
                    }
                    let Some(value) = parse_scalar_expression(&source[..end], "") else {
                        continue;
                    };
                    if !numeric_expression(&value) {
                        continue;
                    }
                    if bindings
                        .get(role)
                        .is_some_and(|previous| previous != &value)
                    {
                        continue;
                    }
                    let mut next = bindings.clone();
                    next.insert(role.clone(), value);
                    walk(&source[end..], rest, next, found);
                }
            }
        }
    }
    let mut found = Vec::new();
    walk(source, reading, BTreeMap::new(), &mut found);
    found
}

fn numeric_expression(value: &CanonicalScalarExpressionV1) -> bool {
    use CanonicalScalarExpressionV1::*;
    match value {
        Parameter(_) | Number(_) => true,
        SquareRoot(value) => numeric_expression(value),
        Add(a, b) | Subtract(a, b) | Multiply(a, b) | Divide(a, b) => {
            numeric_expression(a) && numeric_expression(b)
        }
        _ => false,
    }
}

fn substitute(
    expression: &CanonicalScalarExpressionV1,
    bindings: &BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
) -> CanonicalScalarExpressionV1 {
    use CanonicalScalarExpressionV1::*;
    match expression {
        Parameter(name) => bindings.get(name).unwrap_or(expression).clone(),
        Equal(a, b) => Equal(Box::new(substitute(a, bindings)), Box::new(substitute(b, bindings))),
        GreaterThan(a, b) => GreaterThan(Box::new(substitute(a, bindings)), Box::new(substitute(b, bindings))),
        LessThanOrEqual(a, b) => LessThanOrEqual(Box::new(substitute(a, bindings)), Box::new(substitute(b, bindings))),
        SquareRoot(value) => SquareRoot(Box::new(substitute(value, bindings))),
        Add(a, b) => Add(
            Box::new(substitute(a, bindings)),
            Box::new(substitute(b, bindings)),
        ),
        Subtract(a, b) => Subtract(
            Box::new(substitute(a, bindings)),
            Box::new(substitute(b, bindings)),
        ),
        Multiply(a, b) => Multiply(
            Box::new(substitute(a, bindings)),
            Box::new(substitute(b, bindings)),
        ),
        Divide(a, b) => Divide(
            Box::new(substitute(a, bindings)),
            Box::new(substitute(b, bindings)),
        ),
        _ => expression.clone(),
    }
}

pub(super) fn collect_predicate_parameters(
    predicate: &CanonicalScalarPredicateV1,
    used: &mut BTreeSet<Vec<u8>>,
) {
    let (a, b) = match predicate {
        CanonicalScalarPredicateV1::Equal(a, b)
        | CanonicalScalarPredicateV1::GreaterThan(a, b)
        | CanonicalScalarPredicateV1::LessThanOrEqual(a, b) => (a, b),
    };
    collect_scalar_expression_parameters(a, used);
    collect_scalar_expression_parameters(b, used);
}

fn substitute_predicate(
    predicate: &CanonicalScalarPredicateV1,
    bindings: &BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
) -> CanonicalScalarPredicateV1 {
    use CanonicalScalarPredicateV1::*;
    match predicate {
        Equal(a, b) => Equal(substitute(a, bindings), substitute(b, bindings)),
        GreaterThan(a, b) => GreaterThan(substitute(a, bindings), substitute(b, bindings)),
        LessThanOrEqual(a, b) => LessThanOrEqual(substitute(a, bindings), substitute(b, bindings)),
    }
}

// A strict cycle in the finite order graph proves disjointness. Failure to
// find one is unknown, not evidence of either overlap or completeness.
fn contradictory<'a>(predicates: impl Iterator<Item = &'a CanonicalScalarPredicateV1>) -> bool {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for predicate in predicates {
        let (a, b, strict, equal) = match predicate {
            CanonicalScalarPredicateV1::GreaterThan(a, b) => (b, a, true, false),
            CanonicalScalarPredicateV1::LessThanOrEqual(a, b) => (a, b, false, false),
            CanonicalScalarPredicateV1::Equal(a, b) => (a, b, false, true),
        };
        let mut index = |value| {
            if let Some(index) = nodes.iter().position(|node| node == &value) {
                index
            } else {
                nodes.push(value);
                nodes.len() - 1
            }
        };
        let a = index(a);
        let b = index(b);
        edges.push((a, b, strict));
        if equal {
            edges.push((b, a, false));
        }
    }
    let count = nodes.len();
    let mut closure = vec![vec![None; count]; count];
    for (a, b, strict) in edges {
        closure[a][b] = Some(closure[a][b].unwrap_or(false) || strict);
    }
    for k in 0..count {
        for a in 0..count {
            for b in 0..count {
                if let (Some(left), Some(right)) = (closure[a][k], closure[k][b]) {
                    closure[a][b] = Some(closure[a][b].unwrap_or(false) || left || right);
                }
            }
        }
    }
    (0..count).any(|index| closure[index][index] == Some(true))
}

pub(super) fn binding_cases(
    bindings: &[ScalarLawBindingCst],
) -> Result<Vec<ScalarBindingCase>, CanonicalSourceErrorV1> {
    let mut cases = vec![ScalarBindingCase::default()];
    let mut pending = bindings.iter().collect::<Vec<_>>();
    while !pending.is_empty() {
        let index = pending
            .iter()
            .position(|binding| {
                let mut used = BTreeSet::new();
                for case in &binding.cases {
                    collect_scalar_expression_parameters(&case.value, &mut used);
                    for predicate in &case.predicates {
                        collect_predicate_parameters(predicate, &mut used);
                    }
                }
                !pending
                    .iter()
                    .any(|dependency| used.contains(&dependency.parameter))
            })
            .ok_or(CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                origin: pending[0].origin,
            })?;
        let binding = pending.remove(index);
        if cases.len().saturating_mul(binding.cases.len()) > 4096 {
            return Err(CanonicalSourceErrorV1::ScalarLawExpansionLimit {
                origin: binding.origin,
            });
        }
        cases = cases
            .into_iter()
            .flat_map(|prior| {
                binding.cases.iter().map(move |case| {
                    let mut next = prior.clone();
                    next.bindings
                        .insert(binding.parameter.clone(), case.value.clone());
                    next.predicates.extend(case.predicates.clone());
                    next.origins.extend([case.law_origin, case.derive_origin]);
                    next
                })
            })
            .collect();
    }
    Ok(cases)
}

pub(super) fn guarded_predicates(
    source: &[CanonicalScalarPredicateV1],
    case: &ScalarBindingCase,
) -> Vec<CanonicalScalarPredicateV1> {
    let (mut independent, dependent): (Vec<_>, Vec<_>) =
        source.iter().cloned().partition(|predicate| {
            let mut used = BTreeSet::new();
            collect_predicate_parameters(predicate, &mut used);
            !used
                .iter()
                .any(|parameter| case.bindings.contains_key(parameter))
        });
    independent.extend(case.predicates.clone());
    independent.extend(dependent);
    independent
}
