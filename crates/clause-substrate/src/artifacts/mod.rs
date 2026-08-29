//! Immutable content-addressed artifacts.
//!
//! Artifact identity is derived once from exact bytes. Stored bytes are shared
//! read-only and are always compared on an existing-key hit; hashes remain
//! lookup aids rather than exact-byte authority.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::compiler_package_v2::{
    DecodeFailure, DecodedCompilerPackage, Hash32, compiler_package_hash, decode,
    source_artifact_id,
};

#[derive(Debug)]
pub struct ImmutableArtifact {
    id: Hash32,
    bytes: Arc<[u8]>,
}

impl ImmutableArtifact {
    fn new(id: Hash32, bytes: Arc<[u8]>) -> Self {
        Self { id, bytes }
    }

    #[must_use]
    pub const fn id(&self) -> Hash32 {
        self.id
    }

    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, Default)]
pub struct ArtifactStore {
    artifacts: BTreeMap<Hash32, Arc<ImmutableArtifact>>,
}

impl ArtifactStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern_source(
        &mut self,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Result<Arc<ImmutableArtifact>, ArtifactError> {
        let bytes = bytes.into();
        self.intern_with_id(source_artifact_id(&bytes), bytes)
    }

    fn intern_compiler_package(
        &mut self,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Result<Arc<ImmutableArtifact>, ArtifactError> {
        let bytes = bytes.into();
        self.intern_with_id(compiler_package_hash(&bytes), bytes)
    }

    fn intern_with_id(
        &mut self,
        id: Hash32,
        bytes: Arc<[u8]>,
    ) -> Result<Arc<ImmutableArtifact>, ArtifactError> {
        let candidate = ImmutableArtifact::new(id, bytes);
        if let Some(existing) = self.artifacts.get(&candidate.id) {
            if existing.exact_bytes() != candidate.exact_bytes() {
                return Err(ArtifactError::HashCollision(candidate.id));
            }
            return Ok(Arc::clone(existing));
        }
        let candidate = Arc::new(candidate);
        self.artifacts.insert(candidate.id, Arc::clone(&candidate));
        Ok(candidate)
    }

    #[must_use]
    pub fn get(&self, id: Hash32) -> Option<Arc<ImmutableArtifact>> {
        self.artifacts.get(&id).cloned()
    }
}

/// Exact immutable CLCP-v2 bytes paired with their candidate-only strict
/// decode. This type intentionally has no accepted/authorized state.
#[derive(Debug)]
pub struct CompilerPackageArtifact {
    artifact: Arc<ImmutableArtifact>,
    candidate: DecodedCompilerPackage,
}

impl CompilerPackageArtifact {
    pub fn decode_and_intern(
        store: &mut ArtifactStore,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Result<Self, CompilerArtifactError> {
        let artifact = store
            .intern_compiler_package(bytes)
            .map_err(CompilerArtifactError::Artifact)?;
        let candidate = decode(artifact.exact_bytes()).map_err(CompilerArtifactError::Decode)?;
        Ok(Self {
            artifact,
            candidate,
        })
    }

    #[must_use]
    pub fn artifact(&self) -> &Arc<ImmutableArtifact> {
        &self.artifact
    }

    #[must_use]
    pub const fn candidate(&self) -> &DecodedCompilerPackage {
        &self.candidate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactError {
    HashCollision(Hash32),
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashCollision(_) => formatter
                .write_str("domain-separated artifact hash resolved to non-identical exact bytes"),
        }
    }
}

impl std::error::Error for ArtifactError {}

#[derive(Debug)]
pub enum CompilerArtifactError {
    Artifact(ArtifactError),
    Decode(DecodeFailure),
}

impl fmt::Display for CompilerArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(error) => write!(formatter, "artifact storage failed: {error}"),
            Self::Decode(error) => write!(formatter, "candidate decode failed: {error}"),
        }
    }
}

impl std::error::Error for CompilerArtifactError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Artifact(error) => Some(error),
            Self::Decode(error) => Some(error),
        }
    }
}
