//! Lower the native authoring surface into sealed semantic revisions.
//!
//! Names below are navigation bindings only: the Model is sealed through
//! `wire::admit` before a name is placed in the revision registry.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, AscriptionDecl, Kind, Member, ShapePartDecl, SurfaceClause, SurfaceTerm},
    kernel::{
        self, Clause, Delta, EntityId, InlineSentencePart, Law, Model, ModelId, Name, Relation,
        RelationId, Revision, Role, RoleId, SentencePart, SentenceShape, Term, Type, TypeId,
        VariableId,
    },
    wire,
};

/// Sealed revisions indexed by authored navigation names, plus requests in
/// authored order. Requests are deliberately outside revision identity.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    revisions: BTreeMap<frontend::Name, Revision>,
    requests: Vec<frontend::RequestDecl>,
}

impl CompiledProgram {
    pub fn revisions(&self) -> &BTreeMap<frontend::Name, Revision> {
        &self.revisions
    }
    pub fn requests(&self) -> &[frontend::RequestDecl] {
        &self.requests
    }
    pub fn revision(&self, name: &frontend::Name) -> kernel::Result<&Revision> {
        self.revisions.get(name).ok_or_else(|| {
            kernel::KernelError::new(format!("unknown Revision '{}'", name.as_str()))
        })
    }
}

/// Compile declarations and seal each authored Model/Revision. G5 consumes
/// the returned request declarations and `lower_clause` to execute requests.
pub fn compile(program: frontend::Program) -> kernel::Result<CompiledProgram> {
    let declarations = declaration_map(&program.declarations)?;
    let types = lower_types(&program.declarations)?;
    let relations = lower_relations(&program.declarations, &types)?;
    let models = attach_laws(
        &program.declarations,
        lower_models(&program.declarations, &types, &relations)?,
    )?;
    let mut resolver = Resolver::new(&declarations, models);
    for declaration in &program.declarations {
        match declaration.kind {
            Kind::Model | Kind::Revision => {
                resolver.revision(&declaration.subject.value)?;
            }
            Kind::Delta => {
                resolver.delta(&declaration.subject.value)?;
            }
            Kind::Type | Kind::Relation | Kind::Law => {}
        }
    }
    Ok(CompiledProgram {
        revisions: resolver.revisions,
        requests: program.requests,
    })
}

/// Lower one parsed clause against the selected Revision's typed Model.
pub fn lower_clause(revision: &Revision, surface: &SurfaceClause) -> kernel::Result<Clause> {
    let model = revision.model();
    let relation_id = relation_id(&surface.relation.value.0)?;
    let relation = model.relations().get(&relation_id).ok_or_else(|| {
        kernel::KernelError::new(format!("undeclared Relation '{}'", relation_id.as_str()))
    })?;
    let mut roles = BTreeMap::new();
    for (surface_role, surface_term) in &surface.roles {
        let role_id = role_id(&surface_role.0)?;
        let role = relation.roles().get(&role_id).ok_or_else(|| {
            kernel::KernelError::new(format!(
                "Relation '{}' has no role '{}'",
                relation_id.as_str(),
                role_id.as_str()
            ))
        })?;
        if roles
            .insert(role_id, lower_term(model, role.typ(), surface_term)?)
            .is_some()
        {
            return Err(kernel::KernelError::new("duplicate clause role"));
        }
    }
    let clause = Clause::new(relation_id, roles)?;
    model.validate_clause(&clause, true)?;
    Ok(clause)
}

fn declaration_map(
    declarations: &[AscriptionDecl],
) -> kernel::Result<BTreeMap<frontend::Name, &AscriptionDecl>> {
    let mut map = BTreeMap::new();
    for declaration in declarations {
        if map
            .insert(declaration.subject.value.clone(), declaration)
            .is_some()
        {
            return Err(kernel::KernelError::new(format!(
                "duplicate declaration '{}'",
                declaration.subject.value.as_str()
            )));
        }
    }
    Ok(map)
}

fn lower_types(declarations: &[AscriptionDecl]) -> kernel::Result<BTreeMap<TypeId, Type>> {
    let mut types = BTreeMap::new();
    for declaration in declarations.iter().filter(|d| d.kind == Kind::Type) {
        if !declaration.body.is_empty() {
            return Err(kernel::KernelError::new(
                "Type declarations cannot have members",
            ));
        }
        let id = type_id(&declaration.subject.value.0)?;
        if types.insert(id.clone(), Type::new(id)).is_some() {
            return Err(kernel::KernelError::new("duplicate Type identity"));
        }
    }
    Ok(types)
}

fn lower_relations(
    declarations: &[AscriptionDecl],
    types: &BTreeMap<TypeId, Type>,
) -> kernel::Result<BTreeMap<RelationId, Relation>> {
    let mut relations = BTreeMap::new();
    for declaration in declarations.iter().filter(|d| d.kind == Kind::Relation) {
        let shape = declaration
            .body
            .iter()
            .find_map(|m| {
                if let Member::Sentence(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .ok_or_else(|| kernel::KernelError::new("Relation requires one sentence shape"))?;
        let parts = shape
            .parts
            .iter()
            .map(|part| match part {
                ShapePartDecl::Literal(text) => Ok(InlineSentencePart::Literal(text.value.clone())),
                ShapePartDecl::Role { id, typ } => {
                    let typ = type_id(&typ.value.0)?;
                    if !types.contains_key(&typ) {
                        return Err(kernel::KernelError::new(format!(
                            "undeclared Type '{}'",
                            typ.as_str()
                        )));
                    }
                    Ok(InlineSentencePart::Role(Role::new(
                        role_id(&id.value.0)?,
                        typ,
                    )))
                }
            })
            .collect::<kernel::Result<Vec<_>>>()?;
        let modes = declaration
            .body
            .iter()
            .filter_map(|m| {
                if let Member::Mode(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .map(|mode| {
                kernel::Mode::finite(
                    mode.known
                        .iter()
                        .map(|v| role_id(&v.value.0))
                        .collect::<kernel::Result<Vec<_>>>()?,
                    mode.sought
                        .iter()
                        .map(|v| role_id(&v.value.0))
                        .collect::<kernel::Result<Vec<_>>>()?,
                    cardinality(mode.cardinality),
                )
            })
            .collect::<kernel::Result<Vec<_>>>()?;
        let id = relation_id(&declaration.subject.value.0)?;
        let relation = Relation::new(id.clone(), SentenceShape::new(parts)?, modes)?;
        if relations.insert(id, relation).is_some() {
            return Err(kernel::KernelError::new("duplicate Relation identity"));
        }
    }
    Ok(relations)
}

fn lower_models(
    declarations: &[AscriptionDecl],
    types: &BTreeMap<TypeId, Type>,
    relations: &BTreeMap<RelationId, Relation>,
) -> kernel::Result<BTreeMap<frontend::Name, Model>> {
    let mut models = BTreeMap::new();
    for declaration in declarations.iter().filter(|d| d.kind == Kind::Model) {
        let id = model_id(&declaration.subject.value.0)?;
        let mut entities = BTreeSet::new();
        for entity in declaration.body.iter().filter_map(|m| {
            if let Member::Entity(v) = m {
                Some(v)
            } else {
                None
            }
        }) {
            let typ = type_id(&entity.typ.value.0)?;
            if !types.contains_key(&typ) {
                return Err(kernel::KernelError::new(format!(
                    "undeclared Type '{}'",
                    typ.as_str()
                )));
            }
            if !entities.insert(EntityId::new(
                id.clone(),
                entity_local(&entity.local.value.0)?,
                typ,
            )?) {
                return Err(kernel::KernelError::new("duplicate entity identity"));
            }
        }
        for group in declaration.body.iter().filter_map(|member| {
            if let Member::EntityGroup(group) = member {
                Some(group)
            } else {
                None
            }
        }) {
            let typ = type_id(&group.typ.value.0)?;
            if !types.contains_key(&typ) {
                return Err(kernel::KernelError::new(format!(
                    "undeclared Type '{}'",
                    typ.as_str()
                )));
            }
            for number in group.range.start..=group.range.end {
                let local = format!("{}{}{}", group.prefix.value, number, group.suffix.value);
                if !entities.insert(EntityId::new(
                    id.clone(),
                    entity_local(&local)?,
                    typ.clone(),
                )?) {
                    return Err(kernel::KernelError::new("duplicate entity identity"));
                }
            }
        }
        let shell = Model::new(
            id.clone(),
            types.clone(),
            entities.clone(),
            relations.clone(),
            vec![],
            vec![],
        )?;
        let shell = wire::admit(shell);
        let mut assertions = Vec::new();
        for member in &declaration.body {
            match member {
                Member::Clause(clause) => assertions.push(lower_clause(&shell, clause)?),
                Member::Focus(focus) => assertions.extend(lower_focus(&shell, focus)?),
                _ => {}
            }
        }
        let model = Model::new(
            id,
            types.clone(),
            entities,
            relations.clone(),
            assertions,
            vec![],
        )?;
        if models
            .insert(declaration.subject.value.clone(), model)
            .is_some()
        {
            return Err(kernel::KernelError::new("duplicate Model identity"));
        }
    }
    Ok(models)
}

fn attach_laws(
    declarations: &[AscriptionDecl],
    mut models: BTreeMap<frontend::Name, Model>,
) -> kernel::Result<BTreeMap<frontend::Name, Model>> {
    let mut grouped = BTreeMap::<frontend::Name, Vec<Law>>::new();
    for declaration in declarations.iter().filter(|d| d.kind == Kind::Law) {
        let model_name = models
            .keys()
            .filter(|model| {
                declaration
                    .subject
                    .value
                    .0
                    .strip_prefix(&format!("{}/", model.as_str()))
                    .is_some()
            })
            .max_by_key(|model| model.0.len())
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("Law name has no Model namespace"))?;
        let shell = wire::admit(models[&model_name].clone());
        let conclusion = declaration
            .body
            .iter()
            .find_map(|m| {
                if let Member::Clause(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .ok_or_else(|| kernel::KernelError::new("Law requires a conclusion"))?;
        let premises = declaration
            .body
            .iter()
            .find_map(|m| {
                if let Member::When(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .ok_or_else(|| kernel::KernelError::new("Law requires when premises"))?;
        grouped.entry(model_name).or_default().push(Law::new(
            law_id(&declaration.subject.value.0)?,
            premises
                .iter()
                .map(|v| lower_clause(&shell, v))
                .collect::<kernel::Result<Vec<_>>>()?,
            lower_clause(&shell, conclusion)?,
        )?);
    }
    for (name, laws) in grouped {
        let model = models.remove(&name).expect("selected Model exists");
        models.insert(
            name,
            Model::new(
                model.id().clone(),
                model.types().clone(),
                model.entities().clone(),
                model.relations().clone(),
                model.assertions().to_vec(),
                laws,
            )?,
        );
    }
    Ok(models)
}

struct Resolver<'a> {
    declarations: &'a BTreeMap<frontend::Name, &'a AscriptionDecl>,
    models: BTreeMap<frontend::Name, Model>,
    revisions: BTreeMap<frontend::Name, Revision>,
    deltas: BTreeMap<frontend::Name, Delta>,
    visiting_revisions: BTreeSet<frontend::Name>,
    visiting_deltas: BTreeSet<frontend::Name>,
}

impl<'a> Resolver<'a> {
    fn new(
        declarations: &'a BTreeMap<frontend::Name, &'a AscriptionDecl>,
        models: BTreeMap<frontend::Name, Model>,
    ) -> Self {
        Self {
            declarations,
            models,
            revisions: BTreeMap::new(),
            deltas: BTreeMap::new(),
            visiting_revisions: BTreeSet::new(),
            visiting_deltas: BTreeSet::new(),
        }
    }
    fn declaration(&self, name: &frontend::Name, kind: Kind) -> kernel::Result<&'a AscriptionDecl> {
        match self.declarations.get(name) {
            Some(d) if d.kind == kind => Ok(*d),
            Some(_) => Err(kernel::KernelError::new(format!(
                "'{}' has the wrong declaration kind",
                name.as_str()
            ))),
            None => Err(kernel::KernelError::new(format!(
                "unknown declaration '{}'",
                name.as_str()
            ))),
        }
    }
    fn revision(&mut self, name: &frontend::Name) -> kernel::Result<Revision> {
        if let Some(revision) = self.revisions.get(name) {
            return Ok(revision.clone());
        }
        if let Some(model) = self.models.get(name) {
            let revision = wire::admit(model.clone());
            self.revisions.insert(name.clone(), revision.clone());
            return Ok(revision);
        }
        let declaration = self.declaration(name, Kind::Revision)?;
        if !self.visiting_revisions.insert(name.clone()) {
            return Err(kernel::KernelError::new(format!(
                "Revision/Delta dependency cycle at '{}'",
                name.as_str()
            )));
        }
        let outcome = (|| {
            let base = self.revision(&from(declaration)?)?;
            match apply(declaration)? {
                Some(delta_name) => {
                    let delta = self.delta(&delta_name)?;
                    if delta.base() != base.identity() {
                        return Err(kernel::KernelError::new(format!(
                            "Delta '{}' base does not match Revision '{}'",
                            delta_name.as_str(),
                            name.as_str()
                        )));
                    }
                    delta.apply(&base)
                }
                None => local_delta(&base, declaration)?.apply(&base),
            }
        })();
        self.visiting_revisions.remove(name);
        let revision = outcome?;
        self.revisions.insert(name.clone(), revision.clone());
        Ok(revision)
    }
    fn delta(&mut self, name: &frontend::Name) -> kernel::Result<Delta> {
        if let Some(delta) = self.deltas.get(name) {
            return Ok(delta.clone());
        }
        let declaration = self.declaration(name, Kind::Delta)?;
        if !self.visiting_deltas.insert(name.clone()) {
            return Err(kernel::KernelError::new(format!(
                "Revision/Delta dependency cycle at '{}'",
                name.as_str()
            )));
        }
        let outcome = (|| {
            let base = self.revision(&from(declaration)?)?;
            let delta = local_delta(&base, declaration)?;
            delta.clone().apply(&base)?;
            Ok(delta)
        })();
        self.visiting_deltas.remove(name);
        let delta = outcome?;
        self.deltas.insert(name.clone(), delta.clone());
        Ok(delta)
    }
}

fn from(declaration: &AscriptionDecl) -> kernel::Result<frontend::Name> {
    declaration
        .body
        .iter()
        .find_map(|m| {
            if let Member::From(v) = m {
                Some(v.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| kernel::KernelError::new("Revision or Delta requires from:"))
}
fn apply(declaration: &AscriptionDecl) -> kernel::Result<Option<frontend::Name>> {
    Ok(declaration.body.iter().find_map(|m| {
        if let Member::Apply(v) = m {
            Some(v.clone())
        } else {
            None
        }
    }))
}
fn local_delta(base: &Revision, declaration: &AscriptionDecl) -> kernel::Result<Delta> {
    let admissions = declaration
        .body
        .iter()
        .filter_map(|m| {
            if let Member::Admit(v) = m {
                Some(v)
            } else {
                None
            }
        })
        .flatten()
        .map(|v| lower_clause(base, v))
        .collect::<kernel::Result<Vec<_>>>()?;
    let withdrawals = declaration
        .body
        .iter()
        .filter_map(|m| {
            if let Member::Withdraw(v) = m {
                Some(v)
            } else {
                None
            }
        })
        .flatten()
        .map(|v| lower_clause(base, v))
        .collect::<kernel::Result<Vec<_>>>()?;
    Delta::new(base.identity().clone(), admissions, withdrawals)
}

fn lower_term(model: &Model, expected: &TypeId, term: &SurfaceTerm) -> kernel::Result<Term> {
    match term {
        SurfaceTerm::Variable(value) => Ok(Term::variable(
            variable_id(&value.value.0)?,
            expected.clone(),
        )),
        SurfaceTerm::String(value) => {
            let text = type_id("Text")?;
            if expected != &text || !model.types().contains_key(&text) {
                return Err(kernel::KernelError::new(
                    "scalar strings require an admitted Text role",
                ));
            }
            Term::value(text, value.value.clone())
        }
        SurfaceTerm::Entity(value) => {
            let entity = resolve_entity(model, &value.value)?;
            if entity.typ() != expected {
                return Err(kernel::KernelError::new(format!(
                    "entity '{}' has Type '{}', not '{}'",
                    entity.local().as_str(),
                    entity.typ().as_str(),
                    expected.as_str()
                )));
            }
            Ok(Term::entity(entity))
        }
        SurfaceTerm::Template(_) => Err(kernel::KernelError::new(
            "correlated entity templates are only valid inside a focus block",
        )),
    }
}

fn lower_focus(model: &Revision, focus: &frontend::FocusBlock) -> kernel::Result<Vec<Clause>> {
    let mut clauses = Vec::new();
    for number in focus.binding.range.start..=focus.binding.range.end {
        let focused = focus_entity(model.model(), &focus.template, number)?;
        for slot in &focus.slots {
            let mut candidates = Vec::new();
            for relation in model.model().relations().values() {
                let [
                    SentencePart::Role(focused_role),
                    SentencePart::Literal(literal),
                    SentencePart::Role(value_role),
                ] = relation.shape().parts()
                else {
                    continue;
                };
                if literal != &slot.label.value {
                    continue;
                }
                candidates.push((relation, focused_role, value_role));
            }
            if candidates.is_empty() {
                return Err(kernel::KernelError::new(format!(
                    "no declared sentence shape accepts focused slot '{}'",
                    slot.label.value
                )));
            }
            if candidates.len() > 1 {
                let names = candidates
                    .iter()
                    .map(|(relation, _, _)| relation.id().as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(kernel::KernelError::new(format!(
                    "ambiguous focused slot '{}'; candidates: {names}",
                    slot.label.value
                )));
            }
            let (relation, focused_role, value_role) =
                candidates.pop().expect("nonempty focus candidates");
            let focused_role = relation
                .roles()
                .get(focused_role)
                .expect("sentence shape role belongs to relation");
            if focused.typ() != focused_role.typ() {
                return Err(kernel::KernelError::new(format!(
                    "entity '{}' has Type '{}', not '{}'",
                    focused.local().as_str(),
                    focused.typ().as_str(),
                    focused_role.typ().as_str()
                )));
            }
            let value_role = relation
                .roles()
                .get(value_role)
                .expect("sentence shape role belongs to relation");
            let value = lower_focus_term(
                model.model(),
                value_role.typ(),
                &slot.value,
                &focus.binding.variable.value,
                number,
            )?;
            let clause = Clause::new(
                relation.id().clone(),
                BTreeMap::from([
                    (focused_role.id().clone(), Term::entity(focused.clone())),
                    (value_role.id().clone(), value),
                ]),
            )?;
            model.model().validate_clause(&clause, true)?;
            clauses.push(clause);
        }
    }
    Ok(clauses)
}

fn lower_focus_term(
    model: &Model,
    expected: &TypeId,
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
            let entity = focus_entity(model, template, number)?;
            if entity.typ() != expected {
                return Err(kernel::KernelError::new(format!(
                    "entity '{}' has Type '{}', not '{}'",
                    entity.local().as_str(),
                    entity.typ().as_str(),
                    expected.as_str()
                )));
            }
            Ok(Term::entity(entity))
        }
        _ => lower_term(model, expected, term),
    }
}

fn focus_entity(
    model: &Model,
    template: &frontend::EntityTemplate,
    number: u64,
) -> kernel::Result<EntityId> {
    let local = frontend::Name(format!(
        "{}{}{}",
        template.prefix.value, number, template.suffix.value
    ));
    resolve_entity(model, &local)
}

fn resolve_entity(model: &Model, authored: &frontend::Name) -> kernel::Result<EntityId> {
    let authored = authored.as_str();
    let local = if authored.contains('/') {
        authored
            .strip_prefix(&format!("{}/", model.id().as_str()))
            .ok_or_else(|| {
                kernel::KernelError::new(format!(
                    "qualified entity '{}' is not admitted by Model '{}'",
                    authored,
                    model.id().as_str()
                ))
            })?
    } else {
        authored
    };
    if local.contains('/') {
        return Err(kernel::KernelError::new(format!(
            "qualified entity '{}' is not a local entity of Model '{}'",
            authored,
            model.id().as_str()
        )));
    }
    model
        .entities()
        .iter()
        .find(|entity| entity.local().as_str() == local)
        .cloned()
        .ok_or_else(|| kernel::KernelError::new(format!("unknown entity '{}'", authored)))
}

fn cardinality(value: frontend::Cardinality) -> kernel::Cardinality {
    match value {
        frontend::Cardinality::One => kernel::Cardinality::One,
        frontend::Cardinality::Maybe => kernel::Cardinality::Maybe,
        frontend::Cardinality::Some => kernel::Cardinality::Some,
        frontend::Cardinality::Many => kernel::Cardinality::Many,
    }
}
fn name(value: &str) -> kernel::Result<Name> {
    Name::new(value.to_owned())
}
fn entity_local(value: &str) -> kernel::Result<Name> {
    Name::entity_local(value.to_owned())
}
fn type_id(value: &str) -> kernel::Result<TypeId> {
    TypeId::new(name(value)?)
}
fn model_id(value: &str) -> kernel::Result<ModelId> {
    ModelId::new(name(value)?)
}
fn relation_id(value: &str) -> kernel::Result<RelationId> {
    RelationId::new(name(value)?)
}
fn law_id(value: &str) -> kernel::Result<kernel::LawId> {
    kernel::LawId::new(name(value)?)
}
fn role_id(value: &str) -> kernel::Result<RoleId> {
    RoleId::new(name(value)?)
}
fn variable_id(value: &str) -> kernel::Result<VariableId> {
    VariableId::new(name(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASE: &str = "Module: Type\n\nimpact/imports: Relation\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n";
    fn compiled(source: &str) -> CompiledProgram {
        compile(frontend::parse(source).expect("parse")).expect("lower")
    }

    #[test]
    fn seals_base_and_preserves_request_order() {
        let program = compiled(&format!(
            "{BASE}\nwhy in impact:\n    North imports Store\n\nfind all ?dependency in impact:\n    North imports ?dependency\n"
        ));
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
    fn direct_and_reusable_deltas_seal_identically() {
        let program = compiled(&format!(
            "{BASE}\nimpact/direct: Revision\n    from: impact\n    admit:\n        South imports North\n\nimpact/add: Delta\n    from: impact\n    admit:\n        South imports North\n\nimpact/reusable: Revision\n    from: impact\n    apply: impact/add\n"
        ));
        let direct = program
            .revision(&frontend::Name("impact/direct".into()))
            .unwrap();
        let reusable = program
            .revision(&frontend::Name("impact/reusable".into()))
            .unwrap();
        assert_eq!(wire::serialize(direct), wire::serialize(reusable));
    }
    #[test]
    fn rejects_delta_base_mismatch_and_cross_model_entities() {
        let source = format!(
            "{BASE}\nother: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n\nimpact/change: Delta\n    from: impact\n    admit:\n        South imports North\n\nother/wrong: Revision\n    from: other\n    apply: impact/change\n"
        );
        assert!(
            compile(frontend::parse(&source).unwrap())
                .unwrap_err()
                .to_string()
                .contains("base does not match")
        );
        let source = format!(
            "{BASE}\nother: Model\n    North: Module\n\nimpact/bad: Revision\n    from: impact\n    admit:\n        other/North imports Store\n"
        );
        assert!(
            compile(frontend::parse(&source).unwrap())
                .unwrap_err()
                .to_string()
                .contains("not admitted by Model")
        );
    }

    #[test]
    fn rejects_an_invalid_delta_even_when_no_revision_applies_it() {
        let source = format!(
            "{BASE}\nimpact/orphan: Delta\n    from: impact\n    withdraw:\n        South imports North\n"
        );
        assert!(
            compile(frontend::parse(&source).unwrap())
                .unwrap_err()
                .to_string()
                .contains("withdraws a nonexistent assertion")
        );
    }
    #[test]
    fn rejects_lowered_revision_cycles_and_wrong_scalar_type() {
        let source = format!(
            "{BASE}\nimpact/one: Revision\n    from: impact\n    admit:\n        South imports North\n\nimpact/two: Revision\n    from: impact/one\n    admit:\n        Store imports South\n"
        );
        let mut program = frontend::parse(&source).unwrap();
        for declaration in &mut program.declarations {
            if declaration.subject.value.as_str() == "impact/one" {
                for member in &mut declaration.body {
                    if let Member::From(name) = member {
                        *name = frontend::Name("impact/two".into());
                    }
                }
            }
        }
        assert!(
            compile(program)
                .unwrap_err()
                .to_string()
                .contains("dependency cycle")
        );

        let program = compiled(BASE);
        let revision = program.revision(&frontend::Name("impact".into())).unwrap();
        let span = frontend::Span {
            line: 1,
            column: 1,
            width: 1,
        };
        let clause = SurfaceClause {
            relation: frontend::Spanned {
                value: frontend::Name("impact/imports".into()),
                span,
            },
            roles: BTreeMap::from([
                (
                    frontend::RoleName("consumer".into()),
                    SurfaceTerm::String(frontend::Spanned {
                        value: "North".into(),
                        span,
                    }),
                ),
                (
                    frontend::RoleName("dependency".into()),
                    SurfaceTerm::Entity(frontend::Spanned {
                        value: frontend::Name("Store".into()),
                        span,
                    }),
                ),
            ]),
            span,
        };
        assert!(
            lower_clause(revision, &clause)
                .unwrap_err()
                .to_string()
                .contains("admitted Text role")
        );
    }
    #[test]
    fn rejects_ambiguous_shapes_with_candidates() {
        let source = "Module: Type\n\na/links: Relation\n    {left: Module} links {right: Module}\n    mode left -> right: many\n\nb/links: Relation\n    {left: Module} links {right: Module}\n    mode left -> right: many\n\nm: Model\n    A: Module\n    B: Module\n    A links B\n";
        assert!(
            frontend::parse(source)
                .unwrap_err()
                .to_string()
                .contains("a/links, b/links")
        );
    }
}
