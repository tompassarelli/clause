use super::clause::*;
use super::declaration::*;
use super::model::*;
use super::relation::*;
use super::source::*;
use super::syntax::{DefinitionDecl, MembershipDecl};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let (raw_declarations, raw_requests) = scan(source)?;
    let mut kinds = BTreeMap::new();
    for declaration in &raw_declarations {
        if kinds
            .insert(declaration.subject.value.clone(), declaration.kind)
            .is_some()
        {
            return Err(error(
                declaration.subject.span,
                format!(
                    "duplicate declaration '{}'",
                    declaration.subject.value.as_str()
                ),
            ));
        }
    }
    let types = raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Type)
        .map(|declaration| declaration.subject.value.clone())
        .collect::<BTreeSet<_>>();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Type)
    {
        if nonblank(declaration.body.iter().copied()).is_empty() {
            continue;
        }
        return Err(error(
            line_span(declaration.header),
            "Type declarations cannot have members",
        ));
    }
    let mut relations = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::RelationShape)
    {
        let spec = relation_spec(declaration)?;
        for typ in spec.roles.values() {
            if !types.contains(&Name(typ.0.clone())) {
                return Err(error(
                    line_span(declaration.header),
                    format!("unknown role type '{}'", typ.as_str()),
                ));
            }
        }
        relations.insert(declaration.subject.value.clone(), spec);
    }
    let mut entities = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Model)
    {
        let model_entities = model_entities(declaration)?;
        for typ in model_entities
            .explicit
            .values()
            .chain(model_entities.groups.iter().map(|group| &group.typ.value))
        {
            if !types.contains(&Name(typ.0.clone())) {
                return Err(error(
                    line_span(declaration.header),
                    format!("unknown entity type '{}'", typ.as_str()),
                ));
            }
        }
        entities.insert(declaration.subject.value.clone(), model_entities);
    }
    let mut layouts = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| matches!(declaration.kind, Kind::Revision | Kind::Delta))
    {
        layouts.insert(
            declaration.subject.value.clone(),
            parse_change_layout(declaration)?,
        );
    }
    for (name, layout) in &layouts {
        reference_kind(
            &layout.from,
            &kinds,
            &[Kind::Model, Kind::Revision],
            "revision base",
        )?;
        if let Some(apply) = &layout.apply {
            reference_kind(apply, &kinds, &[Kind::Delta], "applied Delta")?;
            if kinds[name] == Kind::Delta {
                return Err(error(apply.span, "Delta cannot apply another Delta"));
            }
        }
    }
    check_cycles(&kinds, &layouts)?;

    let mut declarations = Vec::new();
    for raw in &raw_declarations {
        let body = match raw.kind {
            Kind::Type => Vec::new(),
            Kind::RelationShape => {
                let spec = relations
                    .get(&raw.subject.value)
                    .expect("relation spec exists");
                let mut members = vec![Member::Sentence(spec.shape.clone())];
                members.extend(spec.modes.iter().cloned().map(Member::LookupMode));
                members
            }
            Kind::Model => {
                let mut members = Vec::new();
                let mut variables = BTreeMap::new();
                let entries = nonblank(raw.body.iter().copied());
                let mut index = 0;
                while index < entries.len() {
                    let line = entries[index];
                    if indent(line)? != 2 {
                        return Err(error(
                            line_span(line),
                            "Model members must use two-space indentation",
                        ));
                    }
                    if let Some(group) = entity_group_line(line) {
                        members.push(Member::EntityGroup(group?));
                        index += 1;
                    } else if let Some(template) = focus_template(line) {
                        let template = template?;
                        index += 1;
                        let slot_start = index;
                        while index < entries.len() && indent(entries[index])? == 4 {
                            index += 1;
                        }
                        if slot_start == index {
                            return Err(error(
                                line_span(line),
                                "focus block requires one or more slots",
                            ));
                        }
                        let slots = entries[slot_start..index]
                            .iter()
                            .copied()
                            .map(focus_slot)
                            .collect::<Result<Vec<_>, _>>()?;
                        let binding_line = entries.get(index).copied().ok_or_else(|| {
                            error(line_span(line), "focus block requires a binding")
                        })?;
                        let binding = focus_binding(binding_line)?;
                        if binding.variable.value != template.variable.value {
                            return Err(error(
                                binding.variable.span,
                                format!(
                                    "focus binding '{}' does not match template variable '{}'",
                                    binding.variable.value.as_str(),
                                    template.variable.value.as_str()
                                ),
                            ));
                        }
                        members.push(Member::Focus(FocusBlock {
                            template,
                            slots,
                            binding,
                            span: line_span(line),
                        }));
                        index += 1;
                    } else if let Some(membership) = membership_line(line) {
                        members.push(Member::Membership(membership?));
                        index += 1;
                    } else if let Some(definition) = definition_line(line) {
                        members.push(Member::Definition(definition?));
                        index += 1;
                    } else if content(line).starts_with("for ") {
                        return Err(error(
                            line_span(line),
                            "focus binding has no preceding focus block",
                        ));
                    } else if entries
                        .get(index + 1)
                        .is_some_and(|child| indent(*child).is_ok_and(|width| width == 4))
                    {
                        let focus = focused_name(line)?;
                        index += 1;
                        while index < entries.len() && indent(entries[index])? == 4 {
                            let child = entries[index];
                            if content(child).split_ascii_whitespace().count() == 1 {
                                members.push(Member::Membership(MembershipDecl {
                                    member: focus.clone(),
                                    group: focused_name(child)?,
                                    span: line_span(child),
                                }));
                            } else if let Some(definition) = definition_line(child) {
                                let definition = definition?;
                                members.push(Member::Definition(DefinitionDecl {
                                    name: Spanned {
                                        value: Name(format!(
                                            "{} of {}",
                                            definition.name.value.as_str(),
                                            focus.value.as_str()
                                        )),
                                        span: definition.name.span,
                                    },
                                    denotation: definition.denotation,
                                    span: definition.span,
                                }));
                            } else {
                                let expanded =
                                    format!("{} {}", focus.value.as_str(), content(child));
                                let parsed = clause(
                                    SourceLine {
                                        number: child.number,
                                        text: &expanded,
                                    },
                                    &raw.subject.value,
                                    &relations,
                                    &entities,
                                    &mut variables,
                                )?;
                                if !ground(&parsed) {
                                    return Err(error(
                                        parsed.span,
                                        "model assertions must be closed",
                                    ));
                                }
                                members.push(Member::RelationalContent(parsed));
                            }
                            index += 1;
                        }
                    } else {
                        let parsed = clause(
                            line,
                            &raw.subject.value,
                            &relations,
                            &entities,
                            &mut variables,
                        )?;
                        if !ground(&parsed) {
                            return Err(error(parsed.span, "model assertions must be closed"));
                        }
                        members.push(Member::RelationalContent(parsed));
                        index += 1;
                    }
                }
                members
            }
            Kind::DerivationRule => {
                let layout = parse_law_layout(raw)?;
                let model =
                    declared_model_for_law(&raw.subject.value, &entities).ok_or_else(|| {
                        error(
                            raw.subject.span,
                            "DerivationRule name must be in a declared Model namespace",
                        )
                    })?;
                let mut variable_types = BTreeMap::new();
                let conclusion = clause(
                    layout.conclusion,
                    &model,
                    &relations,
                    &entities,
                    &mut variable_types,
                )?;
                let premises = layout
                    .premises
                    .iter()
                    .copied()
                    .map(|line| clause(line, &model, &relations, &entities, &mut variable_types))
                    .collect::<Result<Vec<_>, _>>()?;
                let premise_variables =
                    premises.iter().flat_map(variables).collect::<BTreeSet<_>>();
                if !variables(&conclusion).is_subset(&premise_variables) {
                    return Err(error(
                        conclusion.span,
                        "DerivationRule conclusion variables must be range-restricted by when",
                    ));
                }
                vec![
                    Member::RelationalContent(conclusion),
                    Member::When(premises),
                ]
            }
            Kind::Revision | Kind::Delta => {
                let layout = layouts
                    .get(&raw.subject.value)
                    .expect("change layout exists");
                let model = revision_model(&layout.from.value, &kinds, &layouts);
                let mut members = vec![Member::From(layout.from.value.clone())];
                if let Some(apply) = &layout.apply {
                    members.push(Member::Apply(apply.value.clone()));
                } else {
                    let mut admitted = BTreeSet::new();
                    let mut withdrawn = BTreeSet::new();
                    if let Some(lines) = &layout.admit {
                        let mut variables = BTreeMap::new();
                        let clauses = lines
                            .iter()
                            .copied()
                            .map(|line| clause(line, &model, &relations, &entities, &mut variables))
                            .collect::<Result<Vec<_>, _>>()?;
                        for parsed in &clauses {
                            if !ground(parsed) {
                                return Err(error(parsed.span, "changes must be closed"));
                            }
                            admitted.insert(clause_key(parsed));
                        }
                        members.push(Member::Admit(clauses));
                    }
                    if let Some(lines) = &layout.withdraw {
                        let mut variables = BTreeMap::new();
                        let clauses = lines
                            .iter()
                            .copied()
                            .map(|line| clause(line, &model, &relations, &entities, &mut variables))
                            .collect::<Result<Vec<_>, _>>()?;
                        for parsed in &clauses {
                            if !ground(parsed) {
                                return Err(error(parsed.span, "changes must be closed"));
                            }
                            let key = clause_key(parsed);
                            if !withdrawn.insert(key.clone()) {
                                return Err(error(parsed.span, "duplicate withdrawal"));
                            }
                            if admitted.contains(&key) {
                                return Err(error(
                                    parsed.span,
                                    "admit and withdraw cannot overlap",
                                ));
                            }
                        }
                        members.push(Member::Withdraw(clauses));
                    }
                }
                members
            }
        };
        declarations.push(AscriptionDecl {
            subject: raw.subject.clone(),
            kind: raw.kind,
            body,
            span: line_span(raw.header),
        });
    }

    let mut requests = Vec::new();
    for raw in raw_requests {
        match raw {
            RawRequest::Find {
                revision,
                sought,
                clause: line,
                header,
            } => {
                reference_kind(
                    &revision,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "request revision",
                )?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let pattern = clause(line, &model, &relations, &entities, &mut variables)?;
                if !variables.contains_key(&sought.value) {
                    return Err(error(sought.span, "find variable must occur in its clause"));
                }
                requests.push(RequestDecl::Find {
                    revision,
                    pattern,
                    sought,
                    span: line_span(header),
                });
            }
            RawRequest::Why {
                revision,
                all,
                clause: line,
                header,
            } => {
                reference_kind(
                    &revision,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "request revision",
                )?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let target = clause(line, &model, &relations, &entities, &mut variables)?;
                if !ground(&target) {
                    return Err(error(target.span, "why target must be closed"));
                }
                requests.push(RequestDecl::Why {
                    revision,
                    target,
                    all,
                    span: line_span(header),
                });
            }
            RawRequest::Intervention {
                verb,
                revision,
                selection,
                clause: line,
                using,
                header,
            } => {
                reference_kind(
                    &revision,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "request revision",
                )?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let target = clause(line, &model, &relations, &entities, &mut variables)?;
                if !ground(&target) {
                    return Err(error(target.span, "intervention target must be closed"));
                }
                let mut seen = BTreeSet::new();
                for relation in &using {
                    reference_kind(relation, &kinds, &[Kind::RelationShape], "using relation")?;
                    if !seen.insert(relation.value.clone()) {
                        return Err(error(relation.span, "using relations must be unique"));
                    }
                }
                let request = match verb {
                    "prevent" => RequestDecl::Prevent {
                        revision,
                        target,
                        selection,
                        using,
                        span: line_span(header),
                    },
                    "achieve" => RequestDecl::Achieve {
                        revision,
                        target,
                        selection,
                        using,
                        span: line_span(header),
                    },
                    _ => unreachable!("known intervention verb"),
                };
                requests.push(request);
            }
            RawRequest::Diff {
                base,
                successor,
                header,
            } => {
                reference_kind(&base, &kinds, &[Kind::Model, Kind::Revision], "diff base")?;
                reference_kind(
                    &successor,
                    &kinds,
                    &[Kind::Model, Kind::Revision],
                    "diff successor",
                )?;
                requests.push(RequestDecl::Diff {
                    base,
                    successor,
                    span: line_span(header),
                });
            }
        }
    }
    Ok(Program {
        declarations,
        requests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r#"Module: Type
Change: Type

impact/imports: RelationShape
    {consumer: Module} imports {dependency: Module}
    mode consumer -> dependency: many

impact/affects: RelationShape
    {change: Change} affects {consumer: Module}
    mode change -> consumer: many

impact: Model
    North: Module
    Store: Module
    compiler-change: Change
    North imports Store

impact/direct: DerivationRule
    ?consumer imports ?dependency
    when:
        ?consumer imports ?dependency

impact/adopt: Revision
    from: impact
    admit:
        Store imports North

find all ?consumer in impact:
    compiler-change affects ?consumer

why all in impact:
    North imports Store

prevent all minimal in impact:
    North imports Store
using:
    impact/imports

achieve one minimal in impact/adopt:
    Store imports North
using:
    impact/imports

diff impact -> impact/adopt
"#;

    #[test]
    fn parses_the_singular_surface_in_declaration_independent_order() {
        let source = format!(
            "find all ?consumer in impact:\n    compiler-change affects ?consumer\n\n{}\nModule: Type\nChange: Type\n",
            SOURCE
                .replace("Module: Type\nChange: Type\n\n", "")
                .replace(
                    "find all ?consumer in impact:\n    compiler-change affects ?consumer\n\n",
                    ""
                )
        );
        let program = parse(&source).expect("native source parses");
        assert_eq!(program.declarations.len(), 7);
        assert_eq!(program.requests.len(), 5);
        let relation = program
            .declarations
            .iter()
            .find(|declaration| declaration.subject.value.as_str() == "impact/imports")
            .expect("relation exists");
        assert_eq!(relation.subject.value.as_str(), "impact/imports");
        assert!(matches!(relation.body[0], Member::Sentence(_)));
        assert!(matches!(program.requests[0], RequestDecl::Find { .. }));
    }

    #[test]
    fn keeps_when_and_intervention_selection() {
        let program = parse(SOURCE).expect("native source parses");
        let law = program
            .declarations
            .iter()
            .find(|declaration| declaration.kind == Kind::DerivationRule)
            .expect("law exists");
        assert!(matches!(law.body[1], Member::When(_)));
        assert!(matches!(
            program.requests[2],
            RequestDecl::Prevent {
                selection: InterventionSelection::AllMinimal,
                ..
            }
        ));
        assert!(matches!(
            program.requests[3],
            RequestDecl::Achieve {
                selection: InterventionSelection::OneMinimal,
                ..
            }
        ));
    }

    #[test]
    fn parses_reusable_delta_and_revision_apply() {
        let source = SOURCE.replace(
            "impact/adopt: Revision\n    from: impact\n    admit:\n        Store imports North",
            "impact/remove: Delta\n    from: impact\n    withdraw:\n        North imports Store\n\nimpact/adopt: Revision\n    from: impact\n    apply: impact/remove",
        );
        let program = parse(&source).expect("Delta applies from the same base");
        assert!(
            program
                .declarations
                .iter()
                .any(|decl| decl.kind == Kind::Delta)
        );
    }

    #[test]
    fn rejects_retired_prefixes_and_sentence_members() {
        for prefix in [
            "relation", "model", "law", "intent", "query", "claim", "require", "fact",
        ] {
            assert!(
                parse(&format!("{prefix} retired:")).is_err(),
                "{} is retired",
                prefix
            );
        }
        assert!(
            parse(&SOURCE.replace(
                "    {consumer: Module} imports {dependency: Module}",
                "    sentence: {consumer} imports {dependency}",
            ))
            .is_err()
        );
    }

    #[test]
    fn rejects_non_four_space_indentation_and_tabs() {
        assert!(parse(&SOURCE.replace("    North: Module", "  North: Module")).is_err());
        assert!(parse(&SOURCE.replace("    North: Module", "\tNorth: Module")).is_err());
    }

    #[test]
    fn rejects_invalid_shapes_and_modes() {
        assert!(
            parse(&SOURCE.replace(
                "{consumer: Module} imports {dependency: Module}",
                "{consumer: Module} {dependency: Module}"
            ))
            .is_err()
        );
        assert!(
            parse(&SOURCE.replace(
                "mode consumer -> dependency: many",
                "mode consumer -> consumer: many"
            ))
            .is_err()
        );
        assert!(
            parse(&SOURCE.replace(
                "{consumer: Module} imports {dependency: Module}",
                "{consumer: Module} imports {consumer: Module}"
            ))
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_or_wrongly_typed_entities_and_quoted_modules() {
        assert!(parse(&SOURCE.replace("North imports Store", "Missing imports Store")).is_err());
        assert!(
            parse(&SOURCE.replace("North imports Store", "compiler-change imports Store")).is_err()
        );
        assert!(parse(&SOURCE.replace("North imports Store", "\"North\" imports Store")).is_err());
    }

    #[test]
    fn rejects_inconsistent_law_variables() {
        let source = SOURCE.replace(
            "?consumer imports ?dependency\n    when:\n        ?consumer imports ?dependency",
            "?consumer imports ?dependency\n    when:\n        compiler-change affects ?consumer",
        );
        assert!(parse(&source).is_err());
    }

    #[test]
    fn preserves_duplicate_admissions_but_rejects_overlaps() {
        let duplicate = SOURCE.replace(
            "        Store imports North",
            "        Store imports North\n        Store imports North",
        );
        assert!(parse(&duplicate).is_ok());
        let overlap = SOURCE.replace(
            "    admit:\n        Store imports North",
            "    admit:\n        Store imports North\n    withdraw:\n        Store imports North",
        );
        assert!(parse(&overlap).is_err());
    }

    #[test]
    fn accepts_apply_with_different_revision_aliases() {
        let source = SOURCE.replace(
            "impact/adopt: Revision\n    from: impact\n    admit:\n        Store imports North",
            "impact/alias-left: Revision\n    from: impact\n    admit:\n        Store imports North\n\nimpact/alias-right: Revision\n    from: impact\n    admit:\n        Store imports North\n\nimpact/remove: Delta\n    from: impact/alias-left\n    withdraw:\n        North imports Store\n\nimpact/adopt: Revision\n    from: impact/alias-right\n    apply: impact/remove",
        );
        assert!(parse(&source).is_ok());
    }

    #[test]
    fn rejects_cycles_and_bad_request_references() {
        let cycle = SOURCE.replace("impact/adopt: Revision\n    from: impact\n    admit:\n        Store imports North", "impact/first: Revision\n    from: impact/second\n    admit:\n        Store imports North\n\nimpact/second: Revision\n    from: impact/first\n    admit:\n        Store imports North");
        assert!(parse(&cycle).is_err());
        assert!(
            parse(&SOURCE.replace("using:\n    impact/imports", "using:\n    impact/missing"))
                .is_err()
        );
    }

    #[test]
    fn rejects_open_closed_requests_and_missing_find_variable() {
        assert!(
            parse(&SOURCE.replace(
                "    North imports Store\nusing:",
                "    ?north imports Store\nusing:"
            ))
            .is_err()
        );
        assert!(
            parse(&SOURCE.replace(
                "find all ?consumer in impact:",
                "find all ?missing in impact:"
            ))
            .is_err()
        );
    }

    #[test]
    fn parses_why_prefixed_qname_declarations() {
        let program = parse("Type: Type\nwhy: Type\nwhy-not: Type\n")
            .expect("why-prefixed names are declarations");
        assert!(
            program
                .declarations
                .iter()
                .any(|declaration| declaration.subject.value.as_str() == "why")
        );
        assert!(
            program
                .declarations
                .iter()
                .any(|declaration| declaration.subject.value.as_str() == "why-not")
        );
    }

    #[test]
    fn dispatches_only_exact_why_request_heads() {
        let source = SOURCE.replace(
            "why all in impact:\n    North imports Store",
            "why in impact:\n    North imports Store\n\nwhy all in impact:\n    North imports Store",
        );
        let program = parse(&source).expect("exact why request heads parse");
        assert!(matches!(
            program.requests[1],
            RequestDecl::Why { all: false, .. }
        ));
        assert!(matches!(
            program.requests[2],
            RequestDecl::Why { all: true, .. }
        ));
    }

    #[test]
    fn rejects_malformed_focus_ranges_bindings_and_slots() {
        let base = r#"Item: Type
Sensor: Type

pairing/pair: RelationShape
    {item: Item} paired with {sensor: Sensor}
    mode item -> sensor: many

pairing: Model
    Sensor-A: Sensor
    [Item 1..6]: Item
    [Item {n}]:
        paired with: Sensor-A
    for n: 1..4
"#;
        for replacement in [
            "[Item 6..1]: Item",
            "[Item 1..]: Item",
            "[Item {n}]:\n        paired with: Sensor-A\n    for m: 1..4",
            "[Item {n}]:\n        paired with Sensor-A\n    for n: 1..4",
            "[Item {n}]:\n    paired with: Sensor-A\n    for n: 1..4",
        ] {
            let source = base
                .replace(
                    "[Item 1..6]: Item\n    [Item {n}]:\n        paired with: Sensor-A\n    for n: 1..4",
                    replacement,
                );
            assert!(parse(&source).is_err(), "{}", replacement);
        }
        assert!(parse(&base.replace("[Item 1..6]", "[Item 1")).is_err());
        assert!(parse(&base.replace("[Item {n}]:", "[Item {n}] trailing:")).is_err());
    }
}
