//! Typed, finite candidate bases for intervention search.

use crate::kernel::{Clause, KernelError, RelationId, Result, Revision, Role, RoleId, Term};
use std::collections::BTreeSet;

pub(super) fn withdrawal_basis(source: &Revision, using: Vec<RelationId>) -> Result<Vec<Clause>> {
    let using = extensional_relations(source, using)?;
    Ok(source
        .model()
        .assertions()
        .iter()
        .filter(|assertion| using.binary_search(assertion.relation()).is_ok())
        .cloned()
        .collect())
}

/// Cartesian ground clauses use only entities already admitted by the exact
/// Model and only the exact declared type of each role.
pub(super) struct AchievementBasis {
    pub(super) clauses: Vec<Clause>,
    pub(super) complete: bool,
}

pub(super) fn achievement_basis(
    source: &Revision,
    using: Vec<RelationId>,
    max_candidates: usize,
) -> Result<AchievementBasis> {
    let using = extensional_relations(source, using)?;
    let mut candidates = Vec::new();
    for relation_id in using {
        let relation = source
            .model()
            .relations()
            .get(&relation_id)
            .expect("validated relation");
        let roles = relation.roles().iter().collect::<Vec<_>>();
        if collect_ground_clauses(
            source,
            &relation_id,
            &roles,
            0,
            &mut BTreeSet::new(),
            &mut candidates,
            max_candidates,
        )? {
            return Ok(AchievementBasis {
                clauses: candidates,
                complete: false,
            });
        }
    }
    Ok(AchievementBasis {
        clauses: candidates,
        complete: true,
    })
}

fn extensional_relations(source: &Revision, using: Vec<RelationId>) -> Result<Vec<RelationId>> {
    let using = using
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if using.is_empty() {
        return Err(KernelError::new(
            "intervention requires at least one relation",
        ));
    }
    let derived = source
        .model()
        .laws()
        .iter()
        .map(|law| law.conclusion().relation())
        .collect::<BTreeSet<_>>();
    for relation in &using {
        if !source.model().relations().contains_key(relation) {
            return Err(KernelError::new("intervention relation is undeclared"));
        }
        if derived.contains(relation) {
            return Err(KernelError::new("intervention relation is not extensional"));
        }
    }
    Ok(using)
}

fn collect_ground_clauses(
    source: &Revision,
    relation: &RelationId,
    roles: &[(&RoleId, &Role)],
    index: usize,
    values: &mut BTreeSet<(RoleId, Term)>,
    candidates: &mut Vec<Clause>,
    max_candidates: usize,
) -> Result<bool> {
    if index == roles.len() {
        let candidate = Clause::new(relation.clone(), values.iter().cloned().collect())?;
        if source
            .model()
            .assertions()
            .binary_search(&candidate)
            .is_ok()
        {
            return Ok(false);
        }
        if candidates.len() >= max_candidates {
            return Ok(true);
        }
        candidates.push(candidate);
        return Ok(false);
    }
    let (role_id, role) = roles[index];
    for entity in source
        .model()
        .entities()
        .iter()
        .filter(|entity| entity.typ() == role.typ())
    {
        values.insert((role_id.clone(), Term::entity(entity.clone())));
        let exhausted = collect_ground_clauses(
            source,
            relation,
            roles,
            index + 1,
            values,
            candidates,
            max_candidates,
        )?;
        values.remove(&(role_id.clone(), Term::entity(entity.clone())));
        if exhausted {
            return Ok(true);
        }
    }
    Ok(false)
}
