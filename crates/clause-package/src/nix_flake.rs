use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{CanonicalSourceErrorV1, read_canonical_source_v1};

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
    text: String,
}

#[derive(Clone, Debug)]
struct SourceSubject {
    name: NixIdentifierV1,
    membership: Option<NixIdentifierV1>,
    facts: Vec<SourceFact>,
}

#[derive(Default)]
struct RawInput {
    source: Vec<String>,
    follows: Vec<NixIdentifierV1>,
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
    let subjects = cst
        .subject_focuses()
        .iter()
        .map(|focus| {
            let line = source_line(exact_source, focus.origin.start);
            let name = identifier(
                std::str::from_utf8(&focus.subject)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                line,
            )?;
            let membership = match focus.memberships.as_slice() {
                [] => None,
                [membership] => Some(identifier(
                    std::str::from_utf8(membership)
                        .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                    line,
                )?),
                _ => return Err(invalid(line, "Nix subjects have at most one membership")),
            };
            let facts = focus
                .edges
                .iter()
                .map(|edge| {
                    Ok(SourceFact {
                        line: source_line(exact_source, edge.origin.start),
                        text: std::str::from_utf8(&edge.source)
                            .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?
                            .to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, NixFlakeProjectionErrorV1>>()?;
            Ok(SourceSubject {
                name,
                membership,
                facts,
            })
        })
        .collect::<Result<Vec<_>, NixFlakeProjectionErrorV1>>()?;
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
                .membership
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
    let mut inputs = BTreeMap::<NixIdentifierV1, RawInput>::new();
    for fact in &flake.facts {
        if let Some(value) = fact.text.strip_prefix("description ") {
            descriptions.push(parse_quoted(value, fact.line)?);
        } else if let Some(rest) = fact.text.strip_prefix("input ") {
            if let Some((name, source)) = rest.split_once(" from ") {
                let name = identifier(name, fact.line)?;
                let source = parse_quoted(source, fact.line)?;
                if !inputs.contains_key(&name) {
                    input_order.push(name.clone());
                }
                inputs.entry(name).or_default().source.push(source);
            } else if let Some((name, followed)) = rest.split_once(" follows ") {
                let name = identifier(name, fact.line)?;
                let followed = identifier(followed, fact.line)?;
                if !inputs.contains_key(&name) {
                    input_order.push(name.clone());
                }
                inputs.entry(name).or_default().follows.push(followed);
            } else {
                return Err(invalid(
                    fact.line,
                    "expected `input NAME from TEXT` or `input NAME follows NAME`",
                ));
            }
        } else if let Some(name) = fact.text.strip_prefix("development shell ") {
            shell_names.push(identifier(name, fact.line)?);
        } else {
            return Err(invalid(
                fact.line,
                "relation is not in the Nix Flake vocabulary",
            ));
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
        let raw = inputs
            .get(&name)
            .expect("input order is built with the input map");
        let source = one(
            raw.source.clone(),
            NixFlakeProjectionErrorV1::MissingInputSource(name.0.clone()),
            NixFlakeProjectionErrorV1::ConflictingInputSource(name.0.clone()),
        )?;
        let follows = at_most_one(
            raw.follows.clone(),
            NixFlakeProjectionErrorV1::ConflictingInputFollow(name.0.clone()),
        )?;
        checked_inputs.push(NixInputV1 {
            name,
            source,
            follows,
        });
    }
    let declared_inputs = checked_inputs
        .iter()
        .map(|input| input.name.clone())
        .collect::<BTreeSet<_>>();
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
    for subject in &subjects {
        if subject.name != flake.name && subject.name != shell_name {
            return Err(NixFlakeProjectionErrorV1::UnreferencedSubject(
                subject.name.0.clone(),
            ));
        }
    }

    let mut systems = Vec::new();
    let mut imports = Vec::new();
    let mut overlays = Vec::new();
    let mut includes = Vec::new();
    let mut included_names = BTreeSet::new();
    for fact in &shell_subject.facts {
        if let Some(system) = fact.text.strip_prefix("for ") {
            systems.push(identifier(system, fact.line)?);
        } else if let Some(import) = fact.text.strip_prefix("imports ") {
            imports.push(identifier(import, fact.line)?);
        } else if let Some(overlay) = fact.text.strip_prefix("overlays ") {
            let overlay = identifier(overlay, fact.line)?;
            if overlays.contains(&overlay) {
                return Err(NixFlakeProjectionErrorV1::DuplicateShellOverlay(overlay.0));
            }
            overlays.push(overlay);
        } else if let Some(include) = fact.text.strip_prefix("includes ") {
            if let Some((name, manifest)) = include.split_once(" from ") {
                let name = identifier(name, fact.line)?;
                if name.as_str() != "rust" {
                    return Err(NixFlakeProjectionErrorV1::UnsupportedToolchain(name.0));
                }
                let manifest = parse_quoted(manifest, fact.line)?;
                if !valid_relative_nix_path(&manifest) {
                    return Err(NixFlakeProjectionErrorV1::InvalidToolchainManifest(
                        manifest,
                    ));
                }
                if !included_names.insert(name.clone()) {
                    return Err(NixFlakeProjectionErrorV1::DuplicateShellInclude(name.0));
                }
                includes.push(NixDevelopmentIncludeV1::Toolchain(NixToolchainV1::Rust {
                    manifest,
                }));
            } else {
                let name = identifier(include, fact.line)?;
                if !included_names.insert(name.clone()) {
                    return Err(NixFlakeProjectionErrorV1::DuplicateShellInclude(name.0));
                }
                includes.push(NixDevelopmentIncludeV1::Package(NixPackageV1(name)));
            }
        } else {
            return Err(invalid(
                fact.line,
                "relation is not in the Nix DevelopmentShell vocabulary",
            ));
        }
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

fn parse_quoted(source: &str, line: usize) -> Result<String, NixFlakeProjectionErrorV1> {
    let inner = source
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| invalid(line, "expected one Text value"))?;
    let mut characters = inner.chars();
    let mut value = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' | '\n' | '\r' => return Err(invalid(line, "invalid single-line Text value")),
            '\\' => match characters.next() {
                Some('"') => value.push('"'),
                Some('\\') => value.push('\\'),
                Some('n') => value.push('\n'),
                Some('r') => value.push('\r'),
                Some('t') => value.push('\t'),
                _ => return Err(invalid(line, "invalid Text escape")),
            },
            character if character.is_control() => {
                return Err(invalid(line, "invalid control character in Text"));
            }
            character => value.push(character),
        }
    }
    Ok(value)
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

    const SOURCE: &str = r#"using Nix

north ∈ Flake
  description "North-v2 development environment"
  input nixpkgs from "github:NixOS/nixpkgs/nixos-unstable"
  input rust-overlay from "github:oxalica/rust-overlay"
  input rust-overlay follows nixpkgs
  development shell north-shell

north-shell
  for x86_64-linux
  imports nixpkgs
  overlays rust-overlay
  includes rust from "./rust-toolchain.toml"
  includes bun
"#;

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
        let source = SOURCE.replace("imports nixpkgs", "imports missing");
        assert_eq!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::UndeclaredInput("missing".into()))
        );
    }

    #[test]
    fn input_follow_cycles_reject() {
        let source = SOURCE.replace(
            "input rust-overlay follows nixpkgs",
            "input rust-overlay follows nixpkgs\n  input nixpkgs follows rust-overlay",
        );
        assert!(matches!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::InputFollowCycle(_))
        ));
    }

    #[test]
    fn undeclared_overlays_reject() {
        let source = SOURCE.replace("overlays rust-overlay", "overlays missing");
        assert_eq!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::UndeclaredInput("missing".into()))
        );
    }

    #[test]
    fn conflicting_shell_facts_reject() {
        let source = SOURCE.replace("for x86_64-linux", "for x86_64-linux\n  for aarch64-linux");
        assert_eq!(
            project_nix_flake_v1(source.as_bytes()),
            Err(NixFlakeProjectionErrorV1::ConflictingShellSystem(
                "north-shell".into()
            ))
        );
    }
}
