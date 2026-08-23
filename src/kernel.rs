//! Clause's typed semantic kernel.
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
mod tests;
