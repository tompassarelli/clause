//! Canonical finite subset enumeration and all-result search state.

use super::{AchieveAll, Incomplete, Intervention, PreventAll};
use crate::kernel::{Clause, Result};

pub(super) fn without(items: &[Clause], removed: &Clause) -> Vec<Clause> {
    items
        .iter()
        .filter(|item| *item != removed)
        .cloned()
        .collect()
}

pub(super) fn is_subset(left: &[Clause], right: &[Clause]) -> bool {
    left.iter().all(|item| right.binary_search(item).is_ok())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Enumeration {
    Continue,
    Break,
}

pub(super) fn enumerate<F>(
    basis: &[Clause],
    remaining: usize,
    start: usize,
    choice: &mut Vec<Clause>,
    visit: &mut F,
) -> Result<Enumeration>
where
    F: FnMut(&[Clause]) -> Result<Enumeration>,
{
    if remaining == 0 {
        return visit(choice);
    }
    for index in start..=basis.len() - remaining {
        choice.push(basis[index].clone());
        let control = enumerate(basis, remaining - 1, index + 1, choice, visit)?;
        choice.pop();
        if control == Enumeration::Break {
            return Ok(Enumeration::Break);
        }
    }
    Ok(Enumeration::Continue)
}

pub(super) struct AllState {
    pub(super) checked: usize,
    pub(super) items: Vec<Intervention>,
    pub(super) reason: Option<Incomplete>,
}

impl AllState {
    pub(super) fn new() -> Self {
        Self {
            checked: 0,
            items: Vec::new(),
            reason: None,
        }
    }

    pub(super) fn prevent_result(self) -> PreventAll {
        match self.reason {
            Some(reason) => PreventAll::Incomplete {
                interventions: self.items,
                reason,
            },
            None => PreventAll::Complete(self.items),
        }
    }

    pub(super) fn achieve_result(self) -> AchieveAll {
        match self.reason {
            Some(reason) => AchieveAll::Incomplete {
                interventions: self.items,
                reason,
            },
            None => AchieveAll::Complete(self.items),
        }
    }
}
