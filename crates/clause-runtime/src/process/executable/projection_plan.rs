//! Resolve the immutable projection template once; values still come from each admitted state.
use super::*;

pub(super) struct ProjectionPlan {
    node: Node,
    has_roles: bool,
}

enum Node {
    Static(Term),
    State(TermScope, ExecutableProjectionBindingV1),
    Row(TermScope, u16, ExecutableValueV1),
    Triple(Box<[ProjectionPlan; 3]>, bool),
}

impl ProjectionPlan {
    pub fn new(projection: &ExecutableProjectionV1) -> Result<Self, ExecutableErrorV1> {
        let bindings = projection.bindings.iter().map(|binding| (binding.role, *binding)).collect();
        Self::compile(&projection.template, &bindings)
    }

    fn compile(template: &Term, bindings: &BTreeMap<LocalRoleRefV2, ExecutableProjectionBindingV1>) -> Result<Self, ExecutableErrorV1> {
        if let Some((table, subject)) = relational_projection::row_selection(template) {
            let (role, kind) = projection_role(table.as_atom().ok_or(ExecutableErrorV1::MalformedProgram)?)?
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
            if kind != ExecutableValueKindV1::RelationTable { return Err(ExecutableErrorV1::MalformedProgram); }
            let binding = bindings.get(&role).ok_or(ExecutableErrorV1::MalformedProgram)?;
            let subject = projected_referent_value_v1(subject)?.ok_or(ExecutableErrorV1::MalformedProgram)?;
            return Ok(Self { node: Node::Row(template.scope(), binding.slot, ExecutableValueV1::Referent(subject)), has_roles: true });
        }
        if let Some(atom) = template.as_atom() {
            return if let Some((role, kind)) = projection_role(atom)? {
                let binding = *bindings.get(&role).ok_or(ExecutableErrorV1::MalformedProgram)?;
                if binding.value_kind != kind { return Err(ExecutableErrorV1::TypeMismatch); }
                Ok(Self { node: Node::State(template.scope(), binding), has_roles: true })
            } else { Ok(Self { node: Node::Static(template.clone()), has_roles: false }) };
        }
        let [left, operator, right] = template.as_triple().ok_or(ExecutableErrorV1::MalformedProgram)?.slots();
        let field = left.as_atom().is_some_and(|atom| atom.kind() == b"clause/js-field-v1");
        let children = [Self::compile(left, bindings)?, Self::compile(operator, bindings)?, Self::compile(right, bindings)?];
        let has_roles = children.iter().any(|child| child.has_roles);
        Ok(Self { node: if has_roles { Node::Triple(Box::new(children), field) } else { Node::Static(template.clone()) }, has_roles })
    }

    fn row<'a>(slot: u16, configuration: &'a [ExecutableSlotV1]) -> Result<&'a ExecutableRelationTableV1, ExecutableErrorV1> {
        match configuration.get(usize::from(slot)).and_then(ExecutableSlotV1::value) {
            Some(ExecutableValueV1::RelationTable(table)) => Ok(table),
            _ => Err(ExecutableErrorV1::TypeMismatch),
        }
    }

    fn present(&self, configuration: &[ExecutableSlotV1]) -> Result<bool, ExecutableErrorV1> {
        Ok(match &self.node {
            Node::Static(_) => false,
            Node::State(_, binding) => configuration.get(usize::from(binding.slot)).is_some_and(|slot| slot.value().is_some()),
            Node::Row(_, slot, subject) => Self::row(*slot, configuration)?.present(subject)?,
            Node::Triple(children, _) => children[0].present(configuration)? || children[1].present(configuration)? || children[2].present(configuration)?,
        })
    }

    pub fn realize(&self, configuration: &[ExecutableSlotV1]) -> Result<Term, ExecutableErrorV1> {
        match &self.node {
            Node::Static(term) => Ok(term.clone()),
            Node::State(scope, binding) => {
                let slot = configuration.get(usize::from(binding.slot)).ok_or(ExecutableErrorV1::UnknownSlot(binding.slot))?;
                let value = slot.value().ok_or(ExecutableErrorV1::MissingState)?;
                if value.kind() != binding.value_kind { return Err(ExecutableErrorV1::TypeMismatch); }
                projected_value_term(*scope, value.clone())
            }
            Node::Row(scope, slot, subject) => {
                let table = Self::row(*slot, configuration)?;
                if !table.present(subject)? { return Err(ExecutableErrorV1::MissingState); }
                projected_value_term(*scope, table.read(subject)?)
            }
            Node::Triple(children, field) => {
                if *field && children[1].has_roles && !children[1].present(configuration)? {
                    return children[2].realize(configuration);
                }
                Term::triple([children[0].realize(configuration)?, children[1].realize(configuration)?, children[2].realize(configuration)?])
                    .map_err(|_| ExecutableErrorV1::MalformedProgram)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_template_projects_current_values_and_omits_absent_fields() {
        let scope = TermScope { universe: UniverseId::from_bytes([1; 32]), semantics: ClauseSemanticsId::from_bytes([2; 32]) };
        let role = |id| LocalRoleRefV2 { schema: RelationSchemaLocalId::new(0), role: RoleLocalId::new(id) };
        let number = |value| ExecutableValueV1::number(value).unwrap();
        let literal = |value| projected_value_term(scope, number(value)).unwrap();
        let subject = ExecutableValueV1::Referent(ExecutableReferentV1::declared(1, 1));
        let row = Term::triple([
            projection_literal(scope, b"clause/js-relation-row-selector-v1", &[]).unwrap(),
            executable_projection_role_term_v1(scope, role(1), ExecutableValueKindV1::RelationTable).unwrap(),
            projected_value_term(scope, subject.clone()).unwrap(),
        ]).unwrap();
        let projection = ExecutableProjectionV1 {
            bindings: vec![
                ExecutableProjectionBindingV1 { role: role(0), slot: 0, value_kind: ExecutableValueKindV1::Number },
                ExecutableProjectionBindingV1 { role: role(1), slot: 1, value_kind: ExecutableValueKindV1::RelationTable },
            ],
            template: projection_object(scope, vec![
                (b"fixed".to_vec(), literal(7.0)),
                (b"state".to_vec(), executable_projection_role_term_v1(scope, role(0), ExecutableValueKindV1::Number).unwrap()),
                (b"row".to_vec(), row),
            ]).unwrap(),
        };
        let plan = ProjectionPlan::new(&projection).unwrap();
        let mut table = ExecutableRelationTableV1 {
            subject_domain: 1, value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None, cardinality: ExecutableRelationCardinalityV1::Maybe, total: false,
            rows: Arc::default(),
        };
        let absent = [ExecutableSlotV1::Absent(ExecutableValueKindV1::Number), ExecutableSlotV1::Present(ExecutableValueV1::RelationTable(table.clone()))];
        assert_eq!(plan.realize(&absent).unwrap(), projection_object(scope, vec![(b"fixed".to_vec(), literal(7.0))]).unwrap());
        for value in [2.0, 9.0] {
            table.put(&subject, number(value + 1.0)).unwrap();
            let present = [ExecutableSlotV1::Present(number(value)), ExecutableSlotV1::Present(ExecutableValueV1::RelationTable(table.clone()))];
            assert_eq!(plan.realize(&present).unwrap(), projection_object(scope, vec![
                (b"fixed".to_vec(), literal(7.0)), (b"state".to_vec(), literal(value)), (b"row".to_vec(), literal(value + 1.0)),
            ]).unwrap());
        }
        assert_eq!(plan.realize(&absent).unwrap(), projection_object(scope, vec![(b"fixed".to_vec(), literal(7.0))]).unwrap());
    }
}
