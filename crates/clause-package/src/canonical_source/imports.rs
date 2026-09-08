use super::*;

/// Exact declaration sources supplied by the caller, keyed by the source import spelling.
pub type CanonicalSourceImportsV1 = BTreeMap<String, Vec<u8>>;

/// Read explicit imports without consulting the filesystem or an ambient registry.
pub fn canonical_source_imports_v1(
    exact_source: &[u8],
) -> Result<Vec<String>, CanonicalSourceErrorV1> {
    Ok(imports(exact_source)?.into_iter().map(|(name, _)| name).collect())
}

fn imports(exact_source: &[u8]) -> Result<Vec<(String, CanonicalSourceOriginV1)>, CanonicalSourceErrorV1> {
    let source = std::str::from_utf8(exact_source).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
    let artifact = CanonicalSourceArtifactIdV1(domain_hash(SOURCE_ARTIFACT_DOMAIN, &[exact_source]));
    let lines = source_lines(source)?;
    let mut result = Vec::new();
    let mut names = BTreeSet::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(value) = line.text.strip_prefix("import ") else { continue };
        let origin = line_origin(artifact, *line);
        let name = parse_text_literal(value).filter(|name| !name.is_empty())
            .ok_or(CanonicalSourceErrorV1::InvalidImport { origin, reason: "expected a quoted declaration source" })?;
        if lines.iter().skip(index + 1).find(|line| !line.text.trim().is_empty()).is_some_and(|line| line.indent > 0) {
            return Err(CanonicalSourceErrorV1::InvalidImport { origin, reason: "an import has no body" });
        }
        if !names.insert(name.clone()) {
            return Err(CanonicalSourceErrorV1::InvalidImport { origin, reason: "duplicate import" });
        }
        result.push((name, origin));
    }
    Ok(result)
}

#[derive(Default)]
pub(super) struct ImportedDeclarations {
    pub items: Vec<CstItem>,
    pub callables: Vec<callable::CallableCst>,
    pub sources: BTreeMap<CanonicalSourceArtifactIdV1, Box<[u8]>>,
}

pub(super) fn read(
    exact_source: &[u8],
    supplied: &CanonicalSourceImportsV1,
    frontend: &CanonicalDeclaredFrontendV1,
) -> Result<ImportedDeclarations, CanonicalSourceErrorV1> {
    let mut result = ImportedDeclarations::default();
    for (name, import_origin) in imports(exact_source)? {
        let source = supplied.get(&name).ok_or(CanonicalSourceErrorV1::InvalidImport {
            origin: import_origin, reason: "declaration source was not supplied",
        })?;
        if !imports(source)?.is_empty() {
            return Err(CanonicalSourceErrorV1::InvalidImport {
                origin: import_origin, reason: "declaration sources contain foreign declarations only",
            });
        }
        let cst = read_canonical_source_with_declared_frontend_v1(source, frontend)?;
        let source_text = std::str::from_utf8(source).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
        let lines = source_lines(source_text)?;
        let starts = lines.iter().enumerate()
            .filter(|(_, line)| line.indent == 0 && !line.text.trim().is_empty())
            .map(|(index, _)| index).chain(std::iter::once(lines.len())).collect::<Vec<_>>();
        for pair in starts.windows(2) {
            let block = &lines[pair[0]..pair[1]];
            let origin = block_origin(cst.artifact, block);
            if block[0].text.starts_with("foreign ") {
                let (callable, _) = callable::read(block, origin, &cst.items)?
                    .ok_or(CanonicalSourceErrorV1::InvalidImport { origin, reason: "expected a foreign declaration" })?;
                result.callables.push(callable);
            } else if !cst.items.iter().any(|item| item.origin == origin && matches!(item.kind, CstKind::ForeignType { .. }))
                && !cst.callables.iter().any(|callable| callable.origin == origin && callable.exported) {
                return Err(CanonicalSourceErrorV1::InvalidImport {
                    origin, reason: "declaration sources contain exported definitions and foreign declarations only",
                });
            }
        }
        result.sources.insert(cst.artifact, cst.exact_source);
        result.items.extend(cst.items);
    }
    validate_unique_designations(&result.items)?;
    Ok(result)
}
