# Clause Compiler Genesis and Succession

> **Status:** Normative P1 contract; implementation and acceptance evidence are
> pending.
>
> **Authority:** This document owns compiler identity, genesis authority,
> succession, and the Clause/Lean/Rust boundary. The
> [canonical-package contract](canonical-package.md) owns CLCP bytes, the
> [foundation](foundation.md) owns Clause meaning, [syntax](syntax.md) owns the
> human-readable source design, and the [roadmap](roadmap.md) alone records
> implementation status.

This is a constitutional contract, not implementation evidence. The live tree
does not yet contain a CLCP v2 package, a literal `Compiler0`, a genesis
anchor, a universal evaluator, or the certificates described here. The
existing CLCP v1 corpus is narrower bootstrap evidence and does not satisfy
this contract.

## Decision

Clause begins with one literal canonical package, `Compiler0`. One
irreducible external owner act selects its exact bytes at an exact release
object. Nothing inside those bytes can perform that act:

- a materializer may produce candidate bytes but cannot authorize them;
- a hash may identify or retrieve bytes but cannot authorize them;
- canonical decoding and core well-formedness do not authorize them;
- a derivation or evaluation certificate does not authorize them; and
- genesis evidence inside the package is empty and cannot select its carrier.

After that one anchor, compiler authority moves only by exact predecessor
succession. The already accepted predecessor compiles a candidate subject and
executes its own `admitPropose` behavior over that exact subject. Lean checks
those generic evaluations. The candidate, its evidence, its hash, and any
basis it contains are never an admission basis.

The literal package owns executable compiler meaning from genesis. Clause owns
source reading and syntax selection, binding and occurrence identity,
elaboration, schema/type/mode/effect checking, typed macros and
transformations, origins, diagnostics, and compiler evolution. Rust executes
the fixed generic machine. Lean checks the fixed generic constitution. Neither
host silently owns a Clause construct or compiler revision.

No Beagle, JavaScript, TypeScript, Bun, Racket, Rhombus, or other third
language is a bootstrap semantic authority. A tool in any language may
materialize or inspect bytes as non-authoritative evidence, but the accepted
package and predecessor chain remain the only compiler authority.

## P1 boundary

P1 freezes:

- the strict CLCP v2 subject/evidence split and domain-separated identities;
- `Term = Atom | Triple`;
- the exact carried Core manifest, closed static/evaluation/certificate rule
  tables, one-operation physical profile, and deterministic verdict precedence;
- a fixed construct-blind universal evaluator over `Bytes` and `Term`;
- the exact Core ABI, `[Term] -> Term` `compile` and `admitPropose`
  entrypoints, canonical requests/results/observations, and certificate
  statements;
- literal external genesis selection and exact predecessor-only succession;
- lossless source, origin, occurrence, binding, elaboration, transformation,
  and diagnostic invariants;
- deterministic build inputs and later self-rebuild equations; and
- a machine-checkable generic-mechanics versus semantic-dispatch rule and
  structure-preserving nominal-renaming equivariance law.

P1 does not freeze ordinary Clause construct identifiers, grammar
productions, effect syntax, macro libraries, diagnostic wording, optimized IR,
storage layout, scheduling, backends, FFI repertoires, proof compression,
incremental algorithms, or performance claims. Adding an ordinary Clause
construct changes package data. Adding a genuinely new physical primitive is a
substrate change and must cross a separate physical-profile gate.

## Fixed universal evaluator

The kernel has two sorts:

```text
KSort = Bytes | Term

Term =
    Atom(kind:Bytes, canonicalPayload:Bytes, equalityContract:Bytes)
  | Triple(first:Term, second:Term, third:Term)
```

Triples have structural identity only. Nominal and occurrence identities are
explicit Atom values or explicit package fields; host pointers, table indexes,
source positions, and spellings never become identity.

The evaluator recognizes exactly these expressions:

```text
KExpr =
    BytesLiteral(bytes)
  | TermLiteral(term)
  | Var(deBruijnIndex)
  | MakeAtom(kind, payload, equality)
  | MakeTriple(first, second, third)
  | Let(value, body)
  | CaseTerm(scrutinee, atomBody, tripleBody)
  | CaseBytes(scrutinee, emptyBody, consBody)
  | ConcatBytes(parts)
  | CaseBytesEqual(left, right, equalBody, unequalBody)
  | Call(definitionId, arguments)
  | Request(physicalOperationId, arguments)
```

`CaseTerm` binds the three Atom byte fields or the three Triple children.
`CaseBytes` binds the leading octet as one-byte `Bytes` and the remaining
bytes. `ConcatBytes` concatenates byte-valued parts. `CaseBytesEqual` compares
two complete byte strings and selects one of two package expressions. Thus a
package can identify each possible head octet by comparison with literals,
scan tokens and delimiters, assemble dynamic bytes and canonical length
prefixes, compare identifiers or computed hashes, and recursively compare
Terms without a host string, lexer, equality, or collection callback.
Evaluation is call-by-value, left-to-right, with explicit fuel and named
recursive definitions. Definitions are data addressed through opaque `DefId`
lookup. No token, lexer, binder, type, effect, macro, diagnostic, or
compiler-version tag is a kernel tag.

Compiler admission uses a sealed pure physical profile. During `compile` and
`admitPropose`, the only permitted physical request is deterministic
`Request(Sha256OpId, [Bytes]) -> Bytes(32 octets)`. Clock, randomness, locale,
environment, ambient filesystem paths,
filesystem enumeration, cache state, scheduling observations, network, FFI,
target execution, and undeclared operations reject.

The evaluator is intentionally tiny. Literal bytes, byte destructuring,
concatenation and equality, recursive definitions, universal Terms, generic
definition lookup, and SHA-256 make it operationally adequate to express a
reader, graph algorithms, canonical encoding, diagnostics, and successor
construction. Its time, space, certificate size, and optimization tractability
for a real compiler remain unproved P2 questions; that uncertainty cannot be
resolved by adding a construct-specific host branch.

## Compiler subject

A compiler subject contains:

```text
CompilerSubject = {
  lineage:
      Genesis
    | Successor {
        predecessorLocator: CompilerPackageHash,
        changeOccurrenceId: ChangeOccurrenceId
      },
  nominalDeclarations: Seq<NominalDeclaration>,
  interface: {
    compile: DefId,
    admitPropose: DefId
  },
  program: Seq<Definition>,
  buildRequest: Term
}
```

`interface || program` is the authoritative executable compiler. The
`buildRequest` is the exact canonical Core ABI Term defined by the
[package contract](canonical-package.md#clcp-v2-fixed-compiler-abi). It carries
the base locator, core and physical profiles, target profile, exact source
units, base inputs, canonical identity plan, change occurrence, options,
compile/admission fuel, and every declared physical input needed to reproduce
the subject. Embedded source is
not a second authority: if source and executable subject disagree, the
accepted executable subject governs, and exact rebuild failure exposes the
disagreement.

Rust optimizations, caches, indexes, generated targets, and machine layouts are
not package meaning. They require generic translation validation against the
accepted package program.

## Clause-owned source and compiler behavior

`Compiler0` owns source interpretation from the first literal bootstrap. A
conforming implementation represents at least these invariants as package
data, opaque to Lean and Rust.

### Lossless source and origins

```text
LosslessCST = {
  sourceUnitId,
  root: CSTOccurrenceId,
  nodes: Seq<CSTNode>,
  tokens: Seq<TokenOccurrence>,
  recovery: Seq<RecoveryOccurrence>
}

CSTNode = {
  occurrenceId,
  productionId: ReferentId,
  children: Seq<CSTOccurrenceId>,
  origin: OriginId
}
```

Every source byte belongs to exactly one ordered token, trivia, or recovery
leaf. Concatenating leaf slices reproduces the exact input bytes. Missing and
recovery nodes may describe absence, but cannot silently consume, synthesize,
or reinterpret later bytes.

```text
OriginNode =
    Source(sourceUnitId, halfOpenSpan)
  | Derived(transformOccurrenceId, inputOrigins, outputSlot)
  | Synthetic(producerOccurrenceId, reason, relatedOrigins)
```

Origin graphs are finite DAGs. A transformation retains its call origin,
transformer occurrence, ordered input origins, phase, and output slot.

### Binding, elaboration, and transformations

```text
Scope  = { scopeId, parentScopeId?, phase, origin }
Binder = { binderId, scopeId, declarationOccurrenceId, origin, contract }
Use    = { useOccurrenceId, scopeId, phase, spelling, origin,
           resolution: BinderId | exactObligation }
```

Uses resolve to `BinderId`, never to spelling or position. Macro-introduced
scopes, binders, and occurrences receive fresh deterministic IDs. Deliberate
capture is explicit; accidental capture, phase escape, or textual-renaming
hygiene failure rejects.

`Compiler0` chooses readings; constructs the CST, origin, scope, binder, and
use graphs; elaborates Terms and judgments; checks schemas, types, modes,
effects, capabilities, and obligations; executes typed transformations; and
chooses the accepted or rejected compiler result. Those decisions do not move
into Lean or Rust merely because a host stores or evaluates them.

### Diagnostics

```text
Diagnostic = {
  occurrenceId: DiagnosticOccurrenceId,
  code: ReferentId,
  severity: Term,
  primary: OriginId,
  related: Seq<(Term, OriginId)>,
  causes: Seq<ObligationId>,
  document: Term,
  fixes: Seq<Term>
}
```

Diagnostic documents and fixes are Clause data produced by compiler
definitions, not Rust formatter cases or Lean constructors. Canonical ordering
is by primary source artifact, start, end, code identifier, then diagnostic
occurrence identifier.

## Identity allocation and retention

Nominal identity provenance is explicit in the subject's canonical
`nominalDeclarations` table. A successor allocates a fresh ID only from
explicit predecessor-visible inputs:

```text
NewId(domainId, changeInput, producerInput, localSlot) =
  DH("clause/new-nominal/v1",
     domainId,
     canonicalNominalWireRef(changeInput),
     canonicalNominalWireRef(producerInput),
     U64(localSlot))
```

Retained concepts carry their exact prior IDs through the canonical
`IdentityPlan`. An allocated identity retains its original allocation row and
preimage; it never changes provenance merely because a successor retains it.
This preserves distinct equal-looking occurrences and avoids recursive
identity definitions or host-allocation-order dependence.

Identity has five disjoint provenance cases:

- **seed nominal** — a literal `Seed` declaration or a `RetainedSeed`
  declaration backed by an exact predecessor seed; a successor-introduced seed
  is also an explicit predecessor-visible `SeedInput`;
- **allocated nominal** — an `Allocated` declaration whose ID is exactly the
  displayed `NewId` of its declared inputs and slot;
- **fixed core or physical** — wire/Core ABI tags, fixed domain IDs,
  `CoreContractId`, `PhysicalProfileId`, equality contracts, rule tags, and
  fixed physical-operation IDs;
- **source/content** — an ID defined from one independently supplied exact
  content preimage, notably `SourceArtifactId`; and
- **other derived** — origins, build requests, compiler semantics, revisions,
  packages, certificate conclusions, observations, and other hashes defined
  from canonical preimages.

This partition is by each declaration's recorded provenance, never by an ID
type such as binder, use, definition, or occurrence. Two values of the same
semantic ID type may therefore transform differently. Every raw typed nominal
field has one fixed domain and resolves one declaration; every nominal Term
occurrence is `NominalRef(domain,id)` and resolves that same declaration.
Fixed, content, and derived values use their distinct forms. An
undifferentiated semantic ID payload is nonconforming. Coincidental matching
bytes inside source or opaque payloads are not references.

Let `S` be exactly the finite `(domain, id)` identities of `Seed` and
`RetainedSeed` declarations in the transformed closure; repeated retained
appearances of one identity denote one member of `S`. An independent `π` is a
domain-preserving bijection on `S` only. A successor `SeedInput` is a reference
to its `Seed` declaration and follows that declaration's image; it is never a
second independently mapped declaration. Allocated, fixed, content, and other
derived IDs are not in `π`'s domain. `Renameπ` and its induced total map `π*`
apply this single case split to every declaration and reference:

1. a `Seed` or `RetainedSeed` ID and all of its references take the one `π`
   image; retained predecessor links are transformed with their enclosing
   predecessor;
2. an `Allocated` ID is never directly mapped by `π`; first transform its
   change and producer references with `π*`, then set its sole image to
   `NewId(domain, transformedChange, transformedProducer, localSlot)`;
3. a fixed ID and its references remain byte-identical;
4. a source/content preimage is structurally transformed where it contains
   typed references, then its ID is recomputed exactly once from that preimage;
   an opaque source byte string remains byte-identical; and
5. every other derived ID is recomputed exactly once, bottom-up, after all of
   its canonical preimages have their unique images.

Every nominal reference takes the image of its resolved declaration; it never
runs an independent rule. In particular, `π(NewId(...))` is undefined:
`π*` of an allocated ID is only `NewId(π*(...), π*(...), ...)`, so no direct
permutation can compete with allocation recomputation. A provenance mismatch,
missing or duplicate declaration, retention mismatch, allocation mismatch,
reference ambiguity, or collision rejects the transform.

After the case split, `Renameπ` restores every affected canonical order,
canonically re-encodes the result, transforms and rechecks certificate
statements and derivation conclusions, attaches transformed evidence, and
recomputes final whole-package identity.

An external genesis anchor or actual acceptance judgment is not transferred
by `Renameπ`; only the generic decode, check, and evaluation mechanism is
equivariant under a correspondingly transformed premise.

## Genesis acceptance

Let `P0` be exact package bytes, `R0` the supplied canonical build request,
`E0` the supplied evidence, `Gc` and `Ga` the supplied exact genesis compile
and admission fuel limits, and `I0` the supplied final package identity. Let
`anchor` be an owner admission judgment naming the package bytes at an exact
release object:

```text
AcceptGenesis(anchor, P0, R0, E0, Gc, Ga, I0) :=
  anchor selects exactBytes(P0)
  ∧ CanonicalCLCPv2(P0)
  ∧ exactBytes(P0.frame01) = exactCoreManifestBytes
  ∧ P0.subject.lineage = Genesis
  ∧ CoreWF(P0.subject)
  ∧ R0 = P0.subject.buildRequest
  ∧ ValidGenesisBuildRequest(R0, CoreContractId,
                                PhysicalProfileId)
  ∧ Gc = R0.compileFuel ∧ Gc > 0
  ∧ Ga = R0.admissionFuel ∧ Ga > 0
  ∧ E0 = P0.evidence
  ∧ E0 = GenesisEvidence
  ∧ I0.exactPackageBytes = exactBytes(P0)
  ∧ I0.packageHash = CompilerPackageHash(P0)
  ∧ AuthorizeDecoded(
      GenesisAuthorizationRequest(anchor, R0, E0, Gc, Ga, I0),
      P0) = Authorized(exactBytes(P0))
```

`ValidGenesisBuildRequest` is exactly every applicable ordered row of the
canonical package contract's `BuildRequest` stage: `GenesisBase`, carried core
and physical-profile derivations, source ordering and artifact identities,
the genesis identity plan and change occurrence, and the empty sealed-profile
physical-input list. It has no hidden fuel parameter; `Gc` and `Ga` are the
only genesis fuel inputs and are checked explicitly above. Genesis skips both
evaluation stages, so there is no genesis compile/admission remaining fuel or
observation input.

`I0` is the canonical package contract's
`FinalPackageIdentityInput(packageHash, exactPackageBytes)`. Its two checks are
the complete genesis `FinalAuthorization` stage: the bytes must be identical
to the strict decoder's retained exact input, and the hash must be the
domain-separated `CompilerPackageHash` of those same complete bytes, including
`GenesisEvidence`. Neither field can be omitted or replaced by the other.

The anchor records length and hashes for checking and retrieval, but those
values never substitute for the selected bytes. `R0`, `E0`, `Gc`, `Ga`, and
`I0` are mandatory checker inputs and exact-binding claims, not additional
sources of authority. The seed materializer, `GenesisEvidence`, canonicality,
successful execution, or derivability cannot create
`AcceptedCompiler(exactBytes(P0))`.

There is exactly one externally owner-anchored `Compiler0`. Re-encoding,
rebuilding, copying, or independently deriving identical bytes confirms that
selection; it does not create another root.

## Exact predecessor-only succession

For already accepted exact predecessor bytes `P`, candidate package `Q`,
canonical build request `R`, evidence `E`, and final package identity input
`I`:

```text
S  = exactCompilerSubjectBytes(Q)
M  = exactBytes(P.frame01) = exactCoreManifestBytes
X  = exactBytes(P.frame01.physicalProfile) = exactPhysicalProfileBytes
C  = DH("clause/core-contract/v1", exactCoreManifestBytes)
F  = DH("clause/physical-profile/v1", exactPhysicalProfileBytes)
Oc = exact canonical compile Observations
Oa = exact canonical admission Observations
Fc = exact compile remaining fuel
Fa = exact admission remaining fuel

CompileStatement = EvalStatement(
  exactAcceptedPredecessor = exactBytes(P),
  coreContractId = C,
  physicalProfileId = F,
  entrypoint = P.interface.compile,
  arguments = [TermValue(R)],
  fuelLimit = R.compileFuel,
  expected = Returned(TermValue(Built(S)), Fc, Oc))

AdmissionStatement = EvalStatement(
  exactAcceptedPredecessor = exactBytes(P),
  coreContractId = C,
  physicalProfileId = F,
  entrypoint = P.interface.admitPropose,
  arguments = [TermValue(AdmissionRequest(R, S, Oc))],
  fuelLimit = R.admissionFuel,
  expected = Returned(TermValue(Propose(S)), Fa, Oa))

AcceptSuccessor(P, Q, R, E, I) :=
  AcceptedCompiler(exactBytes(P))
  ∧ CanonicalCLCPv2(Q)
  ∧ exactBytes(Q.frame01) = exactCoreManifestBytes
  ∧ exactBytes(Q.frame01) = exactBytes(P.frame01)
  ∧ Q.subject.lineage.predecessorLocator = CompilerPackageHash(P)
  ∧ Q.subject.lineage is Successor
  ∧ CoreWF(Q.subject)
  ∧ R = Q.subject.buildRequest
  ∧ ValidSuccessorBuildRequest(R, exactBytes(P), C, F,
                               Q.subject.lineage.changeOccurrenceId)
  ∧ E = Q.evidence
  ∧ E = SuccessorEvidence(compileCertificate,
                          admissionCertificate)
  ∧ VerifyEvalCertificate(CompileStatement, compileCertificate)
  ∧ VerifyEvalCertificate(AdmissionStatement, admissionCertificate)
  ∧ I.exactPackageBytes = exactBytes(Q)
  ∧ I.packageHash = CompilerPackageHash(Q)
  ∧ AuthorizeDecoded(
      SuccessorAuthorizationRequest(P, R, E, I),
      Q) = Authorized(exactBytes(Q))
```

`Fc` and `Oc` are the exact remaining-fuel and observations fields decoded
from the compile certificate statement; `Fa` and `Oa` are the corresponding
admission fields. They are not host-selected expectations:
`VerifyEvalCertificate` proves each against the independently constructed root
judgment, and the admission argument reuses byte-identical `Oc`.

Both predecessor entrypoints are distinct definitions with the exact
`[Term] -> Term` signatures frozen by the Core ABI. `R`, `Built`, `Rejected`,
`AdmissionRequest`, `Propose`, `Reject`, `Observations`, and every observation
value have the one canonical Term representation defined by the
[package contract](canonical-package.md#clcp-v2-fixed-compiler-abi). A wrong
signature or shape, a `Rejected`/`Reject` return, a subject mismatch, a changed
observation, or an undeclared physical input rejects.

Each certificate states one non-recursive generic evaluation. It binds the
complete exact already accepted predecessor, core contract, physical profile,
entrypoint, canonical arguments, returned value, exact candidate subject, and
ordered observations. It does not bind `exactBytes(Q)` or
`CompilerPackageHash(Q)`, because either would include the certificate itself.
The package hash in `Q`'s lineage is only a predecessor locator; the checker
resolves it to `P` and compares exact bytes. Neither `Q`, a candidate-supplied
predecessor, a basis or rule in `Q`, nor hash equality can replace the accepted
exact predecessor.

The generic packager adds exactly `E = Q.evidence` without changing `S`. Lean
checks both certificates and the final identity's exact bytes and
domain-separated package hash, and only then returns the canonical
authorization result `Authorized(exactBytes(Q))`, whose authority
interpretation is exactly `AcceptedCompiler(exactBytes(Q))`. Only after that
attachment may `CompilerPackageHash(Q)` bind publication or retrieval. Failure
returns the unique first fixed `Unauthorized(stage, code)` and never a
candidate package. Required
rejection classes include a
root tag without the external anchor, self-authorization, candidate-basis
authorization, checking under the candidate, wrong or stale predecessor,
altered subject after compilation, transplanted certificate, altered request,
malformed trace, physical-profile escape, and correct hash paired with
non-identical bytes.

Malformed bytes instead return the separate canonical `DecodeRejected` value
and never enter this predicate. An explicit genesis or successor request
selects the route before candidate data is inspected. Every successfully
decoded failure follows the package contract's ascending stage, row, and
encoded-item precedence table: a rejection predicate includes passage of all
earlier conditions, making the predicates pairwise disjoint and the result
single-valued. There is no host-selected error priority. In particular, either
entrypoint's signature mismatch is only
`(CoreWellFormedness, EntrypointSignature)`.

## Lean checker boundary

Lean owns:

- strict CLCP v2 decoding, its separate deterministic decode verdict, and
  canonicality;
- exact-byte validation of the carried `CoreManifestV1` and physical profile;
- `Bytes`, `Term`, `KSort`, `KExpr`, definition-table
  well-formedness, and fixed generic evaluation relations;
- the closed `VerifyEvalCertificate` algorithm over fixed generic rule tags;
- exact-byte genesis selection as an explicit external premise;
- exact-predecessor succession checking; and
- enforcement of the sealed compiler physical profile.

Lean contains no Clause source parser and no construct-specific lexical,
grammar, binding, type, mode, effect, macro, diagnostic, or compiler-revision
rule. A Lean proof says that the generic machine evaluated the already
authoritative predecessor as claimed; it does not invent Clause feature
meaning or select a compiler.

## Rust evaluator and physical boundary

Rust may strictly decode and re-encode the fixed wire algebra; retain exact
bytes and checked spans; compute hashes; maintain indexes, storage, fuel, and
continuations; evaluate the fixed `KExpr` forms; resolve `DefId` by generic
table lookup; execute admitted physical requests through typed capability
adapters; and persist, schedule, or lower checked results.

Rust may not parse Clause source, resolve bindings, decide types, modes,
effects, capabilities, transformations, diagnostics, or compiler admission. It
may not dispatch a Clause construct through an enum, trait, callback, plugin,
formatter, validator, generated target case, package-local `DefId`, or
special semantic identifier. Successful Rust execution is evidence, not
authority.

```text
RustEval(exactAcceptedCompilerBytes, compileDef, [Term(buildRequest)])
  -> Returned(Term(Built(candidateSubject) | Rejected(diagnostics)),
              remainingFuel,
              Observations(...))
```

Rust executes; Lean checks. Neither silently owns Clause evolution.

## Machine-checkable host boundary

The audit distinguishes fixed generic mechanics from construct-specific
semantic dispatch. Generic mechanics necessarily inspect bytes, lengths,
keys, equality results, and expression data. The allowed host mechanism sites
are exactly:

```text
HostMechanic =
    WireCodec(tag, length, bound, byte)
  | CoreABI(tag, arity, fixed-field-shape)
  | ByteMachine(empty, head-tail, concat, equality)
  | DefinitionTable(key-order, hit-miss, selected-KExpr-data)
  | KernelStep(KSortTag, KExprTag, value-shape, fuel)
  | CertificateStep(CoreCertificateRuleTag, premise-index)
  | PhysicalDispatch(fixed PhysicalOpId)
```

Fixed wire decoding may inspect raw bytes to recognize fixed tags and enforce
lengths and bounds. The byte machine may inspect arbitrary values to implement
`CaseBytes`, `ConcatBytes`, and `CaseBytesEqual`. A package-local `DefId` may
control only generic table comparison and selection of a package `Definition`
record; the selected body remains `KExpr` data and re-enters the same evaluator.
Likewise, a token byte or semantic ID may affect package-program control by
selecting an already encoded case body. These data-plane choices are required
for a universal evaluator.

They may not select or synthesize a host semantic implementation: no lexer,
grammar case, binder, type/effect rule, macro expander, diagnostic formatter,
validator, trait method, plugin, generated target case, native function, or
specialized callback may be selected by `SemanticId`, Atom fields, token
bytes, production or diagnostic IDs, compiler revisions, or package-local
`DefId`. `PhysicalOpId` dispatch is permitted only for the fixed operation and
signature in the accepted physical profile; package data cannot extend it.

A source-AST plus type/information-flow extractor enumerates every branch and
indirect-call target in the trusted decode/check/evaluate closure, labels its
one `HostMechanic` class and controlling values, and rejects an unlabelled
site. For a package-influenced site it must prove that the outcome is only
canonical data, a fixed error, a child `KExpr`, a selected package definition,
or the one fixed mechanic handler selected by an enumerated wire, Core ABI,
`KExpr`, certificate-rule, or physical-operation tag. For a given fixed tag and
signature, the host code target must be invariant under every semantic ID and
raw payload value. No package value may create a new target or select different
host code for the same fixed mechanic. The checked artifact records source
locations, classes, taint sources, tags, and code targets; it is a
machine-produced manifest, not a prose or token search.

The behavioral companion uses the structure-preserving `Renameπ` operation
defined above. Let `StrictDecode(P) = Decoded(P,D)`,
`Dπ = Renameπ(D)`, `Pπ = EncodeCanonical(Dπ)`, and let `π*` include the
induced replacement of all recomputed derived IDs and hashes. Then:

```text
StrictDecode(Pπ) = Decoded(Pπ, Dπ)
EncodeCanonical(Dπ) = Pπ

VerifyEvalCertificate(π*(statement), π*(certificate))
  = VerifyEvalCertificate(statement, certificate)

EvalHost(Pπ, π*(input))
  = π*(EvalHost(P, input))
```

The check law transforms the complete exact-predecessor premise and certificate
statement together; it does not grant acceptance to renamed bytes. Canonical
orders are restored and content-derived values are recomputed, so neither
hashes nor serialized bytes are asserted to stay fixed or to equal a direct
bytewise permutation. Lean must prove these laws for the generic model, and
metamorphic vectors must exercise canonical re-encoding and the Rust
implementation. A nominal ID that selects a host semantic handler violates the
mechanics audit and the law even if hidden behind a callback.

## Compiler0 to Compiler1 falsifier

One ordinary predecessor-authorized `Compiler1` must make all four
package-only changes:

| Evolution | Required observation | Architecture is disproved if |
| --- | --- | --- |
| Binding form | A new reading and binding rules preserve exact binder/use identities through shadowing, closure capture, and alpha-renaming | A host gains a form case, resolution uses spelling/span, capture becomes accidental, or identity follows host allocation |
| Effect form | A new reading, mode, capability, transition, and intent rule admit intent before one separately identified attempt; replay performs no extra attempt | A host recognizes the effect, capability is ambient, attempt precedes admission, evidence is fabricated, or replay repeats the act |
| Typed macro | A binder-introducing typed transformation preserves phase, origins, types, capabilities, and hygiene | A host macro dispatcher appears, origin is lost, phase escape/capture succeeds, or output depends on host order |
| Diagnostic behavior | The unresolved-binding diagnostic document and fix change while its code identity and rejection semantics remain stable | A host formatter/diagnostic case appears, ordering varies, or package data cannot change the behavior |

The same previously built Lean and Rust binaries must accept and execute both
compilers. Across the transition there must be zero Lean or Rust source edits,
zero Lean or Rust toolchain changes, zero Lean or Rust binary changes, and zero
host-mechanics-manifest changes. Exact package, source, origin, binding, judgment,
decision, and diagnostic differences must be the changes proposed by
`Compiler0` and no others.

## Determinism and later fixpoint evidence

A canonical build request names exact source bytes, accepted base compiler,
target and core profiles, identity-retention data, change occurrence, options,
and every permitted physical input. It excludes ambient path, clock, locale,
randomness, environment, cache, thread order, and filesystem enumeration.

```text
DeterministicCompile(C, R) :=
  for all conforming evaluators E1 and E2,
  E1(C.compile, R) = E2(C.compile, R)
```

Equality covers subject bytes, source structures, origins, binder/use graph,
judgments, decisions, and diagnostics.

Later self-hosting evidence must establish:

```text
Compile(Compiler0, recipe(Compiler1)).subject
  = Compile(Compiler1, recipe(Compiler1)).subject
  = subject(Compiler1)
```

Two isolated rebuilds must then reproduce the whole `Compiler1` package,
including immutable historical evidence, byte for byte. Rebuilding never
creates a new admission or change occurrence.

## Residual uncertainty

The principal open risk is tractability: the fixed evaluator may make
`Compiler0` execution or generic Lean certificate checking too large.
Deterministic identity retention under realistic edits, sound static extraction
of host-mechanic and semantic-dispatch influence, the exact Lean trust closure
for decoding and SHA-256, and the size of self-source and immutable evidence
also remain unmeasured. P2 must measure them before introducing proof compression or
checked optimization. None permits a construct-specific host escape; failure
reopens this contract.
