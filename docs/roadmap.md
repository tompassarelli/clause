# Clause Roadmap

> **Status:** Current.
>
> **Authority:** Sole authority for implementation status, dependency order,
> milestone scope, and exit evidence. The [foundation](foundation.md) governs
> semantics, the [syntax](syntax.md) governs canonical source, and the
> [architecture](architecture.md) governs release boundaries.

## Current position

Clause has a substantial implemented kernel and runtime under the migration-era
`kernel::Model` / `kernel::Revision` vocabulary. Capability milestones M1–M6
are implemented and protected by tests. M7 has bounded effect, RenderPlan, and
host checkpoints but is not complete.

The semantic review is also complete: Program, ProgramSnapshot,
ProgramChangeOccurrence, ProgramRevision, RuntimeSession, and StateRevision now
have distinct accepted meanings, while Model is reserved for interpretation
and satisfaction. The first snapshot-identity seam is implemented; the full
ontology is not fully implemented yet.

The identity/parity oracle and Model/Revision/context census prerequisites are
complete. The first constitutional snapshot-identity slice is implemented and
the remaining history/runtime split is in progress:

```text
identity/parity oracles -----------+
                                    +--> Program identity and history split
Model/Revision/context census -----+             |
                                                  v
                                      typed compilation and runtime
                                                  |
                                                  v
                                      canonical syntax migration
                                                  |
                                                  v
                                      one live surface and M7 closure
```

The public documentation authority reset is complete in this line so the
accepted design and the executable gap can no longer be confused while that
migration proceeds.

## Status summary

| Work | Status | Exact boundary |
| --- | --- | --- |
| Semantic ontology | Accepted | [Foundation](foundation.md); implementation pending |
| Documentation authority reset | Complete | Six public documents, one authority per fact, no historical spec in the live tree |
| M1–M6 capability line | Implemented | Current semantic-v10 / Revision-v6 representation and milestone tests |
| M7 effect and JavaScript vertical | Partial | Effect traces, grounded RenderPlan projection, frozen RenderPlan ESM, provisional host contracts |
| Program identity/history migration | In progress | ClauseSemanticsId, typed Program identities/history, canonical preimages, separate SourceMap/ElaborationContext, and single-pass ProgramSnapshotCandidate validation exist; an explicit bridge still feeds checked payloads to Revision-v6 while admission/runtime migration remains pending |
| Canonical agent-first syntax | Accepted design | Parser migration pending; legacy executable surface is isolated in `syntax.md` |
| M8 single live surface | Pending | No compatibility grammar or stale consumer may remain at exit |

## Product direction

Clause is one general-purpose relational programming system. It is not a
semantic-modeling DSL that hands ordinary programming to another language, an
object language with prettier property syntax, or a separate game ontology.

The durable Program contains exact checked relational meaning. A
ProgramSnapshot carries Referents, named-role content, AssertionOccurrences,
Judgments authored as program content, laws, derivation authorization,
invariants, goals, transition contracts, and semantic policy. A
ProgramRevision records causal program history; StateRevision records runtime
history. Source, storage, generated code, and host layouts remain replaceable
projections.

The compiler may lower functional relations to fields, columns, component
arrays, indexes, or specialized code. Physical strategy is free when exact
semantic identity, provenance, results, and bounds remain unchanged.

## Protected behavior

The constitutional migration must preserve these already demonstrated
capabilities unless an explicit semantic decision retires one:

- one addressable ReferentId protocol and exact named RoleIds;
- n-ary recursive RelationalContent;
- separate AssertionOccurrences and Judgments, including duplicate source acts;
- stable relation identity independent of surface voice and focus;
- bounded positive recursive derivation with every independent support;
- inert laws until separately authorized for derivation;
- exact query cardinality, deterministic ordering, and canonical output bytes;
- explanation, support, semantic diff, and bounded intervention results;
- strict canonical wire reload and exact predecessor/delta lineage;
- incremental runtime additions and occurrence-exact retractions;
- deterministic event replay and state diff;
- source-deleted generated-Rust parity; and
- effect-evidence and RenderPlan boundaries already proved by bounded M7 tests.

Current spellings and overloaded type names are not protected merely because
those tests use them. Every syntax or identity migration must compare semantic
IDs, canonical bytes, results, proofs, state histories, and generated outputs;
it may not rewrite expectations to bless an accidental change.

## Implemented capability milestones

These statuses describe public executable capability checkpoints. They do not
claim that the newly accepted ontology or canonical syntax is already live.

## M1 — Grounding, membership, inferred layout, and focus

**Status:** Implemented.

The current frontend grounds Referents, accepts explicit `∈` membership and
colon definitions, infers enumeration/shape/Model blocks, and expands focused
forms. Focused and expanded assertions preserve exact role-labelled content and
independent occurrences.

The inferred declaration and bare-child rules are now legacy migration
behavior. Canonical replacements are explicit `referent`, `enum`, `shape`,
and child-named focus edges.

**Evidence:** surface lowering, focus lowering, top-level context, exact
membership, and raw-`::` rejection tests.

## M2 — Role-labelled relation contracts

**Status:** Implemented.

The checked core and frontend support exact unary, binary, and n-ary relation
identities, stable named roles, recursive participant terms, lookup orientation,
and cardinality. Both ceremonial `RelationShape` and inferred compact schemas
remain executable today.

The canonical replacement is one explicit `relation` declaration with
`reads`, `subject`, and fully written `mode given ... yields ...` clauses.

**Evidence:** compact relation schema, relation resolution, focus, and
role-diagnostic tests.

## M3 — Recursive terms, structural values, and pure definitions

**Status:** Implemented.

Clause implements recursive relation applications, conventional arithmetic and
comparison forms, finite scalar values, tuples, homogeneous sequences,
identity-labelled products, closed pure definitions, bounded evaluation, and
source-deleted generated Rust.

**Evidence:** recursive-term, structural-frontend, pure-definition,
pure-computation, and generated-evaluation tests.

## M4 — Holes, requests, laws, proof, and intervention

**Status:** Implemented.

The current surface implements named correlated holes, fresh anonymous holes,
selection cardinalities, `any`, `find`, `why`, `prevent`, `achieve`, and
`diff`. Universal laws remain inert until a separate `derive` authorization
produces an operational rule with governing law, authority, and scope.

The proof/support engine retains exact input Revision, independent asserted
supports, recursive correlation, finite bounds, canonical order, and
source-deleted generated parity.

The canonical request grammar will remove `find`, naked query inference, and
bare `?`, and will require explicit `where`, `using`, and ordering envelopes.

**Evidence:** M4 selection and rule-to-proof suites plus request, derivation,
explanation, intervention, and semantic-diff tests.

## M5 — Source migration and current Revision parity

**Status:** Implemented.

The current migration path reports source inference and preserves current
Revision identity, canonical wire, exact deltas, hospital results, and
source-deleted Rust output across its supported legacy-to-current rewrite.

This proof is an oracle for the next migration, not proof that the accepted
ProgramSnapshot/ProgramRevision split exists. The new migration must extend the
same discipline to ClauseSemanticsId, nominal identity allocation,
ProgramChangeOccurrence, and explicit designation retention.

**Evidence:** `m5_migration` and strict Revision wire tests.

## M6 — State, events, incremental transitions, and replay

**Status:** Implemented.

The runtime-v3 boundary implements content-derived RuntimePolicyId, immutable
RuntimeSessionId, causal StateRevisionId, authored legacy events,
transaction-wide matching and guards, deterministic conflict rejection, exact
deltas, session-scoped state diff, occurrence-pinned replay, strict typed
reload, and source-deleted generated Rust through the same runtime API.

Additions use compiled relation/rule dependency indexes and retractions use
occurrence-root reverse support indexes. The state hot path does not delegate
to the generic reference closure.

Runtime construction requires a real ProgramRevision whose checked snapshot
matches the frozen Revision-v6 semantic oracle. Session identity binds the
ProgramRevision, complete RuntimePolicy, semantics epoch, and caller-allocated
start occurrence; every state binds that session, predecessor, exact causal
occurrence, and state payload. Equal logical state reached through distinct
occurrences does not collapse, and old runtime envelopes fail closed.

**Evidence:** `m6_replay`, runtime unit tests, canonical state reload, conflict,
tamper, diff, and generated replay checks.

## M7 — Effects, JavaScript, rendering, and one-coin proof

**Status:** Partial; not implemented as a milestone.

Implemented checkpoints:

- EffectRequest, authorization, attempt, receipt, observation, and
  `clause-effect-trace-v2` records remain distinct; the lineage names the exact
  ProgramRevision and post-commit StateRevision.
- A typed projector reads only supported grounded StateRevision content and
  emits exact F32×2 RenderItems under an explicit relation/role/shape spec.
- RenderPlan has canonical `clause-render-plan-v2` bytes bound to exact
  ProgramRevision and StateRevision identities.
- Rust emits import-free frozen ESM containing exact-state RenderPlan lookups;
  a Bun source-deletion test compares its JSON bytes with Rust.
- The provisional host validates plans before applying mesh positions,
  requires caller-owned event and transition occurrence allocation,
  mechanically forwards declared event/effect requests, and owns browser
  lifecycle rather than Clause semantics.
- A source-deleted real-Chrome acceptance emits one sealed empty-payload
  runtime-v3 transition, matches exact Rust session and RenderPlan bytes, and
  observes actual pinned Three.js `WebGLRenderer` execution. This is a bounded
  compiler checkpoint, not a general JavaScript runtime or M7 completion.

Still required for M7:

- ratified authored scene/effect/capability syntax;
- a general generated live-JavaScript runtime for arbitrary and repeated
  transitions rather than one sealed specialized edge;
- generalized runtime/result replay and effect validation without host-authored
  semantics;
- real-browser and Three.js evidence beyond the bounded single-transition
  checkpoint;
- source maps and role/focus diagnostics through generated JavaScript;
- the full one-coin movement, collision, collection, score, replay, render, and
  effect-receipt vertical; and
- matched reference/target evidence for specialized hot-path performance.

JavaScript development and tests use Bun. Node, npm, npx, pnpm, and Yarn are
not fallback tools when Bun can perform the task.

**Exit proof:** one canonical Clause program compiles to generated JavaScript,
runs through the real host and Three.js, replays byte-exactly, returns honest
effect/render evidence, preserves source/role diagnostics, and demonstrates a
specialized physical path without shadow semantic logic.

## M8 — One ontology and one source surface

**Status:** Pending.

M8 removes migration-era ontology and grammar after exact parity:

- current `kernel::Model` and combined `kernel::Revision` responsibilities are
  migrated to the accepted Program layers;
- source, designation, validation, admission, and runtime contexts have distinct
  checked types;
- inferred declaration kinds, implicit membership children, ceremonial
  declaration paths, shorthand synonyms, and naked query inference are absent;
- every in-tree source, example, test fixture, diagnostic, generator, and host
  consumer uses the one canonical representation; and
- deleted compatibility code leaves no tombstone, shim, warning-only parser,
  stale test, or shadow consumer.

**Exit proof:** canonical hospital and one-coin programs preserve intended
semantic identities and exact results through native and generated targets;
repository-wide absence checks find no retired ontology or source grammar.

## Active constitutional migration

### 1. Identity and parity oracle suite

**Status:** Complete.

Capture exact before-migration identities and canonical bytes for Referents,
roles, RelationalContent, duplicate AssertionOccurrences, immutable Judgments,
laws and derivation authorization, current semantic payloads and lineage,
runtime roots/successors, and claimed generated outputs.

Add explicit acceptance cases for attestation neutrality, rename retention,
duplicate occurrence non-collapse, semantics-epoch sensitivity, distinct
change occurrences, runtime-policy/session isolation, and current Disposition
derived from immutable Judgments.

### 2. Model/Revision/context consumer census

**Status:** Complete.

Classify every current use of Model, ModelId/context, and Revision as exactly
one future axis: Program lineage, snapshot payload, source, namespace, scope,
authority, policy, law/event/request owner, change occurrence, history node,
runtime session, state history, or generated-artifact input.

No broad compatibility alias may conceal an unclassified consumer.

### 3. Constitutional identity layer

**Status:** In progress. The snapshot identity seam is implemented; designation
allocation and consumer migration remain.

Introduce `ClauseSemanticsId`, Program/ProgramId,
ProgramSnapshot/ProgramSnapshotId, globally opaque ReferentIds, and explicit
Designation allocation/retention. Define canonical preimages and migrate every
in-tree consumer in coherent slices.

### 4. Program history separation

Separate ProgramChangeOccurrence from ProgramRevision. Attach attestations and
AdmissionJudgments independently. Keep ProgramRef navigational; represent
lifecycle decisions and deployments as immutable authority/target records with
derived current views.

### 5. Typed compilation and runtime boundaries

SourceMap and ElaborationContext are split, and elaboration now produces an
identity-free ProgramSnapshotCandidate consumed by single-pass validation.
Validation has no contextual inputs, so no ceremonial ValidationContext is
present. RuntimeSession and StateRevision now bind exact program, policy,
semantics, session-start, and transition-occurrence identities through
RuntimeProgramRevision and runtime-v3 wire. Add AdmissionContext only when
admission accepts Program lineage, base revision, authority, policy, and
occurrence allocation and returns those typed history artifacts directly.
Program migration creates explicit evidence and a new session.

### 6. Canonical surface rebuild

Reassess private syntax experiments against the corrected identity boundary.
Preserve only changes that pass the parity oracles. Implement explicit
declaration heads, explicit focus edges, one relation/law/delta/request grammar,
prefix binders, normalized layout/trivia, comments, and canonical names without
retaining compatibility syntax.

## Compiler ownership

- The reader owns bytes, lines, indentation, delimiters, comments, literals,
  and source spans without deciding semantic kind.
- The parser owns explicit source constructs and recovery, not identity,
  authority, or domain-relation inference from layout.
- Elaboration owns exact Designation resolution and role-labelled checked
  structure under ElaborationContext.
- Validation owns structural, relational, modal, and bounded-admission checks.
- Admission owns Program lineage, base revision, constitutive change
  occurrence, authority, and policy.
- The kernel owns canonical semantic identities and content.
- The runtime owns exact sessions, transitions, state successors, and effect
  evidence under one ProgramRevision.
- Generators and hosts consume canonical artifacts; they do not reproduce
  Clause semantics independently.

## Completion standard

A roadmap item is complete only when its source, checked representation,
identity, canonical wire where applicable, diagnostics, migration, runtime or
target behavior, and narrow executable exit proof land together. Documentation
specimens are not implementation evidence. A green parser test does not prove
semantic preservation, and a green target demo does not prove source identity
or provenance.

The first next action is the parallel identity/parity oracle and
Model/Revision/context census. Their joined result, not another terminology
discussion, determines the first implementation slice.
