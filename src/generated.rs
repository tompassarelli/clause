//! Source-deletion-safe request-program emission.

#![allow(unexpected_cfgs)]

use std::fmt::Write;

use crate::{
    kernel::{Clause, KernelError, RelationId, Result, RoleId, Term, TypeId, VariableId},
    request::{Request, ResolvedProgram, Selection},
    wire,
};

#[cfg(not(clause_generated))]
struct ChildModule {
    name: &'static str,
    declaration: &'static str,
    source: &'static str,
}

#[cfg(not(clause_generated))]
const NO_CHILDREN: &[ChildModule] = &[];

#[cfg(not(clause_generated))]
const KERNEL_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "clause",
        declaration: "#[path = \"kernel/clause.rs\"]\nmod clause;",
        source: include_str!("kernel/clause.rs"),
    },
    ChildModule {
        name: "error",
        declaration: "#[path = \"kernel/error.rs\"]\nmod error;",
        source: include_str!("kernel/error.rs"),
    },
    ChildModule {
        name: "find",
        declaration: "#[path = \"kernel/find.rs\"]\nmod find;",
        source: include_str!("kernel/find.rs"),
    },
    ChildModule {
        name: "identity",
        declaration: "#[path = \"kernel/identity.rs\"]\nmod identity;",
        source: include_str!("kernel/identity.rs"),
    },
    ChildModule {
        name: "model",
        declaration: "#[path = \"kernel/model.rs\"]\nmod model;",
        source: include_str!("kernel/model.rs"),
    },
    ChildModule {
        name: "revision",
        declaration: "#[path = \"kernel/revision.rs\"]\nmod revision;",
        source: include_str!("kernel/revision.rs"),
    },
    ChildModule {
        name: "schema",
        declaration: "#[path = \"kernel/schema.rs\"]\nmod schema;",
        source: include_str!("kernel/schema.rs"),
    },
];

#[cfg(not(clause_generated))]
const WIRE_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "canonical",
        declaration: "mod canonical;",
        source: include_str!("wire/canonical.rs"),
    },
    ChildModule {
        name: "decode",
        declaration: "mod decode;",
        source: include_str!("wire/decode.rs"),
    },
    ChildModule {
        name: "json",
        declaration: "mod json;",
        source: include_str!("wire/json.rs"),
    },
    ChildModule {
        name: "sha256",
        declaration: "mod sha256;",
        source: include_str!("wire/sha256.rs"),
    },
];

#[cfg(not(clause_generated))]
const DERIVE_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "closure",
        declaration: "mod closure;",
        source: include_str!("derive/closure.rs"),
    },
    ChildModule {
        name: "matching",
        declaration: "mod matching;",
        source: include_str!("derive/matching.rs"),
    },
    ChildModule {
        name: "support",
        declaration: "mod support;",
        source: include_str!("derive/support.rs"),
    },
];

#[cfg(not(clause_generated))]
const EXECUTION_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "explain",
        declaration: "mod explain;",
        source: include_str!("execution/explain.rs"),
    },
    ChildModule {
        name: "query",
        declaration: "mod query;",
        source: include_str!("execution/query.rs"),
    },
];

#[cfg(not(clause_generated))]
const INTERVENTION_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "all",
        declaration: "mod all;",
        source: include_str!("intervention/all.rs"),
    },
    ChildModule {
        name: "basis",
        declaration: "mod basis;",
        source: include_str!("intervention/basis.rs"),
    },
    ChildModule {
        name: "closure",
        declaration: "mod closure;",
        source: include_str!("intervention/closure.rs"),
    },
    ChildModule {
        name: "one",
        declaration: "mod one;",
        source: include_str!("intervention/one.rs"),
    },
    ChildModule {
        name: "search",
        declaration: "mod search;",
        source: include_str!("intervention/search.rs"),
    },
];

#[cfg(not(clause_generated))]
const SEMANTIC_DIFF_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "entailment",
        declaration: "mod entailment;",
        source: include_str!("semantic_diff/entailment.rs"),
    },
    ChildModule {
        name: "proofs",
        declaration: "mod proofs;",
        source: include_str!("semantic_diff/proofs.rs"),
    },
    ChildModule {
        name: "supports",
        declaration: "mod supports;",
        source: include_str!("semantic_diff/supports.rs"),
    },
];

#[cfg(not(clause_generated))]
const REQUEST_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "canonical_rendering",
        declaration: "mod canonical_rendering;",
        source: include_str!("request/canonical_rendering.rs"),
    },
    ChildModule {
        name: "ordered_execution",
        declaration: "mod ordered_execution;",
        source: include_str!("request/ordered_execution.rs"),
    },
];

/// Emit a standalone program that reloads the referenced v3 Revisions and
/// invokes the same ordered request evaluator as the interpreter.
#[cfg(not(clause_generated))]
pub fn emit_rust(program: &ResolvedProgram) -> Result<String> {
    let mut modules = String::new();
    for (name, source, children) in [
        ("kernel", include_str!("kernel.rs"), KERNEL_CHILDREN),
        ("wire", include_str!("wire.rs"), WIRE_CHILDREN),
        ("derive", include_str!("derive.rs"), DERIVE_CHILDREN),
        ("delta", include_str!("delta.rs"), NO_CHILDREN),
        (
            "execution",
            include_str!("execution.rs"),
            EXECUTION_CHILDREN,
        ),
        (
            "intervention",
            include_str!("intervention.rs"),
            INTERVENTION_CHILDREN,
        ),
        (
            "semantic_diff",
            include_str!("semantic_diff.rs"),
            SEMANTIC_DIFF_CHILDREN,
        ),
        ("request", include_str!("request.rs"), REQUEST_CHILDREN),
    ] {
        let body = production_module(name, source, children)?;
        writeln!(modules, "mod {name} {{\n{body}\n}}")
            .expect("writing generated modules to a String cannot fail");
    }
    let mut body = String::new();
    for (index, revision) in program.revisions().values().enumerate() {
        writeln!(
            body,
            "let r{index} = wire::reload({:?}).expect(\"sealed revision reloads\");",
            wire::serialize(revision)
        )
        .expect("writing generated Revision reloads to a String cannot fail");
    }
    writeln!(body, "let program = request::ResolvedProgram::new(std::collections::BTreeMap::from([{}]), vec![{}]).expect(\"generated requests resolve\");", program.revisions().values().enumerate().map(|(index, _)| format!("(r{index}.identity().clone(), r{index}.clone())")).collect::<Vec<_>>().join(","), program.requests().iter().map(|request| request_source(request, program)).collect::<Vec<_>>().join(",")).expect("writing the generated request registry to a String cannot fail");
    writeln!(body, "print!(\"{{}}\", request::run(&program, request::RunLimits::default()).expect(\"generated requests run\").canonical_bytes());").expect("writing the generated entry point to a String cannot fail");
    Ok(format!("{modules}\nfn main() {{ {body} }}"))
}

#[cfg(not(clause_generated))]
fn production_module(name: &str, source: &str, children: &[ChildModule]) -> Result<String> {
    let mut source = source.to_owned();
    for child in children {
        let occurrences = source.match_indices(child.declaration).count();
        if occurrences != 1 {
            return Err(KernelError::new(format!(
                "generated module '{name}' expected exactly one declaration for child '{}', found {occurrences}",
                child.name
            )));
        }
        source = source.replacen(
            child.declaration,
            &format!("mod {} {{\n{}\n}}", child.name, child.source),
            1,
        );
    }
    Ok(source)
}

#[cfg(not(clause_generated))]
fn revision_source(identity: &crate::kernel::RevisionId, program: &ResolvedProgram) -> String {
    let index = program
        .revisions()
        .keys()
        .position(|candidate| candidate == identity)
        .expect("request revision is embedded");
    format!("r{index}.identity().clone()")
}

#[cfg(not(clause_generated))]
fn request_source(request: &Request, program: &ResolvedProgram) -> String {
    match request {
        Request::Find {
            revision,
            pattern,
            sought,
        } => format!(
            "request::Request::Find {{ revision: {}, pattern: {}, sought: {} }}",
            revision_source(revision, program),
            clause_source(pattern),
            variable_source(sought)
        ),
        Request::Why {
            revision,
            target,
            all,
        } => format!(
            "request::Request::Why {{ revision: {}, target: {}, all: {all} }}",
            revision_source(revision, program),
            clause_source(target)
        ),
        Request::Prevent {
            revision,
            target,
            selection,
            using,
        } => format!(
            "request::Request::Prevent {{ revision: {}, target: {}, selection: {}, using: vec![{}] }}",
            revision_source(revision, program),
            clause_source(target),
            selection_source(*selection),
            using
                .iter()
                .map(relation_source)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Request::Achieve {
            revision,
            target,
            selection,
            using,
        } => format!(
            "request::Request::Achieve {{ revision: {}, target: {}, selection: {}, using: vec![{}] }}",
            revision_source(revision, program),
            clause_source(target),
            selection_source(*selection),
            using
                .iter()
                .map(relation_source)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Request::Diff { base, successor } => format!(
            "request::Request::Diff {{ base: {}, successor: {} }}",
            revision_source(base, program),
            revision_source(successor, program)
        ),
    }
}

#[cfg(not(clause_generated))]
fn selection_source(selection: Selection) -> &'static str {
    match selection {
        Selection::OneMinimal => "request::Selection::OneMinimal",
        Selection::AllMinimal => "request::Selection::AllMinimal",
    }
}
#[cfg(not(clause_generated))]
fn name_source(value: &str) -> String {
    format!("kernel::Name::new({value:?}.into()).expect(\"generated name\")")
}
#[cfg(not(clause_generated))]
fn relation_source(value: &RelationId) -> String {
    format!(
        "kernel::RelationId::new({}).expect(\"generated relation\")",
        name_source(value.as_str())
    )
}
#[cfg(not(clause_generated))]
fn role_source(value: &RoleId) -> String {
    format!(
        "kernel::RoleId::new({}).expect(\"generated role\")",
        name_source(value.as_str())
    )
}
#[cfg(not(clause_generated))]
fn type_source(value: &TypeId) -> String {
    format!(
        "kernel::TypeId::new({}).expect(\"generated type\")",
        name_source(value.as_str())
    )
}
#[cfg(not(clause_generated))]
fn variable_source(value: &VariableId) -> String {
    format!(
        "kernel::VariableId::new({}).expect(\"generated variable\")",
        name_source(value.as_str())
    )
}
#[cfg(not(clause_generated))]
fn term_source(value: &Term) -> String {
    match value {
        Term::Entity(entity) => format!(
            "kernel::Term::entity(kernel::EntityId::new(kernel::ModelId::new({}).expect(\"generated model\"), {}, {}).expect(\"generated entity\"))",
            name_source(entity.model().as_str()),
            name_source(entity.local().as_str()),
            type_source(entity.typ())
        ),
        Term::Value { typ, canonical } => format!(
            "kernel::Term::value({}, {canonical:?}.into()).expect(\"generated value\")",
            type_source(typ)
        ),
        Term::Variable { id, typ } => format!(
            "kernel::Term::variable({}, {})",
            variable_source(id),
            type_source(typ)
        ),
    }
}
#[cfg(not(clause_generated))]
fn clause_source(value: &Clause) -> String {
    format!(
        "kernel::Clause::new({}, std::collections::BTreeMap::from([{}])).expect(\"generated clause\")",
        relation_source(value.relation()),
        value
            .roles()
            .iter()
            .map(|(role, term)| format!("({}, {})", role_source(role), term_source(term)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
#[path = "generated/tests.rs"]
mod tests;
