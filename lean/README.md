# Clause Core Lean bootstrap model

This package contains a provisional model of Clause's smallest representation:

```text
Atom(index, kind, canonical payload, equality-contract reference)
Term(index) = Atom | [Term, Term, Term]
```

The index contains an explicit universe and Clause semantics epoch. Candidate
representation comparison is recursive and index-bound. It becomes semantic
structural equality only after a future Clause judgment validates the Atom kind,
payload canonicalization, and equality contract. Triples contain no identity
field; nominal identity is deferred until a judgment can grant that role to an
ordinary Atom Term.

The model also contains index-bound `JudgmentClaim`, `ContextCandidate`, and
`ClauseJudgmentCandidate` carriers. A claim's semantic leaves remain Terms; a
candidate Context enumerates claims, and a proposed judgment pairs one such
Context with one claim. Their only lookup operation compares exact candidate
representations. Constructing a claim or finding it in a candidate Context does
not establish validity, truth, authority, or admission.

`GroundRuleCandidate`, `DerivationBasisCandidate`, and
`DerivationCertificate` add a finite, one-pass ground-certificate layer. The
checker knows only addressed basis roots and generic rule application. Every
support reference must resolve in the already checked prefix, so malformed,
self-referential, forward, and cyclic traces reject while acyclic DAG sharing
works. `checkRelative_sound` connects executable acceptance to the independent
`DerivableFrom` proposition. Both are explicitly relative to the supplied
basis; the package does not select or accept that basis, and candidate Context
membership never enters the checker.

`CanonicalPackageCandidate` groups exact canonical-byte candidates, the
complete structural index, and decoded sections containing the basis,
certificate, requested target, and opaque auxiliary content. Exact binding is
whole-record equality, not a digest or a reconstructed projection.
`ConstitutionalPackageAnchor` is a closed external predicate with no current
constructor: candidate decoded data contains no proof or admission field and
cannot select an authority. `checkExactPackage_sound` composes an exact binding,
that external anchor, and `checkRelative_sound` into only
`PackageBoundDerivable`. It does not establish semantic truth or general
Admission. The examples show that independent byte, epoch, section, basis,
rule, certificate, and target changes break binding, and that self-declared
roots, nullary self-rules, Context membership, and bare relative derivability
remain unauthorized.

The package may model only the host-neutral calculus owned by
`docs/foundation.md`. Lean syntax, expressions, type classes, serialization,
and one-constructor-per-language-feature inductives are implementation tools,
not Clause authority.

The package does not yet implement a literal constitutional anchor, successor-
basis admission, canonical decoding or a codec, schematic rule formation or
substitution, Atom-contract admission, semantic structural equality, valid
Clause judgments, identity judgments, Mode, Run, Trace, Delta, Admission,
Revision, or Clause source. Its examples prove candidate-representation and
exact-binding boundaries, relative ground derivability, finite-trace rejection,
and raw-Context non-authority; they do not pass the adoption spike's later
integer-evaluation gate.

`lake build` compiles with `-t0` and warnings as errors, then runs
`ClauseCoreTrust.lean`. That audit rejects every unsafe declaration, every
partial declaration except the exact compiler-generated runtime helpers for
`Term.sameRepresentation` and the finite premise-reference matcher, every
foreign or replacement implementation, and every axiom except `propext`, which
Lean uses in generated dependent-constructor injectivity support. The
same-kernel replay checks the safe/total declarations and is an additional
check:

```sh
lake build
lake env leanchecker --fresh ClauseCore
```

`leanchecker` excludes the two enumerated partial runtime helpers. Neither check is
an independent verifier or the full package-bound trust closure required by
the adoption spike.
