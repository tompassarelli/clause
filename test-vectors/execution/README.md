# Clause v0 execution corpus

This directory freezes cross-host execution and replay observations for the
three programs selected by `clause:docs/execution-corpus.md`.

The three files under `historical-v0/source-projections/` deliberately use the
noncanonical `.clause-v0.txt` suffix. They are byte-frozen historical v0 source
projections, not current Clause source, parser inputs, or evidence that their
spellings are valid under the current grammar. No current canonical source is
included in this corpus. Effect attempts have no historical source-projection
spelling here; their generic Term/Run boundaries are recorded only in
`manifest.json`.

`manifest.json` uses fixture-local opaque transport strings for Terms, claims,
contexts, Runs, occurrences, and revisions. A slash in one of those strings is
the literal payload byte `/`; it has no namespace, qualification, relation, or
grammar meaning. Consumers must map each complete string through generic Clause
representations without splitting or interpreting it. Matching these strings
in a host feature enum or dispatch table is a corpus failure.

The pure program also carries the exact finite generic ground-rule expansion
of its historical authored laws, so Lean and Rust can execute the oracle before
the source parser exists. Later elaboration from separately ratified canonical
source must reproduce that expansion unchanged.

`SHA256SUMS` covers exact tracked fixture bytes for transport integrity only.
It grants no Clause identity, authority, admission, or package equality.
Run `./verify-historical-v0.sh` to check both the original program-byte hashes
and the noncanonical historical-source boundary.
