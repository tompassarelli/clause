//! Checked process-v1 carrier primitives.
//!
//! Decoded values and proposals are inert candidate data. Only [`ProcessCarrier`]
//! validation establishes checked Applications, Activations, Steps,
//! continuations, Admissions, or State revisions.

#![forbid(unsafe_code)]

mod canonical;
mod identity;
mod process;
mod term;

pub use canonical::{
    CanonicalDecodeError, CanonicalEncodeError, DecodedProcessVector, decode_process_vector,
    encode_process_vector,
};
pub use identity::*;
pub use process::*;
pub use term::{Atom, RawTriple, Term, TermError};
