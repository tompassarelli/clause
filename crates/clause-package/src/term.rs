use std::fmt;

const MAX_ATOM_FIELD_BYTES: usize = 16 * 1024 * 1024;

/// One contextually opaque Atom under an explicit equality contract.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Atom {
    kind: Vec<u8>,
    canonical_payload: Vec<u8>,
    equality_contract: Vec<u8>,
}

impl Atom {
    pub fn new(
        kind: impl Into<Vec<u8>>,
        canonical_payload: impl Into<Vec<u8>>,
        equality_contract: impl Into<Vec<u8>>,
    ) -> Result<Self, TermError> {
        let atom = Self {
            kind: kind.into(),
            canonical_payload: canonical_payload.into(),
            equality_contract: equality_contract.into(),
        };
        atom.validate()?;
        Ok(atom)
    }

    pub(crate) fn from_canonical_parts(
        kind: Vec<u8>,
        canonical_payload: Vec<u8>,
        equality_contract: Vec<u8>,
    ) -> Result<Self, TermError> {
        Self::new(kind, canonical_payload, equality_contract)
    }

    fn validate(&self) -> Result<(), TermError> {
        if self.kind.is_empty() {
            return Err(TermError::EmptyKind);
        }
        if self.equality_contract.is_empty() {
            return Err(TermError::EmptyEqualityContract);
        }
        for (field, length) in [
            ("kind", self.kind.len()),
            ("canonical payload", self.canonical_payload.len()),
            ("equality contract", self.equality_contract.len()),
        ] {
            if length > MAX_ATOM_FIELD_BYTES {
                return Err(TermError::FieldTooLarge { field, length });
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn kind(&self) -> &[u8] {
        &self.kind
    }

    #[must_use]
    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }

    #[must_use]
    pub fn equality_contract(&self) -> &[u8] {
        &self.equality_contract
    }
}

/// The neutral three-slot recursive carrier. No slot has an intrinsic role.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RawTriple([Box<Term>; 3]);

impl RawTriple {
    #[must_use]
    pub fn new(slots: [Term; 3]) -> Self {
        let [a, b, c] = slots;
        Self([Box::new(a), Box::new(b), Box::new(c)])
    }

    #[must_use]
    pub fn slots(&self) -> [&Term; 3] {
        [&self.0[0], &self.0[1], &self.0[2]]
    }
}

/// Clause's structurally neutral recursive carrier.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Term {
    Atom(Atom),
    RawTriple(RawTriple),
}

impl Term {
    pub fn atom(
        kind: impl Into<Vec<u8>>,
        canonical_payload: impl Into<Vec<u8>>,
        equality_contract: impl Into<Vec<u8>>,
    ) -> Result<Self, TermError> {
        Ok(Self::Atom(Atom::new(
            kind,
            canonical_payload,
            equality_contract,
        )?))
    }

    #[must_use]
    pub fn raw_triple(slots: [Term; 3]) -> Self {
        Self::RawTriple(RawTriple::new(slots))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TermError {
    EmptyKind,
    EmptyEqualityContract,
    FieldTooLarge { field: &'static str, length: usize },
}

impl fmt::Display for TermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TermError {}
