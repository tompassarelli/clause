# Clause

The sealed Revision is the program. Source and generated code are projections of it.

Clause is a Rust-only language for immutable, typed semantic programs. A Model
admits stable identities, role-labelled n-ary relations, asserted clauses, and
positive laws; sealing it produces a canonical Revision with a content-derived
identity. A successor Revision is another value, produced by exact admissions
and withdrawals from an exact base.

A `.clause` file is the authoring projection. It uses one native `:` grammar
for Types, Relations, Models, Laws, Revisions, and their members. An emitted
`.rs` file is an executable projection: it carries the referenced sealed
Revisions and the resolved request sequence, not the authoring source or
frontend. Requests navigate Revisions without entering their identity.

Clause currently supplies this semantic-program core. It does not yet supply
the arbitrary I/O, networking, concurrency, or general-purpose effects needed
to author an entire application such as a server or terminal UI without a Rust
boundary.

## Repository and artifact map

| Path or artifact | Authority and lifecycle |
| --- | --- |
| `src/` | Authoritative Rust implementation. Stable facade modules retain the library boundary; their private child modules own the implementation details. |
| `examples/` | Authoritative Clause authoring inputs. They are the inputs exercised by the native CLI routes below. |
| `tests/` | Verification consumers of the public Rust and CLI behavior. |
| `src/generated.rs` | Authoritative source composer for standalone generated Rust. Do not hand-edit an emitted projection. |
| `target/`, emitted `.rs` files and binaries, and temporary sealed Revision files | Disposable build or projection artifacts unless deliberately retained for a named consumer. |
| `bin/clause` | Repository-local shell wrapper for the native Cargo CLI. |

## The complete one-page hospital program

`examples/hospital.clause` is authoritative. The complete, copy-pasteable
block below is its verified documentation projection, not a sketch:

```clause
Space: Type
Door: Type
Inspection: Type

egress/connects: Relation
    {door: Door} connects {origin: Space} to {destination: Space}
    mode door, origin -> destination: many

egress/passed: Relation
    {door: Door} passed {inspection: Inspection}
    mode door -> inspection: many

egress/route: Relation
    {origin: Space} has a usable egress path to {destination: Space}
    mode origin -> destination: many

egress: Model
    ICU-A: Space
    East-Corridor: Space
    West-Corridor: Space
    North-Exit: Space
    Isolation-Room: Space
    Fire-Marshal-Inspection: Inspection

    [Door 101..106]: Door
    [Door {n}]:
        passed: Fire-Marshal-Inspection
    for n: 101..104

    [Door 101] connects ICU-A to East-Corridor
    [Door 102] connects East-Corridor to North-Exit
    [Door 103] connects ICU-A to West-Corridor
    [Door 104] connects West-Corridor to North-Exit
    [Door 105] connects Isolation-Room to East-Corridor
    [Door 106] connects Isolation-Room to West-Corridor

egress/direct-route: Law
    ?origin has a usable egress path to ?destination
    when:
        ?door connects ?origin to ?destination
        ?door passed Fire-Marshal-Inspection

egress/recursive-route: Law
    ?origin has a usable egress path to ?destination
    when:
        ?door connects ?origin to ?intermediate
        ?door passed Fire-Marshal-Inspection
        ?intermediate has a usable egress path to ?destination

egress/door-101-withdrawn: Revision
    from: egress
    withdraw:
        [Door 101] passed Fire-Marshal-Inspection

find all ?destination in egress:
    ICU-A has a usable egress path to ?destination

why all in egress:
    ICU-A has a usable egress path to North-Exit

prevent all minimal in egress:
    ICU-A has a usable egress path to North-Exit
using:
    egress/passed

prevent all minimal in egress/door-101-withdrawn:
    ICU-A has a usable egress path to North-Exit
using:
    egress/passed

achieve all minimal in egress:
    Isolation-Room has a usable egress path to North-Exit
using:
    egress/passed

diff egress -> egress/door-101-withdrawn
```

The three-place `connects` relation is genuinely n-ary: every clause fills its
typed `door`, `origin`, and `destination` roles. Entity identity includes the
Model, local name, and admitted Type, so `Door 101` is distinct from an
equal-looking local name of another Type or Model.

`[Door 101..106]` and the focused `{n}` block are checked semantic ellipsis.
The first admits six ordinary `Door` identities. The second correlates the same
finite binder with exactly four ordinary `passed` clauses; it is not a
Cartesian expansion. Both forms lower away before sealing. No range, focus, or
placeholder survives in Revision identity, derivation, explanation, diff,
intervention synthesis, or generated Rust.

## Six directions through the program

Requests execute once in authored order. The hospital program demonstrates six
distinct navigations over the same semantics:

1. **Forward — `find`.** Bounded recursive closure applies both route laws and
   finds `East-Corridor`, `North-Exit`, and `West-Corridor` from `ICU-A`.
2. **Backward — `why all`.** The result is a complete frontier of two
   inclusion-minimal supports for reaching `North-Exit`: the east path through
   Doors 101 and 102, and the west path through Doors 103 and 104. Each support
   retains its proof tree and presents its assertions in canonical proof-path
   order.
3. **Counterfactual withdrawal — `prevent`.** Restricting changes to
   `egress/passed`, the base Revision has four complete minimal pairs: one
   inspection withdrawal from each route. After Door 101's inspection is
   already withdrawn, the route remains entailed through the west path, but
   the complete prevention frontier degrades to the two singleton withdrawals
   for Doors 103 and 104.
4. **Counterfactual addition — `achieve`.** The complete frontier contains two
   singleton additions: admit the inspection clause for Door 105 or for Door
   106. Either addition connects `Isolation-Room` to an already usable path.
5. **Across Revisions — `diff`.** The authored layer reports only the Door 101
   inspection withdrawal. The entailment layer reports loss of the
   `ICU-A`-to-`East-Corridor` route. The `North-Exit` consequence remains true,
   while its east support is removed and its west support is retained; proof
   and support changes therefore expose support-preserving degradation instead
   of flattening the comparison to added and removed facts.
6. **Outward — materialization.** `emit-rust` resolves the source once and emits
   a standalone Rust program containing the exact referenced Revisions and
   requests. The `.clause` source can then be deleted before compilation; the
   executable produces the same canonical result bytes.

The source therefore yields six request results—`find`, `why-all`, two
`prevent-all` results, `achieve-all`, and `diff`—while the sixth direction above
projects that whole ordered journey out of the authoring environment.

## What “minimal” and “complete” mean

Minimal always means **inclusion-minimal**, not minimum-cardinality. No proper
subset of a returned support still entails its consequence, and no proper
subset of a returned intervention still prevents or achieves its target. A
minimum result would instead have the least cardinality among all successful
sets; Clause makes no such claim. `one minimal` returns one canonical certified
result. `all minimal` returns an antichain and says `Complete` only after the
entire admitted finite search has been exhausted.

An intervention's `using:` block defines a finite, typed basis. `prevent`
considers currently asserted clauses of those extensional relations.
`achieve` constructs ground clauses from the selected relation's exact role
Types and entities already admitted by the selected Revision, then excludes
clauses already asserted. Every returned Delta is checked by applying it to
its exact base Revision and evaluating the target again.

The default runner bounds closure at 100 assertions, 10 rounds, and 10,000 join
attempts; support enumeration at 100 expansions and 100 supports per clause;
and intervention enumeration at 100 candidate checks and 100 solutions.
`why all` reports whether its support frontier is complete; exhausting a
support bound cannot produce a complete status. For `all minimal`, candidate,
solution, closure, support-expansion, or support-frontier exhaustion produces
an explicit incomplete result. Its retained interventions are individually
verified, but the retained set is not claimed to be the complete antichain.
`find` fails on closure exhaustion rather than returning a partial binding set.
All frontiers in the hospital program exhaust their finite bases and report
complete.

## Run and materialize

From the repository root, use the native Cargo CLI to run the authoritative
examples:

| Input | Native Cargo CLI route |
| --- | --- |
| `examples/catalog.clause` | `cargo run --bin clause -- run examples/catalog.clause` |
| `examples/impact.clause` | `cargo run --bin clause -- run examples/impact.clause` |
| `examples/hospital.clause` | `cargo run --bin clause -- run examples/hospital.clause` |

`bin/clause run examples/hospital.clause` is the equivalent repository-local
wrapper route. It uses Cargo's effective target directory, including a private
`CARGO_TARGET_DIR` when one is configured. The following recipe runs an exact
copy, emits standalone Rust, deletes the authoring copy, compiles the
projection, and compares canonical results. Shell command substitution removes
the CLI's single presentation newline before the byte comparison.

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

`clause seal SOURCE REVISION_NAME OUTPUT` separately writes one Revision's
canonical wire form. Revision navigation names, request order, source spans,
and ellipsis syntax do not enter that identity; typed declarations, entities,
relation roles and modes, asserted clauses, and laws do.

## Develop

Clause pins Rust 1.96.1.

```sh
cargo fmt --all --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

The acceptance suite covers native parsing and lowering, strict Revision wire
reload, recursive derivation, complete support and intervention frontiers,
semantic diff, ordered request execution, and source-deleted generated-Rust
parity.

Every concurrently written Clause lane uses a private `CARGO_TARGET_DIR`.

## License

Clause is available under the MIT License or the Apache License, Version 2.0,
at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and
[LICENSE-APACHE](LICENSE-APACHE).
