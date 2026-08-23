use super::{Closure, Limits, Proof, Witness, limit_error};
use crate::kernel::{Clause, Law, Result, Revision, Term, VariableId};
use std::collections::BTreeMap;

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
        let conclusion = super::matching::instantiate(law.conclusion(), &substitution);
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
        let Some(next_substitution) = super::matching::unify(pattern, assertion, &substitution)
        else {
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
