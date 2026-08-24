# Clause Surface Reset

> **Status:** Current target surface; native parser migration is incomplete.
>
> **Authority:** Normative for authoring syntax and canonical formatting,
> governed by the [semantic foundation](foundation.md).
>
> **Relationship:** Supersedes the ceremonial declaration surface and the
> punctuation ruling recorded by [historical M0](history/m0.md). The
> [roadmap](roadmap.md) governs implementation order.

## Distinction-first relational authoring

Clause has one semantic domain: addressable referents. A referent is a
stabilized distinction that can be reidentified across relational content,
assertion occurrences, and Revisions; structural equality never collapses that
identity. Relations are referents in relational position. N-ary relational
content assigns every participant to a stable named role. An assertion
occurrence is the scoped source act that commits to content, while judgment and
modal authority remain separate. Terms project referents but are not themselves
referents.

**Audience:** Clause implementers, reviewers, formatter/tooling authors, and
conformance-suite designers

This file controls authoring projection. `Chess ∈ Game` is ordinary
membership relational content; `gravity: 9.81` is a stable binding. Neither
spelling creates a primitive type or value domain. `∈` is canonical source.
A human-facing editor may transform typed `::` into `∈`, but raw `::` is not
Clause grammar. Canonical indentation is two spaces, spaces only, and tabs are
diagnosed.

**Scope:** this supersedes the human-surface recommendations and specimens in
the earlier strategy packet. It does not discard Clause's semantic core,
immutable Revisions, role-labelled n-ary relations, provenance, explanation,
interventions, or target strategy.

## 1. The decision

Clause should stop asking authors to declare an ontology for the language
before they can describe the domain.

The canonical surface must preserve these semantic commitments:

1. **referents** — stabilized, addressable distinctions;
2. **relational content** — a relation referent and participants assigned to
   explicit named roles, including membership content;
3. **assertion occurrences and judgments** — respectively the scoped act that
   commits to content and a separate authority's status about content or an
   occurrence;
4. **semantic modes** — universal law, oriented derivation rule,
   Revision-admission invariant, goal, observation, requirement, intention,
   transition, and effect remain distinct;
5. **definitions** — explicit denotation from a term to what it designates.

Source terms name or compose these commitments but do not create another
semantic domain. Absence is undetermined rather than denial. Acceptance,
status, authority, and occurrence provenance never become relational content.

The decisive surface thesis is:

> **When realized, the canonical authoritative Model is the Clause program: a
> graph of referents, named-role relational content, assertion occurrences,
> judgments, and explicit modes. Layout is a projection over that Model and
> never creates objects, fields, ownership, or containment. A Revision is
> immutable version and lineage evidence, not the program.**

The decisive implementation thesis is:

> **Infer the authored category from the form itself. Require a keyword only
> when the form does not already prove what it is.**

This means the ordinary surface should not require `Type`, `Relation`, `Law`,
`Model`, `Revision`, `entity`, `class`, `struct`, `let`, or a shape prefix
before every clause.

It also means we must stop writing relational entities as records.

Reject this as canonical Clause:

```clause
iron-door ∈ Door
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

The first form elaborates to ordinary membership relational content, the second is
ordinary co-equal relational content, and the third is a definition. Focus
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

The body permits a derived structural view. It does not prove a primitive
value species. The word `type` adds no information, and a colon after `Vec2`
would merely open a block, which is not a legitimate use of `:`.

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

`player` is a grounded symbol. `Player`, `position`, and `velocity` contribute
relational content about it. Nothing is instantiated and no field is assigned.

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

Use `:` for binding/definition, `∈` for membership, and `=` for equality.
Identity remains distinct from all three.

## 3. The ontology the surface should expose

Clause implementations may derive a type-system view. The surface must not
force authors to declare types as a second semantic domain.

The surface may expose three derived views without turning them into semantic
domains or keyword ceremony.

### 3.1 Representation and structural views

Examples:

```clause
F32
Int
Bool
String
```

These names address referents with built-in representation and operation
contracts. They are not members of a second value domain.

A structural view may be inferred from a homogeneous binding block:

```clause
Vec2
  x: F32
  y: F32
```

This binds the shape labels `x` and `y` to the `F32` representation domain
under the `Vec2` projection. It does not create owned fields or add a primitive
value or record domain. Any derived structural view remains replaceable.

### 3.2 Semantic symbols

Examples:

```clause
Player
Game
Door
iron-door
Chess
```

A declaration-position bare name may ground a semantic distinction in the
program. That act does not automatically create roles, constraints, fields, a
constructor, or immediate classification. A use-position name must resolve an
existing referent unless explicitly fresh or variable.

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

> **Structural views may project named relational roles. Semantic symbols
> participate in relations. Categories may have explicit relational contracts.
> Do not turn semantic entities into records merely because a backend may store
> them that way.**

## 4. Binding and membership are different

The two forms are deliberately separate:

```clause
Chess ∈ Game
gravity: 9.81
```

`∈` is an ordinary membership relation whose member and group are named
roles; it does not bind a name, declare a primitive type, or assert equality.
`:` establishes a stable binding or definition: a source handle is oriented to
what it designates. Binding does not collapse the term into the referent and
does not confer acceptance or authority on relational content.

Examples:

```clause
spawn ∈ Position
spawn: (0, 0)
receipt: render! scene
```

The first line relates `spawn` to a category; the second binds it. A structural view may
use both without creating an object or record ontology:

```clause
Vec2
  x: F32
  y: F32
```

A pure definition uses `:`:

```clause
distance between ?a and ?b:
  length(position of ?a - position of ?b)
```

An explicit relation schema retains named roles and classifies their referents:

```clause
connects:
  door: Door connects origin: Space to destination: Space
```

`∈` is canonical source. Editors may transform typed `::` to `∈` before
parsing, while formatters and agents emit `∈` directly. Raw `::` and word
aliases are never a second live grammar.

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
Sicilian Defense ∈ Famous Chess Openings
Ruy Lopez ∈ Famous Chess Openings
Queen's Gambit ∈ Famous Chess Openings
```

This preserves the ordinary prior that an indented list contains members of
its heading. Reserved list forms declare their own expansion; for example,
`requires` relates the current program to each listed package rather than
asserting ordinary domain membership.

### 5.2 Binding/derived-shape block

A homogeneous block of bindings can project a derived structural view:

```clause
Vec2
  x: F32
  y: F32
```

Each child is a binding. The optional structural view derives from those
labelled domains; it is not a primitive record schema.

### 5.3 Focused block

A block containing relation fragments, contracts, transitions, or other clause
forms establishes its heading as the focus participant:

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

Within a focused block, a bare category name relates the focus through the
ordinary membership relation, an ordinary relation fragment supplies the focus
role, and `name: value` binds a stable focused term. These forms remain
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
Thing ∈ A
Thing ∈ B
Thing relation value
```

It does **not** enumerate `A` and `B` beneath `Thing`.

When a category needs both member enumeration and contracts or relational content, write
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
relational-content block, because that edit changes the interpretation of the existing bare
lines.

This structural rule resolves the apparent conflict between `Game / Chess` and
`iron-door / Door / state: locked` without fuzzy English parsing and without
making membership direction depend on capitalization.

### 5.5 Layout equivalence is an invariant

For every focused block, the compiler must be able to print its fully expanded
semantic forms without collapsing membership relational content, ordinary relational content, or
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
| `:` | stable binding/definition |
| `∈` | ordinary membership relation |
| `=` | equality proposition |
| `>`, `<`, `>=`, `<=`, `!=` | conventional comparisons |
| spaced infix `+`, `-`, `*`, `/` | conventional arithmetic relations |
| `->` | result/projection direction or return contract |
| `?` | anonymous clause hole |
| `?name` | named clause hole |
| `~>` | a proposition in the current state succeeds to another proposition |
| leading `+` in a Delta | exact admission/addition |
| leading `-` in a Delta | exact withdrawal/removal |
| `!` | external effect/capability boundary |

Use these ASCII operators directly when their conventional relation and role
orientation are unambiguous:

```clause
x > y
x < y
x >= y
x <= y
x = y
x != y
a + b
a - b
a * b
a / b
```

They elaborate to addressable relation referents and stable named-role
relational content just like word-shaped relations. Do not print
`x greater-than y` or similar word expansions as canonical source. Do not
invent punctuation where conventions conflict or do not determine the intended
roles; domain relations such as `connects` and `parent-of` remain words with
exact declared shapes. Structurally leading `+` and `-` remain Delta signs.
Slash-qualified semantic names remain names; they are not silently parsed as
division.

Words carry semantic moods:

| Word | Meaning |
| --- | --- |
| `law` | explicit universally generalized content mode |
| `if` | oriented derivation-rule body |
| `invariant` | candidate-Revision admission gate |
| `goal` | desired content without current-truth or derivation authority |
| `on` | event/time-triggered transition scope |
| `from` | exact revision ancestry |
| `requires` | program/package requirements |
| `select` | explicit relational projection |
| `any` | existential truth test |
| `why` | explanation query |
| `prevent` | counterfactual withdrawal synthesis |
| `achieve` | counterfactual addition synthesis |
| `diff` | revision comparison |
| `observe` | evidence-backed observation mood |
| `assume` | scoped premise mood |
| `require` | proof obligation mood |
| `intend` | actor intention mood, distinct from a goal |
| `do` | explicitly procedural stratum |

`find` is not a core relational mood. It may remain an ordinary library or
tooling verb for text, file, symbol, or repository search, where its conventional
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

A public or ambiguous relation can use an explicit relation-contract form. The
following structural spelling is provisional; the mode and named-role content
are authoritative:

```clause
relation contract connects
  role door: Door
  role origin: Space
  role destination: Space
  phrase {door} connects {origin} to {destination}
  project door origin -> destination*
```

This says:

- stable human anchor: `connects`;
- focused or subject role: `door`;
- participant domains: `Door`, `Space`, `Space`;
- phrase: `{door} connects {origin} to {destination}`;
- operational projection: given `door` and `origin`, produce zero or more
  `destination` values.

`relation contract` states a mode; it does not classify `connects` into a
primitive `Relation` species. The final spelling remains an M2 decision.

The checked core still retains relation identity, named roles, role domains,
surface pattern, and voice/cardinality contract.

The contract describes the `connects` referent; it does not execute it.
Relations, rules, Revisions, and evaluation may appear as participants in other
content, but interpreting them requires an admitted shape and mode and may
require quotation, stratification, or an exact Revision boundary. One referent
domain never grants self-execution.

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
`position` is an ordinary relation with its own identity, roles, universal
laws, derivation rules, queries, and strategies. Reserve dot syntax for
explicit foreign-host interoperation,
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

### 8.1 Derivation rules are oriented by `if`

```clause
?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?destination
  ?door passed Fire-Marshal Inspection
```

The conclusion plus `if` body proves that this is an oriented derivation rule.
It does not prove a universal law, invariant, or goal.

A recursive derivation rule is equally direct:

```clause
?origin has a usable egress path to ?destination if
  ?door connects ?origin to ?intermediate
  ?door passed Fire-Marshal Inspection
  ?intermediate has a usable egress path to ?destination
```

### 8.2 Optional human labels are definitions

Rules receive stable semantic identities independently of a human label. When
a human name is useful:

```clause
recursive route:
  ?origin has a usable egress path to ?destination if
    ?door connects ?origin to ?intermediate
    ?intermediate has a usable egress path to ?destination
```

The definition orients the label `recursive route` to the derivation rule. It
does not classify a `Law` object or enter the rule's premise or conclusion
content.

### 8.3 Universal law, derivation rule, invariant, and goal remain distinct

Do not invent field defaults. This oriented rule derives a radius consequence
for every matched coin:

```clause
?coin radius 8 if
  ?coin ∈ Coin
```

That is a derivation rule with a universally matchable pattern. It is not, by
that fact alone, a universal law and is not a default value installed into a
record schema.

A universal law generalizes content within an explicit scope. A derivation rule
authorizes oriented consequence production. An invariant gates admission of a
candidate Revision. A goal describes desired content without asserting current
truth or authorizing derivation. Their final surface spellings remain separate
M0 evidence questions; an implementation may not accept a collapsed mode.

A category contract and a derivation rule remain distinct:

```clause
Coin
  radius -> F32

?coin radius 8 if
  ?coin ∈ Coin
```

## 9. Queries begin with the relation

`find` should not be the default relational query word. It carries a tooling
search prior and makes the result variable syntactically primary.

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

Ground relational content in the current authoring or Revision context creates
assertion occurrences by default. Do not wrap it in `declare` unless the
authority distinction must be visible.

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

An observation, assumption, requirement, intention, goal, receipt, and admitted
assertion occurrence remain different checked modes or judgments. A goal does
not assert current truth, an intention does not make its goal true, and a
receipt does not admit relational content.

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
relation contract connects
  role door: Door
  role origin: Space
  role destination: Space
  phrase {door} connects {origin} to {destination}
```

### Grounded relational content

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
  state of iron-door: locked ~>
    state of iron-door: unlocked
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

relation contract connects
  role door: Door
  role origin: Space
  role destination: Space
  phrase {door} connects {origin} to {destination}
  project door origin -> destination*

relation contract passed
  role door: Door
  role inspection: Inspection
  phrase {door} passed {inspection}
  project door -> inspection*

relation contract route
  role origin: Space
  role destination: Space
  phrase {origin} has a usable egress path to {destination}
  project origin -> destination*

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

Compared with the pre-M1 ceremonial surface and the ceremony retained for
later milestones, this removes:

- `Space: Type`;
- `egress/connects: RelationShape`;
- braces around role holes;
- `mode`;
- `egress: Model`;
- brackets around `Door 101`;
- `: DerivationRule`;
- `: Revision`;
- `from:` and `withdraw:`;
- `find all` around a clause that already contains a hole.

It preserves stable relation identities, named roles, participant domains,
recursive derivation rules, operational cardinality, exact Revision ancestry, explanation,
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
- `Vec2` supplies a derived structural view, not a semantic object or value
  species;
- `Player` and `Coin` are categories with relation contracts, not classes with
  fields;
- `player` and `coin` are grounded referents in co-equal relational content;
- structural projections derive from single-valued relations;
- collision is an ordinary recursive definition or derivation rule;
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
form connects
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
Model and editor transaction history. Renaming a phrase or label should not
create a new relation identity unless the author deliberately creates one.

Clause has no Store implementation. A future Store adapter treats Store as a
neutral substrate and adds a typed Clause envelope for referent terms,
relational content, occurrence attestations, judgments, modality, evidence,
authority, admission/rejection, supersession, and exact Revision-to-storage
lineage. Missing rows, retraction, query negation, equality, liveness, and Store
revision identity do not supply those semantics.

## 17. Parser and elaborator requirements

The implementation should be built around a structured reader and semantic
elaborator, not a growing collection of line-specific parsers.

### Stage A — layout reader

Produce a lossless tree of lines, indentation groups, delimiters, literals,
names, punctuation, and source spans. Do not decide object, type, or relation
semantics here.

### Stage B — block classification

Classify relational content/assertion occurrence, relation contract,
definition, universal law, oriented derivation rule, invariant, query, goal,
observation, requirement, intention, effect, transition, and delta separately
whenever the source structure proves the mode. If Stage B cannot prove one,
retain an explicit `Unresolved...` class for Stage C; never accept a collapsed
union as a semantic class.

### Stage C — recursive phrase resolution

Resolve exact scoped mixfix shapes. Every role accepts a recursive term or
proposition of its expected domain.

Resolution may use lexical scope, declared or imported phrase shapes,
participant domains, precedence and associativity, surrounding mood, and the
current focus role. It may not use probabilistic NLP.

### Stage D — role-labelled elaboration

Elaborate all sugar into stable semantic nodes: referent identities,
named-role relational content, assertion occurrences, judgments, definitions,
universal laws, derivation rules, invariants, goals, query projections,
transitions, exact deltas, and effect requests.

### Stage E — existing semantic core

Where possible, lower the new surface into current Clause model and rule representations,
Revisions, requests, proof structures, and Rust projection before redesigning
the core.

Add new IR only where the old core cannot honestly represent recursive value
terms, local definitions, state transitions, effects and resources, or
JavaScript interoperation.

## 18. Migration from pre-M1 Clause and retained ceremony

Do not keep the old surface indefinitely as a second first-class language.
Implement the new surface as a profile, prove parity, provide a formatter or
codemod, then remove the ceremonial forms unless a real consumer requires them.

| Current | Replacement |
| --- | --- |
| `Space: Type` | bare `Space` grounding, or `Space ∈ Category` only when that membership is intended |
| `thing: Space` used as classification | `thing ∈ Space`; report the membership migration |
| `name: RelationShape` | explicit relation-contract form; never primitive relation classification |
| `{role: Type}` | `role: Type` as a schema-role binding only |
| `mode ...` | arrow and cardinality contract |
| `name: Model` | authoring or revision context inferred or externally named |
| `[Door 101]` | `Door 101` |
| `name: DerivationRule` | conclusion plus `if` for an oriented derivation rule; universal-law mode remains distinct |
| `name: Revision` | `name from base` plus signed clauses |
| `from:` | `from` in the revision header; the retained Revision profile remains migration evidence |
| `withdraw:` | `-`; the retained Revision profile remains migration evidence |
| `declare:` | ground relational content creating assertion occurrences in admission context; the retained Revision profile remains migration evidence |
| `find all ?x` | naked hole clause or `select` |
| `use game` | `requires` block |
| object-like `property: value` content | relational `property value` under focus |
| `name: value` used as a binding | retain `name: value`; `:` is binding |

The migration tool should preserve semantic IDs and print a report for every
inference it made.

## 19. Non-negotiable acceptance tests

The new surface is not accepted merely because examples look attractive.

### Relational honesty

- `iron-door` focus form and its expanded membership content, ordinary relational content, and
  focused definition produce identical checked semantics while retaining their
  three distinct semantic forms.
- No semantic node created by a focus block contains child fields or owned
  nested records.
- `iron-door ∈ Door` elaborates to ordinary membership and never to definition,
  equality, primitive typing, or field ownership.

### Block determinism

- all-bare enumeration lowers child-to-parent membership;
- definition/shape blocks remain derived relational views, not object schemas;
- any non-bare child makes the block a focused block;
- bare children inside a focused block relate the focus through membership;
- `name: value` inside a focused block binds `name of focus`, not a field or
  graph edge;
- the formatter separates enumeration blocks from contract or relational-content blocks and
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

Milestone 0 is complete and historical; its evidence record and the current
implementation order are governed by the [roadmap](roadmap.md).

### Milestone 1 — New layout and focus profile

Implement bare symbol grounding, enumeration blocks, definition or derived
shape blocks,
focused blocks, explicit flattening display, and multiword semantic names
without brackets. Lower into the current semantic core.

### Milestone 2 — Compact relation schemas

Implement schema role patterns, named roles, focused role designation,
arrow/cardinality contracts, ambiguity diagnostics, and stable hidden relation
identities. Remove required `RelationShape` and `mode` syntax in the new profile.

### Milestone 3 — Recursive term grammar

Permit every role to contain recursive terms with explicit grouping and
canonical formatting. Add derived structural views, pure definitions, and
projections. This milestone recovers the recursive-relational thesis.

### Milestone 4 — Holes, rules, and relational selection

Implement `?` and `?name` holes, repeated-hole correlation, naked single-clause
selection, `select` projection blocks, `any` existence, `select one` and
`select first`, `if` derivation-rule recognition, and hidden or optional derivation-rule labels. Replace
`find` in canonical examples. Do not collapse derivation rules into universal
universal laws, invariants, or goals.

### Milestone 5 — Revision surface reset

Implement `name from base`, `+` and `-` clauses, current `why`, `prevent`,
`achieve`, and `diff` parity, and migration from current model, rule, and
Revision syntax.

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

Ordinary focused relational content remains co-equal relational content. Focused membership and
binding retain their distinct semantic forms; none creates objects with
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
2. Categories emerge through ordinary membership relational content and contracts; they
   are not a primitive `Type` domain.
3. Referents do not acquire fields merely from layout.
4. `:` is stable binding/definition; `∈` is membership.
5. Enumeration, definition/shape, and focused blocks are structurally distinct.
6. Indentation is erasable projection over semantic forms; it does not collapse
   membership relational content, ordinary relational content, and definitions.
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
