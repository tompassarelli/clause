# Process-first compiler Terms

> **Status:** candidate process-v1 Terms contract. It is not an admitted
> ProgramSnapshot, compiler package, materialization contract, physical plan,
> or supported runtime.
>
> **Authority:** clause:docs/foundation.md defines Clause semantics and
> clause:docs/syntax.md defines the accepted source projection. This document
> narrows those rules for the position/radius compiler Terms specimens in
> clause:test-vectors/compiler-terms/.

## Decision

The compiler Terms boundary preserves three separate layers:

1. a canonical local-reference process-constitution slice containing neutral
   Terms, declarations, FormationJudgments, laws, ApplicationForms, nominal
   Applications, and candidate constitutive Judgment declarations;
2. occurrence evidence from actual Activations, Runs, Steps, observations, and
   derivation supports, all pinned to an already authoritative external
   context; and
3. non-authoritative refinement requirements which a later physical contract
   and plan must satisfy.

The first layer may contribute to a candidate ProgramSnapshot. The second is
runtime evidence and is excluded from that snapshot preimage. The third states
obligations only. Nothing in this tranche performs Admission, creates a
ProgramRevision or StateRevision, or establishes the unknown graph-to-physical
binding.

## Representation never implies running

The representational kernel remains:

~~~text
RawTriple := [Term, Term, Term]
Term      := Atom | RawTriple
~~~

All three RawTriple positions are structurally neutral. In particular, the
middle position is not inherently an operator, predicate, relation, mode, call,
or control edge. Parsing, constructing, nesting, interning, or transporting a
Term does not form an ApplicationForm, allocate an Application, activate it,
assert it, or authorize anything.

A FormationJudgment supplies the interpretation needed for an application:

~~~text
FormationJudgment {
  subjectTerm,
  formedType,
  interpretation,
  exactDependencyClosure
}

ApplicationForm {
  term,
  RelationSchemaId,
  OperatorRef,
  eligibleModeIds,
  namedRoleBindings,
  pointComponentProjections,
  contextRequirements,
  exactDependencyClosure
}
~~~

Formation checks all of the following before a closed ApplicationForm exists:

- the selected RelationSchemaId and OperatorRef resolve in the exact
  ProgramSnapshot;
- every eligible ModeId belongs to that OperatorRef, names that exact schema,
  permits the known/produced-role orientation, and satisfies the static
  context;
- every required RoleId is bound with the declared cardinality, no undeclared
  role is present, and produced-role placeholders are permitted by every
  eligible mode;
- any scalar consumed by an operator graph is produced by an explicit typed
  component projection from an already bound structured value;
- the context requirements and semantic dependency closure are complete; and
- the eligible mode set is exact, not a preferred mode or a runtime search
  hint.

When an operator definition is itself a generic application graph, dependency
closure includes every inner Term, FormationJudgment, ApplicationForm, and
nominal Application as well as every referenced schema, operator, and mode.
The outer form cannot outsource that closure to a host traversal convention.

An open or malformed candidate remains a Term plus failed formation evidence.
It has no ApplicationShapeId. An eligible-mode set may be empty, leaving a
closed form inspectable but unable to activate.

Every nominal Application instantiating a checked form receives one
ApplicationId. Equal ApplicationForm content does not merge independently
allocated Applications. ApplicationShapeId is structural comparison evidence,
never nominal continuity.

## Local declarations and exact external references

Candidate snapshot content uses typed local references:

~~~text
RelationSchemaLocalId
RoleLocalId
OperatorLocalId
ModeLocalId
ApplicationLocalId
JudgmentLocalId
PremiseOccurrenceLocalId
PremiseSlotLocalId
VariableLocalId
LawLocalId
~~~

No record in the candidate preimage contains an identity derived from the
ProgramSnapshotId being computed. Once the exact canonical checked preimage
has one ProgramSnapshotId S, its external declaration references resolve as:

~~~text
RelationSchemaId = (S, RelationSchemaLocalId)
RoleId           = (RelationSchemaId, RoleLocalId)
OperatorRef      = (S, OperatorLocalId)
ModeId           = (OperatorRef, ModeLocalId)
ApplicationId    = (S, ApplicationLocalId)
JudgmentRef      = (S, JudgmentLocalId)
~~~

The JSON fixtures use visibly fixture-scoped opaque IDs to make every reference
exact without claiming production Clause package hashes. Those IDs are test
coordinates, not names from which a host may infer behavior and not evidence
that the candidate snapshot was admitted.

The Clause source projections use local slash-free Designation spellings.
Namespace membership is carried separately by an exact structured binding:

~~~text
SourceDesignationBinding {
  NamespaceId,
  spelling,
  ReferentId,
  visibility,
  origin
}
~~~

The binding table in `context.json` contains 468 collision-free aliases for the
semantic-package projection, nine for the position/radius source, and eleven
designations that exist only in the fixture context. Hyphens in an alias are
ordinary spelling bytes. A consumer may not split them to recover namespace,
kind, role, path, or identity. `NamespaceId` and `ReferentId` are the only
corresponding semantic coordinates. Each of the 33 context records that carries
Designation metadata names its exact `designation_binding_id` and repeats only
the binding's slash-free local spelling; no record retains a free-standing
qualified name.

U+002F `/` is forbidden in quoted and unquoted authored Designation spelling.
Quotation cannot bypass that rule, and a generated or forged structured
Designation with slash in `spelling` fails before its ReferentId is used.
Slash remains valid in Text, opaque Atom and identifier payloads, source paths,
JSON Pointers, hash-domain labels, and SourceMap evidence such as the prior
spellings retained by this migration. Those values are never implicitly split
or promoted to Designations.

Each candidate authorization Judgment is a `JudgmentLocalId` record whose
subjects are exact local Law or Application identities and whose scope is the
containing ProgramSnapshot. It contains no self-derived `ProgramSnapshotId` or
`JudgmentRef`. After the snapshot is identified, the fixture resolves each
local Judgment to one exact snapshot-scoped `JudgmentRef` and its exact
subjects. Constitutive authorization then requires the separate, already
authoritative `ProgramRevisionId` which selects that exact snapshot. Neither
the local declaration nor post-snapshot reference authorizes itself.

## Canonical constitutive order

JSON object order, JSON table order, source file order, graph traversal order,
and host insertion order are nonsemantic. The candidate canonicalizer follows
these rules:

1. decode each constitutive record into its typed record form;
2. reject duplicate typed local keys;
3. canonicalize unordered record collections by the fixed kind rank declared
   by the record schema and then the unsigned UTF-8 bytes of the typed local
   key;
4. canonicalize semantic sets, including eligible ModeIds and dependency
   closure, by their typed canonical identity after rejecting duplicates;
5. canonicalize named role bindings by exact RoleId;
6. for a semantic sequence, require explicit contiguous ordinal fields,
   reject duplicate or missing ordinals, and order by the ordinal rather than
   transport position; and
7. encode each typed record using its schema-declared field order.

The normative kind ranks are fixed by this contract, not supplied by a
candidate: Term 0, RelationSchema 1, Operator 2, Mode 3, FormationJudgment 4,
ApplicationForm 5, Application 6, Law 7, and Judgment 8. A typed collection
determines its members' kind; the standalone canonical-order vectors carry an
explicit `record_kind`. Any transported `kind_rank` is redundant evidence and
must equal the fixed mapping or validation rejects before ordering.

Premise occurrences, premise slots, point-component projections, conclusion
projection fields, and ordered support selections are semantic sequences.
Their explicit ordinals survive source and transport reorderings. Independent
support occurrences form a multiset keyed by distinct SupportOccurrenceIds.
Two support occurrences with equal content therefore remain two supports.

The two canonical-order JSON vectors carry the same constitutive records and
semantic sequences in different transport orders. The two canonical-source
Clause vectors likewise permute source blocks and the facts within those
blocks. Each pair must normalize to the same abstract canonical preimage. A
transported kind rank that disagrees with the record's normative kind rejects
with `CANONICAL_KIND_RANK_MISMATCH` before ordering. This tranche freezes that
ordering relation, not a new canonical package byte encoding or ProgramSnapshot
hash.

## Process identities and exact pins

Runtime evidence is not snapshot content:

~~~text
Activation {
  ActivationId,
  ApplicationId,
  selectedModeId,
  ActivationCauseFrontier,
  RunMembership,
  initialContext
}

Step {
  StepId,
  owner: (RunId, ActivationId),
  causeFrontier,
  configurationBefore,
  configurationAfter,
  observedBaseStateRevisionId,
  emittedObservationIds,
  emittedSupportOccurrenceIds
}

Observation {
  ObservationId,
  ProducedBy(RunId, ActivationId, StepId),
  content
}
~~~

Executable validity and authorization are separate. The fixture's universal
executable-validity record establishes validity for every formed
Application/Mode pair in the exact snapshot. It grants no permission. Each
Mode separately declares one exact finite authorization set, including the
empty set where appropriate.

The exact initial context pins ClauseSemanticsId, ProgramSnapshotId,
ProgramRevisionId, RuntimeSessionId when present, RuntimePolicyId, observed or
base StateRevisionId when world-sensitive, selected ModeId, authorization,
budget, capabilities, and observable scheduler constraints.
Every positive Activation repeats its exact executable-validity evidence and
the exact evidence for every and only authorization in its selected Mode's
finite set. `pins:mode:04` contains Execution authorization;
`pins:mode:05` and `pins:mode:06` contain Execution plus Derivation
authorization. Each also records an explicit empty capability set for these
pure derivations. Empty is a checked set, not permission for undeclared host
capabilities.

One Application may be activated repeatedly. Each root activation has a fresh
ExternalTriggerOccurrenceId, ActivationId, and RunId. One ActivationId remains
stable through its configurations and receives fresh causal StepIds. Equal
observation content receives distinct ObservationIds when independently
emitted. None of these identities is a content hash, source position, array
index, table row, host object, or alias for another identity domain.

The first Step of an Activation has exactly
ActivationStart(its own ActivationId). Later cause frontiers use the typed
process rules in clause:docs/foundation.md. JSON order and a serialized trace
create no causal edge.

## Laws, premise occurrences, and support

A law is inert without exact Clause-owned DerivationAuthorization. The
position/radius law records:

~~~text
Law {
  LawId,
  conclusionSchemaId,
  derivationOperatorRef,
  derivationModeId,
  premiseOccurrences,
  conclusionProjection,
  contextRequirements
}

PremiseOccurrence {
  PremiseOccurrenceId,
  ordinal,
  RelationSchemaId,
  eligibleModeIds,
  slots
}

PremiseSlot {
  PremiseSlotId,
  ordinal,
  RoleId,
  VariableId | Constant(Term)
}
~~~

PremiseOccurrenceId is not RelationSchemaId. PremiseSlotId is not RoleId.
VariableId is the only join relation between variable-bound slots. Repeating a
schema creates distinct premise occurrences; repeating a schema role creates
distinct slot occurrences where the schema permits that cardinality. A
self-join may select one assertion occurrence for two distinct premise
occurrences, but that AssertionOccurrenceId then appears twice in the ordered
support selection.

An accepted independent support occurrence has:

~~~text
IndependentSupportOccurrence {
  SupportOccurrenceId,
  producedBy: (RunId, ActivationId, StepId),
  LawId,
  derivationModeId,
  exactContextPins,
  AuthorizationEvidence<DerivationAuthorization>,
  orderedPremiseSelections:
    Seq<(PremiseOccurrenceId, AssertionOccurrenceId | ObservationId)>,
  structuralEnvironment:
    Map<VariableId, Term>,
  exactPredicateObservationId,
  conclusionRoleBindings,
  conclusionObservationId
}
~~~

The ordered selection contains exactly one entry for every premise occurrence.
Repeated occurrence content and repeated selected occurrence IDs are retained.
The structural environment is canonicalized by VariableId and must agree with
every selected occurrence's exact RoleId bindings. It is not a host tuple,
column map, callback capture, or reconstructed spelling environment.

Support binds the exact law, derivation mode, program/world/runtime context,
and authorization which governed the producing Activation. A support cannot
be replayed under a changed mode, ProgramRevision, StateRevision, runtime
policy, or structural environment. A conclusion without this support is not a
derivation.

Derivation authorization is either constitutive authorization anchored in an
already authoritative ProgramRevision selecting the exact JudgmentRef, an
irreducible root-policy authorization, or an already issued
AuthorizationOccurrence with an independently authoritative basis. A
candidate JudgmentRef, candidate snapshot, candidate evidence, resulting
observation, or proposed successor cannot authorize the derivation that
produces it.

Fixed-point comparison includes both conclusion occurrences and independent
support occurrences. Adding a second occurrence-distinct support while keeping
equal conclusion content is progress, not a fixed point.
The positive corpus isolates that rule: a later Step emits `support:03` for the
already emitted `observation:conclusion:01` without emitting a new conclusion
Observation. The conclusion occurrence set is byte-for-byte unchanged while
the support occurrence set grows, so the expected fixed-point verdict is
false. A separate vector retains the mixed new-conclusion plus new-support
case.

## Position/radius oracle

clause:test-vectors/compiler-terms/position-radius.clause preserves the exact
ordinary-source semantic oracle while replacing its nine prior slash-qualified
spellings with explicit slash-free local aliases. It declares observer
position, target position, observer radius, within-radius, and in-proximity
roles plus the same law. `context.json` records the exact namespace,
ReferentId, visibility, and origin for every alias; the spelling change itself
does not manufacture identity.

Coordinates are signed big-endian Q16.16 values and radius is nonnegative
Q16.16. Decode to mathematical integers before arithmetic. For raw integers
tx, ty, ox, oy, and r:

~~~text
(tx - ox)^2 + (ty - oy)^2 <= r^2
~~~

The comparison is inclusive. Intermediate arithmetic is widened and admits no
wrapping, saturation, rounding, square root, floating-point conversion, or
narrowing. Space remains an exact relational join outside the numeric
predicate.

The corpus contains both decoded mathematical-integer cases and exact four-byte
signed-big-endian inputs. Its max-scale discriminator is one raw unit outside
the circle at `tx = 2147483647`, `ty = 1`, `ox = oy = 0`, and
`r = 2147483647`: exact arithmetic returns false while binary64 rounding can
incorrectly return true. Oversized unsigned physical counts are transported as
decimal strings with an explicit encoding, never as imprecise JSON numbers.

Within-radius is itself a Clause-owned ApplicationForm with exact schema,
operator, mode, named role bindings, and context. Its actual truth-directed
evaluation is an Activation and emits an exact predicate Observation. The
proximity-law support cites that ObservationId. An index candidate, AABB hit,
host boolean, or predicate name is never predicate evidence.

The constitutive slice contains 16 Terms, 16 FormationJudgments, 16 checked
ApplicationForms, and 16 nominal Applications. Applications 01--03 are the
three outer relational applications. Applications 04--07 are the four ordered
component projections from the bound target and observer `Point2Q16` values.
Applications 08--16 are the closed arithmetic graph, with Application 16 as
the verdict root. The within-radius operator inventories exactly Applications
04--16.

Every inner Application selects one exact external core schema, operator, and
mode and binds every role in that schema. Point projections bind `point` and
the produced `raw`; integer binary applications bind `left`, `right`, and the
produced `result`; the final comparison binds `left`, `right`, and the produced
`verdict`. The outer form's exact dependency closure names every inner Term,
FormationJudgment, ApplicationForm, and Application. A host may execute those
contracts generically; it may not infer `x`/`y` fields, unpack point objects by
spelling, infer produced roles from singleton mode fields, or replace the graph
with a callback selected by the operator name.

The conclusion projection contains every RoleId of world/in-proximity-v1
exactly once:

- observer from variable/observer;
- target from variable/target; and
- space from variable/space.

Projection is by RoleId and VariableId, never tuple position or spelling.

## Refinement requirements and deliberate P3 hold

For nonnegative r, a physical candidate generator may use the widened AABB:

~~~text
ox - r <= tx <= ox + r
and
oy - r <= ty <= oy + r
~~~

Circle truth implies AABB membership. AABB membership never implies circle
truth. Every emitted semantic conclusion and support must still cite the exact
Clause predicate Observation for its exact bound premise occurrences and
environment.

A future materialization contract must also require:

- exact semantic contract and plan pins on every update and receipt;
- occurrence and dependency multiplicity;
- a total bound on extent arithmetic, buckets, candidates, allocations, and
  work;
- exact scan fallback when candidate generation, extent representation,
  allocation, or refinement proof is unavailable; or
- a visible typed exhausted/rejected outcome with no semantic verdict.

The successful oversized-extent fixture makes the fallback bounds observable:
it pins population and row limits, a total exact-scan work limit, an allocation
limit, and measured rows, work units, and bytes strictly within those limits.
Selecting exact scan is therefore neither an unbounded escape hatch nor a
silent whole-state claim.

Clamping an extent, dropping a bucket, interpreting an unknown bound as empty,
returning partial results as Clause observations, or silently rebuilding the
whole state violates this boundary.

This Terms tranche intentionally defines no PhysicalPlan, physical RoleId map,
TranslationValidationWitness, MaterializationContractId, or graph-to-physical
binding. It assigns no materialization or Admission authority. The eventual P3
binding must refine this exact semantic contract and the admitted-state-delta
boundary in clause:docs/roadmap.md; it cannot be guessed by these fixtures.

## Host-semantic exclusion

The fixed host kernel may decode typed records, compare exact identifiers,
follow explicit references, enforce cardinality and ordering, evaluate admitted
generic process data, and dispatch only fixed physical operations already
admitted by the physical profile.

It may not select or synthesize semantic behavior from a relation, operator,
mode, role, variable, law, predicate, package, or designation spelling. There
is no relation-name switch, semantic-role switch, native predicate registry,
filter callback, per-law handler, generated construct case, plugin, or dynamic
host function installation in this contract.

A domain-preserving nominal bijection applied consistently to declarations,
references, environments, causes, and observations must transform results
equivariantly after canonical order is restored. If changing an opaque ID or
designation changes which host code runs, the implementation fails
HOST_SEMANTIC_DISPATCH.

## Vector contract

clause:test-vectors/compiler-terms/manifest.json binds each transport file and
its expected disposition. Positive files contain complete accepted candidates
relative to the exact fixture context. Negative files contain the malformed
structure itself; expectation metadata lives only in the manifest. No negative
is represented by a violates marker or by an error label standing in for the
malformed structure.

The two ApplicationForm negatives use a validation-only counterfactual
envelope:

~~~text
ApplicationFormSubstitutionNegative {
  contextRef,
  substitution: {
    targetApplicationFormLocalId,
    replacement: ResolvedApplicationForm
  }
}
~~~

The manifest, not the malformed payload, names the exact target and the sole
permitted change. `missing-role` equals the fully resolved `form:01` outside
the omission of the exact produced-target role binding.
`ineligible-mode` equals the same form outside replacement of `mode-id:05` by
`mode-id:04`. Both retain all eleven context requirements and the complete
four-member dependency closure, so neither has an undeclared second stage-3
failure.

The three support-occurrence negatives use the analogous transport envelope:

~~~text
SupportSubstitutionNegative {
  contextRef,
  substitution: {
    targetSupportOccurrenceId,
    replacement: IndependentSupportOccurrence
  }
}
~~~

These envelopes are validation-input counterfactuals, not Clause substitution,
retraction, mutation, or occurrence production. The referenced context and
positive process vector remain immutable. The manifest normatively binds each
vector ID to one exact target, one permitted-difference JSON Pointer and
semantic class, the last validation stage through which every other field must
remain equal, and the intended first semantic failure stage. The malformed
file cannot grant itself another permitted difference.

Before semantic validation, a fixture consumer resolves the target occurrence,
requires the target to be emitted by its exact producing Step, requires the
replacement to retain the target's SupportOccurrenceId and `produced_by`, and
compares target and replacement by exact JSON structure after masking only the
manifest-declared pointer. Object member order remains nonsemantic; array
order and multiplicity remain exact. The declared pointer must exist in both
values and actually differ. An unknown target, producer mismatch, changed
identity, absent or unchanged target field, unlisted difference, or manifest
stage mismatch invalidates the fixture before it can claim the expected Clause
error. Passing this envelope check does not mutate or admit either occurrence;
it supplies one exact malformed validation input whose first failure is then
checked by the order below.

Authored Designation reading precedes semantic validation. Unquoted `x/y` and
quoted `` `x/y` `` both fail with
`SOURCE_QUALIFIED_DESIGNATION_FORBIDDEN` at the slash byte, create no
Designation or declaration, and recover at the next valid sibling. A forged
structured record fails `DESIGNATION_SPELLING_NOT_LOCAL` during stage 1 before
its ReferentId participates in resolution.

Semantic validation is deterministic and first-failure ordered:

1. structured Designation formation, typed local-reference resolution, and
   canonical ordering;
2. RelationSchema, Role, Operator, and Mode closure;
3. FormationJudgment and ApplicationForm role/mode/context closure;
4. nominal Application identity;
5. law premise occurrence, slot, variable, and projection closure;
6. pre-existing Clause-owned authorization;
7. Activation, Run, Step, Observation, and exact context pins;
8. structural support environment and occurrence multiplicity;
9. exact predicate observation and support-sensitive fixed point; and
10. refinement safety and host-semantic exclusion.

The decisive errors are:

| Malformation | Error |
| --- | --- |
| slash in an unquoted or quoted authored Designation | SOURCE_QUALIFIED_DESIGNATION_FORBIDDEN |
| slash in a forged structured Designation spelling | DESIGNATION_SPELLING_NOT_LOCAL |
| RawTriple middle slot treated as an operator without formation | FORMATION_REQUIRED |
| missing, extra, duplicate, or wrong-cardinality role | FORMATION_ROLE_CLOSURE_MISMATCH |
| selected or eligible mode does not exactly close schema and context | FORMATION_ELIGIBLE_MODE_MISMATCH |
| equal nominal Applications or actual activations share an occurrence identity | PROCESS_IDENTITY_COLLAPSE |
| premise or slot occurrence is collapsed | PREMISE_OCCURRENCE_MULTIPLICITY_MISMATCH |
| repeated selected support is set-normalized | SUPPORT_MULTIPLICITY_MISMATCH |
| environment or process pins differ from selected evidence | SUPPORT_CONTEXT_MISMATCH |
| candidate content or candidate evidence authorizes its own derivation | AUTHORIZATION_CYCLE |
| transported kind rank disagrees with the record's normative kind | CANONICAL_KIND_RANK_MISMATCH |
| source or JSON order contributes to constitutive identity | CONSTITUTIVE_ORDER_DEPENDENCE |
| candidate membership substitutes for predicate Observation | EXACT_PREDICATE_OBSERVATION_REQUIRED |
| unsafe extent produces partial semantic output without exact fallback | UNSAFE_REFINEMENT_FALLBACK |
| semantic ID or spelling selects host code | HOST_SEMANTIC_DISPATCH |

These vectors freeze the candidate semantic boundary. Static consistency and
checksums do not establish an admitted compiler package, executable compiler,
runtime parity, materialization correctness, canonical Clause package bytes,
or the residual P3 graph-to-physical binding.
