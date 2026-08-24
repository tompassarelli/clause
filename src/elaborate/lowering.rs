use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, Name, SurfaceClause, SurfaceTerm},
    intrinsic::Intrinsic,
    kernel::{
        self, Definition, Model, PatternId, ProposalPath, ProposalPathSegment, ProposalSubject,
        ReferentId, RelationShape, RelationalContent, Role, RoleId, Term,
    },
};

use super::{
    compilation::CompiledProgram,
    identifiers::{DesignationTable, synthetic_referent},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct Projection {
    pub(crate) designations: DesignationTable,
    pub(crate) grounded: BTreeSet<ReferentId>,
    pub(crate) model_referents: BTreeMap<ReferentId, BTreeSet<ReferentId>>,
    pub(crate) structural_fields: BTreeMap<ReferentId, BTreeMap<ReferentId, ReferentId>>,
    pub(crate) role_domains: BTreeMap<(ReferentId, RoleId), ReferentId>,
    pub(crate) focus_shapes: Vec<FocusShape>,
    pub(crate) rule_binders: BTreeMap<ReferentId, BinderTable>,
    pub(crate) request_binders: BTreeMap<usize, BinderTable>,
}

type RelativeProposalSpans = BTreeMap<Vec<ProposalPathSegment>, frontend::Span>;
type LocalTable = BTreeMap<Name, (ReferentId, Term, RelativeProposalSpans)>;

fn child_path(
    path: Option<&[ProposalPathSegment]>,
    segment: ProposalPathSegment,
) -> Option<Vec<ProposalPathSegment>> {
    path.map(|path| {
        let mut child = path.to_vec();
        child.push(segment);
        child
    })
}

fn materialize_proposal_spans(
    subject: ProposalSubject,
    relative: &RelativeProposalSpans,
    proposal_spans: &mut BTreeMap<ProposalPath, frontend::Span>,
) {
    for (segments, span) in relative {
        let path = segments
            .iter()
            .cloned()
            .fold(ProposalPath::new(subject.clone()), |path, segment| {
                path.child(segment)
            });
        proposal_spans.insert(path, *span);
    }
}

fn copy_relative_proposal_spans(
    prefix: &[ProposalPathSegment],
    source: &RelativeProposalSpans,
    destination: &mut RelativeProposalSpans,
) {
    for (segments, span) in source {
        let mut path = prefix.to_vec();
        path.extend(segments.iter().cloned());
        destination.insert(path, *span);
    }
}

pub(crate) fn membership_relation() -> ReferentId {
    kernel::membership_relation()
}

pub(crate) fn membership_member_role() -> RoleId {
    kernel::membership_member_role()
}

pub(crate) fn membership_group_role() -> RoleId {
    kernel::membership_group_role()
}

pub(crate) fn structural_domain(
    projection: &Projection,
    domain: &frontend::DomainName,
) -> ReferentId {
    if let Some(members) = structural_signature_members(domain.as_str(), "tuple") {
        let domains = members
            .into_iter()
            .map(|member| structural_domain(projection, &frontend::DomainName(member.to_owned())))
            .collect::<Vec<_>>();
        return structural_tuple_domain(&domains);
    }
    if let Some([element]) = structural_signature_members(domain.as_str(), "sequence")
        .map(Vec::into_boxed_slice)
        .as_deref()
    {
        let element = structural_domain(projection, &frontend::DomainName((*element).to_owned()));
        return kernel::structural_sequence_domain(&element);
    }
    projection
        .designations
        .global(domain.as_str())
        .unwrap_or_else(|_| synthetic_referent("structural-domain", &[domain.as_str()]))
}

fn structural_signature_members<'a>(domain: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let inner = domain
        .strip_prefix(&format!("@clause/{name}("))?
        .strip_suffix(')')?;
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut members = Vec::new();
    for (index, byte) in inner.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => {
                members.push(&inner[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    (depth == 0 && start < inner.len()).then(|| {
        members.push(&inner[start..]);
        members
    })
}

fn structural_tuple_domain(domains: &[ReferentId]) -> ReferentId {
    synthetic_referent(
        "structural-tuple-domain",
        &domains.iter().map(ReferentId::as_str).collect::<Vec<_>>(),
    )
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

#[derive(Clone, Debug)]
pub(crate) struct LoweredDefinitionGraph {
    pub(crate) dependencies: Vec<RelationalContent>,
    pub(crate) definition: Definition,
}

pub(crate) fn lower_pure_definition(
    projection: &Projection,
    model: &Model,
    surface: &frontend::PureDefinitionDecl,
    proposal_spans: &mut BTreeMap<ProposalPath, frontend::Span>,
) -> kernel::Result<LoweredDefinitionGraph> {
    let id = projection
        .designations
        .scoped(model.id(), surface.name.value.as_str())?;
    require_term_referent(model, &id, "pure definition name")?;

    let mut locals = BTreeMap::new();
    let mut dependencies = Vec::new();
    for local in &surface.locals {
        let expected = definition_term_domain(projection, model, &local.denotation, &locals)?;
        let mut relative = RelativeProposalSpans::new();
        let denotation = lower_term_traced(
            projection,
            model,
            &expected,
            &local.denotation,
            None,
            Some(&locals),
            &mut dependencies,
            Some(&[]),
            &mut relative,
            proposal_spans,
        )?;
        if locals
            .insert(local.name.value.clone(), (expected, denotation, relative))
            .is_some()
        {
            return Err(kernel::KernelError::new(
                "duplicate pure definition local binding",
            ));
        }
    }

    let expected = structural_domain(projection, &surface.domain);
    let mut relative = RelativeProposalSpans::new();
    let denotation = lower_term_traced(
        projection,
        model,
        &expected,
        &surface.result,
        None,
        Some(&locals),
        &mut dependencies,
        Some(&[]),
        &mut relative,
        proposal_spans,
    )?;
    materialize_proposal_spans(
        ProposalSubject::Definition(id.clone()),
        &relative,
        proposal_spans,
    );
    Ok(LoweredDefinitionGraph {
        dependencies,
        definition: Definition::new(id, denotation),
    })
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum BinderKey {
    Named(frontend::VariableName),
    Anonymous(usize, usize),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BinderTable(BTreeMap<BinderKey, PatternId>);

impl BinderTable {
    fn declare<'a>(
        designations: &mut DesignationTable,
        scope: &ReferentId,
        clauses: impl IntoIterator<Item = &'a SurfaceClause>,
        include_anonymous: bool,
    ) -> kernel::Result<Self> {
        fn collect(
            term: &SurfaceTerm,
            include_anonymous: bool,
            first: &mut BTreeMap<BinderKey, (usize, usize)>,
        ) {
            match term {
                SurfaceTerm::Variable(variable) => {
                    first
                        .entry(BinderKey::Named(variable.value.clone()))
                        .and_modify(|span| {
                            *span = (*span).min((variable.span.line, variable.span.column));
                        })
                        .or_insert((variable.span.line, variable.span.column));
                }
                SurfaceTerm::AnonymousHole(span) if include_anonymous => {
                    first.insert(
                        BinderKey::Anonymous(span.line, span.column),
                        (span.line, span.column),
                    );
                }
                SurfaceTerm::Application(application) => {
                    for term in application.roles.values() {
                        collect(term, include_anonymous, first);
                    }
                }
                SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
                    for term in values {
                        collect(term, include_anonymous, first);
                    }
                }
                SurfaceTerm::Product { fields, .. } => {
                    for term in fields.values() {
                        collect(term, include_anonymous, first);
                    }
                }
                SurfaceTerm::Referent(_)
                | SurfaceTerm::Local(_)
                | SurfaceTerm::Template(_)
                | SurfaceTerm::AnonymousHole(_)
                | SurfaceTerm::String(_)
                | SurfaceTerm::F32(_)
                | SurfaceTerm::Int(_)
                | SurfaceTerm::Bool(_)
                | SurfaceTerm::Intrinsic(_) => {}
            }
        }

        let mut first = BTreeMap::new();
        for clause in clauses {
            for term in clause.roles.values() {
                collect(term, include_anonymous, &mut first);
            }
        }
        let mut ordered = first.into_iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.1.cmp(&right.1).then(left.0.cmp(&right.0)));
        let mut binders = BTreeMap::new();
        for (index, (key, _)) in ordered.into_iter().enumerate() {
            let id = designations.declare_pattern(scope, &format!("binder-{index}"))?;
            binders.insert(key, id);
        }
        Ok(Self(binders))
    }

    pub(crate) fn declare_alpha<'a>(
        designations: &mut DesignationTable,
        scope: &ReferentId,
        clauses: impl IntoIterator<Item = &'a SurfaceClause>,
    ) -> kernel::Result<Self> {
        Self::declare(designations, scope, clauses, false)
    }

    pub(crate) fn declare_query(
        designations: &mut DesignationTable,
        scope: &ReferentId,
        clause: &SurfaceClause,
    ) -> kernel::Result<Self> {
        Self::declare(designations, scope, std::iter::once(clause), true)
    }

    pub(crate) fn get(&self, name: &frontend::VariableName) -> kernel::Result<PatternId> {
        self.0
            .get(&BinderKey::Named(name.clone()))
            .cloned()
            .ok_or_else(|| {
                kernel::KernelError::new(format!("unbound pattern variable '{}'", name.as_str()))
            })
    }

    pub(crate) fn anonymous(&self, span: frontend::Span) -> kernel::Result<PatternId> {
        self.0
            .get(&BinderKey::Anonymous(span.line, span.column))
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("unbound anonymous query hole"))
    }

    pub(crate) fn column(&self, column: &frontend::QueryColumnDecl) -> kernel::Result<PatternId> {
        match &column.label {
            Some(label) => self.get(label),
            None => self.anonymous(column.span),
        }
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
    let graph = lower_clause_graph_with(projection, model, surface, binders)?;
    for dependency in &graph.dependencies {
        if model.content(dependency.id()) != Some(dependency) {
            return Err(kernel::KernelError::new(
                "recursive term dependency is not registered in this Model",
            ));
        }
    }
    model.validate_content(&graph.root, binders.is_some())?;
    Ok(graph.root)
}

#[derive(Clone, Debug)]
pub(crate) struct LoweredContentGraph {
    pub(crate) dependencies: Vec<RelationalContent>,
    pub(crate) root: RelationalContent,
}

pub(crate) fn lower_clause_graph_with(
    projection: &Projection,
    model: &Model,
    surface: &SurfaceClause,
    binders: Option<&BinderTable>,
) -> kernel::Result<LoweredContentGraph> {
    let mut ignored = BTreeMap::new();
    lower_clause_graph_traced(projection, model, surface, binders, &mut ignored)
}

pub(crate) fn lower_clause_graph_traced(
    projection: &Projection,
    model: &Model,
    surface: &SurfaceClause,
    binders: Option<&BinderTable>,
    proposal_spans: &mut BTreeMap<ProposalPath, frontend::Span>,
) -> kernel::Result<LoweredContentGraph> {
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
    let mut dependencies = Vec::new();
    let mut relative = RelativeProposalSpans::new();
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
        let path = [ProposalPathSegment::Role(role_id.clone())];
        if roles
            .insert(
                role_id,
                lower_term_traced(
                    projection,
                    model,
                    expected,
                    surface_term,
                    binders,
                    None,
                    &mut dependencies,
                    Some(&path),
                    &mut relative,
                    proposal_spans,
                )?,
            )
            .is_some()
        {
            return Err(kernel::KernelError::new("duplicate relational role"));
        }
    }
    let root = RelationalContent::new(relation_id, roles)?;
    materialize_proposal_spans(
        ProposalSubject::Content(root.id().clone()),
        &relative,
        proposal_spans,
    );
    Ok(LoweredContentGraph { dependencies, root })
}

fn lower_term(
    projection: &Projection,
    model: &Model,
    expected: &ReferentId,
    term: &SurfaceTerm,
    binders: Option<&BinderTable>,
    locals: Option<&LocalTable>,
    dependencies: &mut Vec<RelationalContent>,
) -> kernel::Result<Term> {
    let mut ignored_relative = RelativeProposalSpans::new();
    let mut ignored_proposals = BTreeMap::new();
    lower_term_traced(
        projection,
        model,
        expected,
        term,
        binders,
        locals,
        dependencies,
        None,
        &mut ignored_relative,
        &mut ignored_proposals,
    )
}

#[allow(clippy::too_many_arguments)]
fn lower_term_traced(
    projection: &Projection,
    model: &Model,
    expected: &ReferentId,
    term: &SurfaceTerm,
    binders: Option<&BinderTable>,
    locals: Option<&LocalTable>,
    dependencies: &mut Vec<RelationalContent>,
    path: Option<&[ProposalPathSegment]>,
    relative_spans: &mut RelativeProposalSpans,
    proposal_spans: &mut BTreeMap<ProposalPath, frontend::Span>,
) -> kernel::Result<Term> {
    if let Some(path) = path {
        relative_spans.insert(path.to_vec(), surface_term_span(term));
    }
    let empty_locals = BTreeMap::new();
    let domain_locals = locals.unwrap_or(&empty_locals);
    match term {
        SurfaceTerm::Variable(value) => {
            let binders = binders.ok_or_else(|| {
                kernel::KernelError::new("pattern variable is not valid in ground content")
            })?;
            Ok(Term::pattern(binders.get(&value.value)?))
        }
        SurfaceTerm::AnonymousHole(span) => {
            let binders = binders.ok_or_else(|| {
                kernel::KernelError::new("anonymous hole is not valid in ground content")
            })?;
            Ok(Term::pattern(binders.anonymous(*span)?))
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
        SurfaceTerm::F32(value) => Term::f32_bits(value.value),
        SurfaceTerm::Int(value) => Ok(Term::int(value.value)),
        SurfaceTerm::Bool(value) => Ok(Term::boolean(value.value)),
        SurfaceTerm::Tuple { values, .. } => Term::tuple(
            expected.clone(),
            values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let domain = definition_term_domain(projection, model, value, domain_locals)?;
                    let child = child_path(path, ProposalPathSegment::TupleIndex(index));
                    let lowered = lower_term_traced(
                        projection,
                        model,
                        &domain,
                        value,
                        binders,
                        locals,
                        dependencies,
                        child.as_deref(),
                        relative_spans,
                        proposal_spans,
                    )?;
                    Ok((domain, lowered))
                })
                .collect::<kernel::Result<Vec<_>>>()?,
        ),
        SurfaceTerm::Product { shape, fields, .. } => {
            let shape = projection.designations.global(shape.value.as_str())?;
            Term::labelled_product(
                shape.clone(),
                fields
                    .iter()
                    .map(|(label, value)| {
                        let field = projection.designations.scoped(&shape, label.as_str())?;
                        let domain = projection
                            .structural_fields
                            .get(&shape)
                            .and_then(|fields| fields.get(&field))
                            .cloned()
                            .ok_or_else(|| {
                                kernel::KernelError::new(
                                    "labelled product field has no declared shape binding",
                                )
                            })?;
                        let child =
                            child_path(path, ProposalPathSegment::ProductField(field.clone()));
                        Ok((
                            field,
                            lower_term_traced(
                                projection,
                                model,
                                &domain,
                                value,
                                binders,
                                locals,
                                dependencies,
                                child.as_deref(),
                                relative_spans,
                                proposal_spans,
                            )?,
                        ))
                    })
                    .collect::<kernel::Result<BTreeMap<_, _>>>()?,
            )
        }
        SurfaceTerm::Sequence { values, .. } => Term::sequence(
            expected.clone(),
            definition_term_domain(
                projection,
                model,
                values.first().expect("surface sequence is nonempty"),
                domain_locals,
            )?,
            values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let domain = definition_term_domain(projection, model, value, domain_locals)?;
                    let child = child_path(path, ProposalPathSegment::SequenceIndex(index));
                    lower_term_traced(
                        projection,
                        model,
                        &domain,
                        value,
                        binders,
                        locals,
                        dependencies,
                        child.as_deref(),
                        relative_spans,
                        proposal_spans,
                    )
                })
                .collect::<kernel::Result<Vec<_>>>()?,
        ),
        SurfaceTerm::Intrinsic(value) => Intrinsic::from_source_name(value.value.as_str())
            .map(|intrinsic| Term::referent(intrinsic.callable_identity()))
            .ok_or_else(|| kernel::KernelError::new("unknown pure intrinsic identity")),
        SurfaceTerm::Referent(value) => {
            let referent = projection
                .designations
                .scoped(model.id(), value.value.as_str())?;
            require_term_referent(model, &referent, "referent")?;
            Ok(Term::referent(referent))
        }
        SurfaceTerm::Local(value) => {
            let (actual, denotation, local_spans) = locals
                .and_then(|locals| locals.get(&value.value))
                .ok_or_else(|| {
                    kernel::KernelError::new(format!(
                        "unbound pure definition local '{}'",
                        value.value.as_str()
                    ))
                })?;
            if actual != expected {
                return Err(kernel::KernelError::new(
                    "pure definition local does not satisfy its use domain",
                ));
            }
            if let Some(path) = path {
                copy_relative_proposal_spans(path, local_spans, relative_spans);
            }
            Ok(denotation.clone())
        }
        SurfaceTerm::Application(application) => {
            if let Some(intrinsic) =
                Intrinsic::from_source_name(application.relation.value.as_str())
            {
                let mut roles = BTreeMap::new();
                let mut content_relative = RelativeProposalSpans::new();
                for (surface_role, surface_term) in &application.roles {
                    let role = intrinsic
                        .role_named(surface_role.as_str())
                        .ok_or_else(|| kernel::KernelError::new("unknown pure intrinsic role"))?;
                    let role_path = [ProposalPathSegment::Role(role.clone())];
                    let lowered = if let SurfaceTerm::Local(value) = surface_term {
                        let (_, denotation, local_spans) = locals
                            .and_then(|locals| locals.get(&value.value))
                            .ok_or_else(|| {
                                kernel::KernelError::new(format!(
                                    "unbound pure definition local '{}'",
                                    value.value.as_str()
                                ))
                            })?;
                        content_relative.insert(role_path.to_vec(), value.span);
                        copy_relative_proposal_spans(
                            &role_path,
                            local_spans,
                            &mut content_relative,
                        );
                        denotation.clone()
                    } else {
                        lower_term_traced(
                            projection,
                            model,
                            expected,
                            surface_term,
                            binders,
                            locals,
                            dependencies,
                            Some(&role_path),
                            &mut content_relative,
                            proposal_spans,
                        )?
                    };
                    roles.insert(role, lowered);
                }
                let content = RelationalContent::new(intrinsic.relation(), roles)?;
                materialize_proposal_spans(
                    ProposalSubject::Content(content.id().clone()),
                    &content_relative,
                    proposal_spans,
                );
                if let Some(path) = path {
                    let application_path = child_path(
                        Some(path),
                        ProposalPathSegment::Application(content.id().clone()),
                    )
                    .expect("an authored application path has one child");
                    relative_spans.insert(application_path, application.span);
                }
                if !dependencies
                    .iter()
                    .any(|dependency| dependency.id() == content.id())
                {
                    dependencies.push(content.clone());
                }
                return Ok(Term::application(content.id().clone()));
            }
            let relation_id = projection
                .designations
                .global(application.relation.value.as_str())?;
            let relation = model.relation_shapes().get(&relation_id).ok_or_else(|| {
                kernel::KernelError::new(format!(
                    "undeclared RelationShape '{}'",
                    application.relation.value.as_str()
                ))
            })?;
            let result_role = projection
                .designations
                .role(&relation_id, application.result.value.as_str())?;
            projection
                .role_domains
                .get(&(relation_id.clone(), result_role.clone()))
                .ok_or_else(|| {
                    kernel::KernelError::new(
                        "application result has no authoring domain projection",
                    )
                })?;
            let mut roles = BTreeMap::new();
            let mut content_relative = RelativeProposalSpans::new();
            for (surface_role, surface_term) in &application.roles {
                let role_id = projection
                    .designations
                    .role(&relation_id, surface_role.as_str())?;
                let role_domain = projection
                    .role_domains
                    .get(&(relation_id.clone(), role_id.clone()))
                    .ok_or_else(|| {
                        kernel::KernelError::new(
                            "application role has no authoring domain projection",
                        )
                    })?;
                let role_path = [ProposalPathSegment::Role(role_id.clone())];
                let lowered = lower_term_traced(
                    projection,
                    model,
                    role_domain,
                    surface_term,
                    binders,
                    locals,
                    dependencies,
                    Some(&role_path),
                    &mut content_relative,
                    proposal_spans,
                )?;
                if roles.insert(role_id, lowered).is_some() {
                    return Err(kernel::KernelError::new(
                        "duplicate recursive application role",
                    ));
                }
            }
            let supplied = roles.keys().cloned().collect::<BTreeSet<_>>();
            let matching = relation
                .lookup()
                .iter()
                .filter(|mode| {
                    mode.cardinality() == &kernel::Cardinality::One
                        && mode.sought() == std::slice::from_ref(&result_role)
                        && mode.known().iter().cloned().collect::<BTreeSet<_>>() == supplied
                })
                .count();
            if matching != 1 {
                return Err(kernel::KernelError::new(
                    "recursive term requires exactly one single-result lookup contract",
                ));
            }
            let content = RelationalContent::new(relation_id, roles)?;
            materialize_proposal_spans(
                ProposalSubject::Content(content.id().clone()),
                &content_relative,
                proposal_spans,
            );
            if let Some(path) = path {
                let application_path = child_path(
                    Some(path),
                    ProposalPathSegment::Application(content.id().clone()),
                )
                .expect("an authored application path has one child");
                relative_spans.insert(application_path, application.span);
            }
            if let Some(existing) = dependencies
                .iter()
                .find(|dependency| dependency.id() == content.id())
            {
                if existing != &content {
                    return Err(kernel::KernelError::new(
                        "recursive content identity collision",
                    ));
                }
            } else {
                dependencies.push(content.clone());
            }
            Ok(Term::application(content.id().clone()))
        }
        SurfaceTerm::Template(_) => Err(kernel::KernelError::new(
            "correlated referent templates are only valid inside a focus block",
        )),
    }
}

fn surface_term_span(term: &SurfaceTerm) -> frontend::Span {
    match term {
        SurfaceTerm::Referent(value) => value.span,
        SurfaceTerm::Local(value) => value.span,
        SurfaceTerm::Template(value) => value.span,
        SurfaceTerm::Variable(value) => value.span,
        SurfaceTerm::AnonymousHole(span) => *span,
        SurfaceTerm::String(value) => value.span,
        SurfaceTerm::F32(value) => value.span,
        SurfaceTerm::Int(value) => value.span,
        SurfaceTerm::Bool(value) => value.span,
        SurfaceTerm::Tuple { span, .. }
        | SurfaceTerm::Product { span, .. }
        | SurfaceTerm::Sequence { span, .. } => *span,
        SurfaceTerm::Intrinsic(value) => value.span,
        SurfaceTerm::Application(value) => value.span,
    }
}

fn definition_term_domain(
    projection: &Projection,
    model: &Model,
    term: &SurfaceTerm,
    locals: &LocalTable,
) -> kernel::Result<ReferentId> {
    match term {
        SurfaceTerm::Application(application) => {
            if Intrinsic::from_source_name(application.relation.value.as_str()).is_some() {
                return Ok(structural_domain(projection, &application.domain));
            }
            let relation = projection
                .designations
                .global(application.relation.value.as_str())?;
            let result = projection
                .designations
                .role(&relation, application.result.value.as_str())?;
            projection
                .role_domains
                .get(&(relation, result))
                .cloned()
                .ok_or_else(|| {
                    kernel::KernelError::new(
                        "pure definition application has no result domain projection",
                    )
                })
        }
        SurfaceTerm::Local(value) => locals
            .get(&value.value)
            .map(|(domain, _, _)| domain.clone())
            .ok_or_else(|| {
                kernel::KernelError::new(format!(
                    "unbound pure definition local '{}'",
                    value.value.as_str()
                ))
            }),
        SurfaceTerm::Referent(value) => {
            let referent = projection
                .designations
                .scoped(model.id(), value.value.as_str())?;
            let member = membership_member_role();
            let group = membership_group_role();
            let domains = model
                .admitted_contents()
                .iter()
                .filter(|content| content.relation() == &membership_relation())
                .filter(|content| {
                    content.roles().get(&member) == Some(&Term::referent(referent.clone()))
                })
                .filter_map(|content| content.roles().get(&group)?.referent_id().cloned())
                .collect::<BTreeSet<_>>();
            let mut domains = domains.into_iter();
            let Some(domain) = domains.next() else {
                return Err(kernel::KernelError::new(
                    "pure definition referent must have one admitted domain",
                ));
            };
            if domains.next().is_some() {
                return Err(kernel::KernelError::new(
                    "pure definition referent must have one admitted domain",
                ));
            }
            Ok(domain)
        }
        SurfaceTerm::String(_) => Err(kernel::KernelError::new(
            "pure definition literals are not supported by this lowering boundary",
        )),
        SurfaceTerm::F32(_) => Ok(structural_domain(
            projection,
            &frontend::DomainName("F32".to_owned()),
        )),
        SurfaceTerm::Int(_) => Ok(structural_domain(
            projection,
            &frontend::DomainName("Int".to_owned()),
        )),
        SurfaceTerm::Bool(_) => Ok(structural_domain(
            projection,
            &frontend::DomainName("Bool".to_owned()),
        )),
        SurfaceTerm::Product { shape, .. } => Ok(structural_domain(
            projection,
            &frontend::DomainName(shape.value.0.clone()),
        )),
        SurfaceTerm::Tuple { values, .. } => {
            let domains = values
                .iter()
                .map(|value| definition_term_domain(projection, model, value, locals))
                .collect::<kernel::Result<Vec<_>>>()?;
            Ok(structural_tuple_domain(&domains))
        }
        SurfaceTerm::Sequence { values, .. } => {
            let element = definition_term_domain(
                projection,
                model,
                values.first().expect("surface sequence is nonempty"),
                locals,
            )?;
            Ok(kernel::structural_sequence_domain(&element))
        }
        SurfaceTerm::Intrinsic(_) => Err(kernel::KernelError::new(
            "intrinsic identity is only valid as an intrinsic application role",
        )),
        SurfaceTerm::Variable(_) | SurfaceTerm::AnonymousHole(_) | SurfaceTerm::Template(_) => Err(
            kernel::KernelError::new("pure definitions require closed terms"),
        ),
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
            let expected_focused =
                &projection.role_domains[&(shape.relation.clone(), shape.focused_role.clone())];
            let focused_name = format!(
                "{}{}{}",
                focus.template.prefix.value, number, focus.template.suffix.value
            );
            require_authored_membership(
                projection,
                model,
                &focused,
                expected_focused,
                &focused_name,
                "focused referent",
            )?;
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
            let authored = format!(
                "{}{}{}",
                template.prefix.value, number, template.suffix.value
            );
            require_authored_membership(
                projection,
                model,
                &referent,
                expected,
                &authored,
                "focused slot referent",
            )?;
            Ok(Term::referent(referent))
        }
        _ => {
            let mut dependencies = Vec::new();
            let term = lower_term(
                projection,
                model,
                expected,
                term,
                None,
                None,
                &mut dependencies,
            )?;
            if !dependencies.is_empty() {
                return Err(kernel::KernelError::new(
                    "recursive focused terms require focused graph lowering",
                ));
            }
            Ok(term)
        }
    }
}

fn require_authored_membership(
    projection: &Projection,
    model: &Model,
    referent: &ReferentId,
    expected: &ReferentId,
    authored: &str,
    where_: &str,
) -> kernel::Result<()> {
    let required = membership_content(referent.clone(), expected.clone())?;
    if model.operative_status(&required) == kernel::OpenWorldStatus::Admitted {
        return Ok(());
    }
    let domain = projection
        .designations
        .global_name(expected)
        .unwrap_or_else(|| expected.as_str());
    Err(kernel::KernelError::new(format!(
        "{where_} '{authored}' is not a member of '{domain}'"
    )))
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
