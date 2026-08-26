# Clause Semantic Foundation

> **Status:** Accepted and current.
>
> **Authority:** Sole authority for Clause semantics. The
> [syntax](syntax.md) governs canonical source projection, the
> [architecture](architecture.md) governs implementation boundaries, and the
> [roadmap](roadmap.md) governs implementation status and order.

This document is Clause's semantic authority.

## Decision

Clause is a distinction-first relational programming language with one
protocol for addressable semantic identity. Its executable artifact is a
Program, not a Model:

- A **Program** is one durable evolving lineage, identified by `ProgramId`.
- A **ProgramSnapshot** is one exact immutable checked intensional payload,
  identified by `ProgramSnapshotId` under an exact `ClauseSemanticsId`.
- A **ProgramChangeOccurrence** is the immutable causal occurrence that
  proposes or produces one program-history edge.
- A **ProgramRevision** is one immutable causal history node selecting a
  ProgramSnapshot within a Program.
- A **RuntimeSession** is one execution lineage pinned to a ProgramRevision,
  runtime policy, and semantics epoch.
- A **StateRevision** is one immutable runtime history node inside exactly one
  RuntimeSession.
- A **Model** is reserved for a meta-level interpretation satisfying a Theory
  under a declared semantic regime.

Routine source has no `model ...` grouping construct. Files, namespaces,
enums, shapes, scenes, and focused subject blocks contribute constituents to a
ProgramSnapshot candidate; indentation or source ownership never grants
program identity or authority.

## How the vocabulary developed

This lineage records why the current words were chosen. The earlier rows are
not alternative live semantics.

| Period | Term | What it was trying to name | Why it changed |
| --- | --- | --- | --- |
| Early prototypes | `World` | Whole semantic graph, source scope, and sometimes runtime state | One word had acquired incompatible jobs. `World` is retired as an architectural primitive; an authored world is now an ordinary Referent. |
| Distinction-first kernel | `Model` | The complete checked semantic value, separated from immutable Revision history | This correctly protected semantic content from source and storage, but conflicted with model theory and encouraged domain blocks such as `model world` to look like program roots. |
| Current foundation | `Program`, `ProgramSnapshot`, `ProgramRevision` | Durable lineage, exact content, and causal history respectively | The split follows established programming, version-control, and provenance distinctions while preserving Clause's occurrence and judgment semantics. `Model` returns to interpretation/satisfaction. |

The durable lesson across all three stages is unchanged: source layout, host
objects, storage rows, and runtime state are projections or evidence, never a
second semantic authority.

## Constitutional laws

1. Indentation determines syntactic containment.
2. A block head determines construct-local elaboration.
3. Indentation alone never invents a domain relation.
4. Source organization never creates semantic authority implicitly.
5. A spelling, path, span, host object, or movable ref is not semantic
   identity.
6. A Referent is an addressable semantic object; `ReferentId` identifies it.
7. One universal identity protocol does not imply one universal semantic sort.
8. Relational content is not an assertion occurrence.
9. An assertion occurrence is not a Judgment or current Disposition.
10. Program content, causal history, evidence, and lifecycle selection have
    different identities and lifecycles.
11. A runtime transition changes StateRevision, not ProgramRevision.
12. A program upgrade never silently rebinds runtime state.
13. Current status is derived from immutable judgments or records.
14. Snapshot identity is intensional identity over one canonical checked
    representation, not equivalence of consequences or behavior.
15. Every semantics-bearing hash commits to an explicit `ClauseSemanticsId`.

The corresponding source-design law is:

> A source tree expresses syntactic containment. Its block head may establish
> one explicit construct-local grammar, or a subject-focus block may require
> every child to name its own edge. The tree itself is never an unnamed domain
> relation.

## Referents, terms, and designations

A Referent is a stabilized distinction that can be addressed and reidentified.
Doors, spaces, relations, roles, laws, policies, programs, sources, and
occurrences may all be Referents while retaining checked kinds and distinct
admissible relations.

`ReferentId` is globally opaque and stable. It is independent of the Program
that currently contains or mentions the Referent. `Entity` is not a second
kernel identity category; it may be an authored category, a derived view, or
informal prose.

A term is a source or intermediate designator, not the Referent itself.
Resolution relates a term to one exact Referent in an explicit context. It may
use declarations, imports, role shape, and checked constraints; it may not
guess by capitalization, source order, similarity, or probabilistic English.

A `Designation` is separate metadata:

```text
Designation
  NamespaceId
  spelling
  ReferentId
  visibility/export status
```

Local designation edits and source moves do not change semantic identity.
Exported designations are explicit program-interface content and therefore do
participate in ProgramSnapshot identity.

New nominal identities are allocated only by an explicit lineage-aware
allocation or admission operation. Parsing a renamed spelling never guesses
continuity. A rename that retains identity explicitly changes the Designation
mapping while preserving the ReferentId; without that operation, deletion plus
creation is the honest result.

Assertion occurrence identity follows the same rule. Two identical claims may
be asserted independently and must not collapse because their text or
RelationalContent is equal.

## Relational content, occurrences, and judgments

Clause keeps these layers distinct:

```text
RelationalContent
  != AssertionOccurrence
  != Judgment
  != Disposition
```

- `RelationalContent` places one relation Referent in relational position and
  maps every participant to an exact named role. Arity, source word order,
  grammatical voice, and focus are projections of that role map.
- `AssertionOccurrence` is one independently identified act committing to
  content with constitutional provenance and scope. Equal content may have
  many occurrences.
- `Judgment` is one immutable authority- and policy-bearing assessment of
  content, an occurrence, a change, a revision, or another judgment.
- `Disposition` is a current policy-relative view derived from applicable
  Judgments. It is never a mutable status field inside a constitutional value.

Truth, derivability, acceptance, observation, authorization, intention,
requirement, execution, and external success are distinct modalities. An
effect receipt records an attempt and outcome; it does not make the intended
external proposition true. Authorities may disagree through separate
Judgments without mutating the subject they assess.

## Membership and structural views

Membership is ordinary relational content with `member` and `group` roles:

```clause
iron-door ∈ Door
```

Its semantic identity is an ordinary relation such as `core/member-of`.
Clause does not introduce a primitive `Classifier`, `Set`, or `Type` species
merely to license the group role. Any Referent may occupy that role unless an
explicit relation contract restricts it. Membership supports a derived
category or collection view of the group Referent; it does not transform that
Referent into a different kind.

Structural fields and roles are different from proposition-level membership.
A shape field such as `x: F32` describes a structural role. It neither asserts
`x ∈ F32` nor installs an object field on a domain Referent.

Type, value, object, field, record, set, function, variable, state, mutation,
checking, and evaluation are derived relational or structural views, not
additional semantic universes. A backend may specialize a functional relation
to a field, column, array, or index only while preserving the relational
meaning and exact identities.

## Laws, derivation, invariants, goals, and effects

These are addressable but not interchangeable:

- A universal **law** generalizes a relational pattern in an explicit scope.
  It does not execute or authorize derivation by itself.
- A **derivation authorization** permits an oriented operational projection of
  a law and retains the governing law, authority, and scope.
- An **invariant** is a candidate-admission obligation. Violation rejects the
  candidate under the governing policy.
- A **goal** describes desired content without asserting current truth or
  authorizing derivation.
- A **transition contract** defines possible state change. An accepted
  TransitionOccurrence causes one transactional successor.
- An **effect request**, authorization, attempt, receipt, observation, and
  admitted external claim are distinct evidence nodes.

An implementation may share machinery among these concepts only when the
checked representation preserves their modal differences.

## Open-world reasoning and derivation

Clause is open-world by default. Failure to find, derive, observe, or accept
content does not establish its negation. Explicit negative content, a rejecting
Judgment, an incompatibility constraint, and absence of evidence remain four
different things.

Closed-world reasoning requires an explicit finite scope and named governing
law or operational policy. Its result retains that scope and authority.

Positive derivation preserves every independent support. Retraction removes a
consequence only when its final support disappears. Caches, indexes, schedules,
proof selections, and derived closure are replaceable projections unless a
new assertion occurrence explicitly reifies a consequence into program
content.

## Program content and identity

A Program is the lineage humans mean when they say that a program has many
revisions or is deployed in several places. `ProgramId` identifies that
lineage; it does not identify a source file, namespace, snapshot, revision,
authority, or policy.

A ProgramSnapshot is the complete immutable checked intensional payload for
one exact version. Its canonical content includes, where present:

- Referents, roles, relation identities, and checked contracts;
- admitted RelationalContent and AssertionOccurrences with constitutional
  provenance;
- immutable Judgments authored as program content;
- definitions, laws, derivation authorizations, invariants, and goals;
- transition, event, capability, and semantic-policy contracts; and
- exported designations or explicit semantic source/authority relations.

It excludes incidental source layout and SourceMap data, formatting and
comments, local designation spellings, caches and schedules, replaceable
derived closure, ProgramRefs and lifecycle state, deployment attempts,
RuntimeSessions and StateRevisions, and host/storage/rendering layouts.

ProgramSnapshot identity is over a canonical checked kernel, not every
logically equivalent program:

```text
ProgramSnapshotId = H(
  "clause/program-snapshot/v1",
  ClauseSemanticsId,
  canonical_checked_payload
)
```

ProgramId is not included merely as snapshot ownership. Two Program lineages
that preserve the exact same ReferentIds, semantics epoch, and canonical
checked payload may share a ProgramSnapshotId; their ProgramRevisionIds remain
distinct because each revision commits to ProgramId. Independently recreated
Referents with equal spellings have different ReferentIds and therefore
different snapshots.

An independently asserted consequence changes the snapshot even when it was
already derivable, because a new AssertionOccurrence now exists. Conversely,
moving a source fragment without changing explicit semantic source relations
changes only SourceMap evidence.

`ClauseSemanticsId` identifies the meaning of canonical serialization,
normalization, identity resolution, structural checking, relation and role
interpretation, law and derivation semantics, transition semantics, and every
identity-relevant provenance rule. It is not a compiler build number.
Independent conforming implementations of one semantics epoch must be able to
produce the same checked bytes and IDs.

## Program change, revision, evidence, and lifecycle

Clause applies the same separation at the program-history layer:

```text
ProgramSnapshot
  != ProgramChangeOccurrence
  != ProgramRevision
  != RevisionAttestation
  != AdmissionJudgment
  != LifecycleDecision
```

A ProgramChangeOccurrence identifies the base revision or root, resulting
snapshot, canonical endpoint admissions and withdrawals, constitutive
responsibility/provenance, and semantics epoch. It may exist for a rejected or
unratified proposal and need not produce a revision. The authored change and
the canonical endpoint difference need not be identical.

A ProgramRevision binds only the constitutive causal-node fields:

```text
ProgramRevisionId = H(
  "clause/program-revision/v1",
  ClauseSemanticsId,
  ProgramId,
  predecessor_or_root,
  ProgramSnapshotId,
  ProgramChangeOccurrenceId
)
```

The initial design admits zero or one predecessor. Merge history remains
deferred until Clause has a concrete semantic merge requirement.

Repeatable, accumulable, contestable, or policy-relative evidence never
participates directly in ProgramSnapshotId or ProgramRevisionId. Therefore a
second verifier adds a RevisionAttestation without changing the revision, and
a later authority may add a conflicting AdmissionJudgment without mutating an
earlier Judgment.

Navigation, lifecycle, and deployment are also separate:

- `ProgramRef` is a movable name pointing to a ProgramRevision. Each movement
  has an immutable RefUpdate.
- `LifecycleDecision` is an immutable accepted/released/promoted/withdrawn
  decision naming authority, policy, target, time, revision, and evidence.
- `DeploymentRecord` describes an actual revision/artifact/environment
  attempt or observation and its receipt.

Production and canary may run different revisions simultaneously. “Currently
accepted” and “actively deployed” are derived views over records, not one
constitutional pointer.

These distinctions should reuse Clause's general referent, relation,
occurrence, and judgment machinery wherever it enforces their invariants. They
do not require a parallel provenance universe.

## Source and compilation boundary

A SourceUnit is authored input. A SourceMap connects semantic identities and
diagnostics to SourceArtifactIds, spans, and trivia evidence. Neither becomes
a Program or authority merely by existing.

Compilation separates contexts by type:

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

Judgments and lifecycle decisions determine acceptance/currentness; revision
existence itself is lifecycle-neutral.
```

The current `ElaborationContext` owns only caller-selected root scope and
designation inputs. The candidate owns its exact semantics epoch and unchecked
semantic atoms; SourceMap separately owns source and proposal spans used for
diagnostics. Validation has no policy- or resource-relative input today, so it
takes only the candidate and no ceremonial `ValidationContext` exists.

`AdmissionContext` is the target boundary for ProgramId, base revision,
authority, policy, and constitutive change-occurrence allocation. It becomes a
real type only when the admission API accepts those inputs and returns Program
history artifacts. Future namespace, import, SourceArtifactId, trivia, or
resource-bound inputs belong in their exact typed boundary rather than being
predeclared as optional fields.

There is no broad `ProgramContext` bag whose optional identities can silently
stand in for one another. NamespaceId, AuthorityId, PolicyId,
SourceArtifactId, ProgramId, and their contexts are distinct checked types.

## Runtime identity and migration

A RuntimeSession binds:

- `RuntimeSessionId`;
- `ProgramRevisionId`;
- `RuntimePolicyId`;
- `ClauseSemanticsId`;
- `SessionStartOccurrenceId`; and
- an initial StateRevision.

RuntimePolicyId identifies every immutable policy choice that can affect event
admission, scheduling, transition selection, effects, capabilities, or
successor computation. Two separately created sessions have different
RuntimeSessionIds even when program and policy match.

A StateSnapshot is the exact logical runtime payload at one boundary. It is
conceptually separate from the transition that produced it. Clause does not
add a public StateSnapshotId until a real consumer needs history-independent
state-content equality.

A StateRevision binds its RuntimeSession, predecessor or root,
TransitionOccurrence or session-start occurrence, exact StateSnapshot payload,
runtime policy, and semantics epoch. Equal state payloads reached through
different histories or sessions therefore have different StateRevisionIds.
Additional transition attestations do not change that identity.

A runtime event creates a StateRevision and leaves ProgramRevision unchanged.
A program upgrade creates explicit migration evidence and a new RuntimeSession;
it never silently reuses state under new program semantics.

## Theory and Model

ProgramSnapshot and StateRevision are object-language values. A Model is a
meta-level interpretation satisfying a declared Theory under a declared
semantic regime. Open-world or partial knowledge does not itself prevent
modelhood; an object-language artifact may constrain many possible Models.

Because Clause carries judgments, provenance, and derivation authorization, a
future Theory is likely a parameterized view of a ProgramSnapshot, applicable
judgment basis, entailment regime, and derivation policy. Until Clause has a
concrete Theory projection and satisfaction relation, `Theory` and `Model`
remain reserved and absent from the public kernel and routine source grammar.

## Acceptance laws

The implementation migration must prove at least these cases:

| Case | Required result |
| --- | --- |
| Same checked payload reached from different parents | Same ProgramSnapshotId; different ProgramRevisionIds |
| Same parent and payload, different genuine change occurrences | Same snapshot; different revisions |
| Same revision checked by two verifiers | One revision; two attestations |
| Attestation or judgment added later | Snapshot and revision identities unchanged |
| Source moves without an explicit semantic-source edit | Same snapshot and semantic identities; SourceMap changes only |
| Local designation rename with explicit retention | Same ReferentId and ProgramSnapshotId |
| Export designation rename | Same ReferentId; changed snapshot interface |
| Rename without retention evidence | Delete plus create; no guessed continuity |
| Two identical claims independently asserted | Same RelationalContentId; different AssertionOccurrenceIds |
| A derived fact is later explicitly asserted | Consequences may match; snapshot changes |
| Same payload under different semantics epochs | Different ProgramSnapshotIds |
| Same exact Referents and payload selected by two Programs | Same ProgramSnapshotId; Program-specific revision identities |
| Equal spellings with independently allocated Referents | Different ReferentIds and ProgramSnapshotIds |
| ProgramRef moves | No snapshot or revision change; new RefUpdate |
| Authorities disagree | Separate Judgments; policy-relative Disposition |
| Same state payload reached through different transitions | Different StateRevisionIds |
| Same program revision under different runtime policies | Different RuntimeSessionIds |
| Program upgrade | Explicit migration and new session |
| Production and canary differ | Multiple DeploymentRecords, not one deployed pointer |

## Prior-art boundary

Clause composes, rather than copies, several established distinctions:

- Datalog and Soufflé: authored executable logical artifacts are Programs;
- model theory and RDF semantics: theories/graphs differ from satisfying
  interpretations;
- Unison: semantic identity is separate from movable names;
- Git: content snapshots, causal history nodes, and movable refs differ;
- Datomic: immutable values and atomic successor transactions are useful time
  and persistence priors, without adopting closed-world epistemics; and
- W3C PROV-DM: fixed-aspect entities, producing activities, agents,
  derivations, and attestations differ.

Alloy is the strongest counter-prior: its authored artifact is coherently
called a model because its meaning is a family of satisfying instances or
traces. Clause's admitted occurrences, judgments, operational authorization,
program history, and runtime-state boundary make Program the closer term.

## Falsifiers

This foundation must be reopened if evidence establishes any of the following:

- the authored Clause artifact itself is necessarily a family of satisfying
  interpretations while program facts, provenance, history, and runtime state
  live outside it;
- no consumer needs to distinguish equal semantic snapshots reached through
  different histories or change occurrences;
- source placement is intentionally constitutional even without an explicit
  semantic-source relation; or
- membership requires a closed primitive classifier universe rather than an
  ordinary relation whose group role accepts Referents.

Absent such evidence, implementation terminology must migrate toward this
foundation rather than redefine it.

## Sources

- [Stanford Encyclopedia of Philosophy: Model Theory](https://plato.stanford.edu/entries/model-theory/)
- [Soufflé: Datalog programs](https://souffle-lang.github.io/program)
- [RDF Semantics](https://www.w3.org/TR/rdf12-semantics/)
- [W3C PROV-DM](https://www.w3.org/TR/prov-dm/)
- [Unison: content-addressed definitions and names](https://www.unison-lang.org/docs/the-big-idea/)
- [Pro Git: Git objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- [Pro Git: Git references](https://git-scm.com/book/en/v2/Git-Internals-Git-References)
- [Datomic transaction model](https://docs.datomic.com/transactions/model.html)
- [Alloy language reference](https://alloytools.org/spec.html)

These sources supply concepts only. No external implementation source is
copied or adapted.
