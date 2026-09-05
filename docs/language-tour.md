# Clause language tour

Clause relates values and describes how those relationships may change.
Author each independent semantic fact once; checking, execution, queries,
explanations, and editing must use that same meaning.

## Values, roles, and contracts

A name can denote a value:

```clause
gravity: 9.81
rgb: 255, 0, 0
```

The comma creates ordered positions, not domain roles. A role names what a
value means relative to a subject:

```clause
lamp
  charge: 2.0
```

The number does not tell us whether it means charge, mass, or price. The role
does. Its contract separately constrains the subject, value, and cardinality:

```clause
Device
F64

charge
  domain: Device
  range: F64
  cardinality: one

lamp
  charge: 2.0

on consume ?device
  when
    ?device charge ?prior
  withdraw
    ?device charge ?prior
  include
    ?device charge ?prior - 1.0
```

This is the generated card's checked
[role-contract example](authoring-card.md#ordinary-role-contracts).
The role's description is ordinary Clause data. It needs no separate
`relation`, `reads`, `subject`, or forward `mode` declaration.

The `charge` contract requires every Device participant to have one F64 charge.
The compiler checks `lamp` from its facts; no second entry in a Device registry
is needed. A reference to a Device must satisfy the same obligations. Atomic
changes cannot leave a participant without its required charge.

Membership is separate data when belonging itself matters, such as a team
roster. It cannot substitute for a missing or mistyped required property.

`one`, `maybe`, `some`, and `many` constrain distinct values per subject to
exactly one, at most one, at least one, or any number. Repeating the same
contract fact does not change its meaning; contradictory contract facts fail.

## Focus and repetition

Indentation establishes focus. An explicit role relates that subject to its
objects; indentation alone invents no domain relationship.

```clause
north
  inputs
    nixpkgs
      from: "github:NixOS/nixpkgs/nixos-unstable"
    rust-overlay
      follows: nixpkgs
```

Here `north` has two `inputs`. Each input becomes the subject of its nested
facts. Repeated facts retain their individual origins even when their values
are equal. They are not an ordered product.

A colon or grouped indentation delimits a multiword role such as
`worker count`. The reader does not guess phrase structure from English.

## Derivation and withdrawal

A law relates its premises to its conclusions. In the generated card's checked
[recursive-dependency example](authoring-card.md#recursive-dependencies-with-withdrawal),
a prerequisite's blocker also blocks the dependent task:

```clause
law prerequisite-obstruction
  if
    ?task:
      prerequisite: ?prior
    ?prior:
      blocker: ?root
  then
    ?task:
      blocker: ?root
derive prerequisite-obstruction
```

The law describes the implication; `derive` selects it for computation.
Bindings introduced by the premises are shared with the conclusion. `?task:`
focuses those children on the same variable; it does not declare a type or
assert a separate fact. Flat and focused clauses have the same meaning.

Positive recursion computes consequences supported by the current base facts.
Removing one source of support preserves conclusions supported elsewhere.
Removing the last source removes its consequences, including through cycles:
a cycle cannot support itself. Equal conclusions appear once as values.

## Changes and queries

A handler's conditions read one pre-state. Its removals and additions form
one candidate change; accepting the candidate is a separate operation.
Running the same handler twice creates two occurrences, not one equal value.

Queries inspect a world without changing it. Hypothetical queries inspect an
isolated alternative. Every bounded search distinguishes a completed result
from exhaustion: running out of search does not prove absence or falsehood.

A checked live edit preserves only the identities and state justified by its
continuity mapping. Similar source text is not such a mapping. Physical plans
may specialize the authored meaning; they may not silently replace it.

## Current boundaries

The [authoring card](authoring-card.md) contains generated checked examples.
The [roadmap](roadmap.md) distinguishes those bounded capabilities, the active
five-task scheduling delivery, and the unfinished frontend, collection,
extension, query, and continuity work.

The [syntax](syntax.md) defines source structure. The
[foundation](foundation.md) defines meaning; the
[architecture](architecture.md) defines implementation boundaries.
