# Clause

Clause is a distinction-first relational programming language. A Model keeps
stable referents, n-ary role-labelled relations, assertion occurrences,
judgments, rules, constraints, goals, and transitions. When selected as the
authoritative structure to realize, that Model is the program. Source and
generated code are projections; immutable Revisions preserve history and exact
lineage.

Clause has one semantic domain of addressable referents. Types, values,
objects, fields, functions, and sets are useful derived views, not foundational
kinds.

## Surface at a glance

The compact M1 surface is executable. Later compact relation-schema, query,
transition, effect, and target forms remain on the roadmap.

**Compact surface:**

```clause
gravity: 9.81
Chess ∈ Game

iron-door
  Door
  connects Cellar to Armory
  state: locked
```

`:` binds a stable name, `∈` expresses membership, `=` expresses equality,
and `->` orients a result. Editors may turn typed `::` into `∈`, but raw `::`
is not Clause syntax. Target layout uses two spaces and no tabs.

**Self-contained executable profile:**

```clause
Game

catalog
  Chess ∈ Game
```

Bare forms establish the domain and Model from their structure. A caller can
also compile direct top-level forms into an explicitly supplied stable Model
context. Both paths lower into the same referent kernel.

## Implementation status

- The Rust kernel implements stable referents, named-role relational content,
  assertion occurrences, judgments, definitions, rules, laws, invariants,
  goals, and transitions.
- Canonical persistence uses a `clause-semantic-v6` payload inside a
  `clause-revision-v4` envelope. Successor reload requires its exact
  predecessor and verifies the complete Delta.
- `find`, `why`, `prevent`, `achieve`, `diff`, and standalone generated-Rust
  materialization run through the current profile.
- Compact M1 parsing is executable. Compact relation schemas and recursive
  terms, complete transition/effect execution, generated JavaScript, and the
  Three.js game proof remain in development.

## Run Clause

Clause pins Rust 1.96.1. From the repository root:

```sh
cargo run --bin clause -- run examples/hospital.clause
```

The repository-local equivalent is:

```sh
bin/clause run examples/hospital.clause
```

Both commands consume the compact executable profile shown above.

## Documentation

Start with the [documentation guide](docs/README.md), then choose the authority
you need:

- [semantic foundation](docs/foundation.md);
- [target surface](docs/surface.md);
- [implementation roadmap](docs/roadmap.md);
- [historical M0 checkpoint](docs/history/m0.md).

## Development

```sh
cargo fmt --all --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Concurrent lanes use private `CARGO_TARGET_DIR` values.

## License

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
