# clause-materialization

`clause-materialization` owns replaceable physical projections of already
admitted Clause state. It provides cold scan and indexed/incremental uniform-
grid schedules, occurrence-exact support indexes, bounded typed fallbacks, and
operation receipts.

The crate deliberately does not own Clause meaning. Its graph, contract, plan,
snapshot, occurrence, evidence, and admission references are opaque bytes. It
does not create semantic identities, evaluate a Clause predicate, authorize a
delta, admit a `StateRevision`, or retain semantic history. Equal content from
distinct caller-provided occurrences remains distinct.

The caller supplies:

- a checked physical contract over exact premise slots and opaque bindings;
- an already-admitted snapshot or delta envelope;
- exact graph, contract, physical-plan, base, and result references; and
- explicit row, support, and allocation ceilings.

The crate returns a borrowed physical view plus a receipt covering validation,
candidate/index work, support multiplicity, fallbacks, allocation requests, and
publication. A failed update leaves the preceding view and snapshot reference
unchanged.

This manifest is temporarily standalone because the shared Clause workspace is
owned by the package-split integrator. That integrator must add
`clause:crates/clause-materialization` to the workspace and adapt the opaque
admission envelope to the accepted `clause-package` types; this crate must not
reimplement those identity semantics.

Run the focused gates with the repository-pinned Rust toolchain:

```text
cargo test --manifest-path crates/clause-materialization/Cargo.toml
cargo clippy --manifest-path crates/clause-materialization/Cargo.toml --all-targets -- -D warnings
```
