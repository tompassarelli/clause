//! Canonical RelationalContent semantic-v10 / Revision-v6 wire.
//!
//! The semantic payload is an ordered JSON array whose exact UTF-8 bytes are
//! the revision identity preimage. Reload admits only the v6 envelope and v10
//! payload and accepts no alternate ordering or JSON spelling.

mod canonical;
mod decode;
pub(crate) mod json;
mod sha256;

pub(crate) use canonical::term_json;
pub use canonical::{
    REVISION_TAG, SEMANTIC_TAG, admit, admit_successor, program_snapshot, program_snapshot_id,
    program_snapshot_payload, revision_id, semantic_payload, serialize,
};
pub(crate) use decode::decode_term;
pub use decode::{reload, reload_successor};
pub(crate) use sha256::sha256_digest;
pub use sha256::sha256_hex;
