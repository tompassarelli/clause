//! Immutable runtime state, occurrence-exact incremental support, and replay.
//!
//! The checked [`crate::kernel::Model`] remains the sole semantic authority.
//! Runtime values bind one exact Model Revision and retain only execution
//! history, indexes, dependency support, and deterministic work evidence.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

use crate::{
    intrinsic::Intrinsic,
    kernel::{
        AssertionOccurrence, ContentId, DerivationRule, JudgmentKind, JudgmentStatus,
        JudgmentTarget, KernelError, Model, ReferentId, RelationalContent, Result, Revision,
        RevisionId, Term,
    },
    wire::{
        json::{Json, JsonParser, array, json, list, require_string, string},
        sha256_digest,
    },
};

pub const STATE_REVISION_TAG: &str = "clause-state-revision-v1";
pub const RUNTIME_SESSION_TAG: &str = "clause-runtime-session-v1";

macro_rules! digest_identity {
    ($name:ident, $prefix:literal, $message:literal) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: String) -> Result<Self> {
                let Some(hex) = value.strip_prefix($prefix) else {
                    return Err(KernelError::new($message));
                };
                if hex.len() == 64
                    && hex
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    Ok(Self(value))
                } else {
                    Err(KernelError::new($message))
                }
            }

            fn from_digest(bytes: [u8; 32]) -> Self {
                let mut value = String::from($prefix);
                for byte in bytes {
                    use fmt::Write;
                    write!(value, "{byte:02x}").expect("writing a digest to String cannot fail");
                }
                Self(value)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

digest_identity!(
    StateRevisionId,
    "state-sha256-",
    "invalid StateRevision identity"
);
digest_identity!(
    RuntimeSessionId,
    "session-sha256-",
    "invalid RuntimeSession identity"
);

/// The complete deterministic runtime policy bound into every state/session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimePolicy {
    id: ReferentId,
    max_supports: usize,
    max_join_attempts: usize,
}

impl RuntimePolicy {
    pub fn new(id: ReferentId, max_supports: usize, max_join_attempts: usize) -> Result<Self> {
        if max_supports == 0 || max_join_attempts == 0 {
            return Err(KernelError::new(
                "runtime policy bounds must both be positive",
            ));
        }
        Ok(Self {
            id,
            max_supports,
            max_join_attempts,
        })
    }

    pub fn id(&self) -> &ReferentId {
        &self.id
    }

    pub fn max_supports(&self) -> usize {
        self.max_supports
    }

    pub fn max_join_attempts(&self) -> usize {
        self.max_join_attempts
    }
}

/// One ordered event that invokes a checked Model transition in one scope.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TransitionEvent {
    id: ReferentId,
    transition: ReferentId,
    target_occurrence: ReferentId,
    successor_occurrence: ReferentId,
    scope: ReferentId,
}

impl TransitionEvent {
    pub fn new(
        id: ReferentId,
        transition: ReferentId,
        target_occurrence: ReferentId,
        successor_occurrence: ReferentId,
        scope: ReferentId,
    ) -> Self {
        Self {
            id,
            transition,
            target_occurrence,
            successor_occurrence,
            scope,
        }
    }

    pub fn id(&self) -> &ReferentId {
        &self.id
    }

    pub fn transition(&self) -> &ReferentId {
        &self.transition
    }

    pub fn target_occurrence(&self) -> &ReferentId {
        &self.target_occurrence
    }

    pub fn successor_occurrence(&self) -> &ReferentId {
        &self.successor_occurrence
    }

    pub fn scope(&self) -> &ReferentId {
        &self.scope
    }
}

/// One complete occurrence-level state edge. Its sets are canonical; event
/// order remains separately present in the successor lineage and session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateDelta {
    withdrawals: Vec<ReferentId>,
    admissions: Vec<AssertionOccurrence>,
}

impl StateDelta {
    pub fn new(
        mut withdrawals: Vec<ReferentId>,
        mut admissions: Vec<AssertionOccurrence>,
    ) -> Result<Self> {
        withdrawals.sort();
        admissions.sort_by(|left, right| left.id().cmp(right.id()));
        if withdrawals.windows(2).any(|pair| pair[0] == pair[1])
            || admissions
                .windows(2)
                .any(|pair| pair[0].id() == pair[1].id())
        {
            return Err(KernelError::new(
                "StateDelta changes cannot contain duplicates",
            ));
        }
        if admissions
            .iter()
            .any(|admission| withdrawals.binary_search(admission.id()).is_ok())
        {
            return Err(KernelError::new(
                "StateDelta cannot withdraw and admit one occurrence identity",
            ));
        }
        Ok(Self {
            withdrawals,
            admissions,
        })
    }

    pub fn withdrawals(&self) -> &[ReferentId] {
        &self.withdrawals
    }

    pub fn admissions(&self) -> &[AssertionOccurrence] {
        &self.admissions
    }
}

/// The complete ordered input to one deterministic state fold. Event input
/// uses checked Model transitions; an explicit Delta is the fail-closed
/// fallback when no declarative transition expresses the requested edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeInput {
    Events(Vec<TransitionEvent>),
    Delta(StateDelta),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum StateLineage {
    Root,
    Successor {
        predecessor: StateRevisionId,
        delta: StateDelta,
        input: RuntimeInput,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SupportKey {
    conclusion: ContentId,
    roots: Vec<ReferentId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum GroundProof {
    Asserted {
        occurrence: ReferentId,
    },
    Derived {
        rule: ReferentId,
        premises: Vec<SupportKey>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GroundSupport {
    key: SupportKey,
    proof: GroundProof,
}

/// Exact work performed by one state transition. The counter is diagnostic
/// and intentionally excluded from StateRevision identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransitionWork {
    examined_rules: usize,
    join_attempts: usize,
    added_supports: usize,
    removed_supports: usize,
    touched_contents: BTreeSet<ContentId>,
}

impl TransitionWork {
    pub fn examined_rules(&self) -> usize {
        self.examined_rules
    }

    pub fn join_attempts(&self) -> usize {
        self.join_attempts
    }

    pub fn added_supports(&self) -> usize {
        self.added_supports
    }

    pub fn removed_supports(&self) -> usize {
        self.removed_supports
    }

    pub fn touched_contents(&self) -> &BTreeSet<ContentId> {
        &self.touched_contents
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IncrementalState {
    occurrences: BTreeMap<ReferentId, AssertionOccurrence>,
    catalog: BTreeMap<ContentId, RelationalContent>,
    supports: BTreeMap<ContentId, BTreeMap<Vec<ReferentId>, GroundSupport>>,
    relation_index: BTreeMap<ReferentId, BTreeSet<ContentId>>,
    rule_index: BTreeMap<ReferentId, Vec<(usize, usize)>>,
    root_index: BTreeMap<ReferentId, BTreeSet<SupportKey>>,
    work: TransitionWork,
}

impl IncrementalState {
    fn root(revision: &Revision, policy: &RuntimePolicy) -> Result<Self> {
        let mut state = Self {
            occurrences: BTreeMap::new(),
            catalog: revision.model().relational_contents().clone(),
            supports: BTreeMap::new(),
            relation_index: BTreeMap::new(),
            rule_index: compile_rule_index(revision.model()),
            root_index: BTreeMap::new(),
            work: TransitionWork::default(),
        };
        let mut frontier = VecDeque::new();
        for occurrence in admitted_occurrences(revision.model()) {
            state.insert_occurrence(occurrence, policy, &mut frontier)?;
        }
        state.propagate(revision.model(), policy, &mut frontier)?;
        Ok(state)
    }

    fn successor(
        &self,
        revision: &Revision,
        policy: &RuntimePolicy,
        delta: &StateDelta,
    ) -> Result<Self> {
        let mut state = self.clone();
        state.work = TransitionWork::default();
        for withdrawal in &delta.withdrawals {
            state.remove_occurrence(withdrawal)?;
        }
        let mut frontier = VecDeque::new();
        for admission in &delta.admissions {
            state.insert_occurrence(admission.clone(), policy, &mut frontier)?;
        }
        state.propagate(revision.model(), policy, &mut frontier)?;
        Ok(state)
    }

    fn insert_occurrence(
        &mut self,
        occurrence: AssertionOccurrence,
        policy: &RuntimePolicy,
        frontier: &mut VecDeque<SupportKey>,
    ) -> Result<()> {
        if self.occurrences.contains_key(occurrence.id()) {
            return Err(KernelError::new(
                "runtime occurrence identity is already active",
            ));
        }
        let content = self
            .catalog
            .get(occurrence.content())
            .ok_or_else(|| KernelError::new("runtime occurrence names unknown Model content"))?
            .clone();
        self.occurrences
            .insert(occurrence.id().clone(), occurrence.clone());
        let key = SupportKey {
            conclusion: content.id().clone(),
            roots: vec![occurrence.id().clone()],
        };
        let support = GroundSupport {
            key: key.clone(),
            proof: GroundProof::Asserted {
                occurrence: occurrence.id().clone(),
            },
        };
        if self.insert_support(content, support, policy)? {
            frontier.push_back(key);
        }
        Ok(())
    }

    fn remove_occurrence(&mut self, occurrence: &ReferentId) -> Result<()> {
        if self.occurrences.remove(occurrence).is_none() {
            return Err(KernelError::new(
                "StateDelta withdraws an inactive occurrence",
            ));
        }
        let affected = self.root_index.remove(occurrence).unwrap_or_default();
        for key in affected {
            let Some(by_root) = self.supports.get_mut(&key.conclusion) else {
                continue;
            };
            if by_root.remove(&key.roots).is_none() {
                continue;
            }
            self.work.removed_supports += 1;
            self.work.touched_contents.insert(key.conclusion.clone());
            for root in &key.roots {
                if root != occurrence
                    && let Some(keys) = self.root_index.get_mut(root)
                {
                    keys.remove(&key);
                }
            }
            if by_root.is_empty() {
                self.supports.remove(&key.conclusion);
                if let Some(content) = self.catalog.get(&key.conclusion)
                    && let Some(contents) = self.relation_index.get_mut(content.relation())
                {
                    contents.remove(&key.conclusion);
                }
            }
        }
        Ok(())
    }

    fn propagate(
        &mut self,
        model: &Model,
        policy: &RuntimePolicy,
        frontier: &mut VecDeque<SupportKey>,
    ) -> Result<()> {
        while let Some(new_support) = frontier.pop_front() {
            let Some(actual) = self.catalog.get(&new_support.conclusion).cloned() else {
                return Err(KernelError::new(
                    "incremental support names unavailable content",
                ));
            };
            let dependents = self
                .rule_index
                .get(actual.relation())
                .cloned()
                .unwrap_or_default();
            for (rule_index, premise_index) in dependents {
                self.work.examined_rules += 1;
                let rule = model.derivation_rules()[rule_index].clone();
                let candidates =
                    self.rule_candidates(model, policy, &rule, premise_index, &new_support)?;
                for (content, support) in candidates {
                    let key = support.key.clone();
                    if self.insert_support(content, support, policy)? {
                        frontier.push_back(key);
                    }
                }
            }
        }
        Ok(())
    }

    fn rule_candidates(
        &mut self,
        model: &Model,
        policy: &RuntimePolicy,
        rule: &DerivationRule,
        fixed_index: usize,
        fixed_support: &SupportKey,
    ) -> Result<Vec<(RelationalContent, GroundSupport)>> {
        let patterns = rule
            .premises()
            .forms()
            .iter()
            .map(|id| {
                model
                    .content(id)
                    .cloned()
                    .ok_or_else(|| KernelError::new("checked runtime rule premise is absent"))
            })
            .collect::<Result<Vec<_>>>()?;
        let conclusions =
            rule.conclusion()
                .forms()
                .iter()
                .map(|id| {
                    model.content(id).cloned().ok_or_else(|| {
                        KernelError::new("checked runtime rule conclusion is absent")
                    })
                })
                .collect::<Result<Vec<_>>>()?;
        let fixed_actual = self
            .catalog
            .get(&fixed_support.conclusion)
            .expect("frontier content is checked");
        let Some(substitution) = crate::kernel::matching::unify(
            &patterns[fixed_index],
            fixed_actual,
            &BTreeMap::new(),
            true,
            |id| model.content(id),
            |id| self.catalog.get(id).or_else(|| model.content(id)),
        ) else {
            return Ok(Vec::new());
        };
        let mut selected = vec![None; patterns.len()];
        selected[fixed_index] = Some(fixed_support.clone());
        let mut joined = Vec::new();
        self.collect_joins(
            model,
            policy,
            &patterns,
            fixed_index,
            0,
            substitution,
            &mut selected,
            &mut joined,
        )?;
        let mut candidates = Vec::new();
        for (substitution, premises) in joined {
            let roots = premises
                .iter()
                .flat_map(|premise| premise.roots.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            for conclusion in &conclusions {
                let instantiated =
                    crate::kernel::matching::instantiate(conclusion, &substitution, |id| {
                        model.content(id)
                    })?;
                for dependency in instantiated.dependencies.values() {
                    if let Some(existing) = self
                        .catalog
                        .insert(dependency.id().clone(), dependency.clone())
                        && existing != *dependency
                    {
                        return Err(KernelError::new(
                            "runtime derivation has conflicting content identity",
                        ));
                    }
                }
                let key = SupportKey {
                    conclusion: instantiated.root.id().clone(),
                    roots: roots.clone(),
                };
                candidates.push((
                    instantiated.root,
                    GroundSupport {
                        key,
                        proof: GroundProof::Derived {
                            rule: rule.id().clone(),
                            premises: premises.clone(),
                        },
                    },
                ));
            }
        }
        Ok(candidates)
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_joins(
        &mut self,
        model: &Model,
        policy: &RuntimePolicy,
        patterns: &[RelationalContent],
        fixed_index: usize,
        index: usize,
        substitution: BTreeMap<crate::kernel::PatternId, Term>,
        selected: &mut [Option<SupportKey>],
        joined: &mut Vec<(BTreeMap<crate::kernel::PatternId, Term>, Vec<SupportKey>)>,
    ) -> Result<()> {
        if index == patterns.len() {
            joined.push((
                substitution,
                selected
                    .iter()
                    .map(|item| item.clone().expect("every premise is selected"))
                    .collect(),
            ));
            return Ok(());
        }
        if index == fixed_index {
            return self.collect_joins(
                model,
                policy,
                patterns,
                fixed_index,
                index + 1,
                substitution,
                selected,
                joined,
            );
        }
        let content_ids = self
            .relation_index
            .get(patterns[index].relation())
            .cloned()
            .unwrap_or_default();
        for content_id in content_ids {
            let actual = self
                .catalog
                .get(&content_id)
                .expect("relation index names catalog content");
            self.work.join_attempts += 1;
            if self.work.join_attempts > policy.max_join_attempts {
                return Err(KernelError::new(
                    "runtime incremental join attempt limit exceeded",
                ));
            }
            let Some(next) = crate::kernel::matching::unify(
                &patterns[index],
                actual,
                &substitution,
                true,
                |id| model.content(id),
                |id| self.catalog.get(id).or_else(|| model.content(id)),
            ) else {
                continue;
            };
            let roots = self
                .supports
                .get(&content_id)
                .expect("active content has supports")
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            for roots in roots {
                selected[index] = Some(SupportKey {
                    conclusion: content_id.clone(),
                    roots,
                });
                self.collect_joins(
                    model,
                    policy,
                    patterns,
                    fixed_index,
                    index + 1,
                    next.clone(),
                    selected,
                    joined,
                )?;
            }
            selected[index] = None;
        }
        Ok(())
    }

    fn insert_support(
        &mut self,
        content: RelationalContent,
        support: GroundSupport,
        policy: &RuntimePolicy,
    ) -> Result<bool> {
        if let Some(existing) = self.catalog.insert(content.id().clone(), content.clone())
            && existing != content
        {
            return Err(KernelError::new(
                "runtime support has conflicting content identity",
            ));
        }
        let current_count = self.supports.values().map(BTreeMap::len).sum::<usize>();
        let by_root = self.supports.entry(content.id().clone()).or_default();
        match by_root.get_mut(&support.key.roots) {
            Some(existing) if support.proof < existing.proof => {
                existing.proof = support.proof;
                self.work.touched_contents.insert(content.id().clone());
                Ok(true)
            }
            Some(_) => Ok(false),
            None => {
                if current_count >= policy.max_supports {
                    return Err(KernelError::new(
                        "runtime incremental support limit exceeded",
                    ));
                }
                for root in &support.key.roots {
                    self.root_index
                        .entry(root.clone())
                        .or_default()
                        .insert(support.key.clone());
                }
                by_root.insert(support.key.roots.clone(), support.clone());
                self.relation_index
                    .entry(content.relation().clone())
                    .or_default()
                    .insert(content.id().clone());
                self.work.added_supports += 1;
                self.work.touched_contents.insert(content.id().clone());
                Ok(true)
            }
        }
    }
}

/// One immutable logical runtime snapshot and exact causal edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateRevision {
    identity: StateRevisionId,
    model_revision: RevisionId,
    policy: RuntimePolicy,
    tick: u64,
    lineage: StateLineage,
    state: IncrementalState,
}

impl StateRevision {
    fn root(revision: &Revision, policy: RuntimePolicy) -> Result<Self> {
        validate_policy(revision.model(), &policy)?;
        let incremental = IncrementalState::root(revision, &policy)?;
        let mut state = Self {
            identity: StateRevisionId::from_digest([0; 32]),
            model_revision: revision.identity().clone(),
            policy,
            tick: 0,
            lineage: StateLineage::Root,
            state: incremental,
        };
        state.identity = StateRevisionId::from_digest(sha256_digest(state.payload().as_bytes()));
        Ok(state)
    }

    fn successor(&self, revision: &Revision, input: RuntimeInput) -> Result<Self> {
        if revision.identity() != &self.model_revision {
            return Err(KernelError::new(
                "runtime transition names the wrong Model Revision",
            ));
        }
        let delta = match &input {
            RuntimeInput::Events(events) => validate_events(revision.model(), &self.state, events)?,
            RuntimeInput::Delta(delta) => {
                validate_explicit_delta(revision.model(), &self.state, delta)?;
                delta.clone()
            }
        };
        let next_state = self.state.successor(revision, &self.policy, &delta)?;
        let mut successor = Self {
            identity: StateRevisionId::from_digest([0; 32]),
            model_revision: self.model_revision.clone(),
            policy: self.policy.clone(),
            tick: self
                .tick
                .checked_add(1)
                .ok_or_else(|| KernelError::new("runtime tick overflow"))?,
            lineage: StateLineage::Successor {
                predecessor: self.identity.clone(),
                delta,
                input,
            },
            state: next_state,
        };
        successor.identity =
            StateRevisionId::from_digest(sha256_digest(successor.payload().as_bytes()));
        Ok(successor)
    }

    pub fn identity(&self) -> &StateRevisionId {
        &self.identity
    }

    pub fn model_revision(&self) -> &RevisionId {
        &self.model_revision
    }

    pub fn predecessor(&self) -> Option<&StateRevisionId> {
        match &self.lineage {
            StateLineage::Root => None,
            StateLineage::Successor { predecessor, .. } => Some(predecessor),
        }
    }

    pub fn delta(&self) -> Option<&StateDelta> {
        match &self.lineage {
            StateLineage::Root => None,
            StateLineage::Successor { delta, .. } => Some(delta),
        }
    }

    pub fn events(&self) -> &[TransitionEvent] {
        match &self.lineage {
            StateLineage::Root => &[],
            StateLineage::Successor {
                input: RuntimeInput::Events(events),
                ..
            } => events,
            StateLineage::Successor {
                input: RuntimeInput::Delta(_),
                ..
            } => &[],
        }
    }

    pub fn input(&self) -> Option<&RuntimeInput> {
        match &self.lineage {
            StateLineage::Root => None,
            StateLineage::Successor { input, .. } => Some(input),
        }
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn policy(&self) -> &RuntimePolicy {
        &self.policy
    }

    pub fn occurrences(&self) -> &BTreeMap<ReferentId, AssertionOccurrence> {
        &self.state.occurrences
    }

    pub fn contains_content(&self, content: &ContentId) -> bool {
        self.state.supports.contains_key(content)
    }

    pub fn support_roots(&self, content: &ContentId) -> Vec<Vec<ReferentId>> {
        self.state
            .supports
            .get(content)
            .map(|supports| supports.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn work(&self) -> &TransitionWork {
        &self.state.work
    }

    pub fn canonical_bytes(&self) -> String {
        format!(
            "[\"{STATE_REVISION_TAG}\",\"{}\",{}]",
            self.identity,
            self.payload()
        )
    }

    fn payload(&self) -> String {
        let lineage = match &self.lineage {
            StateLineage::Root => "[\"root\"]".to_owned(),
            StateLineage::Successor {
                predecessor,
                delta,
                input,
            } => format!(
                "[\"successor\",\"{}\",{},[\"input\",{}]]",
                predecessor,
                delta_json(delta),
                input_json(input),
            ),
        };
        let occurrences = join(self.state.occurrences.values().map(occurrence_json));
        let contents = strings(self.state.supports.keys().map(ContentId::as_str));
        let supports = join(
            self.state
                .supports
                .values()
                .flat_map(BTreeMap::values)
                .map(support_json),
        );
        format!(
            "[[\"model-revision\",\"{}\"],[\"tick\",\"{}\"],[\"policy\",{}],[\"lineage\",{}],[\"occurrences\",[{}]],[\"contents\",[{}]],[\"supports\",[{}]]]",
            self.model_revision,
            self.tick,
            policy_json(&self.policy),
            lineage,
            occurrences,
            contents,
            supports,
        )
    }
}

/// An immutable canonical prefix of one deterministic runtime history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSession {
    identity: RuntimeSessionId,
    model_revision: RevisionId,
    policy: RuntimePolicy,
    states: Vec<StateRevision>,
    inputs: Vec<RuntimeInput>,
}

impl RuntimeSession {
    pub fn start(revision: &Revision, policy: RuntimePolicy) -> Result<Self> {
        let root = StateRevision::root(revision, policy.clone())?;
        let mut session = Self {
            identity: RuntimeSessionId::from_digest([0; 32]),
            model_revision: revision.identity().clone(),
            policy,
            states: vec![root],
            inputs: Vec::new(),
        };
        session.recompute_identity();
        Ok(session)
    }

    pub fn transition(&self, revision: &Revision, events: Vec<TransitionEvent>) -> Result<Self> {
        if events.is_empty() {
            return Err(KernelError::new("runtime tick needs at least one event"));
        }
        if revision.identity() != &self.model_revision {
            return Err(KernelError::new(
                "RuntimeSession names the wrong Model Revision",
            ));
        }
        self.apply_input(revision, RuntimeInput::Events(events))
    }

    pub fn apply_delta(&self, revision: &Revision, delta: StateDelta) -> Result<Self> {
        self.apply_input(revision, RuntimeInput::Delta(delta))
    }

    fn apply_input(&self, revision: &Revision, input: RuntimeInput) -> Result<Self> {
        if revision.identity() != &self.model_revision {
            return Err(KernelError::new(
                "RuntimeSession names the wrong Model Revision",
            ));
        }
        let successor = self.latest().successor(revision, input.clone())?;
        let mut next = self.clone();
        next.states.push(successor);
        next.inputs.push(input);
        next.recompute_identity();
        Ok(next)
    }

    pub fn replay(
        revision: &Revision,
        policy: RuntimePolicy,
        ticks: impl IntoIterator<Item = Vec<TransitionEvent>>,
    ) -> Result<Self> {
        Self::replay_inputs(
            revision,
            policy,
            ticks.into_iter().map(RuntimeInput::Events),
        )
    }

    pub fn replay_inputs(
        revision: &Revision,
        policy: RuntimePolicy,
        inputs: impl IntoIterator<Item = RuntimeInput>,
    ) -> Result<Self> {
        let mut session = Self::start(revision, policy)?;
        for input in inputs {
            session = session.apply_input(revision, input)?;
        }
        Ok(session)
    }

    pub fn identity(&self) -> &RuntimeSessionId {
        &self.identity
    }

    pub fn model_revision(&self) -> &RevisionId {
        &self.model_revision
    }

    pub fn states(&self) -> &[StateRevision] {
        &self.states
    }

    pub fn inputs(&self) -> &[RuntimeInput] {
        &self.inputs
    }

    pub fn latest(&self) -> &StateRevision {
        self.states
            .last()
            .expect("RuntimeSession always contains its root state")
    }

    pub fn canonical_bytes(&self) -> String {
        format!(
            "[\"{RUNTIME_SESSION_TAG}\",\"{}\",{}]",
            self.identity,
            self.payload()
        )
    }

    fn recompute_identity(&mut self) {
        self.identity = RuntimeSessionId::from_digest(sha256_digest(self.payload().as_bytes()));
    }

    fn payload(&self) -> String {
        let states = join(self.states.iter().map(StateRevision::canonical_bytes));
        let inputs = join(self.inputs.iter().map(input_json));
        format!(
            "[[\"model-revision\",\"{}\"],[\"policy\",{}],[\"states\",[{}]],[\"inputs\",[{}]]]",
            self.model_revision,
            policy_json(&self.policy),
            states,
            inputs,
        )
    }
}

/// Strictly reload one canonical runtime session by replaying its ordered
/// inputs through the same transition fold and comparing every derived byte.
pub fn reload_session(bytes: &str, revision: &Revision) -> Result<RuntimeSession> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new(
            "RuntimeSession wire is not canonical JSON",
        ));
    }
    let envelope = list(&value, 3, "RuntimeSession envelope")?;
    require_string(
        &envelope[0],
        RUNTIME_SESSION_TAG,
        "RuntimeSession envelope tag",
    )?;
    let claimed = RuntimeSessionId::new(string(&envelope[1], "RuntimeSession identity")?.into())?;
    let payload = list(&envelope[2], 4, "RuntimeSession payload")?;
    let model_revision = tagged(&payload[0], "model-revision", "runtime Model Revision")?;
    if string(model_revision, "runtime Model Revision identity")? != revision.identity().to_string()
    {
        return Err(KernelError::new(
            "RuntimeSession names the wrong Model Revision",
        ));
    }
    let policy = decode_policy(tagged(&payload[1], "policy", "runtime policy")?)?;
    let _states = array(
        tagged(&payload[2], "states", "runtime states")?,
        "runtime states",
    )?;
    let inputs = array(
        tagged(&payload[3], "inputs", "runtime inputs")?,
        "runtime inputs",
    )?
    .iter()
    .map(decode_input)
    .collect::<Result<Vec<_>>>()?;
    let session = RuntimeSession::replay_inputs(revision, policy, inputs)?;
    if session.identity != claimed || session.canonical_bytes() != bytes {
        return Err(KernelError::new(
            "RuntimeSession replay does not match its exact canonical history",
        ));
    }
    Ok(session)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presence {
    Present,
    Absent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPresenceChange {
    content: ContentId,
    before: Presence,
    after: Presence,
}

impl ContentPresenceChange {
    pub fn content(&self) -> &ContentId {
        &self.content
    }

    pub fn before(&self) -> Presence {
        self.before
    }

    pub fn after(&self) -> Presence {
        self.after
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateDiff {
    occurrence_admissions: Vec<AssertionOccurrence>,
    occurrence_withdrawals: Vec<AssertionOccurrence>,
    content_changes: Vec<ContentPresenceChange>,
    support_additions: Vec<(ContentId, Vec<ReferentId>)>,
    support_withdrawals: Vec<(ContentId, Vec<ReferentId>)>,
    proof_changes: Vec<(ContentId, Vec<ReferentId>)>,
    stable_referents: Vec<ReferentId>,
    retained_equalities: Vec<ContentId>,
    authorized_equivalences: Vec<ReferentId>,
}

impl StateDiff {
    pub fn between(
        base: &StateRevision,
        successor: &StateRevision,
        revision: &Revision,
    ) -> Result<Self> {
        if base.model_revision != successor.model_revision
            || base.model_revision != *revision.identity()
        {
            return Err(KernelError::new(
                "StateDiff requires one exact Model Revision",
            ));
        }
        let occurrence_admissions = successor
            .state
            .occurrences
            .iter()
            .filter(|(id, _)| !base.state.occurrences.contains_key(*id))
            .map(|(_, occurrence)| occurrence.clone())
            .collect();
        let occurrence_withdrawals = base
            .state
            .occurrences
            .iter()
            .filter(|(id, _)| !successor.state.occurrences.contains_key(*id))
            .map(|(_, occurrence)| occurrence.clone())
            .collect();
        let base_contents = base.state.supports.keys().cloned().collect::<BTreeSet<_>>();
        let successor_contents = successor
            .state
            .supports
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let content_changes = successor_contents
            .difference(&base_contents)
            .cloned()
            .map(|content| ContentPresenceChange {
                content,
                before: Presence::Absent,
                after: Presence::Present,
            })
            .chain(
                base_contents
                    .difference(&successor_contents)
                    .cloned()
                    .map(|content| ContentPresenceChange {
                        content,
                        before: Presence::Present,
                        after: Presence::Absent,
                    }),
            )
            .collect();
        let base_supports = flattened_supports(&base.state);
        let successor_supports = flattened_supports(&successor.state);
        let support_additions = successor_supports
            .keys()
            .filter(|key| !base_supports.contains_key(*key))
            .map(|key| (key.conclusion.clone(), key.roots.clone()))
            .collect();
        let support_withdrawals = base_supports
            .keys()
            .filter(|key| !successor_supports.contains_key(*key))
            .map(|key| (key.conclusion.clone(), key.roots.clone()))
            .collect();
        let proof_changes = base_supports
            .iter()
            .filter_map(|(key, proof)| {
                successor_supports
                    .get(key)
                    .filter(|successor| *successor != proof)
                    .map(|_| (key.conclusion.clone(), key.roots.clone()))
            })
            .collect();
        let retained_contents = base_contents
            .intersection(&successor_contents)
            .cloned()
            .collect::<Vec<_>>();
        let stable_referents = retained_contents
            .iter()
            .filter_map(|id| successor.state.catalog.get(id))
            .flat_map(content_referents)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let retained_equalities = retained_contents
            .iter()
            .filter(|id| {
                successor
                    .state
                    .catalog
                    .get(*id)
                    .is_some_and(|content| content.relation() == &Intrinsic::Equal.relation())
            })
            .cloned()
            .collect::<Vec<_>>();
        // M6 has no equivalence-authorizing judgment kind. In particular, an
        // admitted equality is still equality and must not be promoted by
        // structural resemblance. Keep the category explicit and empty.
        let authorized_equivalences = Vec::new();
        Ok(Self {
            occurrence_admissions,
            occurrence_withdrawals,
            content_changes,
            support_additions,
            support_withdrawals,
            proof_changes,
            stable_referents,
            retained_equalities,
            authorized_equivalences,
        })
    }

    pub fn occurrence_admissions(&self) -> &[AssertionOccurrence] {
        &self.occurrence_admissions
    }

    pub fn occurrence_withdrawals(&self) -> &[AssertionOccurrence] {
        &self.occurrence_withdrawals
    }

    pub fn content_changes(&self) -> &[ContentPresenceChange] {
        &self.content_changes
    }

    pub fn support_additions(&self) -> &[(ContentId, Vec<ReferentId>)] {
        &self.support_additions
    }

    pub fn support_withdrawals(&self) -> &[(ContentId, Vec<ReferentId>)] {
        &self.support_withdrawals
    }

    pub fn proof_changes(&self) -> &[(ContentId, Vec<ReferentId>)] {
        &self.proof_changes
    }

    pub fn stable_referents(&self) -> &[ReferentId] {
        &self.stable_referents
    }

    pub fn retained_equalities(&self) -> &[ContentId] {
        &self.retained_equalities
    }

    pub fn authorized_equivalences(&self) -> &[ReferentId] {
        &self.authorized_equivalences
    }
}

fn compile_rule_index(model: &Model) -> BTreeMap<ReferentId, Vec<(usize, usize)>> {
    let mut index = BTreeMap::<ReferentId, Vec<(usize, usize)>>::new();
    for (rule_index, rule) in model.derivation_rules().iter().enumerate() {
        for (premise_index, premise) in rule.premises().forms().iter().enumerate() {
            let relation = model
                .content(premise)
                .expect("checked rule premise")
                .relation()
                .clone();
            index
                .entry(relation)
                .or_default()
                .push((rule_index, premise_index));
        }
    }
    index
}

fn admitted_occurrences(model: &Model) -> Vec<AssertionOccurrence> {
    model
        .occurrences()
        .iter()
        .filter(|occurrence| {
            model.judgments().iter().any(|judgment| {
                judgment.authority() == model.id()
                    && judgment.scope() == model.id()
                    && judgment.status() == &JudgmentStatus::Affirmed
                    && matches!(judgment.kind(), JudgmentKind::Admitted { .. })
                    && match judgment.target() {
                        JudgmentTarget::Occurrence(id) => id == occurrence.id(),
                        JudgmentTarget::Content(id) => id == occurrence.content(),
                    }
            })
        })
        .cloned()
        .collect()
}

fn validate_policy(model: &Model, policy: &RuntimePolicy) -> Result<()> {
    if !model.referents().contains_key(policy.id()) {
        return Err(KernelError::new(
            "runtime policy identity is absent from the checked Model",
        ));
    }
    Ok(())
}

fn validate_events(
    model: &Model,
    state: &IncrementalState,
    events: &[TransitionEvent],
) -> Result<StateDelta> {
    let mut event_ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut successors = BTreeSet::new();
    let mut withdrawals = Vec::new();
    let mut admissions = Vec::new();
    for event in events {
        for (id, kind) in [
            (&event.id, "event"),
            (&event.successor_occurrence, "successor occurrence"),
            (&event.scope, "event scope"),
        ] {
            if !model.referents().contains_key(id) {
                return Err(KernelError::new(format!(
                    "runtime {kind} identity is absent from the checked Model"
                )));
            }
        }
        if !event_ids.insert(event.id.clone()) {
            return Err(KernelError::new("runtime tick repeats an event identity"));
        }
        if !targets.insert(event.target_occurrence.clone()) {
            return Err(KernelError::new(
                "runtime tick has conflicting writes to one pre-state occurrence",
            ));
        }
        if !successors.insert(event.successor_occurrence.clone())
            || state.occurrences.contains_key(&event.successor_occurrence)
        {
            return Err(KernelError::new(
                "runtime tick has a conflicting successor occurrence identity",
            ));
        }
        let active = state
            .occurrences
            .get(&event.target_occurrence)
            .ok_or_else(|| KernelError::new("runtime event targets an inactive occurrence"))?;
        if active.scope() != &event.scope {
            return Err(KernelError::new(
                "runtime event scope does not match its pre-state occurrence",
            ));
        }
        let transition = model
            .transitions()
            .iter()
            .find(|transition| transition.id() == &event.transition)
            .ok_or_else(|| KernelError::new("runtime event names an unknown Model transition"))?;
        if transition.from() != active.content() {
            return Err(KernelError::new(
                "runtime transition does not match its exact pre-state content",
            ));
        }
        withdrawals.push(active.id().clone());
        admissions.push(AssertionOccurrence::new(
            event.successor_occurrence.clone(),
            transition.to().clone(),
            event.id.clone(),
            event.scope.clone(),
        ));
    }
    StateDelta::new(withdrawals, admissions)
}

fn validate_explicit_delta(
    model: &Model,
    state: &IncrementalState,
    delta: &StateDelta,
) -> Result<()> {
    for withdrawal in delta.withdrawals() {
        if !state.occurrences.contains_key(withdrawal) {
            return Err(KernelError::new(
                "explicit StateDelta withdraws an inactive occurrence",
            ));
        }
    }
    for admission in delta.admissions() {
        if state.occurrences.contains_key(admission.id()) {
            return Err(KernelError::new(
                "explicit StateDelta admits an active occurrence identity",
            ));
        }
        if !model
            .relational_contents()
            .contains_key(admission.content())
        {
            return Err(KernelError::new(
                "explicit StateDelta admits content absent from the checked Model",
            ));
        }
        for (id, kind) in [
            (admission.id(), "occurrence"),
            (admission.source(), "source"),
            (admission.scope(), "scope"),
        ] {
            if !model.referents().contains_key(id) {
                return Err(KernelError::new(format!(
                    "explicit StateDelta {kind} is absent from the checked Model"
                )));
            }
        }
    }
    Ok(())
}

fn flattened_supports(state: &IncrementalState) -> BTreeMap<SupportKey, GroundProof> {
    state
        .supports
        .values()
        .flat_map(BTreeMap::values)
        .map(|support| (support.key.clone(), support.proof.clone()))
        .collect()
}

fn content_referents(content: &RelationalContent) -> Vec<ReferentId> {
    let mut referents = BTreeSet::from([content.relation().clone()]);
    for term in content.roles().values() {
        term.walk(&mut |term| match term {
            Term::Referent(id) => {
                referents.insert(id.clone());
            }
            Term::Product { shape, fields } => {
                referents.insert(shape.clone());
                referents.extend(fields.values().map(|field| field.domain().clone()));
            }
            Term::LabelledProduct { shape, fields } => {
                referents.insert(shape.clone());
                referents.extend(fields.keys().cloned());
            }
            Term::Sequence { shape, element, .. } => {
                referents.insert(shape.clone());
                referents.insert(element.clone());
            }
            Term::Pattern(_)
            | Term::Application(_)
            | Term::F32(_)
            | Term::Int(_)
            | Term::Bool(_)
            | Term::Sum { .. } => {}
        });
    }
    referents.into_iter().collect()
}

fn policy_json(policy: &RuntimePolicy) -> String {
    format!(
        "[\"reject-conflicts-v1\",\"{}\",[\"max-supports\",\"{}\"],[\"max-join-attempts\",\"{}\"]]",
        policy.id.as_str(),
        policy.max_supports,
        policy.max_join_attempts
    )
}

fn decode_policy(value: &Json) -> Result<RuntimePolicy> {
    let item = list(value, 4, "runtime policy")?;
    require_string(&item[0], "reject-conflicts-v1", "runtime conflict policy")?;
    let id = ReferentId::new(string(&item[1], "runtime policy identity")?.into())?;
    let max_supports = decode_usize(tagged(&item[2], "max-supports", "runtime support bound")?)?;
    let max_join_attempts =
        decode_usize(tagged(&item[3], "max-join-attempts", "runtime join bound")?)?;
    RuntimePolicy::new(id, max_supports, max_join_attempts)
}

fn input_json(input: &RuntimeInput) -> String {
    match input {
        RuntimeInput::Events(events) => {
            format!("[\"events\",[{}]]", join(events.iter().map(event_json)))
        }
        RuntimeInput::Delta(delta) => format!("[\"delta\",{}]", delta_json(delta)),
    }
}

fn decode_input(value: &Json) -> Result<RuntimeInput> {
    let item = list(value, 2, "runtime input")?;
    match string(&item[0], "runtime input tag")? {
        "events" => Ok(RuntimeInput::Events(
            array(&item[1], "runtime event input")?
                .iter()
                .map(decode_event)
                .collect::<Result<Vec<_>>>()?,
        )),
        "delta" => Ok(RuntimeInput::Delta(decode_delta(&item[1])?)),
        _ => Err(KernelError::new("invalid runtime input tag")),
    }
}

fn event_json(event: &TransitionEvent) -> String {
    format!(
        "[\"event\",\"{}\",[\"transition\",\"{}\"],[\"target\",\"{}\"],[\"successor\",\"{}\"],[\"scope\",\"{}\"]]",
        event.id.as_str(),
        event.transition.as_str(),
        event.target_occurrence.as_str(),
        event.successor_occurrence.as_str(),
        event.scope.as_str(),
    )
}

fn decode_event(value: &Json) -> Result<TransitionEvent> {
    let item = list(value, 6, "runtime event")?;
    require_string(&item[0], "event", "runtime event tag")?;
    Ok(TransitionEvent::new(
        ReferentId::new(string(&item[1], "runtime event identity")?.into())?,
        ReferentId::new(
            string(
                tagged(&item[2], "transition", "runtime event transition")?,
                "runtime event transition identity",
            )?
            .into(),
        )?,
        ReferentId::new(
            string(
                tagged(&item[3], "target", "runtime event target")?,
                "runtime event target identity",
            )?
            .into(),
        )?,
        ReferentId::new(
            string(
                tagged(&item[4], "successor", "runtime event successor")?,
                "runtime event successor identity",
            )?
            .into(),
        )?,
        ReferentId::new(
            string(
                tagged(&item[5], "scope", "runtime event scope")?,
                "runtime event scope identity",
            )?
            .into(),
        )?,
    ))
}

fn delta_json(delta: &StateDelta) -> String {
    format!(
        "[\"state-delta\",[\"withdraw\",[{}]],[\"admit\",[{}]]]",
        strings(delta.withdrawals.iter().map(ReferentId::as_str)),
        join(delta.admissions.iter().map(occurrence_json)),
    )
}

fn decode_delta(value: &Json) -> Result<StateDelta> {
    let item = list(value, 3, "StateDelta")?;
    require_string(&item[0], "state-delta", "StateDelta tag")?;
    let withdrawals = array(
        tagged(&item[1], "withdraw", "StateDelta withdrawals")?,
        "StateDelta withdrawals",
    )?
    .iter()
    .map(|value| ReferentId::new(string(value, "withdrawal identity")?.into()))
    .collect::<Result<Vec<_>>>()?;
    let admissions = array(
        tagged(&item[2], "admit", "StateDelta admissions")?,
        "StateDelta admissions",
    )?
    .iter()
    .map(decode_occurrence)
    .collect::<Result<Vec<_>>>()?;
    StateDelta::new(withdrawals, admissions)
}

fn occurrence_json(occurrence: &AssertionOccurrence) -> String {
    format!(
        "[\"occurrence\",\"{}\",\"{}\",\"{}\",\"{}\"]",
        occurrence.id().as_str(),
        occurrence.content().as_str(),
        occurrence.source().as_str(),
        occurrence.scope().as_str(),
    )
}

fn decode_occurrence(value: &Json) -> Result<AssertionOccurrence> {
    let item = list(value, 5, "runtime occurrence")?;
    require_string(&item[0], "occurrence", "runtime occurrence tag")?;
    Ok(AssertionOccurrence::new(
        ReferentId::new(string(&item[1], "runtime occurrence identity")?.into())?,
        ContentId::new(string(&item[2], "runtime occurrence content")?.into())?,
        ReferentId::new(string(&item[3], "runtime occurrence source")?.into())?,
        ReferentId::new(string(&item[4], "runtime occurrence scope")?.into())?,
    ))
}

fn support_json(support: &GroundSupport) -> String {
    let proof = match &support.proof {
        GroundProof::Asserted { occurrence } => {
            format!("[\"asserted\",\"{}\"]", occurrence.as_str())
        }
        GroundProof::Derived { rule, premises } => format!(
            "[\"derived\",\"{}\",[{}]]",
            rule.as_str(),
            join(premises.iter().map(support_key_json)),
        ),
    };
    format!("[{},{}]", support_key_json(&support.key), proof)
}

fn support_key_json(key: &SupportKey) -> String {
    format!(
        "[\"{}\",[{}]]",
        key.conclusion.as_str(),
        strings(key.roots.iter().map(ReferentId::as_str)),
    )
}

fn tagged<'a>(value: &'a Json, tag: &str, where_: &str) -> Result<&'a Json> {
    let group = list(value, 2, where_)?;
    require_string(&group[0], tag, where_)?;
    Ok(&group[1])
}

fn decode_usize(value: &Json) -> Result<usize> {
    string(value, "runtime numeric bound")?
        .parse()
        .map_err(|_| KernelError::new("invalid runtime numeric bound"))
}

fn join(values: impl IntoIterator<Item = String>) -> String {
    values.into_iter().collect::<Vec<_>>().join(",")
}

fn strings<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    join(values.into_iter().map(|value| format!("\"{value}\"")))
}
