# Clause

Clause is a research-language proof of concept. Its semantic program is a
`Model`: an immutable collection of relations, facts, and a query. An
immutable `Revision` gives that Model a stable identity. A `Branch` is a
coherent tracked development of Revisions. A `mode` records an operational
orientation declared by a relation and selected by query planning.

The intended boundary is:

```
external source -> persisted Revision -> source-free execution
```

Authoring source is elaborated into a canonical semantic form. That form is
persisted as a Revision and can be loaded and executed without the source
file. The Racket and Rust implementations are intended to be equivalent
implementations of this contract; they will live at `racket/` and `rust/`.

## Sealed catalog example

This small program declares a finite mode, a catalog Model, a query, and two
operations over immutable Branch/Revision state:

```clause
relation catalog/contains(set: Text, member: Text):
    sentence: {set} contains {member}
    mode set -> member: many

model catalog:
    "letters" contains "a"
    "letters" contains "b"

query catalog:
    ?member where "letters" contains ?member

claim catalog:
    "letters" contains "c"

require catalog:
    "letters" contains "c"
```

The relation's `mode` declares `set` known and `member` sought, with many
answers. `claim` creates a successor Revision when the clause is new;
`require` checks the resulting Revision and returns its proof or a deterministic
unsatisfied result.

## Repository layout

The root documents the semantic contract. Equivalent host implementations
will be kept under `racket/` and `rust/`, with shared examples and fixtures
added as the proof develops.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
