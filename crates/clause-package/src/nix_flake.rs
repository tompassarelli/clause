use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{CanonicalScalarValueV1, CanonicalSourceErrorV1, read_canonical_source_v1};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NixIdentifierV1(String);

impl NixIdentifierV1 {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NixInputV1 {
    pub name: NixIdentifierV1,
    pub source: String,
    pub follows: Option<NixIdentifierV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NixSystemV1(NixIdentifierV1);

impl NixSystemV1 {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NixPackageV1(NixIdentifierV1);

impl NixPackageV1 {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NixToolchainV1 {
    Rust { manifest: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NixDevelopmentIncludeV1 {
    Package(NixPackageV1),
    Toolchain(NixToolchainV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NixDevelopmentShellV1 {
    pub name: NixIdentifierV1,
    pub system: NixSystemV1,
    pub imports: NixIdentifierV1,
    pub overlays: Vec<NixIdentifierV1>,
    pub includes: Vec<NixDevelopmentIncludeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NixFlakeV1 {
    pub name: NixIdentifierV1,
    pub description: String,
    pub inputs: Vec<NixInputV1>,
    pub development_shell: NixDevelopmentShellV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NixFlakeProjectionErrorV1 {
    CanonicalSource(CanonicalSourceErrorV1),
    InvalidLine { line: usize, reason: &'static str },
    MissingVocabulary,
    DuplicateVocabulary,
    UnknownVocabulary(String),
    MissingFlake,
    DuplicateFlake,
    MissingDescription,
    ConflictingDescription,
    MissingDevelopmentShell,
    ConflictingDevelopmentShell,
    MissingShell(String),
    UnreferencedSubject(String),
    MissingInputSource(String),
    ConflictingInputSource(String),
    ConflictingInputFollow(String),
    UndeclaredInput(String),
    InputFollowCycle(Vec<String>),
    MissingShellSystem(String),
    ConflictingShellSystem(String),
    MissingShellImport(String),
    ConflictingShellImport(String),
    DuplicateShellOverlay(String),
    DuplicateShellInclude(String),
    UnsupportedToolchain(String),
    MissingToolchainManifest(String),
    ConflictingToolchainManifest(String),
    InvalidToolchainManifest(String),
}

impl fmt::Display for NixFlakeProjectionErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalSource(error) => write!(formatter, "canonical source rejected: {error}"),
            Self::InvalidLine { line, reason } => {
                write!(
                    formatter,
                    "invalid Nix vocabulary relation on line {line}: {reason}"
                )
            }
            Self::MissingVocabulary => formatter.write_str("source must declare `using Nix`"),
            Self::DuplicateVocabulary => {
                formatter.write_str("source declares `using Nix` more than once")
            }
            Self::UnknownVocabulary(name) => write!(formatter, "unknown vocabulary `{name}`"),
            Self::MissingFlake => formatter.write_str("Nix vocabulary requires one Flake subject"),
            Self::DuplicateFlake => formatter.write_str("Nix vocabulary permits one Flake subject"),
            Self::MissingDescription => formatter.write_str("Flake has no description relation"),
            Self::ConflictingDescription => {
                formatter.write_str("Flake has conflicting description relations")
            }
            Self::MissingDevelopmentShell => {
                formatter.write_str("Flake has no development shell relation")
            }
            Self::ConflictingDevelopmentShell => {
                formatter.write_str("Flake has conflicting development shell relations")
            }
            Self::MissingShell(name) => {
                write!(formatter, "development shell `{name}` has no subject")
            }
            Self::UnreferencedSubject(name) => {
                write!(
                    formatter,
                    "subject `{name}` is not the selected development shell"
                )
            }
            Self::MissingInputSource(name) => write!(formatter, "input `{name}` has no source"),
            Self::ConflictingInputSource(name) => {
                write!(formatter, "input `{name}` has conflicting sources")
            }
            Self::ConflictingInputFollow(name) => {
                write!(
                    formatter,
                    "input `{name}` has conflicting follows relations"
                )
            }
            Self::UndeclaredInput(name) => write!(formatter, "input `{name}` is not declared"),
            Self::InputFollowCycle(cycle) => {
                write!(formatter, "input follows cycle: {}", cycle.join(" -> "))
            }
            Self::MissingShellSystem(name) => {
                write!(formatter, "development shell `{name}` has no system")
            }
            Self::ConflictingShellSystem(name) => {
                write!(
                    formatter,
                    "development shell `{name}` has conflicting systems"
                )
            }
            Self::MissingShellImport(name) => {
                write!(formatter, "development shell `{name}` imports no input")
            }
            Self::ConflictingShellImport(name) => {
                write!(
                    formatter,
                    "development shell `{name}` has conflicting imports"
                )
            }
            Self::DuplicateShellOverlay(name) => {
                write!(formatter, "development shell repeats overlay `{name}`")
            }
            Self::DuplicateShellInclude(name) => {
                write!(formatter, "development shell repeats include `{name}`")
            }
            Self::UnsupportedToolchain(name) => {
                write!(formatter, "`{name}` is not a Nix vocabulary toolchain")
            }
            Self::MissingToolchainManifest(name) => {
                write!(formatter, "toolchain `{name}` has no manifest")
            }
            Self::ConflictingToolchainManifest(name) => {
                write!(formatter, "toolchain `{name}` has conflicting manifests")
            }
            Self::InvalidToolchainManifest(path) => {
                write!(
                    formatter,
                    "toolchain manifest `{path}` is not a relative Nix path"
                )
            }
        }
    }
}

impl std::error::Error for NixFlakeProjectionErrorV1 {}

impl From<CanonicalSourceErrorV1> for NixFlakeProjectionErrorV1 {
    fn from(error: CanonicalSourceErrorV1) -> Self {
        Self::CanonicalSource(error)
    }
}

#[derive(Clone, Debug)]
struct SourceFact {
    line: usize,
    role: String,
    object: CanonicalScalarValueV1,
}

#[derive(Clone, Debug)]
struct SourceSubject {
    name: NixIdentifierV1,
    shape: Option<NixIdentifierV1>,
    shape_line: Option<usize>,
    facts: Vec<SourceFact>,
}

/// Check canonical Clause source against the exact compiler-owned `Nix`
/// vocabulary and project its relational facts into a typed flake value.
pub fn project_nix_flake_v1(exact_source: &[u8]) -> Result<NixFlakeV1, NixFlakeProjectionErrorV1> {
    let cst = read_canonical_source_v1(exact_source)?;
    let vocabulary = cst
        .vocabularies()
        .iter()
        .map(|vocabulary| {
            std::str::from_utf8(&vocabulary.designation)
                .map(str::to_owned)
                .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8.into())
        })
        .collect::<Result<Vec<String>, NixFlakeProjectionErrorV1>>()?;
    let mut subjects = Vec::<SourceSubject>::new();
    let mut subject_positions = BTreeMap::<NixIdentifierV1, usize>::new();
    for application in cst.applications() {
        let line = source_line(exact_source, application.origin.start);
        let name = identifier(
            std::str::from_utf8(&application.subject)
                .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
            line,
        )?;
        let role = std::str::from_utf8(&application.role)
            .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
        let position = if let Some(position) = subject_positions.get(&name) {
            *position
        } else {
            let position = subjects.len();
            subject_positions.insert(name.clone(), position);
            subjects.push(SourceSubject {
                name,
                shape: None,
                shape_line: None,
                facts: Vec::new(),
            });
            position
        };
        let subject = &mut subjects[position];
        if role == "shape" {
            let CanonicalScalarValueV1::Symbol(shape) = &application.object else {
                return Err(invalid(line, "shape expects one designation"));
            };
            let shape = identifier(
                std::str::from_utf8(shape).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                line,
            )?;
            if subject.shape.replace(shape).is_some() {
                return Err(invalid(line, "subject repeats its shape application"));
            }
            subject.shape_line = Some(line);
        } else {
            subject.facts.push(SourceFact {
                line,
                role: role.to_owned(),
                object: application.object.clone(),
            });
        }
    }
    match vocabulary.as_slice() {
        [] => return Err(NixFlakeProjectionErrorV1::MissingVocabulary),
        [name] if name == "Nix" => {}
        [_] => {
            return Err(NixFlakeProjectionErrorV1::UnknownVocabulary(
                vocabulary[0].clone(),
            ));
        }
        _ => return Err(NixFlakeProjectionErrorV1::DuplicateVocabulary),
    }

    let flakes = subjects
        .iter()
        .filter(|subject| {
            subject
                .shape
                .as_ref()
                .is_some_and(|kind| kind.as_str() == "Flake")
        })
        .collect::<Vec<_>>();
    let flake = match flakes.as_slice() {
        [] => return Err(NixFlakeProjectionErrorV1::MissingFlake),
        [flake] => *flake,
        _ => return Err(NixFlakeProjectionErrorV1::DuplicateFlake),
    };

    let mut descriptions = Vec::new();
    let mut shell_names = Vec::new();
    let mut input_order = Vec::new();
    let mut declared_input_names = BTreeSet::new();
    for fact in &flake.facts {
        match fact.role.as_str() {
            "description" => descriptions.push(fact_text(fact)?),
            "inputs" => {
                let name = fact_identifier(fact)?;
                if !declared_input_names.insert(name.clone()) {
                    return Err(invalid(fact.line, "Flake repeats an input"));
                }
                input_order.push(name);
            }
            "development shell" => shell_names.push(fact_identifier(fact)?),
            _ => {
                return Err(invalid(
                    fact.line,
                    "role is not in the Nix Flake vocabulary",
                ));
            }
        }
    }
    let description = one(
        descriptions,
        NixFlakeProjectionErrorV1::MissingDescription,
        NixFlakeProjectionErrorV1::ConflictingDescription,
    )?;
    let shell_name = one(
        shell_names,
        NixFlakeProjectionErrorV1::MissingDevelopmentShell,
        NixFlakeProjectionErrorV1::ConflictingDevelopmentShell,
    )?;

    let mut checked_inputs = Vec::with_capacity(input_order.len());
    for name in input_order {
        let mut sources = Vec::new();
        let mut follows = Vec::new();
        if let Some(input) = subjects.iter().find(|subject| subject.name == name) {
            if let Some(line) = input.shape_line {
                return Err(invalid(
                    line,
                    "only the Flake subject may use the shape role",
                ));
            }
            for fact in &input.facts {
                match fact.role.as_str() {
                    "from" => sources.push(fact_text(fact)?),
                    "follows" => follows.push(fact_identifier(fact)?),
                    _ => {
                        return Err(invalid(
                            fact.line,
                            "role is not in the Nix input vocabulary",
                        ));
                    }
                }
            }
        }
        let source = one(
            sources,
            NixFlakeProjectionErrorV1::MissingInputSource(name.0.clone()),
            NixFlakeProjectionErrorV1::ConflictingInputSource(name.0.clone()),
        )?;
        let follows = at_most_one(
            follows,
            NixFlakeProjectionErrorV1::ConflictingInputFollow(name.0.clone()),
        )?;
        checked_inputs.push(NixInputV1 {
            name,
            source,
            follows,
        });
    }
    let declared_inputs = declared_input_names;
    for input in &checked_inputs {
        if let Some(follows) = &input.follows
            && !declared_inputs.contains(follows)
        {
            return Err(NixFlakeProjectionErrorV1::UndeclaredInput(
                follows.0.clone(),
            ));
        }
    }
    reject_follow_cycles(&checked_inputs)?;

    let shell_subjects = subjects
        .iter()
        .filter(|subject| subject.name == shell_name)
        .collect::<Vec<_>>();
    let shell_subject = match shell_subjects.as_slice() {
        [shell] => *shell,
        _ => {
            return Err(NixFlakeProjectionErrorV1::MissingShell(
                shell_name.0.clone(),
            ));
        }
    };
    if let Some(line) = shell_subject.shape_line {
        return Err(invalid(
            line,
            "only the Flake subject may use the shape role",
        ));
    }
    let mut systems = Vec::new();
    let mut imports = Vec::new();
    let mut overlays = Vec::new();
    let mut includes = Vec::new();
    let mut included_names = BTreeSet::new();
    let mut included_order = Vec::new();
    for fact in &shell_subject.facts {
        match fact.role.as_str() {
            "system" => systems.push(fact_identifier(fact)?),
            "imports" => imports.push(fact_identifier(fact)?),
            "overlays" => {
                let overlay = fact_identifier(fact)?;
                if overlays.contains(&overlay) {
                    return Err(NixFlakeProjectionErrorV1::DuplicateShellOverlay(overlay.0));
                }
                overlays.push(overlay);
            }
            "includes" => {
                let name = fact_identifier(fact)?;
                if !included_names.insert(name.clone()) {
                    return Err(NixFlakeProjectionErrorV1::DuplicateShellInclude(name.0));
                }
                included_order.push(name);
            }
            _ => {
                return Err(invalid(
                    fact.line,
                    "role is not in the Nix DevelopmentShell vocabulary",
                ));
            }
        }
    }

    for name in &included_order {
        let Some(include_subject) = subjects.iter().find(|subject| subject.name == *name) else {
            includes.push(NixDevelopmentIncludeV1::Package(NixPackageV1(name.clone())));
            continue;
        };
        if let Some(line) = include_subject.shape_line {
            return Err(invalid(
                line,
                "only the Flake subject may use the shape role",
            ));
        }
        if name.as_str() != "rust" {
            return Err(NixFlakeProjectionErrorV1::UnsupportedToolchain(
                name.0.clone(),
            ));
        }
        let mut manifests = Vec::new();
        for fact in &include_subject.facts {
            if fact.role != "from" {
                return Err(invalid(
                    fact.line,
                    "role is not in the Nix toolchain vocabulary",
                ));
            }
            manifests.push(fact_text(fact)?);
        }
        let manifest = one(
            manifests,
            NixFlakeProjectionErrorV1::MissingToolchainManifest(name.0.clone()),
            NixFlakeProjectionErrorV1::ConflictingToolchainManifest(name.0.clone()),
        )?;
        if !valid_relative_nix_path(&manifest) {
            return Err(NixFlakeProjectionErrorV1::InvalidToolchainManifest(
                manifest,
            ));
        }
        includes.push(NixDevelopmentIncludeV1::Toolchain(NixToolchainV1::Rust {
            manifest,
        }));
    }
    let system = one(
        systems,
        NixFlakeProjectionErrorV1::MissingShellSystem(shell_name.0.clone()),
        NixFlakeProjectionErrorV1::ConflictingShellSystem(shell_name.0.clone()),
    )?;
    let import = one(
        imports,
        NixFlakeProjectionErrorV1::MissingShellImport(shell_name.0.clone()),
        NixFlakeProjectionErrorV1::ConflictingShellImport(shell_name.0.clone()),
    )?;
    for input in std::iter::once(&import).chain(overlays.iter()) {
        if !declared_inputs.contains(input) {
            return Err(NixFlakeProjectionErrorV1::UndeclaredInput(input.0.clone()));
        }
    }

    let referenced_subjects = std::iter::once(&flake.name)
        .chain(std::iter::once(&shell_name))
        .chain(declared_inputs.iter())
        .chain(included_names.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    for subject in &subjects {
        if !referenced_subjects.contains(&subject.name) {
            return Err(NixFlakeProjectionErrorV1::UnreferencedSubject(
                subject.name.0.clone(),
            ));
        }
    }

    Ok(NixFlakeV1 {
        name: flake.name.clone(),
        description,
        inputs: checked_inputs,
        development_shell: NixDevelopmentShellV1 {
            name: shell_name,
            system: NixSystemV1(system),
            imports: import,
            overlays,
            includes,
        },
    })
}

/// Render one checked relational flake projection as deterministic Nix.
#[must_use]
pub fn render_nix_flake_v1(flake: &NixFlakeV1) -> String {
    let shell = &flake.development_shell;
    let mut output = String::new();
    output.push_str("{\n  description = \"");
    output.push_str(&escape_nix_string(&flake.description));
    output.push_str("\";\n\n  inputs = {\n");
    for input in &flake.inputs {
        if let Some(follows) = &input.follows {
            output.push_str("    ");
            output.push_str(input.name.as_str());
            output.push_str(" = {\n      url = \"");
            output.push_str(&escape_nix_string(&input.source));
            output.push_str("\";\n      inputs.");
            output.push_str(follows.as_str());
            output.push_str(".follows = \"");
            output.push_str(follows.as_str());
            output.push_str("\";\n    };\n");
        } else {
            output.push_str("    ");
            output.push_str(input.name.as_str());
            output.push_str(".url = \"");
            output.push_str(&escape_nix_string(&input.source));
            output.push_str("\";\n");
        }
    }
    output.push_str("  };\n\n  outputs = { ");
    output.push_str(
        &flake
            .inputs
            .iter()
            .map(|input| input.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    output.push_str(", ... }:\n    let\n      system = \"");
    output.push_str(shell.system.0.as_str());
    output.push_str("\";\n      pkgs = import ");
    output.push_str(shell.imports.as_str());
    output.push_str(" {\n        inherit system;\n        overlays = [");
    for overlay in &shell.overlays {
        output.push(' ');
        output.push_str(overlay.as_str());
        output.push_str(".overlays.default");
    }
    output.push_str(" ];\n      };\n");
    for include in &shell.includes {
        if let NixDevelopmentIncludeV1::Toolchain(NixToolchainV1::Rust { manifest }) = include {
            output.push_str("      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ");
            output.push_str(manifest);
            output.push_str(";\n");
        }
    }
    output.push_str(
        "    in {\n      devShells.${system}.default = pkgs.mkShell {\n        packages = [",
    );
    for include in &shell.includes {
        output.push(' ');
        match include {
            NixDevelopmentIncludeV1::Package(package) => {
                output.push_str("pkgs.");
                output.push_str(package.0.as_str());
            }
            NixDevelopmentIncludeV1::Toolchain(NixToolchainV1::Rust { .. }) => {
                output.push_str("rustToolchain");
            }
        }
    }
    output.push_str(" ];\n      };\n    };\n}\n");
    output
}

fn source_line(source: &[u8], offset: u64) -> usize {
    let offset = usize::try_from(offset)
        .unwrap_or(source.len())
        .min(source.len());
    source[..offset]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

fn identifier(source: &str, line: usize) -> Result<NixIdentifierV1, NixFlakeProjectionErrorV1> {
    let bytes = source.as_bytes();
    let valid = bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && bytes
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'));
    if !valid {
        return Err(invalid(line, "expected one designation"));
    }
    Ok(NixIdentifierV1(source.to_owned()))
}

fn fact_identifier(fact: &SourceFact) -> Result<NixIdentifierV1, NixFlakeProjectionErrorV1> {
    let CanonicalScalarValueV1::Symbol(value) = &fact.object else {
        return Err(invalid(fact.line, "expected one designation"));
    };
    identifier(
        std::str::from_utf8(value).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
        fact.line,
    )
}

fn fact_text(fact: &SourceFact) -> Result<String, NixFlakeProjectionErrorV1> {
    let CanonicalScalarValueV1::Text(value) = &fact.object else {
        return Err(invalid(fact.line, "expected one Text value"));
    };
    Ok(value.clone())
}

fn one<T>(
    values: Vec<T>,
    missing: NixFlakeProjectionErrorV1,
    conflicting: NixFlakeProjectionErrorV1,
) -> Result<T, NixFlakeProjectionErrorV1> {
    match values.len() {
        0 => Err(missing),
        1 => Ok(values.into_iter().next().expect("one value exists")),
        _ => Err(conflicting),
    }
}

fn at_most_one<T>(
    values: Vec<T>,
    conflicting: NixFlakeProjectionErrorV1,
) -> Result<Option<T>, NixFlakeProjectionErrorV1> {
    match values.len() {
        0 => Ok(None),
        1 => Ok(values.into_iter().next()),
        _ => Err(conflicting),
    }
}

fn reject_follow_cycles(inputs: &[NixInputV1]) -> Result<(), NixFlakeProjectionErrorV1> {
    let follows = inputs
        .iter()
        .filter_map(|input| input.follows.as_ref().map(|next| (&input.name, next)))
        .collect::<BTreeMap<_, _>>();
    for input in inputs {
        let mut positions = BTreeMap::<&NixIdentifierV1, usize>::new();
        let mut path = Vec::new();
        let mut current = &input.name;
        loop {
            if let Some(start) = positions.insert(current, path.len()) {
                let mut cycle = path[start..]
                    .iter()
                    .map(|name: &&NixIdentifierV1| name.0.clone())
                    .collect::<Vec<_>>();
                cycle.push(current.0.clone());
                return Err(NixFlakeProjectionErrorV1::InputFollowCycle(cycle));
            }
            path.push(current);
            let Some(next) = follows.get(current).copied() else {
                break;
            };
            current = next;
        }
    }
    Ok(())
}

fn valid_relative_nix_path(path: &str) -> bool {
    path.starts_with("./")
        && path.len() > 2
        && path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
}

fn escape_nix_string(value: &str) -> String {
    let mut escaped = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '$' if characters.peek() == Some(&'{') => escaped.push_str("\\$"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character => escaped.push(character),
        }
    }
    escaped
}

fn invalid(line: usize, reason: &'static str) -> NixFlakeProjectionErrorV1 {
    NixFlakeProjectionErrorV1::InvalidLine { line, reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/authoring/relational-nix-flake.clause"
    ));

    const OUTPUT: &str = r#"{
  description = "North-v2 development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = [ rustToolchain pkgs.bun ];
      };
    };
}
"#;

    #[test]
    fn relational_flake_projects_deterministically() {
        let flake = project_nix_flake_v1(SOURCE.as_bytes()).expect("relational source checks");
        assert_eq!(render_nix_flake_v1(&flake), OUTPUT);
    }

    #[test]
    fn undeclared_references_reject() {
        let source = SOURCE.replace("imports: nixpkgs", "imports: missing");
        assert_eq!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::UndeclaredInput("missing".into()))
        );
    }

    #[test]
    fn input_follow_cycles_reject() {
        let source = SOURCE.replace(
            "    nixpkgs\n      from: \"github:NixOS/nixpkgs/nixos-unstable\"",
            "    nixpkgs\n      from: \"github:NixOS/nixpkgs/nixos-unstable\"\n      follows: rust-overlay",
        );
        assert!(matches!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::InputFollowCycle(_))
        ));
    }

    #[test]
    fn undeclared_overlays_reject() {
        let source = SOURCE.replace(
            "      overlays\n        rust-overlay",
            "      overlays\n        missing",
        );
        assert_eq!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::UndeclaredInput("missing".into()))
        );
    }

    #[test]
    fn conflicting_shell_facts_reject() {
        let source = SOURCE.replace(
            "      system: x86_64-linux",
            "      system: x86_64-linux\n      system: aarch64-linux",
        );
        assert_eq!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::ConflictingShellSystem(
                "north-shell".into()
            ))
        );
    }
}
