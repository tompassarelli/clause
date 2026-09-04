//! Strict Clause Core v0 canonical-package transport.
//!
//! Decoding, relative certificate checking, exact binding, and package
//! authorization are deliberately separate operations. In particular,
//! [`decode`] returns only a [`CanonicalPackageCandidate`].

use std::fmt;
use std::sync::OnceLock;

const MAGIC: &[u8; 4] = b"CLCP";
const VERSION: u8 = 1;

const INDEX_FRAME: u8 = 0x01;
const LINEAGE_FRAME: u8 = 0x02;
const BASIS_FRAME: u8 = 0x03;
const CERTIFICATE_FRAME: u8 = 0x04;
const TARGET_FRAME: u8 = 0x05;
const AUXILIARY_FRAME: u8 = 0x06;

const ATOM_TAG: u8 = 0x00;
const TRIPLE_TAG: u8 = 0x01;
const ROOT_LINEAGE_TAG: u8 = 0x00;
const SUCCESSOR_LINEAGE_TAG: u8 = 0x01;
const ROOT_REASON_TAG: u8 = 0x00;
const APPLY_REASON_TAG: u8 = 0x01;

const BASIS_ADMISSION_KIND: &[u8] = &[0xf0];
const BASIS_ADMISSION_EQUALITY_CONTRACT: &[u8] = &[0xf1];
const BASIS_ADMISSION_TYPE_PAYLOAD: &[u8] = &[0xf2];
const BASIS_ADMISSION_MODE_PAYLOAD: &[u8] = &[0xf3];

const LITERAL_BOOTSTRAP_HEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/canonical-package/positive/bootstrap.hex"
));

/// Opaque canonical bytes carried by the v0 grammar.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Blob(pub Vec<u8>);

/// The exact universe and semantics epoch in which representations are read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralIndex {
    pub universe_id: Blob,
    pub semantics_id: Blob,
}

/// The two closed recursive v0 Term representations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Term {
    Atom {
        kind: Blob,
        canonical_payload: Blob,
        equality_contract: Blob,
    },
    Triple(Box<Term>, Box<Term>, Box<Term>),
}

/// A candidate `term : type @ mode` claim. Construction grants no validity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub term: Term,
    pub type_term: Term,
    pub mode: Term,
}

/// One finite ground rule candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundRule {
    pub premises: Vec<Claim>,
    pub conclusion: Claim,
}

/// Ordered candidate roots and ground rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivationBasis {
    pub roots: Vec<Claim>,
    pub rules: Vec<GroundRule>,
}

/// One package-local certificate reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateReason {
    Root {
        root_ref: u32,
    },
    Apply {
        rule_ref: u32,
        premise_refs: Vec<u32>,
    },
}

/// One ordered certificate node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateNode {
    pub claimed: Claim,
    pub reason: CertificateReason,
}

/// A finite ground derivation certificate candidate.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Certificate {
    pub nodes: Vec<CertificateNode>,
}

/// Exact lineage evidence. Embedded predecessor bytes are not replaced by a
/// digest or by a decoded projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Lineage {
    Root,
    Successor {
        predecessor_package: Blob,
        authorization: Certificate,
    },
}

/// All decoded v0 package fields, without any authority claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageValue {
    pub index: StructuralIndex,
    pub lineage: Lineage,
    pub basis: DerivationBasis,
    pub certificate: Certificate,
    pub target: Claim,
    pub auxiliary: Vec<Blob>,
}

/// A strictly decoded package candidate bound to its exact input and the exact
/// INDEX and BASIS frame bytes used by successor authorization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalPackageCandidate {
    exact_bytes: Box<[u8]>,
    exact_index_frame: Box<[u8]>,
    exact_basis_frame: Box<[u8]>,
    value: PackageValue,
}

impl CanonicalPackageCandidate {
    /// The complete exact package input, including magic, frames, and EOF.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.exact_bytes
    }

    /// The full decoded candidate record.
    #[must_use]
    pub fn value(&self) -> &PackageValue {
        &self.value
    }

    /// The exact encoded INDEX frame, including its tag and length.
    #[must_use]
    pub fn exact_index_frame(&self) -> &[u8] {
        &self.exact_index_frame
    }

    /// The exact encoded BASIS frame, including its tag and length.
    #[must_use]
    pub fn exact_basis_frame(&self) -> &[u8] {
        &self.exact_basis_frame
    }

    /// Exact whole-record binding. No digest or projection participates.
    #[must_use]
    pub fn is_exactly(&self, selected: &Self) -> bool {
        self == selected
    }
}

/// A narrow witness that the candidate passed v0 package authorization.
///
/// This witnesses only literal-bootstrap/predecessor authorization plus the
/// package's relative certificate check. It is not Clause semantic truth,
/// general admission, or authorization of auxiliary bytes.
#[derive(Clone, Copy, Debug)]
pub struct V0AuthorizedPackage<'a> {
    package: &'a CanonicalPackageCandidate,
}

impl<'a> V0AuthorizedPackage<'a> {
    #[must_use]
    pub fn package(self) -> &'a CanonicalPackageCandidate {
        self.package
    }
}

/// The closed tagged constructs for which an unknown tag can be reported.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaggedConstruct {
    Term,
    Lineage,
    CertificateReason,
}

/// Strict canonical decoding failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnexpectedEof {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    WrongMagic {
        found: Vec<u8>,
    },
    UnsupportedVersion {
        offset: usize,
        found: u8,
    },
    UnexpectedFrameTag {
        offset: usize,
        expected: u8,
        found: u8,
    },
    UnknownTag {
        offset: usize,
        construct: TaggedConstruct,
        found: u8,
    },
    InvalidPredecessorPackage {
        offset: usize,
        error: Box<DecodeError>,
    },
    CountOutOfBounds {
        offset: usize,
        count: u32,
        remaining: usize,
    },
    AllocationFailed {
        offset: usize,
        count: u32,
    },
    UnderconsumedFrame {
        tag: u8,
        offset: usize,
        remaining: usize,
    },
    TrailingBytes {
        offset: usize,
        remaining: usize,
    },
    NonCanonicalEncoding {
        offset: usize,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof {
                offset,
                needed,
                remaining,
            } => write!(
                formatter,
                "truncated input at byte {offset}: need {needed} bytes, have {remaining}"
            ),
            Self::WrongMagic { found } => write!(formatter, "wrong package magic: {found:02x?}"),
            Self::UnsupportedVersion { offset, found } => {
                write!(formatter, "unsupported version {found} at byte {offset}")
            }
            Self::UnexpectedFrameTag {
                offset,
                expected,
                found,
            } => write!(
                formatter,
                "unexpected frame tag {found:#04x} at byte {offset}; expected {expected:#04x}"
            ),
            Self::UnknownTag {
                offset,
                construct,
                found,
            } => write!(
                formatter,
                "unknown {construct:?} tag {found:#04x} at byte {offset}"
            ),
            Self::InvalidPredecessorPackage { offset, error } => {
                write!(
                    formatter,
                    "invalid predecessor package at byte {offset}: {error}"
                )
            }
            Self::CountOutOfBounds {
                offset,
                count,
                remaining,
            } => write!(
                formatter,
                "list count {count} at byte {offset} exceeds {remaining} enclosing bytes"
            ),
            Self::AllocationFailed { offset, count } => write!(
                formatter,
                "could not allocate list count {count} read at byte {offset}"
            ),
            Self::UnderconsumedFrame {
                tag,
                offset,
                remaining,
            } => write!(
                formatter,
                "frame {tag:#04x} left {remaining} bytes unconsumed at byte {offset}"
            ),
            Self::TrailingBytes { offset, remaining } => {
                write!(formatter, "{remaining} trailing bytes at byte {offset}")
            }
            Self::NonCanonicalEncoding { offset } => {
                write!(
                    formatter,
                    "input differs from canonical re-encoding at byte {offset}"
                )
            }
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPredecessorPackage { error, .. } => Some(error),
            Self::UnexpectedEof { .. }
            | Self::WrongMagic { .. }
            | Self::UnsupportedVersion { .. }
            | Self::UnexpectedFrameTag { .. }
            | Self::UnknownTag { .. }
            | Self::CountOutOfBounds { .. }
            | Self::AllocationFailed { .. }
            | Self::UnderconsumedFrame { .. }
            | Self::TrailingBytes { .. }
            | Self::NonCanonicalEncoding { .. } => None,
        }
    }
}

/// Encoding failures for programmatically constructed candidate values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthExceedsU32 { field: &'static str, length: usize },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthExceedsU32 { field, length } => {
                write!(formatter, "{field} length {length} exceeds u32")
            }
        }
    }
}

impl std::error::Error for EncodeError {}

/// Precise finite ground-certificate rejection reasons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateError {
    Empty,
    RootOutOfBounds {
        node: usize,
        root_ref: u32,
    },
    RootClaimMismatch {
        node: usize,
        root_ref: u32,
    },
    RuleOutOfBounds {
        node: usize,
        rule_ref: u32,
    },
    PremiseCountMismatch {
        node: usize,
        expected: usize,
        found: usize,
    },
    DuplicatePremiseReference {
        node: usize,
        premise_ref: u32,
    },
    PremiseNotEarlier {
        node: usize,
        premise: usize,
        premise_ref: u32,
    },
    PremiseClaimMismatch {
        node: usize,
        premise: usize,
        premise_ref: u32,
    },
    RuleConclusionMismatch {
        node: usize,
        rule_ref: u32,
    },
    TargetMismatch,
}

impl fmt::Display for CertificateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CertificateError {}

/// Rejections from the closed v0 authorization boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
    RootIsNotLiteralBootstrap,
    PredecessorDecode(DecodeError),
    PredecessorUnauthorized(Box<AuthorizationError>),
    UniverseMismatch,
    SemanticsMismatch,
    LineageCertificate(CertificateError),
    PackageCertificate(CertificateError),
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AuthorizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PredecessorDecode(error) => Some(error),
            Self::PredecessorUnauthorized(error) => Some(error),
            Self::LineageCertificate(error) | Self::PackageCertificate(error) => Some(error),
            Self::RootIsNotLiteralBootstrap | Self::UniverseMismatch | Self::SemanticsMismatch => {
                None
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
    base: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            base: 0,
        }
    }

    fn at(bytes: &'a [u8], base: usize) -> Self {
        Self {
            bytes,
            position: 0,
            base,
        }
    }

    fn offset(self) -> usize {
        self.base + self.position
    }

    fn remaining(self) -> usize {
        self.bytes.len() - self.position
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        let remaining = self.remaining();
        if length > remaining {
            return Err(DecodeError::UnexpectedEof {
                offset: self.offset(),
                needed: length,
                remaining,
            });
        }
        let start = self.position;
        self.position += length;
        Ok(&self.bytes[start..self.position])
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        let bytes: [u8; 4] = self
            .take(4)?
            .try_into()
            .expect("a four-byte cursor slice has length four");
        Ok(u32::from_be_bytes(bytes))
    }

    fn finish_frame(self, tag: u8) -> Result<(), DecodeError> {
        let remaining = self.remaining();
        if remaining == 0 {
            Ok(())
        } else {
            Err(DecodeError::UnderconsumedFrame {
                tag,
                offset: self.offset(),
                remaining,
            })
        }
    }
}

fn decode_blob(cursor: &mut Cursor<'_>) -> Result<Blob, DecodeError> {
    let length = cursor.u32()? as usize;
    Ok(Blob(cursor.take(length)?.to_vec()))
}

fn decode_list<T>(
    cursor: &mut Cursor<'_>,
    mut decode_item: impl FnMut(&mut Cursor<'_>) -> Result<T, DecodeError>,
) -> Result<Vec<T>, DecodeError> {
    let count_offset = cursor.offset();
    let count = cursor.u32()?;
    if count as usize > cursor.remaining() {
        return Err(DecodeError::CountOutOfBounds {
            offset: count_offset,
            count,
            remaining: cursor.remaining(),
        });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(count as usize)
        .map_err(|_| DecodeError::AllocationFailed {
            offset: count_offset,
            count,
        })?;
    for _ in 0..count {
        values.push(decode_item(cursor)?);
    }
    Ok(values)
}

fn decode_term(cursor: &mut Cursor<'_>) -> Result<Term, DecodeError> {
    let tag_offset = cursor.offset();
    match cursor.u8()? {
        ATOM_TAG => Ok(Term::Atom {
            kind: decode_blob(cursor)?,
            canonical_payload: decode_blob(cursor)?,
            equality_contract: decode_blob(cursor)?,
        }),
        TRIPLE_TAG => Ok(Term::Triple(
            Box::new(decode_term(cursor)?),
            Box::new(decode_term(cursor)?),
            Box::new(decode_term(cursor)?),
        )),
        found => Err(DecodeError::UnknownTag {
            offset: tag_offset,
            construct: TaggedConstruct::Term,
            found,
        }),
    }
}

fn decode_claim(cursor: &mut Cursor<'_>) -> Result<Claim, DecodeError> {
    Ok(Claim {
        term: decode_term(cursor)?,
        type_term: decode_term(cursor)?,
        mode: decode_term(cursor)?,
    })
}

fn decode_rule(cursor: &mut Cursor<'_>) -> Result<GroundRule, DecodeError> {
    Ok(GroundRule {
        premises: decode_list(cursor, decode_claim)?,
        conclusion: decode_claim(cursor)?,
    })
}

fn decode_basis(cursor: &mut Cursor<'_>) -> Result<DerivationBasis, DecodeError> {
    Ok(DerivationBasis {
        roots: decode_list(cursor, decode_claim)?,
        rules: decode_list(cursor, decode_rule)?,
    })
}

fn decode_reason(cursor: &mut Cursor<'_>) -> Result<CertificateReason, DecodeError> {
    let tag_offset = cursor.offset();
    match cursor.u8()? {
        ROOT_REASON_TAG => Ok(CertificateReason::Root {
            root_ref: cursor.u32()?,
        }),
        APPLY_REASON_TAG => Ok(CertificateReason::Apply {
            rule_ref: cursor.u32()?,
            premise_refs: decode_list(cursor, |cursor| cursor.u32())?,
        }),
        found => Err(DecodeError::UnknownTag {
            offset: tag_offset,
            construct: TaggedConstruct::CertificateReason,
            found,
        }),
    }
}

fn decode_node(cursor: &mut Cursor<'_>) -> Result<CertificateNode, DecodeError> {
    Ok(CertificateNode {
        claimed: decode_claim(cursor)?,
        reason: decode_reason(cursor)?,
    })
}

fn decode_certificate(cursor: &mut Cursor<'_>) -> Result<Certificate, DecodeError> {
    Ok(Certificate {
        nodes: decode_list(cursor, decode_node)?,
    })
}

fn decode_lineage(cursor: &mut Cursor<'_>) -> Result<Lineage, DecodeError> {
    let tag_offset = cursor.offset();
    match cursor.u8()? {
        ROOT_LINEAGE_TAG => Ok(Lineage::Root),
        SUCCESSOR_LINEAGE_TAG => {
            let predecessor_offset = cursor.offset() + 4;
            let predecessor_package = decode_blob(cursor)?;
            decode(&predecessor_package.0).map_err(|error| {
                DecodeError::InvalidPredecessorPackage {
                    offset: predecessor_offset,
                    error: Box::new(error),
                }
            })?;
            Ok(Lineage::Successor {
                predecessor_package,
                authorization: decode_certificate(cursor)?,
            })
        }
        found => Err(DecodeError::UnknownTag {
            offset: tag_offset,
            construct: TaggedConstruct::Lineage,
            found,
        }),
    }
}

fn read_frame<'a>(
    cursor: &mut Cursor<'a>,
    expected_tag: u8,
) -> Result<(Cursor<'a>, &'a [u8]), DecodeError> {
    let frame_start = cursor.position;
    let tag_offset = cursor.offset();
    let found = cursor.u8()?;
    if found != expected_tag {
        return Err(DecodeError::UnexpectedFrameTag {
            offset: tag_offset,
            expected: expected_tag,
            found,
        });
    }
    let length = cursor.u32()? as usize;
    let payload_base = cursor.offset();
    let payload = cursor.take(length)?;
    let exact_frame = &cursor.bytes[frame_start..cursor.position];
    Ok((Cursor::at(payload, payload_base), exact_frame))
}

/// Decode one exact v0 package candidate and reject every alternate spelling.
pub fn decode(bytes: &[u8]) -> Result<CanonicalPackageCandidate, DecodeError> {
    let mut cursor = Cursor::new(bytes);
    let found_magic = cursor.take(MAGIC.len())?;
    if found_magic != MAGIC {
        return Err(DecodeError::WrongMagic {
            found: found_magic.to_vec(),
        });
    }
    let version_offset = cursor.offset();
    let version = cursor.u8()?;
    if version != VERSION {
        return Err(DecodeError::UnsupportedVersion {
            offset: version_offset,
            found: version,
        });
    }

    let (mut index_cursor, exact_index_frame) = read_frame(&mut cursor, INDEX_FRAME)?;
    let index = StructuralIndex {
        universe_id: decode_blob(&mut index_cursor)?,
        semantics_id: decode_blob(&mut index_cursor)?,
    };
    index_cursor.finish_frame(INDEX_FRAME)?;

    let (mut lineage_cursor, _) = read_frame(&mut cursor, LINEAGE_FRAME)?;
    let lineage = decode_lineage(&mut lineage_cursor)?;
    lineage_cursor.finish_frame(LINEAGE_FRAME)?;

    let (mut basis_cursor, exact_basis_frame) = read_frame(&mut cursor, BASIS_FRAME)?;
    let basis = decode_basis(&mut basis_cursor)?;
    basis_cursor.finish_frame(BASIS_FRAME)?;

    let (mut certificate_cursor, _) = read_frame(&mut cursor, CERTIFICATE_FRAME)?;
    let certificate = decode_certificate(&mut certificate_cursor)?;
    certificate_cursor.finish_frame(CERTIFICATE_FRAME)?;

    let (mut target_cursor, _) = read_frame(&mut cursor, TARGET_FRAME)?;
    let target = decode_claim(&mut target_cursor)?;
    target_cursor.finish_frame(TARGET_FRAME)?;

    let (mut auxiliary_cursor, _) = read_frame(&mut cursor, AUXILIARY_FRAME)?;
    let auxiliary = decode_list(&mut auxiliary_cursor, decode_blob)?;
    auxiliary_cursor.finish_frame(AUXILIARY_FRAME)?;

    if cursor.remaining() != 0 {
        return Err(DecodeError::TrailingBytes {
            offset: cursor.offset(),
            remaining: cursor.remaining(),
        });
    }

    let value = PackageValue {
        index,
        lineage,
        basis,
        certificate,
        target,
        auxiliary,
    };
    let reencoded = encode(&value).expect("a decoded package always fits its original u32 lengths");
    if reencoded != bytes {
        let offset = reencoded
            .iter()
            .zip(bytes)
            .position(|(canonical, original)| canonical != original)
            .unwrap_or_else(|| reencoded.len().min(bytes.len()));
        return Err(DecodeError::NonCanonicalEncoding { offset });
    }

    Ok(CanonicalPackageCandidate {
        exact_bytes: bytes.into(),
        exact_index_frame: exact_index_frame.into(),
        exact_basis_frame: exact_basis_frame.into(),
        value,
    })
}

fn encode_u32_length(
    output: &mut Vec<u8>,
    field: &'static str,
    length: usize,
) -> Result<(), EncodeError> {
    let length =
        u32::try_from(length).map_err(|_| EncodeError::LengthExceedsU32 { field, length })?;
    output.extend_from_slice(&length.to_be_bytes());
    Ok(())
}

fn encode_blob(output: &mut Vec<u8>, blob: &Blob) -> Result<(), EncodeError> {
    encode_u32_length(output, "blob", blob.0.len())?;
    output.extend_from_slice(&blob.0);
    Ok(())
}

fn encode_list<T>(
    output: &mut Vec<u8>,
    values: &[T],
    mut encode_item: impl FnMut(&mut Vec<u8>, &T) -> Result<(), EncodeError>,
) -> Result<(), EncodeError> {
    encode_u32_length(output, "list", values.len())?;
    for value in values {
        encode_item(output, value)?;
    }
    Ok(())
}

fn encode_term(output: &mut Vec<u8>, term: &Term) -> Result<(), EncodeError> {
    match term {
        Term::Atom {
            kind,
            canonical_payload,
            equality_contract,
        } => {
            output.push(ATOM_TAG);
            encode_blob(output, kind)?;
            encode_blob(output, canonical_payload)?;
            encode_blob(output, equality_contract)?;
        }
        Term::Triple(first, second, third) => {
            output.push(TRIPLE_TAG);
            encode_term(output, first)?;
            encode_term(output, second)?;
            encode_term(output, third)?;
        }
    }
    Ok(())
}

fn encode_claim(output: &mut Vec<u8>, claim: &Claim) -> Result<(), EncodeError> {
    encode_term(output, &claim.term)?;
    encode_term(output, &claim.type_term)?;
    encode_term(output, &claim.mode)
}

fn encode_rule(output: &mut Vec<u8>, rule: &GroundRule) -> Result<(), EncodeError> {
    encode_list(output, &rule.premises, encode_claim)?;
    encode_claim(output, &rule.conclusion)
}

fn encode_basis(output: &mut Vec<u8>, basis: &DerivationBasis) -> Result<(), EncodeError> {
    encode_list(output, &basis.roots, encode_claim)?;
    encode_list(output, &basis.rules, encode_rule)
}

fn encode_reason(output: &mut Vec<u8>, reason: &CertificateReason) -> Result<(), EncodeError> {
    match reason {
        CertificateReason::Root { root_ref } => {
            output.push(ROOT_REASON_TAG);
            output.extend_from_slice(&root_ref.to_be_bytes());
        }
        CertificateReason::Apply {
            rule_ref,
            premise_refs,
        } => {
            output.push(APPLY_REASON_TAG);
            output.extend_from_slice(&rule_ref.to_be_bytes());
            encode_list(output, premise_refs, |output, premise_ref| {
                output.extend_from_slice(&premise_ref.to_be_bytes());
                Ok(())
            })?;
        }
    }
    Ok(())
}

fn encode_node(output: &mut Vec<u8>, node: &CertificateNode) -> Result<(), EncodeError> {
    encode_claim(output, &node.claimed)?;
    encode_reason(output, &node.reason)
}

fn encode_certificate(output: &mut Vec<u8>, certificate: &Certificate) -> Result<(), EncodeError> {
    encode_list(output, &certificate.nodes, encode_node)
}

fn encode_lineage(output: &mut Vec<u8>, lineage: &Lineage) -> Result<(), EncodeError> {
    match lineage {
        Lineage::Root => output.push(ROOT_LINEAGE_TAG),
        Lineage::Successor {
            predecessor_package,
            authorization,
        } => {
            output.push(SUCCESSOR_LINEAGE_TAG);
            encode_blob(output, predecessor_package)?;
            encode_certificate(output, authorization)?;
        }
    }
    Ok(())
}

fn encode_frame(output: &mut Vec<u8>, tag: u8, payload: Vec<u8>) -> Result<(), EncodeError> {
    output.push(tag);
    encode_u32_length(output, "frame", payload.len())?;
    output.extend_from_slice(&payload);
    Ok(())
}

/// Encode one programmatically constructed v0 candidate value canonically.
pub fn encode(value: &PackageValue) -> Result<Vec<u8>, EncodeError> {
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    output.push(VERSION);

    let mut index = Vec::new();
    encode_blob(&mut index, &value.index.universe_id)?;
    encode_blob(&mut index, &value.index.semantics_id)?;
    encode_frame(&mut output, INDEX_FRAME, index)?;

    let mut lineage = Vec::new();
    encode_lineage(&mut lineage, &value.lineage)?;
    encode_frame(&mut output, LINEAGE_FRAME, lineage)?;

    let mut basis = Vec::new();
    encode_basis(&mut basis, &value.basis)?;
    encode_frame(&mut output, BASIS_FRAME, basis)?;

    let mut certificate = Vec::new();
    encode_certificate(&mut certificate, &value.certificate)?;
    encode_frame(&mut output, CERTIFICATE_FRAME, certificate)?;

    let mut target = Vec::new();
    encode_claim(&mut target, &value.target)?;
    encode_frame(&mut output, TARGET_FRAME, target)?;

    let mut auxiliary = Vec::new();
    encode_list(&mut auxiliary, &value.auxiliary, encode_blob)?;
    encode_frame(&mut output, AUXILIARY_FRAME, auxiliary)?;
    Ok(output)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroundClosureError {
    Exhausted,
    IndexOverflow,
}

impl fmt::Display for GroundClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Exhausted => "finite ground closure exhausted its rule-check budget",
            Self::IndexOverflow => "finite ground closure exceeds certificate index width",
        })
    }
}

impl std::error::Error for GroundClosureError {}

/// Compute the positive least fixed point of this exact finite ground basis.
/// Each distinct claim carries one finite witness consumable by
/// [`check_certificate`]; this is not an enumeration of independent evidence.
/// Recompute from the remaining roots after withdrawal, never from old closure.
/// Like the bootstrap checker, this establishes only relative derivability.
///
/// An exhausted search returns no closure and cannot establish absence.
pub fn derive_ground_closure(
    basis: &DerivationBasis,
    mut rule_checks: usize,
) -> Result<Certificate, GroundClosureError> {
    let mut certificate = Certificate::default();
    for (index, root) in basis.roots.iter().enumerate() {
        if certificate.nodes.iter().any(|node| &node.claimed == root) {
            continue;
        }
        certificate.nodes.push(CertificateNode {
            claimed: root.clone(),
            reason: CertificateReason::Root {
                root_ref: u32::try_from(index).map_err(|_| GroundClosureError::IndexOverflow)?,
            },
        });
    }
    loop {
        let prior = certificate.nodes.len();
        for (index, rule) in basis.rules.iter().enumerate() {
            rule_checks = rule_checks
                .checked_sub(1)
                .ok_or(GroundClosureError::Exhausted)?;
            if certificate
                .nodes
                .iter()
                .any(|node| node.claimed == rule.conclusion)
            {
                continue;
            }
            let premises = rule
                .premises
                .iter()
                .map(|premise| {
                    certificate
                        .nodes
                        .iter()
                        .position(|node| &node.claimed == premise)
                })
                .collect::<Option<Vec<_>>>();
            let Some(premises) = premises else { continue };
            u32::try_from(certificate.nodes.len())
                .map_err(|_| GroundClosureError::IndexOverflow)?;
            certificate.nodes.push(CertificateNode {
                claimed: rule.conclusion.clone(),
                reason: CertificateReason::Apply {
                    rule_ref: u32::try_from(index)
                        .map_err(|_| GroundClosureError::IndexOverflow)?,
                    premise_refs: premises
                        .into_iter()
                        .map(|index| {
                            u32::try_from(index).map_err(|_| GroundClosureError::IndexOverflow)
                        })
                        .collect::<Result<_, _>>()?,
                },
            });
        }
        if certificate.nodes.len() == prior {
            return Ok(certificate);
        }
    }
}

/// Check a finite ground certificate only against the explicitly supplied
/// basis and requested claim.
pub fn check_certificate(
    basis: &DerivationBasis,
    certificate: &Certificate,
    requested: &Claim,
) -> Result<(), CertificateError> {
    if certificate.nodes.is_empty() {
        return Err(CertificateError::Empty);
    }

    for (node_index, node) in certificate.nodes.iter().enumerate() {
        match &node.reason {
            CertificateReason::Root { root_ref } => {
                let Some(root) = basis.roots.get(*root_ref as usize) else {
                    return Err(CertificateError::RootOutOfBounds {
                        node: node_index,
                        root_ref: *root_ref,
                    });
                };
                if node.claimed != *root {
                    return Err(CertificateError::RootClaimMismatch {
                        node: node_index,
                        root_ref: *root_ref,
                    });
                }
            }
            CertificateReason::Apply {
                rule_ref,
                premise_refs,
            } => {
                let Some(rule) = basis.rules.get(*rule_ref as usize) else {
                    return Err(CertificateError::RuleOutOfBounds {
                        node: node_index,
                        rule_ref: *rule_ref,
                    });
                };
                if premise_refs.len() != rule.premises.len() {
                    return Err(CertificateError::PremiseCountMismatch {
                        node: node_index,
                        expected: rule.premises.len(),
                        found: premise_refs.len(),
                    });
                }
                for (premise_index, (premise_ref, premise)) in
                    premise_refs.iter().zip(&rule.premises).enumerate()
                {
                    if premise_refs[premise_index + 1..].contains(premise_ref) {
                        return Err(CertificateError::DuplicatePremiseReference {
                            node: node_index,
                            premise_ref: *premise_ref,
                        });
                    }
                    let Some(prior) = certificate.nodes.get(*premise_ref as usize) else {
                        return Err(CertificateError::PremiseNotEarlier {
                            node: node_index,
                            premise: premise_index,
                            premise_ref: *premise_ref,
                        });
                    };
                    if *premise_ref as usize >= node_index {
                        return Err(CertificateError::PremiseNotEarlier {
                            node: node_index,
                            premise: premise_index,
                            premise_ref: *premise_ref,
                        });
                    }
                    if prior.claimed != *premise {
                        return Err(CertificateError::PremiseClaimMismatch {
                            node: node_index,
                            premise: premise_index,
                            premise_ref: *premise_ref,
                        });
                    }
                }
                if node.claimed != rule.conclusion {
                    return Err(CertificateError::RuleConclusionMismatch {
                        node: node_index,
                        rule_ref: *rule_ref,
                    });
                }
            }
        }
    }

    if certificate.nodes.last().map(|node| &node.claimed) != Some(requested) {
        return Err(CertificateError::TargetMismatch);
    }
    Ok(())
}

/// Construct the frozen injective admission claim from a successor's exact
/// INDEX and BASIS frame bytes.
#[must_use]
pub fn basis_admission_claim(successor: &CanonicalPackageCandidate) -> Claim {
    let mut payload =
        Vec::with_capacity(successor.exact_index_frame.len() + successor.exact_basis_frame.len());
    payload.extend_from_slice(&successor.exact_index_frame);
    payload.extend_from_slice(&successor.exact_basis_frame);
    Claim {
        term: admission_atom(payload),
        type_term: admission_atom(BASIS_ADMISSION_TYPE_PAYLOAD.to_vec()),
        mode: admission_atom(BASIS_ADMISSION_MODE_PAYLOAD.to_vec()),
    }
}

fn admission_atom(canonical_payload: Vec<u8>) -> Term {
    Term::Atom {
        kind: Blob(BASIS_ADMISSION_KIND.to_vec()),
        canonical_payload: Blob(canonical_payload),
        equality_contract: Blob(BASIS_ADMISSION_EQUALITY_CONTRACT.to_vec()),
    }
}

fn literal_bootstrap_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Box<[u8]>> = OnceLock::new();
    BYTES
        .get_or_init(|| {
            decode_hex_transport(LITERAL_BOOTSTRAP_HEX)
                .expect("the tracked literal bootstrap hex transport must be valid")
                .into_boxed_slice()
        })
        .as_ref()
}

fn decode_hex_transport(input: &str) -> Result<Vec<u8>, ()> {
    let digits: Vec<u8> = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if !digits.len().is_multiple_of(2) {
        return Err(());
    }
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).ok_or(())? as u8;
            let low = (pair[1] as char).to_digit(16).ok_or(())? as u8;
            Ok((high << 4) | low)
        })
        .collect()
}

/// Apply the closed v0 runtime authorization boundary.
///
/// A root must be byte-identical to the tracked literal bootstrap. A successor
/// is authorized only through the exact recursively authorized predecessor,
/// equal index, a basis-admission claim built from the successor's exact
/// frames, and a lineage certificate checked against the predecessor basis.
pub fn authorize(
    candidate: &CanonicalPackageCandidate,
) -> Result<V0AuthorizedPackage<'_>, AuthorizationError> {
    match &candidate.value.lineage {
        Lineage::Root => {
            if candidate.exact_bytes() != literal_bootstrap_bytes() {
                return Err(AuthorizationError::RootIsNotLiteralBootstrap);
            }
        }
        Lineage::Successor {
            predecessor_package,
            authorization,
        } => {
            let predecessor =
                decode(&predecessor_package.0).map_err(AuthorizationError::PredecessorDecode)?;
            authorize(&predecessor)
                .map_err(|error| AuthorizationError::PredecessorUnauthorized(Box::new(error)))?;
            if candidate.value.index.universe_id != predecessor.value.index.universe_id {
                return Err(AuthorizationError::UniverseMismatch);
            }
            if candidate.value.index.semantics_id != predecessor.value.index.semantics_id {
                return Err(AuthorizationError::SemanticsMismatch);
            }
            check_certificate(
                &predecessor.value.basis,
                authorization,
                &basis_admission_claim(candidate),
            )
            .map_err(AuthorizationError::LineageCertificate)?;
        }
    }

    check_certificate(
        &candidate.value.basis,
        &candidate.value.certificate,
        &candidate.value.target,
    )
    .map_err(AuthorizationError::PackageCertificate)?;
    Ok(V0AuthorizedPackage { package: candidate })
}
