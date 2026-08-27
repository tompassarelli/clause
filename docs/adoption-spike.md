# Clause Constitutional Adoption Spike

> **Status:** Authorized falsification design; not implemented.
>
> **Authority:** Normative only for the experiment deciding whether the
> mechanism in the [foundation](foundation.md) survives. It cannot add Clause
> semantics or canonical syntax.

## Decision question

Can one generic recursive Term, Clause-judgment, Run, and admission kernel carry
the dangerous semantics of a general-purpose language while Lean and Rust
remain checker/reference and physical implementation boundaries rather than
private semantic authorities?

The decisive test is:

> Can Clause add and understand a new language concept by adding Clause data
> and judgments, or must a host learn a new semantic secret?

The spike starts from the accepted calculus, constructs its own canonical
vectors, and must justify every claimed behavior directly.

## Constraints

- Clause owns the Term algebra, equality contracts, contexts, judgments,
  identity rules, Runs, admission, canonical package, codec, and certificates.
- Lean is the constitutional checker and reference Run-semantics host. Lean
  syntax, expressions, type classes, serialization, and feature-specific
  inductives are not Clause semantics.
- Rust is the physical package, persistence, runtime, FFI, and backend host.
  Rust enums, traits, handles, layouts, and callbacks are not Clause semantics.
- No additional primary compiler host is introduced.
- No external semantic authority or imported ontology participates.
- The experiment uses readable relation-first source for the source-facing
  gates; graph bookkeeping does not leak into routine programs.
- A pass authorizes further implementation only. It does not establish product
  ergonomics, scale, target performance, or maintenance economics.

## Phase A — Minimal constitutional package

Before implementing Clause surface syntax, define the generic core required by
every gate:

```text
Atom(kind, canonical payload, declared equality contract)
RawTriple = [Term, Term, Term]
Term = Atom | RawTriple

Γ ⊢ t clause : T @ M

Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩

Γ ⊢ Γ̂ admissible
───────────────────
admit(Γ, Γ̂) = Γ′
```

The core must represent:

- contextually opaque Atoms and explicit refinements across universes;
- structural Term equality indexed by universe and semantics epoch;
- same identity, equal value, and equivalent denotation as distinct judgments;
- explicit identities for occurrences, binders, definitions, entities,
  concepts, Runs, effects, sessions, and revisions only where continuity
  requires them;
- immutable contexts and typed candidate successors;
- returned, finite-choice, yielded, suspended, failed, and exhausted outcomes;
- total, productive, bounded, partial, nondeterministic, streaming, reactive,
  and effectful mode contracts;
- Clause-authored schemas, stable named roles, readings, completion rules,
  capabilities, laws, and obligations;
- source occurrences, scope, binding, quotation, hygiene, phase, and origin;
- deltas, traces, derivations, certificates, strategies, and evidence; and
- canonical package bytes with cycle-aware, terminating, fail-closed reload.

Raw Triples receive no mandatory nominal identity. Private interning handles
cannot escape as identity. Cycles use explicit identity anchors rather than
recursive content hashes.

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

### Rust boundary

Rust independently decodes the same package and may build physical indexes,
stores, interpreters, or generated plans. Its output must agree with the Lean
reference relation for every declared observable and nonfunctional contract.
Rust may not reimplement semantic classification through a closed feature enum,
opaque callback, source-form match, or private side table.

## Gates 1–8

### 1. Pure evaluation

Represent and run integer addition.

Required distinctions:

- expression Term, result value, and denotation are not one identity;
- the mode is deterministic and effect-free;
- the authoritative context is unchanged; and
- equivalent syntax may produce equal values without collapsing occurrences.

### 2. Binding and closure

Represent a binder-introducing function, lexical capture, application, and
canonical source projection.

Required distinctions:

- binder identity is independent of spelling and source position;
- every use resolves through explicit scope relationships;
- alpha-equivalent forms may be denotationally equivalent without sharing
  occurrence identity; and
- closure capture is inspectable Clause data, not host environment state.

### 3. Algebraic data and exhaustive matching

Represent a user-defined sum type, constructors, patterns, and exhaustive
matching without adding host feature constructors.

The checker rejects missing and unreachable cases with exact obligations.
Pattern binding and result type remain graph-native and source-projectable.

### 4. Structural and nominal higher arity

Represent one structural value and two equal-looking but independently
identified transfers using stable named roles.

Every role has identity, type, cardinality, and complete atomic admission.
Source order is irrelevant after elaboration. Structural equality must not
collapse transfer occurrence or entity identity.

### 5. Recursive derivation and honest modes

Represent recursive reachability with exact independent supports. Exercise:

- one proven terminating mode;
- one bounded or productive mode;
- one nondeterministic or streaming mode; and
- explicit partiality or exhaustion.

The compiler may prove termination for restricted modes but must not claim a
universal halting or executability decision.

### 6. State and effects as Runs

Represent a State transition that stages a candidate successor and effect
intent. Admission accepts the State successor and authorization. A separately
identified effect Run performs the external attempt and produces receipt or
failure evidence for later admission.

The gate distinguishes current world, event request, transition occurrence,
candidate delta, admitted successor, effect intent, authorization, attempt,
receipt, observation, and evidence. Replay of trace data never repeats the act.

### 7. Typed hygienic macro

Represent a binder-introducing macro as a typed transformation between explicit
syntax contexts. It preserves source origin, binding, phase, types,
capabilities, and diagnostics. Expansion is a candidate successor admitted only
after its obligations pass.

### 8. Host-freeze extension

Freeze the Lean checker/model and Rust semantic boundary. Then add through
Clause data alone a new abstraction combining:

- a binder;
- an effect capability;
- a State transition; and
- custom readable syntax with canonical printing.

The feature must require no construct-specific Lean/Rust semantic constructor,
validator, callback, dispatch entry, formatter, refactor, analysis, dependency
rule, or target semantic branch. Failure of this gate falsifies the claimed
single authority.

## Cross-cutting obligations

Every gate must preserve:

- deterministic reading selection before child-domain checking;
- lossless source occurrences and canonical parse/print/parse meaning;
- exact role identity and complete n-ary neighborhoods;
- local dependency invalidation rather than routine whole-graph recomputation;
- canonical reload and tamper rejection;
- occurrence-exact derivation support and retraction;
- act/trace separation under retry, cancellation, and failure;
- explicit strategy judgments for observable ABI, layout, overflow,
  floating-point, ordering, synchronization, durability, resource, and latency
  contracts; and
- specialization into a non-generic hot path for at least one query, State
  transition, and generated target operation.

For the specialization check, add a large disconnected set of unrelated Terms.
The selected plan and semantic result must stay identical, and measured graph
or index access must remain bounded by the declared dependency closure plus
documented lookup cost.

## Required negative evidence

The spike actively rejects or bounds:

- malformed, incomplete, duplicate-role, and missing-role candidates;
- accidental deduplication of equal occurrences or entities;
- host handles, pointers, row IDs, or source spans leaking as identity;
- expression/result/denotation collapse;
- propositions treated as assertions or traces treated as occurrences;
- effect replay, fabricated receipts, and false rollback claims;
- quoted, pattern, hypothetical, or speculative Terms executed as authority;
- NaN, signed-zero, Unicode-normalization, or numeric-width disagreement;
- opaque or cross-epoch Atom equality;
- total modes with unproved termination and productive modes without progress;
- hostile, recursive, nondeterministic, or phase-escaping macros;
- hidden semantic cases in host enums, callbacks, dispatch tables, serializers,
  formatters, or generated runtimes;
- a Lean trust escape or unlisted axiom in the certificate closure;
- unexplained Lean/Rust disagreement;
- source round trips that lose binding, occurrence, or concept continuity;
- whole-graph invalidation for a local edit;
- pure evaluation that creates an authoritative revision; and
- generic Triple execution presented as a credible production strategy.

## Pass and falsification

The mechanism passes only when Phase A meets the trust profile, all eight gates
pass over one exact Clause Core contract, Lean and Rust agree on every declared
observable and nonfunctional contract, every negative fixture fails for the
intended reason, and the host-freeze extension adds no private semantic case.

The mechanism is falsified if a dangerous feature requires private host
semantics, mandatory identity on every Triple, positional convention, untyped
tags, act/trace collapse, an untracked meaning-changing representation,
ordinary graph-wide recomputation, unreadable graph ceremony, or generic
execution that cannot specialize.

Lean is rejected as the constitutional host if ordinary generic work requires
pervasive proof ceremony, constitutional `partial`/`unsafe` escape, distortion
of Clause modes to fit Lean, unworkable canonical exchange, intolerable feedback
cost, or a checker substantially larger and less comprehensible than the
boundary it protects.

Failure preserves the general-purpose Clause mission and records the exact
forcing counterexample. It does not authorize silently shrinking the language
or introducing a second semantic authority.
