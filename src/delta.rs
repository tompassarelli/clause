//! Immutable, completely signed semantic transitions and Revision comparisons.

use crate::{
    kernel::{
        AssertionOccurrence, Delta, Judgment, JudgmentKind, JudgmentStatus, JudgmentTarget,
        KernelError, Referent, ReferentId, RelationalContent, Result, Revision, RevisionId,
        SemanticAtom,
    },
    wire,
};
use std::collections::BTreeSet;

impl Delta {
    /// Apply every signed atom atomically to this exact predecessor.
    pub fn apply(&self, base: &Revision) -> Result<Revision> {
        if self.base() != base.identity() {
            return Err(KernelError::new("delta base revision does not match"));
        }
        let mut atoms = base.model().atoms();
        for withdrawal in self.withdrawals() {
            if !atoms.remove(withdrawal) {
                return Err(KernelError::new(
                    "delta withdraws a nonexistent semantic atom",
                ));
            }
        }
        for admission in self.admissions() {
            if !atoms.insert(admission.clone()) {
                return Err(KernelError::new("delta admits an existing semantic atom"));
            }
        }
        let successor = crate::kernel::Model::from_atoms(base.model().id().clone(), atoms)?;
        wire::admit_successor(base, successor, self.clone())
    }
}

/// Exact signed and admitted-content differences across one lineage edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionDiff {
    base_revision: RevisionId,
    successor_revision: RevisionId,
    admitted_atoms: Vec<SemanticAtom>,
    withdrawn_atoms: Vec<SemanticAtom>,
    added: Vec<RelationalContent>,
    removed: Vec<RelationalContent>,
}

impl RevisionDiff {
    pub fn between(base: &Revision, successor: &Revision) -> Result<Self> {
        if successor.predecessor() != Some(base.identity()) {
            return Err(KernelError::new(
                "cannot diff revisions without an exact predecessor edge",
            ));
        }
        let declared = successor
            .delta()
            .ok_or_else(|| KernelError::new("successor Revision has no signed Delta"))?;
        let expected = exact_delta(base, successor)?;
        if declared != &expected {
            return Err(KernelError::new(
                "Revision lineage Delta does not account for its complete semantic difference",
            ));
        }
        let added = content_difference(
            successor.model().admitted_contents(),
            base.model().admitted_contents(),
        );
        let removed = content_difference(
            base.model().admitted_contents(),
            successor.model().admitted_contents(),
        );
        Ok(Self {
            base_revision: base.identity().clone(),
            successor_revision: successor.identity().clone(),
            admitted_atoms: declared.admissions().to_vec(),
            withdrawn_atoms: declared.withdrawals().to_vec(),
            added,
            removed,
        })
    }

    pub fn base_revision(&self) -> &RevisionId {
        &self.base_revision
    }
    pub fn successor_revision(&self) -> &RevisionId {
        &self.successor_revision
    }
    pub fn admitted_atoms(&self) -> &[SemanticAtom] {
        &self.admitted_atoms
    }
    pub fn withdrawn_atoms(&self) -> &[SemanticAtom] {
        &self.withdrawn_atoms
    }
    pub fn added(&self) -> &[RelationalContent] {
        &self.added
    }
    pub fn removed(&self) -> &[RelationalContent] {
        &self.removed
    }
}

pub(crate) fn exact_delta(base: &Revision, successor: &Revision) -> Result<Delta> {
    if base.model().id() != successor.model().id() {
        return Err(KernelError::new("a Delta cannot change Model identity"));
    }
    let base_atoms = base.model().atoms();
    let successor_atoms = successor.model().atoms();
    let admissions = successor_atoms.difference(&base_atoms).cloned().collect();
    let withdrawals = base_atoms.difference(&successor_atoms).cloned().collect();
    Delta::new(base.identity().clone(), admissions, withdrawals)
}

/// Project legacy assertion-content intervention intent into explicit content,
/// occurrence, and admission-judgment atoms.
pub(crate) fn content_delta(
    base: &Revision,
    admissions: Vec<RelationalContent>,
    withdrawals: Vec<RelationalContent>,
) -> Result<Delta> {
    if admissions.is_empty() && withdrawals.is_empty() {
        return Err(KernelError::new("content Delta has no changes"));
    }
    let source = stable_referent(&format!(
        "{}/intervention-source",
        base.model().id().as_str()
    ));
    let policy = stable_referent(&format!(
        "{}/intervention-policy",
        base.model().id().as_str()
    ));
    let mut added = Vec::new();
    let atoms = base.model().atoms();
    for id in [&source, &policy] {
        let atom = SemanticAtom::Referent(Referent::new(id.clone()));
        if !atoms.contains(&atom) {
            added.push(atom);
        }
    }
    for content in admissions {
        let content_atom = SemanticAtom::RelationalContent(content.clone());
        if !atoms.contains(&content_atom) {
            added.push(content_atom);
        }
        let occurrence_id = stable_referent(&format!(
            "{}/occurrence/{}",
            base.identity(),
            content.id().as_str()
        ));
        let judgment_id = stable_referent(&format!(
            "{}/judgment/{}",
            base.identity(),
            content.id().as_str()
        ));
        added.push(SemanticAtom::Referent(Referent::new(occurrence_id.clone())));
        added.push(SemanticAtom::Referent(Referent::new(judgment_id.clone())));
        let occurrence = AssertionOccurrence::new(
            occurrence_id.clone(),
            content.id().clone(),
            source.clone(),
            base.model().id().clone(),
        );
        added.push(SemanticAtom::AssertionOccurrence(occurrence));
        added.push(SemanticAtom::Judgment(Judgment::new(
            judgment_id,
            base.model().id().clone(),
            base.model().id().clone(),
            JudgmentTarget::Occurrence(occurrence_id),
            JudgmentKind::Admitted {
                policy: policy.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        )));
    }

    let withdrawal_ids = withdrawals
        .iter()
        .map(RelationalContent::id)
        .collect::<BTreeSet<_>>();
    let occurrence_ids = base
        .model()
        .occurrences()
        .iter()
        .filter(|item| withdrawal_ids.contains(item.content()))
        .map(|item| item.id().clone())
        .collect::<BTreeSet<_>>();
    let mut removed = base
        .model()
        .occurrences()
        .iter()
        .filter(|item| occurrence_ids.contains(item.id()))
        .cloned()
        .map(SemanticAtom::AssertionOccurrence)
        .collect::<Vec<_>>();
    removed.extend(
        base.model()
            .judgments()
            .iter()
            .filter(|judgment| match judgment.target() {
                JudgmentTarget::Content(id) => withdrawal_ids.contains(id),
                JudgmentTarget::Occurrence(id) => occurrence_ids.contains(id),
            })
            .cloned()
            .map(SemanticAtom::Judgment),
    );
    Delta::new(base.identity().clone(), added, removed)
}

fn stable_referent(value: &str) -> ReferentId {
    ReferentId::from_digest(crate::wire::sha256_digest(value.as_bytes()))
}

fn content_difference(
    left: &[RelationalContent],
    right: &[RelationalContent],
) -> Vec<RelationalContent> {
    left.iter()
        .filter(|item| right.binary_search(item).is_err())
        .cloned()
        .collect()
}
