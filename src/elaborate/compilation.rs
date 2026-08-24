use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use crate::{
    frontend::{self, Declaration, Kind, Member, ShapePartDecl, SurfaceTerm},
    intrinsic::{Intrinsic, IntrinsicRole},
    kernel::{
        self, AssertionOccurrence, Definition, DerivationRule, Judgment, JudgmentKind,
        JudgmentStatus, JudgmentTarget, LookupMode, Model, Pattern, ProposalPath, Referent,
        ReferentId, RelationShape, RelationalContent, Role, RolePredicate, StructuralContract,
        StructuralFailureClass, StructuralForm, Term,
    },
    wire,
};

/// The disposition of a source-anchored compilation diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompileDiagnosticStatus {
    RejectedProposal,
}

/// Deterministic derived evidence for one authored proposal rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileDiagnostic {
    rank: usize,
    status: CompileDiagnosticStatus,
    class: StructuralFailureClass,
    path: ProposalPath,
    presentation: Vec<String>,
    span: frontend::Span,
}

impl CompileDiagnostic {
    pub fn rank(&self) -> usize {
        self.rank
    }

    pub fn status(&self) -> CompileDiagnosticStatus {
        self.status
    }

    pub fn class(&self) -> StructuralFailureClass {
        self.class
    }

    pub fn path(&self) -> &ProposalPath {
        &self.path
    }

    pub fn presentation(&self) -> &[String] {
        &self.presentation
    }

    pub fn span(&self) -> frontend::Span {
        self.span
    }
}

/// Compilation failure with optional source projection for a typed kernel
/// rejection. The underlying kernel evidence remains available unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileError {
    kernel: kernel::KernelError,
    diagnostic: Option<CompileDiagnostic>,
}

impl CompileError {
    fn from_kernel(
        kernel: kernel::KernelError,
        proposal_spans: &BTreeMap<ProposalPath, frontend::Span>,
        designations: &DesignationTable,
    ) -> Self {
        let diagnostic = kernel.structural_failure().and_then(|failure| {
            proposal_spans
                .get(failure.path())
                .copied()
                .map(|span| CompileDiagnostic {
                    rank: 1,
                    status: CompileDiagnosticStatus::RejectedProposal,
                    class: failure.class(),
                    path: failure.path().clone(),
                    presentation: designations.proposal_path_presentation(failure.path()),
                    span,
                })
        });
        Self { kernel, diagnostic }
    }

    pub fn diagnostic(&self) -> Option<&CompileDiagnostic> {
        self.diagnostic.as_ref()
    }

    pub fn kernel_error(&self) -> &kernel::KernelError {
        &self.kernel
    }
}

impl From<kernel::KernelError> for CompileError {
    fn from(kernel: kernel::KernelError) -> Self {
        Self {
            kernel,
            diagnostic: None,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kernel.fmt(formatter)
    }
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kernel)
    }
}

pub type CompileResult<T> = std::result::Result<T, CompileError>;

use super::{
    identifiers::{DesignationTable, synthetic_referent},
    lowering::{
        BinderTable, LoweredContentGraph, LoweredDefinitionGraph, Projection,
        lower_clause_graph_with, lower_clause_with, lower_definition, lower_focus,
        lower_pure_definition, lower_shape_binding, membership_content, membership_group_role,
        membership_member_role, membership_relation, membership_shape, structural_domain,
    },
    resolution::Resolver,
};

/// Sealed revisions indexed by authored navigation names, plus the source
/// projection required to resolve requests without putting designations into
/// semantic identity.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    revisions: BTreeMap<frontend::Name, kernel::Revision>,
    context_revision: Option<kernel::Revision>,
    requests: Vec<frontend::RequestDecl>,
    projection: Projection,
    source_spans: BTreeMap<ReferentId, frontend::Span>,
    proposal_spans: BTreeMap<ProposalPath, frontend::Span>,
}

impl CompiledProgram {
    pub fn revisions(&self) -> &BTreeMap<frontend::Name, kernel::Revision> {
        &self.revisions
    }
    pub fn requests(&self) -> &[frontend::RequestDecl] {
        &self.requests
    }

    /// The root Revision compiled in an exact caller-owned Model context.
    pub fn context_revision(&self) -> Option<&kernel::Revision> {
        self.context_revision.as_ref()
    }

    pub fn context_model(&self) -> Option<&kernel::Model> {
        self.context_revision.as_ref().map(kernel::Revision::model)
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

    pub(crate) fn request_column(
        &self,
        index: usize,
        column: &frontend::QueryColumnDecl,
    ) -> kernel::Result<kernel::PatternId> {
        self.projection
            .request_binders
            .get(&index)
            .ok_or_else(|| kernel::KernelError::new("request has no pattern-binder projection"))?
            .column(column)
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

    /// Locate one semantic proposal path in the source projection.
    pub fn proposal_span(&self, path: &ProposalPath) -> Option<frontend::Span> {
        self.proposal_spans.get(path).copied()
    }

    /// Compile after an explicit designation rename transaction. The table is
    /// projection state: retaining a designation changes source terms without
    /// changing the referents, roles, or binders sealed into the Revision.
    pub fn compile_with_designations(
        program: frontend::Program,
        designations: DesignationTable,
    ) -> CompileResult<Self> {
        compile_named(program, designations)
    }

    /// Compile direct Model content while preserving a caller-maintained
    /// designation projection across an explicit rename transaction.
    pub fn compile_in_with_designations(
        program: frontend::Program,
        context: ModelContext,
        designations: DesignationTable,
    ) -> CompileResult<Self> {
        compile_context(program, context, designations)
    }
}

/// Exact semantic scope for a source fragment that does not declare a Model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelContext {
    model: ReferentId,
}

impl ModelContext {
    pub fn new(model: ReferentId) -> Self {
        Self { model }
    }

    pub fn id(&self) -> &ReferentId {
        &self.model
    }
}

pub fn compile(program: frontend::Program) -> CompileResult<CompiledProgram> {
    compile_named(program, DesignationTable::new())
}

pub fn compile_in(
    program: frontend::Program,
    context: ModelContext,
) -> CompileResult<CompiledProgram> {
    compile_context(program, context, DesignationTable::new())
}

pub fn compile_in_with_designations(
    program: frontend::Program,
    context: ModelContext,
    designations: DesignationTable,
) -> CompileResult<CompiledProgram> {
    compile_context(program, context, designations)
}

fn compile_named(
    program: frontend::Program,
    designations: DesignationTable,
) -> CompileResult<CompiledProgram> {
    if !program.top_level.is_empty() {
        return Err(kernel::KernelError::new(
            "direct top-level Model content requires an explicit ModelContext",
        )
        .into());
    }
    let declarations = declaration_map(&program.declarations)?;
    let mut projection = declare_projection(&program, designations)?;
    let relation_shapes = lower_relation_shapes(&program.declarations, &mut projection)?;
    let mut proposal_spans = BTreeMap::new();
    let (models, source_spans) = lower_models(
        &program.declarations,
        &relation_shapes,
        &projection,
        &mut proposal_spans,
    )
    .map_err(|error| CompileError::from_kernel(error, &proposal_spans, &projection.designations))?;
    let (revisions, source_spans) = {
        let mut resolver = Resolver::new(&declarations, models, &projection, source_spans);
        for declaration in &program.declarations {
            match declaration.kind {
                Kind::Enumeration | Kind::BindingShape | Kind::Model | Kind::Revision => {
                    resolver.revision(&declaration.subject.value)?;
                }
                Kind::Delta => {
                    resolver.delta(&declaration.subject.value)?;
                }
                Kind::Grounding | Kind::RelationShape | Kind::DerivationRule => {}
            }
        }
        (resolver.revisions, resolver.source_spans)
    };
    Ok(CompiledProgram {
        revisions,
        context_revision: None,
        requests: program.requests,
        projection,
        source_spans,
        proposal_spans,
    })
}

fn compile_context(
    program: frontend::Program,
    context: ModelContext,
    designations: DesignationTable,
) -> CompileResult<CompiledProgram> {
    declaration_map(&program.declarations)?;
    if !program.requests.is_empty()
        || program.declarations.iter().any(|declaration| {
            !matches!(
                declaration.kind,
                Kind::Grounding | Kind::BindingShape | Kind::RelationShape
            )
        })
    {
        return Err(kernel::KernelError::new(
            "ModelContext fragments may contain groundings, RelationShapes, and direct Model content",
        )
        .into());
    }
    let mut projection = declare_projection(&program, designations)?;
    declare_model_members(&context.model, &program.top_level, &mut projection)?;
    let relation_shapes = lower_relation_shapes(&program.declarations, &mut projection)?;
    let mut proposal_spans = BTreeMap::new();
    let (model, source_spans) = lower_context_model(
        context.model,
        &program.top_level,
        &relation_shapes,
        &projection,
        &mut proposal_spans,
    )
    .map_err(|error| CompileError::from_kernel(error, &proposal_spans, &projection.designations))?;
    Ok(CompiledProgram {
        revisions: BTreeMap::new(),
        context_revision: Some(wire::admit(model)),
        requests: Vec::new(),
        projection,
        source_spans,
        proposal_spans,
    })
}

fn declaration_map(
    declarations: &[Declaration],
) -> kernel::Result<BTreeMap<frontend::Name, &Declaration>> {
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
            Kind::Enumeration | Kind::BindingShape | Kind::Model => {
                projection.designations.declare_model(name)?
            }
            Kind::Grounding
            | Kind::RelationShape
            | Kind::DerivationRule
            | Kind::Revision
            | Kind::Delta => projection.designations.declare_global(name)?,
        };
        if matches!(
            declaration.kind,
            Kind::Grounding | Kind::Enumeration | Kind::BindingShape | Kind::Model
        ) {
            projection.grounded.insert(id);
        }
    }
    for declaration in &program.declarations {
        match declaration.kind {
            Kind::RelationShape => declare_relation_projection(declaration, &mut projection)?,
            Kind::Enumeration | Kind::BindingShape | Kind::Model => {
                declare_model_projection(declaration, &mut projection)?
            }
            Kind::DerivationRule => declare_rule_projection(declaration, &mut projection)?,
            _ => {}
        }
        declare_literals(&declaration.body, &mut projection.designations);
    }
    declare_literals(&program.top_level, &mut projection.designations);
    for (index, request) in program.requests.iter().enumerate() {
        declare_request_literals(request, &mut projection.designations);
        let ordinal = index.to_string();
        let scope = synthetic_referent("request-pattern-scope", &[&ordinal]);
        let binders =
            match request {
                frontend::RequestDecl::Any { pattern, .. }
                | frontend::RequestDecl::Select { pattern, .. } => Some(
                    BinderTable::declare_query(&mut projection.designations, &scope, pattern)?,
                ),
                frontend::RequestDecl::Find { pattern, .. } => Some(BinderTable::declare_alpha(
                    &mut projection.designations,
                    &scope,
                    std::iter::once(pattern),
                )?),
                _ => None,
            };
        if let Some(binders) = binders {
            projection.request_binders.insert(index, binders);
        }
    }
    Ok(projection)
}

fn declare_rule_projection(
    declaration: &Declaration,
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
    declaration: &Declaration,
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
            ShapePartDecl::Role { id, domain } => {
                let role = projection
                    .designations
                    .declare_role(&relation, &id.value.0)?;
                let expected = projection.designations.global(&domain.value.0)?;
                if !projection.grounded.contains(&expected) {
                    return Err(kernel::KernelError::new(format!(
                        "ungrounded role domain '{}'",
                        domain.value.0
                    )));
                }
                projection
                    .role_domains
                    .insert((relation.clone(), role.clone()), expected);
                ordered_roles.push(role);
            }
        }
    }
    let focused_role = projection
        .designations
        .role(&relation, sentence.focus.value.as_str())?;
    if ordered_roles.len() == 2
        && let Some(literal) = literal
    {
        let value_role = ordered_roles
            .iter()
            .find(|role| *role != &focused_role)
            .cloned()
            .expect("binary sentence shape has one role besides its checked focus");
        projection.focus_shapes.push(super::lowering::FocusShape {
            relation,
            literal,
            focused_role,
            value_role,
        });
    }
    Ok(())
}

fn declare_model_projection(
    declaration: &Declaration,
    projection: &mut Projection,
) -> kernel::Result<()> {
    let model = projection
        .designations
        .global(declaration.subject.value.as_str())?;
    declare_model_members(&model, &declaration.body, projection)
}

fn declare_model_members(
    model: &ReferentId,
    members: &[Member],
    projection: &mut Projection,
) -> kernel::Result<()> {
    for member in members {
        match member {
            Member::MembershipRange(range) => {
                projection.designations.global(&range.group.value.0)?;
                for number in range.range.start..=range.range.end {
                    let local = format!("{}{}{}", range.prefix.value, number, range.suffix.value);
                    let id = projection.designations.declare_scoped(model, &local)?;
                    projection
                        .model_referents
                        .entry(model.clone())
                        .or_default()
                        .insert(id);
                }
            }
            Member::Membership(membership) => {
                let member = projection
                    .designations
                    .declare_scoped(model, membership.member.value.as_str())?;
                projection
                    .designations
                    .global(membership.group.value.as_str())?;
                projection
                    .model_referents
                    .entry(model.clone())
                    .or_default()
                    .insert(member);
            }
            Member::Definition(definition) => {
                for name in [&definition.name, &definition.denotation] {
                    let referent = projection
                        .designations
                        .declare_scoped(model, name.value.as_str())?;
                    projection
                        .model_referents
                        .entry(model.clone())
                        .or_default()
                        .insert(referent);
                }
            }
            Member::PureDefinition(definition) => {
                let referent = projection
                    .designations
                    .declare_scoped(model, definition.name.value.as_str())?;
                projection
                    .model_referents
                    .entry(model.clone())
                    .or_default()
                    .insert(referent);
            }
            Member::ShapeBinding(binding) => {
                let referent = projection
                    .designations
                    .declare_scoped(model, binding.label.value.as_str())?;
                let domain = projection
                    .designations
                    .global(binding.domain.value.as_str())?;
                if projection
                    .structural_fields
                    .entry(model.clone())
                    .or_default()
                    .insert(referent.clone(), domain)
                    .is_some()
                {
                    return Err(kernel::KernelError::new(
                        "duplicate structural shape binding",
                    ));
                }
                projection
                    .model_referents
                    .entry(model.clone())
                    .or_default()
                    .insert(referent);
            }
            _ => {}
        }
    }
    Ok(())
}

fn lower_relation_shapes(
    declarations: &[Declaration],
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
                    .role_domains
                    .get(&(relation.clone(), role.clone()))
                    .expect("relation role domain was recorded during declaration")
                    .clone();
                Ok((
                    role.clone(),
                    Role::new(role, vec![membership_predicate(expected)?])?,
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
    for intrinsic in Intrinsic::ALL {
        let relation = intrinsic.relation();
        let result = intrinsic.role(IntrinsicRole::Result);
        let mut roles = intrinsic
            .input_roles()
            .iter()
            .map(|role| {
                let id = intrinsic.role(*role);
                Ok((id.clone(), Role::new(id, Vec::new())?))
            })
            .collect::<kernel::Result<BTreeMap<_, _>>>()?;
        roles.insert(result.clone(), Role::new(result.clone(), Vec::new())?);
        let shape = RelationShape::new(
            relation.clone(),
            roles,
            vec![LookupMode::finite(
                intrinsic
                    .input_roles()
                    .iter()
                    .map(|role| intrinsic.role(*role))
                    .collect(),
                vec![result],
                kernel::Cardinality::One,
            )?],
        )?;
        shapes.insert(relation, shape);
    }
    let membership = membership_shape()?;
    if shapes
        .insert(membership.referent().clone(), membership)
        .is_some()
    {
        return Err(kernel::KernelError::new(
            "canonical membership relation collides with an authored RelationShape",
        ));
    }
    Ok(shapes)
}

fn membership_predicate(group: ReferentId) -> kernel::Result<RolePredicate> {
    RolePredicate::new(
        membership_relation(),
        membership_member_role(),
        BTreeMap::from([(membership_group_role(), group)]),
    )
}

fn insert_intrinsic_identities(referents: &mut BTreeMap<ReferentId, Referent>) {
    let id = Intrinsic::Length.callable_identity();
    referents.insert(id.clone(), Referent::new(id));
}

fn require_registered_dependencies(
    contents: &BTreeMap<kernel::ContentId, RelationalContent>,
    content: &RelationalContent,
) -> kernel::Result<()> {
    for term in content.roles().values() {
        let mut missing = None;
        term.walk(&mut |term| {
            if let Term::Application(dependency) = term
                && !contents.contains_key(dependency)
            {
                missing = Some(dependency.clone());
            }
        });
        if missing.is_some() {
            return Err(kernel::KernelError::new(
                "recursive content must be registered in dependency postorder",
            ));
        }
    }
    Ok(())
}

fn register_unasserted_content(
    contents: &mut BTreeMap<kernel::ContentId, RelationalContent>,
    content: RelationalContent,
) -> kernel::Result<()> {
    require_registered_dependencies(contents, &content)?;
    if let Some(existing) = contents.insert(content.id().clone(), content.clone())
        && existing != content
    {
        return Err(kernel::KernelError::new(
            "recursive content identity collision",
        ));
    }
    Ok(())
}

fn register_content_graph(
    contents: &mut BTreeMap<kernel::ContentId, RelationalContent>,
    graph: LoweredContentGraph,
) -> kernel::Result<RelationalContent> {
    for dependency in graph.dependencies {
        register_unasserted_content(contents, dependency)?;
    }
    require_registered_dependencies(contents, &graph.root)?;
    Ok(graph.root)
}

fn register_definition_graph(
    contents: &mut BTreeMap<kernel::ContentId, RelationalContent>,
    graph: LoweredDefinitionGraph,
) -> kernel::Result<kernel::Definition> {
    for dependency in graph.dependencies {
        register_unasserted_content(contents, dependency)?;
    }
    Ok(graph.definition)
}

fn lower_models(
    declarations: &[Declaration],
    shapes: &BTreeMap<ReferentId, RelationShape>,
    projection: &Projection,
    _proposal_spans: &mut BTreeMap<ProposalPath, frontend::Span>,
) -> kernel::Result<(
    BTreeMap<frontend::Name, Model>,
    BTreeMap<ReferentId, frontend::Span>,
)> {
    let mut models = BTreeMap::new();
    let mut source_spans = BTreeMap::new();
    for declaration in declarations.iter().filter(|item| {
        matches!(
            item.kind,
            Kind::Enumeration | Kind::BindingShape | Kind::Model
        )
    }) {
        let model_id = projection
            .designations
            .global(declaration.subject.value.as_str())?;
        let mut referents = projection
            .grounded
            .iter()
            .chain(shapes.keys())
            .cloned()
            .map(|id| (id.clone(), Referent::new(id)))
            .collect::<BTreeMap<_, _>>();
        insert_intrinsic_identities(&mut referents);
        referents.insert(model_id.clone(), Referent::new(model_id.clone()));
        for id in projection
            .model_referents
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
            literal_memberships(&model_id, &declaration.body, projection, &mut referents)?;
        let mut occurrence_index = 0usize;
        for member in &declaration.body {
            match member {
                Member::Membership(membership) => {
                    let member_id = projection
                        .designations
                        .scoped(&model_id, membership.member.value.as_str())?;
                    let group_id = projection
                        .designations
                        .global(membership.group.value.as_str())?;
                    let content = membership_content(member_id, group_id)?;
                    admit_authored_content(
                        &model_id,
                        content,
                        occurrence_index,
                        Some(membership.span),
                        &mut referents,
                        &mut contents,
                        &mut occurrences,
                        &mut judgments,
                        &mut source_spans,
                    )?;
                    occurrence_index += 1;
                }
                Member::MembershipRange(range) => {
                    let group = projection.designations.global(range.group.value.as_str())?;
                    for number in range.range.start..=range.range.end {
                        let local =
                            format!("{}{}{}", range.prefix.value, number, range.suffix.value);
                        let member = projection.designations.scoped(&model_id, &local)?;
                        admit_authored_content(
                            &model_id,
                            membership_content(member, group.clone())?,
                            occurrence_index,
                            Some(range.span),
                            &mut referents,
                            &mut contents,
                            &mut occurrences,
                            &mut judgments,
                            &mut source_spans,
                        )?;
                        occurrence_index += 1;
                    }
                }
                Member::RelationalContent(_) => occurrence_index += 1,
                Member::Focus(focus) => {
                    occurrence_index += (focus.binding.range.end - focus.binding.range.start + 1)
                        as usize
                        * focus.slots.len();
                }
                _ => {}
            }
        }
        let shell = Model::with_distinctions(
            model_id.clone(),
            referents.clone(),
            contents.clone(),
            shapes.clone(),
            BTreeMap::new(),
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
        let mut definitions = Vec::new();
        for member in &declaration.body {
            let (lowered, span) = match member {
                Member::RelationalContent(surface) => {
                    let graph = lower_clause_graph_with(projection, shell.model(), surface, None)?;
                    let root = register_content_graph(&mut contents, graph)?;
                    (vec![root], Some(surface.span))
                }
                Member::Focus(focus) => (
                    lower_focus(projection, shell.model(), focus)?,
                    Some(focus.span),
                ),
                Member::Membership(_) => {
                    occurrence_index += 1;
                    (Vec::new(), None)
                }
                Member::MembershipRange(range) => {
                    occurrence_index += (range.range.end - range.range.start + 1) as usize;
                    (Vec::new(), None)
                }
                Member::Definition(definition) => {
                    definitions.push(lower_definition(
                        projection,
                        shell.model(),
                        &definition.name.value,
                        &definition.denotation.value,
                    )?);
                    (Vec::new(), None)
                }
                Member::ShapeBinding(binding) => {
                    definitions.push(lower_shape_binding(
                        projection,
                        shell.model(),
                        &binding.label.value,
                        &binding.domain.value,
                    )?);
                    (Vec::new(), None)
                }
                _ => (Vec::new(), None),
            };
            for content in lowered {
                admit_authored_content(
                    &model_id,
                    content,
                    occurrence_index,
                    span,
                    &mut referents,
                    &mut contents,
                    &mut occurrences,
                    &mut judgments,
                    &mut source_spans,
                )?;
                occurrence_index += 1;
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
                .map(|surface| {
                    let graph =
                        lower_clause_graph_with(projection, shell.model(), surface, Some(binders))?;
                    register_content_graph(&mut contents, graph)
                })
                .collect::<kernel::Result<Vec<_>>>()?;
            let conclusion_graph =
                lower_clause_graph_with(projection, shell.model(), conclusion, Some(binders))?;
            let conclusion_content = register_content_graph(&mut contents, conclusion_graph)?;
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
        let structural_contracts =
            extend_structural_closure(projection, &mut referents, &contents, &mut definitions)?;
        let model = Model::with_distinctions(
            model_id,
            referents,
            contents,
            shapes.clone(),
            structural_contracts,
            occurrences,
            definitions,
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

fn lower_context_model(
    model_id: ReferentId,
    members: &[Member],
    shapes: &BTreeMap<ReferentId, RelationShape>,
    projection: &Projection,
    proposal_spans: &mut BTreeMap<ProposalPath, frontend::Span>,
) -> kernel::Result<(Model, BTreeMap<ReferentId, frontend::Span>)> {
    let mut source_spans = BTreeMap::new();
    let mut referents = projection
        .grounded
        .iter()
        .chain(shapes.keys())
        .cloned()
        .map(|id| (id.clone(), Referent::new(id)))
        .collect::<BTreeMap<_, _>>();
    insert_intrinsic_identities(&mut referents);
    referents.insert(model_id.clone(), Referent::new(model_id.clone()));
    for id in projection
        .model_referents
        .get(&model_id)
        .into_iter()
        .flatten()
    {
        referents.insert(id.clone(), Referent::new(id.clone()));
    }
    for member in members {
        collect_member_literal_referents(member, projection, &mut referents)?;
    }
    let (mut contents, mut occurrences, mut judgments) =
        literal_memberships(&model_id, members, projection, &mut referents)?;
    let mut occurrence_index = 0usize;
    for member in members {
        match member {
            Member::Membership(membership) => {
                let member_id = projection
                    .designations
                    .scoped(&model_id, membership.member.value.as_str())?;
                let group_id = projection
                    .designations
                    .global(membership.group.value.as_str())?;
                admit_authored_content(
                    &model_id,
                    membership_content(member_id, group_id)?,
                    occurrence_index,
                    Some(membership.span),
                    &mut referents,
                    &mut contents,
                    &mut occurrences,
                    &mut judgments,
                    &mut source_spans,
                )?;
                occurrence_index += 1;
            }
            Member::RelationalContent(_) => occurrence_index += 1,
            Member::Definition(_) | Member::PureDefinition(_) => {}
            _ => {
                return Err(kernel::KernelError::new(
                    "unsupported direct top-level Model member",
                ));
            }
        }
    }
    let shell = wire::admit(Model::with_distinctions(
        model_id.clone(),
        referents.clone(),
        contents.clone(),
        shapes.clone(),
        BTreeMap::new(),
        occurrences.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        judgments.clone(),
    )?);
    let mut occurrence_index = 0usize;
    let mut definitions = Vec::new();
    for member in members {
        match member {
            Member::Membership(_) => occurrence_index += 1,
            Member::RelationalContent(surface) => {
                let graph = lower_clause_graph_with(projection, shell.model(), surface, None)?;
                let content = register_content_graph(&mut contents, graph)?;
                admit_authored_content(
                    &model_id,
                    content,
                    occurrence_index,
                    Some(surface.span),
                    &mut referents,
                    &mut contents,
                    &mut occurrences,
                    &mut judgments,
                    &mut source_spans,
                )?;
                occurrence_index += 1;
            }
            Member::Definition(definition) => definitions.push(lower_definition(
                projection,
                shell.model(),
                &definition.name.value,
                &definition.denotation.value,
            )?),
            Member::PureDefinition(definition) => definitions.push(register_definition_graph(
                &mut contents,
                lower_pure_definition(projection, shell.model(), definition, proposal_spans)?,
            )?),
            _ => unreachable!("direct top-level members were checked in the shell pass"),
        }
    }
    let structural_contracts =
        extend_structural_closure(projection, &mut referents, &contents, &mut definitions)?;
    Ok((
        Model::with_distinctions(
            model_id,
            referents,
            contents,
            shapes.clone(),
            structural_contracts,
            occurrences,
            definitions,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments,
        )?,
        source_spans,
    ))
}

fn extend_structural_closure(
    projection: &Projection,
    referents: &mut BTreeMap<ReferentId, Referent>,
    contents: &BTreeMap<kernel::ContentId, RelationalContent>,
    definitions: &mut Vec<Definition>,
) -> kernel::Result<BTreeMap<ReferentId, StructuralContract>> {
    let terms = contents
        .values()
        .flat_map(|content| content.roles().values())
        .chain(definitions.iter().map(Definition::denotation))
        .cloned()
        .collect::<Vec<_>>();
    let mut contracts = BTreeMap::new();
    for (shape, fields) in &projection.structural_fields {
        referents
            .entry(shape.clone())
            .or_insert_with(|| Referent::new(shape.clone()));
        for (field, domain) in fields {
            referents
                .entry(field.clone())
                .or_insert_with(|| Referent::new(field.clone()));
            referents
                .entry(domain.clone())
                .or_insert_with(|| Referent::new(domain.clone()));
            extend_declared_scalar_contract(projection, domain, &mut contracts)?;
            if !definitions
                .iter()
                .any(|definition| definition.id() == field)
            {
                definitions.push(Definition::new(
                    field.clone(),
                    Term::referent(domain.clone()),
                ));
            }
        }
        contracts.insert(
            shape.clone(),
            StructuralContract::new(
                shape.clone(),
                StructuralForm::Product(fields.keys().cloned().collect()),
            )?,
        );
    }
    for term in &terms {
        extend_term_structural_closure(projection, term, referents, &mut contracts)?;
    }
    Ok(contracts)
}

fn extend_declared_scalar_contract(
    projection: &Projection,
    domain: &ReferentId,
    contracts: &mut BTreeMap<ReferentId, StructuralContract>,
) -> kernel::Result<()> {
    for (name, form) in [
        ("F32", StructuralForm::F32),
        ("Int", StructuralForm::Int),
        ("Bool", StructuralForm::Bool),
    ] {
        let candidate = structural_domain(projection, &frontend::DomainName(name.to_owned()));
        if &candidate == domain {
            contracts
                .entry(candidate.clone())
                .or_insert(StructuralContract::new(candidate, form)?);
            break;
        }
    }
    Ok(())
}

fn extend_term_structural_closure(
    projection: &Projection,
    term: &Term,
    referents: &mut BTreeMap<ReferentId, Referent>,
    contracts: &mut BTreeMap<ReferentId, StructuralContract>,
) -> kernel::Result<()> {
    let scalar = match term {
        Term::F32(_) => Some(("F32", StructuralForm::F32)),
        Term::Int(_) => Some(("Int", StructuralForm::Int)),
        Term::Bool(_) => Some(("Bool", StructuralForm::Bool)),
        _ => None,
    };
    if let Some((name, form)) = scalar {
        let domain = structural_domain(projection, &frontend::DomainName(name.to_owned()));
        referents
            .entry(domain.clone())
            .or_insert_with(|| Referent::new(domain.clone()));
        contracts.insert(domain.clone(), StructuralContract::new(domain, form)?);
    }
    match term {
        Term::Product { shape, fields } => {
            referents
                .entry(shape.clone())
                .or_insert_with(|| Referent::new(shape.clone()));
            for field in fields.values() {
                referents
                    .entry(field.domain().clone())
                    .or_insert_with(|| Referent::new(field.domain().clone()));
                extend_term_structural_closure(projection, field.value(), referents, contracts)?;
            }
        }
        Term::LabelledProduct { shape, fields } => {
            referents
                .entry(shape.clone())
                .or_insert_with(|| Referent::new(shape.clone()));
            for (field, value) in fields {
                referents
                    .entry(field.clone())
                    .or_insert_with(|| Referent::new(field.clone()));
                extend_term_structural_closure(projection, value, referents, contracts)?;
            }
        }
        Term::Sequence {
            shape,
            element,
            values,
        } => {
            for id in [shape, element] {
                referents
                    .entry(id.clone())
                    .or_insert_with(|| Referent::new(id.clone()));
            }
            for value in values {
                extend_term_structural_closure(projection, value, referents, contracts)?;
            }
        }
        Term::Sum { value, .. } => {
            extend_term_structural_closure(projection, value, referents, contracts)?;
        }
        Term::Referent(_)
        | Term::Pattern(_)
        | Term::Application(_)
        | Term::F32(_)
        | Term::Int(_)
        | Term::Bool(_) => {}
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn admit_authored_content(
    model: &ReferentId,
    content: RelationalContent,
    ordinal: usize,
    span: Option<frontend::Span>,
    referents: &mut BTreeMap<ReferentId, Referent>,
    contents: &mut BTreeMap<kernel::ContentId, RelationalContent>,
    occurrences: &mut Vec<AssertionOccurrence>,
    judgments: &mut Vec<Judgment>,
    source_spans: &mut BTreeMap<ReferentId, frontend::Span>,
) -> kernel::Result<()> {
    require_registered_dependencies(contents, &content)?;
    contents.insert(content.id().clone(), content.clone());
    let occurrence = synthetic_referent(
        "assertion-occurrence",
        &[model.as_str(), &ordinal.to_string()],
    );
    let judgment = synthetic_referent("assertion-judgment", &[occurrence.as_str()]);
    referents.insert(occurrence.clone(), Referent::new(occurrence.clone()));
    referents.insert(judgment.clone(), Referent::new(judgment.clone()));
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
        JudgmentTarget::Occurrence(occurrence.clone()),
        JudgmentKind::Admitted {
            policy: model.clone(),
            basis: Vec::new(),
        },
        JudgmentStatus::Affirmed,
    ));
    if let Some(span) = span
        && source_spans.insert(occurrence, span).is_some()
    {
        return Err(kernel::KernelError::new(
            "duplicate assertion occurrence source projection",
        ));
    }
    Ok(())
}

type LiteralMemberships = (
    BTreeMap<kernel::ContentId, RelationalContent>,
    Vec<AssertionOccurrence>,
    Vec<Judgment>,
);

fn literal_memberships(
    model: &ReferentId,
    members: &[Member],
    projection: &Projection,
    referents: &mut BTreeMap<ReferentId, Referent>,
) -> kernel::Result<LiteralMemberships> {
    let mut contents = BTreeMap::new();
    let mut occurrences = Vec::new();
    let mut judgments = Vec::new();
    let literals = members
        .iter()
        .map(|member| member_literal_referents(member, projection))
        .collect::<kernel::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>();
    if !literals.is_empty() {
        let text = projection.designations.global("Text")?;
        if !projection.grounded.contains(&text) {
            return Err(kernel::KernelError::new(
                "model string literal requires grounded Text",
            ));
        }
        for literal in literals {
            let content = membership_content(literal.clone(), text.clone())?;
            let occurrence = synthetic_referent(
                "literal-membership-occurrence",
                &[model.as_str(), literal.as_str()],
            );
            let judgment =
                synthetic_referent("literal-membership-judgment", &[occurrence.as_str()]);
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
    }
    Ok((contents, occurrences, judgments))
}

fn member_literal_referents(
    member: &Member,
    projection: &Projection,
) -> kernel::Result<Vec<ReferentId>> {
    fn collect(
        term: &SurfaceTerm,
        projection: &Projection,
        literals: &mut Vec<ReferentId>,
    ) -> kernel::Result<()> {
        match term {
            SurfaceTerm::String(value) => {
                literals.push(projection.designations.literal(&value.value)?);
            }
            SurfaceTerm::Application(value) => {
                for term in value.roles.values() {
                    collect(term, projection, literals)?;
                }
            }
            SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
                for term in values {
                    collect(term, projection, literals)?;
                }
            }
            SurfaceTerm::Product { fields, .. } => {
                for term in fields.values() {
                    collect(term, projection, literals)?;
                }
            }
            SurfaceTerm::Referent(_)
            | SurfaceTerm::Local(_)
            | SurfaceTerm::Template(_)
            | SurfaceTerm::Variable(_)
            | SurfaceTerm::AnonymousHole(_)
            | SurfaceTerm::F32(_)
            | SurfaceTerm::Int(_)
            | SurfaceTerm::Bool(_)
            | SurfaceTerm::Intrinsic(_) => {}
        }
        Ok(())
    }
    let terms = match member {
        Member::RelationalContent(clause) => clause.roles.values().collect::<Vec<_>>(),
        Member::Focus(focus) => focus.slots.iter().map(|slot| &slot.value).collect(),
        _ => Vec::new(),
    };
    let mut literals = Vec::new();
    for term in terms {
        collect(term, projection, &mut literals)?;
    }
    Ok(literals)
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
        frontend::RequestDecl::Any { pattern, .. }
        | frontend::RequestDecl::Select { pattern, .. }
        | frontend::RequestDecl::Find { pattern, .. } => Some(pattern),
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
    match term {
        SurfaceTerm::String(value) => {
            table.declare_literal(&value.value);
        }
        SurfaceTerm::Application(value) => {
            for term in value.roles.values() {
                declare_term_literal(term, table);
            }
        }
        SurfaceTerm::Tuple { values, .. } | SurfaceTerm::Sequence { values, .. } => {
            for term in values {
                declare_term_literal(term, table);
            }
        }
        SurfaceTerm::Product { fields, .. } => {
            for term in fields.values() {
                declare_term_literal(term, table);
            }
        }
        SurfaceTerm::Referent(_)
        | SurfaceTerm::Local(_)
        | SurfaceTerm::Template(_)
        | SurfaceTerm::Variable(_)
        | SurfaceTerm::AnonymousHole(_)
        | SurfaceTerm::F32(_)
        | SurfaceTerm::Int(_)
        | SurfaceTerm::Bool(_)
        | SurfaceTerm::Intrinsic(_) => {}
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
        kernel::Term,
    };

    const BASE: &str = "Module\n\nimpact/imports: RelationShape\n  {consumer: Module} imports {dependency: Module}\n  mode consumer -> dependency: many\n\nimpact\n  North ∈ Module\n  South ∈ Module\n  Store ∈ Module\n  North imports Store\n";

    #[test]
    fn seals_base_and_preserves_request_order() {
        let program = compile(frontend::parse(&format!("{BASE}\nwhy in impact:\n  North imports Store\n\nfind all ?dependency in impact:\n  North imports ?dependency\n")).unwrap()).unwrap();
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
    fn role_domains_are_enforced_by_sealed_canonical_membership() {
        let source = "Alpha
Beta
Text

typed/pairs: RelationShape
  {left: Alpha} pairs {right: Beta}
  mode left -> right: many

typed/labels: RelationShape
  {owner: Alpha} labels {label: Text}
  mode owner -> label: many

typed
  Alpha-0 ∈ Alpha
  Alpha-1 ∈ Alpha
  Beta-0 ∈ Beta
  Beta-1 ∈ Beta
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
            panic!("sealed membership domains must yield a complete addition frontier");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].admissions(), &[target]);
    }

    #[test]
    fn inferred_enumeration_and_binding_shape_lower_without_type_ontology() {
        let program = compile(
            frontend::parse(
                "F32\n\nGame\n  Chess\n  Soccer\n\nVec2\n  x: F32\n  y: F32\n\nPose\n  position: Vec2\n",
            )
            .unwrap(),
        )
        .unwrap();
        let context_program = compile_in(
            frontend::parse("F32\n\nVec2\n  x: F32\n  y: F32\n\nPose\n  position: Vec2\n").unwrap(),
            ModelContext::new(ReferentId::from_digest([99; 32])),
        )
        .unwrap();
        let game = program.designations().global("Game").unwrap();
        let chess = program.designations().scoped(&game, "Chess").unwrap();
        let game_revision = program.revision(&frontend::Name("Game".into())).unwrap();
        assert!(
            game_revision
                .model()
                .admitted_contents()
                .binary_search(&membership_content(chess, game).unwrap())
                .is_ok()
        );

        let vec2 = program.designations().global("Vec2").unwrap();
        let x = program.designations().scoped(&vec2, "x").unwrap();
        let f32 = program.designations().global("F32").unwrap();
        let vec2_revision = program.revision(&frontend::Name("Vec2".into())).unwrap();
        assert!(
            vec2_revision
                .model()
                .definitions()
                .iter()
                .any(|definition| {
                    definition.id() == &x && definition.denotation() == &Term::referent(f32.clone())
                })
        );

        let pose = program.designations().global("Pose").unwrap();
        let position = program.designations().scoped(&pose, "position").unwrap();
        let pose_revision = program.revision(&frontend::Name("Pose".into())).unwrap();
        assert_eq!(
            pose_revision
                .model()
                .definition(&position)
                .unwrap()
                .denotation(),
            &Term::referent(vec2.clone())
        );

        let expected_vec2 = StructuralForm::Product(BTreeSet::from([
            x,
            program.designations().scoped(&vec2, "y").unwrap(),
        ]));
        let expected_pose = StructuralForm::Product(BTreeSet::from([position]));
        for model in [
            vec2_revision.model(),
            pose_revision.model(),
            context_program.context_model().unwrap(),
        ] {
            assert_eq!(
                model.structural_contracts().get(&f32).unwrap().form(),
                &StructuralForm::F32
            );
            assert_eq!(
                model.structural_contracts().get(&vec2).unwrap().form(),
                &expected_vec2
            );
            assert_eq!(
                model.structural_contracts().get(&pose).unwrap().form(),
                &expected_pose
            );
            assert_eq!(
                wire::reload(&wire::serialize(&wire::admit(model.clone())))
                    .unwrap()
                    .model(),
                model
            );
        }
    }
}
