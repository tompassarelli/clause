# Clause Canonical Packages

> **Contract:** CLCP v3 is the normative compiler-package contract.
>
> **Authority:** This document owns canonical package representation and
> canonical decoding. [Compiler genesis](compiler-genesis.md) owns compiler
> authority and succession, the [foundation](foundation.md) owns Clause
> meaning, and the [roadmap](roadmap.md) owns implementation status. Decoding,
> hashing, materialization, or derivability never grants meaning or authority.

## Version boundary

CLCP v1 and CLCP v3 are distinct closed formats. A decoder selects a format
only from the exact version octet after the `CLCP` magic and rejects all
other versions. There is no permissive common envelope, fallback decoder,
version inference, extension field, or alternate encoding.

CLCP v3 is the compiler carrier required by the
[genesis contract](compiler-genesis.md). It separates the compiler subject
from checking evidence so a receipt never contains, hashes, or authorizes
itself. The existing CLCP v1 implementation and byte corpus prove only their
narrow finite-ground-certificate boundary. They are not `Compiler0`, a v3
receipt, or authority for compiler evolution.

## Semantic namespace boundary

CLCP is a closed compiler-machine carrier, not the Clause process kernel.
Names in its wire grammar, evaluator, and checker remain scoped to that machine
even when they use an English word that also appears in Clause semantics:

- `KExpr`, its evaluation judgment, and any implementation-level evaluator
  continuation, reduction step, index, or fuel counter are compiler-machine
  mechanics. They are not a Clause `Application`, `Activation`, `Step`,
  `Run`, or `Continuation`.
- Core ABI `Observation` and `Observations` values are the evaluator's ordered
  physical-operation record. Their `index` is a machine sequence index, not
  an `ObservationId`, `StepId`, or claim that independent Clause steps are
  causally ordered.
- `EvalReceipt` commits to one exact deterministic replay result, remaining
  fuel, and machine-observation sequence. It is neither a Clause run trace nor
  a substitute for activation identity, causal evidence, or admission.
- `CompilerSemanticsId` and `CompilerRevisionId` identify the exact compiler
  interface/program content and compiler subject defined below. They are not
  Clause process-semantics identity or a `ProgramRevisionId`.
- Core ABI `Propose` is a compiler-program result and checker `Authorized` is
  a compiler-package authorization verdict. They implement this compiler
  succession boundary only; neither is the general Clause `Admission`
  relation nor creates a `ProgramRevision` or `StateRevision`.

A Clause-owned envelope may relate these exact machine artifacts to semantic
applications, activations, runs, observations, evidence, and revisions. The
relation must be explicit; shared spelling never creates semantic identity.

## CLCP v3 primitive encodings

```text
U8       = one octet
U32      = four-octet unsigned big-endian integer
U64      = eight-octet unsigned big-endian integer
Blob     = length:U32 || octets[length]
Seq<X>   = count:U32 || X[count]
Frame<X> = tag:U8 || payloadLength:U32 || X

Id32     = exactly 32 octets
Hash32   = exactly 32 octets
Span     = sourceArtifactId:Id32 || start:U64 || end:U64
```

All arithmetic is checked before cursor advance, conversion, iteration, or
allocation. Every frame and nested value consumes its bounded cursor exactly.
A Span is half-open and requires `start <= end <= byteLength(source)` for the
exact source artifact it names.
A sequence retains order. A sequence declared sorted is sorted by the exact
canonical bytes of its key and rejects duplicate keys. There are no varints,
host serializers, ignored fields, padding, trailing bytes, or alternate
spellings.

Record fields below are concatenated in the displayed order. Sum variants
begin with the displayed `U8` tag. A symbolic identifier ending in `Id` is
an `Id32` unless a different type is written explicitly. All displayed tag
octets are hexadecimal.

## CLCP v3 envelope

Every compiler package is exactly:

```text
43 4c 43 50                 magic ASCII "CLCP"
03                          version
Frame(01, CoreManifest)
Frame(02, CompilerSubject)
Frame(03, CompilerEvidence)
EOF
```

All three frames occur exactly once and in that order. Unknown, missing,
repeated, or reordered frames reject. Frame 01 carries the complete exact
generic machine manifest; it is not an identifier resolved through a host
registry. Its canonical bytes fix every sort, expression, Core ABI, static
rule, evaluation rule, receipt contract, authorization verdict, and physical
operation below. Package data cannot add a rule or reinterpret a tag.

## CLCP v3 Terms and evaluator expressions

```text
Term =
    00 Atom(kind:Blob, canonicalPayload:Blob, equalityContract:Blob)
  | 01 Triple(first:Term, second:Term, third:Term)

KSort =
    00 Bytes
  | 01 Term

KExpr =
    00 BytesLiteral(value:Blob)
  | 01 TermLiteral(value:Term)
  | 02 Var(deBruijnIndex:U32)
  | 03 MakeAtom(kind:KExpr, payload:KExpr, equality:KExpr)
  | 04 MakeTriple(first:KExpr, second:KExpr, third:KExpr)
  | 05 Let(value:KExpr, body:KExpr)
  | 06 CaseTerm(scrutinee:KExpr,
               atomBody:KExpr,
               tripleBody:KExpr)
  | 07 CaseBytes(scrutinee:KExpr,
                emptyBody:KExpr,
                consBody:KExpr)
  | 08 ConcatBytes(parts:Seq<KExpr>)
  | 09 CaseBytesEqual(left:KExpr,
                      right:KExpr,
                      equalBody:KExpr,
                      unequalBody:KExpr)
  | 0a Call(definitionId:Id32, arguments:Seq<KExpr>)
  | 0b Request(physicalOperationId:Id32, arguments:Seq<KExpr>)
```

Term and expression recursion is inline and finite because every nested value
must be consumed from the exact bounded input. Resource exhaustion is a
physical failure, not a different canonical decoding verdict.

Expression sorts are checked statically. `BytesLiteral`, `ConcatBytes`, and both
inputs to `CaseBytesEqual` have sort `Bytes`; every `ConcatBytes` part has sort
`Bytes`. `TermLiteral`, `MakeAtom`, and `MakeTriple` have sort `Term` with the
displayed child sorts. `Let` binds its value as `Var(0)`. In a selected
`CaseTerm` arm, `Var(0..2)` name the Atom kind/payload/equality or Triple
first/second/third in displayed order; in a `CaseBytes` cons arm, `Var(0)` is
the one-octet head and `Var(1)` the remaining tail. Existing variables shift
by the number of new bindings. Definition arguments use `Var(0..n-1)` in
displayed order. Every case arm has the enclosing expression's one common
result sort. `ConcatBytes` evaluates parts left to right and concatenates in
order; an empty sequence yields empty bytes. `CaseBytesEqual` adds no binding.
`Call` must match the referenced definition's exact argument and result sorts.
A `Request` must match an exact fixed physical-profile signature.

These are sufficient byte mechanics rather than semantic conveniences.
`CaseBytes` exposes each octet as a one-byte value;
`CaseBytesEqual(head, BytesLiteral(x), ...)` can distinguish any of the 256
octets. `ConcatBytes` constructs exact dynamic byte strings, and
`CaseBytesEqual` compares arbitrary byte strings, including a computed
SHA-256 result. Together with literals, structural recursion, definitions,
Terms, and the fixed SHA-256 request, the package program can scan tokens and
delimiters, copy and assemble source slices, encode checked lengths, compare
identifiers and hashes, build arbitrary canonical Terms and subject bytes, and
implement finite maps as package data. There is deliberately no host lexer,
string library, map callback, or construct equality hook. Tractability remains
an implementation gate, but operational expressiveness is not delegated to a
host escape.

## Canonical core manifest

Frame 01 has this closed encoding:

```text
CoreManifest =
  manifestVersion:U8
  frameTags:Seq<U8>
  termTags:Seq<U8>
  sortTags:Seq<U8>
  expressionForms:Seq<NamedSignature>
  abiForms:Seq<NamedSignature>
  premisePolicyTags:Seq<U8>
  lineageTags:Seq<U8>
  nominalDeclarationTags:Seq<U8>
  compilerEvidenceTags:Seq<U8>
  valueTags:Seq<U8>
  decodeVerdictTags:Seq<U8>
  decodeCodeTags:Seq<U8>
  authorizationStageTags:Seq<U8>
  authorizationCodeTags:Seq<U8>
  staticRules:Seq<RuleSignature>
  evaluationRules:Seq<RuleSignature>
  receiptFormatVersion:U8
  receiptSignature:Blob
  contractClauses:Seq<Blob>
  physicalProfile:PhysicalProfile

NamedSignature = tag:U8 || signature:Blob
RuleSignature  = tag:U8 || premisePolicy:U8 || clause:Blob

PremisePolicy =
    00 None
  | 01 One
  | 02 Two
  | 03 Three
  | 04 Four
  | 05 ExpressionSequence
  | 06 ArgumentSequence
  | 07 ArgumentSequenceAndBody

PhysicalProfile =
  profileVersion:U8
  observationPolicy:U8
  operations:Seq<PhysicalOperation>

PhysicalOperation =
  operationId:Id32
  arguments:Seq<KSort>
  result:KSort
```

Every `Blob` in the following value is the exact displayed ASCII between
quotes, without quotes or a terminator. `CoreManifestV1` is exactly:

```text
manifestVersion = 00
frameTags = [01, 02, 03]
termTags = [00, 01]
sortTags = [00, 01]

expressionForms = [
  (00, "BytesLiteral(value:Blob)->Bytes"),
  (01, "TermLiteral(value:Term)->Term"),
  (02, "Var(index:U32)->EnvironmentSort"),
  (03, "MakeAtom(kind:Bytes,payload:Bytes,equality:Bytes)->Term"),
  (04, "MakeTriple(first:Term,second:Term,third:Term)->Term"),
  (05, "Let(value:Any,body:(bind Any) Same)->Same"),
  (06, "CaseTerm(scrutinee:Term,atomBody:(bind Bytes Bytes Bytes) Same,tripleBody:(bind Term Term Term) Same)->Same"),
  (07, "CaseBytes(scrutinee:Bytes,emptyBody:Same,consBody:(bind Bytes Bytes) Same)->Same"),
  (08, "ConcatBytes(parts:Seq<Bytes>)->Bytes"),
  (09, "CaseBytesEqual(left:Bytes,right:Bytes,equalBody:Same,unequalBody:Same)->Same"),
  (0a, "Call(definition:Id32,arguments:DefinitionArguments)->DefinitionResult"),
  (0b, "Request(operation:Id32,arguments:PhysicalArguments)->PhysicalResult")
]

abiForms = [
  (00, "ListNil()"),
  (01, "ListCons(head:Term,tail:List)"),
  (02, "ValueBytes(value:Bytes)"),
  (03, "ValueTerm(value:Term)"),
  (04, "NominalRef(domain:Id32,id:Id32)"),
  (05, "FixedId(domain:Id32,id:Id32)"),
  (06, "ContentId(domain:Id32,id:Id32)"),
  (07, "DerivedId(domain:Id32,id:Id32)"),
  (08, "IdentityPlan(retained:List<Retain>,seedInputs:List<SeedInput>)"),
  (09, "Retain(ref:NominalRef)"),
  (0a, "SeedInput(ref:NominalRef)"),
  (10, "GenesisBase()"),
  (11, "AcceptedBase(packageHash:Hash32,revisionId:Id32)"),
  (12, "SourceUnit(unitId:Id32,artifactId:Hash32,bytes:Bytes)"),
  (13, "BuildRequest(base:GenesisBase|AcceptedBase,coreContractId:Hash32,physicalProfileId:Hash32,targetProfile:Term,sourceUnits:List<SourceUnit>,baseInputs:Term,identityRetentions:IdentityPlan,changeOccurrenceId:Id32,options:Term,compileFuel:U64,admissionFuel:U64,declaredPhysicalInputs:List<Term>)"),
  (14, "Built(subjectBytes:Bytes)"),
  (15, "Rejected(diagnostics:List<Term>)"),
  (16, "AdmissionRequest(buildRequest:BuildRequest,subjectBytes:Bytes,compileObservations:Observations)"),
  (17, "Propose(subjectBytes:Bytes)"),
  (18, "Reject(diagnostics:List<Term>)"),
  (19, "Observation(index:U64,operationId:Id32,arguments:List<KValue>,result:KValue)"),
  (1a, "Observations(items:List<Observation>)"),
  (1b, "Authorized(packageBytes:Bytes)"),
  (1c, "Unauthorized(stage:U8,code:U8)")
]

premisePolicyTags = [00, 01, 02, 03, 04, 05, 06, 07]
lineageTags = [00, 01]
nominalDeclarationTags = [00, 01, 02]
compilerEvidenceTags = [00, 01]
valueTags = [00, 01]
decodeVerdictTags = [00, 01]
decodeCodeTags = [00, 01, 02, 03, 04, 05, 06, 07, 08, 09]
authorizationStageTags = [40, 41, 42, 43, 44, 45, 46, 47, 48]
authorizationCodeTags  = [60, 61, 62, 63, 64, 65, 66, 67,
                          68, 69, 6a, 6b, 6c, 6d, 6e, 6f,
                          70, 71, 72, 73, 74, 75, 76, 77,
                          78, 79, 7a, 7b, 7c, 7d, 7e, 7f,
                          80, 81, 82, 83, 84, 85, 86, 87]

staticRules = [
  (20, 00, "Delta;Gamma |- BytesLiteral(b):Bytes"),
  (21, 00, "Delta;Gamma |- TermLiteral(t):Term"),
  (22, 00, "Delta;Gamma |- Var(i):Gamma[i] iff i<len(Gamma)"),
  (23, 03, "Delta;Gamma |- MakeAtom(k,p,q):Term iff k:Bytes and p:Bytes and q:Bytes"),
  (24, 03, "Delta;Gamma |- MakeTriple(a,b,c):Term iff a:Term and b:Term and c:Term"),
  (25, 02, "Delta;Gamma |- Let(v,b):r iff Delta;Gamma |- v:s and Delta;[s]++Gamma |- b:r"),
  (26, 03, "Delta;Gamma |- CaseTerm(s,a,t):r iff s:Term and Delta;[Bytes,Bytes,Bytes]++Gamma |- a:r and Delta;[Term,Term,Term]++Gamma |- t:r"),
  (27, 03, "Delta;Gamma |- CaseBytes(s,e,c):r iff s:Bytes and Delta;Gamma |- e:r and Delta;[Bytes,Bytes]++Gamma |- c:r"),
  (28, 05, "Delta;Gamma |- ConcatBytes(es):Bytes iff every es[i]:Bytes in encoded order"),
  (29, 04, "Delta;Gamma |- CaseBytesEqual(a,b,e,n):r iff a:Bytes and b:Bytes and e:r and n:r"),
  (2a, 06, "Delta;Gamma |- Call(d,args):r iff Delta contains exactly d:(ss)->r and len(args)=len(ss) and every args[i]:ss[i] in encoded order"),
  (2b, 06, "Delta;Gamma |- Request(op,args):r iff physicalProfile contains exactly op:(ss)->r and len(args)=len(ss) and every args[i]:ss[i] in encoded order")
]

evaluationRules = [
  (30, 00, "J(BytesLiteral(b),g,f,o)=>(BytesValue(b),f-1,o) iff f>0"),
  (31, 00, "J(TermLiteral(t),g,f,o)=>(TermValue(t),f-1,o) iff f>0"),
  (32, 00, "J(Var(i),g,f,o)=>(g[i],f-1,o) iff f>0 and i<len(g)"),
  (33, 03, "after charge evaluate k,p,q left-to-right as BytesValue(kb),BytesValue(pb),BytesValue(qb); return TermValue(Atom(kb,pb,qb)) with final fuel and observations"),
  (34, 03, "after charge evaluate a,b,c left-to-right as TermValue(av),TermValue(bv),TermValue(cv); return TermValue(Triple(av,bv,cv)) with final fuel and observations"),
  (35, 02, "after charge evaluate v to x, then evaluate b under [x]++g; return the body value, fuel, and observations"),
  (36, 02, "after charge evaluate s to TermValue(Atom(k,p,q)), then evaluate atomBody under [BytesValue(k),BytesValue(p),BytesValue(q)]++g; return the selected body outcome"),
  (37, 02, "after charge evaluate s to TermValue(Triple(a,b,c)), then evaluate tripleBody under [TermValue(a),TermValue(b),TermValue(c)]++g; return the selected body outcome"),
  (38, 02, "after charge evaluate s to BytesValue(empty), then evaluate emptyBody under g; return the selected body outcome"),
  (39, 02, "after charge evaluate s to BytesValue(head++tail) with len(head)=1, then evaluate consBody under [BytesValue(head),BytesValue(tail)]++g; return the selected body outcome"),
  (3a, 05, "after charge evaluate es left-to-right as BytesValue parts and return BytesValue(concat(parts)); empty es returns empty Bytes with post-charge fuel and unchanged observations"),
  (3b, 03, "after charge evaluate a then b as BytesValue and iff lengths and octets are equal evaluate equalBody under g; return the selected body outcome"),
  (3c, 03, "after charge evaluate a then b as BytesValue and iff lengths or octets differ evaluate unequalBody under g; return the selected body outcome"),
  (3d, 07, "after charge resolve exactly d, evaluate args left-to-right, then evaluate its body under exactly the argument values with no caller environment; return the body outcome"),
  (3e, 01, "after charge evaluate the sole argument as BytesValue(input), compute FIPS-180-4 SHA-256(input), return BytesValue(H0||H1||H2||H3||H4||H5||H6||H7), and append exactly Observation(len(o),Sha256OpId,[Value(Bytes,input)],Value(Bytes,digest))")
]

receiptFormatVersion = 00

receiptSignature = "EvalReceipt(formatVersion:ReceiptFormatVersion,expectedValueHash:Hash32,expectedRemainingFuel:U64,expectedObservationsHash:Hash32); ReceiptFormatVersion=00; expectedValueHash=DH(clause/eval-receipt-value/v1,canonical KValue bytes); expectedObservationsHash=DH(clause/eval-receipt-observations/v1,canonical Term bytes); KValue=00 BytesValue(Blob)|01 TermValue(Term)"

contractClauses = [
  "C00: U8=one octet;U32=four-octet unsigned big-endian;U64=eight-octet unsigned big-endian;Blob=U32 length||octets[length];Seq<X>=U32 count||X[count];Frame<X>=U8 tag||U32 payloadLength||X;Id32 and Hash32 are exactly 32 octets;Span=Id32 sourceArtifactId||U64 start||U64 end with start<=end<=source length;record fields concatenate in displayed order;sum variants begin with displayed U8;all arithmetic is checked before cursor change conversion iteration or allocation;every bounded value consumes exactly;no padding trailing bytes or alternate spelling",
  "C01: Term=00 Atom(kind:Blob,payload:Blob,equality:Blob)|01 Triple(first:Term,second:Term,third:Term); KSort=00 Bytes|01 Term; frameTags,termTags,sortTags,expressionForms,abiForms,premisePolicyTags,lineageTags,nominalDeclarationTags,compilerEvidenceTags,valueTags,decodeVerdictTags,decodeCodeTags,authorizationStageTags,authorizationCodeTags,staticRules,evaluationRules,receiptFormatVersion and physical profile values above are the complete closed tag sets and signatures",
  "C02: KTag=clause/core-abi/tag/v1; KBytes=clause/core-abi/bytes/v1; KId32=clause/core-abi/id32/v1; KU64=clause/core-abi/u64/v1; KEq=clause/core/bytes-equal/v1; Tag(t)=Atom(KTag,U8(t),KEq); Bytes(b)=Atom(KBytes,b,KEq); Id(id)=Atom(KId32,id,KEq) iff len(id)=32; Nat64(n)=Atom(KU64,U64(n),KEq); List([])=Tag(00); List(x::xs)=Triple(Tag(01),x,List(xs)); Record(t,xs)=Triple(Tag(t),List(xs),Tag(00)); Core ABI constructors and field counts are exactly abiForms in tag order; wrong Atom kind field count wrapper fixed width list shape or trailing field is invalid",
  "C03: CompilerSubject=lineage,nominalDeclarations,interface,program,buildRequest; lineage=00 Genesis|01 Successor(predecessorLocator:Hash32,changeOccurrenceId:Id32); interface=compile:Id32,admitPropose:Id32; Definition=id:Id32,arguments:Seq<KSort>,result:KSort,body:KExpr; definitions are sorted unique by id",
  "C04: NominalDeclaration=00 Seed(domain,id)|01 RetainedSeed(domain,id,predecessorRevisionId)|02 Allocated(domain,id,changeInput:NominalWireRef,producerInput:NominalWireRef,localSlot:U64); NominalWireRef=domain:Id32||id:Id32; declarations are sorted unique by domain||id and every nominal reference resolves exactly one declaration in its required domain",
  "C05: Seed is literal primitive provenance; RetainedSeed must match predecessor Seed or RetainedSeed and exact predecessor revision and cannot relabel Allocated; Allocated.id=DH(clause/new-nominal/v1,domain,wire(changeInput),wire(producerInput),U64(localSlot)); allocation inputs resolve and form an acyclic graph; dependency order then domain||id is the unique recomputation order; collision is invalid",
  "C06: IdentityPlan has separately sorted unique Retain(NominalRef) and SeedInput(NominalRef) lists; every successor RetainedSeed appears only in retained; every newly introduced successor Seed appears only in seedInputs; each row matches declaration provenance; genesis retained is empty; no reference appears in both lists",
  "C07: Delta is the canonical sorted unique definition table; Gamma and runtime environments use index-zero-first Var order; a definition is well formed iff its body has its declared result under its declared argument sorts and all transitive Call and Request references resolve; there is no subsorting, coercion, implicit argument, host value, fallback rule, or package-defined rule",
  "C08: J(expression,environment,fuelBefore,observationsBefore)=>(value,fuelAfter,observationsAfter) is the sole successful evaluation judgment; values are only BytesValue or TermValue; fuel is U64; every rule consumes one unit before premises; zero fuel has no judgment; premises run strictly left-to-right and thread exact fuel and observations; integer overflow, bad value sort, unresolved definition, malformed observation, physical failure, or out-of-fuel has no successful judgment",
  "C09: observationPolicy 00 appends exactly one canonical observation for each successful physical Request and none otherwise; observation indices are 0..n-1; the sole operation is Sha256OpId:[Bytes]->Bytes; SHA-256 is FIPS 180-4 over successive eight-bit message units and returns big-endian H0||H1||H2||H3||H4||H5||H6||H7; every other operation or signature is invalid",
  "C10: EvalReceipt is receiptSignature above and exactly 73 bytes: formatVersion 00, expectedValueHash:Hash32, expectedRemainingFuel:U64, expectedObservationsHash:Hash32; it contains no returned value, observations, request, predecessor package, expression, environment, rule, premise, node, graph, trace, or authority; unknown formatVersion is DecodeRejected(06,formatVersionOffset)",
  "C11: EvalRequest is checker-constructed and never encoded; it binds acceptedPredecessorPackageHash=CompilerPackageHash(exact already-accepted predecessor bytes), derived CoreContractId and PhysicalProfileId, exact entrypoint, canonical arguments, and exact nonzero fuel; its expression is Call(entrypoint,map(ValueLiteral,arguments)) under empty environment and Observations([])",
  "C12: Complete deterministic replay under evaluation rules 30..3e is the only receipt verification; success requires DH(clause/eval-receipt-value/v1,canonical actual KValue bytes)=expectedValueHash, actual remaining fuel=expectedRemainingFuel, and DH(clause/eval-receipt-observations/v1,canonical actual Observations Term bytes)=expectedObservationsHash; an unencodable actual value is replay failure and unencodable observations are observation mismatch; faults have no receipt form; an optional trace is diagnostic only and never admission authority",
  "C13: CompilerEvidence=00 GenesisEvidence with no payload|01 SuccessorEvidence(compileReceipt:EvalReceipt,admissionReceipt:EvalReceipt); evidence is never executable compiler meaning and cannot add a Core, evaluation rule, request, trace, or authority",
  "V01: VerifyEvalReceipt first requires receipt formatVersion 00",
  "V02: VerifyEvalReceipt strictly decodes the separately supplied exact predecessor bytes, requires caller-supplied acceptance of those exact bytes, requires request.acceptedPredecessorPackageHash=CompilerPackageHash(exact bytes), requires predecessor Frame01 byte-equal exactCoreManifestBytes, and independently derives CoreContractId and PhysicalProfileId",
  "V03: VerifyEvalReceipt requires both derived IDs equal the checker-constructed request fields, statically checks the predecessor under rules 20..2b, resolves the request entrypoint exactly once, and requires argument sorts equal its signature",
  "V04: VerifyEvalReceipt constructs Call(entrypoint,map(ValueLiteral,arguments)) without receipt input, where only BytesValue maps to BytesLiteral and TermValue maps to TermLiteral",
  "V05: VerifyEvalReceipt completely evaluates that call under empty environment, request fuelLimit, and Observations([]) using only fixed rules 30..3e and the carried physical profile",
  "V06: VerifyEvalReceipt canonicalizes and domain-hashes the actual replayed value and Observations Term and requires exact expectedValueHash, expectedRemainingFuel, and expectedObservationsHash equality; it never uses receipt data to construct either replay",
  "V07: success requires every prior step and uses no graph, trace, callback, theorem name, host rule registry, Boolean evaluator, or package rule",
  "D00: StrictDecode returns only Decoded(exactInput,candidate) or DecodeRejected(code,offset); codes in precedence order are 00 WrongMagic,01 UnknownVersion,02 FrameTagOrderOrCount,03 Truncated,04 LengthOrCountOverflow,05 InvalidFixedWidth,06 UnknownSumTag,07 BoundedValueUnderConsumed,08 BoundedValueOverConsumed,09 TrailingBytes; fields are read depth-first in encoded order and equal-offset ties use lower code",
  "D01: StrictDecode handles only closed byte grammar; order, uniqueness, exact manifest equality, reference bounds, ABI meaning, entrypoint signature, identity derivation, lineage/evidence consistency, receipt replay semantics, and profile conformance are authorization checks; malformed bytes never produce Unauthorized",
  "A00: Authorization starts only after Decoded(exactInput,Q) and requires exactly one explicit request: GenesisAuthorizationRequest(ownerAnchor,R,E,Gc,Ga,I) or SuccessorAuthorizationRequest(P,R,E,I), where ownerAnchor=Missing|Supplied(OwnerAnchorWitness), OwnerAnchorWitness is an opaque non-package-wire capability created only by the external human-owner selection act, observe(witness)=OwnerAnchorObservation(exactSelectedBytes:Blob,selectedByteLength:U64,selectedPackageHash:Hash32), Gc and Ga are U64, and I=FinalPackageIdentityInput(packageHash:Hash32,exactPackageBytes:Blob); no owner-anchor variant, witness, or observation is encoded in Q; the request variant, never candidate data, selects the route; stages run 40..48; successor skips 42; genesis skips 43,45,46,47; both run 44 and 48; each row condition belongs to exactly one stage and route; rows run left-to-right and collection failures use encoded item order; failure at position i means every earlier condition passed and condition i is false, so first-failure predicates are pairwise disjoint and the first false condition is the only verdict",
  "A40: CoreManifest rows=[manifest bytes differ exactCoreManifestBytes->(40,60)]",
  "A41: CoreWellFormedness rows=[subject or ABI semantic structure->(41,61),nominal provenance allocation retention or reference->(41,62),definition order or duplicate->(41,63),compile then admitPropose resolution->(41,64),entrypoints equal->(41,65),compile then admitPropose signature not [Term]->Term->(41,66),other static rule 20..2b->(41,67),Request outside exact profile->(41,68)]",
  "A42: GenesisAnchor rows=[lineage not Genesis->(42,69),supplied E not byte-identical Q.evidence or E not empty GenesisEvidence->(42,6a),ownerAnchor=Missing->(42,6b),ownerAnchor=Supplied(w) and observe(w) is not a self-consistent observation of the complete exact candidate because selectedByteLength!=byteLength(exactSelectedBytes) or selectedPackageHash!=CompilerPackageHash(exactSelectedBytes) or exactSelectedBytes is not octet-for-octet equal exactInput->(42,6c)]; length and hash checks never substitute for the final exact-byte equality or create authority",
  "A43: ExactPredecessor rows=[lineage not Successor->(43,6d),candidate self candidate-basis or candidate-rule authority->(43,6f),supplied predecessor not already accepted including stale revision->(43,6e),locator differs CompilerPackageHash(P)->(43,70),resolved bytes not byte-identical accepted P->(43,71)]",
  "A44: BuildRequest rows=[wrong ABI shape->(44,72),R not byte-identical Q.subject.buildRequest->(44,73),base route or exact base mismatch->(44,74),core ID mismatch->(44,75),profile ID mismatch->(44,76),source order or duplicate->(44,77),source artifact derivation->(44,78),IdentityPlan order uniqueness provenance retention or seed binding->(44,79),request lineage or nominal change occurrence mismatch->(44,7a),declared physical inputs nonempty->(44,7b),on genesis Gc or Ga zero or R.compileFuel!=Gc or R.admissionFuel!=Ga; on successor either R fuel zero->(44,7c)]",
  "A45: CompileEvaluation rows=[evidence or compile receipt shape->(45,7d),no successful complete replay or actual KValue has no canonical encoding->(45,80),DH(clause/eval-receipt-value/v1,canonical actual KValue bytes) differs expectedValueHash->(45,7e),remaining fuel differs expectedRemainingFuel->(45,7f),actual result not Built->(45,81),Built bytes differ Q.subject->(45,82),canonical actual compile Observations Term has no encoding or its DH(clause/eval-receipt-observations/v1,bytes) differs expectedObservationsHash->(45,83)]",
  "A46: AdmissionEvaluation rows=[admission receipt shape->(46,7d),construct admission request from verified actual compile observations then no successful complete replay or actual KValue has no canonical encoding->(46,80),DH(clause/eval-receipt-value/v1,canonical actual KValue bytes) differs expectedValueHash->(46,7e),remaining fuel differs expectedRemainingFuel->(46,7f),actual result not Propose->(46,81),Propose bytes differ Q.subject->(46,82),canonical actual admission Observations Term has no encoding or its DH(clause/eval-receipt-observations/v1,bytes) differs expectedObservationsHash->(46,83)]",
  "A47: EvidenceAttachment rows=[E not byte-identical Q.evidence->(47,84),Frame02 differs certified subject->(47,85),attaching E does not reproduce exact Q->(47,86)]",
  "A48: FinalAuthorization rows=[I.exactPackageBytes not byte-identical exactInput or I.packageHash!=DH(clause/compiler-package/v1,I.exactPackageBytes)->(48,87)]",
  "H00: DH(d,xs)=SHA256(U32(len(d))||ASCII(d)||each(U64(len(x))||x)); CoreContractId=DH(clause/core-contract/v1,exactCoreManifestBytes); PhysicalProfileId=DH(clause/physical-profile/v1,exactPhysicalProfileBytes); EvalReceiptValueHash=DH(clause/eval-receipt-value/v1,canonicalKValueBytes); EvalReceiptObservationsHash=DH(clause/eval-receipt-observations/v1,canonicalObservationsTermBytes); CompilerSemanticsId=DH(clause/compiler-semantics/v1,canonical(interface||program)); CompilerRevisionId=DH(clause/compiler-revision/v1,exactCompilerSubjectBytes); CompilerPackageHash=DH(clause/compiler-package/v1,exactWholePackageBytes); SourceArtifactId=DH(clause/source-artifact/v1,exactSourceBytes); BuildRequestId=DH(clause/compiler-build-request/v1,canonicalTermBytes(BuildRequest)); OriginId=DH(clause/origin/v1,canonicalAcyclicOriginNode); hashes never grant compiler authority",
  "P00: Package bytes are magic CLCP,version 03,Frame(01,CoreManifestV1),Frame(02,CompilerSubject),Frame(03,CompilerEvidence),EOF exactly once in order; Frame03 is excluded from subject and revision identities; successor Frame03 payload is exactly 147 bytes: tag 01 then two ordered 73-byte trace-free receipts; it contains no predecessor bytes, candidate evidence, returned value, observations, or candidate whole-package identity; only exact genesis anchor or separately supplied already-accepted exact predecessor can authorize"
]

physicalProfile = {
  profileVersion = 00,
  observationPolicy = 00,
  operations = [(Sha256OpId, [Bytes], Bytes)]
}

Sha256OpId = DH("clause/physical-op/v1", "sha256")
```

The sequences are encoded in displayed order. No other manifest value is CLCP
v3. `exactCoreManifestBytes` is the canonical encoding of this exact
`CoreManifestV1`; `exactPhysicalProfileBytes` is the canonical encoding of the
displayed `PhysicalProfile` field value alone. Strict decode parses that closed structure;
authorization then compares the carried Frame 01 payload byte for byte with
`exactCoreManifestBytes` before applying any Core rule to Frames 02 or 03.
There is no manifest registry, negotiation, executable rule text, or
package-defined metalanguage.

Observation policy `00` means that every successfully executed physical
request appends exactly one canonical observation and no other event does.
`Request(Sha256OpId, [Bytes]) -> Bytes(32 octets)` is the sole physical
signature. Any other operation, argument/result signature, profile field, or
manifest byte rejects.

### Fixed static judgments

Let `Δ` be the sorted definition table and let `Γ` be a sequence of sorts in
`Var` order. The only static judgment is `Δ ; Γ ⊢ expression : sort`.
Rules `20` and `21` give the literal's displayed sort. Rule `22` requires the
index to be in bounds and returns `Γ[index]`. Rules `23` and `24` require the
three displayed child sorts. Rule `25` checks the value at `s`, then the body
at `r` under `[s] ++ Γ`, and returns `r`. Rule `26` checks a Term scrutinee and
both bodies at one identical result sort under
`[Bytes,Bytes,Bytes] ++ Γ` and `[Term,Term,Term] ++ Γ`. Rule `27` checks a
Bytes scrutinee, the empty arm under `Γ`, and the cons arm at the same result
sort under `[Bytes,Bytes] ++ Γ`. Rule `28` requires every sequence member to be
Bytes. Rule `29` requires two Bytes operands and two arms of one result sort.
Rule `2a` requires one exactly resolved definition and argument sorts equal to
its declared sequence. Rule `2b` requires the one exact operation and
signature in the carried physical profile. No subsorting, coercion, implicit
argument, inferred host value, or fallback rule exists.

A definition is well formed exactly when its ID is unique in canonical order,
its body checks at its declared result under its displayed argument sorts, and
every transitive call and request resolves under these rules. Core
well-formedness additionally checks the exact manifest, the fixed Core ABI
shapes below, the two distinct interface definitions and their exact
`[Term] -> Term` signatures, and all canonical sort/order/identity constraints.

### Fixed evaluation state and rules

A value is `BytesValue` or `TermValue`. An environment is `Seq<KValue>` in
`Var` order. A successful judgment is:

```text
Δ ⊢ <expression, environment, fuelBefore, observationsBefore>
    ⇓ <value, fuelAfter, observationsAfter>
```

Fuel is `U64`. Every rule consumes one unit before any premise; zero fuel has
no successful judgment. Premises execute strictly left to right, threading
the preceding premise's remaining fuel and observations into the next.
Rules `30` and `31` return their literal. Rule `32` returns the in-bounds
environment value. Rules `33` and `34` evaluate three children and construct
the corresponding Term. Rule `35` evaluates the value, then the body under
`[value] ++ environment`. Rules `36` and `37` evaluate the scrutinee, select
exactly its Atom or Triple arm, and evaluate that arm under its three fields in
displayed `Var` order followed by the old environment. Rules `38` and `39`
evaluate Bytes, then select the empty arm or evaluate the cons arm under
`[oneByteHead, tail] ++ environment`.

Rule `3a` evaluates every part left to right and concatenates their Bytes; the
empty sequence returns empty Bytes. Rules `3b` and `3c` evaluate left then
right, compare exact byte length and octets, and evaluate only the equal or
unequal arm under the unchanged environment. Rule `3d` evaluates call
arguments left to right, resolves the one definition by exact ID comparison,
then evaluates its body under exactly the argument values in displayed order;
the caller environment is not inherited. Rule `3e` evaluates the one Bytes
argument, computes SHA-256 as specified by FIPS 180-4 over exactly that octet
string interpreted as successive eight-bit message units, returns the digest
as big-endian `H0 || H1 || ... || H7`, and appends
`Observation(nextIndex, Sha256OpId, [Value(Bytes,input)],
Value(Bytes,digest))`. `nextIndex` is the prior observation count and prior
indices must be `0..nextIndex-1`. Integer overflow, malformed observations,
out-of-fuel, an unresolved definition, a sort/value mismatch, or any physical
failure has no successful judgment.

## CLCP v3 fixed compiler ABI

The two compiler entrypoints exchange only `Term`, but their Term shapes are
not package-defined. They use this fixed Core ABI. Let these literal ASCII
byte strings be fixed Atom kinds:

```text
KTag   = "clause/core-abi/tag/v1"
KBytes = "clause/core-abi/bytes/v1"
KId32  = "clause/core-abi/id32/v1"
KU64   = "clause/core-abi/u64/v1"
KEq    = "clause/core/bytes-equal/v1"

Tag(t)   = Atom(KTag,   U8(t),  KEq)
Bytes(b) = Atom(KBytes, b,      KEq)
Id(id)   = Atom(KId32,  id,     KEq)   where byteLength(id) = 32
Nat64(n) = Atom(KU64,   U64(n), KEq)

List([])       = Tag(00)
List(x :: xs)  = Triple(Tag(01), x, List(xs))
Record(t, xs)  = Triple(Tag(t), List(xs), Tag(00))
Value(Bytes,b) = Record(02, [Bytes(b)])
Value(Term,t)  = Record(03, [t])
NominalRef(domain,id) = Record(04, [Id(domain), Id(id)])
FixedId(domain,id)   = Record(05, [Id(domain), Id(id)])
ContentId(domain,id) = Record(06, [Id(domain), Id(id)])
DerivedId(domain,id) = Record(07, [Id(domain), Id(id)])
IdentityPlan(retained, seedInputs) =
  Record(08, [List(retained), List(seedInputs)])
Retain(ref)    = Record(09, [ref])
SeedInput(ref) = Record(0a, [ref])
```

Only the fixed tags below are valid at their declared positions. A lookalike
Atom kind, wrong field count, wrong field wrapper, non-`Id32` identifier,
noncanonical list, or trailing field rejects. Hosts may validate these fixed
ABI tags; package data cannot add one.

```text
GenesisBase = Record(10, [])
AcceptedBase(packageHash, revisionId) =
  Record(11, [Id(packageHash), Id(revisionId)])

SourceUnit(unitId, artifactId, bytes) =
  Record(12, [Id(unitId), Id(artifactId), Bytes(bytes)])

BuildRequest(base, coreContractId, physicalProfileId, targetProfile,
             sourceUnits, baseInputs, identityRetentions,
             changeOccurrenceId, options, compileFuel, admissionFuel,
             declaredPhysicalInputs) =
  Record(13, [base,
              Id(coreContractId),
              Id(physicalProfileId),
              targetProfile,
              List(sourceUnits),
              baseInputs,
              identityRetentions,
              Id(changeOccurrenceId),
              options,
              Nat64(compileFuel),
              Nat64(admissionFuel),
              List(declaredPhysicalInputs)])

Built(subjectBytes)       = Record(14, [Bytes(subjectBytes)])
Rejected(diagnostics)     = Record(15, [List(diagnostics)])

AdmissionRequest(buildRequest, subjectBytes, compileObservations) =
  Record(16, [buildRequest, Bytes(subjectBytes), compileObservations])

Propose(subjectBytes)     = Record(17, [Bytes(subjectBytes)])
Reject(diagnostics)       = Record(18, [List(diagnostics)])

Observation(index, operationId, arguments, result) =
  Record(19, [Nat64(index), Id(operationId),
              List(arguments), result])
Observations(items)       = Record(1a, [List(items)])

Authorized(packageBytes)  = Record(1b, [Bytes(packageBytes)])
Unauthorized(stage, code) = Record(1c, [Tag(stage), Tag(code)])

AuthorizationStage =
    40 CoreManifest
  | 41 CoreWellFormedness
  | 42 GenesisAnchor
  | 43 ExactPredecessor
  | 44 BuildRequest
  | 45 CompileEvaluation
  | 46 AdmissionEvaluation
  | 47 EvidenceAttachment
  | 48 FinalAuthorization

AuthorizationCode =
    60 ManifestMismatch
  | 61 SubjectStructure
  | 62 NominalTable
  | 63 DefinitionOrderOrDuplicate
  | 64 EntrypointResolution
  | 65 EntrypointAliased
  | 66 EntrypointSignature
  | 67 StaticRule
  | 68 PhysicalRequestSignature
  | 69 GenesisWrongLineage
  | 6a GenesisEvidenceNotEmpty
  | 6b MissingAnchor
  | 6c AnchorBytesMismatch
  | 6d SuccessorWrongLineage
  | 6e PredecessorNotAccepted
  | 6f CandidateOrSelfPredecessor
  | 70 LocatorMismatch
  | 71 PredecessorBytesMismatch
  | 72 BuildRequestShape
  | 73 DetachedBuildRequest
  | 74 BaseMismatch
  | 75 CoreContractMismatch
  | 76 PhysicalProfileMismatch
  | 77 SourceOrderOrDuplicate
  | 78 SourceArtifactMismatch
  | 79 IdentityPlanMismatch
  | 7a ChangeOccurrenceMismatch
  | 7b PhysicalInputsNonempty
  | 7c FuelInvalid
  | 7d EvidenceShapeMismatch
  | 7e ReceiptValueMismatch
  | 7f ReceiptFuelMismatch
  | 80 EvaluationFault
  | 81 UnexpectedResult
  | 82 SubjectMismatch
  | 83 ObservationMismatch
  | 84 EvidenceDetached
  | 85 SubjectChangedAfterCompile
  | 86 PackageChangedAfterEvidence
  | 87 FinalIdentityMismatch
```

Authorization has one explicit request algebra outside the package wire:

```text
AuthorizationRequest =
    GenesisAuthorizationRequest(
      ownerAnchor:OwnerAnchorInput,
      buildRequest:Term,
      evidence:CompilerEvidence,
      compileFuelLimit:U64,
      admissionFuelLimit:U64,
      finalIdentity:FinalPackageIdentityInput)
  | SuccessorAuthorizationRequest(
      exactAcceptedPredecessor:Blob,
      buildRequest:Term,
      evidence:CompilerEvidence,
      finalIdentity:FinalPackageIdentityInput)

FinalPackageIdentityInput = {
  packageHash:Hash32,
  exactPackageBytes:Blob
}

OwnerAnchorInput =
    Missing
  | Supplied(witness:OwnerAnchorWitness)

observe(OwnerAnchorWitness) = OwnerAnchorObservation {
  exactSelectedBytes:Blob,
  selectedByteLength:U64,
  selectedPackageHash:Hash32
}
```

These are checker inputs, not fields added to Frames 01, 02, or 03. The
request variant selects the authorization route before candidate lineage or
evidence is inspected, so a wrong lineage has its route-specific table
verdict. The genesis fuel limits are explicit `U64` inputs and have no ambient
default. They must be nonzero and equal the two fuel fields in the exact
genesis `BuildRequest`. Genesis has no evaluation receipt, remaining-fuel
value, or observation input. The final identity input always carries both the
claimed hash and the complete exact bytes; a hash alone never identifies the
candidate for authorization.

`OwnerAnchorWitness` is an opaque external admission capability issued only by
the irreducible human-owner selection act at the exact release object. Neither
candidate bytes nor a decoder, materializer, hash match, derivation, or
successful evaluation can construct it. Its `OwnerAnchorObservation` is not a
Core `Observations` value and has no CLCP encoding. The checker first preserves
the table's earlier lineage/evidence precedence, then maps `Missing` to
`(42,6b)`. Only for `Supplied(w)` does it obtain `observe(w)`, require the
recorded length and domain-separated package hash to match
`exactSelectedBytes`, and independently compare every selected octet and the
selected length with the strict decoder's retained `exactInput`. Any failure
of that supplied-witness conjunction is `(42,6c)`. Length and digest may expose
corruption or aid retrieval, but exact-byte equality is mandatory and the
external selection act remains the sole source of genesis authority.

An observation index starts at zero and increases by one. Its arguments and
result use `Value`; under the sealed profile the only valid item is an exact
`Sha256OpId`, one `Value(Bytes, input)` argument, and one
`Value(Bytes, 32-octet digest)` result. `declaredPhysicalInputs` is empty in
the sealed profile because SHA-256 has no external input. Diagnostics are
Clause Terms in their compiler-produced canonical order.

`sourceUnits` is sorted by `unitId`, rejects duplicates, and requires every
`artifactId = SourceArtifactId(bytes)`. `AcceptedBase` is a locator only: the
succession checker requires its two values to be derived from the exact
accepted predecessor supplied outside the request. `GenesisBase` is valid
only for the externally anchored genesis subject. The request's
`coreContractId` must equal `CoreContractId` derived from the exact carried
Frame 01 bytes. Its `physicalProfileId` must equal `PhysicalProfileId` derived
from that manifest's exact physical-profile suffix. `compileFuel` and
`admissionFuel` are the respective checker-constructed request limits and must be nonzero. The
successor request's change occurrence must equal its lineage change occurrence,
and `identityRetentions` must be the exact `IdentityPlan` validated with the
subject's nominal declaration table.

The required entrypoint definitions are exactly:

```text
compile      : [Term] -> Term
admitPropose : [Term] -> Term

compile(BuildRequest(...))
  -> Built(exactCompilerSubjectBytes)
   | Rejected(diagnostics)

admitPropose(AdmissionRequest(BuildRequest(...),
                              exactCompilerSubjectBytes,
                              Observations(...)))
  -> Propose(exactCompilerSubjectBytes)
   | Reject(diagnostics)
```

The interface IDs are distinct, each resolves exactly once, and both resolved
definitions have these one-argument signatures. A signature mismatch, wrong
request/result shape, subject mismatch, or unexpected ABI tag rejects rather
than invoking package behavior through a host adapter.

Authorization begins only after a successful strict decode. Exactly one
`AuthorizationRequest` variant selects the route; candidate data never selects
or changes it. The checker visits stages in ascending tag order, skipping
`GenesisAnchor` on a successor and skipping `ExactPredecessor`, both evaluation
stages, and `EvidenceAttachment` on genesis; both routes run `BuildRequest` and
`FinalAuthorization`. Within a stage it performs the following rows top to
bottom and returns the displayed pair for the first false row:

Row ownership is closed rather than overlapping. `SubjectStructure` checks
the residual outer subject and fixed-ABI structure not assigned to another
`41` row or to stages `42..48`; in particular it does not classify route
lineage, the supplied build request, evidence/receipts, or final identity.
Stages `42` and `43` alone classify route authority, `44` alone classifies the
supplied request, `45` and `46` alone classify their respective evaluation
evidence, `47` alone classifies attachment, and `48` alone classifies the final
identity input. An earlier stage never reads a later-stage value whose shape
has not yet passed. Consequently successor stage `44` checks that both request fuels are nonzero;
stages `45` and `46` construct their exact evaluation requests rather than
accepting request fields from Frame 03.

| Stage | Ordered false condition | Exact `Unauthorized(stage, code)` |
| --- | --- | --- |
| `40 CoreManifest` | carried Frame 01 is not byte-identical `CoreManifestV1` | `(40,60)` |
| `41 CoreWellFormedness` | subject/ABI semantic structure is invalid | `(41,61)` |
| | nominal declaration, provenance, allocation, retention, or reference check fails | `(41,62)` |
| | definition order or uniqueness fails | `(41,63)` |
| | `compile`, then `admitPropose`, does not resolve exactly once | `(41,64)` |
| | the two entrypoint IDs are equal | `(41,65)` |
| | `compile`, then `admitPropose`, is not exactly `[Term] -> Term` | `(41,66)` |
| | any other rule `20..2b` check fails | `(41,67)` |
| | a `Request` operation or signature is outside the exact profile | `(41,68)` |
| `42 GenesisAnchor` | lineage is not `Genesis` | `(42,69)` |
| | supplied evidence differs from `Q.evidence` or is not empty `GenesisEvidence` | `(42,6a)` |
| | `ownerAnchor` is `Missing` | `(42,6b)` |
| | `ownerAnchor` is `Supplied(w)`, but its observation length/hash is inconsistent with its selected bytes or those complete selected bytes are not octet-for-octet equal the strict decoder's retained exact candidate input | `(42,6c)` |
| `43 ExactPredecessor` | lineage is not `Successor` | `(43,6d)` |
| | candidate, self, candidate basis, or candidate rule is offered as predecessor authority | `(43,6f)` |
| | supplied predecessor bytes are not already accepted, including a stale revision | `(43,6e)` |
| | lineage locator differs from `CompilerPackageHash(P)` | `(43,70)` |
| | resolved locator/hash is paired with bytes not identical to accepted `P` | `(43,71)` |
| `44 BuildRequest` | request is not the exact fixed ABI shape | `(44,72)` |
| | supplied `R` is not byte-identical `Q.subject.buildRequest` | `(44,73)` |
| | `GenesisBase`/`AcceptedBase` does not match the route and exact base | `(44,74)` |
| | core contract ID does not equal the carried-manifest derivation | `(44,75)` |
| | physical profile ID does not equal the carried-profile derivation | `(44,76)` |
| | source-unit order or uniqueness fails | `(44,77)` |
| | a source artifact ID differs from its exact byte derivation | `(44,78)` |
| | `IdentityPlan` order, uniqueness, provenance, retention, or seed-input binding fails | `(44,79)` |
| | request, lineage, or nominal-table change occurrence differs | `(44,7a)` |
| | declared physical inputs is nonempty | `(44,7b)` |
| | on genesis, either explicit fuel input is zero or differs from its exact request field; on a successor, either request fuel is zero | `(44,7c)` |
| `45 CompileEvaluation` | successor evidence or compile-receipt shape is wrong | `(45,7d)` |
| | complete compile replay has no successful outcome, or the actual value has no canonical encoding | `(45,80)` |
| | the domain hash of canonical actual value bytes differs from `compileReceipt.expectedValueHash` | `(45,7e)` |
| | replayed remaining fuel differs from `compileReceipt.expectedRemainingFuel` | `(45,7f)` |
| | replayed result is not canonical `Built` | `(45,81)` |
| | `Built` bytes differ from `exactCompilerSubjectBytes(Q)` | `(45,82)` |
| | canonical actual observations have no encoding, or their domain hash differs from `compileReceipt.expectedObservationsHash` | `(45,83)` |
| `46 AdmissionEvaluation` | admission-receipt shape is wrong | `(46,7d)` |
| | the checker constructs `AdmissionRequest` from verified actual compile observations and complete admission replay has no successful outcome, or the actual value has no canonical encoding | `(46,80)` |
| | the domain hash of canonical actual value bytes differs from `admissionReceipt.expectedValueHash` | `(46,7e)` |
| | replayed remaining fuel differs from `admissionReceipt.expectedRemainingFuel` | `(46,7f)` |
| | replayed result is not canonical `Propose` | `(46,81)` |
| | proposed bytes differ from `exactCompilerSubjectBytes(Q)` | `(46,82)` |
| | canonical actual observations have no encoding, or their domain hash differs from `admissionReceipt.expectedObservationsHash` | `(46,83)` |
| `47 EvidenceAttachment` | supplied `E` is not byte-identical `Q.evidence` | `(47,84)` |
| | Frame 02 differs from the exact subject certified at compile/admission | `(47,85)` |
| | attaching exact `E` does not reproduce complete exact `Q` | `(47,86)` |
| `48 FinalAuthorization` | final-identity bytes differ from the decoded exact input, or its hash differs from `CompilerPackageHash` over those supplied exact bytes | `(48,87)` |

Collection checks visit encoded fields, definitions, source units, identity
rows and receipt fields in their canonical order. Expand the
selected route into that one ordered sequence of conditions. For condition
`c[i]`, its rejection predicate is `not c[i]` together with every earlier
`c[j]`; those predicates are pairwise disjoint, and a later predicate is not
eligible after an earlier failure. Thus stage order, row order, and then
encoded item order break every tie, even when several unqualified conditions
are false. A condition belongs to exactly one route/stage row, while multiple
items or conditions may intentionally share the same displayed pair. The two
entrypoint fields are checked `compile` then `admitPropose`; either signature
failure has the one pair `(41,66)`, never a BuildRequest or evaluation pair.
Every rejection named by this contract is assigned above: self/candidate basis
is `(43,6f)`, stale predecessor `(43,6e)`, hash-equal nonidentical predecessor
bytes `(43,71)`, an actual canonical value-hash mismatch `(45|46,7e)`, an
actual remaining-fuel mismatch `(45|46,7f)`, profile escape `(41,68)`, detached
evidence `(47,84)`, post-certification mutation `(47,85|86)`, and a final
exact-byte or package-hash mismatch `(48,87)` in the displayed order.

A successful check returns only `Authorized` with the complete exact final
package bytes after `FinalAuthorization` has matched both fields of the
explicit final-identity input. No decoded failure can satisfy two rejection
predicates or produce two observable pairs, and the result cannot degrade into
a Boolean, hash-only success, or candidate subject.

## CLCP v3 compiler subject

```text
CompilerSubject =
  lineage:CompilerLineage
  nominalDeclarations:Seq<NominalDeclaration>
  interface:CompilerInterface
  program:Seq<Definition>
  buildRequest:Term

CompilerLineage =
    00 Genesis
  | 01 Successor(
         predecessorLocator:Hash32,
         changeOccurrenceId:Id32)

CompilerInterface =
  compile:Id32
  admitPropose:Id32

NominalDeclaration =
    00 Seed(domain:Id32, id:Id32)
  | 01 RetainedSeed(
         domain:Id32,
         id:Id32,
         predecessorRevisionId:Id32)
  | 02 Allocated(
         domain:Id32,
         id:Id32,
         changeInput:NominalWireRef,
         producerInput:NominalWireRef,
         localSlot:U64)

NominalWireRef = domain:Id32 || id:Id32

Definition =
  id:Id32
  arguments:Seq<KSort>
  result:KSort
  body:KExpr
```

`nominalDeclarations` is sorted by `domain || id`; `program` is sorted by
`Definition.id`; duplicate keys reject. `DefDomain`, `SourceUnitDomain`, and
`ChangeOccurrenceDomain` are respectively the `DH` values for fixed ASCII
components `"definition"`, `"source-unit"`, and `"change-occurrence"` under
domain `"clause/nominal-domain/v1"`. Interface, definition, and `Call` IDs
resolve under `DefDomain`; source-unit IDs resolve under `SourceUnitDomain`;
lineage and request change IDs resolve under `ChangeOccurrenceDomain`. Every
other semantic ID in a Term uses `NominalRef(domain,id)` and resolves exactly
one declaration. Fixed, content, and derived IDs use their distinct Core ABI
forms. An untyped semantic ID payload or unresolved reference rejects.

`Seed` is a literal primitive identity. `RetainedSeed` must match an exact
`Seed` or `RetainedSeed` in the accepted predecessor and its displayed
predecessor revision; it may not relabel an `Allocated` declaration.
`Allocated.id` must equal
`NewId(domain, changeInput, producerInput, localSlot)`, and both inputs
must resolve. Carrying an allocated identity into a successor preserves the
same `Allocated` row and allocation preimage; it never becomes a retained
seed. Allocation dependencies form a finite acyclic graph. Their unique
topological recomputation order is dependency order with canonical
`domain || id` order as the tie-break; a cycle or recomputed collision rejects.

The build request's `IdentityPlan` is canonical: `retained` and `seedInputs`
are separately sorted unique `Retain(NominalRef(...))` and
`SeedInput(NominalRef(...))` lists. Every successor `RetainedSeed` is in the
first list; every newly introduced successor `Seed` is in the second and is
therefore explicit input to predecessor compilation and admission. Every
listed reference has the matching declaration provenance. Genesis has no
retentions. No identity may appear in both lists.

The interface and request satisfy the fixed ABI above. A Genesis subject uses
`GenesisBase`; a Successor subject uses `AcceptedBase` and the exact matching
lineage change occurrence.

The exact Frame 02 payload is `exactCompilerSubjectBytes`. The interface and
program are executable compiler meaning. The canonical build request is exact
reproducibility input carried inside the subject, not a second executable
authority.

## CLCP v3 evidence

```text
CompilerEvidence =
    00 GenesisEvidence
  | 01 SuccessorEvidence(
         compileReceipt:EvalReceipt,
         admissionReceipt:EvalReceipt)

EvalReceipt =
  formatVersion:ReceiptFormatVersion
  expectedValueHash:Hash32
  expectedRemainingFuel:U64
  expectedObservationsHash:Hash32

ReceiptFormatVersion = 00

KValue =
    00 BytesValue(value:Blob)
  | 01 TermValue(value:Term)
```

`GenesisEvidence` has no payload. A successor Frame 03 contains exactly two
73-byte receipts in compile-then-admission order, so the complete successor
evidence payload is exactly 147 bytes including its leading `01` tag. A receipt
contains only format version `00`, canonical value and observation commitments,
and exact remaining fuel. It contains no returned value, observations,
evaluation request, predecessor bytes, expression, environment, rule tag,
premise, node, graph, or trace. Unknown receipt versions reject at the version
octet. External diagnostic artifacts may accompany a reproducible corpus, but
they are outside authorization and cannot supply a result, skip replay, or add
authority.

The checker constructs this non-wire request for each replay:

```text
EvalRequest =
  acceptedPredecessorPackageHash:Hash32
  coreContractId:Hash32
  physicalProfileId:Hash32
  entrypoint:Id32
  arguments:Seq<KValue>
  fuelLimit:U64
```

The caller separately supplies the complete exact already-accepted predecessor
bytes and their acceptance premise. The request binds those bytes with
`acceptedPredecessorPackageHash = CompilerPackageHash(exactBytes)`; it does
not recursively encode them. The core/profile IDs, entrypoint, arguments, and
fuel are constructed from the accepted predecessor and exact candidate build
request, never copied from Frame 03. Although Lean represents fuel with an
unbounded natural internally, both request fuel and expected remaining fuel
must fit `U64`; zero request fuel and values above `2^64-1` reject, while the
maximum `U64` value remains valid.

`VerifyEvalReceipt(exactPredecessor, accepted, request, receipt)` is this
algorithm, in order:

1. require `receipt.formatVersion = 00` and require
   `receipt.expectedRemainingFuel` to fit `U64`;
2. strictly decode the separately supplied predecessor bytes, require the
   caller's acceptance premise for those exact bytes, require the request's
   predecessor hash to equal `CompilerPackageHash(exactBytes)`, require its
   Frame 01 bytes equal `exactCoreManifestBytes`, and independently derive
   `CoreContractId` and `PhysicalProfileId`;
3. require those derived IDs equal the checker-constructed request fields,
   require positive `U64` request fuel, statically check the predecessor
   subject under rules `20..2b`, resolve the entrypoint exactly once, and
   require argument sorts equal its signature;
4. construct `Call(entrypoint, map(ValueLiteral, arguments))` without receipt
   input, where `ValueLiteral` maps only `BytesValue` to `BytesLiteral` and
   `TermValue` to `TermLiteral`;
5. completely replay that call under the empty environment, exact request fuel,
   `Observations([])`, fixed evaluation rules `30..3e`, and the carried
   physical profile; and
6. canonically encode the actual value and `Observations` Term, hash them under
   `clause/eval-receipt-value/v1` and
   `clause/eval-receipt-observations/v1`, and require those hashes plus actual
   remaining fuel to equal the three receipt fields.

Success requires every step. There is no graph, trace, callback, theorem name,
host rule registry, Boolean-evaluator assertion, or package-defined rule.

For successor authorization the checker first constructs and replays
`compile(BuildRequest)`, requires the exact result
`Built(exactCompilerSubjectBytes)`, and retains the replay's actual canonical
observations. It then constructs
`admitPropose(AdmissionRequest(BuildRequest, exactCompilerSubjectBytes,
actualCompileObservations))`, replays it completely, and requires the exact
`Propose(exactCompilerSubjectBytes)` result. A claimed compile observation
cannot steer admission until complete replay has reproduced it.

Frame 03 is excluded from `exactCompilerSubjectBytes`,
`CompilerSemanticsId`, and `CompilerRevisionId`. Predecessor compilation
and admission target the exact subject bytes, then a generic packager attaches
the two receipts without modifying the subject. No package contains recursively
embedded predecessor packages or its own package hash, subject ID, or revision
ID. `CompilerPackageHash` covers the final whole package and is computed only
after canonical packaging; it can bind publication and exact predecessor
selection but never creates compiler authority.

## CLCP v3 canonical decoding

Strict decoding has a separate result algebra; it never returns a Core ABI
`Unauthorized` Term:

```text
DecodeVerdict =
    00 Decoded(exactInput:Blob, candidate:CompilerPackage)
  | 01 DecodeRejected(code:DecodeCode, offset:U64)

DecodeCode =
    00 WrongMagic
  | 01 UnknownVersion
  | 02 FrameTagOrderOrCount
  | 03 Truncated
  | 04 LengthOrCountOverflow
  | 05 InvalidFixedWidth
  | 06 UnknownSumTag
  | 07 BoundedValueUnderConsumed
  | 08 BoundedValueOverConsumed
  | 09 TrailingBytes
```

The parser walks required fields depth first in encoded order. It performs the
next grammar read only after the preceding read succeeds. A missing required
octet is `Truncated` at EOF; an available wrong magic/version/tag is reported
at its first differing/tag octet. A length/count arithmetic failure is
reported at the first octet of that field. Under-consumption is reported at
the bounded end; over-consumption and trailing input at the first extra octet.
If two grammar checks are provably false at the same cursor, the lower
`DecodeCode` tag wins. These cursor and tag rules determine one and only one
`DecodeRejected` value.

Decode rejects only failures to parse the closed byte grammar: wrong magic or
version; wrong frame tag/order/count; truncation; unsafe length/count
arithmetic; wrong fixed width; unknown Term, KSort, KExpr, lineage, evidence,
receipt-format, or evaluation-rule tag; bounded under/over-consumption;
and trailing bytes. Resource exhaustion is an implementation failure and
cannot be reported as a different canonical verdict.

Order/uniqueness, exact manifest equality, reference bounds, Core ABI and
entrypoint signatures, identity derivations, evidence/lineage consistency,
receipt replay semantics, and physical-profile conformance are
authorization checks with the exact table pairs above. Consequently malformed
strict-decode input never enters authorization, while every failure after
`Decoded` reaches exactly one table pair.

`Decoded` retains the exact input bytes with the decoded value. Re-encoding a
value that passes canonical authorization must reproduce those exact bytes.
Successful decoding returns a candidate package, never an accepted compiler:

```text
Clause-owned Application + context + already-authoritative predecessor/root
      --activate--> compiler-evolution Activation / Run
Run carries exact build inputs through compilation and `admitPropose`
      --> exact candidate package bytes
Run: bytes --strict decode--> candidate package
Run: candidate + external genesis anchor or accepted exact predecessor
      --frozen constitutional check--> Authorized(exact package bytes)
                                       | Unauthorized(stage, code)
Run --emit--> checker evidence + candidate compiler/Program delta + obligations
authorized Run output + authority
      --governed outer Admission--> authoritative compiler + successor ProgramRevision
```

The frozen checker ends at exact authorization evidence. It does not itself
admit a compiler or construct a Clause `ProgramRevision`; only the governed
outer `Admission` of a candidate delta emitted by a Clause-owned `Run` does so.
An API that promotes a package on decode, hash match, receipt production, or
successful Rust execution is nonconforming.

## CLCP v3 hashes and identities

For ASCII domain `d` and byte components `x1 ... xn`:

```text
DH(d, x1, ..., xn) =
  SHA256(
    U32(byteLength(d)) || d ||
    U64(byteLength(x1)) || x1 || ... ||
    U64(byteLength(xn)) || xn)
```

Required identities are:

```text
CoreContractId =
  DH("clause/core-contract/v1", exactCoreManifestBytes)

PhysicalProfileId =
  DH("clause/physical-profile/v1", exactPhysicalProfileBytes)

CompilerSemanticsId =
  DH("clause/compiler-semantics/v1", canonical(interface || program))

CompilerRevisionId =
  DH("clause/compiler-revision/v1", exactCompilerSubjectBytes)

CompilerPackageHash =
  DH("clause/compiler-package/v1", exactWholePackageBytes)

SourceArtifactId =
  DH("clause/source-artifact/v1", exactSourceBytes)

BuildRequestId =
  DH("clause/compiler-build-request/v1",
     canonicalTermBytes(BuildRequest))

OriginId =
  DH("clause/origin/v1", canonicalAcyclicOriginNode)
```

The domains are part of the v3 contract despite their independent `v1`
domain versions; changing one requires a new domain and an explicit migration.
Hashes establish content identity for lookup, comparison, publication, and
reproducibility. They never establish compiler authority. An accepted
predecessor locator must resolve to the already accepted exact bytes, and the
checker must compare those bytes before succession checking.

## CLCP v3 required corpus boundaries

The v3 corpus separates decode, canonicality, core well-formedness,
genesis-anchor, compile replay/receipt, admission replay/receipt, exact binding,
and final-authority verdicts. A negative
specimen falsifies its named claim and does not imply every earlier stage must
reject.

It must cover malformed, truncated, trailing, duplicate, and out-of-order
encodings; root without anchor; candidate/self-basis authorization; wrong and
stale predecessors; transplanted or detached evidence where `E != Q.evidence`;
entrypoint signature and every Core ABI shape mismatch; build-request, subject,
result, remaining-fuel, and observation alteration; a receipt that attempts to
substitute for replay; profile escape; recursively embedded predecessor bytes;
and a valid hash paired with non-identical bytes. Metamorphic positives rename
nominal IDs across a canonical sort-order boundary, recompute dependent
source/content and derived IDs as applicable, regenerate both receipts from
complete replay, and preserve the generic check/evaluation verdict.

## Implemented CLCP v1 evidence boundary

CLCP v1 has one closed binary representation carrying a generic structural
index, finite ground derivation basis, certificates, target, exact lineage
evidence, and ordered opaque auxiliary blobs. Its normative byte corpus is in
[`test-vectors/canonical-package/`](../test-vectors/canonical-package/).
The live Lean and Rust implementations reproduce that corpus. SHA-256 entries
are content-addressing evidence only, and v1 decoding or authorization is not
Clause compiler authority. `Clause Core v0` is a retired historical
implementation alias for this CLCP-v1 package boundary, not a current Clause
semantic or constitutional namespace. Existing symbols may retain the alias
until their consumers migrate; `v0` names that bootstrap implementation, while
the wire version octet is `01`.

The remainder of this document freezes the implemented v1 representation until
its consumers are migrated and removed in a later implementation phase.

### Primitive encodings

`U8` is one unsigned octet. `U32` is an unsigned 32-bit integer in big-endian
byte order. All tags are `U8`. All byte lengths and list counts are `U32`.

The notation used in this document is:

```text
Blob        = length:U32 bytes:U8[length]
List<X>     = count:U32 item:X[count]
Frame(t, X) = tag:U8 (= t) length:U32 payload:X
```

Each frame is decoded through a cursor bounded to exactly `length` bytes. Its
payload decoder must consume that entire cursor. A `Blob` may be empty. Lists
are ordered, retain duplicates unless a rule below forbids them, and have no
map interpretation.

Before advancing any cursor, a decoder must prove that the requested length or
element is within the enclosing cursor. Offset, length, count, allocation-size,
and host-integer conversions use checked arithmetic. In particular, a decoder
must not compute an unchecked `offset + length`; it first compares `length` to
the remaining byte count. A count is not permission to allocate or iterate
past the bytes that remain. Any overflow, unrepresentable host size, or request
beyond the enclosing cursor is rejection.

### Package envelope

Every package is exactly:

```text
43 4c 43 50                 magic ASCII "CLCP"
01                          version
Frame(01, INDEX)
Frame(02, LINEAGE)
Frame(03, BASIS)
Frame(04, CERTIFICATE)
Frame(05, TARGET)
Frame(06, AUXILIARY)
EOF
```

The six frames occur once in that order. A missing, repeated, reordered, or
unknown frame is invalid. Bytes after the `AUXILIARY` frame are invalid. Version
1 has no flags and no reserved fields.

### Structural index

```text
INDEX = universeId:Blob semanticsId:Blob
```

Both byte strings are opaque Clause identifiers. Empty identifiers are
syntactically representable; whether an admitted basis permits them is not a
decoding decision. `semanticsId` is the semantics epoch. Equality of structural
indexes is exact equality of both blobs.

### Terms and claims

Terms are recursively inline. There is no term table, reference form, implicit
sharing, or identity attached to a triple.

```text
Term = 00 kind:Blob canonicalPayload:Blob equalityContract:Blob
     | 01 first:Term second:Term third:Term

Claim = term:Term typeTerm:Term mode:Term
```

`00` is the Atom tag and `01` is the neutral triple tag. No other Term tag is
valid. Atom fields are opaque bytes. The three children of a triple and the
three fields of a claim are ordered and exact. Constructing or decoding either
form asserts nothing.

### Basis

```text
BASIS = roots:List<Claim> rules:List<GroundRule>

GroundRule = premises:List<Claim> conclusion:Claim
```

Roots and rules are package-local ordered lists. Rule premises are ordered.
List position is a certificate address only; it is not Clause nominal identity
or source order.

### Certificates

```text
CERTIFICATE = nodes:List<CertificateNode>

CertificateNode = claimed:Claim reason:CertificateReason

CertificateReason = 00 rootRef:U32
                  | 01 ruleRef:U32 premiseRefs:List<U32>
```

Reason tag `00` selects a root; reason tag `01` applies a ground rule. No other
reason tag is valid. References are zero-based package-local list positions.

A certificate is checked in encoded node order with an initially empty list of
prior claims:

1. A root node succeeds only when `rootRef` exists in the selected basis and
   `claimed` is byte-structurally equal to that root's decoded claim.
2. A rule-application certificate node (historically, an "application node";
   not a Clause semantic `Application`) succeeds only when `ruleRef` exists,
   the number of `premiseRefs` equals the selected rule's premise count, every
   premise reference is distinct, every reference addresses a strictly
   earlier node, each referenced claim equals the corresponding ordered rule
   premise, and `claimed` equals the rule conclusion.
3. Checking a certificate against a requested claim succeeds only when every
   node succeeds, the node list is nonempty, and the last claimed value equals
   the requested claim.

All equality in these checks is exact decoded representation equality at the
already-equal structural index. Certificate checking proves derivability only
relative to the basis explicitly selected by its caller. It does not authorize
that basis.

### Target and auxiliary content

```text
TARGET    = Claim
AUXILIARY = List<Blob>
```

The package certificate is checked under the package's own basis against the
exact target. Auxiliary blobs are ordered opaque bytes reserved for later
separately typed sections. In v0 they have no interpretation and provide no
authority or admission evidence. Their order and bytes remain part of the
exact package record.

### Lineage

```text
LINEAGE = 00
        | 01 predecessorPackage:Blob authorization:CERTIFICATE
```

Tag `00` is a root and has no payload after the tag. Tag `01` is a successor.
Its blob contains a complete exact predecessor package, including that
package's magic, version, six frames, and exact EOF at the blob boundary. It is
not a digest, projection, or re-encoding. The remaining bytes of the LINEAGE
frame are one authorization certificate.

The only authorized root in v0 is exact byte equality with
[`positive/bootstrap.hex`](../test-vectors/canonical-package/positive/bootstrap.hex)
after the hex transport is decoded to bytes. A syntactically valid package
with root tag `00` is not thereby a root of authority.

For a successor, let `nextIndexFrame` and `nextBasisFrame` be the exact encoded
frame bytes, including tag and U32 payload length, found in that successor.
Define the canonical basis-admission claim as:

```text
BasisAdmission(nextIndexFrame, nextBasisFrame) =
  Claim(
    Atom(kind=f0,
         canonicalPayload=nextIndexFrame || nextBasisFrame,
         equalityContract=f1),
    Atom(kind=f0, canonicalPayload=f2, equalityContract=f1),
    Atom(kind=f0, canonicalPayload=f3, equalityContract=f1))
```

All displayed constants are one-byte blobs. `||` is byte concatenation. This
commitment is injective: the first self-delimiting frame determines the exact
INDEX frame and all remaining payload bytes determine the exact BASIS frame.
No digest participates.

Successor authorization performs these steps in order:

1. Decode and authorize the exact embedded predecessor bytes recursively.
2. Require the successor's `universeId` to equal the predecessor's exactly.
3. Require the successor's `semanticsId` to equal the predecessor's exactly.
   Clause v0 therefore has no implicit index or epoch migration.
4. Build `BasisAdmission` from the successor's exact INDEX and BASIS frame
   bytes.
5. Check the LINEAGE authorization certificate only under the already
   authorized predecessor's basis and against that exact admission claim.
6. Independently check the package CERTIFICATE under the successor's own basis
   and against its TARGET.

The successor basis never authorizes its own admission. An implementation must
not fall back to it when predecessor-basis checking fails. Successful decoding,
relative certificate checking, or possession of predecessor-like bytes alone
never grants authority.

### Canonical decoding and exact binding

There is one representation for every decodable package value. A conforming
decoder rejects:

- wrong magic or version;
- a frame tag other than the required tag at its fixed position;
- an unknown Lineage, Term, or CertificateReason tag;
- an out-of-bounds or arithmetically unsafe length, count, or reference;
- an under-consumed or over-consumed frame;
- a truncated primitive, blob, term, list, nested package, or frame; and
- any bytes after the sixth frame.

The decoder returns the exact input bytes together with every decoded field.
Exact binding to a selected package means equality of those bytes and the full
decoded record, including index, lineage, basis, certificate, target, and
auxiliary blobs. Implementations may cache a digest for physical lookup, but
must resolve it back to and compare the exact bytes before making an exact-
binding claim.

Decoding is deliberately separate from authorization:

```text
bytes --decode--> candidate package
candidate + selected literal/predecessor authority --authorize--> verdict
```

An API that returns an authorized package directly from successful decoding is
nonconforming.

### Frozen CLCP-v1 evidence specimens

For compact notation, let:

```text
A(k,p,e) = Atom(kind=k, canonicalPayload=p, equalityContract=e)
J(t,y,m) = Claim(term=t, typeTerm=y, mode=m)

B = J(A(20,40,30), A(21,41,31), A(22,42,32))
S = J(A(20,50,30), A(21,41,31), A(22,42,32))
```

Every argument above is a one-byte blob. These are generic opaque identifiers
and payloads, not host or feature names.

The literal bootstrap has index `(universeId=10, semanticsId=11)`, root
lineage, ordered roots `[B, BasisAdmission(successor INDEX, successor BASIS)]`,
no rules, the one-node certificate `[(B, root 0)]`, target `B`, and no auxiliary
blobs. Its package certificate therefore proves its exact target from the
bootstrap root. Exact literal bytes, not this prose and not a digest, select it
as the root of authority.

The positive successor has the exact same index, embeds the exact bootstrap
bytes, has the one-node lineage certificate
`[(BasisAdmission(successor INDEX, successor BASIS), root 1)]`, basis roots
`[S]`, no rules, package certificate `[(S, root 0)]`, target `S`, and no
auxiliary blobs. Its lineage certificate succeeds under the predecessor basis.
The same certificate fails under the successor basis because that basis has no
root at index 1. This is the frozen predecessor-only authorization witness.

### Corpus verdict conventions

[`manifest.json`](../test-vectors/canonical-package/manifest.json) records
decode, exact-binding, certificate, and authority outcomes separately. A file
under `negative/` falsifies the single named claim in its manifest entry; it
does not imply that every other stage must reject.

In particular, `auxiliary-tamper.hex` is canonically decodable and remains an
authorized successor because v0 deliberately gives auxiliary bytes no
authority role. It fails exact binding to the frozen positive successor. This
distinction prevents a decoder from pretending to authorize while preserving
the v0 decision that opaque auxiliary content is not predecessor-admitted
semantic evidence.

The manifest's failure labels identify the first decisive stage in the order
specified above. Implementations need not expose those strings as public error
messages; they must reproduce the recorded Boolean verdicts and decoded fields.
