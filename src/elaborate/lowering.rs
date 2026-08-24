use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, Name, SurfaceClause, SurfaceTerm},
    kernel::{
        self, Definition, Model, PatternId, ReferentId, RelationShape, RelationalContent, Role,
        RoleId, Term,
    },
};

use super::{
    compilation::CompiledProgram,
    identifiers::{DesignationTable, synthetic_referent, synthetic_role},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct Projection {
    pub(crate) designations: DesignationTable,
    pub(crate) grounded: BTreeSet<ReferentId>,
    pub(crate) model_referents: BTreeMap<ReferentId, BTreeSet<ReferentId>>,
    pub(crate) role_domains: BTreeMap<(ReferentId, RoleId), ReferentId>,
    pub(crate) focus_shapes: Vec<FocusShape>,
    pub(crate) rule_binders: BTreeMap<ReferentId, BinderTable>,
    pub(crate) request_binders: BTreeMap<usize, BinderTable>,
}

pub(crate) fn membership_relation() -> ReferentId {
    synthetic_referent("membership", &["relation"])
}

pub(crate) fn membership_member_role() -> RoleId {
    synthetic_role("membership", &["member"])
}

pub(crate) fn membership_group_role() -> RoleId {
    synthetic_role("membership", &["group"])
}

pub(crate) fn membership_shape() -> kernel::Result<RelationShape> {
    let member = Role::new(membership_member_role(), Vec::new())?;
    let group = Role::new(membership_group_role(), Vec::new())?;
    RelationShape::new(
        membership_relation(),
        BTreeMap::from([(member.id().clone(), member), (group.id().clone(), group)]),
        Vec::new(),
    )
}

pub(crate) fn membership_content(
    member: ReferentId,
    group: ReferentId,
) -> kernel::Result<RelationalContent> {
    RelationalContent::new(
        membership_relation(),
        BTreeMap::from([
            (membership_member_role(), Term::referent(member)),
            (membership_group_role(), Term::referent(group)),
        ]),
    )
}

pub(crate) fn lower_definition(
    projection: &Projection,
    model: &Model,
    name: &Name,
    denotation: &Name,
) -> kernel::Result<Definition> {
    let id = projection.designations.scoped(model.id(), name.as_str())?;
    let denotation = projection
        .designations
        .scoped(model.id(), denotation.as_str())?;
    require_term_referent(model, &id, "definition name")?;
    require_term_referent(model, &denotation, "definition denotation")?;
    Ok(Definition::new(id, Term::referent(denotation)))
}

pub(crate) fn lower_shape_binding(
    projection: &Projection,
    model: &Model,
    label: &Name,
    domain: &Name,
) -> kernel::Result<Definition> {
    let id = projection.designations.scoped(model.id(), label.as_str())?;
    let domain = projection.designations.global(domain.as_str())?;
    require_term_referent(model, &id, "shape binding")?;
    require_term_referent(model, &domain, "shape binding domain")?;
    Ok(Definition::new(id, Term::referent(domain)))
}

#[derive(Clone, Debug)]
pub(crate) struct FocusShape {
    pub(crate) relation: ReferentId,
    pub(crate) literal: String,
    pub(crate) focused_role: RoleId,
    pub(crate) value_role: RoleId,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BinderTable(BTreeMap<frontend::VariableName, PatternId>);

impl BinderTable {
    pub(crate) fn declare_alpha<'a>(
        designations: &mut DesignationTable,
        scope: &ReferentId,
        clauses: impl IntoIterator<Item = &'a SurfaceClause>,
    ) -> kernel::Result<Self> {
        let mut first = BTreeMap::<frontend::VariableName, (usize, usize)>::new();
        for clause in clauses {
            for term in clause.roles.values() {
                if let SurfaceTerm::Variable(variable) = term {
                    first
                        .entry(variable.value.clone())
                        .and_modify(|span| {
                            *span = (*span).min((variable.span.line, variable.span.column))
                        })
                        .or_insert((variable.span.line, variable.span.column));
                }
            }
        }
        let mut ordered = first.into_iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(_, span)| *span);
        let mut binders = BTreeMap::new();
        for (index, (name, _)) in ordered.into_iter().enumerate() {
            let id = designations.declare_pattern(scope, &format!("binder-{index}"))?;
            binders.insert(name, id);
        }
        Ok(Self(binders))
    }

    pub(crate) fn get(&self, name: &frontend::VariableName) -> kernel::Result<PatternId> {
        self.0.get(name).cloned().ok_or_else(|| {
            kernel::KernelError::new(format!("unbound pattern variable '{}'", name.as_str()))
        })
    }
}

/// Lower a request clause through the program's source projection. The
/// designation table remains outside the sealed Revision.
pub fn lower_clause(
    program: &CompiledProgram,
    revision: &crate::kernel::Revision,
    surface: &SurfaceClause,
) -> kernel::Result<RelationalContent> {
    lower_clause_with(program.projection(), revision.model(), surface, None)
}

pub(crate) fn lower_clause_with(
    projection: &Projection,
    model: &Model,
    surface: &SurfaceClause,
    binders: Option<&BinderTable>,
) -> kernel::Result<RelationalContent> {
    let relation_id = projection
        .designations
        .global(surface.relation.value.as_str())?;
    let relation = model.relation_shapes().get(&relation_id).ok_or_else(|| {
        kernel::KernelError::new(format!(
            "undeclared RelationShape '{}'",
            surface.relation.value.as_str()
        ))
    })?;
    let mut roles = BTreeMap::new();
    for (surface_role, surface_term) in &surface.roles {
        let role_id = projection
            .designations
            .role(&relation_id, &surface_role.0)?;
        if !relation.roles().contains_key(&role_id) {
            return Err(kernel::KernelError::new(format!(
                "RelationShape '{}' has no role '{}'",
                surface.relation.value.as_str(),
                surface_role.0
            )));
        }
        let expected = projection
            .role_domains
            .get(&(relation_id.clone(), role_id.clone()))
            .ok_or_else(|| {
                kernel::KernelError::new("relation role has no authoring domain projection")
            })?;
        if roles
            .insert(
                role_id,
                lower_term(projection, model, expected, surface_term, binders)?,
            )
            .is_some()
        {
            return Err(kernel::KernelError::new("duplicate relational role"));
        }
    }
    let content = RelationalContent::new(relation_id, roles)?;
    model.validate_content(&content, binders.is_some())?;
    Ok(content)
}

fn lower_term(
    projection: &Projection,
    model: &Model,
    expected: &ReferentId,
    term: &SurfaceTerm,
    binders: Option<&BinderTable>,
) -> kernel::Result<Term> {
    match term {
        SurfaceTerm::Variable(value) => {
            let binders = binders.ok_or_else(|| {
                kernel::KernelError::new("pattern variable is not valid in ground content")
            })?;
            Ok(Term::pattern(binders.get(&value.value)?))
        }
        SurfaceTerm::String(value) => {
            let text = projection.designations.global("Text")?;
            if expected != &text {
                return Err(kernel::KernelError::new(
                    "scalar strings require an admitted Text role",
                ));
            }
            let referent = projection.designations.literal(&value.value)?;
            require_term_referent(model, &referent, "string literal")?;
            Ok(Term::referent(referent))
        }
        SurfaceTerm::Referent(value) => {
            let referent = projection
                .designations
                .scoped(model.id(), value.value.as_str())?;
            require_term_referent(model, &referent, "referent")?;
            Ok(Term::referent(referent))
        }
        SurfaceTerm::Template(_) => Err(kernel::KernelError::new(
            "correlated referent templates are only valid inside a focus block",
        )),
    }
}

pub(crate) fn lower_focus(
    projection: &Projection,
    model: &Model,
    focus: &frontend::FocusBlock,
) -> kernel::Result<Vec<RelationalContent>> {
    let mut contents = Vec::new();
    for number in focus.binding.range.start..=focus.binding.range.end {
        let focused = focus_referent(projection, model, &focus.template, number)?;
        for slot in &focus.slots {
            let candidates = projection
                .focus_shapes
                .iter()
                .filter(|shape| {
                    shape.literal == slot.label.value
                        && model.relation_shapes().contains_key(&shape.relation)
                })
                .collect::<Vec<_>>();
            let shape = match candidates.as_slice() {
                [] => {
                    return Err(kernel::KernelError::new(format!(
                        "no declared sentence shape accepts focused slot '{}'",
                        slot.label.value
                    )));
                }
                [shape] => *shape,
                many => {
                    let mut designations = many
                        .iter()
                        .map(|shape| {
                            projection
                                .designations
                                .global_name(&shape.relation)
                                .expect("focus relation retains a source designation")
                        })
                        .collect::<Vec<_>>();
                    designations.sort_unstable();
                    return Err(kernel::KernelError::new(format!(
                        "ambiguous focused slot '{}'; candidates {}",
                        slot.label.value,
                        designations.join(", ")
                    )));
                }
            };
            let expected_value =
                &projection.role_domains[&(shape.relation.clone(), shape.value_role.clone())];
            let value = lower_focus_term(
                projection,
                model,
                expected_value,
                &slot.value,
                &focus.binding.variable.value,
                number,
            )?;
            let content = RelationalContent::new(
                shape.relation.clone(),
                BTreeMap::from([
                    (shape.focused_role.clone(), Term::referent(focused.clone())),
                    (shape.value_role.clone(), value),
                ]),
            )?;
            model.validate_content(&content, false)?;
            contents.push(content);
        }
    }
    Ok(contents)
}

fn lower_focus_term(
    projection: &Projection,
    model: &Model,
    expected: &ReferentId,
    term: &SurfaceTerm,
    binding: &frontend::VariableName,
    number: u64,
) -> kernel::Result<Term> {
    match term {
        SurfaceTerm::Template(template) => {
            if &template.variable.value != binding {
                return Err(kernel::KernelError::new(format!(
                    "unbound focus variable '{}'",
                    template.variable.value.as_str()
                )));
            }
            let referent = focus_referent(projection, model, template, number)?;
            Ok(Term::referent(referent))
        }
        _ => lower_term(projection, model, expected, term, None),
    }
}

fn focus_referent(
    projection: &Projection,
    model: &Model,
    template: &frontend::ReferentTemplate,
    number: u64,
) -> kernel::Result<ReferentId> {
    let local = format!(
        "{}{}{}",
        template.prefix.value, number, template.suffix.value
    );
    let referent = projection.designations.scoped(model.id(), &local)?;
    require_term_referent(model, &referent, "focused referent")?;
    Ok(referent)
}

fn require_term_referent(model: &Model, id: &ReferentId, where_: &str) -> kernel::Result<()> {
    if model.referents().contains_key(id) {
        Ok(())
    } else {
        Err(kernel::KernelError::new(format!(
            "{where_} is not admitted by this Model"
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{elaborate::compile, frontend};

    const BASE: &str = "Module\n\nimpact/imports: RelationShape\n  {consumer: Module} imports {dependency: Module}\n  mode consumer -> dependency: many\n\nimpact\n  North ∈ Module\n  South ∈ Module\n  Store ∈ Module\n  North imports Store\n";

    #[test]
    fn repeated_holes_share_one_opaque_pattern_binder() {
        let source = format!(
            "{BASE}\nimpact/reflexive: DerivationRule\n  ?item imports ?item\n  when:\n    ?item imports ?item\n"
        );
        let program = compile(frontend::parse(&source).unwrap()).unwrap();
        let rule = &program
            .revision(&frontend::Name("impact".into()))
            .unwrap()
            .model()
            .derivation_rules()[0];
        let premise = program
            .revision(&frontend::Name("impact".into()))
            .unwrap()
            .model()
            .content(&rule.premises().forms()[0])
            .unwrap();
        let ids = premise
            .roles()
            .values()
            .map(|term| term.pattern_id().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), 1);
    }
}
