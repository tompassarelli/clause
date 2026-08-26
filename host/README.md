# Provisional ordinary-JavaScript host

`provisional.mjs` is the small browser/Three.js boundary for a generated
Clause runtime artifact. The artifact supplies an exact `rev-sha256-*`
identity, `createRuntime`, and a pure `renderPlan(state, revisionId)` function.

The host owns event ordering, DOM/device and RAF lifecycle, Three.js resource
bookkeeping, and capability receipt bookkeeping. It does not decode the Clause
wire, evaluate transitions, infer identity, or mutate a Clause state.
