# Clause authoring card

This card is generated from compiler-owned examples. It is a curated current vocabulary, not an exhaustive language specification. The checked examples and diagnostics from the consuming project's immutable Clause compiler pin are authoritative.

Use that pin's workbench directly:

- `clause-workbench authoring-card` prints this card.
- `clause-workbench check-source FILE.clause` reads, elaborates, lowers, and opens the source in the resident execution workbench.
- `clause-workbench project-nix FILE.clause [OUTPUT]` checks `using Nix` relations and renders their typed flake projection.

Live source tooling offers an explicit checked scalar-effect replacement, not arbitrary text-reload continuity. Use `scalar_effects()` and `edit_scalar_effect()` with the captured generation and exact offered node; settle any pending candidate first. Native and Wasm carry the actual live world internally through the checked operation. Retained explanations describe accepted Steps; finite interventions query an isolated recorded pre-state without applying input or admitting a world. See `docs/live-source-semantics.md` for the compiler/runtime and passive browser contract, bounds, and remaining limits.

## Closed finite query sums

Sums F64 contributions over exact finite row matches in the same pre-state. Query-local variables do not capture the enclosing handler; an empty query yields zero, distinct equal-valued referents contribute independently, and exhausted search is an error.

Catalog ID: `finite-sums`

```clause
F64
Bool
Item
Report

relation enabled
  reads {item: Item} enabled {value: Bool}
  subject item
  mode given item yields value: one
relation amount
  reads {item: Item} amount {value: F64}
  subject item
  mode given item yields value: one
relation total
  reads {report: Report} total {value: F64}
  subject report
  mode given report yields value: one
relation count
  reads {report: Report} count {value: F64}
  subject report
  mode given report yields value: one

first
  shape: Item
second
  shape: Item
report
  shape: Report
first enabled true
first amount -2.0
second enabled false
second amount 2.0
report total 0.0
report count 0.0

on measure ?report
  when
    ?report total ?prior-total
    ?report count ?prior-count
    sum 1.0 where { ?item enabled true; ?item amount ?value } as ?count
    sum ?value where { ?item enabled true; ?item amount ?value } as ?total
  withdraw
    ?report total ?prior-total
    ?report count ?prior-count
  include
    ?report total ?total
    ?report count ?count

on enable-all ?item
  when
    ?item enabled ?prior
  withdraw
    ?item enabled ?prior
  include
    ?item enabled true

on clear ?item
  when
    ?item enabled ?prior
  withdraw
    ?item enabled ?prior
  include
    ?item enabled false

on create-item ?report ?amount
  when
    ?report total ?prior
  create
    ?item
      shape: Item
  include
    ?item amount ?amount
    ?item enabled true
```

## Explicit semantic applications

Applies one Shape and two scalar roles to a subject without confusing those applications with denotation or representation.

Catalog ID: `explicit-semantic-applications`

```clause
Flake

north
  shape: Flake
  priority: 5
  greeting: "hello"
```

## Scalar state transition

Declares referents and a cardinality-one relation, then replaces one numeric state value atomically.

Catalog ID: `scalar-state-transition`

```clause
F64
Account

relation balance
  reads {account: Account} balance {value: F64}
  subject account
  mode given account yields value: one

operating-account balance 100.0

on deposit ?account
  when
    ?account balance ?balance
  withdraw
    ?account balance ?balance
  include
    ?account balance ?balance + 25.0
```

## Structured keyboard transition

Declares structured and Boolean state, binds a physical key, and updates a Vec3 with scalar arithmetic.

Catalog ID: `structured-keyboard-transition`

```clause
F64
Bool
Player

shape Vec3
  x: F64
  y: F64
  z: F64

relation velocity
  reads {player: Player} velocity {value: Vec3}
  subject player
  mode given player yields value: one

relation empowered
  reads {player: Player} empowered {value: Bool}
  subject player
  mode given player yields value: one

player-1
  shape: Player
player-1 velocity Vec3 { x: 0.0, y: 0.0, z: 0.0 }
player-1 empowered true

bind keyboard KeyQ down to planar-burst

on planar-burst ?player
  when
    ?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }
    ?player empowered ?was-empowered
    ?was-empowered = true
  withdraw
    ?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }
  include
    ?player velocity Vec3 { x: ?velocity-x + 3.0, y: ?velocity-y, z: ?velocity-z - 2.0 }
```

## Scalar input transition

Binds one named physical scalar channel to a typed one-argument handler and records its finite observed value.

Catalog ID: `scalar-input-transition`

```clause
F64
Player

relation camera-heading
  reads {player: Player} camera heading {value: F64}
  subject player
  mode given player yields value: one

player-1
  shape: Player
player-1 camera heading 0.0

bind scalar-input CameraHeading to observe-camera-heading

on observe-camera-heading ?player ?heading
  when
    ?player camera heading ?prior
  withdraw
    ?player camera heading ?prior
  include
    ?player camera heading ?heading
```

## Many-valued relation

Retains idempotent values in a cardinality-many relation and requires membership before selecting one.

Catalog ID: `many-valued-relation`

```clause
Root
Item

relation active
  reads {root: Root} active {value: Item}
  subject root
  mode given root yields value: one

relation known
  reads {root: Root} known {value: Item}
  subject root
  mode given root yields value: many

root active none

on discover ?root ?item
  when
    ?root active ?active
  withdraw
    ?root active ?active
  include
    ?root active ?active
    ?root known ?item

on select ?root ?item
  when
    ?root active ?active
    ?root known ?item
  withdraw
    ?root active ?active
  include
    ?root active ?item
```

## Typed occurrence input

Transports an exact projected Item referent to one reusable selection rule; two items of the same class remain distinct and only selected items advance on tick. Retain the projection's generation with the input.

Catalog ID: `referent-input-transition`

```clause
F64
Bool
Item
ItemClass

relation item-class
  reads {item: Item} item class {value: ItemClass}
  subject item
  mode given item yields value: one

relation selected
  reads {item: Item} selected {value: Bool}
  subject item
  mode given item yields value: one

relation progress
  reads {item: Item} progress {value: F64}
  subject item
  mode given item yields value: one

first
  shape: Item
second
  shape: Item
shared-class
  shape: ItemClass
first item class shared-class
second item class shared-class
first selected false
second selected false
first progress 0.0
second progress 0.0

bind referent-input Pick as Item to select-item

on select-item ?item ?target
  when
    ?item selected ?prior
    ?item = ?target
  withdraw
    ?item selected ?prior
  include
    ?item selected true

on tick ?item ?dt
  when
    ?item selected true
    ?item progress ?prior
  withdraw
    ?item progress ?prior
  include
    ?item progress ?prior + ?dt
```

## Independent target selection and explicit contributions

Stores a typed Account input on an independent controller, then sums explicitly declared numeric contributions from eligible occurrences against the same pre-step state. Ordinary overlapping replacements reject atomically; accumulate does not imply source-order execution.

Catalog ID: `selected-account-contributions`

```clause
F64
Bool
Controller
Contributor
Account

relation chosen-account
  reads {controller: Controller} chosen account {value: Account}
  subject controller
  mode given controller yields value: one
relation balance
  reads {account: Account} balance {value: F64}
  subject account
  mode given account yields value: one
relation enabled
  reads {account: Account} enabled {value: Bool}
  subject account
  mode given account yields value: one
relation selected
  reads {contributor: Contributor} selected {value: Bool}
  subject contributor
  mode given contributor yields value: one
relation contribution
  reads {contributor: Contributor} contribution {value: F64}
  subject contributor
  mode given contributor yields value: one
relation cooldown
  reads {contributor: Contributor} cooldown {value: F64}
  subject contributor
  mode given contributor yields value: one

controller
  shape: Controller
first
  shape: Account
  shape: Contributor
second
  shape: Account
alpha
  shape: Contributor
beta
  shape: Contributor

controller chosen account first
first balance 100.0
first enabled true
second balance 200.0
second enabled true
first selected false
first contribution 50.0
first cooldown 0.0
alpha selected true
alpha contribution 7.0
alpha cooldown 0.0
beta selected true
beta contribution 11.0
beta cooldown 0.0

bind referent-input Choose as Account to choose-account
bind referent-input Select as Contributor to select-contributor
bind keyboard Apply down to contribute

on choose-account ?controller ?picked
  when
    ?controller chosen account ?prior
    ?picked balance ?balance
  withdraw
    ?controller chosen account ?prior
  include
    ?controller chosen account ?picked

on select-contributor ?contributor ?picked
  when
    ?contributor selected ?prior
    ?contributor = ?picked
  withdraw
    ?contributor selected ?prior
  include
    ?contributor selected true

on contribute ?contributor
  when
    ?contributor selected true
    ?contributor contribution ?amount
    ?contributor cooldown ?cooldown
    ?cooldown <= 0.0
    controller chosen account ?account
    ?account balance ?balance
    ?account enabled true
  withdraw
    ?contributor cooldown ?cooldown
  include
    ?contributor cooldown 1.0
  accumulate
    ?account balance ?amount

on tick ?contributor ?dt
  when
    ?contributor cooldown ?cooldown
    ?cooldown > 0.0
  withdraw
    ?contributor cooldown ?cooldown
  include
    ?contributor cooldown ?cooldown - ?dt
```

## Text state transition

Accepts bounded UTF-8 text as handler input, stores it in optional state, and replaces it atomically.

Catalog ID: `text-state-transition`

```clause
North
GoalState
Text

relation goal-state
  reads {north: North} goal state {value: GoalState}
  subject north
  mode given north yields value: one

relation goal-title
  reads {north: North} goal title {value: Text}
  subject north
  mode given north yields value: maybe

relation goal-objective
  reads {north: North} goal objective {value: Text}
  subject north
  mode given north yields value: maybe

relation goal-tags
  reads {north: North} goal tags {value: Text}
  subject north
  mode given north yields value: many

relation banner
  reads {north: North} banner {value: Text}
  subject north
  mode given north yields value: one

north-main
  shape: North
north-main goal state no-goal
north-main banner "North says:\n\"ready\" 🚀"

on create-goal ?north ?title ?objective
  when
    ?north goal state ?state
    ?state = no-goal
    ?objective = "North handles goals elegantly 🚀"
  withdraw
    ?north goal state ?state
  include
    ?north goal state active
    ?north goal title ?title
    ?north goal objective ?objective
    ?north goal tags ?title

on tag-goal ?north ?tag
  when
    ?north goal state ?state
    ?state = active
  withdraw
    ?north goal state ?state
  include
    ?north goal state ?state
    ?north goal tags ?tag

on redirect-goal ?north ?objective
  when
    ?north goal state ?state
    ?state = active
    ?north goal objective ?previous
  withdraw
    ?north goal state ?state
    ?north goal objective ?previous
  include
    ?north goal state ?state
    ?north goal objective "North " ++ ?objective
```

## Multiline Text output

Projects an indented multiline Text value while preserving the document's own quotes, layout, and final newline.

Catalog ID: `multiline-text-output`

```clause
Document
Text

relation output
  reads {document: Document} output {value: Text}
  subject document
  mode given document yields value: one

document-main output """
  initial
  """

on render ?document
  when
    ?document output ?previous
  withdraw
    ?document output ?previous
  include
    ?document output """
      {
        title = "North";
        outputs = { nixpkgs, ... }: "Clause emits readable text";
      }
      """
```

## Runtime-created Referent and keyed rows

Creates one typed Referent inside a handler, uses it as the key for several relational rows, and retains immutable Text history on redirect.

Catalog ID: `dynamic-relational-rows`

```clause
North
Goal
GoalStatus
Text

relation known-goal
  reads {north: North} known goal {value: Goal}
  subject north
  mode given north yields value: many

relation goal-title
  reads {goal: Goal} title {value: Text}
  subject goal
  mode given goal yields value: maybe

relation goal-objective
  reads {goal: Goal} objective {value: Text}
  subject goal
  mode given goal yields value: maybe

relation goal-status
  reads {goal: Goal} status {value: GoalStatus}
  subject goal
  mode given goal yields value: maybe

relation prior-goal-objective
  reads {goal: Goal} prior objective {value: Text}
  subject goal
  mode given goal yields value: many

relation goal-catalog-state
  reads {north: North} goal catalog state {value: GoalStatus}
  subject north
  mode given north yields value: one

north-main
  shape: North
ready
  shape: GoalStatus
active
  shape: GoalStatus
north-main goal catalog state ready

on create-goal ?north ?title ?objective
  when
    ?north goal catalog state ?catalog
  create
    ?goal
      shape: Goal
  withdraw
    ?north goal catalog state ?catalog
  include
    ?north goal catalog state ?catalog
    ?north known goal ?goal
    ?goal title ?title
    ?goal objective ?objective
    ?goal status active

on redirect-goal ?north ?goal ?objective
  when
    ?north known goal ?goal
    ?goal objective ?previous
    ?goal status ?status
    ?status = active
  withdraw
    ?goal objective ?previous
  include
    ?goal prior objective ?previous
    ?goal objective ?objective
```

## Finite created relations and per-occurrence contributions

Joins actual runtime-created Goal rows, updates each matching timer, and accumulates each distinct occurrence against one pre-step account balance. Equal-valued creations remain distinct; exact withdrawal removes only its own row. Finite resource exhaustion is an error, never absence. See docs/created-collections.md for bounds and remaining limits.

Catalog ID: `created-timed-contributions`

```clause
F64
Account
Goal

relation balance
  reads {account: Account} balance {value: F64}
  subject account
  mode given account yields value: one
relation known-goal
  reads {account: Account} known goal {value: Goal}
  subject account
  mode given account yields value: many
relation contribution
  reads {goal: Goal} contribution {value: F64}
  subject goal
  mode given goal yields value: one
relation remaining
  reads {goal: Goal} remaining {value: F64}
  subject goal
  mode given goal yields value: one

account
  shape: Account
account balance 100.0

on create-goal ?account ?amount ?duration
  when
    ?account balance ?balance
  create
    ?goal
      shape: Goal
  include
    ?account known goal ?goal
    ?goal contribution ?amount
    ?goal remaining ?duration

on tick ?account ?dt
  when
    ?account known goal ?goal
    ?goal contribution ?amount
    ?goal remaining ?remaining
    ?remaining > 0.0
  withdraw
    ?goal remaining ?remaining
  include
    ?goal remaining ?remaining - ?dt
  accumulate
    ?account balance ?amount * ?dt

on expire ?account
  when
    ?account known goal ?goal
    ?goal remaining ?remaining
    ?remaining <= 0.0
  withdraw
    ?account known goal ?goal
    ?goal remaining ?remaining

on cancel-goal ?account ?goal
  when
    ?account known goal ?goal
    ?goal remaining ?remaining
  withdraw
    ?account known goal ?goal
    ?goal remaining ?remaining
```

## Derived combat transition

Authorizes scalar laws, binds their result in a handler, and publishes one atomic multi-state combat change.

Catalog ID: `derived-combat-transition`

```clause
F64
Actor
Move
CombatRules

relation clamped-between
  reads {value: F64} clamped between {lower: F64} and {upper: F64} as {result: F64}
  mode given value lower upper yields result: maybe

relation vitality
  reads {actor: Actor} vitality {value: F64}
  subject actor
  mode given actor yields value: one

relation destabilization
  reads {actor: Actor} destabilization {value: F64}
  subject actor
  mode given actor yields value: one

relation mass
  reads {actor: Actor} mass {value: F64}
  subject actor
  mode given actor yields value: one

relation launch-velocity
  reads {actor: Actor} launch velocity {value: F64}
  subject actor
  mode given actor yields value: one

relation damage
  reads {move: Move} damage {value: F64}
  subject move
  mode given move yields value: one

relation destabilization-gain
  reads {move: Move} destabilization gain {value: F64}
  subject move
  mode given move yields value: one

relation base-impulse
  reads {move: Move} base impulse {value: F64}
  subject move
  mode given move yields value: one

relation launch-growth
  reads {move: Move} launch growth {value: F64}
  subject move
  mode given move yields value: one

relation destabilization-threshold
  reads {rules: CombatRules} destabilization threshold {value: F64}
  subject rules
  mode given rules yields value: one

law clamp-lower
  if
    ?lower <= ?upper
    ?value < ?lower
  then
    ?value clamped between ?lower and ?upper as ?lower

law clamp-interior
  if
    ?lower <= ?value
    ?value <= ?upper
  then
    ?value clamped between ?lower and ?upper as ?value

law clamp-upper
  if
    ?lower <= ?upper
    ?value > ?upper
  then
    ?value clamped between ?lower and ?upper as ?upper

derive clamp-lower
derive clamp-interior
derive clamp-upper

magitek-boar
  shape: Actor
blade-two
  shape: Move
combat-rules
  shape: CombatRules

magitek-boar vitality 100.0
magitek-boar destabilization 100.0
magitek-boar mass 1.25
magitek-boar launch velocity 0.0
blade-two damage 14.0
blade-two destabilization gain 35.0
blade-two base impulse 5.0
blade-two launch growth 8.0
combat-rules destabilization threshold 100.0

on probe ?defender
  when
    ?defender vitality ?vitality
  withdraw
    ?defender vitality ?vitality
  include
    ?defender vitality ?vitality + 0.0

on blade-two-hit ?defender
  when
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
    ?defender mass ?mass
    ?defender launch velocity ?launch
    blade-two damage ?damage
    blade-two destabilization gain ?gain
    blade-two base impulse ?impulse
    blade-two launch growth ?growth
    combat-rules destabilization threshold ?threshold
    (?destabilization + ?gain) clamped between 0.0 and ?threshold as ?next-destabilization
  withdraw
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
    ?defender launch velocity ?launch
  include
    ?defender vitality ?vitality - ?damage
    ?defender destabilization ?next-destabilization
    ?defender launch velocity (?impulse + ?growth * ?next-destabilization / ?threshold) / ?mass
```

## Relational Nix flake

Selects the compiler-owned Nix vocabulary and describes a development shell entirely through typed focused relations.

Catalog ID: `relational-nix-flake`

```clause
using Nix

clause
  shape: Flake
  description: "Clause development environment"
  inputs
    nixpkgs
      from: "github:NixOS/nixpkgs/nixos-unstable"
    rust-overlay
      from: "github:oxalica/rust-overlay"
      follows: nixpkgs
  development shell
    clause-shell
      system: x86_64-linux
      imports: nixpkgs
      overlays
        rust-overlay
      includes
        rust
          from: "./rust-toolchain.toml"
```

## Symbolic relations compose

Defines absolute value with ordinary guarded laws and a symbolic Reading, then composes two uses in one transition. No formula name selects compiler behavior.

Catalog ID: `composed-scalar-laws`

```clause
F64
Meter

relation magnitude
  reads | {input: F64} | = {output: F64}
  mode given input yields output: maybe

law negative-magnitude
  if
    ?x < 0.0
  then
    | ?x | = (0.0 - ?x)

law nonnegative-magnitude
  if
    ?x >= 0.0
  then
    | ?x | = ?x

derive negative-magnitude
derive nonnegative-magnitude

relation reading
  reads {meter: Meter} reading {value: F64}
  subject meter
  mode given meter yields value: one

meter-1
  shape: Meter
meter-1 reading -4.0

on rectify ?meter
  when
    ?meter reading ?x
    | ?x | = ?magnitude
    | (?magnitude - 10.0) | = ?next
  withdraw
    ?meter reading ?x
  include
    ?meter reading ?next
```
