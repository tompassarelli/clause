# Clause Semantic Foundation

> **Status:** Accepted constitutional hypothesis; authorized for falsification
> by the [adoption spike](adoption-spike.md).
>
> **Authority:** Sole authority for Clause semantics. The
> [syntax](syntax.md) governs canonical source projection, the
> [architecture](architecture.md) maps the current implementation to this
> design, and the [roadmap](roadmap.md) governs implementation status and
> order.

Clause is a process-first relational programming language. Its authoring
surface remains declarative and relation-first: people state relationships,
laws, permissible transitions, effects, and physical constraints; Clause
determines how those distinctions may run and specializes them into efficient
execution.

This document is Clause's semantic authority.

The product mission does not depend on this mechanism surviving. Clause still
aims for exceptional readability, Lisp-level semantic extensibility,
correctness by construction, predictable systems performance, and one language
from native software through Wasm, JavaScript, browsers, and data systems. The
three-slot mechanism remains a falsifiable way to reach that mission, not a
reason to narrow it.

## Decision

Clause has one recursive language for what can be held, one contextual judgment
for what held structure means, one dynamic Run relation, and one authoritative
change boundary:

```text
Atomᵤ := opaque(kind, canonical-payload, equality-contract)

RawTripleᵤ := [Termᵤ, Termᵤ, Termᵤ]
Termᵤ      := Atomᵤ | RawTripleᵤ

ClauseJudgment := Γ ⊢ t clause : T @ M

RunOutcome := returned(value)
            | choices(finite-results)
            | yielded(value, continuation)
            | suspended(continuation)
            | failed(error)
            | exhausted(obligations)

Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩

Γ ⊢ Γ̂ admissible
───────────────────
admit(Γ, Γ̂) = Γ′
```

Running is primitive. A Term is a distinction that running has carried strongly
enough to become canonically holdable, referable, and reusable. A Clause is not
another data constructor; it is the `ClauseJudgment` over a Term. A Run carries
a judged Term toward an outcome and candidate continuation. Admission alone
makes a continuation authoritative. If the judgment itself needs occurrence
identity, Clause allocates an explicit `JudgmentId`; the judged Term does not
inherit that identity.

These are different resolutions of one architecture, not a temporal assembly
line in which static objects somehow exist before activity:

```text
running      activity and carry-through
distinction  a stable difference maintained by running
Term         the holdable and reusable face of a distinction
Clause       a typed contextual judgment over a Term
Run          an occurrence carrying a judged Term toward a verdict
trace        Terms describing a Run, never the Run itself
admission    validation making a candidate successor authoritative
revision     successful carry-through held as a stable context boundary
```

There is no first completed object called `Distinction` that must distinguish
itself. Runs occur in a base universe; higher universes may hold Terms that
describe those Runs; higher judgments relate occurrences, traces, and accepted
evidence. Reflection is well-founded rather than self-authorizing.

“Running comes first” is the architecture's ontological framing, not a circular
compiler rule. Operationally, Term construction, equality, Clause judgment,
Run outcomes, and admission are defined independently by the rules below. No
implementation may infer a Term's validity from a story about which earlier Run
created it, and no correctness claim depends on observing a metaphysical first
act.

## Clause Core transport contract

`Clause Core` names the canonical host-neutral transport contract for the
semantic objects defined by this foundation. It is not another constructor,
graph, context, revision, semantic substance, or authority. A Clause Core
package is a typed envelope carrying existing Clause objects between
implementations; merely constructing, decoding, checking, or persisting one
asserts and admits nothing.

Each package schema keeps three scopes explicit and disjoint:

- candidate or checked semantic material governed by the Term, judgment, Run,
  and admission rules in this document;
- Clause-native certificate proposals, derivations, supports, proofs,
  obligations, and rejection evidence; and
- separately typed source, strategy, trace, artifact, and physical evidence.

The selected schema must represent every Clause object and every declared
observable or nonfunctional contract required by its modes, including
capabilities, identity, cardinality, order, fairness, continuation,
cancellation, resources, effect sequencing, and canonical bytes. There are no
host-only semantic fields or side channels. Clause-native certificate data is
canonical package content; a host proof term, runtime object, pointer, cache,
or compiler-internal witness is local implementation evidence and never the
wire contract.

A package envelope is not a `ProgramSnapshot`. Only the canonical checked
payload enumerated under
[Program identity and history](#program-identity-and-history) contributes to
`ProgramSnapshotId`. Source maps, strategies, runtime traces, certificates,
caches, and physical evidence remain outside that identity unless an explicit
authored Clause judgment places their semantic content inside the snapshot.
Each check result binds the exact canonical package bytes, semantics epoch,
decoded sections, and claimed Clause judgments. Any admission operation
crossing this contract consumes that exact checked package and certificate
binding; it may not substitute a merely equivalent or separately decoded
candidate.

## Why three

Two slots can pair Terms, but cannot make both participants and their
relationship explicit without hiding the relationship in a participant or in
an external node kind. Three slots are therefore the smallest direct compound
form that can hold two Terms and their relating Term. Greater kernel arity is
unnecessary because higher-arity meaning can be represented by checked
neighborhoods or recursively complete structural Terms.

The positions of a `RawTriple` are structurally neutral. A Clause judgment may
interpret one under a relational profile as:

```text
[left Term, relating Term, right Term]
```

That representational three is not role-arity's operational
candidate/criterion/verdict three:

```text
candidate:  (Γ, t)
criterion:  M
verdict:    ⟨Γ̂, outcome, τ⟩
```

Role-arity analyzes functionally distinct operational positions. It can expose
hidden authority, state, result, resource, or failure roles, but does not prove
storage arity and does not replace type theory, operational semantics, or
computability theory.

## Terms, opacity, and equality

An Atom is contextually opaque, not metaphysically indivisible. Its kind,
canonical encoding, and equality contract define which distinction the current
universe can hold. Another universe may explicitly refine Text into Unicode
scalars, bytes, or machine words, but no context may silently inspect beneath
an Atom's declared abstraction boundary.

An Atom equality contract is declarative Clause data: total for its admitted
payload domain, deterministic, canonically serializable, versioned, and
committed by `ClauseSemanticsId`. It cannot be an opaque host callback.
Canonicalization must settle cases such as Unicode normalization, numeric
width, NaNs, signed zero, instants, case-normalized addresses, and foreign
handles; host-language equality or hash-table behavior is not the semantic
contract.

Structural equality is indexed by universe and semantics epoch:

```text
t₁ ≡ᵤ,sem t₂
```

Within that index, Atom equality is equality of kind plus canonical payload
under the declared contract, and `RawTriple` equality is recursive equality of
its three Terms. Terms from different universes or semantics epochs are not
structurally identical without an explicit migration or refinement Run.
Context-relative notions of sameness are value-equality or equivalence
judgments; they do not silently change structural equality.

Constructing, nesting, interning, serializing, or persisting a Term does not
assert it, execute it, authorize it, or place it in a world. An implementation
may hash-cons Terms and use integer handles, pointers, or row keys, but those
are private physical mechanics and never semantic identity.

Clause keeps these relations separate:

```text
Term structural equality
value equality
denotational equivalence
occurrence identity
concept continuity
runtime entity identity
revision identity
```

There is no mandatory nominal identity for every `RawTriple`. Clause allocates an
explicit nominal or coordinate Term only when continuity or occurrence matters,
including for:

- a source use, assertion, retraction, or Run occurrence;
- a binder or definition referenced independently of spelling;
- a concept lineage across structural revisions;
- a runtime entity or unique domain event;
- a Program, State, policy, semantics epoch, or revision;
- an effect intent, attempt, receipt, or observation; or
- a resource whose lifecycle is independent of its representation.

Identity Atoms compare as exact opaque canonical payloads. Structural equality
does not dereference the graph neighborhood named by an identity Atom.
Lineage-aware admission allocates fresh occurrence/entity identities; a
content-derived identity is allowed only for a value whose identity contract is
explicitly structural. Neither allocation strategy may hash recursively through
an identity reference back into its own graph.

`ReferentId` remains Clause's general opaque identity kind for an addressable
semantic concept. A Referent is a Term judged to have continuing nominal
identity, not a second universal data constructor. Names, paths, spans, host
objects, intern handles, and movable refs do not create or recover that identity
by similarity.

Recursive Terms are finite trees or DAGs under structural sharing. Semantic
cycles use explicit identity anchors:

```text
function-f = fresh FunctionId

[function-f binds x]
[function-f body [x calls function-f]]
```

The judgment graph may be cyclic through identity references without assigning
nominal identity to every `RawTriple` or hashing a structure through itself.
Canonical serialization writes finite Terms and opaque identity references; it
does not recursively inline the neighborhoods those identities name. Allocation
is well-founded, reload traversal is cycle-aware and terminating, and reload
rejects unknown kinds, foreign scopes, mismatched universes or semantics
epochs, dangling required anchors, and causal-lineage mismatch. Cross-epoch
conversion is an explicit migration Run, never permissive decoding.

## Clause is a judgment

```text
Γ ⊢ t clause : T @ M
```

This `ClauseJudgment` says that, in context `Γ`, Term `t` has type `T`, relational
meaning, modality, authority, and mode `M`. The same structural Term may be
judged as quoted syntax, a macro or query pattern, a pure expression,
proposition content, assertion content, a transition request, an effect intent,
a compiler plan, or a trace.

`RawTriple`, Term, `ClauseJudgment`, and judgment occurrence are therefore four
different notions: structural compound, holdable value, contextual meaning,
and an independently identified act of judging. Structure alone grants none of
the semantic roles. A proposition is not automatically
asserted. An assertion is not automatically true, authorized, current, or
executable. An effect description is not an effect occurrence.

Typing is a contextual restriction on how a Term may participate, run,
transform, and materialize. Graph shape is one class of constraint alongside
cardinality, effects, ownership, linearity, lifetime, totality, productivity,
temporal behavior, capability, representation, target support, and proof
obligations.

A relation mode declares its direction, known inputs, yielded results,
cardinality, failure and nondeterminism, effects, required capabilities,
identity policy, resource and temporal contract, and admissible strategies. A
pure function is a deterministic pure mode of a relation. Clause does not make
every computation perform logic search or require every relation to be
reversible.

Reserve **capability** for authority over effects and resources. A relation has
a kind, signatures, modes, laws, and strategies; callability is not itself a
capability.

## Run is the dynamic primitive

```text
Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩
```

- `(Γ, t)` is the candidate;
- `M` supplies the selected relation mode, criterion, laws, capabilities, and
  strategy;
- `ρ` identifies this occurrence when occurrence identity matters;
- `outcome` is `returned`, finite `choices`, `yielded`, `suspended`, `failed`,
  or `exhausted` as declared by the mode;
- `τ` contains trace, evidence, staged effect intents, diagnostics, or failed
  obligations appropriate to this Run phase; and
- `Γ̂` is a candidate successor context.

The arrow describes one observable Run step. A completed total evaluation may
use `⇓` as shorthand for a finite sequence ending in `returned`. A streaming or
reactive Run yields a value plus a typed continuation and can take another
step. A suspended Run has made no result claim. A partial Run may fail or never
produce another step; an enclosing bounded mode converts exhausted fuel or
resources into an explicit `exhausted` outcome rather than certifying
termination. A nondeterministic mode declares result cardinality and whether
results are a finite set, an ordered stream, or selected under a recorded
strategy.

Fairness, ordering, scheduling, cancellation, continuation persistence, and
resource budgets are mode or physical-strategy judgments whenever observable
or promised. They are never ambient host behavior.

`Run` is the semantic activity relation, not a requirement to allocate a
heavyweight runtime object for every pure computation. `ρ`, a durable trace,
and a revision are materialized only when occurrence identity, evidence,
replay, or authority requires them. A compiler may implement a pure
context-preserving Run as a direct call or specialized instruction while still
preserving its declared relation.

The same law specializes without multiplying dynamic substances:

| Form | Context result |
| --- | --- |
| Pure evaluation | `Γ̂ = Γ`; returns a value without authoritative change |
| Query | `Γ̂ = Γ`; returns choices or an answer stream |
| Macro expansion | proposes a successor syntax context |
| Elaboration | proposes a context containing typed judgments |
| Refactor or agent edit | proposes a successor Program context |
| Compilation | derives or admits a strategy/artifact context |
| Runtime transition | proposes a successor State context |
| Effect attempt | separately performs one authorized external act and observes attempt/receipt evidence |
| Rejection | `Γ̂ = Γ`; outcome is failure and trace contains exact obligations |

Clause does not claim a universal executability or termination decider.
Executable modes state honest obligations:

- **total** modes require a termination proof or reject;
- **productive** modes require finite observable progress;
- **bounded** modes enforce declared fuel or resource limits;
- **partial** modes include failure or possible divergence in their contract;
  and
- **reactive** modes expect continued running and require stepwise progress
  obligations.

Cycles are not inherently invalid. Recursive fixed points, services, streams,
and state machines may be productive under their declared modes. An ungrounded
cycle is rejected only where the selected mode promises a finite verdict or
another unmet property.

## Admission is the authority boundary

```text
Γ ⊢ Γ̂ admissible
───────────────────
admit(Γ, Γ̂) = Γ′
```

Running alone does not mutate authoritative Clause state. The target context decides
whether a result is constitutive, derived, observational, cached, speculative,
or authoritative. Pure evaluation and rejection preserve the context. A
compiler optimization may remain a replaceable derivation. A runtime transition
becomes current world state only after its identity, invariant, capability, and
effect-boundary obligations pass.

Clause has one authoritative change law: propose a typed successor to an
explicit context, then admit it or reject it with exact obligations. Source
elaboration, macros, refactors, migrations, compiler transformations, runtime
transitions, and AI edits are specialized Runs over differently typed contexts.
They do not thereby share authority or lifecycle. Admission governs Clause's
authoritative contexts; it is not rollback magic over an external system.

An ordinary state/effect protocol is deliberately two-phase:

1. a transition Run stages a candidate State successor and effect intents;
2. admission accepts the State successor and authorized intents atomically;
3. a separately identified effect Run performs each admitted intent at the
   external boundary and produces attempt, receipt, and observation evidence;
4. a later admission records that evidence and any resulting external claim.

If policy requires a different order, the mode must name the external
transactional adapter and its atomicity, retry, idempotency, and failure
contract. Once an external effect Run occurs, rejection or failure to admit its
evidence cannot undo the act or claim that nothing happened; the occurrence and
unadmitted evidence remain visible for reconciliation. A transition proposal
may never fabricate a post-act receipt for an effect that has not run.

Read by querying. Compute by running. Change by admission.

## Act and trace never collapse

A Run may produce Terms that describe its occurrence. Those Terms are a trace,
not the occurrence itself:

```text
[world-before collect world-after]
[attempt produced receipt]
[macro-call expanded-to output]
[compiler-run materialized artifact]
```

These may become accepted knowledge about what happened. They are not the
physical transition, external effect, expansion activity, or compiler process
happening. Reasserting or replaying a trace must not repeat the historical act.

This boundary is required for replay, retries, duplicate delivery, timestamps,
concurrency, cancellation, nondeterministic observations, partial failure, and
external uncertainty. An intent, authorization, attempt, receipt, observation,
and admitted external claim remain distinct.

## The admitted judgment graph

The first persistent compiler-owned semantic representation is an Abstract
Semantic Graph consisting of:

- recursive structural Terms;
- explicit occurrences and nominal identities where required;
- Clause judgments, schemas, types, modes, and capabilities;
- scopes, binders, uses, macro origins, and phase relations;
- derivations, supports, obligations, proofs, and explanations;
- Program and State revisions;
- physical strategies and artifact mappings; and
- trace Terms describing observed Runs.

A random graph, a parse graph, a rejected candidate, or a speculative optimizer
graph is not the program. An accepted ProgramRevision selects the admitted
judgment graph that is the program at rest. Runs are the program in motion.

The graph is semantic authority because it holds every relationship that may
affect meaning. It is not literally the living activity it records. A lossless
CST remains necessary for tokens, indentation, comments, whitespace, errors,
and incomplete edits, but it is a projection-recovery structure rather than a
sovereign AST.

No giant host enum or collection of construct-specific validators may privately
decide what `if`, lambda, match, transition, or a user extension means. Schemas,
readings, typing rules, completion rules, and transformations are Clause
judgments interpreted by a small generic kernel. Host code may bootstrap that
kernel and optimize checked meaning; it may not retain a second secret
language.

## Relations and higher arity

The relational profile reads a Triple as:

```text
[left, relation, right]
```

Higher-arity structural values must include all role assignments in their
canonical recursive Term. A partially described structural root may not gather
unrelated edges merely because two applications share some content.

When a relation instance has independent continuity—such as a particular
transfer, binder, task, payment, event, or effect attempt—it uses an explicit
identity Term as its anchor:

```text
[transfer-42 actor Alice]
[transfer-42 amount $10]
[transfer-42 from Checking]
[transfer-42 to Savings]
```

Two equal transfer descriptions may therefore denote different occurrences or
entities. A schema requires stable named roles, exact role types and
cardinality, complete coverage, source-order independence, and atomic
admission. An incomplete neighborhood is a provisional candidate, never half
an admitted value.

The existing named-role n-ary representation remains a useful checker view,
index, API, and packed runtime materialization. It is not the target's
irreducible semantic substance.

### Membership and structural views

Membership is ordinary relational content, canonically identified by a
relation such as `core/member-of` with `member` and `group` roles. Clause does
not introduce a primitive `Classifier`, `Set`, or `Type` species merely to
license the group role. Any Referent may occupy that role unless the relation's
explicit contract restricts it. Membership may support a derived category or
collection view; it does not convert the group Referent into another kind.

A structural field or role is not proposition-level membership. A shape field
such as `x: F32` describes one structural role; it neither asserts `x ∈ F32`
nor installs an object field on a domain Referent. Type, value, object, field,
record, set, function, variable, state, mutation, checking, and evaluation are
typed relational or structural views, not additional primitive semantic
substances or identity universes. Physical representations may specialize
those views only while preserving their judged meaning and exact identities.

## Bindings, macros, and language extension

Uses relate to explicit binder identity Terms. Names are readable designations,
not binding identity. Closure capture, recursion, shadowing, hygiene, and rename
operate on those identities rather than on spelling or tree position.

Quotation, syntax transformation, typed elaboration, semantic transformation,
and refactoring are Runs over stratified contexts. Their results preserve
origin, binding, type, effect, dependency, identity, and failed-obligation
relationships. Macro phases are deterministic and fuel- or termination-bounded
under their declared modes.

The constitutional extension test is:

> Can Clause add a new language concept by adding Clause judgments, or must the
> host learn a new semantic secret?

After the generic host kernel is frozen, a new construct involving both binding
and effects must be implementable through Clause-authored schemas, readings,
modes, and transformations while inheriting parsing, printing, hygiene, typing,
capability checking, navigation, refactoring, invalidation, lowering,
diagnostics, explanation, and trace semantics. Requiring a new host semantic
enum, validator branch, formatter case, refactor rule, or analysis plugin
falsifies the universal-substrate claim.

The extension's definitions must remain ordinary inspectable Clause Terms and
judgments executable by the frozen generic machinery. A “generic” opaque host
callback, per-construct dispatch table, foreign evaluator, or serialized tag
whose meaning exists only in host code is still a second semantic authority.
Irreducible FFI primitives are allowed only behind explicit typed effect,
capability, identity, and trace contracts; they cannot define the meaning of a
Clause language construct.

## Source projection

Human-readable source is a canonical bidirectional projection, not the
program's identity. Parsing may use a transient lossless CST. Every source line
elaborates to a Term and a designated focus; every indented child receives the
parent's focus as its omitted left operand. The parent reading chooses focus.
The child never guesses a relation from indentation.

Reading lookup is deterministic from the explicit head/operator, declared
grammar, and already selected ElaborationContext before child domain semantics
are inspected. Missing or competing readings are explicit errors. Schema and
type checking may reject the resulting candidate, but may not regroup the CST
or reinterpret siblings. Incremental parsing and recovery therefore depend on
syntactic boundaries and declared readings, never on successful whole-program
inference.

Conceptually:

```text
elaborate(line) -> (term, focus)
```

For a bare subject, term and focus are that subject. For a completed relation,
the relation Term may become focus. A header with a declared open slot may
allocate a structural or nominal focus. Indentation itself never means
membership, body, containment, application, ownership, sequencing, or
authority.

For every closed printable source context:

```text
elaborate(print(P)) ≅ P
print(elaborate(source)) = canonical(source)
```

The equivalence explicitly accounts for layout, comments, source occurrence
identity, and fresh nominal allocation. Stable concept continuity belongs to
the admitted graph, not coincidental source position. Ordinary source must not
expose graph bookkeeping ceremony.

## Relational knowledge

Clause keeps semantic modalities distinct even when they share generic pattern
or Run machinery:

- a universal **law** generalizes a relational pattern in an explicit scope;
  it neither executes nor authorizes derivation by itself;
- a **derivation authorization** selects an oriented executable mode while
  retaining the governing law, authority, and scope;
- an **invariant** is a candidate-admission obligation whose violation rejects
  the candidate under the governing policy;
- a **goal** describes desired content without asserting current truth or
  authorizing derivation;
- a **transition contract** describes permissible state change, while one
  accepted TransitionOccurrence causes one transactional successor; and
- an effect request, authorization, intent, attempt, receipt, observation, and
  admitted external claim remain distinct occurrences or judgments.

Truth, derivability, acceptance, observation, authorization, intention,
requirement, execution, and external success are therefore not aliases.

Clause is open-world by default. Failure to find, derive, observe, or admit a
proposition does not establish its negation. Explicit negative content, a
rejecting judgment, an incompatibility constraint, and absence of evidence
remain distinct. Closed-world reasoning requires a finite scope and an explicit
governing mode or law.

An assertion occurrence is an independently identified act committing to
proposition content with provenance and scope. Equal proposition Terms may have
many assertion occurrences. A Judgment is an immutable authority- and
policy-bearing assessment. A current Disposition is a derived policy-relative
view, never a mutable status field inside the proposition or assertion.

Universal laws remain inert until a separate derivation authorization selects
an operational mode. Positive derivation preserves every independent support;
retraction removes a consequence only when its final support disappears.
Caches, schedules, proof selections, and derived closure are replaceable unless
explicitly admitted as program content.

## Program identity and history

- A **Program** is one durable evolving lineage, identified by `ProgramId`.
- A **ProgramSnapshot** is one exact immutable checked intensional judgment
  graph under an exact `ClauseSemanticsId`.
- A **ProgramChangeOccurrence** is the causal occurrence proposing one program
  history edge.
- A **ProgramRevision** is an immutable causal node selecting one snapshot in a
  Program.
- A **RuntimeSession** is one execution lineage pinned to a ProgramRevision,
  runtime policy, and semantics epoch.
- A **StateRevision** is one immutable runtime history node inside exactly one
  RuntimeSession.
- A **Model** is reserved for a meta-level interpretation satisfying a Theory,
  not an authored source block or executable artifact.

Routine source contributes Terms and judgments to a candidate ProgramSnapshot.
Files, namespaces, source blocks, host objects, storage rows, and heap layouts
do not grant program identity or authority.

A ProgramSnapshot's canonical checked payload includes, where present:

- Referent and identity Terms, roles, relation identities, equality contracts,
  types, and checked schemas;
- admitted relational or proposition content and independently identified
  assertion occurrences with constitutional provenance;
- immutable judgments authored as program content;
- definitions, laws, derivation authorizations, invariants, and goals;
- transition, event, capability, effect, and semantic-policy contracts; and
- exported Designations and explicit semantic source or authority relations.

It excludes incidental source layout, SourceMap data, formatting, comments,
trivia, local Designation spellings, caches, schedules, replaceable derived
closure, ProgramRefs, lifecycle state, deployment attempts, RuntimeSessions,
StateRevisions, runtime traces, and host, storage, rendering, or target layouts.
An excluded item enters snapshot identity only through an explicit judgment
that makes it authored program content.

ProgramSnapshot identity is intensional over that canonical checked payload,
not over all logically or behaviorally equivalent programs:

```text
ProgramSnapshotId = H(
  "clause/program-snapshot/v1",
  ClauseSemanticsId,
  canonical_checked_payload
)
```

`canonical_checked_payload` is the canonical encoding of the exact checked
judgment graph just enumerated. `ClauseSemanticsId` commits to canonical Term
encoding and equality, normalization, identity resolution, structural checking,
relation and role interpretation, law and derivation semantics, transition
semantics, and every identity-relevant provenance rule. It is not a compiler
build number. Independent conforming implementations of one semantics epoch
must produce the same bytes and IDs.

`ProgramId` is not included merely as snapshot ownership. Two Program lineages
that preserve the same ReferentIds, semantics epoch, and canonical checked
payload may share a ProgramSnapshotId; their ProgramRevisionIds remain distinct.
Independently allocated Referents with equal spellings produce different
snapshots. A migration to the Term kernel requires a new semantics epoch and
parity evidence; it must not reinterpret existing snapshot bytes or IDs. An
independently asserted consequence changes the snapshot even if it was already
derivable, while moving source without changing an explicit semantic-source
relation changes only SourceMap evidence.

A ProgramChangeOccurrence records the base revision or root, resulting
ProgramSnapshot, canonical endpoint admissions and withdrawals, constitutive
responsibility and provenance, and semantics epoch. It may describe a rejected
or unratified proposal and need not produce a ProgramRevision. The authored
change and canonical endpoint difference need not be identical.

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

The initial design admits zero or one predecessor; merge history remains
deferred until a concrete semantic merge requirement exists. Attestations,
AdmissionJudgments, lifecycle decisions, deployments, and movable ProgramRefs
remain separate records. Repeatable, accumulable, contestable, or policy-relative
evidence never enters either identity preimage. A second verifier therefore
does not change revision identity.

- A `ProgramRef` is a movable name pointing to a ProgramRevision; every movement
  has an immutable `RefUpdate`.
- A `LifecycleDecision` is an immutable accepted, released, promoted, or
  withdrawn judgment naming authority, policy, target, time, revision, and
  evidence.
- A `DeploymentRecord` describes an actual revision, artifact, and environment
  attempt or observation together with its receipt.

Production and canary may select different revisions simultaneously. Current
acceptance and active deployment are derived views over records, not one
constitutional pointer.

Names are explicit metadata:

```text
Designation
  NamespaceId
  spelling
  ReferentId
  visibility/export status
```

A proven rename changes the Designation while preserving identity. Without
lineage evidence, delete plus create is the honest result. Exported
Designations are interface content and participate in ProgramSnapshot identity;
local spelling and incidental source layout remain projection evidence.

## Source and admission boundaries

A `SourceUnit` is authored input. A `SourceMap` relates semantic identities,
occurrences, and diagnostics to SourceArtifactIds, spans, and trivia evidence.
Neither is a Program or authority merely by existing. The typed boundary is:

```text
read(SourceUnit)
  -> LosslessCST + SourceMap

elaborate(LosslessCST, ElaborationContext)
  -> ProgramSnapshotCandidate

validate(ProgramSnapshotCandidate)
  -> ValidationResult

record_change(validated candidate, base ProgramRevision or root,
              ProgramAdmissionContext)
  -> ProgramChangeOccurrence

constitute(validated occurrence, base ProgramRevision or root)
  -> ProgramRevision
```

`ElaborationContext` owns only caller-selected scope, declarations, imports,
and Designation inputs. The candidate owns its exact semantics epoch and
unchecked Terms and judgments; SourceMap separately owns source and proposal
spans. Validation currently has no policy- or resource-relative input, so no
ceremonial `ValidationContext` exists. `ProgramAdmissionContext` is the exact
boundary for ProgramId, base revision, authority, policy, and constitutive
change-occurrence allocation. Revision existence is lifecycle-neutral.

There is no broad optional `ProgramContext` whose NamespaceId, AuthorityId,
PolicyId, SourceArtifactId, ProgramId, revision, or runtime identities may
silently substitute for one another.

## Runtime identity and effects

A RuntimeSession binds its explicit `RuntimeSessionId`, exact
`ProgramRevisionId`, `RuntimePolicyId`, `ClauseSemanticsId`,
`SessionStartOccurrenceId`, and initial StateRevision. `RuntimePolicyId`
commits to every immutable policy choice that can affect event admission,
scheduling, transition selection, effects, capabilities, successor computation,
cancellation, or other promised runtime behavior. Independently created
sessions have different RuntimeSessionIds even when program and policy match.

A StateSnapshot is the exact logical runtime payload at one boundary and is
conceptually separate from the transition that produced it. Clause does not add
a public StateSnapshotId until a real consumer needs history-independent state
content identity. A StateRevision binds its session, predecessor or root,
causal transition or session-start occurrence, exact StateSnapshot payload,
runtime policy, and semantics epoch.

Equal State payload reached through different sessions or occurrences does not
collapse. Session-start and transition identities are admitted inputs, never
derived from payload, source span, vector position, storage order, or replay
order. A runtime event changes StateRevision and leaves ProgramRevision
unchanged. A program upgrade requires explicit migration evidence and a new
RuntimeSession.

Effect intent, authorization, attempt, receipt, observation, and admitted
external claim are distinct occurrences. Effect evidence names the exact
ProgramRevision and post-commit StateRevision. A receipt records an outcome; it
does not make the intended external proposition true. By default the
StateRevision and authorized intent are admitted before the separately
identified external effect Run. Evidence admission happens after the act and
cannot roll it back. Any adapter claiming atomic state-plus-effect commit must
state and prove that stronger boundary explicitly.

## Theory and Model

ProgramSnapshot and StateRevision are object-language values. A Model is a
meta-level interpretation satisfying a declared Theory under a declared
semantic regime. Open-world or partial knowledge does not by itself prevent
modelhood; one object-language artifact may constrain many possible Models.

A future Theory is likely a parameterized view of a ProgramSnapshot, applicable
judgment basis, entailment regime, and derivation policy. Until Clause defines a
concrete Theory projection and satisfaction relation, `Theory` and `Model`
remain reserved and absent from the public kernel and routine source grammar.

## Compilation and physical realization

Clause owns its own semantic graph, canonical encoding, occurrence history,
persistence rules, and compilation semantics. No external store, database, or
older project is part of Clause's constitutional architecture. A persistence
backend may implement an explicitly checked Clause interface; it never supplies
Clause meaning, equality, identity, truth, or authority.

The logical pipeline is:

```text
readable Clause projection
  -> transient lossless CST
  -> candidate Term and occurrence graph
  -> admitted typed judgment graph
  -> lowering governed by typed Run relations
  -> physical strategy graph
  -> specialized materialization
```

These arrows state semantic relations, not mandatory runtime allocation. Pure
elaboration, query, normalization, and lowering may remain lightweight and
allocate no nominal Run, durable trace, or revision unless a consumer needs
their occurrence or evidence.

Derived representations may include indexes, packed role maps, e-graphs,
control/dataflow IRs, heaps, structs, registers, database plans, native code,
Wasm, JavaScript, and browser objects. They are checked refinements, not rival
semantic substances.

A backend may keep truly unobservable decisions private. Any physical decision
that can affect observable behavior or a declared ABI, layout, overflow,
floating-point, ordering, determinism, synchronization, cancellation,
durability, failure, resource, or latency contract must remain an explicit
strategy or evidence judgment traceable to the admitted graph.

Files are not modules. Names are not identities. Source order is not causality.
Heap addresses are not entity identity. Text diffs are not program diffs. Build
units are exact semantic and physical dependency closures, not files. API types,
database schemas, validators, generated clients, and documentation are
projections unless they intentionally carry distinct semantics.

## Acceptance laws

The adoption spike and any migration must prove at least these cases:

| Case | Required result |
| --- | --- |
| Same structural Triple constructed twice | Same Term; no assertion or execution implied |
| Equal Terms used by independent source or assertion occurrences | Equal content; distinct occurrences |
| Two identical-looking transfers happen | Distinct event/entity identities and Run occurrences |
| Same expression and value | Expression Term and evaluated value remain distinguishable |
| Structurally different Terms have equal behavior | Distinct structure; explicit denotational-equivalence judgment |
| A trace is replayed | Historical effect does not recur merely because its trace is read |
| Same admitted snapshot reached from different parents | Same snapshot identity; different revision identities |
| Same parent and snapshot, different genuine change occurrences | Same snapshot; different revisions |
| Same revision checked by two verifiers | One revision; two attestations |
| Source moves without semantic-source change | Same ProgramSnapshotId and semantic identities; SourceMap changes only |
| Local rename with explicit retention | Same ReferentId and ProgramSnapshotId; changed Designation |
| Exported Designation rename | Same ReferentId; changed ProgramSnapshot interface |
| Rename without retention | Delete plus create; no guessed continuity |
| Two equal claims are independently asserted | Same proposition content; distinct AssertionOccurrenceIds |
| A derived fact is later explicitly asserted | Consequences may match; the new assertion occurrence changes the snapshot |
| Non-constitutive attestation or later AdmissionJudgment is added | Snapshot and revision identities remain unchanged |
| Same checked payload travels with different source, trace, strategy, or certificate evidence | Same ProgramSnapshotId; evidence remains in separately typed package sections |
| A certificate checked for package A is presented with package B | Admission rejects the mismatched byte/epoch/decoded-value binding |
| Same checked payload under different semantics epochs | Different ProgramSnapshotIds |
| Two Programs select the same exact Referents and payload | Same ProgramSnapshotId; Program-specific revision identities |
| Equal spellings use independently allocated Referents | Different ReferentIds and ProgramSnapshotIds |
| ProgramRef moves | No snapshot or revision change; one new RefUpdate |
| Authorities disagree | Separate Judgments; policy-relative Disposition |
| Same State payload reached through different transitions | Different StateRevisionIds |
| Same ProgramRevision under different runtime policies | Different RuntimeSessionIds |
| Program upgrade | Explicit migration and new session |
| Production and canary select different revisions | Multiple DeploymentRecords; no single deployed pointer |
| Pure evaluation or rejection | No ProgramRevision or StateRevision is created |
| Nondeterministic or reactive Run | Cardinality, ordering/fairness, continuation, cancellation, and bounds follow the declared mode |
| Transition stages an external effect | Candidate contains intent only; receipt appears only after a separately identified effect Run |
| Effect evidence admission fails after an attempt | External act remains acknowledged and reconcilable; no rollback is claimed |

## Semantic-foundation falsifiers

The Program, identity, and source-boundary foundation must be reopened if
evidence establishes any of the following:

- an authored Clause artifact is necessarily a family of satisfying
  interpretations while program facts, provenance, history, and runtime state
  must live outside it;
- no real consumer needs to distinguish equal semantic snapshots reached
  through different histories or change occurrences;
- source placement must be constitutional even without an explicit
  semantic-source relation; or
- membership requires a closed primitive classifier universe rather than an
  ordinary relation whose group role accepts Referents.

These are semantic falsifiers independent of whether the process-first Term
kernel survives its implementation spike.

## Constitutional falsifier

The mechanism is rejected if the [adoption spike](adoption-spike.md) shows that
a dangerous general-purpose language feature requires:

- mandatory nominal identity on every Triple;
- a private host-language semantic case or per-construct validator;
- an opaque host callback or dispatch table carrying construct meaning behind a generic interface;
- arbitrary positional conventions or ad hoc untyped tags;
- an untracked representation that changes binding, typing, effects, identity,
  source meaning, or observable behavior;
- act/trace collapse or structural-content/occurrence collapse;
- graph-wide recomputation as the ordinary local-change path;
- generic graph execution that cannot specialize credibly; or
- source ceremony incompatible with Clause's readability mission.

Failure rejects this kernel hypothesis, not Clause's mission. Until the spike
passes, this document describes the accepted direction and disproof boundary;
it does not claim the current implementation already embodies the mechanism.

## Constitution

> **Running comes first. Terms are distinctions held still. A Clause is a
> contextual judgment over a Term. A Run carries that judgment to an outcome and
> candidate continuation. Admission makes a continuation authoritative. The
> admitted judgment graph is the program at rest; Run is the program in motion.
> One recursive Term algebra. One Clause judgment. One Run law.
> No second sovereign semantic authority.**
