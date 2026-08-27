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

The package may model only the host-neutral calculus owned by
`docs/foundation.md`. Lean syntax, expressions, type classes, serialization,
and one-constructor-per-language-feature inductives are implementation tools,
not Clause authority.

The package does not yet implement Atom-contract admission, semantic structural
equality, identity judgments, Context, Judgment, Mode, Run, Trace, Delta,
Admission, Revision, a canonical codec, or Clause source. Its examples prove
only candidate representation distinctions; they do not pass the adoption
spike's later integer-evaluation gate.

`lake build` compiles with `-t0` and warnings as errors, then runs
`ClauseCoreTrust.lean`. That audit rejects every unsafe declaration, every
partial declaration except the exact compiler-generated runtime helper for
`Term.sameRepresentation`, every foreign or replacement implementation, and
every axiom except `propext`, which Lean uses in generated
dependent-constructor injectivity support. The same-kernel replay checks the
safe/total declarations and is an additional check:

```sh
lake build
lake env leanchecker --fresh ClauseCore
```

`leanchecker` excludes the enumerated partial runtime helper. Neither check is
an independent verifier or the full package-bound trust closure required by
the adoption spike.
