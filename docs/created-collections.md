# Runtime-created finite relations

Finite handlers can create referents, join their rows, and change them
atomically. Executable examples include
`clause:test-vectors/authoring/created-timed-contributions.clause` and
`clause:test-vectors/authoring/created-burn-extension.clause`.

## Executable meaning

Positive relation premises match actual typed pre-state rows. A rule-local
variable is a checked binding, not a foreign argument and not an enumeration of
declared subject names. Shared variables join exact values; independently
created referents remain distinct even when all their scalar values are equal.
Duplicate derivations of one exact substitution produce one match. Every
selected match evaluates against the same pre-state, then all row effects
commit atomically. A rule creation allocates a new identity from the runtime
allocation root, Step ordinal, source rule and exact matching substitution.

One/maybe row replacements conflict if multiple ordinary writes target the
same row. `accumulate` permits only numeric one/maybe rows with an existing
value; each occurrence contributes separately. Deltas are sorted by numeric
total order before finite arithmetic, independent of discovery/source order.
Ordinary writes mixed with accumulation reject the Step. Many-row effects are
exact set insert/remove operations: re-inserting an already-present value is
idempotent, but two simultaneous effect claims on that same value conflict.
Withdrawal requires its exact expected row/value. All these failures leave the
world, identity ordinals and accepted-event record unchanged.

Structured fields are typed tables, not opaque dynamic objects. Declared
subjects retain their original projected field views as selectors into those
same tables, not mirrored state. Declared facets follow checked structural
contracts or explicit nominal membership, without copying the referent's
identity or state. A bare creation binder infers one domain from its inserted
roles; cross-domain created facets remain unsupported. Required properties
are checked on the complete initial and candidate worlds. Scalar-domain
conflicts and nonnumeric contributions are rejected during source checking.
Boolean premises remain exact Boolean relation matches, and composed
scalar-law guards/origins remain checked.

## Evidence and transport

Runtime trace records each matched occurrence's bindings, exact row reads,
effect subjects and evaluated deltas. A failed complete positive search records
its visited-row count, not fictitious reads; short-circuited premises remain
unread. Exact offered handler Formation identities select diagnostics when
several handlers share a designation such as `tick`.

Checked scalar edits carry created identity bytes unchanged while the explicit
compiler continuity witness maps old/new nominal domains and declared targets.
A source snapshot address is not a stable identity across arbitrary text edits.
No-op/rejected/stale edits and unadmitted candidates retain the M4 contract.

CPP1 adds Binding (tag 25), RelationMatch (26), RelationEffects (27), and checked
ReferentFacet (28). Bindings must be in scope, with ordered finite membership
lists. The passive TypeScript projection adapter decodes typed relation table
Atoms and ordered set Terms; it performs no joins, expiry, combat or mutation.
Tables project as `{kind: "relation-table", subjectDomain, valueKind,
valueDomain?, cardinality, total, rows: [{subject, values}]}`; sets project as
immutable ordered arrays. Cardinality bytes 0, 1, and 2 represent one, maybe,
and many; byte 3 represents one with a required-property obligation. The
adapter exposes byte 3 as `cardinality: 0, total: true`, preserving the
distinction from mode-only guarantees. Decoders reject truncated/trailing
bytes, domain/cardinality mismatches and unordered or duplicate rows/values.

## Bounds and remaining scope

One Step allows 65,536 visited join row-values, 4,096 intermediate substitutions
per rule and 4,096 selected bound matches overall, with at most 128 local
bindings. Exhaustion returns `ResourceLimit`; it never establishes absence or
uniqueness from a prefix. Existing expression, wire, carrier, projection and
diagnostic bounds still apply; accepted execution trace is bounded to 4,096
rules and reports truncation. Join order deterministically favors keyed and
filtered row matches. Total guards move before remaining matches as soon as
their bindings exist; partial expressions retain their ordered guards and wait
for the row query. This avoids expanding a broad search after an independent
false condition, while genuinely exhausted queries still reject atomically.
A condition that needs the full join can still follow a search that exhausts.
These are explicit resource limits, not universal scalability claims.

This slice lowers connected general/scalar handlers, typed relation rows,
Boolean state guards and existing scalar-law composition. Positive recursive
laws also reclose over current rows after atomic changes, including creations.
Negation, aggregation, and allocation inside recursive rules remain unsupported,
as do general multi-input derived relations, specialization beyond the
checked exact-row selection index (`clause:docs/ordered-collections.md`), and a full
source-language type-system proof. Unsupported seams reject rather than route
to host gameplay. Per-frame projection size and compiler witness work still
need measurement against the application performance targets.

Reproduce nearest native checks with `cargo test -p clause-workbench --test
created_collections --test structural_contracts --test live_semantics --test
authoring_card --locked -j 2`. Build `created-collections-artifact` into
`clause:target/created-collections`, generate fresh runtime Wasm into
`clause:target/created-collections/wasm`, then run the browser package's
`test:created-collections` alongside
`test:live-semantics` and `test:referent-input`. These are compiler/passive
adapter checks; final DPR1 Greywrought browser adoption belongs to the consumer.
