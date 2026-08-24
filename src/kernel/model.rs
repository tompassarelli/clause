use std::collections::{BTreeMap, BTreeSet};

use super::{
    clause::{
        AssertionOccurrence, Definition, DerivationRule, Goal, Invariant, Judgment, JudgmentKind,
        JudgmentStatus, JudgmentTarget, OpenWorldStatus, Pattern, RelationalContent, Term,
        Transition, UniversalLaw,
    },
    error::{KernelError, Result},
    identity::{ContentId, PatternId, ReferentId},
    schema::{Cardinality, Referent, RelationShape, Role, RolePredicate},
};

/// Every signed semantic constituent of a Model snapshot.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SemanticAtom {
    Referent(Referent),
    RelationalContent(RelationalContent),
    RelationShape(RelationShape),
    AssertionOccurrence(AssertionOccurrence),
    Definition(Definition),
    DerivationRule(DerivationRule),
    UniversalLaw(UniversalLaw),
    Invariant(Invariant),
    Goal(Goal),
    Transition(Transition),
    Judgment(Judgment),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    id: ReferentId,
    referents: BTreeMap<ReferentId, Referent>,
    relational_contents: BTreeMap<ContentId, RelationalContent>,
    relation_shapes: BTreeMap<ReferentId, RelationShape>,
    occurrences: Vec<AssertionOccurrence>,
    definitions: Vec<Definition>,
    derivation_rules: Vec<DerivationRule>,
    universal_laws: Vec<UniversalLaw>,
    invariants: Vec<Invariant>,
    goals: Vec<Goal>,
    transitions: Vec<Transition>,
    judgments: Vec<Judgment>,
    admitted_contents: Vec<RelationalContent>,
}

impl Model {
    pub fn new(
        id: ReferentId,
        referents: BTreeMap<ReferentId, Referent>,
        relational_contents: BTreeMap<ContentId, RelationalContent>,
        relation_shapes: BTreeMap<ReferentId, RelationShape>,
        occurrences: Vec<AssertionOccurrence>,
        derivation_rules: Vec<DerivationRule>,
    ) -> Result<Self> {
        Self::with_distinctions(
            id,
            referents,
            relational_contents,
            relation_shapes,
            occurrences,
            Vec::new(),
            derivation_rules,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_distinctions(
        id: ReferentId,
        referents: BTreeMap<ReferentId, Referent>,
        relational_contents: BTreeMap<ContentId, RelationalContent>,
        relation_shapes: BTreeMap<ReferentId, RelationShape>,
        mut occurrences: Vec<AssertionOccurrence>,
        mut definitions: Vec<Definition>,
        mut derivation_rules: Vec<DerivationRule>,
        mut universal_laws: Vec<UniversalLaw>,
        mut invariants: Vec<Invariant>,
        mut goals: Vec<Goal>,
        mut transitions: Vec<Transition>,
        mut judgments: Vec<Judgment>,
    ) -> Result<Self> {
        validate_keyed(&referents, Referent::id, "referent")?;
        validate_keyed(
            &relational_contents,
            RelationalContent::id,
            "relational content",
        )?;
        validate_keyed(&relation_shapes, RelationShape::referent, "relation shape")?;
        require_referent(&referents, &id, "Model identity")?;

        for shape in relation_shapes.values() {
            require_referent(&referents, shape.referent(), "relational-position")?;
            for role in shape.roles().values() {
                for predicate in role.admissibility() {
                    validate_predicate(&referents, &relation_shapes, predicate)?;
                }
            }
        }
        let mut application_targets = BTreeSet::new();
        for term in relational_contents
            .values()
            .flat_map(|content| content.roles().values())
            .chain(definitions.iter().map(Definition::denotation))
        {
            term.walk(&mut |term| {
                if let Term::Application(id) = term {
                    application_targets.insert(id.clone());
                }
            });
        }
        let mut complete_contents = occurrences
            .iter()
            .map(AssertionOccurrence::content)
            .cloned()
            .collect::<BTreeSet<_>>();
        for pattern in derivation_rules
            .iter()
            .flat_map(|rule| [rule.premises(), rule.conclusion()])
            .chain(universal_laws.iter().map(UniversalLaw::generalized))
            .chain(invariants.iter().map(Invariant::condition))
            .chain(goals.iter().map(Goal::desired))
        {
            complete_contents.extend(pattern.forms().iter().cloned());
        }
        for transition in &transitions {
            complete_contents.insert(transition.from().clone());
            complete_contents.insert(transition.to().clone());
        }
        complete_contents.extend(judgments.iter().filter_map(|judgment| {
            if let JudgmentTarget::Content(content) = judgment.target() {
                Some(content.clone())
            } else {
                None
            }
        }));
        for content in relational_contents.values() {
            validate_content_structure(
                &referents,
                &relational_contents,
                &relation_shapes,
                content,
                !application_targets.contains(content.id())
                    || complete_contents.contains(content.id()),
            )?;
        }
        validate_application_acyclic(&relational_contents)?;

        sort_unique_by(
            &mut occurrences,
            AssertionOccurrence::id,
            "assertion occurrence",
        )?;
        let occurrence_ids = occurrences
            .iter()
            .map(|item| item.id().clone())
            .collect::<BTreeSet<_>>();
        for occurrence in &occurrences {
            require_referent(&referents, occurrence.id(), "assertion occurrence")?;
            require_content(
                &relational_contents,
                occurrence.content(),
                "assertion occurrence",
            )?;
            if !content_is_ground(
                &relational_contents,
                &relational_contents[occurrence.content()],
            ) {
                return Err(KernelError::new(
                    "assertion occurrence content must be recursively ground",
                ));
            }
            require_referent(&referents, occurrence.source(), "assertion source")?;
            require_referent(&referents, occurrence.scope(), "assertion scope")?;
        }

        definitions.sort();
        derivation_rules.sort();
        universal_laws.sort();
        invariants.sort();
        goals.sort();
        transitions.sort();
        judgments.sort();
        ensure_unique_ids(definitions.iter().map(Definition::id), "definition")?;
        ensure_unique_ids(
            derivation_rules.iter().map(DerivationRule::id),
            "derivation rule",
        )?;
        ensure_unique_ids(universal_laws.iter().map(UniversalLaw::id), "universal law")?;
        ensure_unique_ids(invariants.iter().map(Invariant::id), "invariant")?;
        ensure_unique_ids(goals.iter().map(Goal::id), "goal")?;
        ensure_unique_ids(transitions.iter().map(Transition::id), "transition")?;
        ensure_unique_ids(judgments.iter().map(Judgment::id), "judgment")?;

        for definition in &definitions {
            require_referent(&referents, definition.id(), "definition")?;
            validate_term(
                &referents,
                &relational_contents,
                definition.denotation(),
                false,
            )?;
        }
        for rule in &derivation_rules {
            require_referent(&referents, rule.id(), "derivation rule")?;
            require_referent(&referents, rule.scope(), "derivation rule scope")?;
            require_referent(&referents, rule.authority(), "derivation rule authority")?;
            validate_rule(&relational_contents, &relation_shapes, rule)?;
        }
        for law in &universal_laws {
            require_referent(&referents, law.id(), "universal law")?;
            require_referent(&referents, law.scope(), "universal law scope")?;
            validate_pattern(&relational_contents, &relation_shapes, law.generalized())?;
        }
        for invariant in &invariants {
            require_referent(&referents, invariant.id(), "invariant")?;
            require_referent(&referents, invariant.scope(), "invariant scope")?;
            require_referent(&referents, invariant.policy(), "invariant admission policy")?;
            validate_pattern(
                &relational_contents,
                &relation_shapes,
                invariant.condition(),
            )?;
        }
        for goal in &goals {
            require_referent(&referents, goal.id(), "goal")?;
            require_referent(&referents, goal.context(), "goal planning context")?;
            validate_pattern(&relational_contents, &relation_shapes, goal.desired())?;
        }
        for transition in &transitions {
            require_referent(&referents, transition.id(), "transition")?;
            require_content(&relational_contents, transition.from(), "transition source")?;
            require_content(
                &relational_contents,
                transition.to(),
                "transition destination",
            )?;
        }
        for judgment in &judgments {
            require_referent(&referents, judgment.id(), "judgment")?;
            require_referent(&referents, judgment.authority(), "judgment authority")?;
            require_referent(&referents, judgment.scope(), "judgment scope")?;
            match judgment.target() {
                JudgmentTarget::Content(content) => {
                    require_content(&relational_contents, content, "judgment target")?;
                }
                JudgmentTarget::Occurrence(occurrence) if !occurrence_ids.contains(occurrence) => {
                    return Err(KernelError::new(
                        "judgment targets an undeclared assertion occurrence",
                    ));
                }
                JudgmentTarget::Occurrence(_) => {}
            }
            validate_judgment(
                &referents,
                &relational_contents,
                &occurrences,
                &derivation_rules,
                judgment,
            )?;
        }

        let admitted_ids = admitted_content_ids(&id, &occurrences, &judgments);
        if admitted_ids.iter().any(|id| {
            relational_contents
                .get(id)
                .is_some_and(|content| !content_is_ground(&relational_contents, content))
        }) {
            return Err(KernelError::new(
                "admitted relational content must be recursively ground",
            ));
        }
        let mut admitted_contents = admitted_ids
            .iter()
            .filter_map(|id| relational_contents.get(id).cloned())
            .collect::<Vec<_>>();
        admitted_contents.sort();
        admitted_contents.dedup();

        for content in relational_contents.values() {
            validate_admissibility(
                &relational_contents,
                &relation_shapes,
                &admitted_ids,
                content,
            )?;
        }

        Ok(Self {
            id,
            referents,
            relational_contents,
            relation_shapes,
            occurrences,
            definitions,
            derivation_rules,
            universal_laws,
            invariants,
            goals,
            transitions,
            judgments,
            admitted_contents,
        })
    }

    pub fn from_atoms(
        id: ReferentId,
        atoms: impl IntoIterator<Item = SemanticAtom>,
    ) -> Result<Self> {
        let mut referents = BTreeMap::new();
        let mut contents = BTreeMap::new();
        let mut shapes = BTreeMap::new();
        let mut occurrences = Vec::new();
        let mut definitions = Vec::new();
        let mut rules = Vec::new();
        let mut laws = Vec::new();
        let mut invariants = Vec::new();
        let mut goals = Vec::new();
        let mut transitions = Vec::new();
        let mut judgments = Vec::new();
        for atom in atoms {
            match atom {
                SemanticAtom::Referent(value) => {
                    insert(&mut referents, value.id().clone(), value, "referent")?
                }
                SemanticAtom::RelationalContent(value) => insert(
                    &mut contents,
                    value.id().clone(),
                    value,
                    "relational content",
                )?,
                SemanticAtom::RelationShape(value) => insert(
                    &mut shapes,
                    value.referent().clone(),
                    value,
                    "relation shape",
                )?,
                SemanticAtom::AssertionOccurrence(value) => occurrences.push(value),
                SemanticAtom::Definition(value) => definitions.push(value),
                SemanticAtom::DerivationRule(value) => rules.push(value),
                SemanticAtom::UniversalLaw(value) => laws.push(value),
                SemanticAtom::Invariant(value) => invariants.push(value),
                SemanticAtom::Goal(value) => goals.push(value),
                SemanticAtom::Transition(value) => transitions.push(value),
                SemanticAtom::Judgment(value) => judgments.push(value),
            }
        }
        Self::with_distinctions(
            id,
            referents,
            contents,
            shapes,
            occurrences,
            definitions,
            rules,
            laws,
            invariants,
            goals,
            transitions,
            judgments,
        )
    }

    pub fn atoms(&self) -> BTreeSet<SemanticAtom> {
        self.referents
            .values()
            .cloned()
            .map(SemanticAtom::Referent)
            .chain(
                self.relational_contents
                    .values()
                    .cloned()
                    .map(SemanticAtom::RelationalContent),
            )
            .chain(
                self.relation_shapes
                    .values()
                    .cloned()
                    .map(SemanticAtom::RelationShape),
            )
            .chain(
                self.occurrences
                    .iter()
                    .cloned()
                    .map(SemanticAtom::AssertionOccurrence),
            )
            .chain(
                self.definitions
                    .iter()
                    .cloned()
                    .map(SemanticAtom::Definition),
            )
            .chain(
                self.derivation_rules
                    .iter()
                    .cloned()
                    .map(SemanticAtom::DerivationRule),
            )
            .chain(
                self.universal_laws
                    .iter()
                    .cloned()
                    .map(SemanticAtom::UniversalLaw),
            )
            .chain(self.invariants.iter().cloned().map(SemanticAtom::Invariant))
            .chain(self.goals.iter().cloned().map(SemanticAtom::Goal))
            .chain(
                self.transitions
                    .iter()
                    .cloned()
                    .map(SemanticAtom::Transition),
            )
            .chain(self.judgments.iter().cloned().map(SemanticAtom::Judgment))
            .collect()
    }

    pub fn id(&self) -> &ReferentId {
        &self.id
    }
    pub fn referents(&self) -> &BTreeMap<ReferentId, Referent> {
        &self.referents
    }
    pub fn relational_contents(&self) -> &BTreeMap<ContentId, RelationalContent> {
        &self.relational_contents
    }
    pub fn relation_shapes(&self) -> &BTreeMap<ReferentId, RelationShape> {
        &self.relation_shapes
    }
    pub fn occurrences(&self) -> &[AssertionOccurrence] {
        &self.occurrences
    }
    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
    pub fn definition(&self, id: &ReferentId) -> Option<&Definition> {
        self.definitions
            .binary_search_by(|definition| definition.id().cmp(id))
            .ok()
            .map(|index| &self.definitions[index])
    }
    pub fn derivation_rules(&self) -> &[DerivationRule] {
        &self.derivation_rules
    }
    pub fn universal_laws(&self) -> &[UniversalLaw] {
        &self.universal_laws
    }
    pub fn invariants(&self) -> &[Invariant] {
        &self.invariants
    }
    pub fn goals(&self) -> &[Goal] {
        &self.goals
    }
    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }
    pub fn judgments(&self) -> &[Judgment] {
        &self.judgments
    }
    pub fn admitted_contents(&self) -> &[RelationalContent] {
        &self.admitted_contents
    }
    pub fn content(&self, id: &ContentId) -> Option<&RelationalContent> {
        self.relational_contents.get(id)
    }

    pub fn validate_content(
        &self,
        content: &RelationalContent,
        allow_patterns: bool,
    ) -> Result<()> {
        validate_content_structure(
            &self.referents,
            &self.relational_contents,
            &self.relation_shapes,
            content,
            true,
        )?;
        if !allow_patterns && !content_is_ground(&self.relational_contents, content) {
            return Err(KernelError::new("admitted content must be ground"));
        }
        validate_admissibility(
            &self.relational_contents,
            &self.relation_shapes,
            &self
                .admitted_contents
                .iter()
                .map(|item| item.id().clone())
                .collect(),
            content,
        )
    }

    pub fn term_is_ground(&self, term: &Term) -> bool {
        term_is_ground(&self.relational_contents, term, &mut BTreeSet::new())
    }

    pub fn content_is_ground(&self, content: &RelationalContent) -> bool {
        content_is_ground(&self.relational_contents, content)
    }

    pub fn status(
        &self,
        content: &RelationalContent,
        authority: &ReferentId,
        scope: &ReferentId,
    ) -> OpenWorldStatus {
        self.status_matching(content, |judgment| {
            judgment.authority() == authority && judgment.scope() == scope
        })
    }

    /// Resolve the status that controls this Model's executable projection.
    pub fn operative_status(&self, content: &RelationalContent) -> OpenWorldStatus {
        self.status(content, &self.id, &self.id)
    }

    pub fn aggregate_status(&self, content: &RelationalContent) -> OpenWorldStatus {
        self.status_matching(content, |_| true)
    }

    fn status_matching(
        &self,
        content: &RelationalContent,
        include: impl Fn(&Judgment) -> bool,
    ) -> OpenWorldStatus {
        open_world_status(content.id(), &self.occurrences, &self.judgments, include)
    }
}

fn open_world_status(
    content: &ContentId,
    occurrences: &[AssertionOccurrence],
    judgments: &[Judgment],
    include: impl Fn(&Judgment) -> bool,
) -> OpenWorldStatus {
    let mut admitted = false;
    let mut rejected = false;
    let mut disputed = false;
    for judgment in judgments {
        if !include(judgment) || !judgment_targets(judgment, content, occurrences) {
            continue;
        }
        match judgment.status() {
            JudgmentStatus::Disputed => disputed = true,
            JudgmentStatus::Withdrawn => {}
            JudgmentStatus::Affirmed => match judgment.kind() {
                JudgmentKind::Admitted { .. } => admitted = true,
                JudgmentKind::Rejected { .. } => rejected = true,
                _ => {}
            },
        }
    }
    if disputed || (admitted && rejected) {
        OpenWorldStatus::Disputed
    } else if admitted {
        OpenWorldStatus::Admitted
    } else if rejected {
        OpenWorldStatus::Rejected
    } else {
        OpenWorldStatus::Undetermined
    }
}

fn validate_keyed<K: Ord, V>(
    map: &BTreeMap<K, V>,
    id: impl Fn(&V) -> &K,
    where_: &str,
) -> Result<()> {
    if map.iter().any(|(key, value)| key != id(value)) {
        Err(KernelError::new(format!(
            "{where_} map key does not match identity"
        )))
    } else {
        Ok(())
    }
}

fn insert<K: Ord, V>(map: &mut BTreeMap<K, V>, key: K, value: V, where_: &str) -> Result<()> {
    if map.insert(key, value).is_some() {
        Err(KernelError::new(format!("duplicate {where_} identity")))
    } else {
        Ok(())
    }
}

fn require_referent(
    referents: &BTreeMap<ReferentId, Referent>,
    id: &ReferentId,
    where_: &str,
) -> Result<()> {
    if referents.contains_key(id) {
        Ok(())
    } else {
        Err(KernelError::new(format!(
            "{where_} names an undeclared referent"
        )))
    }
}

fn require_content(
    contents: &BTreeMap<ContentId, RelationalContent>,
    id: &ContentId,
    where_: &str,
) -> Result<()> {
    if contents.contains_key(id) {
        Ok(())
    } else {
        Err(KernelError::new(format!(
            "{where_} names undeclared relational content"
        )))
    }
}

fn validate_predicate(
    referents: &BTreeMap<ReferentId, Referent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    predicate: &RolePredicate,
) -> Result<()> {
    let shape = shapes
        .get(predicate.relation())
        .ok_or_else(|| KernelError::new("role predicate relation has no shape"))?;
    let supplied = predicate
        .fixed_roles()
        .keys()
        .chain(std::iter::once(predicate.candidate_role()))
        .collect::<BTreeSet<_>>();
    if supplied != shape.roles().keys().collect::<BTreeSet<_>>() {
        return Err(KernelError::new(
            "role predicate must fill exactly one candidate and every fixed role",
        ));
    }
    for fixed in predicate.fixed_roles().values() {
        require_referent(referents, fixed, "role predicate")?;
    }
    Ok(())
}

fn application_result_role<'a>(
    shapes: &'a BTreeMap<ReferentId, RelationShape>,
    content: &RelationalContent,
) -> Result<&'a Role> {
    let shape = shapes
        .get(content.relation())
        .ok_or_else(|| KernelError::new("relational-position referent has no admitted shape"))?;
    let supplied = content.roles().keys().cloned().collect::<BTreeSet<_>>();
    let matching = shape
        .lookup()
        .iter()
        .filter(|mode| mode.known().iter().cloned().collect::<BTreeSet<_>>() == supplied)
        .collect::<Vec<_>>();
    let [mode] = matching.as_slice() else {
        return Err(KernelError::new(
            "recursive term must match exactly one lookup contract by its known roles",
        ));
    };
    if mode.cardinality() != &Cardinality::One || mode.sought().len() != 1 {
        return Err(KernelError::new(
            "recursive term lookup contract must produce exactly one sought role",
        ));
    }
    Ok(&shape.roles()[&mode.sought()[0]])
}

fn validate_content_structure(
    referents: &BTreeMap<ReferentId, Referent>,
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    content: &RelationalContent,
    require_complete: bool,
) -> Result<()> {
    let shape = shapes
        .get(content.relation())
        .ok_or_else(|| KernelError::new("relational-position referent has no admitted shape"))?;
    if require_complete && content.roles().keys().ne(shape.roles().keys()) {
        return Err(KernelError::new(
            "relational content must fill the complete named role map",
        ));
    }
    if !require_complete {
        application_result_role(shapes, content)?;
    }
    for term in content.roles().values() {
        validate_term(referents, contents, term, true)?;
    }
    Ok(())
}

fn validate_application_acyclic(contents: &BTreeMap<ContentId, RelationalContent>) -> Result<()> {
    fn visit(
        id: &ContentId,
        contents: &BTreeMap<ContentId, RelationalContent>,
        active: &mut BTreeSet<ContentId>,
        settled: &mut BTreeSet<ContentId>,
    ) -> Result<()> {
        if settled.contains(id) {
            return Ok(());
        }
        if !active.insert(id.clone()) {
            return Err(KernelError::new(
                "recursive term application graph contains a cycle",
            ));
        }
        let content = contents
            .get(id)
            .ok_or_else(|| KernelError::new("recursive term names undeclared content"))?;
        let mut dependencies = Vec::new();
        for term in content.roles().values() {
            term.walk(&mut |term| {
                if let Term::Application(id) = term {
                    dependencies.push(id.clone());
                }
            });
        }
        for dependency in dependencies {
            visit(&dependency, contents, active, settled)?;
        }
        active.remove(id);
        settled.insert(id.clone());
        Ok(())
    }

    let mut active = BTreeSet::new();
    let mut settled = BTreeSet::new();
    for id in contents.keys() {
        visit(id, contents, &mut active, &mut settled)?;
    }
    Ok(())
}

fn validate_term(
    referents: &BTreeMap<ReferentId, Referent>,
    contents: &BTreeMap<ContentId, RelationalContent>,
    term: &Term,
    allow_patterns: bool,
) -> Result<()> {
    term.validate_structure()?;
    let mut result = Ok(());
    term.walk(&mut |term| {
        if result.is_err() {
            return;
        }
        result = match term {
            Term::Referent(id) => require_referent(referents, id, "term"),
            Term::Application(id) => require_content(contents, id, "recursive term"),
            Term::Pattern(_) if allow_patterns => Ok(()),
            Term::Pattern(_) => Err(KernelError::new("pattern is not valid in ground content")),
            Term::F32(_)
            | Term::Int(_)
            | Term::Bool(_)
            | Term::Product(_)
            | Term::Sum { .. }
            | Term::Sequence(_) => Ok(()),
        };
    });
    result
}

fn content_is_ground(
    contents: &BTreeMap<ContentId, RelationalContent>,
    content: &RelationalContent,
) -> bool {
    let mut visiting = BTreeSet::new();
    visiting.insert(content.id().clone());
    content
        .roles()
        .values()
        .all(|term| term_is_ground(contents, term, &mut visiting))
}

fn term_is_ground(
    contents: &BTreeMap<ContentId, RelationalContent>,
    term: &Term,
    visiting: &mut BTreeSet<ContentId>,
) -> bool {
    match term {
        Term::Referent(_) | Term::F32(_) | Term::Int(_) | Term::Bool(_) => true,
        Term::Pattern(_) => false,
        Term::Application(id) => {
            let Some(content) = contents.get(id) else {
                return false;
            };
            if !visiting.insert(id.clone()) {
                return false;
            }
            let ground = content
                .roles()
                .values()
                .all(|term| term_is_ground(contents, term, visiting));
            visiting.remove(id);
            ground
        }
        Term::Product(fields) => fields
            .values()
            .all(|term| term_is_ground(contents, term, visiting)),
        Term::Sum { value, .. } => term_is_ground(contents, value, visiting),
        Term::Sequence(values) => values
            .iter()
            .all(|term| term_is_ground(contents, term, visiting)),
    }
}

fn admitted_content_ids(
    operative: &ReferentId,
    occurrences: &[AssertionOccurrence],
    judgments: &[Judgment],
) -> BTreeSet<ContentId> {
    let candidates = judgments
        .iter()
        .filter(|judgment| judgment.authority() == operative && judgment.scope() == operative)
        .filter_map(|judgment| match judgment.target() {
            JudgmentTarget::Content(id) => Some(id.clone()),
            JudgmentTarget::Occurrence(id) => occurrences
                .iter()
                .find(|item| item.id() == id)
                .map(|item| item.content().clone()),
        })
        .collect::<BTreeSet<_>>();
    candidates
        .into_iter()
        .filter(|content| {
            open_world_status(content, occurrences, judgments, |judgment| {
                judgment.authority() == operative && judgment.scope() == operative
            }) == OpenWorldStatus::Admitted
        })
        .collect()
}

fn validate_admissibility(
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    admitted: &BTreeSet<ContentId>,
    content: &RelationalContent,
) -> Result<()> {
    let shape = &shapes[content.relation()];
    for (role_id, term) in content.roles() {
        match term {
            Term::Referent(candidate) => {
                for predicate in shape.roles()[role_id].admissibility() {
                    let satisfied = admitted
                        .iter()
                        .filter_map(|id| contents.get(id))
                        .any(|fact| {
                            fact.relation() == predicate.relation()
                                && fact.roles().get(predicate.candidate_role())
                                    == Some(&Term::Referent(candidate.clone()))
                                && predicate.fixed_roles().iter().all(|(role, fixed)| {
                                    fact.roles().get(role) == Some(&Term::Referent(fixed.clone()))
                                })
                        });
                    if !satisfied {
                        return Err(KernelError::new(
                            "role referent does not satisfy its relational admissibility predicates",
                        ));
                    }
                }
            }
            Term::Application(id) => {
                let target = contents
                    .get(id)
                    .ok_or_else(|| KernelError::new("recursive term names undeclared content"))?;
                let result = application_result_role(shapes, target)?;
                if result.admissibility() != shape.roles()[role_id].admissibility() {
                    return Err(KernelError::new(
                        "recursive term result does not satisfy the containing role admissibility contract",
                    ));
                }
            }
            Term::Pattern(_) => {}
            Term::F32(_)
            | Term::Int(_)
            | Term::Bool(_)
            | Term::Product(_)
            | Term::Sum { .. }
            | Term::Sequence(_) => {
                if !shape.roles()[role_id].admissibility().is_empty() {
                    return Err(KernelError::new(
                        "structural term cannot satisfy relational admissibility predicates",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_rule(
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    rule: &DerivationRule,
) -> Result<()> {
    validate_pattern(contents, shapes, rule.premises())?;
    let mut premises = BTreeSet::new();
    let mut requirements = BTreeMap::<PatternId, Vec<RolePredicate>>::new();
    for id in rule.premises().forms() {
        let content = contents
            .get(id)
            .ok_or_else(|| KernelError::new("derivation premise is undeclared"))?;
        record_patterns(content, shapes, &mut requirements, Some(&mut premises))?;
    }
    validate_pattern(contents, shapes, rule.conclusion())?;
    for id in rule.conclusion().forms() {
        let conclusion = contents
            .get(id)
            .ok_or_else(|| KernelError::new("derivation conclusion is undeclared"))?;
        record_patterns(conclusion, shapes, &mut requirements, None)?;
    }
    if rule
        .conclusion()
        .forms()
        .iter()
        .filter_map(|id| contents.get(id))
        .flat_map(|content| content.roles().values())
        .filter_map(Term::pattern_id)
        .any(|id| !premises.contains(id))
    {
        return Err(KernelError::new(
            "every conclusion pattern must occur in a premise",
        ));
    }
    Ok(())
}

fn validate_pattern(
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    pattern: &Pattern,
) -> Result<()> {
    let mut requirements = BTreeMap::<PatternId, Vec<RolePredicate>>::new();
    for id in pattern.forms() {
        let content = contents
            .get(id)
            .ok_or_else(|| KernelError::new("pattern names undeclared relational content"))?;
        record_patterns(content, shapes, &mut requirements, None)?;
    }
    Ok(())
}

fn record_patterns(
    content: &RelationalContent,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    requirements: &mut BTreeMap<PatternId, Vec<RolePredicate>>,
    mut seen: Option<&mut BTreeSet<PatternId>>,
) -> Result<()> {
    for (role, term) in content.roles() {
        let Term::Pattern(id) = term else { continue };
        let current = shapes[content.relation()].roles()[role]
            .admissibility()
            .to_vec();
        if requirements
            .insert(id.clone(), current.clone())
            .is_some_and(|previous| previous != current)
        {
            return Err(KernelError::new(
                "pattern occurs under inconsistent role admissibility",
            ));
        }
        if let Some(seen) = seen.as_deref_mut() {
            seen.insert(id.clone());
        }
    }
    Ok(())
}

fn validate_judgment(
    referents: &BTreeMap<ReferentId, Referent>,
    contents: &BTreeMap<ContentId, RelationalContent>,
    occurrences: &[AssertionOccurrence],
    rules: &[DerivationRule],
    judgment: &Judgment,
) -> Result<()> {
    match judgment.kind() {
        JudgmentKind::Declared => Ok(()),
        JudgmentKind::Derived { rule, premises } => {
            let rule = rules
                .iter()
                .find(|candidate| candidate.id() == rule)
                .ok_or_else(|| {
                    KernelError::new("derived judgment names an undeclared derivation rule")
                })?;
            if judgment.authority() != rule.authority() || judgment.scope() != rule.scope() {
                return Err(KernelError::new(
                    "derived judgment authority and scope must match its derivation rule",
                ));
            }
            let target = judgment_target_content(judgment.target(), contents, occurrences)?;
            if !derivation_instantiates(rule, premises, target, contents)? {
                return Err(KernelError::new(
                    "derived judgment basis and target do not instantiate its derivation rule",
                ));
            }
            Ok(())
        }
        JudgmentKind::Observed { evidence } => {
            require_referent(referents, evidence, "observation evidence")
        }
        JudgmentKind::Admitted { policy, basis } | JudgmentKind::Rejected { policy, basis } => {
            require_referent(referents, policy, "judgment policy")?;
            for item in basis {
                require_referent(referents, item, "judgment basis")?;
            }
            Ok(())
        }
        JudgmentKind::Superseded { by } => {
            if !occurrences.iter().any(|occurrence| occurrence.id() == by) {
                return Err(KernelError::new(
                    "superseding occurrence is not an assertion occurrence",
                ));
            }
            match judgment.target() {
                JudgmentTarget::Occurrence(target) if target != by => Ok(()),
                JudgmentTarget::Occurrence(_) => Err(KernelError::new(
                    "an assertion occurrence cannot supersede itself",
                )),
                JudgmentTarget::Content(_) => Err(KernelError::new(
                    "supersession judgment must target an assertion occurrence",
                )),
            }
        }
    }
}

fn judgment_target_content<'a>(
    target: &JudgmentTarget,
    contents: &'a BTreeMap<ContentId, RelationalContent>,
    occurrences: &[AssertionOccurrence],
) -> Result<&'a RelationalContent> {
    let id = match target {
        JudgmentTarget::Content(id) => id,
        JudgmentTarget::Occurrence(id) => occurrences
            .iter()
            .find(|occurrence| occurrence.id() == id)
            .ok_or_else(|| KernelError::new("judgment targets an undeclared assertion occurrence"))?
            .content(),
    };
    contents
        .get(id)
        .ok_or_else(|| KernelError::new("judgment target names undeclared relational content"))
}

fn derivation_instantiates(
    rule: &DerivationRule,
    premise_ids: &[ContentId],
    target: &RelationalContent,
    contents: &BTreeMap<ContentId, RelationalContent>,
) -> Result<bool> {
    if premise_ids.len() != rule.premises().forms().len() || !content_is_ground(contents, target) {
        return Ok(false);
    }
    let premises = premise_ids
        .iter()
        .map(|id| {
            contents
                .get(id)
                .ok_or_else(|| KernelError::new("derived judgment premise is undeclared"))
        })
        .collect::<Result<Vec<_>>>()?;
    if premises
        .iter()
        .any(|content| !content_is_ground(contents, content))
    {
        return Ok(false);
    }
    let patterns = rule
        .premises()
        .forms()
        .iter()
        .map(|id| &contents[id])
        .collect::<Vec<_>>();
    Ok(match_premises(
        &patterns,
        &premises,
        rule.conclusion()
            .forms()
            .iter()
            .map(|id| &contents[id])
            .collect::<Vec<_>>()
            .as_slice(),
        target,
        0,
        &mut BTreeSet::new(),
        &BTreeMap::new(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn match_premises(
    patterns: &[&RelationalContent],
    premises: &[&RelationalContent],
    conclusions: &[&RelationalContent],
    target: &RelationalContent,
    index: usize,
    used: &mut BTreeSet<usize>,
    substitution: &BTreeMap<PatternId, Term>,
) -> bool {
    if index == patterns.len() {
        let final_substitution = substitution.clone();
        return conclusions.iter().any(|conclusion| {
            let mut candidate = final_substitution.clone();
            match_form(conclusion, target, &mut candidate, false)
        });
    }
    for (candidate, premise) in premises.iter().enumerate() {
        if used.contains(&candidate) {
            continue;
        }
        let mut next = substitution.clone();
        if !match_form(patterns[index], premise, &mut next, true) {
            continue;
        }
        used.insert(candidate);
        if match_premises(
            patterns,
            premises,
            conclusions,
            target,
            index + 1,
            used,
            &next,
        ) {
            return true;
        }
        used.remove(&candidate);
    }
    false
}

fn match_form(
    pattern: &RelationalContent,
    actual: &RelationalContent,
    substitution: &mut BTreeMap<PatternId, Term>,
    allow_new_bindings: bool,
) -> bool {
    if pattern.relation() != actual.relation() || pattern.roles().keys().ne(actual.roles().keys()) {
        return false;
    }
    for (role, expected) in pattern.roles() {
        let actual = &actual.roles()[role];
        match expected {
            Term::Pattern(id) => match substitution.get(id) {
                Some(bound) if bound != actual => return false,
                Some(_) => {}
                None if allow_new_bindings => {
                    substitution.insert(id.clone(), actual.clone());
                }
                None => return false,
            },
            Term::Referent(_)
            | Term::Application(_)
            | Term::F32(_)
            | Term::Int(_)
            | Term::Bool(_)
            | Term::Product(_)
            | Term::Sum { .. }
            | Term::Sequence(_)
                if expected != actual =>
            {
                return false;
            }
            Term::Referent(_)
            | Term::Application(_)
            | Term::F32(_)
            | Term::Int(_)
            | Term::Bool(_)
            | Term::Product(_)
            | Term::Sum { .. }
            | Term::Sequence(_) => {}
        }
    }
    true
}

fn judgment_targets(
    judgment: &Judgment,
    content: &ContentId,
    occurrences: &[AssertionOccurrence],
) -> bool {
    match judgment.target() {
        JudgmentTarget::Content(id) => id == content,
        JudgmentTarget::Occurrence(id) => occurrences
            .iter()
            .any(|item| item.id() == id && item.content() == content),
    }
}

fn sort_unique_by<T, K: Ord + PartialEq>(
    values: &mut [T],
    id: impl Fn(&T) -> &K,
    where_: &str,
) -> Result<()> {
    values.sort_by(|left, right| id(left).cmp(id(right)));
    if values.windows(2).any(|pair| id(&pair[0]) == id(&pair[1])) {
        Err(KernelError::new(format!("duplicate {where_} identity")))
    } else {
        Ok(())
    }
}

fn ensure_unique_ids<'a>(ids: impl Iterator<Item = &'a ReferentId>, where_: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    if ids.cloned().any(|id| !seen.insert(id)) {
        Err(KernelError::new(format!("duplicate {where_} identity")))
    } else {
        Ok(())
    }
}
