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

The semantic foundation governs one host-neutral Clause Core contract, and that
contract is sovereign over every implementation. The accepted bootstrap
direction is deliberately split:

```text
Lean 4  checks proof-bearing Clause Core packages and hosts the reference Run model.
Rust    persists and executes accepted meaning through optimized physical machinery.
Clause  progressively authors its own elaboration, transformations, and compiler middle.
```

Lean is not Clause's source language, ontology, or wire format, and Rust is not
allowed to mint semantic categories that the host-neutral core cannot express.
Both implementations must consume one canonical Clause-owned package and agree
on observable meaning. The adoption spike may still falsify Lean's suitability
or the Term kernel itself before either becomes part of the supported line.

Clause's accepted semantic vocabulary is:

- a **Program** is a durable evolving lineage;
- a **ProgramSnapshot** is one exact immutable checked semantic value;
- a **ProgramRevision** is one immutable causal history node selecting a
  snapshot; and
- a **Model** is reserved for a satisfying meta-level interpretation, not an
  authored source block or executable program artifact.

The Rust implementation predates that vocabulary and still contains frozen
migration types named `kernel::Model` and `kernel::Revision`. Checked snapshots,
Program history, and runtime-v3 identity now cross that bridge through explicit
typed adapters rather than relabelling the legacy identity. The accepted design
is authoritative; the [architecture](docs/architecture.md) and
[roadmap](docs/roadmap.md) state exactly what remains to migrate.

## Documentation authority

Each public fact has one owner:

| Question | Authority |
| --- | --- |
| What does Clause mean? | [Semantic foundation](docs/foundation.md) |
| What is canonical Clause source? | [Syntax](docs/syntax.md) |
| What does the current implementation enforce, and how does it map to the accepted design? | [Architecture](docs/architecture.md) |
| What is implemented, partial, active, or pending? | [Roadmap](docs/roadmap.md) |
| What experiment can falsify the new kernel? | [Adoption spike](docs/adoption-spike.md) |
| What evidence and uncertainty motivated it? | [Design evidence](docs/design-evidence.md) |
| What does the provisional JavaScript host do today? | [Host README](host/README.md) |

The foundation, syntax, architecture, roadmap, and host README have disjoint
authority. The adoption spike and evidence ledger are subordinate records:
neither can add semantics, syntax, or status. A contradiction is a
documentation defect; there is no “newer file wins” rule and no separate
historical document competing with the live public set.

## Current state

Clause now implements the ratified Program snapshot/history boundary, typed
checked-snapshot compilation, immutable RuntimeSession identity, causal
StateRevision identity, strict runtime-v3 replay/reload, and ProgramRevision-
bound render/effect evidence while preserving semantic-v10 / Revision-v6
bytes. It does not yet implement the process-first Term/Clause/Run kernel,
canonical Clause Core package, or Lean constitutional checker; the
[adoption spike](docs/adoption-spike.md) is the next mechanism decision.
Canonical agent-first syntax, the complete generated live host, three
applications, and the preregistered comparison also remain unfinished. The
[roadmap](docs/roadmap.md) is the current status record, and the
[syntax migration ledger](docs/syntax.md#implementation-migration) is the one
place that contrasts canonical source with the legacy parser.

The current CLI can run the checked hospital example:

```sh
cargo run --bin clause -- run examples/hospital.clause
```

That example intentionally remains on the currently executable legacy surface
until migration can preserve its exact identities and results.

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
