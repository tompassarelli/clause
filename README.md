# Clause

Clause is a distinction-first relational programming language. It keeps
semantic identity, relational content, assertion occurrences, judgments,
program history, and runtime state separate so each can carry exact provenance
without impersonating another layer.

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
| What does the provisional JavaScript host do today? | [Host README](host/README.md) |

These documents have disjoint authority. A contradiction is a documentation
defect; there is no “newer file wins” rule and no separate historical document
competing with the live public set.

## Current state

Clause now implements the ratified Program snapshot/history boundary, typed
checked-snapshot compilation, immutable RuntimeSession identity, causal
StateRevision identity, strict runtime-v3 replay/reload, and ProgramRevision-
bound render/effect evidence while preserving semantic-v10 / Revision-v6
bytes. Canonical agent-first syntax, the complete generated live host, three
applications, and the preregistered comparison remain unfinished. The
[roadmap](docs/roadmap.md) is the current status record; the
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
