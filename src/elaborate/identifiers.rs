use crate::kernel::{self, LawId, ModelId, Name, RelationId, RoleId, TypeId, VariableId};

pub(super) fn entity_local(value: &str) -> kernel::Result<Name> {
    Name::entity_local(value.to_owned())
}

pub(super) fn law_id(value: &str) -> kernel::Result<LawId> {
    LawId::new(name(value)?)
}

pub(super) fn model_id(value: &str) -> kernel::Result<ModelId> {
    ModelId::new(name(value)?)
}

pub(super) fn relation_id(value: &str) -> kernel::Result<RelationId> {
    RelationId::new(name(value)?)
}

pub(super) fn role_id(value: &str) -> kernel::Result<RoleId> {
    RoleId::new(name(value)?)
}

pub(super) fn type_id(value: &str) -> kernel::Result<TypeId> {
    TypeId::new(name(value)?)
}

pub(super) fn variable_id(value: &str) -> kernel::Result<VariableId> {
    VariableId::new(name(value)?)
}

fn name(value: &str) -> kernel::Result<Name> {
    Name::new(value.to_owned())
}
