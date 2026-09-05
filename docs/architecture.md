# Clause architecture

> **Authority:** This document defines implementation boundaries, not language
> meaning or status. The [foundation](foundation.md) defines meaning,
> [syntax](syntax.md) defines source, [canonical package](canonical-package.md)
> defines compiler-machine bytes, [compiler genesis](compiler-genesis.md)
> defines compiler succession, and the [roadmap](roadmap.md) reports what runs.

## Boundaries

Clause separates meaning, running, and authority:

```text
source
  -> lossless concrete syntax
  -> checked semantic graph
  -> Application + selected Mode
  -> Activation / Step / Run
  -> observations, continuations, candidate deltas
  -> separately authorized Admission
  -> Program or State successor
```

The semantic graph records every distinction that can affect checked meaning.
It does not run by being stored. A trace describes a Run but cannot replay its
effects. A successful evaluator result does not admit a successor.

The implementation has four responsibilities:

- The **Clause-authored compiler** owns reading, declared grammar and Reading
  selection, binding and occurrence identity, elaboration, contracts, types,
  Modes, effects, transformations, origins, diagnostics, and lowering.
- The **canonical carrier** transports exact Clause objects and evidence
  between implementations without adding meaning or authority.
- The **generic runtime** executes accepted process data, schedules physical
  work, accounts for resources, and exposes operating-system and foreign
  boundaries without recognizing domain constructs.
- **Physical adapters** project exact observations and accept exact inputs.
  They do not own application rules or mutate authoritative state directly.

The Clause-authored compiler and full canonical process carrier are not
implemented. Existing Rust, native, Wasm, and browser paths are bounded
experiments; their exact coverage is in the roadmap.

## Compiler authority

Clause does not begin with a host frontend whose cases quietly become the
language. `Compiler0` must read source, produce the checked graph, and emit
physical plans as accepted Clause data. Later compilers succeed it through the
same exact process rather than by replacing an unrecorded host implementation.

Genesis requires one irreducible owner-selected byte sequence. Its witness is
external to package data and exposes the complete bytes for exact comparison;
a length or digest cannot replace that comparison. Every successor must be
compiled and proposed by the exact accepted predecessor, checked by the frozen
generic machine, then admitted by a separate authorized outer decision. A
package, hash, successful replay, or compiler candidate cannot select itself.

The decisive host-freeze exercise changes one binding form, one effect form,
one typed macro, and one diagnostic behavior through a
`Compiler0 -> Compiler1` transition with no Lean or Rust semantic edits.
Frozen hosts must also execute Clause-defined algebraic data and exhaustive
matching while rejecting missing and unreachable cases. A new physical
primitive may require a host change; a new Clause construct may not.

## Canonical carrier

The host-neutral carrier represents Terms, schemas, Readings, Operators, Modes,
ApplicationForms, Applications, identities, static parameters and evidence,
process contracts, occurrences, Judgments, authorizations, capabilities,
deltas, obligations, certificates, source origins, and declared physical
evidence. There are no host-only semantic fields.

A package envelope is transport, not a ProgramSnapshot or authority.
ProgramSnapshot identity derives once from a checked local-reference semantic
preimage. Runtime Activations, traces, source maps, caches, layouts, strategies,
and artifacts remain outside that identity unless explicit Clause meaning
makes them constitutive.

CLCP v3 is the frozen compiler-machine contract, not the general process
ontology. Its host evaluator understands only:

```text
Term  := Atom | [Term, Term, Term]
Sort  := Bytes | Term

Expr  := BytesLiteral | TermLiteral | Var | MakeAtom | MakeTriple
       | Let | CaseTerm | CaseBytes | ConcatBytes | CaseBytesEqual
       | Call | Request
```

Tokens, productions, binders, relations, Modes, Steps, effects, diagnostics,
and compiler versions remain package data. `CaseBytes`, concatenation, and
byte equality are sufficient for a package-owned reader without a host lexer
or callback.

The carried `CoreManifestV1` fixes every machine tag, signature, rule, fuel
transition, environment behavior, and observation. The two interfaces are
exactly:

```text
compile      : [Term] -> Term
admitPropose : [Term] -> Term
```

Their ABI encodes build, rejection, proposal, observation, and
`Authorized | Unauthorized` results. Malformed bytes produce one
`DecodeRejected(code, offset)` before authorization. Successfully decoded
requests visit the fixed authorization stages in order and produce one
canonical rejection. A signature or shape mismatch is never repaired by an
adapter.

Successor evidence contains two trace-free 73-byte evaluation receipts. Receipt
verification reconstructs the exact request, completely replays the frozen
rules, and compares result, observation commitment, and remaining fuel.
Diagnostic traces carry no authority.

CLCP v1 and v3 use strict independent codecs against one byte contract and
vector corpus. The current `CLPV` process bytes are a Rust-owned experiment,
not a canonical Clause format.

## Constitutional kernel

Lean models the fixed byte decoder, neutral Term and expression machine,
manifest, definition-table well-formedness, evaluation rules, receipt replay,
genesis selection, predecessor succession, and sealed physical profile. Lean
does not parse Clause source, define Clause constructs, select a compiler, or
grant Admission.

The constitutional package permits no `unsafe`, `partial`, `sorry`,
`sorryAx`, native replacement, or skipped declaration. Its transitive
dependency closure is replayed into a fresh kernel environment at
`trustLevel = 0`; the axiom closure is checked against an explicit policy.
Object-language partiality and effects remain data rather than Lean
nontermination. Same-kernel replay is consistency evidence, not an independent
verifier.

## Generic Rust and foreign boundaries

Rust may implement strict codecs, the fixed Bytes/Term evaluator, definition
lookup, fuel and machine continuation, hashing, generic process-envelope
validation, indexes, transactions, persistence, scheduling, resource
accounting, operating-system interfaces, foreign interfaces, and validated
physical plans.

Rust may not parse Clause source or define what a relation, Reading, binder,
Mode, type, law, Step, effect, diagnostic, identity, or Admission means. Rust
enums, traits, callbacks, plugins, `DefId` values, pointers, rows, and layouts
are never semantic authority. Rust remains safe unless one exact irreducible
foreign boundary requires an isolated and separately checked unsafe module.

The host dispatch surface is fixed to codecs, the Core ABI, the byte machine,
definition lookup, kernel/replay steps, and enumerated physical operations.
Package data may select child expressions and package definitions; it may not
select a host semantic implementation. A host-mechanics extraction must reject
every unclassified branch, indirect target, or package-selected callback.

Domain-preserving renaming of explicit seed identities, followed by canonical
reordering and recomputation of derived IDs, must commute with decoding,
encoding, receipt verification, and execution. This detects semantic behavior
hidden in names, table order, or host dispatch.

## Physical realization

The compiler lowers accepted meaning to a physical IR with explicit calls,
control and data flow, layouts, ownership, regions, borrows, Leases,
continuations, effect boundaries, and target ABI decisions. It may then use
registers, structs, arrays, indexes, state machines, native instructions, Wasm,
JavaScript, browser objects, databases, or foreign resources.

Ordinary mutable work stays inside one affinely owned Activation configuration
and needs no StateRevision. A lowering may update it in place only when
non-escape, disjoint access, restoration on failure, and every semantic Step
cut are checked. Parallel mutation consumes an exact split into disjoint
branch custody and rejoins exact settlements; host interleaving provides no
semantic order.

Every allocation has one owner, region, or foreign manager. Borrow and Lease
edges control access without becoming ownership roots. Observable close and
disposal remain explicit processes. The native/Wasm game path cannot rely on a
mandatory tracing collector, implicit reference counting, finalizers, or
unbounded teardown.

Static proof and ownership structure may leave a hot ABI when specialization
fixes it. Dynamic authorizations, capabilities, Leases, configuration custody,
effect inputs, and Admission evidence remain present at every boundary where
they can vary. Checking reuse, semantic specialization, and physical artifact
reuse keep separate identities.

Every activated physical plan needs a versioned refinement witness binding the
exact semantics, snapshot, Application shape, Mode, plan bytes, target,
runtime, compiler, ABI, state/input/output relations, occurrence-identity
mapping, Step/effect/Admission linearization, and applicable result, progress,
fairness, cancellation, failure, resource, and latency obligations. A changed
pin, plan, relation, or bound invalidates physical reuse.

The current CPP1 checker validates physical-plan encoding, shape, Mode, role
bindings, and exact bytes. It does not yet prove transition adequacy, causal
linearization, progress, resource/latency obligations, or composition of
lowerings.

## Runtime, effects, and materialization

Pure running returns values and observations without a revision. Transition
Steps stage candidate deltas; Admission alone creates State history. External
effect execution keeps intent, issued effect Authorization, capability,
attempt, optional receipt, observations, Judgments, and later Admission
separate.

A materializer consumes an already admitted semantic delta plus exact graph,
contract, plan, and budget references. It may atomically update a replaceable
physical view and return a receipt. It cannot allocate State history, admit a
delta, or insert its plan identity into StateRevision identity.

The native/Wasm bounded frame profile preallocates Clause-controlled memory,
transport buffers, renderer pools, active-frontier, continuation, and trace
capacity. After initialization its controlled path performs no allocation,
`memory.grow`, whole-state clone, global scan, observable finalizer, or
unbounded destruction. Foreign calls retain explicit allocation and disposal
contracts; a browser-wide zero-allocation claim requires target measurement.

A typed frontend maps physical inputs to exact immutable observations and maps
render observations to foreign calls. It owns no movement, scheduling,
collision, combat, admission, or application state. A frame pins the Run,
Activation, producing Step, and Observation; it adds StateRevision only when it
projects an admitted boundary.

## Developer workbench

The intended workbench is one long-lived request service for `parse`, `check`,
`explain`, `query`, `diff`, `propose`, `admit`, `run`, and `hotReload`.
Accepted Clause package definitions implement those operations. Rust owns
bounded transport, exact pins, caches, transactional framing, and scheduling;
it does not answer semantic questions or invent diagnostics.

Interactive checking may use accepted incremental summaries. Exact Lean and
compiler-succession replay gates promotion rather than each edit. Hot reload is
an exact base-pinned proposal; it preserves only checked continuity and never
silently migrates a live Activation.

The current `clause-workbench` is a resident bounded source compiler/runtime
for the implemented authoring subset. Its Rust source reader is not the
Clause-authored workbench described here.

## Repository responsibilities

Current paths and intended responsibilities are:

- `lean/`: constitutional reference checker;
- `crates/clause-substrate/`: historical CLCP bootstrap;
- `crates/clause-package/`: canonical package and current source-carrier
  experiments;
- `crates/clause-runtime/`: generic process execution;
- `crates/clause-materialization/`: replaceable physical projections; and
- a future `clause-wasm`: bounded process transport.

The user-facing `clause` crate may aggregate the CLI but owns no duplicate
semantics. Workspace membership and crate names confer no semantic or supported
status. Superseded implementations leave the live tree only after every
in-tree consumer has migrated to a tested successor.

## Admission gates

A capability becomes supported only when its immediate semantic and physical
claim has one host-neutral representation, one constitutional check, matching
Rust behavior, exact negative failures, no package-selected host semantics,
and a refinement link for every optimized result. Admission remains a separate
authorized decision over that evidence.

The host-freeze and executable applications decide whether this architecture is
viable. Passing them does not by itself prove source ergonomics, incremental
performance, target behavior, or maintenance cost; those claims require their
own application evidence recorded in the roadmap.
