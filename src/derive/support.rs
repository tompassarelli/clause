use super::closure::{Limits, limit_error, saturate};
use crate::kernel::{PatternId, RelationalContent, Result, Revision, RevisionId, Term};
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
    conclusion: RelationalContent,
    witness: SupportWitness,
}

impl SupportProof {
    pub fn conclusion(&self) -> &RelationalContent {
        &self.conclusion
    }

    pub fn witness(&self) -> &SupportWitness {
        &self.witness
    }

    fn contains(&self, clause: &RelationalContent) -> bool {
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
        rule: crate::kernel::ReferentId,
        premises: Vec<SupportProof>,
        substitution: BTreeMap<PatternId, Term>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Support {
    assertion_key: Vec<RelationalContent>,
    assertions: Vec<RelationalContent>,
    proof: SupportProof,
}

impl Support {
    pub fn assertions(&self) -> &[RelationalContent] {
        &self.assertions
    }

    pub fn proof(&self) -> &SupportProof {
        &self.proof
    }

    pub(crate) fn assertion_key(&self) -> &[RelationalContent] {
        &self.assertion_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportFrontier {
    revision: RevisionId,
    target: RelationalContent,
    limits: SupportLimits,
    status: SupportStatus,
    expansions: usize,
    supports: Vec<Support>,
}

impl SupportFrontier {
    pub fn revision(&self) -> &RevisionId {
        &self.revision
    }

    pub fn target(&self) -> &RelationalContent {
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
    conclusion: RelationalContent,
    rule: crate::kernel::ReferentId,
    premises: Vec<RelationalContent>,
    substitution: BTreeMap<PatternId, Term>,
}

/// Enumerate bounded inclusion-minimal asserted supports for one ground target.
pub fn support_frontier(
    revision: &Revision,
    target: &RelationalContent,
    limits: SupportLimits,
) -> Result<SupportFrontier> {
    revision.model().validate_content(target, false)?;
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
    for rule in revision.model().derivation_rules() {
        let premises = rule
            .premises()
            .forms()
            .iter()
            .map(|id| revision.model().content(id).expect("checked rule premise"))
            .collect::<Vec<_>>();
        for conclusion in rule.conclusion().forms() {
            collect_ground_derivations(
                rule.id(),
                &premises,
                revision
                    .model()
                    .content(conclusion)
                    .expect("checked rule conclusion"),
                closure.contents(),
                revision.model(),
                &closure,
                &limits.closure,
                &mut join_attempts,
                &mut derivations,
                0,
                BTreeMap::new(),
                Vec::new(),
            )?;
        }
    }
    let mut frontiers = revision
        .model()
        .admitted_contents()
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
    supports_by_clause: &mut BTreeMap<
        RelationalContent,
        BTreeMap<Vec<RelationalContent>, SupportProof>,
    >,
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
                rule: derivation.rule.clone(),
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

fn proof_assertions(proof: &SupportProof) -> Vec<RelationalContent> {
    let mut assertions = BTreeSet::new();
    collect_proof_assertions(proof, &mut assertions);
    assertions.into_iter().collect()
}

fn ordered_proof_assertions(proof: &SupportProof) -> Vec<RelationalContent> {
    let mut seen = BTreeSet::new();
    let mut assertions = Vec::new();
    collect_ordered_proof_assertions(proof, &mut seen, &mut assertions);
    assertions
}

fn collect_ordered_proof_assertions(
    proof: &SupportProof,
    seen: &mut BTreeSet<RelationalContent>,
    assertions: &mut Vec<RelationalContent>,
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

fn collect_proof_assertions(proof: &SupportProof, assertions: &mut BTreeSet<RelationalContent>) {
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
    frontier: &mut BTreeMap<Vec<RelationalContent>, SupportProof>,
    assertions: Vec<RelationalContent>,
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

fn sorted_subset(left: &[RelationalContent], right: &[RelationalContent]) -> bool {
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
    rule: &crate::kernel::ReferentId,
    patterns: &[&RelationalContent],
    conclusion: &RelationalContent,
    assertions: &[RelationalContent],
    model: &crate::kernel::Model,
    closure: &super::closure::Closure,
    limits: &Limits,
    join_attempts: &mut usize,
    derivations: &mut BTreeSet<GroundDerivation>,
    premise_index: usize,
    substitution: BTreeMap<PatternId, Term>,
    premises: Vec<RelationalContent>,
) -> Result<()> {
    if premise_index == patterns.len() {
        let instantiated = crate::kernel::matching::instantiate(conclusion, &substitution, |id| {
            model.content(id)
        })?;
        derivations.insert(GroundDerivation {
            conclusion: instantiated.root,
            rule: rule.clone(),
            premises,
            substitution,
        });
        return Ok(());
    }
    let pattern = patterns[premise_index];
    for assertion in assertions {
        if *join_attempts >= limits.max_join_attempts {
            return Err(limit_error(
                "support join attempt",
                "max_join_attempts",
                limits.max_join_attempts,
            ));
        }
        *join_attempts += 1;
        let Some(next_substitution) = crate::kernel::matching::unify(
            pattern,
            assertion,
            &substitution,
            true,
            |id| model.content(id),
            |id| closure.content(model, id),
        ) else {
            continue;
        };
        let mut next_premises = premises.clone();
        next_premises.push(assertion.clone());
        collect_ground_derivations(
            rule,
            patterns,
            conclusion,
            assertions,
            model,
            closure,
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
