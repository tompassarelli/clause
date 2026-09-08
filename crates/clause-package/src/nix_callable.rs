use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CanonicalCallableV1, CanonicalExecutableExpressionV1 as E,
    CanonicalForeignEvaluationV1, CanonicalForeignOperationV1,
    CanonicalScalarValueV1 as V, CanonicalValueTypeV1 as T,
    check_canonical_callable_v1,
};

/// Physical Nix syntax built from checked common expressions. No module policy
/// or option recipe belongs to this representation.
#[derive(Clone, Debug, PartialEq)]
enum NixExpr {
    Constant(V),
    Sequence(Vec<Self>),
    Record(BTreeMap<Vec<u8>, Self>),
    Dictionary(Box<Self>, Box<Self>),
    Reference { root: String, path: Vec<String> },
    Apply(Box<Self>, Vec<Self>),
    Lambda(u16, Box<Self>),
    Bound(u16),
    Field(Box<Self>, String),
    Concatenate(Box<Self>, Box<Self>),
    Equal(Box<Self>, Box<Self>),
    Conditional(Box<Self>, Box<Self>, Box<Self>),
}

impl NixExpr {
    fn immediate(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            Self::Record(fields) => fields.values().all(Self::immediate),
            Self::Sequence(values) => values.iter().all(Self::immediate),
            Self::Dictionary(key, value) => key.immediate() && value.immediate(),
            _ => false,
        }
    }
}

/// Constructs the target expression without performing its foreign accesses.
/// Strict Clause bindings are evaluated here, before target syntax is printed.
/// Reached foreign roots are module arguments, except Nix's lexical builtins.
pub fn render_nix_callable_v1(callable: &CanonicalCallableV1) -> Result<String, String> {
    check_canonical_callable_v1(callable).map_err(|e| e.to_string())?;
    if !callable.arguments.is_empty() {
        return Err("Nix construction entry requires no unbound Clause arguments".into());
    }
    if !callable.result_kind.contains_delayed() || !callable.result_kind.in_target("nix") {
        return Err("Nix construction requires a result containing delayed Nix values".into());
    }
    let mut roots = BTreeSet::new();
    let expression = construct(&callable.expression, &BTreeMap::new(), &mut roots, 0)?;
    let arguments = roots.iter().map(String::as_str).filter(|root| *root != "builtins")
        .chain(std::iter::once("...")).collect::<Vec<_>>().join(", ");
    Ok(format!("{{ {arguments} }}:\n{}\n", render(&expression, &roots)?))
}

fn construct(e: &E, bindings: &BTreeMap<u16, NixExpr>, roots: &mut BTreeSet<String>, depth: usize) -> Result<NixExpr, String> {
    if depth >= 64 { return Err("Nix construction depth limit".into()); }
    Ok(match e {
        E::Lambda { binding, kind, body } => {
            if !matches!(kind, T::Delayed { target, .. } if target == "nix") { return Err("lambda construction target mismatch".into()); }
            let mut nested = bindings.clone();
            if nested.insert(*binding, NixExpr::Bound(*binding)).is_some() { return Err("duplicate construction binding".into()); }
            NixExpr::Lambda(*binding, Box::new(construct(body, &nested, roots, depth + 1)?))
        }
        E::Apply(function, argument) => NixExpr::Apply(Box::new(construct(function, bindings, roots, depth + 1)?), vec![construct(argument, bindings, roots, depth + 1)?]),
        E::Let { binding, value, body } => {
            let value = construct(value, bindings, roots, depth + 1)?;
            let mut nested = bindings.clone();
            if nested.insert(*binding, value).is_some() { return Err("duplicate construction binding".into()); }
            construct(body, &nested, roots, depth + 1)?
        }
        E::Binding(binding) => bindings.get(binding).ok_or("unresolved construction binding")?.clone(),
        E::Constant(value @ (V::Text(_) | V::Number(_) | V::Boolean(_))) => NixExpr::Constant(value.clone()),
        E::Sequence(values) => NixExpr::Sequence(values.iter().map(|v| construct(v, bindings, roots, depth + 1)).collect::<Result<_,_>>()?),
        E::Dictionary(key, value) => NixExpr::Dictionary(Box::new(construct(key, bindings, roots, depth + 1)?), Box::new(construct(value, bindings, roots, depth + 1)?)),
        E::Record(fields) => NixExpr::Record(fields.iter().map(|(k,v)| Ok((k.clone(),construct(v,bindings,roots,depth+1)?))).collect::<Result<_,String>>()?),
        E::Field(value, field) => {
            match construct(value, bindings, roots, depth + 1)? {
                NixExpr::Record(fields) => fields.get(field).ok_or("unknown constructed field")?.clone(),
                value => NixExpr::Field(Box::new(value), std::str::from_utf8(field).map_err(|_| "non-UTF8 field")?.into()),
            }
        }
        E::Foreign { binding, arguments } => {
            let CanonicalForeignEvaluationV1::Construct { target } = &binding.evaluation else { return Err("Nix construction cannot perform a runtime foreign attempt".into()); };
            if target != "nix" || !matches!(&binding.result, T::Delayed { target, .. } if target == "nix") {
                return Err("foreign construction target mismatch".into());
            }
            if !identifier(&binding.module) { return Err("Nix foreign root must be an identifier".into()); }
            let path = if binding.operation == CanonicalForeignOperationV1::Root { Vec::new() }
                else { binding.member.split('.').map(str::to_owned).collect::<Vec<_>>() };
            if path.iter().any(String::is_empty) { return Err("Nix foreign member path is empty".into()); }
            roots.insert(binding.module.clone());
            let reference = NixExpr::Reference { root: binding.module.clone(), path };
            let values = arguments.iter().map(|v| construct(v, bindings, roots, depth + 1)).collect::<Result<Vec<_>,_>>()?;
            match binding.operation {
                CanonicalForeignOperationV1::Get | CanonicalForeignOperationV1::Root => reference,
                CanonicalForeignOperationV1::Call => NixExpr::Apply(Box::new(reference), values),
            }
        }
        E::Conditional(condition, yes, no) => {
            match construct(condition, bindings, roots, depth + 1)? {
                NixExpr::Constant(V::Boolean(condition)) => construct(if condition { yes } else { no }, bindings, roots, depth + 1)?,
                condition => NixExpr::Conditional(Box::new(condition),
                    Box::new(construct(yes, bindings, roots, depth + 1)?),
                    Box::new(construct(no, bindings, roots, depth + 1)?)),
            }
        }
        E::Require(condition, value, message) => {
            if !boolean(construct(condition, bindings, roots, depth + 1)?)? {
                let NixExpr::Constant(V::Text(message)) = construct(message,bindings,roots,depth+1)? else { return Err("require message must be Text".into()); };
                return Err(message);
            }
            construct(value, bindings, roots, depth + 1)?
        }
        E::Equal(a,b) => {
            let a = construct(a,bindings,roots,depth+1)?;
            let b = construct(b,bindings,roots,depth+1)?;
            if a.immediate() && b.immediate() { NixExpr::Constant(V::Boolean(a == b)) }
            else { NixExpr::Equal(Box::new(a), Box::new(b)) }
        }
        E::Concatenate(a,b) => {
            match (construct(a,bindings,roots,depth+1)?,construct(b,bindings,roots,depth+1)?) {
                (NixExpr::Constant(V::Text(a)), NixExpr::Constant(V::Text(b))) => NixExpr::Constant(V::Text(a + &b)),
                (a, b) => NixExpr::Concatenate(Box::new(a), Box::new(b)),
            }
        }
        E::ScalarText(value) => match construct(value, bindings, roots, depth + 1)? {
            value @ NixExpr::Constant(V::Text(_)) => value,
            NixExpr::Constant(V::Boolean(value)) => NixExpr::Constant(V::Text(value.to_string())),
            NixExpr::Constant(V::Number(bits)) if f64::from_bits(bits).is_finite() => NixExpr::Constant(V::Text(if f64::from_bits(bits) == 0.0 { "0".into() } else { f64::from_bits(bits).to_string() })),
            value => value,
        },
        _ => return Err("expression has no Nix construction refinement".into()),
    })
}
fn boolean(value: NixExpr) -> Result<bool, String> {
    if let NixExpr::Constant(V::Boolean(value)) = value { Ok(value) }
    else { Err("construction condition requires an ordinary Boolean".into()) }
}
fn identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b"_-'".contains(&b))
}
fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}
fn render(e: &NixExpr, roots: &BTreeSet<String>) -> Result<String, String> {
    let render = |value| render(value, roots);
    let bound = |binding| {
        let mut name = format!("__clause_argument_{binding}");
        while roots.contains(&name) { name.push('_'); }
        name
    };
    Ok(match e {
        NixExpr::Lambda(binding, body) => format!("({}: {})", bound(binding), render(body)?),
        NixExpr::Bound(binding) => bound(binding),
        NixExpr::Field(value, field) => format!("({}).{}", render(value)?, quote(field)),
        NixExpr::Concatenate(a, b) => format!("({} + {})", render(a)?, render(b)?),
        NixExpr::Equal(a, b) => format!("({} == {})", render(a)?, render(b)?),
        NixExpr::Conditional(condition, yes, no) => format!("(if {} then {} else {})", render(condition)?, render(yes)?, render(no)?),
        NixExpr::Constant(V::Text(value)) => quote(value),
        NixExpr::Constant(V::Boolean(value)) => value.to_string(),
        NixExpr::Constant(V::Number(bits)) if f64::from_bits(*bits).is_finite() => f64::from_bits(*bits).to_string(),
        NixExpr::Constant(_) => return Err("unsupported Nix constant".into()),
        NixExpr::Sequence(values) => format!("[ {} ]", values.iter().map(|v| render(v).map(|v| format!("({v})"))).collect::<Result<Vec<_>,_>>()?.join(" ")),
        NixExpr::Dictionary(key, value) => format!("{{ ${{{}}} = {}; }}", render(key)?, render(value)?),
        NixExpr::Record(fields) => format!("{{ {} }}", fields.iter().map(|(k,v)| Ok(format!("{} = {};", quote(std::str::from_utf8(k).map_err(|_| "non-UTF8 field")?), render(v)?))).collect::<Result<Vec<_>,String>>()?.join(" ")),
        NixExpr::Reference { root, path } if path.is_empty() => root.clone(),
        NixExpr::Reference { root, path } => format!("{root}.{}", path.iter().map(|p| quote(p)).collect::<Vec<_>>().join(".")),
        NixExpr::Apply(function, arguments) => format!("({} {})", render(function)?, arguments.iter().map(|v| render(v).map(|v| format!("({v})"))).collect::<Result<Vec<_>,_>>()?.join(" ")),
    })
}
