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
Lean 4           checks constitutional evidence and hosts reference Run semantics.
Rust             implements replaceable physical persistence, runtime, FFI, and backends.
Clause           progressively authors its own compiler middle.
```

Neither Lean nor Rust may invent a semantic category absent from Clause Core.
Lean syntax is not Clause syntax, Rust types are not Clause ontology, and no
host representation or wire format is independently authoritative.

The repository is at a clean constitutional bootstrap. The Lean package now
contains provisional indexed Atom/Term representations, generic candidate
Context/Judgment carriers, and a finite ground-certificate checker whose
soundness theorem establishes derivability only relative to a separately
supplied basis. The normative canonical-package grammar and corpus now freeze
one literal bootstrap and one predecessor-authorized successor. Lean provides
the strict reference decoder, literal authority relation, and executable trust
vectors; Rust independently implements the same strict physical codec and
predecessor-only authorization witness against the shared corpus. Decoding,
candidate package fields, raw Context membership, and bare relative
derivability cannot create authority, and the narrow package relation is not
general Clause Admission or semantic truth. There is still no supported Clause
source parser, compiler, runtime, durable persistence layer, or language
feature. Only the current foundation and host-neutral contract may define
their semantics. Git history is not source authority.

## Repository layout

| Path | Authority |
| --- | --- |
| [`docs/foundation.md`](docs/foundation.md) | Clause meaning and minimal calculus |
| [`docs/syntax.md`](docs/syntax.md) | Canonical human-readable source |
| [`docs/architecture.md`](docs/architecture.md) | Implementation and trust boundaries |
| [`docs/adoption-spike.md`](docs/adoption-spike.md) | Falsifiable constitutional experiment |
| [`docs/roadmap.md`](docs/roadmap.md) | Current implementation status and sequence |
| [`docs/design-evidence.md`](docs/design-evidence.md) | Evidence, alternatives, and uncertainty |
| [`docs/execution-corpus.md`](docs/execution-corpus.md) | Frozen cross-host Run, Admission, and replay observations |
| [`lean/`](lean/) | Lean constitutional-model and trust-gate bootstrap |
| [`crates/clause-substrate/`](crates/clause-substrate/) | Rust physical-substrate bootstrap |
| [`test-vectors/`](test-vectors/) | Shared canonical-package and execution corpora |

Each public fact has one owner. Evidence and the spike cannot add semantics;
architecture cannot redefine syntax; status lives only in the roadmap.

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

Passing these commands proves only that the provisional representation,
candidate Context/Judgment carriers, and relative finite-certificate checker
and literal predecessor-authorized package boundary build at trust level zero,
pass the bounded declaration audit, and replay their safe/total declarations
through the same Lean kernel, while the independent Rust physical codec passes
the shared corpus and adversarial authority cases. They do not prove Atom
canonicality, semantic structural equality, valid Clause judgments, Runs,
general Admission, durable persistence, or any language feature; observed
status lives in the [roadmap](docs/roadmap.md).

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
