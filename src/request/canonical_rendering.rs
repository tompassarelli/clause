use super::{EvaluationOutput, QueryColumn, RequestOutput, RunOutput};
use crate::{
    execution::{self, Proof, WhyAll},
    intervention::{AchieveAll, AchieveOne, Incomplete, Intervention, PreventAll, PreventOne},
    kernel::{RelationalContent, RoleId, Term},
    semantic_diff::SemanticDiff,
};

pub(super) fn canonical_bytes(output: &RunOutput) -> String {
    format!(
        "[\"clause-run-v1\",[{}]]",
        output
            .results
            .iter()
            .map(request_output)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(super) fn evaluation_bytes(output: &EvaluationOutput) -> String {
    format!(
        "[\"clause-evaluate-v1\",{},[{}]]",
        string(&output.revision.to_string()),
        output
            .definitions
            .iter()
            .map(|(definition, result)| {
                format!("[{},{}]", string(definition.as_str()), term(result))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn request_output(value: &RequestOutput) -> String {
    match value {
        RequestOutput::Any(value) => format!("[\"any\",{value}]"),
        RequestOutput::Select { columns, rows } => selection("select", columns, rows),
        RequestOutput::SelectOne { columns, rows } => selection("select-one", columns, rows),
        RequestOutput::SelectFirst { columns, rows } => selection("select-first", columns, rows),
        RequestOutput::Find(items) => format!(
            "[\"find\",[{}]]",
            items.iter().map(term).collect::<Vec<_>>().join(",")
        ),
        RequestOutput::WhyOne(chosen) => format!(
            "[\"why\",{}]",
            chosen.as_ref().map(proof).unwrap_or_else(|| "null".into())
        ),
        RequestOutput::WhyAll(frontier) => format!(
            "[\"why-all\",{}]",
            frontier
                .as_ref()
                .map(why_all)
                .unwrap_or_else(|| "null".into())
        ),
        RequestOutput::PreventOne(result) => {
            format!("[\"prevent-one\",{}]", prevent_one(result))
        }
        RequestOutput::PreventAll(result) => {
            format!("[\"prevent-all\",{}]", prevent_all(result))
        }
        RequestOutput::AchieveOne(result) => {
            format!("[\"achieve-one\",{}]", achieve_one(result))
        }
        RequestOutput::AchieveAll(result) => {
            format!("[\"achieve-all\",{}]", achieve_all(result))
        }
        RequestOutput::Diff(diff) => diff_json(diff),
    }
}

fn selection(tag: &str, columns: &[QueryColumn], rows: &[execution::QueryRow]) -> String {
    format!(
        "[{},[{}],[{}]]",
        string(tag),
        columns
            .iter()
            .map(|column| format!(
                "[{},[{}],{}]",
                string(column.binder().as_str()),
                role_origins(column.origins()),
                column
                    .label()
                    .map(string)
                    .unwrap_or_else(|| "null".to_owned())
            ))
            .collect::<Vec<_>>()
            .join(","),
        rows.iter()
            .map(|row| format!(
                "[{}]",
                row.cells()
                    .iter()
                    .map(|cell| format!(
                        "[[{}],{}]",
                        role_origins(cell.origins()),
                        term(cell.value())
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn role_origins(origins: &[RoleId]) -> String {
    origins
        .iter()
        .map(|role| string(role.as_str()))
        .collect::<Vec<_>>()
        .join(",")
}

fn string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('\"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}
fn term(value: &Term) -> String {
    match value {
        Term::Referent(id) => format!("[\"referent\",{}]", string(id.as_str())),
        Term::Pattern(id) => format!("[\"pattern\",{}]", string(id.as_str())),
        Term::Application(id) => format!("[\"application\",{}]", string(id.as_str())),
        Term::F32(value) => format!("[\"f32\",\"{:08x}\"]", value.bits()),
        Term::Int(value) => format!("[\"int\",\"{value}\"]"),
        Term::Bool(value) => format!("[\"bool\",\"{value}\"]"),
        Term::Product { shape, fields } => format!(
            "[\"product\",{},[{}]]",
            string(shape.as_str()),
            fields
                .iter()
                .map(|(label, field)| format!(
                    "[{},{},{}]",
                    string(label.as_str()),
                    string(field.domain().as_str()),
                    term(field.value())
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Term::LabelledProduct { shape, fields } => format!(
            "[\"labelled-product\",{},[{}]]",
            string(shape.as_str()),
            fields
                .iter()
                .map(|(field, value)| format!("[{},{}]", string(field.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Term::Sum { tag, value } => {
            format!("[\"sum\",{},{}]", string(tag.as_str()), term(value))
        }
        Term::Sequence {
            shape,
            element,
            values,
        } => format!(
            "[\"sequence\",{},{},[{}]]",
            string(shape.as_str()),
            string(element.as_str()),
            values.iter().map(term).collect::<Vec<_>>().join(",")
        ),
    }
}
fn clause(value: &RelationalContent) -> String {
    format!(
        "[\"clause\",{},[{}]]",
        string(value.relation().as_str()),
        value
            .roles()
            .iter()
            .map(|(role, value)| format!("[{},{}]", string(role.as_str()), term(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn proof(value: &Proof) -> String {
    format!(
        "[\"proof\",{},{}]",
        string(&value.revision.to_string()),
        graph(&value.why)
    )
}
fn graph(value: &execution::WhyGraph) -> String {
    format!(
        "[\"graph\",{},[{}],[{}]]",
        value.root,
        value
            .nodes
            .iter()
            .map(|node| clause(&node.clause))
            .collect::<Vec<_>>()
            .join(","),
        value
            .witnesses
            .iter()
            .map(|edge| format!("[{},{}]", edge.conclusion, witness(&edge.witness)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn witness(value: &execution::Witness) -> String {
    match value {
        execution::Witness::Asserted => "[\"asserted\"]".into(),
        execution::Witness::Derived {
            rule,
            governing_law,
            authority,
            scope,
            premises,
            substitution,
        } => format!(
            "[\"derived\",{},{},{},{},[{}],[{}]]",
            string(rule.as_str()),
            string(governing_law.as_str()),
            string(authority.as_str()),
            string(scope.as_str()),
            premises
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(","),
            substitution
                .iter()
                .map(|(id, value)| format!("[{},{}]", string(id.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn why_all(value: &WhyAll) -> String {
    format!(
        "[\"why-all\",{},{},{},{},[{}]]",
        string(&value.revision.to_string()),
        clause(&value.target),
        value.complete,
        value.expansions,
        value
            .alternatives
            .iter()
            .map(|item| format!(
                "[[{}],{}]",
                item.assertions
                    .iter()
                    .map(clause)
                    .collect::<Vec<_>>()
                    .join(","),
                proof(&item.proof)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn intervention(value: &Intervention) -> String {
    format!(
        "[\"intervention\",{},[{}],[{}],{},{}]",
        string(&value.revision().identity().to_string()),
        value
            .admissions()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .withdrawals()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        string(&value.delta().base().to_string()),
        value
            .proof()
            .map(derive_proof)
            .unwrap_or_else(|| "null".into())
    )
}
fn derive_proof(value: &crate::derive::Proof) -> String {
    format!(
        "[\"derivation\",{},{}]",
        value.generation(),
        derive_witness(value.witness())
    )
}
fn derive_witness(value: &crate::derive::Witness) -> String {
    match value {
        crate::derive::Witness::Asserted => "[\"asserted\"]".into(),
        crate::derive::Witness::Derived {
            rule,
            governing_law,
            authority,
            scope,
            premises,
            substitution,
        } => format!(
            "[\"derived\",{},{},{},{},[{}],[{}]]",
            string(rule.as_str()),
            string(governing_law.as_str()),
            string(authority.as_str()),
            string(scope.as_str()),
            premises.iter().map(clause).collect::<Vec<_>>().join(","),
            substitution
                .iter()
                .map(|(id, value)| format!("[{},{}]", string(id.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn support_proof(value: &crate::derive::SupportProof) -> String {
    format!(
        "[\"support-proof\",{},{}]",
        clause(value.conclusion()),
        support_witness(value.witness())
    )
}
fn support_witness(value: &crate::derive::SupportWitness) -> String {
    match value {
        crate::derive::SupportWitness::Asserted => "[\"asserted\"]".into(),
        crate::derive::SupportWitness::Derived {
            rule,
            governing_law,
            authority,
            scope,
            premises,
            substitution,
        } => format!(
            "[\"derived\",{},{},{},{},[{}],[{}]]",
            string(rule.as_str()),
            string(governing_law.as_str()),
            string(authority.as_str()),
            string(scope.as_str()),
            premises
                .iter()
                .map(support_proof)
                .collect::<Vec<_>>()
                .join(","),
            substitution
                .iter()
                .map(|(id, value)| format!("[{},{}]", string(id.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn support_frontier(value: &crate::derive::SupportFrontier) -> String {
    format!(
        "[\"support-frontier\",{},{},{},{},[{}]]",
        string(&value.revision().to_string()),
        clause(value.target()),
        string(match value.status() {
            crate::derive::SupportStatus::Complete => "complete",
            crate::derive::SupportStatus::ExpansionBudgetExhausted => "expansion-budget-exhausted",
            crate::derive::SupportStatus::SupportBudgetExhausted => "support-budget-exhausted",
        }),
        value.expansions(),
        value
            .supports()
            .iter()
            .map(|support| format!(
                "[[{}],{}]",
                support
                    .assertions()
                    .iter()
                    .map(clause)
                    .collect::<Vec<_>>()
                    .join(","),
                support_proof(support.proof())
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn incomplete(value: Incomplete) -> &'static str {
    match value {
        Incomplete::CandidateBudgetExhausted => "candidate-budget-exhausted",
        Incomplete::SolutionBudgetExhausted => "solution-budget-exhausted",
        Incomplete::ClosureBudgetExhausted => "closure-budget-exhausted",
        Incomplete::SupportExpansionBudgetExhausted => "support-expansion-budget-exhausted",
        Incomplete::SupportBudgetExhausted => "support-budget-exhausted",
    }
}
fn prevent_one(value: &PreventOne) -> String {
    match value {
        PreventOne::Satisfied(item) => format!("[\"satisfied\",{}]", intervention(item)),
        PreventOne::AlreadyAbsent => "[\"already-absent\"]".into(),
        PreventOne::Impossible => "[\"impossible\"]".into(),
        PreventOne::Incomplete(reason) => {
            format!("[\"incomplete\",{}]", string(incomplete(*reason)))
        }
    }
}
fn achieve_one(value: &AchieveOne) -> String {
    match value {
        AchieveOne::Satisfied(item) => format!("[\"satisfied\",{}]", intervention(item)),
        AchieveOne::AlreadyEntailed => "[\"already-entailed\"]".into(),
        AchieveOne::Impossible => "[\"impossible\"]".into(),
        AchieveOne::Incomplete(reason) => {
            format!("[\"incomplete\",{}]", string(incomplete(*reason)))
        }
    }
}
fn prevent_all(value: &PreventAll) -> String {
    match value {
        PreventAll::Complete(items) => format!(
            "[\"complete\",[{}]]",
            items.iter().map(intervention).collect::<Vec<_>>().join(",")
        ),
        PreventAll::AlreadyAbsent => "[\"already-absent\"]".into(),
        PreventAll::Impossible => "[\"impossible\"]".into(),
        PreventAll::Incomplete {
            interventions,
            reason,
        } => format!(
            "[\"incomplete\",{},[{}]]",
            string(incomplete(*reason)),
            interventions
                .iter()
                .map(intervention)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn achieve_all(value: &AchieveAll) -> String {
    match value {
        AchieveAll::Complete(items) => format!(
            "[\"complete\",[{}]]",
            items.iter().map(intervention).collect::<Vec<_>>().join(",")
        ),
        AchieveAll::AlreadyEntailed => "[\"already-entailed\"]".into(),
        AchieveAll::Impossible => "[\"impossible\"]".into(),
        AchieveAll::Incomplete {
            interventions,
            reason,
        } => format!(
            "[\"incomplete\",{},[{}]]",
            string(incomplete(*reason)),
            interventions
                .iter()
                .map(intervention)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn diff_json(value: &SemanticDiff) -> String {
    format!(
        "[\"diff\",[\"authored\",{},{},[{}],[{}]],[\"entailed-added\",[{}]],[\"entailed-removed\",[{}]],[\"proof-changes\",[{}]],[\"support-changes\",[{}]]]",
        string(&value.authored().base_revision().to_string()),
        string(&value.authored().successor_revision().to_string()),
        value
            .authored()
            .added()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .authored()
            .removed()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .entailed_added()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .entailed_removed()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .changed_proofs()
            .iter()
            .map(|change| format!(
                "[{},{},{}]",
                clause(change.consequence()),
                derive_proof(change.base()),
                derive_proof(change.successor())
            ))
            .collect::<Vec<_>>()
            .join(","),
        value
            .changed_supports()
            .iter()
            .map(|change| format!(
                "[{},{},{},[{}],[{}],[{}]]",
                clause(change.consequence()),
                support_frontier(change.base()),
                support_frontier(change.successor()),
                change
                    .added()
                    .iter()
                    .map(|support| support_proof(support.proof()))
                    .collect::<Vec<_>>()
                    .join(","),
                change
                    .removed()
                    .iter()
                    .map(|support| support_proof(support.proof()))
                    .collect::<Vec<_>>()
                    .join(","),
                change
                    .retained()
                    .iter()
                    .map(|support| support_proof(support.proof()))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
