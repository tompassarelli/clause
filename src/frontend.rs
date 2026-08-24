//! The singular source-facing RelationalContent grammar.
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
pub use syntax::{
    Cardinality, Declaration, DomainName, FocusBinding, FocusBlock, FocusSlot, IntegerRange,
    InterventionSelection, Kind, Member, MembershipRangeDecl, ModeDecl, Name, ParseError, Program,
    ReferentTemplate, RequestDecl, RoleName, SentenceShapeDecl, ShapeBindingDecl, ShapePartDecl,
    Span, Spanned, SurfaceApplication, SurfaceClause, SurfaceTerm, VariableName,
};
