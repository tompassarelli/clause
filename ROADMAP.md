# Clause Roadmap

Clause is becoming one general-purpose, relation-centered programming system.
It will not remain a semantic-modeling DSL that hands ordinary programming to
another language, and it will not grow a separate game language beside the
semantic core.

The current implementation is a real but deliberately narrower foundation. It
seals typed Models as immutable Revisions; navigates them with `find`, `why`,
`prevent`, `achieve`, and `diff`; and projects exact requests and Revisions into
standalone Rust. The milestones below extend that foundation into recursive
values, state, transitions, effects, JavaScript and Three.js, measured Wasm
specialization, a complete authoring loop, and eventually a real North
vertical.

This file distinguishes current truth from committed direction. A milestone,
specimen, or candidate syntax is not implemented merely because it appears
here.

## Strategy authority

This tracked roadmap is the durable normative authority; recovering or
executing the plan never depends on an external Documents path. It
default-adopts the substantive direction supplied in the strategy packet at
`~/Documents/clause/clause-strategy-packet`, which remains provenance/input.
The integration request identifies that packet with manifest digest
`4819feedee3ef98561d64def8a0eafd9bd5778557d57a22cdd59d9b8a7744f66`.
No derivation recipe accompanied that aggregate identity, so this roadmap does
not present it as an independently reproducible file-manifest algorithm.

The directly reproducible SHA-256 identities of the four unique proposal
inputs are:

| Packet input | SHA-256 |
| --- | --- |
| `00-executive-brief.md` | `6362750f50e90d9973bf8e626e79092a4478492431bbc6faf65343157cd57bcc` |
| `01-surface-language-charter.md` | `fcba0c43c5b9c780c3099c2a7d1be4b0075e877e4377345a304ea209d44d0afe` |
| `02-game-capable-programming-system-strategy.md` | `97181cb91749a88f152bbe00d0645b6b19c234d4447b877d2c516666862fbcfb` |
| `03-language-specimen.md` | `5055157daf8b96141667a5c64950bd0ef46c8abaf2140aba05de2b302a45fc9a` |

`CLAUSE-STRATEGY-COMPLETE.md` is a duplicate combined projection of those
numbered documents, not a fifth proposal authority.

The packet's examples are acceptance inputs. They freeze semantic questions
that an implementation must answer, not their exact punctuation. Deviations
from the default-adopt posture require one of three things: a demonstrated
internal contradiction, a dependency or feasibility result that changes the
safe order, or evidence that a proposal violates correctness. The conflict
ledger near the end records the present deviations explicitly.

## Product thesis

For the current semantic core:

> **The sealed Revision is the program. Source and generated code are
> projections of it.**

For the complete programming system:

> **The sealed ProgramRevision is the program. State advances by exact deltas.
> Effects produce receipts. Source and target code are projections.**

A Clause program is navigable:

| Direction | Operation | Question |
| --- | --- | --- |
| Forward | `find` | What follows? |
| Backward | `why` | What supports it? |
| Across revisions | `diff` | What changed semantically? |
| Toward absence | `prevent` | What admissible withdrawal would stop it? |
| Toward presence | `achieve` | What admissible addition would establish it? |
| Across time | transitions and replay | How did admitted state evolve? |
| Across the boundary | effects and receipts | What was requested and what happened? |
| Outward | materialization | What executable realizes these semantics? |

The singular promise is:

> Write the program once. Move forward to its consequences, backward to their
> reasons, across revisions and time to semantic change, and counterfactually
> to certified revisions that would make the answer different—then project the
> same sealed authority into ordinary executable code.

This is a stronger category than “excellent Datalog.” Values and pure
definitions do ordinary computation; propositions and laws retain relational
meaning; transitions advance admitted state; effects cross an explicit
capability boundary; and the compiler selects indexes, schedules, layouts,
solvers, and targets without making those strategies the meaning of the
program.

## Whole-program authority

The current `Revision` is retained as the proven immutable model capability.
The general-purpose system separates authorities that a live application
cannot honestly collapse:

| Authority | Meaning |
| --- | --- |
| `ProgramRevision` | The immutable, content-addressed checked program: types, relations, definitions, laws, transition and effect declarations, dependencies, target requirements/planning declarations, capability and asset requirements, and selected initial content. |
| `ModelRevision` | Immutable admitted domain or content propositions, checked against one exact ProgramRevision/schema. This is the future whole-system placement of today's sealed semantic `Revision`; migration must preserve its canonical identity guarantees. |
| `StateRevision` | The logically immutable runtime state at a simulation or transaction boundary, produced from an exact predecessor and Delta and bound to its exact ProgramRevision and relevant ModelRevision/schema. |
| `RuntimeSession` | One ProgramRevision, initial ModelRevision and StateRevision, one tick policy, an ordered event/input stream, transition results, and capability realizations/effect receipts. |

Program edits, domain-content edits, runtime movement, and external observations
therefore have different identities. A realized TargetPlan or generated
artifact is keyed by ProgramRevision, target, and selected strategy; it remains
a projection rather than program authority. A runtime may implement state with
transient mutation, delta chains, checkpoints, and selective trace retention;
it may not let those physical choices change the logical StateRevision. Not
every frame must be durably sealed forever.

Today's `Revision` fuses checked program declarations with admitted Model
content. The authority split must migrate that representation and every in-tree
consumer atomically; it does not pass through aliases, shims, or two competing
wire meanings. Before public wire spellings are frozen, M0 performs an explicit
terminology and identity audit against Beagle and Store's existing `Model` and
`ModelRevision` vocabulary. The four conceptual authorities above are fixed;
the audit prevents Clause from casually minting incompatible shared wire nouns.

The following distinctions remain load-bearing throughout the system:

| Category | Checked meaning |
| --- | --- |
| Value / Term | Data or a pure result, including recursively composed domain terms. |
| Proposition | A typed role-labelled relation application that may be admitted, derived, queried, or explained. |
| Definition | An oriented computation or term-form meaning. |
| Law | Timeless derivation from propositions to a proposition. |
| Query | An explicit bounded operation such as `find`, `why`, `prevent`, or `achieve`. |
| Transition | A current-state and event match producing an exact successor Delta. |
| Effect | An explicit request to an external capability. |
| Receipt / Observation | Evidence of an attempted effect or a later observation; neither is silently equated with intent. |

One recursive surface and one module, type, identity, package, diagnostic, and
target system serve these categories. They lower through distinct checked IR
strata rather than pretending to be one universal node or one universal
solver.

## Language constitution

These commitments are frozen unless a milestone produces falsifying evidence:

- Relations have stable semantic identities independent of wording, mode,
  strategy, generated procedure, and ordinary surface orientation.
- Relations retain typed, named roles. Ordinary clauses infer a declared shape
  locally; an explicit role-labelled application remains the canonical escape
  hatch.
- A declared role accepts a recursively parsed value, term, or proposition of
  the expected category. Recursive uniformity does not require universal
  prefix notation.
- Meaning remains separate from mode, mode from strategy, and strategy from
  generated procedure.
- Inference is permitted only when one lexical, structural, and type-correct
  interpretation survives; imports cannot silently redirect existing source.
- Definitions, equality, laws, queries, transitions, and effects remain
  visibly distinguishable at their semantic fault lines.
- Search, nondeterminism, bounds, authority changes, temporal change, foreign
  interoperation, and externally observable order are never hidden for
  aesthetic compression.
- Exact deltas, stable identities, canonical formatting, source-to-IR
  provenance, reproducible projections, and explicit completeness claims are
  protected surfaces.
- `world` remains retired. Static admitted content is a Model; runtime content
  is State; the sealed executable authority is ProgramRevision.
- JavaScript ESM is the first browser and Three.js host. Rust remains a native
  projection and parity target. Wasm is a measured specialization target, not
  the first host.
- A game/entity component organization is an optimized library and
  compiler-recognized pattern, not the universal Clause ontology.

The surface pays rent: every visible token must resolve ambiguity or
communicate type, role, category, scope, cardinality, time, search, authority,
or effect. This is semantic compression, not code golf or probabilistic natural
language parsing.

When public modes promise cardinality, the exact lattice is:

| Contract | Result obligation |
| --- | --- |
| `one` | exactly one result; uniqueness and totality are enforced |
| `maybe` | zero or one result; uniqueness is enforced |
| `some` | one or more results; non-emptiness is enforced |
| `many` | zero or more results |

Canonical words versus glyphs remain an M0 prototype choice. A mode that may
not terminate, may exceed its declared bound, or lacks an admitted finite
strategy is rejected rather than silently becoming universal search.

## Current foundation

At the roadmap baseline, Clause implements:

- typed Models with stable entity and relation identities;
- role-labelled n-ary relations and declared known/sought modes;
- asserted clauses and bounded positive recursive laws;
- immutable, content-derived Revisions and exact successor Deltas;
- recursive closure queries;
- bounded complete support enumeration and proof rendering;
- bounded `one minimal` and `all minimal` prevention and achievement over an
  explicit finite intervention basis;
- assertion, entailment, proof, and support diff;
- authored request ordering and canonical result bytes;
- standalone generated Rust whose results remain equal after authoring source
  deletion.

The current surface is a flat, typed sentence-shape grammar whose participant
slots consume atomic surface terms. Current Clause does **not** yet provide the
recursive value language, arbitrary I/O, networking, concurrency, effects,
runtime state machine, JavaScript target, or application tooling required to
author a complete North service, browser game, server, or terminal UI without
a host-language boundary.

Current bounded support and intervention behavior is the compatibility oracle
for the lineage work below. A selected proof, even a canonical proof DAG, is
not by itself a representation of every alternative support.

## Program dependency spine

```text
current semantic foundation
    |
    +--> M0 constitution and corpus
            |
            v
         M1 recursive surface kernel
            |
            v
         M2 pure value core + pure JavaScript ESM
            |
            v
         M3 StateRevision, events, and transitions
            |
            v
         M4 effects, capabilities, assets, and effectful FFI
            |
            v
         M5 dual real verticals: North + Three.js
            |
            v
         M6 physical specialization and measured Wasm
            |
            v
         M7 product-grade authoring loop

current semantic foundation --> L0..L5 lineage and certified intervention lane
                                  |      |                 |
                                  +------M3 state----------+
                                         M5 live North/game debugging
                                         M7 time travel
```

Each milestone is a vertical through reader, elaboration, checked IR,
canonical identity, diagnostics, materialization, and the smallest executable
acceptance program. A parser-only or runtime-only fragment does not close a
milestone. The lineage lane proceeds in parallel, preserving public results
while replacing selected-proof coupling with one shared exact derivability
representation.

Three proof ladders keep those milestones recursive rather than horizontal:

- **Hospital — semantic exactness.** The existing six-direction semantic-time-
  machine journey is permanent from M1 onward. Every affected milestone retains
  its canonical results, exact revisions, bounds, and source-deleted projection
  parity.
- **North — general-purpose systems programming.** A pure application canary
  begins in M2, gains state and lifecycle semantics in M3, crosses explicit
  capabilities in M4, and becomes one real operator-used application slice in
  M5.
- **Orbit — real-time browser/game programming.** Its surface starts in the M0
  corpus, its pure simulation runs in M2/M3, and M5 closes the full Three.js
  game with replay and semantic debugging.

Each applicable ladder advances end to end through surface, checked IR,
runtime, materialization, and evidence. Each milestone and independently
publishable child states its capability delta, non-goals, protected surfaces,
first falsifier, exit proof, and pivot/stop condition before mutation.

## M0 — Constitution, corpus, and executable surface prototypes

**Depends on:** the current foundation.

Deliver:

- a fifty-plus example corpus spanning hospital and dependency models,
  recursive arithmetic, records and sums, collections and pattern equations,
  compiler/data work, errors, async asset loading, deterministic game state,
  rendering, foreign JavaScript, tests, and a representative North program;
- a surface ledger for every construct: semantic distinction, candidate
  syntax, inferable information, ambiguity, canonical/debug rendering,
  diagnostics, checked node, and runtime implication;
- corpus measurements for token rent, repeated scaffolding, punctuation and
  nesting, ambiguity and qualification frequency, formatter stability,
  source/role diagnostic precision, agent repair success, read-aloud quality,
  parse/elaboration cost, import stability, and usability of the explicit
  escape hatch;
- the category and whole-program authority model in this roadmap;
- a canonical formatter prototype and an explicit role-map debug form;
- side-by-side executable prototypes for the intentionally fluid choices
  listed below.

M0 also produces a bounded prior-art decision brief, with concrete reuse,
departure, and falsifier statements rather than a bibliography: Mercury for
modes and determinism; Verse and Flix for functional/relational composition;
Koka for typed effects and handlers; Rhombus for structural extensibility;
Unison for semantic identity independent of mutable names; ECS/data-oriented
engines for semantic-to-physical layout; and Elm-style deterministic update
architectures for event/state/render separation. These systems pressure-test
Clause without becoming a menu to combine indiscriminately.

**Exit proof:** at least fifty representative examples parse deterministically,
format/parse to the same checked trees, and give every canonical token a written
semantic justification. Adding an unrelated import either preserves the old
elaboration or produces a finite ambiguity diagnostic with an exact repair.

## M1 — Recursive surface kernel

**Depends on:** M0 corpus rulings.

Deliver:

- a small fixed lexer/layout kernel for identifiers, literals, logical
  variables, annotations, grouping, blocks, and true boundary words;
- scoped, inspectable typed mixfix shapes with stable identity, literal spine,
  named roles, category, precedence group, associativity, public modes, and
  canonical surface;
- recursive candidate parsing at every role position, followed by
  type/category-directed elimination rather than heuristic ranking;
- explicit grouping and role-map escape hatches, stable source spans,
  deterministic ambiguity diagnostics, and structural macro expansion rather
  than token-string rewriting;
- unary, binary, and genuinely n-ary relations; typed identity families;
  checked finite range distribution; and explicit binders wherever zip versus
  Cartesian expansion is not unique;
- golden lowering of current hospital, impact, and catalog programs to their
  existing role-labelled clauses and canonical results.

**Exit proof:** nested arithmetic, nested property phrases, hospital routes,
dependency impact, and an n-ary transfer all use one recursive surface tree;
all current semantic examples retain canonical result parity; parse → format →
parse is stable; imports cannot silently change a selected declaration.

## M2 — Pure general-purpose value core and pure ESM projection

**Depends on:** M1.

Deliver:

- integers, floats, booleans, strings, bytes, units and explicit conversions;
- tuples, records, algebraic sums, arrays, vectors, maps, sets, sequences,
  options/results, and parametric types;
- pure and mutually recursive definitions, immutable local bindings,
  equation/pattern definitions, guards, pattern matching, comprehensions,
  iterators, folds, collection pipelines, modules, imports, visibility, and
  interfaces;
- type-directed numeric/operator resolution without converting ordinary values
  into stored propositions;
- tests, assertions, and deterministic error values as normal language/library
  facilities rather than host-only escape hatches;
- one named core numeric and collection library whose abstractions exercise the
  same modules, generics, patterns, iterators, and target lowering available to
  application authors;
- typed deterministic JavaScript ESM generation for pure code, source maps,
  browser and Node pure runners, and optional TypeScript declarations;
- continued Rust/native projection and cross-target canonical parity for
  semantics both targets share.

Effectful JavaScript package calls are expressly not part of M2. Generating
pure ESM does not require crossing the capability boundary.

**Exit proof:** a headless particle/physics simulation and a nontrivial data
transformation are authored entirely in Clause and run as generated JavaScript
without a Rust or JavaScript implementation of their ordinary computation. A
pure, bounded North data/decision canary uses the same value, module, and ESM or
native machinery rather than encoding ordinary algorithms as propositions.

## M3 — State, events, transitions, and replay

**Depends on:** M2. It precedes effectful FFI so a modeled transition cannot be
confused with an external observation.

Deliver:

- distinct ProgramRevision, ModelRevision, StateRevision, and RuntimeSession
  identity and encoding contracts;
- keyed and extensional state relations, exact predecessor-bound Deltas, event
  values, fixed simulation ticks, and deterministic clocks supplied as inputs;
- distinct wall-clock observations, render-frame time, deterministic simulation
  ticks, scheduled game time, event occurrence time, and proposition validity
  intervals; a game profile uses a fixed simulation step with optional render
  interpolation;
- normalization of browser/host callbacks into timestamped events or sampled
  inputs under a declared policy; callbacks never mutate semantic state
  directly;
- transition matching from one state snapshot to one committed successor,
  post-state derivation, canonical state diff, event logging, checkpoints, and
  replay;
- deterministic conflict rejection, declared commutative merge, or named
  phases; source order must not accidentally decide otherwise declarative
  state conflicts;
- compiler-displayed inferred withdrawals for any checked keyed replacement;
- bounded aggregations and comprehensions with exact empty-set, completeness,
  and incremental-maintenance contracts.

M3 contains the executable phase-policy gate. It compares one-snapshot commit,
a small explicit phase pipeline, and a dependency-derived acyclic phase graph
on the same movement/collision/derivation corpus. The selected policy must show
cycle and multi-write diagnostics and a canonical transition transaction; the
specimen deliberately does not settle it earlier.

That transaction selects current-state matches, computes pure candidate
Deltas, detects or resolves conflicts, commits one successor StateRevision,
and derives post-state views. Authorized effects occur afterward in M4 and
their receipts become explicit later inputs; they never retroactively become
premises of the state transition that requested them.

`one` and `maybe` mode/cardinality words or glyphs do not by themselves license
implicit replacement. Replacement becomes available only after the checker
enforces the key's uniqueness; an exactly-one contract additionally requires
enforced totality. Until then, source uses an explicit signed Delta.

**Exit proof:** the same ProgramRevision, initial StateRevision, fixed tick
policy, and ordered event log repeatedly produce byte-identical canonical state
results. Conflicting successor writes fail deterministically, and replay plus
state diff explains the chosen change. A North lifecycle/state-machine canary
uses the same authority split and replay contract as the headless Orbit
simulation.

## M4 — Effects, capabilities, assets, and effectful JavaScript FFI

**Depends on:** M3.

Deliver:

- explicit EffectIntent, Authorization, Attempt, EffectReceipt, and Observation
  distinctions, with errors and cancellation represented rather than smuggled
  into propositions;
- explicit effect sites and source-visible observable order inside handlers;
  pure dependencies and independent transitions may still be scheduled by the
  compiler;
- checked effect declarations, opaque foreign/resource types, deterministic
  acquire/use/release ownership, leak diagnostics, async completion events,
  clocks, and replayable randomness;
- a typed future/task or typed completion-event contract, with structured
  ownership, cancellation, timeouts, failure propagation, and restart
  disposition; an async host callback never suspends a logical proof or mutates
  StateRevision behind the transition runtime;
- deterministic generator state as transition input/state for replay, while
  external entropy acquisition is an effect whose receipt is sealed into the
  RuntimeSession inputs;
- npm package imports, stable foreign-value ABI, exception-to-result/receipt
  conversion, effectful ESM calls, browser and Node hosts, and source-mapped
  failures;
- logical asset identities, content/provenance where reproducibility matters,
  import/transform strategies, preload policy, fallbacks, and target asset
  materialization;
- storage, network, input, audio, worker, and other capabilities as typed
  adapters, added only as a vertical requires them.

Each effect site, capability declaration, and observable sequence remains
explicit. The aggregate capability manifest may be deterministically derived
from the checked reachable program and sealed into ProgramRevision; authors do
not maintain a second fallible list unless a public restriction or target
contract requires one.

**Exit proof:** a Clause program loads an asset, consumes normalized input,
calls a typed JavaScript package, uses an audio/storage test adapter, and
records exact outcomes without hidden mutation, uncaught host exceptions, or
claiming that intent proves realization. The North ladder crosses one real
CLI/API or Store/provider capability through the same request/receipt boundary
before its complete M5 slice.

## M5 — Dual real vertical gate: North and Three.js

**Depends on:** M4 and the lineage lane through L2.

M5 has two parallel, independently publishable children. The general-purpose
claim and the one-language evidence gate require both; a game alone could hide
systems-programming gaps, while a control-plane application alone could hide
real-time and browser gaps.

### M5A — Real North application slice

Deliver one bounded but operationally real North subsystem primarily in Clause.
Choose the exact subsystem at milestone entry by its ability to exercise
application requirements: typed CLI or service boundaries, asynchronous I/O,
network or Store interaction, concurrency/cancellation, persistence and
restart, process/resource lifetime, errors, receipts, and semantic
explanation. It is neither a toy canary nor a big-bang North rewrite. Host code
is limited to declared capability adapters and cannot contain a shadow
implementation of the application behavior.

**Exit proof:** the subsystem is used in a real North operation, materialized
from one sealed ProgramRevision, survives its declared restart/failure cases,
records exact external outcomes, and answers a useful `why`, `diff`, or
counterfactual question about an actual decision. Its acceptance budget and
protected operational surfaces are fixed before implementation.

### M5B — Three.js adapter and complete game

Deliver:

- Three.js as an ordinary package/capability adapter rather than language
  syntax;
- stable Clause entity identity separate from mutable ResourceHandle and
  JavaScript Object3D identity;
- a pure StateRevision → RenderPlan projection and a reconciler that emits the
  deterministic canonical necessary scene diff and receipts, without claiming
  a globally minimum mutation plan unless a cost/order proof is defined;
- one Orbit/Asteroids-class game with input, continuous movement, collision,
  spawning/despawning, score, assets, audio, camera/scene graph, particles or
  comparable presentation, pause/restart, and deterministic replay;
- a headless route, source-mapped diagnostics, paused-state query console,
  relation/state inspector, and useful `why` and `diff` answers.

**Exit proof:** at least ninety percent of game-specific behavior is Clause
source; handwritten JavaScript is confined to declared adapters/runtime; the
same event log reproduces the same semantic state headlessly; the game meets a
declared frame budget after ordinary generated-code work; and several thousand
lines of canonical source pass the corpus “wince test,” not merely one demo
screen.

This is the only scheduled separate-language decision gate. One language
remains the commitment. The gate is evidence that the shared system works, not
standing permission to weaken that commitment. A split requires evidence from
the completed North and Orbit verticals that values, relations,
transitions, and effects cannot share types, modules, tooling, identity, and a
front end without lying. Performance alone first triggers representation and
strategy work. Any approved split requires an explicit roadmap amendment and
must preserve the shared semantic protocol and target/runtime infrastructure.

## M6 — Incremental physical specialization and measured Wasm

**Depends on:** M5 measurements.

Deliver:

- compiler-selected dense arrays, sparse sets, structures of arrays, hashes,
  bitsets, spatial indexes, incremental joins, and generated component access
  where declared modes and workloads justify them;
- semantic-source profiling for transition time, cardinality, join attempts,
  allocation, host calls, render reconciliation, and boundary cost;
- safe conflict-free transition batching and incremental derived relations;
- manual target-region Wasm for measured pure kernels, with batched bridges and
  the JavaScript host retaining browser callbacks, Three.js resources, assets,
  UI, and effects;
- JavaScript/Rust/Wasm semantic and replay parity for shared regions.

**Exit proof:** the complete game meets explicit frame, memory, and startup
budgets without source-level semantic compromise, and moving a checked pure
region between JavaScript and Wasm requires no Clause source rewrite.

## M7 — Product-grade authoring and semantic debugging

**Depends on:** M5; consumes M6 capabilities as they prove worthwhile and the
lineage lane through L5.

Deliver:

- canonical formatter and syntax-aware editor protocol; role/type hovers;
  explicit expansion, inferred-mode, chosen-strategy, and generated-code views;
- ProgramRevision hot-reload classification for implementation-only,
  state-compatible, derived-relation, asset-only, schema-extension,
  migration-required, capability-change, and target/runtime-ABI changes;
  hot state crosses revisions only through a checked
  `State under Program A -> State under Program B` migration, with the applied
  migration recorded;
- optional side-session replay validation before live swap,
  capability/ABI restart decisions, and safe resource transfer where possible;
- live relation/state inspection, transition conflict and effect traces,
  source-mapped target stacks, query console, time travel, branching, and
  cross-program/state diff;
- deterministic scenarios, pure RenderPlan tests, optional image snapshots,
  asset dependency tracking, packaging, deployment, and a version policy;
- configurable provenance retention: none, error-only, selected relations,
  debug session, or full deterministic trace.

**Exit proof:** a substantial game change can be formatted, diagnosed,
hot-reloaded or explicitly restarted, migrated, replayed, inspected, and
packaged from Clause tooling; the loop is measured as materially better than
the equivalent direct JavaScript/TypeScript workflow.

## Controlled semantic expansion after the verticals

The first complete North and Orbit verticals, not abstract ambition, decide the
next semantic breadth. Candidate expansions proceed in this order when a real
program earns them: typed finite domains and constraints/equality; stratified
negation with explicit closed-domain boundaries; then scoped e-graphs only for
declared equivalence domains. Multiple modes, stronger finiteness/termination
contracts, cost models, and target-specific strategies may deepen throughout.
No global equality saturation, universal inversion, or open-world negation is
smuggled into the core under this heading.

## L0–L5 — Shared lineage and certified-anytime interventions

This lane is compiler machinery, not a new public metaphysical primitive.
Externally, Clause continues to expose proofs, support frontiers, Deltas,
successor Revisions, and explicit request contracts.

For target proposition `P` in Revision `R`, the engine constructs a shared
lineage function `L_R(P)` over asserted clauses:

```text
AND  — every premise of one law application is required
OR   — any alternative law application or assertion is sufficient
```

Recursive laws may require a shared cyclic/fixed-point representation rather
than expanded proof trees. The same derivability object supports:

```text
find     evaluate lineage
why      project one proof or a complete minimal-support frontier
prevent  find cuts that make lineage false
achieve  satisfy lineage by admitted additions
diff     compare assertions, entailment, and support/lineage across revisions
```

A canonical Proof is one projection. It must never erase independent support
needed by `why all`, `prevent`, or support diff.

### L0 — Exact shared lineage

Build hash-consed AND/OR provenance with canonical asserted inputs, rule
applications, recursion/fixed-point handling, bounded construction, and exact
evaluation. Preserve all current query and generated-Rust bytes on the bounded
acceptance programs.

### L1 — Proof, support, and diff projections

Project one canonical proof without losing alternatives; enumerate
inclusion-minimal supports with an explicit completeness status; and compare
assertions, entailments, proofs, supports, and lineage structure between exact
Revisions. A support-preserving degradation remains visible even when the
target stays entailed.

### L2 — Exact finite intervention contracts

Support distinct requests:

```text
one minimal       one inclusion-minimal certified intervention
all minimal       the complete inclusion-minimal antichain
one minimum       a globally least-cardinality or least-cost intervention
up to n minimal   a certified partial stream under an explicit budget
```

`minimal` means no proper subset works. `minimum` means no qualifying result is
better under the declared ordering. They are never synonyms.

For a finite permitted withdrawal basis, `prevent one minimal` may withdraw the
whole basis, prove the target absent, and canonically restore each element when
absence survives. `achieve one minimal` uses the dual add-all and canonical
removal process. Each needs only a linear number of entailment checks over the
basis after feasibility is established; neither claims minimum cardinality or
complete enumeration.

Complete `prevent all minimal` may use lineage cuts or hitting-set enumeration.
Complete `achieve all minimal` is bounded abduction over an explicitly finite,
typed candidate basis. Every result includes its exact Delta, base and
successor Revision, admissibility result, changed entailment, minimality proof,
and resulting proof or absence certificate.

### L3 — Certified anytime synthesis

Separate four guarantees:

| Guarantee | Meaning |
| --- | --- |
| Soundness | Every returned intervention actually changes the target answer. |
| Minimality | No proper subset of a returned intervention works. |
| Optimality | No qualifying intervention is better under the declared ordering. |
| Completeness | Every qualifying intervention has been enumerated. |

Relax completeness, not correctness. A bounded search may return exact,
admissible, inclusion-minimal interventions while saying that global optimality
or frontier completeness is unproved. Results expose `verified`,
`inclusion-minimal`, `optimal` or `unproven`, and `complete` or `incomplete`.
They never call a sampled prefix “the frontier.”

Search is reproducible by deterministic fuel/expansion count, canonical tie
breaking, a content-addressed strategy identity, a seed derived from the
Revision/request/strategy, and a resumable continuation identity. Wall-clock
limits do not enter canonical semantics.

### L4 — Solver portfolios and speculative guidance

Use the strongest solver justified by proven structure:

- deterministic greedy minimization for one exact minimal result;
- graph reachability cuts where prevention is a cut problem;
- dynamic programming for suitable bounded acyclic laws;
- counterexample-guided implicit hitting sets for general prevention;
- SAT, MaxSAT, or branch-and-bound for bounded achievement and optimization;
- blocking constraints for alternative enumeration.

The prevention loop proposes a withdrawal, evaluates it exactly, and—when the
target survives—extracts a surviving proof that every later candidate must
hit. This avoids enumerating all proofs before useful work begins.

Heuristics, stochastic search, learned ranking, an LLM, or external agents may
order branches or propose semantic macro-Deltas. They may not erase a branch or
admit a result. Pruning is exact; heuristic influence is ordering. Multiple
workers may race and deduplicate content-addressed candidates.

> **Agents speculate. Clause certifies. Search may be plural and speculative;
> admission remains singular and exact.**

### L5 — Cross-revision, runtime, and projection closure

Generalize exact lineage and intervention certificates from ModelRevision to
paused StateRevision debugging where bounds permit. Make searches cacheable,
forkable, resumable, and replayable. Seal any live model/LLM proposal as an
explicit noncanonical guidance artifact before deterministic verification.
Generated Rust, JavaScript, and later Wasm reproduce the applicable canonical
judgments and certificates after source deletion.

**Lane exit proof:** on a bounded redundant dependency or hospital program,
Clause returns multiple independent proofs, all inclusion-minimal prevention
and achievement sets, exact successor Revisions, support-aware diffs, explicit
complete/incomplete and optimal/unproven statuses, resumable deterministic
partial search, and byte-equal standalone results. On a larger search, a
heuristic may improve arrival order but cannot change which candidates the
trusted verifier accepts.

## Surface prototype ledger

M0 decides these through parsing, formatting, diagnostic, lowering, and corpus
experiments—not screenshots:

| Question | Roadmap disposition |
| --- | --- |
| Role-hole notation | Prototype bare `role: Type` against a lightweight delimiter. Named typed roles are fixed; punctuation is not. |
| Typed identities | Prototype `Door 101`, a family separator, and a conventional explicit escape. Stable typed identity is fixed. |
| Cardinality spelling | Prototype `one`/`maybe`/`some`/`many`, glyphs, and debug aliases. Meaning and completeness contracts are fixed. |
| `=` and `+`/`-` | Equality versus oriented definition, arithmetic signs, and Delta admission/withdrawal create real overloading pressure. Their semantic distinctions are fixed; the glyph allocation is an M0 corpus-led decision. |
| Boundary nouns | Use `program`, `model`, and `state` for distinct authorities. `world` is not a candidate. |
| Law labels | Prototype optional `@label`, named blocks, and content identity. Labels may aid navigation but do not define semantic meaning. |
| Term definitions | Compare typed mixfix equations with a conservative function/signature escape hatch. Recursive composition is fixed. |
| Transition and phase surface | Prototype `~>`, a word form, and a small explicit phase pipeline. Snapshot, conflict, and successor semantics are fixed. |
| Precedence | Begin with explicit grouping; earn only a small deterministic lattice from the corpus. |
| Query grouping | Independent request forms and grouped model/state scope must lower to the same ordered requests. |
| Intervention basis wording | Prototype `changing:` against current `using:` while preserving the exact finite typed basis and request contract. |
| Explicit relation application | Keep one stable role-labelled debug/canonical escape; punctuation remains fluid. |
| Foreign result binding | Resolve with the effect system so handle acquisition cannot resemble a timeless pure definition. |
| Entity typing | Keep nominal subtype, tags, rows, and refined capabilities fluid until real APIs demand one. |

The specimen forms in the strategy packet are mandatory corpus members. They
are not frozen punctuation and must not be described as executable until they
pass their milestone gate.

## Cross-cutting acceptance gates

Every milestone preserves the gates it has already crossed:

1. **Recursive structurality:** every eligible role accepts a recursive checked
   form rather than one lexical token.
2. **Canonicality:** authored sugar lowers away; parse → format → parse and
   canonical identity are stable.
3. **Locality:** imports never silently change old meaning; ambiguity reports
   name candidates and repairs.
4. **Category honesty:** value, proposition, definition, law, query,
   transition, effect, and evidence cannot be confused.
5. **Ceremony reduction:** the corpus loses scaffolding without losing roles,
   types, public mode contracts, bounds, time, authority, or effects.
6. **General-purpose reach:** ordinary computation, state, events, effects, and
   real applications do not escape into an unrelated embedded language.
7. **Tooling parity:** spans, role-aware diagnostics, formatter, structural
   diff, generated mappings, and agent repair remain deterministic.
8. **Projection parity:** selected semantic judgments reproduce from sealed
   artifacts after authoring source deletion on every claimed target.
9. **Operational honesty:** modeled transition, authorized intent, attempted
   effect, receipt, and later observation remain distinct.
10. **Bound honesty:** partial search or derivation never claims completeness or
    optimality it did not prove.

## Compiler and implementation ownership

The architecture keeps independently replaceable ownership boundaries:

| Boundary | Owns | Must not own |
| --- | --- | --- |
| Surface/front end | fixed reader, scoped shapes, formatter, spans, ambiguity and explicit rendering | semantic evaluation or target policy |
| Checked program | types, categories, stable identities, Program/Model/State contracts and source provenance | physical layout or host resources |
| Value lowering | pure definitions, patterns, collections and target-neutral value IR | proposition search or effect realization |
| Proposition/lineage | relations, laws, closure, proof/support/lineage, exact intervention verification | heuristic authority or external effects |
| Transition runtime | events, state matches, conflicts, exact Deltas, replay and StateRevision | browser callbacks mutating state directly |
| Effect runtime | capabilities, resource lifetime, async outcomes and receipts | claiming intent as fact or hiding observable order |
| Target planner | JavaScript, Rust and Wasm orientation, indexes, schedules, layouts and bridges | changing source semantics to fit a target |
| Adapters/libraries | Three.js, game/ECS patterns, North and host capability bindings | new kernel metaphysics for one domain |
| Tooling | editor, debugger, hot reload, migration, profiling, package and deployment projections | a second source or semantic authority |

A milestone commander owns end-to-end closure across these boundaries; boundary
owners retain their invariants. Parallel implementation may proceed only where
the dependency spine and canonical acceptance artifacts make reconciliation
deterministic.

## Conflict resolutions

The packet contains a small number of real tensions. They are resolved as
follows:

| Tension | Resolution |
| --- | --- |
| The executive milestone order puts state before effects, while the detailed game plan puts effectful JS FFI before state. | State/events/transitions come first. Pure ESM code generation may ship with M2; effectful foreign calls wait for M4. |
| “The sealed Revision is the program” is exact today but overloads program and every live frame in the proposed system. | Keep the sentence scoped to the current Model/Revision core. The whole system seals ProgramRevision and gives ModelRevision, StateRevision, and RuntimeSession separate authority. |
| The packet alternates between declaring and inferring capability manifests. | Effect declarations, effect sites, foreign boundaries, and observable order are explicit. The aggregate manifest is derived, inspectable, target-validated, and sealed unless an explicit public restriction is required. |
| The packet leaves `model`, `world`, and `state` open. | `world` remains retired. Program, Model, and State name non-overlapping authorities. |
| Single-valued modes are used as if they automatically authorize replacement. | Cardinality alone does not authorize it. The checker must enforce keyed uniqueness and, for exactly-one, totality before implicit replacement; otherwise use explicit Deltas. |
| The symbolic vocabulary reads as settled while the packet also says punctuation must stay fluid. | Semantic distinctions are frozen; `=`, `+`/`-`, cardinality glyphs, transition glyphs, and related overloading are M0 corpus-led prototype decisions. |
| The specimen is concrete enough to look like a draft specification. | It is mandatory acceptance input and a north-star checked rendering exercise, not frozen source syntax. |
| The packet offers a later two-language fallback while the product direction is singular. | One Clause language is the commitment. A split is considered only at the M5 evidence gate and requires an explicit strategic amendment. |
| Complete intervention enumeration is presented beside practical guided search. | Correctness and minimality remain exact per returned result; optimality and completeness are separate explicit guarantees. Heuristics affect discovery order, never admission. |

No other substantive packet proposal is rejected. Remaining choices are routed
to the named prototype or vertical gate rather than silently omitted.

## Anti-goals and deferrals

- no probabilistic natural-language parser or import-sensitive interpretation;
- no universal prefix grammar, positional-role regression, or semantic identity
  derived only from mutable wording;
- no hidden effects, search, temporal mutation, conflict order, or universal
  relation inversion;
- no global equality-saturation ontology or theorem prover before a bounded
  use case earns it;
- no requirement that every value or frame be a stored proposition or durable
  graph snapshot;
- no `mathom` language primitive: the earlier material motivates durable
  identity and provenance but does not add another Clause ontology object;
- no generic graph/ECS tax when a mode permits a specialized array, index, or
  ordinary function;
- no Wasm-first browser host, fine-grained JS/Wasm boundary chatter, or source
  rewrite for target partitioning;
- no self-hosting, unrestricted dependent types, full actors/distributed
  semantics, native UI framework, or lexer-rewriting macro system as a
  prerequisite for the first complete vertical;
- no effects before the state/transition core can distinguish modeled change
  from observed reality;
- no stochastic pruning: exact pruning, heuristic ordering;
- no syntax stabilization before the corpus and complete game have exposed its
  repetitive pain.

## Roadmap completion standard

Clause reaches the committed category only when the same sealed program can:

- express ordinary values and algorithms without relational contortions;
- state role-labelled domain propositions and recursive laws;
- derive consequences and exact explanations from shared lineage;
- synthesize certified exact model changes while reporting partial discovery
  honestly;
- evolve deterministic runtime state through conflict-checked Deltas;
- cross external boundaries through explicit capabilities and receipts;
- materialize inspectable JavaScript, specialized Wasm, and native projections;
- support a complete Three.js game and a real North application vertical;
- retain source-mapped formatting, diagnostics, replay, diff, why, and
  counterfactual navigation through the whole authoring loop.

The result should not merely compile. It should make the formal structure of a
program read in the order of the ideas it expresses while carrying machinery
that conventional languages force every author to rebuild by hand.
