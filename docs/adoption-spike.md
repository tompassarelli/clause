# Clause Process-First Constitutional Adoption Spike

> **Status:** Constitutional cross-phase falsification contract. The
> [roadmap](roadmap.md) alone records implementation and acceptance status.
>
> **Authority:** Normative only for the experiment deciding whether the
> mechanism in the [foundation](foundation.md) survives. It cannot add Clause
> semantics, canonical syntax, or implementation status.

## Decision question

Can one neutral recursive Term carrier plus checked formation, nominal
Applications, distinct Activations, causal Steps and Runs, typed continuation,
and governed admission carry a general-purpose process-first relational
language while Lean and Rust remain checker/reference and physical boundaries
rather than private semantic authorities?

The decisive questions are:

1. Do ApplicationForm, Application, Activation, Configuration, Step, Run, and
   Continuation have distinct consumers and invariants across pure, effectful,
   and ongoing programs?
2. Can RelationSchemas, extensions, operators, modes, source Readings,
   ExecutionAuthorizations, and capabilities remain distinct without making
   ordinary source bureaucratic?
3. Can Clause add and understand a new language concept by changing accepted
   Clause data alone, or must a host learn a new semantic secret?
4. Can checked implementations specialize aggressively while preserving cold
   process semantics and exact relational observations?
5. Can ordinary loops, builders, caches, and frame state use affine local
   mutation with no Admission, revision, mandatory retained trace, or generic-graph hot
   path?
6. Can statically fixed constitutive evidence erase from the hot ABI while its
   semantic basis remains explainable and dynamic Authorization, capability,
   effect, and Admission evidence remain exact where they vary?
7. Can rank-1 generics, coherent constraints, causal-affine lifetimes, rich
   values, collections, layout, and native/Wasm specialization work without a
   second host language or mandatory GC?

The spike starts from the accepted calculus, consumes exact frozen oracles, and
must justify every claimed behavior directly.

## Frozen and versioned oracles

The three historical v0 source-projection payloads are quarantined under
`clause:test-vectors/execution/historical-v0/source-projections/` with the
noncanonical `.clause-v0.txt` suffix. Their exact contents and original SHA-256
digests are frozen:

- `clause:test-vectors/execution/historical-v0/source-projections/pure-dependency-closure.clause-v0.txt`:
  recursive pure dependency closure;
- `clause:test-vectors/execution/historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt`:
  admitted State change plus separate effect evidence; and
- `clause:test-vectors/execution/historical-v0/source-projections/verified-program-evolution.clause-v0.txt`:
  predecessor-bound Program evolution.

Those payload bytes contain slash-joined designations from the superseded v0
draft. They are not current canonical Clause source and no current reader may
accept them as such. Paths, suffixes, README/checksum content, and
classification metadata are not part of the frozen payload-byte oracle and may
change to keep that quarantine honest. The current `manifest.json` therefore
has no syntax authority, classifies the projections as
`historical-v0-noncanonical-fixture`, declares that no canonical source is
included and no spelling authority exists, and treats every fixture-local name
as one opaque transport string. A `/` in such a string is an uninterpreted Atom
payload byte, never designation structure. The manifest's execution
observations and six v0 outcome tags remain exactly `returned`, `choices`,
`yielded`, `suspended`, `failed`, and `exhausted`. They are compatibility
evidence, not the complete process ontology.

A separately versioned ratified process companion must crosswalk those exact
bytes into ApplicationForms, ApplicationIds, ActivationIds, StepIds, RunIds,
ContinuationIds where a boundary is crossed, ObservationIds, exact program and
world pins, and candidate/admission evidence. The current process-v2 Rust corpus
is a reduced experimental input, not that ratified companion. The companion
must not rewrite or infer new identity from fixture-local v0 names. Source
movement, duplicate equal occurrences, n-ary role closure, closure capture,
pure arithmetic, an ongoing service, cancellation, budget exhaustion, and
effect timeout require new ratified process fixtures.

The historical `game_leverage` position/radius law and later implementation
attempts are experimental evidence only. Preserve the unchanged source law as
a cold-semantics oracle; do not promote host-selected relation meanings or
materializer-owned admission from any attempt. Candidate identities, review
dispositions, and current sequencing belong only in the
[roadmap](roadmap.md), never in this constitutional contract.

## Phase A — Minimal process constitution

Before surface implementation, define the generic core required by every gate:

```text
Atom(kind, canonical payload, declared equality contract)
RawTriple = [Term, Term, Term]
Term = Atom | RawTriple

Γ ⊢ t : T @ interpretation

Γ ⊢ form(t, exact RelationSchemaRef, exact OperatorRef,
         exact named-role bindings, exact eligible ModeId set)
  : ApplicationForm<ResultDomain>

Application(ApplicationId, exact ApplicationForm)

activate(ActivationStartRecord)
  = ActivationId + RunMembership + InitialConfiguration

StepCauseFrontier := finite canonical set of StepCause

StepConfigurationTransition(s : StepId) := Serial | Split | Branch | Join

consume exact affine ConfigurationCustody_before + optional Wbase
  -- StepRecord(s = fresh StepId,
                owner = (RunId, ActivationId),
                causes = StepCauseFrontier,
                transition = exact StepConfigurationTransition(s),
                observations, outcome, delta, continuation) -->
produce exact affine ConfigurationCustody_after + same optional Wbase

IncomingRunEdges(s) := StepCauseFrontierEdges(s)
                       ∪ StepConfigurationSuccessionEdges(s)

Run(RunId, root = ActivationId,
    order = transitive closure(all IncomingRunEdges))

admit(BaseRevision, candidate delta, evidence,
      AuthorizationEvidence<AdmissionAuthorization>,
      JudgmentOccurrences, obligations)
  = (AdmissionOccurrenceId, SuccessorRevision | Rejection)
```

The core must represent:

- contextually opaque Atoms and explicit refinements across universes;
- structurally neutral Triple slots and structural Term equality indexed by
  universe and semantics epoch;
- contextual ClauseJudgment and its FormationJudgment specialization, both
  distinct from governed Judgment and JudgmentOccurrence;
- closed ApplicationForms with exact OperatorRef, RelationSchema, named-role
  closure, mode eligibility, and context requirements;
- snapshot-local RelationSchema, Role, Operator, and Mode declarations whose
  exact external references include ProgramSnapshotId and never silently carry
  across a changed snapshot;
- `ApplicationShapeId` only for closed forms, committing to ClauseSemanticsId,
  exact RelationSchemaId, exact OperatorRef, the exact eligible ModeId set,
  named-role bindings, context requirements, exact InstantiationUseRefs with
  their InstantiationKeys and SpecializationKeys, and
  the full resolved semantic-dependency/declaration closure, including proof
  that it is empty where applicable; PhysicalReuseKey is excluded;
- canonical same-snapshot instantiation records use local declaration,
  argument, named-obligation, resolution-scope, and evidence references in the
  ProgramSnapshot preimage; InstantiationUseRefs resolve only after the one
  snapshot hash, while InstantiationKeys and SpecializationKeys derive from
  independent canonical content and never return to that preimage;
- mandatory nominal `ApplicationId` for every Application, with raw, quoted,
  open, and merely structural forms remaining non-nominal ApplicationForms or
  Terms rather than anonymous Applications;
- configured binders, transfers, requests, and tasks distinguished from actual
  event/effect occurrences, which carry typed OccurrenceId plus exact
  provenance: producing Activation/Step for internal production or external-
  boundary provenance for an ingress trigger;
- fresh `ActivationId` for every engagement and one stable Activation across
  any number of configurations and StepIds;
- exact `StaticActivationBasis` for every engagement, separated from a Mode's
  finite named/RoleId-indexed, multiplicity-aware DynamicPrerequisiteSchema,
  exact bindings, and occurrence-only cause frontier; the entire schema may be
  empty and equal bindings in distinct slots never collapse;
- exact CheckedConstitutionBinding selecting either checked non-authoritative
  candidate package/snapshot bytes or an admitted ProgramRevision; the first
  may read an exact admitted world, persist nonauthoritative output or a
  Continuation, and use an inert effect simulator, but fabricates no revision,
  real EffectAttemptOccurrence, or constitutive Authorization;
- affine Activation-local slots, bounded Step-local scratch, anonymous internal
  reductions, exact Step-cut and escape rules, and checked in-place lowering
  without Admission or revision creation;
- a closed `StepConfigurationTransition` sum: Serial consumes and succeeds one
  whole token; Split uses one Mode-owned, canonical multiplicity-aware contract
  with pairwise-disjoint exact coverage and typed
  `BranchSlot = (BranchKey, repeated-spec ordinal)` to consume that token into
  structurally anchored branch tokens; Branch advances or settles one exact
  BranchSlot; Join consumes one canonical settlement per BranchSlot and restores
  one whole owner;
- atomic SplitFormation that validates and co-publishes one fresh split Step,
  its SplitInstance, fresh ChildOf/ChildIn Activation and initial token per
  BranchSlot, every exact binding, and all branch tokens, or publishes none;
- no residual whole token after Split, no new Split/Join/Settlement identity
  domain, and exact discharge or transfer of roots, Borrows, Leases,
  Continuations, effects, and close obligations before a branch can close;
- finite typed StepCauseFrontiers built from exact ActivationStart, PriorStep,
  ContinuationTakeup, and CancellationRequest causes; a normal first Step is the
  exact ActivationStart singleton, ready cancellation is its sole exact pair
  exception, and a nonfirst frontier may be empty only when the transition
  contributes a configuration predecessor; every nonfirst Step has nonempty
  IncomingRunEdges, so concurrency remains a partial order rather than log
  order without inserting an implicit PriorStep;
- RunId as a causal envelope distinct from ActivationId, including child
  activation, handoff, and cancellation scope;
- typed continuation as semantic remainder, with the sole affine configuration
  token and exact ActivationStartRecord when it crosses suspension, handoff,
  persistence, or executor boundaries;
- ObservationId distinct from observed Value and Result;
- immutable typed candidate deltas separate from continuations;
- activation-scoped result relations, separately admitted revision-indexed
  relation extensions, and occurrence-exact support;
- total, productive, bounded, partial, nondeterministic, streaming, reactive,
  and effectful Mode contracts;
- distinct Reading, derivation authorization, ExecutionAuthorization,
  admission authority, and effect/resource capability;
- one rank-1 declaration-level StaticParameterTelescope plus named
  StaticConstraintTelescope, total Clause-owned static normalization,
  terminating resolution contracts, per-obligation ResolutionScopeCommitments,
  normalized explicit evidence, closed uses, distinct checking/
  specialization/physical reuse keys, and separate compilation;
- exactly one `Owned`, `RegionMember`, or `ForeignManaged` allocation root plus
  zero or more typed Borrow/Lease access edges, deterministic semantic
  retirement, explicit close/dispose before mechanical reclaim, explicit cycle
  disposition, and honest foreign boundaries;
- explicit bounded trace-retention contracts for long-lived profiles;
- non-operator RelationSchemas able to form checked bindings, proposition and
  assertion content, rows, and patterns without forming ApplicationForms;
- source occurrences, scope, binding, quotation, hygiene, phase, and origin;
- immutable ProgramSnapshot, ProgramRevision, RuntimeSession, and StateRevision
  boundaries with exact pinning and no silent migration; and
- canonical package bytes with cycle-aware, terminating, fail-closed reload.

Raw Triples receive no mandatory nominal identity. A relation may exist without
an executable Mode, and a Mode may declare no dynamic
`ExecutionAuthorization` requirement. Private interning handles, Wasm handles,
pointers, table indexes, paths, spans, or log positions cannot escape as
semantic identity.

Identity allocation itself is typed and checked; authoritative retention is
governed only where the identity crosses such a boundary. The ratified process
carrier must record the exact allocation basis and, where declared, authority
and predecessor evidence: one unique ApplicationLocalId plus its checked form
inside an exact ProgramSnapshot allocates an ApplicationId, while Program
admission is additionally required to make that Application part of an
authoritative constitution; successful statically valid activation whose
selected Mode's declared dynamic prerequisites hold allocates an Activation and
Run root; actual carry-through allocates a Step; boundary-crossing remainder
allocates a
Continuation, actual distinction for an Observation or Occurrence, and
constitutional Admission for a revision. Content hashing, caller-supplied
bytes, possession of a serialized object, or a physical allocator cannot mint
one of those identities.

For every identity domain, the companion includes exact wrong-kind,
wrong-allocation-basis, wrong-authority where applicable, self-authorizing,
equal-content transplant, and already-used occurrence negatives. Replaying a
record with its existing identity may be an
idempotent observation of history, but presenting that identity as a fresh
Application allocation, Activation, Step, occurrence, continuation use, or
Admission rejects before partial authority. An explicitly declared continuity
relation may retain a permitted identity; equality alone never supplies that
relation.

### Lean trust profile

The constitutional result is admissible only when:

- Lean source, toolchain, imports, and artifacts have exact hashes;
- all newly added declarations use `trustLevel = 0` and the transitive
  constitutional closure is replayed into a fresh kernel environment;
- every reachable `unsafe` or `partial` declaration is rejected;
- the closure contains no `sorry`, `sorryAx`, skipped checking, recovery axiom,
  failed-declaration fallback, or preliminary asynchronous declaration;
- no proof relies on `native_decide`, native reduction, executed
  `implemented_by`/`extern` replacement, a foreign implementation, or a bare
  compiled Boolean;
- the axiom closure matches an explicit policy, including deliberate treatment
  of `propext`, `Quot.sound`, and `Classical.choice`;
- every proof is bound to the exact package bytes, semantics epoch, decoded
  value, and Clause proposition it certifies; and
- same-kernel replay is not misrepresented as an independent verifier.

The decoder, object-language model, certificate proposition, and theorem
connecting certificate acceptance to Clause validity are part of the audited
trusted boundary. Their size and dependency closure are measured.

### Rust and compiler-machine boundary

Rust independently decodes the same package and may build physical indexes,
stores, interpreters, schedulers, adapters, or generated plans. Its output must
agree with the reference process relation for every declared observable and
nonfunctional contract. Rust may not reimplement semantic classification
through a closed feature enum, source-form match, semantic-name switch, opaque
callback, or private side table.

The exact CLCP v3 bytes, carried hashed manifest, 73-byte receipt,
left-to-right KExpr evaluator, machine Continuation, evaluator step, fuel
exhaustion, `admitPropose`, and `CompilerRevisionId` remain compiler-machine
mechanics. They do not define semantic Activation, Step, Run, Continuation, or
general Admission. Clause-owned outer Terms and envelopes carry those process
objects through the fixed machine.

## Phase B — Early pure/local general-purpose slice

The first executable slice deliberately tests ordinary programming before the
full process/effect breadth. It must inhabit the same accepted carrier,
Compiler0, package, runtime, and physical refinement path later phases extend;
a disposable host interpreter does not pass. Rich values, collections,
parametric reuse, local mutation, ownership, erasure, physical IR, and
native/Wasm lowering are load-bearing early evidence rather than Phase 7
optimization polish.

Before the binding, algebraic-data, n-ary, or local-state cases below count as
passed general-purpose evidence, the exact fourth source specimen ratified in
Phase B.13 must pass `parse`, `check`, canonical print/parse round trip, and one
transactional workbench edit with exact typed diagnostics and dependency
invalidation. Semantic-IR experiments may precede that proof, but cannot
substitute for the source and feedback-loop gate.

### 1. Pure arithmetic and repeated activation

Represent a pure deterministic integer addition Application. Activate one exact
Application twice. Required observations:

- one ApplicationId and two distinct ActivationIds and RunIds;
- two independently nominalized equal-shaped Applications have distinct
  ApplicationIds;
- distinct Step and Observation occurrences even when returned Values are
  equal;
- expression Term, Application, result Value, and denotation remain separate;
- no ProgramRevision or StateRevision is created; and
- a direct-call specialization is observationally equivalent.

The same pure tranche must exercise truth-directed evaluation without
assertion. Under one exact finite interpretation, use three independently
identified proposition contents: one supported true, one explicitly supported
false, and one absent. The three activations emit exact `true`, `false`, and
`absent` observations with occurrence-exact positive or negative supports;
absence carries no negative support and is not rewritten as false. Formation,
activation, observation, and return create no AssertionOccurrence, governed
Judgment, ProgramRevision, or StateRevision. A later assertion or Judgment is
a separate identified act and changes only the boundary its policy says it
changes.

### 2. Binding, closure, and hygienic compilation

Represent a binder-introducing function, lexical capture, application, and a
binder-introducing macro/compiler process.

- Binder identity is independent of spelling and source position.
- Every use resolves through explicit scope relations.
- Captured identity is inspectable Clause data, not host environment state.
- Alpha-equivalent forms may be denotationally equivalent without sharing
  occurrence or Application identity.
- Macro/compiler Steps produce a checked candidate Program delta with origins,
  obligations, and evidence; only admission creates the successor revision.

### 3. User-defined algebraic data and exhaustive matching

Represent a user-defined sum type, constructors, patterns, and exhaustive match
as Clause-authored declarations and process definitions. The checker accepts an
exhaustive match and rejects missing and unreachable cases with exact
obligations. Pattern binding and result type remain graph-native and
source-projectable. No Term, KExpr, Lean, or Rust kernel constructor may be
added for the feature. The first fixture is semantic IR and observations; it
does not ratify algebraic-data or match source spelling.

### 4. Exact n-ary role formation

Represent a transfer with actor, amount, source, and destination roles. Two
equal structural descriptions may instantiate distinct Applications. Missing,
extra, duplicate, wrong-type, wrong-cardinality, or position-recovered roles
reject. One RelationSchema with no eligible executable Mode remains
inspectable but cannot activate.

### 5. General-purpose values, local state, and static reuse

One semantic-IR tranche must combine rich numeric values, Unicode Text, Bytes,
an algebraic option/result, immutable and locally built sequences and maps, and
one rank-1 parametric collection operation. It checks a declaration-level
StaticParameterTelescope and named StaticConstraintTelescope, total canonical
static normalization, two valid normalized instantiations, an unsatisfied-
constraint negative after complete resolution, an incoherent-ambiguity
negative, a budget-exhausted/indeterminate negative, and a separately compiled
consumer. Source movement preserves semantic keys and DiagnosticObligations;
`Renameπ`, well-formed substitution, solver results, and diagnostics transform
equivariantly rather than remaining byte-identical.

Each obligation records a complete ResolutionScopeCommitment. Adding a
potentially overlapping candidate invalidates exactly the affected checking
key; adding an unrelated declaration in another dependency frontier preserves
the exact checking, specialization, and physical cache sets. A body-only edit
preserves interface checking reuse but invalidates its SpecializationKey. A
target, compiler/refinement, feature, ABI/layout, strategy, or physical-
dependency edit invalidates PhysicalReuseKey even when InstantiationKey is
unchanged. Cyclic static/evidence instantiation rejects before key allocation;
ordinary runtime recursion after static closure remains legal.

Positive fixtures include one self-recursive and one mutually recursive closed
generic specialization. Each finite call-graph SCC receives canonical local
member anchors and one alpha-normalized `SpecializationSccKey`; member
`SpecializationKey`s select from that object without recursively hashing one
another. Source order, spelling, traversal, and source-only movement preserve
the keys, while any member body or edge edit invalidates the complete SCC and
its dependents.

The tranche also performs an ordinary loop, request-local cache update, and
buffer builder inside one affinely owned ActivationConfiguration. It fixes the
Activation-local slot set, Step-local scratch bound, mandatory Step cuts,
before/after configurations, escape checks, exact allocation roots, Borrow/
Lease edges, trace-retention contract, and resource balance. Thousands of
internal reductions create no extra StepId, StateRevision, Admission, or
mandatory trace record. Functional and in-place implementations of the pure
builder agree observationally. Forced failure or cancellation at every move,
write, drop, and Lease boundary restores the exact consumed
`ConfigurationCustody_before`—whole token, branch token, or canonical
settlement sequence as selected by the transition—and the resource ledger
through an infallible suffix, bounded undo/shadow, or unpublished realization,
with no duplicate or residual custody. Borrow/Lease compatibility is checked across each
root's complete alias set including owner access, and reset/reclaim waits for
causal quiescence acknowledgments from every Borrow, Lease, Continuation, child,
escape, asynchronous/foreign use, and close obligation. Shared mutable alias,
overlapping write Lease, Step-scratch escape, double continuation takeup, a
cancelled split branch whose `Closed` settlement leaves an exact
`AllocationRoot` (`Owned`, `RegionMember`, or `ForeignManaged`, including a
Clause-owned foreign-wrapper obligation), Borrow, Lease, Continuation, effect,
or close obligation neither discharged nor transferred exactly as declared,
missing/double join, any strong cycle across
independently reclaimed roots including Owned↔Owned, unknown close obligation,
observable destructor-at-reclaim, and use-after-move each reject at the exact
formation or lifetime stage. A bounded compiler-proven nonobservable mechanical
drop remains legal. A separate non-game fixture contains one ownership cycle
inside an explicitly selected `ManagedIsland` with exact finite external roots,
capacity, collection strategy, work/pause budget, trigger, and typed overflow;
an escaping strong edge or open semantic obligation rejects. The island is not
a default heap and is unavailable on the controlled game hot path.

The same accepted meaning lowers through a Clause-owned physical IR to a direct
native artifact and a Wasm artifact. Monomorphized, evidence-dictionary,
irrelevant-evidence-erased, and shared-code strategies agree on semantic
identity, values, failures, observations, occurrence support, diagnostics,
resource outcomes, and declared layout/ABI. Every strategy retains the exact
cold link from InstantiationUseRef through InstantiationKey, SpecializationKey,
strategy-specific PhysicalReuseKey, and ArtifactId without collapsing nominal
Applications or Activations. The hot ABI contains no static evidence or
Authorization token for a Mode whose entire dynamic-prerequisite schema is
empty. Issued or effect authority is not part of this pure fixture and cannot
be fabricated.

The native/Wasm game subprofile preallocates declared frame regions, buffers,
active-frontier, continuation, and trace capacity. `capacity + 1` and partial
initialization failure publish nothing and close every established root/Lease;
an already attempted foreign allocation records cleanup success, failure, or
pending quarantine rather than claiming atomic rollback.
After initialization, its Clause/Wasm/adapter-controlled loop performs zero
allocation, `memory.grow`, global scan, whole-carrier clone, observable
destructor/finalizer work, implicit ARC work, or unbounded teardown. Exact receipts
record allocation calls/bytes, pool high-water, Wasm pages, adapter calls, and
resource ledger before/after. Foreign calls retain declared contracts; a whole-
browser zero-allocation claim requires instrumented warm-up/lazy-cache evidence.
The fixture does not require every release time to be statically known; it
proves the runtime causal boundary at which each obligation closes.

### 6. Agent-native workbench proof

One long-lived stdio workbench must execute accepted Clause package definitions
through CLCP03 and the generic runtime. Its exact request surface is `parse`,
`check`, `explain`, `query`, `diff`, `propose`, `admit`, `run`, and `hotReload`.
Rust owns bounded framing, exact pins, cache storage, transactions, and
scheduling only. A Rust parser, type/constraint checker, semantic query engine,
diagnostic switch, or alternate evaluator fails the gate.

The first proof uses one arithmetic singleton-field constraint and one
RelationSchema with one
request. A transactional source edit must produce stable typed diagnostics,
exact semantic dependencies, `why`, `prevent`, `achieve`, and `diff` answers,
then a pure `run` result with no StateRevision. `propose` returns a candidate
against one exact base; `admit` remains a distinct governed operation;
`hotReload` preserves or rejects exact live pins and never silently migrates an
Activation. Interactive calls may use accepted incremental summaries and do
not rerun the full Lean/compiler-succession replay; exact replay remains the
promotion gate. Compact Clause source stays the human audit and token surface.

Every negative response fixes a stable typed code, rejecting semantic stage,
exact failed formation or subject, unsatisfied obligation, source-origin set,
dependency slice, and authoritative boundaries proven unchanged. Any suggested
edit is separately typed advice, never a hidden rewrite or authority grant. An
edit request names the exact source/package base and applies atomically; a stale
base rejects with the current identity and no partial text or semantic change.

Before sections 2–5 count as passed, this same workbench must also parse,
check, canonical-print/parse, explain, edit, and rerun the combined authored
generic/loop/builder/move/borrow/region/Lease specimen. The edit transcript
fixes the exact affected and preserved dependency/cache sets; a semantic-IR-
only side door does not pass.

## Phase C — Process, effect, and integration breadth

The remaining cases extend that same implementation with ongoing running,
governed state, effects, materialization, host-freeze evolution, and passive
rendering. Separate toy runtimes that merely share names do not pass.

### 7. Recursive, streaming, and ongoing running

Represent recursive dependency closure and an ongoing service or actor. One
Activation must produce several Steps, yield, suspend, persist an identified
Continuation, resume under exact pins, and remain live without manufacturing a
terminal result. Cancellation, terminal timeout, and budget exhaustion have
typed outcomes. An ungrounded cycle remains distinguishable from a productive
ongoing process.

The continuation gate is a restart, not an in-memory pause disguised as one.
The fixture must suspend after an exact `suspend-step`, serialize the complete
identified Continuation, destroy its executor, rematerialize it in an
independent runtime, and accept one fresh ingress observation that did not
exist before suspension. The resumed Step:

- retains the original ApplicationId, ActivationId, and RunId;
- has a fresh StepId whose StepCauseFrontier contains exactly
  `ContinuationTakeup(exact ContinuationId, original RunId,
  original ActivationId, suspend-step, exact ResumptionOccurrenceId)`; the
  emitting Step is part of that cause and is not duplicated as `PriorStep`;
- binds the newly identified ingress occurrence through that exact
  ResumptionOccurrence under its declared mode;
- emits a fresh ObservationId rather than replaying a cached observation;
- retains one exact ActivationStartRecord covering StaticActivationBasis,
  InitialContext, DynamicPrerequisiteBindings, and the occurrence-only cause
  frontier; every fixed Application, Mode, constitution, initial-world,
  session, policy, semantics, budget, and cancellation-scope pin derives from
  that record rather than a duplicate authoritative field;
- consumes the sole serialized affine configuration token; and
- creates no ProgramRevision or StateRevision unless a separately staged
  candidate is admitted.

Because this fixture carries affine configuration, the first valid resume
consumes one exact continuation-use/configuration token. A repeated or
concurrent second use rejects as `continuation-already-consumed` before a Step,
observation, effect attempt, or delta exists. A separate immutable/`Copy`
remainder fixture may select reusable takeup; any mutable reusable fork must
create fresh child Activation and configuration identities rather than
duplicating the original token.

Every pinned field has an exact one-field stale negative. Transplanting the
unchanged continuation bytes into an equal-shaped but independently nominal
Application, another Activation or Run, another RuntimeSession, or another
continuation-use authority also rejects before carry-through. The rejection
must name the mismatched identity class without treating equal content, a host
handle, or possession of bytes as authority.

The concurrency gate varies physical execution while keeping semantic
causality fixed. A parent Activation consumes one configuration owner under one
Mode-owned `SplitJoinContract` and splits exact nonoverlapping left/right
subconfiguration tokens through one atomic `SplitFormation`. Under fresh
binders it validates and co-forms the split Step and SplitInstance plus one
`BranchSlot`, matching BranchSpec, fresh child Activation, exact
`ChildOf`/`ChildIn` binding, and live initial child token per contract slot; it
publishes the entire set or none and leaves the parent token unconsumed on
failure. Each child's first StepCauseFrontier
contains exactly one `ActivationStart(child ActivationId)`; that cause's typed
edge projection includes the distinct union of the exact parent split Step,
direct same-Run handoff provenance roots when applicable, and ordinary same-Run
Activation occurrence ancestry. Independently, the first child Branch Step consumes
its exact split-produced `BranchConfigurationToken`, contributing the same
parent-to-child endpoints as a configuration-succession edge. The two typed
edges remain separately inspectable, and neither inserts `PriorStep`. One physical plan completes
the left child first, a second completes the right child first, and a parallel
plan releases both behind a barrier. Each terminal Branch Step consumes its
current `BranchConfigurationToken` into exactly one `Returned` or obligation-
complete `Closed` settlement. In all three runs the Join StepCauseFrontier
contains exactly two causes: `PriorStep(exact RunId, left ActivationId,
left-terminal-step)` and `PriorStep(exact RunId, right ActivationId,
right-terminal-step)`. Neither child's later StepCauseFrontier names the other
child, and trace serialization may differ. Each settlement sequence uses
canonical BranchSlot order. A typed
`ScheduleIsomorphism π` must map every fresh run-local identity so
`encode(π(runA)) = encode(runB)`. Joined payload/Value/Result, occurrence-
support content, candidate delta, Admission decision, and continuation
disposition are literally equal only through their schedule-independent
projections; fresh PriorSteps, terminal-Step-bearing settlements, and other
identity-bearing bytes are isomorphic rather than equal. Join consumes exactly
one settlement per BranchSlot and restores one owner. Overlapping partition or
write Lease; a missing, extra, duplicate, wrong-slot, already-used,
wrong-contract, or cross-
SplitInstance settlement; a Join frontier/settlement mismatch; cancellation
whose `Closed` settlement leaves an exact AllocationRoot, Borrow, Lease,
Continuation, effect, or close obligation neither discharged nor transferred
exactly as declared; and double Join all reject without
publishing a configuration. An obligation-complete `Closed` cancellation
settlement is valid and remains an exact Join input.

Cancellation, yield, and deadline arbitration is also Clause data. The fixture
declares an exact causal decision table and a logical deadline/budget boundary,
then exercises yield-before-cancel, cancel-before-yield, and causally concurrent
cancel/deadline cases under reversed physical schedules. Each outcome, emitted
observation set, terminal/nonterminal condition, continuation disposition, and
typed StepCauseFrontier must match that table. A Step that observes or carries
through cancellation must name the exact
`CancellationRequest(CancellationOccurrenceId)` cause; unrelated Steps must not.
Ambient wall-clock timing, executor queue order, and whichever worker reports
first may not break the tie.

The same ongoing Run executes for many multiples of its declared resident
trace window. Configuration, active causal frontier, continuations,
diagnostics, and trace bytes remain within their separate bounds. Exact
externalization/compaction occurs under the selected retention contract; a
later dependency on an evicted cause must rehydrate its checked witness within
budget or reject typed. A compact summary, log position, or GC reachability may
not authorize the Step.

### 8. Relational recoverability and materialization

Pure-mode observations may populate an activation-scoped result relation
without creating a revision or turning every relation row into an execution.
Only a later explicit Admission may place selected observed bindings into a
revision-indexed RelationExtension. Applications, causal edges, continuations,
evidence, bindings, and supports remain exactly queryable in either scope.

Run one unchanged spatial law through a cold scan and an indexed/incremental
plan. They must agree on observations, occurrence-exact supports, candidate
deltas, failures, and declared resources. Repeated premise slots, self-joins,
and equal content from distinct Activations remain distinct. Disconnected
population growth must not increase declared local update work. Oversized
extents and allocation exhaustion take a visible typed fallback or exhaustion
path without partial publication.

Semantic admission first supplies an `AdmittedStateDelta` with exact Program,
session, predecessor/result StateRevision, and producing Activation/Step
identity. A separate physical update envelope pins that delta plus the exact
semantic graph/contract, physical plan, and budget. The materializer may
validate, project, and apply the envelope to its view, but may not allocate or
admit State history; plan identity never enters `StateRevisionId`.

Locality is accepted only with total boundary accounting. From entry into the
physical update API through its receipt, each plan reports exact integer counts
for input/contract validation, graph and support reads, index-bucket probes,
premise occurrences visited, candidate bindings formed, support entries read
and written, whole-state clones, whole-view rebuilds, support-set clones,
disconnected rows visited, allocation count, allocated bytes, and peak live
bytes. It also reports the selected fallback, typed exhaustion or failure, and
whether a new physical view became visible. Work performed in a helper,
preflight, copy-on-write layer, receipt builder, or deferred rebuild belongs to
the operation that caused it and cannot be excluded as setup.

The indexed-locality fixture runs the same admitted local delta over a base
population and over a second population with a declared multiple of completely
disconnected rows. For the larger population, whole-state clones, whole-view
rebuilds, support-set clones, and disconnected-row visits must remain exactly
zero; every other declared update-work count and update allocation bound must
remain identical. Initial construction of the larger index is reported in a
separate build receipt and may scale. The cold-scan receipt must expose its
corresponding whole-input work rather than being held to the indexed bound.

The allocation gate supplies an exact byte budget, an oversized spatial extent,
and a forced allocator refusal. An extent outside the indexed plan's declared
domain selects a visible typed cold-scan fallback before an unbounded grid or
bucket allocation; if the fallback also exceeds its budget, it returns typed
exhaustion. A forced allocation failure at every allocation boundary must leave
the previously visible physical view and support relation byte-for-byte
unchanged. No fallback or failure may publish a prefix, lose support
multiplicity, change semantic identity, or silently retry through an
unaccounted plan.

### 9. State transition and long-lived world pinning

One valid transition Activation satisfying its selected Mode's exact declared
prerequisites stages a candidate State delta against an exact StateRevision.
Candidate construction leaves the base unchanged.
Admission alone creates the successor StateRevision. Every later world-sensitive
Step names the exact revision it observed. A live Activation never silently
sees a Program or world change; migration, observation advance, or handoff is
explicit and evidence-bearing.

### 10. Honest external effect

Exercise two Mode-selected profiles. The strict governed-per-intent case forms a
causal graph with distinct governed-intent and Admission occurrence, issued
EffectAuthorization occurrence, independent exact CapabilityEvidence, effect
Activation, attempt, optional receipt, zero or more observations, governed
Judgment, and later Admission. The first three inputs occupy distinct named Mode
slots; occurrence-producing inputs alone project to the cause frontier under
those slot identities.

The cheap preauthorized case runs several bounded attempts under each of a
session, Lease, batch, and Activation-local scope where applicable. It binds the
exact intent occurrence, one previously issued EffectAuthorization occurrence,
and independent CapabilityEvidence in three distinct slots; no attempt
manufactures a StateRevision, Admission, or new AuthorizationOccurrence. A
statically pinned issued authorization or capability may erase from the checked
hot ABI but remains an exact semantic slot and cold explanation. Constitutive
execution authority cannot replace issued effect authorization. Intent, issued
authority, capability, attempt, optional receipt, Observation, Judgment, and any
later Admission remain distinct.

Both profiles include success, failure before receipt, and timeout without
receipt. Replaying trace data performs zero attempts. Failed later Admission
acknowledges the act and never claims rollback. Independent negatives omit,
stale, transplant, or place in the wrong slot every prerequisite required by
the selected profile. Governed-only negatives include unadmitted intent; both
profiles reject constitutive-instead-of-issued EffectAuthorization. Every
rejection leaves the affected EffectAttemptOccurrence and
all authoritative boundaries unchanged; pre-Activation failures also allocate
no ActivationId.

### 11. Host-freeze evolution

Freeze the Lean checker/model, Rust semantic boundary, toolchains, binaries,
and host-mechanics manifest. Then perform one predecessor-authorized
`Compiler0 -> Compiler1` evolution that changes:

- one binding form;
- one effect form;
- one typed macro;
- one diagnostic behavior.

The host-freeze experiment requires the ratified process outer envelope to be
fixed in Compiler0 before this evolution. Compiler1 populates that unchanged
envelope; changing its shape would be a fifth host-freeze variable and does not
pass this falsifier.

The user-defined algebraic data and exhaustive-match case must also pass under
the same frozen hosts, including exact missing-case and unreachable-case
rejections.

The change must occur through Clause data alone with no construct-specific
Lean/Rust semantic constructor, validator, callback, dispatch entry, formatter,
refactor, analysis, dependency rule, or target semantic branch.

### 12. Bounded Wasm and passive-render boundary

The Wasm adapter receives bounded canonical bytes and returns bounded canonical
bytes plus replaceable physical handles. The ratified process companion fixes
the exact request bytes, input and output limits, expected typed status, and
handle table before/after state for each of these cases:

- one valid pure activation round trip;
- truncated and noncanonical package bytes;
- a declared input length and an actual input of exactly `limit + 1`;
- an otherwise valid request carrying a stale handle generation, Program pin,
  StateRevision pin, or RuntimeSession pin; and
- a valid pure result whose canonical output is exactly `limit + 1`.

Malformed and oversized input rejects before an Application is activated or a
handle is allocated. A stale request rejects before carry-through. Oversized
output publishes no prefix and creates no authoritative revision. Every case
has bounded decoder/allocation accounting and leaves the previous handle table
unchanged on rejection. A Wasm handle is never accepted in any semantic-ID
field, and a static or native sample may not be labelled Wasm without the exact
Wasm artifact and adapter path that produced it.

The passive renderer consumes an immutable render frame and returns only a
render observation. Its exact vectors include:

- a valid admission-free frame pinned to exact RunId, ActivationId, producing
  StepId, and ObservationId, with no StateRevision and no fabricated Admission;
- a valid frame from the same process identities that additionally names an
  unchanged observed `Wbase`;
- a valid frame projected from one admitted StateRevision;
- the same frozen frame presented through two independently allocated host
  objects, producing the same projection without acquiring shared mutable
  aliases;
- a causally stale Step/Observation frame after its declared successor is
  displayed, plus a stale admitted predecessor after its successor is
  displayed;
- two causally unordered frames, which may not acquire an order merely from
  callback arrival and must follow the declared projection merge policy;
- missing fields, non-finite numeric data, and an object count of exactly
  `limit + 1`;
- canvas focus loss followed by keyboard input; and
- disposal followed by another input event and render request.

Every rejected frame leaves the scene projection and caller-owned frame bytes
unchanged. Freshness is determined by declared process causality and, only when
present, the admitted revision relation. It is never inferred from host object
identity, callback order, or a compulsory StateRevision. The valid case may
mutate only renderer-owned physical scene state;
it cannot change Clause state, integrate movement, infer collision or
groundedness, or perform Admission. The input case emits an immutable intent
observation only while the canvas owns focus. Disposal removes listeners and
resources exactly once through an explicit disposal Activation/Step and
applicable effect receipt before wrapper reclamation; it makes every later call
a typed terminal rejection. Initialization declares exact capacities and
publishes no handle/view on partial failure, rolls back Clause-controlled state,
and records foreign cleanup success, failure, or pending quarantine. After
initialization, valid frame handling
performs no Clause/Wasm/adapter-controlled allocation, `memory.grow`, whole-
frame clone, global scan, observable destructor/finalizer work, or unbounded
teardown. Receipts
record controlled allocation calls/bytes, pool high-water, Wasm pages, adapter
calls, and resource ledger before/after. Ratified foreign calls have declared
allocation/disposal contracts; a whole-browser zero-allocation claim requires
instrumented warm-up and lazy-cache evidence.

### 13. Frozen ordinary-source ergonomics

The process machinery must remain absent from ordinary authored source unless
the source is actually specifying a process boundary. The executable source
acceptance corpus must copy these UTF-8/LF specimens byte-for-byte and preserve
them through canonical parse/print/parse.

Pure singleton field constraint:

```clause
answer: 20 + 22
```

Relational request:

```clause
select all ?destination in egress
  where
    ICU-A has a usable egress path to ?destination
```

State-change constitution, where process structure is relevant:

```clause
on collect ?actor
  when
    ?coin state active
    ?coin owner ?actor
  withdraw
    ?coin state active
  include
    ?coin state collected
```

General-purpose function, static-use, loop, local-mutation, and lifetime
boundaries:

```clause
function map
  parameters
    Item: Type
    Result: Type
  constraints
    mapping: Maps Item to Result
  given
    items: Sequence of Item
  yields
    mapped: Sequence of Result
  run
    region output
      mutable builder: empty Sequence of Result
      borrow read items as source
        lease write builder as sink
          for item in source
            append mapping(item) to sink
      return freeze move builder

upper-names: map(player-names) with
  Item = Text
  Result = Text
  mapping = uppercase
```

This fourth block is the exact syntax-authority specimen. Its dependency
context supplies the accepted meanings of `Type`, `Maps`, `Sequence`, `empty`,
`append`, `freeze`, and `uppercase`; none is a host intrinsic selected by
spelling. The acceptance corpus must freeze the UTF-8/LF bytes and the exact
pre- and post-edit semantic/cache sets.

The first two blocks contain no authored ApplicationId, ActivationId, StepId,
RunId, ContinuationId, revision pin, authority token, scheduler, budget, trace,
or materialization plan. Their checked crosswalk still exposes every required
semantic identity and pin. The third exposes only the already canonical
process-relevant `on`/`when`/`withdraw`/`include` vocabulary; actual activation,
event occurrence, Step identity, and constitutional Admission remain governed
boundaries rather than user bookkeeping. The fourth exposes ownership/lifetime
words only where they change alias, escape, or reclamation meaning. Until
continuation and race surface
syntax is separately ratified, those fixtures are Clause semantic data and
exact observations, not invented source spelling.

## Exact acceptance

The cross-phase program passes only when all of these are executable and exact:

- ApplicationForm/Application/Activation separation;
- one exact Application activated twice retains one ApplicationId and receives
  distinct ActivationIds, while independently nominalized equal-shaped
  Applications receive distinct ApplicationIds;
- one Activation across multiple configurations and StepIds;
- every Step has exactly one Serial, Split, Branch, or Join configuration
  transition; Split/Branch/Join preserve exact structured custody, coverage,
  BranchSlot multiplicity, settlement, and obligation closure without a
  residual parent token or a new global identity domain, while SplitFormation
  publishes its Step/instance/children/bindings/tokens atomically or none and
  repeated equal BranchKeys remain distinct by contiguous canonical ordinal;
- a StepId is a fresh nominal identity whose StepRecord separately carries the
  exact owner, finite StepCauseFrontier, StepConfigurationTransition, and
  outputs;
- Ready means constituted Activation, zero owned Steps, and live unconsumed
  initial custody; its normal first frontier is the exact ActivationStart
  singleton and its sole cancellation exception is the exact ActivationStart +
  matching CancellationRequest pair with a matching Cancel outcome; wrong
  target, pins, Mode, occurrence, outcome, extra cause, or second cancellation
  rejects before StepId allocation;
- Run order is exactly the transitive closure of separately inspectable typed
  StepCauseFrontier edges and typed configuration-succession edges, including
  `s1 <run s2` when `s2` consumes `ConfigurationAfter(s1)` without a redundant
  `PriorStep(s1)`, and parent-Step-before-child-first-Step order projected by
  `ActivationStart` from an exact `ChildOf` or `HandoffFrom` origin without an
  inserted `PriorStep` or a required configuration transfer; every nonfirst
  Step has nonempty IncomingRunEdges, while its frontier may be empty only when
  its transition contributes a configuration predecessor;
- valid StaticActivationBasis, exact InitialContext, complete named/
  RoleId-indexed DynamicPrerequisiteBindings, and a separate occurrence-only
  causal frontier for every Activation, with no binding, AuthorizationEvidence,
  or capability token when the selected Mode's entire schema is empty;
- exact sandbox/candidate running, read-only admitted-world use, persistent
  nonauthoritative output/Continuation, and inert effect simulation with no
  fabricated revision, real attempt, or constitutive authority, plus exact
  authoritative/effect running pinned to an admitted constitution;
- pure isolation with no revision;
- observationally pure affine local mutation across anonymous reductions,
  exact Step cuts, bounded restoration, non-escape, ownership-consuming
  split/join/suspension, and checked in-place lowering without Admission or
  mandatory trace retention;
- rank-1 parameter and named constraint telescopes, total static
  normalization, terminating complete resolution, exact resolution-scope
  commitments, normalized evidence, distinct InstantiationUseRef/
  InstantiationKey/SpecializationKey/PhysicalReuseKey roles, separate
  compilation, finite self/mutual-recursion SCC keys, unrelated-edit reuse, and
  Renameπ/substitution/solver equivariance;
- rich Text/Bytes/numeric/algebraic/sequence/map values through Clause-owned
  physical IR with native/Wasm parity, declared layout/ABI, and verified
  monomorphization/dictionary/erasure strategies;
- one exact Owned/RegionMember/ForeignManaged allocation root plus typed
  Borrow/Lease edges, explicit close/dispose before bounded nonobservable
  mechanical reclaim, root-wide compatibility and acknowledged quiescence, rejection of
  cross-root strong cycles, one explicitly bounded managed-island fixture,
  honest foreign disposal/quarantine, and no managed island, tracing GC, ARC,
  or finalizer fallback in the game hot profile;
- the long-lived agent workbench executing `parse`, `check`, `explain`,
  `query`, `diff`, `propose`, `admit`, `run`, and `hotReload` as accepted Clause
  package behavior rather than host semantics, including the ratified combined
  generic/loop/builder/move/borrow/region/Lease source edit;
- an intentionally ongoing Run with no fake terminal result;
- suspension, persistence, handoff, cancellation, and resumption with exact
  causal identity and pins, including exact emitting-Step identity in the one
  ContinuationTakeup cause; HandoffFrom binds that Continuation's exact emitter,
  destination basis/pins, and well-founded HandoffOccurrence provenance, and
  ActivationStart projects its distinct same-Run ancestry union; executor
  destruction, continuation rematerialization, and a fresh post-suspension
  observation;
- wrong-pin and equal-content continuation transplant rejection, plus exact
  enforcement of the fixture's Clause-declared linear reuse policy;
- reversed and parallel child schedules related by a typed
  ScheduleIsomorphism over every fresh run-local identity, exact equality only
  of schedule-independent payload/result/delta/disposition projections,
  BranchSlot-canonical settlement structure, and Clause-declared cancellation/
  yield/deadline arbitration;
- bounded long-run configuration/frontier/continuation/trace residency with
  exact rehydration or typed unavailable-history rejection;
- effect-stage honesty under both governed-per-intent and preauthorized local/
  session/Lease/batch profiles, including three distinct intent/issued-
  authorization/capability slots, governed-only Admission, no per-attempt
  governance tax in the bounded preauthorized case, and honest receipt absence;
- exact constitution and applicable Program/world/session pinning with no
  silent migration or fabricated revision;
- identity retention across source-only movement, serialization, process
  restart, machine relocation, and physical rematerialization when the exact
  ProgramSnapshot, ApplicationForm, and nominal identity remain unchanged;
- new ApplicationId after a semantic or snapshot-local declaration revision,
  with any intended continuity represented separately by ReferentId evidence;
- independent concurrent Steps not ordered by storage serialization;
- no implicit assertion from formation, evaluation, or observation;
- supported-true, supported-false, and absent proposition observations without
  an implicit AssertionOccurrence, governed Judgment, or revision;
- relational recovery of admissible bindings, accepted observations,
  dependencies, causal edges, and occurrence-exact supports;
- scan/indexed parity plus total update-boundary clone, rebuild, support,
  allocation, fallback, and disconnected-population accounting;
- malformed, ungrounded, unauthorized, cyclic-without-anchor, wrong-revision,
  ambiguous-mode, and over-budget rejection before partial authority;
- bounded Wasm and passive-render acceptance, controlled-allocation receipts,
  and exact malformed, oversized, stale, focus-loss, and post-disposal
  negatives;
- admission-free frame freshness by exact Run/Activation/Step/Observation plus
  optional unchanged Wbase, with an admitted-revision frame as one explicit
  case rather than the universal frame contract;
- source-to-Term-to-Application-to-Activation-to-artifact explanation;
- deterministic Reading selection before child-domain checking, lossless
  source occurrences, canonical parse/print/parse meaning, exact focus,
  binding and origin preservation, local recovery, and semantic round trips;
- exact local-name resolution to structured Designations through
  checked namespace/import/export relations, with `/` rejected in every
  `Designation.spelling` whether its authored form is unquoted or backtick-
  quoted and with forged structured Designations rejected before semantic
  identity, schema, role, or operator closure; Text and opaque Atom/transport
  payloads retain `/` without becoming Designations; and
- ordinary source at least as readable as the accepted surface, with process
  and ownership machinery exposed only where semantically relevant, proven by
  all four canonical process source specimens rather than a prose
  readability claim.

## Required negative evidence

The spike actively rejects or bounds:

- accidental identity on every Triple or operator meaning in Triple slot 2;
- incomplete or open formation candidates receiving semantic
  ApplicationShapeId;
- accidental deduplication of equal Applications, Activations, observations,
  premise occurrences, effect occurrences, or supports;
- host handles, pointers, row IDs, source spans, or Wasm handles leaking as
  identity;
- Application/Activation/Step/Run/trace collapse;
- propositions treated as assertions or relation rows treated as process
  occurrences;
- absent proposition support collapsed into false, or truth observation
  collapsed into a governed Judgment;
- fabricated receipts, replayed effects, or false rollback claims;
- quoted, pattern, hypothetical, or speculative forms executed as authority;
- NaN, signed-zero, Unicode-normalization, or numeric-width disagreement;
- total Modes with unproved termination and productive Modes without progress;
- hostile, recursive, nondeterministic, or phase-escaping macros;
- non-exhaustive or unreachable algebraic-data match cases accepted without
  their exact rejection obligations;
- hidden semantic cases in host enums, callbacks, dispatch tables, serializers,
  formatters, materializers, or generated runtimes;
- mandatory dynamic Authorization for a Mode whose entire prerequisite schema
  is empty; missing/extra/wrong-slot/multiplicity-collapsed bindings; non-
  occurrence evidence fabricated into a causal edge; or erased issued/effect/
  Admission evidence where it may vary;
- governed-only Admission imposed on a preauthorized effect Mode, per-attempt
  Admission/issuance manufactured inside a session/Lease/batch/local scope, or
  any intent/authority/capability/attempt/receipt stage collapsed;
- checked-candidate running that fabricates a ProgramRevision/RuntimeSession/
  StateRevision or real EffectAttemptOccurrence, candidate content used as
  constitutive authority, or an admitted-constitution binding whose revision
  selects another snapshot;
- ambient/global constraint lookup, source-order instance choice, open
  instantiation, incomplete search reported as unsatisfied, uncommitted
  negative dependency, cyclic derived-key preimage, higher-rank smuggling,
  InstantiationKey used as a physical cache key, or host-owned specialization;
- mutable local escape, shared alias, scratch retention, failure-visible
  partial update, rollback that fails to restore the exact whole-token,
  branch-token, or canonical-settlement-sequence custody input, overlapping or
  incomplete partition, residual whole-parent token, cross-split/equal-content
  branch transplant, missing/extra/duplicate/wrong-slot/already-used/
  wrong-contract settlement, double join, wrong-split or repeated branch
  takeup, Join frontier/settlement mismatch, cancellation leaving any exact
  AllocationRoot/Borrow/Lease/Continuation/effect/close obligation neither
  discharged nor transferred exactly as declared, host
  arrival-order dispatch, or
  Admission/StateRevision manufactured for ordinary local mutation;
- semantic identity used as a residency root, unknown lifetime silently leaked,
  Lease confused with reclamation ownership, observable destructor/finalizer
  invoked by deallocation, cross-root Owned↔Owned ownership cycle, reclaim
  before every holder acknowledges quiescence, a managed island crossing its
  declared boundary/budget, or hidden managed-island/tracing/ARC/finalizer
  fallback in the native/Wasm game profile;
- source round trips that lose binding, occurrence, Application, or Referent
  continuity;
- an authored unquoted designation `x/y` surviving the reader or reaching
  Designation resolution;
- an authored backtick-quoted designation `` `x/y` `` using quotation to
  survive the reader or reach Designation resolution;
- a forged structured `Designation` whose `spelling` contains `/` reaching
  Referent identity use, RelationSchema or Role closure, OperatorRef selection,
  equality, behavior, or a multi-segment kind/role/path convention; Text values
  and opaque Atom/transport payloads containing `/` are the required non-
  Designation controls;
- silent Program/world rebinding on continuation resume;
- continuation transplant or Clause-declared single-use continuation reuse;
- inferred causality from child completion or trace serialization order;
- cancellation/yield/deadline arbitration hidden in wall time, queue order, or
  host race behavior;
- unbounded resident causal/trace history, compact summary used as authority,
  or evicted cause accepted without exact witness rehydration;
- whole-graph invalidation, whole-state clone, whole-view rebuild, support-set
  clone, or disconnected-row traversal hidden behind a claimed local edit;
- oversized materialization allocation, invisible fallback, partial physical
  publication, or work omitted from the operation's receipt;
- malformed, oversized, or stale Wasm/render input causing partial output,
  handle allocation, scene mutation, leaked listeners, or semantic authority;
- Clause/Wasm/adapter-controlled per-frame allocation, `memory.grow`, global
  scan, whole-carrier clone, observable destructor/finalizer work, or unbounded teardown after
  bounded frame initialization; or a browser-wide zero-allocation claim
  without instrumented foreign evidence;
- process IDs, scheduler policy, or materializer plans forced into routine
  authored source where no such distinction is relevant;
- semantic-IR general-purpose evidence counted without the ratified combined
  source fixture and exact workbench roundtrip/edit;
- every machine/KExpr reduction being recorded as a semantic Step; and
- generic Triple execution presented as a credible production hot path.

## Pass and falsification

The mechanism passes only when Phase A meets the trust profile, one exact
carrier passes the complete Phase B and Phase C program, Lean and Rust agree on
every declared observable and nonfunctional contract, every negative fixture
fails for the intended reason, the unchanged v0 corpus crosswalks honestly, and
host-freeze evolution adds no private semantic case.

Reject or narrow the mechanism if Application, Activation, and Step have no
distinct consumers; the neutral three-slot carrier requires arbitrary positions
or untyped tags for roles, continuation, binding, effects, or authority;
essential semantics survives only in host functions, schedulers, mutable
objects, or undocumented lowering; relational reasoning becomes materially
worse; ordinary local mutation requires Admission or cannot lower without a
mandatory collector; static reuse or physical specialization requires hidden
host semantics; every ephemeral reduction must become durable graph content;
the trusted kernel grows a second sovereign language; or an ongoing Run cannot
be distinguished from failed or ungrounded evaluation.

Failure preserves Clause's process-first relational mission and records the
exact forcing counterexample. It does not authorize a static fact language, a
static application language, hidden host semantics, or silent scope reduction.
