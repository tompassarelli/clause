# Clause

Clause is a Rust research-language proof of concept organized around typed
relations rather than functions or positional triples. A program elaborates to
an immutable `Model`; a canonical `Revision` gives that Model a stable identity;
a `Branch` tracks coherent Revision development; and a declared `mode` tells
the planner which participants are known and sought.

```clause
relation catalog/contains(set: Text, member: Text):
    sentence: {set} contains {member}
    mode set -> member: many

model catalog:
    "letters" contains "a"
    "letters" contains "b"

intent catalog/restock:
    "letters" contains "c"

query catalog:
    ?member where "letters" contains ?member
```

The example query selects the relation's finite mode and returns `a` and `b`.
The intent proposes admitting `c`; `claim` produces a successor Revision;
`require` proves that the successor contains the desired clause; and the final
query returns `a`, `b`, and `c`. The base Revision remains unchanged.

## Run it

Clause pins Rust 1.96.1. From the repository root:

```sh
revision=$(mktemp)
./bin/clause e2e examples/catalog.clause "$revision"
./bin/clause query "$revision"
rm -f "$revision"
```

`e2e` seals the source, strictly reloads the persisted base Revision, executes
the intent/claim/require/query journey, persists and reloads the successor, and
checks independently generated Rust execution byte-for-byte. It never deletes
the authoring source. `query` reads only the persisted Revision, so it continues
to work after the source is moved or removed.

The narrower commands are:

```sh
./bin/clause seal SOURCE REVISION
./bin/clause query REVISION
```

## Semantic boundary

Canonical semantic arrays exclude source text, spans, and runtime details.
Revision identity is `rev-sha256-` plus SHA-256 of those canonical UTF-8 bytes.
Reload rejects noncanonical bytes, mismatched identities, incomplete role maps,
malformed modes, and invalid intent namespaces. Query results, proofs, claims,
requirements, and intent plans use deterministic array-only encodings.

The current surface deliberately supports a small finite slice: binary mixfix
relations, one Model and query per program, one declared mode per relation, and
pure intent planning. General search, effects, native compilation, packaging,
and compatibility guarantees remain outside this proof.

## Develop

```sh
cargo test
```

The tests cover parsing and spans, named-role elaboration, mode planning,
canonical persistence and tamper rejection, immutable Revision transitions,
source-deleted reload, generic generated execution, and the complete catalog
journey.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
