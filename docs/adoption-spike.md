# Process-First Term Kernel Adoption Spike

> **Status:** Authorized falsification design; not implemented.
>
> **Authority:** Normative only for the experiment that decides whether the
> mechanism in the [semantic foundation](foundation.md) survives. It does not
> define Clause semantics or authorize a representation migration by itself.

## Decision question

Determine whether one generic recursive Term, Clause-judgment, Run, and
admission kernel can carry the dangerous semantics of a general-purpose
language while the host remains a bootstrap and optimization boundary rather
than a private semantic authority.

The decisive question is:

> Can Clause add and understand a new language concept by adding Clause
> judgments, or must the host learn a new semantic secret?

The exact pre-spike behavior baseline is
`4aea6c898f3eec2fe4058d578f491eec008d7f9a`. Current identities, results,
proofs, runtime histories, wires, and generated outputs are parity oracles, not
authority to predetermine the new representation.

## Constraints

- The spike is isolated from the supported implementation until a stop/go
  decision accepts a bounded migration.
- Clause owns every Term codec, equality rule, judgment, occurrence,
  persistence interface, and reload rule used by the spike.
- No external store, database, or older project is a semantic or runtime
  dependency.
- The host may implement a small generic kernel and checked optimizations. It
  may not add one semantic case per source construct.
- Every migration-sensitive identity uses a new explicit semantics epoch. The
  spike never reinterprets semantic-v10 / Revision-v6 bytes or IDs.
- The experiment must preserve readable relation-first source. Graph
  bookkeeping cannot leak into ordinary programs.

## Phase A: generic kernel

Implement only machinery shared by every gate:

```text
Atom(kind, canonical payload, declarative versioned equality contract)
RawTriple = [Term, Term, Term]
Term = Atom | RawTriple
universe- and semantics-epoch-indexed structural Term equality
private hash-consing handles that cannot escape as identity
explicit identity Terms for occurrences and continuity
well-founded identity allocation and cycle-aware terminating reload
same-identity, equal-value, and equivalent-denotation judgments
immutable typed contexts and candidate successors
Γ ⊢ t clause : T @ M
Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩
returned, choices, yielded, suspended, failed, and exhausted outcomes
generic admission with exact failed obligations
constitutive, derived, observational, cached, and speculative contexts
Clause-authored relation schemas, readings, modes, and completion rules
stable named roles and atomic n-ary admission
scope, binder, use, quote, hygiene, phase, and origin relationships
total, productive, bounded, partial, nondeterministic, streaming, and reactive execution contracts
deterministic transformations with declared termination or fuel
canonical source focus, printing, elaboration, and source occurrences
Clause-owned persistence, canonical reload, and tamper rejection
checked lowering into current evaluator and generated targets
traceability from source projection to exact artifact and Run trace
```

Raw Triples receive no mandatory nominal `ClauseId`. Interning handles and
storage coordinates are unobservable implementation details. Semantic cycles
use opaque identity anchors and never content-hash recursively through their
own neighborhoods. Equality contracts are Clause data, not host callbacks.

## Gates 1–8

### 1. Pure evaluation

Represent and evaluate:

```clause
5 + 3
```

The expression is a structural Term distinct from the value `8`. The mode is
deterministic and pure, `Γ̂ = Γ`, and evaluation creates no ProgramRevision,
StateRevision, assertion, nominal entity, durable Run occurrence, or trace.

### 2. Binding and closure

Represent:

```clause
x => x + captured
```

Binder, uses, and function definition have exact identities independent of
spelling. Capture, shadowing, recursion, canonical printing, rename, and source
origins remain exact. Structurally equal lambdas do not silently become the
same binder occurrence or runtime closure.

### 3. Algebraic data and exhaustive matching

Define a closed sum/product type and a total matcher. Constructor membership is
type-directed; equal short names in different types do not collide. Exhaustive
coverage is a derived obligation. A total mode rejects missing cases, while a
partial mode represents failure honestly.

### 4. Structural and nominal higher arity

Demonstrate both cases:

1. a structural n-ary value whose canonical recursive Term contains every
   named role; and
2. a unique transfer or event anchored by an explicit identity Term.

The transfer must retain actor, amount, source, destination, and optional time
with exact RoleIds, role types, cardinality, completeness, and source-order
independence. Two identical-looking transfers remain distinct occurrences or
entities. An incomplete candidate cannot be queried as admitted content.

### 5. Recursive derivation

Implement a recursive relation with exact independent supports,
retraction/invalidation, explanation, and bounded local recomputation. Exercise
at least one terminating mode, one nondeterministic or streaming mode, and one
bounded, productive, or reactive mode. Observe finite choices or a typed
continuation, explicit suspension/failure/bound exhaustion, and declared
ordering/fairness, cancellation, and resource behavior as applicable. Do not
imply that the kernel can decide arbitrary termination.

### 6. State and effects as Runs

Run one state transition from an exact StateRevision through the default
two-phase effect protocol. Preserve separately:

- predecessor StateRevision;
- event and transition occurrences;
- candidate delta and invariant obligations;
- admitted successor StateRevision plus authorized effect intent;
- the separately identified external effect Run;
- external attempt, receipt, observation, and later admitted external claim;
  and
- trace Terms describing the Run.

Replaying or reloading the trace must not repeat the external effect. Equal
post-state content reached through distinct occurrences must retain distinct
StateRevision identities. Rejecting an evidence-admission candidate after the
external attempt must not claim the act was rolled back; the occurrence remains
visible for reconciliation. Any alternative transaction adapter must expose and
test its stronger atomicity, retry, idempotency, and failure contract.

### 7. Typed hygienic macro

Implement a user macro that introduces a binder, consumes an expected type,
proposes a typed successor syntax/semantic context, preserves origins and
navigation, and terminates under a declared total or bounded phase mode. Quote,
pattern, proposition, and executable contexts must not collapse.

### 8. Host-freeze extension

After gates 1–7 pass, freeze the host semantic kernel and record the exact host
files and commit. Then add a new construct combining binding and effects using
only Clause-authored schemas, readings, modes, judgments, and transformations.

A suitable specimen is a scoped resource form:

```clause
resource file from path
  use as handle
  body ...
```

Without changing the frozen host semantics, the construct must inherit:

- parsing and exact grouping;
- canonical printing and stable projection;
- binder identity, scope, capture, and hygiene;
- type, ownership/resource, effect, and capability checking;
- source origins and semantic navigation;
- identity-preserving rename and refactoring;
- precise dependency tracking and invalidation;
- target lowering and artifact traceability;
- diagnostics as exact failed obligations; and
- explanations as ordinary semantic queries.

No new host semantic enum, construct-specific validator, formatter case,
refactoring rule, analysis plugin, or manually maintained dependency rule is
allowed. If a generic-kernel defect requires a host repair, the freeze is
invalidated; repair the kernel, refreeze, and rerun the gate from the beginning.

The extension's schemas, readings, modes, and judgments must round-trip as
inspectable Clause Terms and execute through the same generic machinery used by
the earlier gates. An opaque “generic” callback, per-construct dispatch table,
foreign evaluator, or tag whose meaning exists only in host code fails the
gate. Irreducible FFI primitives may sit behind explicit typed effect and trace
contracts; they may not define the construct's language semantics.

Generated host code is allowed only when Clause-authored meaning and a checked
refinement are authoritative and reproducible. A later hand-optimized lowering
requires a separate equivalence proof.

## Source and focus oracle

Every source line elaborates to `(term, focus)`. Each indented child receives
the parent's focus as its omitted left operand; the declared parent reading
selects focus. Indentation never independently means membership, body,
containment, application, ownership, sequence, or authority.

Reading selection must be deterministic from explicit syntax, declared
grammar, and the already selected ElaborationContext before child domain
semantics are inspected. Missing or competing readings fail explicitly; schema
or type failure may not regroup the CST or reinterpret siblings.

For each printable feature fixture:

```text
elaborate(print(P)) ≅ P
print(elaborate(source)) = canonical(source)
```

The equivalence explicitly accounts for layout, comments, source occurrences,
and fresh nominal allocation. Negative fixtures reject missing/ambiguous
readings, ambiguous focus, multiple unfilled structural holes, child-dependent
reading selection, schema-driven regrouping, and undeclared source-order
semantics.

## Identity and equality oracle

The spike must demonstrate separately:

- two constructions of one structural Term;
- two independent occurrences over equal Term content;
- different explicit identities with equal values;
- different structures with equivalent denotation;
- one pure expression Term and its evaluated value;
- explicit binder, definition, entity, Run, effect, and revision identities;
- well-founded identity allocation, deterministic cyclic identity anchors, and
  terminating canonical reload without recursive content hashing;
- equal-looking transfers as distinct events;
- a source rename preserving lineage only through explicit evidence; and
- exact rejection of foreign, stale, forged, or mismatched identities.

The Atom fixtures must include explicit policies for Unicode normalization,
numeric widths, NaNs, and signed zero. Each equality contract must serialize as
declarative Clause data, round-trip under its exact semantics epoch, and reject
cross-epoch use without explicit migration. An undefined, opaque-callback, or
host-dependent equality case fails the spike; silently excluding the value
after it entered a stable identity domain also fails.

## Act/trace and admission oracle

The spike must prove that:

- constructing or persisting a Term asserts and executes nothing;
- a proposition is not an assertion occurrence;
- a Clause judgment does not make its Term true or current;
- a Run can be observed without its trace becoming the act;
- trace replay does not repeat effects;
- pure, query, and rejected Runs preserve their input context;
- a State transition stages intent but no receipt before admission;
- the external effect has its own occurrence after admitted authorization;
- failed evidence admission after an external attempt acknowledges rather than
  rolls back the act;
- incomplete n-ary candidates cannot leak into admitted contexts;
- only admission creates ProgramRevision or StateRevision successors; and
- failed admission returns the exact unmet obligations without partial
  authoritative mutation.

## Persistence and reload oracle

The spike's persistence mechanism must round-trip Clause-owned canonical Terms,
explicit identities, occurrences, judgments, contexts, and traces. Reload
recomputes structural identity, validates universe, semantics epoch, equality
contract, identity scope, anchors, and causal lineage, terminates on cyclic
identity references, and rejects tamper. Equal structural Terms may share
storage while independent occurrences and entities remain distinct.

Private row IDs, pointers, arena indexes, and content-cache handles must not be
observable as semantic IDs. Restarting or changing the physical backend must
not change canonical meaning.

## Strategy and target oracle

The following may be checked derived forms:

- lossless CST and source graph;
- indexed named-role and support views;
- e-graphs and optimizer candidates;
- control/dataflow and target IRs;
- packed layouts, heaps, registers, database indexes, and browser objects; and
- generated Rust, Wasm, and JavaScript.

Every semantics-affecting path remains explainable back to the admitted graph.
Physical choices affecting observable behavior or declared ABI, layout,
overflow, floating point, ordering, determinism, synchronization, cancellation,
durability, failure, resource, or latency contracts appear as explicit
strategy/evidence judgments.

At least the existing evaluator, generated Rust, and generated JavaScript must
agree on the selected fixtures. The ordinary hot path must demonstrate
specialization rather than generic Triple interpretation. Instrument at least
one recursive query, one State transition, and one generated-target operation;
then add a large disconnected set of unrelated Terms. Their executed plan and
semantic result must remain identical, and graph/index accesses must stay
bounded by the declared dependency closure plus documented index lookup rather
than grow with the unrelated graph.

## Required negative evidence

The spike must actively reject or bound:

- malformed, incomplete, duplicate-role, and missing-role n-ary candidates;
- independent equal occurrences accidentally deduplicated;
- a hash-consing or persistence handle leaking as identity;
- a structural Term reused as a nominal entity merely because contents match;
- propositions treated as assertions or trace Terms treated as occurrences;
- trace replay that repeats an external effect;
- a transition receipt appearing before the external effect Run;
- rejected evidence pretending an already attempted effect did not happen;
- quoted, pattern, or hypothetical Terms accidentally executed;
- NaN, signed-zero, normalization, or width policy disagreement;
- an opaque or cross-epoch Atom equality callback/contract;
- a claimed universal halting or executability decision;
- total modes with unproved termination and productive modes without progress;
- hostile, recursive, nondeterministic, or phase-escaping macros;
- construct-specific host semantics hidden behind Triple serialization, a
  generic callback, dispatch table, or foreign evaluator;
- source round trips that lose binding, occurrence, or concept lineage;
- whole-graph invalidation for a local semantic edit;
- pure evaluation that creates an authoritative revision;
- target behavior diverging from the accepted reference semantics; and
- ordinary source exposing graph bookkeeping ceremony.

## Pass and falsification decisions

The mechanism passes only if all eight gates pass on one exact generic kernel
and every required negative fixture fails for the intended reason.

A pass authorizes a bounded parity-preserving migration proposal. It does not
prove readability, target performance, systems coverage, macro usability,
large-graph incrementality, or lower maintenance cost at product scale.
Before that migration can be called successful, separate gates must measure
real source ergonomics, large-graph incremental cost, and matched
systems/JavaScript performance on representative programs.

The mechanism is falsified if a dangerous feature requires private host
semantics, mandatory identity on every Triple, arbitrary positional convention,
ad hoc untyped tags, act/trace collapse, an untracked meaning-changing
representation, ordinary graph-wide recomputation, unreadable source ceremony,
or generic execution that cannot specialize credibly.

Failure rejects the Term-kernel mechanism. It does not authorize shrinking
Clause's general-purpose mission.

## Out of scope

Later gates must separately measure:

- large-graph incremental precision and scale;
- ownership, concurrency, failure, and security semantics;
- packages, modules, separate compilation, FFI, and ABI;
- native, Wasm, JavaScript, browser, and database performance;
- macro and source ergonomics across real programs; and
- correct-change throughput and maintenance cost over growing systems.

More thesis prose cannot substitute for those observations.
