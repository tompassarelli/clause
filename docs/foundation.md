# Clause Semantic Foundation

> **Status:** Process-first semantic contract. Its falsification boundary is
> the [adoption spike](adoption-spike.md); implementation status belongs only
> to the [roadmap](roadmap.md).
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

The product mission does not depend on this mechanism surviving. Clause aims
to replace general-purpose languages for agent-authored software, not merely to
serve governed databases or causal ledgers. That requires exceptional
readability, Lisp-level semantic extensibility, correctness by construction,
predictable systems performance, ordinary local state without governance tax,
static reuse, explicit resource control, and one language from native software
through Wasm, JavaScript, browsers, and data systems. None of that support is
implemented yet. The three-slot mechanism remains a falsifiable way to reach
the mission, not a reason to narrow it.

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
Tripleᵤ := [Termᵤ, Termᵤ, Termᵤ]
Termᵤ   := Atomᵤ | Tripleᵤ
```

The three positions of `Triple` have no inherent subject, operator, object,
argument, control, or truth meaning. Not every Triple is nominal. Checked
formation may interpret a Term as an application without adding another
universal data constructor:

```text
ClauseJudgment := Δ ⊩ t @ interpretation : contextual stance

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
activate(ActivationStartRecord) =
  (ActivationId, RunMembership, InitialConfiguration)

⟨RunId, ActivationId, ConfigurationCustody_before, optional Wbase⟩
  -- StepRecord(s = fresh StepId,
                boundary = StepBoundaryRef,
                owner = (RunId, ActivationId),
                causes = StepCauseFrontier,
                transition = StepConfigurationTransition(s),
                observations, outcome, candidate delta, continuation) -->
⟨RunId, ActivationId, ConfigurationCustody_after, same optional Wbase⟩

Run(RunId, unique root ActivationId, child Activations,
    order = transitive closure(typed frontier edges
                               ∪ typed configuration-succession edges))

AdmissionRequestKey = H(canonical AdmissionRequest)

admit(AdmissionRequest) =
  Cite(existing AdmissionOccurrence)
  | Decide(AdmissionOccurrence(requestKey,
                               SuccessorRevision | Rejection))
```

The transition semantics is this typed Activation, Step, observation,
continuation, and Run relation. Actual Runs instantiate it. Admission remains a
separate authority relation and the only boundary that creates a governed
successor. The Clause Graph is the canonical inspectable carrier of checked
process constitution, admitted boundaries, and the process relations retained
or recoverable for their declared consumers. It neither runs by being stored
nor acquires independent authority. Every physical execution must refine the
selected constitution and transition semantics and preserve their declared
identities, observations, effects, failures, resources, diagnostics, and causal
order.

These names are constitutional:

| Name | Meaning |
| --- | --- |
| Running | Actual semantic carry-through, whether finite, suspended, streaming, reactive, branching, or ongoing. |
| Distinction | A stable difference maintained by running. |
| Term | The holdable, recursively composable carrier of a distinction. |
| ClauseJudgment | A contextual, non-governance judgment over a neutral Term under an exact stance and interpretation. A lower-case clause is such a judgment; it may establish formation, relation, law, proposition, assertion content, or another declared stance without issuing a governed JudgmentOccurrence or running an Application. |
| FormationJudgment | A contextual typing/formation claim; it grants no policy or execution authority. |
| ApplicationForm | A checked closed exact-schema/operator/named-role/eligible-Mode/context configuration over a Term. |
| Application | A nominal node instantiating one exact ApplicationForm, with `ApplicationId`. |
| AllocationJudgment | The typed `Retain` or `Fresh` decision that alone establishes continuity or a new identity in one declared identity domain. |
| CheckedConstitutionBinding | The exact non-self-referential selection of either a checked non-authoritative ProgramSnapshot candidate or an admitted ProgramRevision selecting that snapshot. It fixes meaning but does not itself grant authority. |
| StaticActivationBasis | The checked formation, mode-executability, dependency, exact constitution binding, and exact citations for any Mode prerequisite discharged statically that make one Application/Mode pair callable. It records but never creates or substitutes for Authorization, capability, or Admission authority. |
| DynamicPrerequisiteSchema | The selected Mode's finite stable-slot contract for values that must be supplied when an Activation begins; it may be empty and is not a causal frontier. |
| DynamicPrerequisiteBindings | Exact slot-and-ordinal-preserving values closing one DynamicPrerequisiteSchema. Equal values and repeated occurrences never collapse. |
| ActivationStartRecord | The one canonical record combining StaticActivationBasis, InitialContext, DynamicPrerequisiteBindings, and the separately projected occurrence-only ActivationCauseFrontier. Every fixed continuation pin derives from this record. |
| Activation | One actual engagement of an Application under one selected mode, valid StaticActivationBasis, immutable initial pins, exact bindings for the Mode's possibly empty dynamic-prerequisite schema, one typed occurrence-only cause frontier, and one exact Run membership. |
| ActivationConfiguration | Semantic process state before or after a Step of one stable `ActivationId`; it is not the stable identity. |
| Internal reduction | An anonymous carry-through inside one Step cut. It may update exclusively owned local configuration but creates no Step, occurrence, trace, or revision unless a declared semantic boundary is crossed. |
| StepBoundaryRef | One `(ModeId, boundary-local-id)` selecting a Mode-owned semantic grouping contract. |
| Step | One externally meaningful instance of one exact StepBoundarySchema under an exact finite typed cause frontier and exactly one permitted configuration transition. |
| Run | A process envelope with one unique root Activation, zero or more uniquely owned child Activations, and exact partial order from typed frontier plus configuration-succession edges. |
| CausalOrder | The heterogeneous partial order obtained as the transitive closure of all checked direct causal edges. Every per-Run `RunOrder` edge embeds in it, while global paths never add `RunOrder` edges. |
| Continuation | The typed semantic remainder of an Activation, never merely a host stack frame. |
| Observation | An identified occurrence reporting a distinction from a Step or external boundary. |
| OccurrenceProvenance | A checked sum naming either the exact producing Activation/Step or the exact external boundary, evidence, and typed causal frontier through which an occurrence entered. |
| Result | A declared completion product; an ongoing Run need not manufacture one. |
| Value | A stabilized typed distinction or denotation reusable under declared equality, whether supplied, observed, or produced. |
| Evaluation | A species of running whose selected Mode seeks an observation, result, verdict, or normal form. It is not the process kernel itself. |
| RelationSchema | A typed named-role and constraint surface over admissible bindings. |
| RelationExtension | An extensional set or multiset of admitted bindings at one exact revision boundary. |
| OperatorRef | An exact reference to the operator/process definition configured by an ApplicationForm. |
| Mode | A contract for direction, known and produced roles, cardinality, purity/effects, failure, continuation, scheduling, identity, lifetime/ownership, resources, and cost. |
| Function | An operator Mode established as pure, deterministic, and single-result for its declared direction. |
| Procedure | An operator Mode whose contract permits effects or authoritative transition proposals. |
| Proposition | Closed truth-apt relational or application content eligible for truth-directed interpretation or evaluation under a world; never an assertion by representation alone and not necessarily executable. |
| AssertionOccurrence | One identified act placing proposition content under an assertive stance, source, scope, and authority. |
| Judgment | Immutable checked assessment content naming its subject, stance, authority kind, policy, and scope. A declaration in a candidate snapshot is not authoritative merely by being checked. |
| JudgmentOccurrence | One identified issuance of an exact Judgment by an exact authority under an exact policy and context; its issuance basis must already be authoritative. |
| Authorization | A typed Judgment permitting one exact action and scope. Its use is either a constitutive citation anchored in an already authoritative ProgramRevision or `IrreducibleRootConstitution`, or an issued AuthorizationOccurrence whose basis was already authoritative; each subtype declares its own exact use contract. It is never a Capability. |
| Entity | A domain-level continuity projection, not the universal kernel noun. |
| Referent | Whatever a Designation picks out under an explicit identity protocol. |
| Identifier | A typed token designating one declared identity domain; its bytes do not define the continuity relation. |
| Type | A constraint on application formation, activation context, observations/results, continuation, failure, effects, deltas, ownership/lifetimes, resources, and representation. |
| Law | A universally available relational/process constraint that authorizes neither derivation nor activation by itself. |
| Rule | A declared transformation or derivation process under an explicit phase and authorization. |
| Query | An Activation seeking observations or bindings, never a syntactically special false assertion. |
| ProgramSnapshot | An exact immutable checked process constitution. |
| ProgramRevision | One admitted historical selection of an exact ProgramSnapshot in a Program lineage. |
| RuntimeSession | One execution lineage pinned to an exact ProgramRevision, policy, and semantics epoch. |
| StateRevision | One admitted runtime process boundary with exact session, predecessor, causal occurrence, payload, policy, and semantics. |
| Effect | A boundary-crossing process whose intent occurrence, issued EffectAuthorization Judgment/occurrence, capability evidence, attempt occurrence, optional receipt occurrence, Observations, JudgmentOccurrences, and any Admission remain distinct. |
| Admission | The only operation that creates an authoritative Program, State, or other governed successor. |
| AdmissionRequest | Canonical complete content of one proposed Admission decision; its content-derived key is retry identity, while the authoritative decision remains nominal. |
| Trace | A retained projection of a Run; it is never the Run itself. |
| StaticParameterTelescope | One declaration-level, rank-1 ordered set of static parameters and their Clause-owned FormationJudgments. |
| InstantiationUseRef | Exact snapshot-local provenance of one closed parametric use. It is not a cross-snapshot reuse key. |
| InstantiationKey | Cross-snapshot semantic checking key over one canonical parametric interface, normalized arguments, constraint obligations, resolution-scope commitments, and evidence. It is not nominal Application or occurrence identity. |
| SpecializationKey | Semantic implementation key adding the exact body and transitive semantic dependency closure to an InstantiationKey. |
| PhysicalReuseKey | Physical cache key adding the exact AcceptedRefinementWitnessId, target profile, ABI/layout/strategy, and physical dependencies to a SpecializationKey. |
| Lifetime root | Exactly one reclamation root for a physical allocation: affine owner, region membership, or a foreign manager. Borrow and Lease are separately typed access edges, not roots. |

Capitalized **Clause** names the language. Lower-case **clause**, where used as
a technical noun, means one contextual `ClauseJudgment` over a neutral Term.
Its exact context, stance, interpretation, and subject determine what it says.
A ClauseJudgment is neither a governed `Judgment` nor its issuance occurrence;
representing it does not assert it, authorize anything, or execute an
Application. An `ApplicationForm` is a different checked object that may be
established or constrained by clauses, and an `Application` is one nominal
instance of a closed form. Compound semantic forms have Term representations
and may participate as nodes in further clauses or applications; Clause does
not claim that every compound thing is ontologically one kind of object.

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

Three slots make two participants and their relating Term directly inspectable
in one uniform compound. This is a carrier choice, not a proof of semantic
minimality: nested pairs can encode triples and triples can encode pairs.
Neither encoding removes the interpretation, binding, equality, or admissibility
obligations. The smallness worth preserving is the number of independent
semantic rules, not the constructor's arity. Higher-arity meaning uses checked
neighborhoods or recursively complete structural Terms.

The positions of a `Triple` are structurally neutral. A checked relational
Reading may interpret one as:

```text
[left Term, relating Term, right Term]
```

That representational three is not an operational formation/activation/step
taxonomy:

```text
formation:   Term + schema/operator/role/context requirements
activation:  ApplicationId + ModeId + StaticActivationBasis + exact initial
             pins + the Mode's declared dynamic prerequisites
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
under the declared contract, and `Triple` equality is recursive equality of
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

There is no mandatory nominal identity for every `Triple`. Clause allocates
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

Every nominal or occurrence identity is established by one extensible typed
allocation judgment owned by its identity domain:

```text
AllocationJudgment<D : IdentityDomain> :=
    Retain(prior : Id<D>,
           witness : ContinuityWitness<D>)
  | Fresh(basis : FreshBasis<D>,
          producer : SemanticProducerRef<D>,
          slot : AllocationSlot<D>)

AllocationSlot<D> :=
    Singleton
  | EmissionSlot(ProductionRef, stable semantic local slot,
                 optional RepetitionSlotId)
  | MultiplicitySlot(stable semantic role, RepetitionSlotId)
  | DomainSlot(exact domain-declared stable semantic slot)

FreshBasis<D> :=
    ConstitutedBasis(exact finite D-declared predecessor inputs)
  | EnteredRootBasis(exact RootAllocationConstructorRef<D>,
                     exact BoundaryRef,
                     exact RootUniquenessWitness<D>)
```

`ContinuityWitness<D>` is the exact domain-declared proof that the prior
identity and proposed object are one continuing semantic occurrence or
referent. `FreshBasis<D>` is a finite canonical well-founded allocation basis
declared by `D`; it cannot cite the identity being allocated. The semantic
producer is the exact constituted declaration, boundary, occurrence, or
earlier binder whose declared act produces the identity. A slot distinguishes
multiple products of one producer without borrowing traversal order. Domains
extend these sums only through checked declarations fixing the basis,
producer, slot, collision, and continuity contracts.

An irreducible root is legal only through a domain-declared
`RootAllocationConstructorRef<D>` already constituted outside the identity it
allocates. Its entered boundary is an irreducible generative binder over one
exact domain scope: it atomically validates a non-self-referential
`RootUniquenessWitness<D>` establishing freshness in that scope and publishes
the root allocation judgment once. A monotone counter, entropy, or another
physical mechanism may realize the binder, but its raw bytes alone are never
semantic uniqueness evidence. Source bytes, wall time, machine identity, a
UUID, a handle, or physical allocation likewise cannot justify the witness.
Domains without this exact scoped root rule can allocate only from an already
constituted basis. Thus a fresh identity never bootstraps itself and the
semantic rule requires no global historical scan.

At checked candidate-snapshot construction every accepted source emission is
assigned a stable `EmissionSlot` in the projected `IdentityPlan`. The slot is
derived from the selected source production and its stable semantic child
slot. When one child slot repeats, each member has an explicit
`RepetitionSlotId`: a retained member carries its prior slot ID through the
IdentityPlan and an inserted member receives a fresh slot ID under the exact
candidate-change producer. A canonical ordinal may order the resulting
encoding, but never proves continuity and insertion never renumbers retained
members. Raw span, byte position, traversal order, caller bytes, random
host UUID, pointer, handle, storage row, interning order, or physical allocation
is never a semantic allocation basis. Such data may help realize an allocator
or recover a source map but cannot justify `Retain`, `Fresh`, or equality.

Allocation construction is checked as one finite dependency graph. A fresh
basis may name already constituted predecessors or earlier fresh binders in
one explicitly atomic constructor; the constructor checks the whole proposed
graph for domain correctness, unique slots, collisions, and cycles before
publishing any member. This is the generic rule that permits a Step and its
outputs, or a split and its child bindings, to be co-formed without pretending
that publication order is causality.

Reload and replay observe the recorded `AllocationJudgment`; they never run a
fresh allocator for an existing occurrence. Exact recomputation and byte
equality apply only to rematerialization of that same recorded occurrence, or
to an identity domain whose declaration says the identity is derived from an
exact canonical preimage. Independently fresh causal identities need not and
ordinarily must not have equal bytes. Their equivalence across independent
elaboration or schedule realizations is a typed, domain-preserving nominal
isomorphism that preserves every allocation-basis, producer, slot, and causal
edge. A printer/elaborator round trip is identity-correct exactly when its
projected IdentityPlans are equal for retained/derived identities and related
by that isomorphism for independently fresh identities. A duplicate identity
for distinct allocation judgments, or equal digest with different canonical
preimages, is typed `IdentityCollision` rejection.

Declarations, nominal Applications, and candidate constitutive Judgment
declarations use typed snapshot-local identifiers inside the canonical
ProgramSnapshot preimage. Their local presence does not make them authoritative.
Once the snapshot identity is known, exact external references are formed
without placing any snapshot-scoped reference back into that preimage:

```text
RelationSchemaId = (ProgramSnapshotId, RelationSchemaLocalId)
RoleId           = (RelationSchemaId, RoleLocalId)
OperatorRef      = (ProgramSnapshotId, OperatorLocalId)
ModeId           = (OperatorRef, ModeLocalId)
ApplicationId    = (ProgramSnapshotId, ApplicationLocalId)
JudgmentRef      = (ProgramSnapshotId, JudgmentLocalId)
InstantiationUseRef = (ProgramSnapshotId, InstantiationLocalId)
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
| `JudgmentRef` | One exact Judgment declaration in one exact ProgramSnapshot. A candidate declaration is not authority. It becomes eligible for constitutive use only when paired with an already authoritative ProgramRevision selecting that snapshot; runtime issuance may instead cite it or carry exact non-constitutive Judgment content in a JudgmentOccurrence. The reference is not the issuance. |
| `InstantiationUseRef` | Exact provenance of one closed use record in one ProgramSnapshot. The local record uses only snapshot-local or already resolved references; the external tuple resolves after ProgramSnapshotId exists. It changes with the containing snapshot and is never a cross-snapshot cache key. |
| `InstantiationKey` | Semantic checking key over one canonical `ParametricInterfaceId`, normalized arguments, named constraint obligations, exact resolution-scope commitments, and normalized evidence. It is cross-snapshot reusable only when every nominal input has a portable typed reuse identity; otherwise it is deliberately snapshot-bound. It is never nominal continuity, authority, or sufficient physical-artifact identity. |
| `SpecializationKey` | Cross-snapshot semantic implementation key adding the exact declaration-body content identity and transitive semantic dependency closure to an InstantiationKey. An interface-stable body change preserves checking reuse but invalidates specialization reuse. |
| `PhysicalReuseKey` | Exact physical-cache key adding one exact AcceptedRefinementWitnessId, target and feature profile, ABI/layout/strategy, and physical dependency closure to a SpecializationKey. `ArtifactId` remains the exact resulting bytes. |
| `ApplicationShapeId` | Post-snapshot identity of canonical closed ApplicationForm content under one `ClauseSemanticsId`, including exact RelationSchemaId, OperatorRef, eligible ModeIds, named-role bindings, context requirements, exact InstantiationUseRefs with their InstantiationKeys and SpecializationKeys, and the exact resolved semantic-dependency/declaration closure, which may be proven empty. PhysicalReuseKey is excluded. It never occurs in its own ProgramSnapshot preimage. Open formation candidates are not ApplicationForms and have no semantic shape ID. Used for exact resolved-form comparison, never nominal occurrence or cross-snapshot reuse. |
| `ApplicationId` | One nominal Application instantiating one exact ApplicationForm under its exact semantics and snapshot-local declaration references. Every Application has one. Source-only movement may preserve it when the exact ProgramSnapshot and form are unchanged; a semantic or declaration revision creates a new ApplicationId, with any intended cross-revision continuity represented separately by ReferentId evidence. |
| `OccurrenceId` and typed refinements | One actual source, assertion, external-trigger, Judgment issuance, authorization, resumption, handoff, cancellation, production, admission, effect-intent, effect-attempt, receipt, or observation occurrence. Every actual occurrence has the explicit provenance sum defined below; equal content never merges independent occurrences, and one refinement never substitutes for another. |
| `ActivationId` | One actual engagement of one exact Application, mode, and initial context. Every activation is distinct, including repeated activation of equal content. |
| `StepId` | One fresh nominal identity for an externally meaningful semantic carry-through in an Activation. The associated StepRecord, not the StepId, cites its exact StepBoundaryRef/schema and carries its owner, finite typed StepCauseFrontier, permitted StepConfigurationTransition, and schema-labelled outputs; serialization order supplies no causal edge. |
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

A `Judgment` is immutable assessed content. A Judgment declaration inside a
candidate ProgramSnapshot has no authority merely because it checks. A
`JudgmentOccurrence` is the actual issuance of Judgment content by an authority
under a policy and context; its provenance names an issuance basis that was
already authoritative before the occurrence. An `Authorization` is a Judgment
subtype whose subject is one exact action and scope;
`ExecutionAuthorization`, `DerivationAuthorization`,
`EffectAuthorization`, and `AdmissionAuthorization` are distinct typed
subtypes. An `AuthorizationOccurrenceId<A>` is a typed JudgmentOccurrence
issuing an Authorization of subtype `A`; it cannot be used as another subtype.
Constitutive use does not manufacture an occurrence: it pairs an exact
Authorization declaration with the already authoritative constitution that
makes that declaration effective.
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
nominal identity to every `Triple` or hashing a structure through itself.
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

`Triple`, Term, FormationJudgment, ApplicationForm, Application, and governed
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

Relation, operator, mode, reading, extension, static executability, and dynamic
authority are separate checked concepts:

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
  nondeterminism, ordering, continuation, scheduling, identity,
  ownership/lifetime, resource,
  temporal, cost, and admissible-strategy contracts. An operator may expose
  separate Modes for separate schemas; no Mode inherits a schema from call
  position or runtime selection.
- a source `Reading` maps syntax to exact Terms, role bindings, and declarations.
  It does not select runtime authority.
- `StaticActivationBasis` proves that an exact Application and selected Mode
  are well formed, executable, closed over their semantic dependencies, and
  bound to the exact constitution that gives those declarations meaning. This
  callability proof is required for every Activation but grants no authority.
- `ExecutionAuthorization` is optional Authorization Judgment content required
  only when the selected Mode declares a dynamically governed execution
  action. When required, the Activation cites either a
  `ConstitutiveAuthorization` anchored in an already authoritative
  ProgramRevision or `IrreducibleRootConstitution`, or an
  `AuthorizationOccurrence` issued from an already authoritative basis. A bare
  `JudgmentRef`, including one in a candidate snapshot, is never authorization.
  A Mode may declare an empty authorization requirement. Dynamic capability,
  effect, derivation, and Admission requirements remain separate.
- `CapabilityEvidence<C>` is the exact typed grant, lease, or boundary evidence
  required to use capability `C` under its declared scope, pins, validity, and
  resource contract. It is present only when the selected Mode declares that
  dynamic prerequisite and never substitutes for Authorization or Admission.

`StaticActivationBasis` establishes callability. Any required
`AuthorizationEvidence<ExecutionAuthorization>` licenses running only under the
selected Mode's exact contract. Neither licenses a real effect attempt or
Admission: a real effect attempt requires the selected effect profile's exact
intent, separately issued `AuthorizationOccurrenceId<EffectAuthorization>`, and
independent `CapabilityEvidence<C>`, while Admission requires separately typed
`AuthorizationEvidence<AdmissionAuthorization>`.

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
not a capability, and a static executability proof is not an Authorization.
Relation, mode, law, and Application existence never by themselves authorize a
dynamically governed execution, derivation, Admission, or external act.

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

ConstitutiveAuthorization<A : Authorization> :=
    ProgramConstitution(ProgramRevisionId,
                        JudgmentRef<A>)
  | IrreducibleRootConstitution(RootPolicyRef,
                                RootAuthorizationRef<A>)

AuthorizationEvidence<A : Authorization> :=
    Constitutive(ConstitutiveAuthorization<A>)
  | IssuedAuthorization(AuthorizationOccurrenceId<A>)

CheckedConstitutionBinding :=
    CheckedCandidate(ProgramSnapshotId, exact CheckedPackageRef)
  | AdmittedConstitution(ProgramRevisionId, exact ProgramSnapshotId)

StaticActivationBasis :=
    exact ClauseSemanticsId
  + exact CheckedConstitutionBinding
  + exact ApplicationId and its closed ApplicationForm
  + exact selected eligible ModeId
  + closed FormationJudgments and mode-executability proof
  + exact citations and proofs for every Mode prerequisite discharged
    statically, each retaining its own authority or capability identity
  + exact InstantiationUseRefs, InstantiationKeys, SpecializationKeys,
    and semantic dependency closure

WorldContext :=
    NoWorld
  | ReadOnlyAdmittedWorld(exact StateRevisionId,
                          exact WorldViewContract)
  | SessionWorld(exact RuntimeSessionId, exact StateRevisionId)

InitialContext :=
    exact WorldContext
  + exact presence or absence of RuntimeSessionId and RuntimePolicyId
  + exact budget, cancellation and continuation scope
  + exact declared scheduler and runtime constraints

DynamicPrerequisiteRequirementKind :=
    Authorization<A>
  | Capability<C>
  | EffectIntent<EffectIntentContract>
  | RequiredObservation<ObservationContract>
  | RequiredAdmission<AdmissionContract>

PrerequisiteSlotId := (ModeId, PrerequisiteLocalId)

DynamicPrerequisiteRequirement :=
    exact PrerequisiteSlotId
  + optional exact RoleId
  + exact DynamicPrerequisiteRequirementKind and expected type
  + exact cardinality
  + finite canonical CauseProjectionSchema whose entries are
    (CauseComponentLocalId, typed occurrence path in the bound value)

DynamicPrerequisiteSchema :=
    finite canonical ordered, multiplicity-aware sequence of
    DynamicPrerequisiteRequirement

EffectIntentContract :=
    GovernedIntent(exact AdmissionContract)
  | PreauthorizedIntent(ActivationLocal | Session | Lease | Batch,
                        exact bounded scope and validity contract)

EffectExecutionProfile :=
    GovernedPerIntent(exact governed-intent,
                      exact issued-authorization,
                      exact capability requirements)
  | PreauthorizedEffect(exact preauthorized-intent,
                        exact issued-authorization scope,
                        exact capability requirements,
                        exact attempt, budget, and renewal bounds)

DynamicPrerequisiteBinding :=
    exact PrerequisiteSlotId
  + exact repeated-value ordinal
  + exact typed value satisfying that requirement kind:
      AuthorizationEvidence<A>
    | CapabilityEvidence<C>
    | EffectIntentOccurrenceId plus the exact AdmissionOccurrenceId
      required only by GovernedIntent
    | ObservationId satisfying ObservationContract
    | AdmissionOccurrenceId satisfying AdmissionContract

DynamicPrerequisiteBindings :=
    finite canonical sequence that closes one exact
    DynamicPrerequisiteSchema without omission, addition, or collapse

ActivationOccurrenceCause :=
    PrerequisiteOccurrence(PrerequisiteSlotId,
                           repeated-value ordinal,
                           CauseComponentLocalId,
                           exact typed OccurrenceId)

ActivationCauseFrontier :=
  exactly one ActivationOrigin
  + a finite canonical sequence of ActivationOccurrenceCause

ActivationStartRecord :=
  exact StaticActivationBasis
  + exact InitialContext, including presence or absence of every pin
  + exact DynamicPrerequisiteBindings
  + exact ActivationCauseFrontier

RunMembership := RootOf(RunId) | ChildIn(RunId)

Γ; G; W ⊢ activate(ActivationStartRecord) ↦
  ⟨ActivationId, RunMembership, InitialConfiguration⟩

StepCause :=
    ActivationStart(ActivationId)
  | PriorStep(RunId, predecessor ActivationId, predecessor StepId)
  | ContinuationTakeup(ContinuationId,
                       emitting RunId,
                       emitting ActivationId,
                       emitting StepId,
                       ResumptionOccurrenceId | HandoffOccurrenceId)
  | CancellationRequest(CancellationOccurrenceId)

StepCauseFrontier := finite canonical set of StepCause

ConfigurationPredecessor :=
    ActivationStart(ActivationId)
  | ConfigurationAfter(StepId)

ActivationConfigurationToken :=
    affine(exact ActivationId,
           exact ConfigurationPredecessor,
           exact configuration commitment)

BranchKey := contract-local key, not a global identity domain

BranchSlot := (exact BranchKey, exact repeated-spec ordinal), where ordinals
              are contiguous and canonical within each repeated BranchKey

BranchSpec :=
    exact BranchSlot
  + exact child-Activation requirements
  + exact configuration partition
  + exact terminal settlement and join obligations

SplitJoinContract :=
    Mode-owned finite canonical BranchSlot-ordered sequence of BranchSpec
  + proof that the partitions are pairwise disjoint
    and exactly cover the consumed parent configuration

SplitInstance :=
    exact RunId
  + exact parent ActivationId
  + exact split StepId
  + exact SplitJoinContract commitment

BranchConfigurationToken :=
    affine(exact SplitInstance,
           exact BranchSlot,
           exact child ActivationId,
           exact child ActivationConfigurationToken)

SplitChildBinding :=
    exact SplitInstance
  + exact BranchSlot and matching BranchSpec
  + exact fresh child ActivationId
  + exact ActivationStartRecord whose ActivationOrigin is
    ChildOf(SplitInstance.RunId,
            SplitInstance.parent ActivationId,
            SplitInstance.split StepId)
    and whose start requirements exactly satisfy that BranchSpec
  + exact ChildIn(SplitInstance.RunId)
  + exact initial child ActivationConfigurationToken whose predecessor is
    ActivationStart(child ActivationId)
  + exact BranchConfigurationToken wrapping that initial token

BranchDischargeProof :=
    every exact AllocationRoot (Owned, RegionMember, or ForeignManaged,
    including every Clause-owned foreign-wrapper obligation), Borrow, Lease,
    Continuation, effect obligation, and close obligation is discharged or
    transferred exactly as declared

BranchSettlement :=
    Returned(exact BranchSlot,
             consumed terminal BranchConfigurationToken)
  | Closed(exact BranchSlot,
           consumed terminal BranchConfigurationToken,
           terminal close-or-cancel StepId,
           typed outcome,
           exact BranchDischargeProof)

StepConfigurationTransition(s : StepId) :=
    Serial(consume ActivationConfigurationToken,
           produce ActivationConfigurationToken whose predecessor is
             ConfigurationAfter(s))
  | Split(consume parent ActivationConfigurationToken,
          exact SplitJoinContract,
          produce canonical BranchConfigurationTokens whose exact
            SplitInstance.split StepId is s)
  | Branch(consume BranchConfigurationToken,
           produce BranchConfigurationToken whose predecessor is
                     ConfigurationAfter(s)
                 | BranchSettlement whose consumed terminal token names
                     ConfigurationAfter(s), and whose Closed terminal StepId
                     is s)
  | Join(consume canonical exact BranchSlot-ordered BranchSettlements,
         produce parent ActivationConfigurationToken whose predecessor is
           ConfigurationAfter(s))

StepBoundaryRef := (exact ModeId, StepBoundaryLocalId)

StepBoundarySchema :=
    exact StepBoundaryRef owned by that Mode
  + permitted transition/custody variant and exact before/after shape
  + finite typed incoming-cause schema
  + finite named output-role and output-kind schema, each with exact
    cardinality and atomic-grouping contract
  + exact visibility of semantic commit, failure, and cancellation
  + only explicitly semantic scheduling, resource, or progress checkpoints

StepRecord(s : fresh StepId) :=
    exact StepBoundaryRef and its checked StepBoundarySchema
  + exact owner (RunId, ActivationId)
  + exact StepCauseFrontier
  + exact StepConfigurationTransition(s)
  + exact schema-labelled observations, outcome, candidate delta,
    Continuation, and other declared outputs

Ready(a : ActivationId) :=
    exact constituted Activation a
  + zero StepRecords owned by a
  + live, unconsumed exact initial ConfigurationCustody

SplitFormation :=
  ν splitStepId, {childActivationId[slot] for each BranchSlot}.
    exact Split StepRecord(splitStepId)
  + exact SplitInstance
  + canonical BranchSlot-ordered SplitChildBindings

ConfigurationCustody :=
  the exact affine input and output token or settlement sequence selected by
  one StepConfigurationTransition variant

ConfigurationSuccessionEdge(producer StepId, consumer StepId) :=
  the consumer's checked StepConfigurationTransition consumes a whole token,
  branch token, or settlement produced or terminally constituted by producer

StepCauseFrontierEdge(predecessor StepId, consumer StepId) :=
    an exact predecessor projected by one checked StepCause in the consumer's
    StepCauseFrontier, where ActivationStart projects the distinct union of the
    exact ChildOf parent Step or HandoffFrom Continuation-emitter Step, direct
    same-Run provenance roots of the exact HandoffOccurrence when that origin is
    HandoffFrom, and exact producer Steps of ordinary occurrence-backed
    Activation causes in that Run

StepCauseFrontierEdges(s) :=
    every StepCauseFrontierEdge whose consumer is s

StepConfigurationSuccessionEdges(s) :=
    every ConfigurationSuccessionEdge whose consumer is s

IncomingRunEdges(s) :=
    StepCauseFrontierEdges(s)
  ∪ StepConfigurationSuccessionEdges(s)

RunOrder := transitive closure(
    typed StepCauseFrontier edges
  ∪ typed ConfigurationSuccessionEdges)

CausalNodeRef :=
    Activation(RunId, ActivationId)
  | Step(RunId, ActivationId, StepId)
  | Occurrence(exact typed OccurrenceId)
  | Output(exact typed producer output identity)
  | Candidate(exact typed candidate identity)
  | Judgment(JudgmentOccurrenceId)
  | Admission(AdmissionOccurrenceId)
  | Authorization(AuthorizationOccurrenceId<exact subtype>)
  | EffectIntent(EffectIntentOccurrenceId)
  | EffectAttempt(EffectAttemptOccurrenceId)
  | Receipt(ReceiptOccurrenceId)
  | Extension(exact node-kind declaration, exact typed identity)

DirectCausalEdge :=
    checked typed edge(CausalNodeRef predecessor,
                       CausalNodeRef dependent,
                       exact dependency role)

CausalOrder := transitive closure(all DirectCausalEdges)
```

One `StepBoundarySchema` may have arbitrarily many dynamic Step instances and
may admit dynamically sized output batches only through its declared
cardinality and stable output slots/RepetitionSlotIds; a canonical ordinal may
order encoding but cannot establish output continuity. Conversely, a change to
boundary placement, atomic grouping, cause shape, custody, output roles,
visibility, or declared checkpoint changes the owning Mode's meaning and its
exact `ModeId`. A compiler reduction, evaluator instruction, loop iteration,
allocation safepoint, scheduler yield, logging boundary, or progress counter is
not a Step unless the selected Mode declares it through a `StepBoundaryRef`.
Anonymous reductions and invisible physical safepoints have no boundary ref,
StepId, `StepRecord`, or causal edge.

Every `RunOrder` edge is also one direct or transitively represented edge in
`CausalOrder`, including both the Step cause frontier and configuration
succession. `RunOrder` remains Step-only and per-Run: no global path through an
Admission, authorization, effect, log, or another Run manufactures a cross-Run
`RunOrder` edge. `CausalOrder` is heterogeneous and may order nodes from
different Runs through their actual dependencies. Its direct edges include
occurrence-to-Activation/Step projections, Step-to-output, output-to-candidate,
candidate/evidence/support-to-Judgment, request inputs and authorization use to
Admission, intent/authorization/capability-to-effect attempt, attempt-to-
receipt/observation, and every exact incoming edge required by an extensible
node declaration. Configuration succession is never omitted merely because a
frontier edge also exists.

Every published causal node therefore declares and checks its complete finite
typed incoming schema. A predecessor is either already constituted or is an
earlier fresh binder in one explicitly atomic constructor whose entire direct-
edge graph is checked acyclic before publication. Encoding, registration,
storage, traversal, log, arrival, clock, and host scheduling order add no edge.
Nodes with no dependency path remain incomparable; canonical encoding order
does not turn `CausalOrder` into a total order.

`CheckedPackageRef` is an exact immutable binding to canonical package bytes,
semantics epoch, decoded sections, and the checker result over those bytes. It
is transport/checking evidence, not a ProgramRevision, authority, or movable
name.

In either variant, the selected ProgramSnapshot must be the exact snapshot in
the ApplicationId and every snapshot-scoped declaration used by the form and
its instantiations. An equal preimage under another semantics epoch, an
equal-looking Application, another checked package, or a ProgramRevision that
selects a different snapshot rejects before Activation.

`G` is the exact ProgramSnapshot constitution, and `W` is the exact initially
observed StateRevision when the `ActivationStartRecord` selects a world. A
nonauthoritative candidate run may select `ReadOnlyAdmittedWorld` without
gaining permission to alter it; its exact `WorldViewContract` proves how that
admitted world's schema is read under the candidate constitution.
`StaticActivationBasis` binds the exact semantics,
checked constitution, ApplicationForm, selected Mode, closed formation,
normalized static instantiations, dependency closure, and exact citations for
any prerequisite that the Mode permits to be discharged statically. Those
citations retain the identity and scope of the underlying Authorization or
capability; the basis cannot invent one, turn it into another kind, or serve as
Admission authority. It proves that the Application/Mode pair can run; it is
not a causal origin. The start record's `InitialContext` additionally
pins RuntimeSession only for `SessionWorld`, runtime policy when present,
budget, continuation/cancellation scope, and observable scheduler constraints.
Dynamic prerequisite values are bound separately and cannot hide inside that
context or the causal frontier.

Activation selects one exact `ModeId` from the ApplicationForm's stored
eligible-Mode set. The selected Mode's activation contract fixes every stable
prerequisite and its discharge boundary. Requirements discharged statically
are cited by `StaticActivationBasis`; only values allowed to vary at Activation
form the Mode-owned `DynamicPrerequisiteSchema`: stable named slots, optional
RoleId association, kind and expected type, relationship, cardinality, and a
typed cause-projection schema. The schema contains no AuthorizationEvidence,
CapabilityEvidence, occurrence identity, AdmissionOccurrenceId, or other
Activation-specific value. It may be empty. Every Activation binds every
exact slot and repeated-value ordinal; RoleId is read from the requirement and
is not duplicated in the binding. Equal values in different roles, slots, or
ordinals remain distinct.
The checker rejects a missing, extra, duplicate-for-one-slot, wrong-slot,
wrongly typed, stale, or multiplicity-collapsed binding before allocating an
ActivationId.

`DynamicPrerequisiteBindings` and `ActivationCauseFrontier` are different
objects. Only an occurrence selected by the requirement's explicit
`CauseProjectionSchema` projects into the frontier, labelled by exact slot,
repeated-value ordinal, and CauseComponentLocalId. A governed intent therefore
projects both its intent and Admission occurrences without merging them; an
issued Authorization, Observation, Admission, or occurrence-backed capability
likewise contributes its exact typed OccurrenceId under its declared component.
Constitutive citations, capability values or leases, and other
non-occurrence evidence do not become causal edges merely because they satisfy
a slot. The complete label preserves dependency multiplicity even when two
slots cite equal content or the same occurrence. Every Activation still requires
one valid `StaticActivationBasis` and exactly one `ActivationOrigin`; basis and
origin suffice without further bindings only when the entire selected Mode's
dynamic-prerequisite schema is empty.

`CheckedCandidate` permits sandbox, compiler, test, query, and other explicitly
nonauthoritative running against exact checked package bytes and their exact
`ProgramSnapshotId`. It may read an exact admitted world, persist a test or
compiler artifact, externalize a trace projection, or suspend a candidate
Continuation. It may also exercise an inert simulated-effect adapter whose
declared observation is simulation rather than an external attempt. None of
those physical persistence operations creates a ProgramRevision,
RuntimeSession, StateRevision, constitutive Authorization, real
EffectAttemptOccurrence, or durable authority by pretending that the candidate
was admitted. Its observations, artifacts, continuations, and diagnostics
retain that nonauthoritative constitution binding. An Activation that
participates as a member of an authoritative `RuntimeSession`, proposes or
performs authoritative world change, relies on constitutive Program authority,
or performs a real external effect under an admitted constitution uses
`AdmittedConstitution`. It is pinned to the exact ProgramRevision selecting the
same ProgramSnapshot and satisfies every additional session/world/policy
contract. Merely reading an exact admitted world through a declared read-only
candidate input does not trigger that rule.

Only `AdmittedConstitution` or `IrreducibleRootConstitution` may discharge a
`ConstitutiveAuthorization`. `CheckedCandidate`, successful checking, package
possession, or execution evidence cannot. An issued Authorization may govern a
nonauthoritative run only where its own exact subject and policy explicitly do
so; it still cannot turn the candidate into an admitted constitution.

`ProgramConstitution` is valid only when the named ProgramRevision already
exists authoritatively before the action being authorized and selects the exact
ProgramSnapshot named by its `JudgmentRef`. `IrreducibleRootConstitution` names
an exact typed authorization in an independently established root policy; it
cannot be synthesized from a Program candidate, candidate evidence, or the
successor under consideration. `IssuedAuthorization` names an already existing
AuthorizationOccurrence whose issuance basis was itself already authoritative
and whose subject, scope, policy, and type cover the exact action. These checks
are well-founded: an authorization occurrence cannot use itself, its authorized
action, or evidence produced by that action as its issuance basis.

`RootPolicyRef` is an exact immutable reference to an irreducible policy anchor
established outside the candidate-governed revision relation;
`RootAuthorizationRef<A>` selects one typed Authorization declaration inside
that policy. Neither reference can be allocated, altered, or vouched for by the
candidate action it authorizes.

A Mode that declares `ExecutionAuthorization` may use a constitutive
authorization where its scope covers the exact Application, Mode, session, and
context. When that constitutive basis and all relevant pins are statically
fixed, a checked physical refinement may erase its evidence from the hot ABI;
the artifact-to-basis explanation remains recoverable and changing a covered
pin invalidates the specialization. A Mode with no Authorization requirement
creates no synthetic grant or runtime check. Issued or otherwise dynamic
Authorization and capability evidence cannot be erased across the boundary at
which it may vary.

Every real external-effect Mode declares one exact `EffectExecutionProfile`.
Every resulting effect Activation closes three independent dynamic slots: the
exact intent occurrence, an issued
`AuthorizationOccurrenceId<EffectAuthorization>` covering that intent and
scope, and independent `CapabilityEvidence<C>` covering the boundary,
resource, pins, validity, and budget. `ConstitutiveAuthorization` may authorize
ordinary execution or issuance policy, but it never replaces the issued effect-
authorization slot.

`GovernedPerIntent` binds an exact AdmissionOccurrence together with the intent
slot. `PreauthorizedEffect` instead binds a previously issued, bounded
activation-local, session, Lease, or batch scope. That scope may cover several
attempts, so it manufactures no per-attempt StateRevision, Admission, or new
AuthorizationOccurrence; each attempt still names the exact intent,
authorization, and capability it consumes. A statically pinned issued
authorization or capability may erase from a checked hot ABI, but it remains an
exact semantic slot and cold artifact-to-basis explanation. Renewal creates a
new issued occurrence at the declared boundary rather than silently extending
scope.

Both profiles keep intent, issued Authorization, capability, attempt, optional
receipt, Observation, Judgment, and possible later Admission distinct. Their
occurrence-backed values project under exact slot or Step-cause identities;
non-occurrence capability values do not become causal edges. Authorization may
cite a capability contract but cannot satisfy it. Ambiguous, missing, stale,
unauthorized, malformed, ungrounded-known-role, or over-budget running rejects
before an EffectAttemptOccurrence or partial authority exists.

Run membership is assigned at activation and never inferred from later graph
reachability. `RootedBy` allocates one fresh `RunId` and makes the Activation
that Run's unique root. `ChildOf` requires the named parent Step to belong to
the named parent Activation and Run. `HandoffFrom` additionally requires its
`parent StepId` to be the named Continuation's exact emitting Step, with the
same recorded parent Activation and Run. Its exact `HandoffOccurrence` must
target that Continuation and the destination's StaticActivationBasis and
InitialContext, including every changed or preserved pin, and its typed
provenance must be well-founded from already constituted roots. A wrong
emitter, parent owner, Continuation, destination basis or pin, future root, or
cycle rejects before child identity allocation. Both origins assign the new
Activation as a child of that same Run. Every Activation has exactly one owning
`RunId`; every Run has exactly one root Activation; a child Activation does not
silently root a second Run. A deliberately detached process uses a new typed
root trigger and a new Run, while its trigger provenance may still name the
earlier causal boundary. These rules prevent a child from being attached to an
arbitrary or multiple Runs.

One stable `ActivationId` advances through any number of
`ActivationConfiguration`s. Configuration is semantic execution state, not a
new Application or Activation. Every externally meaningful carry-through has
a fresh `StepId` and one exact `StepRecord` carrying its owning Activation and
Run, finite typed `StepCauseFrontier`, separate
`StepConfigurationTransition(s)`, and outputs. The StepId is not a content hash
of that record and does not contain its transition.

A Step proposal's frontier, transition, outputs, owner, and applicable outcome
are checked under one fresh Step binder before its `StepId` is allocated and
the `StepRecord` is published. Every cause must resolve to an already
constituted object, and every internal cause must belong to the Step's owning
Run. `PriorStep` is valid only when its Run, Activation, and Step fields exactly
match the predecessor Step's recorded ownership. It may name Steps of any
uniquely owned Activation in that Run, allowing parent/child or sibling joins
without inventing a total order. The checker rejects a self reference, an
unallocated or future Step, a cause reachable from the proposed Step, a pre-
existing causal cycle, a wrong-Run or wrong-Activation owner, a wrong
occurrence refinement, and a cause not permitted by the selected Mode.
Duplicate cause encodings reject rather than disappearing during
canonicalization; canonical typed ordering adds no causal edge. Since every
accepted incoming edge points from an already acyclic constituted predecessor
to the fresh Step, the accepted `IncomingRunEdges` preserve a finite directed
acyclic graph rather than serialization order. The complete `RunOrder` is the
separately validated union with configuration-succession edges defined below.

`Ready(a)` holds only for a constituted Activation with zero owned Steps and
live, unconsumed initial configuration custody. A normal first Step of a Ready
Activation has the exact singleton frontier `{ActivationStart(a)}`. The sole
exception is ready cancellation: after validating an exact existing
CancellationOccurrence `c`, its target and provenance, every Application,
Mode, constitution, world/session/policy and cancellation pin, and an exact
Mode permission and causal condition, plus an exact matching `Cancel(c)`
outcome, the first Step has exactly
`{ActivationStart(a), CancellationRequest(c)}`. Wrong target, pins, Mode,
occurrence kind, outcome, an extra cause, already-consumed initial custody, or
a second cancellation rejects before StepId allocation.

The inherited causal predecessors of `ActivationStart(a)` are not duplicated
as StepCause entries. Its typed edge projection is the distinct union of the
exact `ChildOf` parent Step or `HandoffFrom` Continuation-emitter Step, the
direct same-Run provenance roots of the exact HandoffOccurrence when present,
and ordinary same-Run Step ancestry from occurrence-backed Activation causes. A root or
external occurrence remains inspectable without manufacturing a cross-Run
RunOrder edge. Thus every exact same-Run ancestor precedes the child's first
Step even when no configuration token passes between them, while the child's
canonical frontier bytes remain the singleton or ready-cancellation pair
above. `ActivationStart` for another Activation rejects, and no later Step may
contain any `ActivationStart`.

A nonfirst Step may have an empty `StepCauseFrontier` only when its checked
transition contributes a configuration-succession predecessor. Every nonfirst
Step must have at least one `IncomingRunEdges` member. An empty frontier never
means that a serialized predecessor, scheduler visit, or fabricated
`PriorStep` becomes semantic causality.

`ContinuationTakeup` is valid only for a nonfirst Step of the Continuation's
exact owning Activation in the same Run. Its Run, Activation, and Step fields
must exactly equal the Continuation's already constituted emitting Step, all
semantic pins must match, and the typed ResumptionOccurrence or same-Activation
HandoffOccurrence must target that exact Continuation and Activation. The one
cause therefore records both the emitting Step and takeup occurrence; adding a
second `PriorStep` for that same emitting edge is a duplicate and rejects. A
semantic handoff that
changes Application, Mode, or another semantic pin instead creates a child
Activation through `HandoffFrom`; its normal first Step uses that child's exact
`ActivationStart` singleton, while validated ready cancellation may use only
the constitutional pair. A linear Continuation's already-consumed takeup
rejects before Step allocation.

`CancellationRequest` is valid only when the exact existing
CancellationOccurrence targets either the Step's owning Activation or its
owning Run; for a Run target, the Activation must already be a unique member of
that Run. A cancellation targeting another Activation or Run, naming another
occurrence kind, or depending on the proposed/future Step rejects. Each Step
that observes or carries through cancellation names that occurrence
explicitly. The ready-cancellation pair above is its only first-Step use. Two
Steps are independent only when neither typed frontier edges
nor typed configuration-succession edges order them; a total trace or log order
is storage evidence only. Internal KExpr reduction, CPU instructions, scheduler
ticks, and materializer visits are not semantic Steps unless the declared
observation contract exposes that boundary.

Configuration succession is exact and distinct from `StepCauseFrontier`. On an
unsplit Serial path, the first Step of an Activation consumes the sole token
whose predecessor is that Activation's start. Every later Serial Step consumes
the sole current token produced by the immediately preceding configuration-
changing Step of that Activation; the successor token commits to the new
`StepId` and `Configuration_after`. Split, Branch, and Join instead consume and
produce the exact structured custody declared by their transition variants. A
stale before-token, repeated consumption, skipped predecessor, second successor
from one token, or wrong transition variant rejects before Step allocation.
The `StepCauseFrontier` still records semantic causes and may name causes from
other Activations; it cannot substitute for this affine configuration chain.
Every checked producer-to-consumer custody transfer contributes a typed
configuration-succession edge to `RunOrder`. Thus `s2` may consume
`ConfigurationAfter(s1)` without adding `PriorStep(s1)` to its frontier while
still establishing `s1 <run s2`. No implicit `PriorStep` is inserted, and the
frontier's canonical bytes and every existing identity preimage remain
unchanged. Run reachability, freshness, reclamation, and compaction indices use
the union order rather than a frontier-only projection.
Each configuration-succession edge is validated before Step allocation: its
producer and consumer belong to the exact Run and structured custody lineage,
the consumed token or settlement is live and unconsumed, and adding the edge to
the existing union order remains acyclic. The edge does not become a new
identity or independently authored cause.
Parallel mutation requires the explicit disjoint split/join protocol below,
never two ordinary Steps consuming one configuration token.

### Local configuration and the Step cut

Clause has a cheap mutable middle tier between pure expressions and governed
world state. One live `ActivationConfiguration` is affinely owned by its
Activation. It may contain Activation-local slots that persist across internal
reductions and Steps, plus a Step-local scratch region that is created and
retired within one Step attempt. Updating either is not Admission, creates no
StateRevision, and is not durable graph or trace content by default.

Mode purity is observational; local mutation is an orthogonal execution
discipline, not a third effect class. A pure Mode may use mutable local slots,
builders, regions, and scratch when its result and evidence depend only on its
declared inputs and pins and no local reference, effect, candidate delta,
authority use, or undeclared resource or diagnostic distinction escapes.
Functional and in-place lowerings of one pure Mode must therefore be
observationally identical. A local mutation that violates those conditions is
rejected or belongs to a Mode that declares the resulting observation or
effect.

Anonymous internal reductions may read and update those local slots while all
of the following remain true:

- the current Step attempt has exclusive ownership, or checked disjoint leases,
  for every location it may mutate;
- no intermediate state is observable through another Activation, a foreign
  boundary, an Observation, a candidate delta, or a retained reference;
- failure or cancellation before the Step cut preserves the semantic
  configuration represented by the exact consumed
  `ConfigurationCustody_before`; and
- every live value crossing the cut satisfies its declared type, alias,
  resource, and lifetime obligations.

A semantic Step cut is mandatory before or at any boundary that emits or
consumes an identified occurrence, adds a causal predecessor, exposes an
Observation or result, stages a candidate delta or effect intent, attempts an
effect, yields or suspends a Continuation, hands off or cancels running, changes
an exact world or constitution pin, or exposes a mode-declared failure,
scheduling, resource, or progress distinction. A selected Mode may declare
additional cuts. It may not erase or coalesce a required cut when doing so
changes identity, causality, diagnostics, resource accounting, or another
declared observation. Machine instructions and loop iterations with none of
those consumers remain anonymous reductions.

Cancellation becomes a semantic input only at a declared Step cut or internal
safepoint. After its first destructive write, an anonymous reduction must be
infallible until the next cut, use a bounded undo journal or shadow realization
that can restore the exact consumed `ConfigurationCustody_before`, or keep all
partial writes private and
unpublished. Executor loss discards that private physical realization and
rematerializes from the last completed cut; it never claims to roll back an
external resource, Observation, or occurrence that escaped. A potentially
failing move, write, drop, lease transition, or foreign operation therefore
either precedes its destructive commit, carries exact bounded restoration, or
crosses a Step boundary before becoming observable.

An Activation-local reference cannot escape its owning Activation. A value may
cross through an Observation, result, child Activation, Continuation, candidate
delta, or foreign boundary only by a checked copy, move, stabilization, or
lease that establishes the destination type, identity, and lifetime contract.
Step-local scratch never escapes the Step and is absent from
`Configuration_after`. A boundary-crossing Continuation serializes or otherwise
materializes every still-live Activation-local obligation in its typed
remainder; an executor pointer is insufficient.

There is never an implicit shared mutable alias. Parallel work either receives
immutable values, transfers ownership to child Activations, or uses statically
disjoint affine branch tokens or explicit access leases. A selected Mode owns
each `SplitJoinContract`: its canonical multiplicity-aware BranchSpecs name
typed `BranchSlot = (BranchKey, repeated-spec ordinal)` values and prove pairwise
disjoint, exact coverage of the consumed parent configuration. Equal-content
and equal-key BranchSpecs remain distinct by their complete BranchSlot.

A `SplitFormation` consumes the sole whole-parent token and co-forms one fresh
split StepId, one fresh child ActivationId per BranchSlot, the exact Split
StepRecord, its structurally anchored `SplitInstance`, canonical
`SplitChildBindings`, and every initial child and BranchConfigurationToken.
Each binding carries its complete BranchSlot and matching BranchSpec, a fresh
child identity, exact `ChildOf(instance.run, parent, splitStep)` origin,
`ChildIn(instance.run)` membership, and live initial child custody. The
instance is anchored by RunId, parent ActivationId, split StepId, and exact
contract commitment; it receives no new global ID.

The checker validates the parent token, every partition and BranchSpec, all
freshness and child formation requirements, every origin and Run binding, and
the complete canonical token set under the atomic fresh binders. Only then may
it publish the Step, instance, children, bindings, and tokens together. Any
failure publishes none of them and leaves the parent token live and unconsumed.
The successful split leaves no residual whole-parent token. Any parent
remainder that must survive is an explicit BranchSlot in the same contract.

Every branch Step consumes only the token for its exact SplitInstance,
BranchSlot, child Activation, and current child predecessor. Terminal branch
carry-through consumes that token into exactly one `BranchSettlement`.
The terminal Step atomically forms its own `ConfigurationAfter(terminal
StepId)` token and consumes it into the settlement, leaving no second live
token or extra semantic Step. `Returned` preserves that terminal token as
consumed settlement evidence and retains its BranchSlot;
`Closed` additionally names the exact terminal close or cancellation Step,
typed outcome, and proof that every exact AllocationRoot—`Owned`,
`RegionMember`, or `ForeignManaged`, including every Clause-owned foreign-
wrapper obligation—plus every Borrow, Lease, Continuation, effect obligation,
and close obligation was discharged or transferred. A cancelled branch can
never disappear merely because it produced no value.

A Join Step consumes exactly one settlement for every expected BranchSlot in
canonical BranchSlot order and restores one whole configuration owner. Its
`StepCauseFrontier` contains one exact `PriorStep` for every settlement terminal
Step, independent of completion or arrival order; any other cause must be
separately permitted by the Mode. The frontier and configuration-transition
records stay distinct even where they order the same endpoints. Cross-split or
equal-content transplant, overlap, incomplete coverage, wrong-BranchSlot,
wrong-contract, wrong-SplitInstance, missing, extra, duplicate, or already
consumed settlement, frontier/settlement mismatch, and double join reject
before publication.

For schedule-parity checks, a typed `ScheduleIsomorphism π` is a type-
preserving bijection over fresh run-local Run, Activation, Step, Continuation,
Observation, and other occurrence identities. It preserves ownership,
Activation origins, BranchSlots, frontiers, configuration transitions,
both RunOrder edge kinds, occurrence/support multiplicity, outcomes, and all
references between those identities while fixing the shared constitution,
Application, Mode, SplitJoinContract, BranchSlots, base world, semantic pins,
and schedule-independent content. Runs under opposite
physical completion orders must satisfy `encode(π(runA)) = encode(runB)`.
Only their schedule-independent payload, Value, Result, candidate-delta,
Admission-decision, and continuation-disposition projections are required to
be literally equal. Fresh `PriorStep` fields, settlements that retain terminal
StepIds, candidate/decision records that retain fresh identities, and other
identity-bearing bytes are isomorphic rather than equal.
Host arrival order never selects merge behavior. Static
partition and obligation proofs may erase under checked specialization;
dynamically varying branch custody and settlement may not. The Split Step's
`StepId` and Join Step's `StepId` already identify those occurrences, so Clause
adds no SplitId, JoinId, or SettlementId.

“Fork” never means copying an affine configuration token. An immutable/`Copy`
value may branch freely. A concurrent mutable fork is exactly the consuming
split above and creates fresh child Activation/configuration identities plus
one declared join contract. A speculative implementation may create private
physical alternatives inside one unfinished Step attempt, but they are not
multiple semantic configurations: exactly one alternative may publish the
next token, every loser is reclaimed without an observable action, and an
effect or occurrence forces a Step cut before it can escape.

Likewise, rollback exists only inside an unpublished Step attempt. It restores
the exact consumed `ConfigurationCustody_before` selected by the transition:
the whole token consumed by `Serial` or `Split`, the exact branch token consumed
by `Branch`, or the exact canonical settlement sequence consumed by `Join`.
Restoration uses an infallible suffix, bounded undo/shadow state, or discard of
a private realization and leaves no duplicate or residual custody. It cannot
erase a completed Step, occurrence, Observation, admitted boundary, or
external act; compensating for any of those is a fresh causal process. A
rollback path that cannot close every created AllocationRoot, Borrow, Lease,
child token, or provisional settlement rejects the lowering rather than
leaking or publishing a partial configuration.

A checked physical lowering may update configuration in place when exclusive
ownership, non-escape, cut atomicity, and failure restoration are proven. It may
use registers, stack slots, arenas, mutable arrays, or state-machine fields and
erase local identity records from the hot path. The refinement must still
preserve the declared before/after relation and explain every declared Step
boundary from exact inputs, outputs, and compact retained evidence. It need not
clone or retain the full configuration history. Local mutation is therefore
cheap physical carry-through, not ungoverned authoritative state.

A Step may emit zero or more identified observations, values, evidence,
diagnostics, effect intents, resource use, a candidate delta, or a continuation.
These are separate outputs. Admission consumes only the candidate delta,
evidence, authority, and obligations; it neither consumes nor changes the
continuation. When a world pin is present, the authoritative world remains
`Wbase` throughout candidate computation.

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

A `Continuation` is the typed semantic remainder of an Activation. Suspension
transfers the Activation's sole live configuration ownership into that
remainder; no second mutable configuration remains behind. Same-Activation
takeup consumes that ownership exactly once. Reusable takeup is valid only for
an immutable/`Copy` remainder with no affine resource, or it explicitly forks
fresh child Activation/configuration identities under a declared Mode.

When a Continuation crosses suspension, handoff, persistence, or executor
boundaries it receives a `ContinuationId` and pins at least its owning Run,
Activation, emitting Step, one exact canonical `ActivationStartRecord`, typed
remainder, configuration-ownership token, remaining budget, cancellation
scope, and any explicitly advanced current-world pin. Application, Mode,
semantics epoch, constitution, initial world, session, runtime policy, dynamic
bindings, and original cause frontier are read only from that start record;
they are never serialized as independently authoritative duplicate pins. The
current-world pin must equal the initial world or carry the exact admitted-
successor/observation chain that advanced it. Remaining budget must equal the
initial budget less exact retained resource receipts. A checked-candidate start
record never fabricates ProgramRevision, RuntimeSession, or StateRevision
fields; an admitted start record retains the exact ProgramRevision it selected.

A boundary-crossing Continuation also proves that every retained Owned or
RegionMember resource is canonically portable or exactly rematerializable and
that every foreign Lease is transferable under its adapter contract. A host
pointer, nontransferable handle, executor-local borrow, or unacknowledged Lease
rejects persistence or handoff before the configuration token moves. Resumption
rejects any derived-pin, portability, ownership, or budget mismatch before
taking ownership or allocating a Step.
An implementation may keep a purely local, unobservable continuation in
registers or a host stack under a checked refinement; those mechanics are not
its semantic identity.

A `RunId` identifies its unique root Activation and all uniquely owned child
Activations and Steps under `RunOrder`, the transitive closure of typed
StepCauseFrontier edges and typed configuration-succession edges. The two edge
kinds remain separately inspectable: configuration succession never rewrites a
frontier or inserts an implicit `PriorStep`. Suspension and same-pin
resumption do not add an Activation. A semantic handoff may add one child
Activation; executor relocation alone does not. Cancellation is an occurrence
with exact target and provenance, not a mutable Run flag: every affected
carry-through cites it, so Steps unrelated by either RunOrder edge kind remain
unordered. A Run may include external waits and explicitly nondeterministic branches. It may
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
AdmissionRequest<T : AdmissionTarget> :=
    exact target identity and target kind T
  + exact base/root and target-policy branch or merge coordinates
  + exact canonical candidate commitment
  + exact evidence and occurrence-exact support commitments
  + exact ordered authorization uses
  + exact role-labelled JudgmentOccurrences
  + exact obligation dispositions
  + exact policy and semantics identities
  + exact validity observations
  + exact conflict and successor contract

AdmissionRequestKey<T> = H(
  "clause/admission-request/v1",
  canonicalBytes(AdmissionRequest<T>))

AdmissionOccurrence<T> :=
    fresh nominal authoritative decision identity
  + exact AdmissionRequestKey<T>
  + exact Accepted(exact successor) | Rejected(exact typed disposition)
  + exact direct causal incoming edges required by the request

Γ ⊢ canonical request well formed against its exact base/root
Γ ⊢ evidence, supports, authorization uses, JudgmentOccurrences,
    obligations, policy, validity, semantics, and conflict contract sufficient
Γ ⊢ every authorization basis independently authoritative before decision
Γ ⊢ candidate, proposed successor, and their evidence supply no such basis
─────────────────────────────────────────────────────────────
admit(AdmissionRequest<T>) =
  Cite(existing AdmissionOccurrence<T>) | Decide(new AdmissionOccurrence<T>)
```

The canonical request is the complete content identity of a decision, not the
decision occurrence. Its sequences preserve role, support, authorization-use,
and obligation multiplicity; their canonical order is declared by the target
contract rather than inherited from caller order. Same-key physical delivery,
crash retry, or concurrent replay cites the one existing
`AdmissionOccurrence`; it does not allocate a semantic attempt or make another
decision. Any changed committed input creates another request key. A target may
declare an observable `AdmissionAttemptOccurrence` with its own typed incoming
schema, but retries have no such semantic occurrence by default.

Each Admission target policy declares exactly:

```text
AdmissionTargetPolicy<T> :=
    candidate re-review rule
  + rejection finality and retry/change rule
  + successor topology:
      ExclusiveHead(compare-and-publish exact base)
    | Branch(branch coordinate and predecessor rule)
    | Merge(exact parent set and merge contract)
  + atomic conflict detection and typed winner/loser result
```

`ExclusiveHead` permits exactly one winner for one current base. Branch and
merge acceptance are not weaker authority kinds: they are explicit target
successor policies. Branching therefore belongs to Admission target policy,
never to the Authorization ontology.

Authorization use is independently typed by authorization subtype:

```text
AuthorizationUse<A : Authorization> :=
    exact AuthorizationEvidence<A>
  + exact AuthorizationUseContract<A>
  + exact subtype-specific AuthorizationUseOccurrence<A>
    or declared LinearizationOccurrence<A>
  + exact action/scope and validity observations

AuthorizationUseContract<A : Authorization> :=
    cardinality: Reusable | AtMost(exact bound) | Linear
  + exact subtype-specific use/linearization occurrence kind
  + exact subject/action/scope/policy/semantics coverage
  + exact typed validity observations
  + atomic validate-use-and-publish rule
  + typed conflict and winner result
  + rejection disposition: Preserve | Consume | exact subtype rule
```

Every authorization subtype declares this contract; generic Admission cannot
guess it. Reusable use preserves authority within its exact scope. Bounded and
linear use validates cardinality and publishes the use/decision atomically, so
concurrent contenders receive one typed winner set and typed conflicts rather
than both spending the same authority. Rejection preserves or consumes a use
only as the subtype declares. Revocation, delegation, renewal, and expiry exist
only when that subtype declares their exact occurrences, incoming causal
schemas, validity effect, and conflict rule. Wall time affects authority only
through a typed time observation named by that contract. Authorization remains
a Judgment about permission; Capability remains independent evidence of access
to a boundary or resource. Neither substitutes for the other.

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

Admission cannot bootstrap its own authority. For a successor to an existing
ProgramRevision, a constitutive AdmissionAuthorization must be anchored in the
exact base or another already authoritative ProgramRevision whose declared
scope covers that base, target, action, and policy. An issued authorization must
likewise occur under a basis that is independently authoritative before the
Admission decision. It may be issued after a proposal exists and after an
independent reviewer has inspected candidate and checker evidence; proposal
construction does not receive or consume Admission authority. For a root
ProgramRevision with no authoritative predecessor, the only
constitutive path is an independently established
`IrreducibleRootConstitution`.
Neither a JudgmentRef in the candidate ProgramSnapshot, a JudgmentOccurrence
carried by the candidate, the would-be successor ProgramRevision, nor an
authority claim produced by candidate construction may authorize its Admission.
An independent authority may use candidate/check evidence as the subject of its
review without deriving its authority from that evidence. A self-supporting
cycle rejects before an AdmissionOccurrence or successor is allocated.

An actual authoritative decision at this boundary is the nominal
`AdmissionOccurrence` above. It applies the request's exact authorization uses
and role-labelled JudgmentOccurrences but is neither an Authorization nor a
Judgment itself. It produces one exact successor or typed Rejection and remains
queryable causal evidence. The target revision's constitutional identity
fields remain those declared for that revision kind; request-key equality does
not collapse independently declared successor identity.

The strict governed-per-intent state/effect profile is a causal graph, not one
mandatory total chain:

1. a transition Activation and its Steps stage a candidate State delta and
   effect intents;
2. Admission may accept the State successor and governed intents atomically;
3. a separate `AuthorizationOccurrence<EffectAuthorization>` may issue an exact
   EffectAuthorization Judgment naming one governed intent, action, scope,
   policy, and required capability contract;
4. a separately identified effect Activation closes three distinct Mode slots:
   the governed intent plus its AdmissionOccurrence, that issued
   AuthorizationOccurrence, and independent exact CapabilityEvidence covering
   the boundary/resource/pins/validity/budget; its occurrence-only cause
   frontier projects the intent, Admission, Authorization, and any
   occurrence-backed capability evidence under their exact slot identities;
5. only after all three slots and their causal projection validate may a Step
   produce an EffectAttemptOccurrence;
6. the attempt may cause a ReceiptOccurrence, time out without one, fail before
   a receipt, or later be described by zero or more Observation occurrences;
7. governed JudgmentOccurrences issue exact Judgments over exact evidence; and
8. a later, separate AdmissionOccurrence may record a claim or State
   successor.

The governed-per-intent profile requires admission of intent, issuance of effect
Authorization, effect Activation, and attempt, with receipt optional. The
capability remains an independent prerequisite even when its issuance
occurrence is also causal. Typed occurrence provenance, slot-labelled bindings,
and the Activation cause frontier make that graph checkable without turning a
non-occurrence value into a cause.

A preauthorized profile begins instead from an exact intent occurrence under a
bounded activation-local, session, Lease, or batch scope, plus the previously
issued EffectAuthorization occurrence and independent capability evidence
declared by the Mode. The scope is bound once at its declared boundary and may
cover several bounded attempts; it performs no per-attempt Admission or
Authorization issuance. A statically pinned issued authorization or capability
may erase from the hot ABI but remains an exact semantic slot and explanation.
If a supplied input has an occurrence, that occurrence projects under its exact
slot; non-occurrence capability evidence does not become a cause. The later
attempt, optional receipt, observations, Judgments, and any Admission of an
external claim remain distinct. Observations may describe an attempt, timeout,
receipt, or later external state. Intent, issued Authorization, capability,
Receipt, Observation, Judgment, JudgmentOccurrence, and Admission never
collapse in either profile.

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

Trace retention is an explicit Mode or runtime-profile contract, never an
accidental consequence of identity. The contract fixes the retained projection,
resident byte/record bound, externalization sink when any, acknowledgment and
failure behavior, eviction or compaction rule, and the exact causal evidence
that remains recoverable. It may select no retained trace, a bounded resident
window, or acknowledged externalization. A compact summary may explain history
but cannot by itself authorize a new causal edge: a future Step that cites an
evicted cause must rehydrate its exact checked witness within the declared
budget or reject with a typed unavailable-history obligation. An ongoing Run
therefore need not retain an ever-growing heap merely because its identities or
causal history remain meaningful.

Retirement of a trace projection never retires the Run or rewrites its causal
relation. Conversely, retaining trace bytes does not keep an unrelated physical
object alive. Long-lived profiles must state bounded active-frontier,
continuation, diagnostic, and trace residency independently, including what is
externalized and how failure to externalize affects progress.

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

Clause meaning has three explicit seams. The typed graph distinguishes neutral
`Triple`/`Term` structure, checked ApplicationForms, nominal Applications,
RelationSchemas, revision-indexed RelationExtensions, OperatorRefs, Modes,
identities, contracts, and authority interfaces; a checked process constitution
fixes their exact governing declarations and relations. Transition semantics
governs Activations, Steps, observations, continuations, and Run order under
that constitution. Admission separately governs authoritative successor
formation. The graph is their canonical inspectable carrier and explanation
surface: it must hold every constitutive relationship and admitted boundary
that can affect declared meaning, plus every process relation required by a
declared consumer. Truth status is extrinsic to representation: no Term, graph
node or edge, ApplicationForm, RelationExtension row, or trace is true merely
by existing.

Actual running is not reducible to whichever graph or trace projection was
retained. Conversely, an opaque runtime cannot bypass the graph: every
externally meaningful Activation has recoverable identity, exact constitution
and applicable revision pins, mode, declared dynamic prerequisites,
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

Relations use open-ended, schema-checked named-role bindings, not arbitrary
roles. Open-endedness means schemas may declare new role vocabularies; each
binding is checked against one exact RelationSchema, and an ApplicationForm
closes that schema's required roles under its recorded eligible Modes.

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
relation such as `member-of` with `member` and `group` roles. Clause does
not introduce a primitive `Classifier`, `Set`, or `Type` species merely to
license the group role. Any Referent may occupy that role unless the relation's
explicit contract restricts it. Membership may support a derived category or
collection view; it does not convert the group Referent into another kind.

A structural field or role is not a top-level classification binding. A shape
field such as `x: F32` describes one structural role; it neither emits the
same judgment as top-level `value: F32` nor installs an object field on a
domain Referent. Type, value, object, field,
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

## Parametric abstraction and static constraints

Clause has first-class parametric reuse rather than host-generated families of
nearly identical declarations. The initial constitutional boundary is
deliberately rank 1 and declaration-level: a declaration may introduce one
ordered `StaticParameterTelescope`, and every parameter ranges over a
Clause-owned static domain established by a FormationJudgment. A later
parameter and the declaration body may depend on earlier parameters. A runtime
value cannot smuggle in an unformed type, schema, mode, constraint, or higher-
rank polymorphic value.

```text
StaticParameterTelescope :=
  p₁ : D₁ under F₁,
  p₂ : D₂(p₁) under F₂,
  ...,
  pₙ : Dₙ(p₁ ... pₙ₋₁) under Fₙ

StaticConstraintTelescope :=
  q₁ : C₁ under ConstraintSlotLocalId₁,
  ...,
  qₘ : Cₘ(p₁ ... pₙ) under ConstraintSlotLocalIdₘ

ClosedInstantiationUse :=
  exact snapshot-local declaration and InstantiationLocalId
  + normalized closed static arguments [a₁ ... aₙ]
  + named evidence bindings
      [ConstraintSlotLocalIdᵢ ↦ normalized eᵢ]
  + one exact ResolutionScopeCommitment per obligation
```

A static domain may describe Types, values, RelationSchemas, Modes, effects,
lifetimes, resources, or physical contracts only through accepted Clause
formations. The list is not a host kind switch. Every argument is checked
against its substituted domain in telescope order. Every declaration body and
exported signature is closed relative to its own telescope; every use is
closed by exact arguments and evidence before it can contribute to an
ApplicationForm, executable Mode, layout, or artifact.

Every static-domain formation contract fixes canonical encoding and decidable
equality, total deterministic normalization and substitution, phase isolation,
and either a canonical result or an exact typed non-normalizable rejection.
Static normalization cannot observe RuntimeSession, StateRevision, effects,
ambient host state, source traversal order, or an unbounded host callback.
`ClauseSemanticsId` commits to those formation and normalization rules.

Constraint solving is relative to one finite, canonical `ConstraintBasis`
formed from explicit lexical declarations and imports. There is no ambient
process-global instance registry, filesystem search, host trait lookup, or
source-order preference. Evidence remains an explicit normalized semantic
argument even when canonical source later permits it to be inferred. A finite
basis is not by itself a finite search: every admitted constraint program also
carries a checked finite-resolution or well-founded-decrease contract. An
explicit search budget may instead end only in typed `indeterminate` or
`exhausted`, never masquerade as unsatisfied.

For each named obligation, a `ResolutionScopeCommitment` records every lexical
or imported candidate edge whose presence or absence can affect applicability
or uniqueness as a canonical multiplicity-preserving collection of typed reuse
identities, plus the selected normalized evidence dependency closure. Equal
candidate content appearing twice remains two candidates. The
Clause-owned solver enumerates the complete normalized solution set under the
resolution contract before reporting zero solutions or ambiguity. Adding or
removing a potentially applicable declaration invalidates that obligation;
unrelated declarations outside its committed applicability frontier do not.
A separately compiled body solves only against its recorded scope and evidence
parameters; downstream imports cannot reopen it. Evidence equivalence is
allowed only under checked terminating and confluent coherence rules over every
evidence-observable output. Host order or a cutoff never chooses a
representative. Escaped parameters, nonstatic arguments, rank violations, and
unresolved or cyclic search without a valid resolution contract reject at
their exact formation stage.

Snapshot provenance, semantic checking reuse, semantic specialization reuse,
and physical artifact reuse are different identities:

```text
StaticReuseIdentity :=
    Structural(exact canonical semantic content)
  | Continuous(exact typed continuity identity,
               exact admitted equivalence witness)
  | SnapshotBound(ProgramSnapshotId, exact typed local reference)

ParametricInterfaceId = H(
  "clause/parametric-interface/v1",
  ClauseSemanticsId,
  canonical telescope + exported signature + constraint interface
)

ConstraintSlotId = (ParametricInterfaceId, ConstraintSlotLocalId)

InstantiationUseRef = (ProgramSnapshotId, InstantiationLocalId)

InstantiationKey = H(
  "clause/instantiation/v1",
  ClauseSemanticsId,
  ParametricInterfaceId,
  canonical StaticReuseIdentities for normalized static arguments,
  named ConstraintSlotId bindings,
  exact ResolutionScopeCommitments,
  canonical StaticReuseIdentities for normalized evidence
)

SpecializationSccKey = H(
  "clause/specialization-scc/v1",
  ClauseSemanticsId,
  canonical alpha-normalized SCC graph whose member records contain
    SccMemberLocalId,
    InstantiationKey,
    exact declaration-body semantic content with every intra-SCC call
      replaced by its SccMemberLocalId,
  exact semantic dependencies outside the SCC
)

SpecializationKey = H(
  "clause/specialization/v1",
  SpecializationSccKey,
  exact SccMemberLocalId,
  exact InstantiationKey
)

PhysicalReuseKey = H(
  "clause/physical-reuse/v1",
  SpecializationKey,
  exact AcceptedRefinementWitnessId,
  target + features + runtime profile,
  ABI + layout + strategy,
  exact physical dependency closure
)
```

`InstantiationUseRef` retains exact declaration/use provenance and changes with
the containing snapshot. `InstantiationKey` reuses interface checking across
snapshots only when every argument, evidence value, and resolution candidate
uses `Structural` or `Continuous` identity. A nominal snapshot-local input
without an accepted portable equivalence witness uses `SnapshotBound`, so an
independently allocated equal-shaped value does not reuse another snapshot's
result. `SpecializationKey` invalidates on body or transitive semantic
change; `PhysicalReuseKey` prevents target, compiler, ABI, layout, or strategy
changes from reusing incompatible code. `ArtifactId` remains exact bytes. None
is `ApplicationId`, `ActivationId`, nominal continuity, or authority.
Every portable key preimage uses canonical semantic content or admitted typed
continuity identities and exact content-based resolution commitments, never a
source position or host cache address. Snapshot provenance appears only in an
explicit `SnapshotBound` identity. The exact InstantiationUseRef-to-key mapping
preserves provenance even when a portable reuse preimage intentionally omits
it.

The ProgramSnapshot preimage carries each parameter and constraint telescope,
constraint declaration, basis/import relation, resolution-scope commitment,
and closed use as a canonical local record. Same-snapshot arguments and
evidence refer to typed local instantiation records, never to a post-snapshot
InstantiationKey or ApplicationShapeId. The key hierarchy is an intentional
domain-separated DAG: InstantiationKey feeds specialization, and specialization
feeds physical reuse; no key appears in its own preimage or is inserted into
the snapshot preimage. A static argument may not contain a post-snapshot identity
whose own preimage depends on the key being formed. The checker rejects a
cyclic instantiation/evidence dependency graph before key allocation; this does
not prohibit ordinary recursive calls after static closure.

Recursive semantic specialization is finite by construction. The checker finds
the closed specialization call graph, rejects open static/evidence cycles, and
condenses legal runtime recursion into strongly connected components. Each SCC
receives canonical alpha-normalized local member binders independent of source
order, spelling, snapshot-local IDs, and traversal. Its one
`SpecializationSccKey` commits to the complete multiplicity-preserving member
graph, bodies, instantiations, and external dependency closure; member
SpecializationKeys then select from that object without recursively hashing one
another. A body or edge edit invalidates that entire SCC and its dependents,
while a source-only move preserves it.

After
ProgramSnapshotId exists, each local record resolves to an exact
InstantiationUseRef and its independent reuse keys. Those results are never
inserted back into the same preimage. Resolved ApplicationForms include the
exact use refs, InstantiationKeys, and SpecializationKeys on which their meaning
depends.

Separate compilation publishes the exact parametric signature, telescope,
named constraint telescope, normalization and evidence interface,
resolution-scope commitments, semantic body/dependencies, and any declared
layout/ABI contract needed by consumers. A consumer may check an instantiation
without reading incidental source or host compiler state. Source movement and
unrelated imports preserve the relevant reuse keys; a potentially applicable
constraint edit invalidates the affected InstantiationKey, a body edit
invalidates the affected SpecializationKey, and a physical-input edit
invalidates the affected PhysicalReuseKey.

Substitution, solver normalization, canonical ordering, and `Renameπ` are
equivariant. Renaming explicit declaration and binder identities, substituting
well-formed static arguments, and then solving must produce the canonical
renamed image of solving and then substituting. A host specialization cache or
dictionary address cannot change the result. Semantic diagnostics are typed
`DiagnosticObligation`s over stage, code, exact semantic IDs, obligation slot,
and resolution frontier; SourceMap separately renders spelling and spans.
Source movement preserves the semantic obligation, while Renameπ and
substitution transform it equivariantly rather than pretending its bytes never
change.

Monomorphization, normalized evidence dictionaries, representation erasure,
shared code, and direct physical dispatch are replaceable strategies. A checked
strategy may remove a statically fixed operand from the hot ABI without
erasing its semantic influence: monomorphized code commits that evidence into
SpecializationKey; dictionary code carries dynamically selected normalized
evidence; and complete evidence erasure additionally proves that no declared
semantic or diagnostic distinction depends on it. Shared code may serve
several uses only with checked refinement evidence. In every strategy,
distinct InstantiationUseRefs, Applications, and Activations remain distinct
even if PhysicalReuseKey or ArtifactId is shared. The implementation retains a
cold explanation link from each use through its exact instantiation,
specialization, strategy-specific PhysicalReuseKey, and ArtifactId. Types,
behavior, identity, layout/ABI promises, diagnostics, resources, and support
remain those of the cold parametric semantics. The canonical declaration/use
projection is defined by the syntax authority; additional inference sugar
remains unratified.

## Source projection

Human-readable source is a canonical bidirectional projection, not the
program's identity. Parsing may use a transient lossless CST. Every source
construct elaborates to a nonempty collection of independently identified
semantic emissions and a designated focus. Every block head selects one
declared child grammar before inspecting child semantics; a child receives the
parent focus only when its selected production says how. The child never
guesses a relation from indentation.

The reader chooses a CST production deterministically from explicit head shape
and declared grammar. Elaboration resolves every local designation through the
already selected ElaborationContext to one exact `Designation`, then
selects a declared Reading through that record's ReferentId before child domain
semantics are inspected. Missing or competing resolutions or Readings are
explicit errors. Schema and type checking may reject the resulting candidate,
but may not regroup the CST or reinterpret siblings. Incremental parsing and
recovery therefore depend on syntactic boundaries and exact declared
relations, never successful whole-program inference or raw-spelling dispatch.

Conceptually:

```text
ElaborationResult {
  emissions: NonEmpty<Emission>
  focus: Focus
}

Focus {
  term: Term
  origin: SourceSlice
}

Emission {
  projectedSlot: EmissionSlot
  term: Term
  candidateFormations: FiniteCollection<FormationCandidate>
  stance: Stance
  origin: SourceSlice
}

elaborate(sourceConstruct, ElaborationContext,
          Independent
          | RetainAgainst(exact prior IdentityPlan,
                          explicit ContinuityWitnesses))
  -> ElaborationResult + projected IdentityPlan | Error
```

Each emission is checked and diagnosed independently and retains its own exact
source origin, provenance, and later occurrence identity. Equal emitted Terms
do not deduplicate emissions. One source construct may emit several clauses;
one semantic clause never secretly becomes a list of clauses. For a bare
subject, focus is that subject. For a completed relation, the relation Term may
become focus. A header with a declared open slot may allocate a structural or
nominal focus. Indentation itself never means membership, body, containment,
application, ownership, sequencing, or authority.

For every closed printable source context and checked projection `P`, the
round-trip laws are relative to the projected identity plans:

```text
let P′ = elaborate(print(P), context, Independent)
let α = TypedFreshAlpha(projectionIdentityPlan(P),
                        projectionIdentityPlan(P′))
projectedMeaning(α(P′)) = projectedMeaning(P)

elaborate(print(P), context,
          RetainAgainst(projectionIdentityPlan(P),
                        explicit ContinuityWitnesses))
  preserves exactly the Retain rows whose witnesses validate
print(elaborate(source)) = canonical(source)
```

`TypedFreshAlpha` is the named domain-preserving bijection over independently
fresh identities; it preserves producer, EmissionSlot/RepetitionSlotId,
multiplicity, and causality and is identity on retained/derived identities. An
edit or hot reload obtains continuity only from the exact prior IdentityPlan
and explicit valid `ContinuityWitness` values. The laws separately account for
layout, comments, and source occurrence identity. Stable concept continuity
belongs to the admitted graph, not coincidental source position. Ordinary
source must not expose graph bookkeeping ceremony.

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

For a finite positive ground basis with surviving extensional roots E and
authorized rules R, the derived value set is the least fixed point of
`F(X) = E ∪ { conclusion(r) | r ∈ R and premises(r) ⊆ X }`, starting from E.
An empty-premise authorized rule contributes its conclusion; a cycle without
an extensional or empty-premise anchor contributes nothing. After withdrawal,
old derived claims are not new roots. In particular, `p → q` and `q → p`
cannot preserve either claim after the last independent root for p disappears.
One surviving root does preserve both. Positive closure is separate from
resource-consuming transitions and from explicitly scoped negation.

Mode soundness means that each produced value row satisfies its declared
relation at the exact input binding. Completeness means that every satisfying
row in a declared search scope is produced; it is a separate obligation.
`one`, `maybe`, `some`, and `many` constrain distinct value rows under the
declared value equality, not proof-tree count. Independent assertions,
derivation supports, and emitted occurrences keep their own multiplicity.
An exhausted or partial search establishes neither completeness nor absence.
An overlapping pair of proofs for the same value does not violate `maybe`;
incompatible values do. Physical rule ordering must not resolve that conflict.

Recursive explanations use finite dependency structure with explicit alternative
supports, not enumeration of infinitely many unfolded proof trees. The bounded
bootstrap function `derive_ground_closure` in
`clause:crates/clause-substrate/src/canonical_package.rs` computes this positive
value closure and supplies one finite certificate per discovered claim through
the existing checker. It does not enumerate every support, implement general
source recursion, or prove the source compiler sound.

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
- source Readings and candidate constitutive Authorization Judgment
  declarations, using only local references for declarations in this snapshot;
  those declarations become effective only through an already admitted
  ProgramRevision selecting the snapshot and never authorize that snapshot's
  own admission;
- declaration-level StaticParameterTelescopes, Clause-owned parameter-domain
  FormationJudgments, StaticConstraintTelescopes, constraint declarations,
  finite lexical/import ConstraintBasis relations with finite-resolution or
  well-founded contracts, per-obligation ResolutionScopeCommitments,
  normalized evidence forms, and every closed declaration use in local-
  reference form;
- ApplicationForm records which select one local RelationSchema, one local
  operator, an exact set of eligible local Modes, exact role bindings, context
  requirements, local instantiation-use records, and dependency closure;
  nominal Application records keyed by `ApplicationLocalId`; and independently
  identified AssertionOccurrences or relational content with constitutional
  provenance;
- immutable governed Judgment content and snapshot-carried
  JudgmentOccurrences authored as program content, keyed locally where
  snapshot-scoped; none is an authority source for the candidate that contains
  it;
- definitions, laws, derivation authorizations, invariants, goals, continuation
  and process contracts;
- transition, event, capability, effect, admission, and semantic-policy
  contracts; and
- exported Designations and explicit semantic source or authority
  relations.

Every local reference is typechecked and resolved within that finite preimage.
Canonical local keys are semantic allocation/continuity keys, never source
positions, traversal order, memory addresses, or spellings; canonicalization
orders records by their declared encoding. A reference to an already existing
external snapshot remains an ordinary exact external identity. A reference to
the snapshot under construction must be local. Consequently the preimage
contains none of its own `ProgramSnapshotId`, `RelationSchemaId`, `RoleId`,
`OperatorRef`, `ModeId`, `ApplicationId`, `JudgmentRef`,
`InstantiationUseRef`, `InstantiationKey`, `SpecializationKey`, or
`ApplicationShapeId` values. PhysicalReuseKeys and Artifacts are physical
evidence and are never snapshot-preimage inputs.

It excludes incidental source layout, SourceMap data, formatting, comments,
trivia, local designation spellings, caches, schedules, replaceable derived
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
ModeIds, role bindings, context requirements, InstantiationUseRefs with exactly
their InstantiationKeys and SpecializationKeys, and dependency closure.
PhysicalReuseKeys are excluded because target strategy is not ApplicationForm
meaning. InstantiationUseRefs resolve from
the preimage's local instantiation records after the snapshot hash;
InstantiationKeys and SpecializationKeys derive independently from the
canonical interface, arguments, resolution commitments, evidence, body, and
dependency content defined above. None of those references or keys, that
ApplicationShapeId, or any other external reference derived from the snapshot
is inserted back into the same snapshot preimage. This staged construction
removes self-hashes while separating exact provenance from cross-snapshot
reuse.

Consequently a body-only edit that preserves a parametric interface may
preserve InstantiationKey but changes SpecializationKey and every
ApplicationShapeId whose resolved form depends on that specialization. An
independently nominal equal-looking use in another snapshot does not acquire
the same ApplicationShapeId merely because a portable InstantiationKey can be
reused; cross-snapshot reuse and exact resolved-form identity remain separate.

`ClauseSemanticsId` commits to
canonical Term encoding and equality, normalization, typed identity resolution,
formation, RelationSchema and role interpretation, Application formation,
parametric abstraction, constraint normalization, activation and Step
semantics, local configuration and lifetime rules, modes, continuation,
observation, law and derivation semantics, transition and admission semantics,
and every identity-relevant provenance rule. It is not a compiler build number.
Independent conforming implementations of one semantics epoch must produce the
same bytes and IDs.

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

Designations are structured projection metadata:

```text
Designation
  NamespaceId
  spelling
  ReferentId
  visibility
```

Namespace membership, import, export, visibility, and designation resolution
are explicit checked relations or constraints. They are never inferred from a
spelling prefix or encoded kind, role, or path segment. A local source
designation resolves under one exact ElaborationContext to the structured
`Designation`; semantic consumers then use its exact NamespaceId and
ReferentId, not the source bytes. Every well-formed `Designation.spelling`
excludes U+002F `/`, regardless of whether it originated in unquoted source,
backtick-quoted source, generated candidate data, or a decoded package. The
reader rejects authored `x/y` and `` `x/y` `` before Designation resolution;
candidate formation rejects a forged structured Designation containing `/`
before its ReferentId may participate in identity resolution and before any
RelationSchema, Role, or Operator closure. Text values and opaque Atom or
transport payloads may contain `/`; no such payload is implicitly split or
promoted to a Designation. A future reversible `namespace/local` display may be
ratified only as a SourceMap or diagnostic projection of a valid structured
record; that rendered slash is not its `spelling`. A raw slash-joined string
never crosses elaboration, defines identity or equality, recovers a RoleId or
OperatorRef, selects behavior, or extends to an `x/y/z` kind/role/path
convention.

A proven rename changes the Designation while preserving identity.
Without lineage evidence, delete plus create is the honest result. Exported
Designations are interface content and participate in ProgramSnapshot
identity; local spelling and incidental source layout remain projection
evidence.

## Source and admission boundaries

A `SourceUnit` is authored input. A `SourceMap` relates semantic identities,
occurrences, formations, and diagnostics to SourceArtifactIds, spans, and trivia evidence.
Neither is a Program or authority merely by existing. The typed boundary is:

```text
read(SourceUnit)
  -> LosslessCST + SourceMap

elaborate(LosslessCST, ElaborationContext,
          Independent
          | RetainAgainst(exact prior IdentityPlan,
                          explicit ContinuityWitnesses))
  -> candidate Terms, emissions, formations, application-form candidates,
     declarations, and projected IdentityPlan

check(candidate Terms, emissions, formations, forms, declarations,
      projected IdentityPlan)
  -> checked Terms, FormationJudgments, ApplicationForms, declarations,
     exact EmissionSlots and AllocationJudgments, or exact obligations

propose_change(checked candidate, base ProgramRevision or root,
               ProgramProposalContext)
  -> ProgramChangeOccurrence

form_admission_request(checked ProgramChangeOccurrence,
                       ProgramAdmissionContext)
  -> canonical AdmissionRequest<ProgramRevision>

admit(canonical AdmissionRequest<ProgramRevision>)
  -> Cite(existing AdmissionOccurrence)
   | Decide(AdmissionOccurrence, ProgramRevision | Rejection)
```

`ElaborationContext` owns only caller-selected scope, declarations, imports,
and Designation inputs. The candidate owns its exact semantics epoch and
unchecked Terms, formations, forms, and declarations; SourceMap separately owns
source and proposal spans. Formation checking consumes no policy- or resource-
relative authority; any such inputs belong to the selected Mode's declared
dynamic Activation prerequisites or to Admission. `ProgramProposalContext`
contains only the exact ProgramId, base or root, and nonauthoritative proposal-
construction policy. It contains no occurrence allocator, Admission authority,
Admission policy, or AdmissionOccurrence capability. The exact
`ProgramChangeOccurrenceId` and its provenance are supplied by the generic
occurrence boundary that records the proposal act; candidate content and the
proposal context cannot allocate or authorize that occurrence themselves.

`ProgramAdmissionContext` is consumed only to form the canonical request and
contains the exact ProgramId, base or root, admission target policy, typed
`AuthorizationUse<AdmissionAuthorization>` values, applicable role-labelled
JudgmentOccurrences, obligation dispositions, validity observations, and
AdmissionOccurrence-allocation capability. Each authorization use resolves only from
an already authoritative ProgramRevision or `IrreducibleRootConstitution`, or from a
governed AuthorizationOccurrence issued from such a basis; it is never looked
up in the candidate snapshot being proposed. Revision existence is
lifecycle-neutral.

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

EffectIntentOccurrence, issued EffectAuthorization Judgment and its occurrence,
independent CapabilityEvidence, effect Activation, EffectAttemptOccurrence,
ReceiptOccurrence, Observation, Judgment, JudgmentOccurrence,
AdmissionOccurrence, and admitted external claim are distinct typed objects or
boundaries. Effect evidence names the applicable exact ProgramRevision, RuntimeSession,
observed/base StateRevision, producing Run/Activation/Step, typed occurrence
provenance, and causal frontier. A receipt records an outcome; it does not make
the intended external proposition true. Each effect Mode declares either the
strict governed-per-intent profile or an exact preauthorized profile above. Both
bind three independent semantic slots: exact intent occurrence, issued
EffectAuthorization occurrence, and CapabilityEvidence. The governed profile
additionally binds the intent's exact Admission occurrence. A preauthorized
profile binds a previously issued bounded activation, session, Lease, or batch
scope and may cover several attempts without per-attempt Admission or issuance.
Static ABI erasure never collapses those semantic slots. Both preserve exact
intent and attempt occurrences, project only occurrence-backed values into
causality, and retain distinct optional receipt, Observation, Judgment, and
later Admission stages.
Evidence admission after the act cannot roll it back. Any adapter claiming
atomic State-plus-effect commit must state and prove that stronger boundary
explicitly.

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
- rank-1 static parameter formation, substitution, normalized constraint
  evidence, and closed instantiation;
- context, phase, universe, Mode, policy, authorization, and capability
  formation;
- StaticActivationBasis, Activation-local configuration, Step cut,
  causal-frontier, and continuation protocol;
- candidate-delta and Admission validation;
- immutable revision construction and canonical serialization;
- exact Owned/RegionMember/ForeignManaged root plus Borrow/Lease validation;
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

Every activated physical plan is justified by one versioned translation-
validation judgment for the selected Mode transition system:

```text
RefinementObligations(Mode) := derive independently from the exact Mode's
    determinism/nondeterminism contract
  + totality/partiality/productivity contract
  + pure/state/effect contract
  + finite/streaming/reactive interaction contract
  + termination/progress/liveness/fairness contract
  + result-enumeration and cardinality contract
  + cancellation/failure/resource/latency/bound contracts

OpenSystemRefinementV1 :=
    exact ClauseSemanticsId, ProgramSnapshotId, ApplicationShapeId, ModeId
  + exact semantic transition-system commitment
  + exact physical-plan bytes and target/runtime/compiler/ABI pins
  + state relation R ⊆ PhysicalState × SemanticState
  + contravariant semantic-input/environment representation relation
  + covariant output/observation relation
  + physical-event projection: Tau | exact semantic label
  + typed nominal isomorphism for independently fresh identities
  + resource and latency preorder
  + exact Step, effect-attempt, and Admission linearization map
  + exact derived RefinementObligations(Mode)
  + exact certificate and checker/check route

Γ ⊢ PhysicalPlan ≼₁ SelectedModeTransitionSystem
  iff related initial states
  and every semantically admitted/environment input has its declared
      physical representation
  and every physical output/observation maps covariantly
  and every physical transition is stutter or matches one finite semantic
      transition fragment under the event projection
  and every declared Step/effect/Admission boundary has one exact
      Mode-permitted linearization and no undeclared one
  and all independently derived Mode result, progress, liveness, fairness,
      resource, latency, cancellation, failure, and bound obligations hold
```

This is weak open-system refinement, not universal bisimulation or byte
equality. A Mode that is deterministic, total, and pure requires equality of
its declared extensional result and observations. Any nondeterministic Mode
requires every concrete may-behavior to be contained in allowed semantic may-
behavior plus preservation of its declared must/progress obligations; it
requires complete outcome enumeration only when the Mode promises it. Any
reactive or effectful Mode requires the weak simulation, liveness/fairness its
independent axes declare, and exact Step/effect/Admission linearization where
applicable. Deterministic-effectful, nondeterministic-reactive, partial-pure,
and every other admitted combination therefore receive the conjunction of
their exact Mode-axis obligations rather than being forced into a closed
profile sum. Physical stutter is permitted; an infinite stutter violates any
applicable progress or fairness obligation. A failure to decode, allocate, or
materialize before activation may remain a typed physical rejection and need
not be fabricated as a semantic transition.

Checked lowering chains compose transitively only when adjacent pins and
state/input/output/event relations match and the composed resource/latency
preorder, nominal isomorphism, linearization, progress, and declared bounds are
rechecked. The accepted certificate bytes and checker result derive one
`AcceptedRefinementWitnessId`; `PhysicalReuseKey` binds that exact witness, not
a strategy name or claimed refinement tag. A changed witness, plan, selected
Mode system, pin, relation, projection, or bound invalidates reuse.

CPP1 already places physical-plan allocation outside ProgramSnapshot identity
and removes the earlier magic semantic-Term lookup. The runtime's `New` versus
`Rematerialize` allocation epoch also distinguishes fresh execution from exact
reloading. Those are implemented physical repairs, not proof of this judgment:
the current CPP1 checker validates encoding shape, selected
`ApplicationShapeId`/`ModeId`, role bindings, and exact bytes, while
`ClosedApplicationRuleMachineV1` is only a tag. It does not yet validate
semantic transition adequacy, nominal isomorphism, linearization, progress,
resources/latency, or transitive lowering composition.

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
| observed thread interleaving | never semantic order beyond declared typed frontier or configuration-succession edges |
| thrown string | never an untyped substitute for rejection, cancellation, timeout, exhaustion, or absent evidence |
| missing relation row | never false without an explicit closed-world contract |

## Canonical semantics identity

One canonical `ClauseSemanticsManifestV1` is the complete preimage of a
`ClauseSemanticsId`:

```text
ClauseSemanticsId = SHA256(
  UTF8("clause/semantics-manifest/v1\n")
  || exact ClauseSemanticsManifestV1 bytes)

ClauseSemanticsManifestV1 :=
    exact manifest format/version and canonical-byte contract
  + exact content commitment for this foundation
  + exact content commitment for canonical syntax
  + exact content commitment for architecture
  + exact content commitment for compiler genesis/succession
  + exact selected canonical carrier contract commitments
  + exact semantic checker obligations carried by those authorities/contracts
  + exact required identity/refinement/release metamorphic corpus root
  + exact required authority/causality/grouping negative corpus root
```

The manifest is canonical UTF-8 JSON: no BOM; LF newlines; one final newline;
two-space indentation; lexicographically ordered object keys; arrays in their
declared semantic order; lowercase 64-nybble SHA-256 commitments. The manifest
does not contain its own digest, `ClauseSemanticsId`, path, Git commit, tag, or
publication object. Git objects cannot change the semantic preimage or create
authority, and no publication object is required. Signatures, SBOMs, deployment
facts, packaging ceremony, and provider metadata are outside this semantics
manifest.

`clause:semantics/manifest-v1.json` materializes this smallest manifest,
and `clause:semantics/CLAUSE_SEMANTICS_ID` records the reproducible
derived identifier. Mutable checker/runtime implementation hashes and bounded
capability labels never enter those bytes: a behavior-preserving implementation
edit cannot change Clause semantics.

The current manifest commits to a reviewed document bundle. Consequently even
an explanatory prose edit changes this exact identifier. Unequal identifiers
do not by themselves prove incompatible meanings, just as equal version labels
do not prove compatible meanings. A cross-epoch reuse or transport requires an
explicit checked equivalence at the affected boundary; no such general
compatibility decision procedure is claimed here. Existing identifiers retain
their exact preimages. This distinction does not introduce another manifest or
duplicate the normative semantic facts.

No implementation-artifact or release manifest is implied. Mutable
checker/runtime inventories, capability attestations, Git publication objects,
signatures, distribution facts, and supported-release claims remain ordinary
project state unless one actual consumer requires them; they never enter
`ClauseSemanticsId`.

Universally, every hash-derived Clause identifier or reuse key has exactly one
declared domain-separated canonical byte preimage. Construction retains or can
reconstitute those exact bytes for collision checking. Reusing one digest for
different bytes is typed `HashPreimageCollision` rejection, never equality,
deduplication, overwrite, cache hit, or authority. This law applies to
`ClauseSemanticsId`, `AdmissionRequestKey`, snapshots, shapes, instantiation and
specialization keys, accepted refinement witnesses, physical reuse keys,
semantics manifests, and every later declared hash-derived identity.

## Causal-affine lifetime and reclamation

Semantic identity never implies physical residency. A Term, Application,
Activation, Observation, revision, or trace may remain addressable after one
physical representation is reclaimed, and retaining an identity never licenses
an implementation to retain its entire reachable heap. Conversely, dropping
bytes cannot retire a live semantic obligation.

Every physical allocation used to realize Clause meaning has exactly one
reclamation root:

```text
AllocationRoot :=
    Owned(exact affine owner and allocator)
  | RegionMember(exact RegionId, exact RegionStrategy, region allocator)
  | ForeignManaged(exact adapter, resource identity, and foreign owner)

RegionStrategy :=
    DeterministicRegion(exact reset, capacity, and whole-region closure contract)
  | ManagedIsland(exact finite root set, collector strategy, capacity,
                  work/pause budget, trigger, and overflow contract)

AccessEdge :=
    Borrow(read | write | exclusive, exact alias set, holder, scope)
  | Lease(read | write | exclusive, exact alias set, issuer, holder,
          validity, close/revocation rule, resource budget)
```

`Owned` ownership may move; borrows cannot outlive or mutably alias the owner.
Access compatibility is checked over the root's complete alias set, including
direct owner access: shared reads may coexist; any overlapping write or
exclusive access excludes every other access; and concurrent writes coexist
only when a proof establishes disjoint alias sets. Revocation requests new
quiescence; they do not close an edge until every holder has causally
acknowledged that it can no longer access the root.

Every root has one closure rule. An Owned value cannot move or reclaim, a
Region cannot reset, and a Clause-owned foreign wrapper cannot close or reclaim
while any applicable escape, Borrow, Lease, Continuation, child token,
asynchronous use, foreign use, close obligation, or other declared lifetime
obligation remains. Every dynamic holder must causally acknowledge quiescence
before reset or reclaim; silence, timeout, unreachable host memory, and a
revocation request are not acknowledgments. A `RegionMember` under
`DeterministicRegion` is reclaimed with its exact lexical, Step, Activation,
Run, artifact, or declared physical region only after that universal rule
closes. A `ManagedIsland` is an explicitly selected bounded physical region,
never a default heap: its finite external roots, collection strategy, capacity,
work/pause budget, trigger, and overflow behavior are part of the checked
physical contract. Collection may discover that internal storage is unreachable
only after every semantic escape, access, continuation, foreign-use, and close
obligation for that storage has already closed. A
`ForeignManaged` allocation remains owned and reclaimed by its declared foreign
manager. Clause may own a separate wrapper, but closing that wrapper's Lease
does not claim reclamation of foreign storage.

Borrow and Lease are zero-or-more typed access/obligation edges, never
alternative reclamation roots. A region member or owned allocation may cross a
shared, foreign, asynchronous, or executor boundary under a Lease while
retaining its root. Every Lease fixes access mode, complete alias set, issuer,
holder, validity, scope, close/revocation protocol, and resource budget. Losing
a host reference is not evidence that a Lease closed.

The compiler proves the closure rule; running supplies the causal event that
may satisfy it. Release time therefore need not be a compile-time constant.
Suspension, cancellation, branch selection, dynamic input, external receipts,
and long-lived services may determine at runtime when obligations close. Once
the declared closure condition is met, semantic retirement is deterministic.
Physical reclamation follows only through the selected root allocator or
foreign manager. Mechanical reclaim is effect-free: it allocates no semantic
occurrence and invokes no observable destructor, finalizer, callback, or
effect. A compiler-proven bounded, nonobservable physical drop/deallocation is
permitted, including Rust-style field teardown, when it cannot call user or
foreign code, vary a declared observation, or create an unbounded cascade.
Physical resource accounting may record it; that accounting becomes a semantic
Observation only when the Mode declares the distinction and a Step cut exposes
it. Any observable
`close`, `dispose`, flush, listener removal, or external release is an explicit
Application/Activation/Step and, where it crosses a boundary, an effect with
distinct intent, issued EffectAuthorization occurrence, independent capability
basis, attempt, receipt or typed absence, and applicable Admission under the
selected governed or preauthorized profile.
It must complete or reach its declared failure state before physical reclaim.
Reclamation cannot manufacture that process after the last reference
disappears.

Strong ownership cycles spanning independently reclaimed roots reject,
including Owned-to-Owned and Owned-to-Region cycles. A cycle may instead use a
checked non-owning edge, be dominated by one enclosing `DeterministicRegion`
whose whole-region closure/reset reclaims it without tracing, or be contained
entirely inside one explicitly bounded `ManagedIsland`. Crossing an island
boundary with a strong ownership edge rejects. Clause never silently falls back
to a tracing collector, reference counting, finalizers, or leak-until-process-
exit semantics.

The native/Wasm game profile permits no `ManagedIsland` on its controlled hot
path and no mandatory tracing GC, stop-the-world scan, implicit ARC fallback,
or finalizer-dependent release anywhere in Clause-owned storage. Checked moves,
borrows, leases, region reset, and deterministic teardown are the ordinary
strategies. Unknown lifetime is an exact diagnostic, an explicitly selected
bounded managed island outside that hot path, or an explicit managed foreign
boundary—never a hidden universal heap scan. A foreign runtime may manage its
own storage only behind `ForeignManaged`; that does not introduce a Clause heap
collector or satisfy Clause-owned close and lifetime obligations.

Static ownership, Borrow, lifetime, region-membership, and closed-obligation
proofs erase from a production ABI when the specialization fixes them. Only
dynamically varying configuration ownership, Lease, quiescence, continuation,
or close tokens retain runtime representation. Detailed reclamation receipts
required by the corpus are instrumentation for the declared test/profile, not
a mandatory allocation or tracing tax on every production operation.

Foreign and browser objects remain honest `ForeignManaged` boundaries. A
Clause-owned wrapper carries its Lease and deterministic explicit-disposal
obligation; the browser's garbage collector may reclaim its own unreachable
representation but cannot satisfy Clause teardown, listener removal, receipt,
or resource contracts merely by existing. A disposal receipt proves only the
declared adapter action, not the instant at which a foreign heap reclaims
storage.

The bounded frame profile declares initialization capacities for hot state,
scratch regions, Wasm memory, transport buffers, renderer pools, active causal
frontier, continuations, and trace projection. Partial initialization publishes
no view or handle and rolls back all Clause-controlled state transactionally.
Already attempted foreign allocations follow their exact cleanup protocol;
cleanup success, failure, or pending quarantine remains explicit, and no
atomic external cleanup is claimed. `capacity + 1` rejects before any
avoidable foreign allocation. After initialization, the Clause/Wasm/adapter-
controlled frame path performs no allocation, `memory.grow`, whole-carrier
clone, global heap scan, observable destructor/finalizer, or unbounded drop
cascade. It
updates checked disjoint owned regions in place and resets bounded scratch
deterministically.

Every ratified foreign call has an allocation and disposal contract. A stronger
claim that the entire browser, driver, or foreign heap allocates nothing
requires instrumented target evidence covering warm-up, lazy caches, callbacks,
and disposal; Clause does not infer it from silence. The frame receipt records
controlled allocation calls/bytes, pool high-water marks, Wasm pages, adapter
calls, and resource-ledger state before and after. Deterministic Lease closure
is reported separately from foreign-heap reclamation.

An admission-free simulation or render frame is fresh by exact `RunId`,
`ActivationId`, producing `StepId`, and `ObservationId`; it may additionally
carry an unchanged `Wbase` when it observed admitted world state. Only a frame
actually projected from an admitted boundary is required to name that
`StateRevisionId`. Frame progress never manufactures StateRevisions merely to
prove freshness. A long-run frame or service fixture must also remain inside
its declared resident configuration, active-frontier, continuation, and trace-
retention bounds after many multiples of capacity; compaction or
externalization follows the declared contract and never invokes hidden GC.

## Acceptance laws

The adoption spike and any migration must prove at least these cases:

| Case | Required result |
| --- | --- |
| Canonical print/re-elaboration runs independently with no prior IdentityPlan | Retained/derived identities intrinsic to the projection are exact; independently fresh identities are related by the named `TypedFreshAlpha` between the two `projectionIdentityPlan` values; source spans may differ |
| Edit or hot reload claims continuity | It supplies `RetainAgainst(exact prior IdentityPlan, explicit ContinuityWitnesses)`; only validated witnesses produce `Retain`, while absence of the prior plan or witness produces `Fresh` or typed rejection as the domain declares |
| A repeated equal emission is inserted between retained equal emissions | Under `RetainAgainst`, retained emissions keep their exact `RepetitionSlotId`; the inserted emission receives a fresh one; canonical encoding ordinals may change but establish no continuity |
| The same recorded occurrence is reloaded | Observe its exact `AllocationJudgment` and identity bytes; allocate nothing fresh |
| Equal source and causal content independently create two fresh occurrences | Distinct identities and allocation judgments; compare through typed nominal isomorphism rather than equal bytes |
| Fresh allocation cites a span, position, traversal order, caller/random bytes, UUID, handle, or physical address, or a root cites itself | Reject the allocation basis before publication |
| Two allocation judgments or canonical preimages claim one identity/digest | Typed identity/hash-preimage collision rejection; publish neither conflicting construction |
| The identical canonical AdmissionRequest is delivered or physically attempted twice | One `AdmissionRequestKey` and one authoritative `AdmissionOccurrence`; the retry cites it and creates no attempt occurrence unless the target explicitly declares attempts observable |
| Candidate, evidence, support multiplicity, authorization use, Judgment role, obligation disposition, policy, semantics, validity observation, or conflict contract changes | A different `AdmissionRequestKey` and a separately policy-governed decision |
| Two concurrent requests use one reusable authorization inside its exact scope | Both uses may validate; each retains its exact subtype use/linearization occurrence and neither becomes a Capability |
| Concurrent requests exceed an `AtMost(n)` authorization or contend for one `Linear` authorization | Atomic validate-use-and-publish selects at most the declared winner count; every loser receives the subtype's typed conflict and rejection disposition |
| A policy mentions no revocation, delegation, renewal, expiry, or wall-time observation | None is invented; wall time changes no validity |
| An authorization issuance depends on a candidate whose Admission depends on that authorization | Reject the heterogeneous causal cycle before either node publishes |
| A Step in Run B consumes an output causally produced by Run A through a declared boundary | The heterogeneous dependency orders the nodes in `CausalOrder`; it creates no cross-Run `RunOrder` edge |
| Two nodes are unrelated but encoded, registered, logged, or received in a fixed order | They remain incomparable in `CausalOrder` and every `RunOrder` |
| A Step consumes configuration produced by an earlier Step without a frontier citation | Its configuration-succession edge is included in `RunOrder` and embeds in `CausalOrder` |
| One Mode boundary atomically emits a declared pair | One Step instance with two distinct schema-labelled output slots; splitting it into two Steps changes Mode meaning |
| One StepBoundarySchema emits batches of different allowed sizes across instances | Every instance cites the same boundary ref; values occupy stable output slots/RepetitionSlotIds and satisfy declared cardinality without creating schemas per value |
| A runtime inserts an allocation safepoint, scheduler yield, or progress poll not declared by the Mode | No StepBoundaryRef, Step, StepId, or causal edge is created |
| Deterministic total pure physical plan validates | Related initial states, a declared physical representation for every semantically admitted/environment input, weak finite matching, and extensional equality of every declared result/observation hold under exact pins |
| Nondeterministic physical plan validates | Concrete may-behavior is contained in allowed semantics and declared must/progress obligations hold; outcome equality/enumeration is required only when the Mode promises it |
| Reactive/effectful physical plan validates | Weak simulation plus Mode liveness/fairness and exact Step/effect/Admission linearization hold; infinite tau stutter cannot satisfy declared progress |
| CPP1 plan carries `ClosedApplicationRuleMachineV1` and passes current shape/Mode/role/byte checks | It remains an unproved physical candidate until an `OpenSystemRefinementV1` witness is accepted; the tag grants no reuse identity |
| Compiler0/Compiler1 host-freeze evolution changes only package-declared meaning under unchanged hosts | The exact checked refinement and host-mechanics evidence must still pass; a host semantic edit or unproved refinement rejects the freeze claim |
| Two isolated producers use identical committed files and canonical manifest bytes | Identical `ClauseSemanticsManifestV1` bytes and `ClauseSemanticsId`; Git objects are outside the preimage |
| Identity-relevant authority, carrier/checker contract, or required corpus root changes | Different `ClauseSemanticsManifestV1` bytes and `ClauseSemanticsId`; same digest with different bytes is typed collision rejection |
| Only a checker/runtime implementation or bounded capability label changes | Same `ClauseSemanticsId`; no additional artifact identity is manufactured |
| Same structural Triple constructed twice | Same Term; no Application, assertion, or execution implied |
| Equal Terms used by independent source or assertion occurrences | Equal content; distinct occurrences |
| Closed form compared as one exact resolved form | `ApplicationShapeId` binds `ClauseSemanticsId`, exact RelationSchemaId, OperatorRef, eligible ModeIds, roles, context requirements, exact InstantiationUseRefs with InstantiationKeys and SpecializationKeys, and the exact resolved semantic-dependency/declaration closure; PhysicalReuseKey is excluded and an open form has no shape ID |
| Equal-shaped ApplicationForms independently instantiated without continuity evidence | Distinct ApplicationIds |
| Checked candidate contains a unique ApplicationLocalId and valid closed form | Its ApplicationId resolves from that exact ProgramSnapshot without requiring a fabricated ProgramRevision; Admission is required only to select it into an authoritative constitution |
| Snapshot-scoped declarations, instantiations, and forms are hashed | The canonical local-reference preimage contains no identity derived from its own ProgramSnapshotId; exact InstantiationUseRefs and ApplicationShapeIds resolve only after the one snapshot hash, while portable reuse keys derive from independent canonical content inputs and an unportable nominal input produces an explicit SnapshotBound key; none re-enters the preimage |
| Already authoritative base ProgramRevision declares an in-scope AdmissionAuthorization | Its exact ProgramRevision/JudgmentRef pair may supply the constitutive authorization for a successor; candidate content still supplies none |
| Candidate snapshot declares the AdmissionAuthorization that would admit itself | Reject before allocating an AdmissionOccurrence or successor; only an already authoritative ProgramRevision or `IrreducibleRootConstitution` can supply the constitutive basis |
| Constitutive authorization pairs a JudgmentRef with a ProgramRevision selecting another snapshot | Reject the mismatched authority anchor even when the Judgment content is equal |
| Proposal is constructed before independent Admission authority is issued | Accept the proposal as nonauthoritative; an independently rooted reviewer may issue authority before the Admission decision, while candidate/successor self-support still rejects |
| Issued authorization cites itself, the candidate action, or candidate-produced authority as its issuance basis | Reject the authorization cycle before the authorized action acquires authority |
| One exact Application independently root-activated twice | One ApplicationId; two distinct ExternalTriggerOccurrenceIds, ActivationIds, and Run roots |
| A parent Step starts a child Activation | The child has a fresh ActivationId, inherits exactly the parent's RunId, and cannot also root or join another Run |
| Normal first Step of a Ready Activation | Its frontier is exactly `{ActivationStart(its own ActivationId)}` |
| Ready Activation is cancelled before ordinary carry-through | Its sole first-Step exception has exactly `{ActivationStart(its own ActivationId), CancellationRequest(c)}` and the checked outcome is the matching `Cancel(c)` |
| First Step contains another Activation's start, an extra cause outside the ready-cancellation pair, mismatched cancellation target/pins/Mode/occurrence/outcome, consumed initial custody, or follows an existing Step | Reject before StepId allocation; no first-Step exception beyond the exact ready-cancellation pair exists |
| Fresh two-branch Join consumes both exact settlements | Its canonical settlement sequence is BranchSlot-ordered, and its frontier contains the two exact settlement-terminal PriorSteps independent of physical completion or trace serialization order |
| Later Step names itself, a future Step, a cyclic cause, or a cause with the wrong Run, Activation owner, or occurrence kind | Reject before StepId allocation; the previously constituted RunOrder DAG remains unchanged |
| Step `s2` consumes `ConfigurationAfter(s1)` without `PriorStep(s1)` | Accept when every other contract holds; the typed configuration-succession edge establishes `s1 <run s2` without changing either StepCauseFrontier or inserting an implicit PriorStep |
| Nonfirst Step has an empty StepCauseFrontier and consumes no predecessor custody | Reject before StepId allocation; every nonfirst Step requires nonempty IncomingRunEdges, and an empty frontier is permitted only when its transition contributes a configuration predecessor |
| StepId is allocated for a validated carry-through | One fresh nominal StepId; its StepBoundaryRef/schema, exact owner, finite frontier, permitted StepConfigurationTransition, and schema-labelled outputs live in the associated StepRecord rather than in the identifier |
| One Activation progresses, suspends, and resumes | One ActivationId and Run membership; several StepIds and configurations; the takeup cause names the exact Continuation, emitting Run/Activation/Step, and ResumptionOccurrence without a duplicate PriorStep edge |
| Continuation takeup has stale pins, the wrong owner or target, or repeats a consumed linear use | Reject before Step allocation; semantic handoff with changed pins must create a child Activation through `HandoffFrom` |
| An executor handoff preserves all semantic pins | Same ActivationId and Run membership; the takeup Step names the Continuation and HandoffOccurrence |
| A semantic handoff changes Application, Mode, or a semantic pin | Fresh child ActivationId in the same Run through an exact HandoffFrom cause whose parent Step is the Continuation's exact emitter; destination basis/pins, HandoffOccurrence target, and well-founded provenance validate before allocation, and the original Activation never changes identity or pins |
| Handoff Continuation emitter and direct same-Run HandoffOccurrence provenance root are distinct Steps | The child's ActivationStart projects both exact predecessors plus ordinary same-Run Activation occurrence ancestry as a distinct union; a coincident root appears once, while a missing, future, cyclic, or wrong-Run root rejects before allocation |
| A cancellation races an independent Step | Only Steps whose typed cause frontier names the CancellationOccurrence are ordered after it; unrelated Steps remain unordered |
| Step cites cancellation for another Activation or Run | Reject before Step allocation; Run-targeted cancellation applies only to already owned members of that Run |
| Independent Steps are serialized in a log | No Run ordering unless a typed frontier edge or typed configuration-succession edge relates them; storage order alone contributes neither |
| Two equal-shaped nominal transfer configurations are independently established | Distinct ApplicationIds; every actual transfer event also has a distinct OccurrenceId plus internal producing Activation/Step/Run identity or exact external-boundary provenance |
| Same expression and value | Expression Term and evaluated value remain distinguishable |
| Structurally different Terms have equal behavior | Distinct structure; explicit denotational-equivalence judgment |
| A trace is replayed | Historical effect does not recur merely because its trace is read |
| Same admitted snapshot reached from different parents | Same snapshot identity; different revision identities |
| Same parent and snapshot, different genuine change occurrences | Same snapshot; different revisions |
| Same revision checked by two verifiers | One revision; two attestations |
| Source moves without semantic-source change | Same ProgramSnapshotId and semantic identities; SourceMap changes only |
| Authored unquoted designation `x/y` | Reader rejects before Designation resolution; no identity, schema, role, operator, or behavior is recovered from the bytes |
| Authored backtick-quoted designation `` `x/y` `` | Reader rejects before Designation resolution; quotation cannot bypass the `Designation.spelling` constraint |
| Forged structured Designation whose `spelling` contains `/` | Candidate formation rejects before its ReferentId is used and before RelationSchema, Role, or Operator closure |
| Text value or opaque Atom/transport payload containing `/` | Valid under its own declared contract; never implicitly split, resolved, or promoted to a Designation |
| Local rename with explicit retention | Same ReferentId and ProgramSnapshotId; changed local designation projection |
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
| Checked candidate is activated for sandbox, compiler, test, query, or simulation work | StaticActivationBasis names exact checked package bytes and ProgramSnapshotId; it may read an exact admitted world, persist nonauthoritative output/Continuation, or use an inert effect simulator, but no ProgramRevision, RuntimeSession, StateRevision, real EffectAttemptOccurrence, or constitutive authority is fabricated |
| Checked candidate cites one of its own Authorization declarations as constitutive basis | Reject; only an admitted ProgramRevision selecting that snapshot or `IrreducibleRootConstitution` can make the declaration effective |
| Activation joins an authoritative RuntimeSession, proposes authoritative world change, relies on Program constitutive authority, or performs a real effect under admitted constitution | Its CheckedConstitutionBinding is the exact admitted ProgramRevision selecting the Application's ProgramSnapshot, with all required session/world/policy pins; a checked candidate may still read an exact admitted world through ReadOnlyAdmittedWorld without acquiring those powers |
| Mode declares an empty DynamicPrerequisiteSchema | A valid StaticActivationBasis plus exact InitialContext and causal origin may activate it; no synthetic AuthorizationEvidence, capability token, binding, or runtime governance check is created |
| Mode declares dynamic prerequisites | Every stable named/RoleId-indexed slot and repeated-value ordinal binds exactly once at the boundary where it may vary; missing, extra, stale, wrong-slot, or multiplicity-collapsed evidence rejects before Activation |
| Two prerequisite slots bind equal content or the same occurrence | Both slot-labelled bindings remain distinct; only schema-declared occurrence components project to the cause frontier, with dependency multiplicity preserved by slot, repeated-value ordinal, and CauseComponentLocalId |
| Constitutive Authorization or a non-occurrence capability value satisfies a prerequisite | It remains in DynamicPrerequisiteBindings and creates no causal edge merely by satisfying the slot |
| Constitutive execution basis and covered pins are statically fixed | A checked specialization may erase that proof from the hot ABI while retaining exact artifact-to-basis explanation; issued/effect/Admission authority remains present at its dynamic boundary |
| Pure evaluation or rejection | No ProgramRevision or StateRevision is created |
| Local builder or frame loop performs many anonymous reductions | One affine ActivationConfiguration may update in place; only declared Step cuts receive StepIds, and no StateRevision or Admission is manufactured |
| Pure Mode uses an in-place local builder | Accept only when it is observationally equivalent to the functional realization and no local reference, effect, delta, authority use, or undeclared resource/diagnostic distinction escapes |
| Internal reduction fails or observes cancellation before its Step cut | The exact consumed ConfigurationCustody_before selected by Serial, Split, Branch, or Join is restored through infallible-after-write execution, bounded undo/shadow state, or unpublished private realization, with no duplicate or residual custody; escaped resources are never falsely rolled back |
| Two concurrent computations request the same mutable local slot | Reject the alias or require an explicit disjoint split/lease and causal join; physical interleaving grants no shared ownership |
| Parallel configuration split, branch cancellation, and join | One Mode-owned multiplicity-aware contract proves pairwise-disjoint exact coverage; atomic SplitFormation co-forms the split Step, SplitInstance, one fresh ChildOf/ChildIn Activation and initial token per BranchSlot, and all bindings, every branch produces one exact BranchSlot-bearing settlement, and Join consumes one settlement per BranchSlot in canonical order while citing every settlement-terminal Step |
| Split retains an implicit whole-parent token or omits a parent remainder | Reject; the Split leaves no whole token and every retained remainder must be an explicit branch in the exact-coverage contract |
| Split child, origin, Run membership, BranchSpec, binding, or initial token fails validation | Publish no split Step, SplitInstance, child Activation, binding, or token; retain the live unconsumed parent custody |
| Repeated BranchSpecs share one BranchKey | Contiguous canonical repeated-spec ordinals form distinct BranchSlots carried without collapse through child bindings, tokens, Returned or Closed settlements, and Join; a wrong, missing, duplicate, or noncontiguous ordinal rejects before publication |
| Equal-content branch token or settlement is transplanted across SplitInstances, contracts, BranchSlots, or child Activations | Reject before custody changes; structured split ancestry, not content equality, controls use |
| Branch partition overlaps, omits coverage, or changes BranchSlot multiplicity | Reject the Split before its Step, instance, child, binding, or token set is published |
| Join receives a missing, extra, duplicate, wrong-BranchSlot, wrong-contract, wrong-SplitInstance, or already consumed settlement | Reject before Join Step publication and retain the previously valid branch custody state |
| Closed settlement leaves an exact AllocationRoot (Owned, RegionMember, or ForeignManaged, including a Clause-owned foreign-wrapper obligation), Borrow, Lease, Continuation, effect obligation, or close obligation neither discharged nor transferred exactly as declared | Reject closure; cancellation or missing return value cannot erase unresolved branch custody, while an exact declared transfer may preserve the resource under its destination owner |
| Resumed branch uses another split/contract or repeats a consumed takeup | Reject before the resumed Step acquires branch custody |
| Join frontier omits or adds a settlement-terminal Step | Reject; it must contain one exact PriorStep per consumed settlement terminal, with any additional cause separately Mode-permitted |
| First Step of a ChildOf or HandoffFrom Activation has no configuration-succession edge from its parent Step | Its ActivationStart cause projects the distinct exact same-Run ancestry union—ChildOf parent or HandoffFrom Continuation emitter, direct handoff-occurrence provenance roots, and ordinary Activation occurrence ancestry—into typed StepCauseFrontier edges, so each predecessor orders the child without an inserted PriorStep |
| Equal branch work completes in opposite physical orders | A typed ScheduleIsomorphism maps every fresh run-local identity so `encode(π(runA)) = encode(runB)`; only schedule-independent payload/Value/Result/delta/Admission/continuation-disposition projections are literally equal, and identity-bearing PriorSteps or settlements are not |
| Mutable work forks or rolls back | A semantic fork consumes one token into disjoint fresh child identities and a join contract; private speculative alternatives publish at most one successor; rollback is bounded restoration inside one unpublished Step attempt and cannot erase a completed occurrence, effect, or revision |
| Suspension captures affine configuration | The sole ownership token moves into the Continuation; same-Activation takeup consumes it once, while reusable takeup requires immutable/Copy state or fresh child Activations |
| Continuation is persisted or handed off with a host pointer, executor-local Borrow, nonportable Owned resource, or nontransferable foreign Lease | Reject before the configuration token moves; exact portable/rematerializable roots and acknowledged transferable Leases are required |
| Step-local value is returned, retained, or placed in a Continuation without stabilization | Reject the escape at the exact lifetime boundary |
| Admission-free frame follows another frame | Freshness is keyed by exact Run/Activation/Step/Observation identity and optional unchanged Wbase; no fake StateRevision is created |
| Rank-1 declaration is instantiated twice with equal interface, arguments, named obligations, resolution scopes, and evidence | Same InstantiationKey; same specialization only when body and transitive semantic closure also match; nominal use refs, Applications, and Activations remain exact and independently identified |
| Equal-shaped nominal static arguments arise independently without a portable equivalence witness | Distinct SnapshotBound InstantiationKeys; explicit accepted continuity or structural identity is required for cross-snapshot reuse |
| Constraint resolution reaches zero or multiple incoherent solutions | Report unsatisfied or ambiguous only after complete normalized enumeration under the checked resolution contract; a budget cutoff reports typed indeterminate/exhausted |
| A potentially overlapping constraint candidate is added | Every affected ResolutionScopeCommitment and InstantiationKey changes; an unrelated edit preserves the exact checking, specialization, and physical cache sets it cannot affect |
| Parametric declaration is separately compiled and source moves | Exact interface, telescopes, resolution commitments, evidence interface, and content-derived reuse keys remain stable when semantic dependencies are unchanged; InstantiationUseRef remains exact snapshot provenance |
| Closed self- or mutually recursive generic specializations form | Canonical alpha-normalized SCC records produce one finite SpecializationSccKey and exact member SpecializationKeys; source movement preserves them, while a member body/edge edit invalidates the entire SCC and dependents |
| Static arguments or constraint evidence depend cyclically on their own post-snapshot/reuse identity | Reject before key allocation; SCC construction applies only after static closure to legal runtime-recursive specialization |
| Interface-stable generic body changes | InstantiationKey may remain; SpecializationKey and every dependent exact ApplicationShapeId change; target PhysicalReuseKeys change transitively |
| Renameπ and well-formed static substitution are composed in either order | Canonical instantiation, evidence, typed DiagnosticObligation, and observations are equivariant; SourceMap-rendered spelling/span changes separately |
| Monomorphized, dictionary-passing, irrelevant-evidence-erased, and shared-code strategies implement one specialization | Same declared semantics, diagnostics, observations, and support; strategy-specific PhysicalReuseKeys and cold explanation links remain exact, and shared code never collapses InstantiationUseRefs, Applications, or Activations |
| Owned, RegionMember, or Clause-owned foreign-wrapper obligations close dynamically | Deterministic retirement and bounded effect-free reclamation occur at the proven causal boundary even when its wall-clock time was not statically known; fixed lifetime proofs may erase from the ABI |
| Region reset or owner deallocation would invoke observable destructor/finalizer behavior | Reject the lowering; observable close/dispose must already have run as an explicit process/effect, while bounded compiler-proven nonobservable mechanical drop remains permitted |
| Strong ownership cycle spans independently reclaimed roots, including Owned↔Owned | Reject or require an explicit non-owning edge, one enclosing DeterministicRegion whose whole-region closure reclaims it, or containment wholly inside one explicitly bounded ManagedIsland; cross-island strong edges and hidden tracing/ARC/finalizer fallback reject |
| Overlapping Borrow/Lease access or unacknowledged revocation remains | Reject move/reset/reclaim; shared reads alone may overlap, writes require disjointness or exclusivity, and revocation closes only after causal quiescence acknowledgment |
| Native/Wasm game frame runs after initialization | Clause/Wasm/adapter-controlled allocation is zero; no memory.grow, global scan, whole-carrier clone, observable destructor/finalizer, or unbounded drop cascade; foreign calls obey declared contracts and any browser-wide zero-allocation claim requires instrumented evidence |
| Foreign initialization fails after an external allocation attempt | Publish no view/handle and roll back Clause-controlled state; record cleanup success, failure, or pending quarantine explicitly rather than claiming atomic external cleanup |
| Long-running bounded-profile service exceeds many trace windows | Resident configuration, active frontier, continuations, and trace stay within declared bounds; exact externalization/compaction occurs or progress rejects typed, with no hidden GC or loss of authority evidence |
| Intentionally ongoing service | Remains live or suspended without manufacturing a terminal result |
| Nondeterministic or reactive Run | Cardinality, ordering/fairness, continuation, cancellation, and bounds follow the declared mode |
| Program changes during a live Activation | The Activation remains pinned; only explicit migration or handoff changes its constitution |
| World changes during a live Activation | Each world-sensitive Step names its exact observed/base StateRevision; no silent rebinding |
| Candidate delta and continuation are both emitted | They remain independent; admission consumes only the delta-side inputs and leaves the continuation as a separate process output |
| RelationSchema exists without an OperatorRef | Checked bindings, relational rows, assertions, and patterns may form; no ApplicationForm forms implicitly |
| RelationSchema exists without a Mode | It remains queryable/inspectable but cannot activate |
| User-defined algebraic data and exhaustive match | Clause-authored declarations and process definitions accept the exhaustive case and reject missing/unreachable cases exactly; no kernel feature case is added |
| Forming or evaluating proposition content | Creates no assertion occurrence or truth Judgment |
| Governed-per-intent effect is activated | Three independent slots close: exact governed intent plus Admission, issued EffectAuthorization, and exact CapabilityEvidence; every occurrence-backed component projects under its exact slot before an attempt |
| Preauthorized local/session/Lease/batch effect performs several attempts | Three distinct slots bind the exact intent, previously issued EffectAuthorization, and independent CapabilityEvidence; the bounded scope is established once, no per-attempt Admission or issuance is manufactured, and checked ABI erasure does not collapse a semantic slot |
| Effect running omits or transplants a prerequisite required by its selected profile | Missing/unadmitted-when-governed/stale/wrong-intent input; missing/constitutive-instead-of-issued/stale/wrong-scope Authorization; or missing/expired/wrong-boundary/resource/pin capability rejects before the affected EffectAttemptOccurrence, without imposing governed-only Admission on a preauthorized profile |
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
- no admission-free affine local-configuration tier for ordinary mutable work,
  or local state that can escape, alias, or race without an exact contract;
- mandatory runtime Authorization evidence for a Mode whose dynamic
  prerequisite set is empty, or erasure of issued/effect/Admission authority
  where it may vary;
- inability to express reusable rank-1 parametric declarations with coherent
  finite constraint evidence and separate compilation without host semantics;
- a mandatory tracing collector, implicit ARC/finalizer fallback, or semantic
  identity that forces physical residency in the native/Wasm game profile;
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
> closure. Static formation and executability make ordinary code callable;
> dynamic authority exists only where a Mode declares it. Affine local
> configuration runs without Admission, while Admission alone creates an
> authoritative successor. Relations constrain and expose admissible
> Applications and Runs. Observations report what running distinguished. Terms
> and the Clause Graph are the neutral, recursive, inspectable carrier of
> checked constitution and declared process relations. Parametric declarations
> and causal-affine lifetimes remain Clause meaning; physical execution refines
> and may erase or specialize them aggressively. Typed constitution, explicit
> transition semantics, and separate Admission authority. No hidden host
> language.**
