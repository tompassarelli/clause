use std::collections::{BTreeMap, BTreeSet};

use crate::{
    kernel::{self, ContentId, PatternId, ProposalPath, ProposalPathSegment, ReferentId, RoleId},
    wire::sha256_digest,
};

/// Source designations are a mutable projection over opaque semantic
/// identities. Reusing this table across an explicit rename transaction keeps
/// the referent stable while changing its preferred term.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

    pub(crate) fn identity_count(&self) -> usize {
        self.globals.len()
            + self.scoped.len()
            + self.literals.len()
            + self.roles.len()
            + self.patterns.len()
    }

    pub fn retain_global(&mut self, before: &str, after: &str) -> kernel::Result<()> {
        validate_designation(after)?;
        let id = self
            .globals
            .get(before)
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("rename source designation is unknown"))?;
        if before != after && self.globals.contains_key(after) {
            return Err(kernel::KernelError::new(
                "rename destination designation already exists",
            ));
        }
        self.globals.remove(before);
        self.globals.insert(after.to_owned(), id);
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
            .get(&old)
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("rename source designation is unknown"))?;
        let new = (model.clone(), after.to_owned());
        if old != new && self.scoped.contains_key(&new) {
            return Err(kernel::KernelError::new(
                "rename destination designation already exists",
            ));
        }
        self.scoped.remove(&old);
        self.scoped.insert(new, id);
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
            .get(&old)
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("rename source role is unknown"))?;
        let new = (relation.clone(), after.to_owned());
        if old != new && self.roles.contains_key(&new) {
            return Err(kernel::KernelError::new(
                "rename destination role already exists",
            ));
        }
        self.roles.remove(&old);
        self.roles.insert(new, id);
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
            .get(&old)
            .cloned()
            .ok_or_else(|| kernel::KernelError::new("rename source pattern is unknown"))?;
        let new = (scope.clone(), after.to_owned());
        if old != new && self.patterns.contains_key(&new) {
            return Err(kernel::KernelError::new(
                "rename destination pattern already exists",
            ));
        }
        self.patterns.remove(&old);
        self.patterns.insert(new, id);
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

    pub(crate) fn proposal_path_presentation(&self, path: &ProposalPath) -> Vec<String> {
        path.segments()
            .iter()
            .map(|segment| match segment {
                ProposalPathSegment::ProductField(field) => self
                    .scoped
                    .iter()
                    .find_map(|((scope, name), candidate)| {
                        (candidate == field).then(|| {
                            self.global_name(scope)
                                .map(|scope| format!("{scope}.{name}"))
                                .unwrap_or_else(|| name.clone())
                        })
                    })
                    .unwrap_or_else(|| field.as_str().to_owned()),
                ProposalPathSegment::Role(role) => self
                    .roles
                    .iter()
                    .find_map(|((relation, name), candidate)| {
                        (candidate == role).then(|| {
                            self.global_name(relation)
                                .map(|relation| format!("{relation}.{name}"))
                                .unwrap_or_else(|| name.clone())
                        })
                    })
                    .unwrap_or_else(|| role.as_str().to_owned()),
                ProposalPathSegment::TupleIndex(index) => format!("tuple[{index}]"),
                ProposalPathSegment::SequenceIndex(index) => format!("sequence[{index}]"),
                ProposalPathSegment::SumPayload(tag) => format!("sum.{}", tag.as_str()),
                ProposalPathSegment::Application(content) => {
                    format!("application({})", content.as_str())
                }
            })
            .collect()
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

pub(super) fn derivation_rule_referent(
    model: &ReferentId,
    premises: &[ContentId],
    conclusions: &[ContentId],
) -> ReferentId {
    fn canonical(ids: &[ContentId]) -> Vec<&str> {
        let mut values = ids.iter().map(ContentId::as_str).collect::<Vec<_>>();
        values.sort_unstable();
        values.dedup();
        values
    }

    let premises = canonical(premises);
    let conclusions = canonical(conclusions);
    let mut preimage = b"clause-derivation-rule-v1\0".to_vec();
    write_field(&mut preimage, model.as_str());
    preimage.extend_from_slice(&(premises.len() as u64).to_be_bytes());
    for premise in premises {
        write_field(&mut preimage, premise);
    }
    preimage.extend_from_slice(&(conclusions.len() as u64).to_be_bytes());
    for conclusion in conclusions {
        write_field(&mut preimage, conclusion);
    }
    ReferentId::from_digest(sha256_digest(&preimage))
}

pub(super) fn universal_law_referent(
    model: &ReferentId,
    premises: &[ContentId],
    conclusions: &[ContentId],
) -> ReferentId {
    fn canonical(ids: &[ContentId]) -> Vec<&str> {
        let mut values = ids.iter().map(ContentId::as_str).collect::<Vec<_>>();
        values.sort_unstable();
        values.dedup();
        values
    }

    let premises = canonical(premises);
    let conclusions = canonical(conclusions);
    let mut preimage = b"clause-universal-law-v1\0".to_vec();
    write_field(&mut preimage, model.as_str());
    preimage.extend_from_slice(&(premises.len() as u64).to_be_bytes());
    for premise in premises {
        write_field(&mut preimage, premise);
    }
    preimage.extend_from_slice(&(conclusions.len() as u64).to_be_bytes());
    for conclusion in conclusions {
        write_field(&mut preimage, conclusion);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn populated_table() -> (DesignationTable, ReferentId, ReferentId, ReferentId) {
        let mut table = DesignationTable::new();
        table.declare_global("Global source").unwrap();
        table.declare_global("Global destination").unwrap();
        let model = table.declare_model("Model").unwrap();
        table.declare_scoped(&model, "Scoped source").unwrap();
        table.declare_scoped(&model, "Scoped destination").unwrap();
        table.declare_literal("literal");
        let relation = table.declare_global("Relation").unwrap();
        table.declare_role(&relation, "role source").unwrap();
        table.declare_role(&relation, "role destination").unwrap();
        let scope = table.declare_global("Pattern scope").unwrap();
        table.declare_pattern(&scope, "pattern source").unwrap();
        table
            .declare_pattern(&scope, "pattern destination")
            .unwrap();
        (table, model, relation, scope)
    }

    fn assert_table_unchanged(actual: &DesignationTable, expected: &DesignationTable) {
        assert_eq!(actual.globals, expected.globals);
        assert_eq!(actual.scoped, expected.scoped);
        assert_eq!(actual.literals, expected.literals);
        assert_eq!(actual.models, expected.models);
        assert_eq!(actual.roles, expected.roles);
        assert_eq!(actual.patterns, expected.patterns);
    }

    #[test]
    fn occupied_global_rename_preserves_the_entire_table() {
        let (mut table, _, _, _) = populated_table();
        let source = table.global("Global source").unwrap();
        let destination = table.global("Global destination").unwrap();
        let before = table.clone();

        assert!(
            table
                .retain_global("Global source", "Global destination")
                .is_err()
        );

        assert_eq!(table.global("Global source").unwrap(), source);
        assert_eq!(table.global("Global destination").unwrap(), destination);
        assert_table_unchanged(&table, &before);
    }

    #[test]
    fn occupied_scoped_rename_preserves_the_entire_table() {
        let (mut table, model, _, _) = populated_table();
        let source = table.scoped(&model, "Scoped source").unwrap();
        let destination = table.scoped(&model, "Scoped destination").unwrap();
        let before = table.clone();

        assert!(
            table
                .retain_scoped(&model, "Scoped source", "Scoped destination")
                .is_err()
        );

        assert_eq!(table.scoped(&model, "Scoped source").unwrap(), source);
        assert_eq!(
            table.scoped(&model, "Scoped destination").unwrap(),
            destination
        );
        assert_table_unchanged(&table, &before);
    }

    #[test]
    fn occupied_role_rename_preserves_the_entire_table() {
        let (mut table, _, relation, _) = populated_table();
        let source = table.role(&relation, "role source").unwrap();
        let destination = table.role(&relation, "role destination").unwrap();
        let before = table.clone();

        assert!(
            table
                .retain_role(&relation, "role source", "role destination")
                .is_err()
        );

        assert_eq!(table.role(&relation, "role source").unwrap(), source);
        assert_eq!(
            table.role(&relation, "role destination").unwrap(),
            destination
        );
        assert_table_unchanged(&table, &before);
    }

    #[test]
    fn occupied_pattern_rename_preserves_the_entire_table() {
        let (mut table, _, _, scope) = populated_table();
        let source_key = (scope.clone(), "pattern source".to_owned());
        let destination_key = (scope.clone(), "pattern destination".to_owned());
        let source = table.patterns.get(&source_key).unwrap().clone();
        let destination = table.patterns.get(&destination_key).unwrap().clone();
        let before = table.clone();

        assert!(
            table
                .retain_pattern(&scope, "pattern source", "pattern destination")
                .is_err()
        );

        assert_eq!(table.patterns.get(&source_key), Some(&source));
        assert_eq!(table.patterns.get(&destination_key), Some(&destination));
        assert_table_unchanged(&table, &before);
    }

    #[test]
    fn successful_renames_preserve_ids() {
        let (mut table, model, relation, scope) = populated_table();
        let global = table.global("Global source").unwrap();
        let scoped = table.scoped(&model, "Scoped source").unwrap();
        let role = table.role(&relation, "role source").unwrap();
        let pattern_key = (scope.clone(), "pattern source".to_owned());
        let pattern = table.patterns.get(&pattern_key).unwrap().clone();

        table
            .retain_global("Global source", "Global source")
            .unwrap();
        table
            .retain_scoped(&model, "Scoped source", "Scoped source")
            .unwrap();
        table
            .retain_role(&relation, "role source", "role source")
            .unwrap();
        table
            .retain_pattern(&scope, "pattern source", "pattern source")
            .unwrap();
        table
            .retain_global("Global source", "Global renamed")
            .unwrap();
        table
            .retain_scoped(&model, "Scoped source", "Scoped renamed")
            .unwrap();
        table
            .retain_role(&relation, "role source", "role renamed")
            .unwrap();
        table
            .retain_pattern(&scope, "pattern source", "pattern renamed")
            .unwrap();

        assert!(table.global("Global source").is_err());
        assert_eq!(table.global("Global renamed").unwrap(), global);
        assert!(table.scoped(&model, "Scoped source").is_err());
        assert_eq!(table.scoped(&model, "Scoped renamed").unwrap(), scoped);
        assert!(table.role(&relation, "role source").is_err());
        assert_eq!(table.role(&relation, "role renamed").unwrap(), role);
        assert!(!table.patterns.contains_key(&pattern_key));
        assert_eq!(
            table.patterns.get(&(scope, "pattern renamed".to_owned())),
            Some(&pattern)
        );
    }
}
