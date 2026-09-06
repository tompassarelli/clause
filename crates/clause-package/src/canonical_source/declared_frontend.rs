//! The first executable declared-frontend slice.
//!
//! The Clause source beside this module owns the focused-edge Readings. This
//! Rust module is the generic bootstrap that evaluates those Readings against
//! lossless line slices and projects their explicit role bindings into the
//! resident checker's application carrier. It contains no production-name or
//! source-keyword dispatch.

use super::*;

pub const DECLARED_FOCUSED_FRONTEND_SOURCE_V1: &[u8] = include_bytes!("focused_frontend.clause");

#[derive(Clone, Debug)]
pub struct CanonicalDeclaredFrontendV1 {
    exact_source: Box<[u8]>,
    readings: Vec<DeclaredReading>,
}

#[derive(Clone, Debug)]
struct DeclaredReading {
    pattern: Vec<RelationReadingPartCst>,
    relation: Vec<u8>,
    object: Vec<u8>,
}

#[derive(Clone, Debug)]
struct DeclaredMatch {
    reading: usize,
    bindings: BTreeMap<Vec<u8>, String>,
    focused: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct InputToken {
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Clone, Copy)]
enum DeclaredSurface {
    Edge,
    Focused,
    Prefix,
}

impl CanonicalDeclaredFrontendV1 {
    pub fn read(exact_source: &[u8]) -> Result<Self, CanonicalSourceErrorV1> {
        let source =
            std::str::from_utf8(exact_source).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
        let artifact =
            CanonicalSourceArtifactIdV1(domain_hash(SOURCE_ARTIFACT_DOMAIN, &[exact_source]));
        let lines = source_lines(source)?;
        let starts = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.indent == 0 && !line.text.trim().is_empty())
            .map(|(index, _)| index)
            .chain(std::iter::once(lines.len()))
            .collect::<Vec<_>>();
        let blocks = starts
            .windows(2)
            .map(|pair| &lines[pair[0]..pair[1]])
            .collect::<Vec<_>>();
        let mut readings = Vec::new();
        for item in parse_declarations(artifact, &blocks, &Self::bootstrap())? {
            let CstKind::Relation(relation) = item.kind else {
                continue;
            };
            let origin = item.origin;
            let Some(relation_role) = relation.subject.clone() else {
                return Err(CanonicalSourceErrorV1::InvalidDeclaredFrontend { origin });
            };
            let object_roles = relation
                .roles
                .iter()
                .filter(|role| role.name != relation_role)
                .collect::<Vec<_>>();
            let [object_role] = object_roles.as_slice() else {
                return Err(CanonicalSourceErrorV1::InvalidDeclaredFrontend { origin });
            };
            let reading = DeclaredReading {
                pattern: relation.reading,
                relation: relation_role,
                object: object_role.name.clone(),
            };
            if !reading.prefix_pattern().iter().any(
                |part| matches!(part, RelationReadingPartCst::Role(role) if role == &reading.relation),
            ) {
                return Err(CanonicalSourceErrorV1::InvalidDeclaredFrontend {
                    origin,
                });
            }
            readings.push(reading);
        }
        if readings.is_empty() {
            return Err(CanonicalSourceErrorV1::InvalidDeclaredFrontend {
                origin: CanonicalSourceOriginV1 {
                    artifact,
                    start: 0,
                    end: exact_source.len() as u64,
                },
            });
        }
        Ok(Self {
            exact_source: exact_source.into(),
            readings,
        })
    }

    #[must_use]
    pub fn exact_source(&self) -> &[u8] {
        &self.exact_source
    }

    // The irreducible seed reads the declaration of the focused edge itself.
    // All consumer declarations use their selected frontend, not this seed.
    fn bootstrap() -> Self {
        let relation = b"relation".to_vec();
        let object = b"object".to_vec();
        Self {
            exact_source: Box::new([]),
            readings: vec![DeclaredReading {
                pattern: vec![
                    RelationReadingPartCst::Role(relation.clone()),
                    RelationReadingPartCst::Literal(b":".to_vec()),
                    RelationReadingPartCst::Role(object.clone()),
                ],
                relation,
                object,
            }],
        }
    }

    pub(super) fn edge(
        &self,
        subject: &[u8],
        source: &str,
        origin: CanonicalSourceOriginV1,
    ) -> Result<CanonicalFocusedEdgeV1, CanonicalSourceErrorV1> {
        let mut matches = self.matches(source, DeclaredSurface::Edge)?;
        matches.extend(self.matches(source, DeclaredSurface::Focused)?);
        let matched = unique_match(matches, origin)?;
        let declared = &self.readings[matched.reading];
        let focused_relation;
        let relation = if matched.focused {
            focused_relation = String::from_utf8(reading_surface(&declared.pattern, origin)?)
                .map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
            &focused_relation
        } else {
            matched.bindings.get(&declared.relation)
                .ok_or(CanonicalSourceErrorV1::InvalidDeclaredFrontend { origin })?
        };
        let object = matched
            .bindings
            .get(&declared.object)
            .ok_or(CanonicalSourceErrorV1::InvalidDeclaredFrontend { origin })?;
        let relation = relation.as_bytes().to_vec();
        let canonical = self.render(&matched);
        Ok(CanonicalFocusedEdgeV1 {
            subject: subject.to_vec(),
            relation,
            object: object.as_bytes().to_vec(),
            source: canonical.into_bytes(),
            origin,
        })
    }

    pub(super) fn edge_from_values(
        &self,
        reading: usize,
        subject: &[u8],
        relation_value: &str,
        object_value: &str,
        origin: CanonicalSourceOriginV1,
    ) -> Result<CanonicalFocusedEdgeV1, CanonicalSourceErrorV1> {
        let declared = self
            .readings
            .get(reading)
            .ok_or(CanonicalSourceErrorV1::InvalidDeclaredFrontend { origin })?;
        let matched = DeclaredMatch {
            reading,
            focused: false,
            bindings: [
                (declared.relation.clone(), relation_value.to_owned()),
                (declared.object.clone(), object_value.to_owned()),
            ]
            .into(),
        };
        Ok(CanonicalFocusedEdgeV1 {
            subject: subject.to_vec(),
            relation: application_role_bytes(relation_value, origin)?,
            object: object_value.as_bytes().to_vec(),
            source: self.render(&matched).into_bytes(),
            origin,
        })
    }

    pub(super) fn prefix(
        &self,
        source: &str,
        origin: CanonicalSourceOriginV1,
    ) -> Result<(usize, Vec<u8>, String), CanonicalSourceErrorV1> {
        let matches = self.matches(source, DeclaredSurface::Prefix)?;
        let matched = unique_match(matches, origin)?;
        let declared = &self.readings[matched.reading];
        let relation = matched
            .bindings
            .get(&declared.relation)
            .ok_or(CanonicalSourceErrorV1::InvalidDeclaredFrontend { origin })?;
        Ok((
            matched.reading,
            application_role_bytes(relation, origin)?,
            self.render_prefix(&matched),
        ))
    }

    fn canonical_line(
        &self,
        source: &str,
        origin: CanonicalSourceOriginV1,
    ) -> Result<Option<String>, CanonicalSourceErrorV1> {
        let applications = self.matches(source, DeclaredSurface::Edge)?;
        if !applications.is_empty() {
            return Ok(Some(self.render(&unique_match(applications, origin)?)));
        }
        let prefixes = self.matches(source, DeclaredSurface::Prefix)?;
        if prefixes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.render_prefix(&unique_match(prefixes, origin)?)))
        }
    }

    fn matches(
        &self,
        source: &str,
        surface: DeclaredSurface,
    ) -> Result<Vec<DeclaredMatch>, CanonicalSourceErrorV1> {
        let tokens = input_tokens(source)?;
        let mut matches = Vec::new();
        for (reading, declared) in self.readings.iter().enumerate() {
            let pattern = match surface {
                DeclaredSurface::Edge => declared.pattern.as_slice(),
                DeclaredSurface::Focused => {
                    if !matches!(declared.pattern.first(), Some(RelationReadingPartCst::Role(role))
                        if role == &declared.relation) {
                        continue;
                    }
                    &declared.pattern[1..]
                }
                DeclaredSurface::Prefix => declared.prefix_pattern(),
            };
            let mut candidates = Vec::new();
            match_parts(
                source,
                &tokens,
                pattern,
                0,
                0,
                &mut BTreeMap::new(),
                &mut candidates,
            );
            matches.extend(
                candidates
                    .into_iter()
                    .map(|bindings| DeclaredMatch { reading, bindings,
                        focused: matches!(surface, DeclaredSurface::Focused) }),
            );
        }
        Ok(matches)
    }

    fn render(&self, matched: &DeclaredMatch) -> String {
        let reading = &self.readings[matched.reading];
        let pattern = if matched.focused { &reading.pattern[1..] } else { &reading.pattern };
        render_parts(pattern, &matched.bindings)
    }

    fn render_prefix(&self, matched: &DeclaredMatch) -> String {
        let reading = &self.readings[matched.reading];
        render_parts(reading.prefix_pattern(), &matched.bindings)
    }
}

impl DeclaredReading {
    fn prefix_pattern(&self) -> &[RelationReadingPartCst] {
        let object = self
            .pattern
            .iter()
            .position(
                |part| matches!(part, RelationReadingPartCst::Role(role) if role == &self.object),
            )
            .expect("a checked Reading contains its object role");
        let mut end = object;
        while end > 0 && matches!(self.pattern[end - 1], RelationReadingPartCst::Literal(_)) {
            end -= 1;
        }
        &self.pattern[..end]
    }
}

fn render_parts(parts: &[RelationReadingPartCst], bindings: &BTreeMap<Vec<u8>, String>) -> String {
    let mut output = String::new();
    for part in parts {
        let text = match part {
            RelationReadingPartCst::Literal(literal) => {
                std::str::from_utf8(literal).expect("declared Reading literals are UTF-8")
            }
            RelationReadingPartCst::Role(role) => bindings
                .get(role)
                .expect("every matched Reading role is bound"),
        };
        if needs_space(&output, text) {
            output.push(' ');
        }
        output.push_str(text);
    }
    output
}

fn unique_match(
    mut matches: Vec<DeclaredMatch>,
    origin: CanonicalSourceOriginV1,
) -> Result<DeclaredMatch, CanonicalSourceErrorV1> {
    match matches.len() {
        1 => Ok(matches.pop().expect("one declared match")),
        0 => Err(CanonicalSourceErrorV1::MissingDeclaredProduction { origin }),
        _ => Err(CanonicalSourceErrorV1::AmbiguousDeclaredProduction { origin }),
    }
}

fn match_parts(
    source: &str,
    tokens: &[InputToken],
    parts: &[RelationReadingPartCst],
    part: usize,
    token: usize,
    bindings: &mut BTreeMap<Vec<u8>, String>,
    matches: &mut Vec<BTreeMap<Vec<u8>, String>>,
) {
    if part == parts.len() {
        if token == tokens.len() {
            matches.push(bindings.clone());
        }
        return;
    }
    match &parts[part] {
        RelationReadingPartCst::Literal(literal) => {
            let Some(actual) = tokens.get(token) else {
                return;
            };
            if source.as_bytes().get(actual.start..actual.end) == Some(literal.as_slice()) {
                match_parts(
                    source,
                    tokens,
                    parts,
                    part + 1,
                    token + 1,
                    bindings,
                    matches,
                );
            }
        }
        RelationReadingPartCst::Role(role) => {
            let remaining_parts = parts.len() - part - 1;
            let maximum = tokens.len().saturating_sub(remaining_parts);
            for end in token + 1..=maximum {
                if !balanced_binding(source, &tokens[token..end]) {
                    continue;
                }
                let value = &source[tokens[token].start..tokens[end - 1].end];
                if bindings.insert(role.clone(), value.to_owned()).is_some() {
                    return;
                }
                match_parts(source, tokens, parts, part + 1, end, bindings, matches);
                bindings.remove(role);
            }
        }
    }
}

fn balanced_binding(source: &str, tokens: &[InputToken]) -> bool {
    let mut closing = Vec::new();
    for token in tokens {
        match &source.as_bytes()[token.start..token.end] {
            b"(" => closing.push(b')'),
            b"[" => closing.push(b']'),
            b"{" => closing.push(b'}'),
            b")" | b"]" | b"}" => {
                if closing.pop() != Some(source.as_bytes()[token.start]) {
                    return false;
                }
            }
            _ => {}
        }
    }
    closing.is_empty()
}

pub(super) fn input_tokens(source: &str) -> Result<Vec<InputToken>, CanonicalSourceErrorV1> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b' ' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        if bytes[cursor] == b'"' {
            cursor += 1;
            let mut escaped = false;
            while cursor < bytes.len() {
                let byte = bytes[cursor];
                cursor += 1;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    break;
                }
            }
        } else if is_punctuation(bytes[cursor]) {
            cursor += 1;
            if cursor < bytes.len()
                && matches!(
                    &bytes[start..cursor + 1],
                    b"<=" | b">=" | b"!=" | b"->" | b":=" | b"::" | b"~>"
                )
            {
                cursor += 1;
            }
        } else {
            while cursor < bytes.len()
                && bytes[cursor] != b' '
                && (!is_punctuation(bytes[cursor])
                    || is_designation_hyphen(bytes, start, cursor))
            {
                cursor += 1;
            }
        }
        tokens.push(InputToken { start, end: cursor });
    }
    Ok(tokens)
}

fn is_designation_hyphen(bytes: &[u8], start: usize, cursor: usize) -> bool {
    bytes[cursor] == b'-'
        && cursor > start
        && bytes.get(cursor - 1).is_some_and(u8::is_ascii_alphanumeric)
        && bytes.get(cursor + 1).is_some_and(u8::is_ascii_alphanumeric)
}

const fn is_punctuation(byte: u8) -> bool {
    matches!(
        byte,
        b':' | b','
            | b'('
            | b')'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'<'
            | b'>'
            | b'='
            | b'!'
    )
}

pub(super) fn needs_space(output: &str, next: &str) -> bool {
    let Some(previous) = output.as_bytes().last().copied() else {
        return false;
    };
    let Some(first) = next.as_bytes().first().copied() else {
        return false;
    };
    !matches!(first, b':' | b',' | b')' | b']' | b'}') && !matches!(previous, b'(' | b'[' | b'{')
}

pub(super) fn default_declared_frontend()
-> Result<CanonicalDeclaredFrontendV1, CanonicalSourceErrorV1> {
    CanonicalDeclaredFrontendV1::read(DECLARED_FOCUSED_FRONTEND_SOURCE_V1)
}

pub(super) fn canonical_print(
    cst: &CanonicalSourceCstV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    let source =
        std::str::from_utf8(cst.exact_source()).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
    let lines = source_lines(source)?;
    let mut output = String::new();
    for line in lines {
        if line.indent % 2 != 0 {
            return Err(CanonicalSourceErrorV1::UnexpectedIndentation {
                origin: line_origin(cst.artifact(), line),
            });
        }
        let content = line.text[line.indent..].trim_end();
        if !content.is_empty() {
            output.push_str(&" ".repeat(line.indent));
            let origin = line_origin(cst.artifact(), line);
            if let Some(canonical) = cst.declared_frontend.canonical_line(content, origin)? {
                output.push_str(&canonical);
            } else {
                output.push_str(content);
            }
        }
        output.push('\n');
    }
    Ok(output.into_bytes())
}
