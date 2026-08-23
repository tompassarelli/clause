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
pub struct TypeName(pub String);

impl TypeName {
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
    Type,
    Relation,
    Model,
    Law,
    Revision,
    Delta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub declarations: Vec<AscriptionDecl>,
    pub requests: Vec<RequestDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AscriptionDecl {
    pub subject: Spanned<Name>,
    pub kind: Kind,
    pub body: Vec<Member>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Member {
    Sentence(SentenceShapeDecl),
    Mode(ModeDecl),
    Entity(EntityDecl),
    EntityGroup(EntityGroupDecl),
    Focus(FocusBlock),
    Clause(SurfaceClause),
    When(Vec<SurfaceClause>),
    From(Name),
    Apply(Name),
    Admit(Vec<SurfaceClause>),
    Withdraw(Vec<SurfaceClause>),
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
        typ: Spanned<TypeName>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityDecl {
    pub local: Spanned<Name>,
    pub typ: Spanned<TypeName>,
    pub span: Span,
}

/// A closed, finite family of semantic identities. This remains authored
/// surface data until the focus elaborator distributes it into ordinary
/// entities; parsing never expands the range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityGroupDecl {
    pub prefix: Spanned<String>,
    pub range: IntegerRange,
    pub suffix: Spanned<String>,
    pub typ: Spanned<TypeName>,
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
pub struct EntityTemplate {
    pub prefix: Spanned<String>,
    pub variable: Spanned<VariableName>,
    pub suffix: Spanned<String>,
    pub span: Span,
}

/// A typed-focus block is surface structure, not an implicit relation. The
/// later type-directed elaborator alone chooses the role-labelled clause that
/// each slot denotes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusBlock {
    pub template: EntityTemplate,
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
    Entity(Spanned<Name>),
    /// A bracketed entity identity correlated with a focus binder.  This is
    /// authoring-only structure and must be substituted before lowering.
    Template(EntityTemplate),
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
