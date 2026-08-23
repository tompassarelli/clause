# Clause

Clause is a research-language proof of concept (PoC). Its semantic program is
a `Model`: an immutable collection of relations, facts, intents, and a query.
An immutable `Revision` gives that Model a stable identity. A `Branch` is a
coherent tracked development of Revisions. A `mode` records an operational
orientation declared by a relation and selected by query planning.

## M4 intent program

The implemented M4 authoring program is:

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

The relation declares the sentence and its finite mode. The model supplies the
`a` and `b` facts. The intent names the desired `c` fact, and the query opens
`member` for execution. Selecting the intent on the base branch produces a
claim plan; `claim` admits a successor Revision, and `require` checks that
Revision and returns its proof. The base Revision remains unchanged.

## Persisted Revision contract

Each host reads authoring source only while sealing or running the focused M4
journey. Elaboration produces canonical, host-neutral semantic arrays; Revision
admission assigns a `rev-sha256-...` identity and persists a
`clause-revision-v1` JSON envelope. Reload validates the identity and canonical
payload. `query` executes a reloaded Revision without opening or retaining the
authoring file, so deleting the source after sealing does not change execution.
The Racket and Rust hosts implement this same contract, including immutable
branch/Revision transitions and array-only outputs. This is a PoC: wire shapes,
commands, and language surface are experimental rather than a compatibility
promise.

## Run the hosts

From the repository root, run the focused M4 journey on either host:

```sh
racket_revision=$(mktemp)
./racket/bin/clause run racket/m4.clause "$racket_revision"
rm -f "$racket_revision"

rust_revision=$(mktemp)
./rust/bin/clause e2e racket/m4.clause "$rust_revision"
./rust/bin/clause query "$rust_revision"
rm -f "$rust_revision"
```

Both launchers resolve their project roots independently of the caller's
working directory. The example source and revision paths are repository-root
relative; from another directory, pass paths valid for that directory. The
Rust `query` command demonstrates source-free execution from the persisted
successor Revision.

## Repository layout

The root documents the semantic contract. Host implementations and their
launchers live under `racket/` and `rust/`; `racket/m4.clause` is the shared
authoring fixture used by the commands above.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
