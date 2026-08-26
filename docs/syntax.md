# Clause Syntax

> **Status:** Canonical source design is accepted; implementation migration is
> active.
>
> **Authority:** Sole authority for canonical Clause source. The
> [foundation](foundation.md) governs meaning, the
> [architecture](architecture.md) maps the current implementation to this
> design, and the [roadmap](roadmap.md) alone governs implementation status.

Clause has one canonical source language. The current parser still accepts a
legacy inferred surface while the constitutional identity migration is built.
This document presents canonical source first, then records that executable gap
in one migration ledger. Legacy spellings are current implementation facts,
not a second supported style.

## Governing rule

> Indentation determines syntactic containment. A block head determines
> construct-local elaboration. Indentation alone never invents a domain
> relation.

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
canonical AST kind: `SubjectFocus`. The parser selects that production from the
header form; it does not inspect the mix of children to classify the block.
The header must own at least one explicit edge child. Removing every child
makes it an invalid empty focus, never a bare Referent declaration; use
`referent iron-door` for that declaration.

The second block omits only the repeated subject `iron-door`. It does not infer
membership, containment, ownership, fields, or a relation from indentation.

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

This is the accepted language shape. It is not yet accepted as a whole by the
public parser; see [Implementation migration](#implementation-migration).

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
- `enum` defines a homogeneous finite-member block. Each child contributes one
  independent membership assertion occurrence.
- `shape` defines a homogeneous structural-field block. Each child is a
  `role: Domain` annotation.

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

## Relation declarations

A relation declaration states three things explicitly:

1. durable semantic identity;
2. human-facing reading; and
3. executable lookup modes.

```clause
relation egress/connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many
```

- `egress/connects` is the semantic designation.
- `reads` defines exact mixfix grammar; Clause does not perform probabilistic
  natural-language parsing.
- Braces distinguish role binders from literal phrase words.
- `subject door` is required before focus may omit that role. The first role is
  never implicitly the subject.
- Each `mode` names known inputs, yielded outputs, and cardinality.

All result cardinalities are written as words:

```clause
mode given thing yields value: one
mode given thing yields value: maybe
mode given thing yields value: some
mode given thing yields value: many
```

Omitting cardinality is invalid; absence never defaults to `one`. `0..1`, `+`,
and `*` are not canonical cardinality punctuation.

Once declared, ordinary facts stay compact:

```clause
iron-door connects Cellar to Armory
```

Surface word order is not semantic storage. Elaboration resolves one relation
identity and an exact map from named RoleIds to recursively parsed checked
terms.

## Terms and conventional operators

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

Relation roles accept recursive terms, including declared cardinality-one
applications:

```clause
position of player
radius of coin + radius of player
length (position of player - position of coin)
```

`+`, `-`, `*`, `/`, `<`, `<=`, `>`, `>=`, `=`, and `!=` retain their strong
conventional infix readings when an exact declared relation contract supports
them. Parentheses group recursive terms. Those operators still elaborate to
ordinary role-labelled relations; they do not create a second primitive
numeric ontology.

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

Events, reusable deltas, and program changes share one fact-delta vocabulary:

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

`when` constrains one pre-state. All `withdraw` and `admit` facts are grounded,
conflict-checked, staged together, and committed as one successor transaction.
Source order never resolves competing declarative writes.

A reusable change set is explicit:

```clause
delta impact/import-change
  withdraw
    North imports Store
  admit
    South imports North
```

A program-history candidate names exact ancestry:

```clause
revision impact/adopt from impact
  withdraw
    North imports Store
  admit
    South imports North
```

An existing delta is applied with one spelling:

```clause
revision impact/adopt from impact
  apply impact/import-change
```

Canonical source has no `~>` transition nesting and no signed `+ fact` or
`- fact` delta lines. Those forms hide the common transactional structure and
collide visually with textual diffs.

## Requests

Requests have explicit heads and a shared block envelope. A relational clause
never becomes a query merely because it contains a variable or because exactly
one program context happens to match elsewhere.

The operand after `in` is resolved to an exact ProgramRevision before request
execution. Results retain that exact revision even when the authored operand is
a navigational ProgramRef.

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

The initial `where` envelope supports one recursive relational clause until the
checked query plan gains an explicit conjunction node. This is an honest
current design bound, not indentation-based conjunction inference.

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

## Implementation migration

The public implementation at semantic-v10 / Revision-v6 predates this syntax.
It currently parses to `frontend::Program`, elaborates under
`ElaborationContext` into an identity-free `ProgramSnapshotCandidate`, validates
once to a checked `ProgramSnapshot`, and then uses an explicit legacy bridge to
place the checked payload in `CompiledProgram` / `kernel::Revision`. Those
bridge names are implementation vocabulary under migration; the
[foundation](foundation.md) defines the accepted semantic layers.

| Area | Canonical source | Currently executable legacy surface |
| --- | --- | --- |
| Referents | `referent Door` | bare `Door` |
| Enumeration | `enum Game` with bare members | inferred flat bare block `Game` / `Chess` / `Soccer` |
| Shape | `shape Vec2` with `x: F32` | inferred flat binding block |
| Program grouping | none in routine source | any remaining non-flat bare block is inferred as a `Model` |
| Subject focus | every child explicitly names its edge | contextually inferred focus; bare children imply membership and `state: locked` uses a scoped binding path |
| Definition | `gravity := 9.81` | `gravity: 9.81`, with a historical context-dependent lowering seam |
| Membership | `Chess ∈ Game` | the same explicit form is executable; raw `::` is already rejected |
| Relation declaration | `relation`, `reads`, `subject`, explicit word cardinality | ceremonial `RelationShape` and inferred compact schemas; suffix/default cardinality |
| Law | named `law`, `if` premises, `then` conclusion, separate `derive` | conclusion-before-premise laws plus retained `DerivationRule` and unlabelled positive rules |
| Event transaction | `when`, `withdraw`, `admit` vectors | pairwise `before ~>` / indented successor plus `if` guards |
| Revision/delta | explicit `revision` or `delta` with `withdraw`/`admit` | ceremonial `Revision`/`Delta`, labelled bodies, and signed shorthand |
| Requests | explicit request head plus `where`, `order by`, and `using` | `any`, several `select` forms, `find`, colon-labelled bodies, and context-dependent naked queries |
| Anonymous hole | `?_` | bare `?` |
| Range/template | prefix `for n in 101..106` | bracketed range/focus templates whose binder follows the use |
| Layout/trivia | arbitrary two-space depth; normalized CRLF and blank trivia | only indentation widths 0, 2, and 4; CRLF and whitespace-only blank lines rejected |
| Comments/names | `#`, attached `##`, atomic or backtick-quoted NFC names | no comments; unquoted multiword names; no Unicode normalization |

The current parser also implements recursive terms, labelled products,
role-labelled relations, correlated named holes, bounded requests, pure
definitions, authored event replay, and source-deleted generated Rust. Those
capabilities must survive the migration even though their spellings and owning
identity types change.

The compatibility surface will be removed after exact identity and result
parity. New canonical examples must not extend it. The
[roadmap](roadmap.md) is the sole status record for that work.

## Not yet canonicalized

Effect syntax, capability/resource declarations, dedicated authored scene
syntax, and package/module interchange forms remain unratified. Existing
runtime effect evidence and scene projection do not make an old proposal's
spelling canonical. These forms will enter this document only after their
semantic roles and one normal representation are accepted.
