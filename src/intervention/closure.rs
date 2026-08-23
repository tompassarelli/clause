//! Revision application and bounded closure evaluation shared by searches.

use super::Incomplete;
use crate::{
    derive::{self, Closure, Limits, SupportStatus},
    kernel::{Clause, Delta, KernelError, Result, Revision},
};

pub(super) fn apply(
    source: &Revision,
    admissions: Vec<Clause>,
    withdrawals: Vec<Clause>,
) -> Result<Revision> {
    if admissions.is_empty() && withdrawals.is_empty() {
        return Ok(source.clone());
    }
    Delta::new(source.identity().clone(), admissions, withdrawals)?.apply(source)
}

pub(super) fn complete_closure(revision: &Revision, limits: Limits) -> Result<Option<Closure>> {
    match derive::saturate(revision, limits) {
        Ok(closure) => Ok(Some(closure)),
        Err(error) if is_closure_limit(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) enum ClosureAttempt {
    Complete(Closure),
    CandidateBudget,
    ClosureBudget,
}

pub(super) fn closure_after(
    source: &Revision,
    admissions: &[Clause],
    withdrawals: &[Clause],
    limits: Limits,
    checks: &mut usize,
    max_checks: usize,
) -> Result<ClosureAttempt> {
    if *checks >= max_checks {
        return Ok(ClosureAttempt::CandidateBudget);
    }
    *checks += 1;
    let revision = apply(source, admissions.to_vec(), withdrawals.to_vec())?;
    Ok(match complete_closure(&revision, limits)? {
        Some(closure) => ClosureAttempt::Complete(closure),
        None => ClosureAttempt::ClosureBudget,
    })
}

pub(super) fn is_closure_limit(error: &KernelError) -> bool {
    error.to_string().starts_with("closure ")
}

pub(super) fn incomplete_support(status: SupportStatus) -> Incomplete {
    match status {
        SupportStatus::Complete => {
            unreachable!("complete support status has no incomplete projection")
        }
        SupportStatus::ExpansionBudgetExhausted => Incomplete::SupportExpansionBudgetExhausted,
        SupportStatus::SupportBudgetExhausted => Incomplete::SupportBudgetExhausted,
    }
}
