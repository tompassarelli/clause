# Checked source-transfer profiling

This artifact instruments the existing source/CPP1 witness replay and internal
live-world migration. It introduces no optimized path, cache, skipped proof,
source reshaping, renderer change or gameplay implementation. Baseline phase
evidence must precede any proposed elimination of repeated lowering.

The runtime profiler is explicitly opt-in and thread-local. Fifteen fixed
phase counters retain inclusive/exclusive monotonic wall milliseconds and call
counts; nesting is capped at 64 and incomplete profiles report truncation.
Disabled hooks read no clock and allocate no profile records. Their TLS branch
and the larger binary still have a cost; alternating enabled/disabled samples
estimate incremental measurement overhead, not the hook-versus-no-hook cost.
Native uses `Instant`; Wasm uses `performance.now()` only when enabled.
Profiling values do not enter compiler identity, checking, state, traces or
Admission. Existing public methods retain their signatures; profile exports
are additive. Operations remain synchronous and profiler scopes are !Send.

`source-transfer-profile` writes exact source, initial/edited CWR1, CET1, and
native phase data under the supplied owning-lane output directory. It requires
an exact compiler source-commit label. Two fixtures are the unchanged M4
encounter and that encounter plus the existing created-burn extension. Both
replace the exact offered attack expression `0.0 - ?damage` with
`0.0 - (?damage * 2.0)`. Setup performs BeginEncounter, Attack, optional created
burns lasting 1s/3s, one 16ms tick, and Admission outside measurement. Every sample opens
a fresh world; no imported native state is supplied to Wasm.

There is one excluded warm-up, then 1..5 samples per mode, alternating profiling
off/on. The native outer timer includes resident source preparation and checked
transfer. Its `transfer` phase isolates the shared boundary path. Wasm's outer
timer includes the existing passive adapter and its byte transport; internal
`transfer` excludes those adapter costs. File I/O, setup, rendering, subsequent
Admission and diagnostic reads are outside both timers. Run variants and modes
in the documented order; these small samples are not CPU-isolated estimates.

Within transfer, phase nesting exposes source read/allocation/offered edit,
old/new elaboration, every actual lowering call, snapshot metadata, row-view
projection, CPP1 decoding/comparison, replacement instantiation and typed
world migration. Inclusive values overlap; sum exclusive values instead.
Native preparation outside the shared check is retained as native-edit
exclusive time, not mislabeled as witness-check time. The native operation
checks the witness twice (preflight and independent boundary replay); Wasm
receives the already-prepared witness and checks it once. The global phase
counts reflect this difference; outer native/Wasm times are not equivalent
units of work. Debug-native versus release-Wasm times are also not a target
performance comparison.

The passive TypeScript driver reads only these exact compiled artifacts, uses
ordinary physical inputs and the existing `editSourceSession`, records artifact
SHA256 identities, and writes Wasm data beside native data. It never models
damage, joins, continuity or migration itself. Run nearest `source_profile`,
M4 `live_semantics`, and `created_collections` checks before measurement;
repeat the fresh-Wasm tamper/stale/created-continuity gates after any reuse
optimization. Shared STATUS/RESULT remain the consumer worker's property.

Example (after heavy-slot admission and pinned/scoped tool setup):

```text
cargo test -p clause-workbench --test source_profile --test live_semantics --test created_collections --locked -j 2
cargo run -p clause-workbench --example source-transfer-profile --locked -j 2 -- target/source-transfer-profile EXACT_SOURCE_COMMIT 3
# Fresh runtime Wasm goes in target/source-transfer-profile/wasm.
bun browser/jump-arena-shell/src/source-transfer-profile.ts /absolute/owned/lane/target/source-transfer-profile 3 EXACT_DRIVER_COMMIT
```

The Wasm driver drains the existing physical-retirement batches after timing
and before the next sample (at most 4,097 calls). Otherwise a synchronous test
can open again before the adapter's scheduled timer finishes retirement and
correctly receive SessionOccupied. This drain does not affect the measured
transfer and is not a change to runtime ownership or reclamation semantics.

No baseline timing, overhead or speedup is claimed by this initial authored
driver. Steady Admission/per-frame costs remain a separate required M5 artifact.
