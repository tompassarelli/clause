use std::fmt;

use super::error::{KernelError, Result};

/// A qualified Clause name. Roles and variables remain strict local segments;
/// entity locals use the explicit `Name::entity_local` constructor because
/// human-facing labels may contain spaces.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

impl Name {
    pub fn new(value: String) -> Result<Self> {
        if valid_name(&value) {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid name"))
        }
    }

    /// Construct the local identity of an entity.
    ///
    /// Entity locals are the one semantic identifier that may contain spaces:
    /// a displayed label such as `Zone 7` is still one stable identity, not a
    /// pair of names. Every other identifier continues to use `Name::new` and
    /// therefore retains the strict segment grammar.
    pub fn entity_local(value: String) -> Result<Self> {
        if valid_entity_local(&value) {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid entity local name"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_local(&self) -> bool {
        !self.0.contains('/')
    }

    fn is_strict(&self) -> bool {
        valid_name(&self.0)
    }

    fn is_entity_local(&self) -> bool {
        valid_entity_local(&self.0)
    }
}

macro_rules! identity {
    ($name:ident, $message:literal) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(Name);

        impl $name {
            pub fn new(name: Name) -> Result<Self> {
                if name.is_strict() {
                    Ok(Self(name))
                } else {
                    Err(KernelError::new($message))
                }
            }

            pub fn name(&self) -> &Name {
                &self.0
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

identity!(TypeId, "invalid type identity");
identity!(ModelId, "invalid model identity");
identity!(RelationId, "invalid relation identity");
identity!(LawId, "invalid law identity");

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleId(Name);

impl RoleId {
    pub fn new(name: Name) -> Result<Self> {
        if name.is_local() && name.is_strict() {
            Ok(Self(name))
        } else {
            Err(KernelError::new("role identity must be a local name"))
        }
    }

    pub fn name(&self) -> &Name {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VariableId(Name);

impl VariableId {
    pub fn new(name: Name) -> Result<Self> {
        if name.is_local() && name.is_strict() {
            Ok(Self(name))
        } else {
            Err(KernelError::new("variable identity must be a local name"))
        }
    }

    pub fn name(&self) -> &Name {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// The content-addressed identity of one admitted model revision.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionId([u8; 32]);

impl RevisionId {
    pub(crate) fn from_digest(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("rev-sha256-")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityId {
    model: ModelId,
    local: Name,
    typ: TypeId,
}

impl EntityId {
    pub fn new(model: ModelId, local: Name, typ: TypeId) -> Result<Self> {
        if !local.is_entity_local() {
            return Err(KernelError::new("invalid entity local name"));
        }
        Ok(Self { model, local, typ })
    }

    pub fn model(&self) -> &ModelId {
        &self.model
    }

    pub fn local(&self) -> &Name {
        &self.local
    }

    pub fn typ(&self) -> &TypeId {
        &self.typ
    }
}

fn valid_segment(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn valid_name(value: &str) -> bool {
    !value.is_empty() && value.split('/').all(valid_segment)
}

fn valid_entity_local(value: &str) -> bool {
    !value.is_empty()
        && value.split(' ').all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                })
        })
}
