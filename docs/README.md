# Clause documentation

This directory separates current public authority from historical checkpoints.
The root [README](../README.md) is the product entrance; it is not a complete
language specification.

## Current authorities

| Document | Status and authority | Relationship |
| --- | --- | --- |
| [Semantic foundation](foundation.md) | **Current.** Normative for Clause's distinction-first semantic model. | Governs the surface and roadmap. Supersedes foundations built from primitive types, values, objects, fields, functions, or sets. |
| [Target surface](surface.md) | **Current target.** Normative for authoring syntax and canonical formatting; parser migration is incomplete. | Governed by the foundation. Supersedes the ceremonial declaration surface and the historical M0 spellings. |
| [Roadmap](roadmap.md) | **Current.** Normative for implementation sequence, dependency order, and acceptance gates. | Governed by the foundation and target surface. Amends the earlier strategy sequence around the referent kernel and JavaScript/Three.js proof. |
| [Architecture assurance](architecture.md) | **Current derived acceptance contract.** Non-semantic; makes the architecture ratchet executable. | Governed by the foundation, surface, and roadmap. It cannot add ontology, syntax, or milestone scope. |

## History

| Document | Status and authority | Relationship |
| --- | --- | --- |
| [M0 evidence checkpoint](history/m0.md) | **Historical; superseded.** Records an earlier surface-evidence checkpoint and has no authority over current syntax. | Superseded by the current foundation, surface, roadmap, and implemented semantic-v7 referent kernel. |

## Reading routes

- To understand what Clause means, read the [foundation](foundation.md).
- To author or implement syntax, read the [surface](surface.md) under that
  foundation.
- To choose the next engineering checkpoint, read the [roadmap](roadmap.md).
- To understand why some older examples differ, read the
  [historical M0 record](history/m0.md).

## Current surface ruling

```clause
gravity: 9.81
Chess ∈ Game
x = y
input -> result
```

`:` is binding only. `∈` is membership. `=` is equality. `->` is
production/result orientation. An editor may transform typed `::` into `∈`,
but `::` is not grammar. Canonical target indentation is two spaces, spaces
only. The authoritative Model is the program; a Revision is immutable history
and exact lineage evidence about that Model.

## Run and develop

Clause pins Rust 1.96.1. From the repository root:

```sh
cargo run --bin clause -- run examples/hospital.clause
```

The repository-local equivalent is:

```sh
bin/clause run examples/hospital.clause
```

Release checks are:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
bin/architecture-gate
```

Concurrent worktrees must use separate `CARGO_TARGET_DIR` values.
