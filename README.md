# Clause

Clause is a general-purpose relational language under development.

Its [design discipline](docs/foundation.md#design-discipline): author each
independent semantic fact once.
Checking, execution, queries, explanations, and editing must use that same
meaning. Physical implementations may specialize it, not redefine it.

The [authoring card](docs/authoring-card.md) describes executable source;
the [roadmap](docs/roadmap.md) records unfinished work. Design examples are
not implementation claims.

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

## A Clause transition

Collecting a coin changes its state only when it is active and owned by the actor:

```clause
on collect ?actor
  when
    ?coin state active
    ?coin owner ?actor
  withdraw
    ?coin state active
  include
    ?coin state collected
```

The conditions read one pre-state. Removal and addition form one proposed
change; admitting it is a separate operation. Two collection events remain
distinct even if their values are equal.

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

Broad implementation checks:

```sh
(cd lean && lake build && lake env leanchecker --fresh ClauseCore)
cargo test --workspace --locked --all-targets
```

Use focused tests while developing. The roadmap records known failures and
the scope of observed results.

Available under the [MIT License](LICENSE-MIT) or
[Apache License, Version 2.0](LICENSE-APACHE).
