//! Typed, finite candidate bases for intervention search.

use crate::kernel::{
    KernelError, ReferentId, RelationalContent, Result, Revision, Role, RoleId, Term,
};
use std::collections::BTreeSet;

pub(super) fn withdrawal_basis(
    source: &Revision,
    using: Vec<ReferentId>,
) -> Result<Vec<RelationalContent>> {
    let using = extensional_relations(source, using)?;
    Ok(source
        .model()
        .admitted_contents()
        .iter()
        .filter(|assertion| using.binary_search(assertion.relation()).is_ok())
        .cloned()
        .collect())
}

/// Cartesian ground clauses use only entities already admitted by the exact
/// Model and only the exact declared type of each role.
pub(super) struct AchievementBasis {
    pub(super) clauses: Vec<RelationalContent>,
    pub(super) complete: bool,
}

pub(super) fn achievement_basis(
    source: &Revision,
    using: Vec<ReferentId>,
    max_candidates: usize,
) -> Result<AchievementBasis> {
    let using = extensional_relations(source, using)?;
    let mut candidates = Vec::new();
    for relation_id in using {
        let relation = source
            .model()
            .relation_shapes()
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

fn extensional_relations(source: &Revision, using: Vec<ReferentId>) -> Result<Vec<ReferentId>> {
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
        .derivation_rules()
        .iter()
        .flat_map(|rule| rule.conclusion().forms())
        .map(|content| {
            source
                .model()
                .content(content)
                .expect("checked rule conclusion")
                .relation()
        })
        .collect::<BTreeSet<_>>();
    for relation in &using {
        if !source.model().relation_shapes().contains_key(relation) {
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
    relation: &ReferentId,
    roles: &[(&RoleId, &Role)],
    index: usize,
    values: &mut BTreeSet<(RoleId, Term)>,
    candidates: &mut Vec<RelationalContent>,
    max_candidates: usize,
) -> Result<bool> {
    if index == roles.len() {
        let candidate = RelationalContent::new(relation.clone(), values.iter().cloned().collect())?;
        if source
            .model()
            .admitted_contents()
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
    let (role_id, _) = roles[index];
    for referent in source.model().referents().keys() {
        values.insert((role_id.clone(), Term::referent(referent.clone())));
        let partial = RelationalContent::new(relation.clone(), values.iter().cloned().collect());
        if index + 1 == roles.len()
            && partial
                .as_ref()
                .is_ok_and(|content| source.model().validate_content(content, false).is_err())
        {
            values.remove(&(role_id.clone(), Term::referent(referent.clone())));
            continue;
        }
        let exhausted = collect_ground_clauses(
            source,
            relation,
            roles,
            index + 1,
            values,
            candidates,
            max_candidates,
        )?;
        values.remove(&(role_id.clone(), Term::referent(referent.clone())));
        if exhausted {
            return Ok(true);
        }
    }
    Ok(false)
}
