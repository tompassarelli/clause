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

The spike starts from the accepted calculus, consumes exact frozen oracles, and
must justify every claimed behavior directly.

## Frozen and versioned oracles

`clause:test-vectors/execution/**` remains byte-for-byte Clause execution corpus
v0. Its three canonical source programs and manifest observations are frozen:

- recursive pure dependency closure;
- admitted State change plus separate effect evidence; and
- predecessor-bound Program evolution.

The six v0 outcome tags remain exactly `returned`, `choices`, `yielded`,
`suspended`, `failed`, and `exhausted`. They are compatibility evidence, not
the complete process ontology.

A separate versioned process-v1 companion must crosswalk those exact bytes into
ApplicationForms, ApplicationIds, ActivationIds, StepIds, RunIds,
ContinuationIds where a boundary is crossed, ObservationIds, exact program and
world pins, and candidate/admission evidence. It must not rewrite or infer new
identity from fixture-local v0 names. Source movement, duplicate equal
occurrences, n-ary role closure, closure capture, pure arithmetic, an ongoing
service, cancellation, budget exhaustion, and effect timeout require new
process-v1 fixtures.

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

Γ ⊢ form(t, OperatorRef, exact named-role bindings, requirements)
  : ApplicationForm<ResultDomain>

Application(ApplicationId, exact ApplicationForm)

activate(ApplicationId, ModeId, InitialContext, ActivationCauseFrontier)
  = ActivationId + RunMembership + InitialConfiguration

Configuration_before
  -- StepId(causes = StepCauseFrontier) ; observations ; delta ; continuation -->
Configuration_after

Run(RunId, root = ActivationId, causal closure)

admit(BaseRevision, delta, evidence, authority, obligations)
  = SuccessorRevision | Rejection
```

The core must represent:

- contextually opaque Atoms and explicit refinements across universes;
- structurally neutral Triple slots and structural Term equality indexed by
  universe and semantics epoch;
- FormationJudgment distinct from governed Judgment;
- closed ApplicationForms with exact OperatorRef, RelationSchema, named-role
  closure, mode eligibility, and context requirements;
- snapshot-local RelationSchema, Role, Operator, and Mode declarations whose
  exact external references include ProgramSnapshotId and never silently carry
  across a changed snapshot;
- `ApplicationShapeId` only for closed forms, committing to ClauseSemanticsId,
  exact RelationSchemaId, exact OperatorRef, the exact eligible ModeId set,
  named-role bindings, context requirements, and the full resolved semantic-
  dependency/declaration closure, including proof that it is empty where
  applicable;
- mandatory nominal `ApplicationId` for every Application, with raw, quoted,
  open, and merely structural forms remaining non-nominal ApplicationForms or
  Terms rather than anonymous Applications;
- configured binders, transfers, requests, and tasks distinguished from actual
  event/effect occurrences, which carry typed OccurrenceId plus exact
  provenance: producing Activation/Step for internal production or external-
  boundary provenance for an ingress trigger;
- fresh `ActivationId` for every engagement and one stable Activation across
  any number of configurations and StepIds;
- nonempty finite typed StepCauseFrontiers built from exact ActivationStart,
  PriorStep, ContinuationTakeup, and CancellationRequest causes so concurrency
  remains a partial order rather than log order;
- RunId as a causal envelope distinct from ActivationId, including child
  activation, handoff, and cancellation scope;
- typed continuation as semantic remainder, with exact identified pins when it
  crosses suspension, handoff, persistence, or executor boundaries;
- ObservationId distinct from observed Value and Result;
- immutable typed candidate deltas separate from continuations;
- activation-scoped result relations, separately admitted revision-indexed
  relation extensions, and occurrence-exact support;
- total, productive, bounded, partial, nondeterministic, streaming, reactive,
  and effectful Mode contracts;
- distinct Reading, derivation authorization, ExecutionAuthorization,
  admission authority, and effect/resource capability;
- non-operator RelationSchemas able to form checked bindings, proposition and
  assertion content, rows, and patterns without forming ApplicationForms;
- source occurrences, scope, binding, quotation, hygiene, phase, and origin;
- immutable ProgramSnapshot, ProgramRevision, RuntimeSession, and StateRevision
  boundaries with exact pinning and no silent migration; and
- canonical package bytes with cycle-aware, terminating, fail-closed reload.

Raw Triples receive no mandatory nominal identity. A relation or mode may exist
without executable authorization. Private interning handles, Wasm handles,
pointers, table indexes, paths, spans, or log positions cannot escape as
semantic identity.

Identity creation itself is governed. The process-v1 carrier must record the
exact authority and predecessor evidence that may allocate or retain each
nominal identity: Program admission or an admitted generation rule for an
Application, successful authorized activation for an Activation and Run root,
actual carry-through for a Step, boundary-crossing remainder for a
Continuation, actual distinction for an Observation or Occurrence, and
constitutional Admission for a revision. Content hashing, caller-supplied
bytes, possession of a serialized object, or a physical allocator cannot mint
one of those identities.

For every identity domain, the companion includes exact wrong-kind,
wrong-authority, self-authorizing, equal-content transplant, and already-used
occurrence negatives. Replaying a record with its existing identity may be an
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

## Phase B — Cross-phase process proving program

All cases below must inhabit one semantic carrier and process protocol. Separate
toy runtimes that merely share names do not pass. This is a cross-phase
acceptance program: the semantic process cases can run before Compiler0, Terms,
or materialization integration, while the host-freeze and physical parity cases
complete only in the later integration phase.

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

### 5. Recursive, streaming, and ongoing running

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
  `ContinuationTakeup(exact ContinuationId, exact ResumptionOccurrenceId)` and
  `PriorStep(original RunId, original ActivationId, suspend-step)`;
- consumes the newly identified ingress occurrence under its declared mode;
- emits a fresh ObservationId rather than replaying a cached observation;
- retains the exact Program, session, world, policy, semantics, budget, and
  cancellation-scope pins; and
- creates no ProgramRevision or StateRevision unless a separately staged
  candidate is admitted.

The same fixture selects a Clause-declared linear resume policy. The first
authorized resume consumes one exact continuation-use authority. A repeated or
concurrent second use rejects as `continuation-already-consumed` before a Step,
observation, effect attempt, or delta exists. This is a mode property, not a
universal assertion that every continuation is linear.

Every pinned field has an exact one-field stale negative. Transplanting the
unchanged continuation bytes into an equal-shaped but independently nominal
Application, another Activation or Run, another RuntimeSession, or another
continuation-use authority also rejects before carry-through. The rejection
must name the mismatched identity class without treating equal content, a host
handle, or possession of bytes as authority.

The concurrency gate varies physical execution while keeping semantic
causality fixed. A parent Activation creates two independent child
Activations through exact `ChildOf` ActivationOrigins. Each child's first
StepCauseFrontier contains exactly one
`ActivationStart(child ActivationId)`. One physical plan completes the left
child first, a second completes the right child first, and a parallel plan
releases both behind a barrier. In all three runs the join StepCauseFrontier
contains exactly two causes: `PriorStep(exact RunId, left ActivationId,
left-terminal-step)` and `PriorStep(exact RunId, right ActivationId,
right-terminal-step)`. Neither child's later StepCauseFrontier names the other
child, trace serialization may differ, and the joined observation and support
multiset are identical.

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

### 6. Relational recoverability and materialization

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

### 7. State transition and long-lived world pinning

One authorized transition Activation stages a candidate State delta against an
exact StateRevision. Candidate construction leaves the base unchanged.
Admission alone creates the successor StateRevision. Every later world-sensitive
Step names the exact revision it observed. A live Activation never silently
sees a Program or world change; migration, observation advance, or handoff is
explicit and evidence-bearing.

### 8. Honest external effect

Exercise a causal graph with distinct intent, authorization, attempt, optional
receipt, zero or more observations, governed Judgment, and later admission.
Include success, failure before receipt, and timeout without receipt. Replaying
trace data performs zero attempts. Failed later admission acknowledges the act
and never claims rollback.

### 9. Host-freeze evolution

Freeze the Lean checker/model, Rust semantic boundary, toolchains, binaries,
and host-mechanics manifest. Then perform one predecessor-authorized
`Compiler0 -> Compiler1` evolution that changes:

- one binding form;
- one effect form;
- one typed macro;
- one diagnostic behavior.

The accepted process-v1 outer envelope is already fixed in Compiler0 before
this evolution. Compiler1 populates that unchanged envelope; changing its shape
would be a fifth host-freeze variable and does not pass this falsifier.

The user-defined algebraic data and exhaustive-match case must also pass under
the same frozen hosts, including exact missing-case and unreachable-case
rejections.

The change must occur through Clause data alone with no construct-specific
Lean/Rust semantic constructor, validator, callback, dispatch entry, formatter,
refactor, analysis, dependency rule, or target semantic branch.

### 10. Bounded Wasm and passive-render boundary

The Wasm adapter receives bounded canonical bytes and returns bounded canonical
bytes plus replaceable physical handles. The process-v1 companion fixes the
exact request bytes, input and output limits, expected typed status, and handle
table before/after state for each of these cases:

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

- a valid frame pinned to one admitted StateRevision;
- the same frozen frame presented through two independently allocated host
  objects, producing the same projection without acquiring shared mutable
  aliases;
- a stale predecessor frame after its declared successor is already displayed;
- missing fields, non-finite numeric data, and an object count of exactly
  `limit + 1`;
- canvas focus loss followed by keyboard input; and
- disposal followed by another input event and render request.

Every rejected frame leaves the scene projection and caller-owned frame bytes
unchanged. The valid case may mutate only renderer-owned physical scene state;
it cannot change Clause state, integrate movement, infer collision or
groundedness, or perform Admission. The input case emits an immutable intent
observation only while the canvas owns focus. Disposal removes listeners and
resources exactly once and makes every later call a typed terminal rejection.

### 11. Frozen ordinary-source ergonomics

The process machinery must remain absent from ordinary authored source unless
the source is actually specifying a process boundary. The process-v1 companion
copies these UTF-8/LF specimens byte-for-byte and preserves them through
canonical parse/print/parse.

Pure definition:

```clause
answer := 20 + 22
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
  admit
    ?coin state collected
```

The first two blocks contain no authored ApplicationId, ActivationId, StepId,
RunId, ContinuationId, revision pin, authority token, scheduler, budget, trace,
or materialization plan. Their checked crosswalk still exposes every required
semantic identity and pin. The third exposes only the already canonical
process-relevant `on`/`when`/`withdraw`/`admit` vocabulary; actual activation,
event occurrence, Step identity, and constitutional Admission remain governed
boundaries rather than user bookkeeping. Until continuation and race surface
syntax is separately ratified, those fixtures are Clause semantic data and
exact observations, not invented source spelling.

## Exact acceptance

The cross-phase program passes only when all of these are executable and exact:

- ApplicationForm/Application/Activation separation;
- one exact Application activated twice retains one ApplicationId and receives
  distinct ActivationIds, while independently nominalized equal-shaped
  Applications receive distinct ApplicationIds;
- one Activation across multiple configurations and StepIds;
- pure isolation with no revision;
- an intentionally ongoing Run with no fake terminal result;
- suspension, persistence, handoff, cancellation, and resumption with exact
  causal identity and pins, including executor destruction, continuation
  rematerialization, and a fresh post-suspension observation;
- wrong-pin and equal-content continuation transplant rejection, plus exact
  enforcement of the fixture's Clause-declared linear reuse policy;
- reversed and parallel child schedules with the same exact two-PriorStep join
  StepCauseFrontier, and Clause-declared cancellation/yield/deadline
  arbitration;
- effect-stage honesty, including receipt absence;
- exact Program and world pinning with no silent migration;
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
- bounded Wasm and passive-render acceptance and exact malformed, oversized,
  stale, focus-loss, and post-disposal negatives;
- source-to-Term-to-Application-to-Activation-to-artifact explanation;
- deterministic Reading selection before child-domain checking, lossless
  source occurrences, canonical parse/print/parse meaning, exact focus,
  binding and origin preservation, local recovery, and semantic round trips;
  and
- ordinary source at least as readable as the accepted surface, with process
  machinery exposed only where semantically relevant, proven by the frozen
  source specimens rather than a prose readability claim.

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
- source round trips that lose binding, occurrence, Application, or Referent
  continuity;
- silent Program/world rebinding on continuation resume;
- continuation transplant or Clause-declared single-use continuation reuse;
- inferred causality from child completion or trace serialization order;
- cancellation/yield/deadline arbitration hidden in wall time, queue order, or
  host race behavior;
- whole-graph invalidation, whole-state clone, whole-view rebuild, support-set
  clone, or disconnected-row traversal hidden behind a claimed local edit;
- oversized materialization allocation, invisible fallback, partial physical
  publication, or work omitted from the operation's receipt;
- malformed, oversized, or stale Wasm/render input causing partial output,
  handle allocation, scene mutation, leaked listeners, or semantic authority;
- process IDs, scheduler policy, or materializer plans forced into routine
  authored source where no such distinction is relevant;
- every machine/KExpr reduction being recorded as a semantic Step; and
- generic Triple execution presented as a credible production hot path.

## Pass and falsification

The mechanism passes only when Phase A meets the trust profile, one exact
carrier passes the complete Phase B cross-phase program, Lean and Rust agree on
every declared observable and nonfunctional contract, every negative fixture
fails for the intended reason, the unchanged v0 corpus crosswalks honestly, and
host-freeze evolution adds no private semantic case.

Reject or narrow the mechanism if Application, Activation, and Step have no
distinct consumers; the neutral three-slot carrier requires arbitrary positions
or untyped tags for roles, continuation, binding, effects, or authority;
essential semantics survives only in host functions, schedulers, mutable
objects, or undocumented lowering; relational reasoning becomes materially
worse; every ephemeral reduction must become durable graph content; the trusted
kernel grows a second sovereign language; or an ongoing Run cannot be
distinguished from failed or ungrounded evaluation.

Failure preserves Clause's process-first relational mission and records the
exact forcing counterexample. It does not authorize a static fact language, a
static application language, hidden host semantics, or silent scope reduction.
