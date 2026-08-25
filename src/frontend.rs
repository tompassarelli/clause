//! The singular source-facing RelationalContent grammar.
//!
//! This reader is intentionally independent from the kernel. It preserves
//! authoring names and spans, resolves source structure after every declaration
//! has been collected, and rejects the retired prefix surface outright.

mod clause;
mod declaration;
mod migration;
mod model;
mod parser;
mod relation;
mod source;
mod syntax;

pub use migration::{Migration, MigrationInference, migrate};
pub use parser::parse;
pub use syntax::{
    Cardinality, Declaration, DeriveDecl, DomainName, EventDecl, EventTransitionDecl, FocusBinding,
    FocusBlock, FocusSlot, IntegerRange, InterventionSelection, Kind, LawDecl, LocalDefinitionDecl,
    Member, MembershipRangeDecl, ModeDecl, Name, ParseError, Program, PureDefinitionDecl,
    QueryColumnDecl, QuerySelection, ReferentTemplate, RequestDecl, RoleName, RuleDecl,
    SentenceShapeDecl, ShapeBindingDecl, ShapePartDecl, Span, Spanned, SurfaceApplication,
    SurfaceClause, SurfaceTerm, VariableName,
};
