use super::*;

/// Recursive structural value contract. A coarse scalar kind alone does not
/// establish the element or field contracts of a composite value.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalValueTypeV1 {
    Scalar(CanonicalScalarValueKindV1),
    Delayed { target: String, value: Box<Self> },
    OpaqueForeign { module: String, name: String },
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
        fn check(kind: &CanonicalValueTypeV1, depth: usize, delayed: bool) -> Result<(), &'static str> {
            if depth >= 64 {
                return Err("value type depth limit");
            }
            match kind {
                CanonicalValueTypeV1::Scalar(
                    CanonicalScalarValueKindV1::Sequence | CanonicalScalarValueKindV1::Record,
                ) => Err("composite value requires its recursive contract"),
                CanonicalValueTypeV1::Scalar(_) => Ok(()),
                CanonicalValueTypeV1::Delayed { target, value } => {
                    if delayed || target.is_empty() || target.contains('\0') || !value.in_target(target) {
                        return Err("invalid delayed target contract");
                    }
                    check(value, depth + 1, true)
                }
                CanonicalValueTypeV1::OpaqueForeign { module, name } => {
                    if !delayed || module.is_empty() || name.is_empty() || module.contains('\0') || name.contains('\0') {
                        return Err("opaque foreign types require a delayed contract and exact external identity");
                    }
                    Ok(())
                }
                CanonicalValueTypeV1::Sequence(element) => check(element, depth + 1, delayed),
                CanonicalValueTypeV1::Record(fields) => {
                    for (name, field) in fields {
                        if name.is_empty() || std::str::from_utf8(name).is_err() {
                            return Err("invalid record field designation");
                        }
                        check(field, depth + 1, delayed)?;
                    }
                    Ok(())
                }
            }
        }
        check(self, 0, false)
    }
    pub fn contains_delayed(&self) -> bool {
        match self {
            Self::Delayed { .. } | Self::OpaqueForeign { .. } => true,
            Self::Sequence(value) => value.contains_delayed(),
            Self::Record(fields) => fields.values().any(Self::contains_delayed),
            Self::Scalar(_) => false,
        }
    }
    pub fn in_target(&self, target: &str) -> bool {
        match self {
            Self::Delayed { target: actual, value } => actual == target && value.in_target(target),
            Self::Sequence(value) => value.in_target(target),
            Self::Record(fields) => fields.values().all(|value| value.in_target(target)),
            _ => true,
        }
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
    if let Some(body) = name.strip_prefix(b"Delayed<").and_then(|s| s.strip_suffix(b">")) {
        let comma = body.iter().position(|b| *b == b',').ok_or("delayed contract needs target and value")?;
        let target = std::str::from_utf8(&body[..comma]).map_err(|_| "invalid target")?.trim().to_owned();
        let value = std::str::from_utf8(&body[comma+1..]).map_err(|_| "invalid delayed value")?.trim();
        return Ok(CanonicalValueTypeV1::Delayed { target, value: Box::new(resolve(value.as_bytes(), items, active)?) });
    }
    if let Some(kind) = items.iter().find_map(|item| match &item.kind {
        CstKind::ForeignType { designation, module, name: foreign_name } if designation == name =>
            Some(CanonicalValueTypeV1::OpaqueForeign { module: module.clone(), name: foreign_name.clone() }),
        _ => None,
    }) { return Ok(kind); }
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

/// The declaration grammar uses the same recursive value contracts as callables.
pub(super) fn designation(source: &str, origin: CanonicalSourceOriginV1) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    if let Some(inner) = source.strip_prefix("Sequence<").and_then(|s| s.strip_suffix('>')) {
        designation(inner, origin)?;
    } else if let Some(inner) = source.strip_prefix("Delayed<").and_then(|s| s.strip_suffix('>')) {
        let (target, value) = inner.split_once(',').ok_or(CanonicalSourceErrorV1::InvalidApplication { origin })?;
        application_designation_bytes(target.trim(), origin)?;
        designation(value.trim(), origin)?;
    } else {
        application_designation_bytes(source, origin)?;
    }
    Ok(source.as_bytes().to_vec())
}
