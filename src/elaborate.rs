//! Lower the native authoring surface into sealed semantic revisions.
//!
//! The private owners separate declaration compilation, dependency resolution,
//! and typed clause lowering.  Their public boundary is deliberately small:
//! compiled programs and one-clause lowering for request resolution.

mod compilation;
mod identifiers;
mod lowering;
mod resolution;

pub use compilation::{
    CompileDiagnostic, CompileDiagnosticStatus, CompileError, CompileResult, CompiledProgram,
    ModelContext, compile, compile_in, compile_in_with_designations,
};
pub use identifiers::DesignationTable;
pub use lowering::lower_clause;
pub use resolution::{MigrationParityReport, MigrationRevision};
