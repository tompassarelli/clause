//! Resolve the immutable projection template once; values still come from each admitted state.
use super::*;

pub(super) struct ProjectionPlan {
    retained: Mutex<RetainedProjection>,
}

struct RetainedProjection {
    node: ProjectionNode,
    configuration: Option<Vec<ExecutableSlotV1>>,
    // Omitted fields can skip descendants for several calls; their caches
    // must notice every dependency change since their own last realization.
    versions: Vec<u64>,
}

struct ProjectionNode {
    node: Node,
    has_roles: bool,
    dependencies: Vec<u16>,
    cached_versions: Vec<u64>,
    cache: Option<Cached>,
}

enum Cached {
    Value(ExecutableValueV1, Term),
    Triple(Term),
    Omitted(Term),
}

impl Cached {
    fn term(&self) -> &Term {
        match self { Self::Value(_, term) | Self::Triple(term) | Self::Omitted(term) => term }
    }
}

enum Node {
    Static(Term),
    State(TermScope, ExecutableProjectionBindingV1),
    Row(TermScope, u16, ExecutableValueV1),
    Triple(Box<[ProjectionNode; 3]>, bool),
}

impl ProjectionPlan {
    pub fn new(projection: &ExecutableProjectionV1) -> Result<Self, ExecutableErrorV1> {
        let bindings = projection.bindings.iter().map(|binding| (binding.role, *binding)).collect();
        Ok(Self { retained: Mutex::new(RetainedProjection {
            node: ProjectionNode::compile(&projection.template, &bindings)?, configuration: None, versions: vec![],
        }) })
    }

    pub fn realize(&self, configuration: &[ExecutableSlotV1]) -> Result<Term, ExecutableErrorV1> {
        let mut retained = self.retained.lock().map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        // A failed realization may update only part of the tree. In that case
        // leave no configuration baseline, so the next call visits every node.
        let previous = retained.configuration.take();
        let changed = configuration.iter().enumerate().map(|(index, slot)| {
            previous.as_ref().and_then(|slots| slots.get(index)) != Some(slot)
        }).collect::<Vec<_>>();
        let mut versions = std::mem::take(&mut retained.versions);
        versions.resize(versions.len().max(configuration.len()), 0);
        for (version, changed) in versions.iter_mut().zip(changed) {
            if changed { *version = version.saturating_add(1); }
        }
        let result = retained.node.realize(configuration, &versions);
        retained.versions = versions;
        let term = result?;
        retained.configuration = Some(configuration.to_vec());
        Ok(term)
    }
}

impl ProjectionNode {
    fn compile(template: &Term, bindings: &BTreeMap<LocalRoleRefV2, ExecutableProjectionBindingV1>) -> Result<Self, ExecutableErrorV1> {
        if let Some((table, subject)) = relational_projection::row_selection(template) {
            let (role, kind) = projection_role(table.as_atom().ok_or(ExecutableErrorV1::MalformedProgram)?)?
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
            if kind != ExecutableValueKindV1::RelationTable { return Err(ExecutableErrorV1::MalformedProgram); }
            let binding = bindings.get(&role).ok_or(ExecutableErrorV1::MalformedProgram)?;
            let subject = projected_referent_value_v1(subject)?.ok_or(ExecutableErrorV1::MalformedProgram)?;
            return Ok(Self { node: Node::Row(template.scope(), binding.slot, ExecutableValueV1::Referent(subject)), has_roles: true, dependencies: vec![binding.slot], cached_versions: vec![], cache: None });
        }
        if let Some(atom) = template.as_atom() {
            return if let Some((role, kind)) = projection_role(atom)? {
                let binding = *bindings.get(&role).ok_or(ExecutableErrorV1::MalformedProgram)?;
                if binding.value_kind != kind { return Err(ExecutableErrorV1::TypeMismatch); }
                Ok(Self { node: Node::State(template.scope(), binding), has_roles: true, dependencies: vec![binding.slot], cached_versions: vec![], cache: None })
            } else { Ok(Self { node: Node::Static(template.clone()), has_roles: false, dependencies: vec![], cached_versions: vec![], cache: None }) };
        }
        let [left, operator, right] = template.as_triple().ok_or(ExecutableErrorV1::MalformedProgram)?.slots();
        let field = left.as_atom().is_some_and(|atom| atom.kind() == b"clause/js-field-v1");
        let children = [Self::compile(left, bindings)?, Self::compile(operator, bindings)?, Self::compile(right, bindings)?];
        let has_roles = children.iter().any(|child| child.has_roles);
        let mut dependencies = children.iter().flat_map(|child| child.dependencies.iter().copied()).collect::<Vec<_>>();
        dependencies.sort_unstable();
        dependencies.dedup();
        Ok(Self { node: if has_roles { Node::Triple(Box::new(children), field) } else { Node::Static(template.clone()) }, has_roles, dependencies, cached_versions: vec![], cache: None })
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

    fn cached_value(cache: &mut Option<Cached>, scope: TermScope, value: ExecutableValueV1) -> Result<Term, ExecutableErrorV1> {
        if let Some(Cached::Value(previous, term)) = cache.as_ref() {
            if previous == &value { return Ok(term.clone()); }
        }
        let term = projected_value_term(scope, value.clone())?;
        *cache = Some(Cached::Value(value, term.clone()));
        Ok(term)
    }

    fn realize(&mut self, configuration: &[ExecutableSlotV1], versions: &[u64]) -> Result<Term, ExecutableErrorV1> {
        if self.cached_versions.len() == self.dependencies.len() && self.dependencies.iter().zip(&self.cached_versions).all(|(slot, cached)| {
            *cached != u64::MAX && configuration.get(usize::from(*slot)).is_some() && versions.get(usize::from(*slot)) == Some(cached)
        }) {
            if let Some(cache) = &self.cache { return Ok(cache.term().clone()); }
        }
        let result = self.realize_changed(configuration, versions)?;
        self.cached_versions = self.dependencies.iter().map(|slot| versions.get(usize::from(*slot)).copied().unwrap_or(0)).collect();
        Ok(result)
    }

    fn realize_changed(&mut self, configuration: &[ExecutableSlotV1], versions: &[u64]) -> Result<Term, ExecutableErrorV1> {
        match &mut self.node {
            Node::Static(term) => Ok(term.clone()),
            Node::State(scope, binding) => {
                let slot = configuration.get(usize::from(binding.slot)).ok_or(ExecutableErrorV1::UnknownSlot(binding.slot))?;
                let value = slot.value().ok_or(ExecutableErrorV1::MissingState)?;
                if value.kind() != binding.value_kind { return Err(ExecutableErrorV1::TypeMismatch); }
                Self::cached_value(&mut self.cache, *scope, value.clone())
            }
            Node::Row(scope, slot, subject) => {
                let table = Self::row(*slot, configuration)?;
                Self::cached_value(&mut self.cache, *scope, table.read(subject)?)
            }
            Node::Triple(children, field) => {
                if *field && children[1].has_roles && !children[1].present(configuration)? {
                    let term = children[2].realize(configuration, versions)?;
                    self.cache = Some(Cached::Omitted(term.clone()));
                    return Ok(term);
                }
                let values = [children[0].realize(configuration, versions)?, children[1].realize(configuration, versions)?, children[2].realize(configuration, versions)?];
                if let Some(Cached::Triple(term)) = self.cache.as_ref() {
                    if term.as_triple().is_some_and(|triple| triple.slots() == values.each_ref()) { return Ok(term.clone()); }
                }
                let term = Term::triple(values).map_err(|_| ExecutableErrorV1::MalformedProgram)?;
                self.cache = Some(Cached::Triple(term.clone()));
                Ok(term)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_parent_does_not_leave_descendant_values_current() {
        let scope = TermScope { universe: UniverseId::from_bytes([1; 32]), semantics: ClauseSemanticsId::from_bytes([2; 32]) };
        let role = |id| LocalRoleRefV2 { schema: RelationSchemaLocalId::new(0), role: RoleLocalId::new(id) };
        let number = |value| ExecutableValueV1::number(value).unwrap();
        let literal = |value| projected_value_term(scope, number(value)).unwrap();
        let object = |fields| projection_object(scope, fields).unwrap();
        let projection = ExecutableProjectionV1 {
            bindings: (0..2).map(|slot| ExecutableProjectionBindingV1 { role: role(u32::from(slot)), slot, value_kind: ExecutableValueKindV1::Number }).collect(),
            template: object(vec![(b"group".to_vec(), object(vec![
                (b"a".to_vec(), executable_projection_role_term_v1(scope, role(0), ExecutableValueKindV1::Number).unwrap()),
                (b"b".to_vec(), executable_projection_role_term_v1(scope, role(1), ExecutableValueKindV1::Number).unwrap()),
            ]))]),
        };
        let plan = ProjectionPlan::new(&projection).unwrap();
        let present = |value| ExecutableSlotV1::Present(number(value));
        let absent = || ExecutableSlotV1::Absent(ExecutableValueKindV1::Number);
        plan.realize(&[present(1.0), present(2.0)]).unwrap();
        assert_eq!(plan.realize(&[absent(), absent()]).unwrap(), object(vec![]));
        let expected = object(vec![(b"group".to_vec(), object(vec![(b"a".to_vec(), literal(3.0))]))]);
        assert_eq!(plan.realize(&[present(3.0), absent()]).unwrap(), expected);
        assert_eq!(plan.realize(&[present(3.0), absent()]).unwrap(), expected);
    }

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
            let actual = plan.realize(&present).unwrap();
            assert_eq!(actual, projection_object(scope, vec![
                (b"fixed".to_vec(), literal(7.0)), (b"state".to_vec(), literal(value)), (b"row".to_vec(), literal(value + 1.0)),
            ]).unwrap());
            assert_eq!(plan.realize(&present).unwrap(), actual);
            assert_eq!(plan.realize(&absent).unwrap(), projection_object(scope, vec![(b"fixed".to_vec(), literal(7.0))]).unwrap());
            assert_eq!(plan.realize(&present).unwrap(), actual);
            let malformed = [ExecutableSlotV1::Present(number(value + 10.0)), ExecutableSlotV1::Present(number(0.0))];
            assert!(plan.realize(&malformed).is_err());
            assert_eq!(plan.realize(&present).unwrap(), actual);
        }
        assert_eq!(plan.realize(&absent).unwrap(), projection_object(scope, vec![(b"fixed".to_vec(), literal(7.0))]).unwrap());
    }
}
