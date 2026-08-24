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

The compact M1 grounding, binding, membership, enumeration, and focus surface
is executable. Compact relation schemas and recursive terms, complete
state/effect execution, generated JavaScript, and the Three.js game proof are
the next roadmap capabilities.

`:` binds, `∈` expresses membership, `=` expresses equality, and `->` orients
a result. An editor may transform typed `::` into `∈`; raw `::` is not Clause
syntax. Canonical indentation is two spaces, spaces only.

## Start here

- [Documentation guide](docs/README.md) — authority, reading routes, running,
  and development.
- [Semantic foundation](docs/foundation.md) — what Clause means.
- [Target surface](docs/surface.md) — canonical authoring and formatting.
- [Implementation roadmap](docs/roadmap.md) — dependency order and exit proofs.
- [Historical M0 checkpoint](docs/history/m0.md) — superseded evidence and
  evolution context.

Run the current hospital program from the repository root:

```sh
cargo run --bin clause -- run examples/hospital.clause
```

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
