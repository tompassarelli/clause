# Clause Architecture

> **Status:** Derived process-first architecture and host-boundary map.
>
> **Authority:** Derived and non-semantic. The
> [foundation](foundation.md) governs meaning, [syntax](syntax.md) governs
> canonical source, [canonical package](canonical-package.md) governs bytes,
> [compiler genesis](compiler-genesis.md) governs compiler authority and
> succession, and [roadmap](roadmap.md) governs implementation status.

## Decision

Clause uses one host-neutral semantic contract, one externally anchored
compiler root, and a governed boundary between compiler-machine checking and
Clause Admission:

```text
                         exact candidate CLCP bytes
                          /                    \
 owner anchor witness for genesis       exact accepted predecessor
                                               |
                                   compile + admitPropose
                          \                    /
                           v                  v
                        frozen CLCP checker verdict
                         Authorized | Unauthorized
                                    |
                 Clause-owned Run evidence + candidate delta
                                    |
                        governed outer Admission
                                    |
               authoritative compiler and Program successor
```

The foundation divides Clause meaning among a typed process constitution,
transition semantics, and Admission. The Clause Graph canonically carries the
checked constitution and declared process relations; transition semantics
governs Activations, Steps, observations, continuations, and Run order under
that constitution; and Admission alone establishes governed successors. Truth
status is extrinsic to graph representation, and neither graph storage nor
trace retention runs anything. The graph is the inspectable explanation
surface; the canonical carrier contract is its host-neutral transport and
checking form. The accepted compiler package
owns reading, syntax selection, binding and occurrence identity, elaboration,
type, mode, and effect checking, typed macros and transformations, origins,
diagnostics, and compiler
evolution from the earliest literal bootstrap. Lean checks only the fixed
generic constitution. Rust executes only the fixed generic evaluator and
replaceable physical machinery.

The semantic chain is:

```text
neutral Term + contextual ClauseJudgment / FormationJudgment
  -> checked closed ApplicationForm
     (exact RelationSchema, OperatorRef, complete named-role bindings,
      eligible Mode set, and context requirements)
  -- instantiate when nominal application continuity is required -->
     ApplicationId
  -> fresh ActivationId under one selected eligible ModeId, exact
     StaticActivationBasis with a checked-candidate or admitted-constitution
     binding, initial pins, exact named DynamicPrerequisiteBindings, a separate
     occurrence-only typed ActivationCauseFrontier, and exactly one Run
     membership
  -> stable ActivationId across configurations related by causal StepIds,
     each consuming the exact current affine configuration token, producing its
     successor token, citing one Mode-owned StepBoundaryRef, and carrying a
     finite typed StepCauseFrontier; every
     nonfirst Step has nonempty IncomingRunEdges from typed frontier and/or
     configuration-succession edges
  -> Run(RunId, one unique root and uniquely owned child Activations,
         per-Run RunOrder)
  -> observations, continuations, and candidate deltas
  -> heterogeneous CausalOrder across actual dependencies
  -> canonical AdmissionRequest and separately governed AdmissionOccurrence
```

Neither graph storage, mode existence, successful evaluation, nor a physical
artifact supplies dynamic authority or creates a successor revision. Every
Activation requires static formation/executability and one causal origin; a
Mode may declare an entirely empty dynamic-prerequisite schema. Static basis
and origin suffice without bindings only in that case. A checked-candidate
binding supports exact sandbox/compiler/test running, read-only use of an exact
admitted world, inert effect simulation, and physically persistent
nonauthoritative output or Continuations without fabricating a ProgramRevision
or authority. Joining an authoritative RuntimeSession, proposing authoritative
world change, relying on constitutive Program authority, or performing a real
external effect uses an admitted-constitution binding; merely reading a pinned
admitted world does not. Only that binding or a separately supplied
`IrreducibleRootConstitution` may support constitutive Authorization.

Materialization, hashing, successful decoding, successful execution, and
derivability do not authorize `Compiler0`. One irreducible external human-owner
act selects its exact literal bytes and is presented to admission as
`Missing | Supplied(OwnerAnchorWitness)`. The witness is opaque to package data
and exposes the complete selected byte sequence for octet-for-octet comparison;
recorded length and package hash are secondary consistency observations, never
substitutes for those bytes or sources of authority. Every later compiler can
become eligible for outer Admission only after the already accepted exact
predecessor's `compile` and `admitPropose` behavior and the frozen checker
produce the required exact evidence. Neither entrypoint nor the checker verdict
itself admits the successor.

OCaml has no primary role. Aeneas is not part of the bootstrap or trust chain.
It may be reconsidered later for isolated safe-Rust verification only.

## Capability lifecycle

Implementation is continuous, but authority is promoted in distinct states:

1. An **experimental implementation or falsification artifact** may land with
   explicit non-authority, a bounded claim, deterministic tests for that claim,
   reversible scope, and no supported-language claim.
2. A **semantic candidate** maps its proposed meaning into the host-neutral
   canonical carrier. It remains a candidate and gains no authority from a Lean, Rust, or
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

## Repository responsibility boundary

The repository currently carries its Lean implementation, historical Rust
bootstrap crate, and compiler contract at these literal paths:

```text
lean/                       constitutional checker/reference model
crates/clause-substrate/    historical combined Rust bootstrap crate
docs/compiler-genesis.md    compiler identity and succession contract
```

`clause-substrate` is only that historical crate and path name. It is not a
semantic concept or the target package boundary. The target Rust
responsibilities are `clause-package` for canonical CLCP bytes and validation,
`clause-runtime` for construct-blind evaluation and the generic process
protocol, `clause-materialization` for replaceable physical projections, and
`clause-wasm` for the bounded Wasm transport adapter. The target `clause` crate
is the user-facing facade and CLI aggregation boundary, not a shared semantic
implementation. The
[roadmap](roadmap.md) alone records when this split occurs and the status of
each implementation.

New work derives only from the accepted Clause contract. Git history is
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

## Host-neutral canonical carrier

The canonical carrier contract is the transport and checking form of the
calculus in the foundation. Clause-owned package data and process envelopes
carry semantic meaning. CLCP v3 separately carries compiler-machine execution
through a frozen construct-blind evaluator:

```text
Triple = [Term, Term, Term]
Term   = Atom | Triple

KSort = Bytes | Term

KExpr =
  BytesLiteral | TermLiteral | Var | MakeAtom | MakeTriple |
  Let | CaseTerm | CaseBytes | ConcatBytes | CaseBytesEqual |
  Call | Request
```

Those `KExpr` cases are the complete host evaluator taxonomy. A token,
production, binder, RelationSchema, operator, mode, ApplicationForm,
Activation, Step, effect, macro, diagnostic, or compiler version
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

The CLCP v3 exact bytes, hashed carried manifest, 73-byte `EvalReceipt`, left-
to-right KExpr evaluator, machine `Continuation`, evaluator step, out-of-fuel
behavior, `admitPropose`, and `CompilerRevisionId` are frozen compiler-machine
mechanics inside this exact contract. They are not `ApplicationId`,
`ActivationId`, semantic `StepId`, `RunId`, typed Continuation, general
Admission, or Program/State revision semantics. Clause-owned outer Terms and
envelopes carry those process identities, pins, authorizations, evidence, and
governed deltas through the fixed machine.

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

The package must carry every semantics-affecting object needed by formation and
process execution:

- canonical Terms and explicit equality contracts;
- distinct identities where occurrence or continuity requires them;
- contexts, strata, ClauseJudgments, FormationJudgments, RelationSchemas,
  revision-indexed extensions, OperatorRefs, modes, source Readings, static
  parameter and
  constraint telescopes, finite constraint bases with terminating resolution
  contracts, resolution-scope commitments, normalized evidence,
  InstantiationUseRefs, InstantiationKeys, SpecializationKeys,
  ApplicationForms, Applications, Judgments, JudgmentOccurrences, typed
  AuthorizationOccurrences, process contracts, and capabilities;
- candidate successors, deltas, obligations, derivations, and certificates;
- source origins and separately scoped traces, strategies, and physical
  evidence; and
- a semantics epoch and one canonical byte representation.

ProgramSnapshot construction is two-stage. A finite canonical checked preimage
uses local declaration identities and contains none of the
`ProgramSnapshotId`-scoped external references derived from itself. Hashing
that preimage creates `ProgramSnapshotId` once; exact `RelationSchemaId`,
`RoleId`, `OperatorRef`, `ModeId`, `ApplicationId`, and `JudgmentRef` values are
then resolved. Exact InstantiationUseRefs are derived from canonical local use
records; cross-snapshot InstantiationKeys and SpecializationKeys derive from
independent canonical interface/body/dependency content; and
`ApplicationShapeId` is derived from the resolved form with exact
InstantiationUseRefs, InstantiationKeys, and SpecializationKeys.
PhysicalReuseKey is excluded. None is inserted back into the same preimage.
PhysicalReuseKeys additionally bind the exact
`AcceptedRefinementWitnessId`, target/profile, ABI/layout/strategy, and
physical dependencies outside snapshot identity. Runtime Activations,
Steps, Runs, observations, traces, and physical layouts remain outside snapshot
identity unless governed semantic content explicitly makes them constitutive.

The package is not a new ontology. Lean values, Rust structs, proof terms,
indexes, source maps, traces, caches, and strategies do not enter semantic
identity unless an authored formation or governed Judgment explicitly makes
their content semantic. Lean proof terms remain local. Only Clause-native
semantic evidence crosses the host-neutral boundary.

CLCP v1 and v3 require independent strict codecs derived from one Clause-owned
specification and vector corpus. CLCP v3 replaces the v1 carrier with the
compiler subject/evidence split in the
[canonical-package contract](canonical-package.md). No host serializer is a
wire format.

## Lean constitutional kernel

Lean models the fixed byte decoder, `Term`, `KSort`, `KExpr`,
the exact carried core manifest, definition-table well-formedness, generic
evaluation rules and trace-free receipt replay,
exact-byte genesis selection, exact-predecessor succession, and the sealed
compiler physical profile. Clause features do not become Lean `Syntax`
kinds, `Expr` constructors, type classes, or one inductive constructor per
language form. Lean proves claims about Clause data; it does not parse Clause
source, define Clause's ontology, select a compiler, or invent feature meaning.

The reference process semantics is relational and can represent formation,
Activation, causal Steps, continuation, admission, and total, bounded, partial,
nondeterministic, streaming, reactive, and effectful modes. Fuelled interpreters
may execute bounded specimens. Lean host termination never decides Clause
partiality or converts an open process into a false total function.

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

## Rust physical implementations

Rust may implement:

- strict canonical decoding/re-encoding and interning;
- the fixed construct-blind `Bytes`/`Term` evaluator, including generic byte
  destructuring, concatenation, equality, and fixed Core ABI validation;
- generic `DefId` table lookup, fuel, machine continuations, and checked
  hashing;
- generic validation and transport of Clause-owned Application, Activation,
  Step, Run, continuation, observation, JudgmentOccurrence,
  AuthorizationOccurrence, and admission envelopes without interpreting their
  domain names;
- owned and region-member allocation roots, foreign-managed resource records,
  and separate Borrow/Lease access mechanics under accepted lifetime contracts,
  including bounded arenas and deterministic mechanical reclamation;
- indexes and incremental dependency maintenance;
- durable persistence and transaction machinery;
- operating-system, filesystem, network, browser, and foreign interfaces;
- runtime scheduling and resource accounting;
- Clause-declared physical IR materialization into native, Wasm, JavaScript,
  browser, data-system, and foreign ABIs; and
- profiling and target-specific physical strategies.

Rust may not parse Clause source or define what a RelationSchema, Application,
Activation, Step, Run, production, binder, static parameter, constraint,
evidence argument, type, lifetime, mode, transition, capability, effect
occurrence, macro, diagnostic, identity, compiler revision, or Admission means.
It consumes an
accepted package and may create checked proposals or optimized views. A Rust
enum, trait, callback, plugin, formatter, validator, package-local `DefId`,
pointer, arena index, row number, or object layout is never semantic authority
or identity.

Rust stays `unsafe`-free until an unavoidable foreign boundary is identified
and separately authorized. Any future unsafe module is isolated, documented,
tested, and outside the constitutional checker.

## Static abstraction, local state, and physical memory

The general-purpose path is not a generic graph interpreter with governance on
every mutation. Compiler-owned checking produces closed rank-1 instantiations,
normalized constraint evidence, affine ActivationConfiguration ownership,
Step cuts, one exact allocation root, and separate Borrow/Lease obligations before physical
lowering. The physical IR makes calls, loops, control/data flow, layouts,
borrows, moves, region resets, continuations, effect boundaries, and target ABI
decisions explicit while retaining an exact refinement link to the semantic
graph.

Ordinary local mutation remains inside an affinely owned
ActivationConfiguration and creates no StateRevision or Admission. A backend
may lower anonymous reductions to registers, stack slots, mutable arrays,
arenas, and in-place state-machine fields when non-escape, disjointness, failure
restoration, and Step-cut preservation are checked. Concurrent mutable work
uses ownership transfer, exact disjoint subconfiguration tokens, or explicit
access leases; split and join consume those tokens exactly once and never rely
on accidental host aliasing. Suspension likewise transfers the sole live
configuration token into its Continuation; reusable mutable takeup must fork
fresh child identities.

Parametric code may use monomorphization, normalized evidence dictionaries,
shared code, or complete erasure. Checking, semantic specialization, and
physical cache reuse use their distinct exact keys. Constitutive execution evidence may likewise
leave the hot ABI when the exact covered pins are frozen. Those are translation
strategies, not semantic choices. Issued Authorization, effects, capabilities,
and Admission evidence remain present at every boundary where they can vary.

The native/Wasm game profile has no mandatory tracing collector, implicit ARC,
or finalizer fallback. Compiler-generated ownership roots, borrows, leases,
region reset, and deterministic teardown reclaim physical storage when every
root-wide access, continuation, child, escape, close, and foreign obligation has
causally acknowledged quiescence. Release time may depend on runtime causality;
the compiler proves the rule rather than pretending every time is a static
constant. Strong cycles across independently reclaimed roots, including
Owned↔Owned, reject unless they use non-owning edges, one deterministic enclosing
Region, or one explicitly bounded `ManagedIsland` physical region. Managed
islands are never a default heap and are forbidden on the controlled native/
Wasm game hot path. Region reset and owner deallocation invoke no observable
destructor or finalizer; bounded compiler-proven nonobservable mechanical drop
is permitted, while observable close/dispose is an explicit process/effect
completed before reclaim. Browser and foreign
allocations retain explicit foreign-manager roots plus typed Lease edges and
require deterministic disposal even when the foreign runtime also has a GC.

Rich values and collections, parameterization, ownership, physical IR, layout
and ABI control, and native/Wasm specialization are early constitutional
falsifiers. They are not deferred polish and are not implemented merely because
this architecture names them.

## Clause-authored compiler

Clause does not begin with a host frontend and migrate meaning later.
`Compiler0` owns lossless reading and syntax selection, binding and occurrence
identity, elaboration and formation/schema/operator/mode/effect checks, typed
macros and transformations, rank-1 parameter and constraint checking,
normalized evidence, local ownership and lifetime checking, Step-cut analysis,
physical-IR construction, origin construction, diagnostics, process-envelope
construction, and successor production from genesis. Its semantic output uses
RelationSchemas, operators, modes, ApplicationForms, nominal Applications,
typed process envelopes, Judgments, and JudgmentOccurrences rather than host-
owned construct cases. Stable later capabilities—queries, impact analysis,
refactoring, planning, projection, and selected lowering—also evolve as Clause
package data.

The constitutional host-freeze test is an ordinary predecessor-authorized
`Compiler0 -> Compiler1` transition that changes one binding form, one effect
form, one typed macro, and one diagnostic behavior with zero Lean or Rust
source, toolchain, binary, or host-mechanics-manifest edits.

The same frozen hosts must execute a Clause-defined algebraic data declaration,
constructors, patterns, and exhaustive match, while rejecting missing and
unreachable cases with exact obligations. These are user-defined Clause data
and process definitions, not new Term, KExpr, Lean, or Rust kernel cases.

Host changes are allowed only for a genuinely new primitive physical
capability or a generically translation-validated optimization strategy.

### Agent-native workbench boundary

The first developer loop is one long-lived stdio service, not a collection of
host scripts or a second interpreter. The service accepts exact requests for
`parse`, `check`, `explain`, `query`, `diff`, `propose`, `admit`, `run`, and
`hotReload`. Those operations are accepted Clause package definitions executed
through the fixed CLCP03 evaluator and the generic Clause runtime. Rust owns
bounded transport, exact revision/session pins, replaceable caches,
transactional request framing, and scheduling only; it does not parse source,
solve Clause constraints, answer semantic queries, or invent diagnostics.

Interactive checking may use accepted incremental summaries and need not replay
the full Lean or compiler-succession proof on every edit. Exact replay remains
the promotion and Admission gate. A source edit is proposed transactionally
against one exact base, checked, and either retained as a non-authoritative
candidate or separately admitted; `hotReload` never silently migrates live
Activations. Human-readable source remains a compact audit and token surface,
while machine responses use stable typed diagnostics, exact dependency and
causal slices, and immutable pins.

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
canonical re-encoding, reordered tables, recomputed derived IDs, replay
receipts, and outcomes through metamorphic vectors.

## Execution and physical freedom

Pure running returns values and evidence without creating an authoritative
revision. Transition Steps may stage candidate deltas; Admission alone creates
the successor revision. State transition and external effects cross distinct
boundaries. Each effect Mode selects an exact profile. Every real-effect
Activation binds independent dynamic slots for the exact intent occurrence,
issued EffectAuthorization occurrence, and CapabilityEvidence, projecting only
their occurrence-backed components before an attempt. `GovernedPerIntent`
additionally binds the intent's exact AdmissionOccurrence.
`PreauthorizedEffect` instead binds a previously issued bounded activation,
session, Lease, or batch scope; it may cover several attempts and creates no
per-attempt Admission or issuance. Constitutive execution authority never
replaces the issued effect-authorization slot. A statically pinned slot may
erase from a checked hot ABI but remains in the exact semantic explanation.
Intent, issued authority, capability, attempt, optional receipt, observations,
governed JudgmentOccurrences, and any later Admission remain distinct in every
profile. Candidate delta and continuation
never collapse.

Admission-free local running advances one affinely owned
ActivationConfiguration through anonymous internal reductions and declared
Step cuts. It may drive a loop, builder, request-local cache, simulation, actor
state machine, or render projection without a StateRevision. An immutable frame
from that path is keyed by exact Run, Activation, producing Step, and
Observation identities plus optional unchanged `Wbase`. An admitted
StateRevision pin is required only when the frame actually projects that
boundary.

Semantic State admission supplies an `AdmittedStateDelta` naming the exact
ProgramRevision, RuntimeSession, predecessor and result StateRevisions, and
producing ActivationId and StepId. A separate physical update envelope adds the
exact semantic graph/contract reference, materialization plan reference, and
physical budget. A materializer may validate, project, and apply that envelope
atomically to a replaceable view; it never allocates or admits State history.
Plan identity belongs only to the physical envelope and receipt and never enters
`StateRevisionId`.

The compiler may lower accepted meaning into registers, structs, arrays,
indexes, state machines, native instructions, Wasm, JavaScript, database
layouts, or browser objects. A physical decision that affects observable
behavior or a declared ABI, layout, overflow, floating-point, ordering,
determinism, synchronization, cancellation, durability, failure, resource, or
latency contract must remain an explicit strategy or evidence judgment.

Activation requires the foundation's checked `OpenSystemRefinementV1`, with
exact semantic and physical pins, state/input/output relations, tau/event
projection, typed nominal isomorphism, Step/effect/Admission linearization,
resource/latency preorder, and Mode-appropriate result/progress/fairness/bound
obligations. Checked lowering chains compose only through that judgment and
their exact accepted witness identity enters `PhysicalReuseKey`. CPP1's
external physical-plan allocation and removal of magic semantic-Term lookup
are already-correct implementation facts, but its current
`ClosedApplicationRuleMachineV1` tag and shape/Mode/role/byte checks are not a
refinement certificate.

Semantic identity uses only the foundation's canonical
`ClauseSemanticsManifestV1`: exact manifest bytes derive `ClauseSemanticsId`
and bind the foundation, syntax, architecture, compiler genesis, selected
carrier/checker contracts, and required corpus roots. Mutable checker/runtime
implementation hashes, bounded capability labels, Git commits/tags, release
packaging, signing, deployment, and inventory metadata remain outside the
preimage. No non-semantic artifact manifest is created without one actual
consumer whose next action depends on it.

Generic Triple interpretation is permitted as a bounded reference path, not an
ordinary production hot path.

For the native/Wasm frame profile, initialization preallocates bounded
transport buffers, Wasm memory, local regions, renderer pools, active-frontier,
continuation, and trace capacity. Partial initialization publishes no handle or
view and rolls back Clause-controlled state; every attempted foreign allocation
records cleanup success, failure, or pending quarantine without claiming atomic
external cleanup.
Frame execution then performs no Clause/Wasm/adapter-controlled allocation,
`memory.grow`, whole-carrier clone, global scan, observable destructor/finalizer, or
unbounded destruction. Ratified foreign calls retain explicit allocation and
disposal contracts; a browser-wide zero-allocation claim requires instrumented
target evidence including warm-up and lazy caches. Typed failures leave the
last visible projection and caller-owned inputs unchanged.

Every long-lived runtime selects an explicit trace-retention contract with
bounded resident configuration, active-frontier, continuation, diagnostic, and
trace capacity. Exact externalization or compaction may reduce residency but
cannot authorize new causal edges; an evicted cause must be rehydrated with its
checked witness within budget or the dependent Step rejects typed.

## Admission and parity gates

Materializers, agents, optimizers, and target backends are untrusted
producers. After the one external genesis anchor, the already accepted exact
predecessor must both compile the candidate's exact subject and run its frozen
`admitPropose` proposal check. The small generic checker returns exact
`Authorized` or `Unauthorized` evidence over those compiler-machine results;
it does not admit the candidate. Governed outer Admission alone may consume the
verdict, Clause-owned Run evidence, authority, candidate delta, and obligations
to establish the authoritative compiler and Program successor. Candidate or
self-basis checking and hash-only predecessor equality reject.

A semantic tranche may be promoted or admitted as supported capability only
when:

1. its canonical carrier representation is host-neutral and canonical;
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
A pass supplies evidence and satisfies the named gate; only the governing
policy and authority may authorize promotion or admission. It does not prove
source ergonomics, large-graph incrementality, target performance, replay
tractability, or maintenance economics.
