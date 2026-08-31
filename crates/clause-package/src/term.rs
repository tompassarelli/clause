use std::fmt;

use crate::{ClauseSemanticsId, UniverseId};

const MAX_ATOM_FIELD_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_TERM_DEPTH: usize = 256;
// Every canonical Term node contributes at least its one-octet tag, so a Term
// within the canonical byte ceiling cannot contain more nodes than this.
pub(crate) const MAX_TERM_NODES: usize = 256 * 1024 * 1024;

/// Exact index for structural equality. Equal payload bytes in different
/// universes or Clause semantics epochs are not equal Terms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TermScope {
    pub universe: UniverseId,
    pub semantics: ClauseSemanticsId,
}

/// Closed, total, versioned equality contracts admitted by process-v2.
/// Host callbacks and caller-selected contract labels are deliberately absent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EqualityContract {
    ExactOctetsV1,
}

/// One contextually opaque Atom under a fixed equality contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Atom {
    kind: Vec<u8>,
    canonical_payload: Vec<u8>,
    equality_contract: EqualityContract,
}

impl Atom {
    pub fn new(
        kind: impl Into<Vec<u8>>,
        canonical_payload: impl Into<Vec<u8>>,
        equality_contract: EqualityContract,
    ) -> Result<Self, TermError> {
        let atom = Self {
            kind: kind.into(),
            canonical_payload: canonical_payload.into(),
            equality_contract,
        };
        atom.validate()?;
        Ok(atom)
    }

    pub(crate) fn from_canonical_parts(
        kind: Vec<u8>,
        canonical_payload: Vec<u8>,
        equality_contract: EqualityContract,
    ) -> Result<Self, TermError> {
        Self::new(kind, canonical_payload, equality_contract)
    }

    fn validate(&self) -> Result<(), TermError> {
        if self.kind.is_empty() {
            return Err(TermError::EmptyKind);
        }
        for (field, length) in [
            ("kind", self.kind.len()),
            ("canonical payload", self.canonical_payload.len()),
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
    pub const fn equality_contract(&self) -> EqualityContract {
        self.equality_contract
    }
}

/// The neutral three-slot recursive carrier. No slot has an intrinsic role.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RawTriple([Box<Term>; 3]);

impl RawTriple {
    fn new(slots: [Term; 3]) -> Result<(Self, TermComplexity), TermError> {
        let scope = slots[0].scope;
        if slots.iter().any(|slot| slot.scope != scope) {
            return Err(TermError::MixedScopeTriple);
        }
        let complexity = TermComplexity::for_triple(&slots)?;
        let [a, b, c] = slots;
        Ok((Self([Box::new(a), Box::new(b), Box::new(c)]), complexity))
    }

    #[must_use]
    pub fn slots(&self) -> [&Term; 3] {
        [&self.0[0], &self.0[1], &self.0[2]]
    }

    fn scope(&self) -> TermScope {
        self.0[0].scope
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TermValue {
    Atom(Atom),
    RawTriple(RawTriple),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct TermComplexity {
    depth: usize,
    nodes: usize,
}

impl TermComplexity {
    const ATOM: Self = Self { depth: 0, nodes: 1 };

    fn for_triple(slots: &[Term; 3]) -> Result<Self, TermError> {
        let depth = slots
            .iter()
            .map(|slot| slot.complexity.depth)
            .max()
            .expect("a RawTriple always has three slots")
            .checked_add(1)
            .ok_or(TermError::DepthExceeded {
                maximum: MAX_TERM_DEPTH,
            })?;
        if depth > MAX_TERM_DEPTH {
            return Err(TermError::DepthExceeded {
                maximum: MAX_TERM_DEPTH,
            });
        }

        let nodes = slots.iter().try_fold(1usize, |total, slot| {
            total
                .checked_add(slot.complexity.nodes)
                .filter(|nodes| *nodes <= MAX_TERM_NODES)
                .ok_or(TermError::NodeCountExceeded {
                    maximum: MAX_TERM_NODES,
                })
        })?;

        Ok(Self { depth, nodes })
    }
}

/// Clause's neutral carrier, indexed by universe and semantics epoch.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Term {
    scope: TermScope,
    value: TermValue,
    complexity: TermComplexity,
}

impl Term {
    pub fn atom(
        scope: TermScope,
        kind: impl Into<Vec<u8>>,
        canonical_payload: impl Into<Vec<u8>>,
        equality_contract: EqualityContract,
    ) -> Result<Self, TermError> {
        Ok(Self {
            scope,
            value: TermValue::Atom(Atom::new(kind, canonical_payload, equality_contract)?),
            complexity: TermComplexity::ATOM,
        })
    }

    pub fn raw_triple(slots: [Term; 3]) -> Result<Self, TermError> {
        let (triple, complexity) = RawTriple::new(slots)?;
        let scope = triple.scope();
        Ok(Self {
            scope,
            value: TermValue::RawTriple(triple),
            complexity,
        })
    }

    pub(crate) const fn from_atom(scope: TermScope, atom: Atom) -> Self {
        Self {
            scope,
            value: TermValue::Atom(atom),
            complexity: TermComplexity::ATOM,
        }
    }

    pub(crate) fn value(&self) -> TermValueRef<'_> {
        match &self.value {
            TermValue::Atom(atom) => TermValueRef::Atom(atom),
            TermValue::RawTriple(triple) => TermValueRef::RawTriple(triple),
        }
    }

    #[must_use]
    pub const fn scope(&self) -> TermScope {
        self.scope
    }

    #[must_use]
    pub fn as_atom(&self) -> Option<&Atom> {
        match &self.value {
            TermValue::Atom(atom) => Some(atom),
            TermValue::RawTriple(_) => None,
        }
    }

    #[must_use]
    pub fn as_raw_triple(&self) -> Option<&RawTriple> {
        match &self.value {
            TermValue::Atom(_) => None,
            TermValue::RawTriple(triple) => Some(triple),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TermValueRef<'a> {
    Atom(&'a Atom),
    RawTriple(&'a RawTriple),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TermError {
    EmptyKind,
    FieldTooLarge { field: &'static str, length: usize },
    MixedScopeTriple,
    DepthExceeded { maximum: usize },
    NodeCountExceeded { maximum: usize },
}

impl fmt::Display for TermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TermError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IDENTITY_BYTES;

    fn scope(universe: u8, semantics: u8) -> TermScope {
        TermScope {
            universe: UniverseId::from_bytes([universe; IDENTITY_BYTES]),
            semantics: ClauseSemanticsId::from_bytes([semantics; IDENTITY_BYTES]),
        }
    }

    fn atom(scope: TermScope, payload: &[u8]) -> Term {
        Term::atom(
            scope,
            b"clause.test/octet".to_vec(),
            payload.to_vec(),
            EqualityContract::ExactOctetsV1,
        )
        .expect("bounded test Atom is valid")
    }

    #[test]
    fn structural_equality_is_indexed_by_universe_and_semantics() {
        let baseline = atom(scope(1, 2), b"same");
        assert_eq!(baseline, atom(scope(1, 2), b"same"));
        assert_ne!(baseline, atom(scope(3, 2), b"same"));
        assert_ne!(baseline, atom(scope(1, 4), b"same"));
    }

    #[test]
    fn mixed_scope_triples_reject() {
        let result = Term::raw_triple([
            atom(scope(1, 2), b"left"),
            atom(scope(1, 2), b"relation"),
            atom(scope(1, 3), b"right"),
        ]);

        assert_eq!(result, Err(TermError::MixedScopeTriple));
    }

    #[test]
    fn triple_construction_preserves_one_scope_and_neutral_slots() {
        let expected_scope = scope(1, 2);
        let triple = Term::raw_triple([
            atom(expected_scope, b"left"),
            atom(expected_scope, b"middle"),
            atom(expected_scope, b"right"),
        ])
        .expect("same-scope Triple is valid");

        assert_eq!(triple.scope(), expected_scope);
        assert!(triple.as_atom().is_none());
        let slots = triple
            .as_raw_triple()
            .expect("constructed Triple remains inspectable")
            .slots();
        assert_eq!(
            slots[0].as_atom().expect("Atom slot").canonical_payload(),
            b"left"
        );
        assert_eq!(
            slots[1].as_atom().expect("Atom slot").canonical_payload(),
            b"middle"
        );
        assert_eq!(
            slots[2].as_atom().expect("Atom slot").canonical_payload(),
            b"right"
        );
    }

    #[test]
    fn programmatic_triples_reject_before_exceeding_the_canonical_depth() {
        let term_scope = scope(1, 2);
        let mut nested = atom(term_scope, b"leaf");
        for depth in 1..=MAX_TERM_DEPTH {
            nested = Term::raw_triple([
                nested,
                atom(term_scope, b"middle"),
                atom(term_scope, b"right"),
            ])
            .unwrap_or_else(|error| panic!("depth {depth} must remain constructible: {error}"));
        }

        let result = Term::raw_triple([
            nested,
            atom(term_scope, b"middle"),
            atom(term_scope, b"right"),
        ]);
        assert!(matches!(
            result,
            Err(TermError::DepthExceeded {
                maximum: MAX_TERM_DEPTH
            })
        ));
    }
}
