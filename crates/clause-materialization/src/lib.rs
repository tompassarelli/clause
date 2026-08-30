//! Construct-blind physical projections for Clause.
//!
//! This crate consumes caller-admitted snapshots and deltas through opaque
//! references. It owns scan/index schedules, bounded allocation, replaceable
//! views, and physical receipts. It does not interpret Clause constructs,
//! allocate semantic identities, admit revisions, or own state history.

#![forbid(unsafe_code)]

mod model;
mod scan;
mod support;
mod uniform_grid;
mod work;

#[cfg(test)]
mod tests;

pub use model::{
    AdmittedSnapshot, Binding, ContractError, GridPlan, I32Binding, InputRow,
    MaterializationContract, MaterializationUpdate, OpaqueRef, OpaqueValue, PhysicalBudget,
    PremiseSlot, ScanPlan, SupportRecord,
};
pub use scan::{ScanMaterialization, materialize_scan};
pub use support::{MaterializedOutput, MaterializedView, ReverseIndexSizes};
pub use uniform_grid::UniformGridMaterializer;
pub use work::{
    FailureKind, FallbackReason, FallbackReceipt, LocalityCounters, OperationError,
    OperationReceipt, ReceiptOutcome, Schedule, WorkCounters,
};
