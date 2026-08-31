//! Strict CLCP-v3 compiler-package transport.
//!
//! This module owns only the closed wire algebra. Decoding retains exact
//! bytes and yields a candidate; it never accepts a compiler or interprets
//! Clause source meaning.

mod checker;
mod codec;
mod manifest;
mod types;

pub use checker::{
    AuthorizationCheckError, AuthorizationCode, AuthorizationFailure, AuthorizationStage,
    AuthorizationVerdict, FinalPackageIdentityInput, GenesisAuthorizationRequest, OwnerAnchorInput,
    OwnerAnchorObservation, OwnerAnchorWitness, PredecessorInput, SuccessorAuthorizationRequest,
    authorize_genesis, authorize_successor,
};
pub use codec::{decode, decode_canonical_term, encode, encode_canonical_term};
pub use manifest::{
    compiler_package_hash, core_contract_id, domain_hash, eval_receipt_observations_hash,
    eval_receipt_value_hash, exact_core_manifest_bytes, exact_physical_profile_bytes,
    physical_profile_id, sha256_operation_id, source_artifact_id,
};
pub use types::*;
