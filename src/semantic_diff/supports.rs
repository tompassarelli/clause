//! Minimal-support frontier comparison with explicit bounded-search semantics.

use std::collections::BTreeSet;

use crate::{
    delta::RevisionDiff,
    derive::{self, Closure, Support, SupportLimits},
    kernel::{RelationalContent, Result, Revision},
};

use super::SupportChange;

pub(super) fn changes(
    base_revision: &Revision,
    successor_revision: &Revision,
    base_closure: &Closure,
    successor_closure: &Closure,
    authored: &RevisionDiff,
    limits: SupportLimits,
) -> Result<Vec<SupportChange>> {
    base_closure
        .contents()
        .iter()
        .chain(successor_closure.contents())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        // Assertion deltas are their own layer. Repeating a directly admitted
        // or withdrawn clause as a one-clause support change would duplicate
        // authored history in the semantic layer.
        .filter(|consequence| {
            authored.added().binary_search(consequence).is_err()
                && authored.removed().binary_search(consequence).is_err()
        })
        .map(|consequence| support_change(base_revision, successor_revision, &consequence, limits))
        .collect::<Result<Vec<_>>>()
        .map(|changes| changes.into_iter().flatten().collect())
}

fn support_change(
    base_revision: &Revision,
    successor_revision: &Revision,
    consequence: &RelationalContent,
    limits: SupportLimits,
) -> Result<Option<SupportChange>> {
    let base = derive::support_frontier(base_revision, consequence, limits)?;
    let successor = derive::support_frontier(successor_revision, consequence, limits)?;
    let retained = base
        .supports()
        .iter()
        .filter(|support| {
            successor
                .supports()
                .iter()
                .any(|candidate| candidate.assertion_key() == support.assertion_key())
        })
        .cloned()
        .collect();
    // A gain is exact only if the base frontier proved that support absent.
    let added: Vec<Support> = if base.status().is_complete() {
        successor
            .supports()
            .iter()
            .filter(|support| {
                !base
                    .supports()
                    .iter()
                    .any(|candidate| candidate.assertion_key() == support.assertion_key())
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    // A loss is exact only if the successor frontier proved that support absent.
    let removed: Vec<Support> = if successor.status().is_complete() {
        base.supports()
            .iter()
            .filter(|support| {
                !successor
                    .supports()
                    .iter()
                    .any(|candidate| candidate.assertion_key() == support.assertion_key())
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // An incomplete projection with no observed delta is unknown, not a
    // changed support frontier. Emit a change only when a gain or loss is
    // positively witnessed; frontier statuses retain the exact bounds.
    if added.is_empty() && removed.is_empty() {
        return Ok(None);
    }

    Ok(Some(SupportChange {
        consequence: consequence.clone(),
        base,
        successor,
        added,
        removed,
        retained,
    }))
}
