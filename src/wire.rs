//! Canonical RelationalContent semantic-v9 / Revision-v5 wire.
//!
//! The semantic payload is an ordered JSON array whose exact UTF-8 bytes are
//! the revision identity preimage. Reload admits only the v5 envelope and v9
//! payload and accepts no alternate ordering or JSON spelling.

mod canonical;
mod decode;
pub(crate) mod json;
mod sha256;

pub use canonical::{
    REVISION_TAG, SEMANTIC_TAG, admit, admit_successor, revision_id, semantic_payload, serialize,
};
pub use decode::{reload, reload_successor};
pub(crate) use sha256::sha256_digest;
pub use sha256::sha256_hex;
