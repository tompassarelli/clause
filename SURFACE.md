# Clause Surface Reset

## Distinction-first relational authoring

**Status:** constitutional target; referents, claims, laws, classification,
definition, and indentation frozen by M0

Clause has one semantic domain: addressable referents. A referent is a
stabilized distinction that can be reidentified across claims and Revisions;
structural equality never collapses that identity. Relations are referents in
relational position. Claims arrange referents in named roles. Laws derive,
constrain, reject, or operationally orient claims. Terms project referents but
are not themselves referents, and a claim remains distinct from its acceptance,
status, and authority.

**Audience:** Clause implementers, reviewers, formatter/tooling authors, and
corpus designers

**Authority:** [ROADMAP.md](ROADMAP.md) remains the normative execution and
milestone authority. This file is an editorially normalized tracked copy of the
operator's current surface draft, so decisions do not depend on an external
document. Its relational invariants control direction over earlier
human-surface recommendations. `Chess : Game` is classification sugar for an
ordinary membership relation; `gravity := 9.81` is definition/denotation.
Neither spelling creates a primitive type or value domain. `∈` and `::` are
not canonical source and editors must not rewrite input to `∈`. Canonical
indentation is two spaces, spaces only, and tabs are diagnosed.

**Scope:** this supersedes the human-surface recommendations and specimens in
the earlier strategy packet. It does not discard Clause's semantic core,
immutable Revisions, role-labelled n-ary relations, provenance, explanation,
interventions, or target strategy.

## 1. The decision

Clause should stop asking authors to declare an ontology for the language
before they can describe the domain.

The canonical surface should be organized around four semantic commitments:

1. **referents** — stabilized, addressable distinctions;
2. **claims** — referents arranged in named roles, including membership;
3. **laws** — derivation, constraint, rejection, and operational orientation;
4. **definitions** — explicit denotation from a term to what it designates.

Source terms name or compose these commitments but do not create another
semantic domain. Acceptance, status, authority, and occurrence metadata qualify
claims without becoming claim content.

The decisive surface thesis is:

> **A Clause program is a graph of grounded symbols and role-labelled clauses.
> Layout is a projection over that graph. Layout never creates objects, fields,
> ownership, or containment in the semantic core.**

The decisive implementation thesis is:

> **Infer the authored category from the form itself. Require a keyword only
> when the form does not already prove what it is.**

This means the ordinary surface should not require `Type`, `Relation`, `Law`,
`Model`, `Revision`, `entity`, `class`, `struct`, `let`, or a shape prefix
before every clause.

It also means we must stop writing relational entities as records.

Reject this as canonical Clause:

```clause
iron-door : Door
  connects: Cellar to Armory
  state := locked
```

That form smuggles in four ideas Clause does not need:

- `iron-door` is an object instantiated from `Door`;
- `Door` owns a record schema;
- `connects` and `state` are fields;
- indentation denotes nested storage or ownership.

The canonical relational form is:

```clause
iron-door : Door
  Door
  connects Cellar to Armory
  state := locked
```

Its exact elaboration is:

```clause
iron-door : Door
iron-door connects Cellar to Armory
state of iron-door := locked
```

The first form elaborates to an ordinary membership claim, the second is an
ordinary co-equal relational claim, and the third is a definition. Focus
supplies `iron-door` without turning any form into an owned object property.

The compiler may later lower `state` to an enum field, `connects` to an
adjacency index, and membership to a bitset. Those are physical strategies.
They are not the authored ontology.

## 2. What this retracts from the earlier proposal

The earlier packet was directionally right about recursive relation-centred
syntax, but several specimens still carried conventional object and declaration
assumptions. Replace them.

Retract:

```clause
use game
use three
```

Adopt:

```clause
requires
  game
  three
```

Retract:

```clause
type: Vec2
  x: F32
  y: F32
```

and:

```clause
type Vec2:
  x: F32
  y: F32
```

Adopt:

```clause
Vec2
  x: F32
  y: F32
```

The body proves this is a structural value shape. The word `type` adds no
information. A colon after `Vec2` would merely open a block, which is not a
legitimate use of `:`.

Retract:

```clause
player: Player
  position: (0, 0)
  velocity: (0, 0)
```

Adopt:

```clause
player
  Player
  position (0, 0)
  velocity (0, 0)
```

`player` is a grounded symbol. `Player`, `position`, and `velocity` are claims
about it. Nothing is instantiated and no field is assigned.

Retract `position of player = (0, 0)` when the intended meaning is
initialization or admission. Adopt `player position (0, 0)`, or under focus:

```clause
player
  position (0, 0)
```

Reserve `=` for actual equality.

Retract `find all ?person where` as the default relational question. Adopt:

```clause
?person likes Chess
```

A hole-bearing clause is already visibly a question. The relation should remain
the centre of the query.

Retract the explanation that `?person` is merely inherited logic-variable
syntax. Adopt:

```text
?        an anonymous hole in a clause
?name    the same hole, given a reusable/result name
```

The notation is justified visually and compositionally:

```clause
? likes Chess
?person likes Chess
Alice likes ?opening
```

Use `:` for classification/membership sugar, `:=` for definition/denotation,
and `=` for equality. Identity remains distinct from all three.

## 3. The ontology the surface should expose

Clause implementations may derive a type-system view. The surface must not
force authors to declare types as a second semantic domain.

The surface must distinguish three semantic strata without turning each into a
keyword ceremony.

### 3.1 Value domains and shapes

Examples:

```clause
F32
Int
Bool
String
```

These are built-in value domains with representation and operation contracts.

A user-defined structural value view is inferred from a homogeneous definition
block:

```clause
Vec2
  x := F32
  y := F32
```

This means `Vec2` has definitions for labels `x` and `y`, each denoting `F32`.
It does **not** add a primitive value or record domain.

### 3.2 Semantic symbols

Examples:

```clause
Player
Game
Door
iron-door
Chess
```

A bare name can ground a semantic distinction in the program. It does not need
fields, a constructor, or immediate classification. A semantic symbol may later
participate in membership and relations.

### 3.3 Categories and contracts

A category is not a separate ontological species. It is a semantic symbol used
as the target of membership and, optionally, as the focus of contracts.

```clause
Game
  Chess
  Soccer
```

means:

```clause
Chess : Game
Soccer : Game
```

A member may itself be a category:

```clause
Chess
  Blitz
  Rapid
  Classical
```

Therefore `Chess` is both a member of `Game` and a category for its own
members. Clause should not force it into an exclusive “instance” or “type” box.

A category contract constrains relations its members participate in:

```clause
Player
  position -> Vec2
  velocity -> Vec2
  radius -> F32
```

This does not declare fields on a `Player` object. It says that, for members of
`Player`, the focused `position`, `velocity`, and `radius` projections are
single-valued and return those domains.

The compiler may represent those functional relations as fields, columns,
component arrays, or indexes. That is a lowering decision.

### 3.4 The rule

> **Value shapes may derive named relational roles. Semantic symbols have relations.
> Categories have relational contracts. Do not turn semantic entities into
> records merely because a backend may store them that way.**

## 4. Classification and definition are different

The two forms are deliberately separate:

```clause
Chess : Game
gravity := 9.81
```

`:` is classification sugar. It elaborates to an ordinary membership relation
whose member and group are named roles; it does not bind a name, declare a
primitive type, or assert equality. `:=` is definition/denotation: a source term
is oriented to what it designates. Definition does not collapse the term into
the referent and does not confer acceptance or authority on a claim.

Examples:

```clause
spawn : Position
spawn := (0, 0)
receipt := render! scene
```

The first line classifies `spawn`; the second defines it. A structural view may
use both without creating an object or record ontology:

```clause
Vec2
  x : F32
  y : F32
```

A pure definition uses `:=`:

```clause
distance between ?a and ?b :=
  length(position of ?a - position of ?b)
```

An explicit relation schema retains named roles and classifies their referents:

```clause
connects :=
  door : Door connects origin : Space to destination : Space
```

Neither `∈` nor `::` is canonical source. Editors, formatters, and agents emit
`:` classification and `:=` definition directly; they do not rewrite any input
to `∈`. A migration tool may recognize retired spellings only to produce an
explicit report and corrected source, never as a second live grammar.

## 5. Layout has three inferred forms

Indentation is two spaces per level, spaces only. A tab is diagnosed rather
than expanded or normalized. Layout remains erasable and never introduces
ownership or nested object identity.

The reader should classify an indented block by the forms of its children.

### 5.1 Enumeration block

A block containing only bare symbols is an enumeration unless its heading is a
declared reserved form such as `requires`:

```clause
Famous Chess Openings
  Sicilian Defense
  Ruy Lopez
  Queen's Gambit
```

It lowers to:

```clause
Sicilian Defense : Famous Chess Openings
Ruy Lopez : Famous Chess Openings
Queen's Gambit : Famous Chess Openings
```

This preserves the ordinary prior that an indented list contains members of
its heading. Reserved list forms declare their own expansion; for example,
`requires` relates the current program to each listed package rather than
asserting ordinary domain membership.

### 5.2 Definition/shape block

A homogeneous block of definitions can project a structural shape:

```clause
Vec2
  x : F32
  y : F32
```

Each child is classification sugar. The optional structural view derives from
those membership claims; it is not a primitive record schema.

### 5.3 Focused block

A block containing relation fragments, contracts, transitions, or other clause
forms establishes its heading as the focus participant:

```clause
iron-door
  Door
  connects Cellar to Armory
  state := locked
```

It lowers to:

```clause
iron-door : Door
iron-door connects Cellar to Armory
state of iron-door := locked
```

Within a focused block, a bare category name classifies the focus through the
ordinary membership relation, an ordinary relation fragment supplies the focus
role, and `name := value` defines a stable focused term. These forms remain
semantically distinct.

### 5.4 Mixed blocks have one deterministic reading

The presence of any non-bare child makes the block a focused block.
Therefore:

```clause
Thing
  A
  B
  relation value
```

means:

```clause
Thing : A
Thing : B
Thing relation value
```

It does **not** enumerate `A` and `B` beneath `Thing`.

When a category needs both member enumeration and contracts or claims, write
separate blocks:

```clause
Thing
  A
  B

Thing
  relation value
```

The canonical formatter should preserve this separation. An editor should warn
before an edit changes an all-bare enumeration block into a focused claim
block, because that edit changes the interpretation of the existing bare
lines.

This structural rule resolves the apparent conflict between `Game / Chess` and
`iron-door / Door / state := locked` without fuzzy English parsing and without
making membership direction depend on capitalization.

### 5.5 Layout equivalence is an invariant

For every focused block, the compiler must be able to print its fully expanded
semantic forms without collapsing membership claims, ordinary claims, or
definitions into one node kind.

Moving between focused and expanded forms must not change:

- proposition identity;
- participant roles;
- provenance except source span;
- revision identity after canonical elaboration;
- query results;
- generated code.

## 6. The minimal symbolic vocabulary

Clause should adopt this vocabulary and reject synonyms that blur the axes.

| Form | Meaning |
| --- | --- |
| `:` | classification sugar for ordinary membership |
| `:=` | definition/denotation |
| `=` | equality proposition |
| `->` | result/projection direction or return contract |
| `?` | anonymous clause hole |
| `?name` | named clause hole |
| `~>` | a proposition in the current state succeeds to another proposition |
| `+` | exact admission/addition |
| `-` | exact withdrawal/removal |
| `!` | external effect/capability boundary |

Words carry semantic moods:

| Word | Meaning |
| --- | --- |
| `if` | timeless derivation |
| `on` | event/time-triggered transition scope |
| `from` | exact revision ancestry |
| `requires` | program/package requirements |
| `select` | explicit relational projection |
| `any` | existential truth test |
| `why` | explanation query |
| `prevent` | counterfactual withdrawal synthesis |
| `achieve` | counterfactual addition synthesis |
| `diff` | revision comparison |
| `observe` | evidence-backed claim mood |
| `assume` | scoped premise mood |
| `require` | proof obligation mood |
| `intend` | desired-state mood |
| `do` | explicitly procedural stratum |

`find` is not a core relational mood. It may remain an ordinary library or
tooling verb for text, file, symbol, or corpus search, where its conventional
meaning is useful.

Program/module identity should normally come from the admission or package
context rather than a mandatory `module` line inside every source projection.
`requires` declares dependencies; explicit module declarations remain an
interchange or standalone-file escape hatch.

Do not append colons to these words merely to open blocks:

```clause
if
on frame ?dt
requires
observe
```

Layout already groups their bodies.

## 7. Relation shapes remain exact, recursive, and role-labelled

Clause is not a natural-language parser. Domain phrases are exact declared
mixfix shapes.

A public or ambiguous relation can use an explicit schema definition:

```clause
connects:
  door: Door connects origin: Space to destination: Space
  door origin -> destination*
```

This says:

- stable human anchor: `connects`;
- focused or subject role: `door`;
- participant domains: `Door`, `Space`, `Space`;
- phrase: `{door} connects {origin} to {destination}`;
- operational projection: given `door` and `origin`, produce zero or more
  `destination` values.

No `Relation` keyword, braces, or `mode` keyword is needed.

The checked core still retains relation identity, named roles, role domains,
surface pattern, and voice/cardinality contract.

### 7.1 Functional focused relations

The common one-subject/one-value case should be much smaller:

```clause
Player
  position -> Vec2
  velocity -> Vec2
  radius -> F32
```

Each line declares a focused binary relation. Conceptually:

```text
member: Player position value: Vec2
member: Player velocity value: Vec2
member: Player radius value: F32
```

The ordinary proposition is subject-first:

```clause
player position (0, 0)
```

The value projection is relation-first:

```clause
position of player
```

Both elaborate to the same relation and named roles; surrounding context
selects a proposition form or a one-valued projection.

Canonical Clause should prefer `position of player` over `player.position`.
Dot access makes the object syntactically sovereign and hides the fact that
`position` is an ordinary relation with its own identity, roles, laws, queries,
and strategies. Reserve dot syntax for explicit foreign-host interoperation,
if it exists at all.

Exactly one is the default return cardinality. `*` means zero or more; `+`
means one or more. Do not overload suffix `?` for optionality while `?` is the
hole glyph. Use an explicit `maybe` contract until a better optional notation
earns itself.

### 7.2 Ordinary clauses do not name their schema

After the shape exists, `iron-door connects Cellar to Armory`, or its focused
form, is enough. Qualification is required only when resolution is ambiguous or
when a structural or debug form is requested.

### 7.3 Relation roles accept recursive terms

This is non-negotiable. A participant slot must accept a recursively parsed
term of the expected domain, not merely one token.

```clause
distance between player and coin < radius of player + radius of coin
```

must elaborate as nested relation applications, not a flat token template.
This recovers the recursive-triple insight without forcing the semantic core to
become fragile binary pairs.

## 8. Holes, rules, and correlation

The hole system should be taught from the punctuation itself.

```clause
? likes Chess
```

means: the first participant is absent; solve that role.

```clause
?person likes Chess
```

means: solve the same role and expose its assignments under the name `person`.

```clause
Alice likes ?opening
```

means: solve the opening role.

Each bare `?` is fresh. Repeating a named hole requires the same referent:

```clause
?person likes ?opening
Alice likes ?opening
```

The two `?opening` occurrences correlate through identity.

### 8.1 Laws are inferred from `if`

```clause
?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?destination
  ?door passed Fire-Marshal Inspection
```

The conclusion plus `if` body proves that this is a law. Do not require `Law`.

A recursive law is equally direct:

```clause
?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?intermediate
  ?door passed Fire-Marshal Inspection
  ?intermediate has a usable egress path to ?destination
```

### 8.2 Optional human labels are definitions

Most laws should receive stable semantic identities independently of a human
label. When a human name is useful:

```clause
recursive route :=
  ?origin has a usable egress path to ?destination if
    ?door connects ?origin to ?intermediate
    ?intermediate has a usable egress path to ?destination
```

The definition orients the label `recursive route` to the law. It does not
declare a `Law` object or enter the law's claim content.

### 8.3 Category-wide facts remain ordinary laws

Do not invent field defaults. To say every coin has radius 8:

```clause
?coin radius 8 if
  ?coin : Coin
```

That is a universal relational law. It is not a default value installed into a
record schema.

A category contract and a universal fact remain distinct:

```clause
Coin
  radius -> F32

?coin radius 8 if
  ?coin : Coin
```

## 9. Queries begin with the relation

`find` should not be the default relational query word. It carries a search or
corpus prior and makes the result variable syntactically primary.

A naked hole-bearing clause selects all solutions:

```clause
?person likes Chess
```

Multiple named holes yield rows:

```clause
?person likes ?opening
```

### 9.1 `select` is for explicit projection and multi-clause queries

```clause
select ?person
  ?person likes ?opening
  Alice likes ?opening
```

Only `person` is projected. `opening` is an internal correlated hole. The
default result is all rows. Therefore `select all` is permitted for emphasis
but not required.

### 9.2 Existence and witness selection are different

```clause
any ?person likes Chess
```

returns `Bool`: does at least one solution exist?

```clause
select one ?person
  ?person likes Chess
```

requires exactly one result or fails its cardinality contract.

```clause
select first ?person
  ?person likes Chess
```

returns the canonical first result.

Random choice is not `any`:

```clause
sample ?person
  ?person likes Chess
```

Randomness requires an explicit random capability or seed and is not a pure
query.

### 9.3 Keep `return` for procedural control flow

Do not use `return all ?person likes Chess` for relational selection. `return`
retains its conventional meaning inside the explicit procedural stratum.

### 9.4 Interrogative English may be derived sugar, not the kernel

A relation package may eventually provide exact interrogative projections such
as `who likes Chess` or `what does Alice like`. Add them only where declared
role grammar determines one expansion exactly. Do not build a general English
parser, and do not make relative-clause English the only way to express
correlation. Hole and `select` forms remain the small general substrate.

### 9.5 Preserve distinct semantic queries

Keep explicit words for genuinely different operations:

```clause
why
  ICU-A has a usable egress path to North-Exit

prevent all minimal
  ICU-A has a usable egress path to North-Exit
using
  passed

achieve all minimal
  Isolation-Room has a usable egress path to North-Exit
using
  passed
```

These are not cardinality variants of ordinary selection. They navigate proof
and intervention structure.

## 10. Models and Revisions become forms, not declared kinds

Ground clauses admitted in the current authoring or revision context are
assertions by default. Do not wrap them in `declare` unless the authority
distinction must be visible.

A successor Revision is recognizable from exact ancestry and signed deltas:

```clause
door-101-withdrawn from egress
  - Door 101 passed Fire-Marshal Inspection
```

No `Revision`, `from:`, or `withdraw:` scaffolding is needed.

An additive candidate is equally direct:

```clause
isolation-route-candidate from egress
  + Door 105 passed Fire-Marshal Inspection
```

The semantic core still records exact base Revision, exact admitted or
withdrawn clauses, content-derived identity, proof changes, provenance, and
bounded completeness status. The surface merely stops restating the category
the structure already proves.

### 10.1 Preserve epistemic moods

Surface compression must not collapse authority distinctions.

```clause
observe
  build-host supports wasm
via
  probe! build-host

assume
  target supports threads
within
  require
    worker-pool is safe

intend
  North materializes under wasm as "build/North.wasm"
```

An observation, assumption, intention, receipt, and admitted assertion remain
different checked judgments.

## 11. State is relational succession, not field mutation

A game state is a set of indexed propositions at a simulation boundary. It is
not an object graph whose fields are imperatively assigned.

Use `~>` between complete old and new clauses:

```clause
player position ?position ~>
  player position (?position + velocity of player * ?dt)
```

This means: in the successor state, replace the matched `position` proposition
with the new `position` proposition.

For a status change:

```clause
coin state active ~>
  coin state collected
```

This is not equality and not assignment. It is temporal succession.

### 11.1 Transitions occur under events

```clause
on frame ?dt
  player position ?position ~>
    player position (?position + velocity of player * ?dt)
```

The event shape binds `?dt` for the transition scope.

### 11.2 Functional relations make replacement concise

A contract such as:

```clause
Player
  position -> Vec2
```

establishes that each player has one current `position` value in a state.
Therefore `~>` can compile to keyed replacement.

For multi-valued relations, use exact deltas:

```clause
on KeyLeft pressed
  + player intends movement left

on KeyLeft released
  - player intends movement left
```

### 11.3 Transition blocks are transactions

All matching, guards, candidate deltas, conflict checks, and replacements in one
event phase should be evaluated against a defined pre-state and committed as one
successor state.

Never let incidental source order resolve multiple writes to the same
functional relation. Require one of: unique writer; explicit phase; declared
merge; or rejection as conflict.

Effects written inside an `on` block run after the successor state commits and
observe the post-state by default. A different phase must be named explicitly;
it must never fall out of line order.

### 11.4 Runtime representation remains free

The compiler may implement state with mutable arrays, sparse sets, structures
of arrays, or JavaScript objects inside a tick. The observable semantics remain
immutable successor states and exact deltas.

## 12. Effects remain explicit and produce receipts

Suffix `!` marks an operation that crosses the semantic world into an external
capability:

```clause
render! scene
load! "assets/coin.glb"
play! collect-sound
```

Defining an effect result name is legitimate:

```clause
receipt := render! scene
```

The result is a receipt or resource handle, not proof that the intended
external condition permanently holds.

A pure game program should normally derive a render plan relationally:

```clause
scene includes sprite "player" at position of player

scene includes sprite "coin" at position of coin if
  coin state active
```

The Three.js adapter realizes that plan:

```clause
render! scene
```

The browser and Three.js path should still be JavaScript-first. Wasm should
specialize measured pure kernels later. This proposal changes the surface
ontology, not that target strategy.

## 13. Complete semantic specimen: a door model

### Vocabulary and categories

```clause
Space
  Cellar
  Armory

DoorState
  locked
  unlocked

Door
  state -> DoorState
```

### Relation schema

```clause
connects:
  door: Door connects origin: Space to destination: Space
```

### Grounded semantic node and claims

```clause
iron-door
  Door
  connects Cellar to Armory
  state: locked
```

### Query

```clause
?door connects Cellar to Armory
```

### Boolean existence test

```clause
any ?door connects Cellar to Armory
```

### Transition

```clause
on unlock iron-door
  state of iron-door := locked ~>
    state of iron-door := unlocked
```

### Explicit flattening

The authored focus block is exactly equivalent to:

```clause
iron-door : Door
iron-door connects Cellar to Armory
state of iron-door := locked
```

No record, constructor, instance, field, or object is part of the meaning.

## 14. Complete semantic specimen: hospital egress

```clause
Space
  ICU-A
  East-Corridor
  West-Corridor
  North-Exit
  Isolation-Room

Door
  Door 101
  Door 102
  Door 103
  Door 104
  Door 105
  Door 106

Inspection
  Fire-Marshal Inspection

connects:
  door: Door connects origin: Space to destination: Space
  door origin -> destination*

passed:
  door: Door passed inspection: Inspection
  door -> inspection*

route:
  origin: Space has a usable egress path to destination: Space
  origin -> destination*

Door 101
  connects ICU-A to East-Corridor
  passed Fire-Marshal Inspection

Door 102
  connects East-Corridor to North-Exit
  passed Fire-Marshal Inspection

Door 103
  connects ICU-A to West-Corridor
  passed Fire-Marshal Inspection

Door 104
  connects West-Corridor to North-Exit
  passed Fire-Marshal Inspection

Door 105
  connects Isolation-Room to East-Corridor

Door 106
  connects Isolation-Room to West-Corridor

?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?destination
  ?door passed Fire-Marshal Inspection

?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?intermediate
  ?door passed Fire-Marshal Inspection
  ?intermediate has a usable egress path to ?destination

ICU-A has a usable egress path to ?destination

why
  ICU-A has a usable egress path to North-Exit

door-101-withdrawn from egress
  - Door 101 passed Fire-Marshal Inspection

diff egress -> door-101-withdrawn
```

Compared with the current surface, this removes:

- `Space: Type`;
- `egress/connects: Relation`;
- braces around role holes;
- `mode`;
- `egress: Model`;
- brackets around `Door 101`;
- `: Law`;
- `: Revision`;
- `from:` and `withdraw:`;
- `find all` around a clause that already contains a hole.

It preserves stable relation identities, named roles, participant domains,
recursive laws, operational cardinality, exact Revision ancestry, explanation,
intervention, and diff semantics.

## 15. Complete game specimen: one-coin collection

This is the first game surface Clause should try to make real.

```clause
requires
  game
  three

Vec2
  x: F32
  y: F32

CoinState
  active
  collected

Player
  position -> Vec2
  velocity -> Vec2
  radius -> F32
  score -> Int

Coin
  position -> Vec2
  radius -> F32
  value -> Int
  state -> CoinState

player
  Player
  position (0, 0)
  velocity (0, 0)
  radius 12
  score 0

coin
  Coin
  position (120, 40)
  radius 8
  value 10
  state active

distance between ?a and ?b:
  length(position of ?a - position of ?b)

?a overlaps ?b if
  distance between ?a and ?b < radius of ?a + radius of ?b

player collects coin if
  coin state active
  player overlaps coin

on frame ?dt
  player velocity ? ~>
    player velocity (input movement * 300)

  player position ?position ~>
    player position (?position + velocity of player * ?dt)

  coin state active ~>
    coin state collected
  if
    player collects coin

  player score ?score ~>
    player score (?score + value of coin)
  if
    player collects coin

  receipt: render! scene

scene includes sprite "player" at position of player

scene includes sprite "coin" at position of coin if
  coin state active
```

This specimen is intended to prove that:

- packages enter through `requires`;
- `Vec2` is a value shape, not a semantic object category;
- `Player` and `Coin` are categories with relation contracts, not classes with
  fields;
- `player` and `coin` are grounded symbols with co-equal claims;
- value projections derive from single-valued relations;
- collision is an ordinary recursive definition or law;
- state changes are clause-to-clause transitions;
- rendering is an explicit effect over a derived relational scene;
- the compiler remains free to lower relational state into efficient ECS-like
  storage.

The first implementation may simplify phase semantics or use a fixed tick, but
it must not retreat to object-field syntax in order to make the game executable.

## 16. The explicit structural escape hatch

Aggressive inference requires a precise fallback.

Every surface form must have a canonical role-labelled structural rendering,
conceptually like:

```text
clause connects
    door: iron-door
    origin: Cellar
    destination: Armory
```

or:

```text
member-of
    member: iron-door
    category: Door
```

This form is for ambiguity repair, compiler bootstrap, machine interchange,
diagnostics, agent edits, schema migration, proof inspection, and structural
diffs. It is not the default human surface.

Stable semantic IDs should normally remain hidden and be maintained by the
Store and editor transaction history. Renaming a phrase or label should not
create a new relation identity unless the author deliberately creates one.

## 17. Parser and elaborator requirements

The implementation should be built around a structured reader and semantic
elaborator, not a growing collection of line-specific parsers.

### Stage A — layout reader

Produce a lossless tree of lines, indentation groups, delimiters, literals,
names, punctuation, and source spans. Do not decide object, type, or relation
semantics here.

### Stage B — block classification

Classify homogeneous blocks as enumeration; definition or derived shape;
focused forms or
contracts; definition or law; query; event or transition; revision delta; or
epistemic/effect mood. Reject structurally unresolved forms or require the
formatter to split them.

### Stage C — recursive phrase resolution

Resolve exact scoped mixfix shapes. Every role accepts a recursive term or
proposition of its expected domain.

Resolution may use lexical scope, declared or imported phrase shapes,
participant domains, precedence and associativity, surrounding mood, and the
current focus role. It may not use probabilistic NLP.

### Stage D — role-labelled elaboration

Elaborate all sugar into stable semantic nodes: referent identities, membership
claims, relation identity, named participant roles, definitions, laws, query
projections, transitions, exact deltas, and effect requests.

### Stage E — existing semantic core

Where possible, lower the new surface into current Clause Models, Laws,
Revisions, requests, proof structures, and Rust projection before redesigning
the core.

Add new IR only where the old core cannot honestly represent recursive value
terms, local definitions, state transitions, effects and resources, or
JavaScript interoperation.

## 18. Migration from current Clause

Do not keep the old surface indefinitely as a second first-class language.
Implement the new surface as a profile, prove parity, provide a formatter or
codemod, then remove the ceremonial forms unless a real consumer requires them.

| Current | Replacement |
| --- | --- |
| `Space: Type` | `Space : Type` classification, or a bare grounded term when no classification is intended |
| `thing ∈ Space` | `thing : Space`; report the membership migration |
| `name: Relation` | `name : Relation`; inferred phrase schema may omit it |
| `{role: Type}` | `role : Type` inside schema only |
| `mode ...` | arrow and cardinality contract |
| `name: Model` | authoring or revision context inferred or externally named |
| `[Door 101]` | `Door 101` |
| `name: Law` | `name : Law`, or conclusion `if` premises with separate identity |
| `name: Revision` | `name from base` plus signed clauses |
| `from:` | `from` in the revision header; current executable profile remains migration evidence |
| `withdraw:` | `-`; current executable profile remains migration evidence |
| `declare:` | naked ground claims in admission context; current executable profile remains migration evidence |
| `find all ?x` | naked hole clause or `select` |
| `use game` | `requires` block |
| object-like `property: value` claims | relational `property value` under focus |
| `name: value` used as a binding | `name := value` definition |

The migration tool should preserve semantic IDs and print a report for every
inference it made.

## 19. Non-negotiable acceptance tests

The new surface is not accepted merely because examples look attractive.

### Relational honesty

- `iron-door` focus form and its expanded membership claim, ordinary claim, and
  focused definition produce identical checked semantics while retaining their
  three distinct semantic forms.
- No semantic node created by a focus block contains child fields or owned
  nested records.
- `iron-door : Door` elaborates to ordinary membership and never to definition,
  equality, primitive typing, or field ownership.

### Block determinism

- all-bare enumeration lowers child-to-parent membership;
- definition/shape blocks remain derived relational views, not object schemas;
- any non-bare child makes the block a focused block;
- bare children inside a focused block classify the focus;
- `name := value` inside a focused block defines `name of focus`, not a field or
  graph edge;
- the formatter separates enumeration blocks from contract or claim blocks and
  warns before edits that would reclassify an existing block.

### Hole semantics

- `? likes Chess` returns one anonymous column of role assignments;
- `?person likes Chess` names that column;
- repeated `?name` occurrences correlate by identity;
- each anonymous `?` is fresh;
- `any` returns `Bool` and does not select a random witness.

### Recursive structure

- relation roles accept nested terms;
- canonical formatting preserves grouping;
- ambiguity diagnostics name candidate shapes and role conflicts;
- an explicit role-labelled rendering round-trips exactly.

### Identity and revisions

- layout changes do not alter semantic identity;
- hiding or removing declaration-kind words does not alter Revision content;
- labels and spellings can change without replacing stable identities when
  edited as a rename;
- signed deltas apply to exact bases and retain current completeness behavior.

### General-purpose viability

- the one-coin game compiles to generated JavaScript and runs through Three.js;
- state transitions are deterministic under replay;
- render effects return receipts;
- functional relations lower to efficient direct storage rather than generic
  runtime triples;
- source maps return errors to Clause relation roles and focus blocks.

## 20. Implementation sequence

### Milestone 0 — Freeze the constitution and corpus

Before changing the parser, add golden examples for enumeration, derived shape
views, focused graph claims, explicit flattening, relation schemas, recursive
terms, holes and correlation, laws, queries, revisions, transitions, effects,
the hospital program, and the one-coin game.

For every example retain source, grouped tree, elaborated role graph, canonical
rendering, diagnostics, and expected result. Include positive `x : Group`
classification and `name := term` definition, retired `∈`, `::`, and `in`
contrasts, canonical two-space projection, rejected tabs and noncanonical
four-space projection, and `state := locked` under focus. Oracles distinguish
definition from membership claim, ordinary claim, and object field.

### Milestone 1 — New layout and focus profile

Implement bare symbol grounding, enumeration blocks, definition or derived
shape blocks,
focused blocks, explicit flattening display, and multiword semantic names
without brackets. Lower into the current semantic core.

### Milestone 2 — Compact relation schemas

Implement schema role patterns, named roles, focused role designation,
arrow/cardinality contracts, ambiguity diagnostics, and stable hidden relation
identities. Remove required `Relation` and `mode` syntax in the new profile.

### Milestone 3 — Recursive term grammar

Permit every role to contain recursive terms with explicit grouping and
canonical formatting. Add derived value shapes, pure definitions, projections,
definitions. This milestone recovers the recursive-relational thesis.

### Milestone 4 — Holes, rules, and relational selection

Implement `?` and `?name` holes, repeated-hole correlation, naked single-clause
selection, `select` projection blocks, `any` existence, `select one` and
`select first`, `if` law inference, and hidden or optional law labels. Replace
`find` in canonical examples.

### Milestone 5 — Revision surface reset

Implement `name from base`, `+` and `-` clauses, current `why`, `prevent`,
`achieve`, and `diff` parity, and migration from current Model, Law, and Revision
syntax.

### Milestone 6 — State transitions

Add `on` event scopes, `~>` clause succession, keyed replacement for functional
relations, explicit delta fallback, conflict analysis, and deterministic tick
and replay runtime.

### Milestone 7 — Effects and JavaScript target

Add the `!` effect boundary, receipts and opaque resources, `requires` packages
and capabilities, generated JavaScript ES modules, source maps, the Three.js
adapter, and the one-coin vertical slice.

### Milestone 8 — Retire ceremonial syntax

After parity and migration, format existing examples into the new surface,
remove duplicated old grammar paths, retain only the explicit structural
interchange/debug form, and update README and the language constitution around
the relational model.

## 21. What Clause must not become

### A natural-language parser

Phrases are exact declared grammar, not statistical English interpretation.

### An object language with prettier property syntax

Ordinary focused claims remain co-equal claims. Focused classification and
definition retain their distinct semantic forms; none creates objects with
fields.

### A generic triple interpreter in the hot path

The semantic view is relational. Physical storage is specialized and compiled.

### A language that hides ambiguity

Inference is allowed only when one checked elaboration survives. Otherwise
require explicit structure.

### A universal logic search engine

Holes, reverse projections, and enumeration execute only under admitted,
bounded strategies.

### A denial of procedures

Clause should minimize authored control flow where relations, dataflow, and
transitions determine order. It retains an explicit `do` stratum for algorithms
whose order is the meaning.

### A system that confuses effects with truth

`render! scene` produces a receipt. It does not prove the world permanently
contains a rendered scene.

### A system that persists a generic graph copy every frame

`StateRevision` is semantic. Runtime representation and retention policy remain
optimized.

## 22. Final mandate

Treat this document as a surface reset, not a patch list.

Implementation proceeds from these invariants:

1. Bare terms may denote addressable referents; the term is not the referent.
2. Categories emerge through ordinary membership claims and contracts; they
   are not a primitive `Type` domain.
3. Referents do not acquire fields merely from layout.
4. `:` is classification/membership sugar; `:=` is definition/denotation.
5. Enumeration, definition/shape, and focused blocks are structurally distinct.
6. Indentation is erasable projection over semantic forms; it does not collapse
   membership claims, ordinary claims, and definitions.
7. Relation phrases are exact, role-labelled, recursively compositional
   grammar.
8. `?` is a hole; `?name` is a named or reusable hole.
9. A naked hole-bearing clause selects all matching role assignments.
10. `select` projects; `any` tests existence; random selection is explicit.
11. `=` is equality.
12. `->` is production or projection.
13. `~>` is state succession between complete clauses.
14. `+` and `-` are exact deltas.
15. `!` marks effects and effects produce receipts.
16. Declaration kinds are inferred from form wherever unique.
17. Stable semantic identities and named roles remain in the core even when
    hidden from ordinary source.
18. The first proof of generality is a generated-JavaScript Three.js game, not
    another ontology demo.

The standard is not “less syntax than current Clause.”

The standard is:

> **The shortest surface that states the semantic structure without lying
> about category, identity, time, search, or effect.**

That is the Clause surface worth building.
