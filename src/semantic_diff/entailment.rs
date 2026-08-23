//! Derived-consequence comparison, kept separate from authored assertions.

use crate::{delta::RevisionDiff, derive::Closure, kernel::Clause};

pub(super) fn changes(
    base: &Closure,
    successor: &Closure,
    authored: &RevisionDiff,
) -> (Vec<Clause>, Vec<Clause>) {
    let added = successor
        .assertions()
        .iter()
        .filter(|consequence| {
            base.assertions().binary_search(consequence).is_err()
                && authored.added().binary_search(consequence).is_err()
        })
        .cloned()
        .collect();
    let removed = base
        .assertions()
        .iter()
        .filter(|consequence| {
            successor.assertions().binary_search(consequence).is_err()
                && authored.removed().binary_search(consequence).is_err()
        })
        .cloned()
        .collect();
    (added, removed)
}
