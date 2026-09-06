# Clause authoring card

This card is generated from compiler-owned examples. It is a curated current vocabulary, not an exhaustive language specification. The checked examples and diagnostics from the consuming project's immutable Clause compiler pin are authoritative.

Use that pin's workbench directly:

- `clause-workbench authoring-card` prints this card.
- `clause-workbench check-source FILE.clause` reads, elaborates, lowers, and opens the source in the resident execution workbench.
- `clause-workbench project-nix FILE.clause [OUTPUT]` checks `using Nix` relations and renders their typed flake projection.

Live source tooling offers an explicit checked scalar-effect replacement, not arbitrary text-reload continuity. Use `scalar_effects()` and `edit_scalar_effect()` with the captured generation and exact offered node; settle any pending candidate first. Native and Wasm carry the actual live world internally through the checked operation. Retained explanations describe accepted Steps; finite interventions query an isolated recorded pre-state without applying input or admitting a world. See `docs/live-source-semantics.md` for the compiler/runtime and passive browser contract, bounds, and remaining limits.

## Positioned measurement occurrences

Calibrate each active batch's readings once while preserving its occurrence identities and explicit positions. Equal readings remain independent, including newly appended measurements. An invalid numeric result rejects the whole change.

Catalog ID: `ordered-measurements`

```clause
F64
Bool
Measurement
Batch

position
  domain: Measurement
  range: F64
  cardinality: one
reading
  domain: Measurement
  range: F64
  cardinality: one
batch
  domain: Measurement
  range: Batch
  cardinality: one
active
  domain: Batch
  range: Bool
  cardinality: one

samples
  active: true
archive
  active: false

last
  position: 2.0
  reading: 9.0
  batch: samples
first
  position: 0.0
  reading: 4.0
  batch: samples
middle
  position: 1.0
  reading: 4.0
  batch: samples
saved
  position: 0.0
  reading: 16.0
  batch: archive

on calibrate ?batch ?divisor
  when
    ?batch active true
    ?measurement batch ?batch
    ?measurement position ?position
    ?measurement reading ?reading
  withdraw
    ?measurement reading ?reading
  include
    ?measurement reading sqrt(?reading) / ?divisor

on append ?batch ?position ?reading
  when
    ?batch active true
  create
    ?measurement
  include
    ?measurement batch ?batch
    ?measurement position ?position
    ?measurement reading ?reading
```

## Bindings, focus, and structured values

Ordinary conformance premises constrain the same bindings in declarations, laws, and handlers. Shared-edge focus states a common constraint once and keeps the Reading clean; executable direction remains a separate Mode. A role's declared range selects ordinary field edges for values and patterns, with no repeated constructor or field Referents. A four-binding numeric law and typed field replacement share one atomic transition; checked scalar-effect edits retain the live state identities.

Catalog ID: `coherent-declarations`

```clause
F64
Bool
Item

Point:
  x: F64
  y: F64

limited:
  (shape: F64):
    ?amount ?minimum ?maximum ?result
  ?amount:
    limited between ?minimum and ?maximum as: ?result

mode limited given amount minimum maximum yields result: maybe

law below
  if
    ?minimum:
      shape: F64
    ?minimum <= ?maximum
    ?amount < ?minimum
  then
    ?amount:
      limited between ?minimum and ?maximum as: ?minimum
law inside
  if
    ?minimum <= ?amount
    ?amount <= ?maximum
  then
    ?amount:
      limited between ?minimum and ?maximum as: ?amount
law above
  if
    ?minimum <= ?maximum
    ?amount > ?maximum
  then
    ?amount limited between ?minimum and ?maximum as ?maximum
derive below
derive inside
derive above

position
  domain: Item
  range: Point
  cardinality: one
destination
  domain: Item
  range: Point
  cardinality: one
charge
  domain: Item
  range: F64
  cardinality: one

first
  position:
    x: 2.0
    y: 3.0
  destination:
    x: 8.0
    y: 9.0
  charge: 9.0
second
  position:
    x: 4.0
    y: 5.0
  destination:
    x: 0.0
    y: 0.0
  charge: -4.0

on settle ?item
  when
    ?item:
      position:
        x: ?x
        y: ?y
      destination: ?destination
      charge: ?charge
    (shape: F64):
      ?charge ?limited
    (?charge + 3.0) limited between 0.0 and 10.0 as ?limited
  withdraw
    ?item:
      destination: ?destination
      charge: ?charge
  include
    ?item:
      destination:
        x: ?x
        y: ?y
      charge: ?limited
```

## Derived structured totals

A law can bind finite sums and derive one optional structured value shared by its consumers. Aggregate queries read completed prerequisite relations, including positive recursive closure. Source changes recompute totals atomically; cycles through aggregate dependencies reject without publishing partial values.

Catalog ID: `derived-capacity`

```clause
F64
Text
Component
Workshop
Capacity:
  mass: F64
mass
  domain: Component
  range: F64
  cardinality: one
phase
  domain: Workshop
  range: Text
  cardinality: one
capacity
  domain: Workshop
  range: Capacity
  cardinality: maybe
core
  mass: 8.0
drive
  mass: 7.0
workshop
  phase: "Workshop"
law component-capacity
  if
    ?workshop phase ?phase
    sum ?mass where { ?component mass ?mass } as ?total
  then
    ?workshop capacity Capacity { mass: ?total }
derive component-capacity
on inspect ?workshop
  when
    ?workshop phase ?phase
    ?workshop capacity Capacity { mass: ?mass }
  withdraw
    ?workshop phase ?phase
  include
    ?workshop phase ?phase
```

## Reusable optional structured relations

Positive laws may derive a cardinality-maybe value, including a structured value. Queries consume the same current selection, liveness and health definition. Equal proofs share one value; conflicting conclusions reject, and withdrawn premises remove their consequences.

Catalog ID: `optional-derived-formation`

```clause
F64
Bool
Item
Report

Point:
  x: F64
  z: F64

selected:
  ?item shape Item
  ?value shape Bool
  ?item:
    selected: ?value

mode selected given item yields value: one
alive:
  ?item shape Item
  ?value shape Bool
  ?item:
    alive: ?value

mode alive given item yields value: one
health:
  ?item shape Item
  ?value shape F64
  ?item:
    health: ?value

mode health given item yields value: one
offset:
  ?item shape Item
  ?value shape Point
  ?item:
    offset: ?value

mode offset given item yields value: one
formation:
  ?item shape Item
  ?value shape Point
  ?item:
    formation: ?value

mode formation given item yields value: maybe
total:
  ?report shape Report
  ?value shape F64
  ?report:
    total: ?value

mode total given report yields value: one

law selected-living-formation
  if
    ?item selected true
    ?item alive true
    ?item health ?health
    ?health > 0.0
    ?item offset ?offset
  then
    ?item formation ?offset
derive selected-living-formation

first
  member of: Item
second
  member of: Item
report
  member of: Report
first selected true
first alive true
first health 1.0
first
  offset:
    x: 2.0
    z: 3.0
second selected true
second alive true
second health 1.0
second
  offset:
    x: 2.0
    z: 3.0
report total 0.0

on measure ?report
  when
    ?report total ?prior
    sum ?x where { ?item formation Point { x: ?x, z: ?z } } as ?total
  withdraw
    ?report total ?prior
  include
    ?report total ?total

on select ?item ?selected
  when
    ?item selected ?prior
  withdraw
    ?item selected ?prior
  include
    ?item selected ?selected

on vitality ?item ?health
  when
    ?item health ?prior
  withdraw
    ?item health ?prior
  include
    ?item health ?health

on living ?item ?alive
  when
    ?item alive ?prior
  withdraw
    ?item alive ?prior
  include
    ?item alive ?alive
```

## Checked laws inside checked laws

An acyclic scalar law may call another typed scalar law in its premises. Both execution and feedback consume that definition. Nested laws retain their source origins, and bounded expansion rejects recursion or exhaustion rather than guessing a result.

Catalog ID: `nested-readiness`

```clause
F64
Bool
Text
Device

device-readiness:
  ?enabled shape Bool
  ?charge shape F64
  ?message shape Text
  device enabled ?enabled charge ?charge reports ?message

mode device-readiness given enabled charge yields message: maybe
law device-readiness
  if
    ?enabled enabled with charge ?charge reports ?result
  then
    device enabled ?enabled charge ?charge reports ?result
derive device-readiness

readiness:
  ?enabled shape Bool
  ?charge shape F64
  ?message shape Text
  ?enabled enabled with charge ?charge reports ?message

mode readiness given enabled charge yields message: maybe
enabled:
  ?device shape Device
  ?value shape Bool
  ?device:
    enabled: ?value

mode enabled given device yields value: one
charge:
  ?device shape Device
  ?value shape F64
  ?device:
    charge: ?value

mode charge given device yields value: one
message:
  ?device shape Device
  ?value shape Text
  ?device:
    message: ?value

mode message given device yields value: one

law disabled
  if
    ?enabled = false
  then
    ?enabled enabled with charge ?charge reports "Disabled"
law empty
  if
    ?enabled = true
    ?charge <= 0.0
  then
    ?enabled enabled with charge ?charge reports "Empty"
law ready
  if
    ?enabled = true
    ?charge > 0.0
  then
    ?enabled enabled with charge ?charge reports "Ready"
derive disabled
derive empty
derive ready

device
  member of: Device
device enabled true
device charge 1.0
device message "Unchecked"

on inspect ?device
  when
    ?device enabled ?enabled
    ?device charge ?charge
    ?device message ?prior
    device enabled ?enabled charge ?charge reports ?message
  withdraw
    ?device message ?prior
  include
    ?device message ?message

on use ?device
  when
    ?device enabled ?enabled
    ?device charge ?charge
    device enabled ?enabled charge ?charge reports ?message
    ?message = "Ready"
  withdraw
    ?device charge ?charge
  include
    ?device charge ?charge - 1.0
```

## Composable text search

contains-text(text, query) tests exact substring membership, including an empty query. lowercase(text) applies Unicode lowercase mapping, not locale-specific collation or Unicode normalization. Compose them explicitly for case-insensitive search; both inputs remain typed Text.

Catalog ID: `text-search`

```clause
Text
Bool
Document

text:
  ?document shape Document
  ?text shape Text
  ?document:
    text: ?text

mode text given document yields text: one
matches:
  ?document shape Document
  ?matches shape Bool
  ?document:
    matches: ?matches

mode matches given document yields matches: one

document
  member of: Document
document text ""
document matches false

on search ?document ?text ?query
  when
    ?document text ?prior
    ?document matches ?matched
  withdraw
    ?document text ?prior
    ?document matches ?matched
  include
    ?document text lowercase(?text)
    ?document matches contains-text(lowercase(?text), lowercase(?query))
```

## Exact text-valued conditions

A focused relation condition matches a Text literal using the same typed row equality as numbers and Booleans. Quoting, Unicode, and escaping have their ordinary Text meaning, including for runtime-created subjects.

Catalog ID: `text-selectors`

```clause
Text
Bool
Item

status:
  ?item shape Item
  ?status shape Text
  ?item:
    status: ?status

mode status given item yields status: one

selected:
  ?item shape Item
  ?selected shape Bool
  ?item:
    selected: ?selected

mode selected given item yields selected: one

first
  member of: Item
second
  member of: Item

first status "waiting {世界} \"yes\""
first selected false
second status "finished"
second selected false

on select ?item
  when
    ?item status "waiting {世界} \"yes\""
    ?item selected ?prior
  withdraw
    ?item selected ?prior
  include
    ?item selected true

on spawn ?item
  when
    ?item status "finished"
  create
    ?new
      member of: Item
  include
    ?new status "waiting {世界} \"yes\""
    ?new selected false
```

## Ordinary role contracts

Domain, range, and cardinality facts constrain a binary role. Actual properties establish structural participation without a membership registry. Required properties remain checked across atomic changes.

Catalog ID: `role-contracts`

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

## Recursive dependencies with withdrawal

Positive laws derive blockers through prerequisites. Independent support preserves a conclusion; removing the last root removes its consequences, even through cycles. Bounded closure either completes or returns an error without admitting a prefix.

Catalog ID: `recursive-dependencies`

```clause
Task
Root
F64
Text

duration
  domain: Task
  range: F64
  cardinality: one
reason
  domain: Root
  range: Text
  cardinality: one

prerequisite
  domain: Task
  range: Task
  cardinality: many
obstruction
  domain: Task
  range: Root
  cardinality: many
blocker
  domain: Task
  range: Root
  cardinality: many

approval
  reason: "Review approval is missing"
material
  reason: "Build material is unavailable"

build
  duration: 3.0
  prerequisite: design
review
  duration: 1.0
  prerequisite: build
design
  duration: 2.0
  prerequisite: review
  obstruction: approval
  obstruction: material

law direct-obstruction
  if
    ?task obstruction ?root
  then
    ?task blocker ?root
derive direct-obstruction

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

on inspect ?task
  when
    ?task obstruction ?root
  include
    ?task obstruction ?root

on resolve ?task ?root
  when
    ?task obstruction ?root
  withdraw
    ?task obstruction ?root

on obstruct ?task ?root
  when
    ?task prerequisite ?prior
  include
    ?task obstruction ?root
```

## Typed Unicode text operations

trim(text), first-word(text), remaining-words(text), and starts-with(text, prefix) parse bounded UTF-8 text inside checked source. Word boundaries use Unicode whitespace; the remainder preserves internal and trailing whitespace. Empty text yields empty words. Prefix comparison is exact and case-sensitive.

Catalog ID: `text-operations`

```clause
Text
Bool
Document

cleaned:
  ?document shape Document
  ?cleaned shape Text
  ?document:
    cleaned: ?cleaned

mode cleaned given document yields cleaned: one

first-word:
  ?document shape Document
  ?first-word shape Text
  ?document:
    first word: ?first-word

mode first-word given document yields first-word: one

remaining-words:
  ?document shape Document
  ?remaining-words shape Text
  ?document:
    remaining words: ?remaining-words

mode remaining-words given document yields remaining-words: one

prefixed:
  ?document shape Document
  ?prefixed shape Bool
  ?document:
    prefixed: ?prefixed

mode prefixed given document yields prefixed: one

document-main prefixed false
document-main cleaned ""
document-main first word ""
document-main remaining words ""

on tokenize ?document ?input
  when
    ?document cleaned ?cleaned
    ?document first word ?first
    ?document remaining words ?remaining
    ?document prefixed ?prefixed
  withdraw
    ?document cleaned ?cleaned
    ?document first word ?first
    ?document remaining words ?remaining
    ?document prefixed ?prefixed
  include
    ?document cleaned trim(?input)
    ?document first word first-word(?input)
    ?document remaining words remaining-words(?input)
    ?document prefixed starts-with(trim(?input), "/")
```

## Reusable checked laws inside finite queries

Query-local scalar laws compose with typed rows, explicit inputs and predicates. Each matching row contributes once even when equal-result law cases overlap. A missing law result excludes that row; an invalid expression or exhausted search remains an error.

Catalog ID: `query-laws`

```clause
F64
Item
Report

magnitude:
  (shape: F64):
    ?input ?output
  magnitude of ?input as ?output

mode magnitude given input yields output: maybe
law negative
  if
    ?value < 0.0
  then
    magnitude of ?value as 0.0 - ?value
law positive
  if
    ?value >= 0.0
  then
    magnitude of ?value as ?value
derive negative
derive positive

amount:
  ?item shape Item
  ?amount shape F64
  ?item:
    amount: ?amount

mode amount given item yields amount: one
total:
  ?report shape Report
  ?total shape F64
  ?report:
    total: ?total

mode total given report yields total: one

first
  member of: Item
second
  member of: Item
report
  member of: Report
first amount -3.0
second amount 4.0
report total 0.0

on measure ?report
  when
    ?report total ?prior
    sum ?magnitude where { ?item amount ?value; magnitude of ?value as ?magnitude } as ?sum
  withdraw
    ?report total ?prior
  include
    ?report total ?sum
```

## Typed lazy value choice

if(condition, yes, no) requires Bool and two values of the expected type. Only the selected branch executes; both branches are checked. It composes with source laws, query contributions, structured fields and atomic updates.

Catalog ID: `scalar-conditional`

```clause
F64
Meter

reading:
  ?meter shape Meter
  ?reading shape F64
  ?meter:
    reading: ?reading

mode reading given meter yields reading: one

meter
  member of: Meter
meter reading 0.0

on measure ?meter
  when
    ?meter reading ?value
  withdraw
    ?meter reading ?value
  include
    ?meter reading if(?value > 0.0, 10.0 / ?value, 0.0)

on inspect ?meter
  when
    ?meter reading ?value
  withdraw
    ?meter reading ?value
  include
    ?meter reading ?value
```

## Explicit finite-query inputs

A query's given list passes exact typed values from the enclosing rule. All other query variables remain local. Count matching optional rows to distinguish presence from absence, including runtime-created referents and withdrawal; exhaustion still fails explicitly.

Catalog ID: `query-inputs`

```clause
F64
Bool
Device

charge:
  ?device shape Device
  ?charge shape F64
  ?device:
    charge: ?charge

mode charge given device yields charge: maybe
available:
  ?device shape Device
  ?available shape Bool
  ?device:
    available: ?available

mode available given device yields available: one

first
  member of: Device
second
  member of: Device
first charge 2.0
first available false
second available false

on inspect ?device
  when
    ?device available ?prior
    sum 1.0 given ?device where { ?device charge ?charge; ?charge > 0.0 } as ?count
  withdraw
    ?device available ?prior
  include
    ?device available (?count > 0.0)

on deplete ?device
  when
    ?device charge ?charge
  withdraw
    ?device charge ?charge

on spawn ?device
  when
    ?device charge ?charge
  create
    ?new
      member of: Device
  include
    ?new charge ?charge
    ?new available false
```

## Checked scalar equality

Equality expressions compare matching Boolean, numeric or Text values and produce Bool. Toggle uses the current admitted pre-state, including runtime-created rows; mixed scalar types are rejected.

Catalog ID: `scalar-equality`

```clause
Bool
Item

selected:
  ?item shape Item
  ?selected shape Bool
  ?item:
    selected: ?selected

mode selected given item yields selected: one

first
  member of: Item
first selected false

on toggle ?item
  when
    ?item selected ?prior
  withdraw
    ?item selected ?prior
  include
    ?item selected (?prior = false)
```

## Boolean results from scalar comparisons

Ordered F64 comparisons >, >=, < and <= produce Bool values. Numeric arithmetic binds more tightly; all assigned values read the same pre-transition state, so a numeric update and its completion flag can be one atomic rule.

Catalog ID: `scalar-comparison`

```clause
F64
Bool
Meter

reading:
  ?meter shape Meter
  ?reading shape F64
  ?meter:
    reading: ?reading

mode reading given meter yields reading: one
positive:
  ?meter shape Meter
  ?positive shape Bool
  ?meter:
    positive: ?positive

mode positive given meter yields positive: one

meter
  member of: Meter
meter reading 25.0
meter positive false

on measure ?meter
  when
    ?meter reading ?value
    ?meter positive ?prior
  withdraw
    ?meter reading ?value
    ?meter positive ?prior
  include
    ?meter reading 0.0
    ?meter positive (?value > 0.0)
```

## Atomic structured value copies

Copy a whole typed record between relations while changing other state in the same rule. All fields read one pre-transition state; runtime-created rows use the same rule and incompatible record types are rejected.

Catalog ID: `structured-value-copy`

```clause
F64
Bool
Item

Point:
  x: F64
  y: F64

position:
  ?item shape Item
  ?position shape Point
  ?item:
    position: ?position

mode position given item yields position: one
destination:
  ?item shape Item
  ?destination shape Point
  ?item:
    destination: ?destination

mode destination given item yields destination: one
moving:
  ?item shape Item
  ?moving shape Bool
  ?item:
    moving: ?moving

mode moving given item yields moving: one

item
  member of: Item
item
  position:
    x: 2.0
    y: 3.0
item
  destination:
    x: 8.0
    y: 9.0
item moving true

on stop ?item
  when
    ?item position ?position
    ?item destination ?destination
    ?item moving ?prior
  withdraw
    ?item destination ?destination
    ?item moving ?prior
  include
    ?item destination ?position
    ?item moving false
```

## Checked scalar square root

sqrt(expression) computes the finite F64 square root, including zero. It composes with arithmetic and source-law bindings; nonnumeric values are rejected, and negative inputs fail without admitting a changed world.

Catalog ID: `scalar-square-root`

```clause
F64
Meter

reading:
  ?meter shape Meter
  ?reading shape F64
  ?meter:
    reading: ?reading

mode reading given meter yields reading: one

meter
  member of: Meter
meter reading 25.0

on measure ?meter
  when
    ?meter reading ?value
  withdraw
    ?meter reading ?value
  include
    ?meter reading sqrt(?value)

on inspect ?meter
  when
    ?meter reading ?value
  withdraw
    ?meter reading ?value
  include
    ?meter reading ?value
```

## Closed finite query sums

Sums F64 contributions over exact finite row matches in the same pre-state. Query-local variables do not capture the enclosing handler; an empty query yields zero, distinct equal-valued referents contribute independently, and exhausted search is an error.

Catalog ID: `finite-sums`

```clause
F64
Bool
Item
Report

enabled:
  ?item shape Item
  ?enabled shape Bool
  ?item:
    enabled: ?enabled

mode enabled given item yields enabled: one
amount:
  ?item shape Item
  ?amount shape F64
  ?item:
    amount: ?amount

mode amount given item yields amount: one
total:
  ?report shape Report
  ?total shape F64
  ?report:
    total: ?total

mode total given report yields total: one
count:
  ?report shape Report
  ?count shape F64
  ?report:
    count: ?count

mode count given report yields count: one

first
  member of: Item
second
  member of: Item
report
  member of: Report
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
      member of: Item
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

balance:
  ?account shape Account
  ?balance shape F64
  ?account:
    balance: ?balance

mode balance given account yields balance: one

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

Vec3:
  x: F64
  y: F64
  z: F64

velocity:
  ?player shape Player
  ?velocity shape Vec3
  ?player:
    velocity: ?velocity

mode velocity given player yields velocity: one

empowered:
  ?player shape Player
  ?empowered shape Bool
  ?player:
    empowered: ?empowered

mode empowered given player yields empowered: one

player-1
  member of: Player
player-1
  velocity:
    x: 0.0
    y: 0.0
    z: 0.0
player-1 empowered true

bind keyboard KeyQ down to planar-burst

on planar-burst ?player
  when
    ?player
      velocity:
        x: ?velocity-x
        y: ?velocity-y
        z: ?velocity-z
    ?player empowered ?was-empowered
    ?was-empowered = true
  withdraw
    ?player
      velocity:
        x: ?velocity-x
        y: ?velocity-y
        z: ?velocity-z
  include
    ?player
      velocity:
        x: ?velocity-x + 3.0
        y: ?velocity-y
        z: ?velocity-z - 2.0
```

## Scalar input transition

Binds one named physical scalar channel to a typed one-argument handler and records its finite observed value.

Catalog ID: `scalar-input-transition`

```clause
F64
Player

camera-heading:
  ?player shape Player
  ?camera-heading shape F64
  ?player:
    camera heading: ?camera-heading

mode camera-heading given player yields camera-heading: one

player-1
  member of: Player
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

active:
  ?root shape Root
  ?active shape Item
  ?root:
    active: ?active

mode active given root yields active: one

known:
  ?root shape Root
  ?known shape Item
  ?root:
    known: ?known

mode known given root yields known: many

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

item-class:
  ?item shape Item
  ?item-class shape ItemClass
  ?item:
    item class: ?item-class

mode item-class given item yields item-class: one

selected:
  ?item shape Item
  ?selected shape Bool
  ?item:
    selected: ?selected

mode selected given item yields selected: one

progress:
  ?item shape Item
  ?progress shape F64
  ?item:
    progress: ?progress

mode progress given item yields progress: one

first
  member of: Item
second
  member of: Item
shared-class
  member of: ItemClass
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

chosen-account:
  ?controller shape Controller
  ?chosen-account shape Account
  ?controller:
    chosen account: ?chosen-account

mode chosen-account given controller yields chosen-account: one
balance:
  ?account shape Account
  ?balance shape F64
  ?account:
    balance: ?balance

mode balance given account yields balance: one
enabled:
  ?account shape Account
  ?enabled shape Bool
  ?account:
    enabled: ?enabled

mode enabled given account yields enabled: one
selected:
  ?contributor shape Contributor
  ?selected shape Bool
  ?contributor:
    selected: ?selected

mode selected given contributor yields selected: one
contribution:
  ?contributor shape Contributor
  ?contribution shape F64
  ?contributor:
    contribution: ?contribution

mode contribution given contributor yields contribution: one
cooldown:
  ?contributor shape Contributor
  ?cooldown shape F64
  ?contributor:
    cooldown: ?cooldown

mode cooldown given contributor yields cooldown: one

controller
  member of: Controller
first
  member of: Account
  member of: Contributor
second
  member of: Account
alpha
  member of: Contributor
beta
  member of: Contributor

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

goal-state:
  ?north shape North
  ?goal-state shape GoalState
  ?north:
    goal state: ?goal-state

mode goal-state given north yields goal-state: one

goal-title:
  ?north shape North
  ?goal-title shape Text
  ?north:
    goal title: ?goal-title

mode goal-title given north yields goal-title: maybe

goal-objective:
  ?north shape North
  ?goal-objective shape Text
  ?north:
    goal objective: ?goal-objective

mode goal-objective given north yields goal-objective: maybe

goal-tags:
  ?north shape North
  ?goal-tags shape Text
  ?north:
    goal tags: ?goal-tags

mode goal-tags given north yields goal-tags: many

banner:
  ?north shape North
  ?banner shape Text
  ?north:
    banner: ?banner

mode banner given north yields banner: one

north-main
  member of: North
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

output:
  ?document shape Document
  ?output shape Text
  ?document:
    output: ?output

mode output given document yields output: one

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

known-goal:
  ?north shape North
  ?known-goal shape Goal
  ?north:
    known goal: ?known-goal

mode known-goal given north yields known-goal: many

goal-title:
  ?goal shape Goal
  ?goal-title shape Text
  ?goal:
    title: ?goal-title

mode goal-title given goal yields goal-title: maybe

goal-objective:
  ?goal shape Goal
  ?goal-objective shape Text
  ?goal:
    objective: ?goal-objective

mode goal-objective given goal yields goal-objective: maybe

goal-status:
  ?goal shape Goal
  ?goal-status shape GoalStatus
  ?goal:
    status: ?goal-status

mode goal-status given goal yields goal-status: maybe

prior-goal-objective:
  ?goal shape Goal
  ?prior-goal-objective shape Text
  ?goal:
    prior objective: ?prior-goal-objective

mode prior-goal-objective given goal yields prior-goal-objective: many

goal-catalog-state:
  ?north shape North
  ?goal-catalog-state shape GoalStatus
  ?north:
    goal catalog state: ?goal-catalog-state

mode goal-catalog-state given north yields goal-catalog-state: one

north-main
  member of: North
ready
  member of: GoalStatus
active
  member of: GoalStatus
north-main goal catalog state ready

on create-goal ?north ?title ?objective
  when
    ?north goal catalog state ?catalog
  create
    ?goal
      member of: Goal
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

balance:
  ?account shape Account
  ?balance shape F64
  ?account:
    balance: ?balance

mode balance given account yields balance: one
known-goal:
  ?account shape Account
  ?known-goal shape Goal
  ?account:
    known goal: ?known-goal

mode known-goal given account yields known-goal: many
contribution:
  ?goal shape Goal
  ?contribution shape F64
  ?goal:
    contribution: ?contribution

mode contribution given goal yields contribution: one
remaining:
  ?goal shape Goal
  ?remaining shape F64
  ?goal:
    remaining: ?remaining

mode remaining given goal yields remaining: one

account
  member of: Account
account balance 100.0

on create-goal ?account ?amount ?duration
  when
    ?account balance ?balance
  create
    ?goal
      member of: Goal
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

clamped-between:
  (shape: F64):
    ?value ?lower ?upper ?result
  ?value clamped between ?lower and ?upper as ?result

mode clamped-between given value lower upper yields result: maybe

vitality:
  ?actor shape Actor
  ?vitality shape F64
  ?actor:
    vitality: ?vitality

mode vitality given actor yields vitality: one

destabilization:
  ?actor shape Actor
  ?destabilization shape F64
  ?actor:
    destabilization: ?destabilization

mode destabilization given actor yields destabilization: one

mass:
  ?actor shape Actor
  ?mass shape F64
  ?actor:
    mass: ?mass

mode mass given actor yields mass: one

launch-velocity:
  ?actor shape Actor
  ?launch-velocity shape F64
  ?actor:
    launch velocity: ?launch-velocity

mode launch-velocity given actor yields launch-velocity: one

damage:
  ?move shape Move
  ?damage shape F64
  ?move:
    damage: ?damage

mode damage given move yields damage: one

destabilization-gain:
  ?move shape Move
  ?destabilization-gain shape F64
  ?move:
    destabilization gain: ?destabilization-gain

mode destabilization-gain given move yields destabilization-gain: one

base-impulse:
  ?move shape Move
  ?base-impulse shape F64
  ?move:
    base impulse: ?base-impulse

mode base-impulse given move yields base-impulse: one

launch-growth:
  ?move shape Move
  ?launch-growth shape F64
  ?move:
    launch growth: ?launch-growth

mode launch-growth given move yields launch-growth: one

destabilization-threshold:
  ?rules shape CombatRules
  ?destabilization-threshold shape F64
  ?rules:
    destabilization threshold: ?destabilization-threshold

mode destabilization-threshold given rules yields destabilization-threshold: one

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
  member of: Actor
blade-two
  member of: Move
combat-rules
  member of: CombatRules

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

magnitude:
  (shape: F64):
    ?input ?output
  | ?input | = ?output

mode magnitude given input yields output: maybe

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

reading:
  ?meter shape Meter
  ?reading shape F64
  ?meter:
    reading: ?reading

mode reading given meter yields reading: one

meter-1
  member of: Meter
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
