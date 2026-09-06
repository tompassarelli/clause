# Clause roadmap

> **Authority:** This document alone reports implementation status and
> remaining delivery work. The [foundation](foundation.md) defines meaning,
> [syntax](syntax.md) defines canonical source, and
> [architecture](architecture.md) defines implementation boundaries.

Clause is not a supported language or toolchain yet. The repository contains a
resident bounded compiler/runtime, experimental process carriers, and focused
native/Wasm/browser journeys. Passing a focused test establishes only its
named slice.

## What runs

The resident source compiler uses ordinary `domain`, `range`, and
`cardinality` facts as binary role contracts. The same role definitions govern
initial facts, handler patterns, recursive laws, runtime tables, structural
checking, and live edits.

Structural participation is checked from required properties rather than a
second nominal registry. A referenced Task must satisfy every required Task
role; atomic changes cannot leave a required `one` role absent. Nominal
domains without structural contracts still use membership. Bare creation
binders infer one domain from inserted roles. Behavioral conformance,
multi-domain creation, and general declaration-free derived-role typing remain
unfinished. `some` is canonical but has no runtime table lowering.

Named-variable focus such as `?task:` uses the same nested role grammar in law
premises and conclusions, handler conditions, withdrawals, inclusions, and
contributions. Flat and focused forms compile to the same bindings. This is a
bounded reader feature, not the uniform extensible frontend.

Named declarations constrain bindings under `?example:`. Their optional
Reading uses ordinary flat or focused clauses; declarations without a Reading
define structured fields. Modes remain separate. The declared focused-edge
grammar drives declaration constraints, facts, patterns, and canonical
printing. A four-binding scalar law and structured state journey exercises
checking, execution, independent re-reading, and checked scalar-effect edits.
Declaration discovery, scalar expressions, laws, handlers, and their lowering
still contain host bootstrap cases; this does not complete the frontend below.
Custom focused grammars read, print, execute, reload, and perform checked scalar
edits. Edit witnesses carry the exact selected frontend through package and
runtime checking.

Authorized positive laws compute a bounded least fixed point. The runtime
tracks independent support, preserves conclusions with surviving support, and
removes consequences—including unsupported cycles—after the last root is
withdrawn. Exhaustion rejects without admitting a prefix. Derived relations
are currently separate many-valued tables; recursive negation, aggregation,
allocation, general multi-input conclusions, and a source-soundness proof are
not implemented.

Finite F64 scalar laws support declared symbolic Readings, simultaneous
binder substitution, composed expressions, guarded alternatives, exact source
origins, and conservative uniqueness checking. Finite sums can join typed
rows and accept explicit `given` inputs. The compiler does not provide general
constraint solving, totality inference, or arbitrary collection functions.

Handlers can create finite typed Referents, join their rows, replace one/maybe
values, update many-valued relations, and combine explicit numeric
`accumulate` contributions against one pre-state. Required properties are
checked on initial and candidate worlds. [Created collections](created-collections.md)
records the exact current bounds and unsupported cases.

The resident source session preserves its accepted generation, live state,
handlers, and pending candidate across unchanged or rejected source. Checked
scalar expression edits preserve the existing target and created Referent
identities when the explicit continuity mapping permits it. General structural
continuity across arbitrary edits is not implemented; similar text never
justifies identity retention.

Typed keyboard, scalar, Text, occurrence, and Referent inputs reach handlers
through explicit bindings. Persistent native and real Wasm sessions execute
the bounded source-derived plans through hidden candidates and separate
Admission. Current browser adapters are passive for the tested projections,
but the complete process carrier, semantic refinement proof, controlled
frame-allocation proof, and uniform frontend remain open.

## Scheduling delivery

The active scheduling vertical uses five Tasks—design, prototype, validation,
documentation, and launch—with required title, duration, and completion roles.
Two Root values supply independent obstructions. Positive laws derive direct
and inherited blockers plus direct and inherited waiting dependencies.

Three typed inputs exercise different changes:

- `Resolve` withdraws one direct obstruction and recursively retracts only the
  consequences that lose their final support.
- `Complete` changes one Task from incomplete to complete only when finite
  queries find no waiting prerequisite and no blocker.
- `Extend` increments one duration expression while preserving the selected
  Task identity.

The source contains no Task membership registry. Wrong input domains must fail
checking. A prerequisite cycle may remain waiting but cannot complete merely
because all external obstructions were removed.

The assembled native aggregate passed all 87 tests. An actual Chrome journey
loaded the five-task Wasm application from the same compiled source, rejected
blocked completion, retracted dependent support after `Resolve`, extended a
duration, completed all five tasks, retained their identities, and reported no
page errors.

Scheduling delivery remains incomplete until it records independently authored
fact and edit-touch counts, exposes explanation and bounded hypothetical
queries, and measures the warm source-save-to-visible-result latency. The
language and performance goals remain incomplete with it.

## Remaining language path

The next implementation work follows dependency order; later application work
may proceed when it consumes only already-running semantics.

### One frontend

Replace built-in construct and vocabulary cases with one deterministic,
lossless reader driven by declared grammars and Readings. Implement canonical
printing, local recovery, multi-emission identity slots, precise source
origins, hygienic binding, and checked edit continuity. Existing role
contracts, focus, expressions, laws, handlers, input bindings, and created
relations must pass through that one path without semantic duplicates.

The frontend is complete only when adding a construct with binding and effects
changes Clause-authored declarations and transformations rather than a Rust or
Lean semantic switch. A `Compiler0 -> Compiler1` succession must change one
binding form, one effect form, one typed macro, and one diagnostic without host
semantic edits. Clause-defined algebraic data and exhaustive matching must
accept complete cases and reject missing and unreachable cases through the
same mechanism.

### Relations, definitions, and collections

Generalize positive recursive relations beyond the current binary finite
tables while retaining exact support withdrawal, occurrence multiplicity,
cardinality, and explicit exhaustion. Add reusable parameterized definitions,
checked specialization, separately compiled constraints, rich algebraic
values, Text and Bytes, sequences, maps, and ordinary collection operations.

Every feature must be authored once and remain available to checking,
execution, queries, explanations, optimization, and editing. Physical indexes
and caches may change plans but not relation meaning, support, or identity.
Negation and aggregation require explicit finite scope; failure to complete a
search never becomes absence.

### Queries, explanations, and editing

Implement `select`, `any`, `why`, `prevent`, `achieve`, and semantic
`diff` over the same admitted facts, laws, supports, occurrences, and
candidate deltas used by execution. Hypothetical queries run in isolated
bounded alternatives and report exhaustion separately from a result.

Live edits must preserve only identities justified by checked continuity,
explain every retained and fresh allocation, and never silently migrate a live
Activation. Ordinary checked edits must reach the affected running behavior
without whole-program restart or duplicated host logic.

### Local state, lifetime, and physical code

Implement loops, builders, request-local caches, and actor/frame state through
affine Activation-local configuration rather than StateRevision ceremony.
Parallel mutation must use ownership-consuming disjoint split/join; suspension
must transfer exact configuration custody.

Every allocation needs one owner, region, or foreign manager plus checked
Borrow/Lease edges. Observable close is explicit. The native/Wasm hot path may
not rely on hidden tracing collection, implicit reference counting, finalizers,
or unbounded teardown.

Clause-owned physical IR must support direct calls, registers, packed layouts,
declared ABI, native and Wasm specialization, and separate checking,
specialization, and physical-reuse keys. Optimized paths require exact
refinement evidence for their values, identities, failures, effects, causal
boundaries, and resource claims; generic graph execution is not the production
hot path.

### Processes and external boundaries

Complete the host-neutral process carrier for Activation, Step, Run,
Continuation, causal order, Authorization, capability, effect, and Admission.
Add bounded services, suspension and restart, handoff and cancellation races,
governed State transitions, honest external effects, and passive browser and
operating-system adapters.

Intent, issued effect Authorization, capability, attempt, receipt,
observation, Judgment, and Admission remain distinct. Rejection leaves
authoritative state unchanged; an external attempt cannot be erased by later
rejection.

The Clause-authored workbench must provide long-lived `parse`, `check`,
`explain`, `query`, `diff`, `propose`, `admit`, `run`, and `hotReload`
operations. Rust may own bounded transport, caches, scheduling, persistence,
and foreign calls, but no source grammar, semantic query, or diagnostic rule.

## Application proofs

### Greywrought

Greywrought remains in its own repository and owns game source, assets,
encounters, and passive host code. Clause owns the language, compiler, runtime,
Admission, explanation, specialization, and cross-host contracts it consumes.

The game proof requires several tactically distinct encounters continuously
playable on localhost. One revision-pinned operator journey must:

1. change a consequential combat law in Clause and show the admitted result
   without rebuilding the host, restarting the server, or reloading the page;
2. replay the same Program, world, inputs, authority, and observations through
   native and Wasm specialization with equal declared results;
3. reject one malformed or unauthorized change before execution or
   authoritative mutation;
4. explain the result through the selected behavior, Mode, laws, observations,
   supports, candidate delta, Admission, and successor, and answer the
   corresponding bounded `prevent`, `achieve`, and `diff` questions;
5. demonstrate projectile confirm, energy-spending burst approach, buffered
   direction-sensitive melee, and energy-aware disengagement against readable
   Clause-authored behavior; and
6. admit an untrusted agent or mod proposal only through its bounded candidate
   and a separate authorized decision.

All consequential world rules for that journey remain Clause-only. Rust,
TypeScript, Three.js, Wasm adapters, storage, and networking are passive or
explicit foreign boundaries. A component test cannot substitute for direct
play of the encounter.

### Scheduling

The scheduling application must remain substantially different from
Greywrought while using the same language mechanisms. It must support the five
task dependency journey above, recursive blockers and waiting, typed Resolve,
Complete, and Extend actions, bounded explanation and hypothetical questions,
and identity-preserving live edits from one authored source.

Its proof records independent semantic facts and the number of edits needed for
an ordinary requirement change. Duplicate registries, host-side dependency
logic, or a second UI state model fail the proof even if the screen appears
correct.

## Performance claims

Measure on the named reference PC rather than inferring performance from code
shape. The target is:

- 60 frames per second with real-time simulation;
- 100 active actors;
- ordinary checked edits visible within 250 ms on the warm
  source-save-to-first-visible-result path; and
- bounded memory, continuation, active-frontier, and trace residency over a
  long run.

The measurement must name the exact Program, runtime, target, actor count,
workload, warm-up, controlled allocation domain, and observed distribution.
A fast parser alone does not satisfy edit latency; a headless component alone
does not establish frame or browser behavior.

## Preserved contracts

The historical v0 corpus retains its exact bytes and six outcomes:
`returned`, `choices`, `yielded`, `suspended`, `failed`, and `exhausted`.
It is an oracle, not current syntax or process authority.

CLCP v1/v3 bytes, manifests, receipt replay, and compiler succession remain
scoped to their published compiler-machine contracts. They are not universal
process semantics.

Neutral Triple positions never acquire an inherent operator role. Equal
assertions, supports, and occurrences remain independent. Source movement
preserves nominal identity only through the applicable exact continuity rules.

Working capability is removed only after a tested successor covers each unique
behavior and every in-tree consumer has migrated. Removal then means absence
of superseded source, tests, fixtures, generated artifacts, documentation, and
consumers.

## Completion

Clause is complete for this goal only when both applications are usable through
one supported frontend, compiler, runtime, workbench, and canonical carrier;
execution, explanation, bounded hypothetical queries, optimization, and live
editing consume the same authored facts; recursive withdrawal, reuse,
collections, specialization, continuity, and safe extension meet their
contracts; native/Wasm/browser results agree at their declared boundaries; and
the measured performance targets pass.

Any remaining host-owned semantic case, duplicate authored fact, unsupported
core form, fictitious authority, or unmeasured target remains an open item
rather than a compatibility path or narrowed success claim.
