//! The small, host-neutral semantic kernel used by the Clause M2 experiment.
//!
//! All persistent values have private fields.  They are admitted through the
//! constructors below, which sort maps/facts and reject incomplete clauses.

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KernelError {}

pub type Result<T> = std::result::Result<T, KernelError>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Role {
    name: String,
    typ: String,
}

impl Role {
    pub fn new(name: impl Into<String>, typ: impl Into<String>) -> Result<Self> {
        let role = Self {
            name: name.into(),
            typ: typ.into(),
        };
        valid_name(&role.name, "role name")?;
        valid_name(&role.typ, "role type")?;
        Ok(role)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn typ(&self) -> &str {
        &self.typ
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "one" => Ok(Self::One),
            "maybe" => Ok(Self::Maybe),
            "some" => Ok(Self::Some),
            "many" => Ok(Self::Many),
            _ => Err(KernelError::new("invalid mode cardinality")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mode {
    known: Vec<String>,
    sought: Vec<String>,
    cardinality: Cardinality,
}

impl Mode {
    pub fn finite(
        known: Vec<String>,
        sought: Vec<String>,
        cardinality: Cardinality,
    ) -> Result<Self> {
        let known = sorted_names(known, "known role")?;
        let sought = sorted_names(sought, "sought role")?;
        if known.is_empty()
            || sought.is_empty()
            || known.iter().any(|name| sought.binary_search(name).is_ok())
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
    pub fn known(&self) -> &[String] {
        &self.known
    }
    pub fn sought(&self) -> &[String] {
        &self.sought
    }
    pub fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sentence {
    left: String,
    literal: String,
    right: String,
}

impl Sentence {
    pub fn new(
        left: impl Into<String>,
        literal: impl Into<String>,
        right: impl Into<String>,
    ) -> Result<Self> {
        let sentence = Self {
            left: left.into(),
            literal: literal.into(),
            right: right.into(),
        };
        valid_name(&sentence.left, "sentence role")?;
        valid_name(&sentence.right, "sentence role")?;
        if sentence.literal.is_empty() {
            return Err(KernelError::new("sentence literal cannot be empty"));
        }
        Ok(sentence)
    }
    pub fn left(&self) -> &str {
        &self.left
    }
    pub fn literal(&self) -> &str {
        &self.literal
    }
    pub fn right(&self) -> &str {
        &self.right
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    name: String,
    roles: BTreeMap<String, Role>,
    sentence: Sentence,
    modes: Vec<Mode>,
}

impl Relation {
    pub fn new(
        name: impl Into<String>,
        roles: Vec<Role>,
        sentence: Sentence,
        mut modes: Vec<Mode>,
    ) -> Result<Self> {
        let name = name.into();
        valid_name(&name, "relation name")?;
        let mut role_map = BTreeMap::new();
        for role in roles {
            if role_map.insert(role.name.clone(), role).is_some() {
                return Err(KernelError::new("duplicate relation role"));
            }
        }
        if role_map.len() < 2 {
            return Err(KernelError::new("relation needs at least two named roles"));
        }
        if !role_map.contains_key(sentence.left()) || !role_map.contains_key(sentence.right()) {
            return Err(KernelError::new("sentence names an unknown relation role"));
        }
        for mode in &modes {
            let role_names: BTreeSet<_> = role_map.keys().cloned().collect();
            let covered: BTreeSet<_> = mode.known.iter().chain(&mode.sought).cloned().collect();
            if role_names != covered {
                return Err(KernelError::new("mode must classify every relation role"));
            }
        }
        modes.sort();
        modes.dedup();
        if modes.is_empty() {
            return Err(KernelError::new("relation needs a declared mode"));
        }
        Ok(Self {
            name,
            roles: role_map,
            sentence,
            modes,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn roles(&self) -> &BTreeMap<String, Role> {
        &self.roles
    }
    pub fn sentence(&self) -> &Sentence {
        &self.sentence
    }
    pub fn modes(&self) -> &[Mode] {
        &self.modes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Term {
    variable: bool,
    text: String,
}

impl Term {
    pub fn literal(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        valid_name(&text, "literal")?;
        Ok(Self {
            variable: false,
            text,
        })
    }
    pub fn variable(name: impl Into<String>) -> Result<Self> {
        let text = name.into();
        valid_name(&text, "variable")?;
        Ok(Self {
            variable: true,
            text,
        })
    }
    pub fn is_variable(&self) -> bool {
        self.variable
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Clause {
    relation: String,
    roles: BTreeMap<String, Term>,
}

impl Clause {
    pub fn new(relation: impl Into<String>, roles: Vec<(String, Term)>) -> Result<Self> {
        let relation = relation.into();
        valid_name(&relation, "clause relation")?;
        let mut role_map = BTreeMap::new();
        for (name, term) in roles {
            valid_name(&name, "clause role")?;
            if role_map.insert(name, term).is_some() {
                return Err(KernelError::new("duplicate clause role"));
            }
        }
        if role_map.is_empty() {
            return Err(KernelError::new("clause has no roles"));
        }
        Ok(Self {
            relation,
            roles: role_map,
        })
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    pub fn roles(&self) -> &BTreeMap<String, Term> {
        &self.roles
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Intent {
    name: String,
    desired: Clause,
}

impl Intent {
    pub fn new(name: impl Into<String>, desired: Clause) -> Result<Self> {
        let name = name.into();
        valid_name(&name, "intent name")?;
        Ok(Self { name, desired })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn desired(&self) -> &Clause {
        &self.desired
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    relations: BTreeMap<String, Relation>,
    facts: Vec<Clause>,
    query: Clause,
    intents: Vec<Intent>,
    order: String,
}

impl Model {
    pub fn new(
        relations: Vec<Relation>,
        facts: Vec<Clause>,
        query: Clause,
        order: impl Into<String>,
    ) -> Result<Self> {
        Self::with_intents(relations, facts, query, Vec::new(), order)
    }
    pub fn with_intents(
        relations: Vec<Relation>,
        mut facts: Vec<Clause>,
        query: Clause,
        mut intents: Vec<Intent>,
        order: impl Into<String>,
    ) -> Result<Self> {
        let mut relation_map = BTreeMap::new();
        for relation in relations {
            if relation_map
                .insert(relation.name.clone(), relation)
                .is_some()
            {
                return Err(KernelError::new("duplicate relation identity"));
            }
        }
        if relation_map.is_empty() {
            return Err(KernelError::new("model has no relations"));
        }
        for fact in &facts {
            validate_clause(&relation_map, fact, false)?;
        }
        validate_clause(&relation_map, &query, true)?;
        let mut intent_names = BTreeSet::new();
        for intent in &intents {
            if !intent_names.insert(intent.name.clone()) {
                return Err(KernelError::new("duplicate intent identity"));
            }
            validate_clause(&relation_map, intent.desired(), false)?;
        }
        facts.sort();
        facts.dedup();
        intents.sort();
        let order = order.into();
        let sought = query
            .roles
            .values()
            .filter(|term| term.is_variable())
            .collect::<Vec<_>>();
        if order != "ascending" || sought.len() != 1 {
            return Err(KernelError::new("M2 requires one ascending sought role"));
        }
        Ok(Self {
            relations: relation_map,
            facts,
            query,
            intents,
            order,
        })
    }
    pub fn relations(&self) -> &BTreeMap<String, Relation> {
        &self.relations
    }
    pub fn facts(&self) -> &[Clause] {
        &self.facts
    }
    pub fn query(&self) -> &Clause {
        &self.query
    }
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }
    pub fn order(&self) -> &str {
        &self.order
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revision {
    identity: String,
    model: Model,
}

impl Revision {
    pub fn admit(model: Model) -> Self {
        let identity = crate::wire::revision_id(&model);
        Self { identity, model }
    }
    pub(crate) fn reloaded(identity: String, model: Model) -> Self {
        Self { identity, model }
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    pub fn plan(&self) -> Result<QueryPlan> {
        plan(&self.model)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Branch {
    name: String,
    revision: Revision,
}

impl Branch {
    pub fn new(name: impl Into<String>, revision: Revision) -> Result<Self> {
        let name = name.into();
        valid_name(&name, "branch name")?;
        Ok(Self { name, revision })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn revision(&self) -> &Revision {
        &self.revision
    }
}

/// The complete result of admitting a closed clause to a branch.  Both
/// variants own their snapshots so an operation result cannot alias mutable
/// state in its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimResult {
    Admitted {
        base: Branch,
        successor: Branch,
        fact: Clause,
    },
    Duplicate {
        branch: Branch,
    },
}

impl ClaimResult {
    pub fn branch(&self) -> &Branch {
        match self {
            Self::Admitted { successor, .. } => successor,
            Self::Duplicate { branch } => branch,
        }
    }
    pub fn base_revision(&self) -> &Revision {
        match self {
            Self::Admitted { base, .. } => base.revision(),
            Self::Duplicate { branch } => branch.revision(),
        }
    }
    pub fn successor(&self) -> Option<&Branch> {
        match self {
            Self::Admitted { successor, .. } => Some(successor),
            Self::Duplicate { .. } => None,
        }
    }
    pub fn fact(&self) -> Option<&Clause> {
        match self {
            Self::Admitted { fact, .. } => Some(fact),
            Self::Duplicate { .. } => None,
        }
    }
}

/// A fact proof is always scoped to the exact revision that contained it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    revision: String,
    relation: String,
    roles: BTreeMap<String, String>,
}

impl Proof {
    fn fact(revision: &Revision, fact: &Clause) -> Self {
        Self {
            revision: revision.identity.clone(),
            relation: fact.relation.clone(),
            roles: fact
                .roles
                .iter()
                .map(|(role, term)| (role.clone(), term.text.clone()))
                .collect(),
        }
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    pub fn roles(&self) -> &BTreeMap<String, String> {
        &self.roles
    }
    pub fn identity(&self) -> String {
        let roles = self
            .roles
            .iter()
            .map(|(role, value)| format!("{role}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("proof/{}/{}/{}", self.revision, self.relation, roles)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequireResult {
    Satisfied { revision: Revision, proof: Proof },
    Unsatisfied { revision: Revision, clause: Clause },
}

/// A pure response to one named desired clause.  A proposal describes a
/// possible explicit claim; it never creates a successor branch or revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntentResult {
    Proposed {
        revision: Revision,
        intent: Intent,
    },
    AlreadySatisfied {
        revision: Revision,
        intent: Intent,
        proof: Proof,
    },
    Rejected {
        revision: Revision,
        name: String,
    },
}

impl IntentResult {
    pub fn revision(&self) -> &Revision {
        match self {
            Self::Proposed { revision, .. }
            | Self::AlreadySatisfied { revision, .. }
            | Self::Rejected { revision, .. } => revision,
        }
    }
    pub fn intent(&self) -> Option<&Intent> {
        match self {
            Self::Proposed { intent, .. } | Self::AlreadySatisfied { intent, .. } => Some(intent),
            Self::Rejected { .. } => None,
        }
    }
    pub fn proof(&self) -> Option<&Proof> {
        match self {
            Self::AlreadySatisfied { proof, .. } => Some(proof),
            Self::Proposed { .. } | Self::Rejected { .. } => None,
        }
    }
    pub fn rejected_name(&self) -> Option<&str> {
        match self {
            Self::Rejected { name, .. } => Some(name),
            Self::Proposed { .. } | Self::AlreadySatisfied { .. } => None,
        }
    }
}

impl RequireResult {
    pub fn revision(&self) -> &Revision {
        match self {
            Self::Satisfied { revision, .. } | Self::Unsatisfied { revision, .. } => revision,
        }
    }
    pub fn proof(&self) -> Option<&Proof> {
        match self {
            Self::Satisfied { proof, .. } => Some(proof),
            Self::Unsatisfied { .. } => None,
        }
    }
    pub fn clause(&self) -> Option<&Clause> {
        match self {
            Self::Satisfied { .. } => None,
            Self::Unsatisfied { clause, .. } => Some(clause),
        }
    }
}

/// Purely admit a closed, complete clause.  The original branch is retained
/// unchanged in either result, and the successor revision is canonicalized by
/// Model admission before its identity is computed.
pub fn claim(branch: &Branch, clause: Clause) -> Result<ClaimResult> {
    let model = branch.revision().model();
    validate_clause(model.relations(), &clause, false)?;
    if model.facts().binary_search(&clause).is_ok() {
        return Ok(ClaimResult::Duplicate {
            branch: branch.clone(),
        });
    }
    let mut facts = model.facts.clone();
    facts.push(clause.clone());
    let successor = Branch::new(
        branch.name.clone(),
        Revision::admit(Model::with_intents(
            model.relations.values().cloned().collect(),
            facts,
            model.query.clone(),
            model.intents.clone(),
            model.order.clone(),
        )?),
    )?;
    Ok(ClaimResult::Admitted {
        base: branch.clone(),
        successor,
        fact: clause,
    })
}

/// Pure closed-clause membership.  This deliberately does not plan a query,
/// choose a mode, or enumerate facts beyond locating the exact canonical fact.
pub fn require(revision: &Revision, clause: Clause) -> Result<RequireResult> {
    validate_clause(revision.model.relations(), &clause, false)?;
    if let Ok(index) = revision.model.facts.binary_search(&clause) {
        let fact = &revision.model.facts[index];
        return Ok(RequireResult::Satisfied {
            revision: revision.clone(),
            proof: Proof::fact(revision, fact),
        });
    }
    Ok(RequireResult::Unsatisfied {
        revision: revision.clone(),
        clause,
    })
}

/// Purely inspect a named intent in the branch's immutable revision.  A
/// missing desired fact becomes a proposal for a later explicit `claim`; a
/// present fact uses the same revision-scoped proof as `require`.
pub fn intent(branch: &Branch, name: &str) -> IntentResult {
    let revision = branch.revision.clone();
    let Some(intent) = revision
        .model
        .intents()
        .iter()
        .find(|intent| intent.name() == name)
        .cloned()
    else {
        return IntentResult::Rejected {
            revision,
            name: name.to_owned(),
        };
    };
    match revision.model.facts.binary_search(intent.desired()) {
        Ok(index) => IntentResult::AlreadySatisfied {
            proof: Proof::fact(&revision, &revision.model.facts[index]),
            revision,
            intent,
        },
        Err(_) => IntentResult::Proposed { revision, intent },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    mode: Mode,
    known: Vec<String>,
    sought: Vec<String>,
}

impl QueryPlan {
    pub fn mode(&self) -> &Mode {
        &self.mode
    }
    pub fn known(&self) -> &[String] {
        &self.known
    }
    pub fn sought(&self) -> &[String] {
        &self.sought
    }
}

pub fn plan(model: &Model) -> Result<QueryPlan> {
    let query = model.query();
    let relation = model
        .relations()
        .get(query.relation())
        .ok_or_else(|| KernelError::new("query relation is undeclared"))?;
    let known = query
        .roles()
        .iter()
        .filter_map(|(name, term)| (!term.is_variable()).then(|| name.clone()))
        .collect::<Vec<_>>();
    let sought = query
        .roles()
        .iter()
        .filter_map(|(name, term)| term.is_variable().then(|| name.clone()))
        .collect::<Vec<_>>();
    let mode = relation
        .modes()
        .iter()
        .find(|mode| mode.known == known && mode.sought == sought)
        .cloned()
        .ok_or_else(|| KernelError::new("no declared mode admits this query orientation"))?;
    Ok(QueryPlan {
        mode,
        known,
        sought,
    })
}

fn validate_clause(
    relations: &BTreeMap<String, Relation>,
    clause: &Clause,
    query: bool,
) -> Result<()> {
    let relation = relations
        .get(clause.relation())
        .ok_or_else(|| KernelError::new("clause relation is undeclared"))?;
    if clause.roles.keys().ne(relation.roles.keys()) {
        return Err(KernelError::new(
            "clause must fill the complete named role map",
        ));
    }
    if !query && clause.roles.values().any(Term::is_variable) {
        return Err(KernelError::new("facts cannot contain variables"));
    }
    Ok(())
}

fn sorted_names(values: Vec<String>, where_: &str) -> Result<Vec<String>> {
    let mut values = values;
    for value in &values {
        valid_name(value, where_)?;
    }
    values.sort();
    values.dedup();
    Ok(values)
}

fn valid_name(value: &str, where_: &str) -> Result<()> {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '"' | '\\' | '[' | ']' | '{' | '}' | ','))
    {
        Err(KernelError::new(format!("invalid {where_}")))
    } else {
        Ok(())
    }
}
