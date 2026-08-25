//! Proof and minimal-support explanation projections.

use std::collections::BTreeMap;

use crate::{
    derive::{self, Closure, Limits, SupportLimits, SupportProof, SupportWitness},
    kernel::{KernelError, RelationalContent, Result, Revision, RevisionId},
};

use super::{ClauseNode, Proof, WhyAll, WhyGraph, WhySupport, Witness, WitnessEdge};

pub(super) fn why(
    revision: &Revision,
    target: &RelationalContent,
    limits: Limits,
) -> Result<Option<Proof>> {
    revision.model().validate_content(target, false)?;
    let closure = derive::saturate(revision, limits)?;
    graph(&closure, target, revision.identity().clone())
}

pub(super) fn why_all(
    revision: &Revision,
    target: &RelationalContent,
    limits: SupportLimits,
) -> Result<Option<WhyAll>> {
    revision.model().validate_content(target, false)?;
    let closure = derive::saturate(revision, limits.closure)?;
    if closure.proof(target).is_none() {
        return Ok(None);
    }
    let frontier = derive::support_frontier(revision, target, limits)?;
    let revision_id = revision.identity().clone();
    let alternatives = frontier
        .supports()
        .iter()
        .map(|support| {
            Ok(WhySupport {
                assertions: support.assertions().to_vec(),
                proof: Proof {
                    revision: revision_id.clone(),
                    why: support_graph(support.proof())?,
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(WhyAll {
        revision: revision_id,
        target: target.clone(),
        alternatives,
        complete: frontier.status().is_complete(),
        expansions: frontier.expansions(),
    }))
}

fn graph(
    closure: &Closure,
    root: &RelationalContent,
    revision: RevisionId,
) -> Result<Option<Proof>> {
    if closure.proof(root).is_none() {
        return Ok(None);
    }
    let mut clauses = Vec::new();
    let mut indices = BTreeMap::new();
    let mut witnesses = Vec::new();
    let root = add_clause(root, closure, &mut clauses, &mut indices, &mut witnesses)?;
    witnesses.sort_by_key(|edge| edge.conclusion);
    Ok(Some(Proof {
        revision,
        why: WhyGraph {
            root,
            nodes: clauses
                .into_iter()
                .map(|clause| ClauseNode { clause })
                .collect(),
            witnesses,
        },
    }))
}

fn support_graph(root: &SupportProof) -> Result<WhyGraph> {
    let mut clauses = Vec::new();
    let mut indices = BTreeMap::new();
    let mut witnesses = Vec::new();
    let root = add_support_clause(root, &mut clauses, &mut indices, &mut witnesses)?;
    witnesses.sort_by_key(|edge| edge.conclusion);
    Ok(WhyGraph {
        root,
        nodes: clauses
            .into_iter()
            .map(|clause| ClauseNode { clause })
            .collect(),
        witnesses,
    })
}

fn add_support_clause(
    proof: &SupportProof,
    clauses: &mut Vec<RelationalContent>,
    indices: &mut BTreeMap<RelationalContent, usize>,
    witnesses: &mut Vec<WitnessEdge>,
) -> Result<usize> {
    let clause = proof.conclusion();
    if let Some(index) = indices.get(clause) {
        return Ok(*index);
    }
    let conclusion = clauses.len();
    clauses.push(clause.clone());
    indices.insert(clause.clone(), conclusion);
    let witness = match proof.witness() {
        SupportWitness::Asserted => Witness::Asserted,
        SupportWitness::Derived {
            rule,
            governing_law,
            authority,
            scope,
            premises,
            substitution,
        } => Witness::Derived {
            rule: rule.clone(),
            governing_law: governing_law.clone(),
            authority: authority.clone(),
            scope: scope.clone(),
            premises: premises
                .iter()
                .map(|premise| add_support_clause(premise, clauses, indices, witnesses))
                .collect::<Result<Vec<_>>>()?,
            substitution: substitution.clone(),
        },
    };
    witnesses.push(WitnessEdge {
        conclusion,
        witness,
    });
    Ok(conclusion)
}

fn add_clause(
    clause: &RelationalContent,
    closure: &Closure,
    clauses: &mut Vec<RelationalContent>,
    indices: &mut BTreeMap<RelationalContent, usize>,
    witnesses: &mut Vec<WitnessEdge>,
) -> Result<usize> {
    if let Some(index) = indices.get(clause) {
        return Ok(*index);
    }
    let conclusion = clauses.len();
    clauses.push(clause.clone());
    indices.insert(clause.clone(), conclusion);
    let proof = closure
        .proof(clause)
        .ok_or_else(|| KernelError::new("closure clause has no chosen witness"))?;
    let witness = match proof.witness() {
        derive::Witness::Asserted => Witness::Asserted,
        derive::Witness::Derived {
            rule,
            governing_law,
            authority,
            scope,
            premises,
            substitution,
        } => Witness::Derived {
            rule: rule.clone(),
            governing_law: governing_law.clone(),
            authority: authority.clone(),
            scope: scope.clone(),
            premises: premises
                .iter()
                .map(|premise| add_clause(premise, closure, clauses, indices, witnesses))
                .collect::<Result<Vec<_>>>()?,
            substitution: substitution.clone(),
        },
    };
    witnesses.push(WitnessEdge {
        conclusion,
        witness,
    });
    Ok(conclusion)
}
