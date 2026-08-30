# Clause Architecture

> **Status:** Accepted host boundary plus a normative P1 compiler-genesis
> contract; CLCP v3 Lean replay is implemented while cross-host compiler
> acceptance remains pending.
>
> **Authority:** Derived and non-semantic. The
> [foundation](foundation.md) governs meaning, [syntax](syntax.md) governs
> canonical source, [canonical package](canonical-package.md) governs bytes,
> [compiler genesis](compiler-genesis.md) governs compiler authority and
> succession, and [roadmap](roadmap.md) governs implementation status.

## Decision

Clause uses one host-neutral semantic contract, one externally anchored
compiler root, and three implementation roles:

```text
 human owner selects exact literal Compiler0 bytes once
            and supplies an external anchor witness
                            |
                            v
                  accepted CLCP v3 Compiler0
                   /          |           \
                  v           v            v
        Clause-owned       Lean checks    Rust evaluates
        source/compiler    fixed rules    fixed machine
                  \           |            /
                   \--- exact predecessor -/
                            |
                            v
                  accepted compiler successor
```

Clause Core owns meaning. The accepted compiler package owns reading, syntax
selection, binding and occurrence identity, elaboration, type/mode/effect
checking, typed macros and transformations, origins, diagnostics, and compiler
evolution from the earliest literal bootstrap. Lean checks only the fixed
generic constitution. Rust executes only the fixed generic evaluator and
replaceable physical machinery.

Materialization, hashing, successful decoding, successful execution, and
derivability do not authorize `Compiler0`. One irreducible external human-owner
act selects its exact literal bytes and is presented to admission as
`Missing | Supplied(OwnerAnchorWitness)`. The witness is opaque to package data
and exposes the complete selected byte sequence for octet-for-octet comparison;
recorded length and package hash are secondary consistency observations, never
substitutes for those bytes or sources of authority. Every later compiler is
admitted only through the already accepted exact predecessor's `compile` and
`admitPropose` behavior, checked under that predecessor.

OCaml has no primary role. Aeneas is not part of the bootstrap or trust chain.
It may be reconsidered later for isolated safe-Rust verification only.

## Capability lifecycle

Implementation is continuous, but authority is promoted in distinct states:

1. An **experimental implementation or falsification artifact** may land with
   explicit non-authority, a bounded claim, deterministic tests for that claim,
   reversible scope, and no supported-language claim.
2. A **semantic candidate** maps its proposed meaning into host-neutral Clause
   Core. It remains a candidate and gains no authority from a Lean, Rust, or
   other host representation.
3. **Supported or admitted capability** passes every applicable constitutional
   proof, parity, negative, hidden-authority, optimization, and absence gate in
   this document before promotion or release.

The [foundation](foundation.md) remains the sole semantic authority, and the
[syntax](syntax.md) remains the canonical syntax authority. Prototypes may
falsify, exercise, or implement a bounded candidate, but they cannot invent
Clause meaning. Constitutional dependencies therefore block promotion,
admission, and release, never independent semantic, execution, runtime,
product, or evidence experiments and implementation. Semantic, execution, and
evidence workstreams proceed concurrently; only true dependency edges
serialize them.

## Live-tree boundary

The repository contains two implementation roots and one compiler contract:

```text
lean/                       constitutional checker/reference model
crates/clause-substrate/    physical persistence/runtime/backend substrate
docs/compiler-genesis.md    compiler identity and succession contract
```

The implementation roots began semantic-empty. They implement the narrower
CLCP v1 proof bootstrap plus the Lean CLCP v3 strict codec and complete replay
checker described in the [roadmap](roadmap.md). Rust v3 parity, `Compiler0`,
the genesis anchor, and accepted successor evidence remain pending.
New work derives only from the current Clause contract. Git history is
recovery, not an implementation input.

Every tracked source, test, example, document, generator, host, and release
script must describe only the current architecture. Superseded material leaves
no alias, shim, warning-only decoder, fixture, comment, generated consumer, or
gate that teaches it.

Every removed working capability requires a successor that passes deterministic
tests for each replaced behavior, regardless of current in-tree consumers.
Separately, every in-tree consumer migrates before removal. Once that migration
is complete, removal means absence from the live tree, including the superseded
source, tests, fixtures, generated artifacts, documentation, and consumers.

## Host-neutral Clause Core

The Clause Core contract is the transport and checking form of the calculus in
the foundation. CLCP v3 carries compiler execution through a fixed universal
kernel:

```text
RawTriple = [Term, Term, Term]
Term      = Atom | RawTriple

Γ ⊢ t clause : T @ M

Γ ; M ⊢ runρ(t) ↦ ⟨Γ̂, outcome, τ⟩

Γ ⊢ Γ̂ admissible
───────────────────
admit(Γ, Γ̂) = Γ′

KSort = Bytes | Term

KExpr =
  BytesLiteral | TermLiteral | Var | MakeAtom | MakeTriple |
  Let | CaseTerm | CaseBytes | ConcatBytes | CaseBytesEqual |
  Call | Request
```

Those `KExpr` cases are the complete host evaluator taxonomy. A token,
production, binder, type, mode, effect, macro, diagnostic, or compiler version
is package data and never a host expression case. `CaseBytes` exposes one
octet and a tail, `ConcatBytes` constructs dynamic byte strings, and
`CaseBytesEqual` supplies byte and hash comparison control. The package can
therefore read exact source and construct exact output without a host lexer or
string/equality callback.

CLCP v3 Frame 01 carries the complete exact `CoreManifestV1`, not a symbolic
ID resolved by a host. Its canonical bytes enumerate every Term, sort,
expression, Core ABI, authorization, static-rule, evaluation-rule, receipt,
and physical-profile tag and signature. The fixed prose semantics and closed
replay contract define fuel, environments, left-to-right evaluation,
observations, and every local rule. `CoreContractId` and `PhysicalProfileId`
are derived from those carried exact objects; there is no registry or
package-defined metalanguage.

The two distinct interface definitions have exact signatures
`compile : [Term] -> Term` and `admitPropose : [Term] -> Term`. Their fixed
Core ABI canonically encodes `BuildRequest`, `Built`, `Rejected`,
`AdmissionRequest`, `Propose`, `Reject`, observations, and the final
`Authorized` or `Unauthorized` result using only fixed tag, byte, identifier,
integer, list, and record forms. No host adapter may repair a signature or
shape mismatch.

Malformed wire input returns one separate `DecodeRejected(code, offset)` by
fixed cursor/code precedence and never reaches `Unauthorized`. After successful
decode, an explicit genesis or successor request selects the route, then
authorization visits the fixed stage table and encoded field order. Each
rejection predicate includes passage of every earlier condition, so the
predicates are pairwise disjoint and every failure returns exactly one
canonical `Unauthorized(stage, code)`. Genesis must bind its exact
`BuildRequest`, empty `GenesisEvidence`, explicit nonzero compile/admission
fuel inputs, and a final identity containing both complete exact package bytes
and their domain-separated package hash.
Entrypoint signature mismatch is only `(CoreWellFormedness,
EntrypointSignature)`. Successor evidence contains two trace-free receipts.
`VerifyEvalReceipt` independently constructs each exact request, completely
replays the manifest's `30..3e` rules, and compares canonical value and
observation commitments plus exact remaining fuel. Authorization separately
inspects the actual `Built` or `Propose` result and passes actual verified
compile observations into admission.

The package must carry every semantics-affecting object needed by a judgment:

- canonical Terms and explicit equality contracts;
- distinct identities where occurrence or continuity requires them;
- contexts, strata, judgments, relation schemas, modes, and capabilities;
- candidate successors, deltas, obligations, derivations, and certificates;
- source origins and separately scoped traces, strategies, and physical
  evidence; and
- a semantics epoch and one canonical byte representation.

The package is not a new ontology. Lean values, Rust structs, proof terms,
indexes, source maps, traces, caches, and strategies do not enter semantic
identity unless an authored Clause judgment explicitly makes their content
semantic. Lean proof terms remain local. Only Clause-native semantic evidence crosses the host-neutral boundary.

The implemented Lean and Rust CLCP v1 codecs derive from one Clause-owned
specification and vector corpus. CLCP v3 keeps the same independent,
strict-codec requirement while replacing the v1 carrier with the compiler
subject/evidence split in the [canonical-package contract](canonical-package.md).
No host serializer is a wire format.

## Lean constitutional kernel

Lean models the fixed byte decoder, `Term`, `KSort`, `KExpr`,
the exact carried core manifest, definition-table well-formedness, generic
evaluation rules and trace-free receipt replay,
exact-byte genesis selection, exact-predecessor succession, and the sealed
compiler physical profile. Clause features do not become Lean `Syntax`
kinds, `Expr` constructors, type classes, or one inductive constructor per
language form. Lean proves claims about Clause data; it does not parse Clause
source, define Clause's ontology, select a compiler, or invent feature meaning.

The reference Run semantics is relational and can represent total, bounded,
partial, nondeterministic, streaming, reactive, and effectful modes. Fuelled
interpreters may execute bounded specimens. Lean host termination never decides
Clause partiality or converts an open process into a false total function.

The constitutional checker is accepted only when all of these hold:

- the exact Lean source, toolchain, and imported artifacts are pinned and
  hashed;
- `trustLevel = 0` is used for new declarations without pretending it rechecks
  imports;
- every declaration in the transitive constitutional dependency closure is
  replayed into a fresh kernel environment and every reachable `unsafe` or
  `partial` declaration is rejected;
- the closure contains no `sorry`, `sorryAx`, skipped checking, elaboration
  recovery axiom, failed-declaration fallback, or preliminary asynchronous
  environment;
- acceptance does not rely on `native_decide`, native reduction, executed
  `implemented_by` or `extern` replacements, a compiled Boolean, or a foreign
  implementation;
- the transitive axiom closure is checked against an explicit policy, including
  deliberate decisions for `propext`, `Quot.sound`, and `Classical.choice`;
- every successor replay request is checker-constructed from the separately
  supplied already accepted predecessor, fixed core and physical profiles,
  entrypoint, canonical inputs, and fuel, while its compact receipt contains
  only canonical value and observation commitments plus exact remaining fuel;
  and
- `leanchecker` or equivalent replay is treated as a same-kernel consistency
  check, not an independent verifier.

No `unsafe`, `partial`, or `sorry` is permitted in the constitutional package.
Clause partiality and effects are object-language data and relations.

## Rust physical substrate

Rust may implement:

- strict canonical decoding/re-encoding and interning;
- the fixed construct-blind `Bytes`/`Term` evaluator, including generic byte
  destructuring, concatenation, equality, and fixed Core ABI validation;
- generic `DefId` table lookup, fuel, continuations, and checked hashing;
- indexes and incremental dependency maintenance;
- durable persistence and transaction machinery;
- operating-system, filesystem, network, browser, and foreign interfaces;
- runtime scheduling and resource accounting;
- native, Wasm, and JavaScript materialization; and
- profiling and target-specific physical strategies.

Rust may not parse Clause source or define what a Clause relation, production,
binder, type, mode, transition, capability, effect occurrence, macro,
diagnostic, identity, compiler revision, or admission means. It consumes an
accepted package and may create checked proposals or optimized views. A Rust
enum, trait, callback, plugin, formatter, validator, package-local `DefId`,
pointer, arena index, row number, or object layout is never semantic authority
or identity.

The substrate remains `unsafe`-free until an unavoidable foreign boundary is
identified and separately authorized. Any future unsafe module is isolated,
documented, tested, and outside the constitutional checker.

## Clause-authored compiler

Clause does not begin with a host frontend and migrate meaning later.
`Compiler0` owns lossless reading and syntax selection, binding and occurrence
identity, elaboration and schema/type/mode/effect checks, typed macros and
transformations, origin construction, diagnostics, and successor production
from genesis. Stable later capabilities—queries, impact analysis, refactoring,
planning, projection, and selected lowering—also evolve as Clause package
data.

The constitutional host-freeze test is an ordinary predecessor-authorized
`Compiler0 -> Compiler1` transition that changes one binding form, one effect
form, one typed macro, and one diagnostic behavior with zero Lean or Rust
source, toolchain, binary, or host-mechanics-manifest edits.

Host changes are allowed only for a genuinely new primitive physical
capability or a generically translation-validated optimization strategy.

## Machine-checkable host boundary

The trusted host may perform fixed generic mechanics:

```text
WireCodec | CoreABI | ByteMachine | DefinitionTable | KernelStep |
ReplayStep | PhysicalDispatch
```

Codec mechanics inspect bytes, tags, lengths, and bounds. The byte machine
implements empty/head-tail, concatenation, and equality. Generic `DefId`
lookup compares an opaque key and selects package `KExpr` data. Kernel steps
select child expressions by fixed `KExpr` tags and package-computed conditions.
Consequently token bytes and semantic IDs may change evaluated data and
package-program control. `PhysicalDispatch` recognizes only a fixed operation
and signature from the accepted profile.

No package value may select a host semantic implementation. Semantic IDs,
Atom fields, token bytes, production or diagnostic IDs, compiler revisions,
and package-local `DefId` values cannot choose a host lexer, grammar case,
binder, type/effect rule, macro expander, formatter, validator, trait method,
plugin, generated target case, native function, or specialized callback.

A source-AST and information-flow extractor enumerates every reachable branch
and indirect target, labels its fixed mechanic class and taint sources, and
proves that a package-influenced outcome is only canonical data, a fixed
error, a child `KExpr`, a selected package definition, or the one fixed
mechanic handler named by an enumerated wire, ABI, expression, replay-state, or
physical tag. For a given fixed tag and signature, the target is invariant
under all semantic IDs and raw payloads; package data cannot create a target or
select different host code for the same mechanic. Any unclassified site or
package-selected semantic callable rejects the host. The checked manifest
records the sites, classes, sources, tags, and targets.

The companion equivariance law uses an independent, domain-preserving
bijection only over explicit primitive/literal `Seed` and `RetainedSeed`
declaration identities. Their references, including `SeedInput`, follow the
resolved declaration image and are never mapped independently.
`NewId`-allocated declarations are never direct inputs to that bijection;
their sole image is recomputed from transformed allocation inputs. Fixed
core/physical IDs remain fixed; source/content IDs follow their exact
preimages; and origins, requests, semantics, revisions, packages, and
receipt and package hashes are recomputed from transformed preimages. The
transformation restores canonical ordering and updates all dependent
references before canonical re-encoding. If
`StrictDecode(P) = Decoded(P,D)`, `Dπ = Renameπ(D)`,
`Pπ = EncodeCanonical(Dπ)`, and `π*` includes those induced recomputations,
hosts satisfy:

```text
StrictDecode(Pπ) = Decoded(Pπ, Dπ)
EncodeCanonical(Dπ) = Pπ
VerifyEvalReceipt(π*(exactPredecessor), π*(request), π*(receipt))
  = VerifyEvalReceipt(exactPredecessor, request, receipt)
EvalHost(Pπ, π*(input)) = π*(EvalHost(P, input))
```

This law neither directly permutes hash octets nor transfers a genesis anchor
or acceptance judgment. Lean proves the generic laws and Rust exercises
canonical re-encoding, reordered tables, recomputed derived IDs, replay receipts, and outcomes through metamorphic
vectors.

## Execution and physical freedom

Pure Runs preserve the authoritative context. Authoritative Runs stage a
candidate successor; admission alone creates the successor revision. State
transition and external effects cross distinct boundaries: transition
admission authorizes effect intents, separately identified effect Runs perform
acts, and later evidence may record attempts, receipts, and observations.

The compiler may lower accepted meaning into registers, structs, arrays,
indexes, state machines, native instructions, Wasm, JavaScript, database
layouts, or browser objects. A physical decision that affects observable
behavior or a declared ABI, layout, overflow, floating-point, ordering,
determinism, synchronization, cancellation, durability, failure, resource, or
latency contract must remain an explicit strategy or evidence judgment.

Generic Triple interpretation is permitted as a bounded reference path, not an
ordinary production hot path.

## Admission and parity gates

Materializers, agents, optimizers, and target backends are untrusted
producers. After the one external genesis anchor, a small generic checker
admits a compiler successor only when the already accepted exact predecessor
both compiles its exact subject and proposes its admission. Candidate or
self-basis checking and hash-only predecessor equality reject.

A semantic tranche may be promoted or admitted as supported capability only
when:

1. its Clause Core representation is host-neutral and canonical;
2. Lean checks its certificate under the constitutional trust profile;
3. Rust agrees on every declared observable and nonfunctional contract;
4. negative fixtures fail for the intended reason;
5. the checked host-mechanics manifest has no package-selected semantic target,
   and structure-preserving nominal renaming is equivariant after canonical
   reordering and derived-ID recomputation;
6. no construct-specific host taxonomy or callback carries hidden meaning;
7. every optimized output is tied to a reference result, certificate, or
   translation-validation witness; and
8. tracked-tree absence checks find no superseded representation or authority.

The four-change compiler evolution and bounded
[adoption spike](adoption-spike.md) decide whether this mechanism is viable.
A pass authorizes promotion and admission of further capability; it does not
prove source ergonomics, large-graph incrementality, target performance,
replay tractability, or maintenance economics.
