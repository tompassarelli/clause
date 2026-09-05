# Outer checked source-transfer phases

This follow-on starts from the unoptimized, committed native/fresh-Wasm
baseline in `source-transfer-profile.md`. The large outer-minus-Transfer
residual does **not** establish JavaScript parsing alone as its cause.
No owning optimization has been made and no new timing is claimed yet.

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
retirement. Native/fresh-Wasm checks and before/after measurement remain
pending heavy-slot admission. Reuse or other optimization may start only
after this expanded baseline attributes the dominant phase.
