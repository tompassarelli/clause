use std::fmt;

use crate::wire::sha256_digest;

use super::error::{KernelError, Result};

/// A qualified source/navigation name, never semantic identity.
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The sole identity domain for addressable semantic distinctions.
///
/// A referent can occupy relational position, a participant role, or identify
/// a rule, occurrence, judgment, definition, or modal specification. Its
/// identity is independent of labels and structural equality.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReferentId(String);

impl ReferentId {
    pub fn new(value: String) -> Result<Self> {
        let Some(hex) = value.strip_prefix("ref-sha256-") else {
            return Err(KernelError::new("invalid referent identity"));
        };
        if hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid referent identity"))
        }
    }

    pub fn from_digest(bytes: [u8; 32]) -> Self {
        let mut value = String::from("ref-sha256-");
        for byte in bytes {
            use std::fmt::Write;
            write!(value, "{byte:02x}").expect("writing a digest to String cannot fail");
        }
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Content-derived engineering identity for one canonical role-labelled form.
/// It does not create another addressable referent species.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContentId(String);

impl ContentId {
    pub fn new(value: String) -> Result<Self> {
        let Some(hex) = value.strip_prefix("content-sha256-") else {
            return Err(KernelError::new("invalid relational content identity"));
        };
        if hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid relational content identity"))
        }
    }

    pub(crate) fn from_digest(bytes: [u8; 32]) -> Self {
        let mut value = String::from("content-sha256-");
        for byte in bytes {
            use std::fmt::Write;
            write!(value, "{byte:02x}").expect("writing a digest to String cannot fail");
        }
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque stable identity for a named relational role. Source labels resolve
/// to this identity outside the semantic kernel.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleId(String);

impl RoleId {
    pub fn new(value: String) -> Result<Self> {
        let Some(hex) = value.strip_prefix("role-sha256-") else {
            return Err(KernelError::new("invalid role identity"));
        };
        if hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid role identity"))
        }
    }

    pub fn from_digest(bytes: [u8; 32]) -> Self {
        let mut value = String::from("role-sha256-");
        for byte in bytes {
            use std::fmt::Write;
            write!(value, "{byte:02x}").expect("writing a digest to String cannot fail");
        }
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PatternId(String);

impl PatternId {
    pub fn new(value: String) -> Result<Self> {
        let Some(hex) = value.strip_prefix("pattern-sha256-") else {
            return Err(KernelError::new("invalid pattern identity"));
        };
        if hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid pattern identity"))
        }
    }

    pub fn from_digest(bytes: [u8; 32]) -> Self {
        let mut value = String::from("pattern-sha256-");
        for byte in bytes {
            use std::fmt::Write;
            write!(value, "{byte:02x}").expect("writing a digest to String cannot fail");
        }
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

/// Identifies the language semantics and canonical checked representation
/// used to interpret a program snapshot.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClauseSemanticsId(String);

impl ClauseSemanticsId {
    pub fn new(value: String) -> Result<Self> {
        if value.is_empty()
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
        {
            return Err(KernelError::new("invalid Clause semantics identity"));
        }
        Ok(Self(value))
    }

    pub fn current() -> Self {
        Self("clause-semantics-v1".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Durable identity of an evolving Clause program lineage.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgramId(String);

impl ProgramId {
    pub fn new(value: String) -> Result<Self> {
        validate_prefixed_hex(&value, "program-sha256-")
            .then_some(Self(value))
            .ok_or_else(|| KernelError::new("invalid program identity"))
    }

    pub fn from_digest(bytes: [u8; 32]) -> Self {
        Self(format_digest("program-sha256-", bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Content identity of one canonical checked program snapshot.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgramSnapshotId(String);

impl ProgramSnapshotId {
    pub fn new(value: String) -> Result<Self> {
        validate_prefixed_hex(&value, "program-snapshot-sha256-")
            .then_some(Self(value))
            .ok_or_else(|| KernelError::new("invalid program snapshot identity"))
    }

    pub fn from_digest(bytes: [u8; 32]) -> Self {
        Self(format_digest("program-snapshot-sha256-", bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_prefixed_hex(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

fn format_digest(prefix: &str, bytes: [u8; 32]) -> String {
    let mut value = prefix.to_owned();
    for byte in bytes {
        use std::fmt::Write;
        write!(value, "{byte:02x}").expect("writing a digest to String cannot fail");
    }
    value
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

pub(crate) fn synthetic_referent(namespace: &str, fields: &[&str]) -> ReferentId {
    let mut preimage = b"clause-referent-designation-v1\0".to_vec();
    write_field(&mut preimage, namespace);
    for field in fields {
        write_field(&mut preimage, field);
    }
    ReferentId::from_digest(sha256_digest(&preimage))
}

pub(crate) fn synthetic_role(namespace: &str, fields: &[&str]) -> RoleId {
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

fn valid_segment(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn valid_name(value: &str) -> bool {
    !value.is_empty() && value.split('/').all(valid_segment)
}
