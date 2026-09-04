# Clause Syntax

> **Status:** Canonical source design. The process-first adoption boundary is
> defined by the [adoption spike](adoption-spike.md).
>
> **Authority:** Sole authority for canonical Clause source. The
> [foundation](foundation.md) governs meaning, the
> [architecture](architecture.md) governs implementation boundaries, and the
> [roadmap](roadmap.md) alone governs implementation status.

Clause has one canonical source language. This document contains only that
language. Executable acceptance never makes another spelling canonical.

## Governing rule

> Every source construct elaborates to one or more independently identified
> semantic emissions and one designated focus. Every block head selects one
> declared child grammar before child semantics are inspected. A child receives
> the parent focus only when its selected production says how. Indentation
> determines containment and supplies no domain relation of its own.

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

IdentityPlanInput :=
    Independent
  | RetainAgainst(exact prior IdentityPlan,
                  explicit proposed ContinuityWitnesses)

elaborate(sourceConstruct, ElaborationContext, IdentityPlanInput)
  -> ElaborationResult + projected IdentityPlan | Error
```

`SourceSlice` is the exact lossless subspan responsible for that emission, not
merely the enclosing line. `Stance` is the exact contextual stance selected by
the source production; it is not inferred from the resulting Term. Every
emission is checked separately and retains its own formation diagnostics,
source-origin record, provenance, and later occurrence identity. Structural
equality among emitted Terms never deduplicates emissions.

One source construct may therefore emit several clauses. One semantic clause
never secretly becomes a list of clauses. This is the common contract for
repeated bindings, `enum` children, grouped declarations, and any later
ratified macro or destructuring form. The selected production assigns every
emission one stable semantic child slot. Repeated values in that slot each
receive an explicit `RepetitionSlotId`: retained emissions keep that ID through
the projected IdentityPlan and inserted emissions receive a fresh ID, so an
insertion cannot renumber retained equal emissions. A canonical ordinal may
order encoding but never proves continuity. Checked candidate-snapshot
construction resolves that projection to the foundation's `EmissionSlot` and records the exact
`AllocationJudgment` for every nominal emission product. Source span, byte
position, traversal, formatting, caller order, or host allocation never fills
that slot.

Canonical printing followed by elaboration preserves the focus and the ordered
emission multiplicity, Terms, candidate-formation obligations, stances, and
projected IdentityPlan. Printing may replace physical source slices with their
canonical printed slices, but the source map must retain the item
correspondence; it may not merge equal emissions or transfer an origin between
them. Retained and derived identities remain exact; independently fresh
identities are compared through the typed domain-preserving isomorphism induced
by corresponding EmissionSlots, not by demanding equal fresh bytes.

Precisely, for a checked projection `P`, re-elaboration is evaluated relative
to `projectionIdentityPlan(P)`. An edit or hot reload uses
`RetainAgainst(projectionIdentityPlan(P), explicit ContinuityWitnesses)`;
checking accepts an exact `Retain` only where its witness validates and assigns
`Fresh` to every inserted or discontinuous product. A first or deliberately
independent elaboration uses `Independent`; its fresh outputs compare through a
named `TypedFreshAlpha` domain-preserving bijection satisfying:

```text
projectedMeaning(α(elaborate(print(P), context, Independent)))
  = projectedMeaning(P)

α : TypedFreshAlpha(
      projectionIdentityPlan(P),
      projectionIdentityPlan(elaborate(print(P), context, Independent)))
```

`α` is identity on retained and declared-derived identities and maps only
corresponding independently fresh identities while preserving domains,
semantic producers, EmissionSlots/RepetitionSlotIds, multiplicity, and causal
edges. No bare structural isomorphism, source alignment, or printer position
may establish continuity.

A subject-focus reading designates the subject Term directly. A construct head
may instead designate a structural declaration Term as focus and an exact child
grammar. For example, `enum Game` can elaborate a bare `Chess` child as
`[enum-declaration, has-member-entry, Chess]`; checked elaboration then produces
an ordinary membership emission under an assertive stance. The parent Reading
selects focus and the child's contribution before inspecting the child's domain
meaning. The child never guesses them from indentation.

The reader selects a structural production deterministically from explicit head
shape and layout, retaining a lossless token/layout tree. Application grouping
is then determined by the fixed declared Reading environment, before type
inference. Elaboration resolves each local designation through
the already selected ElaborationContext to one structured `Designation`, and
selects a declared Reading only through that record's exact ReferentId. Missing
or competing resolutions or Readings are
errors. Later schema or type checking may reject the candidate, but cannot
regroup the CST, reinterpret a sibling, or select a different parent Reading.
This keeps structural parsing and recovery independent of successful whole-
program inference without pretending that phrase boundaries can always be
chosen before the available Readings are known. Competing complete parses are
errors, not opportunities to pick whichever one type-checks. Parentheses make
nested application boundaries explicit; imported vocabulary changes the fixed
Reading environment, not the meaning of ambient English.

A block head selects exactly one declared child grammar from its own CST
production. That grammar fixes the accepted child productions, their order and
multiplicity, and whether each child receives the parent focus. It cannot be
changed by a child's successful designation, Reading, formation, or type.

An explicit construct head may select a homogeneous child grammar:

```clause
enum Game
  Chess
  Soccer
```

A subject-focus header selects an explicit-application child grammar. A leaf
spells `role: object`. An interior role groups repeated objects, and any object
with descendants becomes the subject of those nested applications:

```clause
north
  shape: Flake
  description: "North-v2 development environment"
  inputs
    nixpkgs
      from: "github:NixOS/nixpkgs/nixos-unstable"
    rust-overlay
      from: "github:oxalica/rust-overlay"
      follows: nixpkgs
  development shell
    north-shell
```

Constructs such as `relation`, `function`, `law`, `on`, and requests instead
select their own heterogeneous child grammars. Each accepted child head has one
declared production and one declared focus rule. Heterogeneity never licenses
child-driven block reclassification.

An unkeyworded designation has exactly one CST production at each layout
shape. As a leaf it is a Referent declaration; with an indented child block it
is `SubjectFocus`. The parser selects between those productions from layout
alone and never inspects the mix of children to classify the block. A subject
focus must own at least one explicit edge child.

The block omits only repeated positions that layout fixes mechanically. The
`inputs` group emits `(north, inputs, nixpkgs)` and
`(north, inputs, rust-overlay)`. The nested `from` and `follows` applications
use those input objects as subjects. Indentation establishes focus and
grouping; it never invents a relation. Whitespace inside a role phrase is
lexical. The following colon, or the child-object indentation of a grouped
role, fixes the phrase boundary without underscores or type inference.

This is always invalid:

```clause
Foo
  Bar
```

Adding a child may not reinterpret the header or an existing sibling.

## Canonical overview

```clause
Door
Space

enum Game
  Chess
  Soccer

shape Vec2
  x: F32
  y: F32

gravity: 9.81
rgb: 255, 0, 0

relation connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many

Cellar
  shape: Space
Armory
  shape: Space

iron-door
  shape: Door
  shape: Lockable
  connects Cellar to Armory
  state locked

law direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency

derive direct-dependency

on collect ?actor
  when
    ?coin state active
    ?coin owner ?actor
  withdraw
    ?coin state active
  include
    ?coin state collected

select all ?destination in egress
  where
    ICU-A has a usable egress path to ?destination

for n in 101..106
  Door-{n}
    shape: Door
```

This is the accepted source shape. It does not expose ActivationIds, StepIds,
or graph bookkeeping when those are not semantically relevant.

## Declarations and source context

Routine source first contributes Terms, formations, declarations, and closed
uses to one checked candidate ProgramSnapshot. It gains no ProgramRevision,
Admission authority, or constitutive status from parsing, checking, or
grouping lines. A separate proposal may target an exact Program lineage and a
separate Admission may select the checked snapshot.

Canonical declarations use their structurally distinct source shapes:

```clause
Door

enum Game
  Chess
  Soccer

shape Vec2
  x: F32
  y: F32
```

- A bare designation leaf introduces or explicitly resolves one Referent
  through the lineage-aware identity process.
- `enum` declares one homogeneous member-entry reading. Each child contributes
  one independent membership judgment after checked elaboration.
- `shape` declares one homogeneous field-entry reading. Each child contributes
  one `role: Domain` judgment after checked elaboration.

A Shape is the contract for a subject's admissible participation, not merely
the arrangement of its fields. Depending on the Shape, that contract may
include required or permitted roles and cardinalities, value contracts, modes
and variance, failures and effects, transition laws and observable invariants,
and declared progress or resource obligations. Physical layout is separate
representation structure unless the Shape explicitly makes it observable.

Applying `shape: S` is directional satisfaction: the subject meets every
obligation exposed by `S` and is substitutable wherever `S` is required,
relative to `S`'s declared observation, effect, failure, progress, and
representation boundaries. It is not exact Shape equality, nominal membership,
denotation, or physical layout. Extra private structure may exist; additional
public structure depends on whether `S` is open or closed.

The current executable `shape` production checks the field/application subset
of this contract. Modes, laws, observations, effects, failures, and progress
obligations become part of checked Shape satisfaction only as the compiler
actually includes them; the broader definition is the semantic target, not a
claim about the present checker.

There is no routine `model ...` source head. A domain world, scene, game, or
hospital is an ordinary Referent described by relations. Program identity
enters at the proposal boundary; Admission authority enters only at the
separate Admission boundary. Neither comes from a source grouping keyword or
the candidate snapshot itself.

## Denotation, application, equality, and focus

Each surface form has one conceptual job:

| Form | Meaning |
| --- | --- |
| `name: scalar` | make `name` denote one scalar value |
| `name: a, b` | make `name` denote one ordered anonymous product |
| `subject` + `role: object` | apply one explicit semantic role |
| `x = y` | assert equality relational content |
| `?name` | use one correlated logical variable |
| `?_` | use one fresh anonymous query hole |

These concerns remain separate: a literal supplies a value, a comma supplies
an ordinal position, a role supplies meaning relative to a subject, a contract
governs admissibility, and a representation governs storage. None substitutes
for another.

A top-level colon after one name is denotation:

```clause
five: 5
phone-number: 123-456-7890
pair: 5, "hello"
```

`five` denotes the numeric value `5`. `pair` denotes one ordered compound;
its applications are position 0 to `5` and position 1 to `"hello"`. Equal
members at different positions remain distinct occurrences with independent
origins. Products compose without changing the binding boundary:

```clause
rgb: 255, 0, 0
palette: (255, 0, 0), (0, 0, 255)
```

The comma is load-bearing. Parentheses group; commas at the current delimiter
depth form a product. Nested positions remain paths, not flattened ordinal
positions. Commas inside quoted Text are literal contents. A bare head with
children is always subject focus, never a denotation inferred from commas in
its children. Bindings do not own indented children in this source profile.

Denotation does not classify its name. In particular:

```clause
north: Flake
```

means that `north` denotes the value named `Flake`. It never means Shape
satisfaction merely because the right side resolves as a Shape or contract.
Shape satisfaction is an explicit semantic application:

```clause
north
  shape: Flake
```

An identified subject may participate in any number of explicitly named
applications:

```clause
north
  shape: Flake
  worker count: 5
  description: "North-v2 development environment"
  enabled: true
```

The literal kinds constrain the objects but never select the roles. Therefore
`integer: 5`, `string: "hello"`, and `boolean: true` are not generic property
syntax: they are meaningful only if those words are genuinely the intended
domain roles. A relation contract may separately require `worker count` to
admit natural numbers; physical width and encoding remain representation facts.

Repeated roles stay repeated and retain independent provenance:

```clause
iron-door
  shape: Door
  shape: Lockable
```

An interior role groups repeated objects without turning them into one
collection value:

```clause
north
  inputs
    nixpkgs
      from: "github:NixOS/nixpkgs/nixos-unstable"
    rust-overlay
      from: "github:oxalica/rust-overlay"
      follows: nixpkgs
```

This lowers to `(north, inputs, nixpkgs)` and
`(north, inputs, rust-overlay)`, then to applications whose subjects are
`nixpkgs` and `rust-overlay`. Every object line remains its own source and
provenance site. Comma-separated spelling would instead denote one positional
product and is not an alias for repeated semantic applications.

Whitespace may occur inside a role phrase. The colon fixes the complete leaf
role boundary, and the object indentation fixes a grouped role boundary:

```clause
north
  worker count: 5
  development shell
    north-shell
```

Current source preserves those words exactly. A later declared relation
grammar may interpret words such as `to` as participant-slot markers; the
reader never guesses such composition from English heuristics and never
requires underscore encoding.

Canonical printing puts no whitespace before `:` and one ASCII space after
it. Binding denotations also accept horizontal separation around that boundary;
spacing does not select a different meaning. The bounded role/declaration
readers still require their displayed field spelling. `:=`, `::`, and `∈`
are not alternative binding operators.

Inside a declaration, its already-selected child grammar may use the same
punctuation to fill a declared structural role:

```clause
shape Vec2
  x: F32
  y: F32
```

The enclosing `shape` production supplies the subject and meaning of those
entries before their objects are resolved. Cardinality belongs to the shape or
relation contract, never to the colon token. An ordinary top-level relational
line still selects its declared assertive Reading; relation patterns inside
`where`, `when`, `if`, and other child grammars retain their grammar-owned
non-assertive stances.

## Relation, operator, mode, and Reading declarations

The compact `relation` block is a source grouping convenience. Checked
elaboration keeps its semantic products distinct:

1. a durable `RelationSchema` identity with exact named roles and constraints;
2. a human-facing source `Reading`;
3. an `OperatorRef` when the declaration supplies an operator; and
4. zero or more `Mode` declarations for that operator.

```clause
relation connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many
```

- `connects` is the local schema/operator designation in this grouped form;
  the checked graph retains the distinct identities and relation between them.
- `reads` defines an exact source Reading; Clause does not perform probabilistic
  natural-language parsing.
- Braces distinguish role binders from literal phrase words.
- `subject door` is required before focus may omit that role. The first role is
  never implicitly the subject.
- Each `mode` names known inputs, yielded outputs, and cardinality. Full checked
  mode content also includes purity/effects, failures, nondeterminism, ordering,
  continuation, scheduling, identity, ownership/lifetimes, resources, time,
  cost, and admissible physical strategies where those are relevant.

A RelationSchema may have no operator or executable mode. In the currently
ratified compact source projection, a `relation` block with no `mode` clause
declares a schema and Reading only; one or more `mode` clauses also establish
the grouped OperatorRef. The semantic carrier still permits an operator with
zero modes, but no canonical source spelling for that distinct case is ratified
yet. An operator may otherwise have several modes. Schema, extension, operator,
mode, Reading, derivation authorization, ExecutionAuthorization, admission
authority, and effect capability never imply one another. Activation selects
one exact eligible `ModeId` under a checked `StaticActivationBasis` proving
formation, executability, and an exact `CheckedConstitutionBinding`. That
binding may select checked non-authoritative candidate package/snapshot bytes
or an admitted ProgramRevision selecting the snapshot. The Mode separately
declares a canonical, named/RoleId-indexed, multiplicity-aware dynamic-
prerequisite schema; the entire schema may be empty. Each Activation binds
every exact slot separately from its occurrence-only cause frontier. When a
slot requires `ExecutionAuthorization`, the exact evidence is
either a `ConstitutiveAuthorization<ExecutionAuthorization>` pairing an already
authoritative `ProgramRevisionId` with its exact
`JudgmentRef<ExecutionAuthorization>`, an exact
`IrreducibleRootConstitution`, or an
`AuthorizationOccurrenceId<ExecutionAuthorization>` issued from an already
authoritative basis. A bare JudgmentRef, including one inside the candidate
ProgramSnapshot, is never authorization. Static callability, constitutive and
issued Authorization, dynamic capability, and Admission authority are not
interchangeable domains.

Only an `AdmittedConstitution` binding or a separately supplied
`IrreducibleRootConstitution` may discharge constitutive Authorization;
candidate checking and sandbox execution cannot.

An effect Mode also selects its exact governed-per-intent or preauthorized
local/session/Lease/batch profile. Every real-effect Activation has three
distinct semantic slots for its exact intent occurrence, issued
EffectAuthorization occurrence, and independent CapabilityEvidence. Governed
intent additionally binds its exact AdmissionOccurrence. A preauthorized scope
may cover several bounded attempts without manufacturing per-attempt Admission
or issuance; statically pinned slot values may erase from a checked hot ABI but
remain in the exact cold explanation. Constitutive execution authority never
replaces issued effect authorization. This changes neither source effect
syntax—which remains unratified—nor the distinction among intent, authority,
capability, attempt, receipt, observation, and later Admission.

All result cardinalities are written as words:

```clause
mode given thing yields value: one
mode given thing yields value: maybe
mode given thing yields value: some
mode given thing yields value: many
```

Their bounds are exact:

```text
one  = [1, 1]
maybe = [0, 1]
some  = [1, ∞)
many  = [0, ∞)
```

The lower bound is inclusive; a finite upper bound is inclusive; `∞` has no
finite member and denotes no upper bound. These words constrain result
multiplicity only through the selected Mode and never classify the result
value as a collection.

In `yields destination: many`, `many` is the Mode's declared cardinality field
for the `destination` role. Its schema supplies `[0, ∞)`; the colon token
supplies no cardinality or singleton default.

Omitting cardinality is invalid; absence never defaults to `one`. `0..1`, `+`,
and `*` are not canonical cardinality punctuation.

Once declared, ordinary relational assertions stay compact:

```clause
iron-door connects Cellar to Armory
```

Surface word order is not semantic storage. Elaboration resolves one
RelationSchema and exact named RoleIds over recursively parsed Terms. Checked
formation closes every required role and rejects missing, extra, duplicate, or
wrong-cardinality bindings. A schema without an operator can form a checked
relational row, assertion, or pattern, but not an ApplicationForm. When the
Reading also selects an exact OperatorRef, checked formation may produce an
ApplicationForm represented with recursive structurally neutral three-slot
Terms. That closed form explicitly selects and stores one exact
`RelationSchemaId`, one exact `OperatorRef`, and the exact eligible `ModeId` set
for those bindings, their known/produced orientation, and the static context.
Activation may select only a member of that stored set; an empty set leaves the
form inspectable but non-activatable. An implementation may materialize an
indexed named-role map for checking or execution. No semantic consumer may
recover an operator or role from Triple position, tuple position, graph
adjacency, or source order.

## Terms and conventional operators

A source term projects to the recursive semantic algebra defined by the
[foundation](foundation.md):

```text
Term = Atom | [Term, Term, Term]
```

The surface does not require three printed tokens per compound form. Declared
Readings, focus, delimiters, and conventional operators recover one exact Term
and candidate formations. Checked formation may then produce a closed
ApplicationForm. Merely parsing or constructing a Term does not create an
Application, assert it, activate it, authorize it, or identify a unique
occurrence.

Canonical structural terms include:

```clause
true
false
42
9.81
"player"
(3.0, 4.0)
[3.0, 4.0]
Vec2 { x: 3.0, y: 4.0 }
```

Relation roles accept recursive Terms, including declared cardinality-one
application forms:

```clause
position of player
radius of coin + radius of player
length (position of player - position of coin)
```

`+`, `-`, `*`, `/`, `<`, `<=`, `>`, `>=`, `=`, and `!=` retain their strong
conventional infix readings when an exact declared relation contract supports
them. Parentheses group recursive terms. Those operators still elaborate to
ordinary role-labelled application-form candidates; checked formation may
close them as ApplicationForms. They do not create a second primitive numeric
ontology. A closed form may be quoted or inspected without becoming a nominal
Application. Every Application receives `ApplicationId`; every Activation then
receives a distinct `ActivationId`.

`:` is a binder/role field constraint and `=` is equality. Canonical relation
modes use `given` and `yields`; `->` is not generic directional punctuation.

## Declarative definitions and physical realization

A pure function is a relation with a checked deterministic, single-result
Mode, not another kernel callable. Its ordinary definition states the result's
meaning. It does not prescribe a builder, loop, ownership-token choreography,
or execution trace.

For mapping a pure deterministic relation f over a finite sequence x, the
complete denotation is:

```text
indices(y) = indices(x)
for every i in indices(x): f(x[i], y[i])
```

This is mathematics describing the contract, not an additional implemented
source syntax. Exact index-domain equality rules out missing and extra output
positions. Indices preserve order and repeated equal values. Soundness of f
alone does not prove totality or uniqueness of y; the selected Mode must supply
those obligations. An effectful mapping needs an explicit effect/order contract
and is not interchangeable with this pure definition.

A checked physical strategy may realize the relation by fusion, a packed loop,
parallel partitions, or a local builder. It must preserve values, order,
multiplicity, observable failures, and declared resource bounds. Ownership,
regions, and reclamation remain strategy obligations unless they change
observable program meaning. Short source does not promise zero allocation,
automatic parallelism, or a total search.

Static parameters and evidence remain exact named roles with one normalized
solution. Call Readings map surface slots to declared RoleIds; matching value
shape and declaration order never infer that mapping. Static proofs may erase
when the physical refinement preserves the same meaning and explanation.

The running scalar slice is demonstrated in
`clause:test-vectors/authoring/composed-scalar-laws.clause`: a user-defined
symbolic Reading, two ordinary magnitude laws, and two composed uses. Its
finite F64 forward modes currently use `maybe`; totality is not inferred.
Law binders substitute simultaneously, so caller names cannot capture them.
Different result expressions require a proof of disjoint guards; the current
bounded compiler proves strict-cycle contradictions in finite order constraints
and otherwise rejects unknown uniqueness. Equal result expressions may retain
multiple supports without creating multiple values. This is not a general
constraint solver or a completed collection-function implementation.

## Laws and derivation authorization

Durable rules are named laws whose binders and premises precede dependent
conclusions:

```clause
law direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency
```

Every conclusion variable must be bound by the premises. A law is semantic
ground but remains operationally inert until separately authorized:

```clause
derive direct-dependency
```

Canonical source has no parallel `DerivationRule` declaration, unlabelled
durable rule, or conclusion-before-premise form. Laws, derivation
authorization, invariants, and goals remain different semantic moods even if
their implementation shares pattern machinery.

## Events, deltas, and revisions

Events, reusable deltas, and program changes share one relational-content delta
vocabulary:

```clause
on collect ?actor
  when
    ?coin state active
    ?coin owner ?actor
  withdraw
    ?coin state active
  include
    ?coin state collected
```

A transition may create a typed Referent and use that one fresh value
throughout its atomic delta:

```clause
on create-goal ?north ?title ?objective
  when
    ?north goal catalog state ?catalog
  create
    ?goal: Goal
  withdraw
    ?north goal catalog state ?catalog
  include
    ?north goal catalog state ?catalog
    ?north known goal ?goal
    ?goal title ?title
    ?goal objective ?objective
```

Each `create` binder is allocated from the accepted transition Step and its
declared Referent domain. Every use in the same rule denotes that exact value.
Relations keyed by the new Referent are typed dynamic rows: their one, maybe,
or many cardinality is enforced per subject, and the whole row delta commits
or rejects atomically with the rest of the transition.

`when` constrains one exact observed/base StateRevision. All `withdraw` and
`include` content is grounded, conflict-checked, and staged as one candidate delta
by a valid transition Activation and its Steps after the selected Mode's exact
declared prerequisites have been satisfied. `include` names candidate
additions only. Constitutional `admit` is reserved for the separate governed
operation that commits the successor StateRevision. An `admit` child in a
candidate-delta block is a reader error, not an alias for `include`. Source
order never resolves competing declarative writes, and a trace of the
transition is not the Activation, Step, or transition occurrence itself.

The `on` block declares process constitution. Merely representing it or an
event does not run it. An actual trigger remains an independently identified
event occurrence; activation requires an exact nominal Application, one
selected eligible `ModeId`, one explicit `InitialContext` recording the exact
presence or absence of world/session/policy and other declared pins, exact
`StaticActivationBasis`, exact `DynamicPrerequisiteBindings` closing the
selected Mode's possibly empty schema, and a separate exact occurrence-only
`ActivationCauseFrontier`. A Mode whose entire dynamic-prerequisite schema is
empty manufactures no binding, Authorization, or capability. Successful activation
allocates a fresh `ActivationId` with exact `RunMembership`; membership is
assigned at activation and never inferred from later graph reachability. The configured event
`ApplicationId` is not the actual event `OccurrenceId`; the latter carries
typed occurrence provenance. Every internally produced occurrence names the
exact `RunId`, `ActivationId`, and `StepId` that produced it. Every externally
entered occurrence instead names its exact boundary, external evidence, and
typed external cause frontier; boundary entry never fabricates an ingestion
Step. In particular, an externally entered trigger causally precedes rather
than claims production by the Activation it triggers.

A reusable change set is explicit:

```clause
delta import-change
  withdraw
    North imports West
  include
    South imports North
```

A program-history candidate names exact ancestry:

```clause
revision adopt-impact from impact
  withdraw
    North imports West
  include
    South imports North
```

An existing delta is applied with one spelling:

```clause
revision adopt-impact from impact
  apply import-change
```

Canonical source has no `~>` transition nesting and no signed delta lines.
Those forms hide the common transactional structure and
collide visually with textual diffs.

## Requests

Requests have explicit heads and a shared block envelope. A relational
ApplicationForm never becomes a query merely because it contains a variable or
because exactly one program context happens to match elsewhere.

The operand after `in` resolves to an exact `CheckedConstitutionBinding` before
request activation. A request that joins an authoritative RuntimeSession,
proposes authoritative world change, relies on constitutive Program authority,
or performs a real effect names an exact ProgramRevision, including when the
authored operand is a navigational ProgramRef. A sandbox or candidate request
instead names exact checked package and ProgramSnapshot bytes; it may read a
separately pinned admitted world and persist nonauthoritative results or
continuations, but it cannot fabricate a ProgramRevision, StateRevision, real
external effect, or constitutive authority. A request
activates an Application under an observation-seeking mode; it is not a false
assertion. Observations and results retain that exact constitution binding and
Activation identity.

Projection:

```clause
select all ?destination in egress
  where
    ICU-A has a usable egress path to ?destination
```

Exact-one selection:

```clause
select one ?person in World
  where
    World relates ?person to C
```

`select one` requires exactly one deduplicated projected row and fails on zero
or many. A projected row is the ordered sequence of closed projected Terms in
the request head's explicit projection-slot order. Within this initial query
profile, row equality is exact structural Term equality after checked
elaboration; no type-directed coercion or observational equivalence is
inferred. Provenance, supports, derivations, and Observation identities do not
participate in row equality. Equal rows reached through distinct witnesses
deduplicate to one value row while retaining all independently identified
support alternatives for explanation. Deduplication never merges the
underlying assertions, observations, or evidence.

Ordered at-most-one selection:

```clause
select first ?person in World
  where
    World relates ?person to ?destination
  order by ?person
```

`select first` may return no row, but it always requires explicit `order by`.
Each order key must select an exact declared total-order Mode for its value
domain. Keys compare in source order under those Modes; canonical Term bytes of
the complete projected row, length-delimited in projection-slot order, break
remaining ties lexicographically. A request whose projected Term lacks the
ratified canonical encoding or whose order key lacks one exact total-order Mode
is rejected during checking. Clause does not invent a universal semantic order
for such a value. Storage, insertion, derivation, support, or observation order
never becomes language semantics.

Existence uses an explicit request head:

```clause
any in World
  where
    World relates ?_ to C
```

Explanation and intervention retain their distinct operations:

```clause
why all in egress
  where
    ICU-A has a usable egress path to North-Exit

prevent all minimal in egress
  where
    ICU-A has a usable egress path to North-Exit
  using
    passed

achieve one minimal in impact
  where
    compiler-change affects South
  using
    imports

diff impact to adopt-impact
```

`find` and naked-query inference are not canonical. A fresh anonymous hole is
`?_`, not bare `?`. Random witness selection requires an explicit seed or
recorded choice evidence; no unseeded `first` or hidden random selection is
permitted.

The initial `where` envelope supports one recursive relational application
pattern until the checked query plan gains an explicit conjunction node. This
is an honest current design bound, not indentation-based conjunction inference.

## Prefix binders and interpolation

Binders precede every dependent use:

```clause
for n in 101..106
  Door-{n}: Door

for n in 101..104
  Door-{n}
    passed Fire-Marshal-Inspection
```

Ranges are inclusive, ascending integer ranges. Brackets remain structural
sequence terms; they are not also range or focus-template delimiters.

## Normative reader boundary

Lexing and structural layout selection precede designation resolution,
formation, type checking, and child semantics. Full application grouping is
Reading-directed under one fixed environment, as specified above. The reader
applies these structural rules in order:

1. Normalize CRLF to LF for layout while preserving original byte spans in the
   lossless CST. Split physical lines; Clause has no backslash or implicit
   expression-line continuation. An explicitly delimited multiline Text is one
   scalar token, not line continuation.
2. Scan triple-quoted Text spans through their explicit closing-delimiter
   margin. Body lines and the closing delimiter do not emit layout tokens.
3. Establish `INDENT` and `DEDENT` tokens from exact multiples of two ASCII
   spaces outside those spans. Blank and comment-only lines do not alter
   indentation.
4. Scan single-line strings and quoted designations, then longest fixed
   punctuation, then numbers and unquoted designations. Tokens are maximal;
   semantic lookup can never split or join them.
5. Select the line production from its explicit head tokens. A literal keyword
   is a keyword only in the declared grammatical position; successful semantic
   resolution cannot turn an identifier into a construct head.
6. Wrap an already selected line head in its declared block production when an
   `INDENT` follows. The head itself is not reparsed after children are known.

The relevant layout grammar is:

```text
SourceFile       ::= Trivia* TopLevelConstruct* EOF
TopLevelConstruct ::= SimpleConstruct NEWLINE
                    | BlockHead NEWLINE INDENT ChildConstruct+ DEDENT
BindingHead      ::= Designation HSPACE* ":" HSPACE* ProductTerm
BindingConstruct ::= BindingHead
ProductTerm      ::= GroupedTerm ("," HSPACE* GroupedTerm)*
GroupedTerm      ::= ScalarTerm | "(" HSPACE* ProductTerm HSPACE* ")"
SubjectFocus    ::= Designation NEWLINE INDENT FocusedEdgeChild+ DEDENT
FocusedEdgeChild ::= RelationEdge
                   | RelationPrefix NEWLINE INDENT FocusedEdgeChild+ DEDENT
ReferentDeclaration ::= Designation
MultilineText     ::= '"""' NEWLINE MultilineTextBody MultilineTextClose
```

`BindingHead` emits one binding and does not also introduce subject focus.
Products preserve their balanced grouping at every depth. `RelationPrefix` emits no edge by
itself; it prepends its tokens to each descendant `RelationEdge`.
`ReferentDeclaration` is the leaf form of one Designation. `SubjectFocus` is
the block form of one Designation and requires at least one child. Keyworded
heads select their own declared child grammars as specified above.

`HSPACE` is exactly one U+0020 ASCII space. `HSPACE+` means one or more such
spaces where flexible separation is declared. The formatter prints no space
before a binding colon and one after it; harmless horizontal separation does
not change the binding. Layout indentation still uses ASCII spaces only.

Familiar mathematical notation is a usability prior, never semantic
authority. A notation is ratified only when its tokenization, arity,
precedence, associativity, and binding scope are fixed; elaboration is exact and
total for every accepted CST; no hidden coercion, quantification, conjunction,
or cardinality rule is inferred; siblings and children cannot reinterpret it;
nominal identity, multiplicity, provenance, authority, and item origins survive
elaboration; and one canonical formatter plus explicit competing-
interpretation negatives are specified.

### Tokens, operators, and delimiters

- Longest fixed punctuation wins: `<=`, `>=`, `!=`, and `..` are scanned
  before their one-character prefixes. `=`, `:`, `,`, parentheses,
  brackets, and braces are distinct tokens. `:=`, `::`, `->`, and `~>` are
  reader errors in canonical source, not alternate spellings.
- Unquoted semantic identifiers match
  `[A-Za-z_][A-Za-z0-9_-]*` maximally. Consequently `a-b`, `Door-101`, and
  `x--y` are each one Designation token. A symbolic infix operator must have at
  least one ASCII space on both sides: subtraction is `a - b`; `a- b` and
  `a -b` reject instead of being guessed. The same spacing rule applies to
  `+`, `*`, `/`, `<`, `<=`, `>`, `>=`, `=`, and `!=`. Colon is not an infix
  expression operator; its binding production fixes its own spacing.
- At a position where a term may begin, `-` immediately followed by a digit is
  part of a signed numeric literal. Initial canonical integer syntax is `0` or
  an optional `-` followed by a nonzero digit and zero or more digits. Initial
  canonical decimal syntax adds `.` and one or more fractional digits. Leading
  `+`, leading zeroes, omitted integer or fractional digits, digit separators,
  and exponent notation are unratified. Canonical printing removes integer
  leading zeroes and otherwise preserves the exact checked numeric value,
  including a semantically distinct floating negative zero.
- Postfix calls and delimited structural terms bind first; `*` and `/` bind
  next; `+` and `-` next; `<`, `<=`, `>`, and `>=` next; and `=` and `!=` next.
  Arithmetic operators associate left. Comparison and equality operators do
  not chain. `:`, `..`, and statement-level comma have
  only their declared construct roles. Parentheses are required whenever these
  rules do not select one CST.
- `()`, `[]`, and `{}` must balance on one physical line in the initial reader.
  A mismatched or unclosed delimiter rejects that construct; recovery begins at
  the next eligible sibling boundary. Comma separates fields or elements only
  inside the selected delimited production. There is no general comma
  expression or binding-value list.

Single-line double-quoted Text literals contain UTF-8 scalar values and accept exactly
`\"`, `\\`, `\n`, `\r`, `\t`, and `\u{H}` through `\u{HHHHHH}` where the
hexadecimal value is a Unicode scalar. Unknown escapes, surrogate values, raw
newlines, and unescaped control characters reject. Text is not NFC-normalized
by the reader. Canonical printing emits printable scalars directly, escapes
`"` and `\`, uses the named escapes above for newline, carriage return, and
tab, and uses lowercase `\u{...}` for other controls.

A triple-quoted Text literal begins with `"""` as the final token of a scalar
expression line. Its first following line whose sole nonspace content is
`"""` closes the literal and must be indented at least one two-space level
deeper than the expression line. The closing delimiter's indentation is the
explicit content margin: every nonblank body line must begin at or to the right
of that margin, and the reader removes exactly that margin from every body
line. Extra indentation remains Text. The line feed after the opening delimiter
is not content; every body line's normalized LF is content, including the LF
immediately before the closing delimiter. An opener followed immediately by a
closer is empty Text.

Triple-quoted Text accepts the same escapes as single-line Text, permits
unescaped `"` within body lines, preserves printable UTF-8 and spaces after the
explicit margin, and rejects other raw controls. A line containing only
`"""` is therefore a delimiter; quotes on any other body line are content.
No minimum-indent guessing, folding, or chomping mode exists.

### Layout, comments, and names

- Indentation is any depth in exact increments of two ASCII spaces. Tabs are
  invalid anywhere in indentation.
- Input accepts LF or CRLF, whitespace-only blank lines, and trailing spaces.
  Parsing normalizes trivia without changing lossless source evidence.
- Canonical formatting emits LF, removes trailing spaces, and uses two spaces
  per level.
- Outside a single-line or triple-quoted Text literal or quoted designation,
  `#` starts a nonsemantic line
  comment and consumes through the line ending.
- A contiguous run of `##` documentation-comment lines at the indentation of
  the following construct attaches to that next declaration, request, event,
  or `for` head. An intervening blank line or non-documentation comment breaks
  attachment.
- U+002F `/` is forbidden in every Designation spelling. In designation
  position, `x/y` rejects during reading rather than becoming one name; `/`
  remains a separate infix operator where the expression grammar permits it.
- Backticks quote multiword or Unicode designations, but do not bypass the
  spelling rule: `` `x/y` `` also rejects during reading. Quoted contents must
  already be NFC-normalized and cannot contain `/`, control characters, or
  newlines.
- Double quotes, including the triple-quoted form, remain exclusively Text
  literals, which may contain `/`.
  Opaque Atom and transport payloads may also contain `/` under their own
  declared contracts; neither payload kind is a Designation.

These are required reader negatives, not aliases or recoverable spellings:

```clause
x/y
`x/y`
```

A candidate package or generated form that directly constructs a structured
`Designation` whose `spelling` contains `/` must likewise fail Designation
formation before its ReferentId can participate in identity resolution and
before RelationSchema, Role, or Operator closure. Current implementation status
remains governed by the roadmap; this rule does not claim that a supported
reader or checker already exists.

Namespace membership, imports, exports, visibility, and designation resolution
are explicit checked relations and constraints in the semantic carrier; they
are not encoded into names. After an exact namespace/import source grammar is
ratified, a display may render a structured Designation in a reversible
`namespace/local` form. Such display text remains SourceMap or
diagnostic projection: it never crosses elaboration as a semantic identifier,
defines identity or equality, recovers a RoleId or OperatorRef, selects
behavior, or admits multi-segment kind/role/path conventions.

The parser recovers after a malformed construct at the next line whose
indentation is at or above the failed header's level and whose explicit head
can begin a sibling construct. An error must not consume or reinterpret a
later declaration.

## Not yet canonicalized

Effect syntax, capability/resource declarations, continuation/race syntax,
dedicated authored scene syntax, and package/module interchange forms remain
unratified. Their semantics may be accepted and exercised through canonical
packages before one source projection is selected. A runtime or projection
design does not make an unratified spelling canonical. These forms enter this
document only after their semantic roles and one normal representation are
accepted.
