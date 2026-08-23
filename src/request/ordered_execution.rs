use super::{Request, RequestOutput, ResolvedProgram, RunLimits, RunOutput, Selection};
use crate::{
    execution, intervention,
    kernel::{self, Revision, RevisionId},
    semantic_diff::SemanticDiff,
};

pub(super) fn run(program: &ResolvedProgram, limits: RunLimits) -> kernel::Result<RunOutput> {
    let mut results = Vec::with_capacity(program.requests.len());
    for request in &program.requests {
        let output = match request {
            Request::Find {
                revision: identity,
                pattern,
                sought,
            } => {
                let selected = revision(program, identity)?;
                RequestOutput::Find(execution::find(
                    selected,
                    &kernel::FindPlan::new(selected.model(), pattern, sought.clone())?,
                    limits.closure,
                )?)
            }
            Request::Why {
                revision: identity,
                target,
                all: false,
            } => RequestOutput::WhyOne(execution::why(
                revision(program, identity)?,
                target,
                limits.closure,
            )?),
            Request::Why {
                revision: identity,
                target,
                all: true,
            } => RequestOutput::WhyAll(execution::why_all(
                revision(program, identity)?,
                target,
                limits.support,
            )?),
            Request::Prevent {
                revision: identity,
                target,
                selection: Selection::OneMinimal,
                using,
            } => RequestOutput::PreventOne(intervention::prevent_one_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Prevent {
                revision: identity,
                target,
                selection: Selection::AllMinimal,
                using,
            } => RequestOutput::PreventAll(intervention::prevent_all_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Achieve {
                revision: identity,
                target,
                selection: Selection::OneMinimal,
                using,
            } => RequestOutput::AchieveOne(intervention::achieve_one_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Achieve {
                revision: identity,
                target,
                selection: Selection::AllMinimal,
                using,
            } => RequestOutput::AchieveAll(intervention::achieve_all_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Diff { base, successor } => RequestOutput::Diff(SemanticDiff::between(
                revision(program, base)?,
                revision(program, successor)?,
                limits.support,
            )?),
        };
        results.push(output);
    }
    Ok(RunOutput { results })
}

fn revision<'a>(
    program: &'a ResolvedProgram,
    identity: &RevisionId,
) -> kernel::Result<&'a Revision> {
    program
        .revisions
        .get(identity)
        .ok_or_else(|| kernel::KernelError::new("request Revision is unavailable"))
}
