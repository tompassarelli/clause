//! Physical substrate bootstrap for Clause.
//!
//! The canonical-package module implements the host-neutral Clause Core v0
//! transport and its narrow package-authorization boundary. Its Rust types are
//! representations of that external contract; they do not define Clause
//! meaning or make decoded candidates authoritative.

#![forbid(unsafe_code)]

pub mod canonical_package;
