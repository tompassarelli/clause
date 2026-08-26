# Provisional ordinary-JavaScript host

`provisional.mjs` is the small browser/Three.js boundary for generated Clause
runtime data. The artifact supplies an exact `rev-sha256-*` identity,
Clause-owned event/effect authority, and an exact-StateRevision
`renderPlan(stateRevisionId, revisionId)` lookup.

Rust now owns the canonical `clause-render-plan-v1` schema and an import-free
ESM emitter for frozen, exact-state plan snapshots. The acceptance test deletes
the authored Clause source before Bun imports those snapshots and compares
their JSON bytes with Rust. The host validates the whole plan before applying
it, sets listed mesh positions, shows listed meshes, and hides omitted
registered meshes.

Authored scene lowering, generated live JavaScript transitions, real Three.js
or browser execution, source maps, and full M7 remain absent. This is a bounded
schema/emitter snapshot proof, not an M7 completion claim.

The host forwards declared event/effect requests, owns DOM/device and RAF
lifecycle, and retains copies of Clause-validated traces. It does not decode
the Clause wire, evaluate transitions, construct receipts, infer identity, or
manufacture semantic truth.
