# Clause v0 execution corpus

> **Status:** Frozen implementation contract; execution support is not yet
> implemented.
>
> **Authority:** The [foundation](foundation.md) alone defines Clause meaning,
> the [syntax](syntax.md) alone defines canonical source, and the
> [roadmap](roadmap.md) alone defines implementation status. This document and
> its corpus select concrete cross-host observations without adding semantics
> or syntax.

The v0 execution corpus fixes three nontrivial programs that the Lean reference
model, Rust runtime, CLI, and clean replay must consume unchanged:

1. a recursive pure dependency-closure query;
2. an admitted state transition followed by a separately identified effect
   attempt and receipt admission; and
3. two predecessor-bound Program changes, including rejection from a stale
   base.

The machine contract is
[`test-vectors/execution/manifest.json`](../test-vectors/execution/manifest.json).
The source projections use only syntax already ratified in
[`syntax.md`](syntax.md). Effect capability and attempt data remain generic
Terms in the manifest because their authored surface syntax is not yet
ratified.

## Representation boundary

Names in the manifest are fixture-local references. They are not Atom kinds,
host enum cases, package digests, or constitutional identities. A consumer
must lower every referenced value, claim, mode, capability, delta, trace, and
obligation through the same generic Clause Term and judgment representation.
Adding a construct-specific Lean or Rust execution branch to recognize a
fixture name fails the corpus.

The fixture lowering is exact and deliberately generic. A symbol is an Atom at
the published v0 structural index with kind byte `e0`, equality-contract byte
`e1`, and the exact UTF-8 bytes of its manifest string as canonical payload. A
binary relational fact is the neutral Triple `[subject, relation, object]`.
Fixture claims use the symbol Terms `fixture/type/proposition` and
`fixture/mode/asserted` for their type and mode. This selects reproducible
corpus data only; it does not admit the fixture Atom contract or make any such
claim semantically true.

The manifest's `ground_rules` are the finite generic expansion of the pure
program's two authored laws for this exact input graph. Each three-string fact
is shorthand for the same neutral subject/relation/object Triple. This lets the
Lean and Rust execution tranches consume one executable oracle before source
elaboration exists; the later parser must reproduce the same rules from source
and may not replace or reinterpret them.

The corpus deliberately does not publish `ProgramSnapshotId` or
`ProgramRevisionId`. The foundation describes their intended shape but does
not yet ratify an exact hash algorithm and preimage encoding. The opaque
revision names in the vectors are supplied by the admission context and may
not be derived or presented as public IDs.

Every assertion occurrence has a distinct fixture ID. Equal proposition
content does not merge occurrences. Every candidate delta names its exact base,
withdrawals, and admissions. A Run can propose a candidate context, but only an
explicit admission step changes the authoritative context.

## Required observations

The six Run outcome forms remain exactly:

```text
returned(value)
choices(finite-results)
yielded(value, continuation)
suspended(continuation)
failed(error)
exhausted(obligations)
```

For each corpus Run, Lean and Rust must agree on the selected mode, outcome,
candidate delta, inert trace data, and admission decision. Finite choices are
compared as the declared finite set; JSON order is transport order only.

The state/effect program has four distinct boundaries:

1. transition Run proposes inventory and order-state changes plus an effect
   intent;
2. admission atomically accepts that candidate State successor and intent;
3. a separately identified effect Run performs the authorized attempt and
   yields receipt data without rewriting Clause state; and
4. a later admission records the receipt claim.

Reasserting the recorded trace must not perform the effect again. A fabricated
receipt and an attempt without the admitted capability both reject.

The evolution program admits two changes in order. Reapplying the second change
to the root or to an equal-looking but different base rejects. Verification
evidence and lifecycle observations do not enter the fixture revision identity.

The published canonical-package lineage proves one exact bootstrap-to-successor
transition. The successor basis contains no root or rule authorizing a further
basis-admission claim, so a third package must reject under v0. The two Program
changes in this corpus run inside the already accepted successor semantics;
they do not pretend to extend constitutional package authority.

## Replay boundary

The corpus depends on exact bootstrap release
`2ea651db7c525249c465dceb0f8c5474d635fae6`. The final release manifest must
separately supply one exact published Git object that contains this corpus with
the tracked checksums; embedding that future object here would create a hash
self-reference. Two isolated replays must start from that same supplied release
object, use the repository-pinned Lean and Rust toolchains, and share no
checkout, build cache, package cache, output directory, runtime state, or
ambient configuration. They must independently observe identical:

- source and corpus bytes;
- accepted and rejected verification decisions;
- Run outcomes, candidates, traces, and admission decisions;
- canonical package bytes; and
- ordered evolution history.

The effect adapter used by replay is a deterministic isolated fixture. Each
replay may execute its own attempt once; replaying the resulting trace inside
either run must execute it zero additional times. Public CLI commands and
distribution paths are owned by the later CLI tranche and must satisfy this
contract rather than editing it around implementation choices.
