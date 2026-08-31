//! Checked process-first package and carrier primitives.
//!
//! Decoded package data is inert. Formation checking establishes only an exact
//! process constitution and package binding; separately established authority
//! is required before runtime occurrences or admitted revisions can exist.

#![forbid(unsafe_code)]

mod authority;
mod canonical;
mod formation;
mod hash;
mod identity;
mod process;
mod provenance;
mod term;

pub use authority::*;
pub use canonical::{
    CanonicalDecodeError, CanonicalEncodeError, DecodedProcessPackage, ProcessPackageCheckError,
    ProgramSnapshotPreimageV2, RevisionJudgmentAuthorityGrantPreimageV2,
    RevisionStateAdmissionGrantPreimageV2, RevisionStaticExecutionGrantPreimageV2,
    RevisionSuccessorGrantPreimageV2, canonical_term_bytes, check_process_package,
    decode_canonical_term_bytes, decode_process_package, derive_program_snapshot_id,
    encode_process_package,
};
pub use formation::*;
pub use identity::*;
pub use process::*;
pub use provenance::*;
pub use term::{Atom, EqualityContract, RawTriple, Term, TermError, TermScope};
