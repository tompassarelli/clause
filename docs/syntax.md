# Clause syntax

> **Authority:** This document defines canonical Clause source. The
> [foundation](foundation.md) defines meaning, [architecture](architecture.md)
> defines implementation boundaries, and the [roadmap](roadmap.md) identifies
> the implemented subset. Acceptance by an experimental reader does not make a
> spelling canonical.

## Source contract

Every construct elaborates to one or more independently identified semantic
emissions and one focus. Equal emitted Terms remain separate emissions with
separate origins, diagnostics, support, and occurrence identity.

A block head selects one child grammar before child meanings are inspected.
That grammar fixes which child forms are allowed and whether they receive the
parent focus. Indentation supplies containment; it never invents membership,
ownership, sequencing, a relation, or another domain fact.

An unkeyworded designation has two layout-selected forms:

- a leaf declares a Referent; and
- a head with one or more explicit edge children establishes subject focus.

Adding a child cannot reinterpret the head or an existing sibling. A bare
child under a subject is invalid because no role relates it:

```clause
Foo
  Bar
```

Canonical printing followed by independent elaboration preserves focus,
emission order and multiplicity, Terms, stances, and formation obligations.
Fresh identities are compared through a typed identity-domain-preserving
bijection. An edit retains an identity only through a checked continuity
witness; text similarity, source position, and emission ordinal are not
witnesses.

## Values, denotation, and focus

A top-level colon makes a name denote one value:

```clause
gravity: 9.81
rgb: 255, 0, 0
```

The comma creates an ordered product. Positions preserve order and repeated
equal values; they carry no domain role. Parentheses preserve nested product
boundaries.

Inside a subject-focus block, `role: object` applies an explicit role:

```clause
lamp
  charge: 2.0
```

The role gives the object domain meaning. The literal gives its value. A
separate role contract constrains admissibility, and a representation contract
governs storage.

Whitespace may occur in a role phrase. A colon fixes a leaf role boundary; an
indented object fixes a grouped role boundary:

```clause
north
  worker count: 5
  inputs
    nixpkgs
      from: "github:NixOS/nixpkgs/nixos-unstable"
    rust-overlay
      follows: nixpkgs
```

The `inputs` group emits separate applications from `north` to `nixpkgs` and
`rust-overlay`. Each object becomes focus for its nested edges. The group does
not create one collection value.

`name: value` is denotation, never structural conformance. Conformance is an
explicit role application such as `shape: Flake`; nominal membership is an
ordinary `member of: Group` application. Neither follows from the other.

## Declarations

These heads have distinct source jobs:

```clause
Door

enum Game
  Chess
  Soccer

Vec2:
  x: F32
  y: F32
```

A bare designation introduces or resolves a Referent. An `enum` child emits
one independent membership fact. A named structured contract gives its fields
directly; each field has one declared domain and needs no unused variable.

A Shape is a structural participation contract, not physical layout or nominal
membership. The resident checker currently enforces required field/role,
range, reference-target, scalar, and supported one/maybe/many cardinality
constraints; `some` has no runtime table lowering. The broader Mode, effect,
failure, and progress surface remains unimplemented.

There is no routine `model` head. Worlds, games, schedules, and scenes are
Referents related by ordinary Clause facts. Program identity and Admission
authority enter at their separate semantic boundaries, not through source
grouping.

## Ordinary role contracts

An ordinary binary role declares its subject domain, value range, and
cardinality:

```clause
duration
  domain: Task
  range: F64
  cardinality: one

prerequisite
  domain: Task
  range: Task
  cardinality: many
```

`one`, `maybe`, `some`, and `many` mean exactly one, at most one, at least
one, and unrestricted distinct values per subject. Cardinality is required
where the grammar asks for it and has no punctuation aliases.

The same role name and contract govern focused facts, flat patterns, nested
patterns, state tables, and edits. Authors do not repeat it in a registry,
reader template, or forward mode. Contracts sharing a domain jointly determine
structural participation; the checker derives conformance from actual facts.

Nominal domains without structural contracts still use explicit membership.
Membership never discharges a required structural property.

## Focused patterns

Within a law or handler section, `?task:` focuses its children on the same
logical variable:

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

Focus introduces no type or fact. The repeated variable spelling denotes one
binding. Flat and focused forms have the same relational meaning, and both may
occur in `if`, `then`, `when`, `withdraw`, `include`, and `accumulate`.
A focused variable head must have children.

An explicitly delimited edge can instead focus its subject position:

```clause
(charge: 9.0):
  first second
```

The parenthesized edge is read by the selected declared grammar. The final
colon selects shared-edge focus before reading its children. Each child token
is one named or variable subject, with its own source origin. Quoted
designations remain one token. Equal repeated subjects retain their source
occurrences but not a second value or identity. This is not a comma product.
An undelimited `charge: 9.0` remains denotation; adding children rejects rather
than turning it into shared-edge focus.

The children are a subject-token list selected by this head, not arbitrary
whitespace-separated sibling clauses. Subjects may share a line or occupy
separate lines at the same child indentation:

```clause
(charge: 9.0):
  first
  second
```

Both layouts apply `charge: 9.0` separately to `first` and `second`. Ordinary
subject blocks can express those two facts too. Equal charge values on two
subjects are two independent facts; repeating the value does not itself
violate the design discipline. Shared-edge focus groups an explicit common
role/object without merging those facts. The same construct can constrain
several bindings, as `(shape: F64):` does in the
[language tour](language-tour.md#bindings-and-readings).

## Declared readings and modes

Relations that need non-field phrase structure declare one exact Reading and
any executable directions:

```clause
connects:
  ?door shape Door
  (shape: Space):
    ?origin ?destination
  ?door:
    connects ?origin to: ?destination

mode connects given door origin yields destination: many
```

Ordinary `shape` clauses constrain the bindings in the Reading. Shared-edge
focus can state one domain for several explicitly named bindings, without a
default type or a separate declaration registry. These same conformance
premises check already-bound values in laws and handlers; they do not produce
values or fabricate world membership facts. Contradictory constraints reject.
Other words in the Reading are literal phrase tokens. Each binding occurs
exactly once in the Reading. Its focused clause selects the subject role
explicitly. A `mode`
names known and produced roles and the produced cardinality. The checked
semantic form separately retains RelationSchema, Reading, Operator, and Mode
identities.

A declaration without a matching `mode` defines only a schema and Reading. A schema
alone can check facts and patterns but cannot form an executable application.
An Operator may expose several Modes. Activation selects one Mode from the
exact eligible set established during formation.

Result cardinality is always explicit:

```clause
mode charge given device yields amount: one
mode charge given device yields amount: maybe
mode charge given device yields amount: some
mode charge given device yields amount: many
```

These words constrain produced value rows, not the representation of one value.
A relation-level `cardinality: one` additionally requires one value on every
participating subject; a Mode result of `one` guarantees one output only for
that computation.

## Terms and expressions

Surface Terms project to the foundation's `Atom | Triple` carrier. Accepted
forms include Booleans, integers, decimals, Text, products, sequences, and
declared shaped values:

```clause
true
42
9.81
"player"
(3.0, 4.0)
[3.0, 4.0]
```

A role whose declared range is a structured contract receives its value through
ordinary field edges. The range selects the field grammar before those children
are read; no repeated constructor or field Referent is introduced:

```clause
position
  domain: Particle
  range: Vec2
  cardinality: one

particle
  position:
    x: 3.0
    y: 4.0
```

The same field tree matches values in conditions and withdrawals and constructs
them in inclusions. A whole-value binding may be copied or replaced by fields;
field bindings may be used in a whole-value replacement. Field names, order,
completeness, and value types are checked against the one declared contract.
Fields retain their own source origins; they are not newly allocated Referents.

Declared readings may form nested expressions such as:

```clause
radius of coin + radius of player
length (position of player - position of coin)
```

`+ - * / < <= > >= = !=` have conventional precedence only where exact
declared relations support them. Multiplication and division bind tighter than
addition and subtraction, which bind tighter than order comparisons, which
bind tighter than equality. Arithmetic associates left; comparison and
equality do not chain. Boolean `and` binds less tightly than comparisons and
equality; `or` binds less tightly than `and`. Both require `Bool` operands,
associate left, and evaluate the left operand once. The right operand is evaluated
only when the left operand does not determine the result. Parentheses resolve any
remaining boundary.

`:` introduces denotation or a grammar-owned field. `=` is relational
equality. `->`, `:=`, `::`, and `~>` are not aliases.

The running scalar subset supports finite F64 expressions and composed laws.
It does not establish totality inference, a general constraint solver, or a
complete collection language.

## Laws and finite queries

A compact pure callable names one deterministic, single-result direction and
its definition. Arguments carry explicit types; a result annotation may state
an independent constraint:

```clause
export greeting(?name: Text): Text
  "Hello, {trim(?name)}."
```

The body returns its value without introducing world state. `export` selects
public visibility; omitting it keeps the callable private. The compiler retains
the relation, mode, definition, and export as separate semantic emissions.
Scalar argument and result types are `Text`, `F64`, and `Bool`. Effects,
unresolved bindings, and mismatched argument or result types reject in a pure
callable.

A callable may omit `: Result` when its checked body determines one exact type:

```clause
export description(?enabled: Bool)
  {status: {enabled: ?enabled, message: "Ready"}}
```

Every nested record field retains its inferred type. Forward calls use the
callee's checked result. Without an explicit alternative contract, conflicting
conditional branches reject. Unknown fields,
unresolved bindings, and empty sequences without an element contract reject.
An explicit result annotation remains an exact checking obligation.

Text interpolation inside a callable body evaluates the enclosed Clause
expression and requires Text; it performs no implicit numeric conversion.
Ordinary source text outside these callable bodies retains literal braces.
Callables may invoke named pure callables in the same source scope, including
later definitions. Each argument is checked and evaluated once before the body;
an unused argument can still fail. Conditional expressions evaluate only their
selected branch. Cycles and exhausted expansion bounds reject explicitly.

Callable value contracts also admit `Sequence<T>` and named structural Shapes.
A sequence preserves order and repeated equal values. A record checked against
a declared Shape supplies each field once, as in
`{message: "Missing name", status: 1}`; missing, extra, or mistyped fields reject.
The same recursive element and field contracts
check exported arguments, results, and foreign crossings. Equality compares
sequence elements in order and record fields structurally.

`characters(text)` returns an ordered `Sequence<Text>` containing one Unicode
scalar value per element, without normalization; empty Text yields an empty
sequence. `split-text(text, delimiter)` splits at nonoverlapping literal
occurrences from left to right and preserves every leading, interior, and
trailing empty field. An empty delimiter selects the same scalar sequence as
`characters`. Both arguments are Text. `count` continues to require a sequence.

`parse-integer-prefix(text)` returns `F64 | Text`. It skips leading ECMAScript
WhiteSpace and LineTerminator characters, accepts an optional ASCII sign, and
consumes the longest ASCII decimal digit prefix. A finite result is rounded to
F64, with either zero normalized to zero; no digits or a nonfinite result
returns the original Text. Trailing text is unconsumed and does not invalidate
the prefix. Match the alternatives before arithmetic or positivity checks.
The executable composition is in
[clause:test-vectors/authoring/text-decomposition.clause](../test-vectors/authoring/text-decomposition.clause).

An explicit contract `Execution | Diagnostic` admits either exact structural
value type. The alternatives are unordered and must be distinct and disjoint:
a value cannot satisfy two cases. Records with different field sets are disjoint;
records with the same fields can differ through disjoint field contracts.
Sequence alternatives overlap at the empty sequence and therefore reject.
Delayed and opaque foreign contracts are not alternatives in this slice.

When a result or argument requires an alternative contract, the checker includes
a value of one declared alternative, or a smaller alternative contract whose
members all belong to the declared contract. A conditional checks both
branches against the same contract; it does not infer a union from conflicting
branches. The runtime value retains its ordinary representation.

`match(value, ?request: Execution => body, ?error: Diagnostic => body)` evaluates
`value` once and selects the one case whose contract accepts it. Every alternative
must appear exactly once, even when the value is locally known. Missing,
duplicate, or unreachable cases reject. The case binding is available only
inside its body and has that case's exact type; accessing the other case's fields
rejects. Case bodies must have the same checked result type and only the selected
body executes. [The running dispatch example](../test-vectors/authoring/callable-outcomes.clause)
produces and consumes both execution requests and diagnostics through this rule.

`?reply.message` projects the declared `message` field. `drop(sequence, count)`
returns the ordered suffix after a nonnegative integer count.
`require(condition, value, message)` evaluates the Bool condition first; success
evaluates and returns the value, while rejection evaluates the Text message and
fails. Both alternatives are checked.

A foreign declaration identifies one actual module member and its checked
contract:

```clause
foreign arguments(): Sequence<Text>
  get: "argv"
  from: "node:process"
  failure: throw

foreign write-output(?fd: F64, ?message: Text): F64
  call: "writeSync"
  from: "node:fs"
  failure: throw
```

`get` reads a member without arguments; `call` invokes it with the declared
arguments. Module and member names are inert foreign identifiers. The explicit
`throw` contract permits an attempt to fail; it promises neither success nor
rollback. Ordinary callables reject foreign attempts and calls to procedures.
`procedure` selects an effect-permitting direction. Source checking retains
unresolved native foreign obligations; invoking one without a binding rejects.
JavaScript lowering binds the exact declared member and checks values crossing
that boundary. Exported declarations describe the same checked public types;
they do not import or emulate the TypeScript type system.

A private foreign declaration may quantify a structural record contract:

```clause
foreign echo<Body: Record>(?value: Body): Body
  call: "echo"
  from: "records"
  failure: throw
```

`Body` captures the complete actual record type at each call, including its
fields and nested values. It is not an unspecified record value. Repeating the
parameter in another argument or the result requires that same exact type.
The parameter must occur in an argument; `Sequence<Body>` and
`Delayed<target,Body>` preserve the same substitution. Every instantiated
foreign contract passes the ordinary value, target, effect and failure checks.
Ordinary callables, including exported source helpers, may also quantify a
Record parameter. Each call specializes and checks the body with the exact
argument types, retaining ordinary strict argument evaluation. Generic helpers
are source definitions; emitted callables and canonical foreign accesses
contain fully resolved types. Generic foreign declarations remain private.
Foreign declarations always require explicit result contracts.

`import "nixpkgs.clause"` brings the named source's foreign types and functions
into the consumer's checked scope. `check-source` and `compile-nix` resolve
the path relative to the consumer file; the source API accepts an explicit
finite map from import spellings to exact bytes. There is no ambient search.
This slice admits direct imports of foreign declarations and exported callable
definitions, without nested imports, renaming, or shadowing. Duplicate names
reject. Each imported definition retains its own source origin and uses the
same exact type and foreign binding checks as a local declaration. The complete
two-consumer example is in `clause:test-vectors/authoring/shared-foreign/`.

A `Dictionary<Value>` is a finite collection keyed by Text, with one exact
value contract shared by every entry. `dictionary(key, value)` constructs a
singleton dictionary; its value contract is inferred recursively or checked
against the expected dictionary contract. Keys are ordinary data, including
empty text and punctuation. A dictionary does not establish any statically
known field and cannot be projected with `field-at` or field syntax. Immediate
dictionaries use the same finite keyed representation as records; alternative
contracts reject when their accepted representations overlap.

A delayed Text key constructs a delayed dictionary in that same target. All
nested delayed values must belong to that target; construction incorporates
their exact contracts into the dictionary's element contract. For example,
`dictionary(username(), {enabled: true})` with a
`Delayed<nix,Text>` username has contract
`Delayed<nix,Dictionary<Settings>>`, where `Settings` has the exact
`enabled: Bool` field. Nix construction preserves the key expression until Nix
evaluation. The complete consumers are in
`clause:test-vectors/authoring/dynamic-user-modules/`.

A static field path fixes a nonempty sequence of field designations:

```clause
at-path(?path: FieldPath, ?value: Text)
  record-at(?path, ?value)

export specimen()
  at-path(path(details.title), "Clause")
```

`record-at(path(details.title), value)` constructs `{details: {title: value}}`
with the complete inferred type. `field-at(record, path(details.title))`
checks each selected field and returns its exact type. Static parameters accept
path literals or other static parameters; runtime Text, conditional path values,
and returning a path as a runtime value reject. Ordinary value arguments remain
strict and are bound once. Specialization has the same bounded expansion and
recursion checks as ordinary generic callables.

A delayed foreign declaration can use `get: ?path` (or `call: ?path`) when
`?path: FieldPath` is a declared parameter. It specializes the exact
foreign member path without passing a runtime argument. The declared external
root, eventual result, target and failure contract still apply at every use.
The path does not establish that an external member exists or that the foreign
contract is true; the selected foreign boundary must discharge that obligation.
See the complete executing modules in
`clause:test-vectors/authoring/static-modules/`.

A foreign declaration with `construction: "nix"` instead constructs a delayed
expression for that target. It performs no foreign attempt. Its written result
is the eventual value contract; the construction declaration derives the checked
`Delayed<nix,T>` result. Delayed values cannot be used as ordinary values or
compared during construction, and different targets cannot cross a construction
boundary. Call, record, sequence, and strict lexical binding syntax are unchanged.
A strict binding constructs its value once; it does not force the eventual Nix
value. The `throw` contract now describes failure at target evaluation.

Foreign opaque values identify the external type, rather than pretending it is
a Clause scalar or structural record:

```clause
Package:
  foreign: "nixpkgs"
  type: "Package"

foreign btop(): Package
  construction: "nix"
  get: "btop"
  from: "pkgs"
  failure: throw
```

An opaque foreign contract is admitted only inside a delayed type. A parameter
that independently requires a delayed Boolean spells `Delayed<nix,Bool>`.
Ordinary runtime foreign declarations continue to default to attempts and cannot
carry these delayed values. Native and JavaScript refinements reject construction
until they implement it. `compile-nix SOURCE.clause ENTRY OUTPUT.nix` constructs
one exported, argument-free entry using only checked common expressions; reached
foreign module roots become the Nix function's arguments. This bounded renderer
supports scalar constants, records, sequences, fields, strict bindings, checked
foreign construction, equality of ordinary values, concatenation, and static
conditional/require expressions. Other expressions reject explicitly.

A procedure body sequences its expression lines and returns the last value.
Each preceding expression evaluates exactly once before the next one. A wrapped
expression continues on more-indented lines:

```clause
procedure deliver(?outcome: DispatchError): F64
  write-output(2, ?outcome.message)
  ?outcome.status
```

The mode's foreign-access contract is derived from the same checked body,
including composed calls. An effectful direction cannot acquire a pure function
contract merely because it has no state delta or governed effect intent.

A law binds variables in premises before using them in conclusions:

```clause
law direct-dependency
  if
    ?consumer imports ?dependency
  then
    ?consumer depends on ?dependency

derive direct-dependency
```

`derive` separately authorizes the named law for computation. There is no
unnamed durable-rule form. Laws, derivation authorizations, invariants, goals,
and transition handlers remain distinct.

A handler condition can bind a closed finite numeric query:

```clause
sum ?amount where { ?item enabled true; ?item amount ?amount } as ?total
sum 1.0 given ?device where { ?device charge ?charge; ?charge > 0.0 } as ?count
```

Semicolons separate positive row patterns and scalar comparisons. A `given`
list explicitly imports correlated values; other query variables are local.
Every distinct complete row substitution contributes once, including equal
numeric values from different referents. An empty sum is zero. Matching and
arithmetic either complete or fail without publishing a prefix. Nested finite
queries are not canonical in this subset.

## Events and atomic changes

A handler reads one pre-state and proposes one atomic delta:

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

`when` reads. `withdraw` removes exact rows. `include` inserts exact rows.
`accumulate` adds independent numeric contributions under its declared row
contract. Source order does not resolve overlapping writes.

`create` binds one fresh Referent throughout a candidate:

```clause
on add-task ?prior ?chosen ?duration
  when
    ?prior duration ?old-duration
    ?prior = ?chosen
  create
    ?task
  include
    ?task duration ?duration
    ?task prerequisite ?prior
```

Inserted roles must determine one domain, and the complete candidate must
satisfy that domain's required properties. A nominal creation may instead add
explicit membership, which still cannot discharge structural obligations.

An external input binding gives a product control a typed Referent and handler:

```clause
bind referent-input Complete as Task to complete
```

The projected Referent identity reaches the handler unchanged. Equal-looking
referents remain distinct. The current implementation also has checked keyboard
and bounded scalar/Text input bindings; these are input boundaries, not
alternative relation declarations.

Keyboard bindings may supply finite numeric arguments in the handler’s declared order:

```clause
bind keyboard KeyD down to input with 1.0 0.0
```

The compiler rejects missing or extra arguments and non-finite values. Clauses
for the same externally supplied event, including a scheduled root event,
contribute to one atomic Step and read the same pre-state. Automatic reactions
remain separate scheduled Steps after the root event.

A reusable delta and a program-history candidate use explicit heads:

```clause
delta import-change
  withdraw
    North imports West
  include
    South imports North

revision adopt-impact from impact
  apply import-change
```

`include` stages content. The semantic Admission operation that commits an
authoritative successor has no child-block alias.

## Requests

Requests name their operation and exact constitution:

```clause
select all ?destination in egress
  where
    ICU-A has a usable egress path to ?destination

select one ?person in World
  where
    World relates ?person to C

select first ?person in World
  where
    World relates ?person to ?destination
  order by ?person

any in World
  where
    World relates ?_ to C
```

`select all` returns distinct projected rows while preserving their independent
supports. `select one` requires exactly one row. `select first` permits zero
rows and requires a declared total order. `any` tests existence. `?_` is a
fresh anonymous hole.

`why`, `prevent`, `achieve`, and `diff` retain explanation, intervention,
and comparison as separate operations. `find`, a bare `?`, naked-query
inference, storage-order selection, and unseeded random choice are not
canonical.

The current request grammar permits one recursive relational pattern in a
`where` block. General conjunction waits for an explicit semantic node; it is
not inferred from indentation.

## Prefix binders

Prefix binders precede every dependent use:

```clause
for n in 101..106
  Door-{n}
```

Ranges are inclusive ascending integers. Brackets remain structural sequence
terms, not range or template delimiters.

## Reader boundary

Reading proceeds in this order:

1. Normalize CRLF to LF for layout while retaining original byte spans.
2. Scan triple-quoted Text through its explicit closing margin.
3. Produce indentation tokens from exact multiples of two ASCII spaces;
   blank and comment-only lines do not affect layout.
4. Scan quoted values, longest punctuation, numbers, and maximal unquoted
   designations.
5. Select a line production from explicit head tokens.
6. If an indent follows, wrap that selected head in its declared block
   production; children cannot reclassify it.

The essential layout grammar below exposes shared-edge focus separately from
the generic block-head abstraction:

```text
SourceFile        ::= Trivia* TopLevelConstruct* EOF
TopLevelConstruct ::= SimpleConstruct NEWLINE
                    | SharedEdgeFocus
                    | OtherBlockHead NEWLINE INDENT ChildConstruct+ DEDENT
BindingHead       ::= Designation HSPACE* ":" HSPACE* ProductTerm
ProductTerm       ::= GroupedTerm ("," HSPACE* GroupedTerm)*
SubjectFocus      ::= Designation NEWLINE INDENT FocusedEdgeChild+ DEDENT
FocusedEdgeChild  ::= RelationEdge
                    | RelationPrefix NEWLINE INDENT FocusedEdgeChild+ DEDENT
SharedEdgeFocus   ::= SharedEdgeHead NEWLINE INDENT SubjectRow+ DEDENT
SharedEdgeHead    ::= "(" RelationEdge "):"
SubjectRow        ::= SubjectToken (HSPACE+ SubjectToken)* NEWLINE
SubjectToken      ::= Designation | "?" Designation
ReferentDeclaration ::= Designation
```

`OtherBlockHead` stands for the remaining declared block heads; each selects
its own `ChildConstruct` grammar. This is a layout outline, not the complete
set of source productions. A `SharedEdgeHead` selects `SubjectRow` children
both at top level and in the declaration or pattern contexts that admit
shared-edge focus. Its `RelationEdge` uses the selected declared edge grammar;
the head fixes the role and object while the children supply subjects.

Every subject row is exactly one indentation level below its head and has one
or more named or variable subject tokens. A quoted designation remains one
token. Subjects may share one row or occupy separate rows. Both layouts emit
one occurrence per subject token in source order, each with its own origin.
This row grammar belongs to shared-edge focus; it does not permit several
arbitrary sibling clauses on one line or nested children under a subject token.

`HSPACE` is one ASCII space; flexible positions accept zero or more.

Unquoted designations match `[A-Za-z_][A-Za-z0-9_-]*` maximally. Symbolic
infix operators require at least one ASCII space on each side, so `a-b` is one
designation and `a - b` is subtraction. `/` is forbidden inside every
Designation, quoted or not; Text and opaque payloads may contain it. Backticks
quote NFC-normalized multiword or Unicode designations without weakening that
rule.

Integers use `0` or an optional minus followed by a nonzero digit and digits.
Decimals add a dot and at least one fractional digit. Leading plus, leading
zeroes, omitted integer or fractional digits, separators, and exponent notation
are not canonical.

Single-line Text is UTF-8 and accepts `\"`, `\\`, `\n`, `\r`, `\t`,
and `\u{H}` through `\u{HHHHHH}` for Unicode scalar values. Unknown
escapes, surrogates, raw newlines, and unescaped controls reject.
Triple-quoted Text starts with `"""` as the final token on its line. A later
line containing only `"""` at least one two-space level deeper closes it; that
indentation is the content margin removed from every body line.

`#` begins a line comment outside Text or quoted designations. A contiguous
run of `##` lines at the following construct's indentation attaches
documentation to that construct; a blank or ordinary comment breaks the
attachment.

Canonical formatting emits LF, no trailing spaces, two spaces per indentation
level, no space before a colon, and one after it. Recovery resumes at the next
eligible sibling at or above the failed header's indentation; an error cannot
consume or reinterpret a later declaration.

## Unratified source

Effect and capability declarations, continuation and race forms, complete
package/module interchange, and dedicated scene syntax have no canonical
spelling. Their semantic experiments do not ratify source syntax. The roadmap
records when an implementation covers the canonical forms already defined
here.
