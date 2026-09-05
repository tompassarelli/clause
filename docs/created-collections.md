# Runtime-created finite relations

This is the first M5 collection capability, not completion of Clause's wider
language ambitions. The preserved counterexample is
`test-vectors/authoring/created-timed-contributions.clause`: the prior compiler
rejected its tick/accumulate handler with `MissingExecutableBinding` at source
bytes 771..1048. The same mechanism is consumed by the unchanged real encounter
plus `created-burn-extension.clause`, composed by the executable fixture builder.

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
same tables, not mirrored state. Cross-domain declared facets require actual
checked shape membership of that exact referent. Created values do not acquire
unasserted domain facets. Scalar-domain conflicts and nonnumeric contributions
are rejected during source checking. Boolean premises remain exact Boolean
relation matches, and composed scalar-law guards/origins remain checked.

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
valueDomain?, cardinality, rows: [{subject, values}]}`; sets project as immutable
ordered arrays. Decoders reject truncated/trailing bytes, domain/cardinality
mismatches and unordered or duplicate rows/values.

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
Boolean state guards and existing scalar-law composition. It does not implement
arbitrary recursive relations, general multi-input derived relations over
created rows, reusable definition/strategy/specialization ambitions, or a full
source-language type-system proof. Unsupported seams reject rather than route
to host gameplay. The original source profile limitations remain outside the
connected path. Per-frame projection size and repeated compiler witness work
are costs to measure and repair in the following M5 performance slice.

Reproduce nearest native checks with `cargo test -p clause-workbench --test
created_collections --test live_semantics --test resident_source --test
authoring_card --locked -j 2`. Build `created-collections-artifact` into
`target/created-collections`, generate fresh runtime Wasm into its `wasm/`
directory, then run the browser package's `test:created-collections` alongside
`test:live-semantics` and `test:referent-input`. These are compiler/passive
adapter checks; final DPR1 Greywrought browser adoption belongs to the consumer.

## Observed checkpoint (2026-09-05)

Focused native collection tests pass 9/9, existing live semantics 4/4,
resident source 31/31, and compiler authoring-card 3/3. Exact Goal/account and
composed encounter `check-source` pass. Fresh Wasm plus TS7 passive adapter
checks pass 4/4 with 303 assertions (34.11 s), including occurrence bindings,
law origins, fractional lifetimes, explicit continuity, and non-game set views.
The generated runtime Wasm SHA256 is
`11fd5182de742340d557c951701438c64094f47c8bc79a708489b5f6b9071d8d`
(4,441,694 bytes). The encounter CWR1 is 262,593 bytes; CET1 is 367,416 bytes.
These sizes and tests are not a performance improvement claim.

A wider `canonical_source_arena` check reports 25 pass/4 failures. An exact
archive of the untouched published baseline
`c8a7a48fa79b2b54734a926f161bd39f3b463630`, compiled separately in this lane,
reproduces the same four assertions with identical differences:

- `scalar_handler_lowers_one_deterministic_rule_per_referent` expects Symbol
  selector constants;
- `transitive_referent_join_lowers_each_runtime_selectable_target` expects both
  Symbol policy constants but sees only policy-b;
- `boolean_law_lowers_typed_multi_subject_selector_cases` expects three rules
  but sees one;
- `general_handler_lowers_typed_constant_state_selector` expects two Symbol
  selectors but sees none.

No baseline fixture or assertion was changed. These are separately established
published-baseline failures, not a relabeling of the earlier 16 runtime/host
failures and not proof that their owning semantic issues are resolved. The
current actual encounter gates pass; broader soundness/review repair remains
in the program scope.
