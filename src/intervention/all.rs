//! Exhaustive finite intervention frontier searches.

use super::{
    AchieveAll, Incomplete, Intervention, InterventionLimits, PreventAll,
    basis::{achievement_basis, withdrawal_basis},
    closure::{apply, complete_closure, incomplete_support, is_closure_limit},
    search::{AllState, Enumeration, enumerate, is_subset},
};
use crate::{
    derive,
    kernel::{KernelError, ReferentId, RelationalContent, Result, Revision},
};

/// Enumerate every inclusion-minimal withdrawal over the complete support
/// frontier. An incomplete support projection is never treated as exact.
pub(super) fn prevent_all_minimal(
    source: &Revision,
    target: RelationalContent,
    using: Vec<ReferentId>,
    limits: InterventionLimits,
) -> Result<PreventAll> {
    source.model().validate_content(&target, false)?;
    let basis = withdrawal_basis(source, using)?;
    let frontier = match derive::support_frontier(source, &target, limits.support()) {
        Ok(frontier) => frontier,
        Err(error) if is_closure_limit(&error) => {
            return Ok(PreventAll::Incomplete {
                interventions: Vec::new(),
                reason: Incomplete::ClosureBudgetExhausted,
            });
        }
        Err(error) => return Err(error),
    };
    if !frontier.status().is_complete() {
        return Ok(PreventAll::Incomplete {
            interventions: Vec::new(),
            reason: incomplete_support(frontier.status()),
        });
    }
    if frontier.supports().is_empty() {
        return Ok(PreventAll::AlreadyAbsent);
    }
    let supports = frontier
        .supports()
        .iter()
        .map(|support| support.assertions().to_vec())
        .collect::<Vec<_>>();
    if basis.is_empty()
        || supports
            .iter()
            .any(|support| !support.iter().any(|item| basis.binary_search(item).is_ok()))
    {
        return Ok(PreventAll::Impossible);
    }
    let mut state = AllState::new();
    for size in 1..=basis.len() {
        let mut choice = Vec::new();
        let control = enumerate(&basis, size, 0, &mut choice, &mut |withdrawals| {
            if state.reason.is_some() {
                return Ok(Enumeration::Break);
            }
            if state.checked >= limits.max_candidates() {
                state.reason = Some(Incomplete::CandidateBudgetExhausted);
                return Ok(Enumeration::Break);
            }
            state.checked += 1;
            if state
                .items
                .iter()
                .any(|item| is_subset(item.withdrawals(), withdrawals))
            {
                return Ok(Enumeration::Continue);
            }
            if !supports.iter().all(|support| {
                support
                    .iter()
                    .any(|item| withdrawals.binary_search(item).is_ok())
            }) {
                return Ok(Enumeration::Continue);
            }
            let revision = apply(source, Vec::new(), withdrawals.to_vec())?;
            let Some(closure) = complete_closure(&revision, limits.closure())? else {
                state.reason = Some(Incomplete::ClosureBudgetExhausted);
                return Ok(Enumeration::Break);
            };
            if closure.proof(&target).is_some() {
                return Err(KernelError::new(
                    "support hitting set did not prevent target",
                ));
            }
            if state.items.len() >= limits.max_solutions() {
                state.reason = Some(Incomplete::SolutionBudgetExhausted);
                return Ok(Enumeration::Break);
            }
            state.items.push(Intervention::withdrawal(
                source,
                withdrawals.to_vec(),
                revision,
            )?);
            Ok(Enumeration::Continue)
        })?;
        if control == Enumeration::Break {
            break;
        }
    }
    Ok(state.prevent_result())
}

/// Enumerate every inclusion-minimal addition over the finite typed basis.
pub(super) fn achieve_all_minimal(
    source: &Revision,
    target: RelationalContent,
    using: Vec<ReferentId>,
    limits: InterventionLimits,
) -> Result<AchieveAll> {
    source.model().validate_content(&target, false)?;
    let Some(source_closure) = complete_closure(source, limits.closure())? else {
        return Ok(AchieveAll::Incomplete {
            interventions: Vec::new(),
            reason: Incomplete::ClosureBudgetExhausted,
        });
    };
    if source_closure.proof(&target).is_some() {
        return Ok(AchieveAll::AlreadyEntailed);
    }
    let basis = achievement_basis(source, using, limits.max_candidates())?;
    if !basis.complete {
        return Ok(AchieveAll::Incomplete {
            interventions: Vec::new(),
            reason: Incomplete::CandidateBudgetExhausted,
        });
    }
    let mut state = AllState::new();
    for size in 1..=basis.clauses.len() {
        let mut choice = Vec::new();
        let control = enumerate(&basis.clauses, size, 0, &mut choice, &mut |additions| {
            if state.reason.is_some() {
                return Ok(Enumeration::Break);
            }
            if state.checked >= limits.max_candidates() {
                state.reason = Some(Incomplete::CandidateBudgetExhausted);
                return Ok(Enumeration::Break);
            }
            state.checked += 1;
            if state
                .items
                .iter()
                .any(|item| is_subset(item.admissions(), additions))
            {
                return Ok(Enumeration::Continue);
            }
            let revision = apply(source, additions.to_vec(), Vec::new())?;
            let Some(closure) = complete_closure(&revision, limits.closure())? else {
                state.reason = Some(Incomplete::ClosureBudgetExhausted);
                return Ok(Enumeration::Break);
            };
            let Some(proof) = closure.proof(&target).cloned() else {
                return Ok(Enumeration::Continue);
            };
            if state.items.len() >= limits.max_solutions() {
                state.reason = Some(Incomplete::SolutionBudgetExhausted);
                return Ok(Enumeration::Break);
            }
            state.items.push(Intervention::admission(
                source,
                additions.to_vec(),
                revision,
                proof,
            )?);
            Ok(Enumeration::Continue)
        })?;
        if control == Enumeration::Break {
            break;
        }
    }
    if state.items.is_empty() && state.reason.is_none() {
        return Ok(AchieveAll::Impossible);
    }
    Ok(state.achieve_result())
}
