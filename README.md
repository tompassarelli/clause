# Clause

Clause is a general-purpose relational language under development for programs
whose rules must drive both behavior and answers about that behavior. In the
scheduling example, task prerequisites determine what is blocked, what can
complete, and what changes when an obstruction is removed. The application
need not maintain a second dependency model to answer those questions.

The goal is for domain owners to write and audit the specification as the
program itself, using precise domain phrases and familiar mathematics. They
should be able to extend its vocabulary and sentence shapes through checked
declarations, with every domain using the same semantic core.

Clause combines typed relationships, derivation laws, atomic change proposals,
and separate admission of those changes. Its
[design discipline](docs/foundation.md#design-discipline) is to author each
independent semantic fact once: checking, execution, queries, explanations,
and editing must consume that same meaning.

Start with the [runnable scheduling walkthrough](docs/language-tour.md#run-the-scheduling-example),
then read the rest of the [language tour](docs/language-tour.md). Use the
[authoring card](docs/authoring-card.md) for checked source examples, the
[foundation](docs/foundation.md) for the distinctions behind them, and the
[syntax](docs/syntax.md) when writing source. Read the
[architecture](docs/architecture.md) when working on the implementation.
The [roadmap](docs/roadmap.md) separates demonstrated capabilities from
remaining work; Clause is not yet a supported language or toolchain.

## Governing distinctions

- A name denotes a value. A role says what that value means relative to a
  subject. A contract constrains it. Representation determines its encoding.
- Equal values may have different occurrences and independent support.
- A description does not assert, execute, or authorize itself.
- A relation's meaning is distinct from the directions in which it can be
  computed. Each direction must state its result and failure guarantees.
- Exhausted search proves neither absence nor falsehood.
- An edit preserves identity only through checked continuity, not similar text.
- Local computation needs no governed revision. Authoritative shared changes
  and external effects require the authority appropriate to their scope.

The [semantic foundation](docs/foundation.md) defines these distinctions.

## Examples

- [Scheduling walkthrough](docs/language-tour.md#run-the-scheduling-example):
  task dependencies, derived blockers, and checked changes.
- [Jump Arena](https://github.com/tompassarelli/jump-arena): a standalone
  browser game with its own source and pinned Clause dependency.

## Documentation

| Document | Owns |
| --- | --- |
| [Language tour](docs/language-tour.md) | Examples and concepts |
| [Semantic foundation](docs/foundation.md) | Meaning, identity, effects, and authority |
| [Syntax](docs/syntax.md) | Source grammar |
| [Architecture](docs/architecture.md) | Compiler, runtime, host, and trust boundaries |
| [Canonical packages](docs/canonical-package.md) | Exact transport and compiler-machine contracts |
| [Compiler genesis](docs/compiler-genesis.md) | Initial authority and compiler succession |
| [Adoption spike](docs/adoption-spike.md) | Executable tests of the design |
| [Roadmap](docs/roadmap.md) | Implementation status and remaining work |

## Development

From an owned Clause worktree, run the focused example first:

```sh
nix develop --command cargo test -p clause-workbench --locked -j 2 --test scheduling scheduling_controls_use_checked_dependencies_and_preserve_identity -- --exact
```

The [walkthrough](docs/language-tour.md#run-the-scheduling-example) explains its
source and assertions. Existing [live-source commands](docs/live-source-semantics.md#reproduce-the-bounded-gates)
cover the larger native/Wasm journey. Broad implementation checks, when needed:

```sh
(cd lean && lake build && lake env leanchecker --fresh ClauseCore)
cargo test --workspace --locked --all-targets
```

Use focused tests while developing. The roadmap records known failures and
the scope of observed results.

Available under the [MIT License](LICENSE-MIT) or
[Apache License, Version 2.0](LICENSE-APACHE).
