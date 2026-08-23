//! Source-deletion-safe request-program emission.

#![allow(unexpected_cfgs)]

use std::fmt::Write;

use crate::{
    kernel::{Clause, RelationId, Result, RoleId, Term, TypeId, VariableId},
    request::{Request, ResolvedProgram, Selection},
    wire,
};

/// Emit a standalone program that reloads the referenced v3 Revisions and
/// invokes the same ordered request evaluator as the interpreter.
#[cfg(not(clause_generated))]
pub fn emit_rust(program: &ResolvedProgram) -> Result<String> {
    let mut modules = String::new();
    for module in [
        "kernel",
        "wire",
        "derive",
        "delta",
        "execution",
        "intervention",
        "semantic_diff",
        "request",
    ] {
        let body = match module {
            "kernel" => include_str!("kernel.rs"),
            "wire" => include_str!("wire.rs"),
            "derive" => include_str!("derive.rs"),
            "delta" => include_str!("delta.rs"),
            "execution" => include_str!("execution.rs"),
            "intervention" => include_str!("intervention.rs"),
            "semantic_diff" => include_str!("semantic_diff.rs"),
            "request" => include_str!("request.rs"),
            _ => unreachable!(),
        };
        let body = production_source(body);
        writeln!(modules, "mod {module} {{\n{body}\n}}").unwrap();
    }
    let mut body = String::new();
    for (index, revision) in program.revisions().values().enumerate() {
        writeln!(
            body,
            "let r{index} = wire::reload({:?}).expect(\"sealed revision reloads\");",
            wire::serialize(revision)
        )
        .unwrap();
    }
    writeln!(body, "let program = request::ResolvedProgram::new(std::collections::BTreeMap::from([{}]), vec![{}]).expect(\"generated requests resolve\");", program.revisions().values().enumerate().map(|(index, _)| format!("(r{index}.identity().clone(), r{index}.clone())")).collect::<Vec<_>>().join(","), program.requests().iter().map(|request| request_source(request, program)).collect::<Vec<_>>().join(",")).unwrap();
    writeln!(body, "print!(\"{{}}\", request::run(&program, request::RunLimits::default()).expect(\"generated requests run\").canonical_bytes());").unwrap();
    Ok(format!("{modules}\nfn main() {{ {body} }}"))
}

#[cfg(not(clause_generated))]
fn production_source(source: &str) -> &str {
    source
        .split_once("\n#[cfg(test)]")
        .map_or(source, |(production, _)| production)
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
mod tests {
    use super::emit_rust;
    use crate::{elaborate, frontend, request};
    use std::{fs, process::Command};

    #[test]
    fn embeds_resolved_requests_not_source() {
        let source = "Item: Type\nlink: Relation\n    {left: Item} links {right: Item}\n    mode left -> right: many\ngraph: Model\n    A: Item\n    B: Item\n    A links B\nfind all ?right in graph:\n    A links ?right\n";
        let program =
            request::resolve(&elaborate::compile(frontend::parse(source).unwrap()).unwrap())
                .unwrap();
        let emitted = emit_rust(&program).unwrap();
        assert!(emitted.contains("request::Request::Find"));
        assert!(!emitted.contains("find all ?right"));
    }

    #[test]
    fn generated_program_matches_source_deleted_request_transcript() {
        let source = "Item: Type\nlink: Relation\n    {left: Item} links {right: Item}\n    mode left -> right: many\ngraph: Model\n    A: Item\n    B: Item\n    A links B\ngraph/add: Revision\n    from: graph\n    admit:\n        B links A\nfind all ?right in graph:\n    A links ?right\nwhy all in graph:\n    A links B\ndiff graph -> graph/add\n";
        let program =
            request::resolve(&elaborate::compile(frontend::parse(source).unwrap()).unwrap())
                .unwrap();
        let expected = request::run(&program, request::RunLimits::default())
            .unwrap()
            .canonical_bytes();
        let root =
            std::env::temp_dir().join(format!("clause-request-generated-{}", std::process::id()));
        let rust = root.with_extension("rs");
        let binary = root.with_extension("bin");
        fs::write(&rust, emit_rust(&program).unwrap()).unwrap();
        let compiled = Command::new("rustc")
            .args(["--edition=2024", "--cfg", "clause_generated"])
            .arg(&rust)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let actual = Command::new(&binary).output().unwrap();
        assert!(actual.status.success());
        assert_eq!(actual.stdout, expected.as_bytes());
        fs::remove_file(rust).unwrap();
        fs::remove_file(binary).unwrap();
    }
}
