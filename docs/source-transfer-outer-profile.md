# Outer checked source-transfer phases

This follow-on starts from the unoptimized, committed native/fresh-Wasm
baseline in `source-transfer-profile.md`. The large outer-minus-Transfer
residual does **not** establish JavaScript parsing alone as its cause.
The local phase attribution and owning byte-custody optimization are measured
below; browser-visible speed remains a separate consumer observation.

The opt-in synchronous JavaScript observer has 13 fixed phase counters and
at most 64 nested records. It retains only call counts and inclusive/exclusive
`performance.now()` wall milliseconds. Re-entry refuses without resetting an
active observation, unfinished scopes cannot finish, and truncation is visible.
Disabled hooks allocate no scope records and read no clock; additional calls,
branches and callback closures still affect both off/on modes. That overhead
has not been isolated from a no-hook build.

The observed path is:

| Boundary | Observation |
| --- | --- |
| Complete `editSourceSession` | `adapter` |
| JS exact witness byte validation | `witness-validation`, nested `byte-validation` |
| `parse_persistent_cartridge_bang` | `cartridge-parse` |
| `require_request` validation and immutable snapshot | `request-custody`, nested `byte-validation` and `frozen-byte-range` |
| Blob custody during CWR1 parsing | `frozen-byte-range` |
| CWS1 byte assembly and freezing | `cws1-assembly` |
| Typed arrays and wasm-bindgen call | `bulk-call`, nested `typed-array-construction` |
| Rust `source_edit_bulk` | `source-edit-bulk`, enclosing `clear-io`, existing `transfer`, `install-event` |
| Event retrieval through wasm-bindgen | JS `event-bulk`, Rust `event-export` |
| Event array custody / decode | `event-array-construction`, `cse1-decode` |
| New passive session handle/envelope | `session-construction` |

Rust's fixed report grows additively from 15 to 19 phase counters; existing
phase indices/names and source-checking signatures remain unchanged. The
new `clear-io` scope is before Transfer and `install-event` is after it.
`event-export` covers Rust's response copy, not all wasm-bindgen work.

The JS and Rust profiles are two nested views of the **same** operation, not
additive costs. For example, JS `bulk-call` contains typed-array construction,
wasm-bindgen marshalling, and all Rust `source-edit-bulk` work. Its exclusive
time excludes its JS children but still includes Rust work. Compare aligned
per-sample boundaries and subtract included children before attributing a
residual. Inclusive phase totals and medians must not be summed blindly.

The unchanged encounter/collection fixtures, exact CWR1/CET1 artifacts,
input ordering, one warmup and alternating three off/on samples remain the
baseline protocol. The driver records both trees around its existing timer.
Its arguments now include exact driver and runtime source commits separately;
the existing native artifact compiler is retained as `artifactCompiler`.
Use a new lane-local output directory, preserving the original baseline.
No new source or gameplay implementation is part of profiling.

## Explicit test lifecycle

The prior combined suite's SessionOccupied result occurred on reopening
while physical retirement was still scheduled. The passive test helper
`settleRetiredWasmSession` now invokes the existing bounded reclamation ABI,
with a visible failure after 4,096 calls. The collection test first proves
old custody stale, then drains retired storage **before** its existing
created-identity/world-state checks and second independent world. No
assertion, witness check, runtime backpressure or live ownership rule is
removed. Production scheduling is unchanged. The profiler performs the
same housekeeping outside every timer.

New focused tests cover observer return/throw transparency, re-entry,
incomplete scopes, finite nested accounting, truncation and bounded test
retirement. Native source-profile checks passed 2/2 and observer/lifecycle checks
passed 3/3. The complete adapter typecheck passed. The measurements below use
fresh Wasm and unchanged source witnesses with ordinary runtime verification.

## Local byte-custody result, September 5

Baseline driver/runtime: `9a52cfcd1c03ec42f38d2e6d4c8291021a7370e0`.
Optimized driver: `313e27e151997f6d8d39b37331450d2185d95054`.
Both use identical release Wasm SHA-256
`bdbd0ac82d0ad786277164c2ecb1a095a17f43319dfc4aa386c030f0f52b07bc`,
identical source/CWR1/CET1 fixtures, Bun 1.3.13 and a six-CPU/8GiB local scope.
One warm-up is excluded, followed by three alternating off/on samples per
variant. Native fixture preparation and all I/O/setup remain outside the
Wasm transfer timer. No renderer or competing heavy command participates.

| Fixture | Before off samples (ms) | After off samples (ms) | Median reduction |
| --- | --- | --- | --- |
| Encounter | 2930.90, 2906.13, 2900.14 | 268.12, 280.19, 241.21 | 90.77% |
| Collections | 868.77, 877.05, 893.20 | 124.09, 115.66, 114.71 | 86.81% |

The baseline larger fixture spends about 1.69s in frozen byte snapshots and
0.91s assembling the opening request, versus about 0.20s in the checked runtime
transfer. A bounded copy-method probe found that slice/Array.from followed by
freezing retained the cost. The owning change therefore keeps large internal
snapshots as immutable binary text, already used for canonical term custody,
and validates unused blob extents without creating discarded byte vectors.
Only the Wasm ingress materializes the exact byte array. Public request bytes,
wire formats, witness replay, runtime checking and world semantics are unchanged.

New producer-mutation and malformed-octet/blob-bound checks plus the existing
CWI1 command-custody check passed 3/3. The same fresh-Wasm transfer driver
completed both variants after the change. The remaining local transfer lower
bound is dominated by the actual runtime check; native compilation and the
browser's rendering/scheduling are separate costs. The older Droplet timings
are not like-for-like local comparisons. Three samples show a large local
effect, not an isolated hardware-independent bound or a browser FPS claim.

Raw artifacts: clause:build/source-transfer-outer-20260905/baseline and
clause:build/source-transfer-outer-20260905/immutable-bytes. The latter retains
the baseline compiler's exact fixtures and Wasm, changing only the driver.
