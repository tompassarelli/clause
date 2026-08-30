# Clause Roadmap

> **Status:** Current.
>
> **Authority:** Sole authority for implementation status, dependency order,
> milestone scope, and exit evidence. The [foundation](foundation.md) governs
> semantics, [syntax](syntax.md) governs canonical source, and
> [architecture](architecture.md) governs implementation boundaries.

## Current position

Clause has accepted the process-first constitutional reset, but no end-to-end
process-first capability is accepted or supported. The public implementation
evidence predating the reset remains valuable but narrower than the new
semantic claim:

- one neutral recursive `Term = Atom | RawTriple` carrier with no mandatory
  Triple identity;
- provisional Lean Context/Judgment carriers and a relative finite ground-
  certificate checker;
- strict CLCP v1 Lean/Rust codec parity and literal predecessor authority;
- the CLCP v3 Lean strict codec, carried manifest, evaluator/replay model, and
  complete 73-byte-receipt replay checker;
- the frozen v0 execution/admission/replay observation corpus; and
- the bounded historical `game_leverage` cold-scan/indexed experiment.

Those artifacts do not implement `FormationJudgment`, `RelationSchema`,
`OperatorRef`, `ModeId`, `ApplicationForm`, `ApplicationId`, `ActivationId`,
semantic `StepId`, `RunId`, typed Continuation, `ObservationId`, general
Admission, or the process-v1 corpus. There is no supported parser, compiler,
runtime, persistence format, CLI, Wasm boundary, renderer integration, or
example application.

The exact CLCP v3 bytes, hashed manifest, 73-byte receipt, left-to-right KExpr
evaluator, machine Continuation, evaluator step, fuel exhaustion,
`admitPropose`, and `CompilerRevisionId` remain frozen compiler-machine
mechanics. They are not silently reinterpreted as universal process semantics.
Clause-owned outer Terms and envelopes must carry process identities, pins,
authorizations, evidence, and governed deltas through that fixed machine.

No implementation claim follows from a documentation specimen, local branch,
test transcript, or host prototype. Git history is recovery, not semantic
authority.

Public main is `373feb16` and contains both the bounded passive-shell repair
`5bcb46e` and accepted exact compiler Terms from reviewed source object
`fe1de91`. Terms follow-up `e3cc667` was
rejected by its adjacent-boundary review; `b70e3b8` is its unreviewed bounded
successor at this checkpoint. The construct-blind runtime `3ecadab` remains a
clean non-main candidate. Materialization repair is dirty atop `6765a12`, and
jump-law repair is dirty atop `e1b24a7`; neither base is a current accepted
artifact. The process carrier, Compiler0, and Rust-v3 lanes also remain active
recovery inputs. No non-main object is authoritative, accepted, or integrated
merely by existing.

## Preserved oracles

The following evidence remains unchanged while the reset is implemented:

- the three historical v0 payloads keep their exact bytes and original hashes
  under `clause:test-vectors/execution/historical-v0/source-projections/`;
  paths and classification metadata may change to preserve an honest
  noncanonical-fixture boundary;
- its six tags stay `returned`, `choices`, `yielded`, `suspended`, `failed`, and
  `exhausted`;
- pure dependency closure, State/effect fulfillment, and Program evolution
  preserve their exact observations;
- source-only movement preserves ApplicationId only when the exact
  ProgramSnapshot, ApplicationForm, and nominal identity are unchanged;
  cross-revision continuity uses separate ReferentId evidence;
- equal assertion and premise occurrences remain distinct;
- neutral Triple slots never acquire an inherent operator role;
- CLCP v1/v3 exact-byte and replay evidence remains scoped to its published
  contract; and
- the unchanged position/radius source law remains the cold-semantics input for
  later scan/indexed materialization parity.

New process-v1 fixtures extend these oracles; they do not rewrite them.

## Lifecycle and sequencing

Executable capability advances through three distinct states:

1. An **experimental implementation or falsification artifact** may land with
   explicit non-authority, a bounded claim, deterministic checks for that
   claim, reversible scope, and no supported-language claim.
2. A **semantic candidate** maps proposed behavior into the accepted process
   constitution. It remains a candidate; Lean, Rust, a package, an index, a
   browser, or successful execution cannot invent Clause meaning.
3. **Supported or admitted capability** passes every applicable exact identity,
   cross-host parity, negative-fixture, hidden-authority, cold-equivalence,
   specialization, and absence gate before promotion or release.

Constitutional dependencies block integration and promotion, not bounded
independent experiments. Implementation can proceed in parallel where paths
and semantic inputs are genuinely independent. No candidate may be integrated
against superseded graph-first, proposition-first, clause-as-Application, or
host-owned game semantics. A lower-case clause remains `ClauseJudgment`
content over a neutral Term; it is neither every Term nor every Application.

## Gap matrix

| Boundary | Current evidence | Missing exit evidence |
| --- | --- | --- |
| Neutral representation | Term/Atom/RawTriple candidates exist | Accepted equality contracts and process-era checked formation |
| Process constitution | Public documentation defines the candidate | Executable Formation/Application/Activation/Step/Run/Continuation rules |
| Typed identity | Program/State identity design and occurrence evidence exist | Checked ApplicationShape/Application/Activation/Step/Run/Continuation/Observation domains and non-interchange proofs |
| Relations and modes | Source design and historical relation experiments exist | Separate RelationSchema, revision-indexed extension, OperatorRef, Mode, Reading, and authorization contracts |
| General-purpose values and reuse | Numeric/source specimens and ADT falsifier prose only | Rich values/collections, rank-1 generics, coherent constraints, separate compilation, and exact diagnostics |
| Local state and memory | ActivationConfiguration and erasable internal-reduction semantics only | Affine local slots, Step cuts, exact allocation roots plus Borrow/Lease edges, bounded nonobservable mechanical reclaim after explicit observable close, optional bounded managed islands outside the game hot path, bounded trace retention, and no-mandatory-GC frame proof |
| Physical competence | Generic Rust bootstrap and historical materialization experiments | Clause-owned physical IR, layout/ABI control, erasure, and direct native/Wasm specialization |
| Agent workbench | No supported parser, CLI, or semantic service | One long-lived CLCP03-backed service with transactional source and exact semantic operations |
| Compiler machine | CLCP v3 Lean replay is implemented | Rust v3 parity, literal process-aware Compiler0, external anchor, and accepted successor |
| Compiler ownership | Genesis/host-freeze contract exists | Compiler0 migration away from provisional `JUDGMENT_ID`/check-decision payloads and one frozen-host evolution |
| Execution oracle | v0 execution/admission/replay observation corpus is frozen | process-v1 crosswalk plus the accepted pure/local and process/effect slices |
| Terms | Exact compiler Terms are integrated on public main at `373feb16`; follow-up `e3cc667` was rejected and bounded successor `b70e3b8` awaits review | Accept only a reviewed follow-up; preserve the public Terms object meanwhile |
| Materialization | Active dirty repair is based on non-accepted `6765a12`; no current materialization candidate has passed independent review | Finish repair, independent acceptance, admitted-delta adapter, and occurrence-exact locality proof |
| Transition/effects | Active jump-law repair is based on non-accepted `e1b24a7`; `3ecadab` remains a non-main construct-blind runtime candidate and no accepted integration exists | Clause-owned lowering, Mode-selected governed/preauthorized effect profiles, and accepted integration |
| Wasm/browser | `5bcb46e` passive-shell repair is public; no Wasm adapter or process-frame integration exists | Bounded ABI, native/Wasm parity, process-identity frame envelope, preallocated controlled hot path, and integration |
| Product proof | Historical game-leverage experiment exists | Clause-owned playable mechanic, second substantial Clause-only extension, and passive rendering |

## General-purpose replacement gates

Clause targets replacement of Rust, TypeScript, and other general-purpose
languages above explicit irreducible operating-system, browser, device, and
foreign boundaries. That target requires rich values and collections,
parametric reuse, ownership and regions, layout and ABI control, concurrency,
effects and system interfaces, native/Wasm/browser execution, FFI,
modules/packages, and agent-grade tooling. None is supported today. The
following bounded ladder controls the claim; implementation may proceed in
parallel, but only a contiguous passed prefix counts as evidence.

| Gate | Required proof | Decision-changing failure |
| --- | --- | --- |
| 0 — bootstrap tractability | Fixed CLCP03 evaluator executes Compiler0 genesis/succession under the strict published resource bound; accepted source reaches package output with no host parser or semantic interpreter. | The trusted evaluator cannot run the minimum compiler within a bounded profile, or a host semantic shortcut is required. |
| 1 — agent workbench | One long-lived stdio service runs accepted package definitions for `parse`, `check`, `explain`, `query`, `diff`, `propose`, `admit`, `run`, and `hotReload`. Arithmetic plus one relation/query yields exact `why`/`prevent`/`achieve`/`diff`, stable typed diagnostics with exact obligations/origins/dependencies, atomic base-pinned edits, a pure result, and no StateRevision. | Rust must parse, check, answer semantic queries, invent diagnostics, or execute another language; an edit can partially apply or silently rebase; interactive checking requires full Lean/succession replay. |
| 2 — values and static reuse | Rich numbers, Unicode Text, Bytes, algebraic values, sequences/maps, rank-1 parameter/constraint telescopes, total static normalization, terminating complete resolution, exact scope commitments, distinct semantic/physical reuse keys, and separate compilation pass exact positive and negative cases. | Ordinary data work needs host types, ambient or incomplete constraint search, open instantiation, whole-graph invalidation, or source/runtime ceremony inconsistent with the compact surface. |
| 3 — local state and lifetime | Loops, builders, request-local caches, and a frame state machine use affine Activation-local configuration, one exact reclamation root plus Borrow/Lease edges, ownership-consuming split/join/suspension, explicit observable close, bounded nonobservable mechanical reclaim, an explicit bounded managed-island option outside the game hot path, and bounded trace retention. | Ordinary mutation requires Admission/StateRevision, hidden tracing/ARC/finalizers, observable destructor semantics at deallocation, or lifetime/history rules cannot determine a bounded runtime protocol. |
| 4 — physical competence | Clause-owned physical IR validates direct calls, registers, packed layouts, declared ABI, checking/specialization/physical cache separation, and native/Wasm parity; the bounded frame performs no controlled allocation after initialization. | Generic graph execution remains the hot path, layout/resource control requires Rust semantics, specialization changes declared observations/diagnostics, or zero-allocation exceeds its measured accounting domain. |
| 5 — systems competence | Checked concurrency, continuations, files, data transforms, networking, time, browser APIs, and explicit FFI preserve local/effect/Admission distinctions and bounded resources. | External work requires arbitrary host mutation, opaque scheduler semantics, or governance on ephemeral local state. |
| 6 — product development | The Clause-owned jump arena and one non-game system/data application are built and evolved through revision-pinned hot reload, passive hosts, and Clause-only semantic changes. | A feature change requires Rust/JavaScript semantic edits, frame-loop GC/allocation, silent migration, or unqueryable causality. |
| 7 — replacement evidence | Modules/packages, tooling, reproducible distribution, independent implementation, and measured agent development loops let the two products remain Clause-authored above named FFI boundaries with exact explanations and competitive target behavior. | Hidden host language, unbounded semantic tax, missing ordinary-program capability, or irreproducible package/runtime behavior remains load-bearing. |

Gate 1 is intentionally small. Rust owns stdio framing, exact pins, bounded
caches, transactions, and scheduling. Semantic operations execute accepted
Clause package definitions through CLCP03 and `clause-runtime`. Interactive
requests may consume accepted incremental summaries; exact Lean and compiler-
succession replay remains a promotion gate rather than an edit-loop tax.
Human-readable Clause source remains the compact audit and token surface.

Gates 2–4 do not count from semantic IR alone. The syntax authority already
ratifies one canonical source unit combining a rank-1 generic declaration/use,
ordinary loop and collection builder, and semantically relevant move, borrow,
region, and Lease boundaries. The exact UTF-8/LF bytes printed in
[`syntax.md`](syntax.md) and copied into the adoption spike must pass parse,
check, byte-identical canonical print/parse, and one transactional workbench
edit with exact diagnostics plus affected and preserved dependency/cache sets.

## Phase 0 — Freeze and crosswalk oracles

**Status:** The v0 execution/admission/replay observations and original source-
payload hashes are frozen; the slash-bearing source projections are quarantined
under the historical noncanonical fixture path, and the process-v1 companion
is missing.

Freeze the three exact original v0 source-payload byte streams and their actual
execution observations for:

- recursive pure dependency closure and its ground-rule negatives;
- independently identified assertion occurrences;
- candidate State delta versus admitted StateRevision;
- effect intent, attempt, receipt, trace replay, and later receipt admission;
  and
- two Program changes from exact predecessors, including stale-base rejection.

Add new exact process-v1 cases for the current gaps:

- pure arithmetic;
- closure capture and source movement;
- user-defined algebraic data plus exhaustive-match acceptance and exact
  missing/unreachable-case rejection;
- executable exact n-ary roles;
- duplicate equal premise, Application, Activation, Step, and observation
  occurrences;
- query absence versus falsehood;
- an ongoing service, suspension/resumption, cancellation, timeout without
  receipt, and budget exhaustion; and
- generated Rust/JavaScript/Wasm observation parity when those artifacts exist.

The companion maps the frozen v0 observations into the new identity and causal
domains without changing v0 bytes or inferring IDs from fixture-local names.

**Exit evidence:** exact v0 checksum preservation, explicit crosswalk rules,
and negative vectors for wrong-mode, open form, ambiguous mode, stale revision,
unauthorized cancellation, fabricated receipt, and ungrounded cycle.

## Phase 1 — Process constitution

**Status:** Documentation candidate; no implementation claim.

Land one internally consistent public authority defining:

- running, FormationJudgment, ApplicationForm, Application, Activation,
  Configuration, Step, Run, Continuation, Observation, Result, Value,
  Judgment, Admission, and Trace;
- exact typed identity/equivalence rules;
- RelationSchema, activation-scoped result relations, revision-indexed
  RelationExtension, OperatorRef, Mode, Reading, ExecutionAuthorization, and
  capability;
- StaticActivationBasis with an exact checked-candidate or admitted-constitution
  binding, separated from each Mode's possibly empty named/RoleId-indexed
  DynamicPrerequisiteSchema, exact bindings, and occurrence-only cause
  projection;
- affine Activation-local configuration, anonymous reductions, Step cuts,
  escape/alias/concurrency rules, and admission-free local mutation;
- rank-1 static parameter and constraint telescopes, total static
  normalization, terminating complete constraint resolution, exact resolution-
  scope commitments, distinct instantiation/provenance/specialization/physical
  reuse keys, and separate compilation;
- one exact Owned/RegionMember/ForeignManaged allocation root plus separate
  Borrow/Lease obligations, explicit observable close before bounded
  nonobservable mechanical reclamation, deterministic regions, explicitly
  bounded managed islands, bounded trace retention, and exact cycle/foreign-
  boundary rules;
- ProgramSnapshot as process constitution and StateRevision as admitted process
  boundary;
- exact long-lived Program/world pinning;
- three independent real-effect slots for governed-or-preauthorized intent,
  issued EffectAuthorization, and CapabilityEvidence, with governed-only
  Admission and exact attempt/optional-receipt/observation/Judgment/later-
  Admission causality;
- graph-as-canonical-carrier and cold-semantics refinement; and
- the implementation DAG and falsifiers in this roadmap.

**Exit evidence:** one clean exact-object documentation review with no
contradictory clause-as-Application, graph-authority, rest/motion, overloaded
Run arrow, or continuation/candidate-delta collapse in the edited authority
set.

## Phase 2 — Checked formation and process identities

**Status:** Not implemented.

Introduce the smallest host-neutral checked distinctions:

```text
RelationSchemaId
RoleId
OperatorRef
ModeId
InstantiationUseRef
InstantiationKey
SpecializationKey
PhysicalReuseKey
ApplicationShapeId
ApplicationId
ActivationId
StepId
RunId
ContinuationId
ObservationId
```

`InstantiationUseRef` is exact snapshot provenance. `InstantiationKey` is a
cross-snapshot interface-checking key, `SpecializationKey` adds body/transitive
semantics, and `PhysicalReuseKey` adds lowering, target/profile, ABI/layout/
strategy, and physical dependencies. Their presence prevents a host-only cache
from deciding compatibility; none is nominal identity or authority.
Keep `ReferentId` as the established nominal referent identity; do not add an
indistinct universal `ConceptId`. Raw transport bytes may share width while
semantic ID domains remain non-interchangeable.
`ClauseId` is retired as a public identity domain; every nominal Application
has ApplicationId, while raw, quoted, open, or merely structural forms are not
anonymous Applications.

RelationSchemaId and OperatorRef pair `ProgramSnapshotId` with typed snapshot-
local declaration identity; RoleId is local to its exact RelationSchemaId, and
ModeId is local to its exact OperatorRef. A changed ProgramSnapshot never
silently preserves any of them. Cross-revision continuity, where a real
consumer requires it, is a separate ReferentId relation with explicit evidence.
ProgramRevision lineage alone never supplies declaration identity or
continuity.

Every ApplicationShapeId commits to ClauseSemanticsId, exact RelationSchemaId,
exact OperatorRef, the exact eligible ModeId set, named-role bindings, context
requirements, exact InstantiationUseRefs with their InstantiationKeys and
SpecializationKeys, and the complete resolved semantic-dependency/declaration
closure, including proof that the closure is empty where applicable.
PhysicalReuseKey is excluded.

Implement closed ApplicationForm formation, nominal Application allocation,
fresh Activation with an exact StaticActivationBasis, InitialContext,
slot-preserving DynamicPrerequisiteBindings, separate occurrence-only
ActivationCauseFrontier, ActivationStartRecord, and Run membership,
before/after Configurations under one stable ActivationId, fresh nominal StepIds
whose StepRecords separately carry finite typed StepCauseFrontiers and exact
configuration transitions, the normal first-Step ActivationStart singleton and
sole ready-cancellation pair, nonfirst IncomingRunEdges from frontier and/or
configuration succession, the closed Serial/Split/Branch/Join affine
configuration-transition sum with typed BranchSlots and atomic split
Step/instance/child/binding/token co-formation, exact HandoffFrom binding to the
Continuation emitter and destination basis/pins with well-founded occurrence
provenance and distinct ActivationStart ancestry projection, one exact
ActivationStartRecord from which fixed continuation pins derive, and typed
outcomes. Machine/KExpr reductions remain compiler mechanics rather
than semantic Steps.

**Exit evidence:** exact positive/negative identity vectors, reload and mobility
tests, one Application activated twice with distinct ActivationIds,
independently nominalized equal-shaped Applications with distinct
ApplicationIds, one Activation across several Steps, concurrent Steps without
invented total order, and no revision from pure running.

## Phase 3 — Relation, operator, mode, and authorization split

**Status:** Not implemented.

Make role schema, extensional content, process operator, executable mode, source
Reading, derivation authorization, ExecutionAuthorization, admission authority,
and effect capability explicitly separate. Activation selects one exact Mode
only after role closure and a valid StaticActivationBasis. It then checks only
the named/RoleId-indexed dynamic-prerequisite slots that Mode declares, closing
their multiplicity exactly and projecting only occurrence-producing evidence
into the separate causal frontier; the entire schema may be empty. A relation
with zero modes remains valid and non-executable. Checked-candidate running may
read a pinned admitted world, persist nonauthoritative output/Continuation, and
simulate effects inertly, but fabricates no revision, real attempt, or
constitutive authority. Joining an authoritative RuntimeSession, proposing
authoritative world change, relying on constitutive Program authority, or
performing a real effect uses an admitted-constitution binding.

A RelationSchema with no OperatorRef can still form checked bindings,
relational rows, proposition/assertion content, and patterns. It cannot form an
ApplicationForm.

Retain the accepted readable `relation` block as source grouping sugar whose
checked elaboration produces the separate objects. Ordinary source must not
acquire identity or process bookkeeping ceremony.
For the currently ratified projection, no `mode` clause means schema plus
Reading only, while one or more `mode` clauses also establish the grouped
OperatorRef. The semantic carrier may represent an operator with zero modes,
but its distinct canonical source spelling remains unratified rather than
ambiguous.

**Exit evidence:** n-ary complete-role vectors, multiple modes on one operator,
non-executable relation diagnosis, no positional role recovery, no implicit
assertion, and exact revision-indexed relational querying.

## Phase 4 — Early pure/local general-purpose slice

**Status:** Not implemented. The CLCP03 Rust and Compiler0 candidates are
preserved but not accepted.

The first implementation slice tests the cheap ordinary case before bundling
the entire process/effect model. It uses one accepted semantic carrier and the
real Compiler0/package/runtime path, not a disposable host interpreter:

1. implement checked Application/Activation/Step identity, exact
   StaticActivationBasis, exact possibly-empty DynamicPrerequisiteSchema and
   bindings, occurrence-only cause projection, affine ActivationConfiguration,
   anonymous reductions, and exact Step cuts;
2. complete strict bounded CLCP03 Rust parity, execute and admit exact
   Compiler0 from the external anchor, and prove one predecessor-bound
   successor evaluation without changing the frozen machine contract;
3. implement rich numbers, Unicode Text, Bytes, algebraic values,
   sequences/maps, closure capture, exhaustive matching, exact n-ary formation,
   rank-1 parameter/constraint telescopes, total static normalization,
   terminating complete resolution, exact scope commitments, normalized
   evidence, distinct checking/specialization/physical reuse keys, and separate
   compilation;
4. implement exact Owned/RegionMember/ForeignManaged roots, Borrow/Lease edges,
   moves, ownership-consuming split/join/suspension, bounded failure restoration,
   explicit observable close, bounded nonobservable mechanical reclaim,
   deterministic regions plus one explicitly bounded non-game managed-island
   fixture, exact cycle and trace-retention disposition, and the no-managed-
   island/no-mandatory-GC/controlled-no-allocation native/Wasm game profile;
5. lower the same accepted meaning through Clause-owned physical IR to direct
   native and minimal Wasm artifacts with declared layout/ABI and checked
   monomorphization, dictionary, and erasure strategies; and
6. round-trip the already ratified combined generic/loop/builder/move/borrow/
   region/Lease source fixture, then expose the one long-lived agent workbench
   with exact `parse`, `check`, `explain`, `query`, `diff`, `propose`, `admit`,
   `run`, and `hotReload` operations.

The proving program is deliberately small: pure arithmetic, one relation and
query, truth without implicit assertion, closure/ADT/n-ary formation, one
Application activated twice, a local loop/builder/cache, and exact
`why`/`prevent`/`achieve`/`diff`. Pure observations remain queryable without a
revision. Thousands of local reductions create only declared Step cuts and no
StateRevision, Admission, or mandatory retained trace. Interactive checks use accepted
incremental summaries; exact Lean/compiler-succession replay gates promotion,
not each edit.

Streaming, cancellation, governed State, external effects, materialization,
and passive rendering remain Phase 6 breadth. This split prevents a failure in
ordinary values, static reuse, memory, or specialization from hiding behind a
larger distributed-process demonstration.

The semantic-model candidate remains scoped to
`clause:lean/ClauseProcess.lean`, `clause:lean/ClauseProcess/**`, and
`clause:test-vectors/process-v1/**`. The existing CLCP03 and Compiler0 recovery
paths enumerated below remain inputs until the Phase 4 integrator applies their
exact accepted deltas. One integrator owns shared manifests, target crate
registration, the root corpus index, and the exact assembled package.

**Exit evidence:** adoption-spike Phase B passes through the real accepted
Compiler0/package/runtime/workbench, including all exact negatives; direct
native and Wasm observations agree; the frame hot path satisfies its resource
profile; and replacement Gates 0–4 have a contiguous passed prefix. The later
Phase C program remains open.

## Phase 5 — Compiler host-freeze, Terms, and materialization

**Status:** Exact compiler Terms from reviewed source object `fe1de91` are
accepted and integrated on public main at `373feb16`. Follow-up `e3cc667` is
rejected; bounded successor `b70e3b8` awaits
independent review and does not diminish the accepted public object.
Materialization remains a dirty repair atop non-accepted base `6765a12`.
Compiler0 and Rust v3 remain dirty candidates.

Bounded candidate work may proceed in parallel before this phase, but ordered
integration begins only from the accepted Phase 4 process contract:

1. preserve the accepted Terms object, independently review any follow-up
   successor against exact declarations, static parameters and evidence, role
   closure, Application/Activation/Step/Observation identity, and process
   envelopes, and land only a clean accepted successor;
2. complete Compiler0's wider lossless source occurrence preservation,
   deterministic Reading selection, exact focus/binding/origins, canonical
   printing, local recovery, semantic round trips, and Terms projection without
   reintroducing provisional `JUDGMENT_ID` or check-decision payloads;
3. perform one predecessor-authorized Compiler0-to-Compiler1 host-freeze change
   covering exactly one binding form, one effect form, one typed macro, and one
   diagnostic behavior with zero host semantic changes. Compiler0 already fixes
   the accepted process envelope and Compiler1 merely populates it. The user-
   defined algebraic-data/exhaustive-match positive and missing/unreachable
   negatives also pass under the frozen hosts; and
4. finish repair, independently review, and accept the current materialization
   successor behind admitted semantic deltas, then complete cold scan/indexed
   parity and end-to-end locality.

The materialization seam is exact:

```text
AdmittedStateDelta {
  ClauseSemanticsId, ProgramRevisionId, RuntimeSessionId,
  predecessor StateRevisionId, result StateRevisionId,
  producing ActivationId, producing StepId, semantic delta
}

MaterializationUpdate {
  AdmittedStateDelta,
  exact semantic graph ref, exact contract ref, exact physical plan ref,
  physical budget
}
```

Semantic governance creates `AdmittedStateDelta`. A materializer may validate,
project, and apply `MaterializationUpdate` to a replaceable physical view and
return a receipt or typed physical failure. It never allocates, admits, or owns
State history. Graph, contract, and plan identity belong to the physical update
and receipt; plan identity never enters `StateRevisionId`.

Parallel candidate lanes remain path-disjoint. Every `clause-substrate` path
below names a historical bootstrap or recovery location, not the target crate
boundary:

- Rust CLCP v3: `clause:crates/clause-substrate/src/compiler_package_v3/**`,
  `clause:crates/clause-substrate/tests/compiler_package_v3.rs`, and
  `clause:crates/clause-substrate/tests/evaluator_v3.rs`, plus the preserved
  host-mechanics gates
  `clause:crates/clause-substrate/tests/host_mechanics.rs` and
  `clause:crates/clause-substrate/tests/fixtures/compiler_runtime/{host-mechanics.tsv,source-ast-mechanics.tsv}`;
- Compiler0: `clause:compiler0/**`, `clause:tools/compiler0-materializer/**`,
  and `clause:test-vectors/compiler-genesis/**`; this lane owns the lossless
  parser/Reading selection, canonical printer, local recovery, and semantic
  round-trip artifacts inside that scope;
- Terms candidate: `clause:docs/compiler-terms.md` and
  `clause:test-vectors/compiler-terms/**`; and
- materialization candidate: `clause:crates/clause-materialization/**`. The old
  `clause-substrate` materialization paths and `274136a` are recovery evidence
  only.

One integration owner alone edits shared workspace/package manifests, Lean
build manifests and roots, `clause:crates/clause-substrate/src/lib.rs`, shared
`clause:crates/clause-substrate/src/evaluator/mod.rs`,
`clause:crates/clause-substrate/src/physical/mod.rs`,
`clause:crates/clause-substrate/src/artifacts/mod.rs`, and shared corpus
manifests/checksums. Scope-local corpus manifests remain with their named lane.
Existing candidate edits to shared paths stay preserved as recovery evidence;
the integrator reconciles them once, and candidate lanes make no further shared-
path edits concurrently.

The dirty Rust candidate's removals under
`clause:crates/clause-substrate/src/compiler_package_v2/**`,
`clause:crates/clause-substrate/tests/compiler_package_v2.rs`, and
`clause:crates/clause-substrate/tests/evaluator_v2.rs` are not an accepted
retirement. The Phase 5 integrator keeps v2 live until the accepted v3 successor
and every in-tree consumer have migrated, ports any still-required oracle, and
only then removes the complete v2 surface in that same integrated change.

The current `clause-substrate` name and single-crate boundary are historical
bootstrap artifacts, not the target architecture. Phase 4 establishes new
process-facing `clause-package` and `clause-runtime` boundaries while retaining
`clause-substrate` temporarily for frozen bootstrap consumers. After the
accepted v3, runtime, and materialization successors have been reconciled from
their recovery lanes, Phase 5 migrates every remaining consumer and completes
the responsibility split:

- `clause-package` owns exact CLCP codecs, canonical bytes, and package
  validation;
- `clause-runtime` owns the construct-blind evaluator and generic process
  execution protocol; and
- `clause-materialization` owns scan, index, and incremental physical
  projections.

Phase 4 may carry the minimal pure Wasm specialization needed by its falsifier;
Phase 6 establishes `clause-wasm` as the complete bounded process transport
adapter. The crate name `clause` is the target user-facing facade and CLI
aggregation boundary; it delegates to the responsibility crates and owns no
language semantics.
No Rust crate named
`clause-core`, `clause-common`, or `clause-semantics` may become a shared junk
drawer or imply that host code owns language meaning. The old paths above name
recovery scopes only; accepted deltas are applied to the target boundaries
rather than merging candidate branch trees or duplicating frozen bytes.

The unchanged spatial law must produce identical observation and occurrence-
support multisets through cold scan and indexed/incremental plans. Repeated
premise slots, self-joins, equal content from distinct Activations, fallback,
allocation exhaustion, graph/plan pins, and disconnected-population locality
must remain exact. Physical plan changes never change StateRevision identity.

**Exit evidence:** the complete cross-phase adoption program passes: accepted
Compiler1 host-freeze evolution, user-defined algebraic-data/exhaustive-match
positive and negatives under frozen hosts, independently reviewed Terms and
materialization objects, lossless source occurrences, deterministic Reading
selection, canonical parse/print/parse meaning, local recovery and semantic
round trips, construct-blind hosts, cold parity, and measured end-to-end
locality without hidden whole-state work.

## Phase 6 — Clause-owned transition, effects, Wasm, and passive rendering

**Status:** No accepted integrated implementation. Public main contains passive
shell repair `5bcb46e`; `3ecadab` remains a clean non-main construct-blind
runtime candidate, while the active jump-law repair is dirty atop non-accepted
base `e1b24a7`. No `clause-wasm` crate exists.

Complete the adoption-spike Phase C breadth on the same Phase 4 implementation:
recursive derivation; an ongoing service with real suspension/restart;
continuation handoff and cancellation races; budget exhaustion; governed State
transition; honest external effects; and a hygienic compiler process proposing
a Program delta. None may fork a toy runtime or reintroduce a mandatory dynamic
Authorization for ordinary pure/local Modes.

Define the minimum Clause-owned process ABI as opaque accepted package
entrypoints. Rust provides generic evaluation, bounded canonical-byte transport,
physical scheduling, and exact-base atomic commit mechanics; it owns no game or
Clause construct meaning. Materialization consumes admitted deltas as a checked
projection. Wasm exposes bounded byte arrays and physical handles only.

The typed BJS shell maps canvas-scoped physical input to immutable observation
frames and immutable render observations to Three.js. Admission-free frames
pin exact Run, Activation, producing Step, and Observation identities plus
optional unchanged `Wbase`; a frame projected from an admitted boundary also
pins its exact StateRevision. The render observation is not itself admitted. The
shell owns no movement integration, gravity, collision, jump, groundedness,
candidate admission, clock policy, or state mutation. The public shell already
has terminal disposal, canvas-scoped keyboard ownership, and honest static-
sample labeling. Integration still requires the process-identity frame
envelope, bounded preallocated pools/transport, and controlled no-allocation
frame path.

The renderer successor preallocates bounded instance pools, transport buffers,
Wasm memory, active-frontier, continuation, and trace capacity; updates
transforms in place; and disposes through an explicit process before wrapper
reclaim. After initialization, the Clause/Wasm/adapter-controlled frame path
performs no allocation, `memory.grow`, whole-frame clone, global scan,
observable destructor/finalizer work, or unbounded teardown. Foreign calls
retain declared contracts and
stronger browser-wide claims require instrumentation. Freshness follows exact
declared process causality and optional admitted-revision ancestry, never host
callback order.

Bounded pre-ABI candidates already exist but remain non-integrable. After the
process ABI is accepted, continue or reconcile them within these disjoint scopes:

- Clause-owned mechanic laws and semantic vectors:
  `clause:test-vectors/jump-arena/**`;
- construct-blind native process runtime:
  `clause:crates/clause-runtime/src/process/**` and
  `clause:crates/clause-runtime/tests/process_runtime.rs`;
- bounded canonical-byte Wasm adapter:
  `clause:crates/clause-wasm/**`; and
- passive typed shell: `clause:browser/jump-arena-shell/**`.

The rejected Rust `clause:crates/clause-substrate/src/transition.rs` and the
unreviewed/rejected jump-law bases are evidence only, not accepted artifacts or
lane scopes.
One integration owner adds shared crate exports, workspace/package manifests,
the final browser entrypoint, and cross-lane parity fixtures after the four
candidates are accepted.

Prove a playable 3D mechanic whose movement, jump, no-double-jump, gravity,
landing, collision, and render projection are Clause-owned. Then add one second
substantial mechanic by changing Clause alone.

**Exit evidence:** native/Wasm canonical boundary parity; exact process pins and
optional revision pins where applicable; scan/indexed observation parity;
candidate immutability before admission; effect-stage honesty; passive renderer
boundary; and zero semantic-name switches in Rust or BJS.

## Phase 7 — Relational recovery, systems breadth, and release

**Status:** Not started.

Prove that activation-scoped pure result relations, explicitly admitted
RelationExtensions and assertions, causal edges, continuations, evidence, and
occurrence-exact supports remain relationally queryable without treating every
relation row as execution or pure observation as admission. Phase 4 has already
proved direct calls, registers, packed layouts, erasure, and native/Wasm
specialization; this phase extends the accepted physical IR to actor loops,
async continuations, indexes, database queries, JavaScript/browser artifacts,
FFI packages, modules, and later GPU kernels where useful.

A production hot path must not route every reduction through a generic graph
engine. Translation validation and the declared observable contract remain
universal; physical allocation and schedule do not.

**Exit evidence:** replacement Gates 0–7 pass contiguously; independent
reproduction covers the complete pure/local and process/effect slices plus the
browser and non-game product proofs; exact published objects, usable
modules/packages/tooling, measured agent development loops and product
performance, and consolidated documentation contain no superseded semantic
authority.

## Completion standard

A roadmap item is complete as supported or admitted capability only when its
authoritative representation, identity rules, diagnostics, canonical encoding
where applicable, executable behavior, negative cases, and narrow exit proof
land together. A Lean evaluation without a kernel-checked package-bound proof
proves no Clause admission. A Rust, Wasm, or browser result without process
traceability and declared parity proves no Clause meaning.

Never remove working capability before a tested successor exists. Every in-tree
consumer migrates before removal. Once migration is complete, superseded source,
tests, docs, fixtures, generated artifacts, and consumers leave the live tree in
the same change.
