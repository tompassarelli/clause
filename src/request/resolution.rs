use std::collections::BTreeMap;

use super::{Request, ResolvedProgram, Selection};
use crate::{
    elaborate::{self, CompiledProgram},
    frontend,
    kernel::{self, Name, RelationId, VariableId},
};

pub(super) fn resolve(program: &CompiledProgram) -> kernel::Result<ResolvedProgram> {
    let mut revisions = BTreeMap::new();
    let mut requests = Vec::with_capacity(program.requests().len());
    for request in program.requests() {
        let resolved = match request {
            frontend::RequestDecl::Find {
                revision,
                pattern,
                sought,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                let pattern = elaborate::lower_clause(revision, pattern)?;
                let sought = variable(&sought.value)?;
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
                    target: elaborate::lower_clause(revision, target)?,
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
                    target: elaborate::lower_clause(revision, target)?,
                    selection: lower_selection(*requested_selection),
                    using: relations(using)?,
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
                    target: elaborate::lower_clause(revision, target)?,
                    selection: lower_selection(*requested_selection),
                    using: relations(using)?,
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
            revisions.entry(identity.clone()).or_insert_with(|| {
                program
                    .revisions()
                    .values()
                    .find(|revision| revision.identity() == identity)
                    .expect("compiled request revision is registered")
                    .clone()
            });
        }
        requests.push(resolved);
    }
    ResolvedProgram::new(revisions, requests)
}

fn variable(value: &frontend::VariableName) -> kernel::Result<VariableId> {
    VariableId::new(Name::new(value.0.clone())?)
}

fn relations(values: &[frontend::Spanned<frontend::Name>]) -> kernel::Result<Vec<RelationId>> {
    values
        .iter()
        .map(|value| RelationId::new(Name::new(value.value.0.clone())?))
        .collect()
}

fn lower_selection(value: frontend::InterventionSelection) -> Selection {
    match value {
        frontend::InterventionSelection::OneMinimal => Selection::OneMinimal,
        frontend::InterventionSelection::AllMinimal => Selection::AllMinimal,
    }
}
