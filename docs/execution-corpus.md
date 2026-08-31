# Clause v0 execution corpus and process-carrier boundary

> **Authority:** The [foundation](foundation.md) alone defines Clause meaning,
> the [syntax](syntax.md) alone defines canonical source, and the
> [roadmap](roadmap.md) alone defines implementation status. This document and
> its corpus select concrete cross-host observations without adding semantics
> or syntax.

The v0 execution corpus fixes three nontrivial program payloads and their
observations for the Lean reference model, Rust runtime, CLI, and clean replay:

1. a recursive pure dependency-closure query;
2. an admitted state transition followed by a separately identified effect
   attempt and receipt admission; and
3. two predecessor-bound Program changes, including rejection from a stale
   base.

The executable observation contract is
[`test-vectors/execution/manifest.json`](../test-vectors/execution/manifest.json).
Its three historical source-projection payloads are byte-frozen at
`test-vectors/execution/historical-v0/source-projections/*.clause-v0.txt`; the
verifier preserves their original SHA-256 digests. Their paths, suffixes,
README/checksum content, and manifest classification metadata are not part of
that payload-byte identity and may change to keep the quarantine honest. The
current manifest has `syntax_authority: null`, classifies the files as
`historical-v0-noncanonical-fixture`, records that no canonical source or
spelling authority is included, and names them only through
`historical_source_projection`. Their slash-joined spellings predate the
current [`syntax.md`](syntax.md), which must reject those payload bytes as
authored Designations. Effect capability and attempt data remain generic Terms
in the manifest because their authored surface syntax is not yet ratified.

## Representation boundary

Names in the manifest are fixture-local opaque transport strings; `/` is one
uninterpreted payload byte and never namespace or designation structure. They
are not Atom kinds, host enum cases, package digests, or constitutional
identities. A consumer
must lower every referenced value, claim, mode, capability, delta, trace, and
obligation through the same generic Clause Term and provisional v0 candidate-
judgment representation. That frozen carrier is not the current governed
Judgment or process kernel. Adding a construct-specific Lean or Rust execution
branch to recognize a fixture name fails the corpus.

The fixture lowering is exact and deliberately generic. A symbol is an Atom at
the published v0 structural index with kind byte `e0`, equality-contract byte
`e1`, and the exact UTF-8 bytes of its manifest string as canonical payload. A
binary relational fixture item is the neutral Triple `[subject, relation,
object]`.
Fixture claims use the symbol Terms `fixture/type/proposition` and
`fixture/mode/asserted` for their type and mode. This selects reproducible
corpus data only; it does not admit the fixture Atom contract or make any such
claim semantically true. These frozen v0 manifest strings are not canonical
Clause identifiers or qualified-designation syntax, and no host or runtime may
recover kind, role, identity, or behavior from their slash-separated bytes.

The manifest's `ground_rules` are the finite generic expansion of the pure
program's two authored laws for this exact input graph. Each three-string item
is shorthand for the same neutral subject/relation/object Triple. This lets the
Lean and Rust execution tranches consume one executable oracle before source
elaboration exists. A ratified process-carrier successor must express
equivalent laws in separately accepted canonical source and reproduce the same
expansion; a current parser must not reinterpret the historical v0 files.

The corpus deliberately does not publish `ApplicationId`, `ActivationId`,
`StepId`, `RunId`, `ContinuationId`, `ObservationId`, `ProgramSnapshotId`, or
`ProgramRevisionId`. The foundation describes their intended shape but does
not yet ratify an exact hash algorithm and preimage encoding. The opaque
revision names in the vectors are supplied by the admission context and may
not be derived or presented as public IDs. Likewise, fixture-local v0 `run`
names are not process identity and may not be promoted by spelling.

Every assertion occurrence has a distinct fixture ID. Equal proposition
content does not merge occurrences. Every candidate delta names its exact base,
withdrawals, and admissions. The historical v0 Run envelope can propose a
candidate context, but only explicit constitutional Admission changes an
authoritative boundary.

## Frozen v0 observations

The six v0 outcome tags remain exactly:

```text
returned(value)
choices(finite-results)
yielded(value, continuation)
suspended(continuation)
failed(error)
exhausted(obligations)
```

For each v0 Run envelope, Lean and Rust must agree on the selected mode, outcome,
candidate delta, inert trace data, and admission decision. Finite choices are
compared as the declared finite set; JSON order is transport order only.

The state/effect program has four distinct boundaries:

1. the v0 transition envelope proposes inventory and order-state changes plus
   an effect intent;
2. admission atomically accepts that candidate State successor and intent;
3. a separately identified v0 effect envelope performs the authorized attempt
   and yields receipt data without rewriting Clause state; and
4. a later admission records the receipt claim.

Reasserting the recorded trace must not perform the effect again. A fabricated
receipt and an attempt without the admitted capability both reject.

## Required ratified process companion

A separately versioned companion must preserve every frozen v0 program payload
and execution observation while making the process kernel explicit. The current
process-v2 Rust corpus is a reduced experimental input to this work, not the
ratified companion. The crosswalk must show:

- each checked closed form and its ApplicationShapeId, committing to exact
  ClauseSemanticsId, exact RelationSchemaId, exact OperatorRef, the exact
  eligible ModeId set, named RoleId bindings, context requirements, exact
  InstantiationUseRefs with their InstantiationKeys and SpecializationKeys, and
  the complete resolved semantic-dependency/declaration closure, including proof
  that it is empty where applicable; PhysicalReuseKey is excluded;
- local instantiation records in the ProgramSnapshot preimage, with every
  same-snapshot InstantiationUseRef resolved only after the one snapshot hash,
  cross-snapshot checking/specialization keys derived from independent
  canonical content, and no derived key inserted back into that preimage;
- exact RelationSchemaId, RoleId, OperatorRef, and ModeId references formed from
  ProgramSnapshotId plus typed snapshot-local declaration identities, with no
  silent identity carry across a changed snapshot;
- nominal ApplicationId for every Application, with raw, open, quoted, or
  merely structural forms remaining non-nominal;
- contextual ClauseJudgments over neutral Terms remaining distinct from
  governed Judgments, ApplicationForms, and nominal Applications;
- two activations of one exact Application with distinct ActivationIds and
  RunIds;
- independently nominalized equal-shaped Applications with distinct
  ApplicationIds;
- one Activation producing multiple StepIds across yield, suspension, and
  resumption;
- one exact StaticActivationBasis per Activation, with the selected Mode's
  named/RoleId-indexed multiplicity-aware DynamicPrerequisiteSchema and exact
  bindings represented separately from its occurrence-only cause frontier; the
  entire schema is permitted to be empty;
- one exact CheckedConstitutionBinding per basis: checked candidate package and
  snapshot bytes for nonauthoritative running, including read-only admitted-
  world use, persisted output/Continuation, and inert effect simulation; or an
  admitted ProgramRevision selecting that snapshot when joining an authoritative
  RuntimeSession, proposing authoritative world change, relying on constitutive
  Program authority, or performing a real effect; read-only use of an admitted
  world alone remains valid in the checked-candidate form;
- affine Activation-local slots, bounded Step-local scratch, anonymous internal
  reductions, mandatory Step cuts, and exact escape/alias/lifetime outcomes;
- every Step carrying exactly one Serial, Split, Branch, or Join configuration
  transition, with one Mode-owned multiplicity-aware split/join contract,
  pairwise-disjoint exact coverage, structurally anchored branch custody,
  typed `BranchSlot = (BranchKey, repeated-spec ordinal)`, atomic co-formation
  of the split Step/instance/children/bindings/initial tokens, obligation-
  complete BranchSlot-bearing settlements, and canonical BranchSlot-ordered
  Join;
- finite typed StepCauseFrontiers rather than inferred log causality, with a
  normal first-Step ActivationStart singleton, the sole exact ready-cancellation
  pair, and exact PriorStep, ContinuationTakeup, and CancellationRequest causes
  where applicable; a nonfirst frontier may be empty only with a configuration
  predecessor, and every nonfirst Step has nonempty IncomingRunEdges;
- Run order as the transitive closure of those typed frontier edges plus
  separately typed configuration-succession edges, without inserting an
  implicit PriorStep or treating storage/arrival order as semantics;
- one exact ActivationStartRecord from which Application, Mode, static basis,
  semantics, constitution, initial world/session/policy, dynamic prerequisites,
  and original cause frontier derive, plus only explicitly advanced current-
  world, remaining-budget, and continuation-use fields where applicable;
- identified Continuation only where it crosses a suspension, persistence, or
  handoff boundary, carrying the sole affine configuration token and exact
  ActivationStartRecord;
- ObservationIds distinct from Values, Results, and trace Terms;
- candidate deltas separate from continuations and Admission; and
- both governed-per-intent and preauthorized local/session/Lease/batch effect
  profiles, preserving exact intent, issued EffectAuthorization, independent
  CapabilityEvidence, attempt, optional receipt, observations,
  governed Judgment, and later Admission as a causal graph, with every actual
  event or attempt carrying an OccurrenceId plus exact provenance: producing
  Activation/Step for internal production or external-boundary provenance for
  an ingress trigger.

The companion must add exact cases for pure arithmetic, closure capture,
user-defined algebraic data with exhaustive matching, n-ary role closure,
duplicate equal Applications and Activations, an ongoing service, cancellation,
timeout without receipt, resumable exhaustion, source movement, repeated premise
slots, self-joins, occurrence-exact supports, rich values and collections,
rank-1 generic instantiation, local mutation, lifetime closure, native/Wasm
specialization, bounded long-run trace retention, the ratified combined general-
purpose source specimen, and the agent workbench proof. Malformed, open, wrong-mode,
missing or unreachable match cases, ungrounded-known-role, wrong-revision
resume, unauthorized cancel, fabricated receipt, and ambiguous-mode candidates
reject before acquiring partial authority.

Until canonical algebraic-data and match syntax is ratified, that companion
case is semantic IR plus exact observations and obligations, not a new source
spelling.

The crosswalk must not reinterpret the v0 fixture's `run` name as ActivationId,
StepId, or RunId; infer causality from JSON order; fabricate a receipt; or
silently migrate a live Activation to a new Program or world revision.

### Exact companion record

Every ratified process case must commit exact input bytes and checksums, the
selected semantics and Program/world/session pins, identity-allocation basis
and applicable authority,
Application/Activation/Run identities, canonically encoded StepRecords with
fresh nominal StepIds, explicit finite typed StepCauseFrontiers, separate exact
StepConfigurationTransitions, IncomingRunEdges, observations and occurrence-
exact supports, outcome, continuation, candidate delta, Admission decision, and
authoritative-boundary hashes before and after. A physical case additionally commits its
strategy, budget, total work receipt, physical-view hash, and handle/scene
table before and after. A rejection fixes the rejecting stage, typed reason,
obligations, and all boundaries proven unchanged.

Where applicable the record also fixes StaticActivationBasis,
CheckedConstitutionBinding, exact DynamicPrerequisiteSchema, slot-preserving
bindings and occurrence-only projection, StaticParameterTelescope,
StaticConstraintTelescope, resolution contracts and scope commitments,
normalized evidence, InstantiationUseRefs, InstantiationKeys,
SpecializationKeys, PhysicalReuseKeys, Activation-local slot and Step-scratch
contracts, anonymous-reduction count, mandatory Step cuts, allocation roots,
Borrow/Lease edges, retirement and explicit close/dispose events, physical
reclamation receipt, trace-retention contract, declared layout/ABI, and erasure
strategy. A local-only case must prove that no StateRevision, Admission, or
mandatory retained trace was allocated.

Names used while authoring the manifest are never semantic identity by
spelling. Once the process identity encoding is accepted, the companion
must carry its canonical ID bytes and preimages; equality/inequality-only
placeholders cannot satisfy an exact gate. Every identity record names its
exact checked allocation basis. Authority, predecessor, or retention evidence
is additionally required only when that identity kind's declared formation or
governed boundary requires it; static allocation never invents authority.
Wrong-kind, wrong-authority where applicable, self-authorizing, equal-content transplant, and
already-used-occurrence variants reject before partial authority. Historical
replay with an existing identity is not a fresh occurrence.

### Restarted continuation and causal schedule vectors

`resume-rematerialized-fresh-observation` must perform this exact sequence:

1. activate one Application and run through at least one nonterminal Step;
2. suspend at `suspend-step`, emit a boundary-crossing Continuation, and record
   its complete canonical bytes, ActivationStartRecord, and sole affine
   configuration-ownership token;
3. terminate that executor and rematerialize the bytes in an independent
   runtime with no shared heap or handle table;
4. supply one newly identified ingress occurrence absent from the serialized
   remainder; and
5. resume the same Activation and Run, creating one fresh Step whose
   StepCauseFrontier contains exactly
   `ContinuationTakeup(exact ContinuationId, original RunId,
   original ActivationId, suspend-step, exact ResumptionOccurrenceId)`; the
   emitting-Step edge is part of that one cause and is not duplicated as
   `PriorStep`. The Step also emits one fresh ObservationId supported by the
   ingress bound through that ResumptionOccurrence.

The post-resume observation bytes and support must differ from every
pre-suspension observation; trace replay does not pass. Separate one-field
negative vectors change each field of ActivationStartRecord or its derived
Application, Mode, constitution, initial-world/session/policy/semantics/budget/
cancellation pin set, plus the current-world advance proof, Activation, Run,
emitting Step, remaining budget, and configuration token. Additional vectors transplant unchanged
bytes to an equal-shaped independent Application, Activation, Run, or session.
All reject before a new Step. Because this fixture carries affine
configuration, first takeup consumes its token and both sequential and
concurrent reuse reject as `continuation-already-consumed` without duplicate
observations or effects. A separate reusable vector must use an immutable/Copy
remainder; a mutable reusable fork receives fresh child Activation and
configuration identities.

`handoff-distinct-roots` uses a Continuation emitted by Step `e` and a
HandoffOccurrence whose direct same-Run provenance root is a different already
constituted Step `p`. `HandoffFrom.parent_step` must equal `e` and match the
Continuation's recorded Run and Activation; the occurrence targets that exact
Continuation plus the destination StaticActivationBasis and InitialContext.
The child first Step uses its exact legal ActivationStart frontier, whose edge
projection contains the distinct union of `e`, `p`, and its ordinary same-Run
Activation occurrence ancestry. A coincident emitter/provenance root appears
once. One-field negatives select a wrong emitter, Continuation, parent Run or
Activation, target basis or pin, future/cyclic occurrence, or omit either
ancestry root; all reject before child or StepId allocation.

`join-left-first`, `join-right-first`, and `join-parallel` use fresh identities
in separate Runs but the same Clause process data. The parent split consumes
one configuration owner under one exact Mode-owned `SplitJoinContract` and
atomically co-forms the split Step, SplitInstance, and one typed BranchSlot,
matching BranchSpec, fresh child Activation, exact ChildOf/ChildIn binding, and
initial child and branch token per slot. Every component validates before any
is published; a negative must leave the parent token live and publish no Step,
instance, child, binding, or token. Each child Activation has an exact
`ChildOf` origin, and its normal first StepCauseFrontier is exactly one
`ActivationStart(child ActivationId)`. That cause projects the distinct union
of the exact parent split Step, direct same-Run handoff provenance roots when
present, and ordinary same-Run Activation occurrence ancestry into RunOrder. Independently,
the first child Branch Step consumes its exact split-produced
`BranchConfigurationToken` and contributes the same endpoints as a typed
configuration-succession edge. The two edge kinds remain inspectable, and
neither inserts `PriorStep`. The first two cases
force opposite child completion orders; the third uses physically separate
workers and a barrier. Each terminal Branch Step consumes its current token
into exactly one `Returned` or obligation-complete `Closed` settlement. In
every Run the Join StepCauseFrontier contains exactly two causes:
`PriorStep(exact RunId, left ActivationId, left-terminal-step)` and
`PriorStep(exact RunId, right ActivationId, right-terminal-step)`. Neither
child's later StepCauseFrontier names the other child. Each Run carries its
canonical BranchSlot-ordered settlement sequence. A typed
`ScheduleIsomorphism π` over every fresh run-local identity must satisfy
`encode(π(runA)) = encode(runB)`. Joined payload/Value/Result, observation and
support content, candidate delta, Admission decision, and continuation
disposition are literally equal only as schedule-independent projections;
fresh PriorSteps, terminal-Step-bearing settlements, and other identity bytes
are isomorphic rather than equal. Trace order may differ. The Join consumes
exactly one settlement per BranchSlot in canonical BranchSlot order and
restores one owner. Separate negatives cover an overlapping partition or write
Lease; a missing, extra, duplicate, wrong-slot, already-used,
wrong-contract, or cross-SplitInstance settlement; a Join frontier/settlement
mismatch; a cancellation settlement leaving an exact AllocationRoot, Borrow,
Lease, Continuation, effect, or close obligation neither discharged nor
transferred exactly as declared; and double Join. None
publishes a successor configuration. An obligation-complete `Closed`
cancellation settlement is a valid exact Join input.

`join-repeated-key` declares two BranchSpecs with one equal BranchKey and
contiguous ordinals zero and one. Atomic SplitFormation, both child bindings,
tokens, Returned or Closed settlements, and canonical Join retain the two exact
BranchSlots without collapse. A missing, duplicated, noncontiguous, reordered,
or transplanted ordinal rejects before the applicable Split or Join publishes.

The race fixture carries its cancellation/yield/deadline decision table and
logical deadline boundary as Clause data. It fixes cases for yield causally
before cancel, cancel causally before yield, and cancel concurrent with the
deadline, then runs each under opposite queue order and worker count. Expected
typed StepCauseFrontiers, including exact
`CancellationRequest(CancellationOccurrenceId)` causes only for Steps that
observe or carry through cancellation, yielded observations, continuation
disposition, typed terminal outcome, and resource balance come from that table.
Wall-clock arrival, log order, and first host callback are not inputs.

`cancel-ready` constitutes one Activation with zero owned Steps and live,
unconsumed initial custody, then presents one already validated cancellation
occurrence. Its sole StepCauseFrontier is exactly
`{ActivationStart(a), CancellationRequest(c)}` and its checked outcome is the
matching `Cancel(c)`. One-field negatives change target, Application/Mode or
context pins, occurrence refinement, outcome, add a cause, consume the initial
token, or create a prior Step; each rejects before StepId allocation. The
ordinary ready case retains the exact ActivationStart singleton. A nonfirst
Step with an empty frontier passes only when its transition contributes a live
configuration predecessor; a vector with empty IncomingRunEdges rejects before
allocation.

`bounded-history-long-run` executes for at least ten times each configured
resident window. It fixes maximum resident configuration, active-frontier,
continuation, diagnostic, and trace bytes/records; exact externalization or
compaction operations and acknowledgments; and resource-ledger high-water.
Referencing one evicted cause either rehydrates its exact checked witness within
budget or rejects `history-unavailable` before Step allocation. A compact
summary, host log position, or GC reachability never supplies authority.

### Truth without implicit assertion

One exact finite interpretation contains independently identified proposition
contents for supported true, explicitly supported false, and absent. Three pure
truth-directed Activations must emit respectively `true`, `false`, and
`absent`. The first two name their exact positive or negative assertion
occurrence supports. `absent` has no such support and never aliases false.
Hashes of the assertion set, governed Judgments, ProgramRevision, and
StateRevision remain unchanged in all three cases. A separate later assertion
and a separate later Judgment receive fresh occurrence identities; neither may
be retroactively attributed to evaluation.

### Exact effect-prerequisite vectors

`governed-per-intent` binds three distinct Mode slots: exact governed
EffectIntentOccurrence plus the AdmissionOccurrence establishing it; issued
`AuthorizationOccurrenceId<EffectAuthorization>` covering that intent, action,
scope, and policy; and independent `CapabilityEvidence<C>` covering the exact
boundary, resource, pins, validity, and budget. The occurrence-only
ActivationCauseFrontier projects the intent, Admission, Authorization, and any
occurrence-backed capability under exact slot/ordinal labels. Only then may its
ActivationId and EffectAttemptOccurrence be allocated.

`preauthorized-effect` contains exact session, Lease, batch, and Activation-
local variants. Each binds three distinct slots for the exact intent occurrence,
a previously issued EffectAuthorization occurrence covering its bounded scope,
and independent CapabilityEvidence, plus attempt bounds and renewal/expiry
rules. The scope may cover several attempts. No preauthorized variant allocates
a per-attempt StateRevision, AdmissionOccurrence, or new
AuthorizationOccurrence. A statically pinned authorization or capability may
erase from the checked hot ABI but retains its exact semantic slot and cold
explanation. Constitutive execution authority never replaces issued effect
authorization.

Independent negatives cover every prerequisite required by the selected
profile: missing, unadmitted when governed, stale, wrong-intent, and equal-
content-transplanted intent; missing, wrong-kind, stale, wrong-scope/policy
Authorization; and missing, expired, wrong-type/boundary/resource/pin
capability. Governed-per-intent additionally rejects constitutive-instead-of-
issued Authorization; preauthorized profiles reject any injected governed-only
Admission slot and also reject constitutive-instead-of-issued authorization.
Wrong-slot, duplicate-for-one-slot, and same-kind multiplicity collapse also
reject. Every negative leaves its affected EffectAttemptOccurrence unallocated
and every authoritative boundary unchanged; a pre-Activation failure also
leaves ActivationId unallocated.

### Recursive specialization and physical-strategy vectors

`generic-self-scc` and `generic-mutual-scc` form closed rank-1 specializations
with respectively one self-recursive member and two mutually recursive members.
Canonical alpha-normalized local member anchors make each graph finite: one
`SpecializationSccKey` commits to its complete multiplicity-preserving member
graph and external dependency closure, and each member `SpecializationKey`
selects from that object. Reordered source, renamed local spellings, reversed
discovery order, and source-only movement preserve every key. A body or call-
edge edit changes the whole SCC key and all dependent member specializations.
Static-argument/evidence cycles remain a separate negative that rejects before
SCC construction.

Each positive specialization runs through monomorphized, dictionary-passing,
irrelevant-evidence-erased, and shared-code physical strategies on native and
Wasm targets. All produce the same declared semantic identities, values,
observations, occurrence-exact supports, failures, diagnostics, and resource
outcomes. Strategy-specific PhysicalReuseKeys and ArtifactIds may differ, but
every result retains the exact cold link from InstantiationUseRef through
InstantiationKey and SpecializationKey to its strategy and artifact; no
strategy collapses nominal Applications or Activations.

### Local lifetime and reclamation vectors

Each allocation record carries exactly one Owned, RegionMember, or
ForeignManaged root and zero or more typed Borrow/Lease edges. Forced failure
or cancellation fires at every move, write, drop, Lease issue/revoke/close, and
Step-cut boundary; the exact consumed ConfigurationCustody_before (whole token,
branch token, or canonical settlement sequence selected by the transition) and
the resource ledger must be restored through an infallible suffix, bounded
undo/shadow, or discarded private realization, with no duplicate or residual
custody. Overlapping write Lease, scratch escape, a cancelled split branch whose
`Closed` settlement leaves an exact AllocationRoot, Borrow, Lease,
Continuation, effect, or close obligation neither discharged nor transferred
exactly as declared, missing/double Join, use-after-move,
and unknown close obligation reject without publication. An obligation-complete
`Closed` cancellation settlement remains valid.

The root-wide matrix includes direct owner access plus every Borrow and Lease,
and proves shared-read compatibility, exclusive/overlapping-write rejection,
and disjoint-write acceptance. Reset/reclaim is withheld until every Borrow,
Lease, Continuation, child, escape, asynchronous/foreign use, and close
obligation has causally acknowledged quiescence. Silence, timeout, revocation
request, and host unreachability do not close a token. Strong Owned↔Owned,
Owned↔Region, and other cross-root cycles reject; one enclosing
DeterministicRegion may reclaim an internal cycle only through its declared
whole-region closure. A separate non-game `managed-island-bounded-cycle` vector
declares its finite external roots, collection strategy, capacity, work/pause
budget, trigger, and overflow result. It accepts only an entirely internal cycle;
a strong edge crossing the island, open semantic obligation, budget overrun, or
use in the controlled game hot profile rejects.

`reclaim-without-observable-finalization` resets a Region and deallocates an
Owned object while proving zero observable destructor, finalizer, callback,
effect, or Observation invocations. A separate explicit close/dispose
Activation and, where external, effect attempt/receipt must precede reclaim.
Closing a foreign Lease discharges its declared adapter obligation but never
claims the foreign heap reclaimed bytes at that instant.

`bounded-mechanical-drop` proves a finite compiler-generated teardown that
cannot call user/foreign code, emit an Observation, or cascade beyond its bound.
`foreign-partial-init` fails after an external allocation attempt, publishes no
Clause view or handle, restores all Clause-controlled state, and records exactly
one of cleanup-success, cleanup-failure, or pending-quarantine without claiming
atomic foreign rollback.

### Total materialization receipts

Every materialization operation is accounted from API entry through the
returned receipt. The receipt contains exact counts for contract/input
validation, graph and support reads, index-bucket probes, premise occurrences
visited, candidate bindings, support entries read and written, whole-state
clones, whole-view rebuilds, support-set clones, disconnected rows visited,
allocation calls, allocated bytes, and peak live bytes. It also contains the
selected plan and fallback, typed failure or exhaustion, and whether a new
physical view was published. Deferred work and work performed by helpers,
copy-on-write layers, preflights, and receipt construction are included.

The companion declares concrete `base-population` and larger
`disconnected-population` integers and the exact receipt expected from each
plan. For the same local admitted delta, the larger indexed/incremental case
must report exactly zero whole-state clones, whole-view rebuilds, support-set
clones, and disconnected-row visits. Its remaining update counts and allocation
bound must equal the base case. Index construction has a separate receipt and
may scale. The cold scan reports its complete scaling work.

Separate vectors cover repeated premise slots, self-joins, equal content from
distinct Activations, and retraction with two independent supports. An
oversized extent must select the declared typed cold-scan fallback before any
unbounded indexed allocation. Exact `limit + 1` allocation and forced failure
vectors either complete through that bounded fallback or return typed
exhaustion. Every failure point preserves the previous physical-view and
support hashes, publishes no prefix, and retains exact support multiplicity.

### Wasm and passive-host vectors

The Wasm cases fix request/response bytes, exact length and allocation limits,
typed status, and handle table before/after. They include one valid pure round
trip; truncated and noncanonical canonical-package input; declared and actual
input length `limit + 1`; stale handle generation; stale Program,
StateRevision, and RuntimeSession pins; and a pure canonical result of exactly
`output-limit + 1`. Malformed or oversized input allocates no handle and starts
no Activation. Stale input performs no Step. Oversized output publishes no
prefix. Rejection never traps, wraps a length, mutates a semantic boundary, or
accepts a physical handle as semantic identity.

The passive-host cases fix immutable input-frame bytes, prior scene-projection
hash, expected render observation, resulting scene-projection hash, and listener
and resource counts. They include a valid admission-free frame with exact
Run/Activation/Step/Observation identity and no StateRevision; an otherwise
identical frame with optional unchanged `Wbase`; a valid admitted-StateRevision
frame; two independently allocated host objects containing the same immutable
frame; a causally stale Step/Observation predecessor; a stale admitted predecessor after
its successor is displayed; two causally unordered frames under an explicit
merge policy; missing and non-finite fields; object count `limit + 1`;
keyboard input after canvas focus loss; and input plus render after disposal.
Callback or transport order never creates freshness. Rejection leaves both
caller frame and scene projection unchanged. A valid render may change only
renderer-owned physical objects. After initialization the Clause/Wasm/adapter-
controlled path allocates nothing, does not call `memory.grow`, clone the whole
frame/carrier, scan a global heap, invoke observable destructor/finalizer work,
or trigger unbounded teardown. Initialization fixes capacities, Clause-state
rollback, explicit foreign cleanup success/failure/quarantine, allocation calls/
bytes, pool high-water, Wasm pages, adapter calls,
and resource-ledger before/after; `capacity + 1` publishes nothing. Every
foreign call has an allocation/disposal contract, and a browser-wide zero-
allocation claim additionally requires instrumented warm-up/lazy-cache
evidence. Disposal is an explicit terminal Activation/Step and applicable
effect before wrapper reclamation, is idempotent, removes every owned
listener/resource exactly once, and permits no later callback-owned work.

### Process-v1 canonical source specimens

The companion does not copy or reinterpret the three historical v0
`.clause-v0.txt` payloads. It copies the three printed code blocks and the separately ratified
fourth combined general-purpose source specimen in the adoption spike's
“Frozen ordinary-source ergonomics” section into separately checksummed source
files. Pure definition and relational request source contain no process IDs,
revision pins, authority token, scheduler, budget, trace, or physical plan. The
state-change specimen exposes only its semantically relevant canonical process
words. Canonical parse/print/parse preserves the exact accepted projection,
while the semantic crosswalk still exposes every generated identity and pin
plus the exact possibly empty DynamicPrerequisiteSchema, slot bindings, and
occurrence-only projection. The fourth source combines rank-1 generic use,
loop/builder, and move/borrow/region/Lease boundaries and must also pass one
workbench edit with exact affected and preserved dependency/cache sets. A
printer that injects process bookkeeping or a host sidecar needed to recover
hidden meaning fails the fixture.

### Agent-native workbench transcript

The companion fixes one ordered request/response transcript against one
long-lived stdio service. It includes `parse`, `check`, `explain`, `query`,
`diff`, `propose`, `admit`, `run`, and `hotReload` over one arithmetic
definition, one relation/query, and the ratified combined general-purpose
source fixture. Exact responses contain canonical package and constitution plus
applicable revision pins, stable typed diagnostics, rejecting semantic stage,
exact failed formation/subject and obligation, source-origin sets, dependency
slices, boundaries proven unchanged, and
`why`/`prevent`/`achieve`/`diff` results. The pure run changes no StateRevision.
The source proposal is distinct from Admission, and hot reload either preserves
all live pins or rejects with exact obligations.

The edit case names an exact source/package base, applies all changes atomically,
and returns exact affected and preserved dependency/cache sets. A stale-base
variant returns the current exact identity and leaves source, semantic package,
and live pins unchanged. Suggested edits are separately typed advice rather
than silently applied rewrites or authority.

The transcript also fixes the Rust boundary: bounded stdio framing, pins,
cache/transaction mechanics, and scheduling only. Every semantic response must
come from accepted package definitions executed through CLCP03 and the generic
runtime. A host parser, checker, query engine, diagnostic case, or secondary
interpreter fails even if its bytes match once. Interactive requests may use
accepted incremental summaries; the separately fixed promotion transcript
performs exact Lean and compiler-succession replay.

The evolution program admits two changes in order. Reapplying the second change
to the root or to an equal-looking but different base rejects. Verification
evidence and lifecycle observations do not enter the fixture revision identity.

The published canonical-package lineage proves one exact bootstrap-to-successor
transition. The successor basis contains no root or rule authorizing a further
basis-admission claim, so a third package must reject under v0. The two Program
changes in this corpus run inside the already accepted successor semantics;
they do not pretend to extend constitutional package authority.

## Replay boundary

The corpus depends on exact bootstrap revision
`2ea651db7c525249c465dceb0f8c5474d635fae6`. A replay uses the current tracked
corpus and its checksums; no publication object or release manifest is part of
its semantics. Two isolated replays use the repository-pinned Lean and Rust
toolchains and share no
checkout, build cache, package cache, output directory, runtime state, or
ambient configuration. They must independently observe identical:

- source and corpus bytes;
- accepted and rejected verification decisions;
- Run outcomes, candidates, traces, and admission decisions;
- canonical package bytes; and
- ordered evolution history.

The effect adapter used by replay is a deterministic isolated fixture. Each
replay may execute its own attempt once; replaying the resulting trace inside
either run must execute it zero additional times. Public CLI commands and
distribution paths are owned by the later CLI tranche and must satisfy this
contract rather than editing it around implementation choices.
