# Clause Roadmap

> **Status:** Current.
>
> **Authority:** Sole authority for implementation status, dependency order,
> milestone scope, and exit evidence. The [foundation](foundation.md) governs
> semantics, [syntax](syntax.md) governs canonical source, and
> [architecture](architecture.md) governs implementation boundaries.

## Current position

Clause is at a clean constitutional bootstrap. The live repository contains:

- the accepted process-first semantic foundation;
- the canonical source design;
- an empty Lean 4 library for the constitutional checker/reference model; and
- an empty Rust crate for the physical substrate.

There is no supported parser, compiler, runtime, persistence format, CLI,
backend, or example application. No implemented language capability is
claimed. Git history is not source authority or a design template.

The next decision is whether the minimal Clause calculus can be encoded,
checked, run, and exchanged without Lean or Rust becoming a second semantic
authority.

## Status summary

| Work | Status | Exit boundary |
| --- | --- | --- |
| Semantic foundation | Accepted constitutional hypothesis | [Foundation](foundation.md) |
| Canonical source design | Accepted; unimplemented | [Syntax](syntax.md) |
| Repository reset | Complete | Live tree contains only current documents, licenses, and semantic-empty package roots |
| Lean package bootstrap | Scaffolded; build unverified | Stable toolchain declared; first successful `lake build` still required |
| Rust substrate bootstrap | Scaffolded | Empty workspace builds with pinned Rust toolchain and forbids unsafe code |
| Clause Core calculus | Pending; next | Generic Atom/Term/Judgment/Run/Admission model with no feature taxonomy |
| Canonical package and certificates | Pending | One host-neutral codec, exact byte vectors, and package-bound evidence |
| Constitutional adoption spike | Pending | All eight gates, negative evidence, Lean/Rust parity, and host freeze |
| Clause-authored compiler middle | Blocked on spike | Stable proposal machinery moves into Clause |
| Product gates | Blocked on spike | Readability, incrementality, native/JS performance, systems coverage, and maintenance evidence |

## Phase 0 — Clean reset

**Status:** Complete when this change lands.

The supported line contains only current constitutional documents, licenses,
and semantic-empty Lean/Rust package roots. Repository-wide absence checks
cover every tracked source, test, example, document, generated consumer, host,
and build or release file.

**Exit evidence:** tracked-tree census, documentation checks, and a successful
empty Rust workspace build. The first Lean build is the entry check for
Phase 1 because the pinned release toolchain was not locally available during
the reset.

## Phase 1 — Constitutional calculus

**Status:** Pending; next.

Define in Lean the smallest host-neutral model required by the foundation:

```text
Atom
Term = Atom | [Term, Term, Term]
Context
Judgment
Mode
RunOutcome
Run
Delta
Trace
Admission
Revision
```

The model must keep structural equality, value equality, denotational
equivalence, occurrence identity, entity identity, concept continuity, and
revision identity distinct. Clause is a judgment over a Term, not a constructor.
Run is a relation, not a total host function. A trace is data about a Run, not
the occurrence itself.

Initial proofs protect only constitutional boundaries:

- Term construction grants no assertion, execution, or authority;
- accepted judgments are well-formed;
- pure Runs preserve the authoritative context;
- admitted candidates preserve context validity;
- deterministic modes have at most one returned result;
- effects require declared capability;
- intent, attempt, receipt, observation, and evidence remain distinct; and
- host handles cannot become Clause identity.

**Exit evidence:** safe/total Lean build; no `sorry`, `unsafe`, `partial`,
feature-specific syntax constructor, or unlisted axiom in the constitutional
closure.

## Phase 2 — Canonical package and Rust parity

**Status:** Blocked on Phase 1.

Specify one Clause-owned canonical package and byte encoding. Build independent
Lean and Rust decoders from that specification. Freeze canonical positive and
negative vectors covering Atom policy, structural Terms, identity anchors,
contexts, judgments, Runs, deltas, traces, certificates, and tamper rejection.

The Rust crate may add only physical decoding, indexing, persistence, and
execution mechanics. It may not reproduce the semantic checker through Rust
enums or construct-specific pattern matching.

**Exit evidence:** byte-exact cross-host round trips, package-bound Lean
certificate checking, Rust/Lean observable parity, and dependency scans showing
no hidden semantic authority.

## Phase 3 — Dangerous semantic gates

**Status:** Blocked on Phase 2.

Run the eight cases in [adoption-spike.md](adoption-spike.md):

1. pure evaluation;
2. binding and closure;
3. algebraic data and exhaustive matching;
4. structural and nominal higher arity;
5. recursive derivation and honest non-total modes;
6. State and effect Runs;
7. typed hygienic macro expansion; and
8. a frozen-host extension combining binding, effects, transition, and custom
   readable syntax.

Every case uses generic Terms, schemas, judgments, modes, and certificates. A
new per-feature Lean or Rust semantic branch falsifies the architecture.

## Phase 4 — Clause authors the middle

**Status:** Blocked on a passing spike.

Move stable capabilities into Clause in this order:

1. schemas and relation modes;
2. elaboration and macros;
3. obligations and diagnostics;
4. semantic queries, impact analysis, and refactoring;
5. planning and source projection; and
6. compiler orchestration and selected lowering.

Lean remains the independent constitutional checker. Rust remains replaceable
physical machinery. Neither host grows an ordinary-language feature taxonomy.

## Phase 5 — Canonical source and tooling

**Status:** Blocked on Phases 3–4.

Implement the grammar in [syntax.md](syntax.md) as a lossless source projection
with deterministic readings, exact focus, binding and origin preservation,
canonical printing, local recovery, and semantic round trips.

Files are transport containers, names are readable designations, and source
order is not causality unless represented explicitly. Text diff is never the
authoritative program diff.

## Phase 6 — Physical systems and product gates

**Status:** Blocked on the constitutional mechanism.

Measure independently:

- large-graph incremental dependency precision, time, and memory;
- matched reference/native/Wasm/JavaScript behavior and performance;
- ownership, concurrency, cancellation, failure, and security semantics;
- packages, modules, separate compilation, FFI, ABI, and deployment;
- real source ergonomics and comprehension; and
- correct-change throughput and maintenance cost.

A mechanism pass does not waive these gates. Generic Triple interpretation may
remain a bounded oracle but cannot be the production hot path.

## Completion standard

A roadmap item is complete only when its authoritative representation,
identity rules, diagnostics, canonical encoding where applicable, executable
behavior, negative cases, and narrow exit proof land together. Documentation
specimens are not implementation evidence. A successful Lean evaluation
without a kernel-checked package-bound proof proves no Clause admission. A Rust
result without parity and traceability proves no Clause meaning.

Every change leaves one live architecture. Superseded source, tests, docs,
fixtures, generated artifacts, and consumers are removed in the same change.
