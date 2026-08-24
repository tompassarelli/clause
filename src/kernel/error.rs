use std::fmt;

use super::identity::{ContentId, Name, ReferentId, RoleId};

/// The semantic proposal whose structure the kernel is validating.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProposalSubject {
    Definition(ReferentId),
    Content(ContentId),
    Contract(ReferentId),
}

/// One canonical selector within a proposal. Selectors navigate existing
/// semantic identities; they never create a second identity namespace.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProposalPathSegment {
    Role(RoleId),
    ProductField(ReferentId),
    TupleIndex(usize),
    SequenceIndex(usize),
    SumPayload(Name),
    Application(ContentId),
}

/// A source-free semantic route to one proposed term.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProposalPath {
    subject: ProposalSubject,
    segments: Vec<ProposalPathSegment>,
}

impl ProposalPath {
    pub fn new(subject: ProposalSubject) -> Self {
        Self {
            subject,
            segments: Vec::new(),
        }
    }

    pub fn child(&self, segment: ProposalPathSegment) -> Self {
        let mut path = self.clone();
        path.segments.push(segment);
        path
    }

    pub fn subject(&self) -> &ProposalSubject {
        &self.subject
    }

    pub fn segments(&self) -> &[ProposalPathSegment] {
        &self.segments
    }
}

/// Stable kernel classification for a structural rejection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StructuralFailureClass {
    ContractUnavailable,
    DomainMismatch,
    FieldSetMismatch,
    NonCanonicalPosition,
}

/// Typed, source-free evidence produced by kernel validation alone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralFailure {
    class: StructuralFailureClass,
    path: ProposalPath,
}

impl StructuralFailure {
    pub fn class(&self) -> StructuralFailureClass {
        self.class
    }

    pub fn path(&self) -> &ProposalPath {
        &self.path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelError {
    message: String,
    structural_failure: Option<StructuralFailure>,
}

impl KernelError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            structural_failure: None,
        }
    }

    pub(crate) fn structural(
        message: impl Into<String>,
        class: StructuralFailureClass,
        path: ProposalPath,
    ) -> Self {
        Self {
            message: message.into(),
            structural_failure: Some(StructuralFailure { class, path }),
        }
    }

    pub(crate) fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn structural_failure(&self) -> Option<&StructuralFailure> {
        self.structural_failure.as_ref()
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for KernelError {}

pub type Result<T> = std::result::Result<T, KernelError>;
