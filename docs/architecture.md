# Clause Architecture Assurance

> **Status:** Current implementation boundary and release contract.
>
> **Authority:** Derived and non-semantic. The
> [foundation](foundation.md) alone governs meaning, the
> [syntax](syntax.md) governs canonical source, and the
> [roadmap](roadmap.md) governs implementation status and order. This document turns
> their architecture boundaries into a release decision; it may not add an
> ontology, syntax form, or milestone.

<!-- clause-architecture-gate:v2 -->
<!-- milestone:M4:public-base:af9a0b9952f42f95851b47a071d9efb01a5fda0f -->
<!-- milestone:M5:public-base:23786abdb26c47638d819eea400555b0446a5451 -->

## Decision

A candidate is architecture-acceptable only when
`bin/architecture-gate FULL_GIT_OBJECT_ID M<N>` passes from a clean worktree
whose exact HEAD is that full commit, and the selected milestone's roadmap exit
proof also passes. The gate is a ratchet over checked code and exact evidence;
it is not a substitute for feature tests or independent review.

Unknown, incomplete, ambiguous, tampered, dirty, or mismatched evidence fails
closed. Public-base markers admit inherited architecture evidence only. They do
not make a later milestone implemented.

The accepted semantic authority is a checked ProgramSnapshot under an exact
ClauseSemanticsId. Program lineage, ProgramRevision history, source, caches,
runtime state, generated code, storage, host objects, refs, lifecycle records,
deployments, explanations, and receipts retain exact links to the semantic
content they concern; none silently becomes another ProgramSnapshot.

## Current implementation mapping

The live compiler now crosses an explicit checked-snapshot boundary while its
public result remains on the frozen Revision-v6 bridge:

```text
frontend::parse
  -> frontend::Program
  -> elaborate under ElaborationContext
  -> ProgramSnapshotCandidate + separate SourceMap
  -> validate(candidate)
  -> ValidationResult { ProgramSnapshot }
  -> explicit legacy checked-payload bridge
  -> CompiledProgram
  -> kernel::Revision { RevisionLineage, kernel::Model }
  -> RuntimeProgramRevision { ProgramRevisionId, ClauseSemanticsId, Revision-v6 }
  -> RuntimeSession / StateRevision / generated projections
```

These names describe live code, not final semantics:

| Current implementation type | Current job | Accepted destination |
| --- | --- | --- |
| `frontend::Program` | parsed source AST | lossless syntax plus source projection inputs |
| `ElaborationContext` | caller-owned semantic root scope and designation inputs | retained as the honest elaboration boundary |
| `ProgramSnapshotCandidate` | unchecked semantics epoch, root scope, and duplicate-preserving semantic atoms | consumed exactly once by validation; it has no identity and carries no source, lineage, authority, or policy data |
| `ValidationResult` | owns the checked `ProgramSnapshot` produced by comprehensive kernel validation | retained; no `ValidationContext` exists until validation has a genuine contextual input |
| `CompiledProgram` | legacy aggregate of named Revision-v6 values, requests, runtime journeys, designations, and SourceMap | migrate consumers to checked snapshots and explicit admission results |
| `kernel::Model` | current checked semantic payload container | `ProgramSnapshot` payload; it is not a model-theoretic Model |
| `kernel::Revision` | current envelope whose ID hashes lineage and Model payload | split `ProgramSnapshot`, `ProgramChangeOccurrence`, and `ProgramRevision` identities |
| current designation table | source mapping plus explicit ID-retention helpers | durable, lineage-aware Designation allocation and SourceMap evidence |
| `RuntimeSession` | immutable runtime-v3 history pinned to ProgramRevisionId, RuntimePolicyId, ClauseSemanticsId, and an explicit session-start occurrence while retaining Revision-v6 as the semantic oracle | replace the private legacy payload bridge only when constitutional admission directly produces the executable artifact |
| `StateRevision` | causal runtime-v3 node pinned to its session, ProgramRevision, policy, semantics epoch, predecessor, exact start/transition occurrence, and state payload | retain this identity law while future storage and migration evidence become explicit |

Canonical persistence is currently `clause-semantic-v10` inside
`clause-revision-v6`. The current `RevisionId` hashes both lineage and Model
payload, so it is neither the accepted `ProgramSnapshotId` nor the accepted
`ProgramRevisionId`. The identity/history seam now includes typed
`ProgramId`, `ProgramSnapshot`, `ProgramDelta`, `ProgramChangeOccurrence`, and
`ProgramRevision` with canonical preimages. Elaboration now constructs an
identity-free `ProgramSnapshotCandidate`; validation performs the complete
kernel check once and hashes one checked snapshot. An explicit private bridge
then extracts that checked payload for unchanged Revision-v6 consumers. Typed
runtime construction validates a real ProgramRevision against that exact
payload; generated standalone Rust carries a canonical ProgramRevision pin and
strictly reloads the runtime-v3 history. Typed admission, persistence, refs,
lifecycle, deployment, and durable Designation representations remain pending.

This mapping is the migration contract. Code using the old names remains real
and test-backed, but it cannot override the semantic vocabulary in the
foundation.

## Target pipeline

The corrected boundary is:

```text
read(SourceUnit)
  -> LosslessSyntax + SourceMap

elaborate(LosslessSyntax, ElaborationContext)
  -> ProgramSnapshotCandidate

validate(ProgramSnapshotCandidate)
  -> ValidationResult

record change(validated candidate, base ProgramRevision, AdmissionContext)
  -> ProgramChangeOccurrence

constitute(validated occurrence, base ProgramRevision or root)
  -> ProgramRevision

Revision existence is lifecycle-neutral; judgments and lifecycle decisions
determine acceptance and currentness.

execute(ProgramRevision, RuntimePolicy, SessionStartOccurrence)
  -> RuntimeSession -> StateRevision successors
```

Every contextual authority receives a distinct checked type only when the
operation has a real input for it. Validation currently needs only the
candidate, so `ValidationContext` is intentionally absent. `AdmissionContext`
remains a target type until admission accepts ProgramId, base revision,
authority, policy, and constitutive occurrence allocation. Source identity,
namespace, ProgramId, authority, policy, semantics epoch, and runtime session
identity may be related explicitly, but are never interchangeable defaults.

## Constitution

| ID | Required invariant | Reject the candidate when |
| --- | --- | --- |
| A1 | **One address protocol.** `ReferentId` is the sole general addressable semantic identity protocol. Content, role, pattern, snapshot, revision, occurrence, session, and state IDs identify their own structures; they are not rival Referent species. | A host type, source, table, object, storage path, event record, or target symbol becomes semantic authority or a second object ontology. |
| A2 | **Irreducible checked core.** The kernel owns Referents, named-role RelationalContent, AssertionOccurrences, Judgments and modalities, exact snapshot validation, and constitutional identities. | Parser convenience, target policy, storage layout, scheduling, adapters, or host resources enter semantic identity without an explicit semantic relation. |
| A3 | **Hard category boundaries.** Term/Designation, Referent, content, occurrence, Judgment, Disposition, snapshot, change occurrence, history node, attestation, and runtime state remain distinct checked structures. | Equality, liveness, absence, source position, or evidence accumulation silently converts one layer into another. |
| A4 | **N-ary named roles.** RelationalContent maps stable RoleIds to recursive terms at any arity. | A role is dropped, inferred from tuple position after elaboration, or flattened into a generic triple. |
| A5 | **Intensional identity.** Snapshot hashes commit to ClauseSemanticsId and canonical checked content; history-node hashes commit separately to ProgramId, parent, snapshot, and constitutive change occurrence. | Logical equivalence collapses independent occurrences, later evidence mutates a revision, or one hash silently changes meaning across semantics epochs. |
| A6 | **Explicit nominal continuity.** Referent and occurrence identity allocation is lineage-aware; local names and source spans remain projections. | A rename or move guesses continuity from similarity, position, or spelling. |
| A7 | **Physical freedom without a generic hot path.** The bounded interpreter may remain an oracle. Performance-sensitive state and target execution use compiled indexes, exact incremental changes, and specialized layouts while retaining every support. | A full relation scan or generic content interpreter is the state/target hot path, or a cache/layout changes meaning. |
| A8 | **End-to-end trace.** The chain from source/designation through occurrence, content, snapshot, revision, runtime session/state, and result/evidence is exact. | A result cannot name its exact semantic input, causal boundary, authority, or source/role origin, or a receipt is treated as truth. |
| A9 | **Replaceable strategies and explicit failure.** Storage and generated targets preserve canonical identities, results, provenance, and bounds; failures are checked outcomes. | A target or store leaks into semantic identity, or exceptions, retries, fallback order, or host accidents determine Clause meaning. |
| A10 | **Fail closed with visible obligations.** Resolution admits one exact elaboration; canonical reload recomputes identity and lineage; bounded work never overclaims completeness. | Ambiguity is guessed, tamper is normalized, partial work is certified as exact, or a milestone-crossing obligation lacks an executable exit. |

## Identity and parity gate

The constitutional migration must establish executable oracles before changing
the current representation. At minimum it must distinguish:

- equal snapshot payloads reached through different parents;
- equal parent and endpoint reached by different genuine change occurrences;
- additional attestations that leave revision identity unchanged;
- local rename retention versus delete-and-create;
- duplicate assertion occurrences over equal RelationalContent;
- explicit assertion of an already derivable consequence;
- equal checked payload under different ClauseSemanticsIds;
- equal state payload under different sessions, transitions, or policies; and
- ProgramRef, lifecycle, and deployment updates that do not mutate snapshots.

Every current M1–M7 capability selected for preservation also needs
before/after canonical identity, result, proof, runtime, and generated-output
parity. Tests must not rewrite expected identities merely to bless a migration.

## Milestone ratchet

The architecture gate currently protects the implemented semantic-v10 /
Revision-v6 line through M6. Those checks remain useful migration oracles; they
do not prove that the Program ontology is already implemented.

| Milestone | Additional architecture evidence |
| --- | --- |
| M1–M3 | One ReferentId domain; distinct content/occurrence/Judgment structures; exact named roles; deterministic source projection; strict canonical reload; bounded recursive evaluation and source-deleted generated-Rust parity. |
| M4 | Query holes remain scoped PatternIds; recursive correlation, projection cardinality, ordering, proof/support provenance, law-versus-derive authority, exact input Revision, bounds, and generated parity remain explicit. |
| M5 | Migration reports every source inference and proves source/designation to stable identity to exact successor continuity. |
| M6 | RuntimeSession and StateRevision replay binds exact ProgramRevision, RuntimePolicy, semantics epoch, session-start and transition occurrences, session-scoped predecessor history, deltas, and ordered inputs; additions and retractions use compiled dependency/support indexes rather than generic closure scanning. |
| M7 | Effect intent, authorization, attempt, receipt, observation, and admission remain separate; effect-trace-v2 binds ProgramRevision plus the post-commit StateRevision; generated JavaScript must contain no shadow semantics; real target claims require matched evidence. |
| M8 | One live ontology and source grammar remain; compatibility parsers, inferred declaration kinds, stale fixtures, and shadow consumers are absent. |

<!-- obligation:source-migration:fulfilled:M5:test=m5_migration -->
<!-- obligation:incremental-runtime-trace:fulfilled:M6:test=m6_replay -->
<!-- obligation:specialized-target-effect-trace:pending:M7 -->
<!-- obligation:single-live-surface:pending:M8 -->

The gate refuses a milestone when its obligation remains pending. Closing one
requires the narrow executable proof and marker change in the same commit;
prose or a renamed marker cannot make the gate green.

## M7 acceptance boundary

A frozen exact-state RenderPlan table proves data projection and target-byte
parity; it does not prove live JavaScript transition execution. A mechanical
host adapter proves its validation boundary; synthetic artifact or Three.js
substitutes do not prove a real browser vertical. M7 architecture acceptance
therefore requires generated transition authority, real-host evidence, source
mapping, and a matched specialized-target measurement in addition to the
effect and RenderPlan boundaries. The [roadmap](roadmap.md) alone records which
of those capabilities currently exist.

## Current architecture gap

`derive::saturate` remains the bounded reference oracle. The M6 runtime uses
compiled relation/rule and occurrence-root reverse indexes, semi-naive support
addition, and occurrence-exact affected-support retraction. Authored legacy
events, strict runtime-v3 replay, causal canonical state history,
ProgramRevision-bound effect evidence, and source-deleted generated Rust share
the frozen semantic-v10 / Revision-v6 oracle without reusing its ID as
constitutional program identity.

The Program identity/history foundation, identity/parity oracle, consumer
census, SourceMap split, typed elaboration boundary, checked-snapshot boundary,
and runtime identity v3 are complete. The next edge is rebuilding the canonical
agent-first surface on that corrected runtime boundary, followed by the live
generated host, applications, and empirical comparison.

## Running the gate

From a clean candidate worktree at exact HEAD:

```sh
candidate=$(git rev-parse --verify 'HEAD^{commit}')
bin/architecture-gate "$candidate"
bin/architecture-gate "$candidate" M6
bin/architecture-gate --self-test
```

The self-test attacks the gate's authority marker, milestone parser,
shadow-identity denial, severe-deferral denial, pending-obligation boundary,
and full-object-identity comparison. It does not rerun every milestone feature
suite. The selected gate runs the exact behavioral seams encoded by the
script; the [roadmap](roadmap.md) names the broader completion evidence.
