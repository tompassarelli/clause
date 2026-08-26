# Current executable Clause syntax

> **Status:** Descriptive implementation snapshot.
>
> **Authority:** The checked frontend and executable tests decide what is
> accepted today. The [semantic foundation](foundation.md) governs meaning;
> the [target surface](surface.md) governs intended syntax and is ahead of the
> parser in places. This document records that gap rather than hiding it.

This is the compact reference for Clause source accepted by the current
frontend. Anything not listed here is not a supported source promise merely
because it appears in the target surface.

## 1. Core distinctions

| Source form | Current meaning |
| --- | --- |
| `Game` | Ground one semantic referent when written as a bare declaration. |
| `Chess ∈ Game` | Assert membership as ordinary relational content. |
| `gravity: 9.81` | Bind a name to one denotation. |
| `iron-door connects Cellar to Armory` | Assert one declared role-labelled relation. |
| `?door connects ?origin to ?destination` | Use correlated variables in a rule or request. |
| `?` | Use one fresh anonymous query hole. |
| `known -> sought` | Orient a relation lookup mode; it is not implication. |
| `before ~>` / `after` | Declare one state-transition candidate. |

`:` and `∈` are deliberately different today. `:` binds; `∈` asserts
membership. Raw `::` is rejected by the parser, although an editor rewrite may
turn it into `∈` before parsing.

## 2. Layout and names

- Source is line-oriented.
- Indentation is exactly zero, two, or four ASCII spaces.
- Tabs, carriage returns, whitespace-only blank lines, and other indentation
  widths are rejected.
- Clause words are separated by ASCII spaces.
- There is no implemented Clause comment syntax.
- Semantic names may contain multiple words. Their spelling and case are
  exact; the frontend performs no case-folding or Unicode normalization.
- Qualified declaration names use slash-separated local names, such as
  `egress/connects`.
- Role and variable names begin with an ASCII letter or `_`, then use ASCII
  letters, digits, `_`, or `-`.
- Concrete bracketed referents such as `[iron-door]` are retired. Brackets are
  reserved for finite ranges, focus templates, and structural sequences.

## 3. Inferred declarations

### Grounding

A bare top-level name grounds one referent:

```clause
Door
Space
F32
```

`F32`, `Int`, and `Bool` name the current built-in structural domains.

### Enumeration

A flat bare block of names is inferred as a finite enumeration:

```clause
Game
  Chess
  Soccer
```

This lowers to the membership facts `Chess ∈ Game` and `Soccer ∈ Game`.

### Labelled structural shape

A flat bare block of bindings is inferred as a labelled product contract:

```clause
Vec2
  x: F32
  y: F32
```

The field names are not runtime string keys. Elaboration resolves the global
`Vec2` designation and the shape-scoped `x` and `y` designations to exact
semantic identities:

| Source designation | Checked representation |
| --- | --- |
| `Vec2` | one global `ReferentId` |
| `Vec2` / `x` | one shape-scoped `ReferentId` |
| `Vec2` / `y` | one shape-scoped `ReferentId` |

The initial IDs are deterministically seeded by those designations; an
ordinary rename changes identity unless an explicit migration retains it. Once
lowered, labelled products carry the IDs, not raw `"x"` or `"y"` keys.

The real normalization gap is library ownership: Clause does not yet ship one
canonical prelude `Vec2`, so each program currently declares the shape and its
field spellings. The scene projector binds the exact shape and field IDs and
never recovers axes from map order or host strings.

### Model

A non-flat bare block that is not another uniquely inferred declaration is a
Model:

```clause
world
  Cellar ∈ Space
  Armory ∈ Space
  iron-door ∈ Door
  iron-door connects Cellar to Armory
```

Model assertions must be closed: variables and query holes cannot survive into
admitted Model content.

## 4. Membership and focused authoring

The explicit one-line membership form is repeatable:

```clause
iron-door ∈ Door
iron-door ∈ Something-Else
```

Multiple memberships can also be written in one focused block:

```clause
world
  iron-door
    Door
    Something-Else
    connects Cellar to Armory
    state: locked
```

Within that block:

- each grounded bare child becomes a membership fact for `iron-door`;
- each relational phrase is expanded with `iron-door` as its focused
  participant; and
- `state: locked` becomes the binding `state of iron-door: locked`. It is not
  object-field mutation.

Each membership remains an independent assertion occurrence with its own
provenance.

These spellings are **not** current membership syntax:

```clause
Chess : Game
iron-door : Door
iron-door : Door, Something-Else
```

Whitespace before `:` is rejected. Without that whitespace, `iron-door: Door`
is a binding, not membership; a comma RHS is at most one denotation spelling,
not multiple claims. Reinterpreting `name: Category` from context would collide
with bindings such as `state: locked` and structural fields such as `x: F32`.

## 5. Bindings and pure definitions

A top-level one-line binding may contain a checked closed term:

```clause
gravity: 9.81
truth: true
spawn: (0.0, 0.0)
origin: Vec2 { x: 0.0, y: 0.0 }
```

There is one historical context difference: the same `gravity: 9.81` row
inside a named Model follows the older simple-binding path and denotes a scoped
referent spelled `9.81`; at top level in a caller Model context it lowers to an
`F32` term. Do not rely on that difference as a language feature. It is an
implementation migration seam.

A pure definition block has zero or more immutable local bindings followed by
one final result term:

```clause
energy:
  base: gravity + measured-gravity
  base + base
```

The locals are authoring names only. They do not become exported semantic
referents or independent assertions.

## 6. Structural and recursive terms

Implemented structural forms include:

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

- Decimal `F32` values must be finite; negative zero is normalized to positive
  zero.
- Integers are signed 64-bit values.
- Strings support `\"`, `\\`, `\n`, and `\t` and inhabit `Text` in relational
  clauses and laws. A program using them must ground `Text`; string
  pure-definition lowering is not yet supported.
- Tuples have at least two members.
- Sequences are nonempty and homogeneous.
- A labelled product fills one declared shape's exact field set with values of
  the declared domains.
- Parentheses group recursive relation applications and arithmetic terms.

Single-result relation modes may be used recursively as terms:

```clause
position of player
radius of coin + radius of player
length (position of player - position of coin)
if collected then 10 else 0
map length over vectors
```

The relation must expose one unique cardinality-one result mode. An application
is term structure; it does not assert new content merely by appearing.

## 7. Relation contracts and clauses

The retained explicit form is:

```clause
egress/connects: RelationShape
  {door: Door} connects {origin: Space} to {destination: Space}
  mode door, origin -> destination: many
```

The inferred compact equivalent is:

```clause
connects
  door: Door connects origin: Space to destination: Space
  door origin -> destination*
```

Compact cardinality suffixes are:

| Suffix | Cardinality |
| --- | --- |
| none | `one` |
| ` 0..1` | `maybe` |
| `+` | `some` |
| `*` | `many` |

Once declared, a fact uses the sentence phrase:

```clause
iron-door connects Cellar to Armory
```

Surface word order is not the semantic representation. Elaboration produces
one relation identity and a map from exact `RoleId`s to checked terms. N-ary
relations retain every named role.

Top-level and Model-contained clauses may carry labelled products directly:

```clause
player scene-position Vec2 { x: 0.0, y: 0.0 }
```

Only a colon at delimiter depth zero starts a binding. Colons inside a checked
product do not.

## 8. Rules and laws

The retained explicit rule form is:

```clause
impact/direct-dependency: DerivationRule
  ?consumer depends on ?dependency
  when:
    ?consumer imports ?dependency
```

An unlabelled positive rule may use conclusion-plus-`if`:

```clause
coin scene-position Vec2 { x: 10.0, y: 0.0 } if
  coin state active
```

Every conclusion variable must be range-restricted by its premises. Anonymous
holes are query-only.

A universal law is authored separately from operational authorization:

```clause
law collision overlap
  ?body overlaps ?other if
    ?body collides with ?other

derive collision overlap
```

The law alone is semantic ground; `derive` separately authorizes its projection
as a derivation rule.

## 9. State transitions

An event declares optional payload variables and one or more transactional
before/after pairs:

```clause
on collect ?actor
  ?coin state active ~>
    ?coin state collected
  if
    ?coin owner ?actor
```

- The source clause is two-space indented and ends in ` ~>`.
- Its successor is four-space indented.
- An optional two-space `if` is followed by one or more four-space guards.
- All candidate writes match one pre-state and commit one successor state.
- Event payload and pre-state bindings must range-restrict every successor.

The effect runtime and canonical effect evidence exist, but source forms such
as `render! scene`, capabilities, resources, and receipt bindings are not yet
accepted by this frontend.

## 10. Revisions and reusable deltas

The retained explicit form is:

```clause
impact/adopt: Revision
  from: impact
  admit:
    South imports North
  withdraw:
    North imports Store
```

A reusable change set uses `Delta`, and a Revision may `apply:` one Delta. The
canonical signed shorthand is also accepted:

```clause
impact/adopt from impact
  + South imports North
  - North imports Store
```

Admissions and withdrawals must be closed and cannot overlap.

## 11. Requests

Current request forms include:

```clause
any World relates ?person to ?

select ?person
  World relates ?person to ?destination

select one ?person
  World relates ?person to C

select first ?person
  World relates ?person to ?destination

find all ?destination in egress:
  ICU-A has a usable egress path to ?destination

why all in egress:
  ICU-A has a usable egress path to North-Exit

prevent all minimal in egress:
  ICU-A has a usable egress path to North-Exit
using:
  egress/passed

achieve one minimal in impact:
  compiler-change affects South
using:
  impact/imports

diff impact -> impact/adopt
```

A naked relational clause containing `?name` or `?` is also a query when it
matches exactly one declared Model.

## 12. Finite ranges and correlated focus

Finite membership ranges and correlated focus blocks are implemented:

```clause
egress
  [Door 101..106] ∈ Door
  [Door {n}]
    passed Fire-Marshal-Inspection
  for n: 101..104
```

The range is inclusive and ascending. The template variable and `for` binding
must match; expansion is bounded and occurs before the Model is sealed.

## 13. Important target syntax not yet executable

These remain target-surface work rather than current parser claims:

- dedicated `scene includes sprite ... at position of ...` lowering;
- authored `render!`, capability, resource, and receipt syntax;
- category contract lines such as `position -> Vec2` without an explicit
  relation contract;
- `select all`, multi-clause selection, and random selection;
- authored invariant, goal, observation, assumption, requirement, and
  intention forms;
- generated live JavaScript transition execution;
- real browser/Three.js execution and source maps; and
- the full M7 performance and parity claim.

The current grounded scene checkpoint instead uses an ordinary declared
relation carrying an entity referent and an exact labelled `Vec2`, then projects
only supported grounded runtime content into a canonical RenderPlan.
