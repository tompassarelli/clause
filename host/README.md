# Provisional ordinary-JavaScript host

`provisional.mjs` is the small browser/Three.js boundary for a generated
Clause runtime artifact. The artifact supplies an exact `rev-sha256-*`
identity, Clause-owned event/effect authority, and a pure
`renderPlan(state, revisionId)` function.

There is not yet a Rust-owned JavaScript emitter or a settled render-plan
schema. This commit is only a bounded adapter contract/prototype and cannot
claim M7 completion or generated-target/source-deleted parity.

The host owns event ordering, DOM/device and RAF lifecycle, Three.js resource
bookkeeping, and capability receipt bookkeeping. It does not decode the Clause
wire, evaluate transitions, infer identity, or mutate a Clause state.
