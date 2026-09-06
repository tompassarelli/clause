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
  ?example:
    x: F32
    y: F32
```

A bare designation introduces or resolves a Referent. An `enum` child emits
one independent membership fact. A `shape` child emits one field contract.
The selected head supplies those meanings before child designations are
resolved.

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

## Declared readings and modes

Relations that need non-field phrase structure declare one exact Reading and
any executable directions:

```clause
connects:
  ?example:
    door: Door
    origin: Space
    destination: Space
    ?door:
      connects ?origin to: ?destination

mode connects given door origin yields destination: many
```

Braces mark role binders; other words are literal phrase tokens. `subject`
names the role that focus may omit. A `mode` names known and produced roles
and the produced cardinality. The checked semantic form separately retains
RelationSchema, Reading, Operator, and Mode identities.

A relation block without `mode` declares only a schema and Reading. A schema
alone can check facts and patterns but cannot form an executable application.
An Operator may expose several Modes. Activation selects one Mode from the
exact eligible set established during formation.

Result cardinality is always explicit:

```clause
mode given thing yields value: one
mode given thing yields value: maybe
mode given thing yields value: some
mode given thing yields value: many
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
Vec2 { x: 3.0, y: 4.0 }
```

Declared readings may form nested expressions such as:

```clause
radius of coin + radius of player
length (position of player - position of coin)
```

`+ - * / < <= > >= = !=` have conventional precedence only where exact
declared relations support them. Multiplication and division bind tighter than
addition and subtraction, which bind tighter than order comparisons, which
bind tighter than equality. Arithmetic associates left; comparison and
equality do not chain. Parentheses resolve any remaining boundary.

`:` introduces denotation or a grammar-owned field. `=` is relational
equality. `->`, `:=`, `::`, and `~>` are not aliases.

The running scalar subset supports finite F64 expressions and composed laws.
It does not establish totality inference, a general constraint solver, or a
complete collection language.

## Laws and finite queries

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

Several zero-input `on` clauses with the same external event name contribute
to one atomic Step and read the same pre-state. Other handler arities and tick
scheduling do not acquire that grouping implicitly.

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

The essential layout grammar is:

```text
SourceFile        ::= Trivia* TopLevelConstruct* EOF
TopLevelConstruct ::= SimpleConstruct NEWLINE
                    | BlockHead NEWLINE INDENT ChildConstruct+ DEDENT
BindingHead       ::= Designation HSPACE* ":" HSPACE* ProductTerm
ProductTerm       ::= GroupedTerm ("," HSPACE* GroupedTerm)*
SubjectFocus      ::= Designation NEWLINE INDENT FocusedEdgeChild+ DEDENT
FocusedEdgeChild  ::= RelationEdge
                    | RelationPrefix NEWLINE INDENT FocusedEdgeChild+ DEDENT
ReferentDeclaration ::= Designation
```

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
