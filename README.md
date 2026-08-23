# Clause

Clause is a small language for immutable, typed relation models. A named Model
is sealed into a Revision with a stable semantic identity. Revisions can add or
withdraw asserted clauses; requests select bounded derivation, explanation,
intervention, and comparison over those sealed values.

## Native source surface

One `:` grammar introduces every declaration. Types, relations, models, laws,
Revisions, and reusable Deltas have stable names. A relation declares its
roles inline in its sentence shape, so clauses are always role-labelled and can
be binary or n-ary.

```clause
Module: Type
Change: Type

impact/imports: Relation
    {consumer: Module} imports {dependency: Module}
    mode consumer -> dependency: many

impact/changes: Relation
    {change: Change} changes {component: Module}
    mode change -> component: many

impact: Model
    North: Module
    Store: Module
    compiler-change: Change
    North imports Store
    compiler-change changes Store

impact/direct-dependency: Law
    ?consumer imports ?dependency
    when:
        ?consumer imports ?dependency

impact/add-south: Revision
    from: impact
    admit:
        South imports North
```

Entities have an admitted Type and belong to their selected Model. Scalar text
is written with an explicit admitted `Text: Type`; ordinary entity references
are unquoted. A clause must fill every role exactly once with a type-correct
entity, scalar value, or law variable. A relation shape begins and ends with a
role and has a nonempty literal between adjacent roles.

`when:` preserves positive, range-restricted Horn-law premises. `mode` states
the relation's known and sought roles plus cardinality. Both forms participate
in the sealed semantic payload.

## Revisions and Deltas

A Model name may be used as the base Revision. A Revision owns its change set;
a Delta is reusable only against its exact typed base Revision.

```clause
impact/remove-relay: Delta
    from: impact
    withdraw:
        North imports Relay

impact/no-relay: Revision
    from: impact
    apply: impact/remove-relay
```

The only persisted Revision envelope contains its canonical semantic payload.
Names used for source navigation, request order, and Delta names do not enter
the Revision identity.

## Requests

Requests are outside Revision identity and run in source order:

```clause
find all ?consumer in impact:
    compiler-change affects ?consumer

why all in impact:
    compiler-change affects North

prevent all minimal in impact:
    compiler-change affects North
using:
    impact/imports

achieve one minimal in impact/add-south:
    compiler-change affects South
using:
    impact/imports

diff impact -> impact/add-south
```

`find` returns canonical typed bindings. `why` selects one deterministic proof;
`why all` returns a bounded support frontier and says whether it is complete.
`prevent` searches typed asserted-clause withdrawals. `achieve` forms its finite
candidate basis from active entities of the allowed relation's role Types in
the selected Revision, then excludes clauses already asserted. `one minimal`
proves the first canonical inclusion-minimal result; `all minimal` exhausts the
admitted finite search or reports a bounded incomplete result. `diff` preserves
authored, entailed, proof, and support changes between two Revisions.

## Run

From the repository root:

```sh
revision=$(mktemp)
./bin/clause seal examples/impact.clause impact "$revision"
./bin/clause run examples/impact.clause
rm -f "$revision"
```

`seal SOURCE REVISION_NAME REVISION_FILE` writes one canonical Revision. `run
SOURCE` compiles the named Revision registry, resolves every authored request,
and prints one deterministic ordered transcript. Generated Rust embeds the
resolved request program and referenced Revisions, so it can produce the same
transcript after the authoring source is removed.

## Semantic boundary

Clause admits finite positive Horn derivation with explicit resource bounds.
There are no effects, hidden solver choices, or unbounded search. Canonical
semantic bytes contain Types, entities, relation shapes, asserted clauses, and
laws; source spans and request navigation stay outside the sealed identity.

## Develop

Clause pins Rust 1.96.1.

```sh
cargo test
```

The focused tests cover native lowering, Revision wire strictness, the typed
impact journey, support frontiers, intervention selection, semantic diff, and
source-deleted generated-request parity.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
