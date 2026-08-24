use super::clause::*;
use super::declaration::*;
use super::model::*;
use super::relation::*;
use super::source::*;
use super::syntax::{DefinitionDecl, MembershipDecl, ShapeBindingDecl};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

const MODEL_OR_REVISION: &[Kind] = &[
    Kind::Enumeration,
    Kind::BindingShape,
    Kind::Model,
    Kind::Revision,
];

fn is_direct_focus(
    raw: &RawDecl<'_>,
    grounded: &BTreeSet<Name>,
    relations: &BTreeMap<Name, RelationSpec>,
) -> Result<bool, ParseError> {
    if raw.kind != Kind::Model {
        return Ok(false);
    }
    let entries = nonblank(raw.body.iter().copied());
    if entries.is_empty() || entries.iter().any(|line| !matches!(indent(*line), Ok(2))) {
        return Ok(false);
    }
    let has_grounded_category = entries
        .iter()
        .any(|line| focused_name(*line).is_ok_and(|name| grounded.contains(&name.value)));
    let has_non_bare = entries.iter().try_fold(false, |found, line| {
        if found || definition_line(*line).is_some() || relation_line_matches(*line, relations)? {
            return Ok::<_, ParseError>(true);
        }
        let expanded = format!("{} {}", raw.subject.value.as_str(), content(*line));
        relation_line_matches(
            SourceLine {
                number: line.number,
                text: &expanded,
            },
            relations,
        )
    })?;
    Ok(has_grounded_category && has_non_bare)
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let (mut raw_declarations, mut raw_top_level, raw_requests) = scan(source)?;
    for declaration in &mut raw_declarations {
        if declaration.bare_block && content(declaration.header).ends_with(':') {
            return Err(error(
                line_span(declaration.header),
                "':' only establishes a binding; it cannot introduce a block",
            ));
        }
        if declaration.bare_block && compact_relation_candidate(declaration) {
            declaration.kind = Kind::RelationShape;
            declaration.bare_block = false;
        }
    }
    let provisional_grounded = raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Grounding || declaration.bare_block)
        .map(|declaration| declaration.subject.value.clone())
        .collect::<BTreeSet<_>>();
    let mut relations = BTreeMap::new();
    for declaration in raw_declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::RelationShape)
    {
        relations.insert(
            declaration.subject.value.clone(),
            relation_spec(declaration, &provisional_grounded)?,
        );
    }

    let mut retained = Vec::new();
    for declaration in raw_declarations {
        if declaration.kind == Kind::Grounding
            && relation_line_matches(declaration.header, &relations)?
        {
            raw_top_level.push(RawTopLevel {
                line: declaration.header,
            });
        } else {
            retained.push(declaration);
        }
    }
    let raw_declarations = retained;
    let mut raw_focuses = Vec::new();
    let mut retained = Vec::new();
    for declaration in raw_declarations {
        if is_direct_focus(&declaration, &provisional_grounded, &relations)? {
            raw_focuses.push(declaration);
        } else {
            retained.push(declaration);
        }
    }
    let mut raw_declarations = retained;
    for declaration in &mut raw_declarations {
        if declaration.bare_block {
            declaration.kind = infer_bare_block_kind(declaration, &relations)?;
            declaration.bare_block = false;
        }
    }
    let grounded = raw_declarations
        .iter()
        .filter(|declaration| {
            matches!(
                declaration.kind,
                Kind::Grounding | Kind::Enumeration | Kind::BindingShape | Kind::Model
            )
        })
        .map(|declaration| declaration.subject.value.clone())
        .collect::<BTreeSet<_>>();
    for (name, spec) in &relations {
        for domain in spec.roles.values() {
            if !grounded.contains(&Name(domain.0.clone())) {
                return Err(error(
                    raw_declarations
                        .iter()
                        .find(|declaration| declaration.subject.value == *name)
                        .map_or(
                            Span {
                                line: 1,
                                column: 1,
                                width: 0,
                            },
                            |declaration| line_span(declaration.header),
                        ),
                    format!("unknown role domain '{}'", domain.as_str()),
                ));
            }
        }
    }

    let mut declaration_names = BTreeSet::new();
    for declaration in &raw_declarations {
        if !declaration_names.insert(declaration.subject.value.clone()) {
            return Err(error(
                declaration.subject.span,
                format!(
                    "duplicate declaration '{}'",
                    declaration.subject.value.as_str()
                ),
            ));
        }
    }
    let mut top_memberships = MembershipCatalog {
        explicit: BTreeMap::new(),
        ranges: Vec::new(),
    };
    for fragment in &raw_top_level {
        if let Some(membership) = membership_line(fragment.line) {
            let membership = membership?;
            if !grounded.contains(&membership.group.value) {
                return Err(error(
                    membership.group.span,
                    format!(
                        "unknown membership group '{}'",
                        membership.group.value.as_str()
                    ),
                ));
            }
            insert_membership(&mut top_memberships.explicit, &membership);
        } else if let Some(definition) = definition_line(fragment.line) {
            definition?;
        }
    }
    for focus in &raw_focuses {
        for line in nonblank(focus.body.iter().copied()) {
            if let Ok(group) = focused_name(line)
                && grounded.contains(&group.value)
            {
                insert_membership(
                    &mut top_memberships.explicit,
                    &MembershipDecl {
                        member: focus.subject.clone(),
                        group,
                        span: line_span(line),
                    },
                );
            }
        }
    }
    let kinds = raw_declarations
        .iter()
        .map(|declaration| (declaration.subject.value.clone(), declaration.kind))
        .collect::<BTreeMap<_, _>>();
    let mut memberships = BTreeMap::new();
    for declaration in raw_declarations.iter().filter(|declaration| {
        matches!(
            declaration.kind,
            Kind::Enumeration | Kind::BindingShape | Kind::Model
        )
    }) {
        let catalog = model_memberships(declaration, &grounded)?;
        for domain in catalog
            .explicit
            .values()
            .flatten()
            .chain(catalog.ranges.iter().map(|range| &range.group.value))
        {
            if !grounded.contains(&Name(domain.0.clone())) {
                return Err(error(
                    line_span(declaration.header),
                    format!("unknown membership group '{}'", domain.as_str()),
                ));
            }
        }
        memberships.insert(declaration.subject.value.clone(), catalog);
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
        reference_kind(&layout.from, &kinds, MODEL_OR_REVISION, "revision base")?;
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
            Kind::Grounding => Vec::new(),
            Kind::Enumeration => nonblank(raw.body.iter().copied())
                .into_iter()
                .map(|line| {
                    Ok(Member::Membership(MembershipDecl {
                        member: semantic_name(line, 2, content(line))?,
                        group: raw.subject.clone(),
                        span: line_span(line),
                    }))
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
            Kind::BindingShape => nonblank(raw.body.iter().copied())
                .into_iter()
                .map(|line| {
                    let binding = definition_line(line)
                        .expect("binding-shape classification checked every member")?;
                    if !grounded.contains(&binding.denotation.value) {
                        return Err(error(
                            binding.denotation.span,
                            format!(
                                "unknown binding domain '{}'",
                                binding.denotation.value.as_str()
                            ),
                        ));
                    }
                    Ok(Member::ShapeBinding(ShapeBindingDecl {
                        label: binding.name,
                        domain: binding.denotation,
                        span: binding.span,
                    }))
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
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
                    if let Some(range) = membership_range_line(line) {
                        members.push(Member::MembershipRange(range?));
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
                            if let Some(definition) = definition_line(child) {
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
                                let group = focused_name(child)?;
                                if grounded.contains(&group.value) {
                                    members.push(Member::Membership(MembershipDecl {
                                        member: focus.clone(),
                                        group,
                                        span: line_span(child),
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
                                        &memberships,
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
                            }
                            index += 1;
                        }
                    } else {
                        let parsed = clause(
                            line,
                            &raw.subject.value,
                            &relations,
                            &memberships,
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
                    declared_model_for_law(&raw.subject.value, &memberships).ok_or_else(|| {
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
                    &memberships,
                    &mut variable_types,
                )?;
                let premises = layout
                    .premises
                    .iter()
                    .copied()
                    .map(|line| clause(line, &model, &relations, &memberships, &mut variable_types))
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
                            .map(|line| {
                                clause(line, &model, &relations, &memberships, &mut variables)
                            })
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
                            .map(|line| {
                                clause(line, &model, &relations, &memberships, &mut variables)
                            })
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
        declarations.push(Declaration {
            subject: raw.subject.clone(),
            kind: raw.kind,
            body,
            span: line_span(raw.header),
        });
    }

    let mut top_level = Vec::new();
    let mut ordered_top_level = Vec::new();
    for fragment in raw_top_level {
        let member = if let Some(membership) = membership_line(fragment.line) {
            Member::Membership(membership?)
        } else if let Some(definition) = definition_line(fragment.line) {
            Member::Definition(definition?)
        } else {
            let mut variables = BTreeMap::new();
            let parsed = clause_with_catalog(
                fragment.line,
                &top_memberships,
                &relations,
                &memberships,
                &mut variables,
            )?;
            if !ground(&parsed) {
                return Err(error(parsed.span, "model assertions must be closed"));
            }
            Member::RelationalContent(parsed)
        };
        ordered_top_level.push((fragment.line.number, member));
    }
    for focus in raw_focuses {
        for line in nonblank(focus.body.iter().copied()) {
            let member = if let Some(definition) = definition_line(line) {
                let definition = definition?;
                Member::Definition(DefinitionDecl {
                    name: Spanned {
                        value: Name(format!(
                            "{} of {}",
                            definition.name.value.as_str(),
                            focus.subject.value.as_str()
                        )),
                        span: definition.name.span,
                    },
                    denotation: definition.denotation,
                    span: definition.span,
                })
            } else if let Ok(group) = focused_name(line)
                && grounded.contains(&group.value)
            {
                Member::Membership(MembershipDecl {
                    member: focus.subject.clone(),
                    group,
                    span: line_span(line),
                })
            } else {
                let expanded = format!("{} {}", focus.subject.value.as_str(), content(line));
                let mut parsed = clause_with_catalog(
                    SourceLine {
                        number: line.number,
                        text: &expanded,
                    },
                    &top_memberships,
                    &relations,
                    &memberships,
                    &mut BTreeMap::new(),
                )?;
                parsed.span = line_span(line);
                Member::RelationalContent(parsed)
            };
            ordered_top_level.push((line.number, member));
        }
    }
    ordered_top_level.sort_by_key(|(line, _)| *line);
    top_level.extend(ordered_top_level.into_iter().map(|(_, member)| member));

    let mut requests = Vec::new();
    for raw in raw_requests {
        match raw {
            RawRequest::Find {
                revision,
                sought,
                clause: line,
                header,
            } => {
                reference_kind(&revision, &kinds, MODEL_OR_REVISION, "request revision")?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let pattern = clause(line, &model, &relations, &memberships, &mut variables)?;
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
                reference_kind(&revision, &kinds, MODEL_OR_REVISION, "request revision")?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let target = clause(line, &model, &relations, &memberships, &mut variables)?;
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
                reference_kind(&revision, &kinds, MODEL_OR_REVISION, "request revision")?;
                let model = revision_model(&revision.value, &kinds, &layouts);
                let mut variables = BTreeMap::new();
                let target = clause(line, &model, &relations, &memberships, &mut variables)?;
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
                reference_kind(&base, &kinds, MODEL_OR_REVISION, "diff base")?;
                reference_kind(&successor, &kinds, MODEL_OR_REVISION, "diff successor")?;
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
        top_level,
        requests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r#"Module
Change

impact/imports: RelationShape
  {consumer: Module} imports {dependency: Module}
  mode consumer -> dependency: many

impact/affects: RelationShape
  {change: Change} affects {consumer: Module}
  mode change -> consumer: many

impact
  North ∈ Module
  Store ∈ Module
  compiler-change ∈ Change
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
            "find all ?consumer in impact:\n  compiler-change affects ?consumer\n\n{}\nModule\nChange\n",
            SOURCE.replace("Module\nChange\n\n", "").replace(
                "find all ?consumer in impact:\n  compiler-change affects ?consumer\n\n",
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
    fn infers_grounding_enumeration_binding_shape_and_model_from_bare_form() {
        let program = parse(
            "F32\nGame\n  Chess\n  Soccer\n\nVec2\n  x: F32\n  y: F32\n\nDoor\nPlace\n\negress/connects: RelationShape\n  {connector: Door} connects {from: Place} to {to: Place}\n  mode connector, from -> to: many\n\negress\n  Cellar ∈ Place\n  Armory ∈ Place\n  iron-door\n    Door\n    connects Cellar to Armory\n",
        )
        .expect("each bare form has one checked structural interpretation");
        let kind = |name: &str| {
            program
                .declarations
                .iter()
                .find(|declaration| declaration.subject.value.as_str() == name)
                .map(|declaration| declaration.kind)
        };
        assert_eq!(kind("F32"), Some(Kind::Grounding));
        assert_eq!(kind("Game"), Some(Kind::Enumeration));
        assert_eq!(kind("Vec2"), Some(Kind::BindingShape));
        assert_eq!(kind("egress"), Some(Kind::Model));
        assert!(
            program
                .declarations
                .iter()
                .find(|declaration| declaration.subject.value.as_str() == "Game")
                .is_some_and(|declaration| declaration
                    .body
                    .iter()
                    .all(|member| matches!(member, Member::Membership(_))))
        );
    }

    #[test]
    fn compact_relation_block_matches_the_ceremonial_relation_ast() {
        let compact = parse(
            "Door\nSpace\n\nconnects\n  door: Door connects origin: Space to destination: Space\n  door origin -> destination*\n",
        )
        .expect("compact relation schema parses");
        let ceremonial = parse(
            "Door\nSpace\n\nconnects: RelationShape\n  {door: Door} connects {origin: Space} to {destination: Space}\n  mode door, origin -> destination: many\n",
        )
        .expect("ceremonial relation schema parses");

        let contract = |program: &Program| {
            let relation = program
                .declarations
                .iter()
                .find(|declaration| declaration.subject.value.as_str() == "connects")
                .expect("connects relation exists");
            assert_eq!(relation.kind, Kind::RelationShape);
            let Member::Sentence(sentence) = &relation.body[0] else {
                panic!("relation starts with a sentence shape");
            };
            assert_eq!(sentence.focus.value.as_str(), "door");
            let parts = sentence
                .parts
                .iter()
                .map(|part| match part {
                    ShapePartDecl::Role { id, domain } => {
                        format!("{}:{}", id.value.as_str(), domain.value.as_str())
                    }
                    ShapePartDecl::Literal(value) => format!("={}", value.value),
                })
                .collect::<Vec<_>>();
            let Member::LookupMode(mode) = &relation.body[1] else {
                panic!("relation ends with a lookup mode");
            };
            (
                parts,
                mode.known
                    .iter()
                    .map(|role| role.value.as_str().to_owned())
                    .collect::<Vec<_>>(),
                mode.sought
                    .iter()
                    .map(|role| role.value.as_str().to_owned())
                    .collect::<Vec<_>>(),
                mode.cardinality,
            )
        };

        assert_eq!(contract(&compact), contract(&ceremonial));
    }

    #[test]
    fn compact_relation_domains_resolve_exact_grounded_multiword_names() {
        let compact = parse(
            "security door\ninterior space\n\nconnects\n  door: security door connects origin: interior space to destination: interior space\n  door origin -> destination*\n",
        )
        .expect("grounded multiword compact domains resolve");
        let relation = compact
            .declarations
            .iter()
            .find(|declaration| declaration.subject.value.as_str() == "connects")
            .expect("compact relation exists");
        let Member::Sentence(sentence) = &relation.body[0] else {
            panic!("compact relation starts with its sentence shape");
        };
        let domains = sentence
            .parts
            .iter()
            .filter_map(|part| match part {
                ShapePartDecl::Role { domain, .. } => Some(domain.value.as_str()),
                ShapePartDecl::Literal(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            domains,
            ["security door", "interior space", "interior space"]
        );
        let literals = sentence
            .parts
            .iter()
            .filter_map(|part| match part {
                ShapePartDecl::Literal(literal) => Some(literal.value.as_str()),
                ShapePartDecl::Role { .. } => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(literals, ["connects", "to"]);
    }

    #[test]
    fn compact_relation_domains_reject_ambiguous_grounded_segmentations() {
        let error = parse(
            "security\nsecurity door\ninterior space\n\nconnects\n  door: security door connects origin: interior space to destination: interior space\n  door origin -> destination*\n",
        )
        .expect_err("two grounded domain prefixes must remain ambiguous")
        .to_string();

        assert!(error.contains("ambiguous"), "{error}");
        assert!(error.contains("'security'"), "{error}");
        assert!(error.contains("'security door'"), "{error}");
    }

    #[test]
    fn compact_optional_cardinality_is_exact_and_does_not_reserve_role_names() {
        let program = parse(
            "Item\nState\n\nstatus\n  subject: Item has maybe: State\n  subject -> maybe 0..1\n",
        )
        .expect("0..1 is an exact optional contract beside an ordinary 'maybe' role");
        let relation = program
            .declarations
            .iter()
            .find(|declaration| declaration.subject.value.as_str() == "status")
            .expect("status relation exists");
        let Member::Sentence(shape) = &relation.body[0] else {
            panic!("status starts with its sentence shape");
        };
        assert_eq!(shape.focus.value.as_str(), "subject");
        let Member::LookupMode(mode) = &relation.body[1] else {
            panic!("status ends with its projection contract");
        };
        assert_eq!(mode.cardinality, Cardinality::Maybe);
        assert_eq!(mode.sought[0].value.as_str(), "maybe");
    }

    #[test]
    fn rejects_colon_used_only_as_block_punctuation() {
        let error = parse("Game:\n  Chess\n  Soccer\n").unwrap_err().to_string();
        assert!(error.contains("':' only establishes a binding"), "{error}");

        let error = parse("Item\n\nsettings:\n  left: Item pairs right: Item\n  left -> right\n")
            .unwrap_err()
            .to_string();
        assert!(error.contains("':' only establishes a binding"), "{error}");

        let error = parse("Item\n\nsettings:\n  left: Item pairs right: Item\n  left, -> right\n")
            .unwrap_err()
            .to_string();
        assert!(error.contains("':' only establishes a binding"), "{error}");
    }

    #[test]
    fn compact_schema_candidates_require_exact_role_tokens() {
        for source in [
            "Item\n\nsettings\n  left: Item, right: Item\n  left -> right\n",
            "Item\n\nsettings\n  left: Item , right: Item\n  left -> right\n",
            "Item\n\nsettings\n  left: Item pairs right: Item\n  left, -> right\n",
        ] {
            let (declarations, _, _) = scan(source).expect("candidate source scans");
            let settings = declarations
                .iter()
                .find(|declaration| declaration.subject.value.as_str() == "settings")
                .expect("settings block exists");
            assert!(!compact_relation_candidate(settings));
            assert!(parse(source).is_err());
        }
    }

    #[test]
    fn top_level_membership_is_preserved_for_a_stable_model_context() {
        let program = parse("Game\nChess ∈ Game\n").expect("contextual form parses");
        assert!(matches!(
            program.top_level.as_slice(),
            [Member::Membership(_)]
        ));
    }

    #[test]
    fn rejects_bracketed_concrete_memberships_at_every_scope() {
        for source in [
            "Item\n[Item 1] ∈ Item\n",
            "Item\n\npairing\n  [Item 1] ∈ Item\n",
        ] {
            let error = parse(source).unwrap_err().to_string();
            assert!(
                error.contains("bracketed concrete referents are retired"),
                "{error}"
            );
        }
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
            "impact/adopt: Revision\n  from: impact\n  admit:\n    Store imports North",
            "impact/remove: Delta\n  from: impact\n  withdraw:\n    North imports Store\n\nimpact/adopt: Revision\n  from: impact\n  apply: impact/remove",
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
                "  {consumer: Module} imports {dependency: Module}",
                "  sentence: {consumer} imports {dependency}",
            ))
            .is_err()
        );
    }

    #[test]
    fn rejects_non_two_space_indentation_and_tabs() {
        assert!(parse(&SOURCE.replace("  North ∈ Module", "    North ∈ Module")).is_err());
        assert!(parse(&SOURCE.replace("  North ∈ Module", "\tNorth ∈ Module")).is_err());
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
    fn rejects_unknown_or_wrong_domain_referents_and_quoted_modules() {
        assert!(parse(&SOURCE.replace("North imports Store", "Missing imports Store")).is_err());
        assert!(
            parse(&SOURCE.replace("North imports Store", "compiler-change imports Store")).is_err()
        );
        assert!(parse(&SOURCE.replace("North imports Store", "\"North\" imports Store")).is_err());
    }

    #[test]
    fn rejects_inconsistent_law_variables() {
        let source = SOURCE.replace(
            "?consumer imports ?dependency\n  when:\n    ?consumer imports ?dependency",
            "?consumer imports ?dependency\n  when:\n    compiler-change affects ?consumer",
        );
        assert!(parse(&source).is_err());
    }

    #[test]
    fn preserves_duplicate_admissions_but_rejects_overlaps() {
        let duplicate = SOURCE.replace(
            "    Store imports North",
            "    Store imports North\n    Store imports North",
        );
        assert!(parse(&duplicate).is_ok());
        let overlap = SOURCE.replace(
            "  admit:\n    Store imports North",
            "  admit:\n    Store imports North\n  withdraw:\n    Store imports North",
        );
        assert!(parse(&overlap).is_err());
    }

    #[test]
    fn accepts_apply_with_different_revision_aliases() {
        let source = SOURCE.replace(
            "impact/adopt: Revision\n  from: impact\n  admit:\n    Store imports North",
            "impact/alias-left: Revision\n  from: impact\n  admit:\n    Store imports North\n\nimpact/alias-right: Revision\n  from: impact\n  admit:\n    Store imports North\n\nimpact/remove: Delta\n  from: impact/alias-left\n  withdraw:\n    North imports Store\n\nimpact/adopt: Revision\n  from: impact/alias-right\n  apply: impact/remove",
        );
        assert!(parse(&source).is_ok());
    }

    #[test]
    fn rejects_cycles_and_bad_request_references() {
        let cycle = SOURCE.replace("impact/adopt: Revision\n  from: impact\n  admit:\n    Store imports North", "impact/first: Revision\n  from: impact/second\n  admit:\n    Store imports North\n\nimpact/second: Revision\n  from: impact/first\n  admit:\n    Store imports North");
        assert!(parse(&cycle).is_err());
        assert!(
            parse(&SOURCE.replace("using:\n  impact/imports", "using:\n  impact/missing")).is_err()
        );
    }

    #[test]
    fn rejects_open_closed_requests_and_missing_find_variable() {
        assert!(
            parse(&SOURCE.replace(
                "  North imports Store\nusing:",
                "  ?north imports Store\nusing:"
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
        let program = parse("Type\nwhy\nwhy-not\n").expect("why-prefixed names are declarations");
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
            "why all in impact:\n  North imports Store",
            "why in impact:\n  North imports Store\n\nwhy all in impact:\n  North imports Store",
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
        let base = r#"Item
Sensor

pairing/pair: RelationShape
  {item: Item} paired with {sensor: Sensor}
  mode item -> sensor: many

pairing
  Sensor-A ∈ Sensor
  [Item 1..6] ∈ Item
  [Item {n}]
    paired with Sensor-A
  for n: 1..4
"#;
        parse(base).expect("canonical finite membership and correlated focus parse");
        for replacement in [
            "[Item 6..1] ∈ Item",
            "[Item 1..] ∈ Item",
            "[Item {n}]\n    paired with Sensor-A\n  for m: 1..4",
            "[Item {n}]\n    paired with: Sensor-A\n  for n: 1..4",
            "[Item {n}]\n  paired with Sensor-A\n  for n: 1..4",
        ] {
            let source = base.replace(
                "[Item 1..6] ∈ Item\n  [Item {n}]\n    paired with Sensor-A\n  for n: 1..4",
                replacement,
            );
            assert!(parse(&source).is_err(), "{}", replacement);
        }
        assert!(parse(&base.replace("[Item 1..6]", "[Item 1")).is_err());
        assert!(parse(&base.replace("[Item {n}]", "[Item {n}] trailing:")).is_err());
    }
}
