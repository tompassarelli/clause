# Clause

Clause is a distinction-first relational programming language. Its program is
an authoritative Model of stable referents and role-labelled relations;
immutable Revisions preserve history and exact lineage.

```clause
gravity: 9.81
Chess ∈ Game

iron-door
  Door
  connects Cellar to Armory
  state: locked
```

The compact M1 grounding, binding, membership, enumeration, and focus surface,
M2's exact role-labelled relation schemas, and M3's recursive terms, checked
structural values, pure definitions, and source-deleted generated evaluation
are executable. M4 adds recursive relational holes, explicit query cardinality,
and separately authorized law-backed derivation. M6 adds deterministic authored
events, state transitions, incremental successor state, and replay. The bounded
M7 path now carries explicit effect traces, canonical RenderPlan ESM snapshots,
and grounded StateRevision scene projection. Dedicated scene syntax, live
generated-JavaScript transitions, real browser/Three.js execution, source maps,
and the full M7 proof remain roadmap capabilities.

`:` binds, `∈` expresses membership, `=` expresses equality, and `->` orients
a result. An editor may transform typed `::` into `∈`; raw `::` is not Clause
syntax. Canonical indentation is two spaces, spaces only.

## Start here

- [Documentation guide](docs/README.md) — authority, reading routes, running,
  and development.
- [Semantic foundation](docs/foundation.md) — what Clause means.
- [Target surface](docs/surface.md) — canonical authoring and formatting.
- [Current executable syntax](docs/current-syntax.md) — what the checked
  frontend accepts today and what remains target-only.
- [Implementation roadmap](docs/roadmap.md) — dependency order and exit proofs.
- [Historical M0 checkpoint](docs/history/m0.md) — superseded evidence and
  evolution context.

Run the current hospital program from the repository root:

```sh
cargo run --bin clause -- run examples/hospital.clause
```

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
