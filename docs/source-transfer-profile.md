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

## Observed baseline, 2026-09-05

Runtime/native-driver source: `b5f65ea27cf772ee412d6a9bfdb16f82e4df31a7`.
Wasm driver including explicit outside-timer retirement drain:
`1f8f29261a3a3cc251bdbe7451c8b43c477b798b`.
Raw observations are preserved in `source-transfer-profile-baseline.json`.
Exact generated artifacts remain in the owning lane at
`target/source-transfer-profile/b5f65ea/`; native/wasm JSON hashes are retained
in the committed evidence. No speedup or optimization is claimed.

Host: greywrought-dev, shared session-816.scope CPU 2, MemoryHigh 3GiB/Max 4GiB;
one admitted heavy command at a time. Rust 1.96.1, native dev profile
(unoptimized plus debuginfo), fresh release Wasm, wasm-bindgen 0.2.108 web,
Bun 1.3.13. Wasm is 4,451,239 bytes, SHA256
`7b8a5faacca0ac22425e8f62efca0031a14295f1efaa5bac38fcc6e63c92fa93`.
Each cell below is the median of three observations, excluding warm-up.

| Target / source | Outer off (ms) | Outer on (ms) | Runtime transfer (ms) | All lowering (calls; ms) | Old / new elaboration (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native dev / encounter | 3,534.29 | 3,657.17 | 1,643.39 | 10; 338.23 | 652.07 / 669.74 |
| Native dev / collections | 1,768.92 | 1,770.14 | 666.57 | 15; 95.94 | 288.53 / 274.30 |
| Wasm release / encounter | 5,353.06 | 5,451.87 | 309.22 | 4; 31.56 | 66.61 / 58.93 |
| Wasm release / collections | 1,469.10 | 1,583.59 | 167.42 | 6; 9.94 | 42.47 / 44.21 |

Native global phase totals include its preflight check as well as the measured
runtime transfer. They cannot be divided by the transfer column to infer that
phase's share. Wasm has one check per transfer. Old/new elaboration each occur
twice natively and once in Wasm. All profiles completed without truncation.
Inclusive medians overlap and medians do not add; use per-sample exclusive
measurements in the raw artifact for additive accounting.

Enabled-minus-disabled median outer time is +3.48% native encounter, +0.07%
native collections, +1.85% Wasm encounter and +7.79% Wasm collections. These
are noisy paired-mode observations, not isolated profiler-overhead estimates:
off/on ranges overlap in every case. Wasm encounter off range is
5,078.30–5,670.59ms versus on 4,913.98–5,888.63ms; collections off
1,453.81–1,736.93ms versus on 1,523.52–1,651.21ms. A no-hook binary comparison,
randomized order and larger replication remain unmeasured.

The collection source is 22,212 bytes versus encounter 20,847; its initial
CWR1 is **smaller**, 262,593 versus 791,420 bytes. It uses a different lowered
representation. These two variants are different workloads, not evidence
that more complex source is intrinsically cheaper.

### What the evidence permits next

The large Wasm outer delay is mostly **outside** the instrumented checked
runtime transfer: median per-sample outer-minus-transfer is 5,142.66ms for
encounter and 1,416.17ms for collections. This locates the main elapsed cost
in the surrounding passive adapter/binding/byte transport, but does not yet
separate parsing, copying, marshalling or event decoding. Profile those
boundaries before choosing an owning repair. No renderer is in this driver.

All four/six Wasm lowering calls together cost only 31.56/9.94ms median.
Removing repeated pure lowering by sharing the already-produced old/new
lowered values with snapshot metadata and relational projection **within
the same check** is an evidenced but limited opportunity; retaining the
first old/new lowerings means even those complete totals are not attainable
savings. Old/new elaboration is the larger measured compiler phase, and the
outer transport gap is larger still. No cross-source cache, skipped witness
replay, metadata omission, unchecked edit or weakened identity comparison is
licensed by this profile. Choose and measure the next repair accordingly.

Nearest observed checks: native created_collections 9/9, live_semantics 4/4,
source_profile 2/2; TS7 typecheck/build passed. Fresh-Wasm combined run:
referent 2 and live-semantics 1 passed, created-collections reached its
second-world open and failed SessionOccupied (3 passed/1 failed, 290 asserts).
The unchanged created-collections suite then passed alone (1/1, 47 asserts).
The failure is consistent with opening before scheduled deferred retirement
finishes, not a witness/collection assertion failure; no baseline attribution
or universal green claim is made and no existing test was changed. The
profiling driver drains those batches explicitly outside its timer and both
variants completed all seven samples. Consumer lifecycle/backpressure and
steady Admission/per-frame costs remain required M5 work.
