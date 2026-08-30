//! Immutable content-addressed artifacts.
//!
//! Artifact identity is derived once from exact bytes. Stored bytes are shared
//! read-only and are always compared on an existing-key hit; hashes remain
//! lookup aids rather than exact-byte authority.

use std::fmt;

use crate::compiler_package_v3::{
    DecodeFailure, DecodedCompilerPackage, Hash32, compiler_package_hash, decode,
    source_artifact_id,
};

#[derive(Debug)]
pub struct ImmutableArtifact {
    id: Hash32,
    bytes: Vec<u8>,
}

impl ImmutableArtifact {
    fn new(id: Hash32, bytes: Vec<u8>) -> Self {
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
    artifacts: Vec<(Hash32, ImmutableArtifact)>,
}

impl ArtifactStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern_source(&mut self, bytes: &[u8]) -> Result<&ImmutableArtifact, ArtifactError> {
        self.intern_with_id(source_artifact_id(bytes), bytes)
    }

    fn intern_compiler_package(
        &mut self,
        bytes: &[u8],
    ) -> Result<&ImmutableArtifact, ArtifactError> {
        self.intern_with_id(compiler_package_hash(bytes), bytes)
    }

    fn intern_with_id(
        &mut self,
        id: Hash32,
        bytes: &[u8],
    ) -> Result<&ImmutableArtifact, ArtifactError> {
        let position = self
            .artifacts
            .binary_search_by_key(&id, |(candidate, _)| *candidate);
        match position {
            Ok(index) => {
                let existing = self
                    .artifacts
                    .get(index)
                    .map(|(_, artifact)| artifact)
                    .ok_or(ArtifactError::ResourceExhausted)?;
                if existing.exact_bytes() != bytes {
                    return Err(ArtifactError::HashCollision(id));
                }
                Ok(existing)
            }
            Err(index) => {
                if index > self.artifacts.len() {
                    return Err(ArtifactError::ResourceExhausted);
                }
                self.artifacts
                    .try_reserve(1)
                    .map_err(|_| ArtifactError::ResourceExhausted)?;
                let mut exact_bytes = Vec::new();
                exact_bytes
                    .try_reserve_exact(bytes.len())
                    .map_err(|_| ArtifactError::ResourceExhausted)?;
                exact_bytes.extend_from_slice(bytes);
                self.artifacts
                    .insert(index, (id, ImmutableArtifact::new(id, exact_bytes)));
                self.artifacts
                    .get(index)
                    .map(|(_, artifact)| artifact)
                    .ok_or(ArtifactError::ResourceExhausted)
            }
        }
    }

    #[must_use]
    pub fn get(&self, id: Hash32) -> Option<&ImmutableArtifact> {
        self.artifacts
            .binary_search_by_key(&id, |(candidate, _)| *candidate)
            .ok()
            .and_then(|index| self.artifacts.get(index))
            .map(|(_, artifact)| artifact)
    }
}

/// Exact immutable CLCP-v3 bytes paired with their candidate-only strict
/// decode. This type intentionally has no accepted/authorized state.
#[derive(Debug)]
pub struct CompilerPackageArtifact<'a> {
    artifact: &'a ImmutableArtifact,
    candidate: DecodedCompilerPackage,
}

impl<'a> CompilerPackageArtifact<'a> {
    pub fn decode_and_intern(
        store: &'a mut ArtifactStore,
        bytes: &[u8],
    ) -> Result<Self, CompilerArtifactError> {
        let candidate = decode(bytes).map_err(CompilerArtifactError::Decode)?;
        let artifact = store
            .intern_compiler_package(candidate.exact_input())
            .map_err(CompilerArtifactError::Artifact)?;
        Ok(Self {
            artifact,
            candidate,
        })
    }

    #[must_use]
    pub const fn artifact(&self) -> &ImmutableArtifact {
        self.artifact
    }

    #[must_use]
    pub const fn candidate(&self) -> &DecodedCompilerPackage {
        &self.candidate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactError {
    HashCollision(Hash32),
    ResourceExhausted,
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashCollision(_) => formatter
                .write_str("domain-separated artifact hash resolved to non-identical exact bytes"),
            Self::ResourceExhausted => {
                formatter.write_str("artifact index exhausted physical resources")
            }
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
