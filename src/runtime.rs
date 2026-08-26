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
        RevisionId, RoleId, Term,
    },
    wire::{
        json::{Json, JsonParser, array, json, list, require_string, string},
        sha256_digest,
    },
};

pub const STATE_REVISION_TAG: &str = "clause-state-revision-v2";
pub const RUNTIME_SESSION_TAG: &str = "clause-runtime-session-v2";
pub const EFFECT_TRACE_TAG: &str = "clause-effect-trace-v1";
pub const EFFECT_REQUEST_TAG: &str = "clause-effect-request-v1";

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
digest_identity!(
    AuthorizationId,
    "authorization-sha256-",
    "invalid Authorization identity"
);
digest_identity!(AttemptId, "attempt-sha256-", "invalid Attempt identity");
digest_identity!(ReceiptId, "receipt-sha256-", "invalid Receipt identity");
digest_identity!(
    ObservationId,
    "observation-sha256-",
    "invalid Observation identity"
);
digest_identity!(
    EffectTraceId,
    "effect-trace-sha256-",
    "invalid effect trace identity"
);
digest_identity!(
    EffectRequestId,
    "effect-request-sha256-",
    "invalid effect request identity"
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

/// One explicit ordered occurrence of a checked event with its payload.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TransitionEvent {
    id: ReferentId,
    event: ReferentId,
    payload: Vec<Term>,
}

impl TransitionEvent {
    pub fn new(id: ReferentId, event: ReferentId, payload: Vec<Term>) -> Self {
        Self { id, event, payload }
    }

    pub fn id(&self) -> &ReferentId {
        &self.id
    }

    pub fn event(&self) -> &ReferentId {
        &self.event
    }

    pub fn payload(&self) -> &[Term] {
        &self.payload
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
    support_accounting_steps: usize,
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

    /// Constant-bounded support-total reads performed while admitting new
    /// supports. This excludes proof joins and support propagation.
    pub fn support_accounting_steps(&self) -> usize {
        self.support_accounting_steps
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
    support_count: usize,
    relation_index: BTreeMap<ReferentId, BTreeSet<ContentId>>,
    occurrence_index: BTreeMap<ContentId, BTreeSet<ReferentId>>,
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
            support_count: 0,
            relation_index: BTreeMap::new(),
            occurrence_index: BTreeMap::new(),
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
        grounded_contents: Vec<RelationalContent>,
    ) -> Result<Self> {
        let mut state = self.clone();
        state.work = TransitionWork::default();
        for content in grounded_contents {
            if let Some(existing) = state.catalog.insert(content.id().clone(), content.clone())
                && existing != content
            {
                return Err(KernelError::new(
                    "runtime event grounded conflicting content identity",
                ));
            }
        }
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
        self.occurrence_index
            .entry(occurrence.content().clone())
            .or_default()
            .insert(occurrence.id().clone());
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
        let Some(removed) = self.occurrences.remove(occurrence) else {
            return Err(KernelError::new(
                "StateDelta withdraws an inactive occurrence",
            ));
        };
        if let Some(occurrences) = self.occurrence_index.get_mut(removed.content()) {
            occurrences.remove(occurrence);
            if occurrences.is_empty() {
                self.occurrence_index.remove(removed.content());
            }
        }
        let affected = self.root_index.remove(occurrence).unwrap_or_default();
        for key in affected {
            let Some(by_root) = self.supports.get_mut(&key.conclusion) else {
                continue;
            };
            if by_root.remove(&key.roots).is_none() {
                continue;
            }
            self.support_count = self
                .support_count
                .checked_sub(1)
                .expect("removed support was included in the exact total");
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
        self.work.support_accounting_steps += 1;
        let by_root = self.supports.entry(content.id().clone()).or_default();
        match by_root.get_mut(&support.key.roots) {
            Some(existing) if support.proof < existing.proof => {
                existing.proof = support.proof;
                self.work.touched_contents.insert(content.id().clone());
                Ok(true)
            }
            Some(_) => Ok(false),
            None => {
                if self.support_count >= policy.max_supports {
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
                self.support_count = self
                    .support_count
                    .checked_add(1)
                    .ok_or_else(|| KernelError::new("runtime support total overflow"))?;
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
        let (delta, grounded_contents) = match &input {
            RuntimeInput::Events(events) => validate_events(revision.model(), &self.state, events)?,
            RuntimeInput::Delta(delta) => {
                validate_explicit_delta(revision.model(), &self.state, delta)?;
                (delta.clone(), Vec::new())
            }
        };
        let next_state = self
            .state
            .successor(revision, &self.policy, &delta, grounded_contents)?;
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
        if let RuntimeInput::Events(events) = &input {
            let prior = self
                .inputs
                .iter()
                .filter_map(|input| match input {
                    RuntimeInput::Events(events) => Some(events.as_slice()),
                    RuntimeInput::Delta(_) => None,
                })
                .flatten()
                .map(TransitionEvent::id)
                .collect::<BTreeSet<_>>();
            if events.iter().any(|event| prior.contains(event.id())) {
                return Err(KernelError::new(
                    "runtime history repeats an event occurrence identity",
                ));
            }
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

/// Runtime-local lineage shared by every node in one effect evidence chain.
/// These links are evidence about execution; they do not add Model content or
/// admit a State change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLineage {
    producer: ReferentId,
    request: ReferentId,
    input_model_revision: RevisionId,
    post_commit_state: StateRevisionId,
    authority: ReferentId,
    event: ReferentId,
    phase: ReferentId,
    order: u64,
}

impl EffectLineage {
    pub fn producer(&self) -> &ReferentId {
        &self.producer
    }

    pub fn request(&self) -> &ReferentId {
        &self.request
    }

    pub fn input_model_revision(&self) -> &RevisionId {
        &self.input_model_revision
    }

    pub fn post_commit_state(&self) -> &StateRevisionId {
        &self.post_commit_state
    }

    pub fn authority(&self) -> &ReferentId {
        &self.authority
    }

    pub fn event(&self) -> &ReferentId {
        &self.event
    }

    pub fn phase(&self) -> &ReferentId {
        &self.phase
    }

    pub fn order(&self) -> u64 {
        self.order
    }
}

/// The existing checked identities that locate one requested external effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRequest {
    identity: EffectRequestId,
    producer: ReferentId,
    request: ReferentId,
    authority: ReferentId,
    event: ReferentId,
    phase: ReferentId,
    order: u64,
}

impl EffectRequest {
    pub fn new(
        producer: ReferentId,
        request: ReferentId,
        authority: ReferentId,
        event: ReferentId,
        phase: ReferentId,
        order: u64,
    ) -> Self {
        let mut request = Self {
            identity: EffectRequestId::from_digest([0; 32]),
            producer,
            request,
            authority,
            event,
            phase,
            order,
        };
        request.identity =
            EffectRequestId::from_digest(sha256_digest(request.preimage().as_bytes()));
        request
    }

    pub fn identity(&self) -> &EffectRequestId {
        &self.identity
    }

    /// Canonical, standalone representation of this intent/effect request.
    pub fn canonical_bytes(&self) -> String {
        format!(
            "[\"{EFFECT_REQUEST_TAG}\",\"{}\",{}]",
            self.identity,
            self.preimage()
        )
    }

    fn preimage(&self) -> String {
        format!(
            "[[\"producer\",\"{}\"],[\"request\",\"{}\"],[\"authority\",\"{}\"],[\"event\",\"{}\"],[\"phase\",\"{}\"],[\"order\",\"{}\"]]",
            self.producer.as_str(),
            self.request.as_str(),
            self.authority.as_str(),
            self.event.as_str(),
            self.phase.as_str(),
            self.order
        )
    }

    fn validate(&self, revision: &Revision) -> Result<()> {
        for (id, kind) in [
            (&self.producer, "producer"),
            (&self.request, "request"),
            (&self.authority, "authority"),
            (&self.phase, "phase"),
        ] {
            if !revision.model().referents().contains_key(id) {
                return Err(KernelError::new(format!(
                    "effect request {kind} is absent from the checked Model"
                )));
            }
        }
        let expected = EffectRequestId::from_digest(sha256_digest(self.preimage().as_bytes()));
        if self.identity != expected {
            return Err(KernelError::new("effect request identity does not match"));
        }
        Ok(())
    }
}

/// Reload a standalone canonical effect request and verify its content-derived identity.
pub fn reload_effect_request(bytes: &str) -> Result<EffectRequest> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new(
            "effect request wire is not canonical JSON",
        ));
    }
    let item = list(&value, 3, "effect request envelope")?;
    require_string(&item[0], EFFECT_REQUEST_TAG, "effect request envelope tag")?;
    let claimed = EffectRequestId::new(string(&item[1], "effect request identity")?.into())?;
    let body = list(&item[2], 6, "effect request body")?;
    let field = |index: usize, tag: &str| -> Result<ReferentId> {
        ReferentId::new(
            string(
                tagged(&body[index], tag, "effect request field")?,
                "effect request referent",
            )?
            .into(),
        )
    };
    let order = string(
        tagged(&body[5], "order", "effect request order")?,
        "effect request order value",
    )?
    .parse()
    .map_err(|_| KernelError::new("invalid effect request order"))?;
    let request = EffectRequest::new(
        field(0, "producer")?,
        field(1, "request")?,
        field(2, "authority")?,
        field(3, "event")?,
        field(4, "phase")?,
        order,
    );
    if request.identity != claimed || request.canonical_bytes() != bytes {
        return Err(KernelError::new(
            "effect request identity does not match canonical content",
        ));
    }
    Ok(request)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Denied,
    Authorized,
}

impl AuthorizationDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Denied => "denied",
            Self::Authorized => "authorized",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptOutcome {
    Succeeded,
    Failed,
}

impl ReceiptOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// An adapter's explicit result. Failure is a recorded outcome, not an
/// exception. Evidence must name an existing checked referent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectOutcome {
    Succeeded { evidence: ReferentId },
    Failed { evidence: ReferentId },
}

impl EffectOutcome {
    fn receipt_outcome(&self) -> ReceiptOutcome {
        match self {
            Self::Succeeded { .. } => ReceiptOutcome::Succeeded,
            Self::Failed { .. } => ReceiptOutcome::Failed,
        }
    }

    fn evidence(&self) -> &ReferentId {
        match self {
            Self::Succeeded { evidence } | Self::Failed { evidence } => evidence,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authorization {
    identity: AuthorizationId,
    lineage: EffectLineage,
    decision: AuthorizationDecision,
}

impl Authorization {
    fn new(lineage: EffectLineage, decision: AuthorizationDecision) -> Self {
        let identity = AuthorizationId::from_digest(sha256_digest(
            authorization_preimage(&lineage, decision).as_bytes(),
        ));
        Self {
            identity,
            lineage,
            decision,
        }
    }

    pub fn identity(&self) -> &AuthorizationId {
        &self.identity
    }

    pub fn lineage(&self) -> &EffectLineage {
        &self.lineage
    }

    pub fn decision(&self) -> AuthorizationDecision {
        self.decision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attempt {
    identity: AttemptId,
    authorization: AuthorizationId,
    lineage: EffectLineage,
}

impl Attempt {
    fn new(authorization: &Authorization) -> Self {
        let authorization_id = authorization.identity.clone();
        let lineage = authorization.lineage.clone();
        let identity = AttemptId::from_digest(sha256_digest(
            attempt_preimage(&authorization_id, &lineage).as_bytes(),
        ));
        Self {
            identity,
            authorization: authorization_id,
            lineage,
        }
    }

    pub fn identity(&self) -> &AttemptId {
        &self.identity
    }

    pub fn authorization(&self) -> &AuthorizationId {
        &self.authorization
    }

    pub fn lineage(&self) -> &EffectLineage {
        &self.lineage
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    identity: ReceiptId,
    attempt: AttemptId,
    lineage: EffectLineage,
    outcome: ReceiptOutcome,
}

impl Receipt {
    fn new(attempt: &Attempt, outcome: ReceiptOutcome) -> Self {
        let attempt_id = attempt.identity.clone();
        let lineage = attempt.lineage.clone();
        let identity = ReceiptId::from_digest(sha256_digest(
            receipt_preimage(&attempt_id, &lineage, outcome).as_bytes(),
        ));
        Self {
            identity,
            attempt: attempt_id,
            lineage,
            outcome,
        }
    }

    pub fn identity(&self) -> &ReceiptId {
        &self.identity
    }

    pub fn attempt(&self) -> &AttemptId {
        &self.attempt
    }

    pub fn lineage(&self) -> &EffectLineage {
        &self.lineage
    }

    pub fn outcome(&self) -> ReceiptOutcome {
        self.outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    identity: ObservationId,
    receipt: ReceiptId,
    lineage: EffectLineage,
    evidence: ReferentId,
}

impl Observation {
    fn new(receipt: &Receipt, evidence: ReferentId) -> Self {
        let receipt_id = receipt.identity.clone();
        let lineage = receipt.lineage.clone();
        let identity = ObservationId::from_digest(sha256_digest(
            observation_preimage(&receipt_id, &lineage, &evidence).as_bytes(),
        ));
        Self {
            identity,
            receipt: receipt_id,
            lineage,
            evidence,
        }
    }

    pub fn identity(&self) -> &ObservationId {
        &self.identity
    }

    pub fn receipt(&self) -> &ReceiptId {
        &self.receipt
    }

    pub fn lineage(&self) -> &EffectLineage {
        &self.lineage
    }

    pub fn evidence(&self) -> &ReferentId {
        &self.evidence
    }
}

/// One canonical runtime evidence chain. The absence of Attempt, Receipt, and
/// Observation is part of a denied authorization's checked representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectTrace {
    identity: EffectTraceId,
    authorization: Authorization,
    attempt: Option<Attempt>,
    receipt: Option<Receipt>,
    observation: Option<Observation>,
}

impl EffectTrace {
    /// Record a denied authorization. No adapter is accepted or invoked by
    /// this path, so denial cannot create an attempt or external effect.
    pub fn denied(
        revision: &Revision,
        post_commit: &StateRevision,
        request: EffectRequest,
    ) -> Result<Self> {
        let lineage = effect_lineage(revision, post_commit, request)?;
        Ok(Self::from_parts(
            Authorization::new(lineage, AuthorizationDecision::Denied),
            None,
            None,
            None,
        ))
    }

    /// Execute one authorized adapter only after validating the exact
    /// post-commit StateRevision and its event lineage.
    pub fn attempt<F>(
        revision: &Revision,
        post_commit: &StateRevision,
        request: EffectRequest,
        realize: F,
    ) -> Result<Self>
    where
        F: FnOnce(&EffectLineage) -> EffectOutcome,
    {
        let lineage = effect_lineage(revision, post_commit, request)?;
        let authorization = Authorization::new(lineage, AuthorizationDecision::Authorized);
        let attempt = Attempt::new(&authorization);
        let outcome = realize(attempt.lineage());
        if !revision
            .model()
            .referents()
            .contains_key(outcome.evidence())
        {
            return Err(KernelError::new(
                "effect observation evidence is absent from the checked Model",
            ));
        }
        let receipt = Receipt::new(&attempt, outcome.receipt_outcome());
        let observation = Observation::new(&receipt, outcome.evidence().clone());
        Ok(Self::from_parts(
            authorization,
            Some(attempt),
            Some(receipt),
            Some(observation),
        ))
    }

    fn from_parts(
        authorization: Authorization,
        attempt: Option<Attempt>,
        receipt: Option<Receipt>,
        observation: Option<Observation>,
    ) -> Self {
        let mut trace = Self {
            identity: EffectTraceId::from_digest([0; 32]),
            authorization,
            attempt,
            receipt,
            observation,
        };
        trace.identity = EffectTraceId::from_digest(sha256_digest(trace.payload().as_bytes()));
        trace
    }

    pub fn identity(&self) -> &EffectTraceId {
        &self.identity
    }

    pub fn authorization(&self) -> &Authorization {
        &self.authorization
    }

    pub fn attempt_record(&self) -> Option<&Attempt> {
        self.attempt.as_ref()
    }

    pub fn receipt(&self) -> Option<&Receipt> {
        self.receipt.as_ref()
    }

    pub fn observation(&self) -> Option<&Observation> {
        self.observation.as_ref()
    }

    pub fn canonical_bytes(&self) -> String {
        format!(
            "[\"{EFFECT_TRACE_TAG}\",\"{}\",{}]",
            self.identity,
            self.payload()
        )
    }

    fn payload(&self) -> String {
        format!(
            "[[\"authorization\",{}],[\"attempt\",{}],[\"receipt\",{}],[\"observation\",{}]]",
            authorization_json(&self.authorization),
            optional_json(self.attempt.as_ref().map(attempt_json)),
            optional_json(self.receipt.as_ref().map(receipt_json)),
            optional_json(self.observation.as_ref().map(observation_json)),
        )
    }
}

/// Strictly reload runtime effect evidence without projecting it into Model or
/// State admission.
pub fn reload_effect_trace(
    bytes: &str,
    revision: &Revision,
    post_commit: &StateRevision,
) -> Result<EffectTrace> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new("effect trace wire is not canonical JSON"));
    }
    let envelope = list(&value, 3, "effect trace envelope")?;
    require_string(&envelope[0], EFFECT_TRACE_TAG, "effect trace envelope tag")?;
    let claimed = EffectTraceId::new(string(&envelope[1], "effect trace identity")?.into())?;
    let payload = list(&envelope[2], 4, "effect trace payload")?;
    let authorization = decode_authorization(
        tagged(&payload[0], "authorization", "effect authorization")?,
        revision,
        post_commit,
    )?;
    let attempt = optional(
        tagged(&payload[1], "attempt", "effect attempt")?,
        "effect attempt",
    )?
    .map(|value| decode_attempt(value, &authorization))
    .transpose()?;
    let receipt = optional(
        tagged(&payload[2], "receipt", "effect receipt")?,
        "effect receipt",
    )?
    .map(|value| {
        let attempt = attempt
            .as_ref()
            .ok_or_else(|| KernelError::new("effect receipt has no attempt"))?;
        decode_receipt(value, attempt)
    })
    .transpose()?;
    let observation = optional(
        tagged(&payload[3], "observation", "effect observation")?,
        "effect observation",
    )?
    .map(|value| {
        let receipt = receipt
            .as_ref()
            .ok_or_else(|| KernelError::new("effect observation has no receipt"))?;
        decode_observation(value, receipt, revision)
    })
    .transpose()?;
    match authorization.decision {
        AuthorizationDecision::Denied
            if attempt.is_some() || receipt.is_some() || observation.is_some() =>
        {
            return Err(KernelError::new(
                "denied effect authorization contains execution evidence",
            ));
        }
        AuthorizationDecision::Authorized
            if attempt.is_none() || receipt.is_none() || observation.is_none() =>
        {
            return Err(KernelError::new(
                "authorized effect trace is missing execution evidence",
            ));
        }
        _ => {}
    }
    let trace = EffectTrace::from_parts(authorization, attempt, receipt, observation);
    if trace.identity != claimed || trace.canonical_bytes() != bytes {
        return Err(KernelError::new(
            "effect trace does not match its exact canonical lineage",
        ));
    }
    Ok(trace)
}

fn effect_lineage(
    revision: &Revision,
    post_commit: &StateRevision,
    request: EffectRequest,
) -> Result<EffectLineage> {
    request.validate(revision)?;
    let lineage = EffectLineage {
        producer: request.producer,
        request: request.request,
        input_model_revision: revision.identity().clone(),
        post_commit_state: post_commit.identity().clone(),
        authority: request.authority,
        event: request.event,
        phase: request.phase,
        order: request.order,
    };
    validate_effect_lineage(&lineage, revision, post_commit)?;
    Ok(lineage)
}

fn validate_effect_lineage(
    lineage: &EffectLineage,
    revision: &Revision,
    post_commit: &StateRevision,
) -> Result<()> {
    if lineage.input_model_revision != *revision.identity()
        || post_commit.model_revision() != revision.identity()
    {
        return Err(KernelError::new(
            "effect trace names the wrong Model Revision",
        ));
    }
    if lineage.post_commit_state != *post_commit.identity() || post_commit.predecessor().is_none() {
        return Err(KernelError::new(
            "effect trace needs the exact committed successor StateRevision",
        ));
    }
    let Some(event) = post_commit
        .events()
        .iter()
        .find(|event| event.id() == &lineage.event)
    else {
        return Err(KernelError::new(
            "effect trace event is absent from the committed StateRevision input",
        ));
    };
    if event.event() != &lineage.request {
        return Err(KernelError::new(
            "effect trace event occurrence does not match the requested event",
        ));
    }
    for (id, kind) in [
        (&lineage.producer, "producer"),
        (&lineage.request, "request"),
        (&lineage.authority, "authority"),
        (&lineage.phase, "phase"),
    ] {
        if !revision.model().referents().contains_key(id) {
            return Err(KernelError::new(format!(
                "effect trace {kind} is absent from the checked Model"
            )));
        }
    }
    Ok(())
}

fn lineage_json(lineage: &EffectLineage) -> String {
    format!(
        "[\"effect-lineage\",[\"producer\",\"{}\"],[\"request\",\"{}\"],[\"model-revision\",\"{}\"],[\"state-revision\",\"{}\"],[\"authority\",\"{}\"],[\"event\",\"{}\"],[\"phase\",\"{}\"],[\"order\",\"{}\"]]",
        lineage.producer.as_str(),
        lineage.request.as_str(),
        lineage.input_model_revision,
        lineage.post_commit_state,
        lineage.authority.as_str(),
        lineage.event.as_str(),
        lineage.phase.as_str(),
        lineage.order,
    )
}

fn decode_lineage(
    value: &Json,
    revision: &Revision,
    post_commit: &StateRevision,
) -> Result<EffectLineage> {
    let item = list(value, 9, "effect lineage")?;
    require_string(&item[0], "effect-lineage", "effect lineage tag")?;
    let model_revision = string(
        tagged(&item[3], "model-revision", "effect Model Revision")?,
        "effect Model Revision identity",
    )?;
    let state_revision = string(
        tagged(&item[4], "state-revision", "effect StateRevision")?,
        "effect StateRevision identity",
    )?;
    if model_revision != revision.identity().to_string()
        || state_revision != post_commit.identity().as_str()
    {
        return Err(KernelError::new("effect trace has broken Revision lineage"));
    }
    let lineage = EffectLineage {
        producer: ReferentId::new(
            string(
                tagged(&item[1], "producer", "effect producer")?,
                "effect producer identity",
            )?
            .into(),
        )?,
        request: ReferentId::new(
            string(
                tagged(&item[2], "request", "effect request")?,
                "effect request identity",
            )?
            .into(),
        )?,
        input_model_revision: revision.identity().clone(),
        post_commit_state: post_commit.identity().clone(),
        authority: ReferentId::new(
            string(
                tagged(&item[5], "authority", "effect authority")?,
                "effect authority identity",
            )?
            .into(),
        )?,
        event: ReferentId::new(
            string(
                tagged(&item[6], "event", "effect event")?,
                "effect event identity",
            )?
            .into(),
        )?,
        phase: ReferentId::new(
            string(
                tagged(&item[7], "phase", "effect phase")?,
                "effect phase identity",
            )?
            .into(),
        )?,
        order: string(
            tagged(&item[8], "order", "effect order")?,
            "effect order value",
        )?
        .parse()
        .map_err(|_| KernelError::new("invalid effect order"))?,
    };
    validate_effect_lineage(&lineage, revision, post_commit)?;
    Ok(lineage)
}

fn authorization_preimage(lineage: &EffectLineage, decision: AuthorizationDecision) -> String {
    format!(
        "[{},[\"decision\",\"{}\"]]",
        lineage_json(lineage),
        decision.as_str()
    )
}

fn authorization_json(value: &Authorization) -> String {
    format!(
        "[\"authorization\",\"{}\",{}]",
        value.identity,
        authorization_preimage(&value.lineage, value.decision)
    )
}

fn decode_authorization(
    value: &Json,
    revision: &Revision,
    post_commit: &StateRevision,
) -> Result<Authorization> {
    let item = list(value, 3, "Authorization record")?;
    require_string(&item[0], "authorization", "Authorization record tag")?;
    let claimed = AuthorizationId::new(string(&item[1], "Authorization identity")?.into())?;
    let body = list(&item[2], 2, "Authorization body")?;
    let lineage = decode_lineage(&body[0], revision, post_commit)?;
    let decision = match string(
        tagged(&body[1], "decision", "Authorization decision")?,
        "Authorization decision value",
    )? {
        "denied" => AuthorizationDecision::Denied,
        "authorized" => AuthorizationDecision::Authorized,
        _ => return Err(KernelError::new("invalid Authorization decision")),
    };
    let authorization = Authorization::new(lineage, decision);
    if authorization.identity != claimed {
        return Err(KernelError::new("Authorization identity does not match"));
    }
    Ok(authorization)
}

fn attempt_preimage(authorization: &AuthorizationId, lineage: &EffectLineage) -> String {
    format!(
        "[[\"authorization\",\"{}\"],{}]",
        authorization,
        lineage_json(lineage)
    )
}

fn attempt_json(value: &Attempt) -> String {
    format!(
        "[\"attempt\",\"{}\",{}]",
        value.identity,
        attempt_preimage(&value.authorization, &value.lineage)
    )
}

fn decode_attempt(value: &Json, authorization: &Authorization) -> Result<Attempt> {
    let item = list(value, 3, "Attempt record")?;
    require_string(&item[0], "attempt", "Attempt record tag")?;
    let claimed = AttemptId::new(string(&item[1], "Attempt identity")?.into())?;
    let body = list(&item[2], 2, "Attempt body")?;
    if string(
        tagged(&body[0], "authorization", "Attempt authorization")?,
        "Attempt Authorization identity",
    )? != authorization.identity.as_str()
    {
        return Err(KernelError::new("Attempt has broken Authorization lineage"));
    }
    if json(&body[1]) != lineage_json(&authorization.lineage) {
        return Err(KernelError::new("Attempt has divergent effect lineage"));
    }
    let attempt = Attempt::new(authorization);
    if attempt.identity != claimed {
        return Err(KernelError::new("Attempt identity does not match"));
    }
    Ok(attempt)
}

fn receipt_preimage(
    attempt: &AttemptId,
    lineage: &EffectLineage,
    outcome: ReceiptOutcome,
) -> String {
    format!(
        "[[\"attempt\",\"{}\"],{},[\"outcome\",\"{}\"]]",
        attempt,
        lineage_json(lineage),
        outcome.as_str()
    )
}

fn receipt_json(value: &Receipt) -> String {
    format!(
        "[\"receipt\",\"{}\",{}]",
        value.identity,
        receipt_preimage(&value.attempt, &value.lineage, value.outcome)
    )
}

fn decode_receipt(value: &Json, attempt: &Attempt) -> Result<Receipt> {
    let item = list(value, 3, "Receipt record")?;
    require_string(&item[0], "receipt", "Receipt record tag")?;
    let claimed = ReceiptId::new(string(&item[1], "Receipt identity")?.into())?;
    let body = list(&item[2], 3, "Receipt body")?;
    if string(
        tagged(&body[0], "attempt", "Receipt attempt")?,
        "Receipt Attempt identity",
    )? != attempt.identity.as_str()
    {
        return Err(KernelError::new("Receipt has broken Attempt lineage"));
    }
    if json(&body[1]) != lineage_json(&attempt.lineage) {
        return Err(KernelError::new("Receipt has divergent effect lineage"));
    }
    let outcome = match string(
        tagged(&body[2], "outcome", "Receipt outcome")?,
        "Receipt outcome value",
    )? {
        "succeeded" => ReceiptOutcome::Succeeded,
        "failed" => ReceiptOutcome::Failed,
        _ => return Err(KernelError::new("invalid Receipt outcome")),
    };
    let receipt = Receipt::new(attempt, outcome);
    if receipt.identity != claimed {
        return Err(KernelError::new("Receipt identity does not match"));
    }
    Ok(receipt)
}

fn observation_preimage(
    receipt: &ReceiptId,
    lineage: &EffectLineage,
    evidence: &ReferentId,
) -> String {
    format!(
        "[[\"receipt\",\"{}\"],{},[\"evidence\",\"{}\"]]",
        receipt,
        lineage_json(lineage),
        evidence.as_str()
    )
}

fn observation_json(value: &Observation) -> String {
    format!(
        "[\"observation\",\"{}\",{}]",
        value.identity,
        observation_preimage(&value.receipt, &value.lineage, &value.evidence)
    )
}

fn decode_observation(value: &Json, receipt: &Receipt, revision: &Revision) -> Result<Observation> {
    let item = list(value, 3, "Observation record")?;
    require_string(&item[0], "observation", "Observation record tag")?;
    let claimed = ObservationId::new(string(&item[1], "Observation identity")?.into())?;
    let body = list(&item[2], 3, "Observation body")?;
    if string(
        tagged(&body[0], "receipt", "Observation receipt")?,
        "Observation Receipt identity",
    )? != receipt.identity.as_str()
    {
        return Err(KernelError::new("Observation has broken Receipt lineage"));
    }
    if json(&body[1]) != lineage_json(&receipt.lineage) {
        return Err(KernelError::new("Observation has divergent effect lineage"));
    }
    let evidence = ReferentId::new(
        string(
            tagged(&body[2], "evidence", "Observation evidence")?,
            "Observation evidence identity",
        )?
        .into(),
    )?;
    if !revision.model().referents().contains_key(&evidence) {
        return Err(KernelError::new(
            "effect observation evidence is absent from the checked Model",
        ));
    }
    let observation = Observation::new(receipt, evidence);
    if observation.identity != claimed {
        return Err(KernelError::new("Observation identity does not match"));
    }
    Ok(observation)
}

fn optional_json(value: Option<String>) -> String {
    match value {
        Some(value) => format!("[\"some\",{value}]"),
        None => "[\"none\"]".into(),
    }
}

fn optional<'a>(value: &'a Json, where_: &str) -> Result<Option<&'a Json>> {
    let item = array(value, where_)?;
    match item {
        [tag] if string(tag, where_)? == "none" => Ok(None),
        [tag, value] if string(tag, where_)? == "some" => Ok(Some(value)),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
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

pub(crate) fn validate_policy(model: &Model, policy: &RuntimePolicy) -> Result<()> {
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
) -> Result<(StateDelta, Vec<RelationalContent>)> {
    if events.is_empty() {
        return Err(KernelError::new("runtime tick needs at least one event"));
    }
    let mut event_ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut successors = BTreeSet::new();
    let mut functional_keys = BTreeSet::new();
    let mut withdrawals = Vec::new();
    let mut admissions = Vec::new();
    let mut grounded_contents = BTreeMap::new();
    for event in events {
        if !event_ids.insert(event.id.clone()) {
            return Err(KernelError::new(
                "runtime tick repeats an event occurrence identity",
            ));
        }
        if !model.referents().contains_key(&event.event) {
            return Err(KernelError::new(
                "runtime event pattern is absent from the checked Model",
            ));
        }
        let transitions = model
            .transitions()
            .iter()
            .filter(|transition| transition.event() == &event.event)
            .collect::<Vec<_>>();
        let Some(first) = transitions.first() else {
            return Err(KernelError::new(
                "runtime event names no checked transaction",
            ));
        };
        if event.payload.len() != first.payload_bindings().len() {
            return Err(KernelError::new(
                "runtime event payload does not match its checked binding shape",
            ));
        }
        for term in &event.payload {
            validate_event_payload(model, term)?;
        }
        let substitution = first
            .payload_bindings()
            .iter()
            .cloned()
            .zip(event.payload.iter().cloned())
            .collect::<BTreeMap<_, _>>();
        let mut matches = vec![(substitution, Vec::<AssertionOccurrence>::new())];
        for transition in &transitions {
            let before = model
                .content(transition.from())
                .expect("checked transition source is registered");
            let mut next = Vec::new();
            let contents = state
                .relation_index
                .get(before.relation())
                .cloned()
                .unwrap_or_default();
            for (substitution, selected) in matches {
                for content_id in &contents {
                    let actual = state
                        .catalog
                        .get(content_id)
                        .expect("pre-state relation index names registered content");
                    let Some(substitution) = crate::kernel::matching::unify(
                        before,
                        actual,
                        &substitution,
                        true,
                        |id| model.content(id),
                        |id| state.catalog.get(id).or_else(|| model.content(id)),
                    ) else {
                        continue;
                    };
                    for occurrence in state.occurrence_index.get(content_id).into_iter().flatten() {
                        let target = state
                            .occurrences
                            .get(occurrence)
                            .expect("occurrence index names active occurrence");
                        let mut selected = selected.clone();
                        selected.push(target.clone());
                        next.push((substitution.clone(), selected));
                    }
                }
            }
            matches = next;
            for guard_id in transition.guards() {
                let guard = model
                    .content(guard_id)
                    .expect("checked transition guard is registered");
                let candidates = state
                    .relation_index
                    .get(guard.relation())
                    .cloned()
                    .unwrap_or_default();
                let mut next = Vec::new();
                for (substitution, selected) in matches {
                    for actual_id in &candidates {
                        let actual = state
                            .catalog
                            .get(actual_id)
                            .expect("pre-state relation index names registered content");
                        if let Some(substitution) = crate::kernel::matching::unify(
                            guard,
                            actual,
                            &substitution,
                            true,
                            |id| model.content(id),
                            |id| state.catalog.get(id).or_else(|| model.content(id)),
                        ) {
                            next.push((substitution, selected.clone()));
                        }
                    }
                }
                matches = next;
            }
        }
        if matches.is_empty() {
            return Err(KernelError::new(
                "runtime event transaction has no joint pre-state and guard match",
            ));
        }
        if matches.len() != 1 {
            return Err(KernelError::new(
                "runtime event transaction match is ambiguous",
            ));
        }
        let (substitution, selected) = matches.pop().expect("one event match remains");
        for (transition, target) in transitions.into_iter().zip(selected) {
            let after_pattern = model
                .content(transition.to())
                .expect("checked transition destination is registered");
            let instantiated =
                crate::kernel::matching::instantiate(after_pattern, &substitution, |id| {
                    model.content(id)
                })?;
            let after_id = instantiated.root.id().clone();
            let dependencies = instantiated.dependencies.into_values().collect::<Vec<_>>();
            model.validate_query_content(&instantiated.root, &dependencies)?;
            for content in dependencies
                .into_iter()
                .chain(std::iter::once(instantiated.root))
            {
                if let Some(existing) =
                    grounded_contents.insert(content.id().clone(), content.clone())
                    && existing != content
                {
                    return Err(KernelError::new(
                        "runtime event grounded conflicting content identity",
                    ));
                }
            }
            let after = grounded_contents
                .get(&after_id)
                .expect("grounded transition destination was registered");
            let before = state
                .catalog
                .get(target.content())
                .expect("selected pre-state occurrence content is registered");
            for key in functional_replacement_keys(model, before, after) {
                if !functional_keys.insert(key) {
                    return Err(KernelError::new(
                        "runtime tick has conflicting writes to one functional relation key",
                    ));
                }
            }
            if !targets.insert(target.id().clone()) {
                return Err(KernelError::new(
                    "runtime tick has conflicting writes to one pre-state occurrence",
                ));
            }
            let successor = event_successor(event, transition.id(), target.id(), after.id());
            if !successors.insert(successor.clone()) || state.occurrences.contains_key(&successor) {
                return Err(KernelError::new(
                    "runtime tick has a conflicting successor occurrence identity",
                ));
            }
            withdrawals.push(target.id().clone());
            admissions.push(AssertionOccurrence::new(
                successor,
                after.id().clone(),
                event.id.clone(),
                model.id().clone(),
            ));
        }
    }
    Ok((
        StateDelta::new(withdrawals, admissions)?,
        grounded_contents.into_values().collect(),
    ))
}

fn validate_event_payload(model: &Model, term: &Term) -> Result<()> {
    if !model.term_is_ground(term) {
        return Err(KernelError::new("runtime event payload must be ground"));
    }
    let mut result = Ok(());
    term.walk(&mut |term| {
        if result.is_err() {
            return;
        }
        let referents = match term {
            Term::Referent(id) => vec![id],
            Term::Product { shape, fields } => std::iter::once(shape)
                .chain(fields.values().map(|field| field.domain()))
                .collect(),
            Term::LabelledProduct { shape, fields } => {
                std::iter::once(shape).chain(fields.keys()).collect()
            }
            Term::Sequence { shape, element, .. } => vec![shape, element],
            _ => Vec::new(),
        };
        if referents
            .into_iter()
            .any(|id| !model.referents().contains_key(id))
        {
            result = Err(KernelError::new(
                "runtime event payload names a referent absent from the checked Model",
            ));
        }
    });
    result
}

fn event_successor(
    event: &TransitionEvent,
    transition: &ReferentId,
    target: &ReferentId,
    content: &ContentId,
) -> ReferentId {
    ReferentId::from_digest(sha256_digest(
        format!(
            "clause-runtime-event-successor-v1\0{}\0{}\0{}\0{}",
            event.id.as_str(),
            transition.as_str(),
            target.as_str(),
            content.as_str()
        )
        .as_bytes(),
    ))
}

fn functional_replacement_keys(
    model: &Model,
    before: &RelationalContent,
    after: &RelationalContent,
) -> Vec<(ReferentId, Vec<(RoleId, Term)>)> {
    if before.relation() != after.relation() {
        return Vec::new();
    }
    model
        .relation_shapes()
        .get(before.relation())
        .into_iter()
        .flat_map(|shape| shape.lookup())
        .filter(|mode| {
            mode.cardinality() == &crate::kernel::Cardinality::One
                && mode
                    .known()
                    .iter()
                    .all(|role| before.roles().get(role) == after.roles().get(role))
                && mode
                    .sought()
                    .iter()
                    .any(|role| before.roles().get(role) != after.roles().get(role))
        })
        .map(|mode| {
            (
                before.relation().clone(),
                mode.known()
                    .iter()
                    .map(|role| (role.clone(), before.roles()[role].clone()))
                    .collect(),
            )
        })
        .collect()
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
        "[\"event-occurrence\",\"{}\",[\"event\",\"{}\"],[\"payload\",[{}]]]",
        event.id.as_str(),
        event.event.as_str(),
        join(event.payload.iter().map(crate::wire::term_json)),
    )
}

fn decode_event(value: &Json) -> Result<TransitionEvent> {
    let item = list(value, 4, "runtime event")?;
    require_string(&item[0], "event-occurrence", "runtime event tag")?;
    Ok(TransitionEvent::new(
        ReferentId::new(string(&item[1], "runtime event identity")?.into())?,
        ReferentId::new(
            string(
                tagged(&item[2], "event", "runtime event pattern")?,
                "runtime event pattern identity",
            )?
            .into(),
        )?,
        array(
            tagged(&item[3], "payload", "runtime event payload")?,
            "runtime event payload",
        )?
        .iter()
        .map(crate::wire::decode_term)
        .collect::<Result<Vec<_>>>()?,
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
