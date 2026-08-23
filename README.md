# Clause

Clause makes a small but consequential question executable: *a compiler change
touches Beagle; which consumers are affected, and why?*  Its impact example
derives `Store` and `North` from declared dependencies, then returns a
deterministic, role-labelled proof graph for each answer.  The same sealed
revision supports change review and bounded interventions without mutating the
past.

## Run the impact demo

From the repository root:

```sh
revision=$(mktemp)
source=$(mktemp)
cp examples/impact.clause "$source"
./bin/clause e2e "$source" "$revision"
rm -f "$source"
./bin/clause query "$revision"
rm -f "$revision"
```

`e2e` seals [`examples/impact.clause`](examples/impact.clause) into an
immutable revision, derives the impact closure, and emits one deterministic
`clause-demo-output-v1` envelope. It includes the base and successor
`clause-query-output-v2` results and WhyGraphs, the asserted and entailed
semantic diff, bounded `prevent` and `achieve` reports, and
`generated-parity=true`. That parity result means standalone generated Rust,
which embeds and reloads only the sealed revision, produced bytes identical to
the nested base query output. The following source-deleted `query` proves the
persisted successor does not depend on the authoring file either.

The example starts with a chain from `North` through `Store` to `Beagle` and a
change to `Beagle`. Its finite laws derive that the change affects `Store` and
`North`; the proof graph names the exact law, role bindings, and premise facts
for each conclusion. The included intent adds `South` to that chain, making the
new impact and its proof an explicit successor-revision result rather than a
silent update to history.

That is the leverage: a relation can be queried as a model, explained as a
deterministic proof graph, compared as semantics, and changed as a new
immutable revision.

## What the semantic core gives you

- **Finite laws with inspectable causes.** Queries close over declared laws and
  select an acyclic, deterministic WhyGraph. A result is not just an answer:
  it carries the asserted facts or labelled law applications that establish it.

- **Semantic change review.** Revisions stay immutable. Their diff separates
  asserted additions and removals from newly entailed or lost consequences, and
  identifies selected-proof changes for facts that remain true.

- **Bounded ways to intervene.** `achieve` searches for a candidate assertion
  that makes a goal true; `prevent` enumerates inclusion-minimal withdrawals
  that make an entailed target absent. Both return new candidate revisions and
  both are explicitly bounded, so neither hides an unbounded planner behind a
  convenient verb.

- **Source-independent execution.** The persisted revision is the execution
  boundary. Generated standalone Rust reloads that revision and is checked
  against the interpreter's canonical result; the authoring source is not a
  runtime dependency.

## Semantic boundary

Clause currently admits a deliberately narrow, useful fragment: finite,
positive, role-labelled Horn laws. Facts and laws use named roles rather than
positional triples; closure, query proof construction,
and intervention search have explicit resource bounds. Those bounds are part
of the contract: exceeding one fails visibly rather than silently broadening a
search.

This is not a general theorem prover or workflow engine. There are no effects,
no negation or unrestricted search, and no hidden solver. Canonical semantic
arrays exclude source text, spans, and runtime details. A Revision identity is
`rev-sha256-` plus SHA-256 of those canonical UTF-8 bytes; reload rejects
noncanonical bytes, mismatched identities, incomplete role maps, malformed
modes, and invalid intent namespaces.

## Roadmap

Clause will grow only where the evidence says this kernel holds:

1. **Scale the finite core.** Establish a representative law corpus and
   benchmark closure size, rounds, and join attempts. Any indexing work must
   preserve canonical results, selected WhyGraphs, and limit failures exactly.
2. **Make revisions incremental.** Compare incremental closure and semantic
   diffs against a full recomputation across generated revision sequences;
   adopt caching only when the equality proof and measured win are both clear.
3. **Strengthen interventions.** Measure bounded achievement and prevention on
   multi-cause examples, then improve search only with deterministic minimality
   and explicit exhaustion evidence intact.
4. **Earn larger demonstrations.** Add domains when a runnable scenario shows
   a real decision improved by a role-labelled explanation and immutable
   semantic diff, not merely more syntax.

## Develop

Clause pins Rust 1.96.1.

```sh
cargo test
```

The tests cover parsing and elaboration, named roles and modes, finite closure
and deterministic WhyGraphs, immutable revision transitions and semantic diffs,
bounded achievement and prevention, source-deleted generated-Rust parity, and
the runnable impact journey.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
