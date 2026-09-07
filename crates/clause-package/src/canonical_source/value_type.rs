use super::*;

/// Recursive structural value contract. A coarse scalar kind alone does not
/// establish the element or field contracts of a composite value.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalValueTypeV1 {
    Scalar(CanonicalScalarValueKindV1),
    Sequence(Box<Self>),
    Record(BTreeMap<Vec<u8>, Self>),
}

impl From<CanonicalScalarValueKindV1> for CanonicalValueTypeV1 {
    fn from(kind: CanonicalScalarValueKindV1) -> Self {
        Self::Scalar(kind)
    }
}

impl CanonicalValueTypeV1 {
    pub fn check(&self) -> Result<(), &'static str> {
        fn check(kind: &CanonicalValueTypeV1, depth: usize) -> Result<(), &'static str> {
            if depth >= 64 {
                return Err("value type depth limit");
            }
            match kind {
                CanonicalValueTypeV1::Scalar(
                    CanonicalScalarValueKindV1::Sequence | CanonicalScalarValueKindV1::Record,
                ) => Err("composite value requires its recursive contract"),
                CanonicalValueTypeV1::Scalar(_) => Ok(()),
                CanonicalValueTypeV1::Sequence(element) => check(element, depth + 1),
                CanonicalValueTypeV1::Record(fields) => {
                    for (name, field) in fields {
                        if name.is_empty() || std::str::from_utf8(name).is_err() {
                            return Err("invalid record field designation");
                        }
                        check(field, depth + 1)?;
                    }
                    Ok(())
                }
            }
        }
        check(self, 0)
    }
    pub fn accepts(&self, value: &CanonicalScalarValueV1) -> bool {
        use CanonicalScalarValueKindV1 as K;
        use CanonicalScalarValueV1 as V;
        match (self, value) {
            (Self::Scalar(K::Number), V::Number(bits)) => f64::from_bits(*bits).is_finite(),
            (Self::Scalar(K::Boolean), V::Boolean(_))
            | (Self::Scalar(K::Text), V::Text(_))
            | (Self::Scalar(K::Symbol), V::Symbol(_))
            | (Self::Scalar(K::Referent), V::Referent(_))
            | (Self::Scalar(K::RelationTable), V::RelationTable(_)) => true,
            (Self::Sequence(element), V::Sequence(values)) => {
                values.iter().all(|v| element.accepts(v))
            }
            (Self::Record(fields), V::Record(values)) => {
                fields.len() == values.len()
                    && fields
                        .iter()
                        .all(|(name, field)| values.get(name).is_some_and(|v| field.accepts(v)))
            }
            _ => false,
        }
    }
}

pub(super) fn resolve(
    name: &[u8],
    items: &[CstItem],
    active: &mut BTreeSet<Vec<u8>>,
) -> Result<CanonicalValueTypeV1, &'static str> {
    use CanonicalScalarValueKindV1 as K;
    let scalar = match name {
        b"Text" => Some(K::Text),
        b"F64" => Some(K::Number),
        b"Bool" => Some(K::Boolean),
        _ => None,
    };
    if let Some(kind) = scalar {
        return Ok(kind.into());
    }
    if let Some(element) = name
        .strip_prefix(b"Sequence<")
        .and_then(|s| s.strip_suffix(b">"))
    {
        return resolve(element, items, active)
            .map(|t| CanonicalValueTypeV1::Sequence(Box::new(t)));
    }
    if !active.insert(name.to_vec()) {
        return Err("recursive value shape is unsupported");
    }
    let fields = items
        .iter()
        .find_map(|item| match &item.kind {
            CstKind::Shape {
                designation,
                fields,
            } if designation == name => Some(fields),
            _ => None,
        })
        .ok_or("unknown callable value type")?;
    let mut resolved = BTreeMap::new();
    for field in fields {
        if resolved
            .insert(field.name.clone(), resolve(&field.domain, items, active)?)
            .is_some()
        {
            return Err("duplicate record field contract");
        }
    }
    active.remove(name);
    Ok(CanonicalValueTypeV1::Record(resolved))
}
