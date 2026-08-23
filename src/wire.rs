//! Canonical Clause semantic wire v5.
//!
//! The semantic payload is an ordered JSON array whose exact UTF-8 bytes are
//! the revision identity preimage. Reload admits only the v3 envelope and v5
//! payload and accepts no alternate ordering or JSON spelling.

mod canonical;
mod decode;
mod json;
mod sha256;

pub use canonical::{REVISION_TAG, SEMANTIC_TAG, admit, revision_id, semantic_payload, serialize};
pub use decode::reload;
pub use sha256::sha256_hex;
