use crate::kernel::{KernelError, PatternId, RelationalContent, Result, Revision, Term};
use std::collections::BTreeMap;

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
        rule: crate::kernel::ReferentId,
        premises: Vec<RelationalContent>,
        substitution: BTreeMap<PatternId, Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    assertions: Vec<RelationalContent>,
    proofs: BTreeMap<RelationalContent, Proof>,
}

impl Closure {
    pub fn contents(&self) -> &[RelationalContent] {
        &self.assertions
    }

    pub fn proof(&self, clause: &RelationalContent) -> Option<&Proof> {
        self.proofs.get(clause)
    }
}

pub(super) fn limit_error(kind: &str, name: &str, value: usize) -> KernelError {
    KernelError::new(format!("closure {kind} limit exceeded ({name}={value})"))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Candidate {
    rule: crate::kernel::ReferentId,
    premises: Vec<RelationalContent>,
    substitution: BTreeMap<PatternId, Term>,
}

/// Saturate a Revision's admitted assertions under its positive, range-restricted laws.
pub fn saturate(revision: &Revision, limits: Limits) -> Result<Closure> {
    let mut proofs = revision
        .model()
        .admitted_contents()
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
        let mut candidates = BTreeMap::<RelationalContent, Candidate>::new();
        for rule in revision.model().derivation_rules() {
            let premises = rule
                .premises()
                .forms()
                .iter()
                .map(|id| revision.model().content(id).expect("checked rule premise"))
                .collect::<Vec<_>>();
            for conclusion in rule.conclusion().forms() {
                collect_rule_candidates(
                    rule.id(),
                    &premises,
                    revision
                        .model()
                        .content(conclusion)
                        .expect("checked rule conclusion"),
                    &assertions,
                    &limits,
                    &mut join_attempts,
                    &mut candidates,
                )?;
            }
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
                        rule: candidate.rule,
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

fn collect_rule_candidates(
    rule: &crate::kernel::ReferentId,
    patterns: &[&RelationalContent],
    conclusion: &RelationalContent,
    assertions: &[RelationalContent],
    limits: &Limits,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<RelationalContent, Candidate>,
) -> Result<()> {
    collect_joins(
        rule,
        patterns,
        conclusion,
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
    rule: &crate::kernel::ReferentId,
    patterns: &[&RelationalContent],
    conclusion: &RelationalContent,
    assertions: &[RelationalContent],
    limits: &Limits,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<RelationalContent, Candidate>,
    premise_index: usize,
    substitution: BTreeMap<PatternId, Term>,
    premises: Vec<RelationalContent>,
) -> Result<()> {
    if premise_index == patterns.len() {
        let conclusion = super::matching::instantiate(conclusion, &substitution);
        let candidate = Candidate {
            rule: rule.clone(),
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
    let pattern = patterns[premise_index];
    for assertion in assertions {
        if *join_attempts >= limits.max_join_attempts {
            return Err(limit_error(
                "join attempt",
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
        collect_joins(
            rule,
            patterns,
            conclusion,
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
