use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, AscriptionDecl, Kind, Member, ShapePartDecl, SurfaceTerm},
    kernel::{
        self, AssertionOccurrence, DerivationRule, Judgment, JudgmentKind, JudgmentStatus,
        JudgmentTarget, LookupMode, Model, Pattern, Referent, ReferentId, RelationShape,
        RelationalContent, Role, RoleId, RolePredicate, Term,
    },
    wire,
};

use super::{
    identifiers::{DesignationTable, synthetic_referent, synthetic_role},
    lowering::{BinderTable, Projection, lower_clause_with, lower_focus},
    resolution::Resolver,
};

/// Sealed revisions indexed by authored navigation names, plus the source
/// projection required to resolve requests without putting designations into
/// semantic identity.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    revisions: BTreeMap<frontend::Name, kernel::Revision>,
    requests: Vec<frontend::RequestDecl>,
    projection: Projection,
    source_spans: BTreeMap<ReferentId, frontend::Span>,
}

impl CompiledProgram {
    pub fn revisions(&self) -> &BTreeMap<frontend::Name, kernel::Revision> {
        &self.revisions
    }
    pub fn requests(&self) -> &[frontend::RequestDecl] {
        &self.requests
    }

    pub fn revision(&self, name: &frontend::Name) -> kernel::Result<&kernel::Revision> {
        self.revisions.get(name).ok_or_else(|| {
            kernel::KernelError::new(format!("unknown Revision '{}'", name.as_str()))
        })
    }

    pub fn lower_clause(
        &self,
        revision: &kernel::Revision,
        surface: &frontend::SurfaceClause,
    ) -> kernel::Result<kernel::RelationalContent> {
        lower_clause_with(&self.projection, revision.model(), surface, None)
    }

    pub(crate) fn lower_request_clause(
        &self,
        index: usize,
        revision: &kernel::Revision,
        surface: &frontend::SurfaceClause,
    ) -> kernel::Result<kernel::RelationalContent> {
        let binders =
            self.projection.request_binders.get(&index).ok_or_else(|| {
                kernel::KernelError::new("request has no pattern-binder projection")
            })?;
        lower_clause_with(&self.projection, revision.model(), surface, Some(binders))
    }

    pub(crate) fn request_pattern(
        &self,
        index: usize,
        variable: &frontend::VariableName,
    ) -> kernel::Result<kernel::PatternId> {
        self.projection
            .request_binders
            .get(&index)
            .ok_or_else(|| kernel::KernelError::new("request has no pattern-binder projection"))?
            .get(variable)
    }

    pub(crate) fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn designations(&self) -> &DesignationTable {
        &self.projection.designations
    }

    /// Locate one assertion occurrence in the parsed source projection.
    pub fn source_span(&self, occurrence: &ReferentId) -> Option<frontend::Span> {
        self.source_spans.get(occurrence).copied()
    }

    /// Compile after an explicit designation rename transaction. The table is
    /// projection state: retaining a designation changes source terms without
    /// changing the referents, roles, or binders sealed into the Revision.
    pub fn compile_with_designations(
        program: frontend::Program,
        designations: DesignationTable,
    ) -> kernel::Result<Self> {
        compile_projected(program, designations)
    }
}

pub fn compile(program: frontend::Program) -> kernel::Result<CompiledProgram> {
    compile_projected(program, DesignationTable::new())
}

fn compile_projected(
    program: frontend::Program,
    designations: DesignationTable,
) -> kernel::Result<CompiledProgram> {
    let declarations = declaration_map(&program.declarations)?;
    let mut projection = declare_projection(&program, designations)?;
    let relation_shapes = lower_relation_shapes(&program.declarations, &mut projection)?;
    let (models, source_spans) =
        lower_models(&program.declarations, &relation_shapes, &projection)?;
    let (revisions, source_spans) = {
        let mut resolver = Resolver::new(&declarations, models, &projection, source_spans);
        for declaration in &program.declarations {
            match declaration.kind {
                Kind::Model | Kind::Revision => {
                    resolver.revision(&declaration.subject.value)?;
                }
                Kind::Delta => {
                    resolver.delta(&declaration.subject.value)?;
                }
                Kind::Type | Kind::RelationShape | Kind::DerivationRule => {}
            }
        }
        (resolver.revisions, resolver.source_spans)
    };
    Ok(CompiledProgram {
        revisions,
        requests: program.requests,
        projection,
        source_spans,
    })
}

fn declaration_map(
    declarations: &[AscriptionDecl],
) -> kernel::Result<BTreeMap<frontend::Name, &AscriptionDecl>> {
    let mut by_name = BTreeMap::new();
    for declaration in declarations {
        if by_name
            .insert(declaration.subject.value.clone(), declaration)
            .is_some()
        {
            return Err(kernel::KernelError::new(format!(
                "duplicate declaration '{}'",
                declaration.subject.value.as_str()
            )));
        }
    }
    Ok(by_name)
}

fn declare_projection(
    program: &frontend::Program,
    designations: DesignationTable,
) -> kernel::Result<Projection> {
    let mut projection = Projection {
        designations,
        ..Projection::default()
    };
    for declaration in &program.declarations {
        let name = declaration.subject.value.as_str();
        let id = match declaration.kind {
            Kind::Model => projection.designations.declare_model(name)?,
            Kind::Type
            | Kind::RelationShape
            | Kind::DerivationRule
            | Kind::Revision
            | Kind::Delta => projection.designations.declare_global(name)?,
        };
        if declaration.kind == Kind::Type {
            projection.types.insert(id);
        }
    }
    for declaration in &program.declarations {
        match declaration.kind {
            Kind::RelationShape => declare_relation_projection(declaration, &mut projection)?,
            Kind::Model => declare_model_projection(declaration, &mut projection)?,
            Kind::DerivationRule => declare_rule_projection(declaration, &mut projection)?,
            _ => {}
        }
        declare_literals(&declaration.body, &mut projection.designations);
    }
    for (index, request) in program.requests.iter().enumerate() {
        declare_request_literals(request, &mut projection.designations);
        if let frontend::RequestDecl::Find { pattern, .. } = request {
            let ordinal = index.to_string();
            let scope = synthetic_referent("request-pattern-scope", &[&ordinal]);
            let binders = BinderTable::declare_alpha(
                &mut projection.designations,
                &scope,
                std::iter::once(pattern),
            )?;
            projection.request_binders.insert(index, binders);
        }
    }
    Ok(projection)
}

fn declare_rule_projection(
    declaration: &AscriptionDecl,
    projection: &mut Projection,
) -> kernel::Result<()> {
    let rule = projection
        .designations
        .global(declaration.subject.value.as_str())?;
    let conclusion = declaration
        .body
        .iter()
        .find_map(|member| match member {
            Member::RelationalContent(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| kernel::KernelError::new("DerivationRule requires a conclusion"))?;
    let premises = declaration
        .body
        .iter()
        .find_map(|member| match member {
            Member::When(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| kernel::KernelError::new("DerivationRule requires when premises"))?;
    let binders = BinderTable::declare_alpha(
        &mut projection.designations,
        &rule,
        premises.iter().chain(std::iter::once(conclusion)),
    )?;
    projection.rule_binders.insert(rule, binders);
    Ok(())
}

fn declare_relation_projection(
    declaration: &AscriptionDecl,
    projection: &mut Projection,
) -> kernel::Result<()> {
    let relation = projection
        .designations
        .global(declaration.subject.value.as_str())?;
    let sentence = declaration
        .body
        .iter()
        .find_map(|member| match member {
            Member::Sentence(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| kernel::KernelError::new("RelationShape requires one sentence shape"))?;
    let mut ordered_roles = Vec::new();
    let mut literal = None;
    for part in &sentence.parts {
        match part {
            ShapePartDecl::Literal(value) => {
                if literal.is_none() {
                    literal = Some(value.value.clone());
                }
            }
            ShapePartDecl::Role { id, typ } => {
                let role = projection
                    .designations
                    .declare_role(&relation, &id.value.0)?;
                let expected = projection.designations.global(&typ.value.0)?;
                if !projection.types.contains(&expected) {
                    return Err(kernel::KernelError::new(format!(
                        "undeclared Type '{}'",
                        typ.value.0
                    )));
                }
                projection
                    .role_types
                    .insert((relation.clone(), role.clone()), expected);
                ordered_roles.push(role);
            }
        }
    }
    if ordered_roles.len() == 2
        && let Some(literal) = literal
    {
        projection.focus_shapes.push(super::lowering::FocusShape {
            relation,
            literal,
            focused_role: ordered_roles[0].clone(),
            value_role: ordered_roles[1].clone(),
        });
    }
    Ok(())
}

fn declare_model_projection(
    declaration: &AscriptionDecl,
    projection: &mut Projection,
) -> kernel::Result<()> {
    let model = projection
        .designations
        .global(declaration.subject.value.as_str())?;
    for member in &declaration.body {
        match member {
            Member::Entity(entity) => {
                let id = projection
                    .designations
                    .declare_scoped(&model, entity.local.value.as_str())?;
                let typ = projection.designations.global(&entity.typ.value.0)?;
                projection.entity_types.insert(id.clone(), typ);
                projection
                    .model_entities
                    .entry(model.clone())
                    .or_default()
                    .insert(id);
            }
            Member::EntityGroup(group) => {
                let typ = projection.designations.global(&group.typ.value.0)?;
                for number in group.range.start..=group.range.end {
                    let local = format!("{}{}{}", group.prefix.value, number, group.suffix.value);
                    let id = projection.designations.declare_scoped(&model, &local)?;
                    projection.entity_types.insert(id.clone(), typ.clone());
                    projection
                        .model_entities
                        .entry(model.clone())
                        .or_default()
                        .insert(id);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn lower_relation_shapes(
    declarations: &[AscriptionDecl],
    projection: &mut Projection,
) -> kernel::Result<BTreeMap<ReferentId, RelationShape>> {
    let mut shapes = BTreeMap::new();
    for declaration in declarations
        .iter()
        .filter(|item| item.kind == Kind::RelationShape)
    {
        let relation = projection
            .designations
            .global(declaration.subject.value.as_str())?;
        let sentence = declaration
            .body
            .iter()
            .find_map(|member| match member {
                Member::Sentence(value) => Some(value),
                _ => None,
            })
            .ok_or_else(|| kernel::KernelError::new("RelationShape requires one sentence shape"))?;
        let roles = sentence
            .parts
            .iter()
            .filter_map(|part| match part {
                ShapePartDecl::Role { id, .. } => Some(id),
                _ => None,
            })
            .map(|id| {
                let role = projection.designations.role(&relation, &id.value.0)?;
                let expected = projection
                    .role_types
                    .get(&(relation.clone(), role.clone()))
                    .expect("relation role type was recorded during declaration")
                    .clone();
                Ok((
                    role.clone(),
                    Role::new(role, vec![legacy_type_predicate(expected)?])?,
                ))
            })
            .collect::<kernel::Result<BTreeMap<_, _>>>()?;
        let lookup = declaration
            .body
            .iter()
            .filter_map(|member| match member {
                Member::LookupMode(value) => Some(value),
                _ => None,
            })
            .map(|mode| {
                LookupMode::finite(
                    mode.known
                        .iter()
                        .map(|role| projection.designations.role(&relation, &role.value.0))
                        .collect::<kernel::Result<Vec<_>>>()?,
                    mode.sought
                        .iter()
                        .map(|role| projection.designations.role(&relation, &role.value.0))
                        .collect::<kernel::Result<Vec<_>>>()?,
                    cardinality(mode.cardinality),
                )
            })
            .collect::<kernel::Result<Vec<_>>>()?;
        let shape = RelationShape::new(relation.clone(), roles, lookup)?;
        if shapes.insert(relation, shape).is_some() {
            return Err(kernel::KernelError::new("duplicate RelationShape identity"));
        }
    }
    let classification = legacy_type_classification_shape()?;
    if shapes
        .insert(classification.referent().clone(), classification)
        .is_some()
    {
        return Err(kernel::KernelError::new(
            "legacy type classification relation collides with an authored RelationShape",
        ));
    }
    Ok(shapes)
}

fn legacy_type_classification_relation() -> ReferentId {
    synthetic_referent("legacy-type-classification", &["relation"])
}

fn legacy_type_candidate_role() -> RoleId {
    synthetic_role("legacy-type-classification", &["candidate"])
}

fn legacy_type_class_role() -> RoleId {
    synthetic_role("legacy-type-classification", &["class"])
}

fn legacy_type_predicate(class: ReferentId) -> kernel::Result<RolePredicate> {
    RolePredicate::new(
        legacy_type_classification_relation(),
        legacy_type_candidate_role(),
        BTreeMap::from([(legacy_type_class_role(), class)]),
    )
}

fn legacy_type_classification_shape() -> kernel::Result<RelationShape> {
    let candidate = Role::new(legacy_type_candidate_role(), Vec::new())?;
    let class = Role::new(legacy_type_class_role(), Vec::new())?;
    RelationShape::new(
        legacy_type_classification_relation(),
        BTreeMap::from([
            (candidate.id().clone(), candidate),
            (class.id().clone(), class),
        ]),
        Vec::new(),
    )
}

fn legacy_type_membership(
    candidate: &ReferentId,
    class: &ReferentId,
) -> kernel::Result<RelationalContent> {
    RelationalContent::new(
        legacy_type_classification_relation(),
        BTreeMap::from([
            (
                legacy_type_candidate_role(),
                Term::referent(candidate.clone()),
            ),
            (legacy_type_class_role(), Term::referent(class.clone())),
        ]),
    )
}

fn lower_models(
    declarations: &[AscriptionDecl],
    shapes: &BTreeMap<ReferentId, RelationShape>,
    projection: &Projection,
) -> kernel::Result<(
    BTreeMap<frontend::Name, Model>,
    BTreeMap<ReferentId, frontend::Span>,
)> {
    let mut models = BTreeMap::new();
    let mut source_spans = BTreeMap::new();
    for declaration in declarations.iter().filter(|item| item.kind == Kind::Model) {
        let model_id = projection
            .designations
            .global(declaration.subject.value.as_str())?;
        let mut referents = projection
            .types
            .iter()
            .chain(shapes.keys())
            .cloned()
            .map(|id| (id.clone(), Referent::new(id)))
            .collect::<BTreeMap<_, _>>();
        referents.insert(model_id.clone(), Referent::new(model_id.clone()));
        for id in projection
            .model_entities
            .get(&model_id)
            .into_iter()
            .flatten()
        {
            referents.insert(id.clone(), Referent::new(id.clone()));
        }
        for member in &declaration.body {
            collect_member_literal_referents(member, projection, &mut referents)?;
        }
        let (mut contents, mut occurrences, mut judgments) =
            legacy_type_memberships(&model_id, declaration, projection, &mut referents)?;
        let shell = Model::with_distinctions(
            model_id.clone(),
            referents.clone(),
            contents.clone(),
            shapes.clone(),
            occurrences.clone(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments.clone(),
        )?;
        let shell = wire::admit(shell);
        let mut occurrence_index = 0usize;
        for member in &declaration.body {
            let (lowered, span) = match member {
                Member::RelationalContent(surface) => (
                    vec![lower_clause_with(projection, shell.model(), surface, None)?],
                    Some(surface.span),
                ),
                Member::Focus(focus) => (
                    lower_focus(projection, shell.model(), focus)?,
                    Some(focus.span),
                ),
                _ => (Vec::new(), None),
            };
            for content in lowered {
                contents.insert(content.id().clone(), content.clone());
                let occurrence_id = synthetic_referent(
                    "assertion-occurrence",
                    &[model_id.as_str(), &occurrence_index.to_string()],
                );
                let judgment_id =
                    synthetic_referent("assertion-judgment", &[occurrence_id.as_str()]);
                occurrence_index += 1;
                referents.insert(occurrence_id.clone(), Referent::new(occurrence_id.clone()));
                referents.insert(judgment_id.clone(), Referent::new(judgment_id.clone()));
                occurrences.push(AssertionOccurrence::new(
                    occurrence_id.clone(),
                    content.id().clone(),
                    model_id.clone(),
                    model_id.clone(),
                ));
                judgments.push(Judgment::new(
                    judgment_id,
                    model_id.clone(),
                    model_id.clone(),
                    JudgmentTarget::Occurrence(occurrence_id.clone()),
                    JudgmentKind::Admitted {
                        policy: model_id.clone(),
                        basis: Vec::new(),
                    },
                    JudgmentStatus::Affirmed,
                ));
                if let Some(span) = span
                    && source_spans.insert(occurrence_id, span).is_some()
                {
                    return Err(kernel::KernelError::new(
                        "duplicate assertion occurrence source projection",
                    ));
                }
            }
        }
        let mut rules = Vec::new();
        for rule_decl in declarations.iter().filter(|item| {
            item.kind == Kind::DerivationRule
                && item
                    .subject
                    .value
                    .as_str()
                    .strip_prefix(&format!("{}/", declaration.subject.value.as_str()))
                    .is_some()
        }) {
            let rule_id = projection
                .designations
                .global(rule_decl.subject.value.as_str())?;
            referents.insert(rule_id.clone(), Referent::new(rule_id.clone()));
            let conclusion = rule_decl
                .body
                .iter()
                .find_map(|member| match member {
                    Member::RelationalContent(value) => Some(value),
                    _ => None,
                })
                .ok_or_else(|| kernel::KernelError::new("DerivationRule requires a conclusion"))?;
            let premises = rule_decl
                .body
                .iter()
                .find_map(|member| match member {
                    Member::When(value) => Some(value),
                    _ => None,
                })
                .ok_or_else(|| kernel::KernelError::new("DerivationRule requires when premises"))?;
            let binders = projection.rule_binders.get(&rule_id).ok_or_else(|| {
                kernel::KernelError::new("DerivationRule has no binder projection")
            })?;
            let premise_contents = premises
                .iter()
                .map(|surface| lower_clause_with(projection, shell.model(), surface, Some(binders)))
                .collect::<kernel::Result<Vec<_>>>()?;
            let conclusion_content =
                lower_clause_with(projection, shell.model(), conclusion, Some(binders))?;
            for content in premise_contents
                .iter()
                .chain(std::iter::once(&conclusion_content))
            {
                contents.insert(content.id().clone(), content.clone());
            }
            rules.push(DerivationRule::new(
                rule_id,
                model_id.clone(),
                model_id.clone(),
                Pattern::new(
                    premise_contents
                        .into_iter()
                        .map(|item| item.id().clone())
                        .collect(),
                )?,
                Pattern::new(vec![conclusion_content.id().clone()])?,
            )?);
        }
        let model = Model::with_distinctions(
            model_id,
            referents,
            contents,
            shapes.clone(),
            occurrences,
            Vec::new(),
            rules,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments,
        )?;
        models.insert(declaration.subject.value.clone(), model);
    }
    Ok((models, source_spans))
}

type LegacyTypeMemberships = (
    BTreeMap<kernel::ContentId, RelationalContent>,
    Vec<AssertionOccurrence>,
    Vec<Judgment>,
);

fn legacy_type_memberships(
    model: &ReferentId,
    declaration: &AscriptionDecl,
    projection: &Projection,
    referents: &mut BTreeMap<ReferentId, Referent>,
) -> kernel::Result<LegacyTypeMemberships> {
    let mut contents = BTreeMap::new();
    let mut occurrences = Vec::new();
    let mut judgments = Vec::new();
    let mut memberships = projection
        .model_entities
        .get(model)
        .into_iter()
        .flatten()
        .map(|entity| {
            let class = projection
                .entity_types
                .get(entity)
                .expect("declared model entity has a recorded legacy type");
            (entity.clone(), class.clone())
        })
        .collect::<BTreeSet<_>>();
    let literals = declaration
        .body
        .iter()
        .map(|member| member_literal_referents(member, projection))
        .collect::<kernel::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>();
    if !literals.is_empty() {
        let text = projection.designations.global("Text")?;
        if !projection.types.contains(&text) {
            return Err(kernel::KernelError::new(
                "model string literal requires declared Text Type",
            ));
        }
        memberships.extend(literals.into_iter().map(|literal| (literal, text.clone())));
    }
    for (entity, class) in memberships {
        let content = legacy_type_membership(&entity, &class)?;
        let occurrence = synthetic_referent(
            "legacy-type-membership-occurrence",
            &[model.as_str(), entity.as_str()],
        );
        let judgment =
            synthetic_referent("legacy-type-membership-judgment", &[occurrence.as_str()]);
        referents.insert(occurrence.clone(), Referent::new(occurrence.clone()));
        referents.insert(judgment.clone(), Referent::new(judgment.clone()));
        contents.insert(content.id().clone(), content.clone());
        occurrences.push(AssertionOccurrence::new(
            occurrence.clone(),
            content.id().clone(),
            model.clone(),
            model.clone(),
        ));
        judgments.push(Judgment::new(
            judgment,
            model.clone(),
            model.clone(),
            JudgmentTarget::Occurrence(occurrence),
            JudgmentKind::Admitted {
                policy: model.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        ));
    }
    Ok((contents, occurrences, judgments))
}

fn member_literal_referents(
    member: &Member,
    projection: &Projection,
) -> kernel::Result<Vec<ReferentId>> {
    let literal = |term: &SurfaceTerm| -> kernel::Result<Option<ReferentId>> {
        match term {
            SurfaceTerm::String(value) => Ok(Some(projection.designations.literal(&value.value)?)),
            SurfaceTerm::Entity(_) | SurfaceTerm::Template(_) | SurfaceTerm::Variable(_) => {
                Ok(None)
            }
        }
    };
    let terms = match member {
        Member::RelationalContent(clause) => clause.roles.values().collect::<Vec<_>>(),
        Member::Focus(focus) => focus.slots.iter().map(|slot| &slot.value).collect(),
        _ => Vec::new(),
    };
    terms
        .into_iter()
        .filter_map(|term| literal(term).transpose())
        .collect()
}

fn collect_member_literal_referents(
    member: &Member,
    projection: &Projection,
    referents: &mut BTreeMap<ReferentId, Referent>,
) -> kernel::Result<()> {
    for id in member_literal_referents(member, projection)? {
        referents.insert(id.clone(), Referent::new(id));
    }
    Ok(())
}

fn declare_literals(members: &[Member], table: &mut DesignationTable) {
    for member in members {
        match member {
            Member::RelationalContent(clause) => {
                for term in clause.roles.values() {
                    declare_term_literal(term, table);
                }
            }
            Member::When(clauses) | Member::Admit(clauses) | Member::Withdraw(clauses) => {
                for clause in clauses {
                    for term in clause.roles.values() {
                        declare_term_literal(term, table);
                    }
                }
            }
            Member::Focus(focus) => {
                for slot in &focus.slots {
                    declare_term_literal(&slot.value, table);
                }
            }
            _ => {}
        }
    }
}

fn declare_request_literals(request: &frontend::RequestDecl, table: &mut DesignationTable) {
    let clause = match request {
        frontend::RequestDecl::Find { pattern, .. } => Some(pattern),
        frontend::RequestDecl::Why { target, .. }
        | frontend::RequestDecl::Prevent { target, .. }
        | frontend::RequestDecl::Achieve { target, .. } => Some(target),
        frontend::RequestDecl::Diff { .. } => None,
    };
    if let Some(clause) = clause {
        for term in clause.roles.values() {
            declare_term_literal(term, table);
        }
    }
}

fn declare_term_literal(term: &SurfaceTerm, table: &mut DesignationTable) {
    if let SurfaceTerm::String(value) = term {
        table.declare_literal(&value.value);
    }
}

fn cardinality(value: frontend::Cardinality) -> kernel::Cardinality {
    match value {
        frontend::Cardinality::One => kernel::Cardinality::One,
        frontend::Cardinality::Maybe => kernel::Cardinality::Maybe,
        frontend::Cardinality::Some => kernel::Cardinality::Some,
        frontend::Cardinality::Many => kernel::Cardinality::Many,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        derive::Limits,
        frontend,
        intervention::{self, AchieveAll, InterventionLimits},
    };

    const BASE: &str = "Module: Type\n\nimpact/imports: RelationShape\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n";

    #[test]
    fn seals_base_and_preserves_request_order() {
        let program = compile(frontend::parse(&format!("{BASE}\nwhy in impact:\n    North imports Store\n\nfind all ?dependency in impact:\n    North imports ?dependency\n")).unwrap()).unwrap();
        assert_eq!(program.revisions().len(), 1);
        assert!(matches!(
            program.requests()[0],
            frontend::RequestDecl::Why { .. }
        ));
        assert!(matches!(
            program.requests()[1],
            frontend::RequestDecl::Find { .. }
        ));
    }

    #[test]
    fn explicit_renames_preserve_sealed_identity() {
        let before = compile(frontend::parse(BASE).unwrap()).unwrap();
        let mut designations = before.designations().clone();
        let model = designations.global("impact").unwrap();
        let relation = designations.global("impact/imports").unwrap();
        designations.retain_global("Module", "Component").unwrap();
        designations.retain_scoped(&model, "North", "Core").unwrap();
        designations
            .retain_role(&relation, "consumer", "importer")
            .unwrap();
        let renamed = BASE
            .replace("Module", "Component")
            .replace("consumer", "importer")
            .replace("North", "Core");
        let after = CompiledProgram::compile_with_designations(
            frontend::parse(&renamed).unwrap(),
            designations,
        )
        .unwrap();
        assert_eq!(
            wire::serialize(before.revision(&frontend::Name("impact".into())).unwrap()),
            wire::serialize(after.revision(&frontend::Name("impact".into())).unwrap()),
        );
    }

    #[test]
    fn legacy_role_domains_become_sealed_admitted_classifications() {
        let source = "Alpha: Type
Beta: Type
Text: Type

typed/pairs: RelationShape
    {left: Alpha} pairs {right: Beta}
    mode left -> right: many

typed/labels: RelationShape
    {owner: Alpha} labels {label: Text}
    mode owner -> label: many

typed: Model
    Alpha-0: Alpha
    Alpha-1: Alpha
    Beta-0: Beta
    Beta-1: Beta
    Alpha-0 pairs Beta-0
    Alpha-0 labels \"north\"
";
        let program = compile(frontend::parse(source).unwrap()).unwrap();
        let revision = program.revision(&frontend::Name("typed".into())).unwrap();
        let model = program.designations().global("typed").unwrap();
        let relation = program.designations().global("typed/pairs").unwrap();
        let labels = program.designations().global("typed/labels").unwrap();
        let left = program.designations().role(&relation, "left").unwrap();
        let right = program.designations().role(&relation, "right").unwrap();
        let owner = program.designations().role(&labels, "owner").unwrap();
        let label = program.designations().role(&labels, "label").unwrap();
        let alpha_0 = program.designations().scoped(&model, "Alpha-0").unwrap();
        let alpha_1 = program.designations().scoped(&model, "Alpha-1").unwrap();
        let beta_0 = program.designations().scoped(&model, "Beta-0").unwrap();
        let beta_1 = program.designations().scoped(&model, "Beta-1").unwrap();
        let north = program.designations().literal("north").unwrap();
        let content = |left_value: &ReferentId, right_value: &ReferentId| {
            RelationalContent::new(
                relation.clone(),
                BTreeMap::from([
                    (left.clone(), Term::referent(left_value.clone())),
                    (right.clone(), Term::referent(right_value.clone())),
                ]),
            )
            .unwrap()
        };
        let existing = content(&alpha_0, &beta_0);
        let target = content(&alpha_1, &beta_1);
        assert_eq!(
            revision.model().relation_shapes()[&relation]
                .roles()
                .values()
                .map(|role| role.admissibility().len())
                .collect::<Vec<_>>(),
            vec![1, 1],
        );
        assert!(revision.model().validate_content(&target, false).is_ok());
        assert!(
            revision
                .model()
                .validate_content(&content(&beta_0, &beta_1), false)
                .is_err()
        );
        assert!(
            revision
                .model()
                .validate_content(&content(&alpha_0, &alpha_1), false)
                .is_err()
        );
        assert!(
            revision
                .model()
                .admitted_contents()
                .binary_search(&existing)
                .is_ok()
        );
        let text_content = |owner_value: &ReferentId| {
            RelationalContent::new(
                labels.clone(),
                BTreeMap::from([
                    (owner.clone(), Term::referent(owner_value.clone())),
                    (label.clone(), Term::referent(north.clone())),
                ]),
            )
            .unwrap()
        };
        assert!(
            revision
                .model()
                .validate_content(&text_content(&alpha_1), false)
                .is_ok()
        );
        assert!(
            revision
                .model()
                .validate_content(&text_content(&beta_0), false)
                .is_err()
        );

        let sealed = wire::reload(&wire::serialize(revision)).unwrap();
        let AchieveAll::Complete(items) = intervention::achieve_all_minimal(
            &sealed,
            target.clone(),
            vec![relation],
            InterventionLimits::new(Limits::new(100, 10, 20_000), 100, 100),
        )
        .unwrap() else {
            panic!("sealed classifier domains must yield a complete addition frontier");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].admissions(), &[target]);
    }
}
