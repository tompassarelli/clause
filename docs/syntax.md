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

The parser selects a candidate Reading deterministically from the explicit
head/operator and declared grammar in the already selected ElaborationContext.
Missing or competing readings are errors. Later schema or type checking may
reject the candidate, but cannot regroup the CST, reinterpret a sibling, or
select a different parent reading. This keeps incremental parsing and recovery
independent of successful whole-program inference.

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

relation egress/connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many

Cellar ∈ Space
Armory ∈ Space

iron-door
  ∈ Door
  connects Cellar to Armory
  state locked

law impact/direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency

derive impact/direct-dependency

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

Routine source contributes to the Program selected by the admission context.
It does not declare a nested Program, ProgramRevision, or Model merely by
grouping lines.

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
hospital is an ordinary Referent described by relations. Program identity and
admission authority come from the compilation/admission boundary, not a source
grouping keyword.

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
relation egress/connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many
```

- `egress/connects` is the schema/operator designation in this grouped form;
  the checked graph retains the distinct identities and relation between them.
- `reads` defines an exact source Reading; Clause does not perform probabilistic
  natural-language parsing.
- Braces distinguish role binders from literal phrase words.
- `subject door` is required before focus may omit that role. The first role is
  never implicitly the subject.
- Each `mode` names known inputs, yielded outputs, and cardinality. Full checked
  mode content also includes purity/effects, failures, nondeterminism, ordering,
  continuation, scheduling, identity, resources, time, cost, and admissible
  physical strategies where those are relevant.

A RelationSchema may have no operator or executable mode. In the currently
ratified compact source projection, a `relation` block with no `mode` clause
declares a schema and Reading only; one or more `mode` clauses also establish
the grouped OperatorRef. The semantic carrier still permits an operator with
zero modes, but no canonical source spelling for that distinct case is ratified
yet. An operator may otherwise have several modes. Schema, extension, operator,
mode, Reading, derivation authorization, ExecutionAuthorization, admission
authority, and effect capability never imply one another. Activation selects
one exact eligible `ModeId` and cites exact
`AuthorizationEvidence<ExecutionAuthorization>`: either a constitutive
`JudgmentRef<ExecutionAuthorization>` whose declared scope covers the exact
activation context, or an `AuthorizationOccurrenceId<ExecutionAuthorization>`
that issued it. Constitutive and issued authorization are not interchangeable
identity domains.

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

## Laws and derivation authorization

Durable rules are named laws whose binders and premises precede dependent
conclusions:

```clause
law impact/direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency
```

Every conclusion variable must be bound by the premises. A law is semantic
ground but remains operationally inert until separately authorized:

```clause
derive impact/direct-dependency
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
by an authorized transition Activation and its Steps. The source word `admit`
names candidate additions in this established delta vocabulary; it does not
perform constitutional Admission. Only the separate governance operation
commits the successor StateRevision. Source order never resolves competing
declarative writes, and a trace of the transition is not the Activation, Step,
or transition occurrence itself.

The `on` block declares process constitution. Merely representing it or an
event does not run it. An actual trigger remains an independently identified
event occurrence; activation requires an exact nominal Application, one
selected eligible `ModeId`, exact initial program/session/world pins, exact
`AuthorizationEvidence<ExecutionAuthorization>`, and an exact
`ActivationCauseFrontier`. Successful activation allocates a fresh
`ActivationId` with exact `RunMembership`; membership is assigned at activation
and never inferred from later graph reachability. The configured event
`ApplicationId` is not the actual event `OccurrenceId`; the latter carries
typed occurrence provenance. Every internally produced occurrence names the
exact `RunId`, `ActivationId`, and `StepId` that produced it. Every externally
entered occurrence instead names its exact boundary, external evidence, and
typed external cause frontier; boundary entry never fabricates an ingestion
Step. In particular, an externally entered trigger causally precedes rather
than claims production by the Activation it triggers.

A reusable change set is explicit:

```clause
delta impact/import-change
  withdraw
    North imports West
  admit
    South imports North
```

A program-history candidate names exact ancestry:

```clause
revision impact/adopt from impact
  withdraw
    North imports West
  admit
    South imports North
```

An existing delta is applied with one spelling:

```clause
revision impact/adopt from impact
  apply impact/import-change
```

Canonical source has no `~>` transition nesting and no signed delta lines.
Those forms hide the common transactional structure and
collide visually with textual diffs.

## Requests

Requests have explicit heads and a shared block envelope. A relational
ApplicationForm never becomes a query merely because it contains a variable or
because exactly one program context happens to match elsewhere.

The operand after `in` is resolved to an exact ProgramRevision before request
activation. A request activates an Application under an observation-seeking
mode; it is not a false assertion. Observations and results retain that exact
revision and Activation identity even when the authored operand is a
navigational ProgramRef.

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
    egress/passed

achieve one minimal in impact
  where
    compiler-change affects South
  using
    impact/imports

diff impact to impact/adopt
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
- Slash joins atoms in qualified designations such as `egress/connects`.
- Backticks quote multiword or Unicode designations. Their contents must
  already be NFC-normalized and cannot contain control characters or newlines.
- Double quotes remain exclusively Text literals.

The parser recovers after a malformed construct at the next line whose
indentation is at or above the failed header's level and whose explicit head
can begin a sibling construct. An error must not consume or reinterpret a
later declaration.

## Not yet canonicalized

Effect syntax, capability/resource declarations, dedicated authored scene
syntax, and package/module interchange forms remain unratified. A runtime or
projection design does not make an unratified spelling canonical. These forms
will enter this document only after their
semantic roles and one normal representation are accepted.
