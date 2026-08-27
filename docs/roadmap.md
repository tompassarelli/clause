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

The semantic foundation is now process-first: recursive Terms hold
distinctions, Clause is contextual judgment, Run is the dynamic relation, and
admission alone makes a successor authoritative. Program, ProgramSnapshot,
ProgramChangeOccurrence, ProgramRevision, RuntimeSession, and StateRevision
retain distinct meanings; Model remains reserved for interpretation and
satisfaction.

The current implementation does not embody that Term/Clause/Run kernel. The
identity/parity oracle, consumer census, snapshot boundary, runtime-v3 identity,
and existing capabilities are inputs to the adoption experiment. No broad
representation migration proceeds until the host-freeze spike decides whether
the mechanism is real:

```text
host-neutral Clause Core package
              |
       Lean checker/reference model
              +-------- observable parity -------- Rust physical engine
              |
       frozen generic host boundary
              |
       Clause-authored compiler middle
```

This is the accepted bootstrap order, not an implementation claim. The
foundation remains the semantic authority and its host-neutral Clause Core
contract governs every implementation; Lean checks certificates and provides
reference Run semantics, Rust retains persistence/runtime/FFI/backend
responsibility, and stable proposal machinery moves into Clause. OCaml has no
primary architecture role. Lean may be rejected by the spike without weakening
the semantic mission or licensing Rust to become the ontology.

```text
current parity oracles ------------+
                                     +--> process-first adoption spike
consumer and identity census -------+                |
                                                     v
                                                stop / go
                                                     |
                         +---------------------------+------------------+
                         v                                              v
               bounded kernel migration                  reject mechanism,
                         |                                preserve mission
                         v
               canonical source + targets
```

The public documentation authority reset is complete. The foundation is the
only semantic canon; the spike and evidence ledger are subordinate and cannot
silently change it.

## Status summary

| Work | Status | Exact boundary |
| --- | --- | --- |
| Process-first semantic foundation | Accepted hypothesis | [Foundation](foundation.md); implementation unproved |
| Constitutional implementation split | Accepted bootstrap plan | One canonical Clause Core package; Lean checker/reference semantics; Rust physical engine; eventual Clause-authored compiler middle |
| Lean constitutional checker and reference Run model | Pending; first spike tranche | Host-neutral encoding, checked certificates, explicit trust profile, and canonical vectors; no Clause surface parser required initially |
| Adoption spike | Pending; next mechanism gate | [Constitutional profile, eight gates, parity, host freeze, and falsifiers](adoption-spike.md) |
| Post-spike product gates | Pending; blocked on mechanism pass | Source ergonomics, large-graph incrementality, and matched systems/JavaScript performance before migration success |
| Documentation authority reset | Complete | One owner per public fact; spike/evidence are subordinate, not competing canons |
| M1–M6 capability line | Implemented | Current semantic-v10 / Revision-v6 representation and milestone tests |
| M7 effect and JavaScript vertical | Partial | Effect traces, grounded RenderPlan projection, frozen RenderPlan ESM, provisional host contracts |
| Program identity/history migration | Implemented slices preserved; further representation work gated | ClauseSemanticsId, typed Program identities/history, canonical preimages, separate SourceMap/ElaborationContext, and single-pass ProgramSnapshotCandidate validation exist; the Revision-v6 bridge remains an oracle until the adoption decision |
| Canonical agent-first syntax | Accepted design | Parser migration pending; legacy executable surface is isolated in `syntax.md` |
| M8 single live surface | Pending | No compatibility grammar or stale consumer may remain at exit |

## Product direction

Clause is one general-purpose relational programming system. It is process-first
underneath and relation-first at the authoring surface. It is not a semantic
modeling DSL that hands ordinary programming to another language, an object
language with prettier property syntax, a generic Triple database, or a
separate game ontology.

The target ProgramSnapshot is an exact checked judgment graph over recursive
Terms. An admitted ProgramRevision makes one snapshot authoritative in a
Program lineage. The snapshot includes Clause judgments, explicit identities
and occurrences, named-role schemas, laws, derivation authorization,
invariants, goals, transition contracts, capabilities, and semantic policy.
ProgramRevision records causal program history; StateRevision records runtime
history. Source, persistence layout, generated code, and host objects remain
replaceable projections. A trace records a Run without becoming the Run.

The compiler may lower functional relations to fields, columns, component
arrays, indexes, or specialized code. Physical strategy is free when exact
semantic identity, provenance, results, and bounds remain unchanged.

## Protected behavior

The constitutional migration must preserve these already demonstrated
capabilities unless an explicit semantic decision retires one:

- one addressable ReferentId protocol and exact named RoleIds;
- n-ary named-role meaning, whether represented by current
  `RelationalContent` or a future checked Term view;
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

## Process-first adoption spike

**Status:** Pending; this is the next mechanism decision.

The spike begins with a canonical Clause Core package and a minimal Lean 4
constitutional model of `Atom`, `Term`, context, judgment, mode, Run, delta,
trace, admission, and revision. It then makes the retained Rust engine consume
the same package. Lean owns neither source syntax nor semantic categories;
Rust owns neither judgment nor admission. Both are implementations of the
Clause-owned contract.

The [adoption spike](adoption-spike.md) must prove one generic kernel across:

1. pure evaluation with expression Term distinct from result;
2. binder identity, closure capture, hygiene, and canonical projection;
3. algebraic data and exhaustive matching;
4. structurally complete and explicitly nominal n-ary cases;
5. recursive derivation with completed, nondeterministic/streaming, and
   productive/bounded outcomes under honest modes;
6. State/effect Runs with admitted intent, external act, trace, and evidence
   admission separated;
7. a typed binder-introducing macro; and
8. a host-freeze language extension combining binding and effects.

The spike also owns Clause's Term codec, declarative versioned equality policy,
well-founded identity allocation, cycle-aware persistence/reload, measurable
specialization, and negative cases for NaN/signed zero, equal-looking distinct
events, effect trace replay, leaked intern handles, opaque callbacks, and
universal-halting overclaim.

**Exit proof:** the Lean checker satisfies its pinned trust profile; Lean and
Rust agree across all eight gates on every declared observable and
nonfunctional contract over one canonical vector corpus; all required negative
fixtures pass on one exact Clause Core contract; and the host-freeze extension
remains inspectable Clause data while adding no construct-specific Lean/Rust
semantic, callback/dispatch, validator, formatter, refactor, analysis, or
dependency case.

**Failure result:** reject the Term-kernel mechanism, preserve the
general-purpose mission and current behavioral oracles, and record the exact
forcing counterexample. Do not weaken a gate or quietly retain the graph as an
interchange costume over private host semantics.

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

## Migration inputs and gated sequence

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

### 3. Process-first adoption spike

**Status:** Pending; next.

Implement the bounded experiment in [adoption-spike.md](adoption-spike.md):
freeze the canonical Clause Core codec and vectors; build the Lean
constitutional checker/reference semantics; make Rust consume the same
package; establish observable parity; then freeze both generic host boundaries
for gate 8 and return one stop/go result. No current oracle is retired and no
canonical identity is reinterpreted during the experiment.

### 4. Mechanism stop/go

**Status:** Blocked on the spike.

On pass, authorize one bounded parity-preserving migration proposal under a new
`ClauseSemanticsId`. On failure, reject the mechanism and retain the mission,
protected behavior, and exact forcing counterexample. This decision cannot be
made by prose alone.

### 5. Constitutional identity layer

**Status:** Implemented slices preserved; further representation migration is
gated by the stop/go decision. The snapshot identity seam exists; designation
allocation and consumer migration remain.

Introduce `ClauseSemanticsId`, Program/ProgramId,
ProgramSnapshot/ProgramSnapshotId, globally opaque ReferentIds, and explicit
Designation allocation/retention. Define canonical preimages and migrate every
in-tree consumer in coherent slices.

### 6. Program history separation

Separate ProgramChangeOccurrence from ProgramRevision. Attach attestations and
AdmissionJudgments independently. Keep ProgramRef navigational; represent
lifecycle decisions and deployments as immutable authority/target records with
derived current views.

### 7. Typed compilation and runtime boundaries

SourceMap and ElaborationContext are split, and elaboration now produces an
identity-free ProgramSnapshotCandidate consumed by single-pass validation.
Validation has no contextual inputs, so no ceremonial ValidationContext is
present. RuntimeSession and StateRevision now bind exact program, policy,
semantics, session-start, and transition-occurrence identities through
RuntimeProgramRevision and runtime-v3 wire. Add ProgramAdmissionContext only
when admission accepts Program lineage, base revision, authority, policy, and
occurrence allocation and returns those typed history artifacts directly.
Program migration creates explicit evidence and a new session.

### 8. Canonical surface rebuild

Reassess private syntax experiments against the corrected identity boundary.
Preserve only changes that pass the parity oracles. Implement explicit
declaration heads, explicit focus edges, one relation/law/delta/request grammar,
prefix binders, normalized layout/trivia, comments, and canonical names without
retaining compatibility syntax.

### 9. Product-scale adoption gates

**Status:** Blocked on a passing spike and bounded migration candidate.

Before the Term-kernel migration is called successful, measure three independent
claims on representative programs:

- canonical source ergonomics and comprehension against the readability target;
- large-graph incremental dependency precision, update cost, and memory; and
- matched native/systems and JavaScript target performance with specialized
  plans and no generic Triple hot path.

These gates may reject or revise a passing mechanism. A spike pass proves
semantic extensibility at bounded scale, not product viability.

## Compiler ownership

- The reader owns bytes, lines, indentation, delimiters, comments, literals,
  and source spans without deciding semantic kind.
- The parser owns a transient lossless CST and recovery, not identity, binding,
  authority, or domain-relation inference from layout.
- Elaboration owns exact Designation resolution and proposes recursive Terms,
  occurrences, focus, named roles, and Clause judgments.
- The generic Clause Core checker owns structural, relational, binding, modal,
  capability, and bounded-execution obligations. Lean is its first checked
  implementation and reference semantics, not the owner of those rules.
- Run owns selected execution mode, outcome or continuation, candidate
  successor, and trace production.
- Admission owns Program lineage, base revision, constitutive change
  occurrence, authority, and policy.
- The runtime owns exact sessions, transition occurrences, State candidates,
  effect boundaries, and receipts under one ProgramRevision.
- Persistence stores Clause-owned canonical Terms, judgments, occurrences, and
  history without supplying equality, identity, truth, or authority.
- Rust owns compact persistence, indexes, FFI, production runtime, and
  optimized backends after the canonical package boundary. Rust semantic
  proposals remain untrusted until checked and parity-gated.
- Clause progressively owns schemas, modes, elaboration, macros, obligation
  construction, diagnostics, refactors, planning, projection, and compiler
  orchestration without changing either frozen generic host for ordinary
  language extensions.
- Generators and hosts consume canonical artifacts; they do not reproduce
  Clause semantics independently.

## Completion standard

A roadmap item is complete only when its source, checked representation,
identity, canonical wire where applicable, diagnostics, migration, runtime or
target behavior, and narrow executable exit proof land together. Documentation
specimens are not implementation evidence. A green parser test does not prove
semantic preservation, and a green target demo does not prove source identity
or provenance. A successful Lean execution without a kernel-checked,
axiom-policy-compliant certificate proves no Clause admission, and agreement
inside one host does not substitute for the required cross-host parity.

The first next action is the process-first adoption spike. Its host-freeze gate
and negative fixtures—not another terminology discussion—determine whether a
Term-kernel migration is allowed.
