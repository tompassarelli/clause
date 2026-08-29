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
executes its own `admit-propose` behavior over that exact subject. Lean checks
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
- a fixed construct-blind universal evaluator over `Bytes` and `Term`;
- the `compile` and `admit-propose` compiler entrypoints;
- literal external genesis selection and exact predecessor-only succession;
- lossless source, origin, occurrence, binding, elaboration, transformation,
  and diagnostic invariants;
- deterministic build inputs and later self-rebuild equations; and
- a machine-checkable host information-flow rule and identifier-permutation
  equivariance law.

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
  | Call(definitionId, arguments)
  | Request(physicalOperationId, arguments)
```

`CaseTerm` binds the three Atom byte fields or the three Triple children.
`CaseBytes` binds the leading octet and remaining bytes. Evaluation is
call-by-value, left-to-right, with explicit fuel and named recursive
definitions. Definitions are data addressed through opaque `DefId` lookup.
No token, lexer, binder, type, effect, macro, diagnostic, or compiler-version
tag is a kernel tag.

Compiler admission uses a sealed pure physical profile. During `compile` and
`admit-propose`, the only permitted physical request is deterministic
SHA-256. Clock, randomness, locale, environment, ambient filesystem paths,
filesystem enumeration, cache state, scheduling observations, network, FFI,
target execution, and undeclared operations reject.

The evaluator is intentionally tiny. Its adequacy and tractability for a real
compiler are unproved P2 questions; that uncertainty cannot be resolved by
adding a construct-specific host branch.

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
  interface: {
    compile: DefId,
    admitPropose: DefId
  },
  program: Seq<Definition>,
  buildBundle: BuildBundle
}
```

`interface || program` is the authoritative executable compiler. The build
bundle carries exact source units, base inputs, identity retentions, change
occurrence, and options needed to reproduce it. Embedded source is not a
second authority: if source and executable subject disagree, the accepted
executable subject governs, and exact rebuild failure exposes the disagreement.

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

Genesis nominal IDs are literal package data. A successor allocates a fresh ID
only from explicit predecessor-visible inputs:

```text
NewId(domain, changeOccurrenceId, producerOccurrenceId, localSlot) =
  DH(domain, changeOccurrenceId, producerOccurrenceId, U64(localSlot))
```

Retained concepts carry their exact prior IDs through explicit retention
judgments. `ReferentId`, `ScopeId`, `BinderId`, `UseOccurrenceId`,
`TransformOccurrenceId`, `DiagnosticOccurrenceId`, and
`ChangeOccurrenceId` are not content hashes of their containing graphs.
This preserves distinct equal-looking occurrences and avoids recursive
identity definitions or host-allocation-order dependence.

## Genesis acceptance

Let `P0` be exact package bytes and let `anchor` be an owner admission
judgment naming those bytes at an exact release object:

```text
AcceptGenesis(anchor, P0) :=
  anchor selects exactBytes(P0)
  ∧ CanonicalCLCPv2(P0)
  ∧ P0.subject.lineage = Genesis
  ∧ CoreWF(P0.subject)
```

The anchor records length and hashes for checking and retrieval, but those
values never substitute for the selected bytes. The seed materializer,
`GenesisEvidence`, canonicality, successful execution, or derivability
cannot create `AcceptedCompiler(exactBytes(P0))`.

There is exactly one externally owner-anchored `Compiler0`. Re-encoding,
rebuilding, copying, or independently deriving identical bytes confirms that
selection; it does not create another root.

## Exact predecessor-only succession

For already accepted exact predecessor bytes `P`, candidate package `Q`,
canonical build request `R`, and evidence `E`:

```text
AcceptSuccessor(P, Q, R, E) :=
  AcceptedCompiler(exactBytes(P))
  ∧ CanonicalCLCPv2(Q)
  ∧ Q.subject.lineage.predecessorLocator = CompilerPackageHash(P)
  ∧ CoreWF(Q.subject)
  ∧ CheckEval(
       exact program from P,
       P.interface.compile,
       R,
       Built(exactSubjectBytes(Q), observations),
       E.compileCertificate)
  ∧ CheckEval(
       exact program from P,
       P.interface.admitPropose,
       (R, exactSubjectBytes(Q), observations),
       Propose(exactSubjectBytes(Q)),
       E.admissionCertificate)
```

Both checks use the already accepted predecessor's exact program and
entrypoints. The package hash in `Q` is only a predecessor locator; the
checker resolves it to `P` and compares exact bytes. Neither `Q`, a
candidate-supplied predecessor, a basis or rule in `Q`, nor hash equality can
replace the accepted exact predecessor.

The generic packager adds `E` without changing the admitted subject. Lean
checks both certificates and only then yields
`AcceptedCompiler(exactBytes(Q))`. Required rejection classes include a
root tag without the external anchor, self-authorization, candidate-basis
authorization, checking under the candidate, wrong or stale predecessor,
altered subject after compilation, transplanted certificate, altered request,
malformed trace, physical-profile escape, and correct hash paired with
non-identical bytes.

## Lean checker boundary

Lean owns:

- strict CLCP v2 decoding and canonicality;
- `Bytes`, `Term`, `KSort`, `KExpr`, definition-table
  well-formedness, and fixed generic evaluation relations;
- fixed generic certificate-rule checking;
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
RustEval(exactAcceptedCompilerBytes, compileDef, exactBuildRequest)
  -> Built(candidateSubject, observations) | Rejected(diagnostics)
```

Rust executes; Lean checks. Neither silently owns Clause evolution.

## Machine-checkable host boundary

Every host branch and indirect-call target reachable from CLCP decoding,
checking, or evaluation must be controlled only by:

```text
AllowedHostDiscriminants =
  WireTag
  ∪ KSortTag
  ∪ KExprTag
  ∪ CoreCertificateRuleTag
  ∪ PhysicalOpId
```

`PhysicalOpId` here means only an operation fixed by the accepted core
physical profile. A package-local operation identifier cannot extend this set.

A source-AST and type/information-flow extractor must reject a reachable branch
or indirect-call target influenced by:

```text
SemanticId | Atom.kind | Atom.payload | token bytes | productionId |
diagnostic code | compiler revision | package-local DefId
```

A package-local `DefId` may be used only as an opaque key in generic table
lookup. The audit result is a checked host-branch manifest, not a prose search.

The behavioral companion is identifier-permutation equivariance. For every
bijection `π` over package-owned identifiers that fixes core and physical
identifiers:

```text
Decode(π(P))                   = π(Decode(P))
Check(π(P), π(claim), π(cert)) = Check(P, claim, cert)
EvalHost(π(P), π(input))       = π(EvalHost(P, input))
```

Lean must prove the law for the generic model, and metamorphic vectors must
exercise the Rust implementation. A semantic identifier that changes host
control flow violates the law even if the branch is hidden behind a generic
callback.

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
host-branch-manifest changes. Exact package, source, origin, binding, judgment,
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
of host-branch influence, the exact Lean trust closure for decoding and
SHA-256, and the size of self-source and immutable evidence also remain
unmeasured. P2 must measure them before introducing proof compression or
checked optimization. None permits a construct-specific host escape; failure
reopens this contract.
