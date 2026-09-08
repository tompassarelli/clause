use super::*;

/// Recursive structural value contract. A coarse scalar kind alone does not
/// establish the element or field contracts of a composite value.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalValueTypeV1 {
    Scalar(CanonicalScalarValueKindV1),
    Delayed { target: String, value: Box<Self> },
    OpaqueForeign { module: String, name: String },
    Function { argument: Box<Self>, result: Box<Self> },
    Sequence(Box<Self>),
    Dictionary(Box<Self>),
    Record(BTreeMap<Vec<u8>, Self>),
    Alternatives(BTreeSet<Self>),
}

impl From<CanonicalScalarValueKindV1> for CanonicalValueTypeV1 {
    fn from(kind: CanonicalScalarValueKindV1) -> Self {
        Self::Scalar(kind)
    }
}

impl CanonicalValueTypeV1 {
    pub fn check(&self) -> Result<(), &'static str> {
        self.check_at(0, false)
    }

    fn check_at(&self, depth: usize, delayed: bool) -> Result<(), &'static str> {
        if depth >= 64 {
            return Err("value type depth limit");
        }
        match self {
            Self::Function { argument, result } => {
                if !delayed { return Err("function values require a delayed target contract"); }
                argument.check_at(depth + 1, true)?;
                result.check_at(depth + 1, true)
            }
            CanonicalValueTypeV1::Scalar(
                CanonicalScalarValueKindV1::Sequence | CanonicalScalarValueKindV1::Record,
            ) => Err("composite value requires its recursive contract"),
            CanonicalValueTypeV1::Scalar(_) => Ok(()),
            CanonicalValueTypeV1::Delayed { target, value } => {
                if delayed || target.is_empty() || target.contains('\0') || !value.in_target(target) {
                    return Err("invalid delayed target contract");
                }
                value.check_at(depth + 1, true)
            }
            CanonicalValueTypeV1::OpaqueForeign { module, name } => {
                if !delayed || module.is_empty() || name.is_empty() || module.contains('\0') || name.contains('\0') {
                    return Err("opaque foreign types require a delayed contract and exact external identity");
                }
                Ok(())
            }
            CanonicalValueTypeV1::Sequence(element) | CanonicalValueTypeV1::Dictionary(element) => element.check_at(depth + 1, delayed),
            CanonicalValueTypeV1::Alternatives(types) => {
                if types.len() < 2 { return Err("alternatives require at least two distinct contracts"); }
                for (index, kind) in types.iter().enumerate() {
                    kind.check_at(depth + 1, delayed)?;
                    if matches!(kind, Self::Alternatives(_)) || (!delayed && kind.contains_delayed()) {
                        return Err("alternatives require concrete immediate contracts");
                    }
                    if types.iter().skip(index + 1).any(|other| kind.overlaps(other)) {
                        return Err("alternative contracts overlap");
                    }
                }
                Ok(())
            }
            CanonicalValueTypeV1::Record(fields) => {
                for (name, field) in fields {
                    if name.is_empty() || std::str::from_utf8(name).is_err() {
                        return Err("invalid record field designation");
                    }
                    field.check_at(depth + 1, delayed)?;
                }
                Ok(())
            }
        }
    }
    pub(super) fn includes_alternatives(&self, actual: &Self) -> bool {
        let Self::Alternatives(expected) = self else { return false; };
        match actual {
            Self::Alternatives(actual) => actual.is_subset(expected),
            actual => expected.contains(actual),
        }
    }
    fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(a), Self::Scalar(b)) => a == b,
            (Self::OpaqueForeign { module: a, name: an }, Self::OpaqueForeign { module: b, name: bn }) => a == b && an == bn,
            (Self::Sequence(_), Self::Sequence(_)) | (Self::Dictionary(_), Self::Dictionary(_)) => true,
            (Self::Dictionary(element), Self::Record(fields)) | (Self::Record(fields), Self::Dictionary(element)) => fields.values().all(|value| element.overlaps(value)),
            (Self::Record(a), Self::Record(b)) => a.keys().eq(b.keys()) && a.iter().all(|(key, value)| value.overlaps(&b[key])),
            (Self::Alternatives(types), other) | (other, Self::Alternatives(types)) => types.iter().any(|kind| kind.overlaps(other)),
            _ => false,
        }
    }
    pub fn contains_delayed(&self) -> bool {
        match self {
            Self::Function { argument, result } => argument.contains_delayed() || result.contains_delayed(),
            Self::Delayed { .. } | Self::OpaqueForeign { .. } => true,
            Self::Sequence(value) | Self::Dictionary(value) => value.contains_delayed(),
            Self::Record(fields) => fields.values().any(Self::contains_delayed),
            Self::Alternatives(types) => types.iter().any(Self::contains_delayed),
            Self::Scalar(_) => false,
        }
    }
    pub fn in_target(&self, target: &str) -> bool {
        match self {
            Self::Function { argument, result } => argument.in_target(target) && result.in_target(target),
            Self::Delayed { target: actual, value } => actual == target && value.in_target(target),
            Self::Sequence(value) | Self::Dictionary(value) => value.in_target(target),
            Self::Record(fields) => fields.values().all(|value| value.in_target(target)),
            Self::Alternatives(types) => types.iter().all(|kind| kind.in_target(target)),
            _ => true,
        }
    }
    pub(super) fn constructed_value(&self, target: &str) -> Result<Self, &'static str> {
        Ok(match self {
            Self::Delayed { target: actual, value } => {
                if actual != target { return Err("dictionary construction target mismatch"); }
                value.constructed_value(target)?
            }
            Self::Sequence(value) => Self::Sequence(Box::new(value.constructed_value(target)?)),
            Self::Dictionary(value) => Self::Dictionary(Box::new(value.constructed_value(target)?)),
            Self::Record(fields) => Self::Record(fields.iter().map(|(key, value)| Ok((key.clone(), value.constructed_value(target)?))).collect::<Result<_, &'static str>>()?),
            _ => self.clone(),
        })
    }
    pub fn accepts(&self, value: &CanonicalScalarValueV1) -> bool {
        use CanonicalScalarValueKindV1 as K;
        use CanonicalScalarValueV1 as V;
        match (self, value) {
            (Self::Alternatives(types), value) => types.iter().any(|kind| kind.accepts(value)),
            (Self::Scalar(K::Number), V::Number(bits)) => f64::from_bits(*bits).is_finite(),
            (Self::Scalar(K::Boolean), V::Boolean(_))
            | (Self::Scalar(K::Text), V::Text(_))
            | (Self::Scalar(K::Symbol), V::Symbol(_))
            | (Self::Scalar(K::Referent), V::Referent(_))
            | (Self::Scalar(K::RelationTable), V::RelationTable(_)) => true,
            (Self::Sequence(element), V::Sequence(values)) => {
                values.iter().all(|v| element.accepts(v))
            }
            (Self::Dictionary(element), V::Record(values)) => values.iter().all(|(key, value)| std::str::from_utf8(key).is_ok() && element.accepts(value)),
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

/// Source-level quantified contracts are instantiated before canonical checking.
/// A record parameter binds one entire exact type, including every nested field.
#[derive(Clone, Debug)]
pub(super) enum Pattern {
    Exact(CanonicalValueTypeV1),
    RecordParameter(Vec<u8>),
    Sequence(Box<Self>),
    Dictionary(Box<Self>),
    Delayed { target: String, value: Box<Self> },
}

impl Pattern {
    pub(super) fn read(
        name: &[u8],
        items: &[CstItem],
        parameters: &BTreeSet<Vec<u8>>,
    ) -> Result<Self, &'static str> {
        fn read(name: &[u8], items: &[CstItem], parameters: &BTreeSet<Vec<u8>>, depth: usize) -> Result<Pattern, &'static str> {
            if depth >= 64 { return Err("value type depth limit"); }
            if parameters.contains(name) { return Ok(Pattern::RecordParameter(name.to_vec())); }
            if let Some(inner) = name.strip_prefix(b"Dictionary<").and_then(|s| s.strip_suffix(b">")) {
                return Ok(Pattern::Dictionary(Box::new(read(inner, items, parameters, depth + 1)?)));
            }
            if let Some(inner) = name.strip_prefix(b"Sequence<").and_then(|s| s.strip_suffix(b">")) {
                return Ok(Pattern::Sequence(Box::new(read(inner, items, parameters, depth + 1)?)));
            }
            if let Some(inner) = name.strip_prefix(b"Delayed<").and_then(|s| s.strip_suffix(b">")) {
                let source = std::str::from_utf8(inner).map_err(|_| "invalid delayed value")?;
                let (target, value) = source.split_once(',').ok_or("delayed contract needs target and value")?;
                return Ok(Pattern::Delayed {
                    target: target.trim().into(),
                    value: Box::new(read(value.trim().as_bytes(), items, parameters, depth + 1)?),
                });
            }
            resolve(name, items, &mut BTreeSet::new()).map(Pattern::Exact)
        }
        read(name, items, parameters, 0)
    }

    pub(super) fn contains(&self, parameter: &[u8]) -> bool {
        match self {
            Self::RecordParameter(name) => name == parameter,
            Self::Sequence(value) | Self::Dictionary(value) | Self::Delayed { value, .. } => value.contains(parameter),
            Self::Exact(_) => false,
        }
    }

    pub(super) fn check(&self) -> Result<(), &'static str> {
        fn check(pattern: &Pattern, depth: usize, delayed: bool) -> Result<(), &'static str> {
            if depth >= 64 { return Err("value type depth limit"); }
            match pattern {
                Pattern::Exact(kind) => kind.check_at(depth, delayed),
                Pattern::RecordParameter(_) => Ok(()),
                Pattern::Sequence(value) | Pattern::Dictionary(value) => check(value, depth + 1, delayed),
                Pattern::Delayed { target, value } => {
                    if delayed || target.is_empty() || target.contains('\0') || !value.in_target(target) {
                        return Err("invalid delayed target contract");
                    }
                    check(value, depth + 1, true)
                }
            }
        }
        check(self, 0, false)
    }

    pub(super) fn contains_delayed(&self) -> bool {
        match self {
            Self::Exact(kind) => kind.contains_delayed(),
            Self::Delayed { .. } => true,
            Self::Sequence(value) | Self::Dictionary(value) => value.contains_delayed(),
            Self::RecordParameter(_) => false,
        }
    }

    pub(super) fn in_target(&self, target: &str) -> bool {
        match self {
            Self::Exact(kind) => kind.in_target(target),
            Self::Delayed { target: actual, value } => actual == target && value.in_target(target),
            Self::Sequence(value) | Self::Dictionary(value) => value.in_target(target),
            Self::RecordParameter(_) => true,
        }
    }

    pub(super) fn instantiate(
        &self,
        substitutions: &BTreeMap<Vec<u8>, CanonicalValueTypeV1>,
    ) -> Result<CanonicalValueTypeV1, &'static str> {
        Ok(match self {
            Self::Exact(kind) => kind.clone(),
            Self::RecordParameter(name) => substitutions.get(name).ok_or("unresolved record type parameter")?.clone(),
            Self::Dictionary(value) => CanonicalValueTypeV1::Dictionary(Box::new(value.instantiate(substitutions)?)),
            Self::Sequence(value) => CanonicalValueTypeV1::Sequence(Box::new(value.instantiate(substitutions)?)),
            Self::Delayed { target, value } => CanonicalValueTypeV1::Delayed {
                target: target.clone(), value: Box::new(value.instantiate(substitutions)?),
            },
        })
    }

    pub(super) fn unify(
        &self,
        actual: &CanonicalValueTypeV1,
        substitutions: &mut BTreeMap<Vec<u8>, CanonicalValueTypeV1>,
    ) -> Result<(), &'static str> {
        match (self, actual) {
            (Self::Exact(expected), actual) if expected == actual => Ok(()),
            (Self::RecordParameter(name), CanonicalValueTypeV1::Record(_)) => {
                if let Some(previous) = substitutions.get(name) {
                    if previous != actual { return Err("record type parameter has conflicting argument types"); }
                } else {
                    substitutions.insert(name.clone(), actual.clone());
                }
                Ok(())
            }
            (Self::Dictionary(pattern), CanonicalValueTypeV1::Dictionary(value)) => pattern.unify(value, substitutions),
            (Self::Sequence(pattern), CanonicalValueTypeV1::Sequence(value)) => pattern.unify(value, substitutions),
            (Self::Delayed { target, value: pattern }, CanonicalValueTypeV1::Delayed { target: actual, value })
                if target == actual => pattern.unify(value, substitutions),
            _ => Err("callable argument type mismatch"),
        }
    }
}

pub(super) fn resolve<Item: std::borrow::Borrow<CstItem>>(
    name: &[u8],
    items: &[Item],
    active: &mut BTreeSet<Vec<u8>>,
) -> Result<CanonicalValueTypeV1, &'static str> {
    use CanonicalScalarValueKindV1 as K;
    let source = std::str::from_utf8(name).map_err(|_| "invalid value contract")?.trim();
    if source == "{}" {
        return Ok(CanonicalValueTypeV1::Record(BTreeMap::new()));
    }
    let parts = alternatives(source);
    if parts.len() > 1 {
        let mut types = BTreeSet::new();
        for part in parts {
            let kind = resolve(part.trim().as_bytes(), items, active)?;
            if !types.insert(kind) { return Err("duplicate alternative contract"); }
        }
        return Ok(CanonicalValueTypeV1::Alternatives(types));
    }
    let name = source.as_bytes();
    let scalar = match name {
        b"Text" => Some(K::Text),
        b"F64" => Some(K::Number),
        b"Bool" => Some(K::Boolean),
        _ => None,
    };
    if let Some(kind) = scalar {
        return Ok(kind.into());
    }
    if let Some(body) = source.strip_prefix("Function<").and_then(|s| s.strip_suffix('>')) {
        let (argument, result) = function_parts(body).ok_or("function contract needs argument and result")?;
        return Ok(CanonicalValueTypeV1::Function {
            argument: Box::new(resolve(argument.trim().as_bytes(), items, active)?),
            result: Box::new(resolve(result.trim().as_bytes(), items, active)?),
        });
    }
    if let Some(body) = name.strip_prefix(b"Delayed<").and_then(|s| s.strip_suffix(b">")) {
        let comma = body.iter().position(|b| *b == b',').ok_or("delayed contract needs target and value")?;
        let target = std::str::from_utf8(&body[..comma]).map_err(|_| "invalid target")?.trim().to_owned();
        let value = std::str::from_utf8(&body[comma+1..]).map_err(|_| "invalid delayed value")?.trim();
        return Ok(CanonicalValueTypeV1::Delayed { target, value: Box::new(resolve(value.as_bytes(), items, active)?) });
    }
    if let Some(kind) = items.iter().find_map(|item| match &item.borrow().kind {
        CstKind::ForeignType { designation, module, name: foreign_name } if designation == name =>
            Some(CanonicalValueTypeV1::OpaqueForeign { module: module.clone(), name: foreign_name.clone() }),
        _ => None,
    }) { return Ok(kind); }
    if let Some(element) = name.strip_prefix(b"Dictionary<").and_then(|s| s.strip_suffix(b">")) {
        return resolve(element, items, active).map(|t| CanonicalValueTypeV1::Dictionary(Box::new(t)));
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
        .find_map(|item| match &item.borrow().kind {
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
    if source == "{}" { return Ok(source.as_bytes().to_vec()); }
    let parts = alternatives(source);
    if parts.len() > 1 {
        for part in parts { designation(part.trim(), origin)?; }
    } else if let Some(inner) = source.strip_prefix("Function<").and_then(|s| s.strip_suffix('>')) {
        let (argument, result) = function_parts(inner).ok_or(CanonicalSourceErrorV1::InvalidApplication { origin })?;
        designation(argument.trim(), origin)?;
        designation(result.trim(), origin)?;
    } else if let Some(inner) = source.strip_prefix("Dictionary<").and_then(|s| s.strip_suffix('>')) {
        designation(inner, origin)?;
    } else if let Some(inner) = source.strip_prefix("Sequence<").and_then(|s| s.strip_suffix('>')) {
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

fn function_parts(source: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    for (index, byte) in source.bytes().enumerate() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => return Some((&source[..index], &source[index + 1..])),
            _ => {}
        }
    }
    None
}

fn alternatives(source: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut start = 0;
    let mut parts = Vec::new();
    for (index, byte) in source.bytes().enumerate() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b'|' if depth == 0 => { parts.push(&source[start..index]); start = index + 1; }
            _ => {}
        }
    }
    parts.push(&source[start..]);
    parts
}
