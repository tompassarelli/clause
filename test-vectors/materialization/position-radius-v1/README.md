# Position/radius materialization falsifier v1

This corpus supplies normalized generic graph data and adversarial endpoint
streams for two physical materializations of one unchanged proximity law. It
is bounded translation-validation evidence for the construct-blind Rust
materialization crate; it is not a parser, compiler, supported Clause feature,
semantic promotion, or gameplay change.

`program.clause` is a human-readable source projection only, not parser or
compiler evidence. Its authored Designations are local spellings.
`source-context.json` is the machine-checked fixture context: it binds the
three declared relations and one declared law to exact typed paths in
`normalized-graph.json`. It explicitly classifies `point2-v1`,
`nonnegative-q16-16-v1`, and `within-radius-v1` as `unbound-unknown`; this
fixture does not guess their future compiler binding.

That context is fixture evidence, not authored qualification, semantic
identity, or a new Clause namespace syntax. `normalized-graph.json` retains
its exact slash-bearing fixture tokens as opaque physical references.

`normalized-graph.json` is the checked input used to construct exact graph and
plan root Terms, opaque physical premise/role bindings, invalidation keys, and
both candidate plans. The fixture adapter supplies Clause-owned exact-filter
evidence and conclusions as already-bound support data. The product Rust code
does not evaluate a Clause predicate, project a conclusion, infer a role from a
name, or validate a semantic relation mode. `streams.json` carries raw Q16.16
coordinates and workload sizes. `expected.json` carries the hand-calculated
inclusion set and named mutation gates.

The candidate P3 graph-to-physical binding is not admitted and is deliberately
not implemented here. The physical profile's IDs are fixture-local opaque
references; this corpus tests physical behavior without granting those IDs
semantic authority.

Numeric fixtures use signed big-endian four-byte Q16.16 atoms. `U = 65,536`;
the uniform-grid bucket width is `16U = 1,048,576` raw units; negative buckets
use mathematical floor division. Grid membership is only a candidate
superset. A candidate support is admitted only when the endpoint carries the
matching already-bound exact-filter evidence; neither Rust materialization
evaluates that filter.

Assertion occurrence IDs are independent of row content. Therefore equal
rows with different occurrences create distinct support records, and an
equal-content occurrence replacement changes provenance without changing the
visible output term.

`SHA256SUMS` covers exact tracked fixture bytes for transport integrity only.
It grants no Clause identity, authority, admission, or package equality.
