# Clause Surface Reset

## Relational authoring without object smuggling

**Status:** provisional direction; strongly opinionated replacement proposal;
still undergoing revision

**Audience:** Clause implementers, reviewers, formatter/tooling authors, and
corpus designers

**Authority:** [ROADMAP.md](ROADMAP.md) remains the normative execution and
milestone authority. This file is an editorially normalized tracked copy of the
operator's current surface draft, so decisions do not depend on an external
document. Its relational invariants control direction over earlier
human-surface recommendations. Its exact punctuation and specimens remain
revisable until M0 freezes them through corpus and elaboration evidence, except
for three controlling design rulings already fixed here: membership is only
`x ∈ Y`; `:` is stable-handle binding and never membership; and canonical
layout is exactly two spaces per level, spaces only. None of these rulings is a
claim that the current parser implements the reset profile.

**Scope:** this supersedes the human-surface recommendations and specimens in
the earlier strategy packet. It does not discard Clause's semantic core,
immutable Revisions, role-labelled n-ary relations, provenance, explanation,
interventions, or target strategy.

## 1. The decision

Clause should stop asking authors to declare an ontology for the language
before they can describe the domain.

The canonical surface should be organized around five authored things:

1. **grounded symbols** — names admitted into the program;
2. **bindings** — names oriented to values, terms, or structural domains;
3. **membership** — one semantic thing belonging to another;
4. **clauses** — role-labelled relations among co-equal participants;
5. **moods** — assertion, derivation, query, transition, effect, explanation,
   and intervention.

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
iron-door: Door
  connects: Cellar to Armory
  state: locked
```

That form smuggles in four ideas Clause does not need:

- `iron-door` is an object instantiated from `Door`;
- `Door` owns a record schema;
- `connects` and `state` are fields;
- indentation denotes nested storage or ownership.

The canonical relational form is:

```clause
iron-door
  Door
  connects Cellar to Armory
  state: locked
```

Its exact elaboration is:

```clause
iron-door ∈ Door
iron-door connects Cellar to Armory
state of iron-door: locked
```

The three children do not smuggle in object fields, but neither do they collapse
to one undifferentiated edge form. Bare `Door` is focus membership sugar;
`connects Cellar to Armory` is an ordinary role-labelled relational claim; and
`state: locked` binds the focused `state of iron-door` projection handle/current
binding to `locked`. The block avoids repeating the designated focus while each
child retains the semantics proved by its own form.

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

Retract `:=` as a second binding operator. `:` is binding. `=` is equality. Do
not introduce two binding glyphs.

## 3. The ontology the surface should expose

Clause still has a type system internally. It should not force authors to
describe every domain concept as a type declaration.

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

A user-defined structural value domain is inferred from a homogeneous binding
block:

```clause
Vec2
  x: F32
  y: F32
```

This means `Vec2` is a value shape with labels `x` and `y`, each bound to
`F32`. It does **not** mean every semantic thing in Clause is a record.

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

A category is not a separate ontological species. It is an ordinary semantic
set used on the right of membership and, optionally, as the focus of contracts.

```clause
Game
  Chess
  Soccer
```

means:

```clause
Chess ∈ Game
Soccer ∈ Game
```

A member may itself be a category:

```clause
Chess
  Blitz
  Rapid
  Classical
```

Therefore `Chess` is both a member of the `Game` set and a semantic set for its
own members. Sets may themselves belong to sets. Clause should not force a
symbol into an exclusive “instance” or “type” box.

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

> **Value shapes may have bound labels. Semantic symbols have relations.
> Categories have relational contracts. Do not turn semantic entities into
> records merely because a backend may store them that way.**

## 4. `:` means binding

This must remain brutally simple:

```text
left: right
```

means: bind the name or term pattern on the left to the value, term, domain,
schema, or result on the right.

Examples:

```clause
gravity: 9.81
spawn: (0, 0)
receipt: render! scene
```

In a value-shape block:

```clause
Vec2
  x: F32
  y: F32
```

`x` is bound to the domain `F32` as a structural slot declaration.

A pure definition is also a binding:

```clause
distance between ?a and ?b:
  length(position of ?a - position of ?b)
```

The term pattern on the left is bound to the pure computation on the right.

An explicit relation schema may bind a stable human anchor to a phrase pattern:

```clause
connects:
  door: Door connects origin: Space to destination: Space
```

The inner colons bind role names to participant domains.

`:` must never mean:

- opens an indented block;
- is an instance of;
- expresses membership;
- has a field;
- assign a mutable property;
- equality;
- state transition.

Therefore `iron-door: Door` is not canonical membership syntax. If accepted at
all, it means an alias/binding and must never silently elaborate to
`iron-door ∈ Door`. There is no ASCII `in` alias, no `::` alias, and no colon
membership form. Canonical membership is only:

```clause
iron-door ∈ Door
```

Membership is a role-labelled semantic relation whose member and set operands
accept checked terms or patterns, including holes in laws and queries. The
surface gives that relation exactly one authored infix spelling.

Within a focus block, `state: locked` is canonical binding syntax. It binds the
focused projection handle/current binding `state of iron-door` to `locked`; it
does not elaborate to a generic `iron-door state locked` edge.

## 5. Layout has three inferred forms

Indentation should be semantically useful while remaining erasable. It must
never introduce ownership or nested object identity.

Canonical indentation is exactly two spaces per level, using spaces only. A
tab is rejected. A line whose leading-space width is not the exact width for
its structural depth is rejected with the observed width and expected depth;
the reader does not guess whether it meant a sibling or child. The canonical
formatter may normalize whitespace only after a structural tree has parsed
successfully. Canonical whitespace-only reformatting must not change elaborated
semantics or semantic identity.

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
Sicilian Defense ∈ Famous Chess Openings
Ruy Lopez ∈ Famous Chess Openings
Queen's Gambit ∈ Famous Chess Openings
```

This preserves the ordinary prior that an indented list contains members of
its heading. Reserved list forms declare their own expansion; for example,
`requires` relates the current program to each listed package rather than
asserting ordinary domain membership.

### 5.2 Binding/shape block

A homogeneous block of bindings defines a structural value shape or binding
namespace:

```clause
Vec2
  x: F32
  y: F32
```

The children do not become graph claims. They are bindings in the structural
definition of `Vec2`.

### 5.3 Focused block

A block containing relation fragments, bindings, contracts, transitions, or
other non-bare forms establishes its heading as the focus participant:

```clause
iron-door
  Door
  connects Cellar to Armory
  state: locked
```

It lowers to:

```clause
iron-door ∈ Door
iron-door connects Cellar to Armory
state of iron-door: locked
```

Within a focused block, a bare set name is membership sugar about the focus, a
relation fragment is an ordinary relational claim with the focus supplied to
its designated role, and a binding binds the corresponding focused projection
handle/current binding.

### 5.4 Mixed blocks have one deterministic reading

The presence of any non-bare child makes the block a focused block.
Therefore:

```clause
Thing
  A
  B
  relation value
  state: active
```

means:

```clause
Thing ∈ A
Thing ∈ B
Thing relation value
state of Thing: active
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
before an edit changes an all-bare enumeration block into a focused
block, because that edit changes the interpretation of the existing bare
lines.

This structural rule resolves the apparent conflict between `Game / Chess` and
`iron-door / Door / state: locked` without fuzzy English parsing and without
making membership direction depend on capitalization.

### 5.5 Layout equivalence is an invariant

For every focused block, the compiler must print a fully expanded sequence of
semantically typed judgments: membership, relational claim, or focused binding
according to each child form.

Moving between focused and expanded forms must not change:

- proposition or binding identity;
- participant roles and focused projection handles;
- provenance except source span;
- revision identity after canonical elaboration;
- query results;
- generated code.

## 6. The minimal symbolic vocabulary

Clause should adopt this vocabulary and reject synonyms that blur the axes.

| Form | Meaning |
| --- | --- |
| `∈` | membership; the only authored membership form |
| `:` | binding |
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

A public or ambiguous relation can use an explicit schema binding:

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

means: solve the same role and expose its bindings under the name `person`.

```clause
Alice likes ?opening
```

means: solve the opening role.

Each bare `?` is fresh. Repeating a named hole requires the same binding:

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

### 8.2 Optional human labels are bindings

Most laws should receive hidden stable semantic identities from the Store and
tooling. When a human name is useful:

```clause
recursive route:
  ?origin has a usable egress path to ?destination if
    ?door connects ?origin to ?intermediate
    ?intermediate has a usable egress path to ?destination
```

The colon binds the label `recursive route` to the law. It does not declare a
`Law` object.

### 8.3 Category-wide facts remain ordinary laws

Do not invent field defaults. To say every coin has radius 8:

```clause
?coin radius 8 if
  ?coin ∈ Coin
```

That is a universal relational law. It is not a default value installed into a
record schema.

A category contract and a universal fact remain distinct:

```clause
Coin
  radius -> F32

?coin radius 8 if
  ?coin ∈ Coin
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

Binding an effect result is legitimate:

```clause
receipt: render! scene
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
  iron-door state locked ~>
    iron-door state unlocked
```

### Explicit flattening

The authored focus block is exactly equivalent to:

```clause
iron-door ∈ Door
iron-door connects Cellar to Armory
state of iron-door: locked
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
- `player` and `coin` are grounded symbols whose focused children retain their
  membership, relation, or binding semantics by form;
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
clause
  relation: ∈
  member: iron-door
  set: Door
```

The latter is structural interchange, not a second authored membership form:
its `:` lines are bindings inside the debug record and its relation value is
the same `∈` identity. Ordinary authored membership remains only `x ∈ Y`.

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

Classify homogeneous blocks as enumeration; binding or shape; focused blocks
containing membership sugar, relational claims, or focused bindings; definition
or law; query; event or transition; revision delta; or epistemic/effect mood.
Reject structurally unresolved forms or require the formatter to split them.

### Stage C — recursive phrase resolution

Resolve exact scoped mixfix shapes. Every role accepts a recursive term or
proposition of its expected domain.

Resolution may use lexical scope, declared or imported phrase shapes,
participant domains, precedence and associativity, surrounding mood, and the
current focus role. It may not use probabilistic NLP.

### Stage D — role-labelled elaboration

Elaborate all sugar into stable semantic nodes: grounded identities, `∈`
membership clauses, relation identity, named participant roles, stable-handle
bindings including focused projection bindings, laws, query projections,
transitions, exact deltas, and effect requests.

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
| `Space: Type` | `Space` |
| legacy `thing: Space` where classification was intended | never reinterpret the colon; rewrite explicitly to `thing ∈ Space` or unambiguous list/focus membership sugar and report the inference |
| `name: Relation` | inferred phrase schema; optional `name:` schema binding |
| `{role: Type}` | `role: Type` inside schema only |
| `mode ...` | arrow and cardinality contract |
| `name: Model` | authoring or revision context inferred or externally named |
| `[Door 101]` | `Door 101` |
| `name: Law` | conclusion `if` premises; optional label binding |
| `name: Revision` | `name from base` plus signed clauses |
| `from:` | `from` in the revision header |
| `withdraw:` | `-` |
| `declare:` | naked ground claims in admission context |
| `find all ?x` | naked hole clause or `select` |
| `use game` | `requires` block |
| object-like `property: value` under focus | focused projection binding such as `state of thing: value`; never an owned field or generic relation edge |
| `:=` | `:` |

The migration tool should preserve semantic IDs and print a report for every
inference it made.

## 19. Non-negotiable acceptance tests

The new surface is not accepted merely because examples look attractive.

### Relational honesty

- the `iron-door` focus form and its expanded membership, relational claim, and
  focused binding produce identical checked semantics;
- No semantic node created by a focus block contains child fields or owned
  nested records.
- `iron-door: Door` never silently means membership.

### Block determinism

- all-bare enumeration lowers each child to `child ∈ heading`;
- binding blocks remain structural bindings, not graph assertions;
- any non-bare child makes the block a focused block;
- bare children inside a focused block classify the focus;
- the formatter separates enumeration blocks from contract or claim blocks and
  warns before edits that would reclassify an existing block.
- canonical indentation is exactly two spaces per level and spaces only;
- tabs reject with an exact diagnostic;
- noncanonical leading-space widths reject rather than guessing structure;
- formatting an already parsed tree to canonical whitespace preserves
  elaborated semantics and identity.

### Hole semantics

- `? likes Chess` returns one anonymous column of bindings;
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

Before changing the parser, add golden examples for enumeration, shape
bindings, focused graph claims, explicit flattening, relation schemas, recursive
terms, holes and correlation, laws, queries, revisions, transitions, effects,
the hospital program, and the one-coin game.

For every example retain source, grouped tree, elaborated role graph, canonical
rendering, diagnostics, and expected result. Every example uses exactly two
spaces per level. The corpus includes nested-depth formatting/round-trip cases,
tab rejection, explicit one-, three-, and four-space rejection at a depth that
expects two spaces, and other noncanonical-width rejection with observed width
and exact expected depth. It also includes ordinary-term and hole-pattern `∈`
membership in laws, a focus/expanded oracle that distinguishes membership,
relation, and binding judgments, whitespace-only canonical-reformat identity
parity, and legacy-colon classification migration that writes and reports
explicit `∈` rather than reinterpreting `:`.

### Milestone 1 — New layout and focus profile

Implement bare symbol grounding, `∈` membership, enumeration blocks, binding or
shape blocks, focused blocks with membership sugar, relational claims, and
focused projection bindings, explicit flattening display, and multiword
semantic names without brackets. Lower into the current semantic core.

### Milestone 2 — Compact relation schemas

Implement schema pattern bindings, named roles, focused role designation,
arrow/cardinality contracts, ambiguity diagnostics, and stable hidden relation
identities. Remove required `Relation` and `mode` syntax in the new profile.

### Milestone 3 — Recursive term grammar

Permit every role to contain recursive terms with explicit grouping and
canonical formatting. Add value shapes, pure bindings, projections, and
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

Focused relational claims are co-equal clauses; focused bindings remain
distinct binding judgments. Neither creates objects with fields.

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

1. Bare names ground semantic symbols.
2. Categories are ordinary semantic sets; membership is only `x ∈ Y`, and sets
   may themselves belong to sets.
3. Semantic entities do not have fields.
4. `:` means stable-handle binding and nothing else, including focused
   projection/current bindings.
5. Enumeration, binding, and focused blocks are structurally distinct; focused
   children retain membership, relation, or binding semantics by form.
6. Indentation is erasable projection sugar, exactly two spaces per level,
   spaces only; tabs and noncanonical widths reject.
7. Relation phrases are exact, role-labelled, recursively compositional
   grammar.
8. `?` is a hole; `?name` is a named or reusable hole.
9. A naked hole-bearing clause selects all bindings.
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
