use crate::{
    kernel::{ReferentId, RoleId},
    wire::sha256_digest,
};

const SOURCE_PREFIX: &str = "@clause/intrinsic/";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Intrinsic {
    Add,
    Subtract,
    Multiply,
    Divide,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Length,
    Map,
    Conditional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntrinsicRole {
    Left,
    Right,
    Input,
    Mapper,
    Sequence,
    Condition,
    Then,
    Else,
    Result,
}

impl Intrinsic {
    pub(crate) const ALL: [Self; 13] = [
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::Divide,
        Self::LessThan,
        Self::LessOrEqual,
        Self::GreaterThan,
        Self::GreaterOrEqual,
        Self::Equal,
        Self::NotEqual,
        Self::Length,
        Self::Map,
        Self::Conditional,
    ];

    pub(crate) fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|intrinsic| intrinsic.name() == name)
    }

    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        Self::named(name.strip_prefix(SOURCE_PREFIX)?)
    }

    pub(crate) fn from_relation(relation: &ReferentId) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|intrinsic| intrinsic.relation() == *relation)
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::Divide => "divide",
            Self::LessThan => "less-than",
            Self::LessOrEqual => "less-or-equal",
            Self::GreaterThan => "greater-than",
            Self::GreaterOrEqual => "greater-or-equal",
            Self::Equal => "equal",
            Self::NotEqual => "not-equal",
            Self::Length => "length",
            Self::Map => "map",
            Self::Conditional => "conditional",
        }
    }

    pub(crate) fn input_roles(self) -> &'static [IntrinsicRole] {
        use IntrinsicRole::{Condition, Else, Input, Left, Mapper, Right, Sequence, Then};
        match self {
            Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::LessThan
            | Self::LessOrEqual
            | Self::GreaterThan
            | Self::GreaterOrEqual
            | Self::Equal
            | Self::NotEqual => &[Left, Right],
            Self::Length => &[Input],
            Self::Map => &[Mapper, Sequence],
            Self::Conditional => &[Condition, Then, Else],
        }
    }

    pub(crate) fn relation(self) -> ReferentId {
        synthetic_referent("pure-intrinsic-relation", &[self.name()])
    }

    pub(crate) fn role(self, role: IntrinsicRole) -> RoleId {
        synthetic_role("pure-intrinsic-role", &[self.name(), role.name()])
    }

    pub(crate) fn role_named(self, role: &str) -> Option<RoleId> {
        self.input_roles()
            .iter()
            .copied()
            .find(|candidate| candidate.name() == role)
            .map(|candidate| self.role(candidate))
    }

    pub(crate) fn callable_identity(self) -> ReferentId {
        let source_name = format!("{SOURCE_PREFIX}{}", self.name());
        synthetic_referent("pure-intrinsic-identity", &[&source_name])
    }
}

impl IntrinsicRole {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Input => "input",
            Self::Mapper => "mapper",
            Self::Sequence => "sequence",
            Self::Condition => "condition",
            Self::Then => "then",
            Self::Else => "else",
            Self::Result => "result",
        }
    }
}

fn synthetic_referent(namespace: &str, fields: &[&str]) -> ReferentId {
    let mut preimage = b"clause-referent-designation-v1\0".to_vec();
    write_field(&mut preimage, namespace);
    for field in fields {
        write_field(&mut preimage, field);
    }
    ReferentId::from_digest(sha256_digest(&preimage))
}

fn synthetic_role(namespace: &str, fields: &[&str]) -> RoleId {
    let mut preimage = b"clause-scoped-identity-v1\0".to_vec();
    write_field(&mut preimage, namespace);
    for field in fields {
        write_field(&mut preimage, field);
    }
    RoleId::from_digest(sha256_digest(&preimage))
}

fn write_field(preimage: &mut Vec<u8>, value: &str) {
    preimage.extend_from_slice(&(value.len() as u64).to_be_bytes());
    preimage.extend_from_slice(value.as_bytes());
}
