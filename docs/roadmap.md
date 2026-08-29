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
- a provisional Lean 4 model containing indexed Atom/Term representations,
  generic candidate Context/Judgment carriers, a relative finite ground-
  certificate checker and soundness theorem, an exact but deliberately
  uninhabited package-anchor boundary, and a bounded trust audit; and
- an empty Rust crate for the physical substrate.

There is no supported parser, compiler, runtime, persistence format, CLI,
backend, or example application. No implemented language capability is
claimed. Git history is not source authority or a design template.

The next decision is the literal initial constitutional package and basis. The
current exact package-binding boundary intentionally supplies no such anchor;
candidate data therefore cannot select its own authority. Successor-basis
admission remains blocked until a canonical basis-admission claim exists. Only
after an independently reviewed initial anchor may Atom equality contracts be
derived.

## Status summary

| Work | Status | Exit boundary |
| --- | --- | --- |
| Semantic foundation | Accepted constitutional hypothesis | [Foundation](foundation.md) |
| Canonical source design | Accepted; unimplemented | [Syntax](syntax.md) |
| Repository reset | Complete | Live tree contains only current documents, licenses, and current package roots |
| Lean package bootstrap | Complete | Lean 4.33.1 build at trust level zero, bounded declaration audit, and same-kernel replay |
| Rust substrate bootstrap | Scaffolded | Empty workspace builds with pinned Rust toolchain and forbids unsafe code |
| Clause Core calculus | In progress; provisional Term, candidate Context/Judgment, relative finite ground-certificate checking, and an uninhabited exact package-anchor seam | Admitted Atom/Term equality plus generic Context/Judgment/Run/Admission with no feature taxonomy |
| Canonical package and certificates | In progress; exact whole-record binding only | One host-neutral codec, literal independently reviewed bootstrap anchor, exact byte vectors, and package-bound evidence |
| Constitutional adoption spike | Pending | All eight gates, negative evidence, Lean/Rust parity, and host freeze |
| Clause-authored compiler middle | Blocked on spike | Stable proposal machinery moves into Clause |
| Product gates | Blocked on spike | Readability, incrementality, native/JS performance, systems coverage, and maintenance evidence |

## Phase 0 — Clean reset

**Status:** Complete.

The reset established a supported line containing only current constitutional
documents, licenses, and semantic-empty Lean/Rust package roots. Repository-wide
absence checks covered every tracked source, test, example, document, generated
consumer, host, and build or release file. Phase 1 then began from that exact
boundary.

**Exit evidence:** tracked-tree census, documentation checks, and a successful
empty Rust workspace build. The first Lean build is the entry check for
Phase 1 because the pinned release toolchain was not locally available during
the reset.

## Phase 1 — Constitutional calculus

**Status:** In progress.

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

The current provisional model contains:

- canonical-byte candidates and explicit universe/semantics indexes;
- generic Atom kind and equality-contract references with no host callback;
- exactly two Term constructors: Atom and a neutral three-Term Triple;
- recursive candidate representation comparison, including explicit
  cross-index rejection;
- no identity field on Triple, while all nominal meaning remains deferred to a
  future Clause judgment; and
- generic, index-bound Judgment claims whose semantic fields remain ordinary
  Terms, candidate Contexts enumerating claims, and proposed contextual
  judgments pairing the two;
- representation-only candidate premise lookup that grants no derivation or
  authority;
- generic ground rules, separately supplied root/rule bases, and topologically
  ordered finite certificate traces;
- an executable one-pass checker whose successful result has a kernel-checked
  proof of `DerivableFrom` the exact supplied basis; and
- positive shared-DAG evidence plus rejection of empty, missing, mismatched,
  duplicate node-address, self-referential, forward, mutual-cycle,
  altered-target, and self-authorizing all-equal specimens;
- one candidate package record binding raw canonical-byte candidates, exact
  structural index, decoded basis/certificate/target sections, and all
  auxiliary content by whole-record equality; and
- a closed, currently uninhabited constitutional-anchor predicate plus a
  theorem yielding only package-bound relative derivability when exact binding,
  external authority, and relative checking all hold.

It deliberately does not yet define or claim a valid Context, a valid Clause
Judgment relation, an inhabited constitutional anchor or accepted basis,
successor-basis admission, schematic rule formation or substitution, Mode,
RunOutcome, Run, Delta, Trace, Admission, Revision, Atom-contract validation,
semantic structural equality, nominal identity, canonical decoding, or a
canonical codec. Relative or package-bound derivability is not semantic truth
or general Admission. This source remains non-authoritative bootstrap evidence
rather than an admitted semantic tranche under the architecture's admission
and parity gates.

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

**Current evidence:** the pinned Lean 4.33.1 provisional Term,
Context/Judgment, relative certificate, and exact package-binding model builds
with `-t0` and warnings as errors. Its examples independently alter package
bytes, semantics epoch, decoded auxiliary content, roots, rule content and
order, rule premises and conclusion, certificate, and target; every mutation
breaks exact binding. Cross-package certificate transplant, self-declared
roots, nullary self-rules, all-equal candidate content, raw Context membership,
and bare `DerivableFrom` remain unauthorized. A build-time environment audit
rejects every unsafe declaration, every partial declaration except the two
exact generated runtime helpers for `Term.sameRepresentation` and finite
premise-reference matching, foreign/replacement implementations, and axioms
outside the explicit `propext` policy.
`leanchecker --fresh ClauseCore` successfully replays the safe/total
declarations through the same Lean kernel and excludes those enumerated runtime
helpers. This is not an inhabited package or basis acceptance, canonical codec,
semantic admission, the full transitive trust closure, Rust parity, or Phase 1
completion.

**Exit evidence:** safe/total completed calculus; no `sorry`, authored
`unsafe` or `partial`, feature-specific syntax constructor, or unlisted axiom
in the constitutional closure.

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
