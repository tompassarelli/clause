# Clause Architecture

> **Status:** Accepted bootstrap boundary; semantic implementation pending.
>
> **Authority:** Derived and non-semantic. The
> [foundation](foundation.md) governs meaning, [syntax](syntax.md) governs
> canonical source, and [roadmap](roadmap.md) governs implementation status.

## Decision

Clause uses one host-neutral semantic contract and three implementation roles:

```text
                       Clause source
                            |
                            v
                  canonical Clause Core package
                     /                 \
                    v                   v
       Lean constitutional        Rust physical substrate
       checker/reference Run      persistence/runtime/FFI/backends
                     \                 /
                      \---- parity ---/
                            |
                            v
                  admitted Clause revision
```

Clause Core owns meaning. Lean is the first rigorous checker and executable
reference model. Rust owns replaceable physical machinery. Clause progressively
takes over elaboration, macros, diagnostics, refactoring, planning, projection,
and compiler orchestration.

OCaml has no primary role. Aeneas is not part of the bootstrap or trust chain.
It may be reconsidered later for isolated safe-Rust verification only.

## Live-tree boundary

The repository contains two new implementation roots:

```text
lean/                       constitutional checker/reference model
crates/clause-substrate/    physical persistence/runtime/backend substrate
```

Both begin semantic-empty. New work derives only from the current Clause
contract. Git history is recovery, not an implementation input.

Every tracked source, test, example, document, generator, host, and release
script must describe only the current architecture. Superseded material leaves
no alias, shim, warning-only decoder, fixture, comment, generated consumer, or
gate that teaches it.

## Host-neutral Clause Core

The Clause Core contract is the transport and checking form of the calculus in
the foundation:

```text
RawTriple = [Term, Term, Term]
Term      = Atom | RawTriple

Γ ⊢ t clause : T @ M

Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩

Γ ⊢ Γ̂ admissible
───────────────────
admit(Γ, Γ̂) = Γ′
```

The package must carry every semantics-affecting object needed by a judgment:

- canonical Terms and explicit equality contracts;
- distinct identities where occurrence or continuity requires them;
- contexts, strata, judgments, relation schemas, modes, and capabilities;
- candidate successors, deltas, obligations, derivations, and certificates;
- source origins and separately scoped traces, strategies, and physical
  evidence; and
- a semantics epoch and one canonical byte representation.

The package is not a new ontology. Lean values, Rust structs, proof terms,
indexes, source maps, traces, caches, and strategies do not enter semantic
identity unless an authored Clause judgment explicitly makes their content
semantic. Lean proof terms remain local. Only Clause-native certificate data
crosses the host-neutral boundary.

The Lean canonical codec and independent Rust physical codec are implemented
from one normative Clause-owned specification and vector corpus. Their shared
release gate requires byte-identical positive re-encoding and matched negative
verdict classes. No host serializer is the wire format.

## Lean constitutional kernel

Lean models Clause's own generic Terms, judgments, modes, Runs, contexts, and
admission rules. Clause features do not become Lean `Syntax` kinds, `Expr`
constructors, type classes, or one inductive constructor per language form.
Lean proves claims about Clause data; it does not become Clause's source
language or ontology.

The reference Run semantics is relational and can represent total, bounded,
partial, nondeterministic, streaming, reactive, and effectful modes. Fuelled
interpreters may execute bounded specimens. Lean host termination never decides
Clause partiality or converts an open process into a false total function.

The constitutional checker is accepted only when all of these hold:

- the exact Lean source, toolchain, and imported artifacts are pinned and
  hashed;
- `trustLevel = 0` is used for new declarations without pretending it rechecks
  imports;
- every declaration in the transitive constitutional dependency closure is
  replayed into a fresh kernel environment and every reachable `unsafe` or
  `partial` declaration is rejected;
- the closure contains no `sorry`, `sorryAx`, skipped checking, elaboration
  recovery axiom, failed-declaration fallback, or preliminary asynchronous
  environment;
- acceptance does not rely on `native_decide`, native reduction, executed
  `implemented_by` or `extern` replacements, a compiled Boolean, or a foreign
  implementation;
- the transitive axiom closure is checked against an explicit policy, including
  deliberate decisions for `propext`, `Quot.sound`, and `Classical.choice`;
- every certificate is bound to the exact canonical package bytes, semantics
  epoch, decoded value, and claimed Clause proposition; and
- `leanchecker` or equivalent replay is treated as a same-kernel consistency
  check, not an independent verifier.

No `unsafe`, `partial`, or `sorry` is permitted in the constitutional package.
Clause partiality and effects are object-language data and relations.

## Rust physical substrate

Rust may implement:

- compact canonical decoding and interning;
- indexes and incremental dependency maintenance;
- durable persistence and transaction machinery;
- operating-system, filesystem, network, browser, and foreign interfaces;
- runtime scheduling and resource accounting;
- native, Wasm, and JavaScript materialization; and
- profiling and target-specific physical strategies.

Rust may not define what a Clause relation, binder, type, transition,
capability, effect occurrence, identity, or admission means. It consumes an
accepted Clause Core package and may create checked proposals or optimized
views. A Rust enum, trait, pointer, arena index, row number, or object layout is
never semantic authority or identity.

The substrate remains `unsafe`-free until an unavoidable foreign boundary is
identified and separately authorized. Any future unsafe module is isolated,
documented, tested, and outside the constitutional checker.

## Clause-authored compiler middle

Stable semantic machinery moves into Clause in this order:

1. relation schemas and modes;
2. elaboration and typed macro rules;
3. obligation construction and diagnostics;
4. semantic queries, impact analysis, and refactoring;
5. planning, source projection, and compiler orchestration; and
6. selected checking and lowering machinery.

The host-freeze test is constitutional:

> An ordinary language abstraction combining binding, effects, and readable
> source must be addable as Clause data without a feature-specific Lean or Rust
> semantic branch.

Host changes are allowed only for a genuinely new primitive physical
capability or checked optimization strategy.

## Execution and physical freedom

Pure Runs preserve the authoritative context. Authoritative Runs stage a
candidate successor; admission alone creates the successor revision. State
transition and external effects cross distinct boundaries: transition
admission authorizes effect intents, separately identified effect Runs perform
acts, and later evidence may record attempts, receipts, and observations.

The compiler may lower accepted meaning into registers, structs, arrays,
indexes, state machines, native instructions, Wasm, JavaScript, database
layouts, or browser objects. A physical decision that affects observable
behavior or a declared ABI, layout, overflow, floating-point, ordering,
determinism, synchronization, cancellation, durability, failure, resource, or
latency contract must remain an explicit strategy or evidence judgment.

Generic Triple interpretation is permitted as a bounded reference path, not an
ordinary production hot path.

## Admission and parity gates

Parsers, macros, agents, elaborators, planners, optimizers, and target backends
are untrusted producers. A small checker admits or rejects their packages with
exact obligations.

A semantic tranche may land only when:

1. its Clause Core representation is host-neutral and canonical;
2. Lean checks its certificate under the constitutional trust profile;
3. Rust agrees on every declared observable and nonfunctional contract;
4. negative fixtures fail for the intended reason;
5. no construct-specific host taxonomy or callback carries hidden meaning;
6. every optimized output is tied to a reference result, certificate, or
   translation-validation witness; and
7. tracked-tree absence checks find no superseded representation or authority.

The bounded [adoption spike](adoption-spike.md) decides whether this mechanism
is viable. A pass authorizes further implementation; it does not prove source
ergonomics, large-graph incrementality, target performance, or maintenance
economics.
