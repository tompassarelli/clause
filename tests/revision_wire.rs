//! Exact typed Revision/Delta wire contract for semantic-v7.

use std::collections::BTreeMap;

use clause::{
    delta::RevisionDiff,
    kernel::{
        AssertionOccurrence, Cardinality, Definition, Delta, DerivationRule, Judgment,
        JudgmentKind, JudgmentStatus, JudgmentTarget, LookupMode, Model, Name, OpenWorldStatus,
        Pattern, PatternId, Referent, ReferentId, RelationShape, RelationalContent, Role, RoleId,
        SemanticAtom, Term,
    },
    wire,
};

const MODEL: u8 = 0x01;
const CARRIES: u8 = 0x02;
const ROUTE: u8 = 0x03;
const MODULE: u8 = 0x04;
const EAST: u8 = 0x05;
const SOURCE_A: u8 = 0x06;
const SCOPE: u8 = 0x07;
const AUTHORITY: u8 = 0x08;
const POLICY: u8 = 0x09;
const EAST_OCCURRENCE: u8 = 0x0a;
const EAST_JUDGMENT: u8 = 0x0b;
const REFLEXIVE_RULE: u8 = 0x0c;
const WEST: u8 = 0x0d;
const WEST_OCCURRENCE: u8 = 0x0e;
const WEST_JUDGMENT: u8 = 0x0f;
const SOURCE_B: u8 = 0x10;
const SECOND_OCCURRENCE: u8 = 0x11;
const SECOND_JUDGMENT: u8 = 0x12;

const ROUTE_ROLE: u8 = 0x21;
const MODULE_ROLE: u8 = 0x22;
const LABEL_ROLE: u8 = 0x23;

const ROUTE_PATTERN: u8 = 0x31;
const MODULE_PATTERN: u8 = 0x32;
const LABEL_PATTERN: u8 = 0x33;

fn referent_id(seed: u8) -> ReferentId {
    ReferentId::from_digest([seed; 32])
}

fn role_id(seed: u8) -> RoleId {
    RoleId::from_digest([seed; 32])
}

fn pattern_id(seed: u8) -> PatternId {
    PatternId::from_digest([seed; 32])
}

fn carries(label: u8) -> RelationalContent {
    RelationalContent::new(
        referent_id(CARRIES),
        BTreeMap::from([
            (role_id(ROUTE_ROLE), Term::referent(referent_id(ROUTE))),
            (role_id(MODULE_ROLE), Term::referent(referent_id(MODULE))),
            (role_id(LABEL_ROLE), Term::referent(referent_id(label))),
        ]),
    )
    .unwrap()
}

fn carries_pattern() -> RelationalContent {
    RelationalContent::new(
        referent_id(CARRIES),
        BTreeMap::from([
            (
                role_id(ROUTE_ROLE),
                Term::pattern(pattern_id(ROUTE_PATTERN)),
            ),
            (
                role_id(MODULE_ROLE),
                Term::pattern(pattern_id(MODULE_PATTERN)),
            ),
            (
                role_id(LABEL_ROLE),
                Term::pattern(pattern_id(LABEL_PATTERN)),
            ),
        ]),
    )
    .unwrap()
}

fn occurrence(content: &RelationalContent, id: u8, source: u8) -> AssertionOccurrence {
    AssertionOccurrence::new(
        referent_id(id),
        content.id().clone(),
        referent_id(source),
        referent_id(SCOPE),
    )
}

fn judgment(id: u8, occurrence: u8, source: u8) -> Judgment {
    Judgment::new(
        referent_id(id),
        referent_id(AUTHORITY),
        referent_id(SCOPE),
        JudgmentTarget::Occurrence(referent_id(occurrence)),
        JudgmentKind::Admitted {
            policy: referent_id(POLICY),
            basis: vec![referent_id(source)],
        },
        JudgmentStatus::Affirmed,
    )
}

fn fixture_model(label: u8, occurrence_id: u8, judgment_id: u8, source: u8) -> Model {
    let ground = carries(label);
    let pattern_content = carries_pattern();
    let shape = RelationShape::new(
        referent_id(CARRIES),
        [ROUTE_ROLE, MODULE_ROLE, LABEL_ROLE]
            .map(|seed| {
                let id = role_id(seed);
                (id.clone(), Role::new(id, Vec::new()).unwrap())
            })
            .into(),
        vec![
            LookupMode::finite(
                vec![role_id(ROUTE_ROLE)],
                vec![role_id(MODULE_ROLE), role_id(LABEL_ROLE)],
                Cardinality::Many,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let referents = [
        MODEL,
        CARRIES,
        ROUTE,
        MODULE,
        label,
        source,
        SCOPE,
        AUTHORITY,
        POLICY,
        occurrence_id,
        judgment_id,
        REFLEXIVE_RULE,
    ]
    .map(|seed| {
        let id = referent_id(seed);
        (id.clone(), Referent::new(id))
    })
    .into();
    let contents = [ground.clone(), pattern_content.clone()]
        .map(|content| (content.id().clone(), content))
        .into();
    let pattern = Pattern::new(vec![pattern_content.id().clone()]).unwrap();
    Model::with_distinctions(
        referent_id(MODEL),
        referents,
        contents,
        BTreeMap::from([(shape.referent().clone(), shape)]),
        vec![occurrence(&ground, occurrence_id, source)],
        Vec::new(),
        vec![
            DerivationRule::new(
                referent_id(REFLEXIVE_RULE),
                referent_id(SCOPE),
                referent_id(AUTHORITY),
                pattern.clone(),
                pattern,
            )
            .unwrap(),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![judgment(judgment_id, occurrence_id, source)],
    )
    .unwrap()
}

const EAST_CONTENT_ID: &str =
    "content-sha256-4e3d6e6691133c4346c06e561f2849e05fb0f535efd11d3a69d9a033e83a9b1a";
const PATTERN_CONTENT_ID: &str =
    "content-sha256-97ab3c6be6fa68e4e876fb35c107efbf523f336736173515186df358b19cfa2b";
const WEST_CONTENT_ID: &str =
    "content-sha256-6f99f9d878a067fc9a7e94bae2e2b8fef060d81629c845cb9b769107d83fb0d0";
const EXPECTED_REVISION_ID: &str =
    "rev-sha256-75b9d2923d7566e1697ff249fba9ffde5def40c56f27bc49d35ac5d5aa46aebf";

fn expected_semantic() -> String {
    let referent = |seed: u8| format!("ref-sha256-{}", format!("{seed:02x}").repeat(32));
    let role = |seed: u8| format!("role-sha256-{}", format!("{seed:02x}").repeat(32));
    let pattern = |seed: u8| format!("pattern-sha256-{}", format!("{seed:02x}").repeat(32));
    let referents = (MODEL..=REFLEXIVE_RULE)
        .map(|seed| format!("[\"referent\",\"{}\"]", referent(seed)))
        .collect::<Vec<_>>()
        .join(",");
    let ground_roles = format!(
        "[\"{}\",[\"referent\",\"{}\"]],[\"{}\",[\"referent\",\"{}\"]],[\"{}\",[\"referent\",\"{}\"]]",
        role(ROUTE_ROLE),
        referent(ROUTE),
        role(MODULE_ROLE),
        referent(MODULE),
        role(LABEL_ROLE),
        referent(EAST),
    );
    let pattern_roles = format!(
        "[\"{}\",[\"pattern\",\"{}\"]],[\"{}\",[\"pattern\",\"{}\"]],[\"{}\",[\"pattern\",\"{}\"]]",
        role(ROUTE_ROLE),
        pattern(ROUTE_PATTERN),
        role(MODULE_ROLE),
        pattern(MODULE_PATTERN),
        role(LABEL_ROLE),
        pattern(LABEL_PATTERN),
    );
    let contents = format!(
        "[\"relational-content\",\"{EAST_CONTENT_ID}\",\"{}\",[\"roles\",[{ground_roles}]]],[\"relational-content\",\"{PATTERN_CONTENT_ID}\",\"{}\",[\"roles\",[{pattern_roles}]]]",
        referent(CARRIES),
        referent(CARRIES),
    );
    let shape_roles = [ROUTE_ROLE, MODULE_ROLE, LABEL_ROLE]
        .map(|seed| format!("[\"role\",\"{}\",[\"admissibility\",[]]]", role(seed)))
        .join(",");
    let shape = format!(
        "[\"relation-shape\",\"{}\",[\"roles\",[{shape_roles}]],[\"lookup\",[[\"lookup\",[\"known\",[\"{}\"]],[\"sought\",[\"{}\",\"{}\"]],[\"cardinality\",\"many\"]]]]]",
        referent(CARRIES),
        role(ROUTE_ROLE),
        role(MODULE_ROLE),
        role(LABEL_ROLE),
    );
    let occurrence = format!(
        "[\"assertion-occurrence\",\"{}\",\"{EAST_CONTENT_ID}\",[\"source\",\"{}\"],[\"scope\",\"{}\"]]",
        referent(EAST_OCCURRENCE),
        referent(SOURCE_A),
        referent(SCOPE),
    );
    let rule = format!(
        "[\"derivation-rule\",\"{}\",[\"scope\",\"{}\"],[\"authority\",\"{}\"],[\"premises\",[\"pattern\",[\"{PATTERN_CONTENT_ID}\"]]],[\"conclusion\",[\"pattern\",[\"{PATTERN_CONTENT_ID}\"]]]]",
        referent(REFLEXIVE_RULE),
        referent(SCOPE),
        referent(AUTHORITY),
    );
    let judgment = format!(
        "[\"judgment\",\"{}\",[\"authority\",\"{}\"],[\"scope\",\"{}\"],[\"target\",[\"occurrence\",\"{}\"]],[\"kind\",[\"admitted\",\"{}\",[\"{}\"]]],[\"status\",\"affirmed\"]]",
        referent(EAST_JUDGMENT),
        referent(AUTHORITY),
        referent(SCOPE),
        referent(EAST_OCCURRENCE),
        referent(POLICY),
        referent(SOURCE_A),
    );
    format!(
        "[\"clause-semantic-v7\",[\"lineage\",[\"root\"]],[\"model\",\"{}\"],[\"referents\",[{referents}]],[\"relational-contents\",[{contents}]],[\"relation-shapes\",[{shape}]],[\"occurrences\",[{occurrence}]],[\"definitions\",[]],[\"derivation-rules\",[{rule}]],[\"universal-laws\",[]],[\"invariants\",[]],[\"goals\",[]],[\"transitions\",[]],[\"judgments\",[{judgment}]]]",
        referent(MODEL),
    )
}

fn wire_for_semantic(semantic: &str) -> String {
    let identity = wire::sha256_hex(semantic.as_bytes());
    format!(
        "[\"{}\",\"rev-sha256-{identity}\",{semantic}]",
        wire::REVISION_TAG
    )
}

fn replacement_delta(base: &clause::kernel::Revision) -> Delta {
    let east = carries(EAST);
    let west = carries(WEST);
    Delta::new(
        base.identity().clone(),
        vec![
            SemanticAtom::Referent(Referent::new(referent_id(WEST))),
            SemanticAtom::Referent(Referent::new(referent_id(WEST_OCCURRENCE))),
            SemanticAtom::Referent(Referent::new(referent_id(WEST_JUDGMENT))),
            SemanticAtom::RelationalContent(west.clone()),
            SemanticAtom::AssertionOccurrence(occurrence(&west, WEST_OCCURRENCE, SOURCE_A)),
            SemanticAtom::Judgment(judgment(WEST_JUDGMENT, WEST_OCCURRENCE, SOURCE_A)),
        ],
        vec![
            SemanticAtom::Referent(Referent::new(referent_id(EAST))),
            SemanticAtom::Referent(Referent::new(referent_id(EAST_OCCURRENCE))),
            SemanticAtom::Referent(Referent::new(referent_id(EAST_JUDGMENT))),
            SemanticAtom::RelationalContent(east.clone()),
            SemanticAtom::AssertionOccurrence(occurrence(&east, EAST_OCCURRENCE, SOURCE_A)),
            SemanticAtom::Judgment(judgment(EAST_JUDGMENT, EAST_OCCURRENCE, SOURCE_A)),
        ],
    )
    .unwrap()
}

#[test]
fn exact_semantic_v7_bytes_hash_and_revision_v5_roundtrip_are_frozen() {
    assert_eq!(
        wire::sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(carries(EAST).id().as_str(), EAST_CONTENT_ID);
    assert_eq!(carries_pattern().id().as_str(), PATTERN_CONTENT_ID);
    assert_eq!(carries(WEST).id().as_str(), WEST_CONTENT_ID);

    let revision = wire::admit(fixture_model(
        EAST,
        EAST_OCCURRENCE,
        EAST_JUDGMENT,
        SOURCE_A,
    ));
    let occurrence = &revision.model().occurrences()[0];
    assert_eq!(occurrence.content(), carries(EAST).id());
    assert_eq!(occurrence.source(), &referent_id(SOURCE_A));
    assert_eq!(occurrence.scope(), &referent_id(SCOPE));
    let rule = &revision.model().derivation_rules()[0];
    assert_eq!(rule.authority(), &referent_id(AUTHORITY));
    assert_eq!(rule.scope(), &referent_id(SCOPE));
    assert_eq!(rule.premises(), rule.conclusion());
    assert_eq!(rule.premises().forms(), [carries_pattern().id().clone()]);
    let judgment = &revision.model().judgments()[0];
    assert_eq!(judgment.authority(), &referent_id(AUTHORITY));
    assert_eq!(judgment.scope(), &referent_id(SCOPE));

    let expected_semantic = expected_semantic();
    assert_eq!(wire::semantic_payload(&revision), expected_semantic);
    assert_eq!(revision.identity().to_string(), EXPECTED_REVISION_ID);
    let expected_wire = format!(
        "[\"{}\",\"{EXPECTED_REVISION_ID}\",{expected_semantic}]",
        wire::REVISION_TAG
    );
    assert_eq!(wire::serialize(&revision), expected_wire);
    assert_eq!(wire::reload(&expected_wire).unwrap(), revision);
    assert_eq!(revision.predecessor(), None);
}

#[test]
fn structural_terms_roundtrip_and_malformed_encodings_fail_closed() {
    let model_id = referent_id(0x40);
    let definition_id = referent_id(0x41);
    let structural = Term::product(BTreeMap::from([
        (Name::new("flag".into()).unwrap(), Term::boolean(true)),
        (
            Name::new("number".into()).unwrap(),
            Term::f32(-0.0).unwrap(),
        ),
        (
            Name::new("tuple".into()).unwrap(),
            Term::tuple((0..11).map(Term::int).collect()).unwrap(),
        ),
        (
            Name::new("variant".into()).unwrap(),
            Term::sum(Name::new("some".into()).unwrap(), Term::int(5)).unwrap(),
        ),
    ]))
    .unwrap();
    let referents = [model_id.clone(), definition_id.clone()]
        .into_iter()
        .map(|id| (id.clone(), Referent::new(id)))
        .collect();
    let model = Model::with_distinctions(
        model_id,
        referents,
        BTreeMap::new(),
        BTreeMap::new(),
        Vec::new(),
        vec![Definition::new(definition_id, structural)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    let revision = wire::admit(model);
    let canonical = wire::serialize(&revision);

    assert!(canonical.contains("[\"f32\",\"00000000\"]"));
    assert!(canonical.contains("[\"sum\",\"some\",[\"int\",\"5\"]]"));
    assert_eq!(wire::reload(&canonical).unwrap(), revision);

    let infinite = wire::semantic_payload(&revision).replacen(
        "[\"f32\",\"00000000\"]",
        "[\"f32\",\"7f800000\"]",
        1,
    );
    assert_eq!(
        wire::reload(&wire_for_semantic(&infinite))
            .unwrap_err()
            .to_string(),
        "F32 term must be finite"
    );

    let duplicate = wire::semantic_payload(&revision).replacen(
        "[[\"flag\",[\"bool\",\"true\"]]",
        "[[\"flag\",[\"bool\",\"true\"]],[\"flag\",[\"bool\",\"false\"]]",
        1,
    );
    assert_eq!(
        wire::reload(&wire_for_semantic(&duplicate))
            .unwrap_err()
            .to_string(),
        "duplicate product label"
    );
}

#[test]
fn reload_rejects_retired_tags_noncanonical_order_and_hash_mismatch() {
    let revision = wire::admit(fixture_model(
        EAST,
        EAST_OCCURRENCE,
        EAST_JUDGMENT,
        SOURCE_A,
    ));
    let canonical = wire::serialize(&revision);
    let retired_revision_tag = ["clause-revision-", "v4"].concat();
    let retired_semantic_tag = ["clause-semantic-", "v6"].concat();
    assert!(
        wire::reload(&canonical.replacen(wire::REVISION_TAG, &retired_revision_tag, 1)).is_err()
    );
    let retired_semantic =
        expected_semantic().replacen(wire::SEMANTIC_TAG, &retired_semantic_tag, 1);
    assert!(wire::reload(&wire_for_semantic(&retired_semantic)).is_err());
    assert!(wire::reload(&(" ".to_owned() + &canonical)).is_err());
    assert!(wire::reload(&(canonical.clone() + " ")).is_err());
    assert!(wire::reload(&canonical.replacen("rev-sha256-", "rev-sha256-a", 1)).is_err());

    let first = format!(
        "[\"referent\",\"{}\"],[\"referent\",\"{}\"]",
        referent_id(MODEL).as_str(),
        referent_id(CARRIES).as_str(),
    );
    let swapped = format!(
        "[\"referent\",\"{}\"],[\"referent\",\"{}\"]",
        referent_id(CARRIES).as_str(),
        referent_id(MODEL).as_str(),
    );
    let noncanonical_semantic = expected_semantic().replacen(&first, &swapped, 1);
    assert_ne!(noncanonical_semantic, expected_semantic());
    assert!(wire::reload(&wire_for_semantic(&noncanonical_semantic)).is_err());
}

#[test]
fn reload_rejects_typed_content_tampering_even_with_a_recomputed_revision_hash() {
    let original = format!(
        "[\"{}\",[\"referent\",\"{}\"]]",
        role_id(LABEL_ROLE).as_str(),
        referent_id(EAST).as_str(),
    );
    let replacement = format!(
        "[\"{}\",[\"referent\",\"{}\"]]",
        role_id(LABEL_ROLE).as_str(),
        referent_id(WEST).as_str(),
    );
    let tampered = expected_semantic().replacen(&original, &replacement, 1);
    assert_ne!(tampered, expected_semantic());
    assert!(wire::reload(&wire_for_semantic(&tampered)).is_err());
}

#[test]
fn occurrences_preserve_distinct_source_acts_for_one_canonical_content() {
    let base = fixture_model(EAST, EAST_OCCURRENCE, EAST_JUDGMENT, SOURCE_A);
    let content = carries(EAST);
    let mut atoms = base.atoms();
    for seed in [SOURCE_B, SECOND_OCCURRENCE, SECOND_JUDGMENT] {
        atoms.insert(SemanticAtom::Referent(Referent::new(referent_id(seed))));
    }
    atoms.insert(SemanticAtom::AssertionOccurrence(occurrence(
        &content,
        SECOND_OCCURRENCE,
        SOURCE_B,
    )));
    atoms.insert(SemanticAtom::Judgment(judgment(
        SECOND_JUDGMENT,
        SECOND_OCCURRENCE,
        SOURCE_B,
    )));
    let model = Model::from_atoms(referent_id(MODEL), atoms).unwrap();

    assert_eq!(model.relational_contents().len(), 2);
    assert_eq!(model.occurrences().len(), 2);
    assert_eq!(model.occurrences()[0].content(), content.id());
    assert_eq!(model.occurrences()[1].content(), content.id());
    assert_ne!(model.occurrences()[0].id(), model.occurrences()[1].id());
    assert_ne!(
        model.occurrences()[0].source(),
        model.occurrences()[1].source()
    );
    assert!(
        model
            .occurrences()
            .iter()
            .all(|item| item.scope() == &referent_id(SCOPE))
    );
    assert!(
        model
            .judgments()
            .iter()
            .all(|item| item.authority() == &referent_id(AUTHORITY)
                && item.scope() == &referent_id(SCOPE))
    );
    assert_eq!(
        model.status(&content, &referent_id(AUTHORITY), &referent_id(SCOPE)),
        OpenWorldStatus::Admitted
    );
    assert_eq!(
        model.operative_status(&content),
        OpenWorldStatus::Undetermined
    );
    assert!(model.admitted_contents().is_empty());

    let revision = wire::admit(model);
    assert_eq!(wire::reload(&wire::serialize(&revision)).unwrap(), revision);
}

#[test]
fn semantic_atom_delta_records_the_exact_predecessor_edge() {
    let base = wire::admit(fixture_model(
        EAST,
        EAST_OCCURRENCE,
        EAST_JUDGMENT,
        SOURCE_A,
    ));
    let original = base.clone();
    let delta = replacement_delta(&base);
    let successor = delta.apply(&base).unwrap();

    assert_eq!(base, original);
    assert_eq!(
        successor.model(),
        &fixture_model(WEST, WEST_OCCURRENCE, WEST_JUDGMENT, SOURCE_A,)
    );
    assert_eq!(successor.predecessor(), Some(base.identity()));
    assert_eq!(successor.delta(), Some(&delta));
    assert_eq!(
        wire::reload_successor(&wire::serialize(&successor), &base).unwrap(),
        successor
    );
    assert!(wire::reload(&wire::serialize(&successor)).is_err());

    let diff = RevisionDiff::between(&base, &successor).unwrap();
    assert_eq!(diff.base_revision(), base.identity());
    assert_eq!(diff.successor_revision(), successor.identity());
    assert_eq!(diff.admitted_atoms(), delta.admissions());
    assert_eq!(diff.withdrawn_atoms(), delta.withdrawals());
    assert!(diff.added().is_empty());
    assert!(diff.removed().is_empty());
    assert_eq!(
        successor
            .model()
            .status(&carries(WEST), &referent_id(AUTHORITY), &referent_id(SCOPE)),
        OpenWorldStatus::Admitted
    );
    assert!(RevisionDiff::between(&successor, &base).is_err());
}

#[test]
fn successor_reload_rejects_a_delta_atom_missing_from_its_snapshot() {
    let base = wire::admit(fixture_model(
        EAST,
        EAST_OCCURRENCE,
        EAST_JUDGMENT,
        SOURCE_A,
    ));
    let successor = replacement_delta(&base).apply(&base).unwrap();
    let semantic = wire::semantic_payload(&successor);
    let admission = format!(
        "[\"admit\",[[\"referent\",\"{}\"]",
        referent_id(WEST).as_str(),
    );
    let absent_admission = format!("[\"admit\",[[\"referent\",\"{}\"]", referent_id(0).as_str(),);
    let tampered = semantic.replacen(&admission, &absent_admission, 1);
    assert_ne!(tampered, semantic);
    assert!(wire::reload_successor(&wire_for_semantic(&tampered), &base).is_err());
}

#[test]
fn successor_reload_requires_predecessor_and_rejects_an_inexact_delta() {
    let base = wire::admit(fixture_model(
        EAST,
        EAST_OCCURRENCE,
        EAST_JUDGMENT,
        SOURCE_A,
    ));
    let successor = replacement_delta(&base).apply(&base).unwrap();
    let semantic = wire::semantic_payload(&successor);
    let admission = format!(
        "[\"admit\",[[\"referent\",\"{}\"]",
        referent_id(WEST).as_str(),
    );
    let already_present = format!(
        "[\"admit\",[[\"referent\",\"{}\"]",
        referent_id(MODEL).as_str(),
    );
    let tampered = semantic.replacen(&admission, &already_present, 1);
    assert_ne!(tampered, semantic);

    assert!(wire::reload(&wire::serialize(&successor)).is_err());
    assert!(wire::reload_successor(&wire_for_semantic(&tampered), &base).is_err());
}

#[test]
fn semantic_atom_delta_rejects_invalid_base_and_atom_sets() {
    let base = wire::admit(fixture_model(
        EAST,
        EAST_OCCURRENCE,
        EAST_JUDGMENT,
        SOURCE_A,
    ));
    let other = wire::admit(fixture_model(
        WEST,
        WEST_OCCURRENCE,
        WEST_JUDGMENT,
        SOURCE_A,
    ));
    let east = SemanticAtom::RelationalContent(carries(EAST));
    let west = SemanticAtom::RelationalContent(carries(WEST));

    assert!(
        Delta::new(other.identity().clone(), vec![west.clone()], Vec::new())
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(
        Delta::new(base.identity().clone(), Vec::new(), vec![west.clone()])
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(
        Delta::new(base.identity().clone(), vec![east.clone()], Vec::new())
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(
        Delta::new(base.identity().clone(), Vec::new(), vec![east.clone()])
            .unwrap()
            .apply(&base)
            .is_err()
    );
    assert!(Delta::new(base.identity().clone(), Vec::new(), Vec::new()).is_err());
    assert!(
        Delta::new(
            base.identity().clone(),
            vec![west.clone(), west.clone()],
            Vec::new(),
        )
        .is_err()
    );
    assert!(Delta::new(base.identity().clone(), vec![west.clone()], vec![west],).is_err());
}
