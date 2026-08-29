# Clause Roadmap

> **Status:** Current.
>
> **Authority:** Sole authority for implementation status, dependency order,
> milestone scope, and exit evidence. The [foundation](foundation.md) governs
> semantics, [syntax](syntax.md) governs canonical source, and
> [architecture](architecture.md) governs implementation boundaries.

## Current position

Clause is at a clean constitutional bootstrap with a compiler-genesis contract.
The live repository contains:

- the accepted process-first semantic foundation;
- the canonical source design;
- a provisional Lean 4 model containing indexed Atom/Term representations,
  generic candidate Context/Judgment carriers, a relative finite ground-
  certificate checker and soundness theorem, the strict CLCP v1 codec, the
  literal bootstrap and predecessor-authorized successor, exact executable
  vectors, and a bounded trust audit; and
- a semantic-empty Rust physical substrate with an independent strict CLCP v1
  codec, exact-byte carrier, relative certificate checker, predecessor-only
  package authorization witness, and shared-corpus tests; and
- the P1 contract for strict CLCP v2, one externally owner-anchored literal
  `Compiler0`, a fixed construct-blind universal evaluator, exact
  predecessor-only succession, and Clause-owned source/compiler behavior.

There is no supported parser, compiler, runtime, persistence format, CLI,
backend, or example application. In particular, there is no CLCP v2 codec,
universal evaluator, literal `Compiler0`, genesis anchor, v2 certificate
checker, or compiler-evolution artifact. No implemented language capability is
claimed. Git history is not source authority or a design template.

The shared v0 execution corpus now fixes three substantial implementation
targets: recursive pure dependency closure, a two-phase admitted State/effect
protocol, and predecessor-bound Program evolution with two isolated replay
runs. These are source and observation fixtures only. They do not make a
parser, Run relation, Admission relation, effect adapter, or replay command
implemented.

The CLCP v1 literal proof bootstrap and its one preauthorized successor are
fixed by exact bytes. Lean and Rust independently decode and re-encode the same
positive corpus and reject the recorded negative classes. Those artifacts are
narrow constitutional evidence, not `Compiler0` and not an implementation of
the v2 contract. Atom equality contracts and broader resource/fuzz evidence
remain later tranches.

## Status summary

| Work | Status | Exit boundary |
| --- | --- | --- |
| Semantic foundation | Accepted constitutional hypothesis | [Foundation](foundation.md) |
| Canonical source design | Accepted; unimplemented | [Syntax](syntax.md) |
| Repository reset | Complete | Live tree contains only current documents, licenses, and current package roots |
| Lean package bootstrap | Complete | Lean 4.33.1 build at trust level zero, bounded declaration audit, and same-kernel replay |
| Rust substrate bootstrap | Canonical-package codec complete; broader substrate scaffolded | Pinned Rust toolchain passes formatting, all-target checks/tests, Clippy, and forbids unsafe code |
| Clause Core calculus | In progress; provisional Term, candidate Context/Judgment, relative ground-certificate checking, and narrow package authority | Admitted Atom/Term equality plus generic Context/Judgment/Run/Admission with no feature taxonomy |
| CLCP v1 proof package | Lean/Rust codec and frozen-corpus parity complete; evidence only | Published exact corpus, strict nested decoding, byte-identical positive re-encoding, and matched negative verdict classes |
| P1 compiler-genesis contract | Specified; implementation evidence absent | [Compiler genesis](compiler-genesis.md) and [CLCP v2](canonical-package.md) agree on exact authority and host boundaries |
| CLCP v2 generic hosts and literal Compiler0 | Pending | Independent strict codecs, fixed generic evaluator/checker, exact owner anchor, and shared positive/negative corpus |
| Constitutional adoption and evolution | Blocked on CLCP v2 and Compiler0 | All eight spike gates plus one four-change predecessor-authorized Compiler1 and frozen hosts |
| Clause-authored compiler behavior | Contracted from genesis; implementation pending | Reading, binding, elaboration, effects, macros, origins, diagnostics, and evolution execute from the accepted package |
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
  structural index, exact lineage, decoded basis/certificate/target sections,
  and all auxiliary content by strict decode and byte-for-byte re-encoding;
- one total U8/U32 canonical codec with exact frame and EOF consumption;
- the exact 334-byte literal bootstrap and 681-byte authorized successor;
- an injective ordinary-Term basis-admission claim over the exact next INDEX
  and BASIS frames; and
- a two-constructor authority relation plus a theorem yielding only relative
  derivability from the exact packaged basis.

It deliberately does not yet define or claim a valid Context, a valid Clause
Judgment relation, schematic rule formation or substitution, Mode, RunOutcome,
Run, Delta, Trace, general Admission, Revision, Atom-contract validation,
semantic structural equality, or nominal identity. Narrow package authority
and relative derivability are not semantic truth or general Admission.

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

**Current evidence:** the pinned Lean 4.33.1 Term, Context/Judgment, relative
certificate, canonical codec, literal authority, and executable vector model
builds with `-t0` and warnings as errors. The exact positive corpus decodes and
authorizes; wrong magic/version/order/tag/length, truncation, and trailing bytes
reject. Bound-field, predecessor, authorization, self/cycle, cross-index,
Context, nullary-rule, and bare-derivability adversaries retain the intended
separate decode, binding, relative-check, and authority verdicts. A build-time
environment audit covers the core and vectors and rejects every unsafe
declaration, every partial declaration except seven exact compiler-generated
runtime helpers for total recursion, foreign/replacement implementations, and
axioms outside the explicit `propext` policy.
`leanchecker --fresh ClauseCore` successfully replays the safe/total
declarations through the same Lean kernel and excludes those enumerated runtime
helpers. This is narrow v0 package authority, not semantic Admission, the full
transitive trust closure or Phase 1 completion.

**Exit evidence:** safe/total completed calculus; no `sorry`, authored
`unsafe` or `partial`, feature-specific syntax constructor, or unlisted axiom
in the constitutional closure.

## Phase 2 — CLCP v1 proof package and Rust parity

**Status:** Complete for the frozen CLCP v1 proof package and corpus boundary;
not compiler-genesis completion.

The Clause-owned CLCP v1 grammar, literal bootstrap, one authorized successor,
Lean decoder, independent Rust decoder, and canonical positive/negative corpus
are fixed. Both implementations strictly validate nested predecessors,
byte-identically re-encode the positives, and reproduce the recorded decode,
exact-binding, certificate, and authority boundaries.

The Rust crate may add only physical decoding, indexing, persistence, and
execution mechanics. It may not reproduce the semantic checker through Rust
enums or construct-specific pattern matching.

**Exit evidence:** byte-exact cross-host round trips, package-bound Lean
certificate checking, Rust/Lean observable parity, and dependency scans showing
no hidden semantic authority.

## Phase 2a — P1 compiler-genesis contract

**Status:** Specified; implementation and independent acceptance evidence are
pending.

The [compiler-genesis contract](compiler-genesis.md) and
[canonical-package contract](canonical-package.md) define:

- one literal externally owner-anchored `Compiler0`;
- CLCP v2 with separate subject and evidence frames;
- a fixed universal `Bytes`/`Term` evaluator and generic Lean rules;
- exact predecessor-only `compile` plus `admit-propose` succession;
- Clause ownership of reading, binding, elaboration, effects, typed macros,
  origins, diagnostics, and compiler evolution from genesis; and
- the allowed-host-discriminant information-flow rule and
  identifier-permutation equivariance law.

This phase changes contracts only. It does not claim a v2 package, compiler,
codec, checker, evaluator, fixture, host audit, or measurement.

**Exit evidence:** one internally linked contract with no host-language
semantic authority and explicit residual tractability uncertainty.

## Phase 2b — CLCP v2 genesis implementation

**Status:** Pending on accepted Phase 2a.

Implement independent strict CLCP v2 decoding, the fixed generic Lean checker
and certificate rules, the fixed generic Rust evaluator and physical profile,
one literal `Compiler0`, its external exact-byte owner anchor, and
stage-separated positive and adversarial vectors. Materialization provenance
is recorded as untrusted evidence and cannot create the anchor.

This phase must measure evaluator and certificate tractability before adding
proof compression or checked optimization. It must not add a Rust or Lean
Clause frontend, construct dispatch, third-language semantic bootstrap, or
candidate/self authorization.

**Exit evidence:** exact-byte genesis acceptance; independent decode/re-encode
parity; generic evaluation/check parity; required negative verdicts; checked
host-branch manifest; and observed timings for the narrow compiler request.

## Phase 3 — Dangerous semantic gates

**Status:** Blocked on Phase 2b.

The cross-host source/observation corpus for pure computation, State/effects,
and verified Program evolution is frozen in
[`test-vectors/execution`](../test-vectors/execution). Lean reference semantics,
Rust execution, public tooling, and isolated replay remain pending.

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

The decisive evolution case is one ordinary `Compiler0 -> Compiler1`
succession changing a binding form, an effect form, a typed macro, and
diagnostic behavior. The same previously built Lean and Rust binaries,
toolchains, and host-branch manifest must remain byte-identical. Any candidate
or self basis, hash-only predecessor match, or host construct branch
falsifies the architecture.

## Phase 4 — First Clause-owned compilation

**Status:** Blocked on a passing Phase 3.

Exercise the compiler behavior already owned by the accepted package in this
order:

1. schemas and relation modes;
2. elaboration and macros;
3. obligations and diagnostics;
4. semantic queries, impact analysis, and refactoring;
5. planning and source projection; and
6. compiler orchestration and selected lowering.

Lean remains the independent generic constitutional checker. Rust remains the
generic evaluator and replaceable physical machinery. Neither host grows an
ordinary-language feature taxonomy.

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
