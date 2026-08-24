use std::collections::BTreeMap;

use super::{QueryColumn, Request, ResolvedProgram, Selection, any_plan, projected_plan};
use crate::{
    elaborate::{self, CompiledProgram},
    frontend,
    kernel::{self, ReferentId},
};

pub(super) fn resolve(program: &CompiledProgram) -> kernel::Result<ResolvedProgram> {
    let mut revisions = BTreeMap::new();
    let mut requests = Vec::with_capacity(program.requests().len());
    for (index, request) in program.requests().iter().enumerate() {
        let resolved = match request {
            frontend::RequestDecl::Any {
                revision, pattern, ..
            } => {
                let revision = program.revision(&revision.value)?;
                let pattern = program.lower_request_clause(index, revision, pattern)?;
                let _ = any_plan(revision.model(), &pattern)?;
                Request::Any {
                    revision: revision.identity().clone(),
                    pattern,
                }
            }
            frontend::RequestDecl::Select {
                revision,
                pattern,
                columns,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                let pattern = program.lower_request_clause(index, revision, pattern)?;
                let columns = columns
                    .iter()
                    .map(|column| {
                        Ok((
                            column.label.as_ref().map(|label| label.0.clone()),
                            program.request_column(index, column)?,
                        ))
                    })
                    .collect::<kernel::Result<Vec<_>>>()?;
                let projected_count = columns.len();
                let plan = projected_plan(
                    revision.model(),
                    &pattern,
                    columns.iter().map(|(_, binder)| binder.clone()).collect(),
                )?;
                let columns = columns
                    .into_iter()
                    .zip(&plan.columns()[..projected_count])
                    .map(|((label, binder), column)| {
                        debug_assert_eq!(&binder, column.binder());
                        QueryColumn::new(label, binder, column.origins().to_vec())
                    })
                    .collect();
                Request::Select {
                    revision: revision.identity().clone(),
                    pattern,
                    columns,
                }
            }
            frontend::RequestDecl::Find {
                revision,
                pattern,
                sought,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                let pattern = program.lower_request_clause(index, revision, pattern)?;
                let sought = program.request_pattern(index, &sought.value)?;
                let _ = kernel::FindPlan::new(revision.model(), &pattern, sought.clone())?;
                Request::Find {
                    revision: revision.identity().clone(),
                    pattern,
                    sought,
                }
            }
            frontend::RequestDecl::Why {
                revision,
                target,
                all,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                Request::Why {
                    revision: revision.identity().clone(),
                    target: elaborate::lower_clause(program, revision, target)?,
                    all: *all,
                }
            }
            frontend::RequestDecl::Prevent {
                revision,
                target,
                selection: requested_selection,
                using,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                Request::Prevent {
                    revision: revision.identity().clone(),
                    target: elaborate::lower_clause(program, revision, target)?,
                    selection: lower_selection(*requested_selection),
                    using: relations(program, revision, using)?,
                }
            }
            frontend::RequestDecl::Achieve {
                revision,
                target,
                selection: requested_selection,
                using,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                Request::Achieve {
                    revision: revision.identity().clone(),
                    target: elaborate::lower_clause(program, revision, target)?,
                    selection: lower_selection(*requested_selection),
                    using: relations(program, revision, using)?,
                }
            }
            frontend::RequestDecl::Diff {
                base, successor, ..
            } => Request::Diff {
                base: program.revision(&base.value)?.identity().clone(),
                successor: program.revision(&successor.value)?.identity().clone(),
            },
        };
        for identity in resolved.revisions() {
            let revision = program
                .revisions()
                .values()
                .find(|revision| revision.identity() == identity)
                .expect("compiled request revision is registered");
            register_revision_closure(program, revision, &mut revisions)?;
        }
        requests.push(resolved);
    }
    ResolvedProgram::new(revisions, requests)
}

fn register_revision_closure(
    program: &CompiledProgram,
    revision: &kernel::Revision,
    revisions: &mut BTreeMap<kernel::RevisionId, kernel::Revision>,
) -> kernel::Result<()> {
    if revisions.contains_key(revision.identity()) {
        return Ok(());
    }
    if let Some(predecessor) = revision.predecessor() {
        let base = program
            .revisions()
            .values()
            .find(|candidate| candidate.identity() == predecessor)
            .ok_or_else(|| {
                kernel::KernelError::new("compiled successor is missing its exact predecessor")
            })?;
        register_revision_closure(program, base, revisions)?;
    }
    revisions.insert(revision.identity().clone(), revision.clone());
    Ok(())
}

fn relations(
    program: &CompiledProgram,
    revision: &kernel::Revision,
    values: &[frontend::Spanned<frontend::Name>],
) -> kernel::Result<Vec<ReferentId>> {
    values
        .iter()
        .map(|value| {
            let relation = program.designations().global(value.value.as_str())?;
            if !revision.model().relation_shapes().contains_key(&relation) {
                return Err(kernel::KernelError::new(format!(
                    "relation '{}' is not admitted by this Revision",
                    value.value.as_str()
                )));
            }
            Ok(relation)
        })
        .collect()
}

fn lower_selection(value: frontend::InterventionSelection) -> Selection {
    match value {
        frontend::InterventionSelection::OneMinimal => Selection::OneMinimal,
        frontend::InterventionSelection::AllMinimal => Selection::AllMinimal,
    }
}
