//! Clause's typed semantic kernel.
//!
//! The kernel admits semantic values only. Parsing, revision aliases, wire
//! representation, and requests live outside this module.

#[path = "kernel/clause.rs"]
mod clause;
#[path = "kernel/error.rs"]
mod error;
#[path = "kernel/find.rs"]
mod find;
#[path = "kernel/identity.rs"]
mod identity;
#[path = "kernel/model.rs"]
mod model;
#[path = "kernel/revision.rs"]
mod revision;
#[path = "kernel/schema.rs"]
mod schema;

pub use clause::{Clause, Law, Term};
pub use error::{KernelError, Result};
pub use find::FindPlan;
pub use identity::{
    EntityId, LawId, ModelId, Name, RelationId, RevisionId, RoleId, TypeId, VariableId,
};
pub use model::Model;
pub use revision::{Delta, Revision};
pub use schema::{
    Cardinality, InlineSentencePart, Mode, Relation, Role, SentencePart, SentenceShape, Type,
};

#[cfg(test)]
#[path = "kernel/tests.rs"]
mod tests;
