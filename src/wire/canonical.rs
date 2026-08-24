use crate::kernel::{
    AssertionOccurrence, Definition, Delta, DerivationRule, Goal, Invariant, Judgment,
    JudgmentKind, JudgmentTarget, LookupMode, Model, Pattern, Referent, RelationShape,
    RelationalContent, Revision, RevisionId, RevisionLineage, RoleId, RolePredicate, SemanticAtom,
    StructuralContract, StructuralForm, Term, Transition, UniversalLaw,
};

use super::{json::escape, sha256::sha256_digest};

pub const SEMANTIC_TAG: &str = "clause-semantic-v7";
pub const REVISION_TAG: &str = "clause-revision-v5";

pub fn semantic_payload(revision: &Revision) -> String {
    payload(revision.lineage(), revision.model())
}

fn payload(lineage: &RevisionLineage, model: &Model) -> String {
    let referents = join(model.referents().values().map(referent_json));
    let contents = join(model.relational_contents().values().map(content_json));
    let shapes = join(model.relation_shapes().values().map(shape_json));
    let structural_contracts = join(
        model
            .structural_contracts()
            .values()
            .map(structural_contract_json),
    );
    let occurrences = join(model.occurrences().iter().map(occurrence_json));
    let definitions = join(model.definitions().iter().map(definition_json));
    let rules = join(model.derivation_rules().iter().map(rule_json));
    let laws = join(model.universal_laws().iter().map(law_json));
    let invariants = join(model.invariants().iter().map(invariant_json));
    let goals = join(model.goals().iter().map(goal_json));
    let transitions = join(model.transitions().iter().map(transition_json));
    let judgments = join(model.judgments().iter().map(judgment_json));
    format!(
        "[\"{SEMANTIC_TAG}\",[\"lineage\",{}],[\"model\",\"{}\"],[\"referents\",[{referents}]],[\"relational-contents\",[{contents}]],[\"relation-shapes\",[{shapes}]],[\"structural-contracts\",[{structural_contracts}]],[\"occurrences\",[{occurrences}]],[\"definitions\",[{definitions}]],[\"derivation-rules\",[{rules}]],[\"universal-laws\",[{laws}]],[\"invariants\",[{invariants}]],[\"goals\",[{goals}]],[\"transitions\",[{transitions}]],[\"judgments\",[{judgments}]]]",
        lineage_json(lineage),
        escape(model.id().as_str()),
    )
}

pub fn revision_id(lineage: &RevisionLineage, model: &Model) -> RevisionId {
    RevisionId::from_digest(sha256_digest(payload(lineage, model).as_bytes()))
}

pub fn admit(model: Model) -> Revision {
    admit_with_lineage(model, RevisionLineage::Root)
}

pub fn admit_successor(
    base: &Revision,
    model: Model,
    delta: Delta,
) -> crate::kernel::Result<Revision> {
    if delta.base() != base.identity() {
        return Err(crate::kernel::KernelError::new(
            "successor Delta names the wrong predecessor",
        ));
    }
    let mut atoms = base.model().atoms();
    for withdrawal in delta.withdrawals() {
        if !atoms.remove(withdrawal) {
            return Err(crate::kernel::KernelError::new(
                "successor Delta withdraws an absent atom",
            ));
        }
    }
    for admission in delta.admissions() {
        if !atoms.insert(admission.clone()) {
            return Err(crate::kernel::KernelError::new(
                "successor Delta admits an existing atom",
            ));
        }
    }
    if atoms != model.atoms() || base.model().id() != model.id() {
        return Err(crate::kernel::KernelError::new(
            "successor Delta does not account for the complete semantic snapshot",
        ));
    }
    Ok(admit_with_lineage(model, RevisionLineage::Successor(delta)))
}

fn admit_with_lineage(model: Model, lineage: RevisionLineage) -> Revision {
    let identity = revision_id(&lineage, &model);
    Revision::reloaded(identity, lineage, model)
}

pub fn serialize(revision: &Revision) -> String {
    format!(
        "[\"{REVISION_TAG}\",\"{}\",{}]",
        revision.identity(),
        semantic_payload(revision)
    )
}

fn lineage_json(lineage: &RevisionLineage) -> String {
    match lineage {
        RevisionLineage::Root => "[\"root\"]".into(),
        RevisionLineage::Successor(delta) => {
            format!("[\"successor\",\"{}\",{}]", delta.base(), delta_json(delta))
        }
    }
}

fn delta_json(delta: &Delta) -> String {
    let admissions = join(delta.admissions().iter().map(atom_json));
    let withdrawals = join(delta.withdrawals().iter().map(atom_json));
    format!("[\"delta\",[\"admit\",[{admissions}]],[\"withdraw\",[{withdrawals}]]]")
}

fn atom_json(atom: &SemanticAtom) -> String {
    match atom {
        SemanticAtom::Referent(value) => referent_json(value),
        SemanticAtom::RelationalContent(value) => content_json(value),
        SemanticAtom::RelationShape(value) => shape_json(value),
        SemanticAtom::StructuralContract(value) => structural_contract_json(value),
        SemanticAtom::AssertionOccurrence(value) => occurrence_json(value),
        SemanticAtom::Definition(value) => definition_json(value),
        SemanticAtom::DerivationRule(value) => rule_json(value),
        SemanticAtom::UniversalLaw(value) => law_json(value),
        SemanticAtom::Invariant(value) => invariant_json(value),
        SemanticAtom::Goal(value) => goal_json(value),
        SemanticAtom::Transition(value) => transition_json(value),
        SemanticAtom::Judgment(value) => judgment_json(value),
    }
}

fn referent_json(referent: &Referent) -> String {
    format!("[\"referent\",\"{}\"]", escape(referent.id().as_str()))
}

fn structural_contract_json(contract: &StructuralContract) -> String {
    let form = match contract.form() {
        StructuralForm::F32 => "[\"f32\"]".to_owned(),
        StructuralForm::Int => "[\"int\"]".to_owned(),
        StructuralForm::Bool => "[\"bool\"]".to_owned(),
        StructuralForm::Product(fields) => format!(
            "[\"product\",[{}]]",
            strings(fields.iter().map(|field| field.as_str()))
        ),
    };
    format!(
        "[\"structural-contract\",\"{}\",{form}]",
        escape(contract.referent().as_str())
    )
}

fn content_json(content: &RelationalContent) -> String {
    let roles = join(
        content
            .roles()
            .iter()
            .map(|(role, term)| format!("[\"{}\",{}]", escape(role.as_str()), term_json(term))),
    );
    format!(
        "[\"relational-content\",\"{}\",\"{}\",[\"roles\",[{roles}]]]",
        escape(content.id().as_str()),
        escape(content.relation().as_str())
    )
}

fn shape_json(shape: &RelationShape) -> String {
    let roles = join(shape.roles().values().map(|role| {
        let predicates = join(role.admissibility().iter().map(predicate_json));
        format!(
            "[\"role\",\"{}\",[\"admissibility\",[{predicates}]]]",
            escape(role.id().as_str())
        )
    }));
    let lookup = join(shape.lookup().iter().map(lookup_json));
    format!(
        "[\"relation-shape\",\"{}\",[\"roles\",[{roles}]],[\"lookup\",[{lookup}]]]",
        escape(shape.referent().as_str())
    )
}

fn predicate_json(predicate: &RolePredicate) -> String {
    let fixed = join(predicate.fixed_roles().iter().map(|(role, referent)| {
        format!(
            "[\"{}\",\"{}\"]",
            escape(role.as_str()),
            escape(referent.as_str())
        )
    }));
    format!(
        "[\"predicate\",\"{}\",\"{}\",[\"fixed\",[{fixed}]]]",
        escape(predicate.relation().as_str()),
        escape(predicate.candidate_role().as_str())
    )
}

fn lookup_json(mode: &LookupMode) -> String {
    let known = strings(mode.known().iter().map(RoleId::as_str));
    let sought = strings(mode.sought().iter().map(RoleId::as_str));
    format!(
        "[\"lookup\",[\"known\",[{known}]],[\"sought\",[{sought}]],[\"cardinality\",\"{}\"]]",
        mode.cardinality().as_str()
    )
}

fn occurrence_json(occurrence: &AssertionOccurrence) -> String {
    format!(
        "[\"assertion-occurrence\",\"{}\",\"{}\",[\"source\",\"{}\"],[\"scope\",\"{}\"]]",
        escape(occurrence.id().as_str()),
        escape(occurrence.content().as_str()),
        escape(occurrence.source().as_str()),
        escape(occurrence.scope().as_str()),
    )
}

fn definition_json(value: &Definition) -> String {
    format!(
        "[\"definition\",\"{}\",{}]",
        escape(value.id().as_str()),
        term_json(value.denotation())
    )
}

fn rule_json(value: &DerivationRule) -> String {
    format!(
        "[\"derivation-rule\",\"{}\",[\"scope\",\"{}\"],[\"authority\",\"{}\"],[\"premises\",{}],[\"conclusion\",{}]]",
        escape(value.id().as_str()),
        escape(value.scope().as_str()),
        escape(value.authority().as_str()),
        pattern_json(value.premises()),
        pattern_json(value.conclusion())
    )
}

fn law_json(value: &UniversalLaw) -> String {
    format!(
        "[\"universal-law\",\"{}\",[\"scope\",\"{}\"],[\"generalized\",{}]]",
        escape(value.id().as_str()),
        escape(value.scope().as_str()),
        pattern_json(value.generalized())
    )
}

fn invariant_json(value: &Invariant) -> String {
    format!(
        "[\"invariant\",\"{}\",[\"scope\",\"{}\"],[\"policy\",\"{}\"],[\"condition\",{}],[\"admission\",\"{}\"]]",
        escape(value.id().as_str()),
        escape(value.scope().as_str()),
        escape(value.policy().as_str()),
        pattern_json(value.condition()),
        value.admission().as_str()
    )
}

fn goal_json(value: &Goal) -> String {
    format!(
        "[\"goal\",\"{}\",[\"context\",\"{}\"],[\"desired\",{}]]",
        escape(value.id().as_str()),
        escape(value.context().as_str()),
        pattern_json(value.desired())
    )
}

fn pattern_json(value: &Pattern) -> String {
    format!(
        "[\"pattern\",[{}]]",
        strings(value.forms().iter().map(|id| id.as_str()))
    )
}

fn transition_json(value: &Transition) -> String {
    format!(
        "[\"transition\",\"{}\",[\"from\",\"{}\"],[\"to\",\"{}\"]]",
        escape(value.id().as_str()),
        escape(value.from().as_str()),
        escape(value.to().as_str())
    )
}

fn judgment_json(value: &Judgment) -> String {
    let target = match value.target() {
        JudgmentTarget::Content(id) => format!("[\"content\",\"{}\"]", escape(id.as_str())),
        JudgmentTarget::Occurrence(id) => format!("[\"occurrence\",\"{}\"]", escape(id.as_str())),
    };
    format!(
        "[\"judgment\",\"{}\",[\"authority\",\"{}\"],[\"scope\",\"{}\"],[\"target\",{target}],[\"kind\",{}],[\"status\",\"{}\"]]",
        escape(value.id().as_str()),
        escape(value.authority().as_str()),
        escape(value.scope().as_str()),
        judgment_kind_json(value.kind()),
        value.status().as_str()
    )
}

fn judgment_kind_json(kind: &JudgmentKind) -> String {
    match kind {
        JudgmentKind::Declared => "[\"declared\"]".into(),
        JudgmentKind::Derived { rule, premises } => format!(
            "[\"derived\",\"{}\",[{}]]",
            escape(rule.as_str()),
            strings(premises.iter().map(|id| id.as_str()))
        ),
        JudgmentKind::Observed { evidence } => {
            format!("[\"observed\",\"{}\"]", escape(evidence.as_str()))
        }
        JudgmentKind::Admitted { policy, basis } => format!(
            "[\"admitted\",\"{}\",[{}]]",
            escape(policy.as_str()),
            strings(basis.iter().map(|id| id.as_str()))
        ),
        JudgmentKind::Rejected { policy, basis } => format!(
            "[\"rejected\",\"{}\",[{}]]",
            escape(policy.as_str()),
            strings(basis.iter().map(|id| id.as_str()))
        ),
        JudgmentKind::Superseded { by } => format!("[\"superseded\",\"{}\"]", escape(by.as_str())),
    }
}

fn term_json(term: &Term) -> String {
    match term {
        Term::Referent(id) => format!("[\"referent\",\"{}\"]", escape(id.as_str())),
        Term::Pattern(id) => format!("[\"pattern\",\"{}\"]", escape(id.as_str())),
        Term::Application(id) => format!("[\"application\",\"{}\"]", escape(id.as_str())),
        Term::F32(value) => format!("[\"f32\",\"{:08x}\"]", value.bits()),
        Term::Int(value) => format!("[\"int\",\"{value}\"]"),
        Term::Bool(value) => format!("[\"bool\",\"{value}\"]"),
        Term::Product { shape, fields } => format!(
            "[\"product\",\"{}\",[{}]]",
            escape(shape.as_str()),
            join(fields.iter().map(|(label, field)| format!(
                "[\"{}\",\"{}\",{}]",
                escape(label.as_str()),
                escape(field.domain().as_str()),
                term_json(field.value())
            )))
        ),
        Term::LabelledProduct { shape, fields } => format!(
            "[\"labelled-product\",\"{}\",[{}]]",
            escape(shape.as_str()),
            join(fields.iter().map(|(field, value)| format!(
                "[\"{}\",{}]",
                escape(field.as_str()),
                term_json(value)
            )))
        ),
        Term::Sum { tag, value } => format!(
            "[\"sum\",\"{}\",{}]",
            escape(tag.as_str()),
            term_json(value)
        ),
        Term::Sequence {
            shape,
            element,
            values,
        } => {
            format!(
                "[\"sequence\",\"{}\",\"{}\",[{}]]",
                escape(shape.as_str()),
                escape(element.as_str()),
                join(values.iter().map(term_json))
            )
        }
    }
}

fn join(values: impl Iterator<Item = String>) -> String {
    values.collect::<Vec<_>>().join(",")
}
fn strings<'a>(values: impl Iterator<Item = &'a str>) -> String {
    join(values.map(|value| format!("\"{}\"", escape(value))))
}
