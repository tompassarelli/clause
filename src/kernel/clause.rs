use std::collections::BTreeMap;

use crate::wire::sha256_digest;

use super::{
    error::{KernelError, Result},
    identity::{ContentId, PatternId, ReferentId, RoleId},
};

/// A recursive resolved term. Pattern binders remain scoped machinery rather
/// than semantic referents.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Term {
    Referent(ReferentId),
    Pattern(PatternId),
    Application(ContentId),
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
        matches!(self, Self::Referent(_))
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
        match term {
            Term::Referent(id) => {
                bytes.push(b'r');
                field(&mut bytes, id.as_str());
            }
            Term::Pattern(id) => {
                bytes.push(b'p');
                field(&mut bytes, id.as_str());
            }
            Term::Application(id) => {
                bytes.push(b'a');
                field(&mut bytes, id.as_str());
            }
        }
    }
    bytes
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
