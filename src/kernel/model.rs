use std::collections::{BTreeMap, BTreeSet};

use crate::intrinsic::{Intrinsic, IntrinsicRole};

use super::{
    clause::{
        AssertionOccurrence, Definition, DerivationRule, Goal, Invariant, Judgment, JudgmentKind,
        JudgmentStatus, JudgmentTarget, OpenWorldStatus, Pattern, RelationalContent, Term,
        Transition, UniversalLaw,
    },
    error::{
        KernelError, ProposalPath, ProposalPathSegment, ProposalSubject, Result,
        StructuralFailureClass,
    },
    identity::{ContentId, Name, PatternId, ReferentId},
    schema::{
        Cardinality, Referent, RelationShape, Role, RolePredicate, StructuralContract,
        StructuralForm, membership_group_role, membership_member_role, membership_relation,
    },
};

/// Every signed semantic constituent of a Model snapshot.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SemanticAtom {
    Referent(Referent),
    RelationalContent(RelationalContent),
    RelationShape(RelationShape),
    StructuralContract(StructuralContract),
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
    structural_contracts: BTreeMap<ReferentId, StructuralContract>,
    structural_referents: BTreeMap<StructuralForm, Vec<ReferentId>>,
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
        structural_contracts: BTreeMap<ReferentId, StructuralContract>,
        occurrences: Vec<AssertionOccurrence>,
        derivation_rules: Vec<DerivationRule>,
    ) -> Result<Self> {
        Self::with_distinctions(
            id,
            referents,
            relational_contents,
            relation_shapes,
            structural_contracts,
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
        structural_contracts: BTreeMap<ReferentId, StructuralContract>,
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
        validate_keyed(
            &structural_contracts,
            StructuralContract::referent,
            "structural contract",
        )?;
        require_referent(&referents, &id, "Model identity")?;

        for shape in relation_shapes.values() {
            require_referent(&referents, shape.referent(), "relational-position")?;
            for role in shape.roles().values() {
                for predicate in role.admissibility() {
                    validate_predicate(&referents, &relation_shapes, predicate)?;
                }
            }
        }
        for contract in structural_contracts.values() {
            validate_structural_contract(&referents, contract)?;
        }
        let mut structural_referents = BTreeMap::<StructuralForm, Vec<ReferentId>>::new();
        for contract in structural_contracts.values() {
            structural_referents
                .entry(contract.form().clone())
                .or_default()
                .push(contract.referent().clone());
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
            .chain(
                universal_laws
                    .iter()
                    .flat_map(|law| [law.premises(), law.conclusion()]),
            )
            .chain(invariants.iter().map(Invariant::condition))
            .chain(goals.iter().map(Goal::desired))
        {
            complete_contents.extend(pattern.forms().iter().cloned());
        }
        for transition in &transitions {
            complete_contents.insert(transition.from().clone());
            complete_contents.insert(transition.to().clone());
            complete_contents.extend(transition.guards().iter().cloned());
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
        validate_structural_contract_definitions(&structural_contracts, &definitions)?;
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
            require_referent(
                &referents,
                rule.governing_law(),
                "derivation rule governing law",
            )?;
            require_referent(&referents, rule.scope(), "derivation rule scope")?;
            require_referent(&referents, rule.authority(), "derivation rule authority")?;
            validate_rule(&relational_contents, &relation_shapes, rule)?;
            let law = universal_laws
                .iter()
                .find(|law| law.id() == rule.governing_law())
                .ok_or_else(|| KernelError::new("derivation rule governing law is undeclared"))?;
            if law.scope() != rule.scope()
                || law.premises() != rule.premises()
                || law.conclusion() != rule.conclusion()
            {
                return Err(KernelError::new(
                    "derivation rule must exactly project its governing law pattern and scope",
                ));
            }
        }
        for law in &universal_laws {
            require_referent(&referents, law.id(), "universal law")?;
            require_referent(&referents, law.scope(), "universal law scope")?;
            validate_pattern(&relational_contents, &relation_shapes, law.premises())?;
            validate_pattern(&relational_contents, &relation_shapes, law.conclusion())?;
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
            require_referent(&referents, transition.event(), "transition event")?;
            require_content(&relational_contents, transition.from(), "transition source")?;
            require_content(
                &relational_contents,
                transition.to(),
                "transition destination",
            )?;
            for guard in transition.guards() {
                require_content(&relational_contents, guard, "transition guard")?;
            }
        }
        let mut event_contracts = BTreeMap::<
            ReferentId,
            (Vec<PatternId>, BTreeSet<PatternId>, BTreeSet<PatternId>),
        >::new();
        for transition in &transitions {
            let contract = event_contracts
                .entry(transition.event().clone())
                .or_insert_with(|| {
                    (
                        transition.payload_bindings().to_vec(),
                        transition.payload_bindings().iter().cloned().collect(),
                        BTreeSet::new(),
                    )
                });
            if contract.0 != transition.payload_bindings() {
                return Err(KernelError::new(
                    "one checked event must use one ordered payload binding shape",
                ));
            }
            collect_content_patterns(
                transition.from(),
                &relational_contents,
                &mut contract.1,
                &mut BTreeSet::new(),
            );
            for guard in transition.guards() {
                collect_content_patterns(
                    guard,
                    &relational_contents,
                    &mut contract.1,
                    &mut BTreeSet::new(),
                );
            }
            collect_content_patterns(
                transition.to(),
                &relational_contents,
                &mut contract.2,
                &mut BTreeSet::new(),
            );
        }
        if event_contracts
            .values()
            .any(|(_, available, successors)| !successors.is_subset(available))
        {
            return Err(KernelError::new(
                "transition successor has a binder absent from its event payload and pre-state patterns",
            ));
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
            for (role, term) in content.roles() {
                let path = ProposalPath::new(ProposalSubject::Content(content.id().clone()))
                    .child(ProposalPathSegment::Role(role.clone()));
                validate_structural_term(
                    &structural_contracts,
                    &definitions,
                    &relational_contents,
                    &relation_shapes,
                    &admitted_ids,
                    &path,
                    term,
                )?;
            }
        }
        for definition in &definitions {
            let path = ProposalPath::new(ProposalSubject::Definition(definition.id().clone()));
            validate_structural_term(
                &structural_contracts,
                &definitions,
                &relational_contents,
                &relation_shapes,
                &admitted_ids,
                &path,
                definition.denotation(),
            )?;
        }

        for content in relational_contents.values() {
            validate_admissibility(
                &relational_contents,
                &relation_shapes,
                &structural_contracts,
                &definitions,
                &admitted_ids,
                content,
            )?;
        }

        Ok(Self {
            id,
            referents,
            relational_contents,
            relation_shapes,
            structural_contracts,
            structural_referents,
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
        let mut structural_contracts = BTreeMap::new();
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
                SemanticAtom::StructuralContract(value) => insert(
                    &mut structural_contracts,
                    value.referent().clone(),
                    value,
                    "structural contract",
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
            structural_contracts,
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
                self.structural_contracts
                    .values()
                    .cloned()
                    .map(SemanticAtom::StructuralContract),
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
    pub fn structural_contracts(&self) -> &BTreeMap<ReferentId, StructuralContract> {
        &self.structural_contracts
    }
    pub(crate) fn structural_referents(&self, form: &StructuralForm) -> &[ReferentId] {
        self.structural_referents
            .get(form)
            .map(Vec::as_slice)
            .unwrap_or_default()
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
            &self.structural_contracts,
            &self.definitions,
            &self
                .admitted_contents
                .iter()
                .map(|item| item.id().clone())
                .collect(),
            content,
        )
    }

    pub(crate) fn validate_query_content(
        &self,
        content: &RelationalContent,
        dependencies: &[RelationalContent],
    ) -> Result<()> {
        let mut contents = self.relational_contents.clone();
        let mut local = BTreeSet::new();
        for dependency in dependencies {
            if !local.insert(dependency.id().clone()) {
                return Err(KernelError::new(
                    "query application graph repeats a dependency identity",
                ));
            }
            if let Some(existing) = contents.insert(dependency.id().clone(), dependency.clone())
                && existing != *dependency
            {
                return Err(KernelError::new(
                    "query application graph conflicts with Model content",
                ));
            }
        }
        validate_application_acyclic(&contents)?;
        for dependency in dependencies {
            validate_content_structure(
                &self.referents,
                &contents,
                &self.relation_shapes,
                dependency,
                false,
            )?;
        }
        validate_content_structure(
            &self.referents,
            &contents,
            &self.relation_shapes,
            content,
            true,
        )?;

        let mut reachable = BTreeSet::new();
        collect_application_dependencies(content, &contents, &mut reachable)?;
        if !local.is_subset(&reachable) {
            return Err(KernelError::new(
                "query application graph contains an unreachable dependency",
            ));
        }

        let admitted = self
            .admitted_contents
            .iter()
            .map(|item| item.id().clone())
            .collect();
        for dependency in dependencies {
            validate_admissibility(
                &contents,
                &self.relation_shapes,
                &self.structural_contracts,
                &self.definitions,
                &admitted,
                dependency,
            )?;
        }
        validate_admissibility(
            &contents,
            &self.relation_shapes,
            &self.structural_contracts,
            &self.definitions,
            &admitted,
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

fn validate_structural_contract(
    referents: &BTreeMap<ReferentId, Referent>,
    contract: &StructuralContract,
) -> Result<()> {
    require_referent(referents, contract.referent(), "structural contract")?;
    match contract.form() {
        StructuralForm::Product(fields) => {
            for field in fields {
                require_referent(referents, field, "structural product field binding")?;
            }
        }
        StructuralForm::Tuple(domains) => {
            for domain in domains {
                require_referent(referents, domain, "structural tuple element domain")?;
            }
        }
        StructuralForm::F32 | StructuralForm::Int | StructuralForm::Bool => {}
    }
    Ok(())
}

fn validate_structural_contract_definitions(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
) -> Result<()> {
    for contract in contracts.values() {
        match contract.form() {
            StructuralForm::Product(fields) => {
                for field in fields {
                    let definition = definitions
                        .binary_search_by(|definition| definition.id().cmp(field))
                        .ok()
                        .map(|index| &definitions[index])
                        .ok_or_else(|| {
                            KernelError::new(
                                "structural product field has no exact binding definition",
                            )
                        })?;
                    let Some(domain) = definition.denotation().referent_id() else {
                        return Err(KernelError::new(
                            "structural product field binding must denote one domain referent",
                        ));
                    };
                    if !contracts.contains_key(domain) {
                        return Err(KernelError::new(
                            "structural product field domain has no sealed representation contract",
                        ));
                    }
                }
            }
            StructuralForm::Tuple(domains) => {
                if domains.iter().any(|domain| !contracts.contains_key(domain)) {
                    return Err(KernelError::new(
                        "structural tuple element domain has no sealed representation contract",
                    ));
                }
            }
            StructuralForm::F32 | StructuralForm::Int | StructuralForm::Bool => {}
        }
    }
    Ok(())
}

fn validate_structural_term(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    admitted: &BTreeSet<ContentId>,
    path: &ProposalPath,
    term: &Term,
) -> Result<()> {
    match term {
        Term::F32(_) => require_structural_form(contracts, StructuralForm::F32, path),
        Term::Int(_) => require_structural_form(contracts, StructuralForm::Int, path),
        Term::Bool(_) => require_structural_form(contracts, StructuralForm::Bool, path),
        Term::Product { shape, .. } => validate_term_against(
            contracts,
            definitions,
            contents,
            shapes,
            admitted,
            path,
            shape,
            term,
        ),
        Term::LabelledProduct { shape, .. } => validate_term_against(
            contracts,
            definitions,
            contents,
            shapes,
            admitted,
            path,
            shape,
            term,
        ),
        Term::Sequence {
            element, values, ..
        } => {
            for (index, value) in values.iter().enumerate() {
                let child = path.child(ProposalPathSegment::SequenceIndex(index));
                validate_term_against(
                    contracts,
                    definitions,
                    contents,
                    shapes,
                    admitted,
                    &child,
                    element,
                    value,
                )?;
            }
            Ok(())
        }
        Term::Sum { tag, value } => {
            let child = path.child(ProposalPathSegment::SumPayload(tag.clone()));
            validate_structural_term(
                contracts,
                definitions,
                contents,
                shapes,
                admitted,
                &child,
                value,
            )
        }
        Term::Referent(_) | Term::Pattern(_) | Term::Application(_) => Ok(()),
    }
}

fn require_structural_form(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    expected: StructuralForm,
    path: &ProposalPath,
) -> Result<()> {
    if contracts
        .values()
        .any(|contract| contract.form() == &expected)
    {
        Ok(())
    } else {
        Err(KernelError::structural(
            "structural scalar has no sealed representation contract",
            StructuralFailureClass::ContractUnavailable,
            path.clone(),
        ))
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_term_against(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    admitted: &BTreeSet<ContentId>,
    path: &ProposalPath,
    expected: &ReferentId,
    term: &Term,
) -> Result<()> {
    if matches!(term, Term::Pattern(_)) {
        return Ok(());
    }
    if let Term::Referent(candidate) = term {
        let membership = RelationalContent::new(
            membership_relation(),
            BTreeMap::from([
                (membership_member_role(), Term::referent(candidate.clone())),
                (membership_group_role(), Term::referent(expected.clone())),
            ]),
        )?;
        return if contents.get(membership.id()) == Some(&membership)
            && admitted.contains(membership.id())
        {
            Ok(())
        } else {
            Err(KernelError::structural(
                "structural term does not match its expected domain",
                StructuralFailureClass::DomainMismatch,
                path.clone(),
            ))
        };
    }
    if let Term::Application(id) = term {
        let application_path = path.child(ProposalPathSegment::Application(id.clone()));
        let target = contents
            .get(id)
            .ok_or_else(|| KernelError::new("recursive term names undeclared content"))?;
        return if application_result_domain(contracts, definitions, contents, shapes, target)?
            .as_ref()
            == Some(expected)
        {
            Ok(())
        } else {
            Err(KernelError::structural(
                "structural term does not match its expected domain",
                StructuralFailureClass::DomainMismatch,
                application_path,
            ))
        };
    }
    if let Some(contract) = contracts.get(expected) {
        return match (contract.form(), term) {
            (StructuralForm::F32, Term::F32(_))
            | (StructuralForm::Int, Term::Int(_))
            | (StructuralForm::Bool, Term::Bool(_)) => Ok(()),
            (StructuralForm::Tuple(required), Term::Product { shape, fields }) => {
                if shape != expected {
                    return Err(KernelError::structural(
                        "structural term does not match its expected domain",
                        StructuralFailureClass::DomainMismatch,
                        path.clone(),
                    ));
                }
                validate_inline_product(
                    contracts,
                    definitions,
                    contents,
                    shapes,
                    admitted,
                    path,
                    required,
                    fields,
                )
            }
            (StructuralForm::Product(required), Term::LabelledProduct { shape, fields }) => {
                if shape != expected {
                    return Err(KernelError::structural(
                        "structural term does not match its expected domain",
                        StructuralFailureClass::DomainMismatch,
                        path.clone(),
                    ));
                }
                if fields.keys().collect::<BTreeSet<_>>() != required.iter().collect() {
                    return Err(KernelError::structural(
                        "labelled product must fill its exact structural contract",
                        StructuralFailureClass::FieldSetMismatch,
                        path.clone(),
                    ));
                }
                for (field, value) in fields {
                    let definition = definitions
                        .binary_search_by(|definition| definition.id().cmp(field))
                        .ok()
                        .map(|index| &definitions[index])
                        .expect("structural product definitions were validated");
                    let domain = definition
                        .denotation()
                        .referent_id()
                        .expect("structural product definitions denote domains");
                    let child = path.child(ProposalPathSegment::ProductField(field.clone()));
                    validate_term_against(
                        contracts,
                        definitions,
                        contents,
                        shapes,
                        admitted,
                        &child,
                        domain,
                        value,
                    )
                    .map_err(|error| {
                        error.with_message(
                            "labelled product field does not satisfy its bound domain",
                        )
                    })?;
                }
                Ok(())
            }
            _ => Err(KernelError::structural(
                "structural term does not match its expected domain",
                StructuralFailureClass::DomainMismatch,
                path.clone(),
            )),
        };
    }
    match term {
        Term::Sequence {
            shape,
            element,
            values,
        } if shape == expected => {
            for (index, value) in values.iter().enumerate() {
                let child = path.child(ProposalPathSegment::SequenceIndex(index));
                validate_term_against(
                    contracts,
                    definitions,
                    contents,
                    shapes,
                    admitted,
                    &child,
                    element,
                    value,
                )?;
            }
            Ok(())
        }
        _ => Err(KernelError::structural(
            "structural term does not match its expected domain",
            StructuralFailureClass::DomainMismatch,
            path.clone(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_inline_product(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    admitted: &BTreeSet<ContentId>,
    path: &ProposalPath,
    required: &[ReferentId],
    fields: &BTreeMap<Name, super::clause::ProductField>,
) -> Result<()> {
    if fields.len() != required.len() {
        return Err(KernelError::structural(
            "tuple does not fill its exact structural contract",
            StructuralFailureClass::FieldSetMismatch,
            path.clone(),
        ));
    }
    for (index, (label, field)) in fields.iter().enumerate() {
        let expected_label =
            Name::new(format!("_{index:020}")).expect("fixed-width ordinal tuple label is valid");
        if label != &expected_label {
            return Err(KernelError::structural(
                "tuple does not use canonical structural positions",
                StructuralFailureClass::NonCanonicalPosition,
                path.child(ProposalPathSegment::TupleIndex(index)),
            ));
        }
        let child = path.child(ProposalPathSegment::TupleIndex(index));
        if field.domain() != &required[index] {
            return Err(KernelError::structural(
                "tuple field does not satisfy its bound domain",
                StructuralFailureClass::DomainMismatch,
                child,
            ));
        }
        validate_term_against(
            contracts,
            definitions,
            contents,
            shapes,
            admitted,
            &child,
            field.domain(),
            field.value(),
        )
        .map_err(|error| error.with_message("tuple field does not satisfy its bound domain"))?;
    }
    Ok(())
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum NumericRepresentation {
    F32,
    Int,
    TupleF32,
    TupleInt,
}

fn application_result_domain(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    content: &RelationalContent,
) -> Result<Option<ReferentId>> {
    application_result_domain_inner(
        contracts,
        definitions,
        contents,
        shapes,
        &mut BTreeSet::new(),
        content,
    )
}

fn application_result_domain_inner(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    active_definitions: &mut BTreeSet<ReferentId>,
    content: &RelationalContent,
) -> Result<Option<ReferentId>> {
    let result = application_result_role(shapes, content)?;
    if let Some(domain) = structural_role_domain(result)? {
        return Ok(Some(domain.clone()));
    }
    let Some(intrinsic) = Intrinsic::from_relation(content.relation()) else {
        return Ok(None);
    };
    if result.id() != &intrinsic.role(IntrinsicRole::Result)
        || content.roles().keys().cloned().collect::<BTreeSet<_>>()
            != intrinsic
                .input_roles()
                .iter()
                .map(|role| intrinsic.role(*role))
                .collect::<BTreeSet<_>>()
    {
        return Ok(None);
    }
    intrinsic_result_domain(
        contracts,
        definitions,
        contents,
        shapes,
        active_definitions,
        content,
        intrinsic,
    )
}

fn intrinsic_result_domain(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    active_definitions: &mut BTreeSet<ReferentId>,
    content: &RelationalContent,
    intrinsic: Intrinsic,
) -> Result<Option<ReferentId>> {
    let mut domain = |role| {
        let term = content
            .roles()
            .get(&intrinsic.role(role))
            .expect("exact intrinsic roles precede result-domain derivation");
        structural_term_domain(
            contracts,
            definitions,
            contents,
            shapes,
            active_definitions,
            term,
        )
    };
    let unique = |form| unique_structural_referent(contracts, form).cloned();
    match intrinsic {
        Intrinsic::Add | Intrinsic::Subtract => {
            let left = domain(IntrinsicRole::Left)?;
            let right = domain(IntrinsicRole::Right)?;
            Ok(match (left, right) {
                (Some(left), Some(right))
                    if left == right && numeric_representation(contracts, &left).is_some() =>
                {
                    Some(left)
                }
                _ => None,
            })
        }
        Intrinsic::Multiply | Intrinsic::Divide => {
            let left = domain(IntrinsicRole::Left)?;
            let right = domain(IntrinsicRole::Right)?;
            Ok(match (left, right) {
                (Some(left), Some(right))
                    if left == right
                        && matches!(
                            numeric_representation(contracts, &left),
                            Some(NumericRepresentation::F32 | NumericRepresentation::Int)
                        ) =>
                {
                    Some(left)
                }
                (Some(left), Some(right))
                    if matches!(
                        (
                            numeric_representation(contracts, &left),
                            numeric_representation(contracts, &right),
                        ),
                        (
                            Some(NumericRepresentation::TupleF32),
                            Some(NumericRepresentation::F32),
                        ) | (
                            Some(NumericRepresentation::TupleInt),
                            Some(NumericRepresentation::Int),
                        )
                    ) =>
                {
                    Some(left)
                }
                _ => None,
            })
        }
        Intrinsic::LessThan
        | Intrinsic::LessOrEqual
        | Intrinsic::GreaterThan
        | Intrinsic::GreaterOrEqual => {
            let left = domain(IntrinsicRole::Left)?;
            let right = domain(IntrinsicRole::Right)?;
            Ok(
                if left == right
                    && left.as_ref().is_some_and(|domain| {
                        matches!(
                            numeric_representation(contracts, domain),
                            Some(NumericRepresentation::F32 | NumericRepresentation::Int)
                        )
                    })
                {
                    unique(&StructuralForm::Bool)
                } else {
                    None
                },
            )
        }
        Intrinsic::Equal | Intrinsic::NotEqual => {
            let left = domain(IntrinsicRole::Left)?;
            let right = domain(IntrinsicRole::Right)?;
            Ok(match (left, right) {
                (Some(left), Some(right)) if left == right => unique(&StructuralForm::Bool),
                _ => None,
            })
        }
        Intrinsic::Length => Ok(
            if domain(IntrinsicRole::Input)?
                .as_ref()
                .is_some_and(|domain| {
                    matches!(
                        numeric_representation(contracts, domain),
                        Some(NumericRepresentation::TupleF32 | NumericRepresentation::TupleInt)
                    )
                })
            {
                unique(&StructuralForm::F32)
            } else {
                None
            },
        ),
        Intrinsic::Conditional => {
            let condition = domain(IntrinsicRole::Condition)?;
            let then_domain = domain(IntrinsicRole::Then)?;
            let else_domain = domain(IntrinsicRole::Else)?;
            Ok(
                if condition.as_ref()
                    == unique_structural_referent(contracts, &StructuralForm::Bool)
                    && then_domain == else_domain
                {
                    then_domain
                } else {
                    None
                },
            )
        }
        Intrinsic::Map => {
            let mapper = content.roles().get(&intrinsic.role(IntrinsicRole::Mapper));
            let sequence = content
                .roles()
                .get(&intrinsic.role(IntrinsicRole::Sequence));
            let f32 = unique(&StructuralForm::F32);
            Ok(match (mapper, sequence, f32) {
                (Some(Term::Referent(mapper)), Some(Term::Sequence { element, .. }), Some(f32))
                    if mapper == &Intrinsic::Length.callable_identity()
                        && matches!(
                            numeric_representation(contracts, element),
                            Some(NumericRepresentation::TupleF32 | NumericRepresentation::TupleInt)
                        ) =>
                {
                    Some(super::schema::structural_sequence_domain(&f32))
                }
                _ => None,
            })
        }
    }
}

fn structural_term_domain(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    active_definitions: &mut BTreeSet<ReferentId>,
    term: &Term,
) -> Result<Option<ReferentId>> {
    match term {
        Term::Application(id) => contents
            .get(id)
            .map(|content| {
                application_result_domain_inner(
                    contracts,
                    definitions,
                    contents,
                    shapes,
                    active_definitions,
                    content,
                )
            })
            .transpose()
            .map(Option::flatten),
        Term::Referent(id) => {
            let Some(definition) = definitions
                .binary_search_by(|definition| definition.id().cmp(id))
                .ok()
                .map(|index| &definitions[index])
            else {
                return Ok(None);
            };
            if !active_definitions.insert(id.clone()) {
                return Ok(None);
            }
            let domain = structural_term_domain(
                contracts,
                definitions,
                contents,
                shapes,
                active_definitions,
                definition.denotation(),
            );
            active_definitions.remove(id);
            domain
        }
        Term::F32(_) => Ok(unique_structural_referent(contracts, &StructuralForm::F32).cloned()),
        Term::Int(_) => Ok(unique_structural_referent(contracts, &StructuralForm::Int).cloned()),
        Term::Bool(_) => Ok(unique_structural_referent(contracts, &StructuralForm::Bool).cloned()),
        Term::Product { shape, .. }
            if matches!(
                contracts.get(shape).map(StructuralContract::form),
                Some(StructuralForm::Tuple(_))
            ) =>
        {
            Ok(Some(shape.clone()))
        }
        Term::LabelledProduct { shape, .. }
            if matches!(
                contracts.get(shape).map(StructuralContract::form),
                Some(StructuralForm::Product(_))
            ) =>
        {
            Ok(Some(shape.clone()))
        }
        Term::Sequence { shape, .. } => Ok(Some(shape.clone())),
        Term::Pattern(_)
        | Term::Product { .. }
        | Term::LabelledProduct { .. }
        | Term::Sum { .. } => Ok(None),
    }
}

fn unique_structural_referent<'a>(
    contracts: &'a BTreeMap<ReferentId, StructuralContract>,
    form: &StructuralForm,
) -> Option<&'a ReferentId> {
    let mut matches = contracts
        .values()
        .filter(|contract| contract.form() == form)
        .map(StructuralContract::referent);
    let referent = matches.next()?;
    matches.next().is_none().then_some(referent)
}

fn numeric_representation(
    contracts: &BTreeMap<ReferentId, StructuralContract>,
    domain: &ReferentId,
) -> Option<NumericRepresentation> {
    match contracts.get(domain)?.form() {
        StructuralForm::F32 => Some(NumericRepresentation::F32),
        StructuralForm::Int => Some(NumericRepresentation::Int),
        StructuralForm::Tuple(domains) => {
            let mut representations =
                domains
                    .iter()
                    .map(|domain| match contracts.get(domain)?.form() {
                        StructuralForm::F32 => Some(NumericRepresentation::F32),
                        StructuralForm::Int => Some(NumericRepresentation::Int),
                        StructuralForm::Bool
                        | StructuralForm::Tuple(_)
                        | StructuralForm::Product(_) => None,
                    });
            match representations.next().flatten()? {
                NumericRepresentation::F32
                    if representations.all(|item| item == Some(NumericRepresentation::F32)) =>
                {
                    Some(NumericRepresentation::TupleF32)
                }
                NumericRepresentation::Int
                    if representations.all(|item| item == Some(NumericRepresentation::Int)) =>
                {
                    Some(NumericRepresentation::TupleInt)
                }
                NumericRepresentation::F32
                | NumericRepresentation::Int
                | NumericRepresentation::TupleF32
                | NumericRepresentation::TupleInt => None,
            }
        }
        StructuralForm::Bool | StructuralForm::Product(_) => None,
    }
}

fn structural_role_domain(role: &Role) -> Result<Option<&ReferentId>> {
    if role.admissibility().is_empty() {
        return Ok(None);
    }
    let [predicate] = role.admissibility() else {
        return Err(KernelError::new(
            "structural term requires one exact domain predicate",
        ));
    };
    if predicate.relation() != &membership_relation()
        || predicate.candidate_role() != &membership_member_role()
        || predicate.fixed_roles().len() != 1
    {
        return Err(KernelError::new(
            "structural term requires one exact domain predicate",
        ));
    }
    predicate
        .fixed_roles()
        .get(&membership_group_role())
        .map(Some)
        .ok_or_else(|| KernelError::new("structural term requires one exact domain predicate"))
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

fn collect_application_dependencies(
    content: &RelationalContent,
    contents: &BTreeMap<ContentId, RelationalContent>,
    reachable: &mut BTreeSet<ContentId>,
) -> Result<()> {
    let mut dependencies = Vec::new();
    for term in content.roles().values() {
        term.walk(&mut |term| {
            if let Term::Application(id) = term {
                dependencies.push(id.clone());
            }
        });
    }
    for id in dependencies {
        if !reachable.insert(id.clone()) {
            continue;
        }
        let dependency = contents
            .get(&id)
            .ok_or_else(|| KernelError::new("query term names undeclared content"))?;
        collect_application_dependencies(dependency, contents, reachable)?;
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
            Term::Product { shape, fields } => require_referent(referents, shape, "tuple shape")
                .and_then(|()| {
                    fields.values().try_for_each(|field| {
                        require_referent(referents, field.domain(), "tuple field domain")
                    })
                }),
            Term::LabelledProduct { shape, fields } => {
                require_referent(referents, shape, "labelled product shape").and_then(|()| {
                    fields.keys().try_for_each(|field| {
                        require_referent(referents, field, "labelled product field")
                    })
                })
            }
            Term::Sequence { shape, element, .. } => {
                require_referent(referents, shape, "sequence shape")
                    .and_then(|()| require_referent(referents, element, "sequence element domain"))
            }
            Term::F32(_) | Term::Int(_) | Term::Bool(_) | Term::Sum { .. } => Ok(()),
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

fn collect_content_patterns(
    id: &ContentId,
    contents: &BTreeMap<ContentId, RelationalContent>,
    patterns: &mut BTreeSet<PatternId>,
    visiting: &mut BTreeSet<ContentId>,
) {
    if !visiting.insert(id.clone()) {
        return;
    }
    if let Some(content) = contents.get(id) {
        for term in content.roles().values() {
            term.walk(&mut |term| match term {
                Term::Pattern(id) => {
                    patterns.insert(id.clone());
                }
                Term::Application(id) => {
                    collect_content_patterns(id, contents, patterns, visiting);
                }
                _ => {}
            });
        }
    }
    visiting.remove(id);
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
        Term::Product { fields, .. } => fields
            .values()
            .all(|field| term_is_ground(contents, field.value(), visiting)),
        Term::LabelledProduct { fields, .. } => fields
            .values()
            .all(|term| term_is_ground(contents, term, visiting)),
        Term::Sum { value, .. } => term_is_ground(contents, value, visiting),
        Term::Sequence { values, .. } => values
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
    structural_contracts: &BTreeMap<ReferentId, StructuralContract>,
    definitions: &[Definition],
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
            | Term::Product { .. }
            | Term::LabelledProduct { .. }
            | Term::Sum { .. }
            | Term::Sequence { .. } => {
                if let Some(expected) = structural_role_domain(&shape.roles()[role_id])? {
                    let path = ProposalPath::new(ProposalSubject::Content(content.id().clone()))
                        .child(ProposalPathSegment::Role(role_id.clone()));
                    validate_term_against(
                        structural_contracts,
                        definitions,
                        contents,
                        shapes,
                        admitted,
                        &path,
                        expected,
                        term,
                    )?;
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
        record_patterns(
            content,
            contents,
            shapes,
            &mut requirements,
            Some(&mut premises),
        )?;
    }
    validate_pattern(contents, shapes, rule.conclusion())?;
    let mut conclusions = BTreeSet::new();
    for id in rule.conclusion().forms() {
        let conclusion = contents
            .get(id)
            .ok_or_else(|| KernelError::new("derivation conclusion is undeclared"))?;
        record_patterns(
            conclusion,
            contents,
            shapes,
            &mut requirements,
            Some(&mut conclusions),
        )?;
    }
    if !conclusions.is_subset(&premises) {
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
        record_patterns(content, contents, shapes, &mut requirements, None)?;
    }
    Ok(())
}

fn record_patterns(
    content: &RelationalContent,
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    requirements: &mut BTreeMap<PatternId, Vec<RolePredicate>>,
    seen: Option<&mut BTreeSet<PatternId>>,
) -> Result<()> {
    record_patterns_inner(
        content,
        contents,
        shapes,
        requirements,
        seen,
        &mut BTreeSet::new(),
    )
}

fn record_patterns_inner(
    content: &RelationalContent,
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    requirements: &mut BTreeMap<PatternId, Vec<RolePredicate>>,
    mut seen: Option<&mut BTreeSet<PatternId>>,
    active: &mut BTreeSet<ContentId>,
) -> Result<()> {
    for (role, term) in content.roles() {
        let current = shapes[content.relation()].roles()[role].admissibility();
        record_term_patterns(
            term,
            current,
            contents,
            shapes,
            requirements,
            seen.as_deref_mut(),
            active,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn record_term_patterns(
    term: &Term,
    current: &[RolePredicate],
    contents: &BTreeMap<ContentId, RelationalContent>,
    shapes: &BTreeMap<ReferentId, RelationShape>,
    requirements: &mut BTreeMap<PatternId, Vec<RolePredicate>>,
    mut seen: Option<&mut BTreeSet<PatternId>>,
    active: &mut BTreeSet<ContentId>,
) -> Result<()> {
    match term {
        Term::Pattern(id) => {
            if requirements
                .insert(id.clone(), current.to_vec())
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
        Term::Application(id) => {
            if !active.insert(id.clone()) {
                return Err(KernelError::new(
                    "recursive term application graph contains a cycle",
                ));
            }
            let dependency = contents
                .get(id)
                .ok_or_else(|| KernelError::new("recursive term names undeclared content"))?;
            let result = record_patterns_inner(
                dependency,
                contents,
                shapes,
                requirements,
                seen.as_deref_mut(),
                active,
            );
            active.remove(id);
            result?;
        }
        Term::Product { fields, .. } => {
            for field in fields.values() {
                record_term_patterns(
                    field.value(),
                    current,
                    contents,
                    shapes,
                    requirements,
                    seen.as_deref_mut(),
                    active,
                )?;
            }
        }
        Term::LabelledProduct { fields, .. } => {
            for value in fields.values() {
                record_term_patterns(
                    value,
                    current,
                    contents,
                    shapes,
                    requirements,
                    seen.as_deref_mut(),
                    active,
                )?;
            }
        }
        Term::Sum { value, .. } => {
            record_term_patterns(value, current, contents, shapes, requirements, seen, active)?
        }
        Term::Sequence { values, .. } => {
            for value in values {
                record_term_patterns(
                    value,
                    current,
                    contents,
                    shapes,
                    requirements,
                    seen.as_deref_mut(),
                    active,
                )?;
            }
        }
        Term::Referent(_) | Term::F32(_) | Term::Int(_) | Term::Bool(_) => {}
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
        contents,
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
    contents: &BTreeMap<ContentId, RelationalContent>,
) -> bool {
    if index == patterns.len() {
        return conclusions.iter().any(|conclusion| {
            super::matching::unify(
                conclusion,
                target,
                substitution,
                false,
                |id| contents.get(id),
                |id| contents.get(id),
            )
            .is_some()
        });
    }
    for (candidate, premise) in premises.iter().enumerate() {
        if used.contains(&candidate) {
            continue;
        }
        let Some(next) = super::matching::unify(
            patterns[index],
            premise,
            substitution,
            true,
            |id| contents.get(id),
            |id| contents.get(id),
        ) else {
            continue;
        };
        used.insert(candidate);
        if match_premises(
            patterns,
            premises,
            conclusions,
            target,
            index + 1,
            used,
            &next,
            contents,
        ) {
            return true;
        }
        used.remove(&candidate);
    }
    false
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
