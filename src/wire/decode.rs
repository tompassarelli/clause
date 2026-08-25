use std::collections::{BTreeMap, BTreeSet};

use crate::kernel::{
    AssertionOccurrence, Cardinality, ContentId, Definition, Delta, DerivationRule, Goal,
    Invariant, InvariantAdmission, Judgment, JudgmentKind, JudgmentStatus, JudgmentTarget,
    KernelError, LookupMode, Model, Name, Pattern, PatternId, ProductField, Referent, ReferentId,
    RelationShape, RelationalContent, Result, Revision, RevisionId, RevisionLineage, Role, RoleId,
    RolePredicate, SemanticAtom, StructuralContract, StructuralForm, Term, Transition,
    UniversalLaw,
};

use super::{
    canonical::{REVISION_TAG, SEMANTIC_TAG, revision_id, semantic_payload, serialize},
    json::{Json, JsonParser, array, json, list, require_string, string},
};

/// Strictly reload one canonical root Revision-v6 / semantic-v10 artifact.
///
/// A successor carries an exact Delta claim whose completeness cannot be
/// checked without its predecessor snapshot. Use [`reload_successor`] for
/// those artifacts.
pub fn reload(bytes: &str) -> Result<Revision> {
    let revision = decode_canonical(bytes)?;
    if revision.predecessor().is_some() {
        return Err(KernelError::new(
            "successor Revision reload requires its exact predecessor",
        ));
    }
    Ok(revision)
}

/// Strictly reload a successor against the exact predecessor named by its
/// signed Delta.
pub fn reload_successor(bytes: &str, predecessor: &Revision) -> Result<Revision> {
    let revision = decode_canonical(bytes)?;
    let delta = revision
        .delta()
        .ok_or_else(|| KernelError::new("successor reload requires successor lineage"))?;
    let expected =
        super::canonical::admit_successor(predecessor, revision.model().clone(), delta.clone())?;
    if expected != revision {
        return Err(KernelError::new(
            "successor Revision does not match its exact predecessor",
        ));
    }
    Ok(revision)
}

fn decode_canonical(bytes: &str) -> Result<Revision> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new("Revision wire is not canonical JSON"));
    }
    let envelope = list(&value, 3, "Revision envelope")?;
    require_string(&envelope[0], REVISION_TAG, "Revision envelope tag")?;
    let claimed = decode_revision_id(string(&envelope[1], "Revision identity")?)?;
    let (lineage, model) = decode_payload(&envelope[2])?;
    if claimed != revision_id(&lineage, &model) {
        return Err(KernelError::new(
            "Revision identity does not match the complete semantic-v10 payload",
        ));
    }
    validate_lineage_snapshot(&lineage, &model)?;
    let revision = Revision::reloaded(claimed, lineage, model);
    if semantic_payload(&revision) != json(&envelope[2]) || serialize(&revision) != bytes {
        return Err(KernelError::new("Revision payload is not canonical"));
    }
    Ok(revision)
}

fn decode_payload(value: &Json) -> Result<(RevisionLineage, Model)> {
    let root = list(value, 15, "semantic payload")?;
    require_string(&root[0], SEMANTIC_TAG, "semantic tag")?;
    let lineage = decode_lineage(tagged_group(&root[1], "lineage", "lineage")?)?;
    let model_id = decode_referent_id(tagged_group(&root[2], "model", "model")?)?;

    let mut referents = BTreeMap::new();
    for item in group_array(&root[3], "referents")? {
        let referent = decode_referent(item)?;
        insert(&mut referents, referent.id().clone(), referent, "referent")?;
    }
    let mut contents = BTreeMap::new();
    for item in group_array(&root[4], "relational-contents")? {
        let content = decode_content(item)?;
        insert(
            &mut contents,
            content.id().clone(),
            content,
            "relational content",
        )?;
    }
    let mut shapes = BTreeMap::new();
    for item in group_array(&root[5], "relation-shapes")? {
        let shape = decode_shape(item)?;
        insert(
            &mut shapes,
            shape.referent().clone(),
            shape,
            "relation shape",
        )?;
    }
    let mut structural_contracts = BTreeMap::new();
    for item in group_array(&root[6], "structural-contracts")? {
        let contract = decode_structural_contract(item)?;
        insert(
            &mut structural_contracts,
            contract.referent().clone(),
            contract,
            "structural contract",
        )?;
    }
    let occurrences = group_array(&root[7], "occurrences")?
        .iter()
        .map(decode_occurrence)
        .collect::<Result<Vec<_>>>()?;
    let definitions = group_array(&root[8], "definitions")?
        .iter()
        .map(decode_definition)
        .collect::<Result<Vec<_>>>()?;
    let rules = group_array(&root[9], "derivation-rules")?
        .iter()
        .map(decode_rule)
        .collect::<Result<Vec<_>>>()?;
    let laws = group_array(&root[10], "universal-laws")?
        .iter()
        .map(decode_law)
        .collect::<Result<Vec<_>>>()?;
    let invariants = group_array(&root[11], "invariants")?
        .iter()
        .map(decode_invariant)
        .collect::<Result<Vec<_>>>()?;
    let goals = group_array(&root[12], "goals")?
        .iter()
        .map(decode_goal)
        .collect::<Result<Vec<_>>>()?;
    let transitions = group_array(&root[13], "transitions")?
        .iter()
        .map(decode_transition)
        .collect::<Result<Vec<_>>>()?;
    let judgments = group_array(&root[14], "judgments")?
        .iter()
        .map(decode_judgment)
        .collect::<Result<Vec<_>>>()?;
    let model = Model::with_distinctions(
        model_id,
        referents,
        contents,
        shapes,
        structural_contracts,
        occurrences,
        definitions,
        rules,
        laws,
        invariants,
        goals,
        transitions,
        judgments,
    )?;
    Ok((lineage, model))
}

fn decode_lineage(value: &Json) -> Result<RevisionLineage> {
    let item = array(value, "Revision lineage")?;
    let tag = item
        .first()
        .ok_or_else(|| KernelError::new("invalid Revision lineage"))?;
    match string(tag, "Revision lineage tag")? {
        "root" => {
            list(value, 1, "root lineage")?;
            Ok(RevisionLineage::Root)
        }
        "successor" => {
            let item = list(value, 3, "successor lineage")?;
            let predecessor = decode_revision_id(string(&item[1], "predecessor identity")?)?;
            let delta = decode_delta(&item[2], predecessor.clone())?;
            if delta.base() != &predecessor {
                return Err(KernelError::new(
                    "lineage predecessor and Delta base differ",
                ));
            }
            Ok(RevisionLineage::Successor(delta))
        }
        _ => Err(KernelError::new("invalid Revision lineage tag")),
    }
}

fn decode_delta(value: &Json, base: RevisionId) -> Result<Delta> {
    let item = list(value, 3, "Delta")?;
    require_string(&item[0], "delta", "Delta tag")?;
    let admissions = tagged_array(&item[1], "admit", "Delta admissions")?
        .iter()
        .map(decode_atom)
        .collect::<Result<Vec<_>>>()?;
    let withdrawals = tagged_array(&item[2], "withdraw", "Delta withdrawals")?
        .iter()
        .map(decode_atom)
        .collect::<Result<Vec<_>>>()?;
    Delta::new(base, admissions, withdrawals)
}

fn decode_atom(value: &Json) -> Result<SemanticAtom> {
    let item = array(value, "semantic atom")?;
    let tag = string(
        item.first()
            .ok_or_else(|| KernelError::new("invalid semantic atom"))?,
        "semantic atom tag",
    )?;
    match tag {
        "referent" => decode_referent(value).map(SemanticAtom::Referent),
        "relational-content" => decode_content(value).map(SemanticAtom::RelationalContent),
        "relation-shape" => decode_shape(value).map(SemanticAtom::RelationShape),
        "structural-contract" => {
            decode_structural_contract(value).map(SemanticAtom::StructuralContract)
        }
        "assertion-occurrence" => decode_occurrence(value).map(SemanticAtom::AssertionOccurrence),
        "definition" => decode_definition(value).map(SemanticAtom::Definition),
        "derivation-rule" => decode_rule(value).map(SemanticAtom::DerivationRule),
        "universal-law" => decode_law(value).map(SemanticAtom::UniversalLaw),
        "invariant" => decode_invariant(value).map(SemanticAtom::Invariant),
        "goal" => decode_goal(value).map(SemanticAtom::Goal),
        "transition" => decode_transition(value).map(SemanticAtom::Transition),
        "judgment" => decode_judgment(value).map(SemanticAtom::Judgment),
        _ => Err(KernelError::new("invalid semantic atom tag")),
    }
}

fn decode_referent(value: &Json) -> Result<Referent> {
    let item = list(value, 2, "referent")?;
    require_string(&item[0], "referent", "referent tag")?;
    Ok(Referent::new(decode_referent_id(&item[1])?))
}

fn decode_structural_contract(value: &Json) -> Result<StructuralContract> {
    let item = list(value, 3, "structural contract")?;
    require_string(&item[0], "structural-contract", "structural contract tag")?;
    let referent = decode_referent_id(&item[1])?;
    let form = array(&item[2], "structural form")?;
    let tag = form
        .first()
        .ok_or_else(|| KernelError::new("invalid structural form"))?;
    let form = match string(tag, "structural form tag")? {
        "f32" => {
            list(&item[2], 1, "F32 structural form")?;
            StructuralForm::F32
        }
        "int" => {
            list(&item[2], 1, "Int structural form")?;
            StructuralForm::Int
        }
        "bool" => {
            list(&item[2], 1, "Bool structural form")?;
            StructuralForm::Bool
        }
        "tuple" => {
            let form = list(&item[2], 2, "tuple structural form")?;
            StructuralForm::Tuple(
                array(&form[1], "structural tuple domains")?
                    .iter()
                    .map(decode_referent_id)
                    .collect::<Result<Vec<_>>>()?,
            )
        }
        "product" => {
            let form = list(&item[2], 2, "product structural form")?;
            StructuralForm::Product(
                array(&form[1], "structural product fields")?
                    .iter()
                    .map(decode_referent_id)
                    .collect::<Result<BTreeSet<_>>>()?,
            )
        }
        _ => return Err(KernelError::new("invalid structural form tag")),
    };
    StructuralContract::new(referent, form)
}

fn decode_content(value: &Json) -> Result<RelationalContent> {
    let item = list(value, 4, "relational content")?;
    require_string(&item[0], "relational-content", "relational content tag")?;
    let claimed = decode_content_id(&item[1])?;
    let relation = decode_referent_id(&item[2])?;
    let mut roles = BTreeMap::new();
    for value in tagged_array(&item[3], "roles", "content roles")? {
        let pair = list(value, 2, "content role")?;
        insert(
            &mut roles,
            decode_role_id(&pair[0])?,
            decode_term(&pair[1])?,
            "content role",
        )?;
    }
    let content = RelationalContent::new(relation, roles)?;
    if content.id() != &claimed {
        return Err(KernelError::new("relational content identity mismatch"));
    }
    Ok(content)
}

fn decode_shape(value: &Json) -> Result<RelationShape> {
    let item = list(value, 4, "relation shape")?;
    require_string(&item[0], "relation-shape", "relation shape tag")?;
    let relation = decode_referent_id(&item[1])?;
    let mut roles = BTreeMap::new();
    for value in tagged_array(&item[2], "roles", "shape roles")? {
        let role = decode_role(value)?;
        insert(&mut roles, role.id().clone(), role, "shape role")?;
    }
    let lookup = tagged_array(&item[3], "lookup", "lookup contracts")?
        .iter()
        .map(decode_lookup)
        .collect::<Result<Vec<_>>>()?;
    RelationShape::new(relation, roles, lookup)
}

fn decode_role(value: &Json) -> Result<Role> {
    let item = list(value, 3, "role")?;
    require_string(&item[0], "role", "role tag")?;
    let predicates = tagged_array(&item[2], "admissibility", "role admissibility")?
        .iter()
        .map(decode_predicate)
        .collect::<Result<Vec<_>>>()?;
    Role::new(decode_role_id(&item[1])?, predicates)
}

fn decode_predicate(value: &Json) -> Result<RolePredicate> {
    let item = list(value, 4, "role predicate")?;
    require_string(&item[0], "predicate", "role predicate tag")?;
    let relation = decode_referent_id(&item[1])?;
    let candidate = decode_role_id(&item[2])?;
    let mut fixed = BTreeMap::new();
    for value in tagged_array(&item[3], "fixed", "fixed predicate roles")? {
        let pair = list(value, 2, "fixed predicate role")?;
        insert(
            &mut fixed,
            decode_role_id(&pair[0])?,
            decode_referent_id(&pair[1])?,
            "fixed predicate role",
        )?;
    }
    RolePredicate::new(relation, candidate, fixed)
}

fn decode_lookup(value: &Json) -> Result<LookupMode> {
    let item = list(value, 4, "lookup contract")?;
    require_string(&item[0], "lookup", "lookup tag")?;
    let known = decode_role_list(tagged_group(&item[1], "known", "known roles")?)?;
    let sought = decode_role_list(tagged_group(&item[2], "sought", "sought roles")?)?;
    let cardinality = match string(
        tagged_group(&item[3], "cardinality", "cardinality")?,
        "cardinality value",
    )? {
        "one" => Cardinality::One,
        "maybe" => Cardinality::Maybe,
        "some" => Cardinality::Some,
        "many" => Cardinality::Many,
        _ => return Err(KernelError::new("invalid lookup cardinality")),
    };
    LookupMode::finite(known, sought, cardinality)
}

fn decode_occurrence(value: &Json) -> Result<AssertionOccurrence> {
    let item = list(value, 5, "assertion occurrence")?;
    require_string(&item[0], "assertion-occurrence", "assertion occurrence tag")?;
    Ok(AssertionOccurrence::new(
        decode_referent_id(&item[1])?,
        decode_content_id(&item[2])?,
        decode_referent_id(tagged_group(&item[3], "source", "occurrence source")?)?,
        decode_referent_id(tagged_group(&item[4], "scope", "occurrence scope")?)?,
    ))
}

fn decode_definition(value: &Json) -> Result<Definition> {
    let item = list(value, 3, "definition")?;
    require_string(&item[0], "definition", "definition tag")?;
    Ok(Definition::new(
        decode_referent_id(&item[1])?,
        decode_term(&item[2])?,
    ))
}

fn decode_rule(value: &Json) -> Result<DerivationRule> {
    let item = list(value, 7, "derivation rule")?;
    require_string(&item[0], "derivation-rule", "derivation rule tag")?;
    DerivationRule::new(
        decode_referent_id(&item[1])?,
        decode_referent_id(tagged_group(
            &item[2],
            "governing-law",
            "rule governing law",
        )?)?,
        decode_referent_id(tagged_group(&item[3], "scope", "rule scope")?)?,
        decode_referent_id(tagged_group(&item[4], "authority", "rule authority")?)?,
        decode_pattern(tagged_group(&item[5], "premises", "rule premises")?)?,
        decode_pattern(tagged_group(&item[6], "conclusion", "rule conclusion")?)?,
    )
}

fn decode_law(value: &Json) -> Result<UniversalLaw> {
    let item = list(value, 5, "universal law")?;
    require_string(&item[0], "universal-law", "universal law tag")?;
    Ok(UniversalLaw::new(
        decode_referent_id(&item[1])?,
        decode_referent_id(tagged_group(&item[2], "scope", "law scope")?)?,
        decode_pattern(tagged_group(&item[3], "premises", "law premises")?)?,
        decode_pattern(tagged_group(&item[4], "conclusion", "law conclusion")?)?,
    ))
}

fn decode_invariant(value: &Json) -> Result<Invariant> {
    let item = list(value, 6, "invariant")?;
    require_string(&item[0], "invariant", "invariant tag")?;
    let admission = match string(
        tagged_group(&item[5], "admission", "invariant admission")?,
        "invariant admission value",
    )? {
        "reject-on-match" => InvariantAdmission::RejectOnMatch,
        "require-match" => InvariantAdmission::RequireMatch,
        _ => return Err(KernelError::new("invalid invariant admission behavior")),
    };
    Ok(Invariant::new(
        decode_referent_id(&item[1])?,
        decode_referent_id(tagged_group(&item[2], "scope", "invariant scope")?)?,
        decode_referent_id(tagged_group(&item[3], "policy", "invariant policy")?)?,
        decode_pattern(tagged_group(&item[4], "condition", "invariant condition")?)?,
        admission,
    ))
}

fn decode_goal(value: &Json) -> Result<Goal> {
    let item = list(value, 4, "goal")?;
    require_string(&item[0], "goal", "goal tag")?;
    Ok(Goal::new(
        decode_referent_id(&item[1])?,
        decode_referent_id(tagged_group(&item[2], "context", "goal context")?)?,
        decode_pattern(tagged_group(&item[3], "desired", "goal pattern")?)?,
    ))
}

fn decode_pattern(value: &Json) -> Result<Pattern> {
    let item = list(value, 2, "pattern")?;
    require_string(&item[0], "pattern", "pattern tag")?;
    Pattern::new(decode_content_list(&item[1])?)
}

fn decode_transition(value: &Json) -> Result<Transition> {
    let item = list(value, 7, "transition")?;
    require_string(&item[0], "transition", "transition tag")?;
    Transition::for_event(
        decode_referent_id(&item[1])?,
        decode_referent_id(tagged_group(&item[2], "event", "transition event")?)?,
        array(
            tagged_group(&item[3], "payload-bindings", "transition payload bindings")?,
            "transition payload bindings",
        )?
        .iter()
        .map(decode_pattern_id)
        .collect::<Result<Vec<_>>>()?,
        array(
            tagged_group(&item[4], "guards", "transition guards")?,
            "transition guards",
        )?
        .iter()
        .map(decode_content_id)
        .collect::<Result<Vec<_>>>()?,
        decode_content_id(tagged_group(&item[5], "from", "transition source")?)?,
        decode_content_id(tagged_group(&item[6], "to", "transition destination")?)?,
    )
}

fn decode_judgment(value: &Json) -> Result<Judgment> {
    let item = list(value, 7, "judgment")?;
    require_string(&item[0], "judgment", "judgment tag")?;
    let target_value = tagged_group(&item[4], "target", "judgment target")?;
    let target_item = list(target_value, 2, "judgment target value")?;
    let target = match string(&target_item[0], "judgment target tag")? {
        "content" => JudgmentTarget::Content(decode_content_id(&target_item[1])?),
        "occurrence" => JudgmentTarget::Occurrence(decode_referent_id(&target_item[1])?),
        _ => return Err(KernelError::new("invalid judgment target tag")),
    };
    let status = match string(
        tagged_group(&item[6], "status", "judgment status")?,
        "judgment status value",
    )? {
        "affirmed" => JudgmentStatus::Affirmed,
        "disputed" => JudgmentStatus::Disputed,
        "withdrawn" => JudgmentStatus::Withdrawn,
        _ => return Err(KernelError::new("invalid judgment status")),
    };
    Ok(Judgment::new(
        decode_referent_id(&item[1])?,
        decode_referent_id(tagged_group(&item[2], "authority", "judgment authority")?)?,
        decode_referent_id(tagged_group(&item[3], "scope", "judgment scope")?)?,
        target,
        decode_judgment_kind(tagged_group(&item[5], "kind", "judgment kind")?)?,
        status,
    ))
}

fn decode_judgment_kind(value: &Json) -> Result<JudgmentKind> {
    let item = array(value, "judgment kind")?;
    let tag = string(
        item.first()
            .ok_or_else(|| KernelError::new("invalid judgment kind"))?,
        "judgment kind tag",
    )?;
    match tag {
        "declared" => {
            list(value, 1, "declared judgment")?;
            Ok(JudgmentKind::Declared)
        }
        "derived" => {
            let item = list(value, 3, "derived judgment")?;
            Ok(JudgmentKind::Derived {
                rule: decode_referent_id(&item[1])?,
                premises: decode_content_list(&item[2])?,
            })
        }
        "observed" => {
            let item = list(value, 2, "observed judgment")?;
            Ok(JudgmentKind::Observed {
                evidence: decode_referent_id(&item[1])?,
            })
        }
        "admitted" | "rejected" => {
            let item = list(value, 3, "admission judgment")?;
            let policy = decode_referent_id(&item[1])?;
            let basis = array(&item[2], "judgment basis")?
                .iter()
                .map(decode_referent_id)
                .collect::<Result<Vec<_>>>()?;
            if tag == "admitted" {
                Ok(JudgmentKind::Admitted { policy, basis })
            } else {
                Ok(JudgmentKind::Rejected { policy, basis })
            }
        }
        "superseded" => {
            let item = list(value, 2, "superseded judgment")?;
            Ok(JudgmentKind::Superseded {
                by: decode_referent_id(&item[1])?,
            })
        }
        _ => Err(KernelError::new("invalid judgment kind tag")),
    }
}

pub(crate) fn decode_term(value: &Json) -> Result<Term> {
    let item = array(value, "term")?;
    let tag = item
        .first()
        .ok_or_else(|| KernelError::new("invalid term"))?;
    match string(tag, "term tag")? {
        "referent" => {
            let item = list(value, 2, "referent term")?;
            Ok(Term::referent(decode_referent_id(&item[1])?))
        }
        "pattern" => {
            let item = list(value, 2, "pattern term")?;
            Ok(Term::pattern(PatternId::new(
                string(&item[1], "pattern identity")?.to_owned(),
            )?))
        }
        "application" => {
            let item = list(value, 2, "application term")?;
            Ok(Term::application(decode_content_id(&item[1])?))
        }
        "f32" => {
            let item = list(value, 2, "F32 term")?;
            let bits = string(&item[1], "F32 bits")?;
            if bits.len() != 8 {
                return Err(KernelError::new("invalid F32 bits"));
            }
            let bits =
                u32::from_str_radix(bits, 16).map_err(|_| KernelError::new("invalid F32 bits"))?;
            Term::f32_bits(bits)
        }
        "int" => {
            let item = list(value, 2, "Int term")?;
            let value = string(&item[1], "Int value")?
                .parse::<i64>()
                .map_err(|_| KernelError::new("invalid Int value"))?;
            Ok(Term::int(value))
        }
        "bool" => {
            let item = list(value, 2, "Bool term")?;
            match string(&item[1], "Bool value")? {
                "true" => Ok(Term::boolean(true)),
                "false" => Ok(Term::boolean(false)),
                _ => Err(KernelError::new("invalid Bool value")),
            }
        }
        "product" => {
            let item = list(value, 3, "product term")?;
            let shape = decode_referent_id(&item[1])?;
            let mut fields = BTreeMap::new();
            for field in array(&item[2], "product fields")? {
                let field = list(field, 3, "product field")?;
                let label = Name::new(string(&field[0], "product label")?.to_owned())?;
                let domain = decode_referent_id(&field[1])?;
                if fields
                    .insert(label, ProductField::new(domain, decode_term(&field[2])?))
                    .is_some()
                {
                    return Err(KernelError::new("duplicate product label"));
                }
            }
            Term::product(shape, fields)
        }
        "labelled-product" => {
            let item = list(value, 3, "labelled product term")?;
            let shape = decode_referent_id(&item[1])?;
            let mut fields = BTreeMap::new();
            for field in array(&item[2], "labelled product fields")? {
                let field = list(field, 2, "labelled product field")?;
                if fields
                    .insert(decode_referent_id(&field[0])?, decode_term(&field[1])?)
                    .is_some()
                {
                    return Err(KernelError::new("duplicate labelled product field"));
                }
            }
            Term::labelled_product(shape, fields)
        }
        "sum" => {
            let item = list(value, 3, "sum term")?;
            Term::sum(
                Name::new(string(&item[1], "sum tag")?.to_owned())?,
                decode_term(&item[2])?,
            )
        }
        "sequence" => {
            let item = list(value, 4, "sequence term")?;
            Term::sequence(
                decode_referent_id(&item[1])?,
                decode_referent_id(&item[2])?,
                array(&item[3], "sequence values")?
                    .iter()
                    .map(decode_term)
                    .collect::<Result<Vec<_>>>()?,
            )
        }
        _ => Err(KernelError::new("invalid term tag")),
    }
}

fn validate_lineage_snapshot(lineage: &RevisionLineage, model: &Model) -> Result<()> {
    let RevisionLineage::Successor(delta) = lineage else {
        return Ok(());
    };
    let atoms = model.atoms();
    if delta.admissions().iter().any(|atom| !atoms.contains(atom))
        || delta.withdrawals().iter().any(|atom| atoms.contains(atom))
    {
        return Err(KernelError::new(
            "successor snapshot contradicts its signed Delta",
        ));
    }
    Ok(())
}

fn group_array<'a>(value: &'a Json, tag: &str) -> Result<&'a [Json]> {
    tagged_array(value, tag, tag)
}
fn tagged_array<'a>(value: &'a Json, tag: &str, where_: &str) -> Result<&'a [Json]> {
    array(tagged_group(value, tag, where_)?, where_)
}
fn tagged_group<'a>(value: &'a Json, tag: &str, where_: &str) -> Result<&'a Json> {
    let item = list(value, 2, where_)?;
    require_string(&item[0], tag, &format!("{where_} tag"))?;
    Ok(&item[1])
}
fn decode_role_list(value: &Json) -> Result<Vec<RoleId>> {
    array(value, "role list")?
        .iter()
        .map(decode_role_id)
        .collect()
}
fn decode_content_list(value: &Json) -> Result<Vec<ContentId>> {
    array(value, "content identity list")?
        .iter()
        .map(decode_content_id)
        .collect()
}
fn decode_referent_id(value: &Json) -> Result<ReferentId> {
    ReferentId::new(string(value, "referent identity")?.to_owned())
}
fn decode_content_id(value: &Json) -> Result<ContentId> {
    ContentId::new(string(value, "content identity")?.to_owned())
}
fn decode_role_id(value: &Json) -> Result<RoleId> {
    RoleId::new(string(value, "role identity")?.to_owned())
}

fn decode_pattern_id(value: &Json) -> Result<PatternId> {
    PatternId::new(string(value, "pattern identity")?.into())
}
fn decode_revision_id(value: &str) -> Result<RevisionId> {
    let hex = value
        .strip_prefix("rev-sha256-")
        .ok_or_else(|| KernelError::new("invalid Revision identity"))?;
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(KernelError::new("invalid Revision identity"));
    }
    let mut digest = [0; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| KernelError::new("invalid Revision identity"))?;
    }
    Ok(RevisionId::from_digest(digest))
}
fn insert<K: Ord, V>(map: &mut BTreeMap<K, V>, key: K, value: V, where_: &str) -> Result<()> {
    if map.insert(key, value).is_some() {
        Err(KernelError::new(format!("duplicate {where_}")))
    } else {
        Ok(())
    }
}
