use super::closure::{Limits, limit_error, saturate};
use crate::kernel::{Clause, Law, Result, Revision, RevisionId, Term, VariableId};
use std::collections::{BTreeMap, BTreeSet};

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
            conclusion: super::matching::instantiate(law.conclusion(), &substitution),
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
        let Some(next_substitution) = super::matching::unify(pattern, assertion, &substitution)
        else {
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
