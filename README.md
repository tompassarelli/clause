# Clause

Clause is a distinction-first relational programming language and Rust
implementation. Its explanatory starting point is the act of distinguishing.
Its usable semantic kernel begins with stable referent identities and n-ary,
role-labelled relational forms.

When a Model is selected as the canonical authoritative structure to realize,
that Model is the program. Source files and generated code are projections of
it. A Revision is an immutable snapshot and exact lineage record for a Model;
it supports history, provenance, diff, rollback, synchronization, and
reproducibility, but it is not the conceptual center of programming.

Clause has one semantic domain of addressable referents. It does not begin with
separate universes of values, types, objects, data, fields, or functions. Those
are derived interpretations of relational structure when useful. A relation is
a referent used in relational position, and its participants occupy stable
named roles. Functions are relations with uniqueness constraints and an
operational orientation; type checking is proof of relational admissibility.

The kernel keeps three structures deliberately separate:

- **relational content** identifies an n-ary relation and its role-labelled
  participants;
- an **assertion occurrence** records one scoped source act concerning that
  content;
- a **judgment** records an authority's declared, derived, observed, admitted,
  rejected, or superseded stance.

Canonicalizing shared content therefore does not erase independently authored
occurrences or their provenance. Assertion does not imply admission. Operative
content requires the Model's exact authority and scope. Identity also remains
separate from equality and theory-specific equivalence, and the default is
open-world: failure to derive a relation is not its denial.

## Implementation status

| Layer | Status |
| --- | --- |
| Semantic kernel | The distinction-first referent kernel is implemented. `ReferentId`, role-based `RelationalContent`, `AssertionOccurrence`, `Judgment`, definitions, derivation rules, universal laws, invariants, goals, and transitions are explicit semantic structures. |
| Revision wire | The canonical wire is `clause-semantic-v6` inside a `clause-revision-v4` envelope. Root and successor reload are strict. |
| Executable parser | The native parser still accepts the older ceremonial `Type`, `Relation`, `Model`, `Law`, and `Revision` profile with four-space layout. It lowers that source into the referent kernel. |
| Target surface | The compact two-space surface below is the canonical direction. The native parser does not accept it yet. |
| General-purpose runtime | Recursive relational programming, transitions, effects and receipts, generated JavaScript, and the Three.js game proof remain active product work. |

Clause currently has no Store implementation. Store is a neutral persistence
and query substrate, not semantic authority. A future adapter must preserve
referents, relational content, assertion occurrences, judgments, modal
authority, admission and rejection, supersession, and exact Revision lineage.
It must not infer those semantics from equality, missing rows, retraction,
query negation, liveness, or storage revision identity.

[FOUNDATION.md](FOUNDATION.md), [SURFACE.md](SURFACE.md), [M0.md](M0.md), and
[ROADMAP.md](ROADMAP.md) contain the wider doctrine, target evidence, and
implementation sequence. Their older surface specimens are being migrated
after substantial syntax churn. Until that migration is atomic, the surface
ruling below supersedes conflicting `:`/membership/definition examples in
those documents.

## Canonical target surface

These four forms keep distinct operations visibly distinct:

```clause
gravity: 9.81
Chess ∈ Game
x = y
input -> result
```

Their meanings are:

| Form | Meaning |
| --- | --- |
| `name: term` | Establish `name` as the stable handle for that binding or definition. |
| `thing ∈ Group` | Assert membership. |
| `x = y` | Assert equality under the active theory. Never assignment. |
| `input -> result` | Orient production or a solved/result participant. |

`:` is binding only. It never means membership, assignment, list
introduction, or “a block follows.” A bare indented enumeration needs no
punctuation:

```clause
Game
  Chess
  Soccer
```

It elaborates to:

```clause
Chess ∈ Game
Soccer ∈ Game
```

`∈` is the only Clause membership spelling. A human-facing editor or input
tool may immediately transform the keystrokes `::` into `∈` before parsing
or storage. Raw `::` is not Clause grammar, and agents and formatters emit `∈`
directly. There is no word alias.

Strong-prior ASCII symbols remain symbols where their meaning is already
obvious:

```clause
x > y
x < y
x >= y
x <= y
x != y
a + b
a - b
a * b
a / b
```

The slash is spaced when it is an operator, distinguishing it from qualified
semantic names such as `egress/route`. These operators still elaborate to
role-labelled relational forms. Domain relations remain words:

```clause
iron-door connects Cellar to Armory
Alice parent-of Bob
North depends-on Store
```

Structurally leading `+` and `-` may orient exact Delta admissions and
withdrawals; infix `+` and `-` retain their conventional arithmetic priors.

Canonical target layout uses two ASCII spaces per level, spaces only, with no
tabs. Indentation projects structure; punctuation is used only when it carries
semantics.

### Focus is projection, not an object

The controlling focused specimen is:

```clause
iron-door
  Door
  connects Cellar to Armory
  state: locked
```

It elaborates to three co-located meanings:

```clause
iron-door ∈ Door
iron-door connects Cellar to Armory
state of iron-door: locked
```

The bare `Door` child is membership. `connects Cellar to Armory` is an
ordinary relational claim with `iron-door` in its focus role. `state: locked`
is a stable binding under the current focus. The layout does not instantiate a
`Door` object, create fields, confer ownership, or introduce nested storage.

## The implemented semantic-v6 kernel

The current lowerer resolves source terms and declarations into stable
referents before constructing a Model. A source spelling is a designation, not
semantic identity; source spans remain projection metadata. Relation shapes
name participant roles and encode admissibility and lookup contracts without
creating a privileged Type universe. Relational content has a content identity
independent of any assertion occurrence or judgment concerning it.

A Model can carry referents, relational content, relation shapes, assertion
occurrences, definitions, derivation rules, universal laws, invariants, goals,
transitions, and judgments. These share pattern and relational machinery while
retaining distinct modal behavior. In particular, a derivation rule can add a
consequence, while an invariant governs admission and a goal may describe a
condition that does not yet hold.

Sealing produces canonical JSON with exact UTF-8 spelling and ordering:

- `clause-semantic-v6` is the complete semantic payload;
- `clause-revision-v4` carries its content-derived Revision identity;
- a root Revision reloads independently;
- a successor names one exact predecessor and one exact Delta;
- successor reload requires that predecessor Revision and verifies that the
  Delta accounts for the complete successor snapshot.

Source spans, request order, revision aliases, and layout sugar do not enter
Revision identity. The immutable Revision records the Model snapshot and its
lineage; the authoritative Model remains the program.

## Repository and artifact map

| Path or artifact | Authority and lifecycle |
| --- | --- |
| `src/` | Authoritative Rust implementation. Stable facade modules retain the library boundary; their private child modules own implementation details. |
| `examples/` | Authoritative inputs for the current executable parser profile. They are migration and behavior oracles, not target-surface specimens. |
| `tests/` | Verification consumers of the public Rust and CLI behavior. |
| `src/generated.rs` | Authoritative composer for standalone generated Rust. Do not hand-edit an emitted projection. |
| `target/`, emitted `.rs` files and binaries, and temporary Revision files | Disposable build or projection artifacts unless deliberately retained for a named consumer. |
| `bin/clause` | Repository-local shell wrapper for the native Cargo CLI. |

## The current-profile hospital oracle

`examples/hospital.clause` is authoritative for the currently executable
profile and remains the semantic and migration oracle. Its four-space,
ceremonial grammar is implemented parser behavior, not canonical target layout
and not an example for new surface work.

The source still declares `Type`, `Relation`, `Model`, `Law`, and `Revision`
forms. Lowering resolves those declarations into semantic-v6 referents,
relation shapes, relational content, assertion occurrences, judgments, rules,
and exact Revision lineage. The kernel and wire do not preserve a primitive
Type, Entity, or Value universe. Legacy role domains become relational
admissibility constraints, while separately declared doors remain separately
identified referents even when their known relational structure is identical.

The three-place `connects` relation is genuinely n-ary: every clause fills its
named `door`, `origin`, and `destination` roles. `[Door 101..106]` and the
focused `{n}` block are checked semantic ellipsis. The first establishes six
ordinary door referents. The second correlates the same finite binder with
exactly four ordinary `passed` clauses; it is not a Cartesian expansion. Range,
focus, and placeholder syntax lower away before sealing and do not survive in
Revision identity, derivation, explanation, diff, intervention synthesis, or
generated Rust.

## Six directions through the program

Requests execute once in authored order. The hospital program demonstrates six
distinct navigations over the same Model:

1. **Forward — `find`.** Bounded recursive closure applies both route
   derivation rules and finds `East-Corridor`, `North-Exit`, and
   `West-Corridor` from `ICU-A`.
2. **Backward — `why all`.** The result is a complete frontier of two
   inclusion-minimal supports for reaching `North-Exit`: the east path through
   Doors 101 and 102, and the west path through Doors 103 and 104. Each support
   retains its proof tree and presents supporting relational content in
   canonical proof-path order.
3. **Counterfactual withdrawal — `prevent`.** Restricting changes to
   `egress/passed`, the base Revision has four complete minimal pairs: one
   inspection withdrawal from each route. After Door 101's inspection is
   withdrawn, the route remains entailed through the west path, and the
   prevention frontier becomes the two singleton withdrawals for Doors 103 and
   104.
4. **Counterfactual addition — `achieve`.** The complete frontier contains two
   singleton admissions: the inspection clause for Door 105 or Door 106.
   Either connects `Isolation-Room` to an already usable path.
5. **Across Revisions — `diff`.** The authored layer reports the Door 101
   inspection withdrawal. The entailment layer reports loss of the
   `ICU-A`-to-`East-Corridor` route. The `North-Exit` consequence remains true,
   while its east support is removed and its west support is retained.
6. **Outward — materialization.** `emit-rust` resolves the source once and
   emits a standalone Rust program containing the exact referenced Revisions
   and requests. The `.clause` source can then be deleted before compilation;
   the executable produces the same canonical result bytes.

The source yields six request results—`find`, `why-all`, two `prevent-all`
results, `achieve-all`, and `diff`—while materialization projects that ordered
journey out of the authoring environment.

## What “minimal” and “complete” mean

Minimal means **inclusion-minimal**, not minimum-cardinality. No proper subset
of a returned support still entails its consequence, and no proper subset of a
returned intervention still prevents or achieves its target. `one minimal`
returns one canonical certified result. `all minimal` returns an antichain and
says `Complete` only after the entire admitted finite search is exhausted.

In the current executable profile, an intervention's legacy `using:` block
defines a finite basis. `prevent` considers currently admitted relational
content of those extensional relations. `achieve` constructs ground relational
content from the selected relation shape's admissibility constraints and the
referents admitted by the selected Revision, then excludes content already
admitted. Every returned Delta is checked by applying it to its exact base
Revision and evaluating the target again.

The default runner bounds closure at 100 admitted or derived content items, 10
rounds, and 10,000 join attempts; support enumeration at 100 expansions and
100 supports per clause; and intervention enumeration at 100 candidate checks
and 100 solutions. `why all` reports whether its support frontier is complete;
exhausting a support bound cannot produce complete status. For `all minimal`,
candidate, solution, closure, support-expansion, or support-frontier exhaustion
produces an explicit incomplete result. Retained interventions remain
individually verified, but the retained set is not certified as the complete
antichain. `find` fails on closure exhaustion rather than returning partial
role assignments. All frontiers in the hospital program exhaust their finite
bases and report complete.

## Run and materialize

The commands in this section consume the current legacy parser profile, not the
target surface shown above.

| Input | Native Cargo CLI route |
| --- | --- |
| `examples/catalog.clause` | `cargo run --bin clause -- run examples/catalog.clause` |
| `examples/impact.clause` | `cargo run --bin clause -- run examples/impact.clause` |
| `examples/hospital.clause` | `cargo run --bin clause -- run examples/hospital.clause` |

`bin/clause run examples/hospital.clause` is the equivalent repository-local
wrapper route. It uses Cargo's effective target directory, including a private
`CARGO_TARGET_DIR` when configured. The following recipe runs an exact copy,
emits standalone Rust, deletes the authoring copy, compiles the projection, and
compares canonical results:

```sh
build_dir=$(mktemp -d)
cp examples/hospital.clause "$build_dir/hospital.clause"

expected=$(bin/clause run "$build_dir/hospital.clause")
bin/clause emit-rust "$build_dir/hospital.clause" "$build_dir/hospital.rs"
rm "$build_dir/hospital.clause"

rustc --edition=2024 --cfg clause_generated \
    "$build_dir/hospital.rs" -o "$build_dir/hospital"
actual=$("$build_dir/hospital")
test "$actual" = "$expected"
```

`clause seal SOURCE REVISION_NAME OUTPUT` writes one Revision's canonical wire
form. A root artifact can be strictly reloaded alone. A successor can be
strictly reloaded only with the exact predecessor named by its Delta.

## Develop

Clause pins Rust 1.96.1.

```sh
cargo fmt --all --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

The acceptance suite covers native parsing and lowering, strict semantic-v6 /
Revision-v4 wire reload, exact predecessor lineage, recursive derivation,
complete support and intervention frontiers, semantic diff, ordered request
execution, and source-deleted generated-Rust parity.

Every concurrently written Clause lane uses a private `CARGO_TARGET_DIR`.

## Product direction

The implemented kernel is not yet the complete general-purpose language.
Clause is growing toward recursive terms and relational selection, exact state
transitions, explicit effects with receipts, generated JavaScript ES modules,
Three.js integration, and a one-coin game as the first proof of generality.
Procedures remain available where execution order itself is the meaning. The
compiler should specialize relational semantics into direct storage and target
code rather than carrying a generic triple interpreter through hot paths.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
