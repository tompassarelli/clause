# Positioned finite occurrences and checked selection

`clause:test-vectors/authoring/ordered-measurements.clause` authors one
non-game transformation: calibrate the readings of active measurement batches
with `sqrt(?reading) / ?divisor`. Each measurement has its own Referent,
position, reading, and batch. Equal readings do not identify measurements.
Calibration changes only readings, atomically, preserving every position,
batch, and occurrence identity. Appended occurrences use the existing runtime
allocation law, not their position or reading as identity.

Positions are ordinary explicit F64 values, not physical table offsets.
The example's distinct positions give its collection order independently of
source declaration order and canonical Referent traversal. This is a positioned
finite-relation slice, not a new sequence value kind, dense-index guarantee,
ordering operation, or general collection library. Append accepts the supplied
position; uniqueness, contiguity, and integer positions are not inferred or
promised. Calibration preserves positions even when positions coincide.

## Checked physical specialization

Exact value selection in finite relation matching uses an inverse index over
one immutable typed pre-state table. Before using its buckets, the runtime
checks that:

- each bucket is nonempty and strictly ordered by the original row pair;
- each row has its exact source subject and value, and its value equals the key;
- the sum of bucket lengths equals the complete table's row-value count.

Strict order prevents duplicate row claims. Exact membership prevents invented
rows. Disjoint exact value keys and equal cardinality establish complete
coverage. Therefore each bucket is precisely the order-preserving subsequence
of source rows having that value, including all distinct equal-valued
Referents. The checked object borrows the pre-state, so it cannot outlive or
be reused against a changed table. No example name, role name, domain callback,
or second authored calibration rule selects this implementation.

This refines the existing lookup strategy; it does not reorder predicates,
change unification, alter match identities, or replace effect evaluation.
Existing canonical match ordering, visit charges, lazy evaluation boundaries,
resource failures, and atomic rejection remain unchanged. Invalid index
witnesses reject before lookup. The checker adds bounded work; no performance
improvement or new resource claim is asserted.

Execution, retained explanations, and isolated finite queries consume the same
checked source plan and matching path. The focused vertical checks unchanged
positions/identities, equal-valued declared and created occurrences, inactive
batch isolation, numeric rejection without admission, exact explanation
projections, and a query prediction equal to the recorded successor. Runtime
counterexamples reject omitted, duplicate, reordered, wrong-key, and invented
index rows and retain the existing visit-limit boundary.

Nearest checks:

- `cargo test -p clause-runtime --lib ordered_specialization --locked -j 2`
- `cargo test -p clause-workbench --test ordered_collections --test created_collections --test authoring_card --locked -j 2`

Native checks do not establish Wasm/browser performance, arbitrary structural
edit continuity, general sequences/maps, or a universal specialization proof.
