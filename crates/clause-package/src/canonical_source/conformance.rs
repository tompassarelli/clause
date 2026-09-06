//! Structural participation in relation-level contracts. Mode guarantees do
//! not define a subject's structure, and checking never emits membership facts.
use super::*;

#[derive(Clone, Debug, Default)]
pub(super) struct Domains {
    structural: BTreeSet<Vec<u8>>,
    facets: BTreeMap<Vec<u8>, BTreeSet<Vec<u8>>>,
}

pub(super) fn is_structural(cst: &CanonicalSourceCstV1, domain: &[u8]) -> bool {
    cst.conformance.get_or_init(|| check(cst)).structural.contains(domain)
}

pub(super) fn creation_domain(
    parameter: &[u8],
    includes: &[LogicalSourceLine],
    environment: &ScalarLawEnvironment,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    let mut domains = BTreeSet::new();
    for source in includes {
        let source_origin = &source.origin;
        let Some(insertions) = structured_values::insertion(source, "") else { continue };
        for insertion in insertions {
            let relations = environment.relations.iter().filter(|relation|
                relation.surface == insertion.target.relation).collect::<Vec<_>>();
            let [relation] = relations.as_slice() else {
                return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin: *source_origin });
            };
            let Some(subject) = relation.roles.iter().find(|role| Some(&role.name) == relation.subject.as_ref())
                else { return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin: *source_origin }); };
            if insertion.target.subject == parameter { domains.insert(subject.domain.clone()); }
            if matches!(&insertion.value, CanonicalScalarExpressionV1::Parameter(name) if name == parameter) {
                let values = relation.roles.iter().filter(|role| role.name != subject.name).collect::<Vec<_>>();
                let [value] = values.as_slice() else {
                    return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin: *source_origin });
                };
                domains.insert(value.domain.clone());
            }
        }
    }
    if domains.len() != 1 {
        return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin });
    }
    Ok(domains.into_iter().next().expect("one inferred creation domain"))
}

pub(super) fn members<'a>(
    cst: &'a CanonicalSourceCstV1,
    domain: &[u8],
) -> impl Iterator<Item = &'a Vec<u8>> {
    cst.conformance.get_or_init(|| check(cst)).facets.get(domain)
        .into_iter().flat_map(|subjects| subjects.iter())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Value {
    Scalar(CanonicalScalarValueV1),
    Record(Vec<u8>, BTreeMap<Vec<u8>, CanonicalScalarValueV1>),
}

fn fact(item: &CstItem) -> Option<(&[u8], &[u8], Value)> {
    use CanonicalScalarValueV1 as S;
    Some(match &item.kind {
        CstKind::Application(a) => (&a.subject, &a.role, Value::Scalar(a.object.clone())),
        CstKind::NumberAssertion(a) => (&a.subject, &a.relation, Value::Scalar(S::Number(a.value))),
        CstKind::BooleanAssertion(a) => (&a.subject, &a.relation, Value::Scalar(S::Boolean(a.value))),
        CstKind::TextAssertion(a) => (&a.subject, &a.relation, Value::Scalar(S::Text(a.value.clone()))),
        CstKind::SymbolAssertion(a) => (&a.subject, &a.relation, Value::Scalar(S::Symbol(a.value.clone()))),
        CstKind::ShapeAssertion(a) => (&a.subject, &a.relation, Value::Record(a.shape.clone(),
            a.fields.iter().map(|field| (field.name.clone(), field.value.clone())).collect())),
        CstKind::VectorAssertion(a) => (&a.subject, &a.relation, Value::Record(b"Vec3".to_vec(),
            [(b"x".to_vec(), S::Number(a.x)), (b"y".to_vec(), S::Number(a.y)), (b"z".to_vec(), S::Number(a.z))].into())),
        _ => return None,
    })
}

struct Requirement<'a> {
    role: &'a [u8],
    range: &'a [u8],
    cardinality: SourceCardinality,
}

fn check(cst: &CanonicalSourceCstV1) -> Domains {
    let mut requirements = BTreeMap::<Vec<u8>, Vec<Requirement<'_>>>::new();
    let mut referents = BTreeSet::new();
    let mut facts = BTreeMap::<(&[u8], &[u8]), BTreeSet<Value>>::new();
    let mut shapes = BTreeMap::new();
    let mut domains = Domains::default();
    for item in &cst.items {
        match &item.kind {
            CstKind::Relation(relation) if relation.contract_origin.is_some() => {
                let subject = relation.roles.iter().find(|role| Some(&role.name) == relation.subject.as_ref())
                    .expect("binary contracts declare their subject");
                let value = relation.roles.iter().find(|role| role.name != subject.name)
                    .expect("binary contracts declare their object");
                requirements.entry(subject.domain.clone()).or_default().push(Requirement {
                    role: &relation.surface, range: &value.domain,
                    cardinality: relation.modes[0].cardinality,
                });
            }
            CstKind::Referent { designation, .. } => { referents.insert(designation.clone()); }
            CstKind::Shape { designation, fields } => { shapes.insert(designation.as_slice(), fields); }
            CstKind::Application(a) => {
                referents.insert(a.subject.clone());
                if a.role == MEMBERSHIP_ROLE && let CanonicalScalarValueV1::Symbol(group) = &a.object {
                    domains.facets.entry(group.clone()).or_default().insert(a.subject.clone());
                }
            }
            _ => {}
        }
        if let Some((subject, role, value)) = fact(item) {
            referents.insert(subject.to_vec());
            facts.entry((subject, role)).or_default().insert(value);
        }
    }
    // Greatest fixed point: a recursive reference contract is satisfied only
    // if every actual property satisfies its obligations. No facts are derived
    // from circular support; missing required properties eliminate candidates.
    for domain in requirements.keys() {
        domains.structural.insert(domain.clone());
        domains.facets.insert(domain.clone(), referents.clone());
    }
    loop {
        let mut next = domains.clone();
        for (domain, requirements) in &requirements {
            next.facets.get_mut(domain).expect("initialized structural domain").retain(|subject| {
                requirements.iter().all(|requirement| {
                    let values = facts.get(&(subject.as_slice(), requirement.role));
                    let count = values.map_or(0, BTreeSet::len);
                    let cardinality = match requirement.cardinality {
                        SourceCardinality::One => count == 1,
                        SourceCardinality::Maybe => count <= 1,
                        SourceCardinality::Some => count >= 1,
                        SourceCardinality::Many => true,
                    };
                    cardinality && values.into_iter().flatten().all(|value| {
                        value_matches(value, requirement.range, &domains, &shapes)
                    })
                })
            });
        }
        if next.facets == domains.facets { return domains; }
        domains = next;
    }
}

fn scalar_matches(value: &CanonicalScalarValueV1, domain: &[u8], domains: &Domains) -> bool {
    match (value, domain) {
        (CanonicalScalarValueV1::Number(_), b"F64")
        | (CanonicalScalarValueV1::Boolean(_), b"Bool")
        | (CanonicalScalarValueV1::Text(_), b"Text") => true,
        (CanonicalScalarValueV1::Symbol(subject), _) => domains.facets.get(domain)
            .is_some_and(|subjects| subjects.contains(subject)),
        _ => false,
    }
}

fn value_matches(
    value: &Value,
    domain: &[u8],
    domains: &Domains,
    shapes: &BTreeMap<&[u8], &Vec<ShapeField>>,
) -> bool {
    match value {
        Value::Scalar(value) => scalar_matches(value, domain, domains),
        Value::Record(name, fields) => name == domain && shapes.get(domain).is_some_and(|declared| {
            fields.len() == declared.len() && declared.iter().all(|field| fields.get(&field.name)
                .is_some_and(|value| scalar_matches(value, &field.domain, domains)))
        }),
    }
}
