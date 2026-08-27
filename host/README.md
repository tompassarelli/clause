# Provisional JavaScript host

`host/provisional.mjs` is a mechanical browser/Three.js adapter boundary. It
does not parse Clause source, decode Clause wire data, evaluate relations,
compute transitions, allocate semantic identities, or manufacture effect
evidence.

## Implemented now

- Rust owns canonical `clause-render-plan-v2` bytes and can emit an import-free
  frozen ESM table keyed by exact StateRevision identity and bound to one
  ProgramRevision.
- `renderPlanFor` asks that generated module for one exact-state plan, validates
  its ProgramRevision and StateRevision identities, canonical item ordering,
  Referent IDs, and finite F32 coordinates, then returns a frozen copy.
- `createTwoMeshBinding` applies a validated total plan to two registered
  meshes: listed meshes receive positions and become visible; omitted meshes
  become hidden; unknown identities fail before mutation.
- `createEventBridge` requires caller-owned event and transition occurrence
  allocators, rejects reused or malformed pins, mechanically orders declared
  input events, forwards them to an artifact-owned runtime, and retains only
  results that return the exact ProgramRevision and TransitionOccurrence and
  pass the artifact's transition validator.
- `createEffectBridge` forwards only declared capabilities and retains only
  artifact-validated traces.
- `startLifecycle` owns input listeners, animation-frame scheduling, and
  idempotent teardown. Those host concerns do not enter Clause semantics.

The event, effect, mesh, and lifecycle APIs are unit-tested with synthetic
artifact and Three.js substitutes. The generated artifact implemented by Rust
today contains frozen RenderPlan lookup data only; it does not yet implement
the full live-runtime contract expected by `loadArtifact`.

The source-deletion RenderPlan test removes the authored Clause file before Bun
imports the generated ESM and compares its plan JSON exactly with Rust.

## Current boundary

There is no generated live-JavaScript transition runtime, real browser or
Three.js integration proof, source map, dedicated scene/effect source syntax,
or complete M7 one-coin vertical. The current `programRevisionId` field names
the constitutional ProgramRevision; exact StateRevision IDs transitively bind
their runtime session, policy, semantics epoch, predecessor, and causal
occurrence. The host validates and forwards those opaque Clause-produced pins;
it does not recompute identity preimages or invent provenance.

Run the host unit boundary with Bun:

```sh
bun test host/provisional.test.mjs
```
