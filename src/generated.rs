//! Source-deletion-safe request-program emission.

#![allow(unexpected_cfgs)]

use std::fmt::Write;

use crate::{
    kernel::{KernelError, PatternId, ReferentId, RelationalContent, Result, RoleId, Term},
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
        declaration: "mod clause;",
        source: include_str!("kernel/clause.rs"),
    },
    ChildModule {
        name: "error",
        declaration: "mod error;",
        source: include_str!("kernel/error.rs"),
    },
    ChildModule {
        name: "find",
        declaration: "mod find;",
        source: include_str!("kernel/find.rs"),
    },
    ChildModule {
        name: "identity",
        declaration: "mod identity;",
        source: include_str!("kernel/identity.rs"),
    },
    ChildModule {
        name: "model",
        declaration: "mod model;",
        source: include_str!("kernel/model.rs"),
    },
    ChildModule {
        name: "revision",
        declaration: "mod revision;",
        source: include_str!("kernel/revision.rs"),
    },
    ChildModule {
        name: "schema",
        declaration: "mod schema;",
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
        name: "evaluate",
        declaration: "mod evaluate;",
        source: include_str!("execution/evaluate.rs"),
    },
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

/// Emit a standalone program that reloads the referenced Revisions and
/// invokes the same ordered request evaluator as the interpreter.
#[cfg(not(clause_generated))]
pub fn emit_rust(program: &ResolvedProgram) -> Result<String> {
    let modules = target_neutral_modules()?;
    let mut body = String::new();
    let revisions = revision_order(program)?;
    let indices = revisions
        .iter()
        .enumerate()
        .map(|(index, revision)| (revision.identity().clone(), index))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (index, revision) in revisions.iter().enumerate() {
        let serialized = wire::serialize(revision);
        match revision.predecessor() {
            Some(predecessor) => {
                let base = indices
                    .get(predecessor)
                    .expect("lineage order contains each predecessor");
                writeln!(
                    body,
                    "let r{index} = wire::reload_successor({serialized:?}, &r{base}).expect(\"sealed successor reloads\");"
                )
                .expect("writing generated Revision reloads to a String cannot fail");
            }
            None => {
                writeln!(
                    body,
                    "let r{index} = wire::reload({serialized:?}).expect(\"sealed root reloads\");"
                )
                .expect("writing generated Revision reloads to a String cannot fail");
            }
        }
    }
    writeln!(body, "let program = request::ResolvedProgram::new(std::collections::BTreeMap::from([{}]), vec![{}]).expect(\"generated requests resolve\");", revisions.iter().enumerate().map(|(index, _)| format!("(r{index}.identity().clone(), r{index}.clone())")).collect::<Vec<_>>().join(","), program.requests().iter().map(|request| request_source(request, &indices)).collect::<Vec<_>>().join(",")).expect("writing the generated request registry to a String cannot fail");
    writeln!(body, "print!(\"{{}}\", request::run(&program, request::RunLimits::default()).expect(\"generated requests run\").canonical_bytes());").expect("writing the generated entry point to a String cannot fail");
    Ok(format!("{modules}\nfn main() {{ {body} }}"))
}

/// Emit a standalone program that strictly reloads one root Revision and
/// evaluates the requested definitions in caller-supplied order.
#[cfg(not(clause_generated))]
pub fn emit_evaluation_rust(
    revision: &crate::kernel::Revision,
    definitions: &[ReferentId],
) -> Result<String> {
    if revision.predecessor().is_some() {
        return Err(KernelError::new(
            "generated evaluation requires a root Revision",
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    for definition in definitions {
        if !unique.insert(definition) {
            return Err(KernelError::new(format!(
                "generated evaluation requested duplicate definition '{}'",
                definition.as_str()
            )));
        }
        if revision.model().definition(definition).is_none() {
            return Err(KernelError::new(format!(
                "generated evaluation requested missing definition '{}'",
                definition.as_str()
            )));
        }
    }

    let modules = target_neutral_modules()?;
    let serialized = wire::serialize(revision);
    let requested = definitions
        .iter()
        .map(definition_source)
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "let revision = wire::reload({serialized:?}).expect(\"sealed root reloads\");\n\
         let requested = vec![{requested}];\n\
         let mut results = Vec::with_capacity(requested.len());\n\
         for definition_id in requested {{\n\
             let definition = revision.model().definition(&definition_id).expect(\"validated definition survives strict reload\");\n\
             let result = execution::evaluate(&revision, definition.denotation()).expect(\"sealed pure definition evaluates\");\n\
             results.push((definition_id, result));\n\
         }}\n\
         let output = request::EvaluationOutput::new(revision.identity().clone(), results).expect(\"validated evaluation requests remain unique\");\n\
         print!(\"{{}}\", output.canonical_bytes());"
    );
    Ok(format!("{modules}\nfn main() {{ {body} }}"))
}

#[cfg(not(clause_generated))]
fn target_neutral_modules() -> Result<String> {
    let mut modules = String::new();
    for (name, source, children) in [
        ("kernel", include_str!("kernel.rs"), KERNEL_CHILDREN),
        ("wire", include_str!("wire.rs"), WIRE_CHILDREN),
        ("intrinsic", include_str!("intrinsic.rs"), NO_CHILDREN),
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
    Ok(modules)
}

#[cfg(not(clause_generated))]
fn revision_order(program: &ResolvedProgram) -> Result<Vec<&crate::kernel::Revision>> {
    let mut ordered = Vec::with_capacity(program.revisions().len());
    let mut admitted = std::collections::BTreeSet::new();
    while ordered.len() < program.revisions().len() {
        let before = ordered.len();
        for revision in program.revisions().values() {
            if admitted.contains(revision.identity())
                || revision
                    .predecessor()
                    .is_some_and(|predecessor| !admitted.contains(predecessor))
            {
                continue;
            }
            admitted.insert(revision.identity().clone());
            ordered.push(revision);
        }
        if ordered.len() == before {
            return Err(KernelError::new(
                "resolved Revision registry has incomplete or cyclic lineage",
            ));
        }
    }
    Ok(ordered)
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
fn revision_source(
    identity: &crate::kernel::RevisionId,
    indices: &std::collections::BTreeMap<crate::kernel::RevisionId, usize>,
) -> String {
    let index = indices.get(identity).expect("request revision is embedded");
    format!("r{index}.identity().clone()")
}

#[cfg(not(clause_generated))]
fn request_source(
    request: &Request,
    indices: &std::collections::BTreeMap<crate::kernel::RevisionId, usize>,
) -> String {
    match request {
        Request::Find {
            revision,
            pattern,
            sought,
        } => format!(
            "request::Request::Find {{ revision: {}, pattern: {}, sought: {} }}",
            revision_source(revision, indices),
            clause_source(pattern),
            variable_source(sought)
        ),
        Request::Why {
            revision,
            target,
            all,
        } => format!(
            "request::Request::Why {{ revision: {}, target: {}, all: {all} }}",
            revision_source(revision, indices),
            clause_source(target)
        ),
        Request::Prevent {
            revision,
            target,
            selection,
            using,
        } => format!(
            "request::Request::Prevent {{ revision: {}, target: {}, selection: {}, using: vec![{}] }}",
            revision_source(revision, indices),
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
            revision_source(revision, indices),
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
            revision_source(base, indices),
            revision_source(successor, indices)
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
fn relation_source(value: &ReferentId) -> String {
    format!(
        "kernel::ReferentId::new({:?}.into()).expect(\"generated relation\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn definition_source(value: &ReferentId) -> String {
    format!(
        "kernel::ReferentId::new({:?}.into()).expect(\"generated definition identity\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn role_source(value: &RoleId) -> String {
    format!(
        "kernel::RoleId::new({:?}.into()).expect(\"generated role\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn variable_source(value: &PatternId) -> String {
    format!(
        "kernel::PatternId::new({:?}.into()).expect(\"generated variable\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn term_source(value: &Term) -> String {
    match value {
        Term::Referent(id) => format!(
            "kernel::Term::referent(kernel::ReferentId::new({:?}.into()).expect(\"generated referent\"))",
            id.as_str()
        ),
        Term::Pattern(id) => format!("kernel::Term::pattern({})", variable_source(id)),
        Term::Application(id) => format!(
            "kernel::Term::application(kernel::ContentId::new({:?}.into()).expect(\"generated content\"))",
            id.as_str()
        ),
        Term::F32(value) => format!(
            "kernel::Term::f32_bits({}).expect(\"generated finite F32\")",
            value.bits()
        ),
        Term::Int(value) => format!("kernel::Term::int({value})"),
        Term::Bool(value) => format!("kernel::Term::boolean({value})"),
        Term::Product(fields) => {
            let fields = fields
                .iter()
                .map(|(label, value)| {
                    format!(
                        "(kernel::Name::new({:?}.into()).expect(\"generated product label\"), {})",
                        label.as_str(),
                        term_source(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "kernel::Term::product(std::collections::BTreeMap::from([{fields}])).expect(\"generated structural product\")"
            )
        }
        Term::Sum { tag, value } => format!(
            "kernel::Term::sum(kernel::Name::new({:?}.into()).expect(\"generated sum tag\"), {}).expect(\"generated structural sum\")",
            tag.as_str(),
            term_source(value)
        ),
        Term::Sequence(values) => format!(
            "kernel::Term::sequence(vec![{}]).expect(\"generated structural sequence\")",
            values.iter().map(term_source).collect::<Vec<_>>().join(",")
        ),
    }
}
#[cfg(not(clause_generated))]
fn clause_source(value: &RelationalContent) -> String {
    format!(
        "kernel::RelationalContent::new({}, std::collections::BTreeMap::from([{}])).expect(\"generated clause\")",
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
mod tests;
