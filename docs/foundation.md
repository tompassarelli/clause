# Clause Semantic Foundation

> **Status:** Accepted process-first constitutional hypothesis; not yet
> implemented. It is authorized for falsification by the
> [adoption spike](adoption-spike.md).
>
> **Authority:** Sole authority for Clause semantics. The
> [syntax](syntax.md) governs canonical source projection, the
> [architecture](architecture.md) governs implementation boundaries, and the
> [roadmap](roadmap.md) governs implementation status and order.

Clause is a process-first relational programming language. Its authoring
surface remains declarative and relation-first: people state applications,
relationships, laws, permissible transitions, effects, and physical
constraints. Clause process semantics determines which typed activations and
steps are admissible; checked physical execution specializes that meaning.

This document is Clause's semantic authority.

The product mission does not depend on this mechanism surviving. Clause still
aims for exceptional readability, Lisp-level semantic extensibility,
correctness by construction, predictable systems performance, and one language
from native software through Wasm, JavaScript, browsers, and data systems. The
three-slot mechanism remains a falsifiable way to reach that mission, not a
reason to narrow it.

## Decision

Running comes first. An application is the least articulated typed engagement
in which running can become explicit. An activation is one actual engagement
of an addressable application under an exact mode and initial context. A step
is one semantically meaningful carry-through between configurations of that
activation. A Run is the causal envelope with one unique root Activation and
any uniquely owned child Activations. Admission alone creates an authoritative
successor boundary.

Clause retains one structurally neutral recursive carrier:

```text
Atomᵤ      := opaque(kind, canonical-payload, equality-contract)
RawTripleᵤ := [Termᵤ, Termᵤ, Termᵤ]
Termᵤ      := Atomᵤ | RawTripleᵤ
```

The three positions of `RawTriple` have no inherent subject, operator, object,
argument, control, or truth meaning. Not every Triple is nominal. Checked
formation may interpret a Term as an application without adding another
universal data constructor:

```text
FormationJudgment := Γ ⊢ t : T @ interpretation

Γ ⊢ schema : exact RelationSchemaRef
Γ ⊢ op : exact OperatorRef
Γ ⊢ eligible = modes(op) restricted to schema and bindings
Γ ⊢ bindings exactly close schema.roles under every mode in eligible
Γ ⊢ requirements satisfied
──────────────────────────────────────────
Γ ⊢ form(t, schema, op, bindings, eligible)
    : ApplicationForm<ResultDomain>
```

An `ApplicationForm` is checked and closed: it records one exact
RelationSchema reference, one exact operator reference, complete named role
bindings, the exact set of eligible Mode references, and context requirements.
Every eligible Mode is declared against that same RelationSchema. It is
configured application possibility. It may have no eligible executable mode
and may be quoted, inspected, transformed, or rejected. It asserts, executes,
authorizes, and admits nothing. It may have an `ApplicationShapeId` where
structural comparison is safe and useful.
An **Application** is a nominal node instantiating one exact ApplicationForm;
every Application has `ApplicationId`. Raw Terms and quoted, open, or merely
structural forms are not Applications and need no nominal identity. `ClauseId`
is retired as a public identity domain rather than aliased to `ApplicationId`.

Process and governance use distinct relations:

```text
activate(ApplicationId, ModeId, InitialContext, ActivationCauseFrontier) =
  (ActivationId, RunMembership, InitialConfiguration)

⟨ActivationId, Configuration_before, observed StateRevision⟩
  -- StepId(causes = StepCauseFrontier) ; observations ;
     candidate delta ; continuation -->
⟨ActivationId, Configuration_after, same authoritative StateRevision⟩

Run(RunId, unique root ActivationId, child Activations, causal closure)

admit(BaseRevision, candidate delta, evidence,
      AuthorizationEvidence<AdmissionAuthorization>,
      JudgmentOccurrences, obligations)
  = (AdmissionOccurrenceId, SuccessorRevision | Rejection)
```

The process semantics is this typed activation, step, observation,
continuation, and admission relation. Actual Runs instantiate it. The Clause
Graph is the canonical inspectable carrier of process constitution and admitted
boundaries. It neither runs by being stored nor acquires independent authority.
Every physical execution must refine the process semantics and preserve its
declared identities, observations, effects, failures, resources, diagnostics,
and causal order.

These names are constitutional:

| Name | Meaning |
| --- | --- |
| Running | Actual semantic carry-through, whether finite, suspended, streaming, reactive, branching, or ongoing. |
| Distinction | A stable difference maintained by running. |
| Term | The holdable, recursively composable carrier of a distinction. |
| FormationJudgment | A contextual typing/formation claim; it grants no policy or execution authority. |
| ApplicationForm | A checked closed exact-schema/operator/named-role/eligible-Mode/context configuration over a Term. |
| Application | A nominal node instantiating one exact ApplicationForm, with `ApplicationId`. |
| Activation | One actual engagement of an Application under one selected mode, immutable initial pins, one typed cause frontier, and one exact Run membership. |
| ActivationConfiguration | Semantic process state before or after a Step of one stable `ActivationId`; it is not the stable identity. |
| Step | One externally meaningful causal carry-through between before/after ActivationConfigurations under an exact finite typed cause frontier. |
| Run | A causal process envelope with one unique root Activation and zero or more uniquely owned child Activations. |
| Continuation | The typed semantic remainder of an Activation, never merely a host stack frame. |
| Observation | An identified occurrence reporting a distinction from a Step or external boundary. |
| OccurrenceProvenance | A checked sum naming either the exact producing Activation/Step or the exact external boundary, evidence, and typed causal frontier through which an occurrence entered. |
| Result | A declared completion product; an ongoing Run need not manufacture one. |
| Value | A stabilized typed distinction or denotation reusable under declared equality, whether supplied, observed, or produced. |
| Evaluation | A species of running whose selected Mode seeks an observation, result, verdict, or normal form. It is not the process kernel itself. |
| RelationSchema | A typed named-role and constraint surface over admissible bindings. |
| RelationExtension | An extensional set or multiset of admitted bindings at one exact revision boundary. |
| OperatorRef | An exact reference to the operator/process definition configured by an ApplicationForm. |
| Mode | A contract for direction, known and produced roles, cardinality, purity/effects, failure, continuation, scheduling, identity, resources, and cost. |
| Function | An operator Mode established as pure, deterministic, and single-result for its declared direction. |
| Procedure | An operator Mode whose contract permits effects or authoritative transition proposals. |
| Proposition | Closed truth-apt relational or application content eligible for truth-directed interpretation or evaluation under a world; never an assertion by representation alone and not necessarily executable. |
| AssertionOccurrence | One identified act placing proposition content under an assertive stance, source, scope, and authority. |
| Judgment | Immutable checked assessment content naming its subject, stance, authority kind, policy, and scope; it is not an actual issuance until carried by a JudgmentOccurrence. |
| JudgmentOccurrence | One identified issuance of an exact Judgment by an exact authority under an exact policy and context. |
| Authorization | A Judgment whose stance permits one exact typed action and scope; contextual use is carried by an AuthorizationOccurrence. |
| Entity | A domain-level continuity projection, not the universal kernel noun. |
| Referent | Whatever a Designation picks out under an explicit identity protocol. |
| Identifier | A typed token designating one declared identity domain; its bytes do not define the continuity relation. |
| Type | A constraint on application formation, activation context, observations/results, continuation, failure, effects, deltas, resources, and representation. |
| Law | A universally available relational/process constraint that authorizes neither derivation nor activation by itself. |
| Rule | A declared transformation or derivation process under an explicit phase and authorization. |
| Query | An Activation seeking observations or bindings, never a syntactically special false assertion. |
| ProgramSnapshot | An exact immutable checked process constitution. |
| ProgramRevision | One admitted historical selection of an exact ProgramSnapshot in a Program lineage. |
| RuntimeSession | One execution lineage pinned to an exact ProgramRevision, policy, and semantics epoch. |
| StateRevision | One admitted runtime process boundary with exact session, predecessor, causal occurrence, payload, policy, and semantics. |
| Effect | A boundary-crossing process whose intent occurrence, Authorization Judgment and occurrence, attempt occurrence, optional receipt occurrence, Observations, JudgmentOccurrences, and Admission remain distinct. |
| Admission | The only operation that creates an authoritative Program, State, or other governed successor. |
| Trace | A retained projection of a Run; it is never the Run itself. |

Capitalized **Clause** names the language. Lower-case **clause**, where used as
a technical noun, means exactly an Application: a nominal node instantiating
one exact checked ApplicationForm. Application is the preferred unambiguous
name. Compound semantic forms have Term representations and may participate as
nodes in further applications; Clause does not claim that every compound thing
is ontologically one kind of object.

There is no first completed object called `Distinction` that must distinguish
itself. Running occurs in a base universe; higher universes may hold Terms that
describe Applications, Activations, Steps, Runs, observations, and evidence.
Reflection is well-founded rather than self-authorizing. Operationally, Term
formation, equality, process relations, and admission have explicit rules. No
implementation may infer validity from a story about which earlier Run created
a Term, and no correctness claim depends on observing a metaphysical first act.

## Canonical carrier contract

The **Clause canonical carrier** is the host-neutral transport contract for the
semantic objects defined by this foundation. It is not another constructor,
graph, context, revision, semantic substance, or authority. A canonical carrier
package is a typed envelope carrying existing Clause objects between
implementations; merely constructing, decoding, checking, or persisting one
asserts and admits nothing.

Each package schema keeps three scopes explicit and disjoint:

- candidate or checked semantic material governed by the Term, formation,
  Application, process, Judgment, and admission rules in this document;
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
snapshot preimage enumerated under
[Program identity and history](#program-identity-and-history) contributes to
`ProgramSnapshotId`. Source maps, strategies, runtime traces, certificates,
caches, and physical evidence remain outside that identity unless an explicit
authored FormationJudgment or governed Judgment places their semantic content
inside the snapshot.
Each check result binds the exact canonical package bytes, semantics epoch,
decoded sections, and claimed formations or Judgments. Any admission operation
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

The positions of a `RawTriple` are structurally neutral. A checked relational
Reading may interpret one as:

```text
[left Term, relating Term, right Term]
```

That representational three is not an operational formation/activation/step
taxonomy:

```text
formation:   Term + schema/operator/role/context requirements
activation:  ApplicationId + ModeId + exact initial pins + authorization
step:        configuration-before + typed cause frontier -> observations,
             delta, continuation, and configuration-after
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

Clause keeps these equivalence and continuity relations separate:

```text
Term structural equality
ApplicationShape equality
Application nominal continuity
activation occurrence identity
step causality
Run causal-envelope identity
value equality
denotational equivalence
source/assertion/observation occurrence identity
concept continuity
runtime entity identity
revision identity
artifact identity
```

There is no mandatory nominal identity for every `RawTriple`. Clause allocates
an explicit nominal or coordinate Term only when continuity or occurrence
matters, including for:

- a source use, assertion, retraction, Application, Activation, Step, Run, or
  observation occurrence;
- a binder or definition referenced independently of spelling;
- a concept lineage across structural revisions;
- a runtime entity or unique domain event;
- a Program, State, policy, semantics epoch, or revision;
- an effect intent, attempt, receipt, or observation; or
- a resource whose lifecycle is independent of its representation.

Identity Atoms compare as exact opaque canonical payloads within their declared
identity domain. Structural equality does not dereference the graph
neighborhood named by an identity Atom. Lineage-aware formation or admission
allocates fresh nominal and occurrence identities; a content-derived identity
is allowed only for a value whose identity contract is explicitly structural.
Neither allocation strategy may hash recursively through an identity reference
back into its own graph.

### Typed identity protocol

An opaque token is an identifier, not the continuity relation it denotes.
Semantic identity domains are disjoint even when a transport uses one fixed
byte width for all of them. No unchecked cast, shared string, host handle, or
wire `Id32` may substitute one domain for another.

Declarations, nominal Applications, and constitutive Judgments use typed
snapshot-local identifiers inside the canonical ProgramSnapshot preimage.
Once the snapshot identity is known, exact external references are formed
without placing any snapshot-scoped reference back into that preimage:

```text
RelationSchemaId = (ProgramSnapshotId, RelationSchemaLocalId)
RoleId           = (RelationSchemaId, RoleLocalId)
OperatorRef      = (ProgramSnapshotId, OperatorLocalId)
ModeId           = (OperatorRef, ModeLocalId)
ApplicationId    = (ProgramSnapshotId, ApplicationLocalId)
JudgmentRef      = (ProgramSnapshotId, JudgmentLocalId)
```

Each local identifier is unique in its declared domain and its declaration is
part of the canonical checked snapshot preimage. Local records refer only to
other local identifiers or external identities that were already resolved
before this snapshot; none contains `ProgramSnapshotId`, `RelationSchemaId`,
`OperatorRef`, `ModeId`, `ApplicationId`, `JudgmentRef`, or
`ApplicationShapeId` for the snapshot being constructed. Selecting the same
exact ProgramSnapshot through another ProgramRevision preserves the resulting
external references; changing the snapshot never silently preserves them. A
deliberately named cross-revision declaration continuity is a separate
`ReferentId` relation with explicit evidence, not equality of snapshot-local
identifiers. ProgramRevision lineage alone never supplies declaration identity
or continuity.

| Identity | Exact continuity criterion and consumer |
| --- | --- |
| `RelationSchemaId` | One exact checked RelationSchema declaration in one exact ProgramSnapshot, represented as above. A changed snapshot produces a new exact reference unless explicit Referent continuity is separately established. |
| `RoleId` | One exact role declaration inside one exact RelationSchemaId. Role spelling or position is never identity. |
| `OperatorRef` | One exact operator/process declaration in one exact ProgramSnapshot. It is a reference, not execution authority or cross-revision continuity. |
| `ModeId` | One exact Mode declaration under one exact OperatorRef. Each Mode declaration names exactly one RelationSchemaId; an Operator may expose Modes over several schemas. A Mode revision changes its exact reference; revision lineage never silently rebinds it. |
| `JudgmentRef` | One exact constitutive Judgment declaration in one exact ProgramSnapshot. Runtime issuance cites it or carries exact non-constitutive Judgment content in a JudgmentOccurrence; the reference is not the issuance. |
| `ApplicationShapeId` | Post-snapshot identity of canonical closed ApplicationForm content under one `ClauseSemanticsId`, including exact RelationSchemaId, OperatorRef, eligible ModeIds, named-role bindings, context requirements, and the exact resolved semantic-dependency/declaration closure, which may be proven empty. It never occurs in its own ProgramSnapshot preimage. Open formation candidates are not ApplicationForms and have no semantic shape ID. Used for comparison and reuse, never occurrence. |
| `ApplicationId` | One nominal Application instantiating one exact ApplicationForm under its exact semantics and snapshot-local declaration references. Every Application has one. Source-only movement may preserve it when the exact ProgramSnapshot and form are unchanged; a semantic or declaration revision creates a new ApplicationId, with any intended cross-revision continuity represented separately by ReferentId evidence. |
| `OccurrenceId` and typed refinements | One actual source, assertion, external-trigger, Judgment issuance, authorization, resumption, handoff, cancellation, production, admission, effect-intent, effect-attempt, receipt, or observation occurrence. Every actual occurrence has the explicit provenance sum defined below; equal content never merges independent occurrences, and one refinement never substitutes for another. |
| `ActivationId` | One actual engagement of one exact Application, mode, and initial context. Every activation is distinct, including repeated activation of equal content. |
| `StepId` | One externally meaningful semantic carry-through in an Activation. It names an exact finite typed StepCauseFrontier; serialization order supplies no causal edge. |
| `RunId` | One causal envelope with exactly one root Activation and unique membership for every child Activation it owns. It is not an alias for ActivationId or a log identifier. |
| `ContinuationId` | One persisted, suspended, or handed-off semantic remainder. Ephemeral in-process remainder may be physically erased when refinement proves it unobservable. |
| `ObservationId` | One occurrence reporting a distinction from an exact Step or external boundary. Equal observed values do not merge observations. |
| `ReferentId` | Deliberately retained nominal continuity of an addressable semantic referent across admitted changes. Similar content or spelling is insufficient. No separate `ConceptId` is introduced until a distinct continuity consumer exists. |
| `ProgramSnapshotId` | Hash of one exact canonical checked local-reference process-constitution preimage under one `ClauseSemanticsId`; snapshot-scoped external references resolve only after this hash. |
| `ProgramRevisionId` | Exact Program lineage edge: snapshot plus Program, predecessor/root, and change occurrence. |
| `StateRevisionId` | Exact admitted runtime boundary: session, predecessor/root, causal occurrence, payload, policy, and semantics. Materialization graph, contract, and plan identities are excluded. |
| `ArtifactId` | Exact physical materialization or byte content. It cannot stand in for any semantic or revision identity. |

Application identity survives serialization, process restart, machine
relocation, and physical rematerialization when the exact nominal identity and
form survive. Source-only movement may also preserve it when source mapping is
non-constitutive and the exact ProgramSnapshot is unchanged. A changed form or
snapshot-local declaration reference receives a new ApplicationId; explicit
ReferentId evidence may relate the old and new Applications but does not make
their IDs equal. `ApplicationShapeId` never proves nominal continuity.
`ActivationId`, `StepId`, and occurrence identities are fresh causal identities
and are never content hashes. A checked compiler may erase a pure Activation or
Step allocation from generated code only when a refinement proves every
declared consumer and observation unchanged; semantic distinctness remains.

An `Occurrence` is an actual identified event, never merely the content it
carries. Its kind is a typed refinement such as `AssertionOccurrenceId`,
`ExternalTriggerOccurrenceId`, `JudgmentOccurrenceId`,
`AuthorizationOccurrenceId`, `ResumptionOccurrenceId`,
`HandoffOccurrenceId`, `CancellationOccurrenceId`,
`AdmissionOccurrenceId`, `EffectIntentOccurrenceId`,
`EffectAttemptOccurrenceId`, `ReceiptOccurrenceId`, or `ObservationId`.
This list is extensible through checked declarations, not through unchecked
tags. Each declaration fixes the occurrence payload and the types allowed in
its causal frontier.

A `Judgment` is immutable assessed content. A `JudgmentOccurrence` is the
actual issuance of that content by an authority under a policy and context.
An `Authorization` is a Judgment subtype whose subject is one exact action and
scope; `ExecutionAuthorization`, `DerivationAuthorization`,
`EffectAuthorization`, and `AdmissionAuthorization` are distinct typed
subtypes. An `AuthorizationOccurrenceId<A>` is a typed JudgmentOccurrence
issuing an Authorization of subtype `A`; it cannot be used as another subtype.
Proposition content, an AssertionOccurrence, an Observation, a receipt, and an
Admission are not Judgments merely because a later Judgment may assess or
consume them. Equal Judgment content issued twice produces two occurrences.

Actual occurrence provenance is indexed by occurrence kind and is never a
guessed host relation:

```text
OccurrenceProvenance<K> :=
    ProducedBy<K>(RunId, ActivationId, StepId)
  | EnteredThrough<K>(exact BoundaryRef,
                      exact ExternalEvidenceRef,
                      ExternalCauseFrontier<K>)
```

`ProducedBy` requires the named Step to belong to the named Run and to emit
that exact occurrence kind and identity. `EnteredThrough` records an externally
sourced occurrence at one declared boundary. Its finite canonical cause
frontier contains only the typed causal references permitted for `K`; the
checker rejects a self-reference, future reference, cycle, wrong occurrence
kind, wrong Run membership, or missing required cause. An external root trigger
normally has an empty external cause frontier. An entered receipt names the
attempt it reports; an entered authorization names the exact request or effect
intent and carries the exact Authorization Judgment it issues; a cancellation
names its exact target; and a resumption or handoff names its exact
Continuation. Boundary entry does not
fabricate an ingestion Step or claim that a triggered process produced its own
trigger.

A configured binder, transfer, request, or task may require nominal
`ApplicationId` continuity. An actual trigger, domain event, Judgment issuance,
or effect attempt does not reuse that identity as its occurrence: it carries
the matching typed OccurrenceId plus exact provenance. Internally produced
occurrences name the Run, Activation, and Step that carried them through.
External triggers instead name their exact external-boundary provenance and
causally precede, rather than claim production by, the Activation they trigger.

Because this reset changes formation, process, and typed-identity rules, an
implementation of it selects a new `ClauseSemanticsId`. Existing v0 structural
indexes, package bytes, receipts, and fixture identities retain their published
meaning; no decoder may reinterpret them as process-v1 objects. Migration is an
explicit process with exact source/target epochs, evidence, and obligations.

`ReferentId` is Clause's general opaque identity kind for an addressable
semantic concept, not a universal interchangeable ID. A Referent is a Term
formed as having continuing nominal identity, not a second universal data
constructor. Names, paths, spans, host objects, intern handles, and movable
refs do not create or recover that identity by similarity.

Recursive Terms are finite trees or DAGs under structural sharing. Semantic
cycles use explicit identity anchors:

```text
operator-f-local = fresh OperatorLocalId

[operator-f-local binds x]
[operator-f-local body [x calls operator-f-local]]
```

The semantic graph may be cyclic through identity references without assigning
nominal identity to every `RawTriple` or hashing a structure through itself.
Canonical serialization writes finite Terms and opaque identity references; it
does not recursively inline the neighborhoods those identities name. The local
operator identifier is resolved to an exact `OperatorRef` once the containing
ProgramSnapshotId is known. Allocation is well-founded, reload traversal is
cycle-aware and terminating, and reload rejects unknown kinds, foreign scopes,
mismatched universes or semantics epochs, dangling required anchors, and
causal-lineage mismatch. Cross-epoch conversion is an explicit migration Run,
never permissive decoding.

## Formation, application, relation, and judgment

The general contextual mechanism is a `FormationJudgment`:

```text
Γ ⊢ t : T @ interpretation
```

It states that Term `t` has a checked type and contextual interpretation.
`FormationJudgment` is the conventional type-theoretic name for a formation
relation, not a governed `Judgment` in the authority taxonomy and not a
JudgmentOccurrence. This formation annotation is not the source `Reading`
defined below: generated,
quoted, runtime, and foreign Terms need no source projection. The
same structural Term may be formed as quoted syntax, a macro or query pattern,
a pure expression, proposition content, assertion content, a transition
request, an effect intent, a compiler plan, a trace, or the Term component of
an ApplicationForm. Formation proves neither truth, authority, executability,
nor current world membership. The prior equation of Clause with a
FormationJudgment is retired; capitalized Clause is the language, not this
relation.

`RawTriple`, Term, FormationJudgment, ApplicationForm, Application, and governed
Judgment are therefore distinct: structural compound, holdable value,
contextual formation, configured application possibility, nominal application
node, and authority-bearing assessment. Structure grants none of the later
roles. A proposition is not automatically asserted. An assertion is not
automatically true, authorized, current, or executable. Forming or evaluating
an effect description is not an effect occurrence.

A Proposition is a Term under a closed truth-apt FormationJudgment. It may be a
checked binding under a non-operator RelationSchema, such as membership or
equality, or an ApplicationForm eligible for truth-directed evaluation.
Executability is therefore neither required for proposition content nor
implied by it. A truth-directed executable mode may observe or assess
proposition content, but does not turn representation into AssertionOccurrence
or Judgment.

Typing constrains the entire process boundary: application formation, mode
selection, role satisfaction, possible observations and results, continuation
shape, failure, effects, capabilities, state delta, resources, invariants,
representations, and target support.

Relation, operator, mode, reading, extension, and authorization are separate
checked concepts:

- `RelationSchema` names exact roles, role types, cardinalities, constraints,
  and admissible binding shape. It may have no executable mode.
- a revision-indexed `RelationExtension` is an extensional set or multiset of
  admitted role bindings under one exact Program or State boundary. Its rows
  are not universal truth and their storage is not process occurrence.
- an activation-scoped result relation is a non-authoritative extensional view
  over bindings or observations from exact Activations and Steps. Pure running
  may produce this view without creating a revision. Only a later explicit
  Admission may place selected rows into a revision-indexed RelationExtension.
- `OperatorRef` selects an operator/process definition in one exact
  ProgramSnapshot. An operator may expose zero or more modes and may relate to
  one or more RelationSchemas.
- `Mode` names exactly one RelationSchema and declares known and produced roles
  from that schema, result cardinality, purity, effects, typed failure,
  nondeterminism, ordering, continuation, scheduling, identity, resource,
  temporal, cost, and admissible-strategy contracts. An operator may expose
  separate Modes for separate schemas; no Mode inherits a schema from call
  position or runtime selection.
- a source `Reading` maps syntax to exact Terms, role bindings, and declarations.
  It does not select runtime authority.
- `ExecutionAuthorization` is Authorization Judgment content permitting an
  exact Application and Mode to activate under a stated scope and policy. An
  Activation cites either a constitutive JudgmentRef whose scope covers the
  exact context or an AuthorizationOccurrence issuing it. This is separate
  from relation existence, mode existence, derivation authorization, admission
  authority, and effect capability.

A RelationSchema without an operator can still form checked role bindings,
revision-indexed relational rows, assertion content, and open patterns. None of
those is an ApplicationForm. ApplicationForm formation requires an exact
`OperatorRef`; RelationSchema existence or a checked relational binding cannot
supply one implicitly.

Application formation selects one exact RelationSchema and one exact
OperatorRef. Its exact eligible-Mode set is the set of that operator's Modes
which name the selected schema, permit the supplied known/produced-role
orientation, satisfy the form's static context requirements, and close the
bindings. The checked form stores that exact set; activation may select only a
member. A form with an empty set remains inspectable but cannot activate.
Ambiguous schema selection rejects formation rather than being deferred to the
runtime.

Formation requires exact role closure against the selected schema. Every
required role appears with its declared cardinality; no undeclared role
appears; and repeated roles, premise slots, and occurrences remain explicitly
ordered or multiplicity-aware where the schema requires them. Closed means
that every role is explicitly bound to a value, binder, or produced-role
placeholder permitted by every recorded eligible Mode; it does not mean every
role is initially ground. No consumer may infer a role or schema from Triple
position, source word order, graph adjacency, operator spelling, or a host
field name. A partially described pattern may be a query or rule pattern, but
it is not a closed ApplicationForm eligible for activation.

A pure function is an operator mode established as pure, deterministic, and
single-result for the declared direction. A procedure is an executable mode
whose contract permits effects or authoritative transition proposals. Clause
does not make every computation perform logic search or require every relation
to be reversible.

Reserve **capability** for authority over effects and resources. Callability is
not a capability. Relation, mode, law, and Application existence never by
themselves authorize execution, derivation, admission, or an external act.

## Finite derivation checking is relative

A finite certificate establishes derivability only relative to an exact,
separately supplied basis:

```text
B := roots + ground rules

root:
  q is an addressed root of B
  ----------------------------
  B ⊢ q derivable

apply:
  r is an addressed rule of B
  B ⊢ every declared premise of r derivable
  ----------------------------------------------
  B ⊢ conclusion(r) derivable
```

The primitive checker has only `root` and generic `apply`. A certificate is a
finite topologically ordered trace; each application references only earlier
checked conclusions. Self-references, forward references, back-edges, missing
supports, arity changes, and conclusion changes therefore reject without proof
search or fuel. Shared earlier conclusions remain valid DAG support.

Roots, rules, claims, and support references are bound by their exact structural
index and candidate representation at this bootstrap layer. That comparison is
address binding, not semantic Term equality. A ground rule's premise sequence
is explicit rule data, not source order or candidate-Context order. Schematic
matching, substitution, named-role normalization, and rule formation require a
later Clause-owned schema calculus; the ground checker may not invent them in
host code.

Certificate-node addresses cannot be duplicated within one application. This
does not yet establish occurrence-exact or linear support: two nodes may still
derive equal-looking claims from the same reusable root. A rule whose meaning
requires distinct occurrences must carry those occurrence identities and
linearity obligations explicitly in the later judgment/schema calculus.

Raw membership in a candidate Context is not a certificate reason. Nor may a
candidate rule, trace, proof-looking Term, or basis claim authorize itself.
For an arbitrary supplied `B`, successful checking means exactly `B ⊢ q
derivable`; it does not mean that `B`, `q`, or their Context is accepted, true,
valid, or authoritative. Basis acceptance enters only through a separate
governed Judgment and admission boundary tied to the exact semantics epoch and
canonical package. The v0 bootstrap selects one exact literal basis; every v0
successor basis is authorized only by a certificate checked against its exact
authoritative predecessor and the canonical claim committing to the successor's
exact INDEX and BASIS frames. A candidate basis never checks its own selection.

## Activation, configuration, step, continuation, and Run

Activation is actual engagement, not an edge saying that an application
timelessly evaluates to a value:

```text
RootTrigger :=
    ExternalTrigger(ExternalTriggerOccurrenceId)
  | SessionStart(SessionStartOccurrenceId)
  | AdmittedTrigger(AdmissionOccurrenceId)

ActivationOrigin :=
    RootedBy(RootTrigger)
  | ChildOf(RunId, parent ActivationId, parent StepId)
  | HandoffFrom(RunId, parent ActivationId, parent StepId,
                ContinuationId, HandoffOccurrenceId)

AuthorizationEvidence<A : Authorization> :=
    ConstitutiveAuthorization(JudgmentRef<A>)
  | IssuedAuthorization(AuthorizationOccurrenceId<A>)

ActivationPrerequisite :=
    Authorization(AuthorizationEvidence<A>)
  | AdmittedEffectIntent(EffectIntentOccurrenceId)
  | RequiredObservation(ObservationId)
  | RequiredAdmission(AdmissionOccurrenceId)

ActivationCauseFrontier :=
  exactly one ActivationOrigin
  + a finite canonical set of ActivationPrerequisite

RunMembership := RootOf(RunId) | ChildIn(RunId)

Γ; G; W; κ ⊢ activate(ApplicationId, ModeId,
                   ActivationCauseFrontier) ↦
  ⟨ActivationId, RunMembership, InitialConfiguration⟩

StepCause :=
    ActivationStart(ActivationId)
  | PriorStep(RunId, predecessor ActivationId, predecessor StepId)
  | ContinuationTakeup(ContinuationId,
                       ResumptionOccurrenceId | HandoffOccurrenceId)
  | CancellationRequest(CancellationOccurrenceId)

StepCauseFrontier := nonempty finite canonical set of StepCause

⟨ActivationId, Configuration_before, Wbase⟩
  -- StepId(causes = StepCauseFrontier) ; e ; δ ; k -->
⟨ActivationId, Configuration_after, Wbase⟩
```

`G` is the exact ProgramSnapshot constitution; `W` is the exact initially
observed StateRevision when the Application is world-sensitive; and `κ` is the
typed initial context. `κ` pins the exact `ClauseSemanticsId`,
`ProgramSnapshotId`, `ProgramRevisionId`, RuntimeSession when present, runtime
policy, exact required `AuthorizationEvidence<A>` values, capabilities, budget,
continuation/cancellation scope, and observable scheduler constraints.
Activation selects
one exact `ModeId` from the ApplicationForm's stored eligible-Mode set. The
selected Mode's activation contract fixes the allowed and required
`ActivationPrerequisite` kinds and Authorization subtypes; the checker rejects
a missing, extra, wrongly typed, or causally invalid prerequisite. A
constitutive authorization may
authorize an ordinary execution where its declared scope covers the exact
Application, Mode, session, and context. An effect Mode always requires an
`IssuedAuthorization<EffectAuthorization>` naming the exact admitted effect
intent and capability;
mere mode existence or a broad constitutive grant cannot authorize the external
attempt. Ambiguous, missing, unauthorized, malformed, ungrounded-known-role, or
over-budget activation rejects before acquiring partial authority.

Run membership is assigned at activation and never inferred from later graph
reachability. `RootedBy` allocates one fresh `RunId` and makes the Activation
that Run's unique root. `ChildOf` and `HandoffFrom` require the named parent
Step to belong to the named Run and assign the new Activation as a child of
that same Run. Every Activation has exactly one owning `RunId`; every Run has
exactly one root Activation; a child Activation does not silently root a second
Run. A deliberately detached process uses a new typed root trigger and a new
Run, while its trigger provenance may still name the earlier causal boundary.
These rules prevent a child from being attached to an arbitrary or multiple
Runs.

One stable `ActivationId` advances through any number of
`ActivationConfiguration`s. Configuration is semantic execution state, not a
new Application or Activation. Every externally meaningful carry-through has
a distinct `StepId` and a nonempty typed `StepCauseFrontier`. The first Step of
an Activation contains exactly one `ActivationStart`, whose causal predecessors
are the ActivationCauseFrontier; later Steps cannot use `ActivationStart`.
`PriorStep` may name one or several predecessor Steps in the same Run, allowing
join causality without inventing a total order. `ContinuationTakeup` must match
the exact continuation pins and causally includes the Step that emitted the
Continuation. `CancellationRequest` must target the exact Activation or its
owning Run; each Step that observes or carries through cancellation names that
occurrence explicitly. Independent Steps are unordered unless one of these
typed causes orders them; a total trace or log order is storage evidence only.
Internal KExpr reduction, CPU instructions, scheduler ticks, and materializer
visits are not semantic Steps unless the declared observation contract exposes
that boundary.

A Step may emit zero or more identified observations, values, evidence,
diagnostics, effect intents, resource use, a candidate delta, or a continuation.
These are separate outputs. Admission consumes only the candidate delta,
evidence, authority, and obligations; it neither consumes nor changes the
continuation. The authoritative world remains `Wbase` throughout candidate
computation.

Every world-sensitive Step names the exact StateRevision it observed or used as
its base. A long-lived Activation never silently sees a newer world or Program.
It may advance its world view only through an explicit observation, admitted
successor relation, or typed handoff that records the old and new pins. A
Program change never migrates a live Activation. Resumption or executor handoff
with identical Application, Mode, constitution, world, policy, authority, and
continuation pins preserves the same ActivationId and Run membership; the next
Step cites the exact Continuation and ResumptionOccurrence or
HandoffOccurrence. A handoff that changes any semantic pin, Application, or
Mode creates a fresh child Activation through `HandoffFrom`, with explicit
migration evidence and obligations. It may remain in the same Run because the
causal lineage is preserved, but it never makes the old Activation silently
change identity or constitution.

A `Continuation` is the typed semantic remainder of an Activation. When it
crosses suspension, handoff, persistence, or executor boundaries it receives a
`ContinuationId` and pins at least its owning Run, Activation, emitting Step, exact
Application and Mode, ProgramSnapshot and ProgramRevision, RuntimeSession,
observed/base StateRevision, runtime policy, semantics epoch, typed remainder,
remaining budget, and cancellation scope. Resumption rejects a mismatched pin.
An implementation may keep a purely local, unobservable continuation in
registers or a host stack under a checked refinement; those mechanics are not
its semantic identity.

A `RunId` identifies the causal closure of its unique root Activation and all
of its uniquely owned child Activations and Steps. Suspension and same-pin
resumption do not add an Activation. A semantic handoff may add one child
Activation; executor relocation alone does not. Cancellation is an occurrence
with exact target and provenance, not a mutable Run flag: every affected
carry-through cites it, so unrelated concurrent Steps remain unordered. A Run
may include external waits and explicitly nondeterministic branches. It may
terminate, fail, suspend, stream indefinitely, or remain receptive.
Intentionally ongoing running is a live configuration, not a fake result or a
third truth value.

The frozen v0 corpus retains exactly these compatibility envelopes:

```text
returned(value)
choices(finite-results)
yielded(value, continuation)
suspended(continuation)
failed(typed-reason)
exhausted(obligations)
```

They are `RunOutcomeV0`, not the entire process ontology. `yielded` and
`suspended` preserve nonterminal continuation. Cancellation and terminal
timeout may initially be typed `failed` reasons. Resource exhaustion remains
`exhausted`; a versioned refinement may attach a resumable continuation without
changing v0 bytes. A Run that is simply live emits no terminal envelope.

A completed total evaluation may use `⇓` as shorthand for a finite Step closure
ending in `returned`. A nondeterministic mode declares result cardinality and
whether results are a finite set, ordered stream, or selected under a recorded
strategy. Fairness, ordering, scheduling, cancellation, deadlines,
continuation persistence, and resource budgets are mode or checked physical-
strategy judgments whenever observable or promised. They are never ambient
host behavior.

The same process kernel specializes without making every machine operation a
durable graph occurrence:

| Form | Process result |
| --- | --- |
| Pure evaluation | Returns a value and evidence; produces no ProgramRevision or StateRevision. |
| Query | Emits observations or bindings from an exact relation input, pinned either to an Activation/Step-scoped result relation or to a revision-indexed RelationExtension. |
| Macro/elaboration/compiler process | Proposes a typed Program delta with origins, obligations, and evidence. |
| Runtime transition | Proposes a typed State delta against one exact base revision. |
| Ongoing service or actor | Remains live, yields, waits, or suspends under explicit continuation and liveness contracts. |
| Effect attempt | Performs one separately authorized external act and records causal evidence without fabricating success. |
| Rejection | Leaves every authoritative boundary unchanged and reports exact typed obligations. |

Clause does not claim a universal executability or termination decider.
Executable modes state honest obligations:

- **total** modes require a termination proof or reject;
- **productive** modes require finite observable progress;
- **bounded** modes enforce declared fuel, deadline, or resource limits;
- **partial** modes include failure or possible divergence in their contract;
  and
- **reactive** modes expect continued running and require stepwise progress,
  wait, cancellation, and handoff obligations.

Cycles are not inherently invalid. Recursive fixed points, services, streams,
actors, and state machines may be productive under their declared modes. An
ungrounded cycle is rejected where the selected mode promises a finite verdict
or another unmet property; it is not confused with an intentionally ongoing
Run.

## Admission is the authority boundary

```text
Γ ⊢ candidate delta well formed against exact BaseRevision
Γ ⊢ evidence, AuthorizationEvidence<AdmissionAuthorization>,
    JudgmentOccurrences, policy, and obligations sufficient
───────────────────────────────────────────────────────────
admit(BaseRevision, delta, evidence,
      AuthorizationEvidence<AdmissionAuthorization>,
      JudgmentOccurrences, obligations)
  = (AdmissionOccurrenceId, SuccessorRevision | Rejection)
```

Running alone does not mutate authoritative Clause state. A candidate delta and
a continuation are distinct: either, both, or neither may be emitted by one
Step. The target boundary decides whether proposed content is constitutive,
derived, observational, cached, speculative, or authoritative. Pure evaluation,
ongoing running, suspension, and rejection create no revision. A compiler
optimization may remain a replaceable derivation. A runtime transition becomes
current world state only after its identity, invariant, authority, policy,
capability, and effect-boundary obligations pass.

Clause has one authoritative change law: propose a typed delta against an exact
base, then admit it or reject it with exact obligations. Admission is typed to
its target boundary; Program, State, policy, and other governed successors do
not share identity or authority merely because they share the protocol. Source
elaboration, macros, refactors, migrations, compiler transformations, runtime
transitions, and agent edits may all be process applications. They do not
thereby share authority or lifecycle. Admission governs Clause's authoritative
boundaries; it is not rollback magic over an external system.

An actual invocation of this boundary is an `AdmissionOccurrence` with exact
typed provenance. It consumes the applicable
`AuthorizationEvidence<AdmissionAuthorization>` and JudgmentOccurrences but is
neither an Authorization nor a Judgment itself. An
AdmissionOccurrence may produce one exact successor or a typed Rejection;
repeating an already decided admission does not invent a different successor
identity merely because the attempt occurrence differs. The target revision's
constitutional identity fields remain those declared for that revision kind;
the AdmissionOccurrence remains queryable causal evidence unless that target's
identity contract explicitly includes it.

An ordinary state/effect protocol is a causal graph, not one mandatory total
chain:

1. a transition Activation and its Steps stage a candidate State delta and
   effect intents;
2. admission may accept the State successor and admitted intents atomically;
3. a separate `AuthorizationOccurrence<EffectAuthorization>` may issue an exact
   EffectAuthorization Judgment naming one admitted intent, capability, action,
   scope, and policy;
4. a separately identified effect Activation has that intent and authorization
   in its ActivationCauseFrontier and produces an EffectAttemptOccurrence;
5. the attempt may cause a ReceiptOccurrence, time out without one, fail before
   a receipt, or later be described by zero or more Observation occurrences;
6. governed JudgmentOccurrences issue exact Judgments over exact evidence; and
7. a later, separate AdmissionOccurrence may record a claim or State
   successor.

The required causal edges are intent to authorization to effect Activation to
attempt, with receipt optional. Their typed occurrence provenance and activation
cause frontier make that order checkable. Observations may describe an attempt,
timeout, receipt, or later external state. Receipt, Observation, Judgment,
JudgmentOccurrence, and Admission remain distinct.

If policy requires a stronger order, the mode must name the external
transactional adapter and its atomicity, retry, idempotency, timeout, and failure
contract. Once an external attempt occurs, rejection or failure to admit its
evidence cannot undo the act or claim that nothing happened; the occurrence and
unadmitted evidence remain visible for reconciliation. A transition proposal
may never fabricate an attempt, receipt, observation, or post-act success.
Trace replay performs zero attempts.

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

## The process-constitution graph

The first persistent compiler-owned semantic representation is an Abstract
Semantic Graph consisting of:

- recursive structural Terms;
- explicit occurrences and nominal identities where required;
- FormationJudgments, ApplicationForms, Applications, schemas, operators,
  modes, readings, authorizations, types, and capabilities;
- scopes, binders, uses, macro origins, and phase relations;
- derivations, supports, obligations, proofs, and explanations;
- Program and State revisions;
- physical strategies and artifact mappings; and
- recoverable Activation, Step, Run, continuation, observation, and trace
  relations wherever their consumers require them.

A random graph, a parse graph, a rejected candidate, or a speculative optimizer
graph is not a Program. An accepted ProgramRevision selects one exact checked
process constitution; a StateRevision selects one admitted process boundary.
Neither graph presence nor row presence makes an Application activate, an
observation true, or a candidate authoritative.

Clause process semantics owns meaning. The graph is its canonical inspectable
carrier and explanation surface: it must hold every constitutive relationship
and every admitted boundary that can affect declared meaning. Actual running is
not reducible to whichever graph or trace projection was retained. Conversely,
an opaque runtime cannot bypass the graph: every externally meaningful
Activation has recoverable identity, revision pins, mode, authority,
capabilities, evidence, and causal relation. A lossless CST remains necessary
for tokens, indentation, comments, whitespace, errors, and incomplete edits,
but it is a projection-recovery structure rather than a sovereign AST.

No giant host enum or collection of construct-specific validators may privately
decide what `if`, lambda, match, transition, or a user extension means. Schemas,
readings, FormationJudgments, modes, process rules, completion rules, and
transformations are Clause data interpreted by a small generic kernel. Host
code may bootstrap that kernel and optimize checked meaning; it may not retain
a second secret language.

## Relations and higher arity

One checked relational Reading may read a Triple as:

```text
[left, relation, right]
```

That is an interpretation established by a RelationSchema and Reading, not
inherent meaning of the middle slot.

Higher-arity ApplicationForms include all role assignments in their canonical
recursive Term and checked formation. A partially described structural root
may not gather unrelated edges merely because two forms share content or graph
neighbors.

When a configured application has independent continuity—such as a particular
transfer, binder, request, task, or payment configuration—it has an
`ApplicationId` used as its explicit identity anchor:

```text
[transfer-42 actor Alice]
[transfer-42 amount $10]
[transfer-42 from Checking]
[transfer-42 to Savings]
```

Two equal transfer forms may therefore instantiate different Applications.
Their later Activations remain distinct again. An actual transfer event or
effect attempt has its own typed `OccurrenceId`; it is not the configured
Application. An internally produced occurrence names its producing
Activation/Step, while an external transfer trigger names exact boundary
provenance that precedes the triggered Activation. A schema requires stable
named roles, exact role types and cardinality, complete coverage, source-order
independence, and atomic admission. An incomplete neighborhood is a provisional
pattern or candidate, never half an admitted Application.

A named-role n-ary representation may be a useful checker view, index, API, or
packed runtime materialization. It is not an irreducible semantic substance.
RelationExtension rows are similarly revision-indexed extensional views; they
do not replace the Applications or process occurrences from which a declared
extension may be recovered.

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

> Can Clause add a new language concept through Clause-authored formations,
> schemas, operators, modes, and process definitions, or must the host learn a
> new semantic secret?

After the generic host kernel is frozen, a new construct involving both binding
and effects must be implementable through Clause-authored schemas, readings,
modes, and transformations while inheriting parsing, printing, hygiene, typing,
capability checking, navigation, refactoring, invalidation, lowering,
diagnostics, explanation, and trace semantics. Requiring a new host semantic
enum, validator branch, formatter case, refactor rule, or analysis plugin
falsifies the universal-substrate claim.

The extension's definitions must remain ordinary inspectable Clause Terms,
formations, modes, and process definitions executable by the frozen generic
machinery. A “generic” opaque host
callback, per-construct dispatch table, foreign evaluator, or serialized tag
whose meaning exists only in host code is still a second semantic authority.
Irreducible FFI primitives are allowed only behind explicit typed effect,
capability, identity, and trace contracts; they cannot define the meaning of a
Clause language construct.

## Source projection

Human-readable source is a canonical bidirectional projection, not the
program's identity. Parsing may use a transient lossless CST. Every source line
elaborates to a Term, candidate formations, and a designated focus; every
indented child receives the parent's focus as its omitted left operand. The
parent Reading chooses focus. The child never guesses a relation from
indentation.

Reading lookup is deterministic from the explicit head/operator, declared
grammar, and already selected ElaborationContext before child domain semantics
are inspected. Missing or competing readings are explicit errors. Schema and
type checking may reject the resulting candidate, but may not regroup the CST
or reinterpret siblings. Incremental parsing and recovery therefore depend on
syntactic boundaries and declared readings, never on successful whole-program
inference.

Conceptually:

```text
elaborate(line) -> (term, candidate formations, focus)
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
  authorized Activation may propose a candidate and only one exact Admission
  causes a transactional successor; and
- an effect request, Authorization Judgment, AuthorizationOccurrence, intent,
  attempt, receipt, Observation, JudgmentOccurrence, and admitted external
  claim remain distinct typed content, occurrences, Judgments, or boundaries.

Truth, derivability, acceptance, observation, authorization, intention,
requirement, execution, and external success are therefore not aliases.

Clause is open-world by default. Failure to find, derive, observe, or admit a
proposition does not establish its negation. Explicit negative content, a
rejecting judgment, an incompatibility constraint, and absence of evidence
remain distinct. Closed-world reasoning requires a finite scope and an explicit
governing mode or law.

An AssertionOccurrence is an independently identified act committing to
proposition content with provenance and scope. Equal proposition Terms may have
many AssertionOccurrences. It is not a truth Judgment. A Judgment is immutable
checked assessment content; its actual issuance is a separately identified
JudgmentOccurrence with typed provenance. An Authorization is one Judgment
subtype, never a synonym for every Judgment or for authority itself. A current
Disposition is a derived policy-relative view over Judgments and their issuance
occurrences, never a mutable status field inside the proposition or assertion.

Universal laws remain inert until a separate derivation authorization selects
an operational mode. Positive derivation preserves every independent support;
retraction removes a consequence only when its final support disappears.
Caches, schedules, proof selections, and derived closure are replaceable unless
explicitly admitted as program content.

## Program identity and history

- A **Program** is one durable evolving lineage, identified by `ProgramId`.
- A **ProgramSnapshot** is one exact immutable checked process-constitution
  graph under an exact `ClauseSemanticsId`.
- A **ProgramChangeOccurrence** is the causal occurrence proposing one program
  history edge.
- A **ProgramRevision** is an immutable causal node selecting one snapshot in a
  Program.
- A **RuntimeSession** is one execution lineage pinned to an exact
  ProgramRevision, runtime policy, and semantics epoch.
- A **StateRevision** is one immutable admitted process boundary inside exactly
  one RuntimeSession.
- A **Model** is reserved for a meta-level interpretation satisfying a Theory,
  not an authored source block or executable artifact.

Routine source contributes Terms, FormationJudgments, schemas, operator and mode
definitions, ApplicationForms, nominal Applications where required, and
governed semantic content to a candidate ProgramSnapshot. Files, namespaces,
source blocks, host objects, storage rows, and heap layouts do not grant program
identity or authority.

A ProgramSnapshot is constructed from a canonical checked **snapshot
preimage**, never from records which already contain the ProgramSnapshotId being
computed. The preimage includes, where present:

- Referent and typed identity Terms, revision-independent equality contracts,
  types, and checked formations;
- RelationSchema and role declaration records keyed by
  `RelationSchemaLocalId` and `RoleLocalId`;
- operator and Mode declaration records keyed by `OperatorLocalId` and
  `ModeLocalId`, with every Mode naming exactly one local RelationSchema and
  carrying its exact contract;
- source Readings and constitutive Authorization Judgments, using only local
  references for declarations in this snapshot;
- ApplicationForm records which select one local RelationSchema, one local
  operator, an exact set of eligible local Modes, exact role bindings, context
  requirements, and dependency closure; nominal Application records keyed by
  `ApplicationLocalId`; and independently identified AssertionOccurrences or
  relational content with constitutional provenance;
- immutable governed Judgment content and constitutive JudgmentOccurrences
  authored as program content, keyed locally where snapshot-scoped;
- definitions, laws, derivation authorizations, invariants, goals, continuation
  and process contracts;
- transition, event, capability, effect, admission, and semantic-policy
  contracts; and
- exported Designations and explicit semantic source or authority relations.

Every local reference is typechecked and resolved within that finite preimage.
Canonical local keys are semantic allocation/continuity keys, never source
positions, traversal order, memory addresses, or spellings; canonicalization
orders records by their declared encoding. A reference to an already existing
external snapshot remains an ordinary exact external identity. A reference to
the snapshot under construction must be local. Consequently the preimage
contains none of its own `ProgramSnapshotId`, `RelationSchemaId`, `RoleId`,
`OperatorRef`, `ModeId`, `ApplicationId`, `JudgmentRef`, or
`ApplicationShapeId` values.

It excludes incidental source layout, SourceMap data, formatting, comments,
trivia, local Designation spellings, caches, schedules, replaceable derived
closure, ProgramRefs, lifecycle state, deployment attempts, RuntimeSessions,
StateRevisions, runtime Activations, Steps, Runs, observations, traces, and host,
storage, rendering, or target layouts. An excluded item enters snapshot identity
only through an explicit formation or governed Judgment that makes its semantic
content constitutive.

ProgramSnapshot identity is intensional over that canonical checked preimage,
not over all logically or behaviorally equivalent programs:

```text
ProgramSnapshotId = H(
  "clause/program-snapshot/v1",
  ClauseSemanticsId,
  canonical_snapshot_preimage
)
```

`canonical_snapshot_preimage` is the canonical encoding of the exact checked
process-constitution graph just enumerated in local-reference form. Hashing it
creates the ProgramSnapshotId exactly once. External snapshot-scoped references
are then resolved as the tuples in the typed identity protocol; resolution does
not alter or rehash the preimage:

```text
ApplicationShapeId = H(
  "clause/application-shape/v1",
  ClauseSemanticsId,
  ProgramSnapshotId,
  canonical_resolved_application_form_without_shape_id
)
```

The resolved form contains the exact RelationSchemaId, OperatorRef, eligible
ModeIds, role bindings, context requirements, and dependency closure. Neither
that ApplicationShapeId nor any external reference derived from the snapshot
is inserted back into the same snapshot preimage. This two-stage construction
removes the self-hash while keeping every external reference exact.

`ClauseSemanticsId` commits to
canonical Term encoding and equality, normalization, typed identity resolution,
formation, RelationSchema and role interpretation, Application formation,
activation and Step semantics, modes, continuation, observation, law and
derivation semantics, transition and admission semantics, and every identity-
relevant provenance rule. It is not a compiler build number. Independent
conforming implementations of one semantics epoch must produce the same bytes
and IDs.

`ProgramId` is not included merely as snapshot ownership. Two Program lineages
that preserve the same ReferentIds, semantics epoch, and canonical snapshot
preimage may share a ProgramSnapshotId; their ProgramRevisionIds remain distinct.
Independently allocated Referents with equal spellings produce different
snapshots. Any change to the Term encoding or identity rules requires a new
semantics epoch and explicit conversion evidence; an implementation may not
reinterpret bytes or IDs from another epoch. An independently asserted
consequence changes the snapshot even if it was already derivable, while moving
source without changing an explicit semantic-source relation changes only
SourceMap evidence.

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
Admission Judgments and their JudgmentOccurrences, lifecycle decisions,
deployments, and movable ProgramRefs remain separate records. Repeatable,
accumulable, contestable, or policy-relative evidence never enters either
identity preimage. A second verifier therefore does not change revision
identity.

- A `ProgramRef` is a movable name pointing to a ProgramRevision; every movement
  has an immutable `RefUpdate`.
- A `LifecycleDecision` is an immutable JudgmentOccurrence issuing an accepted,
  released, promoted, or withdrawn Judgment naming authority, policy, target,
  time, revision, and evidence.
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
occurrences, formations, and diagnostics to SourceArtifactIds, spans, and trivia evidence.
Neither is a Program or authority merely by existing. The typed boundary is:

```text
read(SourceUnit)
  -> LosslessCST + SourceMap

elaborate(LosslessCST, ElaborationContext)
  -> candidate Terms, occurrences, formations, application-form candidates,
     and declarations

check(candidate Terms, occurrences, formations, forms, and declarations)
  -> checked Terms, FormationJudgments, ApplicationForms, declarations,
     or exact obligations

propose_change(checked candidate, base ProgramRevision or root,
               ProgramAdmissionContext)
  -> ProgramChangeOccurrence

admit(checked ProgramChangeOccurrence, base ProgramRevision or root,
      exact AuthorizationEvidence<AdmissionAuthorization>,
      exact JudgmentOccurrences)
  -> (AdmissionOccurrence, ProgramRevision | Rejection)
```

`ElaborationContext` owns only caller-selected scope, declarations, imports,
and Designation inputs. The candidate owns its exact semantics epoch and
unchecked Terms, formations, forms, and declarations; SourceMap separately owns
source and proposal spans. Formation checking consumes no policy- or resource-
relative authority; those inputs belong to activation authorization or
admission. `ProgramAdmissionContext` is the exact boundary for ProgramId, base
revision, authority, policy, constitutive change-occurrence allocation, and
admission-occurrence allocation.
Revision existence is lifecycle-neutral.

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

A StateSnapshot is the exact logical runtime payload at one admitted process
boundary and is conceptually separate from the transition that produced it.
Clause does not add a public StateSnapshotId until a real consumer needs
history-independent state content identity. A StateRevision binds its session,
predecessor or root, causal transition occurrence with its producing
Activation/Step identity or session-start occurrence, exact StateSnapshot
payload, runtime policy, and semantics epoch.

Equal State payload reached through different sessions or occurrences does not
collapse. Session-start and transition-producing Activation/Step identities are
admitted inputs, never derived from payload, source span, vector position,
storage order, or replay order. A runtime transition admission changes
StateRevision and leaves ProgramRevision unchanged. A Program upgrade requires
explicit migration evidence and a new RuntimeSession.

EffectIntentOccurrence, EffectAuthorization Judgment,
AuthorizationOccurrence, effect Activation, EffectAttemptOccurrence,
ReceiptOccurrence, Observation, Judgment, JudgmentOccurrence,
AdmissionOccurrence, and admitted external claim are distinct typed objects or
boundaries. Effect evidence names the exact ProgramRevision, RuntimeSession,
observed/base StateRevision, producing Run/Activation/Step, typed occurrence
provenance, and causal frontier. A receipt records an outcome; it does not make
the intended external proposition true. By default an intent is admitted before
a separately authorized effect Activation attempts the act. That Activation's
cause frontier names both the exact admitted intent and the exact
AuthorizationOccurrence. The attempt may have no receipt, and zero or more
later observations may describe it. Evidence admission happens after the act
and cannot roll it back. Any adapter claiming atomic State-plus-effect commit
must state and prove that stronger boundary explicitly.

## Theory and Model

ProgramSnapshot and StateRevision are object-language values. A Model is a
meta-level interpretation satisfying a declared Theory under a declared
semantic regime. Open-world or partial knowledge does not by itself prevent
modelhood; one object-language artifact may constrain many possible Models.

A future Theory is likely a parameterized view of a ProgramSnapshot, applicable
judgment basis, entailment regime, and derivation policy. Until Clause defines a
concrete Theory projection and satisfaction relation, `Theory` and `Model`
remain reserved and absent from the public kernel and routine source grammar.

## Finite bedrock

“Turtles all the way down” means that every semantic layer above the trusted
base uses the same account of formation, Application, Activation, Step,
evidence, continuation, and Admission. It does not deny the physical executor.
The reviewed bedrock is limited to:

- Atom, Term, and typed identifier representation;
- nominal Application and occurrence allocation;
- typed role and ApplicationForm formation;
- context, phase, universe, Mode, policy, authorization, and capability
  formation;
- Activation, Step, causal-frontier, and continuation protocol;
- candidate-delta and Admission validation;
- immutable revision construction and canonical serialization;
- effect-boundary and receipt-verification hooks; and
- the fixed physical execution primitives required to bootstrap the system.

No Term, schema, operator, mode, law, Application, process, or package may
establish its own authority through an unstratified self-supporting cycle. A
trusted kernel that grows construct-specific Clause meanings is a second
sovereign language and fails the constitution.

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
  -> candidate Terms, formations, application-form candidates, and occurrences
  -> checked process-constitution graph
  -> admitted ProgramRevision
  -> Application activation and typed Step relations
  -> physical strategy graph
  -> specialized materialization
```

These arrows state semantic relations, not mandatory runtime allocation. Pure
elaboration, query, normalization, and lowering may remain lightweight and
allocate no durable Activation/Step/Run record, trace, or revision when a
checked refinement proves every semantic consumer and declared observation
unchanged. Their semantic identities and distinctions may not be collapsed by
the optimization.

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

Efficient implementations may use mutable heaps, registers, arrays, stacks,
threads, JITs, actors, database engines, and specialized instructions. Their
accidental structure never defines Clause meaning:

| Physical accident | Clause rule |
| --- | --- |
| memory address or host handle | never semantic identity |
| current mutable cell contents | never an authoritative StateRevision |
| source order, program counter, or serialized log | never causality without an explicit edge |
| host closure or function pointer | never Operator, Application, Mode, or Activation identity |
| arbitrary host mutation | never an Effect without the typed causal boundary |
| bytes or placement | never Value, Referent, Application, or revision identity by themselves |
| observed thread interleaving | never semantic order beyond declared typed cause frontiers |
| thrown string | never an untyped substitute for rejection, cancellation, timeout, exhaustion, or absent evidence |
| missing relation row | never false without an explicit closed-world contract |

## Acceptance laws

The adoption spike and any migration must prove at least these cases:

| Case | Required result |
| --- | --- |
| Same structural Triple constructed twice | Same Term; no Application, assertion, or execution implied |
| Equal Terms used by independent source or assertion occurrences | Equal content; distinct occurrences |
| Closed form compared structurally | `ApplicationShapeId` binds `ClauseSemanticsId`, exact RelationSchemaId, OperatorRef, eligible ModeIds, roles, context requirements, and the exact resolved semantic-dependency/declaration closure, including proof that it is empty when applicable; an open form has no shape ID |
| Equal-shaped ApplicationForms independently instantiated without continuity evidence | Distinct ApplicationIds |
| Snapshot-scoped declarations and forms are hashed | The canonical local-reference preimage contains no identity derived from its own ProgramSnapshotId; external references and ApplicationShapeIds resolve only after the one snapshot hash |
| One exact Application independently root-activated twice | One ApplicationId; two distinct ExternalTriggerOccurrenceIds, ActivationIds, and Run roots |
| A parent Step starts a child Activation | The child has a fresh ActivationId, inherits exactly the parent's RunId, and cannot also root or join another Run |
| One Activation progresses, suspends, and resumes | One ActivationId and Run membership; several StepIds and configurations; the takeup Step names the exact Continuation and ResumptionOccurrence |
| An executor handoff preserves all semantic pins | Same ActivationId and Run membership; the takeup Step names the Continuation and HandoffOccurrence |
| A semantic handoff changes Application, Mode, or a semantic pin | Fresh child ActivationId in the same Run through an exact HandoffFrom cause; the original Activation never changes identity or pins |
| A cancellation races an independent Step | Only Steps whose typed cause frontier names the CancellationOccurrence are ordered after it; unrelated Steps remain unordered |
| Independent Steps are serialized in a log | No causal ordering unless an explicit typed cause frontier relates them |
| Two equal-shaped nominal transfer configurations are independently established | Distinct ApplicationIds; every actual transfer event also has a distinct OccurrenceId plus internal producing Activation/Step/Run identity or exact external-boundary provenance |
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
| A derived proposition is later explicitly asserted | Consequences may match; the new assertion occurrence changes the snapshot |
| Non-constitutive attestation or later Admission JudgmentOccurrence is added | Snapshot and revision identities remain unchanged |
| Same checked payload travels with different source, trace, strategy, or certificate evidence | Same ProgramSnapshotId; evidence remains in separately typed package sections |
| A certificate checked for package A is presented with package B | Admission rejects the mismatched byte/epoch/decoded-value binding |
| Same canonical snapshot preimage under different semantics epochs | Different ProgramSnapshotIds |
| Two Programs select the same exact Referents and snapshot preimage | Same ProgramSnapshotId; Program-specific revision identities |
| Equal spellings use independently allocated Referents | Different ReferentIds and ProgramSnapshotIds |
| ProgramRef moves | No snapshot or revision change; one new RefUpdate |
| Equal Judgment content is issued twice | One content-equivalence result; two distinct JudgmentOccurrences with exact provenance |
| Authorities disagree | Separate Judgments and JudgmentOccurrences; policy-relative Disposition |
| Same State payload reached through different transitions | Different StateRevisionIds |
| Same ProgramRevision under different runtime policies | Different RuntimeSessionIds |
| Program upgrade | Explicit migration and new session |
| Production and canary select different revisions | Multiple DeploymentRecords; no single deployed pointer |
| Pure evaluation or rejection | No ProgramRevision or StateRevision is created |
| Intentionally ongoing service | Remains live or suspended without manufacturing a terminal result |
| Nondeterministic or reactive Run | Cardinality, ordering/fairness, continuation, cancellation, and bounds follow the declared mode |
| Program changes during a live Activation | The Activation remains pinned; only explicit migration or handoff changes its constitution |
| World changes during a live Activation | Each world-sensitive Step names its exact observed/base StateRevision; no silent rebinding |
| Candidate delta and continuation are both emitted | They remain independent; admission consumes only the delta-side inputs and leaves the continuation as a separate process output |
| RelationSchema exists without an OperatorRef | Checked bindings, relational rows, assertions, and patterns may form; no ApplicationForm forms implicitly |
| RelationSchema exists without a Mode | It remains queryable/inspectable but cannot activate |
| User-defined algebraic data and exhaustive match | Clause-authored declarations and process definitions accept the exhaustive case and reject missing/unreachable cases exactly; no kernel feature case is added |
| Forming or evaluating proposition content | Creates no assertion occurrence or truth Judgment |
| Transition stages an external effect | Candidate contains intent only; an effect Activation may attempt it only when its cause frontier names the exact admitted intent and EffectAuthorization occurrence |
| Attempt times out without receipt | Attempt and timeout observations remain honest; no receipt or success is fabricated |
| Effect evidence admission fails after an attempt | External act remains acknowledged and reconcilable; no rollback is claimed |
| Materializer applies an admitted State delta | Separate physical envelope pins graph, contract, and plan; materializer allocates no State history and plan identity does not enter StateRevisionId |
| Scan and indexed materializations | Same declared observations, occurrence-exact supports, failures, and candidate deltas from the unchanged process law |

## Semantic-foundation falsifiers

The Program, identity, and source-boundary foundation must be reopened if
evidence establishes any of the following:

- an authored Clause artifact is necessarily a family of satisfying
  interpretations while admitted program content, provenance, history, and runtime state
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
- collapse of ApplicationForm, Application, Activation, Step, and Run despite
  their distinct consumers across pure, effectful, and ongoing programs;
- treating the middle Triple slot as an inherent operator or relying on
  arbitrary positional conventions for n-ary roles, continuation, binding,
  effects, or authority;
- a private host-language semantic case or per-construct validator;
- an opaque host callback or dispatch table carrying construct meaning behind a generic interface;
- ad hoc untyped tags;
- an untracked representation that changes binding, typing, effects, identity,
  source meaning, or observable behavior;
- act/trace collapse or structural-content/occurrence collapse;
- inability to distinguish an intentionally ongoing Run from failed,
  ungrounded, or exhausted evaluation;
- loss of exact relational querying over admissible bindings, observations,
  dependencies, or occurrence-exact supports;
- a requirement that every ephemeral machine reduction become durable graph
  content;
- graph-wide recomputation as the ordinary local-change path;
- generic graph execution that cannot specialize credibly;
- a trusted kernel that grows a second sovereign feature language; or
- source ceremony incompatible with Clause's readability mission.

Failure rejects this kernel hypothesis, not Clause's mission. Until the spike
passes, this document describes the accepted direction and disproof boundary;
it does not claim that an implementation already embodies the mechanism.

## Constitution

> **Running comes first. A checked ApplicationForm is configured application
> possibility; an Application is its nominal node; an Activation is one actual
> engagement under an exact mode and pinned context; a Step is causal
> carry-through between configurations; and a Run is that Activation's causal
> closure. Relations constrain and expose admissible Applications and Runs.
> Observations report what running distinguished. Admission alone creates an
> authoritative successor. Terms and the Clause Graph are the neutral,
> recursive, inspectable carrier of this process semantics. Physical execution
> refines it and may specialize aggressively. One process authority. No hidden
> host language.**
