# Clause

Clause is a process-first relational programming language. Humans declare
relationships, laws, transitions, effects, and physical constraints; Clause
judges how recursively structured Terms may run and specializes accepted
meaning into efficient execution.

Its constitutional mechanism is deliberately small:

```text
RawTriple = [Term, Term, Term]
Term      = Atom | RawTriple
Clause    = contextual typed judgment over a Term
Run       = judged carry-through to an outcome, trace, and candidate successor
Admission = the only boundary that makes a successor authoritative
```

Terms hold distinctions; they do not assert or execute themselves. The
admitted judgment graph is the program at rest, Run is the program in motion,
and a trace describes an occurrence without becoming that occurrence.

## Implementation constitution

```text
Clause Core      owns the host-neutral semantic contract.
Compiler0        is one literal externally owner-anchored Clause package.
Lean 4           checks the fixed generic constitution and certificates.
Rust             evaluates the fixed generic machine and physical operations.
Clause           owns source and compiler evolution from genesis.
```

Neither Lean nor Rust may invent a semantic category absent from Clause Core.
Lean syntax is not Clause syntax, Rust types are not Clause ontology, and no
host representation or wire format is independently authoritative.

The repository is at a clean constitutional bootstrap. Lean and Rust
independently implement the narrow CLCP v1 codec, finite ground-certificate
checker, literal proof bootstrap, and predecessor-only package witness against
the shared corpus. That work is evidence, not a Clause compiler.

The P1 contract now specifies CLCP v2 and one future literal `Compiler0`.
Its subject/evidence split prevents evidence from self-hashing or
self-authorizing. One external owner anchor selects the exact genesis bytes;
materialization, hashes, derivability, decoding, and successful execution do
not. Every successor must be compiled and proposed by the already accepted
exact predecessor through two fixed `[Term] -> Term` entrypoints and canonical
request, result, observation, and certificate forms. The fixed host evaluator
has sufficient generic byte inspection, construction, equality, recursion,
and hashing mechanics but remains construct-blind: package data may steer
package-program control, never select a host semantic handler. `Compiler0`
owns reading, binding, elaboration, effects, typed macros, origins,
diagnostics, and evolution as package data. Host-independence is tested by
structure-preserving nominal renaming with all content-derived identities
recomputed, not by pretending hash bytes can be directly permuted.

No CLCP v2 implementation, `Compiler0` package, genesis anchor, supported
Clause source parser, compiler, runtime, durable persistence layer, or
language feature exists yet. Observed implementation status lives only in the
[roadmap](docs/roadmap.md). Git history is not source authority.

## Repository layout

| Path | Authority |
| --- | --- |
| [`docs/foundation.md`](docs/foundation.md) | Clause meaning and minimal calculus |
| [`docs/syntax.md`](docs/syntax.md) | Canonical human-readable source |
| [`docs/architecture.md`](docs/architecture.md) | Implementation and trust boundaries |
| [`docs/canonical-package.md`](docs/canonical-package.md) | CLCP v2 wire contract and implemented CLCP v1 evidence boundary |
| [`docs/compiler-genesis.md`](docs/compiler-genesis.md) | Compiler genesis, succession, and host-freeze contract |
| [`docs/adoption-spike.md`](docs/adoption-spike.md) | Falsifiable constitutional experiment |
| [`docs/roadmap.md`](docs/roadmap.md) | Current implementation status and sequence |
| [`docs/design-evidence.md`](docs/design-evidence.md) | Evidence, alternatives, and uncertainty |
| [`docs/execution-corpus.md`](docs/execution-corpus.md) | Frozen cross-host Run, Admission, and replay observations |
| [`lean/`](lean/) | Lean constitutional-model and trust-gate bootstrap |
| [`crates/clause-substrate/`](crates/clause-substrate/) | Rust physical-substrate bootstrap |
| [`test-vectors/`](test-vectors/) | Shared canonical-package and execution corpora |

Each public fact has one owner. Canonical bytes cannot select authority;
compiler genesis cannot redefine Clause meaning or source syntax; evidence and
the spike cannot add semantics; status lives only in the roadmap.

## Bootstrap checks

```sh
cd lean
lake build
lake env leanchecker --fresh ClauseCore
cd ..
cargo check --workspace --locked
cargo test --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets -- -D warnings
```

Passing these commands proves only the currently implemented CLCP v1
representation, candidate Context/Judgment carriers, relative
finite-certificate checker, literal proof-bootstrap boundary, and independent
Rust corpus parity. They do not implement or prove CLCP v2, the universal
evaluator, `Compiler0`, the external genesis anchor, compiler succession,
Atom canonicality, valid Clause judgments, Runs, general Admission, durable
persistence, or any language feature.

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
