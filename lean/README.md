# Clause Core Lean bootstrap model

This package contains Clause's smallest provisional representation:

```text
Atom(index, kind, canonical payload, equality-contract reference)
Term(index) = Atom | [Term, Term, Term]
```

The index contains an explicit universe and Clause semantics epoch. Candidate
representation comparison is recursive and index-bound. It becomes semantic
structural equality only after a later Clause judgment validates the Atom kind,
payload canonicalization, and equality contract. Triples contain no identity
field; nominal identity remains an ordinary Atom role granted by judgment.

`JudgmentClaim`, `ContextCandidate`, and `ClauseJudgmentCandidate` remain raw
candidate carriers. `GroundRuleCandidate`, `DerivationBasisCandidate`, and
`DerivationCertificate` implement finite one-pass ground certificates using
only addressed roots and generic rule application. `checkRelative_sound`
connects executable success to `DerivableFrom` the exact supplied basis.
Context membership, basis construction, and bare relative derivability grant no
authority.

## Canonical CLCP v1 package

`Codec` independently implements the grammar owned by
`clause:docs/canonical-package.md`:

```text
ASCII "CLCP" | version 01
INDEX | LINEAGE | BASIS | CERTIFICATE | TARGET | AUXILIARY | EOF
```

Tags are U8; every length and count is U32 big-endian. Frame payloads and EOF
must be consumed exactly. Terms retain only Atom and neutral recursive Triple.
Lineage is either root or exact length-delimited predecessor package bytes plus
an authorization certificate. Auxiliary blobs are ordered and opaque.

The decoder bounds every read, uses unbounded natural arithmetic for decoded
U32 values, rejects encoder overflow, and rejects wrong tags, order, lengths,
counts, truncation, frame residue, trailing bytes, malformed recursively
embedded predecessors, and noncanonical re-encoding. A successful result
preserves the exact raw input.
`decodePackage_canonical_binding` proves that every successful result both
retains those bytes and re-encodes every dependent field to the same bytes.

## Literal authority

`Constitution.bootstrapBytes` is the exact 334-byte bootstrap literal and
`Constitution.successorBytes` is the exact 681-byte successor from
`clause:test-vectors/canonical-package/`. The bootstrap certificate proves its
target from root zero. Its second root preauthorizes one exact successor BASIS.

The canonical basis-admission claim is an ordinary Clause claim. Its term Atom
payload is the exact successor INDEX frame followed by the exact successor
BASIS frame; the self-delimiting INDEX frame makes the commitment injective.
There is no digest, host callback, semantic-equality assertion, or admission
field.

`AuthoritativePackage` has exactly two constructors: the literal bootstrap and
a predecessor-authorized successor. A successor requires strict canonical
binding, the exact recursively decoded predecessor bytes, the same literal v0
index, lineage-certificate checking only under the predecessor basis against
the canonical next basis-admission claim, and separate checking of its packaged
certificate under its own basis and target. The narrow soundness theorem
concludes only `DerivableFrom` the authoritative package's exact basis.

`clause:lean/ClauseCoreVectors.lean` exercises the exact positive corpus, malformed decoder
inputs, every bound package field, bytes/value mismatch, nonliteral roots,
wrong and transplanted predecessors, self-declared successor-basis checks,
cross-index attempts, the exact successor-lineage nullary-rule specimen, raw
Context membership, bare relative derivability, and self/cycle attempts. It
also round-trips an ordinary package with a Triple, a rule application, and
ordered auxiliary blobs. Opaque
auxiliary-only mutation breaks exact positive binding but correctly does not
change v0 authority.

This tranche still does not define Atom-contract admission, semantic structural
equality, valid Clause judgments, identity judgments, Mode, Run, Trace, Delta,
general Admission, Revision, Clause source, or any language feature. Package
authority and relative derivability are not semantic truth.

## CLCP v3 compiler constitution

`ClauseCompiler` implements the construct-blind CLCP v3 compiler carrier. Its
strict decoder accepts only version `03`; Frame 03 successor evidence contains
exactly a compile receipt and admission receipt. Each `EvalReceipt` contains
only format version `00` and one expected `EvalOutcome`.

Evaluation requests are never encoded. The checker constructs the predecessor
package hash, carried core/profile IDs, entrypoint, arguments, and fuel from the
separately supplied exact accepted predecessor and candidate build request. It
then completely evaluates the call and compares returned value, remaining fuel,
and canonical observations exactly. Admission uses the verified actual compile
observations. No node graph, trace, callback, or receipt-produced assertion can
grant authority.

## Checks

`lake build` uses `-t0` and warnings as errors, builds the core, executable
vectors, CLCP v3 compiler checker, and both trust gates. The audits reject every
unlisted unsafe or partial declaration, foreign or replacement implementation,
and unapproved axiom in their exact reachable closures.

```sh
lake build
lake env leanchecker --fresh ClauseCore
```

`leanchecker` is a same-kernel replay, not an independent verifier. Neither
check establishes the still-missing semantic calculus; Rust parity is a
separate pinned-toolchain corpus gate.
