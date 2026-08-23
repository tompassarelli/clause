use crate::{
    derive::{self, Closure, Limits, SupportLimits, SupportProof, SupportWitness},
    kernel::{Clause, KernelError, QueryPlan, Result, Revision},
};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClauseNode {
    pub relation: String,
    pub roles: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Witness {
    Asserted,
    Derived {
        law: String,
        premises: Vec<usize>,
        substitution: Vec<(String, String)>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WitnessEdge {
    pub conclusion: usize,
    pub witness: Witness,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhyGraph {
    pub root: usize,
    pub nodes: Vec<ClauseNode>,
    pub witnesses: Vec<WitnessEdge>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Proof {
    pub why: WhyGraph,
}

/// One inclusion-minimal asserted support and its canonical derivation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhySupport {
    pub assertions: Vec<Clause>,
    pub why: WhyGraph,
}

/// The bounded projection of every minimal support for one target fact.
///
/// `complete` is false when the shared support kernel stopped at a budget.  A
/// partial result is still useful for inspection, but is never presented as
/// the complete set of alternatives.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhyAll {
    pub target: Clause,
    pub alternatives: Vec<WhySupport>,
    pub complete: bool,
    pub expansions: usize,
}

impl WhyAll {
    pub fn alternative_count(&self) -> usize {
        self.alternatives.len()
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOutput {
    pub results: Vec<String>,
    pub proofs: Vec<Proof>,
}

/// Answer one admitted query from a bounded derived closure.
///
/// The supplied limits are part of execution's contract: it never performs
/// unbounded saturation or mutates the sealed revision it explains.
pub fn execute(revision: &Revision, plan: &QueryPlan, limits: Limits) -> Result<QueryOutput> {
    if revision.plan()? != *plan {
        return Err(KernelError::new("query plan does not belong to revision"));
    }
    let sought = match plan.sought() {
        [role] => role,
        _ => {
            return Err(KernelError::new(
                "query output requires exactly one sought role",
            ));
        }
    };
    let query = revision.model().query();
    let requested = query
        .roles()
        .iter()
        .filter(|(_, term)| !term.is_variable())
        .collect::<Vec<_>>();
    let closure = derive::saturate(revision, limits)?;

    let mut rows = closure
        .facts()
        .iter()
        .filter(|fact| {
            fact.relation() == query.relation()
                && requested.iter().all(|(role, wanted)| {
                    fact.roles().get(*role).map(|term| term.text()) == Some(wanted.text())
                })
        })
        .map(|fact| row(&closure, fact, sought))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.why.cmp(&right.1.why))
    });

    Ok(QueryOutput {
        results: rows.iter().map(|(value, _)| value.clone()).collect(),
        proofs: rows.into_iter().map(|(_, proof)| proof).collect(),
    })
}

/// Return the chosen, acyclic proof graph for `fact`, if the bounded closure
/// entails it.
pub fn why(revision: &Revision, fact: &Clause, limits: Limits) -> Result<Option<WhyGraph>> {
    let frontier = derive::support_frontier(
        revision,
        fact,
        SupportLimits::new(limits, limits.max_join_attempts, limits.max_facts),
    )?;
    frontier
        .supports()
        .first()
        .map(|support| support_graph(support.proof()))
        .transpose()
}

/// Return all bounded, inclusion-minimal asserted supports for `fact`.
///
/// This is a projection only: support enumeration and deduplication remain in
/// `derive::support_frontier`.  `Limits` is accepted as a convenience and
/// derives a support budget from its fact bound; callers needing an explicit
/// alternative budget should pass `SupportLimits` directly.
pub fn why_all<L>(revision: &Revision, fact: &Clause, limits: L) -> Result<Option<WhyAll>>
where
    L: Into<SupportLimits>,
{
    let frontier = derive::support_frontier(revision, fact, limits.into())?;
    if frontier.supports().is_empty() {
        return Ok(None);
    }
    let alternatives = frontier
        .supports()
        .iter()
        .map(|support| {
            Ok(WhySupport {
                assertions: support.assertions().to_vec(),
                why: support_graph(support.proof())?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(WhyAll {
        target: fact.clone(),
        alternatives,
        complete: frontier.status().is_complete(),
        expansions: frontier.expansions(),
    }))
}

impl From<Limits> for SupportLimits {
    fn from(limits: Limits) -> Self {
        SupportLimits::new(limits, limits.max_join_attempts, limits.max_facts)
    }
}

fn row(closure: &Closure, fact: &Clause, sought: &str) -> Result<(String, Proof)> {
    let result = fact
        .roles()
        .get(sought)
        .ok_or_else(|| KernelError::new("fact does not fill sought role"))?
        .text()
        .to_owned();
    Ok((
        result,
        Proof {
            why: graph(closure, fact)?.expect("closure fact has a proof"),
        },
    ))
}

fn graph(closure: &Closure, root: &Clause) -> Result<Option<WhyGraph>> {
    if closure.proof(root).is_none() {
        return Ok(None);
    }

    let mut facts = Vec::new();
    let mut indices = BTreeMap::new();
    let mut witnesses = Vec::new();
    let root = add_fact(root, closure, &mut facts, &mut indices, &mut witnesses)?;
    witnesses.sort_by_key(|edge| edge.conclusion);
    Ok(Some(WhyGraph {
        root,
        nodes: facts.into_iter().map(node).collect(),
        witnesses,
    }))
}

fn support_graph(root: &SupportProof) -> Result<WhyGraph> {
    let mut facts = Vec::new();
    let mut indices = BTreeMap::new();
    let mut witnesses = Vec::new();
    let root_index = add_support_fact(root, &mut facts, &mut indices, &mut witnesses)?;
    witnesses.sort_by_key(|edge| edge.conclusion);
    Ok(WhyGraph {
        root: root_index,
        nodes: facts.into_iter().map(node).collect(),
        witnesses,
    })
}

fn add_support_fact(
    proof: &SupportProof,
    facts: &mut Vec<Clause>,
    indices: &mut BTreeMap<Clause, usize>,
    witnesses: &mut Vec<WitnessEdge>,
) -> Result<usize> {
    let fact = proof.conclusion();
    if let Some(index) = indices.get(fact) {
        return Ok(*index);
    }
    let conclusion = facts.len();
    facts.push(fact.clone());
    indices.insert(fact.clone(), conclusion);
    let witness = match proof.witness() {
        SupportWitness::Asserted => Witness::Asserted,
        SupportWitness::Derived {
            law,
            premises,
            substitution,
        } => Witness::Derived {
            law: law.clone(),
            premises: premises
                .iter()
                .map(|premise| add_support_fact(premise, facts, indices, witnesses))
                .collect::<Result<Vec<_>>>()?,
            substitution: substitution
                .iter()
                .map(|(variable, value)| (variable.clone(), value.clone()))
                .collect(),
        },
    };
    witnesses.push(WitnessEdge {
        conclusion,
        witness,
    });
    Ok(conclusion)
}

fn add_fact(
    fact: &Clause,
    closure: &Closure,
    facts: &mut Vec<Clause>,
    indices: &mut BTreeMap<Clause, usize>,
    witnesses: &mut Vec<WitnessEdge>,
) -> Result<usize> {
    if let Some(index) = indices.get(fact) {
        return Ok(*index);
    }

    let conclusion = facts.len();
    facts.push(fact.clone());
    indices.insert(fact.clone(), conclusion);
    let proof = closure
        .proof(fact)
        .ok_or_else(|| KernelError::new("closure fact has no chosen witness"))?;
    let witness = match proof.witness() {
        derive::Witness::Asserted => Witness::Asserted,
        derive::Witness::Derived {
            law,
            premises,
            substitution,
        } => Witness::Derived {
            law: law.clone(),
            premises: premises
                .iter()
                .map(|premise| add_fact(premise, closure, facts, indices, witnesses))
                .collect::<Result<Vec<_>>>()?,
            substitution: substitution
                .iter()
                .map(|(variable, value)| (variable.clone(), value.clone()))
                .collect(),
        },
    };
    witnesses.push(WitnessEdge {
        conclusion,
        witness,
    });
    Ok(conclusion)
}

fn node(fact: Clause) -> ClauseNode {
    ClauseNode {
        relation: fact.relation().to_owned(),
        roles: fact
            .roles()
            .iter()
            .map(|(role, term)| (role.clone(), term.text().to_owned()))
            .collect(),
    }
}

pub fn canonical_json(output: &QueryOutput) -> String {
    let results = output
        .results
        .iter()
        .map(|value| quoted(value))
        .collect::<Vec<_>>()
        .join(",");
    let proofs = output
        .proofs
        .iter()
        .map(|proof| {
            let nodes = proof
                .why
                .nodes
                .iter()
                .map(|node| {
                    let roles = node
                        .roles
                        .iter()
                        .map(|(name, value)| format!("[{},{}]", quoted(name), quoted(value)))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "[\"clause\",\"relation\",{},\"roles\",[{roles}]]",
                        quoted(&node.relation)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let witnesses = proof
                .why
                .witnesses
                .iter()
                .map(|edge| match &edge.witness {
                    Witness::Asserted => format!("[\"asserted\",{}]", edge.conclusion),
                    Witness::Derived {
                        law,
                        premises,
                        substitution,
                    } => {
                        let premises = premises
                            .iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        let substitution = substitution
                            .iter()
                            .map(|(variable, value)| {
                                format!("[{},{}]", quoted(variable), quoted(value))
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(
                            "[\"derived\",{},\"law\",{},\"premises\",[{premises}],\"substitution\",[{substitution}]]",
                            edge.conclusion,
                            quoted(law),
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "[\"why\",[\"root\",{}],[\"clauses\",[{nodes}]],[\"witnesses\",[{witnesses}]]]",
                proof.why.root,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[\"clause-query-output-v2\",[\"results\",[{results}]],[\"proofs\",[{proofs}]]]")
}

fn quoted(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value <= '\u{1f}' => write!(escaped, "\\u{:04x}", value as u32).unwrap(),
            value => escaped.push(value),
        }
    }
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{Cardinality, Clause, Law, Mode, Model, Relation, Role, Sentence, Term};
    fn limits() -> Limits {
        Limits::new(100, 10, 10_000)
    }

    fn relation(name: &str) -> Relation {
        Relation::new(
            name,
            vec![
                Role::new("from", "Place").unwrap(),
                Role::new("to", "Place").unwrap(),
            ],
            Sentence::new("from", "reaches", "to").unwrap(),
            vec![Mode::finite(vec!["from".into()], vec!["to".into()], Cardinality::Many).unwrap()],
        )
        .unwrap()
    }

    fn clause(relation: &str, from: Term, to: Term) -> Clause {
        Clause::new(relation, vec![("from".into(), from), ("to".into(), to)]).unwrap()
    }

    #[test]
    fn impact_query_returns_derived_results_with_acyclic_why_graphs() {
        let literal = |relation: &str, from: &str, to: &str| {
            clause(
                relation,
                Term::literal(from).unwrap(),
                Term::literal(to).unwrap(),
            )
        };
        let pattern = |relation: &str, from: &str, to: &str| {
            clause(
                relation,
                Term::variable(from).unwrap(),
                Term::variable(to).unwrap(),
            )
        };
        let north_store = literal("impact/links", "North", "Store");
        let store_beagle = literal("impact/hosts", "Store", "Beagle");
        let query = clause(
            "impact/reaches",
            Term::literal("North").unwrap(),
            Term::variable("destination").unwrap(),
        );
        let revision = Revision::admit(
            Model::with_laws(
                vec![
                    relation("impact/links"),
                    relation("impact/hosts"),
                    relation("impact/reaches"),
                ],
                vec![north_store.clone(), store_beagle.clone()],
                vec![
                    Law::new(
                        "impact/direct",
                        vec![pattern("impact/links", "source", "destination")],
                        pattern("impact/reaches", "source", "destination"),
                    )
                    .unwrap(),
                    Law::new(
                        "impact/transitive",
                        vec![
                            pattern("impact/reaches", "source", "middle"),
                            pattern("impact/hosts", "middle", "destination"),
                        ],
                        pattern("impact/reaches", "source", "destination"),
                    )
                    .unwrap(),
                ],
                query,
                "ascending",
            )
            .unwrap(),
        );

        let output = execute(&revision, &revision.plan().unwrap(), limits()).unwrap();
        assert_eq!(output.results, ["Beagle", "Store"]);
        let graph = &output.proofs[0].why;
        assert_eq!(graph.root, 0);
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.witnesses.len(), 4);
        assert_eq!(graph.nodes[0].relation, "impact/reaches");
        assert_eq!(
            graph.nodes[0].roles,
            vec![
                ("from".into(), "North".into()),
                ("to".into(), "Beagle".into())
            ]
        );
        assert!(matches!(
            graph.witnesses[0].witness,
            Witness::Derived { ref law, ref premises, .. }
                if law == "impact/transitive" && premises == &[1, 3]
        ));
        assert!(matches!(
            graph.witnesses[1].witness,
            Witness::Derived { ref law, ref premises, .. }
                if law == "impact/direct" && premises == &[2]
        ));
        assert!(graph.witnesses.iter().all(|edge| match &edge.witness {
            Witness::Asserted => true,
            Witness::Derived { premises, .. } =>
                premises.iter().all(|premise| *premise > edge.conclusion),
        }));
        assert_eq!(
            why(
                &revision,
                &literal("impact/reaches", "North", "Beagle"),
                limits()
            )
            .unwrap(),
            Some(graph.clone())
        );
        assert!(
            why(
                &revision,
                &literal("impact/reaches", "North", "Missing"),
                limits()
            )
            .unwrap()
            .is_none()
        );
        assert!(canonical_json(&output).starts_with("[\"clause-query-output-v2\","));
    }

    #[test]
    fn why_all_projects_independent_supports_and_deduplicates_derivations() {
        let literal = |relation: &str, from: &str, to: &str| {
            clause(
                relation,
                Term::literal(from).unwrap(),
                Term::literal(to).unwrap(),
            )
        };
        let pattern = |relation: &str, from: &str, to: &str| {
            clause(
                relation,
                Term::variable(from).unwrap(),
                Term::variable(to).unwrap(),
            )
        };
        let target = literal("impact/reaches", "North", "Beagle");
        let revision = Revision::admit(
            Model::with_laws(
                vec![
                    relation("impact/links"),
                    relation("impact/hosts"),
                    relation("impact/reaches"),
                ],
                vec![
                    literal("impact/links", "North", "Store"),
                    literal("impact/hosts", "Store", "Beagle"),
                    literal("impact/links", "North", "Relay"),
                    literal("impact/hosts", "Relay", "Beagle"),
                ],
                vec![
                    Law::new(
                        "impact/direct",
                        vec![pattern("impact/links", "source", "destination")],
                        pattern("impact/reaches", "source", "destination"),
                    )
                    .unwrap(),
                    Law::new(
                        "impact/transitive",
                        vec![
                            pattern("impact/reaches", "source", "middle"),
                            pattern("impact/hosts", "middle", "destination"),
                        ],
                        pattern("impact/reaches", "source", "destination"),
                    )
                    .unwrap(),
                    // A distinct derivation must not create a third support.
                    Law::new(
                        "impact/transitive-copy",
                        vec![
                            pattern("impact/reaches", "source", "middle"),
                            pattern("impact/hosts", "middle", "destination"),
                        ],
                        pattern("impact/reaches", "source", "destination"),
                    )
                    .unwrap(),
                ],
                clause(
                    "impact/reaches",
                    Term::literal("North").unwrap(),
                    Term::variable("destination").unwrap(),
                ),
                "ascending",
            )
            .unwrap(),
        );

        let limits = SupportLimits::new(limits(), 10_000, 10);
        let all = why_all(&revision, &target, limits).unwrap().unwrap();
        assert!(all.is_complete());
        assert_eq!(all.alternative_count(), 2);
        assert_eq!(
            all.alternatives
                .iter()
                .map(|alternative| alternative.assertions.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![
                    literal("impact/hosts", "Relay", "Beagle"),
                    literal("impact/links", "North", "Relay"),
                ],
                vec![
                    literal("impact/hosts", "Store", "Beagle"),
                    literal("impact/links", "North", "Store"),
                ],
            ]
        );
        assert_eq!(
            why(&revision, &target, limits()).unwrap(),
            Some(all.alternatives[0].why.clone())
        );
    }
}
