# Clause Architecture Assurance

> **Status:** Current implementation boundary and release contract.
>
> **Authority:** Derived and non-semantic. The
> [foundation](foundation.md) alone governs meaning, the
> [syntax](syntax.md) governs canonical source, and the
> [roadmap](roadmap.md) governs implementation status and order. This document
> turns their architecture boundaries into a release decision; it may not add an
> ontology, source form, or milestone.

<!-- clause-architecture-gate:v2 -->
<!-- milestone:M4:public-base:af9a0b9952f42f95851b47a071d9efb01a5fda0f -->
<!-- milestone:M5:public-base:23786abdb26c47638d819eea400555b0446a5451 -->

## Decision

The accepted target is process-first underneath and relation-first at the
surface:

```text
RawTriple  = [Term, Term, Term]
Term       = Atom | RawTriple
Clause     = contextual typed judgment over a Term
Run        = judged carry-through to outcome, trace, and candidate successor
Admission  = the only operation making a successor authoritative
```

The admitted judgment graph is the program at rest. Run is the program in
motion. A trace describes a Run but is never the Run itself.

This mechanism is accepted as a constitutional hypothesis, not as an
implemented fact. The [adoption spike](adoption-spike.md) must pass before a
product migration begins. The current implementation remains the exact parity
oracle until each selected behavior moves under a new `ClauseSemanticsId` with
before/after evidence.

A release candidate is architecture-acceptable only when
`bin/architecture-gate FULL_GIT_OBJECT_ID M<N>` passes from a clean worktree at
that exact commit and the roadmap exit proof for the selected milestone also
passes. Unknown, incomplete, dirty, tampered, or mismatched evidence fails
closed. Public-base markers admit inherited evidence only; they do not make a
later milestone implemented.

## Constitutional implementation split

The foundation governs the host-neutral Clause Core contract; that contract is
sovereign over its implementations. The accepted bootstrap architecture assigns
three implementation roles without turning any host into a second semantic
authority:

```text
canonical Clause Core package
          |                    |
          v                    v
Lean constitutional       Rust physical engine
checker and reference     persistence, indexes, runtime,
Run semantics             FFI, and optimized backends
          \                    /
           \------ parity ----/
                    |
                    v
        accepted Clause revision or exact rejection
```

Later, Clause-authored elaborators, macros, refactors, planners, and compiler
drivers propose the same package to the same acceptance boundary.

The package is the typed transport envelope defined by the foundation, not a
new semantic substance, serialized Lean value, or Rust struct layout. Its
disjoint sections carry the exact Clause objects, declared observables, and
separately scoped evidence required by each gate. Only the foundation-defined
checked Program payload contributes to `ProgramSnapshotId`; package
certificates, source maps, strategies, traces, caches, and physical evidence do
not enter that identity unless an explicit authored judgment makes their
semantic content part of the snapshot. Clause-native certificate data may
cross the wire; Lean proof terms remain local evidence bound to the exact
canonical package bytes and decoded sections. One versioned codec and one
corpus of canonical vectors define exchange. Lean and Rust may use private
indexes and intern handles after decoding, but neither may add an uncheckable
semantic field or side channel.

Lean receives a generic model of Clause's own Terms, judgments, Runs, and
admission rules. Clause constructs are not Lean `Syntax` kinds, `Expr`
constructors, type classes, or one inductive variant per language feature.
Lean's kernel checks Lean declarations and proof terms, not arbitrary Clause
graphs directly; therefore the Clause decoder, object-language definitions,
certificate proposition, and theorem connecting an accepted certificate to
Clause validity form one small audited bridge. A compiled Boolean or successful
reference execution is an oracle, not admission evidence, unless accompanied
by a kernel-checked certificate for the corresponding Clause proposition.

The reference Run semantics is a relation capable of representing total,
partial, nondeterministic, streaming, reactive, and effectful modes. Fuelled
total interpreters may execute bounded specimens, but Lean host recursion may
not decide Clause partiality or silently redefine a mode. Rust consumes the
same package for fast algorithms and physical realization. It may select
arrays, arenas, indexes, native code, Wasm, JavaScript, or foreign interfaces;
it may not decide a binder, identity, transition, effect occurrence, or
language category that the package and checker cannot express.

The compiler middle moves into Clause in this order when stable: schemas and
relation modes; elaboration and macro rules; obligation construction,
diagnostics, queries, and refactors; planner and projection policy; then
compiler orchestration and selected checking or lowering. The governing test
is that an ordinary new abstraction changes Clause data, not a Lean or Rust
feature taxonomy. No additional primary compiler host is planned; OCaml has no
primary compiler-host role in the accepted bootstrap architecture.

### Lean constitutional trust profile

The first checker is admitted only under this profile:

- pin and hash the exact Lean source, toolchain, and imported `.olean` artifacts
  used to produce every constitutional result. Use `trustLevel = 0` for newly
  added declarations, while recognizing that it does not recheck imported
  bodies; compute the transitive constitutional dependency closure, reject any
  reachable `unsafe` or `partial` declaration, and replay every reachable
  safe/total declaration into a fresh kernel environment from the pinned
  artifacts before acceptance;
- keep constitutional definitions and proofs safe and total; represent Clause
  partiality, divergence, reactivity, effects, and bounds in the object model,
  not with Lean `partial`, `unsafe`, foreign, or compiler replacement paths;
- never enable skipped kernel type checking, accept elaboration recovery,
  `sorry`/`sorryAx`, failed-declaration fallback axioms, or an asynchronous
  preliminary environment; acceptance waits for the checked environment;
- reject compiler-trust proof bridges such as `native_decide`, native reduction,
  execution of or reliance on `implemented_by`/`extern` implementations, or a
  compiled Boolean in the constitutional proof closure; an `extern` attribute
  does not by itself invalidate a kernel-checked definition body;
- audit the transitive axiom closure against an explicit Clause policy. Any
  permitted logical foundations, including choices about `propext`,
  `Quot.sound`, or `Classical.choice`, are named rather than inherited
  accidentally;
- bind every accepted theorem to the exact canonical package bytes, semantics
  epoch, and decoded value, rejecting alternate or noncanonical encodings; and
- use `leanchecker` or an equivalent declaration replay for that safe/total
  closure while recognizing that Lean's replay skips unsafe/partial constants,
  shares the kernel, and is not an independent verifier.

Logical-certificate trust and executable-runtime trust remain separate. Lean's
own runtime may contain foreign or compiler-specific machinery outside the
certificate closure; production Rust and generated targets remain pinned and
differentially checked rather than being misdescribed as kernel-proved. A later
small independent Clause-core checker or translation validator may reduce this
trust further without changing the canonical contract.

## Current implementation mapping

The live compiler at the baseline for this decision is
`4aea6c898f3eec2fe4058d578f491eec008d7f9a`. It crosses an explicit checked
snapshot boundary while retaining the semantic-v10 / Revision-v6 bridge:

```text
frontend::parse
  -> frontend::Program
  -> elaborate under ElaborationContext
  -> ProgramSnapshotCandidate + separate SourceMap
  -> validate(candidate)
  -> ValidationResult { ProgramSnapshot }
  -> explicit legacy checked-payload bridge
  -> CompiledProgram
  -> kernel::Revision { RevisionLineage, kernel::Model }
  -> RuntimeProgramRevision { ProgramRevisionId, ClauseSemanticsId, Revision-v6 }
  -> RuntimeSession / StateRevision / generated projections
```

These are current code facts, not final ontology:

| Current implementation | Current job | Accepted destination |
| --- | --- | --- |
| `frontend::Program` | parsed host-owned AST | lossless CST/source occurrences feeding canonical Term elaboration |
| `ElaborationContext` | caller-selected root scope and designation inputs | explicit typed context for Clause-judgment Runs |
| `ProgramSnapshotCandidate` | identity-free candidate holding duplicate-preserving semantic atoms | candidate judgment graph; it gains no authority until admission |
| `ValidationResult` | checked `ProgramSnapshot` from one comprehensive kernel validation | exact obligations and admissible candidate output of a validation Run |
| `kernel::RelationalContent` | irreducible n-ary named-role value | checked/indexed view over recursive Terms and role judgments, preserving all named-role guarantees |
| `CompiledProgram` | legacy aggregate of revisions, requests, journeys, designations, and SourceMap | admitted ProgramRevision plus separately typed source, request, and strategy contexts |
| `kernel::Model` | checked semantic payload container | ProgramSnapshot judgment graph; not a model-theoretic Model |
| `kernel::Revision` | envelope hashing lineage plus Model payload | separate ProgramSnapshot, ProgramChangeOccurrence, and ProgramRevision identities |
| designation table | source mapping plus ID-retention helpers | explicit Designation judgments and source occurrence evidence |
| `derive::saturate` | bounded reference closure | declared total/bounded relation modes and derived-support Runs |
| `RuntimeSession` | runtime-v3 history pinned to ProgramRevision, policy, semantics, and start occurrence | retained identity boundary under the universal Run law |
| `StateRevision` | causal runtime-v3 node pinned to session, predecessor, occurrence, and payload | admitted State successor; trace remains separate from transition occurrence |
| generated Rust/JavaScript and host objects | specialized execution and rendering projections | checked materializations traceable to strategies and exact admitted input |

Current interning keys, vector positions, Rust discriminants, wire row IDs, and
host addresses remain physical mechanics. None may become Term, occurrence,
entity, or revision identity merely because it is convenient.

Clause owns its future Term codec, equality contracts, occurrence history,
persistence interfaces, semantic graph, and compilation semantics. No external
store or older project supplies those definitions. A database may implement a
checked persistence interface; it never becomes semantic authority.

Atom equality contracts are declarative, total over their admitted domain,
canonical, versioned, and committed by `ClauseSemanticsId`; an opaque host
callback is not an equality contract. Structural equality is indexed by
universe and semantics epoch. Identity Atoms are opaque leaves, and graph
serialization never content-hashes recursively through the neighborhoods they
name. Reload is cycle-aware, terminating, and fail-closed on foreign epochs,
scopes, kinds, anchors, or lineage.

## Target pipeline

The target specifies each semantics-affecting arrow by a typed Run relation
while retaining a transient lossless source representation:

```text
read(SourceUnit)
  -> LosslessCST + SourceMap

elaborate(LosslessCST, ElaborationContext)
  -> candidate Terms, occurrences, and Clause judgments

encode(candidate judgment graph, derivation proposal)
  -> canonical Clause Core package

check(package)
  -> kernel-checked certificate bound to exact package or failed obligations

admit(package, bound certificate, ProgramAdmissionContext)
  -> ProgramChangeOccurrence + ProgramRevision

lower(ProgramRevision, target and physical contracts)
  -> checked strategy graph + materialized artifacts

execute(ProgramRevision, RuntimePolicy, SessionStartOccurrence)
  -> RuntimeSession
  -> Run traces and admitted StateRevision successors
```

In kernel notation:

```text
Γ ⊢ t clause : T @ M

Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩

Γ ⊢ Γ̂ admissible
───────────────────
admit(Γ, Γ̂) = Γ′
```

Pure evaluation, query, and rejection preserve `Γ`. Elaboration, macros,
refactors, compilation, runtime transitions, and agent edits produce differently
typed candidates; only their target admission boundary may make them
authoritative. No broad optional `ProgramContext` may let source, namespace,
authority, policy, Program, revision, or runtime identities impersonate one
another.

This notation does not require a durable Run object, trace, or revision for
every pure arrow. A direct function or specialized instruction may implement a
context-preserving Run with no nominal allocation. Occurrence identity and
durable evidence are materialized only when observation, replay, or authority
requires them. Streaming and reactive modes expose typed continuations;
nondeterministic modes expose declared finite choices or ordered streams;
failure, suspension, and exhausted bounds are explicit outcomes.

State and external effects cross separate commit boundaries. A transition Run
stages a State candidate plus intents; admission accepts that State and its
authorized intents; separately identified effect Runs then perform external
acts and produce attempt/receipt evidence for later admission. An explicitly
declared transactional adapter may strengthen this ordering. Evidence-admission
failure never claims to roll back an external act that already occurred.

## Target semantic graph

The first persistent compiler-owned semantic representation contains:

- recursive structural Terms and their explicit equality contracts;
- distinct source, assertion, Run, binder, definition, entity, and revision
  identities where continuity matters;
- Clause judgments carrying types, modalities, relation modes, and authority;
- scope, binding, macro origin, phase, and quotation relationships;
- schemas, stable named roles, capabilities, laws, invariants, and strategies;
- derivations, exact supports, obligations, proofs, and explanations;
- Program and State history; and
- trace Terms linked to—but never substituted for—the occurrences they
  describe.

`RawTriple`, Term, Clause judgment, and judgment occurrence are distinct. Raw
Triples have structural equality and no mandatory nominal `ClauseId`.
Explicit identity atoms anchor cycles and independently continuing entities.
Higher-arity structural values contain every named role in their complete
recursive Term; unique relation instances use explicit identity anchors.
Provisional neighborhoods cannot leak into an admitted Program or State.

The target graph is not a generic triple database and need not be executed by
graph scans. It is universal semantic meaning with aggressively specialized
physical realization.

## Compiler ownership

- The reader owns bytes, lines, indentation, delimiters, comments, literals,
  source spans, and incomplete edits without deciding semantic kind.
- The parser owns a transient lossless CST and exact grouping. It does not own
  binding, identity, domain meaning, or authority.
- Elaboration deterministically selects declared readings from explicit syntax
  before child domain checking, then resolves Designations and proposes Terms,
  occurrences, focus, named roles, and Clause judgments.
- The generic Clause Core checker owns structural, relational, modal, binding,
  effect, and bounded-execution obligations. Lean is its first constitutional
  implementation, not the owner of those categories.
- Run machinery owns selected execution mode, outcome, candidate
  successor, and trace production.
- Admission owns Program or State lineage, base revision, constitutive
  occurrence, authority, policy, and immutable successor creation.
- The runtime owns RuntimeSession activity, transition occurrences, State
  candidates, effect boundaries, and receipts under one ProgramRevision.
- Persistence stores canonical Clause-owned Terms, judgments, occurrences, and
  history without inventing identity or truth.
- Rust owns compact storage, indexes, FFI, production runtime, and optimized
  backends after the Clause Core boundary. Its semantic proposal machinery is
  untrusted until checked and parity-gated.
- Generators and hosts consume checked strategies and canonical artifacts. They
  may optimize but cannot reproduce or extend Clause semantics privately.

## Derived representations and physical freedom

Lossless CSTs, source graphs, packed role maps, indexes, support tables,
e-graphs, optimizer graphs, control/dataflow IRs, heap layouts, registers,
database plans, native code, Wasm, JavaScript, and browser objects are allowed
derived representations.

Each semantics-affecting representation needs a checked and explainable
refinement path to the exact admitted graph. A derived form may not privately
decide binding, typing, effects, identity, source meaning, or observable
behavior. Truly unobservable allocation and scheduling decisions may remain
backend-private. ABI, layout promises, overflow, floating-point mode, effect
ordering, determinism, synchronization, cancellation, durability, failure,
resource, and latency contracts must remain explicit strategy or evidence
judgments when promised.

Performance-sensitive State and target execution must use compiled indexes,
exact incremental changes, and specialized layouts. The bounded interpreter
may remain an oracle; a generic relation scan or Triple interpreter may not be
the ordinary hot path. The adoption spike measures this by adding unrelated
graph content and requiring selected indexed/generated operations to stay
bounded by their declared dependency closure rather than the whole graph.

## Architecture constitution

| ID | Required invariant | Reject the candidate when |
| --- | --- | --- |
| A1 | **Process, structure, meaning, and authority stay distinct.** Run is activity; Term is held structure; Clause is contextual judgment; admission makes successors authoritative. | A Term executes or asserts itself, a trace substitutes for an act, or any Run mutates authority without admission. |
| A2 | **One recursive Term algebra.** Compound holdable structure is `RawTriple = [Term, Term, Term]`; raw equality is structural, epoch-indexed, and governed by declarative canonical Atom contracts. | Every Triple receives mandatory nominal identity, a host handle leaks as identity, equality delegates to opaque host code, or canonical reload is undefined/nonterminating. |
| A3 | **Explicit continuity.** Referents, binders, definitions, occurrences, runtime entities, effects, sessions, and revisions receive nominal identity only where their independent continuity matters. | Equality collapses occurrences or entities, or spelling, source position, payload, or replay order guesses continuity. |
| A4 | **Clause is judgment.** Type, modality, authority, relation modes, and executability are contextual judgments over Terms. | A raw structural shape is automatically treated as proposition, assertion, value, effect, or execution. |
| A5 | **Named roles survive three-slot lowering.** Higher-arity meaning retains stable RoleIds, role types, cardinality, completeness, atomic admission, and source-order independence. | A role is dropped, inferred positionally after elaboration, or equal partial roots conflate distinct n-ary values. |
| A6 | **Honest modes and outcomes.** Total, productive, bounded, partial, nondeterministic, streaming, and reactive modes state result cardinality, continuation, fairness/ordering, failure, cancellation, and resource obligations. | Clause claims a universal halting decision, silently diverges under a total mode, invents one verdict for an open stream, or calls expected reactivity an error. |
| A7 | **One semantic authority.** Schemas, readings, typing, completion, language extension, and failed obligations are inspectable Clause-authored judgments checked by a generic kernel. | A source construct requires a private Lean/Rust feature case, host enum, opaque callback/dispatch table, validator, formatter, refactor, analysis, or dependency rule. |
| A8 | **Act and trace never collapse.** State Runs stage intents; admitted intents authorize separately identified external effect Runs; attempts, receipts, observations, and later evidence admission remain distinct. | A transition fabricates a receipt, rejection claims to roll back an external act, replay repeats an effect, or a receipt becomes truth. |
| A9 | **Intensional identity and history.** Snapshot identity commits to semantics epoch plus canonical checked payload; history commits separately to Program, predecessor, snapshot, and genuine change occurrence. | Equivalence collapses independent history, later evidence mutates a revision, or a hash silently changes meaning across epochs. |
| A10 | **Physical freedom under traceable contracts.** Targets specialize aggressively while preserving exact semantics and every declared observable or nonfunctional contract. | A target, store, retry, fallback, exception, cache, layout, or host accident changes meaning privately. |
| A11 | **Exact source projection.** Deterministic reading lookup precedes child semantics; indentation supplies only declared focus; parsing and printing obey the canonical bidirectional laws. | Layout invents a domain edge, child meaning selects or regroups its parent, ambiguity is guessed, or round-trip loses binding, identity, or meaning. |
| A12 | **Fail closed with visible obligations.** Ambiguity, incomplete neighborhoods, tamper, exhausted bounds, and missing capabilities remain explicit failures. | Partial work is certified exact, malformed content leaks into an admitted revision, or a milestone is closed by prose alone. |

## Identity and parity gate

Before changing the current representation, executable oracles must distinguish:

- two constructions of one structural Term from two occurrences of that Term;
- expression Term from evaluated value and from denotational equivalence;
- equal source or assertion content under distinct occurrences;
- two equal-looking transfers under distinct event/entity identities;
- trace replay from effect re-execution;
- local rename retention from delete-and-create;
- equal snapshot payloads reached through different parents or change
  occurrences;
- additional attestations that leave snapshot and revision identity unchanged;
- equal State payload under different sessions, transitions, or policies; and
- ProgramRef, lifecycle, and deployment changes that do not mutate snapshots.

Migration changes the canonical representation and therefore requires an
explicit new `ClauseSemanticsId`. Existing semantic-v10 / Revision-v6 IDs are
never reinterpreted or rewritten merely to make a test pass. Every selected
M1–M7 behavior needs before/after identity, result, proof, runtime, wire, and
generated-output parity.

## Adoption boundary

The [adoption spike](adoption-spike.md) is the next mechanism decision. It must
carry pure evaluation, binding and closure, algebraic data and exhaustive
matching, structurally complete and nominal n-ary cases, recursive derivation,
State/effect Runs, hygienic macros, canonical projection, and Clause-owned
persistence/reload through one generic kernel.

The final gate freezes both the Lean generic checker/model and Rust's semantic
proposal boundary, then adds a new construct combining binding and effects
entirely through Clause-authored schemas, readings, modes, and transformations.
Those definitions must be inspectable and executed by generic Clause machinery;
an opaque “generic” callback or per-construct dispatch table fails the gate.
Generic checker defects require an explicit refreeze and full rerun; optimized
backend implementation remains allowed behind a checked strategy. If either
host must learn construct-specific meaning, the graph is an AST in witness
protection and the mechanism fails.

A pass authorizes only a bounded parity-preserving migration proposal. It does
not prove readability, performance, systems coverage, or maintenance economics
at product scale. Source-ergonomics, large-graph incrementality, and matched
systems/JavaScript performance gates remain mandatory before the migration can
be called successful.

## Milestone ratchet

The architecture gate currently protects the implemented semantic-v10 /
Revision-v6 line through M6. These remain migration oracles; they do not prove
the process-first Term kernel.

| Milestone | Additional architecture evidence |
| --- | --- |
| M1–M3 | One ReferentId domain; distinct content/occurrence/Judgment structures; exact named roles; deterministic source projection; strict reload; bounded recursive evaluation and source-deleted generated-Rust parity. |
| M4 | Query holes remain scoped PatternIds; recursion, projection cardinality, ordering, proof/support provenance, law-versus-derive authority, exact input Revision, bounds, and generated parity remain explicit. |
| M5 | Migration reports every source inference and proves source/designation to stable identity to exact successor continuity. |
| M6 | RuntimeSession and StateRevision replay binds exact ProgramRevision, RuntimePolicy, semantics epoch, start and transition occurrences, predecessor history, deltas, and ordered inputs; additions and retractions use compiled dependency/support indexes. |
| M7 | Effect intent, authorization, attempt, receipt, observation, and admission remain separate; generated JavaScript contains no shadow semantics; real target claims require matched evidence. |
| M8 | One live ontology and source grammar remain; compatibility parsers, inferred declaration kinds, stale fixtures, and shadow consumers are absent. |

<!-- obligation:source-migration:fulfilled:M5:test=m5_migration -->
<!-- obligation:incremental-runtime-trace:fulfilled:M6:test=m6_replay -->
<!-- obligation:specialized-target-effect-trace:pending:M7 -->
<!-- obligation:single-live-surface:pending:M8 -->

The gate refuses a milestone while its obligation remains pending. Closing one
requires the narrow executable proof and marker change in the same commit;
prose or a renamed marker cannot make the gate green.

## Current architecture gap

The current implementation has a conventional AST, an irreducible n-ary
`RelationalContent`, Rust-owned semantic variants, no canonical Clause Core
package or Lean checker, no graph-homoiconic macro system, and no universal Run
interface. It already proves valuable oracles for
identity, duplicate occurrences, named roles, recursive derivation and support,
queries, explanations, causal revisions, runtime sessions and State history,
effects, generated Rust/JavaScript, and a bounded real-browser checkpoint.

Those strengths are migration evidence, not reasons to pretend the new kernel
already exists. The next action is the adoption spike, not another broad
ontology migration. The roadmap alone records its status.

## Running the existing gate

From a clean candidate worktree at exact HEAD:

```sh
candidate=$(git rev-parse --verify 'HEAD^{commit}')
bin/architecture-gate "$candidate"
bin/architecture-gate "$candidate" M6
bin/architecture-gate --self-test
```

The self-test attacks marker authority, milestone parsing, shadow identity,
severe deferral, pending obligations, and full-object comparison. It does not
prove the new kernel or rerun every milestone feature. The [roadmap](roadmap.md)
names broader completion evidence.
