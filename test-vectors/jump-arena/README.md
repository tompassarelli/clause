# Clause jump-arena process oracle

This directory freezes a small process-first semantic oracle for a bounded 3D
jump arena. It is an exact fixture, not an implementation claim or a second
Clause carrier.

The governing semantics stay in the project documentation:

- clause:docs/foundation.md defines ApplicationForm, Application, Activation,
  Step, Run, continuation, observation, and Admission.
- clause:docs/syntax.md owns the source spelling used by
  clause:test-vectors/jump-arena/world.clause.
- This README only explains the local fixture. It does not restate the language
  constitution.

## Files and authority

- world.clause is the Clause-owned gameplay constitution. Its laws define
  input, movement, jump, gravity, groundedness, landing, and all four wall
  boundaries.
- process-ir.json is a typed fixture crosswalk: exact schemas and roles,
  operator Modes, ApplicationForms, nominal Applications, binary64 behavior,
  authority order, RuntimeSessions, and Admission obligations.
- vectors.json fixes exact occurrences, StateRevisions, candidate deltas,
  TransitionOccurrences tied to exact Clause source declarations,
  JudgmentOccurrences, Admissions, causal frontiers, continuations, positive
  outcomes, and typed failures.

The JSON is deliberately noncanonical. Fixture identifiers are opaque typed
references supplied by a harness. Host code may follow reference edges; it may
not derive meaning from their spelling. Renaming every designation while
preserving those edges must preserve every result.

## Process and authority boundaries

Representing an ApplicationForm does not run it. A nominal Application refers
to one checked form; an Activation is one actual engagement; Steps are its
causal carry-through.

The fixture proves both identity directions:

- one Application supports two distinct Activations, Runs, Steps, and
  Observation occurrences;
- two distinct ApplicationIds can share one equal ApplicationForm.

Every Activation selects one exact Mode and pins an exact ProgramRevision
selection, ProgramSnapshot, RuntimeSession, runtime policy, semantics, and
StateRevision. RuntimeSessions also name their SessionStartOccurrence and
initial StateRevision. Every actual Activation has one checked executable-
validity record and one exact causal origin. Authorization is separate: each
Mode declares a finite authorization-requirement set, and every local pure,
observational, and physics Mode in this fixture declares the empty set. The
fixture therefore carries no synthetic execution authorization. Browser input
still retains an exact typed input occurrence and external-trigger provenance.

Authority is well founded:

    irreducible root policy
      -> root Program Admission
      -> program-revision/jump-arena-r0
      -> constitutive Admission and Judgment-issuance authorization

Source admit only stages a candidate delta. Candidate construction leaves the
base revision authoritative and immutable. A separately authorized outer
Admission, backed by a recorded typed JudgmentOccurrence, creates the
successor StateRevision. Candidate output cannot authorize itself.

Each Admission obligation now points to a structural requirement Term. Its
meaning is carried by typed reference edges, declared relation/operator/Mode
applications, and exact candidate/base bindings; no obligation is selected by
identifier spelling. The candidate-producing Step emits one occurrence-exact
Observation per satisfied requirement. The authorized JudgmentOccurrence cites
those Observations, and Admission cites that Judgment. Candidate evidence is
therefore inspectable without becoming authority. Evaluation is ordered and a
first false requirement emits no synthetic success for later requirements.

Judgment, Program Admission, and State Admission occurrences use only the
constitutional `ProducedBy` or `EnteredThrough` provenance sum. The governed
boundary occurrences in this corpus use `EnteredThrough` with exact typed
occurrence frontiers. Every successor StateRevision is caused by the exact
TransitionOccurrence and producing Run, Activation, and Step; its separate
AdmissionOccurrence remains queryable evidence but is not state identity.

Every source `when` line has a stable premise slot. Each
TransitionOccurrence carries an ordered support use for every slot, including
non-withdrawn state/program bindings, numeric guards, both same-relation clamp
slots, and the airborne/landing branch verdict. Support identity is
occurrence-exact: distinct slots are never collapsed merely because they cite
the same occurrence or relation schema.

Render frames are derived observation projections, not StateRevisions. Their
identity key is the exact Run, Activation, Step, and Observation tuple. A frame
may additionally retain the observed StateRevision and payload as a paired
diagnostic pin, but those pins do not participate in frame identity. The corpus
includes two different frame values and proves that projecting them allocates
no candidate, Admission, or StateRevision.

## Numeric contract

F64 means IEEE-754 binary64 here:

- decimal source conversion is correctly rounded with roundTiesToEven;
- every declared operator rounds separately with roundTiesToEven;
- fused contraction is forbidden;
- only finite, non-NaN values are admitted;
- signed zero is canonicalized to positive zero;
- equality and ordered comparison are defined over that admitted domain; and
- division by zero, invalid operations, nonfinite input, and overflow to a
  nonfinite result produce the typed numeric-domain failure with no partial
  candidate visibility.

All current fixture values are exactly representable binary64 dyadics. The
four wall traces nevertheless record every operator and RoleId explicitly:

    east:        9.5 +  1 * 5 * 0.25 ->  10.75 -> clamp  10 -> velocity  2
    west:       -9.5 + -1 * 5 * 0.25 -> -10.75 -> clamp -10 -> velocity -2
    positive z:  9.5 +  1 * 5 * 0.25 ->  10.75 -> clamp  10 -> velocity  2
    negative z: -9.5 + -1 * 5 * 0.25 -> -10.75 -> clamp -10 -> velocity -2

## Frozen cases

The positive vectors cover:

- Application/ApplicationForm and Application/Activation separation;
- pure revision-indexed observation with no StateRevision creation;
- changing admission-free render-frame projections with exact process keys;
- immutable input proposal from an exact typed external-input occurrence,
  followed by explicit Admission;
- grounded horizontal movement;
- grounded jump impulse;
- airborne gravity;
- east, west, positive-z, and negative-z collision;
- landing at a wall with groundedness restoration; and
- one listener Activation suspending, resuming, emitting an immutable typed
  input Observation, and cancelling across three causally linked Steps.

The negative vectors cover airborne double-jump, stale program and world pins,
ambiguous or ineligible Mode selection, activation-budget exhaustion with an
exact resource obligation, stale continuation pins, linear-continuation reuse,
and binary64 division by zero as a typed numeric-domain failure. Pre-activation
exhaustion and pre-Step failures allocate no process or state identities. The
valid airborne jump Activation runs and returns the typed
no-applicable-process-rule failure without creating a candidate or successor.

## Host boundary and current gap

A host may generically resolve typed references, close Mode roles, execute
declared binary64 operators, derive laws, match guards, construct immutable
deltas, check causal frontiers and continuations, issue authorized judgments,
and perform governed Admission.

Rust, Wasm, JavaScript, and rendering code must not recognize game
designations, relation/schema IDs, roles, operators, or Modes by spelling. They
must not inject movement, jump, gravity, collision, groundedness, landing, or
transition selection.

There is intentionally no local evaluator yet. Writing a fixture-specific one
would create a second game-semantic implementation. From a repository checkout,
the current cheap gate is:

    jq empty "$PWD/test-vectors/jump-arena/process-ir.json" \
      "$PWD/test-vectors/jump-arena/vectors.json"

A canonical process runtime must additionally prove typed-reference closure,
occurrence-exact withdrawals, authority order, cause ownership, binary64
operator traces, ordered premise-slot/support bijections, occurrence-exact
obligation Observations, first-failure behavior, continuation pins,
JudgmentOccurrence coverage, and Admission/successor lineage against this
corpus.
