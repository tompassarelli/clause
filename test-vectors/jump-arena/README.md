# Clause jump-arena process oracle

This directory freezes the smallest process-first semantic oracle for a
playable bounded 3D jump arena. It is an exact fixture, not an implementation
claim and not a new source or carrier format.

The gameplay constitution is Clause-owned:

- `clause:test-vectors/jump-arena/world.clause` uses only source spelling
  ratified by `clause:docs/syntax.md`. It declares input, horizontal movement,
  jump impulse, gravity, groundedness, landing, and arena collision. The three
  `scalar/clamp-*` laws, rather than a host callback, define all four wall
  boundaries.
- `clause:test-vectors/jump-arena/process-ir.json` records the exact checked
  process concepts governed by `clause:docs/foundation.md`: schemas, modes,
  Applications, authorizations, cause frontiers, continuation policy,
  Admission obligations, and host limits.
- `clause:test-vectors/jump-arena/vectors.json` fixes exact StateRevisions,
  candidate deltas, Admissions, Runs, Steps, observations, positive outcomes,
  and typed rejections.

The JSON files are a typed test description while the canonical process-v1
carrier remains unimplemented. Their strings are fixture-local opaque typed
references. They are not public Clause IDs, canonical bytes, executable tags,
or permission for host code to switch on a designation. A conforming harness
must preserve every result after an equivariant renaming of all fixture-local
designations.

## Authority boundaries

`world.clause` represents process constitution. It does not execute, authorize,
or admit itself. In particular, source `admit` stages candidate additions; it
does not perform constitutional Admission.

Every actual trigger in `vectors.json` has an independent occurrence identity.
Each Activation selects one exact eligible Mode and pins the exact semantics,
ProgramRevision, RuntimeSession, runtime policy, and StateRevision. Its first
Step has exactly one `ActivationStart` cause. Candidate construction leaves the
base StateRevision unchanged. A separately authorized outer Admission alone
creates the named successor.

The execution and Admission authorizations are constitutive citations into the
already authoritative `program-revision/jump-arena-r0`. No candidate delta,
candidate successor, Step output, or candidate-produced Judgment can authorize
its own execution or Admission.

The following remain semantic-IR-only because no canonical source projection
is ratified for them:

- a runtime StateRevision observation request;
- Activation, Run, Step, and cause-frontier records;
- reactive continuation, resumption, linear-use, and cancellation policy;
- multiple declared event Modes; and
- governed outer Admission.

Keeping those objects in semantic IR is deliberate. Inventing readable-looking
syntax here would give a test fixture authority that belongs only to
`clause:docs/syntax.md`.

## Exact run assertions

The positive vectors establish:

| Vector | Exact assertion |
| --- | --- |
| `same-application-two-pure-activations` | One `ApplicationId` supports two distinct triggers, Activations, Run roots, Steps, and Observation occurrences. Equal observed position Values do not merge occurrences, and no StateRevision is created. |
| `input-proposal-is-immutable-until-admission` | An immutable input observation produces one candidate intent delta; the base stays byte-identical and no successor exists until outer Admission. |
| `grounded-horizontal-movement` | A half-step at speed five moves `(0, 0, 0)` to `(2.5, 0, -2.5)` and produces velocity `(5, 0, -5)`. |
| `grounded-jump-impulse` | Grounded jump replaces vertical velocity with eight and groundedness with false. |
| `airborne-gravity` | Semi-implicit gravity over one quarter-step changes vertical velocity from eight to six and height from zero to `1.5`. |
| `east-wall-collision-clamps-and-reduces-velocity` | An unclamped `x = 10.75` becomes `x = 10`; realized velocity is two, derived from actual displacement rather than desired motion. The same laws cover both x and z bounds. |
| `landing-at-wall-restores-groundedness` | An unclamped `y = -0.5` lands exactly at the floor, vertical velocity becomes zero, groundedness becomes true, and simultaneous horizontal wall collision remains bounded. |
| `input-listener-suspend-resume-cancel` | One stable Activation suspends, resumes under exact continuation pins, emits one fresh immutable input Observation, and then carries through explicit cancellation without fabricating a StateRevision. |

The negative vectors require exact rejection of airborne double-jump, stale
Program and world pins, omitted ambiguous Mode selection, a Mode outside the
ApplicationForm's eligible set, insufficient activation budget, stale
continuation pins, and reuse of a Clause-declared linear continuation. Every
pre-activation rejection allocates no Activation, Run, Step, observation,
candidate delta, or StateRevision. Airborne jump engages valid process
machinery but produces only the typed no-applicable-rule rejection and leaves
its exact base authoritative.

## Host contract

A host may generically parse and check declared Readings, close named roles,
select an exact Mode, evaluate conventional exact arithmetic, derive the clamp
laws, match guards, construct immutable deltas, check typed cause frontiers and
budgets, and perform governed Admission. Rust, Wasm, JavaScript, and a renderer
must not recognize game designations or inject movement, jump, gravity,
groundedness, collision, landing, or transition selection.

The renderer's future input boundary emits immutable input observations. Its
state boundary consumes an immutable frame derived from an admitted
StateRevision. It never integrates game state or performs Admission.

## Structural validation

There is intentionally no local evaluator: writing one here would create a
second game-semantic implementation. The bounded structural gate is:

```text
jq empty ~/code/clause/main/test-vectors/jump-arena/process-ir.json \
  ~/code/clause/main/test-vectors/jump-arena/vectors.json
```

A process-v1 runtime must additionally prove all typed references, predecessor
links, withdrawal occurrences, cause ownership, exact outputs, and rejection
allocation sets against these fixtures. Until that runtime exists, this corpus
is the semantic oracle and exposes, rather than hides, the missing executable
capability.
