# Clause Canonical Packages

> **Status:** CLCP v2 is the normative P1 compiler-package contract and is not
> implemented. The CLCP v1 codec and corpus remain implemented narrow
> bootstrap evidence pending their P2 migration.
>
> **Authority:** This document owns canonical package representation and
> canonical decoding. [Compiler genesis](compiler-genesis.md) owns compiler
> authority and succession, the [foundation](foundation.md) owns Clause
> meaning, and the [roadmap](roadmap.md) owns implementation status. Decoding,
> hashing, materialization, or derivability never grants meaning or authority.

## Version boundary

CLCP v1 and CLCP v2 are distinct closed formats. A decoder selects a format
only from the exact version octet after the `CLCP` magic and rejects all
other versions. There is no permissive common envelope, fallback decoder,
version inference, extension field, or alternate encoding.

CLCP v2 is the compiler carrier required by the
[genesis contract](compiler-genesis.md). It separates the compiler subject
from checking evidence so a certificate never contains, hashes, or authorizes
itself. The existing CLCP v1 implementation and byte corpus prove only their
narrow finite-ground-certificate boundary. They are not `Compiler0`, a v2
decoder, a universal evaluator, or evidence that the v2 contract is
implemented.

## CLCP v2 primitive encodings

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
an `Id32` unless a different type is written explicitly.

## CLCP v2 envelope

Every compiler package is exactly:

```text
43 4c 43 50                 magic ASCII "CLCP"
02                          version
Frame(01, coreContractId:Hash32)
Frame(02, CompilerSubject)
Frame(03, CompilerEvidence)
EOF
```

All three frames occur exactly once and in that order. Unknown, missing,
repeated, or reordered frames reject. A core contract identifier selects one
fixed, canonical set of `KSort`, `KExpr`, core well-formedness,
certificate-rule, and physical-profile rules. Package data cannot add a rule
or reinterpret a tag.

## CLCP v2 Terms and evaluator expressions

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
  | 08 Call(definitionId:Id32, arguments:Seq<KExpr>)
  | 09 Request(physicalOperationId:Id32, arguments:Seq<KExpr>)
```

Term and expression recursion is inline and finite because every nested value
must be consumed from the exact bounded input. Resource exhaustion is a
physical failure, not a different canonical decoding verdict.

## CLCP v2 compiler subject

```text
CompilerSubject =
  lineage:CompilerLineage
  interface:CompilerInterface
  program:Seq<Definition>
  buildBundle:BuildBundle

CompilerLineage =
    00 Genesis
  | 01 Successor(
         predecessorLocator:Hash32,
         changeOccurrenceId:Id32)

CompilerInterface =
  compile:Id32
  admitPropose:Id32

Definition =
  id:Id32
  arguments:Seq<KSort>
  result:KSort
  body:KExpr

BuildBundle =
  sourceUnits:Seq<SourceUnit>
  baseInputs:Term
  identityRetentions:Term
  changeOccurrenceId:Id32
  options:Term

SourceUnit =
  unitId:Id32
  artifactId:Id32
  bytes:Blob
```

`program` is sorted by `Definition.id`, and
`buildBundle.sourceUnits` is sorted by `SourceUnit.unitId`; duplicate IDs
reject. Both interface IDs must resolve exactly once in `program`.
`SourceUnit.artifactId` must equal the required domain hash of its exact
`bytes`. A Genesis subject's build-bundle change occurrence is its literal
genesis ID. A Successor subject's lineage and build-bundle change occurrence
must be equal.

The exact Frame 02 payload is `exactCompilerSubjectBytes`. The interface and
program are executable compiler meaning. The build bundle is exact
reproducibility input carried inside the subject, not a second executable
authority.

## CLCP v2 evidence

```text
CompilerEvidence =
    00 GenesisEvidence
  | 01 SuccessorEvidence(
         compileCertificate:EvalCertificate,
         admissionCertificate:EvalCertificate)

EvalCertificate = certificateBytes:Blob
```

`GenesisEvidence` has no payload. Each certificate blob has exactly one
canonical decoding under the package's fixed `coreContractId`: a
topologically ordered derivation DAG containing only fixed core
well-formedness and generic evaluator-step rule tags. Nodes refer only to
earlier premises and carry their exact generic conclusion. Unknown rules,
forward references, unused trailing bytes, alternate DAG encodings, and
package-supplied checker rules reject.

Frame 03 is excluded from `exactCompilerSubjectBytes`,
`CompilerSemanticsId`, and `CompilerRevisionId`. Predecessor compilation
and admission target the exact subject bytes, then a generic packager attaches
the two certificates without modifying the subject. This prevents evidence
from containing or hashing itself.

No v2 package contains its own package hash, subject ID, or revision ID.
`CompilerPackageHash` covers the final whole package, including evidence, and
therefore is computed externally after canonical packaging.

## CLCP v2 canonical decoding

A conforming decoder rejects:

- wrong magic, unknown version, wrong frame order or count, and trailing bytes;
- an unknown Term, KSort, KExpr, lineage, evidence, or certificate-rule tag;
- an unsafe or out-of-bounds length, count, reference, or nesting operation;
- under-consumed or over-consumed frames and nested values;
- duplicate or out-of-order definitions or source units;
- an unresolved or multiply defined compiler entrypoint;
- mismatched source-artifact or change-occurrence identities;
- malformed certificate DAGs or evidence inconsistent with lineage; and
- a physical request outside the core contract's declared operation set.

The decoder returns the exact input bytes with the decoded value. Re-encoding
that value must reproduce the exact input bytes. Successful decoding returns a
candidate package, never an accepted compiler:

```text
bytes --strict decode--> candidate package
candidate + external genesis anchor or accepted exact predecessor
      --constitutional check--> accepted compiler
```

An API that promotes a package on decode, hash match, certificate validity, or
successful Rust execution is nonconforming.

## CLCP v2 hashes and identities

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
  DH("clause/core-contract/v1", canonicalCoreContractBytes)

CompilerSemanticsId =
  DH("clause/compiler-semantics/v1", canonical(interface || program))

CompilerRevisionId =
  DH("clause/compiler-revision/v1", exactCompilerSubjectBytes)

CompilerPackageHash =
  DH("clause/compiler-package/v1", exactWholePackageBytes)

SourceArtifactId =
  DH("clause/source-artifact/v1", exactSourceBytes)

BuildRequestId =
  DH("clause/compiler-build-request/v1", canonicalBuildRequest)

OriginId =
  DH("clause/origin/v1", canonicalAcyclicOriginNode)
```

The domains are part of the v2 contract despite their independent `v1`
domain versions; changing one requires a new domain and an explicit migration.
Hashes establish content identity for lookup, comparison, publication, and
reproducibility. They never establish compiler authority. An accepted
predecessor locator must resolve to the already accepted exact bytes, and the
checker must compare those bytes before succession checking.

## CLCP v2 required corpus boundaries

The future v2 corpus must separate decode, canonicality, core
well-formedness, genesis-anchor, compile-certificate,
admission-certificate, exact-binding, and final-authority verdicts. A negative
specimen falsifies its named claim and does not imply every earlier stage must
reject.

It must cover malformed, truncated, trailing, duplicate, and out-of-order
encodings; root without anchor; candidate/self-basis authorization; wrong and
stale predecessors; transplanted evidence; build-request and subject
alteration; profile escape; and a valid hash paired with non-identical bytes.
No such corpus or implementation is present in P1.

## Implemented CLCP v1 evidence boundary

CLCP v1 has one closed binary representation carrying a generic structural
index, finite ground derivation basis, certificates, target, exact lineage
evidence, and ordered opaque auxiliary blobs. Its normative byte corpus is in
[`test-vectors/canonical-package/`](../test-vectors/canonical-package/).
The live Lean and Rust implementations reproduce that corpus. SHA-256 entries
are content-addressing evidence only, and v1 decoding or authorization is not
Clause compiler authority. Existing prose and code also call this the Clause
Core v0 package; `v0` names that semantic bootstrap, while the wire version
octet is `01`.

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
2. An application node succeeds only when `ruleRef` exists, the number of
   `premiseRefs` equals the selected rule's premise count, every premise
   reference is distinct, every reference addresses a strictly earlier node,
   each referenced claim equals the corresponding ordered rule premise, and
   `claimed` equals the rule conclusion.
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

### Frozen constitutional specimens

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
