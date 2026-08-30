# Clause

Clause is a process-first relational programming language. Its source describes
applications, relationships, laws, transitions, and effects; its semantics
defines how typed activations actually run. The Clause Graph is the canonical
inspectable shape of that process, while checked physical implementations may
specialize it into efficient native, Wasm, browser, or data-system execution.
A lower-case clause is a contextual judgment over a neutral Term, not an
Application, assertion, or runtime event.

The target is a general-purpose language for agent-authored software, not a
governed-state niche. Ordinary local mutation needs no Admission or
StateRevision. Modes declare only the dynamic prerequisites their Activations
actually need; effect Modes choose strict per-intent governance or bounded
preauthorized local/session/Lease/batch scope, and Admission keeps its own
authority boundary.
Causal-affine reclamation has no automatic Clause tracing/ARC/finalizer
fallback. Generic reuse, deterministic lifetimes, and direct native/Wasm
lowering are early falsifiers, not implemented capability.

> **Current status:** Clause has an accepted semantic constitution and canonical
> source design, but no supported parser, compiler, runtime, CLI, Wasm boundary,
> renderer integration, or example application yet. The
> [roadmap](docs/roadmap.md) is the sole implementation-status authority.

## A Clause transition

This canonical source form describes a complete transition from a pinned state
observation to a candidate delta:

```clause
on collect ?actor
  when
    ?coin state active
    ?coin owner ?actor
  withdraw
    ?coin state active
  admit
    ?coin state collected
```

The spelling is canonical but is not yet runnable by a supported toolchain.
`on` declares process constitution; it does not execute merely by being stored.
A matching occurrence can activate one exact Application under a selected Mode
and context. `when` observes one StateRevision, while `withdraw` and source
`admit` stage a candidate delta. Only a separate governed Admission can create
the successor StateRevision.

The semantic path is:

```text
neutral Term
  -> checked, closed ApplicationForm
  -> nominal Application
  -> Activation from one exact ActivationStartRecord: static basis, context,
     Mode-declared prerequisite bindings, and a separate occurrence frontier
  -> causal Steps within one Run
  -> observations, results, continuations, and candidate deltas
  -> governed Admission, when authoritative change is requested
```

One Application may be activated many times; every Activation remains distinct
and keeps one identity across its Steps, suspension, and resumption. Pure
running and Activation-local mutation can advance without creating any
revision. Canonical examples and their exact elaboration contracts live in the
[syntax specification](docs/syntax.md); executable falsification programs are
defined by the [adoption spike](docs/adoption-spike.md).

## Read the contracts

| Document | Owns |
| --- | --- |
| [Language tour](docs/language-tour.md) | Compact examples and the shortest conceptual path |
| [Semantic foundation](docs/foundation.md) | Clause meaning, process identities, effects, and Admission |
| [Syntax](docs/syntax.md) | The sole canonical human-readable source design |
| [Architecture](docs/architecture.md) | Trust, host, compiler, runtime, and physical boundaries |
| [Canonical packages](docs/canonical-package.md) | Exact CLCP transport and compiler-machine wire contracts |
| [Compiler genesis](docs/compiler-genesis.md) | External genesis anchor and predecessor-owned succession |
| [Adoption spike](docs/adoption-spike.md) | Executable falsifiers for the constitutional design |
| [Roadmap](docs/roadmap.md) | Current implementation facts, dependency order, and exit evidence |

The frozen [execution corpus](docs/execution-corpus.md) preserves narrower v0
observations while the process-v1 companion is built. Historical experiments,
including the game-leverage materialization probe, are evidence rather than
supported language features.

## Bootstrap evidence

The repository currently contains a Lean constitutional/checker bootstrap, a
historical combined Rust bootstrap crate, and exact shared test vectors. The
shortest broad checks are:

```sh
(cd lean && lake build && lake env leanchecker --fresh ClauseCore)
cargo test --workspace --locked --all-targets
```

These commands check their bounded bootstrap contracts; they do not establish
the still-missing language implementation. `ClauseCore` and
`clause-substrate` are historical implementation names, not names for Clause
semantics or the target package architecture. The target split is a user-facing
`clause` facade over `clause-package`, `clause-runtime`,
`clause-materialization`, and `clause-wasm`.

Clause is available under the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
