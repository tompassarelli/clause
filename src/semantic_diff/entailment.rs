//! Derived-consequence comparison, kept separate from authored assertions.

use crate::{delta::RevisionDiff, derive::Closure, kernel::RelationalContent};

pub(super) fn changes(
    base: &Closure,
    successor: &Closure,
    authored: &RevisionDiff,
) -> (Vec<RelationalContent>, Vec<RelationalContent>) {
    let added = successor
        .contents()
        .iter()
        .filter(|consequence| {
            base.contents().binary_search(consequence).is_err()
                && authored.added().binary_search(consequence).is_err()
        })
        .cloned()
        .collect();
    let removed = base
        .contents()
        .iter()
        .filter(|consequence| {
            successor.contents().binary_search(consequence).is_err()
                && authored.removed().binary_search(consequence).is_err()
        })
        .cloned()
        .collect();
    (added, removed)
}
