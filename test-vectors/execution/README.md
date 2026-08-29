# Clause v0 execution corpus

This directory freezes cross-host execution and replay observations for the
three programs selected by `clause:docs/execution-corpus.md`.

The `.clause` files are canonical source projections over syntax already
ratified by `clause:docs/syntax.md`. They are fixtures, not evidence that a
parser or runtime exists. Effect attempts have no `.clause` spelling here
because effect syntax remains unratified; their generic Term/Run boundaries are
recorded only in `manifest.json`.

`manifest.json` uses fixture-local names for Terms, claims, contexts, Runs,
occurrences, and revisions. Consumers must map them through generic Clause
representations. Matching these strings in a host feature enum or dispatch
table is a corpus failure.

The pure program also carries the exact finite generic ground-rule expansion
of its authored laws, so Lean and Rust can execute the oracle before the source
parser exists. Later elaboration must reproduce that expansion unchanged.

`SHA256SUMS` covers exact tracked fixture bytes for transport integrity only.
It grants no Clause identity, authority, admission, or package equality.
