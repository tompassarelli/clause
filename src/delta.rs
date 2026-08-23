//! Immutable asserted-fact transitions and comparisons between revisions.
//!
//! These values deliberately live outside revision identity and persistence.
//! They describe a transition or comparison of already-admitted revisions;
//! admitting their successor recomputes the revision identity from its model.

use crate::kernel::{Clause, KernelError, Model, Result, Revision};

/// A set of asserted-fact additions and withdrawals for one exact base revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionDelta {
    base_revision: String,
    additions: Vec<Clause>,
    withdrawals: Vec<Clause>,
}

impl RevisionDelta {
    /// Build a canonical transition for one exact base revision identity.
    pub fn new(
        base_revision: impl Into<String>,
        mut additions: Vec<Clause>,
        mut withdrawals: Vec<Clause>,
    ) -> Result<Self> {
        let base_revision = base_revision.into();
        if base_revision.is_empty() {
            return Err(KernelError::new("delta requires a base revision identity"));
        }
        canonical_set(&mut additions, "duplicate delta addition")?;
        canonical_set(&mut withdrawals, "duplicate delta withdrawal")?;
        if additions
            .iter()
            .any(|fact| withdrawals.binary_search(fact).is_ok())
        {
            return Err(KernelError::new(
                "delta cannot add and withdraw the same fact",
            ));
        }
        Ok(Self {
            base_revision,
            additions,
            withdrawals,
        })
    }

    pub fn base_revision(&self) -> &str {
        &self.base_revision
    }

    pub fn additions(&self) -> &[Clause] {
        &self.additions
    }

    pub fn withdrawals(&self) -> &[Clause] {
        &self.withdrawals
    }

    /// Apply this transition atomically to its exact base revision.
    pub fn apply(&self, base: &Revision) -> Result<Revision> {
        if self.base_revision != base.identity() {
            return Err(KernelError::new("delta base revision does not match"));
        }
        let model = base.model();
        for fact in &self.withdrawals {
            if model.facts().binary_search(fact).is_err() {
                return Err(KernelError::new("delta withdraws a nonexistent fact"));
            }
        }
        for fact in &self.additions {
            if model.facts().binary_search(fact).is_ok() {
                return Err(KernelError::new("delta adds an existing fact"));
            }
        }

        let mut facts = model
            .facts()
            .iter()
            .filter(|fact| self.withdrawals.binary_search(fact).is_err())
            .cloned()
            .collect::<Vec<_>>();
        facts.extend(self.additions.iter().cloned());
        Ok(Revision::admit(Model::with_laws_and_intents(
            model.relations().values().cloned().collect(),
            facts,
            model.laws().to_vec(),
            model.query().clone(),
            model.intents().to_vec(),
            model.order(),
        )?))
    }
}

/// The asserted-fact difference between two revisions with identical declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionDiff {
    base_revision: String,
    successor_revision: String,
    added: Vec<Clause>,
    removed: Vec<Clause>,
}

impl RevisionDiff {
    /// Compare asserted facts only when all declarations are identical.
    pub fn between(base: &Revision, successor: &Revision) -> Result<Self> {
        let base_model = base.model();
        let successor_model = successor.model();
        if base_model.relations() != successor_model.relations()
            || base_model.laws() != successor_model.laws()
            || base_model.query() != successor_model.query()
            || base_model.intents() != successor_model.intents()
            || base_model.order() != successor_model.order()
        {
            return Err(KernelError::new(
                "cannot diff revisions with different declarations",
            ));
        }
        let added = successor_model
            .facts()
            .iter()
            .filter(|fact| base_model.facts().binary_search(fact).is_err())
            .cloned()
            .collect();
        let removed = base_model
            .facts()
            .iter()
            .filter(|fact| successor_model.facts().binary_search(fact).is_err())
            .cloned()
            .collect();
        Ok(Self {
            base_revision: base.identity().to_owned(),
            successor_revision: successor.identity().to_owned(),
            added,
            removed,
        })
    }

    pub fn base_revision(&self) -> &str {
        &self.base_revision
    }

    pub fn successor_revision(&self) -> &str {
        &self.successor_revision
    }

    pub fn added(&self) -> &[Clause] {
        &self.added
    }

    pub fn removed(&self) -> &[Clause] {
        &self.removed
    }
}

fn canonical_set(facts: &mut Vec<Clause>, duplicate: &str) -> Result<()> {
    facts.sort();
    if facts.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(KernelError::new(duplicate));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{RevisionDelta, RevisionDiff};
    use crate::kernel::{
        Cardinality, Clause, Intent, Law, Mode, Model, Relation, Revision, Role, Sentence, Term,
    };

    fn relation() -> Relation {
        Relation::new(
            "catalog/contains",
            vec![
                Role::new("set", "Text").unwrap(),
                Role::new("member", "Text").unwrap(),
            ],
            Sentence::new("set", "contains", "member").unwrap(),
            vec![
                Mode::finite(vec!["set".into()], vec!["member".into()], Cardinality::Many).unwrap(),
            ],
        )
        .unwrap()
    }

    fn fact(member: &str) -> Clause {
        Clause::new(
            "catalog/contains",
            vec![
                ("set".into(), Term::literal("letters").unwrap()),
                ("member".into(), Term::literal(member).unwrap()),
            ],
        )
        .unwrap()
    }

    fn revision(facts: Vec<Clause>) -> Revision {
        let variable_fact = |left: &str, right: &str| {
            Clause::new(
                "catalog/contains",
                vec![
                    ("set".into(), Term::variable(left).unwrap()),
                    ("member".into(), Term::variable(right).unwrap()),
                ],
            )
            .unwrap()
        };
        Revision::admit(
            Model::with_laws_and_intents(
                vec![relation()],
                facts,
                vec![
                    Law::new(
                        "catalog/reverse",
                        vec![variable_fact("set", "member")],
                        variable_fact("member", "set"),
                    )
                    .unwrap(),
                ],
                Clause::new(
                    "catalog/contains",
                    vec![
                        ("set".into(), Term::literal("letters").unwrap()),
                        ("member".into(), Term::variable("member").unwrap()),
                    ],
                )
                .unwrap(),
                vec![Intent::new("catalog/restock", fact("z")).unwrap()],
                "ascending",
            )
            .unwrap(),
        )
    }

    fn members(revision: &Revision) -> Vec<&str> {
        revision
            .model()
            .facts()
            .iter()
            .map(|fact| fact.roles().get("member").unwrap().text())
            .collect()
    }

    #[test]
    fn adds_facts_and_preserves_declarations() {
        let base = revision(vec![fact("a")]);
        let delta = RevisionDelta::new(base.identity(), vec![fact("b")], Vec::new()).unwrap();

        let successor = delta.apply(&base).unwrap();

        assert_eq!(members(&successor), ["a", "b"]);
        assert_eq!(successor.model().relations(), base.model().relations());
        assert_eq!(successor.model().laws(), base.model().laws());
        assert_eq!(successor.model().query(), base.model().query());
        assert_eq!(successor.model().intents(), base.model().intents());
        assert_eq!(successor.model().order(), base.model().order());
    }

    #[test]
    fn withdraws_facts_without_mutating_the_base() {
        let base = revision(vec![fact("a"), fact("b")]);
        let original = base.clone();
        let delta = RevisionDelta::new(base.identity(), Vec::new(), vec![fact("a")]).unwrap();

        let successor = delta.apply(&base).unwrap();

        assert_eq!(members(&successor), ["b"]);
        assert_eq!(base, original);
        assert_eq!(members(&base), ["a", "b"]);
    }

    #[test]
    fn rejects_a_wrong_base_or_an_invalid_withdrawal() {
        let base = revision(vec![fact("a")]);
        let wrong_base =
            RevisionDelta::new("rev-sha256-wrong", vec![fact("b")], Vec::new()).unwrap();
        assert_eq!(
            wrong_base.apply(&base).unwrap_err().to_string(),
            "delta base revision does not match"
        );

        let invalid_withdrawal =
            RevisionDelta::new(base.identity(), Vec::new(), vec![fact("missing")]).unwrap();
        assert_eq!(
            invalid_withdrawal.apply(&base).unwrap_err().to_string(),
            "delta withdraws a nonexistent fact"
        );
    }

    #[test]
    fn rejects_duplicate_or_conflicting_changes() {
        let base = revision(vec![fact("a")]);
        assert!(
            RevisionDelta::new(base.identity(), vec![fact("b"), fact("b")], Vec::new()).is_err()
        );
        assert!(RevisionDelta::new(base.identity(), vec![fact("b")], vec![fact("b")]).is_err());
        assert!(
            RevisionDelta::new(base.identity(), vec![fact("a")], Vec::new())
                .unwrap()
                .apply(&base)
                .is_err()
        );
    }

    #[test]
    fn transitions_and_diffs_are_deterministically_sorted() {
        let base = revision(vec![fact("b")]);
        let delta =
            RevisionDelta::new(base.identity(), vec![fact("z"), fact("a")], vec![fact("b")])
                .unwrap();
        let successor = delta.apply(&base).unwrap();
        let diff = RevisionDiff::between(&base, &successor).unwrap();

        assert_eq!(members(&successor), ["a", "z"]);
        assert_eq!(
            diff.added()
                .iter()
                .map(|fact| fact.roles().get("member").unwrap().text())
                .collect::<Vec<_>>(),
            ["a", "z"]
        );
        assert_eq!(
            diff.removed()
                .iter()
                .map(|fact| fact.roles().get("member").unwrap().text())
                .collect::<Vec<_>>(),
            ["b"]
        );
    }

    #[test]
    fn diff_rejects_declaration_mismatches() {
        let base = revision(vec![fact("a")]);
        let mismatched = Revision::admit(
            Model::with_laws_and_intents(
                vec![relation()],
                vec![fact("b")],
                base.model().laws().to_vec(),
                Clause::new(
                    "catalog/contains",
                    vec![
                        ("set".into(), Term::variable("set").unwrap()),
                        ("member".into(), Term::literal("letters").unwrap()),
                    ],
                )
                .unwrap(),
                base.model().intents().to_vec(),
                "ascending",
            )
            .unwrap(),
        );

        assert_eq!(
            RevisionDiff::between(&base, &mismatched)
                .unwrap_err()
                .to_string(),
            "cannot diff revisions with different declarations"
        );
    }
}
