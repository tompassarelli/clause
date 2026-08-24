//! Certified single inclusion-minimal intervention searches.

use super::{
    AchieveOne, Incomplete, Intervention, InterventionLimits, PreventOne,
    basis::{achievement_basis, withdrawal_basis},
    closure::{ClosureAttempt, apply, closure_after, complete_closure},
    search::without,
};
use crate::kernel::{KernelError, ReferentId, RelationalContent, Result, Revision};

/// Return one canonical inclusion-minimal asserted-clause withdrawal.
///
/// The deletion/restoration algorithm is valid only because the admitted law
/// fragment is positive and monotone. It proves each retained withdrawal is
/// necessary, but intentionally does not prove it has minimum cardinality.
pub(super) fn prevent_one_minimal(
    source: &Revision,
    target: RelationalContent,
    using: Vec<ReferentId>,
    limits: InterventionLimits,
) -> Result<PreventOne> {
    source.model().validate_content(&target, false)?;
    let Some(source_closure) = complete_closure(source, limits.closure())? else {
        return Ok(PreventOne::Incomplete(Incomplete::ClosureBudgetExhausted));
    };
    if source_closure.proof(&target).is_none() {
        return Ok(PreventOne::AlreadyAbsent);
    }
    let basis = withdrawal_basis(source, using)?;
    if basis.is_empty() {
        return Ok(PreventOne::Impossible);
    }
    let mut checks = 0;
    match closure_after(
        source,
        &[],
        &basis,
        limits.closure(),
        &mut checks,
        limits.max_candidates(),
    )? {
        ClosureAttempt::CandidateBudget => {
            return Ok(PreventOne::Incomplete(Incomplete::CandidateBudgetExhausted));
        }
        ClosureAttempt::ClosureBudget => {
            return Ok(PreventOne::Incomplete(Incomplete::ClosureBudgetExhausted));
        }
        ClosureAttempt::Complete(closure) if closure.proof(&target).is_some() => {
            return Ok(PreventOne::Impossible);
        }
        ClosureAttempt::Complete(_) => {}
    }

    let mut withdrawals = basis;
    for candidate in withdrawals.clone() {
        let restored = without(&withdrawals, &candidate);
        let closure = match closure_after(
            source,
            &[],
            &restored,
            limits.closure(),
            &mut checks,
            limits.max_candidates(),
        )? {
            ClosureAttempt::CandidateBudget => {
                return Ok(PreventOne::Incomplete(Incomplete::CandidateBudgetExhausted));
            }
            ClosureAttempt::ClosureBudget => {
                return Ok(PreventOne::Incomplete(Incomplete::ClosureBudgetExhausted));
            }
            ClosureAttempt::Complete(closure) => closure,
        };
        if closure.proof(&target).is_none() {
            withdrawals = restored;
        }
    }
    let revision = apply(source, Vec::new(), withdrawals.clone())?;
    let Some(closure) = complete_closure(&revision, limits.closure())? else {
        return Ok(PreventOne::Incomplete(Incomplete::ClosureBudgetExhausted));
    };
    if closure.proof(&target).is_some() {
        return Err(KernelError::new(
            "prevent minimizer returned an entailed target",
        ));
    }
    Ok(PreventOne::Satisfied(Box::new(Intervention::withdrawal(
        source,
        withdrawals,
        revision,
    )?)))
}

/// Return one canonical inclusion-minimal asserted-clause admission.
///
/// This is the dual of [`super::achieve_one_minimal`]. It proves subset
/// necessity, not cardinality optimality or complete-frontier enumeration.
pub(super) fn achieve_one_minimal(
    source: &Revision,
    target: RelationalContent,
    using: Vec<ReferentId>,
    limits: InterventionLimits,
) -> Result<AchieveOne> {
    source.model().validate_content(&target, false)?;
    let Some(source_closure) = complete_closure(source, limits.closure())? else {
        return Ok(AchieveOne::Incomplete(Incomplete::ClosureBudgetExhausted));
    };
    if source_closure.proof(&target).is_some() {
        return Ok(AchieveOne::AlreadyEntailed);
    }
    let basis = achievement_basis(source, using, limits.max_candidates())?;
    if !basis.complete {
        return Ok(AchieveOne::Incomplete(Incomplete::CandidateBudgetExhausted));
    }
    let mut additions = basis.clauses;
    if additions.is_empty() {
        return Ok(AchieveOne::Impossible);
    }
    let mut checks = 0;
    let full = match closure_after(
        source,
        &additions,
        &[],
        limits.closure(),
        &mut checks,
        limits.max_candidates(),
    )? {
        ClosureAttempt::CandidateBudget => {
            return Ok(AchieveOne::Incomplete(Incomplete::CandidateBudgetExhausted));
        }
        ClosureAttempt::ClosureBudget => {
            return Ok(AchieveOne::Incomplete(Incomplete::ClosureBudgetExhausted));
        }
        ClosureAttempt::Complete(closure) => closure,
    };
    if full.proof(&target).is_none() {
        return Ok(AchieveOne::Impossible);
    }
    for candidate in additions.clone() {
        let reduced = without(&additions, &candidate);
        let closure = match closure_after(
            source,
            &reduced,
            &[],
            limits.closure(),
            &mut checks,
            limits.max_candidates(),
        )? {
            ClosureAttempt::CandidateBudget => {
                return Ok(AchieveOne::Incomplete(Incomplete::CandidateBudgetExhausted));
            }
            ClosureAttempt::ClosureBudget => {
                return Ok(AchieveOne::Incomplete(Incomplete::ClosureBudgetExhausted));
            }
            ClosureAttempt::Complete(closure) => closure,
        };
        if closure.proof(&target).is_some() {
            additions = reduced;
        }
    }
    let revision = apply(source, additions.clone(), Vec::new())?;
    let Some(closure) = complete_closure(&revision, limits.closure())? else {
        return Ok(AchieveOne::Incomplete(Incomplete::ClosureBudgetExhausted));
    };
    let proof = closure
        .proof(&target)
        .cloned()
        .ok_or_else(|| KernelError::new("achieve minimizer returned an absent target"))?;
    Ok(AchieveOne::Satisfied(Box::new(Intervention::admission(
        source, additions, revision, proof,
    )?)))
}
