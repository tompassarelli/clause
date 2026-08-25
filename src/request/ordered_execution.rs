use super::{
    QuerySelection, Request, RequestOutput, ResolvedProgram, RevisionOutput, RunLimits, RunOutput,
    RunResult, Selection, any_plan, select_plan,
};
use crate::{
    execution, intervention,
    kernel::{self, Revision, RevisionId},
    semantic_diff::SemanticDiff,
};

pub(super) fn run(program: &ResolvedProgram, limits: RunLimits) -> kernel::Result<RunOutput> {
    let mut results = Vec::with_capacity(program.requests.len());
    for request in &program.requests {
        let (producer, result) = match request {
            Request::Any {
                revision: identity,
                pattern,
                dependencies,
            } => {
                let selected = revision(program, identity)?;
                let plan = any_plan(selected.model(), pattern, dependencies)?;
                (
                    selected,
                    RequestOutput::Any(execution::any(selected, &plan, limits.closure)?),
                )
            }
            Request::Select {
                revision: identity,
                pattern,
                dependencies,
                columns,
                selection,
            } => {
                let selected = revision(program, identity)?;
                let plan = select_plan(selected.model(), pattern, dependencies, columns)?;
                let mut rows =
                    execution::select_projected(selected, &plan, columns.len(), limits.closure)?;
                let output = match selection {
                    QuerySelection::All => RequestOutput::Select {
                        columns: columns.clone(),
                        rows,
                    },
                    QuerySelection::ExactlyOne if rows.len() == 1 => RequestOutput::SelectOne {
                        columns: columns.clone(),
                        rows,
                    },
                    QuerySelection::ExactlyOne => {
                        return Err(kernel::KernelError::new(format!(
                            "select one requires exactly one row, found {}",
                            rows.len()
                        )));
                    }
                    QuerySelection::CanonicalFirst => {
                        rows.truncate(1);
                        RequestOutput::SelectFirst {
                            columns: columns.clone(),
                            rows,
                        }
                    }
                };
                (selected, output)
            }
            Request::Find {
                revision: identity,
                pattern,
                sought,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::Find(execution::find(
                        selected,
                        &kernel::FindPlan::new(selected.model(), pattern, sought.clone())?,
                        limits.closure,
                    )?),
                )
            }
            Request::Why {
                revision: identity,
                target,
                all: false,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::WhyOne(execution::why(selected, target, limits.closure)?),
                )
            }
            Request::Why {
                revision: identity,
                target,
                all: true,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::WhyAll(execution::why_all(selected, target, limits.support)?),
                )
            }
            Request::Prevent {
                revision: identity,
                target,
                selection: Selection::OneMinimal,
                using,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::PreventOne(intervention::prevent_one_minimal(
                        selected,
                        target.clone(),
                        using.clone(),
                        limits.intervention,
                    )?),
                )
            }
            Request::Prevent {
                revision: identity,
                target,
                selection: Selection::AllMinimal,
                using,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::PreventAll(intervention::prevent_all_minimal(
                        selected,
                        target.clone(),
                        using.clone(),
                        limits.intervention,
                    )?),
                )
            }
            Request::Achieve {
                revision: identity,
                target,
                selection: Selection::OneMinimal,
                using,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::AchieveOne(intervention::achieve_one_minimal(
                        selected,
                        target.clone(),
                        using.clone(),
                        limits.intervention,
                    )?),
                )
            }
            Request::Achieve {
                revision: identity,
                target,
                selection: Selection::AllMinimal,
                using,
            } => {
                let selected = revision(program, identity)?;
                (
                    selected,
                    RequestOutput::AchieveAll(intervention::achieve_all_minimal(
                        selected,
                        target.clone(),
                        using.clone(),
                        limits.intervention,
                    )?),
                )
            }
            Request::Diff { base, successor } => {
                results.push(RunResult::Diff(SemanticDiff::between(
                    revision(program, base)?,
                    revision(program, successor)?,
                    limits.support,
                )?));
                continue;
            }
        };
        results.push(RunResult::Revision(RevisionOutput::produced_by(
            producer, result,
        )));
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
