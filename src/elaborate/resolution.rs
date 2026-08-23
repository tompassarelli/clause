use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, AscriptionDecl, Kind, Member},
    kernel::{self, Delta, Model, Revision},
    wire,
};

use super::lowering::lower_clause;

pub(super) struct Resolver<'a> {
    declarations: &'a BTreeMap<frontend::Name, &'a AscriptionDecl>,
    models: BTreeMap<frontend::Name, Model>,
    pub(super) revisions: BTreeMap<frontend::Name, Revision>,
    deltas: BTreeMap<frontend::Name, Delta>,
    visiting_revisions: BTreeSet<frontend::Name>,
    visiting_deltas: BTreeSet<frontend::Name>,
}

impl<'a> Resolver<'a> {
    pub(super) fn new(
        declarations: &'a BTreeMap<frontend::Name, &'a AscriptionDecl>,
        models: BTreeMap<frontend::Name, Model>,
    ) -> Self {
        Self {
            declarations,
            models,
            revisions: BTreeMap::new(),
            deltas: BTreeMap::new(),
            visiting_revisions: BTreeSet::new(),
            visiting_deltas: BTreeSet::new(),
        }
    }

    fn declaration(&self, name: &frontend::Name, kind: Kind) -> kernel::Result<&'a AscriptionDecl> {
        match self.declarations.get(name) {
            Some(declaration) if declaration.kind == kind => Ok(*declaration),
            Some(_) => Err(kernel::KernelError::new(format!(
                "'{}' has the wrong declaration kind",
                name.as_str()
            ))),
            None => Err(kernel::KernelError::new(format!(
                "unknown declaration '{}'",
                name.as_str()
            ))),
        }
    }

    pub(super) fn revision(&mut self, name: &frontend::Name) -> kernel::Result<Revision> {
        if let Some(revision) = self.revisions.get(name) {
            return Ok(revision.clone());
        }
        if let Some(model) = self.models.get(name) {
            let revision = wire::admit(model.clone());
            self.revisions.insert(name.clone(), revision.clone());
            return Ok(revision);
        }
        let declaration = self.declaration(name, Kind::Revision)?;
        if !self.visiting_revisions.insert(name.clone()) {
            return Err(kernel::KernelError::new(format!(
                "Revision/Delta dependency cycle at '{}'",
                name.as_str()
            )));
        }
        let outcome = (|| {
            let base = self.revision(from(declaration)?)?;
            match apply(declaration) {
                Some(delta_name) => {
                    let delta = self.delta(delta_name)?;
                    if delta.base() != base.identity() {
                        return Err(kernel::KernelError::new(format!(
                            "Delta '{}' base does not match Revision '{}'",
                            delta_name.as_str(),
                            name.as_str()
                        )));
                    }
                    delta.apply(&base)
                }
                None => local_delta(&base, declaration)?.apply(&base),
            }
        })();
        self.visiting_revisions.remove(name);
        let revision = outcome?;
        self.revisions.insert(name.clone(), revision.clone());
        Ok(revision)
    }

    pub(super) fn delta(&mut self, name: &frontend::Name) -> kernel::Result<Delta> {
        if let Some(delta) = self.deltas.get(name) {
            return Ok(delta.clone());
        }
        let declaration = self.declaration(name, Kind::Delta)?;
        if !self.visiting_deltas.insert(name.clone()) {
            return Err(kernel::KernelError::new(format!(
                "Revision/Delta dependency cycle at '{}'",
                name.as_str()
            )));
        }
        let outcome = (|| {
            let base = self.revision(from(declaration)?)?;
            let delta = local_delta(&base, declaration)?;
            delta.apply(&base)?;
            Ok(delta)
        })();
        self.visiting_deltas.remove(name);
        let delta = outcome?;
        self.deltas.insert(name.clone(), delta.clone());
        Ok(delta)
    }
}

fn from(declaration: &AscriptionDecl) -> kernel::Result<&frontend::Name> {
    declaration
        .body
        .iter()
        .find_map(|member| match member {
            Member::From(name) => Some(name),
            _ => None,
        })
        .ok_or_else(|| kernel::KernelError::new("Revision or Delta requires from:"))
}

fn apply(declaration: &AscriptionDecl) -> Option<&frontend::Name> {
    declaration.body.iter().find_map(|member| match member {
        Member::Apply(name) => Some(name),
        _ => None,
    })
}

fn local_delta(base: &Revision, declaration: &AscriptionDecl) -> kernel::Result<Delta> {
    let admissions = declaration
        .body
        .iter()
        .filter_map(|member| match member {
            Member::Admit(clauses) => Some(clauses),
            _ => None,
        })
        .flatten()
        .map(|clause| lower_clause(base, clause))
        .collect::<kernel::Result<Vec<_>>>()?;
    let withdrawals = declaration
        .body
        .iter()
        .filter_map(|member| match member {
            Member::Withdraw(clauses) => Some(clauses),
            _ => None,
        })
        .flatten()
        .map(|clause| lower_clause(base, clause))
        .collect::<kernel::Result<Vec<_>>>()?;
    Delta::new(base.identity().clone(), admissions, withdrawals)
}

#[cfg(test)]
mod tests {
    use crate::{
        elaborate::compile,
        frontend::{self, Member},
    };

    const BASE: &str = "Module: Type\n\nimpact/imports: Relation\n    {consumer: Module} imports {dependency: Module}\n    mode consumer -> dependency: many\n\nimpact: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n";

    #[test]
    fn direct_and_reusable_deltas_seal_identically() {
        let program = compile(frontend::parse(&format!(
            "{BASE}\nimpact/direct: Revision\n    from: impact\n    admit:\n        South imports North\n\nimpact/add: Delta\n    from: impact\n    admit:\n        South imports North\n\nimpact/reusable: Revision\n    from: impact\n    apply: impact/add\n"
        )).unwrap()).unwrap();
        let direct = program
            .revision(&frontend::Name("impact/direct".into()))
            .unwrap();
        let reusable = program
            .revision(&frontend::Name("impact/reusable".into()))
            .unwrap();
        assert_eq!(
            crate::wire::serialize(direct),
            crate::wire::serialize(reusable)
        );
    }

    #[test]
    fn rejects_delta_base_mismatch_and_cross_model_entities() {
        let source = format!(
            "{BASE}\nother: Model\n    North: Module\n    South: Module\n    Store: Module\n    North imports Store\n\nimpact/change: Delta\n    from: impact\n    admit:\n        South imports North\n\nother/wrong: Revision\n    from: other\n    apply: impact/change\n"
        );
        assert!(
            compile(frontend::parse(&source).unwrap())
                .unwrap_err()
                .to_string()
                .contains("base does not match")
        );
        let source = format!(
            "{BASE}\nother: Model\n    North: Module\n\nimpact/bad: Revision\n    from: impact\n    admit:\n        other/North imports Store\n"
        );
        assert!(
            compile(frontend::parse(&source).unwrap())
                .unwrap_err()
                .to_string()
                .contains("not admitted by Model")
        );
    }

    #[test]
    fn rejects_an_invalid_delta_even_when_no_revision_applies_it() {
        let source = format!(
            "{BASE}\nimpact/orphan: Delta\n    from: impact\n    withdraw:\n        South imports North\n"
        );
        assert!(
            compile(frontend::parse(&source).unwrap())
                .unwrap_err()
                .to_string()
                .contains("withdraws a nonexistent assertion")
        );
    }

    #[test]
    fn rejects_lowered_revision_cycles() {
        let source = format!(
            "{BASE}\nimpact/one: Revision\n    from: impact\n    admit:\n        South imports North\n\nimpact/two: Revision\n    from: impact/one\n    admit:\n        Store imports South\n"
        );
        let mut program = frontend::parse(&source).unwrap();
        for declaration in &mut program.declarations {
            if declaration.subject.value.as_str() == "impact/one" {
                for member in &mut declaration.body {
                    if let Member::From(name) = member {
                        *name = frontend::Name("impact/two".into());
                    }
                }
            }
        }
        assert!(
            compile(program)
                .unwrap_err()
                .to_string()
                .contains("dependency cycle")
        );
    }
}
