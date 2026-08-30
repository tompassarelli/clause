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

## Declare distinctions without inventing objects

Declarations use explicit heads. An `enum` gives every child one homogeneous
membership role; a `shape` gives every child one homogeneous field role:

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
```

`referent` introduces or resolves a designation. `∈` expresses ordinary
membership, `:=` defines one denotation, and `:` remains structural. Source
forms elaborate into neutral recursive Terms plus candidate formations;
checked formation still does not assert, authorize, or execute a Term.

Indentation supplies containment and an explicitly declared omitted role. It
does not guess a domain relationship:

```clause
iron-door
  ∈ Door
  ∈ Lockable
  connects Cellar to Armory
  state locked
```

The header focuses `iron-door`; every child still names its edge. A bare
unkeyworded parent and child such as `Foo` followed by `Bar` is invalid because
no Reading says what connects them.

## Separate relational shape from executable running

A compact `relation` block groups several checked concepts without collapsing
them:

```clause
relation egress/connects
  reads {door: Door} connects {origin: Space} to {destination: Space}
  subject door
  mode given door origin yields destination: many

Cellar ∈ Space
Armory ∈ Space

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

Every Activation pins exact `ClauseSemanticsId`, `ProgramSnapshotId`,
`ProgramRevisionId`, selected `ModeId`, typed initial context, required
authorization, and cause frontier. It also pins a RuntimeSession when present
and an observed StateRevision when the Application is world-sensitive. Equal
syntax run under different contexts therefore cannot silently collapse into
one timeless `evaluates-to` edge.

## Laws need separate authorization

A durable law puts binders and premises before its conclusions:

```clause
law impact/direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency
```

The law is available semantic ground but remains operationally inert until
derivation is explicitly authorized:

```clause
derive impact/direct-dependency
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
  admit
    ?coin state collected
```

`when` observes one exact base StateRevision. `withdraw` and source `admit`
stage one grounded, conflict-checked candidate delta during an authorized
Activation. The lower-case source word does not perform constitutional
Admission. Only a separate governance operation can admit the candidate and
allocate a successor StateRevision.

Pure running needs no revision at all. It may return values and evidence while
the observed world remains unchanged. Effects are stricter still: intent,
authorization, attempt, optional receipt, observation, Judgment, and possible
later Admission remain separate occurrences. Canonical effect source syntax is
not yet ratified, so this tour does not invent one.

## Queries seek observations; they do not assert

A request has an explicit head and exact ProgramRevision context:

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
none.

## Repetition is explicit and regular

Prefix binders precede every dependent use:

```clause
for n in 101..106
  Door-{n} ∈ Door
```

Ranges are inclusive ascending integer ranges. Brackets remain sequence Terms,
not a second range notation. Ordinary source exposes process machinery only
where it changes meaning.

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
