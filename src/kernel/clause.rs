use std::collections::BTreeMap;

use crate::wire::sha256_digest;

use super::{
    error::{KernelError, Result},
    identity::{ContentId, Name, PatternId, ReferentId, RoleId},
};

/// A recursive resolved term. Pattern binders remain scoped machinery rather
/// than semantic referents.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Term {
    Referent(ReferentId),
    Pattern(PatternId),
    Application(ContentId),
    F32(FiniteF32),
    Int(i64),
    Bool(bool),
    Product {
        shape: ReferentId,
        fields: BTreeMap<Name, ProductField>,
    },
    LabelledProduct {
        shape: ReferentId,
        fields: BTreeMap<ReferentId, Term>,
    },
    Sum {
        tag: Name,
        value: Box<Term>,
    },
    Sequence {
        shape: ReferentId,
        element: ReferentId,
        values: Vec<Term>,
    },
}

/// One tuple/product position with its exact expected representation domain.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProductField {
    domain: ReferentId,
    value: Term,
}

impl ProductField {
    pub fn new(domain: ReferentId, value: Term) -> Self {
        Self { domain, value }
    }

    pub fn domain(&self) -> &ReferentId {
        &self.domain
    }

    pub fn value(&self) -> &Term {
        &self.value
    }
}

/// The exact IEEE-754 binary32 bits of a finite structural number.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FiniteF32(u32);

impl FiniteF32 {
    pub fn from_bits(bits: u32) -> Result<Self> {
        if !f32::from_bits(bits).is_finite() {
            Err(KernelError::new("F32 term must be finite"))
        } else if bits == (-0.0_f32).to_bits() {
            Ok(Self(0.0_f32.to_bits()))
        } else {
            Ok(Self(bits))
        }
    }

    pub fn from_f32(value: f32) -> Result<Self> {
        Self::from_bits(value.to_bits())
    }

    pub fn bits(self) -> u32 {
        self.0
    }

    pub fn value(self) -> f32 {
        f32::from_bits(self.0)
    }
}

impl Term {
    pub fn referent(referent: ReferentId) -> Self {
        Self::Referent(referent)
    }
    pub fn pattern(id: PatternId) -> Self {
        Self::Pattern(id)
    }
    pub fn application(content: ContentId) -> Self {
        Self::Application(content)
    }
    pub fn f32(value: f32) -> Result<Self> {
        Ok(Self::F32(FiniteF32::from_f32(value)?))
    }
    pub fn f32_bits(bits: u32) -> Result<Self> {
        Ok(Self::F32(FiniteF32::from_bits(bits)?))
    }
    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }
    pub fn boolean(value: bool) -> Self {
        Self::Bool(value)
    }
    pub fn product(shape: ReferentId, fields: BTreeMap<Name, ProductField>) -> Result<Self> {
        if fields.is_empty() {
            return Err(KernelError::new(
                "structural product requires at least one field",
            ));
        }
        let term = Self::Product { shape, fields };
        term.validate_structure()?;
        Ok(term)
    }
    pub fn tuple(shape: ReferentId, values: Vec<(ReferentId, Term)>) -> Result<Self> {
        Self::product(
            shape,
            values
                .into_iter()
                .enumerate()
                .map(|(index, (domain, value))| {
                    (
                        Name::new(format!("_{index:020}"))
                            .expect("fixed-width ordinal tuple label is valid"),
                        ProductField::new(domain, value),
                    )
                })
                .collect(),
        )
    }
    pub fn labelled_product(shape: ReferentId, fields: BTreeMap<ReferentId, Term>) -> Result<Self> {
        if fields.is_empty() {
            return Err(KernelError::new(
                "labelled product requires at least one field",
            ));
        }
        let term = Self::LabelledProduct { shape, fields };
        term.validate_structure()?;
        Ok(term)
    }
    pub fn sum(tag: Name, value: Term) -> Result<Self> {
        let term = Self::Sum {
            tag,
            value: Box::new(value),
        };
        term.validate_structure()?;
        Ok(term)
    }
    pub fn sequence(shape: ReferentId, element: ReferentId, values: Vec<Term>) -> Result<Self> {
        let term = Self::Sequence {
            shape,
            element,
            values,
        };
        term.validate_structure()?;
        Ok(term)
    }
    pub fn walk(&self, visitor: &mut impl FnMut(&Term)) {
        visitor(self);
        match self {
            Self::Product { fields, .. } => {
                for field in fields.values() {
                    field.value.walk(visitor);
                }
            }
            Self::LabelledProduct { fields, .. } => {
                for value in fields.values() {
                    value.walk(visitor);
                }
            }
            Self::Sum { value, .. } => value.walk(visitor),
            Self::Sequence { values, .. } => {
                for value in values {
                    value.walk(visitor);
                }
            }
            Self::Referent(_)
            | Self::Pattern(_)
            | Self::Application(_)
            | Self::F32(_)
            | Self::Int(_)
            | Self::Bool(_) => {}
        }
    }
    pub fn validate_structure(&self) -> Result<()> {
        fn validate_member(term: &Term) -> Result<()> {
            term.validate_structure()
        }

        match self {
            Self::Product { fields, .. } => fields
                .values()
                .map(ProductField::value)
                .try_for_each(validate_member),
            Self::LabelledProduct { fields, .. } => fields.values().try_for_each(validate_member),
            Self::Sum { value, .. } => validate_member(value),
            Self::Sequence { values, .. } => values.iter().try_for_each(validate_member),
            Self::Referent(_)
            | Self::Pattern(_)
            | Self::Application(_)
            | Self::F32(_)
            | Self::Int(_)
            | Self::Bool(_) => Ok(()),
        }
    }
    pub fn referent_id(&self) -> Option<&ReferentId> {
        match self {
            Self::Referent(id) => Some(id),
            _ => None,
        }
    }
    pub fn pattern_id(&self) -> Option<&PatternId> {
        match self {
            Self::Pattern(id) => Some(id),
            _ => None,
        }
    }
    pub fn content_id(&self) -> Option<&ContentId> {
        match self {
            Self::Application(id) => Some(id),
            _ => None,
        }
    }
    /// Whether groundness is decidable from this term alone. Applications
    /// require a Model lookup and are conservatively non-ground here.
    pub fn is_ground(&self) -> bool {
        match self {
            Self::Referent(_) | Self::F32(_) | Self::Int(_) | Self::Bool(_) => true,
            Self::Pattern(_) | Self::Application(_) => false,
            Self::Product { fields, .. } => fields.values().all(|field| field.value.is_ground()),
            Self::LabelledProduct { fields, .. } => fields.values().all(Self::is_ground),
            Self::Sum { value, .. } => value.is_ground(),
            Self::Sequence { values, .. } => values.iter().all(Self::is_ground),
        }
    }
}

/// Canonically identified n-ary relational content. It carries no occurrence
/// or judgment authority.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationalContent {
    id: ContentId,
    relation: ReferentId,
    roles: BTreeMap<RoleId, Term>,
}

/// One nonempty, canonical collection of relational forms interpreted
/// together by a semantic mode.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pattern {
    forms: Vec<ContentId>,
}

impl Pattern {
    pub fn new(mut forms: Vec<ContentId>) -> Result<Self> {
        if forms.is_empty() {
            return Err(KernelError::new(
                "pattern needs at least one relational form",
            ));
        }
        forms.sort();
        forms.dedup();
        Ok(Self { forms })
    }

    pub fn forms(&self) -> &[ContentId] {
        &self.forms
    }
}

impl RelationalContent {
    pub fn new(relation: ReferentId, roles: BTreeMap<RoleId, Term>) -> Result<Self> {
        if roles.is_empty() {
            return Err(KernelError::new("relational content has no roles"));
        }
        let id = ContentId::from_digest(sha256_digest(&content_preimage(&relation, &roles)));
        Ok(Self {
            id,
            relation,
            roles,
        })
    }

    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn relation(&self) -> &ReferentId {
        &self.relation
    }
    pub fn roles(&self) -> &BTreeMap<RoleId, Term> {
        &self.roles
    }
    pub fn is_ground(&self) -> bool {
        self.roles.values().all(Term::is_ground)
    }
}

fn content_preimage(relation: &ReferentId, roles: &BTreeMap<RoleId, Term>) -> Vec<u8> {
    let mut bytes = b"clause-relational-content-v1\0".to_vec();
    field(&mut bytes, relation.as_str());
    for (role, term) in roles {
        field(&mut bytes, role.as_str());
        term_preimage(&mut bytes, term);
    }
    bytes
}

fn term_preimage(bytes: &mut Vec<u8>, term: &Term) {
    match term {
        Term::Referent(id) => {
            bytes.push(b'r');
            field(bytes, id.as_str());
        }
        Term::Pattern(id) => {
            bytes.push(b'p');
            field(bytes, id.as_str());
        }
        Term::Application(id) => {
            bytes.push(b'a');
            field(bytes, id.as_str());
        }
        Term::F32(value) => {
            bytes.push(b'f');
            bytes.extend_from_slice(&value.bits().to_be_bytes());
        }
        Term::Int(value) => {
            bytes.push(b'i');
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        Term::Bool(value) => {
            bytes.push(b'b');
            bytes.push(u8::from(*value));
        }
        Term::Product { shape, fields } => {
            bytes.push(b'P');
            field(bytes, shape.as_str());
            bytes.extend_from_slice(&(fields.len() as u64).to_be_bytes());
            for (label, product_field) in fields {
                field(bytes, label.as_str());
                field(bytes, product_field.domain.as_str());
                term_preimage(bytes, &product_field.value);
            }
        }
        Term::LabelledProduct { shape, fields } => {
            bytes.push(b'L');
            field(bytes, shape.as_str());
            bytes.extend_from_slice(&(fields.len() as u64).to_be_bytes());
            for (field_id, value) in fields {
                field(bytes, field_id.as_str());
                term_preimage(bytes, value);
            }
        }
        Term::Sum { tag, value } => {
            bytes.push(b'S');
            field(bytes, tag.as_str());
            term_preimage(bytes, value);
        }
        Term::Sequence {
            shape,
            element,
            values,
        } => {
            bytes.push(b'Q');
            field(bytes, shape.as_str());
            field(bytes, element.as_str());
            bytes.extend_from_slice(&(values.len() as u64).to_be_bytes());
            for value in values {
                term_preimage(bytes, value);
            }
        }
    }
}

fn field(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// One scoped source act referring to separately canonicalized content.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AssertionOccurrence {
    id: ReferentId,
    content: ContentId,
    source: ReferentId,
    scope: ReferentId,
}

impl AssertionOccurrence {
    pub fn new(id: ReferentId, content: ContentId, source: ReferentId, scope: ReferentId) -> Self {
        Self {
            id,
            content,
            source,
            scope,
        }
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn content(&self) -> &ContentId {
        &self.content
    }
    pub fn source(&self) -> &ReferentId {
        &self.source
    }
    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DerivationRule {
    id: ReferentId,
    scope: ReferentId,
    authority: ReferentId,
    premises: Pattern,
    conclusion: Pattern,
}

impl DerivationRule {
    pub fn new(
        id: ReferentId,
        scope: ReferentId,
        authority: ReferentId,
        premises: Pattern,
        conclusion: Pattern,
    ) -> Result<Self> {
        Ok(Self {
            id,
            scope,
            authority,
            premises,
            conclusion,
        })
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }
    pub fn authority(&self) -> &ReferentId {
        &self.authority
    }
    pub fn premises(&self) -> &Pattern {
        &self.premises
    }
    pub fn conclusion(&self) -> &Pattern {
        &self.conclusion
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UniversalLaw {
    id: ReferentId,
    scope: ReferentId,
    generalized: Pattern,
}
impl UniversalLaw {
    pub fn new(id: ReferentId, scope: ReferentId, generalized: Pattern) -> Self {
        Self {
            id,
            scope,
            generalized,
        }
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }
    pub fn generalized(&self) -> &Pattern {
        &self.generalized
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InvariantAdmission {
    RejectOnMatch,
    RequireMatch,
}

impl InvariantAdmission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RejectOnMatch => "reject-on-match",
            Self::RequireMatch => "require-match",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Invariant {
    id: ReferentId,
    scope: ReferentId,
    policy: ReferentId,
    condition: Pattern,
    admission: InvariantAdmission,
}

impl Invariant {
    pub fn new(
        id: ReferentId,
        scope: ReferentId,
        policy: ReferentId,
        condition: Pattern,
        admission: InvariantAdmission,
    ) -> Self {
        Self {
            id,
            scope,
            policy,
            condition,
            admission,
        }
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }
    pub fn policy(&self) -> &ReferentId {
        &self.policy
    }
    pub fn condition(&self) -> &Pattern {
        &self.condition
    }
    pub fn admission(&self) -> &InvariantAdmission {
        &self.admission
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Goal {
    id: ReferentId,
    context: ReferentId,
    desired: Pattern,
}
impl Goal {
    pub fn new(id: ReferentId, context: ReferentId, desired: Pattern) -> Self {
        Self {
            id,
            context,
            desired,
        }
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn context(&self) -> &ReferentId {
        &self.context
    }
    pub fn desired(&self) -> &Pattern {
        &self.desired
    }
}

/// A definition orients one stable referent toward a recursive denoting term.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Definition {
    id: ReferentId,
    denotation: Term,
}
impl Definition {
    pub fn new(id: ReferentId, denotation: Term) -> Self {
        Self { id, denotation }
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn denotation(&self) -> &Term {
        &self.denotation
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Transition {
    id: ReferentId,
    from: ContentId,
    to: ContentId,
}
impl Transition {
    pub fn new(id: ReferentId, from: ContentId, to: ContentId) -> Result<Self> {
        if from == to {
            return Err(KernelError::new(
                "transition must change relational content",
            ));
        }
        Ok(Self { id, from, to })
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn from(&self) -> &ContentId {
        &self.from
    }
    pub fn to(&self) -> &ContentId {
        &self.to
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum JudgmentTarget {
    Content(ContentId),
    Occurrence(ReferentId),
}

/// The semantic basis of a judgment. Each variant carries the evidence needed
/// for its authority and is not interchangeable with another mood.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum JudgmentKind {
    Declared,
    Derived {
        rule: ReferentId,
        premises: Vec<ContentId>,
    },
    Observed {
        evidence: ReferentId,
    },
    Admitted {
        policy: ReferentId,
        basis: Vec<ReferentId>,
    },
    Rejected {
        policy: ReferentId,
        basis: Vec<ReferentId>,
    },
    Superseded {
        by: ReferentId,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum JudgmentStatus {
    Affirmed,
    Disputed,
    Withdrawn,
}
impl JudgmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Affirmed => "affirmed",
            Self::Disputed => "disputed",
            Self::Withdrawn => "withdrawn",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Judgment {
    id: ReferentId,
    authority: ReferentId,
    scope: ReferentId,
    target: JudgmentTarget,
    kind: JudgmentKind,
    status: JudgmentStatus,
}

impl Judgment {
    pub fn new(
        id: ReferentId,
        authority: ReferentId,
        scope: ReferentId,
        target: JudgmentTarget,
        mut kind: JudgmentKind,
        status: JudgmentStatus,
    ) -> Self {
        match &mut kind {
            JudgmentKind::Derived { premises, .. } => {
                premises.sort();
                premises.dedup();
            }
            JudgmentKind::Admitted { basis, .. } | JudgmentKind::Rejected { basis, .. } => {
                basis.sort();
                basis.dedup();
            }
            JudgmentKind::Declared
            | JudgmentKind::Observed { .. }
            | JudgmentKind::Superseded { .. } => {}
        }
        Self {
            id,
            authority,
            scope,
            target,
            kind,
            status,
        }
    }
    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn authority(&self) -> &ReferentId {
        &self.authority
    }
    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }
    pub fn target(&self) -> &JudgmentTarget {
        &self.target
    }
    pub fn kind(&self) -> &JudgmentKind {
        &self.kind
    }
    pub fn status(&self) -> &JudgmentStatus {
        &self.status
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenWorldStatus {
    Admitted,
    Rejected,
    Disputed,
    Undetermined,
}
