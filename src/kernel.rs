//! RelationalContent's typed semantic kernel.
//!
//! The kernel admits semantic values only. Parsing, revision aliases, wire
//! representation, and requests live outside this module.

mod clause;
mod error;
mod find;
mod identity;
mod model;
mod revision;
mod schema;

pub use clause::{
    AssertionOccurrence, Definition, DerivationRule, Goal, Invariant, InvariantAdmission, Judgment,
    JudgmentKind, JudgmentStatus, JudgmentTarget, OpenWorldStatus, Pattern, RelationalContent,
    Term, Transition, UniversalLaw,
};
pub use error::{KernelError, Result};
pub use find::FindPlan;
pub use identity::{ContentId, Name, PatternId, ReferentId, RevisionId, RoleId};
pub use model::{Model, SemanticAtom};
pub use revision::{Delta, Revision, RevisionLineage};
pub use schema::{Cardinality, LookupMode, Referent, RelationShape, Role, RolePredicate};

#[cfg(test)]
mod tests;
