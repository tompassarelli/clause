# Clause Constitutional Design Evidence

> **Status:** Evidence and uncertainty ledger for the
> [foundation](foundation.md), [architecture](architecture.md), and
> [adoption spike](adoption-spike.md).
>
> **Authority:** Informative. This document cannot change semantics, syntax, or
> roadmap status.

## Decision and boundary

Clause provisionally adopts this implementation split:

```text
Clause Core       = host-neutral semantic contract
Lean 4            = constitutional checker and reference Run semantics
Rust              = physical persistence, runtime, FFI, and backends
Clause            = eventual author of the compiler middle
```

The split exists to keep sophisticated parsers, macros, agents, elaborators,
optimizers, and targets in the role of proposal producers while a smaller
boundary decides whether proposed meaning is admissible.

The implementation packages begin semantic-empty and take only the current
Clause calculus as authority. This removes the strongest path by which
bootstrap convenience could become Clause ontology.

## Equality bootstrap order

Semantic Term equality cannot honestly precede generic Clause judgment
checking. Atom equality contracts are declarative Clause data, so raw presence
of a contract Term or validity claim in a candidate Context cannot authorize
itself. Conversely, a generic checker can use exact IDs and candidate
representation comparison for lookup and certificate binding without claiming
semantic equality. The dependency is therefore stratified rather than cyclic:

```text
candidate Term representation
  -> generic Context/Judgment certificate checking
  -> admitted Atom contract and canonical-payload evidence
  -> Atom semantic equality
  -> recursive Term semantic equality
```

The first Context/Judgment carrier remains candidate data only. It introduces
no `Term.semanticEq`, semantic `DecidableEq`, contract callback, function-valued
proof field, opcode taxonomy, quotient, or raw-membership admission rule.

The required negative specimen for the next checker is a candidate Context
containing an equality contract that claims all payloads equal, plus raw claims
that it is total, deterministic, and canonical, but no independent valid
derivation. The checker must reject that candidate, produce no admitted
contract or semantic-equality result, and leave distinct payload
representations unequal. A design that accepts the specimen makes Context
membership self-authorizing and is rejected.

At pinned Lean 4.33.1, `DecidableEq` and `LawfulBEq` establish Lean
propositional equality, not a separately governed Clause relation. Quotienting
would add `Quot.sound`, collapse candidate representations before Clause
chooses to, and violate the current axiom policy. These mechanisms remain
implementation prior art, not Clause semantics. No Lean source was copied or
adapted for this decision.

## Lean 4 source evidence

The bootstrap pins Lean `v4.33.1`, whose upstream source tag resolves to
`819816b2e0a3bf405af45ae5c7af2491d8f5bee6` and is licensed Apache-2.0.
Constitutional kernel and trust-boundary behavior was inspected in the newer
[`leanprover/lean4`](https://github.com/leanprover/lean4/tree/342db4dbdb3aab611e0b92ddba0c134c9b28b2f9)
revision `342db4dbdb3aab611e0b92ddba0c134c9b28b2f9`, licensed Apache-2.0. The
newer checkout is evidence, not the pinned compiler binary. No Lean source was
copied, adapted, or vendored into Clause.

The inspection supports a bounded conclusion: Lean can host a small checked
model of Clause, but its kernel does not natively understand Clause Terms or
graphs.

- Lean's kernel checks Lean expressions and declarations. Clause therefore
  needs an explicit object-language model, decoder, certificate proposition,
  and theorem connecting certificate acceptance to Clause validity.
- Lean distinguishes safe, partial, and unsafe declarations. The constitutional
  closure can remain safe and total while Clause partiality, streaming,
  reactivity, and effects remain explicit object-language relations.
- Lean exposes skipped checking, asynchronous preliminary environments,
  elaboration recovery, and `sorry` paths. Clause admission must reject all of
  them and wait for checked declarations.
- `trustLevel = 0` checks newly added declarations but does not recheck imported
  bodies. Exact source/toolchain/artifact identity and replay of the complete
  reachable safe/total closure are therefore required.
- `implemented_by`, foreign implementations, native reduction, and
  `native_decide` can rely on compiled execution not proved equivalent by the
  kernel. They are excluded from constitutional evidence.
- Axiom collection can expose the proof closure, but Clause still needs an
  explicit policy for logical axioms rather than inheriting one accidentally.
- `leanchecker` replays declarations through Lean's kernel and is useful, but it
  is not an independent verifier and skips unsafe/partial constants. Their
  absence is checked separately.

This evidence selects Lean for the first semantic implementation tranche. The
[adoption spike](adoption-spike.md) must still measure the trusted boundary,
proof ceremony, compilation feedback, codec boundary, and Rust parity.

## Aeneas boundary

The exact upstream Aeneas source inspected is
[`AeneasVerif/aeneas`](https://github.com/AeneasVerif/aeneas/tree/9467a32f98437dd2812fc693fd475827775f5186)
revision `9467a32f98437dd2812fc693fd475827775f5186`, licensed Apache-2.0. That
revision requires Charon revision
`2881d1238bcb1f2f30a62f07018da1e397bcb181`. No Aeneas or Charon source was
copied, adapted, vendored, built, or added as a dependency.

Aeneas translates a supported Rust subset through Charon and its own
intermediate representation into proof-assistant code, including Lean. That may
later help verify isolated safe-Rust helpers in the physical substrate.

It is not part of Clause Core, admission, or the initial trust chain. Its closed
translation IR, opaque-external assumptions, unsupported unsafe/concurrent Rust,
and extra tool/version boundary cannot establish Clause meaning. It is optional
verification prior art only.

## Alternatives rejected for the bootstrap

### Rust as semantic home

Rust is excellent for physical correctness and systems integration. Its closed
enums, traits, ownership boundaries, arenas, and representation-oriented APIs
also make it easy for host types to become the actual language. Rust therefore
owns physical execution but not Clause's constitutional semantics.

### OCaml compiler layer

OCaml is an excellent conventional compiler language. Adding it between Lean,
Rust, and eventual Clause self-hosting would introduce another codec, build
system, host taxonomy, and potential semantic authority without a unique
required capability. It remains a fallback only if the Lean spike specifically
falsifies Lean's suitability.

### Lean for the complete system

Lean's unique value is precise checked meaning. Making it own the durable store,
hot incremental engine, operating-system integration, production runtime, and
all targets would unnecessarily bind Clause to Lean's runtime and FFI choices.
Those remain Rust's replaceable physical domain.

### Immediate self-hosting

Clause should eventually author its compiler middle, but it cannot honestly
bootstrap its own checker before a smaller independent boundary exists.

## Prior art

Individual ingredients have established precedent:

- abstract syntax graphs and hierarchical graph representations show that
  binding and sharing need not be reconstructed from a sovereign AST:
  <https://arxiv.org/pdf/2102.02363>;
- scope graphs make name resolution explicit as paths and relations:
  <https://eelcovisser.org/publications/2015/NeronTVW15.pdf>;
- W3C n-ary relation patterns show relation-instance and role-edge encodings:
  <https://www.w3.org/TR/swbp-n-aryRelations/>;
- Sea of Nodes shows that graph-shaped compiler IR can specialize to efficient
  machine execution:
  <https://assets.ctfassets.net/oxjq45e8ilak/12JQgkvXnnXcPoAGoxB6le/5481932e755600401d607e20345d81d4/100752_1543361625_Cliff_Click_The_Sea_of_Nodes_and_the_HotSpot_JIT.pdf>;
  and
- e-graphs show one graph-based technique for explicit equivalence and
  optimization search: <https://arxiv.org/abs/2004.03082>.

These sources support ingredients, not the Clause composition. The papers were
not independently reproduced.

## Remaining uncertainty

The following remain unproved:

- that the minimal calculus covers every dangerous general-purpose language
  semantic without hidden host categories;
- that the Lean object model and certificate bridge remain smaller and clearer
  than the boundary they protect;
- that a sufficiently narrow axiom policy and reproducible imported-module
  closure are practical;
- that structural equality, explicit identity, canonical reload, and cyclic
  references remain comprehensible and efficient at scale;
- that Clause-authored schemas and macros avoid ontology ceremony;
- that canonical source stays clearer than conventional languages on real
  programs;
- that incremental dependency closure stays local on large graphs;
- that generic meaning specializes competitively to native, Wasm, JavaScript,
  browsers, and databases;
- that ownership, concurrency, packages, FFI, ABI, and deployment compose
  without a second authority; and
- that correct-change throughput and maintenance cost improve in practice.

The bounded spike is the next honest evidence. More architecture prose cannot
resolve these uncertainties.
