use crate::kernel::{Clause, KernelError, QueryPlan, Result, Revision};
use std::fmt::Write;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    pub id: String,
    pub relation: String,
    pub roles: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOutput {
    pub results: Vec<String>,
    pub proofs: Vec<Proof>,
}

pub fn execute(revision: &Revision, plan: &QueryPlan) -> Result<QueryOutput> {
    if revision.plan()? != *plan {
        return Err(KernelError::new("query plan does not belong to revision"));
    }
    let sought = match plan.sought() {
        [role] => role,
        _ => return Err(KernelError::new("query output requires exactly one sought role")),
    };
    let query = revision.model().query();
    let requested = query
        .roles()
        .iter()
        .filter(|(_, term)| !term.is_variable())
        .collect::<Vec<_>>();

    let mut rows = revision
        .model()
        .facts()
        .iter()
        .filter(|fact| {
            fact.relation() == query.relation()
                && requested.iter().all(|(role, wanted)| {
                    fact.roles().get(*role).map(|term| term.text()) == Some(wanted.text())
                })
        })
        .map(|fact| row(revision.identity(), fact, sought))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.id.cmp(&right.1.id)));

    Ok(QueryOutput {
        results: rows.iter().map(|(value, _)| value.clone()).collect(),
        proofs: rows.into_iter().map(|(_, proof)| proof).collect(),
    })
}

fn row(revision_id: &str, fact: &Clause, sought: &str) -> Result<(String, Proof)> {
    let result = fact
        .roles()
        .get(sought)
        .ok_or_else(|| KernelError::new("fact does not fill sought role"))?
        .text()
        .to_owned();
    let roles = fact
        .roles()
        .iter()
        .map(|(name, term)| (name.clone(), term.text().to_owned()))
        .collect::<Vec<_>>();
    let identity_roles = roles
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(",");
    Ok((
        result,
        Proof {
            id: format!("proof/{revision_id}/{}/{identity_roles}", fact.relation()),
            relation: fact.relation().to_owned(),
            roles,
        },
    ))
}

pub fn canonical_json(output: &QueryOutput) -> String {
    let results = output
        .results
        .iter()
        .map(|value| quoted(value))
        .collect::<Vec<_>>()
        .join(",");
    let proofs = output
        .proofs
        .iter()
        .map(|proof| {
            let roles = proof
                .roles
                .iter()
                .map(|(name, value)| format!("[{},{}]", quoted(name), quoted(value)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "[\"proof\",{},\"relation\",{},\"roles\",[{roles}]]",
                quoted(&proof.id),
                quoted(&proof.relation)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"clause-query-output-v1\",[\"results\",[{results}]],[\"proofs\",[{proofs}]]]"
    )
}

fn quoted(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value <= '\u{1f}' => write!(escaped, "\\u{:04x}", value as u32).unwrap(),
            value => escaped.push(value),
        }
    }
    format!("\"{escaped}\"")
}

pub fn emit_rust(revision: &Revision, plan: &QueryPlan) -> Result<String> {
    if revision.plan()? != *plan {
        return Err(KernelError::new("query plan does not belong to revision"));
    }
    let sought = match plan.sought() {
        [role] => role,
        _ => return Err(KernelError::new("generated query requires one sought role")),
    };
    let query = revision.model().query();
    let requested = query
        .roles()
        .iter()
        .filter(|(_, term)| !term.is_variable())
        .map(|(role, term)| format!("({role:?}, {:?})", term.text()))
        .collect::<Vec<_>>()
        .join(",");
    let facts = revision
        .model()
        .facts()
        .iter()
        .map(|fact| {
            let roles = fact
                .roles()
                .iter()
                .map(|(role, term)| format!("({role:?}, {:?})", term.text()))
                .collect::<Vec<_>>()
                .join(",");
            format!("Fact {{ relation: {:?}, roles: &[{roles}] }}", fact.relation())
        })
        .collect::<Vec<_>>()
        .join(",");

    let mut source = String::new();
    writeln!(source, "const REVISION: &str = {:?};", revision.identity()).unwrap();
    writeln!(source, "const RELATION: &str = {:?};", query.relation()).unwrap();
    writeln!(source, "const SOUGHT: &str = {sought:?};").unwrap();
    writeln!(source, "const REQUESTED: &[(&str,&str)] = &[{requested}];").unwrap();
    writeln!(source, "const FACTS: &[Fact] = &[{facts}];").unwrap();
    source.push_str(GENERATED_RUNTIME);
    Ok(source)
}

const GENERATED_RUNTIME: &str = r#"
struct Fact { relation: &'static str, roles: &'static [(&'static str, &'static str)] }
fn role<'a>(roles: &'a [(&str, &str)], name: &str) -> Option<&'a str> {
    roles.iter().find(|(candidate, _)| *candidate == name).map(|(_, value)| *value)
}
fn json(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"), '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"), ch if ch <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('\"'); out
}
fn main() {
    let mut rows = FACTS.iter().filter(|fact| {
        fact.relation == RELATION && REQUESTED.iter().all(|(name, wanted)| role(fact.roles, name) == Some(*wanted))
    }).map(|fact| {
        let result = role(fact.roles, SOUGHT).expect("complete admitted fact").to_owned();
        let role_values = fact.roles.iter().map(|(name, value)| format!("{}={}", name, value)).collect::<Vec<_>>().join(",");
        let proof_id = format!("proof/{}/{}/{}", REVISION, fact.relation, role_values);
        let roles = fact.roles.iter().map(|(name, value)| format!("[{},{}]", json(name), json(value))).collect::<Vec<_>>().join(",");
        let proof = format!("[\"proof\",{},\"relation\",{},\"roles\",[{}]]", json(&proof_id), json(fact.relation), roles);
        (result, proof)
    }).collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let results = rows.iter().map(|(value, _)| json(value)).collect::<Vec<_>>().join(",");
    let proofs = rows.iter().map(|(_, proof)| proof.as_str()).collect::<Vec<_>>().join(",");
    print!("[\"clause-query-output-v1\",[\"results\",[{}]],[\"proofs\",[{}]]]", results, proofs);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{Cardinality, Clause, Mode, Model, Relation, Role, Sentence, Term};
    use std::fs;
    use std::process::Command;

    #[test]
    fn non_catalog_plan_interprets_and_generates_identically() {
        let relation = Relation::new(
            "inventory/stores",
            vec![Role::new("container", "Text").unwrap(), Role::new("item", "Text").unwrap()],
            Sentence::new("container", "stores", "item").unwrap(),
            vec![Mode::finite(vec!["container".into()], vec!["item".into()], Cardinality::Many).unwrap()],
        ).unwrap();
        let fact = |item: &str| Clause::new("inventory/stores", vec![
            ("container".into(), Term::literal("shelf-7").unwrap()),
            ("item".into(), Term::literal(item).unwrap()),
        ]).unwrap();
        let query = Clause::new("inventory/stores", vec![
            ("container".into(), Term::literal("shelf-7").unwrap()),
            ("item".into(), Term::variable("found").unwrap()),
        ]).unwrap();
        let revision = Revision::admit(Model::new(vec![relation], vec![fact("widget"), fact("gadget")], query, "ascending").unwrap());
        let plan = revision.plan().unwrap();
        let interpreted = canonical_json(&execute(&revision, &plan).unwrap());
        assert!(interpreted.contains("[\"results\",[\"gadget\",\"widget\"]]"));
        let generated = emit_rust(&revision, &plan).unwrap();
        assert!(!generated.contains("catalog/contains"));
        assert!(!generated.contains("letters"));

        let root = std::env::temp_dir().join(format!("clause-generic-execution-{}", std::process::id()));
        let source = root.with_extension("rs");
        let binary = root.with_extension("bin");
        fs::write(&source, generated).unwrap();
        let compile = Command::new("rustc").arg("--edition=2024").arg(&source).arg("-o").arg(&binary).output().unwrap();
        assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
        let output = Command::new(&binary).output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), interpreted);
        let _ = fs::remove_file(source);
        let _ = fs::remove_file(binary);
    }
}
