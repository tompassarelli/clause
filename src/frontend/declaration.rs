use super::model::EntityCatalog;
use super::source::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct ChangeLayout<'a> {
    pub(super) from: Spanned<Name>,
    pub(super) apply: Option<Spanned<Name>>,
    pub(super) admit: Option<Vec<SourceLine<'a>>>,
    pub(super) withdraw: Option<Vec<SourceLine<'a>>>,
}

#[derive(Clone, Debug)]
pub(super) struct LawLayout<'a> {
    pub(super) conclusion: SourceLine<'a>,
    pub(super) premises: Vec<SourceLine<'a>>,
}

pub(super) fn parse_law_layout<'a>(raw: &RawDecl<'a>) -> Result<LawLayout<'a>, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    if entries.len() < 3
        || indent(entries[0])? != 4
        || indent(entries[1])? != 4
        || content(entries[1]) != "when:"
    {
        return Err(error(
            line_span(raw.header),
            "DerivationRule requires one conclusion followed by 'when:' and premises",
        ));
    }
    let premises = entries[2..].to_vec();
    if premises.is_empty()
        || premises.iter().any(|line| {
            indent(*line).expect("source indentation was validated before parsing") != 8
        })
    {
        return Err(error(
            line_span(raw.header),
            "when requires one or more eight-space clauses",
        ));
    }
    Ok(LawLayout {
        conclusion: entries[0],
        premises,
    })
}

pub(super) fn parse_change_layout<'a>(raw: &RawDecl<'a>) -> Result<ChangeLayout<'a>, ParseError> {
    let entries = nonblank(raw.body.iter().copied());
    let first = entries
        .first()
        .copied()
        .ok_or_else(|| error(line_span(raw.header), "Revision and Delta require 'from:'"))?;
    if indent(first)? != 4 || !content(first).starts_with("from: ") {
        return Err(error(
            line_span(first),
            "first member must be 'from: revision'",
        ));
    }
    let from_text = &content(first)["from: ".len()..];
    let from = qname(first, 4 + "from: ".len(), from_text)?;
    let mut apply = None;
    let mut admit = None;
    let mut withdraw = None;
    let mut index = 1;
    while index < entries.len() {
        let member = entries[index];
        if indent(member)? != 4 {
            return Err(error(line_span(member), "unexpected nested member"));
        }
        match content(member) {
            text if text.starts_with("from:") => {
                return Err(error(line_span(member), "exactly one 'from:' is required"));
            }
            text if text.starts_with("apply: ") => {
                if apply.is_some() {
                    return Err(error(
                        line_span(member),
                        "exactly one 'apply:' is permitted",
                    ));
                }
                apply = Some(qname(
                    member,
                    4 + "apply: ".len(),
                    &text["apply: ".len()..],
                )?);
                index += 1;
            }
            "admit:" | "withdraw:" => {
                let is_admit = content(member) == "admit:";
                if (is_admit && admit.is_some()) || (!is_admit && withdraw.is_some()) {
                    return Err(error(line_span(member), "change blocks occur at most once"));
                }
                index += 1;
                let start = index;
                while index < entries.len() && indent(entries[index])? == 8 {
                    index += 1;
                }
                if start == index {
                    return Err(error(
                        line_span(member),
                        "change blocks require one or more clauses",
                    ));
                }
                let clauses = entries[start..index].to_vec();
                if is_admit {
                    if withdraw.is_some() {
                        return Err(error(line_span(member), "admit must precede withdraw"));
                    }
                    admit = Some(clauses);
                } else {
                    withdraw = Some(clauses);
                }
            }
            _ => return Err(error(line_span(member), "unknown Revision or Delta member")),
        }
    }
    if apply.is_some() && (admit.is_some() || withdraw.is_some()) {
        return Err(error(
            line_span(raw.header),
            "Revision has either apply or a local change set, not both",
        ));
    }
    if apply.is_none() && admit.is_none() && withdraw.is_none() {
        return Err(error(
            line_span(raw.header),
            "Revision and Delta require a nonempty change set or apply",
        ));
    }
    if raw.kind == Kind::Delta && apply.is_some() {
        return Err(error(
            line_span(raw.header),
            "Delta cannot apply another Delta",
        ));
    }
    Ok(ChangeLayout {
        from,
        apply,
        admit,
        withdraw,
    })
}

pub(super) fn declared_model_for_law(
    name: &Name,
    models: &BTreeMap<Name, EntityCatalog>,
) -> Option<Name> {
    models
        .keys()
        .filter(|model| {
            name.0
                .strip_prefix(&format!("{}/", model.as_str()))
                .is_some()
        })
        .max_by_key(|model| model.0.len())
        .cloned()
}

pub(super) fn reference_kind(
    name: &Spanned<Name>,
    kinds: &BTreeMap<Name, Kind>,
    allowed: &[Kind],
    description: &str,
) -> Result<(), ParseError> {
    match kinds.get(&name.value) {
        Some(kind) if allowed.contains(kind) => Ok(()),
        _ => Err(error(
            name.span,
            format!("{description} '{}' is not declared", name.value.as_str()),
        )),
    }
}

pub(super) fn check_cycles(
    kinds: &BTreeMap<Name, Kind>,
    layouts: &BTreeMap<Name, ChangeLayout<'_>>,
) -> Result<(), ParseError> {
    fn visit(
        node: &Name,
        kinds: &BTreeMap<Name, Kind>,
        layouts: &BTreeMap<Name, ChangeLayout<'_>>,
        active: &mut BTreeSet<Name>,
        settled: &mut BTreeSet<Name>,
    ) -> Result<(), ParseError> {
        if settled.contains(node) {
            return Ok(());
        }
        if !active.insert(node.clone()) {
            return Err(error(
                Span {
                    line: 1,
                    column: 1,
                    width: 0,
                },
                format!("Revision/Delta dependency cycle at '{}'", node.as_str()),
            ));
        }
        let layout = layouts.get(node).expect("layout for revision or delta");
        let mut dependencies = vec![layout.from.value.clone()];
        if let Some(apply) = &layout.apply {
            dependencies.push(apply.value.clone());
        }
        for dependency in dependencies {
            if matches!(kinds.get(&dependency), Some(Kind::Revision | Kind::Delta)) {
                visit(&dependency, kinds, layouts, active, settled)?;
            }
        }
        active.remove(node);
        settled.insert(node.clone());
        Ok(())
    }
    let mut active = BTreeSet::new();
    let mut settled = BTreeSet::new();
    for node in layouts.keys() {
        visit(node, kinds, layouts, &mut active, &mut settled)?;
    }
    Ok(())
}

pub(super) fn revision_model(
    name: &Name,
    kinds: &BTreeMap<Name, Kind>,
    layouts: &BTreeMap<Name, ChangeLayout<'_>>,
) -> Name {
    match kinds[name] {
        Kind::Model => name.clone(),
        Kind::Revision => revision_model(&layouts[name].from.value, kinds, layouts),
        _ => unreachable!("validated revision reference"),
    }
}
