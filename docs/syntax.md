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

> Every line elaborates to a Term, candidate formations, and a designated
> focus. Every indented child receives the parent focus as its omitted left
> operand under the parent's declared Reading. Indentation determines
> containment and supplies no domain relation of its own.

Conceptually:

```text
elaborate(line) -> (term, candidate formations, focus)
```

A subject-focus reading designates the subject Term directly. A construct head
may instead designate a structural declaration Term as focus and one explicit
child relation. For example, `enum Game` can elaborate a bare `Chess` child as
`[enum-declaration, has-member-entry, Chess]`; checked elaboration then produces
the ordinary membership formation and assertion candidate. The parent Reading
selects focus and child
relation before inspecting the child's domain meaning. The child never guesses
them from indentation.

The reader selects a CST production deterministically from explicit head shape
and declared grammar. Elaboration then resolves each local designation through
the already selected ElaborationContext to one structured `Designation`, and
selects a declared Reading only through that record's exact ReferentId. Missing
or competing resolutions or Readings are
errors. Later schema or type checking may reject the candidate, but cannot
regroup the CST, reinterpret a sibling, or select a different parent Reading.
This keeps incremental parsing and recovery independent of successful whole-
program inference without permitting raw spelling to select behavior.

A parent can license children in exactly two ways.

An explicit construct head can give every child one homogeneous role:

```clause
enum Game
  Chess
  Soccer
```

Or a subject-focus header can require every child to name its own edge:

```clause
iron-door
  ∈ Door
  ∈ Lockable
  connects Cellar to Armory
  state locked
```

An unkeyworded block header containing one designation has exactly one
canonical CST production: `SubjectFocus`. The parser selects that production
from the header form; it does not inspect the mix of children to classify the block.
The header must own at least one explicit edge child. Removing every child
makes it an invalid empty focus, never a bare Referent declaration; use
`referent iron-door` for that declaration.

The second block omits only the repeated subject `iron-door`; every child still
names its relation. It does not infer membership, containment, ownership,
fields, or a relation from indentation.

This is always invalid:

```clause
Foo
  Bar
```

Adding a child may not reinterpret the header or an existing sibling.

## Canonical overview

```clause
referent Door
referent Space

enum Game
  Chess
  Soccer

shape Vec2
  x: F32
  y: F32

gravity := 9.81
origin := Vec2 { x: 0.0, y: 0.0 }

relation connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many

Cellar ∈ Space
Armory ∈ Space

iron-door
  ∈ Door
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
  admit
    ?coin state collected

select all ?destination in egress
  where
    ICU-A has a usable egress path to ?destination

for n in 101..106
  Door-{n} ∈ Door
```

This is the accepted source shape. It does not expose ActivationIds, StepIds,
or graph bookkeeping when those are not semantically relevant.

## Declarations and source context

Routine source first contributes Terms, formations, declarations, and closed
uses to one checked candidate ProgramSnapshot. It gains no ProgramRevision,
Admission authority, or constitutive status from parsing, checking, or
grouping lines. A separate proposal may target an exact Program lineage and a
separate Admission may select the checked snapshot.

Canonical declaration heads are explicit:

```clause
referent Door

enum Game
  Chess
  Soccer

shape Vec2
  x: F32
  y: F32
```

- `referent` introduces or explicitly resolves one designation through the
  lineage-aware identity process.
- `enum` declares one homogeneous member-entry reading. Each child contributes
  one independent membership assertion occurrence after checked elaboration.
- `shape` declares one homogeneous field-entry reading. Each child contributes
  one `role: Domain` judgment after checked elaboration.

There is no routine `model ...` source head. A domain world, scene, game, or
hospital is an ordinary Referent described by relations. Program identity
enters at the proposal boundary; Admission authority enters only at the
separate Admission boundary. Neither comes from a source grouping keyword or
the candidate snapshot itself.

## Membership, definition, equality, and focus

Each operator has one conceptual job:

| Form | Meaning |
| --- | --- |
| `x ∈ C` | assert ordinary membership relational content |
| `name := term` | define a name as one denotation |
| `role: Domain` | annotate a structural role or field |
| `x = y` | assert equality relational content |
| `?name` | use one correlated logical variable |
| `?_` | use one fresh anonymous query hole |

Membership is repeatable and independently provenanced:

```clause
iron-door ∈ Door
iron-door ∈ Lockable
```

Inside a focus block, leading `∈` supplies the membership edge while the block
supplies only its subject:

```clause
iron-door
  ∈ Door
  ∈ Lockable
```

Every non-membership child must resolve as a declared relation phrase whose
contract explicitly names the omitted subject role. A child cannot donate
tokens back into the focus designation, and changing one child cannot
reclassify its siblings or header.

Canonical membership uses only `∈`. Raw `::` and `member_of` are invalid.
An editor may replace the input chord `\in` with `∈` before parsing; the
formatter and agents emit the glyph directly.

Definitions use `:=`:

```clause
gravity := 9.81
origin := Vec2 { x: 0.0, y: 0.0 }
```

Colon remains structural:

```clause
shape Vec2
  x: F32
  y: F32

origin := Vec2 { x: 0.0, y: 0.0 }
```

Focused `state locked` is an ordinary declared relation with the focused
Referent in its declared subject role. It is not object-field mutation or a
scoped definition. Cardinality belongs to the relation contract.

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

`:=` is definition and `=` is equality. Canonical relation modes use `given`
and `yields`; `->` is not generic directional punctuation.

## Functions, static reuse, and local ownership

`function` is the compact canonical grouping for one RelationSchema, one
Operator, and one pure, deterministic, single-result Mode. It does not add a
kernel callable or host closure. `parameters` forms one declaration-level
rank-1 StaticParameterTelescope; `constraints` forms named static evidence
slots; `given` and `yields` form exact RelationSchema roles; and `run` supplies
the Clause process definition.

The first ratified general-purpose source specimen is:

```clause
function map
  parameters
    Item: Type
    Result: Type
  constraints
    mapping: Maps Item to Result
  given
    items: Sequence of Item
  yields
    mapped: Sequence of Result
  run
    region output
      mutable builder := empty Sequence of Result
      borrow read items as source
        lease write builder as sink
          for item in source
            append mapping(item) to sink
      return freeze move builder

upper-names := map(player-names) with
  Item = Text
  Result = Text
  mapping = uppercase
```

The declaration has these exact surface rules:

- parameter and constraint children introduce stable named static slots;
  dependency determines their checked telescope order, while source traversal
  order does not become identity;
- `with` closes every uninferred static parameter and constraint by name.
  Omission, addition, ambiguity, or a wrong-domain value rejects before an
  ApplicationForm exists; ambient instance lookup and positional evidence are
  not canonical;
- a `function` call uses the exact Reading derived from its named `given` roles
  and the selected single-result Mode. The elaborated ApplicationForm stores
  RoleIds and static use records rather than argument positions;
- `region name` opens a lexical DeterministicRegion. `mutable` introduces an
  Activation-local slot; neither operation creates a StateRevision or
  Admission;
- `borrow read value as name` opens a scoped Borrow. `lease read|write|exclusive
  value as name` opens a scoped Lease and must receive causally acknowledged
  closure before the block can retire;
- `for binder in value` selects the value's exact iteration Mode. Iterations
  remain anonymous internal reductions unless a declared Step boundary is
  crossed;
- `move name` consumes the source ownership token. `freeze` stabilizes the
  moved builder into the yielded immutable value and must prove any required
  allocation-root transfer out of `output`; and
- `return` closes the produced role and is a semantic Step cut. Failure before
  it restores the exact before-configuration or discards the unpublished
  realization and closes every established root and access edge.

Static ownership and lifetime proofs may erase from a checked production ABI;
dynamically varying Lease, continuation, and close tokens remain. Equivalent
monomorphized, dictionary, irrelevant-evidence-erased, and shared-code
strategies preserve the exact cold explanation and never merge nominal uses or
Activations. Additional inference sugar is not canonical.

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
  admit
    ?coin state collected
```

`when` constrains one exact observed/base StateRevision. All `withdraw` and
`admit` content is grounded, conflict-checked, and staged as one candidate delta
by a valid transition Activation and its Steps after the selected Mode's exact
declared prerequisites have been satisfied. The source word `admit`
names candidate additions in this established delta vocabulary; it does not
perform constitutional Admission. Only the separate governance operation
commits the successor StateRevision. Source order never resolves competing
declarative writes, and a trace of the transition is not the Activation, Step,
or transition occurrence itself.

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
  admit
    South imports North
```

A program-history candidate names exact ancestry:

```clause
revision adopt-impact from impact
  withdraw
    North imports West
  admit
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
or many.

Ordered at-most-one selection:

```clause
select first ?person in World
  where
    World relates ?person to ?destination
  order by ?person
```

`select first` may return no row, but it always requires explicit `order by`.
Complete canonical row order breaks remaining ties. Storage or insertion order
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
  Door-{n} ∈ Door

for n in 101..104
  Door-{n}
    passed Fire-Marshal-Inspection
```

Ranges are inclusive, ascending integer ranges. Brackets remain structural
sequence terms; they are not also range or focus-template delimiters.

## Layout, comments, and names

- Indentation is any depth in exact increments of two ASCII spaces.
- Tabs are invalid indentation.
- Input accepts LF or CRLF, whitespace-only blank lines, and trailing spaces.
  Parsing normalizes trivia without changing lossless source evidence.
- Canonical formatting emits LF, removes trailing spaces, and uses two spaces
  per level.
- `#` starts a nonsemantic line comment.
- A contiguous `##` documentation comment attaches to the next declaration,
  request, event, or `for` head.
- Unquoted semantic identifiers are ASCII atoms beginning with a letter or `_`
  and continuing with letters, digits, `_`, or `-`.
- U+002F `/` is forbidden in every Designation spelling. In designation
  position, `x/y` rejects during reading rather than becoming one name; `/`
  remains a separate infix operator where the expression grammar permits it.
- Backticks quote multiword or Unicode designations, but do not bypass the
  spelling rule: `` `x/y` `` also rejects during reading. Quoted contents must
  already be NFC-normalized and cannot contain `/`, control characters, or
  newlines.
- Double quotes remain exclusively Text literals, which may contain `/`.
  Opaque Atom and transport payloads may also contain `/` under their own
  declared contracts; neither payload kind is a Designation.

These are required reader negatives, not aliases or recoverable spellings:

```clause
referent x/y
referent `x/y`
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
