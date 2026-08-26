# Provisional ordinary-JavaScript host

`provisional.mjs` is the small browser/Three.js boundary for a generated
Clause runtime artifact. The artifact supplies an exact `rev-sha256-*`
identity, Clause-owned event/effect authority, and a pure
`renderPlan(state, revisionId)` function.

There is not yet a Rust-owned JavaScript emitter or a settled render-plan
schema. This commit is only a bounded adapter contract/prototype and cannot
claim M7 completion or generated-target/source-deleted parity.

The host forwards declared event/effect requests, owns DOM/device and RAF
lifecycle, and retains copies of Clause-validated traces. It does not decode
the Clause wire, evaluate transitions, construct receipts, infer identity, or
manufacture semantic truth.
