# Process-First Kernel Design Evidence

> **Status:** Evidence and uncertainty ledger for the
> [semantic foundation](foundation.md) and
> [adoption spike](adoption-spike.md).
>
> **Authority:** Informative. This document records observations, alternatives,
> and unknowns; it cannot change semantics, syntax, or roadmap status.

## Decision and boundary

Clause has accepted, provisionally and subject to the adoption spike, this
mechanism:

```text
RawTriple = [Term, Term, Term]
Term      = Atom | RawTriple
Clause    = contextual typed judgment over a Term
Run       = universal stepwise dynamic relation
Admission = authoritative successor boundary
```

The important correction is process-first rather than Triple-first. A Triple is
the smallest recursively holdable compound representation, not the living act
of computation. A trace Term describes a Run but is not the Run. Structural
Term identity is not occurrence, binder, entity, lineage, or revision identity.

Clause owns this entire semantic and persistence boundary. No older project,
external store, graph database, or shared serialization is a Clause dependency
or co-authority. Similar historical experiments may have suggested questions,
but no foreign product boundary is imported into the design.

The selected implementation experiment keeps that authority host-neutral:

```text
Clause foundation     = semantic authority
Clause Core contract  = its host-neutral canonical package and rules
Lean 4                = constitutional checker and reference Run model
Rust                  = physical persistence, runtime, FFI, and backends
Clause                = eventual author of the expanding compiler middle
```

OCaml is not a primary implementation layer. This split is a bootstrap decision
subject to the spike, not a claim that Lean's type theory or Rust's data model
is Clause semantics.

## Observed implementation baseline

The exact Clause baseline inspected for this decision is:

```text
4aea6c898f3eec2fe4058d578f491eec008d7f9a
```

At that object the repository provides executable evidence for:

- stable semantic identities and duplicate-preserving occurrences;
- typed unary, binary, and n-ary named-role relational content;
- recursive terms, pure definitions, and bounded evaluation;
- recursive derivation with exact independent supports;
- query cardinality, explanation, semantic diff, and intervention;
- Program snapshot/history types across a legacy Revision-v6 bridge;
- RuntimeSession and causal StateRevision identity;
- deterministic replay and exact deltas;
- distinct effect intent, authorization, attempt, receipt, and observation
  records;
- source-deleted generated Rust;
- bounded generated JavaScript and render-plan parity; and
- a bounded real-browser/Three.js execution checkpoint.

These are valuable parity oracles. They do not prove the accepted mechanism.

The same baseline still has:

- a conventional host-owned frontend AST;
- an irreducible n-ary `RelationalContent` kernel value;
- Rust-owned semantic variants and construct-specific stages;
- no universal Term/Clause/Run interface;
- no graph-homoiconic typed macro system; and
- no host-freeze extension proof.

The current representation is therefore neither dismissed nor declared
constitutional. Its behavior remains protected until a new semantics epoch
proves exact selected parity.

## Why the design changed

The first three-slot proposal assigned a nominal identity to every Clause and
made the Clause the universal semantic substance. Adversarial review exposed
four problems:

1. structural content, occurrence, entity, and revision identity were being
   routed through one mandatory address even when their equality laws differ;
2. the Triple node, its typed expression, its evaluated value, and its Run
   occurrence could slide into one another;
3. admission was described as mutation while the activity producing a
   candidate remained secondary; and
4. a graph record risked being mistaken for the act it records.

The process-first correction keeps the useful three-slot representation and
removes those conflations:

- raw Triple identity is recursive and structural;
- explicit nominal identities exist only for real continuity or occurrence;
- Clause is a judgment, not another constructor;
- Run is the dynamic primitive;
- admission alone changes authority; and
- act and trace are separate categories.

This is a reduction of ontology, not an added runtime layer.

A final repository-level review found no fatal contradiction and tightened six
boundaries before landing:

- `RunOutcome` is stepwise and typed for returned values, finite choices,
  yielded continuations, suspension, failure, and exhausted bounds rather than
  pretending every mode has one terminating verdict;
- State transition admission and irreversible external effects use separate
  phases, so a rejected evidence write never claims to roll back an act;
- Atom equality contracts are declarative, canonical, epoch-bound Clause data,
  while explicit identity anchors make cyclic reload well-founded;
- deterministic source-reading lookup happens before child domain checking;
- a host-freeze extension cannot hide meaning in an opaque callback or dispatch
  table; and
- specialization is measured against disconnected graph noise, with
  source-ergonomics, scale, and target-performance gates still required after a
  bounded spike pass.

A separate regression review found no imported persistence product or lost
general-purpose mission.

## Protected guarantees

Changing the irreducible representation does not retire the guarantees already
won by the named-role and identity work:

- stable RoleIds and declared role types;
- exact cardinality and complete role coverage;
- provisional incomplete candidates and atomic admission;
- source-order independence after elaboration;
- no positional role inference in semantic consumers;
- duplicate equal assertions remaining distinct occurrences;
- independent supports and occurrence-exact retraction;
- distinct Program, snapshot, change, revision, session, State, transition,
  effect, and receipt identities;
- exact source and generated-artifact traceability; and
- strict canonical reload and tamper rejection.

The target may retain packed role maps and current structs as checked indexed
views or runtime materializations. The rejected claim is only that they are a
second irreducible semantic substance.

## Lean 4 constitutional-checker evidence

The exact Lean source inspected for the bootstrap decision is upstream
[`leanprover/lean4`](https://github.com/leanprover/lean4/tree/342db4dbdb3aab611e0b92ddba0c134c9b28b2f9)
revision `342db4dbdb3aab611e0b92ddba0c134c9b28b2f9`, licensed Apache-2.0. No Lean
code was copied, adapted, vendored, or built for this decision. No
constitutional checker binary exists yet, so its exact Lean toolchain and
artifact hashes remain a required spike output rather than recorded evidence.

The source supports a narrow, useful conclusion: Lean can provide a strong
proof-admission boundary for a Clause model encoded in Lean, but its kernel
does not natively understand Clause Terms or graphs.

- `src/kernel/type_checker.h:135-166` and
  `src/kernel/environment.cpp:271-280` show a kernel that checks Lean
  expressions and declarations. Clause therefore needs an explicit decoder,
  object-language model, certificate proposition, and theorem connecting a
  certificate to Clause validity; a host Boolean alone proves nothing.
- `src/kernel/declaration.h:91-118` and
  `src/kernel/type_checker.cpp:101-122` enforce Lean's safe/partial/unsafe
  distinction. Clause's constitutional path can remain safe and total while
  partial, reactive, streaming, and effectful Clause modes stay explicit in
  the object language.
- `src/Lean/AddDecl.lean:22-28` exposes skipped kernel checking, while
  `src/Lean/AddDecl.lean:109-221` shows preliminary asynchronous declarations,
  error recovery through axioms, and `sorryAx` fallback. Clause admission must
  await the checked environment and reject every such recovery artifact.
- `src/Lean/Environment.lean:138-144` documents import trust levels, and
  `src/Lean/Shell.lean:207-238` exposes a high believer default for the shell.
  `trustLevel = 0` checks newly added declarations but does not recheck imported
  `.olean` bodies. Constitutional validation therefore needs exact source,
  toolchain, and artifact hashes, rejection of unsafe/partial declarations in
  the transitive certificate closure, and replay of every reachable safe/total
  declaration into a fresh kernel environment before acceptance. Lean's stock
  replay skips unsafe/partial constants, so their absence from the closure is a
  separate required check.
- `src/Lean/Compiler/ImplementedByAttr.lean:15-43` says an
  `implemented_by` replacement is not checked equivalent, and
  `src/Init/Tactics.lean:1452-1495` says `native_decide` trusts compiled
  execution through an axiom. Neither is valid constitutional evidence.
- `src/Lean/Util/CollectAxioms.lean:29-154` can enumerate axiom closure, but
  policy must still explicitly allow or reject foundations such as `propext`,
  `Quot.sound`, and `Classical.choice`.
- `src/LeanChecker.lean:56-73` can replay declarations into a kernel
  environment and explicitly says it is not an external verifier. It is a
  useful same-kernel consistency gate, not an independent trust root.

This evidence selects Lean for the next semantic implementation tranche, not
as Clause's ontology or source-language host. The spike must still measure the
encoding/TCB size, proof ceremony, canonical-codec boundary, feedback cost, and
Lean/Rust parity. If Clause meaning migrates into Lean syntax constructors,
closed per-feature inductives, compiler-trust execution, or unchecked imports,
the implementation has failed the same no-second-authority test applied to
Rust.

## Aeneas boundary

The exact Aeneas source inspected is upstream
[`AeneasVerif/aeneas`](https://github.com/AeneasVerif/aeneas/tree/9467a32f98437dd2812fc693fd475827775f5186)
revision `9467a32f98437dd2812fc693fd475827775f5186`, licensed Apache-2.0. No Aeneas
or Charon code was copied, adapted, vendored, built, or added as a dependency.
That Aeneas revision requires Charon revision
`2881d1238bcb1f2f30a62f07018da1e397bcb181`.

Aeneas translates a supported Rust subset through Charon's LLBC and an OCaml
pure-functional intermediate representation into proof-assistant code,
including Lean. That makes it potentially useful later as an optional,
independently gated verifier for selected isolated safe-Rust helpers in the
physical substrate.

It is not part of the Clause Core spike or architecture. Its closed translation
IR, handwritten/axiomatic treatment of opaque externals, unsupported unsafe and
concurrent Rust, additional Charon/OCaml/version-pin boundary, and generated
Lean model do not establish Clause's canonical meaning. Its Pure IR's explicit
state, failure, and fuel rewriting is interesting implementation precedent for
honest Run outcomes, but its pinned Lean backend rejects the fuel option. It
remains Rust-specific compiler translation rather than Clause
context/admission semantics.

## Prior art considered

Individual ingredients have serious precedent:

- abstract syntax graphs and hierarchical graph representations show that
  binding and sharing need not be reconstructed from a sovereign AST:
  <https://arxiv.org/pdf/2102.02363>;
- scope graphs show name resolution as explicit paths and relations:
  <https://eelcovisser.org/publications/2015/NeronTVW15.pdf>;
- W3C n-ary relation patterns show relation-instance and role-edge encodings:
  <https://www.w3.org/TR/swbp-n-aryRelations/>;
- Sea of Nodes shows a graph-shaped compiler IR can still specialize to
  efficient machine execution:
  <https://assets.ctfassets.net/oxjq45e8ilak/12JQgkvXnnXcPoAGoxB6le/5481932e755600401d607e20345d81d4/100752_1543361625_Cliff_Click_The_Sea_of_Nodes_and_the_HotSpot_JIT.pdf>;
  and
- e-graphs show one graph-based technique for explicit equivalence and
  optimization search:
  <https://arxiv.org/abs/2004.03082>.

Conventional compiler pipelines remain the main alternative: lossless or typed
AST, symbol/type side tables, dedicated control/dataflow IRs, and target IRs.
They are mature, locally explicit, and easy to optimize, but permit semantic
relationships and language extension to fragment across host-owned structures.

The other rejected alternative is a universal nominal graph in which every
Triple has an identity. It handles edge metadata and cycles simply, but gives
one address too many semantic jobs and forces later distinctions between the
node ID and the identities users actually mean.

Clause departs from both only if the host-freeze gate proves that one judged
Term graph genuinely reduces semantic duplication without sacrificing source
clarity or target specialization.

These sources support ingredients, not the composition. The published papers
were not independently reproduced; Lean and Aeneas were inspected at the exact
source revisions recorded above. No external implementation source was copied,
adapted, or vendored.

## Disproof evidence required

The [adoption spike](adoption-spike.md) is deliberately shaped to disprove the
kernel if any of these occur:

- a meaningful construct escapes into a private Lean or Rust semantic case;
- the Lean certificate path admits skipped checking, recovery or unlisted
  axioms, compiler-trust evaluation, unchecked imports, or a result not tied to
  the exact Clause proposition;
- Lean and Rust disagree on acceptance, identity, outcome, delta, obligation,
  or trace under the canonical package;
- named roles become positional or partial n-ary roots conflate values;
- raw Term handles leak as occurrence or entity identity;
- structurally equal transfers collapse into one event;
- expression, evaluated value, and denotation become indistinguishable;
- trace replay repeats an act or external effect;
- Atom equality depends on host behavior for NaN, signed zero, Unicode, or
  numeric width;
- macros lose binding, source, type, effect, or phase information;
- a total mode smuggles in divergence or the compiler overclaims a universal
  halting decision;
- a local edit requires ordinary whole-graph recomputation;
- readable source requires exposing graph bookkeeping; or
- generic Triple execution cannot specialize to credible targets.

The host-freeze extension is the strongest check. If a new binding-and-effect
construct needs any construct-specific host semantic case, the claimed
universal authority has not been achieved.

## Remaining uncertainty

The following remain unproved:

- that every dangerous language semantic fits one generic judged Term graph;
- that the focus calculus stays clearer than Python across real programs;
- that structural equality, explicit identities, canonical reload, and cyclic
  references remain comprehensible and efficient at scale;
- that Clause-authored schemas and macros avoid ontology ceremony;
- that the Lean encoding and certificate bridge stay smaller, clearer, and
  cheaper than the semantic boundary they protect;
- that an acceptably small logical-axiom policy and reproducible imported-module
  provenance can be maintained;
- that incremental dependency closure stays precise on large graphs;
- that generic meaning specializes competitively to systems, Wasm,
  JavaScript, browsers, and databases;
- that ownership, concurrency, packages, FFI, ABI, and deployment semantics
  compose without a second authority; and
- that correct-change time or maintenance cost improves materially in real
  teams and systems.

The next honest evidence is the bounded adoption spike. More architecture prose
cannot resolve these uncertainties.
