# Clause roadmap

> **Authority:** This document alone reports implementation status and
> remaining delivery work. The [foundation](foundation.md) defines meaning,
> [syntax](syntax.md) defines canonical source, and
> [architecture](architecture.md) defines implementation boundaries.

Clause is not a supported language or toolchain yet. The repository contains a
resident bounded compiler/runtime, experimental process carriers, and focused
native/Wasm/browser journeys. Passing a focused test establishes only its
named slice.

Read [demonstrated capabilities](#demonstrated-capabilities) for current
behavior, [remaining language work](#remaining-language-work) for dependency
order, and [application acceptance criteria](#application-acceptance-criteria) plus
[performance targets](#performance-targets) for the broader acceptance criteria.
A delivered application slice does not complete those remaining criteria.

## Demonstrated capabilities

### Resident source compiler and runtime

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

Named declarations constrain bindings with the same ordinary `shape` premises
checked in laws and handlers. Explicit shared-edge focus `(shape: F64):`
applies one bound role/object to each named subject; it introduces no type
default or membership fact. Plain structured fields need no unused binders.
Readings remain clean ordinary clauses and Modes remain separate. The declared
edge grammar drives constraints, facts, patterns, and printing, including a
changed-grammar checked live edit. A four-binding scalar law and structured
state journey exercises execution, re-reading, and identity continuity.
The grouping delimiter, declaration discovery and conformance interpretation,
structured grammar selection, scalar expressions, laws, handlers, and lowering
still contain host bootstrap cases; this does not complete the frontend below.
Contextual structured objects use subject focus `first`, then `position:`, then
ordinary `x: 2.0` and `y: 3.0` field edges. The role range selects the existing
field contract. One typed field CST reaches initial values, matching, and atomic
replacement directly, without rewriting into brace source. Whole-value copies,
field expressions, and runtime-created rows use the same checked structure.
Nested record lowering and generic structured event arguments remain unfinished;
the historical jump-arena input header still uses its specialized Vec3 carrier.
Custom focused grammars read, print, execute, reload, and perform checked scalar
edits. Edit witnesses carry the exact selected frontend through package and
runtime checking.

Authorized positive laws compute a bounded least fixed point. The runtime
tracks independent support, preserves conclusions with surviving support, and
removes consequences—including unsupported cycles—after the last root is
withdrawn. Exhaustion rejects without admitting a prefix. Derived relations
use optional or many-valued tables, including structured values. Finite sums
in derived laws read completed prerequisite relations. The runtime infers
strata from checked row dependencies, completes positive recursion within
each stratum, and rejects cycles through aggregates. Root changes recompute
all strata atomically. Recursive negation, recursive aggregation, allocation,
general multi-input conclusions, and a source-soundness proof are not implemented.

Finite positive matching currently returns complete matches or an explicit
error, including a resource-limit error. Bounded intervention queries instead
return `completed` and `exhausted` fields alongside an optional solution;
an evaluator error rejects the query. These are distinct API contracts, not
one universal three-valued Boolean. Keeping query outcomes, transactional
admission, and local ownership obligations coherent is unfinished work; their
cost cannot be inferred from the semantic distinctions alone. The measured
runtime and edit observations below do not isolate open-world reasoning or
affine ownership as a performance cause.

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

Positioned measurement occurrences exercise one ordered-value transformation
without changing their explicit positions or identities. Exact relation-value
selection uses a checked inverse index over the same typed pre-state, with
complete coverage and order-preserving buckets. Execution, explanation, and
finite queries share that path. This is a bounded positioned-relation slice,
not general sequence operations or arbitrary specialization; see
`clause:docs/ordered-collections.md`.

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

### Scheduling delivery

The demonstrated scheduling vertical uses five Tasks—design, prototype, validation,
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

The scheduling application now uses one retained session for execution,
explanation, bounded hypothetical queries, and identity-preserving checked
edits. Its 24 independently authored initial domain assertions are 2 blocker
reasons, 15 task scalar attributes, 5 prerequisites, and 2 direct obstructions;
derived rows, schemas, and rules are not counted. An ordinary duration-rule
change touches one scalar-expression source span. The measured same-page warm
source-save-to-visible samples were 206.1, 210.2, and 212.6 ms. Adoption also
observed a 273.4 ms first valid edit, so this application slice is delivered
without claiming the 250 ms performance target. The broader language and
performance goals remain incomplete.

### Latest Greywrought runtime evidence

At Clause revision `756895f60879e8e06cfe59b6a7cc4adb2cb2c040`, focused
relational checks passed for reuse of input-independent relation prefixes
within sum queries. The supplied 2026-09-07 hardware profile recorded 127.9 ms
total and 96.4 ms in actual queries, compared with an earlier 258.4/221.5 ms
profile. The measurements used different warmed ticks; they are not a matched
comparison or proof of a performance improvement. Greywrought's full 100 FPS,
20 ms, real-time simulation, and edit-latency delivery gates remain unmet.

### Walkthrough evidence

The [language-tour journey](language-tour.md#run-the-scheduling-example) uses
`clause:test-vectors/authoring/scheduling.clause` and the existing
`clause:crates/clause-workbench/tests/scheduling.rs` public-library test. Its
check covers source opening, recursive blockers, finite completion queries,
separate candidate execution and admission, task completion, and retained
Referent identities. On 2026-09-07 at revision
`756895f60879e8e06cfe59b6a7cc4adb2cb2c040`, the exact focused command in the
tour passed: 1 test, 0 failures, 3 filtered out; 31.73 s cold build and 0.14 s
test execution. It does not establish the complete frontend, general
query syntax, browser behavior, or any performance target.

## Remaining language work

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

## Application acceptance criteria

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

## Performance targets

The earlier language-wide reference target below remains an unpassed baseline;
it does not replace the stricter Greywrought delivery gates recorded above.
Measure on the named reference PC rather than inferring performance from code
shape:

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
