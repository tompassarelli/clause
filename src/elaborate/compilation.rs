use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, AscriptionDecl, Kind, Member, ShapePartDecl},
    kernel::{self, EntityId, InlineSentencePart, Law, Model, Relation, Role, SentenceShape, Type},
    wire,
};

use super::{
    identifiers::{entity_local, model_id, relation_id, role_id, type_id},
    lowering::{lower_clause, lower_focus},
    resolution::Resolver,
};

/// Sealed revisions indexed by authored navigation names, plus requests in
/// authored order. Requests are deliberately outside revision identity.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    revisions: BTreeMap<frontend::Name, kernel::Revision>,
    requests: Vec<frontend::RequestDecl>,
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

fn declaration_map(
    declarations: &[AscriptionDecl],
) -> kernel::Result<BTreeMap<frontend::Name, &AscriptionDecl>> {
    let mut declarations_by_name = BTreeMap::new();
    for declaration in declarations {
        if declarations_by_name
            .insert(declaration.subject.value.clone(), declaration)
            .is_some()
        {
            return Err(kernel::KernelError::new(format!(
                "duplicate declaration '{}'",
                declaration.subject.value.as_str()
            )));
        }
    }
    Ok(declarations_by_name)
}

fn lower_types(declarations: &[AscriptionDecl]) -> kernel::Result<BTreeMap<kernel::TypeId, Type>> {
    let mut types_by_id = BTreeMap::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Type)
    {
        if !declaration.body.is_empty() {
            return Err(kernel::KernelError::new(
                "Type declarations cannot have members",
            ));
        }
        let id = type_id(&declaration.subject.value.0)?;
        if types_by_id.insert(id.clone(), Type::new(id)).is_some() {
            return Err(kernel::KernelError::new("duplicate Type identity"));
        }
    }
    Ok(types_by_id)
}

fn lower_relations(
    declarations: &[AscriptionDecl],
    types: &BTreeMap<kernel::TypeId, Type>,
) -> kernel::Result<BTreeMap<kernel::RelationId, Relation>> {
    let mut relations_by_id = BTreeMap::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Relation)
    {
        let shape = declaration
            .body
            .iter()
            .find_map(|member| match member {
                Member::Sentence(shape) => Some(shape),
                _ => None,
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
            .filter_map(|member| match member {
                Member::Mode(mode) => Some(mode),
                _ => None,
            })
            .map(|mode| {
                kernel::Mode::finite(
                    mode.known
                        .iter()
                        .map(|role| role_id(&role.value.0))
                        .collect::<kernel::Result<Vec<_>>>()?,
                    mode.sought
                        .iter()
                        .map(|role| role_id(&role.value.0))
                        .collect::<kernel::Result<Vec<_>>>()?,
                    cardinality(mode.cardinality),
                )
            })
            .collect::<kernel::Result<Vec<_>>>()?;
        let id = relation_id(&declaration.subject.value.0)?;
        let relation = Relation::new(id.clone(), SentenceShape::new(parts)?, modes)?;
        if relations_by_id.insert(id, relation).is_some() {
            return Err(kernel::KernelError::new("duplicate Relation identity"));
        }
    }
    Ok(relations_by_id)
}

fn lower_models(
    declarations: &[AscriptionDecl],
    types: &BTreeMap<kernel::TypeId, Type>,
    relations: &BTreeMap<kernel::RelationId, Relation>,
) -> kernel::Result<BTreeMap<frontend::Name, Model>> {
    let mut models_by_name = BTreeMap::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Model)
    {
        let id = model_id(&declaration.subject.value.0)?;
        let mut entities = BTreeSet::new();
        for entity in declaration.body.iter().filter_map(|member| match member {
            Member::Entity(entity) => Some(entity),
            _ => None,
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
        for group in declaration.body.iter().filter_map(|member| match member {
            Member::EntityGroup(group) => Some(group),
            _ => None,
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
        if models_by_name
            .insert(declaration.subject.value.clone(), model)
            .is_some()
        {
            return Err(kernel::KernelError::new("duplicate Model identity"));
        }
    }
    Ok(models_by_name)
}

fn attach_laws(
    declarations: &[AscriptionDecl],
    mut models: BTreeMap<frontend::Name, Model>,
) -> kernel::Result<BTreeMap<frontend::Name, Model>> {
    let mut laws_by_model = BTreeMap::<frontend::Name, Vec<Law>>::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.kind == Kind::Law)
    {
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
            .find_map(|member| match member {
                Member::Clause(clause) => Some(clause),
                _ => None,
            })
            .ok_or_else(|| kernel::KernelError::new("Law requires a conclusion"))?;
        let premises = declaration
            .body
            .iter()
            .find_map(|member| match member {
                Member::When(premises) => Some(premises),
                _ => None,
            })
            .ok_or_else(|| kernel::KernelError::new("Law requires when premises"))?;
        laws_by_model.entry(model_name).or_default().push(Law::new(
            super::identifiers::law_id(&declaration.subject.value.0)?,
            premises
                .iter()
                .map(|clause| lower_clause(&shell, clause))
                .collect::<kernel::Result<Vec<_>>>()?,
            lower_clause(&shell, conclusion)?,
        )?);
    }
    for (name, laws) in laws_by_model {
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
    use crate::frontend;

    use super::*;

    const BASE: &str = "Module: Type\n\nimpact/imports: Relation\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n";

    #[test]
    fn seals_base_and_preserves_request_order() {
        let program = compile(frontend::parse(&format!(
            "{BASE}\nwhy in impact:\n    North imports Store\n\nfind all ?dependency in impact:\n    North imports ?dependency\n"
        )).unwrap()).unwrap();
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
