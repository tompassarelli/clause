//! Deterministic finite closure for admitted positive laws.

use crate::kernel::{Clause, KernelError, Law, Result, Revision, RevisionId, Term, VariableId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_assertions: usize,
    pub max_rounds: usize,
    pub max_join_attempts: usize,
}

impl Limits {
    pub fn new(max_assertions: usize, max_rounds: usize, max_join_attempts: usize) -> Self {
        Self {
            max_assertions,
            max_rounds,
            max_join_attempts,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    generation: usize,
    witness: Witness,
}

impl Proof {
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Witness {
    Asserted,
    Derived {
        law: crate::kernel::LawId,
        premises: Vec<Clause>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    assertions: Vec<Clause>,
    proofs: BTreeMap<Clause, Proof>,
}

impl Closure {
    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }
    pub fn proof(&self, clause: &Clause) -> Option<&Proof> {
        self.proofs.get(clause)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportLimits {
    pub closure: Limits,
    pub max_expansions: usize,
    pub max_supports_per_clause: usize,
}

impl SupportLimits {
    pub fn new(closure: Limits, max_expansions: usize, max_supports_per_clause: usize) -> Self {
        Self {
            closure,
            max_expansions,
            max_supports_per_clause,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportStatus {
    Complete,
    ExpansionBudgetExhausted,
    SupportBudgetExhausted,
}

impl SupportStatus {
    pub fn is_complete(self) -> bool {
        self == Self::Complete
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SupportProof {
    conclusion: Clause,
    witness: SupportWitness,
}

impl SupportProof {
    pub fn conclusion(&self) -> &Clause {
        &self.conclusion
    }
    pub fn witness(&self) -> &SupportWitness {
        &self.witness
    }

    fn contains(&self, clause: &Clause) -> bool {
        self.conclusion == *clause
            || match &self.witness {
                SupportWitness::Asserted => false,
                SupportWitness::Derived { premises, .. } => {
                    premises.iter().any(|premise| premise.contains(clause))
                }
            }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SupportWitness {
    Asserted,
    Derived {
        law: crate::kernel::LawId,
        premises: Vec<SupportProof>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Support {
    assertion_key: Vec<Clause>,
    assertions: Vec<Clause>,
    proof: SupportProof,
}

impl Support {
    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }
    pub fn proof(&self) -> &SupportProof {
        &self.proof
    }

    pub(crate) fn assertion_key(&self) -> &[Clause] {
        &self.assertion_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportFrontier {
    revision: RevisionId,
    target: Clause,
    limits: SupportLimits,
    status: SupportStatus,
    expansions: usize,
    supports: Vec<Support>,
}

impl SupportFrontier {
    pub fn revision(&self) -> &RevisionId {
        &self.revision
    }
    pub fn target(&self) -> &Clause {
        &self.target
    }
    pub fn limits(&self) -> SupportLimits {
        self.limits
    }
    pub fn status(&self) -> SupportStatus {
        self.status
    }
    pub fn expansions(&self) -> usize {
        self.expansions
    }
    pub fn supports(&self) -> &[Support] {
        &self.supports
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Candidate {
    law: crate::kernel::LawId,
    premises: Vec<Clause>,
    substitution: BTreeMap<VariableId, Term>,
}

/// Saturate a Revision's admitted assertions under its positive, range-restricted laws.
pub fn saturate(revision: &Revision, limits: Limits) -> Result<Closure> {
    let mut proofs = revision
        .model()
        .assertions()
        .iter()
        .cloned()
        .map(|assertion| {
            (
                assertion,
                Proof {
                    generation: 0,
                    witness: Witness::Asserted,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    if proofs.len() > limits.max_assertions {
        return Err(limit_error(
            "assertion",
            "max_assertions",
            limits.max_assertions,
        ));
    }

    let mut join_attempts = 0usize;
    let mut generation = 1usize;
    loop {
        let assertions = proofs.keys().cloned().collect::<Vec<_>>();
        let mut candidates = BTreeMap::<Clause, Candidate>::new();
        for law in revision.model().laws() {
            collect_law_candidates(
                law,
                &assertions,
                &limits,
                &mut join_attempts,
                &mut candidates,
            )?;
        }
        candidates.retain(|clause, _| !proofs.contains_key(clause));
        if candidates.is_empty() {
            break;
        }
        if generation > limits.max_rounds {
            return Err(limit_error("round", "max_rounds", limits.max_rounds));
        }
        if candidates.len() > limits.max_assertions.saturating_sub(proofs.len()) {
            return Err(limit_error(
                "assertion",
                "max_assertions",
                limits.max_assertions,
            ));
        }
        for (clause, candidate) in candidates {
            proofs.insert(
                clause,
                Proof {
                    generation,
                    witness: Witness::Derived {
                        law: candidate.law,
                        premises: candidate.premises,
                        substitution: candidate.substitution,
                    },
                },
            );
        }
        generation += 1;
    }
    Ok(Closure {
        assertions: proofs.keys().cloned().collect(),
        proofs,
    })
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GroundDerivation {
    conclusion: Clause,
    law: crate::kernel::LawId,
    premises: Vec<Clause>,
    substitution: BTreeMap<VariableId, Term>,
}

/// Enumerate bounded inclusion-minimal asserted supports for one ground target.
pub fn support_frontier(
    revision: &Revision,
    target: &Clause,
    limits: SupportLimits,
) -> Result<SupportFrontier> {
    revision.model().validate_clause(target, false)?;
    let closure = saturate(revision, limits.closure)?;
    if closure.proof(target).is_none() {
        return Ok(SupportFrontier {
            revision: revision.identity().clone(),
            target: target.clone(),
            limits,
            status: SupportStatus::Complete,
            expansions: 0,
            supports: Vec::new(),
        });
    }
    if limits.max_supports_per_clause == 0 {
        return Ok(SupportFrontier {
            revision: revision.identity().clone(),
            target: target.clone(),
            limits,
            status: SupportStatus::SupportBudgetExhausted,
            expansions: 0,
            supports: Vec::new(),
        });
    }
    let mut derivations = BTreeSet::new();
    let mut join_attempts = 0;
    for law in revision.model().laws() {
        collect_ground_derivations(
            law,
            closure.assertions(),
            &limits.closure,
            &mut join_attempts,
            &mut derivations,
            0,
            BTreeMap::new(),
            Vec::new(),
        )?;
    }
    let mut frontiers = revision
        .model()
        .assertions()
        .iter()
        .cloned()
        .map(|assertion| {
            let proof = SupportProof {
                conclusion: assertion.clone(),
                witness: SupportWitness::Asserted,
            };
            (
                assertion.clone(),
                BTreeMap::from([(vec![assertion], proof)]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut explored = BTreeSet::<SupportProof>::new();
    let mut expansions = 0;
    let mut status = SupportStatus::Complete;
    'fixed_point: loop {
        let mut changed = false;
        for derivation in &derivations {
            let Some(premise_frontiers) = derivation
                .premises
                .iter()
                .map(|premise| frontiers.get(premise).map(|supports| supports.values()))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let combinations = premise_frontiers
                .into_iter()
                .map(|supports| supports.cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let mut selected = Vec::with_capacity(combinations.len());
            if let Some(exhausted) = expand_derivation(
                derivation,
                &combinations,
                0,
                &mut selected,
                &mut explored,
                &mut frontiers,
                &limits,
                &mut expansions,
                &mut changed,
            ) {
                status = exhausted;
                break 'fixed_point;
            }
        }
        if !changed {
            break;
        }
    }
    let supports = if status.is_complete() {
        frontiers
            .remove(target)
            .unwrap_or_default()
            .into_iter()
            .map(|(assertion_key, proof)| Support {
                assertions: ordered_proof_assertions(&proof),
                assertion_key,
                proof,
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(SupportFrontier {
        revision: revision.identity().clone(),
        target: target.clone(),
        limits,
        status,
        expansions,
        supports,
    })
}

#[allow(clippy::too_many_arguments)]
fn expand_derivation(
    derivation: &GroundDerivation,
    frontiers: &[Vec<SupportProof>],
    index: usize,
    selected: &mut Vec<SupportProof>,
    explored: &mut BTreeSet<SupportProof>,
    supports_by_clause: &mut BTreeMap<Clause, BTreeMap<Vec<Clause>, SupportProof>>,
    limits: &SupportLimits,
    expansions: &mut usize,
    changed: &mut bool,
) -> Option<SupportStatus> {
    if index == frontiers.len() {
        if selected
            .iter()
            .any(|premise| premise.contains(&derivation.conclusion))
        {
            return None;
        }
        let proof = SupportProof {
            conclusion: derivation.conclusion.clone(),
            witness: SupportWitness::Derived {
                law: derivation.law.clone(),
                premises: selected.clone(),
                substitution: derivation.substitution.clone(),
            },
        };
        if !explored.insert(proof.clone()) {
            return None;
        }
        if *expansions >= limits.max_expansions {
            return Some(SupportStatus::ExpansionBudgetExhausted);
        }
        *expansions += 1;
        let assertions = proof_assertions(&proof);
        let frontier = supports_by_clause
            .entry(derivation.conclusion.clone())
            .or_default();
        return match insert_support(frontier, assertions, proof, limits.max_supports_per_clause) {
            InsertSupport::Unchanged => None,
            InsertSupport::Changed => {
                *changed = true;
                None
            }
            InsertSupport::BudgetExhausted => Some(SupportStatus::SupportBudgetExhausted),
        };
    }
    for proof in &frontiers[index] {
        selected.push(proof.clone());
        let exhausted = expand_derivation(
            derivation,
            frontiers,
            index + 1,
            selected,
            explored,
            supports_by_clause,
            limits,
            expansions,
            changed,
        );
        selected.pop();
        if exhausted.is_some() {
            return exhausted;
        }
    }
    None
}

fn proof_assertions(proof: &SupportProof) -> Vec<Clause> {
    let mut assertions = BTreeSet::new();
    collect_proof_assertions(proof, &mut assertions);
    assertions.into_iter().collect()
}

fn ordered_proof_assertions(proof: &SupportProof) -> Vec<Clause> {
    let mut seen = BTreeSet::new();
    let mut assertions = Vec::new();
    collect_ordered_proof_assertions(proof, &mut seen, &mut assertions);
    assertions
}

fn collect_ordered_proof_assertions(
    proof: &SupportProof,
    seen: &mut BTreeSet<Clause>,
    assertions: &mut Vec<Clause>,
) {
    match &proof.witness {
        SupportWitness::Asserted => {
            if seen.insert(proof.conclusion.clone()) {
                assertions.push(proof.conclusion.clone());
            }
        }
        SupportWitness::Derived { premises, .. } => {
            for premise in premises {
                collect_ordered_proof_assertions(premise, seen, assertions);
            }
        }
    }
}

fn collect_proof_assertions(proof: &SupportProof, assertions: &mut BTreeSet<Clause>) {
    match &proof.witness {
        SupportWitness::Asserted => {
            assertions.insert(proof.conclusion.clone());
        }
        SupportWitness::Derived { premises, .. } => {
            for premise in premises {
                collect_proof_assertions(premise, assertions);
            }
        }
    }
}

enum InsertSupport {
    Unchanged,
    Changed,
    BudgetExhausted,
}

fn insert_support(
    frontier: &mut BTreeMap<Vec<Clause>, SupportProof>,
    assertions: Vec<Clause>,
    proof: SupportProof,
    max_supports: usize,
) -> InsertSupport {
    if let Some(chosen) = frontier.get_mut(&assertions) {
        if proof < *chosen {
            *chosen = proof;
            return InsertSupport::Changed;
        }
        return InsertSupport::Unchanged;
    }
    if frontier
        .keys()
        .any(|known| sorted_subset(known, &assertions))
    {
        return InsertSupport::Unchanged;
    }
    let supersets = frontier
        .keys()
        .filter(|known| sorted_subset(&assertions, known))
        .cloned()
        .collect::<Vec<_>>();
    if frontier.len() + 1 - supersets.len() > max_supports {
        return InsertSupport::BudgetExhausted;
    }
    for superset in supersets {
        frontier.remove(&superset);
    }
    frontier.insert(assertions, proof);
    InsertSupport::Changed
}

fn sorted_subset(left: &[Clause], right: &[Clause]) -> bool {
    let mut right_index = 0;
    for wanted in left {
        while right_index < right.len() && right[right_index] < *wanted {
            right_index += 1;
        }
        if right.get(right_index) != Some(wanted) {
            return false;
        }
        right_index += 1;
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn collect_ground_derivations(
    law: &Law,
    assertions: &[Clause],
    limits: &Limits,
    join_attempts: &mut usize,
    derivations: &mut BTreeSet<GroundDerivation>,
    premise_index: usize,
    substitution: BTreeMap<VariableId, Term>,
    premises: Vec<Clause>,
) -> Result<()> {
    if premise_index == law.premises().len() {
        derivations.insert(GroundDerivation {
            conclusion: instantiate(law.conclusion(), &substitution),
            law: law.id().clone(),
            premises,
            substitution,
        });
        return Ok(());
    }
    let pattern = &law.premises()[premise_index];
    for assertion in assertions {
        if *join_attempts >= limits.max_join_attempts {
            return Err(limit_error(
                "support join attempt",
                "max_join_attempts",
                limits.max_join_attempts,
            ));
        }
        *join_attempts += 1;
        let Some(next_substitution) = unify(pattern, assertion, &substitution) else {
            continue;
        };
        let mut next_premises = premises.clone();
        next_premises.push(assertion.clone());
        collect_ground_derivations(
            law,
            assertions,
            limits,
            join_attempts,
            derivations,
            premise_index + 1,
            next_substitution,
            next_premises,
        )?;
    }
    Ok(())
}

fn collect_law_candidates(
    law: &Law,
    assertions: &[Clause],
    limits: &Limits,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<Clause, Candidate>,
) -> Result<()> {
    collect_joins(
        law,
        assertions,
        limits,
        join_attempts,
        candidates,
        0,
        BTreeMap::new(),
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn collect_joins(
    law: &Law,
    assertions: &[Clause],
    limits: &Limits,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<Clause, Candidate>,
    premise_index: usize,
    substitution: BTreeMap<VariableId, Term>,
    premises: Vec<Clause>,
) -> Result<()> {
    if premise_index == law.premises().len() {
        let conclusion = instantiate(law.conclusion(), &substitution);
        let candidate = Candidate {
            law: law.id().clone(),
            premises,
            substitution,
        };
        match candidates.get_mut(&conclusion) {
            Some(chosen) if candidate < *chosen => *chosen = candidate,
            None => {
                candidates.insert(conclusion, candidate);
            }
            _ => {}
        }
        return Ok(());
    }
    let pattern = &law.premises()[premise_index];
    for assertion in assertions {
        if *join_attempts >= limits.max_join_attempts {
            return Err(limit_error(
                "join attempt",
                "max_join_attempts",
                limits.max_join_attempts,
            ));
        }
        *join_attempts += 1;
        let Some(next_substitution) = unify(pattern, assertion, &substitution) else {
            continue;
        };
        let mut next_premises = premises.clone();
        next_premises.push(assertion.clone());
        collect_joins(
            law,
            assertions,
            limits,
            join_attempts,
            candidates,
            premise_index + 1,
            next_substitution,
            next_premises,
        )?;
    }
    Ok(())
}

fn unify(
    pattern: &Clause,
    assertion: &Clause,
    substitution: &BTreeMap<VariableId, Term>,
) -> Option<BTreeMap<VariableId, Term>> {
    if pattern.relation() != assertion.relation()
        || pattern.roles().len() != assertion.roles().len()
    {
        return None;
    }
    let mut substitution = substitution.clone();
    for (role, pattern_term) in pattern.roles() {
        let assertion_term = assertion.roles().get(role)?;
        match pattern_term {
            Term::Variable { id, typ } if typ == assertion_term.typ() => match substitution.get(id)
            {
                Some(bound) if bound != assertion_term => return None,
                Some(_) => {}
                None => {
                    substitution.insert(id.clone(), assertion_term.clone());
                }
            },
            Term::Variable { .. } => return None,
            _ if pattern_term != assertion_term => return None,
            _ => {}
        }
    }
    Some(substitution)
}

fn instantiate(pattern: &Clause, substitution: &BTreeMap<VariableId, Term>) -> Clause {
    Clause::new(
        pattern.relation().clone(),
        pattern
            .roles()
            .iter()
            .map(|(role, term)| {
                let value = match term {
                    Term::Variable { id, .. } => substitution
                        .get(id)
                        .expect("admitted law conclusions are range-restricted")
                        .clone(),
                    _ => term.clone(),
                };
                (role.clone(), value)
            })
            .collect(),
    )
    .expect("instantiating an admitted conclusion preserves its complete role map")
}

fn limit_error(kind: &str, name: &str, value: usize) -> KernelError {
    KernelError::new(format!("closure {kind} limit exceeded ({name}={value})"))
}

#[cfg(test)]
mod tests {
    use super::{Limits, SupportLimits, SupportStatus, Witness, saturate, support_frontier};
    use crate::kernel::{
        Cardinality, Clause, InlineSentencePart, Law, LawId, Mode, Model, ModelId, Name, Relation,
        RelationId, Revision, RevisionId, Role, RoleId, SentenceShape, Term, Type, TypeId,
        VariableId,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn name(value: &str) -> Name {
        Name::new(value.to_owned()).unwrap()
    }
    fn id(value: &str) -> TypeId {
        TypeId::new(name(value)).unwrap()
    }
    fn relation_id(value: &str) -> RelationId {
        RelationId::new(name(value)).unwrap()
    }
    fn role(value: &str, typ: &TypeId) -> Role {
        Role::new(RoleId::new(name(value)).unwrap(), typ.clone())
    }
    fn variable(value: &str, typ: &TypeId) -> Term {
        Term::variable(VariableId::new(name(value)).unwrap(), typ.clone())
    }
    fn text(value: &str, typ: &TypeId) -> Term {
        Term::value(typ.clone(), value.to_owned()).unwrap()
    }

    fn relation(identity: &RelationId, typ: &TypeId) -> Relation {
        let from = role("from", typ);
        let to = role("to", typ);
        Relation::new(
            identity.clone(),
            SentenceShape::new(vec![
                InlineSentencePart::Role(from.clone()),
                InlineSentencePart::Literal("reaches".to_owned()),
                InlineSentencePart::Role(to.clone()),
            ])
            .unwrap(),
            vec![
                Mode::finite(
                    vec![from.id().clone()],
                    vec![to.id().clone()],
                    Cardinality::Many,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn clause(identity: &RelationId, from: Term, to: Term) -> Clause {
        Clause::new(
            identity.clone(),
            BTreeMap::from([
                (RoleId::new(name("from")).unwrap(), from),
                (RoleId::new(name("to")).unwrap(), to),
            ]),
        )
        .unwrap()
    }

    fn revision(assertions: Vec<Clause>, laws: Vec<Law>) -> Revision {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let model = Model::new(
            ModelId::new(name("map")).unwrap(),
            BTreeMap::from([(text_type.clone(), Type::new(text_type.clone()))]),
            BTreeSet::new(),
            BTreeMap::from([
                (reaches.clone(), relation(&reaches, &text_type)),
                (links.clone(), relation(&links, &text_type)),
            ]),
            assertions,
            laws,
        )
        .unwrap();
        Revision::reloaded(RevisionId::from_digest([3; 32]), model)
    }

    #[test]
    fn typed_multi_round_closure_selects_canonical_witnesses() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let subject = variable("subject", &text_type);
        let destination = variable("destination", &text_type);
        let copy = Law::new(
            LawId::new(name("map/copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject, destination),
        )
        .unwrap();
        let closure = saturate(
            &revision(
                vec![clause(
                    &links,
                    text("North", &text_type),
                    text("Store", &text_type),
                )],
                vec![copy],
            ),
            Limits::new(10, 10, 100),
        )
        .unwrap();
        let derived = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        assert_eq!(closure.assertions().len(), 2);
        assert_eq!(closure.proof(&derived).unwrap().generation(), 1);
        assert!(matches!(
            closure.proof(&derived).unwrap().witness(),
            Witness::Derived { .. }
        ));
    }

    #[test]
    fn reversed_law_source_order_admits_the_same_model_and_closure() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let subject = variable("subject", &text_type);
        let destination = variable("destination", &text_type);
        let law_a = Law::new(
            LawId::new(name("map/a-copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject.clone(), destination.clone()),
        )
        .unwrap();
        let law_z = Law::new(
            LawId::new(name("map/z-copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject, destination),
        )
        .unwrap();
        let assertions = vec![clause(
            &links,
            text("North", &text_type),
            text("Store", &text_type),
        )];
        let forward = revision(assertions.clone(), vec![law_a.clone(), law_z.clone()]);
        let reversed = revision(assertions, vec![law_z, law_a]);
        assert_eq!(forward.model(), reversed.model());
        assert_eq!(
            saturate(&forward, Limits::new(10, 10, 100)).unwrap(),
            saturate(&reversed, Limits::new(10, 10, 100)).unwrap(),
        );
    }

    #[test]
    fn support_frontier_remains_minimal_and_typed() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let subject = variable("subject", &text_type);
        let destination = variable("destination", &text_type);
        let copy = Law::new(
            LawId::new(name("map/copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject, destination),
        )
        .unwrap();
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let frontier = support_frontier(
            &revision(
                vec![clause(
                    &links,
                    text("North", &text_type),
                    text("Store", &text_type),
                )],
                vec![copy],
            ),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 10, 10),
        )
        .unwrap();
        assert_eq!(frontier.status(), SupportStatus::Complete);
        assert_eq!(frontier.supports().len(), 1);
        assert_eq!(frontier.supports()[0].assertions().len(), 1);
    }

    #[test]
    fn support_members_follow_the_canonical_proof_path() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let first = clause(&links, text("Zulu", &text_type), text("First", &text_type));
        let second = clause(
            &links,
            text("Alpha", &text_type),
            text("Second", &text_type),
        );
        assert!(second < first);
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let law = Law::new(
            LawId::new(name("map/path-order")).unwrap(),
            vec![first.clone(), second.clone()],
            target.clone(),
        )
        .unwrap();
        let frontier = support_frontier(
            &revision(vec![second.clone(), first.clone()], vec![law]),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 10, 10),
        )
        .unwrap();
        let support = &frontier.supports()[0];
        assert_eq!(support.assertion_key(), &[second.clone(), first.clone()]);
        assert_eq!(support.assertions(), &[first, second]);
    }

    #[test]
    fn incomplete_frontier_does_not_expose_a_provisional_superset() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let alpha = clause(&links, text("Alpha", &text_type), text("One", &text_type));
        let beta = clause(&links, text("Beta", &text_type), text("Two", &text_type));
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let wide = Law::new(
            LawId::new(name("map/a-wide")).unwrap(),
            vec![alpha.clone(), beta.clone()],
            target.clone(),
        )
        .unwrap();
        let narrow = Law::new(
            LawId::new(name("map/z-narrow")).unwrap(),
            vec![alpha.clone()],
            target.clone(),
        )
        .unwrap();
        let frontier = support_frontier(
            &revision(vec![alpha, beta], vec![wide, narrow]),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 1, 10),
        )
        .unwrap();
        assert_eq!(frontier.status(), SupportStatus::ExpansionBudgetExhausted);
        assert!(frontier.supports().is_empty());
    }

    #[test]
    fn absent_target_has_a_complete_empty_frontier_without_support_budget() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let frontier = support_frontier(
            &revision(
                vec![clause(
                    &links,
                    text("Alpha", &text_type),
                    text("Beta", &text_type),
                )],
                Vec::new(),
            ),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 0, 0),
        )
        .unwrap();
        assert_eq!(frontier.status(), SupportStatus::Complete);
        assert!(frontier.supports().is_empty());
    }
}
