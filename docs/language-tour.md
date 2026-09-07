# Clause language tour

Clause relates values and describes how those relationships may change.
Author each independent semantic fact once; checking, execution, queries,
explanations, and editing must use that same meaning.

## Run the scheduling example

Five tasks share prerequisites: design, prototype, validation, documentation,
and launch. Two independent obstructions—approval and components—block parts
of that graph. Clearing approval should unlock design while preserving the
component obstruction farther downstream.

The complete source is
[clause:test-vectors/authoring/scheduling.clause](../test-vectors/authoring/scheduling.clause).
The existing public-library journey is
[clause:crates/clause-workbench/tests/scheduling.rs](../crates/clause-workbench/tests/scheduling.rs),
starting at `scheduling_controls_use_checked_dependencies_and_preserve_identity`.
From an owned Clause worktree, using the repository's pinned Rust development
shell and C compiler, run:

```sh
nix develop --command cargo test -p clause-workbench --locked -j 2 --test scheduling scheduling_controls_use_checked_dependencies_and_preserve_identity -- --exact
```

This command runs the journey and checks its results; its output is a test
verdict. It does not launch the browser application. The
[roadmap records the observed run and its limits](roadmap.md#walkthrough-evidence).
Follow the source and test together:

1. **Declare the data.** `title`, `duration`, and `completed` each require one
   value per Task; `prerequisite` relates Tasks to Tasks, and `obstruction`
   relates Tasks to Roots. The five task blocks supply their required values.
   They need no second Task membership registry. For example, prototype starts
   with duration `4.0`, prerequisite `design`, and obstruction `components`.
2. **State and enable the laws.** `direct-obstruction` derives a blocker from
   an obstruction. `inherited-obstruction` carries that blocker through
   prerequisites. The two waiting laws similarly propagate unfinished
   prerequisites. Each law has a separate `derive` declaration; writing an
   implication alone does not select it for computation.
3. **Open and inspect.** `ResidentSourceWorkbenchV1::open` reads, checks, lowers,
   and opens this source. `project_current_world()` observes its current world.
   The test reads the projected `duration` and `blocker` relations: five tasks
   and eight task/root blocker pairs.
4. **Query while proposing completion.** The source's `complete` handler selects
   the requested Task and computes two finite counts against one pre-state:

   ```clause
   sum 1.0 given ?task where { ?task waiting ?prior } as ?waiting
   ?waiting = 0.0
   sum 1.0 given ?task where { ?task blocker ?root } as ?blockers
   ?blockers = 0.0
   ```

   These are handler conditions from the complete source, not a standalone
   program. An empty completed query gives zero; an exhausted computation must
   fail rather than pretend the Task is unblocked. This example queries through
   the handler and reads projected relations through the library; it does not
   use the design-level `select`, `why`, or `achieve` request syntax.
5. **Execute, then admit.** The test's `run` helper calls `handler_occurrence`
   with the exact input Referent, then `run_occurrences_to_candidate`, then
   `admit`. Only the last call publishes the successor projection. The library
   driver supplies this separate admission decision; the Clause handler's
   `include` block does not authorize or admit itself. A blocked completion
   produces no completed Task even though the helper admits the resulting
   candidate.
6. **Observe the dependency changes.** Resolving approval retracts only its
   supported blocker consequences. Completing design then succeeds. Prototype
   still cannot complete until components is resolved. Extending prototype
   changes its duration from `4.0` to `6.0` while retaining its Referent.
   After resolving components and completing prototype, validation,
   documentation, and launch, all five are complete and both waiting and
   blocker relations are empty. Design retains its original Referent too.

The role and law definitions in the source supply both the transition checks
and these observations. The following sections explain the smaller language
forms used to express them; isolated fragments illustrate a concept and are
not all standalone runnable programs.

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
The role's description is ordinary Clause data and supplies its field Reading.

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

## Bindings and readings

A phrase with several bindings states each binding's domain once, then uses
the ordinary clause syntax to relate them:

```clause
limited:
  (shape: F64):
    ?amount ?minimum ?maximum ?result
  ?amount:
    limited between ?minimum and ?maximum as: ?result

mode limited given amount minimum maximum yields result: maybe
```

The shared edge explicitly constrains all four bindings to F64 while the
Reading remains independent of its executable direction. Laws and handlers
use that same Reading and can check the same conformance premises. A
structured value declares its fields directly, without unused binders. The generated card's
[checked journey](authoring-card.md#bindings-focus-and-structured-values)
combines both forms with atomic updates and a checked live edit.

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
five-task scheduling journey, and the unfinished frontend, collection,
extension, query, and continuity work.

The [syntax](syntax.md) defines source structure. The
[foundation](foundation.md) defines meaning; the
[architecture](architecture.md) defines implementation boundaries.
