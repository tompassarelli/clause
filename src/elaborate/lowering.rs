use std::collections::BTreeMap;

use crate::{
    frontend::{self, SurfaceClause, SurfaceTerm},
    kernel::{self, Clause, EntityId, Model, Revision, SentencePart, Term, TypeId},
};

use super::identifiers::{relation_id, role_id, type_id, variable_id};

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

pub(super) fn lower_focus(
    revision: &Revision,
    focus: &frontend::FocusBlock,
) -> kernel::Result<Vec<Clause>> {
    let mut clauses = Vec::new();
    for number in focus.binding.range.start..=focus.binding.range.end {
        let focused = focus_entity(revision.model(), &focus.template, number)?;
        for slot in &focus.slots {
            let mut candidates = Vec::new();
            for relation in revision.model().relations().values() {
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
                revision.model(),
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
            revision.model().validate_clause(&clause, true)?;
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{elaborate::compile, frontend};

    use super::*;

    const BASE: &str = "Module: Type\n\nimpact/imports: Relation\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n";

    #[test]
    fn rejects_wrong_scalar_type() {
        let program = compile(frontend::parse(BASE).unwrap()).unwrap();
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
}
