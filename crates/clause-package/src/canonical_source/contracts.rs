//! Binary role contracts elaborated from ordinary applications. Parsing never
//! reclassifies a subject-focus block by inspecting its children.
use super::*;

pub(super) fn read(
    artifact: CanonicalSourceArtifactIdV1,
    blocks: &[&[SourceLine<'_>]],
    frontend: &CanonicalDeclaredFrontendV1,
) -> Result<Vec<RelationCst>, CanonicalSourceErrorV1> {
    let mut subjects = BTreeMap::<Vec<u8>, Vec<ApplicationCst>>::new();
    for block in blocks {
        if declaration_designation(block, artifact)?.is_some()
            || block[0].text.starts_with("mode ")
        {
            continue;
        }
        let edges = if let Some(edges) = patterns::edge_focus(&logical_source_lines(artifact, block)?
            .into_iter().filter(|line| !line.text.is_empty()).collect::<Vec<_>>(), frontend)? {
            edges
        } else {
            descriptor_edges(artifact, block, frontend)?
        };
        for edge in edges {
            let application = declared_application(&edge)?;
            if !matches!(application.role.as_slice(), b"domain" | b"range" | b"cardinality") { continue; }
            subjects.entry(application.subject.clone()).or_default().push(application);
        }
    }
    let mut contracts = Vec::new();
    for (designation, applications) in subjects {
        // A partial description stays ordinary data. Executable use must still
        // resolve a complete contract; an isolated role never selects a schema.
        if ![b"domain".as_slice(), b"range", b"cardinality"].iter().all(|role|
            applications.iter().any(|application| application.role == *role)) {
            continue;
        }
        let mut facts = BTreeMap::<Vec<u8>, ApplicationCst>::new();
        for application in applications {
            if let Some(prior) = facts.get(&application.role) {
                if prior.object != application.object {
                    return Err(CanonicalSourceErrorV1::DuplicateChild {
                        producer: designation,
                        child: application.role,
                    });
                }
            } else {
                facts.insert(application.role.clone(), application);
            }
        }
        let domain = &facts[b"domain".as_slice()];
        let range = &facts[b"range".as_slice()];
        let cardinality = &facts[b"cardinality".as_slice()];
        let origin = domain.emission.origin;
        let name = |application: &ApplicationCst| match &application.object {
            CanonicalScalarValueV1::Symbol(value) => Ok(value.clone()),
            _ => Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin: application.emission.origin }),
        };
        let cardinality_name = name(cardinality)?;
        let cardinality_value = match cardinality_name.as_slice() {
            b"one" => SourceCardinality::One,
            b"maybe" => SourceCardinality::Maybe,
            b"some" => SourceCardinality::Some,
            b"many" => SourceCardinality::Many,
            _ => return Err(CanonicalSourceErrorV1::InvalidMode { origin: cardinality.emission.origin }),
        };
        // These are the explicit roles of a binary semantic application, not
        // positions guessed from a user Reading or its inferred types.
        let subject = b"subject".to_vec();
        let object = b"object".to_vec();
        let mut reading = vec![RelationReadingPartCst::Role(subject.clone())];
        reading.extend(designation.split(|byte| byte.is_ascii_whitespace())
            .map(|part| RelationReadingPartCst::Literal(part.to_vec())));
        reading.push(RelationReadingPartCst::Role(object.clone()));
        let mut canonical = Vec::new();
        frame_bytes(&mut canonical, &subject);
        frame_bytes(&mut canonical, &object);
        frame_bytes(&mut canonical, &cardinality_name);
        contracts.push(RelationCst {
            contract_origin: Some(origin), surface: designation.clone(), designation,
            reading, subject: Some(subject.clone()),
            roles: vec![
                RelationRoleCst { name: subject.clone(), domain: name(domain)?, origin: domain.emission.origin },
                RelationRoleCst { name: object.clone(), domain: name(range)?, origin: range.emission.origin },
            ],
            modes: vec![RelationModeCst {
                known: vec![subject], produced: vec![object], cardinality: cardinality_value,
                reactive_obligation: None, continues_linearly: false, effect: None,
                canonical, origin: cardinality.emission.origin,
            }],
        });
    }
    Ok(contracts)
}

// Resolve role ranges before interpreting contextual object children. This
// query reads only descriptor edges; the complete reader checks every other
// edge once the resulting range environment is available.
fn descriptor_edges(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    frontend: &CanonicalDeclaredFrontendV1,
) -> Result<Vec<CanonicalFocusedEdgeV1>, CanonicalSourceErrorV1> {
    let head = block[0];
    if head.text.contains(char::is_whitespace) || head.text.contains(':') {
        return Ok(vec![]);
    }
    let subject = designation_bytes(head.text, line_origin(artifact, head))?;
    let lines = logical_source_lines(artifact, &block[1..])?;
    let mut edges = Vec::new();
    for (index, line) in lines.iter().enumerate().filter(|(_, line)| line.indent == 2) {
        if let Ok(edge) = frontend.edge(&subject, &line.text, line.origin) {
            if matches!(edge.relation.as_slice(), b"domain" | b"range" | b"cardinality") {
                edges.push(edge);
            }
        } else if let Ok((reading, role, _)) = frontend.prefix(&line.text, line.origin)
            && matches!(role.as_slice(), b"domain" | b"range" | b"cardinality") {
            for object in lines[index + 1..].iter().take_while(|line| line.indent > 2) {
                if object.indent != 4 { return Err(CanonicalSourceErrorV1::UnexpectedIndentation { origin: object.origin }); }
                edges.push(frontend.edge_from_values(reading, &subject,
                    std::str::from_utf8(&role).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?,
                    &object.text, object.origin)?);
            }
        }
    }
    Ok(edges)
}
