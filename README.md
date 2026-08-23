# Clause

**The sealed Revision is the program.** Clause evaluates one immutable semantic
model in several directions: forward to what follows, backward to every minimal
reason it follows, across revisions to what changed, and counterfactually to the
smallest admitted additions or withdrawals that change the answer. Generated
Rust is another executable projection of that same Revision.

The current 45-line impact program asks which consumers a compiler change can
affect. Relations declare stable identities, named roles, exact sentence shapes,
and admitted modes; asserted clauses and laws then use those shapes directly:

```clause
relation impact/imports(consumer: Text, dependency: Text):
    sentence: {consumer} imports {dependency}
    mode consumer -> dependency: many

relation impact/depends(consumer: Text, dependency: Text):
    sentence: {consumer} depends on {dependency}
    mode consumer -> dependency: many

relation impact/changes(change: Text, component: Text):
    sentence: {change} changes {component}
    mode change -> component: many

relation impact/affected(change: Text, consumer: Text):
    sentence: {change} affects {consumer}
    mode change -> consumer: many

model impact:
    "North" imports "Store"
    "North" imports "Relay"
    "Store" imports "Beagle"
    "Relay" imports "Beagle"
    "compiler-change" changes "Beagle"

law impact/direct-dependency:
    ?consumer depends on ?dependency
    when:
        ?consumer imports ?dependency

law impact/recursive-dependency:
    ?consumer depends on ?dependency
    when:
        ?consumer imports ?intermediate
        ?intermediate depends on ?dependency

law impact/impact:
    ?change affects ?consumer
    when:
        ?change changes ?component
        ?consumer depends on ?component

intent impact/adopt-south:
    "South" imports "North"

query impact:
    ?consumer where "compiler-change" affects ?consumer
```

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

`e2e` seals [clause:examples/impact.clause](examples/impact.clause), derives its
finite closure, and emits one deterministic `clause-semantic-journey-v1`
envelope containing `find`, the complete support frontier, `why all`, semantic
and support diff, `prevent`, and `achieve`. It compiles standalone Rust and
requires that executable to produce the complete envelope byte for byte. The
following `query` runs from the persisted successor after the authoring source
has been removed.

The named acceptance journey goes further: it removes its test-owned Clause
source before emitting and compiling the standalone Rust, then requires the
source-free executable's full semantic-journey bytes to equal the interpreter's
golden bytes. Parity covers every semantic direction above, not only query
results.

## What the semantic core gives you

- **Forward and backward traversal.** `find` returns `North`, `Relay`, and
  `Store`. `why all` proves `compiler-change affects North` from exactly two
  inclusion-minimal supports: the common `compiler-change changes Beagle`
  assertion plus either `North -> Relay -> Beagle` or
  `North -> Store -> Beagle`. The complete status means the bounded support
  frontier was exhausted, not that one convenient proof was selected.

- **Support-preserving semantic diff.** A successor withdraws
  `North imports Relay`. The assertion and the lost entailment
  `North depends on Relay` appear in their respective diff layers. North remains
  affected through Store, while support diff records the lost Relay route and
  retained Store route. Clause can therefore report degraded justification even
  when a consequence remains true.

- **Complete minimal prevention.** Restricted to import withdrawals, `prevent`
  returns the four inclusion-minimal ways to hit both supports:
  `{North->Relay, North->Store}`, `{North->Relay, Store->Beagle}`,
  `{North->Store, Relay->Beagle}`, and
  `{Relay->Beagle, Store->Beagle}`. No returned set contains another.

- **Complete bounded achievement.** Given an explicit finite basis of checked,
  ground import clauses, `achieve` exhausts that basis and returns exactly four
  singleton ways to make `compiler-change affects South` true:
  `South imports Beagle`, `South imports North`, `South imports Relay`, or
  `South imports Store`. This frontier is complete because its declared basis
  was exhausted; a candidate-budget-exhausted result is reported as incomplete.

- **Source-independent execution.** The persisted Revision is the execution
  boundary. Standalone generated Rust reproduces the interpreter's canonical
  find, supports, explanations, diff, and intervention frontiers without the
  authoring source.

## Semantic boundary

Clause currently admits a deliberately narrow, useful fragment: finite,
positive, role-labelled Horn laws. Asserted clauses and laws use named roles
rather than positional triples; closure, complete support construction, and
intervention search have explicit resource bounds. Those bounds are part of the
contract: exceeding one fails visibly rather than silently claiming
completeness.

This is not a general theorem prover or workflow engine. There are no effects,
no negation or unrestricted search, and no hidden solver. Canonical semantic
arrays exclude source text, spans, and runtime details. A Revision identity is
`rev-sha256-` plus SHA-256 of those canonical UTF-8 bytes; reload rejects
noncanonical bytes, mismatched identities, incomplete role maps, malformed
modes, and invalid intent namespaces.

## Roadmap

The north star remains a semantic time machine: one sealed Revision that can
compute, explain, compare, synthesize interventions, and project ordinary
executables. The next milestones are deliberately ordered:

1. Replace the current prefixed authoring syntax atomically with one native `:`
   grammar, typed stable identities, n-ary relation shapes, explicit Revision
   deltas, and first-class `find`, `why`, `diff`, `prevent`, and `achieve`
   requests. The syntax shown above is the implemented surface today.
2. Add checked semantic ellipsis: typed focus blocks, finite groups, and
   correlated patterns that lower to ordinary canonical clauses without adding
   a second ontology.
3. Generalize relation modes and bounded constraint domains only after their
   determinism, cardinality, and termination contracts are explicit.

Every expansion must preserve deterministic semantic bytes, bounded failure,
and inspectable evidence. Faster machinery is welcome; a second ontology is not.

## Develop

Clause pins Rust 1.96.1.

```sh
cargo test
```

The tests cover parsing and elaboration, named roles and modes, finite closure,
complete minimal support frontiers and explanations, immutable Revision
transitions, three-layer semantic diff, complete bounded intervention
antichains, and source-deleted full-journey generated-Rust parity.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
