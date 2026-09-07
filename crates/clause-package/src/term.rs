use std::fmt;
use std::sync::{Arc, OnceLock};
use std::hash::{Hash, Hasher};

use crate::{ClauseSemanticsId, UniverseId};

/// Maximum byte length of a canonical Atom kind or payload.
pub const MAX_ATOM_FIELD_BYTES: usize = 16 * 1024 * 1024;
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

/// An immutable portion of an exact Atom payload. Segment boundaries have no
/// semantic or canonical-wire significance.
#[derive(Clone, Debug)]
pub enum AtomPayloadSegment {
    Bytes(Arc<[u8]>),
    Text(Arc<str>),
}

impl AtomPayloadSegment {
    pub fn as_bytes(&self) -> &[u8] {
        match self { Self::Bytes(value) => value, Self::Text(value) => value.as_bytes() }
    }
}

#[derive(Clone)]
struct AtomPayload {
    segments: Arc<[AtomPayloadSegment]>,
    length: usize,
    contiguous: Arc<OnceLock<Vec<u8>>>,
}

impl AtomPayload {
    fn new(segments: Vec<AtomPayloadSegment>) -> Result<Self, TermError> {
        let mut length = 0_usize;
        for segment in &segments {
            length = length.checked_add(segment.as_bytes().len()).ok_or(TermError::FieldTooLarge {
                field: "canonical payload", length: usize::MAX,
            })?;
            if length > MAX_ATOM_FIELD_BYTES {
                return Err(TermError::FieldTooLarge { field: "canonical payload", length });
            }
        }
        Ok(Self { segments: segments.into(), length, contiguous: Arc::new(OnceLock::new()) })
    }

    fn bytes(&self) -> &[u8] {
        if let [segment] = self.segments.as_ref() { return segment.as_bytes(); }
        self.contiguous.get_or_init(|| {
            let mut bytes = Vec::with_capacity(self.length);
            for segment in self.segments.iter() { bytes.extend_from_slice(segment.as_bytes()); }
            bytes
        })
    }

    fn iter(&self) -> impl Iterator<Item = &u8> {
        self.segments.iter().flat_map(|segment| segment.as_bytes())
    }
}

impl PartialEq for AtomPayload {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.segments, &other.segments)
            || (self.length == other.length && self.iter().eq(other.iter()))
    }
}
impl Eq for AtomPayload {}
impl PartialOrd for AtomPayload {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for AtomPayload {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if Arc::ptr_eq(&self.segments, &other.segments) { std::cmp::Ordering::Equal }
        else { self.iter().cmp(other.iter()) }
    }
}
impl Hash for AtomPayload {
    fn hash<H: Hasher>(&self, state: &mut H) { self.bytes().hash(state); }
}
impl fmt::Debug for AtomPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { self.bytes().fmt(formatter) }
}

/// One contextually opaque Atom under a fixed equality contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Atom {
    kind: Vec<u8>,
    canonical_payload: AtomPayload,
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
            canonical_payload: AtomPayload::new(vec![AtomPayloadSegment::Bytes(canonical_payload.into().into())])?,
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
            ("canonical payload", self.canonical_payload.length),
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
        self.canonical_payload.bytes()
    }

    pub(crate) fn shared_payload_segments(&self) -> &[AtomPayloadSegment] { &self.canonical_payload.segments }

    pub(crate) fn payload_len(&self) -> usize { self.canonical_payload.length }

    #[must_use]
    pub const fn equality_contract(&self) -> EqualityContract {
        self.equality_contract
    }
}

/// The neutral three-slot recursive carrier. No slot has an intrinsic role.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Triple([Box<Term>; 3]);

impl Triple {
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
    Triple(Triple),
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
            .expect("a Triple always has three slots")
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
    value: Arc<TermValue>,
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
            value: Arc::new(TermValue::Atom(Atom::new(
                kind,
                canonical_payload,
                equality_contract,
            )?)),
            complexity: TermComplexity::ATOM,
        })
    }

    /// Construct the same exact Atom while sharing immutable payload segments.
    pub fn atom_segments(
        scope: TermScope,
        kind: impl Into<Vec<u8>>,
        segments: Vec<AtomPayloadSegment>,
        equality_contract: EqualityContract,
    ) -> Result<Self, TermError> {
        let atom = Atom { kind: kind.into(), canonical_payload: AtomPayload::new(segments)?, equality_contract };
        atom.validate()?;
        Ok(Self { scope, value: Arc::new(TermValue::Atom(atom)), complexity: TermComplexity::ATOM })
    }

    pub fn triple(slots: [Term; 3]) -> Result<Self, TermError> {
        let (triple, complexity) = Triple::new(slots)?;
        let scope = triple.scope();
        Ok(Self {
            scope,
            value: Arc::new(TermValue::Triple(triple)),
            complexity,
        })
    }

    pub(crate) fn from_atom(scope: TermScope, atom: Atom) -> Self {
        Self {
            scope,
            value: Arc::new(TermValue::Atom(atom)),
            complexity: TermComplexity::ATOM,
        }
    }

    pub(crate) fn value(&self) -> TermValueRef<'_> {
        match self.value.as_ref() {
            TermValue::Atom(atom) => TermValueRef::Atom(atom),
            TermValue::Triple(triple) => TermValueRef::Triple(triple),
        }
    }

    #[must_use]
    pub const fn scope(&self) -> TermScope {
        self.scope
    }

    #[must_use]
    pub fn as_atom(&self) -> Option<&Atom> {
        match self.value.as_ref() {
            TermValue::Atom(atom) => Some(atom),
            TermValue::Triple(_) => None,
        }
    }

    #[must_use]
    pub fn as_triple(&self) -> Option<&Triple> {
        match self.value.as_ref() {
            TermValue::Atom(_) => None,
            TermValue::Triple(triple) => Some(triple),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TermValueRef<'a> {
    Atom(&'a Atom),
    Triple(&'a Triple),
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
        let result = Term::triple([
            atom(scope(1, 2), b"left"),
            atom(scope(1, 2), b"relation"),
            atom(scope(1, 3), b"right"),
        ]);

        assert_eq!(result, Err(TermError::MixedScopeTriple));
    }

    #[test]
    fn triple_construction_preserves_one_scope_and_neutral_slots() {
        let expected_scope = scope(1, 2);
        let triple = Term::triple([
            atom(expected_scope, b"left"),
            atom(expected_scope, b"middle"),
            atom(expected_scope, b"right"),
        ])
        .expect("same-scope Triple is valid");

        assert_eq!(triple.scope(), expected_scope);
        assert!(triple.as_atom().is_none());
        let slots = triple
            .as_triple()
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
    fn segmented_atoms_preserve_exact_identity_order_hash_and_encoding_without_flattening() {
        let shared: Arc<str> = Arc::from("世界");
        let segmented = Term::atom_segments(scope(1, 2), b"test".to_vec(), vec![
            AtomPayloadSegment::Bytes(Arc::from(b"a".as_slice())),
            AtomPayloadSegment::Text(shared.clone()),
            AtomPayloadSegment::Bytes(Arc::from(b"z".as_slice())),
        ], EqualityContract::ExactOctetsV1).unwrap();
        let contiguous = Term::atom(scope(1, 2), b"test".to_vec(), "a世界z".as_bytes().to_vec(), EqualityContract::ExactOctetsV1).unwrap();
        assert_eq!(Arc::strong_count(&shared), 2);
        assert_eq!(segmented, contiguous);
        assert_eq!(segmented.cmp(&contiguous), std::cmp::Ordering::Equal);
        assert_eq!(crate::canonical_term_bytes(&segmented).unwrap(), crate::canonical_term_bytes(&contiguous).unwrap());
        assert!(segmented.as_atom().unwrap().canonical_payload.contiguous.get().is_none());
        let hash = |term: &Term| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            term.hash(&mut hasher); hasher.finish()
        };
        assert_eq!(hash(&segmented), hash(&contiguous));
        assert_eq!(segmented.as_atom().unwrap().canonical_payload(), "a世界z".as_bytes());
        let part: Arc<[u8]> = vec![0; MAX_ATOM_FIELD_BYTES].into();
        assert!(Term::atom_segments(scope(1, 2), b"test".to_vec(), vec![AtomPayloadSegment::Bytes(part.clone())], EqualityContract::ExactOctetsV1).is_ok());
        assert!(matches!(Term::atom_segments(scope(1, 2), b"test".to_vec(), vec![AtomPayloadSegment::Bytes(part), AtomPayloadSegment::Bytes(Arc::from(b"x".as_slice()))], EqualityContract::ExactOctetsV1), Err(TermError::FieldTooLarge { .. })));
    }

    #[test]
    fn immutable_term_clones_share_the_recursive_value() {
        let original = Term::triple([
            atom(scope(1, 2), b"left"),
            atom(scope(1, 2), b"middle"),
            atom(scope(1, 2), b"right"),
        ])
        .expect("same-scope Triple is valid");
        let cloned = original.clone();

        assert!(Arc::ptr_eq(&original.value, &cloned.value));
        assert_eq!(original, cloned);
    }

    #[test]
    fn programmatic_triples_reject_before_exceeding_the_canonical_depth() {
        let term_scope = scope(1, 2);
        let mut nested = atom(term_scope, b"leaf");
        for depth in 1..=MAX_TERM_DEPTH {
            nested = Term::triple([
                nested,
                atom(term_scope, b"middle"),
                atom(term_scope, b"right"),
            ])
            .unwrap_or_else(|error| panic!("depth {depth} must remain constructible: {error}"));
        }

        let result = Term::triple([
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
