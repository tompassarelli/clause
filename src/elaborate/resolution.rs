use std::collections::{BTreeMap, BTreeSet};

use crate::{
    frontend::{self, Declaration, Kind, Member},
    kernel::{
        self, AssertionOccurrence, Delta, Judgment, JudgmentKind, JudgmentStatus, JudgmentTarget,
        Model, Referent, RelationalContent, Revision, SemanticAtom,
    },
    wire,
};

use super::{
    identifiers::synthetic_referent,
    lowering::{Projection, lower_clause_with},
};

pub(super) struct Resolver<'a> {
    declarations: &'a BTreeMap<frontend::Name, &'a Declaration>,
    models: BTreeMap<frontend::Name, Model>,
    projection: &'a Projection,
    pub(super) revisions: BTreeMap<frontend::Name, Revision>,
    pub(super) source_spans: BTreeMap<kernel::ReferentId, frontend::Span>,
    deltas: BTreeMap<frontend::Name, Delta>,
    visiting_revisions: BTreeSet<frontend::Name>,
    visiting_deltas: BTreeSet<frontend::Name>,
}

impl<'a> Resolver<'a> {
    pub(super) fn new(
        declarations: &'a BTreeMap<frontend::Name, &'a Declaration>,
        models: BTreeMap<frontend::Name, Model>,
        projection: &'a Projection,
        source_spans: BTreeMap<kernel::ReferentId, frontend::Span>,
    ) -> Self {
        Self {
            declarations,
            models,
            projection,
            revisions: BTreeMap::new(),
            source_spans,
            deltas: BTreeMap::new(),
            visiting_revisions: BTreeSet::new(),
            visiting_deltas: BTreeSet::new(),
        }
    }

    fn declaration(&self, name: &frontend::Name, kind: Kind) -> kernel::Result<&'a Declaration> {
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
                None => {
                    let (delta, source_spans) = local_delta(self.projection, &base, declaration)?;
                    let revision = delta.apply(&base)?;
                    extend_source_spans(&mut self.source_spans, source_spans)?;
                    Ok(revision)
                }
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
            let (delta, source_spans) = local_delta(self.projection, &base, declaration)?;
            delta.apply(&base)?;
            Ok((delta, source_spans))
        })();
        self.visiting_deltas.remove(name);
        let (delta, source_spans) = outcome?;
        extend_source_spans(&mut self.source_spans, source_spans)?;
        self.deltas.insert(name.clone(), delta.clone());
        Ok(delta)
    }
}

fn from(declaration: &Declaration) -> kernel::Result<&frontend::Name> {
    declaration
        .body
        .iter()
        .find_map(|member| match member {
            Member::From(name) => Some(name),
            _ => None,
        })
        .ok_or_else(|| kernel::KernelError::new("Revision or Delta requires from:"))
}

fn apply(declaration: &Declaration) -> Option<&frontend::Name> {
    declaration.body.iter().find_map(|member| match member {
        Member::Apply(name) => Some(name),
        _ => None,
    })
}

fn local_delta(
    projection: &Projection,
    base: &Revision,
    declaration: &Declaration,
) -> kernel::Result<(Delta, BTreeMap<kernel::ReferentId, frontend::Span>)> {
    let declaration_id = projection
        .designations
        .global(declaration.subject.value.as_str())?;
    let admissions = declaration
        .body
        .iter()
        .filter_map(|member| match member {
            Member::Admit(clauses) => Some(clauses),
            _ => None,
        })
        .flatten()
        .enumerate()
        .map(|(index, surface)| {
            Ok((
                lower_clause_with(projection, base.model(), surface, None)?,
                surface.span,
                synthetic_referent(
                    "delta-assertion-occurrence",
                    &[declaration_id.as_str(), &index.to_string()],
                ),
            ))
        })
        .collect::<kernel::Result<Vec<_>>>()?;
    let withdrawals = declaration
        .body
        .iter()
        .filter_map(|member| match member {
            Member::Withdraw(clauses) => Some(clauses),
            _ => None,
        })
        .flatten()
        .map(|surface| lower_clause_with(projection, base.model(), surface, None))
        .collect::<kernel::Result<Vec<_>>>()?;
    let delta = semantic_delta(
        base,
        declaration_id,
        admissions
            .iter()
            .map(|(content, _, occurrence)| (content.clone(), occurrence.clone()))
            .collect(),
        withdrawals,
    )?;
    let mut source_spans = BTreeMap::new();
    for (_, span, occurrence) in admissions {
        if source_spans.insert(occurrence, span).is_some() {
            return Err(kernel::KernelError::new(
                "duplicate Delta assertion source projection",
            ));
        }
    }
    Ok((delta, source_spans))
}

fn semantic_delta(
    base: &Revision,
    source: kernel::ReferentId,
    admissions: Vec<(RelationalContent, kernel::ReferentId)>,
    withdrawals: Vec<RelationalContent>,
) -> kernel::Result<Delta> {
    let mut added = BTreeSet::new();
    let mut removed = Vec::new();
    let atoms = base.model().atoms();
    for (content, occurrence_id) in admissions {
        let source_atom = SemanticAtom::Referent(Referent::new(source.clone()));
        if !atoms.contains(&source_atom) {
            added.insert(source_atom);
        }
        let content_atom = SemanticAtom::RelationalContent(content.clone());
        if !atoms.contains(&content_atom) {
            added.insert(content_atom);
        }
        let judgment_id = synthetic_referent("delta-admission-judgment", &[occurrence_id.as_str()]);
        for id in [&occurrence_id, &judgment_id] {
            let atom = SemanticAtom::Referent(Referent::new(id.clone()));
            if !atoms.contains(&atom) {
                added.insert(atom);
            }
        }
        added.insert(SemanticAtom::AssertionOccurrence(AssertionOccurrence::new(
            occurrence_id.clone(),
            content.id().clone(),
            source.clone(),
            base.model().id().clone(),
        )));
        added.insert(SemanticAtom::Judgment(Judgment::new(
            judgment_id,
            base.model().id().clone(),
            base.model().id().clone(),
            JudgmentTarget::Occurrence(occurrence_id),
            JudgmentKind::Admitted {
                policy: base.model().id().clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        )));
    }
    for content in withdrawals {
        let occurrences = base
            .model()
            .occurrences()
            .iter()
            .filter(|occurrence| occurrence.content() == content.id())
            .collect::<Vec<_>>();
        if occurrences.is_empty() {
            return Err(kernel::KernelError::new(
                "Delta withdraws a nonexistent assertion",
            ));
        }
        let ids = occurrences
            .iter()
            .map(|item| item.id())
            .collect::<BTreeSet<_>>();
        removed.extend(
            occurrences
                .into_iter()
                .cloned()
                .map(SemanticAtom::AssertionOccurrence),
        );
        removed.extend(
            base.model()
                .judgments()
                .iter()
                .filter(|judgment| match judgment.target() {
                    JudgmentTarget::Occurrence(id) => ids.contains(id),
                    JudgmentTarget::Content(id) => id == content.id(),
                })
                .cloned()
                .map(SemanticAtom::Judgment),
        );
    }
    Delta::new(
        base.identity().clone(),
        added.into_iter().collect(),
        removed,
    )
}

fn extend_source_spans(
    target: &mut BTreeMap<kernel::ReferentId, frontend::Span>,
    additions: BTreeMap<kernel::ReferentId, frontend::Span>,
) -> kernel::Result<()> {
    for (occurrence, span) in additions {
        match target.get(&occurrence) {
            Some(existing) if existing != &span => {
                return Err(kernel::KernelError::new(
                    "one assertion occurrence has conflicting source projections",
                ));
            }
            Some(_) => {}
            None => {
                target.insert(occurrence, span);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{elaborate::compile, frontend};

    const BASE: &str = "Module\n\nimpact/imports: RelationShape\n  {consumer: Module} imports {dependency: Module}\n  mode consumer -> dependency: many\n\nimpact\n  North ∈ Module\n  South ∈ Module\n  Store ∈ Module\n  North imports Store\n";

    #[test]
    fn direct_and_reusable_deltas_preserve_content_without_collapsing_occurrences() {
        let program = compile(frontend::parse(&format!(
            "{BASE}\nimpact/direct: Revision\n  from: impact\n  admit:\n    South imports North\n\nimpact/add: Delta\n  from: impact\n  admit:\n    South imports North\n\nimpact/reusable: Revision\n  from: impact\n  apply: impact/add\n"
        )).unwrap()).unwrap();
        let direct = program
            .revision(&frontend::Name("impact/direct".into()))
            .unwrap();
        let reusable = program
            .revision(&frontend::Name("impact/reusable".into()))
            .unwrap();
        let direct_source = program.designations().global("impact/direct").unwrap();
        let reusable_source = program.designations().global("impact/add").unwrap();
        let direct_occurrence = direct
            .model()
            .occurrences()
            .iter()
            .find(|occurrence| occurrence.source() == &direct_source)
            .unwrap();
        let reusable_occurrence = reusable
            .model()
            .occurrences()
            .iter()
            .find(|occurrence| occurrence.source() == &reusable_source)
            .unwrap();
        assert_eq!(
            direct.model().admitted_contents(),
            reusable.model().admitted_contents()
        );
        assert_eq!(direct_occurrence.content(), reusable_occurrence.content());
        assert_ne!(direct_occurrence.source(), reusable_occurrence.source());
        assert_ne!(direct_occurrence.id(), reusable_occurrence.id());
        assert_ne!(
            crate::wire::serialize(direct),
            crate::wire::serialize(reusable)
        );
    }

    #[test]
    fn repeated_content_admissions_preserve_each_authored_occurrence() {
        let program = compile(frontend::parse(&format!(
            "{BASE}\nimpact/repeated: Revision\n  from: impact\n  admit:\n    North imports Store\n    North imports Store\n"
        )).unwrap()).unwrap();
        let base = program.revision(&frontend::Name("impact".into())).unwrap();
        let repeated = program
            .revision(&frontend::Name("impact/repeated".into()))
            .unwrap();
        assert_eq!(
            repeated.model().admitted_contents(),
            base.model().admitted_contents()
        );
        assert_eq!(
            repeated.model().occurrences().len(),
            base.model().occurrences().len() + 2
        );
        let source = program.designations().global("impact/repeated").unwrap();
        let authored = repeated
            .model()
            .occurrences()
            .iter()
            .filter(|occurrence| occurrence.source() == &source)
            .collect::<Vec<_>>();
        assert_eq!(authored.len(), 2);
        assert_eq!(authored[0].content(), authored[1].content());
        assert_ne!(authored[0].id(), authored[1].id());
        assert_eq!(authored[0].source(), authored[1].source());
    }
}
