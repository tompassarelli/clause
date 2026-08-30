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

### Exact companion record

Every process-v1 case must commit exact input bytes and checksums, the selected
semantics and Program/world/session pins, identity-allocation authority,
Application/Activation/Run identities, ordered Step records with explicit
predecessor sets, observations and occurrence-exact supports, outcome,
continuation, candidate delta, Admission decision, and authoritative-boundary
hashes before and after. A physical case additionally commits its strategy,
budget, total work receipt, physical-view hash, and handle/scene table before
and after. A rejection fixes the rejecting stage, typed reason, obligations,
and all boundaries proven unchanged.

Names used while authoring the manifest are never semantic identity by
spelling. Once the process-v1 identity encoding is accepted, the companion
must carry its canonical ID bytes and preimages; equality/inequality-only
placeholders cannot satisfy an exact gate. Each identity record names the
Clause authority and predecessor evidence that allocated or retained it.
Wrong-kind, wrong-authority, self-authorizing, equal-content transplant, and
already-used-occurrence variants reject before partial authority. Historical
replay with an existing identity is not a fresh occurrence.

### Restarted continuation and causal schedule vectors

`resume-rematerialized-fresh-observation` must perform this exact sequence:

1. activate one Application and run through at least one nonterminal Step;
2. suspend at `suspend-step`, emit a boundary-crossing Continuation, and record
   its complete canonical bytes and pins;
3. terminate that executor and rematerialize the bytes in an independent
   runtime with no shared heap or handle table;
4. supply one newly identified ingress occurrence absent from the serialized
   remainder; and
5. resume the same Activation and Run, creating one fresh Step whose
   predecessor set is exactly `{suspend-step}` and one fresh ObservationId
   supported by that ingress occurrence.

The post-resume observation bytes and support must differ from every
pre-suspension observation; trace replay does not pass. Separate one-field
negative vectors change each of Application, Mode, ProgramSnapshot,
ProgramRevision, RuntimeSession, observed/base StateRevision, runtime policy,
semantics epoch, Activation, Run, emitting Step, remaining budget, cancellation
scope, and continuation-use authority. Additional vectors transplant unchanged
bytes to an equal-shaped independent Application, Activation, Run, or session.
All reject before a new Step. Under the fixture's Clause-declared linear policy,
the first use succeeds and both sequential and concurrent reuse reject as
`continuation-already-consumed` without duplicate observations or effects.

`join-left-first`, `join-right-first`, and `join-parallel` use fresh identities
in separate Runs but the same Clause process data. The first two force opposite
child completion orders; the third uses physically separate workers and a
barrier. In every Run the join Step's predecessor set is exactly that Run's two
child terminal StepIds. Neither child names the other as a predecessor. Joined
Value, observation content, occurrence-support multiset, candidate delta, and
Admission decision are equal; trace order is explicitly permitted to differ.

The race fixture carries its cancellation/yield/deadline decision table and
logical deadline boundary as Clause data. It fixes cases for yield causally
before cancel, cancel causally before yield, and cancel concurrent with the
deadline, then runs each under opposite queue order and worker count. Expected
Step frontiers, yielded observations, continuation disposition, typed terminal
outcome, and resource balance come from that table. Wall-clock arrival, log
order, and first host callback are not inputs.

### Truth without implicit assertion

One exact finite interpretation contains independently identified proposition
contents for supported true, explicitly supported false, and absent. Three pure
truth-directed Activations must emit respectively `true`, `false`, and
`absent`. The first two name their exact positive or negative assertion
occurrence supports. `absent` has no such support and never aliases false.
Hashes of the assertion set, governed Judgments, ProgramRevision, and
StateRevision remain unchanged in all three cases. A separate later assertion
and a separate later Judgment receive fresh occurrence identities; neither may
be retroactively attributed to evaluation.

### Total materialization receipts

Every materialization operation is accounted from API entry through the
returned receipt. The receipt contains exact counts for contract/input
validation, graph and support reads, index-bucket probes, premise occurrences
visited, candidate bindings, support entries read and written, whole-state
clones, whole-view rebuilds, support-set clones, disconnected rows visited,
allocation calls, allocated bytes, and peak live bytes. It also contains the
selected plan and fallback, typed failure or exhaustion, and whether a new
physical view was published. Deferred work and work performed by helpers,
copy-on-write layers, preflights, and receipt construction are included.

The companion declares concrete `base-population` and larger
`disconnected-population` integers and the exact receipt expected from each
plan. For the same local admitted delta, the larger indexed/incremental case
must report exactly zero whole-state clones, whole-view rebuilds, support-set
clones, and disconnected-row visits. Its remaining update counts and allocation
bound must equal the base case. Index construction has a separate receipt and
may scale. The cold scan reports its complete scaling work.

Separate vectors cover repeated premise slots, self-joins, equal content from
distinct Activations, and retraction with two independent supports. An
oversized extent must select the declared typed cold-scan fallback before any
unbounded indexed allocation. Exact `limit + 1` allocation and forced failure
vectors either complete through that bounded fallback or return typed
exhaustion. Every failure point preserves the previous physical-view and
support hashes, publishes no prefix, and retains exact support multiplicity.

### Wasm and passive-host vectors

The Wasm cases fix request/response bytes, exact length and allocation limits,
typed status, and handle table before/after. They include one valid pure round
trip; truncated and noncanonical canonical-package input; declared and actual
input length `limit + 1`; stale handle generation; stale Program,
StateRevision, and RuntimeSession pins; and a pure canonical result of exactly
`output-limit + 1`. Malformed or oversized input allocates no handle and starts
no Activation. Stale input performs no Step. Oversized output publishes no
prefix. Rejection never traps, wraps a length, mutates a semantic boundary, or
accepts a physical handle as semantic identity.

The passive-host cases fix immutable input-frame bytes, prior scene-projection
hash, expected render observation, resulting scene-projection hash, and listener
and resource counts. They include a valid admitted-StateRevision frame; two
independently allocated host objects containing that same frame; a stale
predecessor after its successor is displayed; missing and non-finite fields;
object count `limit + 1`; keyboard input after canvas focus loss; and input plus
render after disposal. Rejection leaves both caller frame and scene projection
unchanged. A valid render may change only renderer-owned physical objects.
Disposal is terminal and idempotent, removes every owned listener/resource
exactly once, and permits no later callback-owned work.

### Frozen ordinary-source specimens

The companion copies the three exact code blocks in the adoption spike's
“Frozen ordinary-source ergonomics” section into separately checksummed source
files. Pure definition and relational request source contain no process IDs,
revision pins, authority token, scheduler, budget, trace, or physical plan. The
state-change specimen exposes only its semantically relevant canonical process
words. Canonical parse/print/parse preserves the exact accepted projection,
while the semantic crosswalk still exposes every generated identity, pin, and
authority. A printer that injects process bookkeeping or a host sidecar needed
to recover hidden meaning fails the fixture.

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
