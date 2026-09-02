# A Short Tour of Clause

> **Status:** Tour of the canonical source design and process-first semantic
> contract. These examples are not yet runnable by a supported Clause
> toolchain; the [roadmap](roadmap.md) alone records implementation status.
>
> **Authority:** The [syntax](syntax.md) governs every source spelling shown
> here, and the [semantic foundation](foundation.md) governs its meaning. This
> tour is explanatory, not a second language specification.

Clause source is compact and declarative. It says which applications,
relationships, laws, observations, and changes are admissible without turning
source order, graph storage, or host-language calls into semantics.

## Declare distinctions and describe participation

Declarations use explicit heads. An `enum` gives every child one homogeneous
member role; a `shape` gives every child one homogeneous field role:

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
```

A bare designation leaf introduces or resolves a Referent. At top level,
`gravity: 9.81` makes `gravity` denote one scalar value. Commas instead denote
an ordered anonymous product, as in `rgb: 255, 0, 0`; a vertical list of bare
children is not a product because indentation does not invent position or a
relation. Denotation, equality, Shape satisfaction, and representation remain
separate. In particular, `north: Flake` means that `north` denotes `Flake`; it
does not classify `north` or make it satisfy the `Flake` Shape.

An identified subject receives explicitly named semantic applications:

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

`shape: Flake`, `description: ...`, and each `from:` application name the role
that its object plays. Literal kind constrains the supplied value but never
invents that role. An interior role such as `inputs` groups repeated
applications; each object remains independently provenanced, and descendants
make that object the subject of their nested applications. Commas are not an
alias for repeated roles. Spaces may occur within a role designation, while a
colon or grouped indentation fixes its boundary. The reader never splits a
multiword English phrase heuristically.

`shape: S` means directional Shape satisfaction: the subject must satisfy every
obligation exposed by `S` and be substitutable wherever `S` is required,
relative to the Shape's declared observation, effect, failure, progress, and
representation boundaries. A Shape may describe roles and cardinalities,
value contracts, modes, effects, failures, laws, observable invariants, and
progress or resource obligations. The current compiler checks only its
field/application subset; the broader behavioral contract remains the semantic
target. A Shape is neither nominal membership nor exact equality, and physical
layout matters only when explicitly observable.

Source forms elaborate into neutral recursive Terms plus contextual
`ClauseJudgment`s and candidate formations. Lower-case clause means one such
contextual judgment over a Term, not an Application or governed Judgment.
Checked formation still does not assert, authorize, or execute a Term.

## Separate relational shape from executable running

A compact `relation` block groups several checked concepts without collapsing
them:

```clause
relation connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many

Cellar
  shape: Space
Armory
  shape: Space

iron-door connects Cellar to Armory
```

The declaration supplies a `RelationSchema` with exact named roles, a source
`Reading`, an explicit focus subject, and an executable `Mode`. The Mode says
which roles are known, which are produced, and the result cardinality. A
relation may instead have no executable mode at all; relation existence never
grants execution authority.

Checked formation resolves the Reading into complete named-role bindings and
one exact operator. It records the eligible Mode set and context requirements
in an `ApplicationForm`. No runtime may recover roles from triple position,
source order, spelling, or host fields.

## Application is possibility; Activation is occurrence

The process distinction is semantic, not extra everyday syntax:

```text
Term + FormationJudgment
  -> closed ApplicationForm
  -> nominal Application with ApplicationId
  -> fresh Activation with ActivationId and one Run membership
  -> configurations connected by causal Steps
  -> observations, result, continuation, or candidate delta
```

An ApplicationForm can be quoted, inspected, transformed, or rejected. A
nominal Application instantiates one exact closed form and may be activated.
Two activations of the same Application are different occurrences, and one
Activation keeps its identity across many Steps, suspension, and resumption.
A Run is the causal envelope rooted at one Activation, not whichever trace or
total log order happened to be retained.

Every Activation has one exact `ActivationStartRecord`: a
`StaticActivationBasis` selecting the `ClauseSemanticsId`, Application, Mode,
and exact `CheckedConstitutionBinding`; an explicit `InitialContext`; exact
bindings for the Mode's possibly empty dynamic-prerequisite schema; and a
separate occurrence-only cause frontier. The constitution binding may name
checked non-authoritative package/snapshot bytes or an admitted ProgramRevision.
RuntimeSession, runtime policy, and observed StateRevision pins are present only
where the selected context requires them, and a Mode with no dynamic
prerequisites manufactures no authorization or capability token. Equal syntax
run under different contexts therefore cannot silently collapse into one
timeless `evaluates-to` edge.

## Laws need separate authorization

A durable law puts binders and premises before its conclusions:

```clause
law direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency
```

The law is available semantic ground but remains operationally inert until
derivation is explicitly authorized:

```clause
derive direct-dependency
```

This preserves the difference between describing a universally available
constraint and authorizing a process to derive consequences from it.

## Transitions propose; Admission commits

Events use one common candidate-delta vocabulary:

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

`when` observes one exact base StateRevision. `withdraw` and source `include`
stage one grounded, conflict-checked candidate delta during a valid Activation
after only the prerequisites declared by its selected Mode have closed. The
source addition word does not perform constitutional Admission. Only a
separate governance operation can admit the candidate and
allocate a successor StateRevision.

Pure running needs no revision at all. It may return values and evidence while
the observed world remains unchanged. Every real-effect Activation keeps three
semantic slots distinct: exact intent occurrence, issued EffectAuthorization
occurrence, and independent CapabilityEvidence. A governed-per-intent profile
additionally requires the intent's exact AdmissionOccurrence. A preauthorized
local/session/Lease/batch scope may cover several bounded attempts without
manufacturing per-attempt Admission or issuance. Constitutive execution
authority never replaces issued effect authorization. Intent, authority,
capability, attempt, optional receipt, observation, Judgment, and possible later
Admission remain distinct in every profile. Canonical effect source syntax is
not yet ratified, so this tour does not invent one.

## Queries seek observations; they do not assert

A request has an explicit head and exact `CheckedConstitutionBinding`:

```clause
select all ?destination in egress
  where
    ICU-A has a usable egress path to ?destination
```

Existence is equally explicit:

```clause
any in World
  where
    World relates ?_ to C
```

A variable or missing row never turns a relational form into a false
assertion. Query absence is not falsehood. `select one` requires exactly one
deduplicated result; `select first` requires explicit ordering and may return
none. A request that joins an authoritative RuntimeSession, proposes
authoritative world change, relies on constitutive Program authority, or
performs a real effect resolves its operand to an exact ProgramRevision. A
sandbox/candidate request instead pins exact checked package and ProgramSnapshot
bytes and may read a separately pinned admitted world; it gains no constitutive
authority unless an admitted constitution or separately supplied
`IrreducibleRootConstitution` actually provides it.

## Repetition is explicit and regular

Prefix binders precede every dependent use:

```clause
for n in 101..106
  Door-{n}
    shape: Door
```

Ranges are inclusive ascending integer ranges. Brackets remain sequence Terms,
not a second range notation. Ordinary source exposes process machinery only
where it changes meaning.

## General-purpose local work stays local

Parameters, collections, loops, builders, ownership, and regions are ordinary
Clause work rather than foreign Rust semantics:

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
      mutable builder: empty Sequence of Result
      borrow read items as source
        lease write builder as sink
          for item in source
            append mapping(item) to sink
      return freeze move builder

upper-names: map(player-names) with
  Item = Text
  Result = Text
  mapping = uppercase
```

This is the same ratified UTF-8/LF specimen frozen by the syntax and adoption
contracts. The builder is Activation-local: thousands of internal reductions
need no Admission, StateRevision, or mandatory trace entry. `region`, `borrow`,
`lease`, `move`, and `freeze` appear because aliasing, escape, or reclamation
meaning changes there. Static proofs may erase from a checked production ABI.
The native/Wasm game hot profile uses affine ownership, borrows, leases, and
deterministic regions with no managed island, no mandatory tracing GC, no
implicit ARC, and no finalizer fallback. Other profiles may explicitly select
a finite, budgeted managed island; it is never an ambient Clause heap.

## Agents get one semantic workbench

The primary development consumer is an agent, so the target interface is one
long-lived transactional service rather than a pile of unrelated text tools:

```text
parse      -> exact Reading, source occurrences, and recoverable local errors
check      -> typed diagnostics and unsatisfied obligations
explain    -> source -> Term -> Application -> Activation/artifact path
query      -> exact bindings, supports, and causal dependencies
diff       -> affected and preserved semantic/cache sets
propose    -> candidate delta against one exact base
admit      -> separate governed decision
run        -> values, observations, continuations, or typed outcomes
hotReload  -> preserve exact live pins or reject with migration obligations
```

Stable diagnostic codes, exact identities, machine-readable dependency slices,
and atomic edits are the interface; prose is a rendering. Interactive requests
may use accepted incremental summaries. Full Lean and compiler-succession replay
is a promotion gate, not a tax on every edit.

## The graph explains; checked execution runs

The Clause Graph carries process constitution, admitted boundaries,
provenance, causal structure, and relational views in an inspectable form. It
does not run because rows exist. Implementations may lower accepted meaning to
direct calls, arrays, indexes, state machines, actor loops, native code, Wasm,
JavaScript, databases, or later GPU kernels. Those physical forms remain
replaceable refinements of the declared identities, observations, effects,
failures, resources, and causal order.

The [architecture](architecture.md) defines the host and trust boundary. The
[canonical-package contract](canonical-package.md) defines exact transport;
the [compiler-genesis contract](compiler-genesis.md) defines external genesis
and predecessor-owned compiler succession. Neither wire bytes, Lean checking,
Rust execution, nor successful materialization can create semantic authority.

## Continue reading

- [Semantic foundation](foundation.md) for the complete constitutional model.
- [Canonical syntax](syntax.md) for grammar, layout, names, requests, and
  currently unratified forms.
- [Architecture](architecture.md) for compiler, runtime, physical, and Wasm
  responsibility boundaries.
- [Adoption spike](adoption-spike.md) for the exact executable falsifiers,
  including the game-oriented vertical slice.
- [Roadmap](roadmap.md) for what exists now and the dependency path to a
  supported language.
