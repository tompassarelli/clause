//! Clause's typed semantic kernel.
//!
//! The kernel admits semantic values only.  Parsing, revision aliases, wire
//! representation, and requests live outside this module.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelError(String);

impl KernelError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for KernelError {}

pub type Result<T> = std::result::Result<T, KernelError>;

/// A qualified Clause name. Roles and variables remain strict local segments;
/// entity locals use the explicit `Name::entity_local` constructor because
/// human-facing labels may contain spaces.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

impl Name {
    pub fn new(value: String) -> Result<Self> {
        if valid_name(&value) {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid name"))
        }
    }

    /// Construct the local identity of an entity.
    ///
    /// Entity locals are the one semantic identifier that may contain spaces:
    /// a displayed label such as `Zone 7` is still one stable identity, not a
    /// pair of names. Every other identifier continues to use `Name::new` and
    /// therefore retains the strict segment grammar.
    pub fn entity_local(value: String) -> Result<Self> {
        if valid_entity_local(&value) {
            Ok(Self(value))
        } else {
            Err(KernelError::new("invalid entity local name"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_local(&self) -> bool {
        !self.0.contains('/')
    }

    fn is_strict(&self) -> bool {
        valid_name(&self.0)
    }

    fn is_entity_local(&self) -> bool {
        valid_entity_local(&self.0)
    }
}

macro_rules! identity {
    ($name:ident, $message:literal) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(Name);

        impl $name {
            pub fn new(name: Name) -> Result<Self> {
                if name.is_strict() {
                    Ok(Self(name))
                } else {
                    Err(KernelError::new($message))
                }
            }

            pub fn name(&self) -> &Name {
                &self.0
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

identity!(TypeId, "invalid type identity");
identity!(ModelId, "invalid model identity");
identity!(RelationId, "invalid relation identity");
identity!(LawId, "invalid law identity");

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleId(Name);

impl RoleId {
    pub fn new(name: Name) -> Result<Self> {
        if name.is_local() && name.is_strict() {
            Ok(Self(name))
        } else {
            Err(KernelError::new("role identity must be a local name"))
        }
    }

    pub fn name(&self) -> &Name {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VariableId(Name);

impl VariableId {
    pub fn new(name: Name) -> Result<Self> {
        if name.is_local() && name.is_strict() {
            Ok(Self(name))
        } else {
            Err(KernelError::new("variable identity must be a local name"))
        }
    }

    pub fn name(&self) -> &Name {
        &self.0
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// The content-addressed identity of one admitted model revision.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionId([u8; 32]);

impl RevisionId {
    pub(crate) fn from_digest(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("rev-sha256-")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityId {
    model: ModelId,
    local: Name,
    typ: TypeId,
}

impl EntityId {
    pub fn new(model: ModelId, local: Name, typ: TypeId) -> Result<Self> {
        if !local.is_entity_local() {
            return Err(KernelError::new("invalid entity local name"));
        }
        Ok(Self { model, local, typ })
    }

    pub fn model(&self) -> &ModelId {
        &self.model
    }

    pub fn local(&self) -> &Name {
        &self.local
    }

    pub fn typ(&self) -> &TypeId {
        &self.typ
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    id: TypeId,
}

impl Type {
    pub fn new(id: TypeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &TypeId {
        &self.id
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Role {
    id: RoleId,
    typ: TypeId,
}

impl Role {
    pub fn new(id: RoleId, typ: TypeId) -> Self {
        Self { id, typ }
    }

    pub fn id(&self) -> &RoleId {
        &self.id
    }

    pub fn typ(&self) -> &TypeId {
        &self.typ
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cardinality {
    One,
    Maybe,
    Some,
    Many,
}

impl Cardinality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Maybe => "maybe",
            Self::Some => "some",
            Self::Many => "many",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mode {
    known: Vec<RoleId>,
    sought: Vec<RoleId>,
    cardinality: Cardinality,
}

impl Mode {
    pub fn finite(
        known: Vec<RoleId>,
        sought: Vec<RoleId>,
        cardinality: Cardinality,
    ) -> Result<Self> {
        let known = sorted_unique(known, "known role")?;
        let sought = sorted_unique(sought, "sought role")?;
        if known.is_empty()
            || sought.is_empty()
            || known.iter().any(|role| sought.binary_search(role).is_ok())
        {
            return Err(KernelError::new(
                "mode must have disjoint nonempty known and sought roles",
            ));
        }
        Ok(Self {
            known,
            sought,
            cardinality,
        })
    }

    pub fn known(&self) -> &[RoleId] {
        &self.known
    }

    pub fn sought(&self) -> &[RoleId] {
        &self.sought
    }

    pub fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SentencePart {
    Literal(String),
    Role(RoleId),
}

/// One inline shape.  Role types travel with the shape until `Relation::new`
/// derives the Relation role map; the public parts remain the semantic n-ary
/// sentence pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceShape {
    parts: Vec<SentencePart>,
    inline_roles: BTreeMap<RoleId, Role>,
}

impl SentenceShape {
    pub fn new(parts: Vec<InlineSentencePart>) -> Result<Self> {
        if parts.len() < 3
            || !matches!(parts.first(), Some(InlineSentencePart::Role(_)))
            || !matches!(parts.last(), Some(InlineSentencePart::Role(_)))
        {
            return Err(KernelError::new(
                "sentence shape must begin and end with a role and contain a literal",
            ));
        }
        let mut inline_roles = BTreeMap::new();
        let mut canonical = Vec::with_capacity(parts.len());
        let mut role_count = 0;
        let mut previous_was_role = false;
        for part in parts {
            match part {
                InlineSentencePart::Role(role) => {
                    if !previous_was_role && !canonical.is_empty() {
                        // A role after a literal is valid.
                    } else if previous_was_role {
                        return Err(KernelError::new(
                            "sentence roles need a literal between them",
                        ));
                    }
                    if inline_roles.insert(role.id.clone(), role.clone()).is_some() {
                        return Err(KernelError::new("duplicate inline relation role"));
                    }
                    canonical.push(SentencePart::Role(role.id));
                    role_count += 1;
                    previous_was_role = true;
                }
                InlineSentencePart::Literal(literal) => {
                    if !previous_was_role {
                        return Err(KernelError::new("sentence literals must follow a role"));
                    }
                    canonical.push(SentencePart::Literal(canonical_literal(literal)?));
                    previous_was_role = false;
                }
            }
        }
        if role_count < 2 {
            return Err(KernelError::new("relation needs at least two inline roles"));
        }
        Ok(Self {
            parts: canonical,
            inline_roles,
        })
    }

    pub fn parts(&self) -> &[SentencePart] {
        &self.parts
    }

    fn roles(&self) -> &BTreeMap<RoleId, Role> {
        &self.inline_roles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InlineSentencePart {
    Literal(String),
    Role(Role),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    id: RelationId,
    roles: BTreeMap<RoleId, Role>,
    shape: SentenceShape,
    modes: Vec<Mode>,
}

impl Relation {
    pub fn new(id: RelationId, shape: SentenceShape, mut modes: Vec<Mode>) -> Result<Self> {
        let roles = shape.roles().clone();
        for mode in &modes {
            let covered = mode
                .known()
                .iter()
                .chain(mode.sought())
                .cloned()
                .collect::<BTreeSet<_>>();
            if roles.keys().cloned().collect::<BTreeSet<_>>() != covered {
                return Err(KernelError::new("mode must classify every relation role"));
            }
        }
        modes.sort();
        modes.dedup();
        if modes.is_empty() {
            return Err(KernelError::new("relation needs a declared mode"));
        }
        Ok(Self {
            id,
            roles,
            shape,
            modes,
        })
    }

    pub fn id(&self) -> &RelationId {
        &self.id
    }

    pub fn roles(&self) -> &BTreeMap<RoleId, Role> {
        &self.roles
    }

    pub fn shape(&self) -> &SentenceShape {
        &self.shape
    }

    pub fn modes(&self) -> &[Mode] {
        &self.modes
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Term {
    Entity(EntityId),
    Value { typ: TypeId, canonical: String },
    Variable { id: VariableId, typ: TypeId },
}

impl Term {
    pub fn entity(entity: EntityId) -> Self {
        Self::Entity(entity)
    }

    pub fn value(typ: TypeId, canonical: String) -> Result<Self> {
        if typ.as_str() != "Text" {
            return Err(KernelError::new(
                "only the admitted Text type may carry scalar values",
            ));
        }
        if canonical.is_empty() || canonical.chars().any(char::is_control) {
            return Err(KernelError::new("invalid canonical Text value"));
        }
        Ok(Self::Value { typ, canonical })
    }

    pub fn variable(id: VariableId, typ: TypeId) -> Self {
        Self::Variable { id, typ }
    }

    pub fn typ(&self) -> &TypeId {
        match self {
            Self::Entity(entity) => entity.typ(),
            Self::Value { typ, .. } | Self::Variable { typ, .. } => typ,
        }
    }

    pub fn variable_id(&self) -> Option<&VariableId> {
        match self {
            Self::Variable { id, .. } => Some(id),
            Self::Entity(_) | Self::Value { .. } => None,
        }
    }

    pub fn is_ground(&self) -> bool {
        self.variable_id().is_none()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Clause {
    relation: RelationId,
    roles: BTreeMap<RoleId, Term>,
}

impl Clause {
    pub fn new(relation: RelationId, roles: BTreeMap<RoleId, Term>) -> Result<Self> {
        if roles.is_empty() {
            return Err(KernelError::new("clause has no roles"));
        }
        Ok(Self { relation, roles })
    }

    pub fn relation(&self) -> &RelationId {
        &self.relation
    }

    pub fn roles(&self) -> &BTreeMap<RoleId, Term> {
        &self.roles
    }

    pub fn is_ground(&self) -> bool {
        self.roles.values().all(Term::is_ground)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Law {
    id: LawId,
    premises: Vec<Clause>,
    conclusion: Clause,
}

impl Law {
    pub fn new(id: LawId, premises: Vec<Clause>, conclusion: Clause) -> Result<Self> {
        if premises.is_empty() {
            return Err(KernelError::new("law needs at least one premise"));
        }
        Ok(Self {
            id,
            premises,
            conclusion,
        })
    }

    pub fn id(&self) -> &LawId {
        &self.id
    }

    pub fn premises(&self) -> &[Clause] {
        &self.premises
    }

    pub fn conclusion(&self) -> &Clause {
        &self.conclusion
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    id: ModelId,
    types: BTreeMap<TypeId, Type>,
    entities: BTreeSet<EntityId>,
    relations: BTreeMap<RelationId, Relation>,
    assertions: Vec<Clause>,
    laws: Vec<Law>,
}

impl Model {
    pub fn new(
        id: ModelId,
        types: BTreeMap<TypeId, Type>,
        entities: BTreeSet<EntityId>,
        relations: BTreeMap<RelationId, Relation>,
        mut assertions: Vec<Clause>,
        mut laws: Vec<Law>,
    ) -> Result<Self> {
        if types.iter().any(|(identity, typ)| typ.id() != identity) {
            return Err(KernelError::new(
                "type map key must match its Type identity",
            ));
        }
        if entities
            .iter()
            .any(|entity| entity.model() != &id || !types.contains_key(entity.typ()))
        {
            return Err(KernelError::new(
                "entity must belong to this model and declare an admitted type",
            ));
        }
        if relations
            .iter()
            .any(|(identity, relation)| relation.id() != identity)
        {
            return Err(KernelError::new(
                "relation map key must match its Relation identity",
            ));
        }
        for relation in relations.values() {
            if relation
                .roles()
                .values()
                .any(|role| !types.contains_key(role.typ()))
            {
                return Err(KernelError::new("relation role has an undeclared type"));
            }
        }
        for assertion in &assertions {
            validate_clause(&id, &types, &entities, &relations, assertion, false)?;
        }
        assertions.sort();
        assertions.dedup();
        let mut law_ids = BTreeSet::new();
        for law in &laws {
            if !law_ids.insert(law.id().clone()) {
                return Err(KernelError::new("duplicate law identity"));
            }
            validate_law(&id, &types, &entities, &relations, law)?;
        }
        laws.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(Self {
            id,
            types,
            entities,
            relations,
            assertions,
            laws,
        })
    }

    pub fn id(&self) -> &ModelId {
        &self.id
    }

    pub fn types(&self) -> &BTreeMap<TypeId, Type> {
        &self.types
    }

    pub fn entities(&self) -> &BTreeSet<EntityId> {
        &self.entities
    }

    pub fn relations(&self) -> &BTreeMap<RelationId, Relation> {
        &self.relations
    }

    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }

    pub fn laws(&self) -> &[Law] {
        &self.laws
    }

    pub fn validate_clause(&self, clause: &Clause, allow_variables: bool) -> Result<()> {
        validate_clause(
            &self.id,
            &self.types,
            &self.entities,
            &self.relations,
            clause,
            allow_variables,
        )
    }

    /// Rebuild this semantic model with a replacement asserted-clause set.
    /// Delta application uses this named operation so the remaining model
    /// fields cannot be accidentally reordered during a breaking migration.
    pub fn with_assertions(&self, assertions: Vec<Clause>) -> Result<Self> {
        Self::new(
            self.id.clone(),
            self.types.clone(),
            self.entities.clone(),
            self.relations.clone(),
            assertions,
            self.laws.clone(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revision {
    identity: RevisionId,
    model: Model,
}

impl Revision {
    /// Wire admission owns semantic hashing and is the only module that pairs
    /// a checked digest with its admitted model.
    pub(crate) fn reloaded(identity: RevisionId, model: Model) -> Self {
        Self { identity, model }
    }

    pub fn identity(&self) -> &RevisionId {
        &self.identity
    }

    pub fn model(&self) -> &Model {
        &self.model
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delta {
    base: RevisionId,
    admissions: Vec<Clause>,
    withdrawals: Vec<Clause>,
}

impl Delta {
    pub fn new(
        base: RevisionId,
        mut admissions: Vec<Clause>,
        mut withdrawals: Vec<Clause>,
    ) -> Result<Self> {
        if admissions.is_empty() && withdrawals.is_empty() {
            return Err(KernelError::new("delta needs an admission or withdrawal"));
        }
        admissions.sort();
        withdrawals.sort();
        if admissions.windows(2).any(|pair| pair[0] == pair[1])
            || withdrawals.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(KernelError::new("delta changes cannot contain duplicates"));
        }
        if admissions
            .iter()
            .any(|clause| withdrawals.binary_search(clause).is_ok())
        {
            return Err(KernelError::new("delta admissions and withdrawals overlap"));
        }
        if !admissions.iter().chain(&withdrawals).all(Clause::is_ground) {
            return Err(KernelError::new("delta changes must be ground clauses"));
        }
        Ok(Self {
            base,
            admissions,
            withdrawals,
        })
    }

    pub fn base(&self) -> &RevisionId {
        &self.base
    }

    pub fn admissions(&self) -> &[Clause] {
        &self.admissions
    }

    pub fn withdrawals(&self) -> &[Clause] {
        &self.withdrawals
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindPlan {
    pattern: Clause,
    relation: RelationId,
    known: Vec<RoleId>,
    sought: RoleId,
    mode: Mode,
}

impl FindPlan {
    pub fn new(model: &Model, pattern: &Clause, sought: VariableId) -> Result<Self> {
        model.validate_clause(pattern, true)?;
        let relation = model
            .relations()
            .get(pattern.relation())
            .expect("validated clause relation is declared");
        let mut sought_roles = pattern
            .roles()
            .iter()
            .filter_map(|(role, term)| (term.variable_id() == Some(&sought)).then(|| role.clone()))
            .collect::<Vec<_>>();
        if sought_roles.len() != 1
            || pattern
                .roles()
                .values()
                .any(|term| term.variable_id().is_some_and(|id| id != &sought))
        {
            return Err(KernelError::new(
                "find pattern must contain exactly one sought variable",
            ));
        }
        let known = pattern
            .roles()
            .iter()
            .filter_map(|(role, term)| term.is_ground().then(|| role.clone()))
            .collect::<Vec<_>>();
        let sought_role = sought_roles.remove(0);
        let mode = relation
            .modes()
            .iter()
            .find(|mode| mode.known() == known && mode.sought() == [sought_role.clone()])
            .cloned()
            .ok_or_else(|| KernelError::new("no declared mode admits this find orientation"))?;
        Ok(Self {
            pattern: pattern.clone(),
            relation: pattern.relation().clone(),
            known,
            sought: sought_role,
            mode,
        })
    }

    pub fn pattern(&self) -> &Clause {
        &self.pattern
    }

    pub fn relation(&self) -> &RelationId {
        &self.relation
    }

    pub fn known(&self) -> &[RoleId] {
        &self.known
    }

    pub fn sought(&self) -> &RoleId {
        &self.sought
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }
}

fn validate_clause(
    model: &ModelId,
    types: &BTreeMap<TypeId, Type>,
    entities: &BTreeSet<EntityId>,
    relations: &BTreeMap<RelationId, Relation>,
    clause: &Clause,
    allow_variables: bool,
) -> Result<()> {
    let relation = relations
        .get(clause.relation())
        .ok_or_else(|| KernelError::new("clause relation is undeclared"))?;
    if clause.roles().keys().ne(relation.roles().keys()) {
        return Err(KernelError::new(
            "clause must fill the complete named role map",
        ));
    }
    for (role_id, term) in clause.roles() {
        let role = relation
            .roles()
            .get(role_id)
            .expect("complete role map was checked");
        if term.typ() != role.typ() || !types.contains_key(term.typ()) {
            return Err(KernelError::new(
                "clause term type does not match its role type",
            ));
        }
        match term {
            Term::Entity(entity) if entity.model() != model || !entities.contains(entity) => {
                return Err(KernelError::new(
                    "clause entity is not admitted by this model",
                ));
            }
            Term::Value { typ, canonical }
                if typ.as_str() != "Text"
                    || canonical.is_empty()
                    || canonical.chars().any(char::is_control) =>
            {
                return Err(KernelError::new(
                    "clause scalar values must be canonical Text",
                ));
            }
            Term::Variable { .. } if !allow_variables => {
                return Err(KernelError::new(
                    "assertions and delta changes must be ground",
                ));
            }
            Term::Entity(_) | Term::Value { .. } | Term::Variable { .. } => {}
        }
    }
    Ok(())
}

fn validate_law(
    model: &ModelId,
    types: &BTreeMap<TypeId, Type>,
    entities: &BTreeSet<EntityId>,
    relations: &BTreeMap<RelationId, Relation>,
    law: &Law,
) -> Result<()> {
    let mut premise_variables = BTreeSet::new();
    let mut variable_types = BTreeMap::new();
    for premise in law.premises() {
        validate_clause(model, types, entities, relations, premise, true)?;
        record_variables(premise, &mut variable_types, Some(&mut premise_variables))?;
    }
    validate_clause(model, types, entities, relations, law.conclusion(), true)?;
    record_variables(law.conclusion(), &mut variable_types, None)?;
    if law
        .conclusion()
        .roles()
        .values()
        .filter_map(Term::variable_id)
        .any(|variable| !premise_variables.contains(variable))
    {
        return Err(KernelError::new(
            "every conclusion variable must occur in a premise",
        ));
    }
    Ok(())
}

fn record_variables(
    clause: &Clause,
    variable_types: &mut BTreeMap<VariableId, TypeId>,
    mut variables: Option<&mut BTreeSet<VariableId>>,
) -> Result<()> {
    for term in clause.roles().values() {
        let Some(variable) = term.variable_id() else {
            continue;
        };
        if variable_types
            .insert(variable.clone(), term.typ().clone())
            .is_some_and(|previous| previous != *term.typ())
        {
            return Err(KernelError::new(
                "law variable occurs at inconsistent declared role types",
            ));
        }
        if let Some(variables) = variables.as_deref_mut() {
            variables.insert(variable.clone());
        }
    }
    Ok(())
}

fn sorted_unique<T: Ord>(mut values: Vec<T>, where_: &str) -> Result<Vec<T>> {
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(KernelError::new(format!("duplicate {where_}")));
    }
    Ok(values)
}

fn canonical_literal(value: String) -> Result<String> {
    let literal = value.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    if literal.is_empty() {
        Err(KernelError::new("sentence literal cannot be empty"))
    } else {
        Ok(literal)
    }
}

fn valid_segment(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn valid_name(value: &str) -> bool {
    !value.is_empty() && value.split('/').all(valid_segment)
}

fn valid_entity_local(value: &str) -> bool {
    !value.is_empty()
        && value.split(' ').all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(value: &str) -> Name {
        Name::new(value.to_owned()).unwrap()
    }

    fn type_id(value: &str) -> TypeId {
        TypeId::new(name(value)).unwrap()
    }

    fn role(value: &str, typ: &TypeId) -> Role {
        Role::new(RoleId::new(name(value)).unwrap(), typ.clone())
    }

    fn relation(id: &RelationId, left: &TypeId, right: &TypeId) -> Relation {
        let left_role = role("left", left);
        let right_role = role("right", right);
        Relation::new(
            id.clone(),
            SentenceShape::new(vec![
                InlineSentencePart::Role(left_role.clone()),
                InlineSentencePart::Literal(" relates   to ".to_owned()),
                InlineSentencePart::Role(right_role.clone()),
            ])
            .unwrap(),
            vec![
                Mode::finite(
                    vec![left_role.id().clone()],
                    vec![right_role.id().clone()],
                    Cardinality::Many,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn clause(relation: &RelationId, left: Term, right: Term) -> Clause {
        Clause::new(
            relation.clone(),
            BTreeMap::from([
                (RoleId::new(name("left")).unwrap(), left),
                (RoleId::new(name("right")).unwrap(), right),
            ]),
        )
        .unwrap()
    }

    fn model(laws: Vec<Law>) -> Result<Model> {
        let model_id = ModelId::new(name("catalog")).unwrap();
        let text = type_id("Text");
        let number = type_id("Number");
        let relation_id = RelationId::new(name("catalog/text")).unwrap();
        Model::new(
            model_id,
            BTreeMap::from([
                (text.clone(), Type::new(text.clone())),
                (number.clone(), Type::new(number)),
            ]),
            BTreeSet::new(),
            BTreeMap::from([(relation_id.clone(), relation(&relation_id, &text, &text))]),
            Vec::new(),
            laws,
        )
    }

    fn variable(value: &str, typ: &TypeId) -> Term {
        Term::variable(VariableId::new(name(value)).unwrap(), typ.clone())
    }

    #[test]
    fn inline_shape_derives_roles_and_canonicalizes_literals() {
        let text = type_id("Text");
        let relation_id = RelationId::new(name("catalog/mentions")).unwrap();
        let relation = relation(&relation_id, &text, &text);
        assert_eq!(relation.roles().len(), 2);
        assert_eq!(
            relation.shape().parts(),
            &[
                SentencePart::Role(RoleId::new(name("left")).unwrap()),
                SentencePart::Literal("relates to".to_owned()),
                SentencePart::Role(RoleId::new(name("right")).unwrap()),
            ]
        );
    }

    #[test]
    fn model_validation_enforces_types_ground_entities_and_range() {
        let text = type_id("Text");
        let number = type_id("Number");
        let relation_id = RelationId::new(name("catalog/text")).unwrap();
        let malformed = clause(
            &relation_id,
            variable("subject", &text),
            variable("value", &text),
        );
        assert!(
            Model::new(
                ModelId::new(name("catalog")).unwrap(),
                BTreeMap::from([
                    (text.clone(), Type::new(text.clone())),
                    (number.clone(), Type::new(number)),
                ]),
                BTreeSet::new(),
                BTreeMap::from([(relation_id.clone(), relation(&relation_id, &text, &text))]),
                vec![malformed],
                Vec::new(),
            )
            .is_err()
        );

        let unbound = Law::new(
            LawId::new(name("catalog/unbound")).unwrap(),
            vec![clause(
                &relation_id,
                variable("subject", &text),
                variable("value", &text),
            )],
            clause(
                &relation_id,
                variable("subject", &text),
                variable("fresh", &text),
            ),
        )
        .unwrap();
        assert!(model(vec![unbound]).is_err());
    }

    #[test]
    fn find_plan_is_request_independent_and_mode_checked() {
        let text = type_id("Text");
        let relation_id = RelationId::new(name("catalog/text")).unwrap();
        let model = model(Vec::new()).unwrap();
        let sought = VariableId::new(name("answer")).unwrap();
        let pattern = clause(
            &relation_id,
            Term::value(text.clone(), "known".to_owned()).unwrap(),
            Term::variable(sought.clone(), text),
        );
        let plan = FindPlan::new(&model, &pattern, sought).unwrap();
        assert_eq!(plan.relation(), &relation_id);
        assert_eq!(plan.known().len(), 1);
    }

    #[test]
    fn find_plan_preserves_known_entity_bindings_for_execution() {
        let model_id = ModelId::new(name("catalog")).unwrap();
        let text = type_id("Text");
        let relation_id = RelationId::new(name("catalog/text")).unwrap();
        let left_role = RoleId::new(name("left")).unwrap();
        let right_role = RoleId::new(name("right")).unwrap();
        let first = EntityId::new(model_id.clone(), name("first"), text.clone()).unwrap();
        let second = EntityId::new(model_id.clone(), name("second"), text.clone()).unwrap();
        let first_result =
            EntityId::new(model_id.clone(), name("first-result"), text.clone()).unwrap();
        let second_result =
            EntityId::new(model_id.clone(), name("second-result"), text.clone()).unwrap();
        let first_fact = clause(
            &relation_id,
            Term::entity(first.clone()),
            Term::entity(first_result.clone()),
        );
        let second_fact = clause(
            &relation_id,
            Term::entity(second.clone()),
            Term::entity(second_result.clone()),
        );
        let model = Model::new(
            model_id,
            BTreeMap::from([(text.clone(), Type::new(text.clone()))]),
            BTreeSet::from([
                first.clone(),
                second.clone(),
                first_result.clone(),
                second_result.clone(),
            ]),
            BTreeMap::from([(relation_id.clone(), relation(&relation_id, &text, &text))]),
            vec![first_fact, second_fact],
            Vec::new(),
        )
        .unwrap();
        let sought = VariableId::new(name("answer")).unwrap();
        let first_pattern = clause(
            &relation_id,
            Term::entity(first.clone()),
            Term::variable(sought.clone(), text.clone()),
        );
        let second_pattern = clause(
            &relation_id,
            Term::entity(second.clone()),
            Term::variable(sought.clone(), text),
        );
        let first_plan = FindPlan::new(&model, &first_pattern, sought.clone()).unwrap();
        let second_plan = FindPlan::new(&model, &second_pattern, sought).unwrap();

        assert_eq!(first_plan.relation(), second_plan.relation());
        assert_eq!(first_plan.known(), second_plan.known());
        assert_eq!(first_plan.sought(), second_plan.sought());
        assert_eq!(first_plan.mode(), second_plan.mode());
        assert_ne!(first_plan.pattern(), second_plan.pattern());

        let execute = |plan: &FindPlan| {
            model
                .assertions()
                .iter()
                .find(|candidate| {
                    candidate.relation() == plan.pattern().relation()
                        && candidate.roles().get(&left_role)
                            == plan.pattern().roles().get(&left_role)
                })
                .and_then(|candidate| candidate.roles().get(&right_role))
                .cloned()
        };
        assert_eq!(execute(&first_plan), Some(Term::entity(first_result)));
        assert_eq!(execute(&second_plan), Some(Term::entity(second_result)));
    }

    #[test]
    fn delta_is_canonical_ground_and_scoped() {
        let text = type_id("Text");
        let relation_id = RelationId::new(name("catalog/text")).unwrap();
        let clause = clause(
            &relation_id,
            Term::value(text.clone(), "left".to_owned()).unwrap(),
            Term::value(text, "right".to_owned()).unwrap(),
        );
        let identity = RevisionId::from_digest([7; 32]);
        assert!(Delta::new(identity, vec![clause.clone()], vec![clause]).is_err());
    }
}
