# Clause v0 execution corpus and process-v1 boundary

> **Status:** Frozen implementation contract; execution support is not yet
> implemented.
>
> **Authority:** The [foundation](foundation.md) alone defines Clause meaning,
> the [syntax](syntax.md) alone defines canonical source, and the
> [roadmap](roadmap.md) alone defines implementation status. This document and
> its corpus select concrete cross-host observations without adding semantics
> or syntax.

The v0 execution corpus fixes three nontrivial programs that the Lean reference
model, Rust runtime, CLI, and clean replay must consume byte-for-byte unchanged:

1. a recursive pure dependency-closure query;
2. an admitted state transition followed by a separately identified effect
   attempt and receipt admission; and
3. two predecessor-bound Program changes, including rejection from a stale
   base.

The frozen machine contract is
[`test-vectors/execution/manifest.json`](../test-vectors/execution/manifest.json).
The source projections use only syntax already ratified in
[`syntax.md`](syntax.md). Effect capability and attempt data remain generic
Terms in the manifest because their authored surface syntax is not yet
ratified.

## Representation boundary

Names in the manifest are fixture-local references. They are not Atom kinds,
host enum cases, package digests, or constitutional identities. A consumer
must lower every referenced value, claim, mode, capability, delta, trace, and
obligation through the same generic Clause Term and provisional v0 candidate-
judgment representation. That frozen carrier is not the current governed
Judgment or process kernel. Adding a construct-specific Lean or Rust execution
branch to recognize a fixture name fails the corpus.

The fixture lowering is exact and deliberately generic. A symbol is an Atom at
the published v0 structural index with kind byte `e0`, equality-contract byte
`e1`, and the exact UTF-8 bytes of its manifest string as canonical payload. A
binary relational fixture item is the neutral Triple `[subject, relation,
object]`.
Fixture claims use the symbol Terms `fixture/type/proposition` and
`fixture/mode/asserted` for their type and mode. This selects reproducible
corpus data only; it does not admit the fixture Atom contract or make any such
claim semantically true.

The manifest's `ground_rules` are the finite generic expansion of the pure
program's two authored laws for this exact input graph. Each three-string item
is shorthand for the same neutral subject/relation/object Triple. This lets the
Lean and Rust execution tranches consume one executable oracle before source
elaboration exists; the later parser must reproduce the same rules from source
and may not replace or reinterpret them.

The corpus deliberately does not publish `ApplicationId`, `ActivationId`,
`StepId`, `RunId`, `ContinuationId`, `ObservationId`, `ProgramSnapshotId`, or
`ProgramRevisionId`. The foundation describes their intended shape but does
not yet ratify an exact hash algorithm and preimage encoding. The opaque
revision names in the vectors are supplied by the admission context and may
not be derived or presented as public IDs. Likewise, fixture-local v0 `run`
names are not process identity and may not be promoted by spelling.

Every assertion occurrence has a distinct fixture ID. Equal proposition
content does not merge occurrences. Every candidate delta names its exact base,
withdrawals, and admissions. The historical v0 Run envelope can propose a
candidate context, but only explicit constitutional Admission changes an
authoritative boundary.

## Frozen v0 observations

The six v0 outcome tags remain exactly:

```text
returned(value)
choices(finite-results)
yielded(value, continuation)
suspended(continuation)
failed(error)
exhausted(obligations)
```

For each v0 Run envelope, Lean and Rust must agree on the selected mode, outcome,
candidate delta, inert trace data, and admission decision. Finite choices are
compared as the declared finite set; JSON order is transport order only.

The state/effect program has four distinct boundaries:

1. the v0 transition envelope proposes inventory and order-state changes plus
   an effect intent;
2. admission atomically accepts that candidate State successor and intent;
3. a separately identified v0 effect envelope performs the authorized attempt
   and yields receipt data without rewriting Clause state; and
4. a later admission records the receipt claim.

Reasserting the recorded trace must not perform the effect again. A fabricated
receipt and an attempt without the admitted capability both reject.

## Required process-v1 companion

A new separately versioned companion must preserve every v0 byte and
observation while making the process kernel explicit. Its crosswalk must show:

- each checked closed form and exact resolved declaration closure;
- exact RelationSchemaId, RoleId, OperatorRef, and ModeId references formed from
  ProgramSnapshotId plus typed snapshot-local declaration identities, with no
  silent identity carry across a changed snapshot;
- nominal ApplicationId for every Application, with raw, open, quoted, or
  merely structural forms remaining non-nominal;
- two activations of one exact Application with distinct ActivationIds and
  RunIds;
- independently nominalized equal-shaped Applications with distinct
  ApplicationIds;
- one Activation producing multiple StepIds across yield, suspension, and
  resumption;
- finite predecessor frontiers rather than inferred log causality;
- exact ModeId, ExecutionAuthorization, ClauseSemanticsId,
  ProgramSnapshotId, ProgramRevisionId, RuntimeSessionId, RuntimePolicyId, and
  observed/base StateRevisionId pins where applicable;
- identified Continuation only where it crosses a suspension, persistence, or
  handoff boundary;
- ObservationIds distinct from Values, Results, and trace Terms;
- candidate deltas separate from continuations and Admission; and
- effect intent, authorization, attempt, optional receipt, observations,
  governed Judgment, and later Admission as a causal graph, with every actual
  event or attempt carrying an OccurrenceId plus exact provenance: producing
  Activation/Step for internal production or external-boundary provenance for
  an ingress trigger.

The companion must add exact cases for pure arithmetic, closure capture,
user-defined algebraic data with exhaustive matching, n-ary role closure,
duplicate equal Applications and Activations, an ongoing service, cancellation,
timeout without receipt, resumable exhaustion, source movement, repeated premise
slots, self-joins, and occurrence-exact supports. Malformed, open, wrong-mode,
missing or unreachable match cases, ungrounded-known-role, wrong-revision
resume, unauthorized cancel, fabricated receipt, and ambiguous-mode candidates
reject before acquiring partial authority.

Until canonical algebraic-data and match syntax is ratified, that companion
case is semantic IR plus exact observations and obligations, not a new source
spelling.

The crosswalk must not reinterpret the v0 fixture's `run` name as ActivationId,
StepId, or RunId; infer causality from JSON order; fabricate a receipt; or
silently migrate a live Activation to a new Program or world revision.

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
