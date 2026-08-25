use crate::kernel::{
    ContentId, DerivationRule, JudgmentKind, JudgmentStatus, JudgmentTarget, KernelError, Model,
    PatternId, ReferentId, RelationalContent, Result, Revision, Term,
};
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

/// The exact semantic acts supporting one asserted content leaf.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AssertionProvenance {
    occurrence: ReferentId,
    source: ReferentId,
    scope: ReferentId,
    judgment: ReferentId,
}

impl AssertionProvenance {
    pub fn occurrence(&self) -> &ReferentId {
        &self.occurrence
    }

    pub fn source(&self) -> &ReferentId {
        &self.source
    }

    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }

    pub fn judgment(&self) -> &ReferentId {
        &self.judgment
    }
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
    Asserted {
        provenance: Vec<AssertionProvenance>,
    },
    Derived {
        rule: crate::kernel::ReferentId,
        governing_law: crate::kernel::ReferentId,
        authority: crate::kernel::ReferentId,
        scope: crate::kernel::ReferentId,
        premises: Vec<RelationalContent>,
        substitution: BTreeMap<PatternId, Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    assertions: Vec<RelationalContent>,
    proofs: BTreeMap<RelationalContent, Proof>,
    applications: BTreeMap<ContentId, RelationalContent>,
}

impl Closure {
    pub fn contents(&self) -> &[RelationalContent] {
        &self.assertions
    }

    pub fn proof(&self, clause: &RelationalContent) -> Option<&Proof> {
        self.proofs.get(clause)
    }

    pub(crate) fn content<'a>(
        &'a self,
        model: &'a Model,
        id: &ContentId,
    ) -> Option<&'a RelationalContent> {
        self.applications.get(id).or_else(|| model.content(id))
    }
}

pub(super) fn limit_error(kind: &str, name: &str, value: usize) -> KernelError {
    KernelError::new(format!("closure {kind} limit exceeded ({name}={value})"))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Candidate {
    rule: crate::kernel::ReferentId,
    governing_law: crate::kernel::ReferentId,
    authority: crate::kernel::ReferentId,
    scope: crate::kernel::ReferentId,
    premises: Vec<RelationalContent>,
    substitution: BTreeMap<PatternId, Term>,
    dependencies: BTreeMap<ContentId, RelationalContent>,
}

struct RuleCandidateContext<'a> {
    rule: &'a DerivationRule,
    patterns: &'a [&'a RelationalContent],
    conclusion: &'a RelationalContent,
    assertions: &'a [RelationalContent],
    model: &'a Model,
    applications: &'a BTreeMap<ContentId, RelationalContent>,
    limits: &'a Limits,
}

/// Saturate a Revision's admitted assertions under its positive, range-restricted laws.
pub fn saturate(revision: &Revision, limits: Limits) -> Result<Closure> {
    let mut proofs = BTreeMap::new();
    for assertion in revision.model().admitted_contents() {
        proofs.insert(
            assertion.clone(),
            Proof {
                generation: 0,
                witness: Witness::Asserted {
                    provenance: assertion_provenance(revision.model(), assertion)?,
                },
            },
        );
    }
    if proofs.len() > limits.max_assertions {
        return Err(limit_error(
            "assertion",
            "max_assertions",
            limits.max_assertions,
        ));
    }

    let mut join_attempts = 0usize;
    let mut applications = BTreeMap::new();
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
                let context = RuleCandidateContext {
                    rule,
                    patterns: &premises,
                    conclusion: revision
                        .model()
                        .content(conclusion)
                        .expect("checked rule conclusion"),
                    assertions: &assertions,
                    model: revision.model(),
                    applications: &applications,
                    limits: &limits,
                };
                collect_rule_candidates(&context, &mut join_attempts, &mut candidates)?;
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
            for (id, dependency) in candidate.dependencies {
                if let Some(existing) = applications.insert(id, dependency.clone())
                    && existing != dependency
                {
                    return Err(KernelError::new(
                        "derived recursive term has conflicting content identity",
                    ));
                }
            }
            proofs.insert(
                clause,
                Proof {
                    generation,
                    witness: Witness::Derived {
                        rule: candidate.rule,
                        governing_law: candidate.governing_law,
                        authority: candidate.authority,
                        scope: candidate.scope,
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
        applications,
    })
}

pub(super) fn assertion_provenance(
    model: &Model,
    assertion: &RelationalContent,
) -> Result<Vec<AssertionProvenance>> {
    let mut provenance = Vec::new();
    for occurrence in model
        .occurrences()
        .iter()
        .filter(|occurrence| occurrence.content() == assertion.id())
    {
        for judgment in model.judgments().iter().filter(|judgment| {
            judgment.authority() == model.id()
                && judgment.scope() == model.id()
                && judgment.status() == &JudgmentStatus::Affirmed
                && matches!(judgment.kind(), JudgmentKind::Admitted { .. })
                && match judgment.target() {
                    JudgmentTarget::Occurrence(id) => id == occurrence.id(),
                    JudgmentTarget::Content(id) => id == assertion.id(),
                }
        }) {
            provenance.push(AssertionProvenance {
                occurrence: occurrence.id().clone(),
                source: occurrence.source().clone(),
                scope: occurrence.scope().clone(),
                judgment: judgment.id().clone(),
            });
        }
    }
    provenance.sort();
    provenance.dedup();
    if provenance.is_empty() {
        return Err(KernelError::new(
            "admitted assertion has no exact occurrence and judgment provenance",
        ));
    }
    Ok(provenance)
}

fn collect_rule_candidates(
    context: &RuleCandidateContext<'_>,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<RelationalContent, Candidate>,
) -> Result<()> {
    collect_joins(
        context,
        join_attempts,
        candidates,
        0,
        BTreeMap::new(),
        Vec::new(),
    )
}

fn collect_joins(
    context: &RuleCandidateContext<'_>,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<RelationalContent, Candidate>,
    premise_index: usize,
    substitution: BTreeMap<PatternId, Term>,
    premises: Vec<RelationalContent>,
) -> Result<()> {
    if premise_index == context.patterns.len() {
        let instantiated =
            crate::kernel::matching::instantiate(context.conclusion, &substitution, |id| {
                context.model.content(id)
            })?;
        let candidate = Candidate {
            rule: context.rule.id().clone(),
            governing_law: context.rule.governing_law().clone(),
            authority: context.rule.authority().clone(),
            scope: context.rule.scope().clone(),
            premises,
            substitution,
            dependencies: instantiated.dependencies,
        };
        match candidates.get_mut(&instantiated.root) {
            Some(chosen) if candidate < *chosen => *chosen = candidate,
            None => {
                candidates.insert(instantiated.root, candidate);
            }
            _ => {}
        }
        return Ok(());
    }
    let pattern = context.patterns[premise_index];
    for assertion in context.assertions {
        if *join_attempts >= context.limits.max_join_attempts {
            return Err(limit_error(
                "join attempt",
                "max_join_attempts",
                context.limits.max_join_attempts,
            ));
        }
        *join_attempts += 1;
        let Some(next_substitution) = crate::kernel::matching::unify(
            pattern,
            assertion,
            &substitution,
            true,
            |id| context.model.content(id),
            |id| {
                context
                    .applications
                    .get(id)
                    .or_else(|| context.model.content(id))
            },
        ) else {
            continue;
        };
        let mut next_premises = premises.clone();
        next_premises.push(assertion.clone());
        collect_joins(
            context,
            join_attempts,
            candidates,
            premise_index + 1,
            next_substitution,
            next_premises,
        )?;
    }
    Ok(())
}
