//! Bounded game-leverage falsifier: one historical law fixture, two schedules.
//!
//! This experiment implements only historical, untyped Fact-set semantics. Its
//! fixture uses the noncanonical `RelationShape`, arrow-mode, and
//! conclusion-first syntax once exercised by a historical frontend; it is not
//! current canonical Clause syntax or typed Clause semantics. Domain names are
//! parsed labels and are not type-checked. Both schedules consume the same
//! [`LawIr`]; relation names and phrase literals select no host behavior.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fact {
    relation: String,
    roles: BTreeMap<String, String>,
}

impl Fact {
    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn role(&self, role: &str) -> Option<&str> {
        self.roles.get(role).map(String::as_str)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Change {
    Admit(Fact),
    Withdraw(Fact),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct World {
    facts: BTreeSet<Fact>,
}

impl World {
    pub fn facts(&self) -> &BTreeSet<Fact> {
        &self.facts
    }

    pub fn admit(&mut self, fact: Fact) -> Result<(), Error> {
        if self.facts.insert(fact) {
            Ok(())
        } else {
            Err(Error::new("cannot admit an existing fact"))
        }
    }

    pub fn apply(&mut self, changes: &[Change]) -> Result<(), Error> {
        for change in changes {
            match change {
                Change::Admit(fact) => self.admit(fact.clone())?,
                Change::Withdraw(fact) if self.facts.remove(fact) => {}
                Change::Withdraw(_) => return Err(Error::new("cannot withdraw a missing fact")),
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeIr {
    pub known: Vec<String>,
    pub sought: Vec<String>,
    pub cardinality: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleIr {
    name: String,
    domain: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShapePart {
    Role(RoleIr),
    Literal(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationIr {
    designation: String,
    shape: Vec<ShapePart>,
    roles: Vec<RoleIr>,
    modes: Vec<ModeIr>,
}

impl RelationIr {
    pub fn designation(&self) -> &str {
        &self.designation
    }

    pub fn modes(&self) -> &[ModeIr] {
        &self.modes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PatternTerm {
    Variable(String),
    Constant(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PatternIr {
    relation: String,
    roles: BTreeMap<String, PatternTerm>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LawIr {
    designation: String,
    premises: Vec<PatternIr>,
    conclusion: PatternIr,
}

impl LawIr {
    pub fn designation(&self) -> &str {
        &self.designation
    }

    pub fn dependencies(&self) -> Vec<&str> {
        self.premises
            .iter()
            .map(|premise| premise.relation.as_str())
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramIr {
    relations: BTreeMap<String, RelationIr>,
    law: LawIr,
}

impl ProgramIr {
    pub fn parse(source: &str) -> Result<Self, Error> {
        Parser::new(source).parse()
    }

    pub fn law(&self) -> &LawIr {
        &self.law
    }

    pub fn relation(&self, designation: &str) -> Option<&RelationIr> {
        self.relations.get(designation)
    }

    /// Construct a Fact after checking relation and role names only.
    ///
    /// Historical domain labels are intentionally not type-checked.
    pub fn fact(&self, relation: &str, roles: &[(&str, &str)]) -> Result<Fact, Error> {
        let declaration = self
            .relations
            .get(relation)
            .ok_or_else(|| Error::new(format!("unknown relation '{relation}'")))?;
        let roles = roles
            .iter()
            .map(|(role, value)| ((*role).to_owned(), (*value).to_owned()))
            .collect::<BTreeMap<_, _>>();
        let expected = declaration
            .roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<BTreeSet<_>>();
        let actual = roles.keys().map(String::as_str).collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(Error::new(format!(
                "fact for '{relation}' has roles {actual:?}, expected {expected:?}"
            )));
        }
        Ok(Fact {
            relation: relation.to_owned(),
            roles,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error(String);

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

struct Parser<'a> {
    lines: Vec<&'a str>,
    cursor: usize,
    relations: BTreeMap<String, RelationIr>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
            cursor: 0,
            relations: BTreeMap::new(),
        }
    }

    fn parse(mut self) -> Result<ProgramIr, Error> {
        while self.cursor < self.lines.len() {
            let line = self.lines[self.cursor];
            if line.trim().is_empty() || !line.starts_with(char::is_whitespace) {
                if let Some(designation) = line.strip_suffix(": RelationShape") {
                    self.parse_relation(designation)?;
                    continue;
                }
                if let Some(designation) = line.strip_prefix("law ") {
                    let law = self.parse_law(designation)?;
                    self.skip_blank();
                    let authorization = self
                        .lines
                        .get(self.cursor)
                        .and_then(|line| line.strip_prefix("derive "))
                        .ok_or_else(|| Error::new("law requires a matching derive declaration"))?;
                    if authorization != law.designation {
                        return Err(Error::new(
                            "derive declaration does not name the parsed law",
                        ));
                    }
                    self.cursor += 1;
                    return Ok(ProgramIr {
                        relations: self.relations,
                        law,
                    });
                }
            }
            self.cursor += 1;
        }
        Err(Error::new("source contains no law"))
    }

    fn parse_relation(&mut self, designation: &str) -> Result<(), Error> {
        self.cursor += 1;
        let shape_line = self
            .lines
            .get(self.cursor)
            .filter(|line| line.starts_with("  ") && !line.starts_with("    "))
            .ok_or_else(|| Error::new("relation requires one indented shape"))?
            .trim();
        let shape = parse_shape(shape_line)?;
        let roles = shape
            .iter()
            .filter_map(|part| match part {
                ShapePart::Role(role) => Some(role.clone()),
                ShapePart::Literal(_) => None,
            })
            .collect::<Vec<_>>();
        self.cursor += 1;
        let mut modes = Vec::new();
        while let Some(line) = self.lines.get(self.cursor) {
            let Some(mode) = line.strip_prefix("  mode ") else {
                break;
            };
            modes.push(parse_mode(mode, &roles)?);
            self.cursor += 1;
        }
        if modes.is_empty() {
            return Err(Error::new("relation requires at least one mode"));
        }
        let relation = RelationIr {
            designation: designation.to_owned(),
            shape,
            roles,
            modes,
        };
        if self
            .relations
            .insert(designation.to_owned(), relation)
            .is_some()
        {
            return Err(Error::new(format!("duplicate relation '{designation}'")));
        }
        Ok(())
    }

    fn parse_law(&mut self, designation: &str) -> Result<LawIr, Error> {
        self.cursor += 1;
        let conclusion_line = self
            .lines
            .get(self.cursor)
            .and_then(|line| line.strip_prefix("  "))
            .and_then(|line| line.strip_suffix(" if"))
            .ok_or_else(|| Error::new("law requires an indented '<conclusion> if'"))?;
        let conclusion = parse_pattern(conclusion_line, &self.relations)?;
        self.cursor += 1;
        let mut premises = Vec::new();
        while let Some(line) = self.lines.get(self.cursor) {
            let Some(premise) = line.strip_prefix("    ") else {
                break;
            };
            if !premise.trim().is_empty() {
                premises.push(parse_pattern(premise, &self.relations)?);
            }
            self.cursor += 1;
        }
        if premises.is_empty() {
            return Err(Error::new("law requires at least one premise"));
        }
        let bound = premises
            .iter()
            .flat_map(|premise| premise.roles.values())
            .filter_map(variable)
            .collect::<BTreeSet<_>>();
        if conclusion
            .roles
            .values()
            .filter_map(variable)
            .any(|name| !bound.contains(name))
        {
            return Err(Error::new("law conclusion is not range-restricted"));
        }
        Ok(LawIr {
            designation: designation.to_owned(),
            premises,
            conclusion,
        })
    }

    fn skip_blank(&mut self) {
        while self
            .lines
            .get(self.cursor)
            .is_some_and(|line| line.trim().is_empty())
        {
            self.cursor += 1;
        }
    }
}

fn variable(term: &PatternTerm) -> Option<&str> {
    match term {
        PatternTerm::Variable(name) => Some(name),
        PatternTerm::Constant(_) => None,
    }
}

fn parse_shape(source: &str) -> Result<Vec<ShapePart>, Error> {
    let mut rest = source;
    let mut parts = Vec::new();
    while !rest.is_empty() {
        if let Some(open) = rest.find('{') {
            let literal = rest[..open].trim();
            if !literal.is_empty() {
                parts.push(ShapePart::Literal(literal.to_owned()));
            }
            let after_open = &rest[open + 1..];
            let close = after_open
                .find('}')
                .ok_or_else(|| Error::new("unterminated relation role"))?;
            let (name, domain) = after_open[..close]
                .split_once(": ")
                .ok_or_else(|| Error::new("relation role must be '{name: Domain}'"))?;
            parts.push(ShapePart::Role(RoleIr {
                name: name.to_owned(),
                domain: domain.to_owned(),
            }));
            rest = after_open[close + 1..].trim_start();
        } else {
            let literal = rest.trim();
            if !literal.is_empty() {
                parts.push(ShapePart::Literal(literal.to_owned()));
            }
            rest = "";
        }
    }
    if parts
        .iter()
        .filter(|part| matches!(part, ShapePart::Role(_)))
        .count()
        < 2
    {
        return Err(Error::new("relation shape requires at least two roles"));
    }
    Ok(parts)
}

fn parse_mode(source: &str, roles: &[RoleIr]) -> Result<ModeIr, Error> {
    let (orientation, cardinality) = source
        .rsplit_once(": ")
        .ok_or_else(|| Error::new("mode requires a cardinality"))?;
    let (known, sought) = orientation
        .split_once(" -> ")
        .ok_or_else(|| Error::new("mode requires 'known -> sought'"))?;
    let split_roles = |value: &str| {
        value
            .split(", ")
            .flat_map(|part| part.split_whitespace())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let known = split_roles(known);
    let sought = split_roles(sought);
    let declared = roles
        .iter()
        .map(|role| role.name.as_str())
        .collect::<BTreeSet<_>>();
    if known
        .iter()
        .chain(&sought)
        .any(|role| !declared.contains(role.as_str()))
    {
        return Err(Error::new("mode names an undeclared role"));
    }
    if !matches!(cardinality, "one" | "maybe" | "some" | "many") {
        return Err(Error::new("unsupported mode cardinality"));
    }
    Ok(ModeIr {
        known,
        sought,
        cardinality: cardinality.to_owned(),
    })
}

fn parse_pattern(
    source: &str,
    relations: &BTreeMap<String, RelationIr>,
) -> Result<PatternIr, Error> {
    let tokens = source.split_whitespace().collect::<Vec<_>>();
    let mut matches = Vec::new();
    for relation in relations.values() {
        let mut token = 0;
        let mut roles = BTreeMap::new();
        let mut matched = true;
        for part in &relation.shape {
            match part {
                ShapePart::Role(role) => {
                    let Some(value) = tokens.get(token) else {
                        matched = false;
                        break;
                    };
                    roles.insert(
                        role.name.clone(),
                        value.strip_prefix('?').map_or_else(
                            || PatternTerm::Constant((*value).to_owned()),
                            |name| PatternTerm::Variable(name.to_owned()),
                        ),
                    );
                    token += 1;
                }
                ShapePart::Literal(literal) => {
                    for expected in literal.split_whitespace() {
                        if tokens.get(token) != Some(&expected) {
                            matched = false;
                            break;
                        }
                        token += 1;
                    }
                }
            }
            if !matched {
                break;
            }
        }
        if matched && token == tokens.len() {
            matches.push(PatternIr {
                relation: relation.designation.clone(),
                roles,
            });
        }
    }
    match matches.as_slice() {
        [pattern] => Ok(pattern.clone()),
        [] => Err(Error::new(format!("no relation reads '{source}'"))),
        _ => Err(Error::new(format!("ambiguous relation phrase '{source}'"))),
    }
}

pub type FactSetView = BTreeSet<Fact>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScanWork {
    pub fact_checks: usize,
}

/// Exhaustive tuple interpretation under the historical Fact-set semantics.
///
/// This reference path deliberately interprets the law from grounded premise
/// occurrences. It neither compiles an [`IndexedPlan`] nor constructs the
/// binding maps used by the indexed matcher and projector.
pub fn reference_materialize(law: &LawIr, world: &World) -> (FactSetView, ScanWork) {
    let mut work = ScanWork::default();
    let mut tuples = vec![Vec::new()];
    for premise in &law.premises {
        let mut next = Vec::new();
        for tuple in tuples {
            for fact in world.facts() {
                work.fact_checks += 1;
                if fact.relation() == premise.relation {
                    let mut extended = tuple.clone();
                    extended.push(fact);
                    next.push(extended);
                }
            }
        }
        tuples = next;
    }

    let view = tuples
        .iter()
        .filter(|tuple| reference_tuple_satisfies(law, tuple))
        .filter_map(|tuple| reference_conclusion(law, tuple))
        .collect();
    (view, work)
}

fn reference_tuple_satisfies(law: &LawIr, tuple: &[&Fact]) -> bool {
    law.premises.len() == tuple.len()
        && law.premises.iter().zip(tuple).all(|(premise, fact)| {
            premise.relation == fact.relation()
                && premise.roles.iter().all(|(role, term)| {
                    let Some(actual) = fact.role(role) else {
                        return false;
                    };
                    match term {
                        PatternTerm::Constant(expected) => actual == expected,
                        PatternTerm::Variable(variable) => {
                            law.premises
                                .iter()
                                .zip(tuple)
                                .all(|(other_premise, other_fact)| {
                                    other_premise.roles.iter().all(|(other_role, other_term)| {
                                        match other_term {
                                            PatternTerm::Variable(other_variable)
                                                if other_variable == variable =>
                                            {
                                                other_fact.role(other_role) == Some(actual)
                                            }
                                            PatternTerm::Variable(_) | PatternTerm::Constant(_) => {
                                                true
                                            }
                                        }
                                    })
                                })
                        }
                    }
                })
        })
}

fn reference_conclusion(law: &LawIr, tuple: &[&Fact]) -> Option<Fact> {
    let roles = law
        .conclusion
        .roles
        .iter()
        .map(|(role, term)| {
            let value = match term {
                PatternTerm::Constant(value) => value.clone(),
                PatternTerm::Variable(_) => law
                    .premises
                    .iter()
                    .zip(tuple)
                    .find_map(|(premise, fact)| {
                        premise
                            .roles
                            .iter()
                            .find_map(|(premise_role, premise_term)| {
                                (premise_term == term)
                                    .then(|| fact.role(premise_role))
                                    .flatten()
                            })
                    })?
                    .to_owned(),
            };
            Some((role.clone(), value))
        })
        .collect::<Option<BTreeMap<_, _>>>()?;
    Some(Fact {
        relation: law.conclusion.relation.clone(),
        roles,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanTrace {
    pub law: String,
    pub dependencies: Vec<String>,
    pub lookup_modes: Vec<ModeIr>,
    pub join_variable: String,
    pub index_roles: Vec<(String, String)>,
    pub view_relation: String,
    pub view_projection: Vec<(String, String)>,
    pub update_probes: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IndexedAccess {
    pattern: PatternIr,
    key_role: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedPlan {
    left: IndexedAccess,
    right: IndexedAccess,
    conclusion: PatternIr,
    trace: PlanTrace,
}

impl IndexedPlan {
    /// Select join roles from binders and validate them against declared lookup modes.
    pub fn compile(program: &ProgramIr, law: &LawIr) -> Result<Self, Error> {
        let [left, right] = law.premises.as_slice() else {
            return Err(Error::new("indexed proof requires exactly two premises"));
        };
        if left.relation == right.relation {
            return Err(Error::new(
                "indexed proof requires two distinct dependency relations",
            ));
        }
        let left_variables = pattern_variables(left);
        let right_variables = pattern_variables(right);
        let shared = left_variables
            .intersection(&right_variables)
            .cloned()
            .collect::<Vec<_>>();
        let [join_variable] = shared.as_slice() else {
            return Err(Error::new(
                "indexed proof requires exactly one shared premise variable",
            ));
        };
        let left_key = role_for_variable(left, join_variable)?;
        let right_key = role_for_variable(right, join_variable)?;
        let left_mode = lookup_mode(program, left, &left_key)?;
        let right_mode = lookup_mode(program, right, &right_key)?;
        let dependencies = vec![left.relation.clone(), right.relation.clone()];
        let view_projection = law
            .conclusion
            .roles
            .iter()
            .map(|(role, term)| {
                let source = match term {
                    PatternTerm::Variable(variable) => format!("?{variable}"),
                    PatternTerm::Constant(value) => value.clone(),
                };
                (role.clone(), source)
            })
            .collect();
        let trace = PlanTrace {
            law: law.designation.clone(),
            dependencies: dependencies.clone(),
            lookup_modes: vec![left_mode, right_mode],
            join_variable: join_variable.clone(),
            index_roles: vec![
                (left.relation.clone(), left_key.clone()),
                (right.relation.clone(), right_key.clone()),
            ],
            view_relation: law.conclusion.relation.clone(),
            view_projection,
            update_probes: vec![
                (left.relation.clone(), right.relation.clone()),
                (right.relation.clone(), left.relation.clone()),
            ],
        };
        Ok(Self {
            left: IndexedAccess {
                pattern: left.clone(),
                key_role: left_key,
            },
            right: IndexedAccess {
                pattern: right.clone(),
                key_role: right_key,
            },
            conclusion: law.conclusion.clone(),
            trace,
        })
    }

    pub fn trace(&self) -> &PlanTrace {
        &self.trace
    }
}

fn pattern_variables(pattern: &PatternIr) -> BTreeSet<String> {
    pattern
        .roles
        .values()
        .filter_map(variable)
        .map(str::to_owned)
        .collect()
}

fn role_for_variable(pattern: &PatternIr, variable: &str) -> Result<String, Error> {
    let roles = pattern
        .roles
        .iter()
        .filter_map(|(role, term)| (self::variable(term) == Some(variable)).then_some(role.clone()))
        .collect::<Vec<_>>();
    match roles.as_slice() {
        [role] => Ok(role.clone()),
        _ => Err(Error::new("join variable must occur once in each premise")),
    }
}

fn lookup_mode(program: &ProgramIr, pattern: &PatternIr, key_role: &str) -> Result<ModeIr, Error> {
    let relation = program
        .relation(&pattern.relation)
        .ok_or_else(|| Error::new("indexed dependency relation is undeclared"))?;
    relation
        .modes
        .iter()
        .find(|mode| mode.known == [key_role] && mode.cardinality == "many")
        .cloned()
        .ok_or_else(|| Error::new("join dependency lacks a many lookup mode for its key"))
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IndexedWork {
    pub build_fact_checks: usize,
    pub counterpart_bucket_probes: usize,
    pub pair_visits: usize,
    pub ignored_changes: usize,
}

pub struct IndexedMaterialization {
    plan: IndexedPlan,
    left: BTreeMap<String, BTreeSet<Fact>>,
    right: BTreeMap<String, BTreeSet<Fact>>,
    supports: BTreeMap<Fact, usize>,
    view: FactSetView,
    work: IndexedWork,
}

impl IndexedMaterialization {
    pub fn build(plan: IndexedPlan, world: &World) -> Result<Self, Error> {
        let mut materialization = Self {
            plan,
            left: BTreeMap::new(),
            right: BTreeMap::new(),
            supports: BTreeMap::new(),
            view: BTreeSet::new(),
            work: IndexedWork::default(),
        };
        for fact in &world.facts {
            materialization.work.build_fact_checks += 1;
            if fact.relation == materialization.plan.left.pattern.relation {
                insert_index(&mut materialization.left, &materialization.plan.left, fact)?;
            }
            if fact.relation == materialization.plan.right.pattern.relation {
                insert_index(
                    &mut materialization.right,
                    &materialization.plan.right,
                    fact,
                )?;
            }
        }
        let mut initial_supports = Vec::new();
        for (key, left_facts) in &materialization.left {
            let Some(right_facts) = materialization.right.get(key) else {
                continue;
            };
            for left in left_facts {
                for right in right_facts {
                    if let Some(fact) = join_fact(&materialization.plan, left, right) {
                        initial_supports.push(fact);
                    }
                }
            }
        }
        for fact in initial_supports {
            materialization.add_support(fact);
        }
        Ok(materialization)
    }

    pub fn view(&self) -> &FactSetView {
        &self.view
    }

    pub fn work(&self) -> IndexedWork {
        self.work
    }

    pub fn reset_update_work(&mut self) {
        self.work.counterpart_bucket_probes = 0;
        self.work.pair_visits = 0;
        self.work.ignored_changes = 0;
    }

    pub fn apply(&mut self, changes: &[Change]) -> Result<(), Error> {
        for change in changes {
            let fact = match change {
                Change::Admit(fact) | Change::Withdraw(fact) => fact,
            };
            let side = if fact.relation == self.plan.left.pattern.relation {
                Some(true)
            } else if fact.relation == self.plan.right.pattern.relation {
                Some(false)
            } else {
                None
            };
            let Some(is_left) = side else {
                self.work.ignored_changes += 1;
                continue;
            };
            let access = if is_left {
                &self.plan.left
            } else {
                &self.plan.right
            };
            let key = fact
                .role(&access.key_role)
                .ok_or_else(|| Error::new("updated fact lacks its indexed key role"))?
                .to_owned();
            match change {
                Change::Admit(_) => {
                    if is_left {
                        insert_index(&mut self.left, access, fact)?;
                    } else {
                        insert_index(&mut self.right, access, fact)?;
                    }
                    self.visit_pairs(is_left, &key, fact, true);
                }
                Change::Withdraw(_) => {
                    let present = if is_left {
                        self.left
                            .get(&key)
                            .is_some_and(|bucket| bucket.contains(fact))
                    } else {
                        self.right
                            .get(&key)
                            .is_some_and(|bucket| bucket.contains(fact))
                    };
                    if !present {
                        return Err(Error::new("indexed plan cannot withdraw a missing fact"));
                    }
                    self.visit_pairs(is_left, &key, fact, false);
                    let removed = if is_left {
                        remove_index(&mut self.left, &key, fact)
                    } else {
                        remove_index(&mut self.right, &key, fact)
                    };
                    debug_assert!(removed, "presence was checked before withdrawal");
                }
            }
        }
        Ok(())
    }

    fn visit_pairs(&mut self, is_left: bool, key: &str, changed: &Fact, admit: bool) {
        self.work.counterpart_bucket_probes += 1;
        let others = if is_left {
            self.right.get(key)
        } else {
            self.left.get(key)
        }
        .map(|facts| facts.iter().cloned().collect::<Vec<_>>());
        let Some(others) = others else {
            return;
        };
        for other in &others {
            self.work.pair_visits += 1;
            let (left, right) = if is_left {
                (changed, other)
            } else {
                (other, changed)
            };
            if let Some(fact) = join_fact(&self.plan, left, right) {
                if admit {
                    self.add_support(fact);
                } else {
                    self.remove_support(&fact);
                }
            }
        }
    }

    fn add_support(&mut self, fact: Fact) {
        *self.supports.entry(fact.clone()).or_default() += 1;
        self.view.insert(fact);
    }

    fn remove_support(&mut self, fact: &Fact) {
        let Some(count) = self.supports.get_mut(fact) else {
            return;
        };
        *count -= 1;
        if *count == 0 {
            self.supports.remove(fact);
            self.view.remove(fact);
        }
    }
}

fn insert_index(
    index: &mut BTreeMap<String, BTreeSet<Fact>>,
    access: &IndexedAccess,
    fact: &Fact,
) -> Result<(), Error> {
    let key = fact
        .role(&access.key_role)
        .ok_or_else(|| Error::new("indexed fact lacks its key role"))?;
    if !index
        .entry(key.to_owned())
        .or_default()
        .insert(fact.clone())
    {
        return Err(Error::new("indexed plan cannot admit an existing fact"));
    }
    Ok(())
}

fn remove_index(index: &mut BTreeMap<String, BTreeSet<Fact>>, key: &str, fact: &Fact) -> bool {
    let Some(bucket) = index.get_mut(key) else {
        return false;
    };
    let removed = bucket.remove(fact);
    if bucket.is_empty() {
        index.remove(key);
    }
    removed
}

fn join_fact(plan: &IndexedPlan, left: &Fact, right: &Fact) -> Option<Fact> {
    let bindings = indexed_match(&plan.left.pattern, left, &BTreeMap::new())?;
    let bindings = indexed_match(&plan.right.pattern, right, &bindings)?;
    indexed_project(&plan.conclusion, &bindings)
}

fn indexed_match(
    pattern: &PatternIr,
    fact: &Fact,
    existing: &BTreeMap<String, String>,
) -> Option<BTreeMap<String, String>> {
    if pattern.relation != fact.relation() {
        return None;
    }
    let mut bindings = existing.clone();
    for (role, term) in &pattern.roles {
        let actual = fact.role(role)?;
        match term {
            PatternTerm::Constant(expected) if actual != expected => return None,
            PatternTerm::Constant(_) => {}
            PatternTerm::Variable(variable) => match bindings.get(variable) {
                Some(expected) if expected != actual => return None,
                Some(_) => {}
                None => {
                    bindings.insert(variable.clone(), actual.to_owned());
                }
            },
        }
    }
    Some(bindings)
}

fn indexed_project(pattern: &PatternIr, bindings: &BTreeMap<String, String>) -> Option<Fact> {
    let roles = pattern
        .roles
        .iter()
        .map(|(role, term)| {
            let value = match term {
                PatternTerm::Variable(variable) => bindings.get(variable)?.clone(),
                PatternTerm::Constant(value) => value.clone(),
            };
            Some((role.clone(), value))
        })
        .collect::<Option<BTreeMap<_, _>>>()?;
    Some(Fact {
        relation: pattern.relation.clone(),
        roles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../examples/spatial_visibility.clause");

    fn world(program: &ProgramIr, noise: usize) -> World {
        let mut world = World::default();
        for index in 0..64 {
            let cell = format!("cell-{}", index % 16);
            world
                .admit(
                    program
                        .fact(
                            "spatial/viewer-cell",
                            &[("observer", &format!("viewer-{index}")), ("cell", &cell)],
                        )
                        .expect("generated viewer fact has the declared roles"),
                )
                .expect("generated viewer fact is unique");
            world
                .admit(
                    program
                        .fact(
                            "spatial/target-cell",
                            &[("target", &format!("target-{index}")), ("cell", &cell)],
                        )
                        .expect("generated target fact has the declared roles"),
                )
                .expect("generated target fact is unique");
        }
        for index in 0..noise {
            world
                .admit(
                    program
                        .fact(
                            "diagnostic/noise",
                            &[
                                ("subject", &format!("noise-{index}")),
                                ("value", "unrelated"),
                            ],
                        )
                        .expect("generated noise fact has the declared roles"),
                )
                .expect("generated noise fact is unique");
        }
        world
    }

    fn move_viewer(program: &ProgramIr, index: usize, from: usize, to: usize) -> [Change; 2] {
        [
            Change::Withdraw(
                program
                    .fact(
                        "spatial/viewer-cell",
                        &[
                            ("observer", &format!("viewer-{index}")),
                            ("cell", &format!("cell-{from}")),
                        ],
                    )
                    .expect("old position has the declared roles"),
            ),
            Change::Admit(
                program
                    .fact(
                        "spatial/viewer-cell",
                        &[
                            ("observer", &format!("viewer-{index}")),
                            ("cell", &format!("cell-{to}")),
                        ],
                    )
                    .expect("new position has the declared roles"),
            ),
        ]
    }

    fn expected_visibility(program: &ProgramIr, observer: &str, targets: &[&str]) -> FactSetView {
        targets
            .iter()
            .map(|target| {
                program
                    .fact(
                        "spatial/visible",
                        &[("observer", observer), ("target", target)],
                    )
                    .expect("expected output fact has the declared roles")
            })
            .collect()
    }

    fn visibility_for_observer(view: &FactSetView, observer: &str) -> FactSetView {
        view.iter()
            .filter(|fact| {
                fact.relation() == "spatial/visible" && fact.role("observer") == Some(observer)
            })
            .cloned()
            .collect()
    }

    #[test]
    fn one_historical_law_yields_one_ir_and_a_complete_physical_trace() {
        let program = ProgramIr::parse(SOURCE).expect("historical fixture parses");
        assert_eq!(program.law().designation(), "spatial/co-cell-visibility");
        assert_eq!(
            program.law().dependencies(),
            ["spatial/viewer-cell", "spatial/target-cell"]
        );

        let plan = IndexedPlan::compile(&program, program.law()).expect("historical law plans");
        let trace = plan.trace();
        assert_eq!(trace.law, "spatial/co-cell-visibility");
        assert_eq!(
            trace.dependencies,
            ["spatial/viewer-cell", "spatial/target-cell"]
        );
        assert_eq!(trace.join_variable, "cell");
        assert_eq!(
            trace.index_roles,
            [
                ("spatial/viewer-cell".to_owned(), "cell".to_owned()),
                ("spatial/target-cell".to_owned(), "cell".to_owned()),
            ]
        );
        assert_eq!(trace.view_relation, "spatial/visible");
        assert_eq!(
            trace.view_projection,
            [
                ("observer".to_owned(), "?observer".to_owned()),
                ("target".to_owned(), "?target".to_owned()),
            ]
        );
        assert_eq!(
            trace.update_probes,
            [
                (
                    "spatial/viewer-cell".to_owned(),
                    "spatial/target-cell".to_owned(),
                ),
                (
                    "spatial/target-cell".to_owned(),
                    "spatial/viewer-cell".to_owned(),
                ),
            ]
        );
        assert_eq!(
            trace.lookup_modes,
            [
                ModeIr {
                    known: vec!["cell".to_owned()],
                    sought: vec!["observer".to_owned()],
                    cardinality: "many".to_owned(),
                },
                ModeIr {
                    known: vec!["cell".to_owned()],
                    sought: vec!["target".to_owned()],
                    cardinality: "many".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parser_and_plan_are_independent_of_all_fixture_names_and_literals() {
        let renamed = SOURCE
            .replace("spatial/viewer-cell", "renamed/alpha")
            .replace("spatial/target-cell", "renamed/beta")
            .replace("spatial/visible", "renamed/gamma")
            .replace("diagnostic/noise", "renamed/delta")
            .replace("spatial/co-cell-visibility", "renamed/equijoin")
            .replace("Viewer", "SentinelDomain")
            .replace("Target", "ArtifactDomain")
            .replace("Cell", "ZoneDomain")
            .replace("Noise", "DebrisDomain")
            .replace("?observer", "?watcher_binding")
            .replace("?target", "?object_binding")
            .replace("?cell", "?junction_binding")
            .replace("observer", "watcher_role")
            .replace("target", "object_role")
            .replace("cell", "zone_role")
            .replace("subject", "debris_role")
            .replace("value", "payload_role")
            .replace("views from", "occupies within")
            .replace("appears in", "rests beside")
            .replace("can see", "correlates against")
            .replace("is unrelated to", "drifts beyond");
        let program = ProgramIr::parse(&renamed).expect("renamed historical fixture parses");
        let plan =
            IndexedPlan::compile(&program, program.law()).expect("renamed historical law plans");
        assert_eq!(plan.trace().law, "renamed/equijoin");
        assert_eq!(plan.trace().view_relation, "renamed/gamma");
        assert_eq!(plan.trace().dependencies, ["renamed/alpha", "renamed/beta"]);
        assert_eq!(plan.trace().join_variable, "junction_binding");
        assert_eq!(
            plan.trace().index_roles,
            [
                ("renamed/alpha".to_owned(), "zone_role".to_owned()),
                ("renamed/beta".to_owned(), "zone_role".to_owned()),
            ]
        );
        assert_eq!(
            plan.trace().view_projection,
            [
                ("object_role".to_owned(), "?object_binding".to_owned()),
                ("watcher_role".to_owned(), "?watcher_binding".to_owned()),
            ]
        );
        assert_eq!(
            plan.trace().lookup_modes,
            [
                ModeIr {
                    known: vec!["zone_role".to_owned()],
                    sought: vec!["watcher_role".to_owned()],
                    cardinality: "many".to_owned(),
                },
                ModeIr {
                    known: vec!["zone_role".to_owned()],
                    sought: vec!["object_role".to_owned()],
                    cardinality: "many".to_owned(),
                },
            ]
        );
        assert_eq!(
            program
                .relation("renamed/alpha")
                .expect("left relation")
                .shape,
            [
                ShapePart::Role(RoleIr {
                    name: "watcher_role".to_owned(),
                    domain: "SentinelDomain".to_owned(),
                }),
                ShapePart::Literal("occupies within".to_owned()),
                ShapePart::Role(RoleIr {
                    name: "zone_role".to_owned(),
                    domain: "ZoneDomain".to_owned(),
                }),
            ]
        );
        assert_eq!(
            program
                .relation("renamed/delta")
                .expect("noise relation")
                .shape,
            [
                ShapePart::Role(RoleIr {
                    name: "debris_role".to_owned(),
                    domain: "DebrisDomain".to_owned(),
                }),
                ShapePart::Literal("drifts beyond".to_owned()),
                ShapePart::Role(RoleIr {
                    name: "payload_role".to_owned(),
                    domain: "DebrisDomain".to_owned(),
                }),
            ]
        );

        let mut world = World::default();
        world
            .admit(
                program
                    .fact(
                        "renamed/alpha",
                        &[("watcher_role", "sentinel-7"), ("zone_role", "zone-3")],
                    )
                    .expect("renamed left fact has the declared roles"),
            )
            .expect("renamed left fact is unique");
        world
            .admit(
                program
                    .fact(
                        "renamed/beta",
                        &[("object_role", "artifact-9"), ("zone_role", "zone-3")],
                    )
                    .expect("renamed right fact has the declared roles"),
            )
            .expect("renamed right fact is unique");
        let expected = BTreeSet::from([program
            .fact(
                "renamed/gamma",
                &[
                    ("watcher_role", "sentinel-7"),
                    ("object_role", "artifact-9"),
                ],
            )
            .expect("renamed output fact has the declared roles")]);
        let (reference, _) = reference_materialize(program.law(), &world);
        let indexed = IndexedMaterialization::build(plan, &world).expect("renamed plan executes");
        assert_eq!(reference, expected);
        assert_eq!(indexed.view(), &expected);
    }

    #[test]
    fn asymmetric_indexed_projection_mutation_breaks_reference_equality() {
        let program = ProgramIr::parse(SOURCE).expect("historical fixture parses");
        let mut plan = IndexedPlan::compile(&program, program.law()).expect("historical law plans");
        let observer_source = plan
            .conclusion
            .roles
            .get("observer")
            .cloned()
            .expect("output has an observer projection");
        let target_source = plan
            .conclusion
            .roles
            .get("target")
            .cloned()
            .expect("output has a target projection");
        plan.conclusion
            .roles
            .insert("observer".to_owned(), target_source);
        plan.conclusion
            .roles
            .insert("target".to_owned(), observer_source);

        let mut world = World::default();
        world
            .admit(
                program
                    .fact(
                        "spatial/viewer-cell",
                        &[("observer", "viewer-left"), ("cell", "cell-shared")],
                    )
                    .expect("viewer fact has the declared roles"),
            )
            .expect("viewer fact is unique");
        world
            .admit(
                program
                    .fact(
                        "spatial/target-cell",
                        &[("target", "target-right"), ("cell", "cell-shared")],
                    )
                    .expect("target fact has the declared roles"),
            )
            .expect("target fact is unique");

        let expected_reference = expected_visibility(&program, "viewer-left", &["target-right"]);
        let expected_mutant = expected_visibility(&program, "target-right", &["viewer-left"]);
        let (reference, _) = reference_materialize(program.law(), &world);
        let indexed = IndexedMaterialization::build(plan, &world).expect("mutant plan executes");
        assert_eq!(reference, expected_reference);
        assert_eq!(indexed.view(), &expected_mutant);
        assert_ne!(&reference, indexed.view());
    }

    #[test]
    fn asymmetric_indexed_matching_mutation_breaks_reference_equality() {
        let program = ProgramIr::parse(SOURCE).expect("historical fixture parses");
        let mut plan = IndexedPlan::compile(&program, program.law()).expect("historical law plans");
        plan.right.pattern.roles.insert(
            "target".to_owned(),
            PatternTerm::Variable("observer".to_owned()),
        );

        let mut world = World::default();
        world
            .admit(
                program
                    .fact(
                        "spatial/viewer-cell",
                        &[("observer", "viewer-left"), ("cell", "cell-shared")],
                    )
                    .expect("viewer fact has the declared roles"),
            )
            .expect("viewer fact is unique");
        world
            .admit(
                program
                    .fact(
                        "spatial/target-cell",
                        &[("target", "target-right"), ("cell", "cell-shared")],
                    )
                    .expect("target fact has the declared roles"),
            )
            .expect("target fact is unique");

        let expected_reference = expected_visibility(&program, "viewer-left", &["target-right"]);
        let (reference, _) = reference_materialize(program.law(), &world);
        let indexed = IndexedMaterialization::build(plan, &world).expect("mutant plan executes");
        assert_eq!(reference, expected_reference);
        assert_eq!(indexed.view(), &FactSetView::new());
        assert_ne!(&reference, indexed.view());
    }

    #[test]
    fn fact_set_views_match_and_indexed_updates_stay_local() {
        let program = ProgramIr::parse(SOURCE).expect("historical fixture parses");
        let plan = IndexedPlan::compile(&program, program.law()).expect("historical law plans");
        let mut clean_world = world(&program, 0);
        let mut noisy_world = world(&program, 16_384);
        let (clean_reference, clean_scan) = reference_materialize(program.law(), &clean_world);
        let (noisy_reference, noisy_scan) = reference_materialize(program.law(), &noisy_world);
        assert_eq!(clean_reference, noisy_reference);
        assert_eq!(clean_reference.len(), 256);
        let viewer_zero_before = expected_visibility(
            &program,
            "viewer-0",
            &["target-0", "target-16", "target-32", "target-48"],
        );
        assert_eq!(
            visibility_for_observer(&clean_reference, "viewer-0"),
            viewer_zero_before
        );

        let mut clean_index =
            IndexedMaterialization::build(plan.clone(), &clean_world).expect("clean index builds");
        let mut noisy_index =
            IndexedMaterialization::build(plan, &noisy_world).expect("noisy index builds");
        assert_eq!(&clean_reference, clean_index.view());
        assert_eq!(&noisy_reference, noisy_index.view());

        let changes = move_viewer(&program, 0, 0, 1);
        clean_world.apply(&changes).expect("clean update applies");
        noisy_world.apply(&changes).expect("noisy update applies");
        clean_index.reset_update_work();
        noisy_index.reset_update_work();
        clean_index.apply(&changes).expect("clean index updates");
        noisy_index.apply(&changes).expect("noisy index updates");

        let (clean_after, clean_update_scan) = reference_materialize(program.law(), &clean_world);
        let (noisy_after, noisy_update_scan) = reference_materialize(program.law(), &noisy_world);
        assert_eq!(clean_after, noisy_after);
        assert_eq!(&clean_after, clean_index.view());
        assert_eq!(&noisy_after, noisy_index.view());
        let viewer_zero_after = expected_visibility(
            &program,
            "viewer-0",
            &["target-1", "target-17", "target-33", "target-49"],
        );
        assert_eq!(
            visibility_for_observer(&clean_after, "viewer-0"),
            viewer_zero_after
        );

        let clean_update = clean_index.work();
        let noisy_update = noisy_index.work();
        assert_eq!(clean_update.counterpart_bucket_probes, 2);
        assert_eq!(clean_update.pair_visits, 8);
        assert_eq!(
            (
                clean_update.counterpart_bucket_probes,
                clean_update.pair_visits,
            ),
            (
                noisy_update.counterpart_bucket_probes,
                noisy_update.pair_visits,
            ),
            "unrelated facts must not enter indexed update work"
        );
        assert!(
            noisy_scan.fact_checks >= clean_scan.fact_checks * 100,
            "reference initial scan must expose unrelated-population growth"
        );
        assert!(
            noisy_update_scan.fact_checks >= clean_update_scan.fact_checks * 100,
            "reference update recomputation must expose unrelated-population growth"
        );

        for index in 1..16 {
            let from = index % 16;
            let to = (from + 1) % 16;
            let changes = move_viewer(&program, index, from, to);
            noisy_world.apply(&changes).expect("world sequence applies");
            noisy_index.apply(&changes).expect("index sequence applies");
            let (reference, _) = reference_materialize(program.law(), &noisy_world);
            assert_eq!(&reference, noisy_index.view(), "revision {index} differs");
        }
    }

    #[test]
    fn indexed_retraction_preserves_an_independent_equal_support() {
        let program = ProgramIr::parse(SOURCE).expect("historical fixture parses");
        let plan = IndexedPlan::compile(&program, program.law()).expect("historical law plans");
        let mut world = World::default();
        for cell in ["cell-a", "cell-b"] {
            world
                .admit(
                    program
                        .fact(
                            "spatial/viewer-cell",
                            &[("observer", "viewer"), ("cell", cell)],
                        )
                        .expect("viewer support has the declared roles"),
                )
                .expect("viewer support is unique");
            world
                .admit(
                    program
                        .fact(
                            "spatial/target-cell",
                            &[("target", "target"), ("cell", cell)],
                        )
                        .expect("target support has the declared roles"),
                )
                .expect("target support is unique");
        }
        let mut indexed = IndexedMaterialization::build(plan, &world).expect("index builds");
        let expected = expected_visibility(&program, "viewer", &["target"]);
        let visible = expected.first().expect("one expected output fact").clone();
        let (initial_reference, _) = reference_materialize(program.law(), &world);
        assert_eq!(initial_reference, expected);
        assert_eq!(indexed.view(), &expected);
        assert_eq!(indexed.supports.get(&visible), Some(&2));
        let withdrawn = program
            .fact(
                "spatial/viewer-cell",
                &[("observer", "viewer"), ("cell", "cell-a")],
            )
            .expect("withdrawal has the declared roles");
        let changes = [Change::Withdraw(withdrawn)];
        world.apply(&changes).expect("world withdrawal applies");
        indexed.apply(&changes).expect("indexed withdrawal applies");
        let (reference, _) = reference_materialize(program.law(), &world);
        assert_eq!(reference, expected);
        assert_eq!(indexed.view(), &expected);
        assert_eq!(indexed.supports.get(&visible), Some(&1));

        let final_withdrawal = program
            .fact(
                "spatial/viewer-cell",
                &[("observer", "viewer"), ("cell", "cell-b")],
            )
            .expect("final withdrawal has the declared roles");
        let final_changes = [Change::Withdraw(final_withdrawal)];
        world
            .apply(&final_changes)
            .expect("final world withdrawal applies");
        indexed
            .apply(&final_changes)
            .expect("final indexed withdrawal applies");
        let (final_reference, _) = reference_materialize(program.law(), &world);
        assert_eq!(final_reference, FactSetView::new());
        assert_eq!(indexed.view(), &FactSetView::new());
        assert_eq!(indexed.supports.get(&visible), None);
    }
}
