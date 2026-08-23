//! The singular source-facing Clause grammar.
//!
//! This reader is intentionally independent from the kernel. It preserves
//! authoring names and spans, resolves source structure after every declaration
//! has been collected, and rejects the retired prefix surface outright.

mod clause;
mod declaration;
mod model;
mod parser;
mod relation;
mod source;
mod syntax;

pub use parser::parse;
pub use syntax::*;
