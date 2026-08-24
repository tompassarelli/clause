use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(pub String);

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleName(pub String);

impl RoleName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DomainName(pub String);

impl DomainName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VariableName(pub String);

impl VariableName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Grounding,
    Enumeration,
    BindingShape,
    RelationShape,
    Model,
    DerivationRule,
    Revision,
    Delta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    /// Direct Model content whose semantic scope is supplied by the caller at
    /// compilation time rather than declared in this source projection.
    pub top_level: Vec<Member>,
    pub requests: Vec<RequestDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    pub subject: Spanned<Name>,
    pub kind: Kind,
    pub body: Vec<Member>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Member {
    Sentence(SentenceShapeDecl),
    LookupMode(ModeDecl),
    MembershipRange(MembershipRangeDecl),
    ShapeBinding(ShapeBindingDecl),
    Definition(DefinitionDecl),
    Membership(MembershipDecl),
    Focus(FocusBlock),
    RelationalContent(SurfaceClause),
    When(Vec<SurfaceClause>),
    From(Name),
    Apply(Name),
    Admit(Vec<SurfaceClause>),
    Withdraw(Vec<SurfaceClause>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionDecl {
    pub name: Spanned<Name>,
    pub denotation: Spanned<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeBindingDecl {
    pub label: Spanned<Name>,
    pub domain: Spanned<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipDecl {
    pub member: Spanned<Name>,
    pub group: Spanned<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceShapeDecl {
    pub parts: Vec<ShapePartDecl>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapePartDecl {
    Literal(Spanned<String>),
    Role {
        id: Spanned<RoleName>,
        domain: Spanned<DomainName>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cardinality {
    One,
    Maybe,
    Some,
    Many,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDecl {
    pub known: Vec<Spanned<RoleName>>,
    pub sought: Vec<Spanned<RoleName>>,
    pub cardinality: Cardinality,
    pub span: Span,
}

/// A closed, finite family of membership claims. This remains authored
/// surface structure until elaboration distributes it into ordinary
/// role-labelled membership content; parsing never expands the range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipRangeDecl {
    pub prefix: Spanned<String>,
    pub range: IntegerRange,
    pub suffix: Spanned<String>,
    pub group: Spanned<DomainName>,
    pub span: Span,
}

/// An inclusive integer interval written in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerRange {
    pub start: u64,
    pub end: u64,
    pub span: Span,
}

/// One correlated placeholder in a focus head. `prefix`, `variable`, and
/// `suffix` are deliberately retained rather than expanded or interpolated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferentTemplate {
    pub prefix: Spanned<String>,
    pub variable: Spanned<VariableName>,
    pub suffix: Spanned<String>,
    pub span: Span,
}

/// A correlated focus block is surface structure, not an implicit relation. The
/// later domain-directed elaborator alone chooses the role-labelled clause that
/// each slot denotes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusBlock {
    pub template: ReferentTemplate,
    pub slots: Vec<FocusSlot>,
    pub binding: FocusBinding,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusSlot {
    /// The literal sequence immediately after the focused role.  It is a
    /// sentence-shape prefix rather than a relation or role identity.
    pub label: Spanned<String>,
    pub value: SurfaceTerm,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusBinding {
    pub variable: Spanned<VariableName>,
    pub range: IntegerRange,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SurfaceTerm {
    Referent(Spanned<Name>),
    /// A bracketed referent identity correlated with a focus binder.  This is
    /// authoring-only structure and must be substituted before lowering.
    Template(ReferentTemplate),
    Variable(Spanned<VariableName>),
    String(Spanned<String>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceClause {
    pub relation: Spanned<Name>,
    pub roles: BTreeMap<RoleName, SurfaceTerm>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterventionSelection {
    OneMinimal,
    AllMinimal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestDecl {
    Find {
        revision: Spanned<Name>,
        pattern: SurfaceClause,
        sought: Spanned<VariableName>,
        span: Span,
    },
    Why {
        revision: Spanned<Name>,
        target: SurfaceClause,
        all: bool,
        span: Span,
    },
    Prevent {
        revision: Spanned<Name>,
        target: SurfaceClause,
        selection: InterventionSelection,
        using: Vec<Spanned<Name>>,
        span: Span,
    },
    Achieve {
        revision: Spanned<Name>,
        target: SurfaceClause,
        selection: InterventionSelection,
        using: Vec<Spanned<Name>>,
        span: Span,
    },
    Diff {
        base: Spanned<Name>,
        successor: Spanned<Name>,
        span: Span,
    },
}
