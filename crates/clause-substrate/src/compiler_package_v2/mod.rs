//! Strict CLCP-v2 compiler-package transport.
//!
//! This module owns only the closed wire algebra. Decoding retains exact
//! bytes and yields a candidate; it never accepts a compiler or interprets
//! Clause source meaning.

mod codec;
mod manifest;
mod types;

pub use codec::{decode, encode};
pub use manifest::{
    compiler_package_hash, core_contract_id, domain_hash, exact_core_manifest_bytes,
    exact_physical_profile_bytes, physical_profile_id, sha256_operation_id, source_artifact_id,
};
pub use types::*;
