# Live source operations and recorded execution

This bounded surface is driven by the live encounter, not a general editor or
general counterfactual solver. Native and Wasm use the same normalized evaluator.

## Explicit edit

`ResidentSourceWorkbenchV1::scalar_effects()` offers exact artifact-scoped
handler/effect identities and display origins for scalar effect expressions.
`edit_scalar_effect(captured_handle, selected, expression)` replaces only that
selected expression subtree. It cannot add/remove declarations, facets, state
bindings, or handlers. It replays the operation, allocates a new source snapshot,
checks both snapshots, and relates each retained old address to its checked new
address. Equal names, source spans, initial values and arbitrary text similarity
are not continuity evidence. Structured-product field editing is not yet offered.

Exact no-op expressions preserve the handle, handler table and pending candidate.
Rejected edits do likewise. A changed edit requires no hidden candidate: settle
the candidate before submitting it. Successful replacement retires the old
generation, fences captured inputs and old candidates, and transfers state
inside runtime ownership. The first subsequent candidate and separate Admission
publish the carried world. Snapshot-local references are readdressed; their
explicit continuity relationship is not equality of the old/new coordinates.
`sourceContinuity(module, session)` exposes the checked old/new address map and
runtime-epoch-rooted continuing occurrence tokens. The tokens are separate from
snapshot addresses, survive subsequent explicit edits, and do not identify an
independently opened run as the same world.
Runtime-created referent occurrence bytes are retained while domain addresses
are mapped. Unstructured `hot_reload` remains a fresh import.

The new generation's CWR1 is a compiler artifact, **not** the native compiler
process's live browser world. `last_source_edit()` returns a CET1 witness with
the exact old source/root, new root, selected identities, replacement expression,
and old/new CPP1. No host configuration is included. Its bound is 4 MiB.

`exact_source()` immutably borrows the exact checked source installed in the
current native workbench generation. After an accepted edit, persist these
compiler-produced bytes rather than reconstructing the source in the host.
No-op and rejected edits leave these bytes unchanged.

The passive adapter's `editSourceSession(module, liveSession, generation,
newCwr1Request, cet1, policy)` applies that witness to the browser's captured live
Wasm session. It returns `SessionStarted`/`SessionFailed`, not an admitted frame.
Retain the currently displayed frame until the next normal Admission. Do not use
`startSession` for an explicit continuing edit. No page/server restart is needed.

### Compiler transport and workbench integration

Keep a resident native compiler workbench for the exact served source generation.
Offer its `scalar_effects()` catalog to the editor; each entry already supplies
artifact, allocated handler/effect, handler/expression origins and exact old
expression bytes. The UI displays those fields, captures the server generation,
and returns the offered node identity plus new expression bytes. Resolve the
selection against that generation's catalog, never by a host text search. The
compiler-side operation is:

```rust,ignore
let catalog = compiler.scalar_effects()?;
let selected = &catalog[chosen_catalog_index]; // index validated in captured catalog
let captured = compiler.generation().handle;
let edited = compiler.edit_scalar_effect(captured, selected, replacement.as_bytes())?;
if edited.handle == captured { return Ok(()); } // unchanged: no transport reload
let new_cwr1 = &compiler.generation().cwr1;
let cet1 = compiler.last_source_edit().expect("accepted changed operation has witness");
let attack_entry = clause_runtime::decode_executable_occurrence_v1(
    &compiler.handler_occurrence(b"party-attack", &[])?,
)?.entry;
```

The handler designation in this example is a consumer request, not compiler
dispatch. Send the new CWR1, CET1 and compiler-resolved diagnostic entries as
opaque transport data. Do not send the native process's configuration. Preserve
the old compiler generation until a changed operation has successfully checked;
no-op and rejected operations preserve it. `last_source_edit()` retains the last
accepted changed operation's witness; compare the returned handle before sending
anything so a no-op cannot retransmit an older witness. The complete executable artifact
producer is `crates/clause-workbench/examples/live-semantics-artifact.rs`.

`CartridgeWorkbench` deliberately does not expose its private session. A passive
port wrapper can capture the exact `SessionStarted.session` in `startSession`'s
completion callback, then intercept the next explicit-edit `startSession` call
made by `controller.reloadPackage(newCwr1Request)`. For that one call, invoke
`editSourceSession(module, capturedLiveSession, generation, newCwr1Request,
cet1, policy)` and forward its completion to the existing workbench callback;
capture the returned session only on success. Delegate ordinary initial/fresh
opens to the original port. Do not call the original `startSession` on the
explicit edit path: it intentionally opens fresh state.

Before requesting this reload, stop accepting old-generation inputs and let any
in-flight candidate reach its ordinary Admission/rejection callback. The runtime
rejects changed edits with a hidden candidate, so do not use reload as candidate
disposal. In the current synchronous Wasm port, candidate and Admission finish
within their callback turn; still check the workbench phase and pending host
queue at the consumer boundary. Install a single generation-paired pending edit,
not a global fallback to CET1 for every reload. Clear it after completion.
`reloadPackage` provides workbench generation fencing and retirement. Preserve
the last displayed admitted frame when it sees the replacement's empty bootstrap
frame; the next ordinary tick, candidate and Admission render the carried world.
SessionStarted alone is not a renderable world change.

Use the captured *current* Wasm session for `explainSession`, `interveneSession`
and `sourceContinuity`. The server generation, workbench generation, Wasm handle,
source snapshot and recorded Step are distinct identities: pair them explicitly.
A stale selection, diagnostic entry, candidate or input must not be relabeled as
current. Handwritten non-source `ExecutablePhysicalPlanV1` constructors must add
`source_metadata: None`; source compilation installs metadata automatically.

## Executed explanation

`recorded_event(handler)` and `explanation(handler)` expose an actually accepted
Step. Metadata alone is not execution evidence: predicates record their evaluated
prefix, real state/argument reads, selected rules, individual contributions and
before/after state. CPP1's CSM1 section binds handler and law origins, state roles,
facets and snapshot provenance to the executable artifact. It is not a rendered
world field or an assertion that any rule fired.

Only successfully entered Steps are retained. Speculative configuration checks
cannot manufacture an event. Retention is the latest event per physical entry,
up to 64 entries, so tick traffic does not erase a recent attack/heal. A later
event on the same entry replaces its record. Up to 4096 rules are retained per
event; `truncated` reports omissions. The diagnostic term reports evaluated
terminal premises on blocked rules and marks omitted preceding premises with
`premises-elided`; selected rules retain their full evaluated premises.

`explainSession(module, liveSession, checkedEntry)` returns the passive decoded
diagnostic, bounded to 1 MiB and never substituted for an admitted world frame.
It names the exact event Step and old source artifact. It explains that event,
not necessarily the battle's current state.

Relation states also expose paged `rows`, each carrying its exact typed subject
and recorded before/after values (omitted on the absent side). These are state
snapshots, not a claim that every row was read: premise reads still name the
actual evaluated rows/searches. `explanationRelationRows` passively decodes these
coordinates, retaining the state source metadata and created occurrence identity.

## Finite intervention

`intervene(query)` / `interveneSession(module, liveSession, queryBytes)` operates on a
retained event's exact pre-state and occurrence. Allowed changes are a caller-
supplied finite list of typed coordinate alternatives (at most 20). One value per
changed coordinate is enumerated. Cost is increasing changed-coordinate count,
then canonical `(slot, typed subject, typed value)` lexicographic order. The empty intervention
is evaluated first. Maximum evaluator runs is explicit and at most 4096.

Each hypothesis uses the same `prepare_step` evaluator as actual execution on
an isolated configuration. There is no mutable host evaluator, automatic input,
candidate creation, or Admission. The desired predicate is evaluated on the
predicted resulting configuration. Evaluator/domain/conflict/unsupported errors
reject the query; they are never counted as false hypotheses. `completed` means
all finite valid combinations were enumerated; `exhausted` means the evaluation
bound stopped a prefix. A found first solution is minimal under the declared
order and stops the search, so it need not report full enumeration. Exhaustion
is not impossibility.

`finiteScalarInterventionQuery` passively serializes Boolean/numeric alternatives
and a caller-supplied threshold or constant predicate. An optional exact typed
`subject` on an allowed change or desired coordinate addresses one relation row.
Rows must exist in the recorded pre-state, have the exact declared subject/value
domain, and have `one` or `maybe` cardinality; many-valued rows are not scalar
alternatives. Whole-table and row alternatives on the same slot reject as
overlapping. Different subjects in one table have independent costs; alternative
values for the same coordinate are mutually exclusive. Duplicate alternatives
and unchanged values do not increase cost. Queries never allocate missing rows.

CIQ1 slot-only bytes retain their meaning. CIQ2 adds a subject-presence tag and,
when present, the canonical typed referent after each allowed slot. Encoders use
CIQ1 for slot-only alternatives and CIQ2 for row alternatives. Both carry the same
normalized desired expression; row thresholds use `RelationRead`. Solution keys
for rows contain the slot and canonical subject bytes, never a row ordinal.
`projectedRelationRowValue` reads an exact predicted row without evaluating rules.
Neither API applies historical answers to a later moving world.

General source edits, arbitrary reconciliation, general recursive explanations,
unbounded/inverse solving and future-state tactical prediction remain outside
this implemented slice. Greywrought's same-open-page acceptance remains the
consumer completion boundary for Milestone 4.

## Reproduce the bounded gates

Within the repository's pinned Rust development shell (with its required C
compiler), from the Clause lane root:

```sh
cargo test -p clause-workbench --test live_semantics --test resident_source --test authoring_card --locked -j 2
cargo run -p clause-workbench --example live-semantics-artifact --locked -j 2 -- target/live-semantics
cargo run -p clause-workbench --example referent-input-artifact --locked -j 2
cargo run -p clause-workbench --locked -j 2 -- check-source test-vectors/authoring/live-encounter.clause
cargo build -p clause-runtime --target wasm32-unknown-unknown --release --locked -j 2
```

The Wasm build needs that same pinned Rust toolchain's declared wasm32 target.
Run exact `wasm-bindgen` 0.2.108 with `--target web` on
`target/wasm32-unknown-unknown/release/clause_runtime.wasm`, separately producing
`target/live-semantics/wasm`, `target/referent-wasm` and the repository's tracked
`browser/jump-arena-shell/generated/wasm` artifacts. From the browser package,
run `bun run build`, `bun run test:live-semantics` and
`bun run test:referent-input`. Apply outer command time bounds and serialize
these builds/tests on a resource-capped host. New diagnostics require the fresh
runtime and adapter together, not a source-only repin with old generated files.

The coupled native gate verifies the real attack's five contributions, retained
evidence across later ticks, minimum finite change replay through normal typed
inputs, actual healing law/derive provenance and +28 result, live world/facet
continuity, stable continuing tokens across two edits, stale/pending rejection,
and a non-game account example. The Wasm gate verifies the actual live session
and passive browser port; it is not a rendered Three.js page acceptance.

Observed on the bounded development host on 2026-09-05: native live semantics
4/4 (8.35 s), resident source 31/31 (4.71 s), authoring 3/3 (2.20 s);
fresh Wasm live semantics 165 assertions (18.60 s) and existing referent/target/
contribution adapter tests 2/2 with 91 assertions (2.33 s). TypeScript 7.0.2
build passed. Final generated Wasm SHA-256:
`7808bdf57cad4fdd5056917d3ef01d795cb719e86a591d919ea3f36e54532547`.
The encounter's checked CPP1 is 701397 bytes, CWR1 791420 bytes; CET1 is about
1.4 MiB. Valid live-edit wall times across the observed runs were 6030.75,
5256.52 and 5114.57 ms. Tracing currently adds work to normal evaluation and
query evaluation; no low-latency or performance-advantage claim follows from
these gates. Same-open-page rendered consumer integration remains unobserved
at this compiler handoff.
