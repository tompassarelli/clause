use std::collections::{BTreeMap, BTreeSet};

use crate::{
    kernel::{self, PatternId, ReferentId, RoleId},
    wire::sha256_digest,
};

/// Source designations are a mutable projection over opaque semantic
/// identities. Reusing this table across an explicit rename transaction keeps
/// the referent stable while changing its preferred term.
#[derive(Clone, Debug, Default)]
pub struct DesignationTable {
    globals: BTreeMap<String, ReferentId>,
    scoped: BTreeMap<(ReferentId, String), ReferentId>,
    literals: BTreeMap<String, ReferentId>,
    models: BTreeSet<ReferentId>,
    roles: BTreeMap<(ReferentId, String), RoleId>,
    patterns: BTreeMap<(ReferentId, String), PatternId>,
}

impl DesignationTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retain_global(&mut self, before: &str, after: &str) -> kernel::Result<()> {
        validate_designation(after)?;
        let id = self
            .globals
            .remove(before)
            .ok_or_else(|| kernel::KernelError::new("rename source designation is unknown"))?;
        if self.globals.insert(after.to_owned(), id.clone()).is_some() {
            self.globals.insert(before.to_owned(), id);
            return Err(kernel::KernelError::new(
                "rename destination designation already exists",
            ));
        }
        Ok(())
    }

    pub fn retain_scoped(
        &mut self,
        model: &ReferentId,
        before: &str,
        after: &str,
    ) -> kernel::Result<()> {
        validate_designation(after)?;
        let old = (model.clone(), before.to_owned());
        let id = self
            .scoped
            .remove(&old)
            .ok_or_else(|| kernel::KernelError::new("rename source designation is unknown"))?;
        let new = (model.clone(), after.to_owned());
        if self.scoped.insert(new, id.clone()).is_some() {
            self.scoped.insert(old, id);
            return Err(kernel::KernelError::new(
                "rename destination designation already exists",
            ));
        }
        Ok(())
    }

    pub fn retain_role(
        &mut self,
        relation: &ReferentId,
        before: &str,
        after: &str,
    ) -> kernel::Result<()> {
        validate_designation(after)?;
        let old = (relation.clone(), before.to_owned());
        let id = self
            .roles
            .remove(&old)
            .ok_or_else(|| kernel::KernelError::new("rename source role is unknown"))?;
        let new = (relation.clone(), after.to_owned());
        if self.roles.insert(new, id.clone()).is_some() {
            self.roles.insert(old, id);
            return Err(kernel::KernelError::new(
                "rename destination role already exists",
            ));
        }
        Ok(())
    }

    pub fn retain_pattern(
        &mut self,
        scope: &ReferentId,
        before: &str,
        after: &str,
    ) -> kernel::Result<()> {
        validate_designation(after)?;
        let old = (scope.clone(), before.to_owned());
        let id = self
            .patterns
            .remove(&old)
            .ok_or_else(|| kernel::KernelError::new("rename source pattern is unknown"))?;
        let new = (scope.clone(), after.to_owned());
        if self.patterns.insert(new, id.clone()).is_some() {
            self.patterns.insert(old, id);
            return Err(kernel::KernelError::new(
                "rename destination pattern already exists",
            ));
        }
        Ok(())
    }

    pub(crate) fn declare_global(&mut self, value: &str) -> kernel::Result<ReferentId> {
        validate_designation(value)?;
        if let Some(id) = self.globals.get(value) {
            return Ok(id.clone());
        }
        let id = synthetic_referent("global-designation", &[value]);
        self.globals.insert(value.to_owned(), id.clone());
        Ok(id)
    }

    pub(crate) fn declare_model(&mut self, value: &str) -> kernel::Result<ReferentId> {
        let id = self.declare_global(value)?;
        self.models.insert(id.clone());
        Ok(id)
    }

    pub(crate) fn declare_scoped(
        &mut self,
        model: &ReferentId,
        value: &str,
    ) -> kernel::Result<ReferentId> {
        validate_designation(value)?;
        let key = (model.clone(), value.to_owned());
        if let Some(id) = self.scoped.get(&key) {
            return Ok(id.clone());
        }
        let id = synthetic_referent("scoped-designation", &[model.as_str(), value]);
        self.scoped.insert(key, id.clone());
        Ok(id)
    }

    pub(crate) fn declare_literal(&mut self, value: &str) -> ReferentId {
        if let Some(id) = self.literals.get(value) {
            return id.clone();
        }
        let id = synthetic_referent("literal-designation", &[value]);
        self.literals.insert(value.to_owned(), id.clone());
        id
    }

    pub(crate) fn declare_role(
        &mut self,
        relation: &ReferentId,
        label: &str,
    ) -> kernel::Result<RoleId> {
        validate_designation(label)?;
        let key = (relation.clone(), label.to_owned());
        if let Some(id) = self.roles.get(&key) {
            return Ok(id.clone());
        }
        let id = synthetic_role("relation-role", &[relation.as_str(), label]);
        self.roles.insert(key, id.clone());
        Ok(id)
    }

    pub(crate) fn declare_pattern(
        &mut self,
        scope: &ReferentId,
        label: &str,
    ) -> kernel::Result<PatternId> {
        validate_designation(label)?;
        let key = (scope.clone(), label.to_owned());
        if let Some(id) = self.patterns.get(&key) {
            return Ok(id.clone());
        }
        let id = synthetic_pattern("pattern-binding", &[scope.as_str(), label]);
        self.patterns.insert(key, id.clone());
        Ok(id)
    }

    /// Resolve one global source designation to its stable semantic referent.
    pub fn global(&self, value: &str) -> kernel::Result<ReferentId> {
        self.globals
            .get(value)
            .cloned()
            .ok_or_else(|| kernel::KernelError::new(format!("unknown designation '{value}'")))
    }

    /// Resolve a source designation admitted within one Model projection.
    pub fn scoped(&self, current_model: &ReferentId, authored: &str) -> kernel::Result<ReferentId> {
        let (model, local) = if authored.contains('/') {
            self.models
                .iter()
                .filter_map(|model| {
                    let name = self.global_name(model)?;
                    authored
                        .strip_prefix(&format!("{name}/"))
                        .map(|local| (model, local, name.len()))
                })
                .max_by_key(|(_, _, width)| *width)
                .map(|(model, local, _)| (model, local))
                .ok_or_else(|| {
                    kernel::KernelError::new(format!("unknown qualified designation '{authored}'"))
                })?
        } else {
            (current_model, authored)
        };
        if model != current_model {
            return Err(kernel::KernelError::new(format!(
                "qualified designation '{authored}' is not admitted by this Model"
            )));
        }
        self.scoped
            .get(&(model.clone(), local.to_owned()))
            .cloned()
            .ok_or_else(|| kernel::KernelError::new(format!("unknown designation '{authored}'")))
    }

    /// Resolve one literal spelling to the referent declared for that term.
    pub fn literal(&self, value: &str) -> kernel::Result<ReferentId> {
        self.literals
            .get(value)
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("string literal has no declared referent"))
    }

    /// Resolve an authored role label within one relation shape.
    pub fn role(&self, relation: &ReferentId, label: &str) -> kernel::Result<RoleId> {
        self.roles
            .get(&(relation.clone(), label.to_owned()))
            .cloned()
            .ok_or_else(|| kernel::KernelError::new(format!("unknown role '{label}'")))
    }

    pub(crate) fn global_name(&self, id: &ReferentId) -> Option<&str> {
        self.globals
            .iter()
            .find_map(|(name, candidate)| (candidate == id).then_some(name.as_str()))
    }
}

pub(super) fn synthetic_referent(namespace: &str, fields: &[&str]) -> ReferentId {
    let mut preimage = b"clause-referent-designation-v1\0".to_vec();
    write_field(&mut preimage, namespace);
    for field in fields {
        write_field(&mut preimage, field);
    }
    ReferentId::from_digest(sha256_digest(&preimage))
}

pub(super) fn synthetic_role(namespace: &str, fields: &[&str]) -> RoleId {
    RoleId::from_digest(synthetic_digest(namespace, fields))
}

pub(super) fn synthetic_pattern(namespace: &str, fields: &[&str]) -> PatternId {
    PatternId::from_digest(synthetic_digest(namespace, fields))
}

fn synthetic_digest(namespace: &str, fields: &[&str]) -> [u8; 32] {
    let mut preimage = b"clause-scoped-identity-v1\0".to_vec();
    write_field(&mut preimage, namespace);
    for field in fields {
        write_field(&mut preimage, field);
    }
    sha256_digest(&preimage)
}

fn validate_designation(value: &str) -> kernel::Result<()> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        Err(kernel::KernelError::new("invalid source designation"))
    } else {
        Ok(())
    }
}

fn write_field(preimage: &mut Vec<u8>, value: &str) {
    preimage.extend_from_slice(&(value.len() as u64).to_be_bytes());
    preimage.extend_from_slice(value.as_bytes());
}
