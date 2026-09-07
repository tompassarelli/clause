//! Checked process-first package and carrier primitives.
//!
//! Decoded package data is inert. Formation checking establishes only an exact
//! process constitution and package binding; separately established authority
//! is required before runtime occurrences or admitted revisions can exist.

#![forbid(unsafe_code)]

mod authority;
mod canonical;
mod canonical_source;
mod formation;
mod hash;
mod identity;
mod javascript;
mod nix_flake;
mod process;
mod provenance;
mod term;

pub use authority::*;
pub use canonical::{
    CanonicalBytes, canonical_term_shared_bytes,
    CanonicalDecodeError, CanonicalEncodeError, DecodedProcessPackage, ProcessPackageCheckError,
    ProgramSnapshotPreimageV2, RevisionJudgmentAuthorityGrantPreimageV2,
    RevisionStateAdmissionGrantPreimageV2, RevisionStaticExecutionGrantPreimageV2,
    RevisionSuccessorGrantPreimageV2, canonical_term_bytes, canonical_term_byte_len, check_process_package,
    decode_canonical_term_bytes, decode_process_package, derive_program_snapshot_id,
    encode_process_package,
    encode_recorded_admitted_frontier_segments_v1, encode_recorded_admitted_frontier_v1, decode_recorded_admitted_frontier_v1,
};
pub use canonical_source::*;
pub use formation::*;
pub use identity::*;
pub use javascript::*;
pub use nix_flake::*;
pub use process::*;
pub use provenance::*;
pub use term::{Atom, AtomPayloadSegment, EqualityContract, MAX_ATOM_FIELD_BYTES, Term, TermError, TermScope, Triple};
