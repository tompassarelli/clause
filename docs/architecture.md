# Clause Architecture Assurance

> **Status:** Current architecture acceptance contract.
>
> **Authority:** Derived and non-semantic. The
> [foundation](foundation.md) alone governs meaning, the
> [surface](surface.md) governs authoring projection, and the
> [roadmap](roadmap.md) governs sequence and feature exit. This document turns
> their architecture boundaries into a release decision; it may not add an
> ontology, syntax, or milestone.

<!-- clause-architecture-gate:v2 -->
<!-- milestone:M4:public-base:af9a0b9952f42f95851b47a071d9efb01a5fda0f -->

## Decision

A milestone is architecture-acceptable only when
`bin/architecture-gate FULL_GIT_OBJECT_ID M<N>` passes from a clean worktree
whose exact HEAD is that full candidate Git commit, and its roadmap exit proof
also passes. The milestone argument is optional and defaults to the highest one
marked implemented or bound to an exact public milestone base above. A public
base marker admits architecture evidence only; it does not replace the
roadmap's feature exit proof. The gate is a ratchet, not a substitute for
feature tests. Unknown, incomplete, ambiguous, tampered, dirty, or unreviewed
evidence fails closed.

The checked Model remains the only semantic authority. Source, Revisions,
indexes, caches, schedules, runtime sessions, storage rows, target code, event
history, explanations, and receipts are projections or evidence with exact
links back to that Model. None may silently become a second program.

## Constitution

| ID | Required invariant | Reject the candidate when |
| --- | --- | --- |
| A1 | **One authority.** Addressable semantic identity has one domain, `ReferentId`. `ContentId`, `RoleId`, `PatternId`, and `RevisionId` identify content or engineering structure; they are not referent species. | A source, host type, table, object, store path, event record, or target symbol becomes semantic authority or a second identity ontology. |
| A2 | **Irreducible kernel.** The kernel owns referents, role-labelled content, occurrences, judgments and modes, exact Model validation, and Revision lineage. Derived contracts remain anchored to referents. | Parser convenience, target policy, storage layout, scheduling, adapters, or host resources enter semantic identity; a new kernel form lacks a failure that existing forms cannot express. |
| A3 | **Hard category boundaries.** Term/designation, referent, content, occurrence, judgment, and modality remain distinct checked types. Truth, derivation, acceptance, observation, intention, requirement, transition, effect, attempt, receipt, and external fact never imply one another. | One layer is inferred from another by structural equality, liveness, absence, source position, or host success. |
| A4 | **N-ary named roles.** Relational content maps stable `RoleId`s to recursive terms at any arity. Subject/predicate/object and tuple order are never the semantic representation. | A role is dropped, inferred from position after elaboration, or flattened into a generic triple. |
| A5 | **Physical freedom without a generic hot path.** The bounded interpreter may remain a reference oracle. Performance-sensitive execution uses compiled relation/role indexes, exact incremental additions and retractions, and specialized target layouts; proofs retain every independent support. | A full relation scan or generic content interpreter is the state/target hot path, or an index/cache/layout changes meaning. |
| A6 | **End-to-end trace.** The chain is exact: source span/designation → occurrence → content/Model → Revision → runtime plan/session/state → result, proof/support, diagnostic, or effect receipt. | A step cannot name its exact producer, input Revision, governing authority, or source/role origin; history or a receipt is treated as truth. |
| A7 | **Replaceable strategies.** Storage and target plans consume canonical identities and can be replaced while preserving results, provenance, bounds, and source-deleted parity. | Store rows, JavaScript objects, Rust types, indexes, or target addresses leak back into the Model or wire identity. |
| A8 | **No exception semantics.** Clause failures are explicit checked outcomes. Host panics may only expose an implementation invariant defect; catching, throwing, retrying, or crashing never determines Clause meaning. | A practical path uses an implicit exception, retry, fallback, or host control-flow accident as a language mode or result. |
| A9 | **Fail closed.** Resolution accepts one exact elaboration or reports every survivor and repair. Wire admission recomputes canonical identity and exact lineage. Bounded work never overclaims completeness or optimality. | Ambiguity is guessed, tamper is normalized, an unknown mode is accepted, or partial work is certified as exact. |
| A10 | **No hidden severe debt.** Every known architecture deferral names severity, blocking milestone, and executable exit. High or critical core debt blocks publication. | A high/critical deferral exists, a medium deferral crosses its blocking milestone, or the gate lacks an adversarial negative for a new boundary. |

## Milestone ratchet

The current reference evaluator is an oracle, not the final physical strategy.
That distinction is load-bearing: M3 proved checked recursive computation and
source-deleted parity, not specialized incremental target execution.

| Milestone | Architecture evidence due in addition to inherited checks |
| --- | --- |
| M1–M3 | One referent domain; distinct content/occurrence/judgment/modes; exact named roles; deterministic source projection; strict canonical Revision reload; bounded reference evaluation and source-deleted parity. |
| M4 | Holes remain scoped `PatternId` machinery, never referents; every query column retains its binder, complete recursive role-origin set, and presentation-only label through resolution, execution, canonical output, and source-deleted generation. Nested applications remain request-local, recursive correlation is exact, cardinality is explicit, laws remain inert until a distinct authorized rule projects them, and selection stays bounded. No compiled-performance claim is admitted. |
| M5 | Migration reports every source inference and proves source/designation → stable semantic identity → exact successor Revision continuity. |
| M6 | `StateRevision` and `RuntimeSession` bind exact Model Revision, predecessor, Delta, inputs, policy, and replay. Add/retract dependency work is incremental; generic closure scanning is not the state hot path. |
| M7 | Effects retain intent, authorization, attempt, receipt, observation, and admission as separate trace nodes. Generated JavaScript uses specialized layouts/indexes and contains no shadow domain logic; a matched reference/target measurement decides the hot-path claim. |
| M8 | One live surface and ontology remain. Ceremonial grammar, compatibility paths, stale fixtures, and shadow consumers are absent after exact migration parity. |

<!-- obligation:source-migration:pending:M5 -->
<!-- obligation:incremental-runtime-trace:pending:M6 -->
<!-- obligation:specialized-target-effect-trace:pending:M7 -->
<!-- obligation:single-live-surface:pending:M8 -->

The executable gate deliberately refuses a milestone when its pending marker is
due. The implementation that closes an obligation must replace the marker and
add the narrow executable check for the actual mechanism in the same change;
prose or a renamed marker cannot make the gate green.

## Current gap and debt boundary

At public M4, `derive::saturate` rebuilds the assertion set and tries generic
n-ary joins each round, while generated Rust embeds the target-neutral
reference evaluator. This is a **medium** physical-strategy deferral, blocks M6,
and does not block M4's bounded semantic selection slice. Its exit is an exact
add/retract dependency plan with proof-support preservation at M6 and a
specialized measured target plan at M7.

<!-- debt:medium:reference-evaluator:block=M6 -->

No high or critical core architecture debt is currently known. Any such finding
must add a gate-recognized `debt:high` or `debt:critical` marker; the gate
rejects either severity.

## Precedent boundary

The bounded paradigm proof already chose the reusable mechanisms. Clause uses
Soufflé/Datafrog-style compiled relation indexes and semi-naive deltas;
Differential-style add/retract work accounting; Unison/Nix-style
content-derived cache discipline without treating a hash or store path as a
referent; Durable Task/Temporal-style deterministic replay context without
making event history the Model; Koka-style explicit effect boundaries without
a primitive effect/type ontology; and Electric-style live-propagation
measurement without treating tables as semantic authority.

Clause rejects their incompatible ontologies and retains one role-labelled
Model, exact provenance, explanations, certified interventions, and
replaceable targets. The six proof axes are semantic unity and identity;
compounding authoring leverage; debugging/explanation/intervention;
incrementality/replay; target performance; and interop/transfer. After M3, the
first cross-axis falsifier is the same one-coin program under five changes:
format/focus only, rename, second coin, score change, and malformed `Vec2`.

## Running the decision

From the repository root:

```sh
candidate=$(git rev-parse --verify 'HEAD^{commit}')
bin/architecture-gate "$candidate"       # highest milestone marked Implemented
bin/architecture-gate "$candidate" M4    # exact candidate milestone
bin/architecture-gate --self-test
```

The self-test attacks the gate's own authority marker, milestone parser,
shadow-identity denial, severe-debt denial, pending-obligation boundary, and
full-object-identity comparison. It does not rerun a milestone's feature or
regression suite. The M4 decision separately runs exact M4/S1 selection and
M4/S2 rule-to-proof regressions. Together they cover descriptor and cardinality
parity, recursive request-local nested holes, strict semantic-v9 tamper
rejection, inert law and distinct rule authority,
governing-law/authority/scope proof trace, retained hospital proof and
intervention parity, alpha-label isolation, deterministic bounds, and
source-deleted generated output parity. M4/S2 admits at most 40,320
alpha-identity candidates per rule, then fails closed so canonicalization has an
exact work bound.
