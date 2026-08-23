# Clause Roadmap

Clause is becoming one general-purpose, relational programming system. It will
not remain a semantic-modeling DSL that hands ordinary programming to another
language, grow a separate game language, or disguise objects and fields as
lighter syntax.

The current Rust implementation is a real but narrower foundation. It seals
typed Models as immutable Revisions; navigates them with `find`, `why`,
`prevent`, `achieve`, and `diff`; and projects exact requests and Revisions into
standalone Rust. Its executable syntax is current truth, not the target human
surface.

This roadmap distinguishes four things:

- **implemented truth** — behavior present on public `main`;
- **protected contract** — evidence a successor must preserve;
- **committed direction** — the dependency and acceptance order below;
- **provisional surface** — spellings and specimens still being tested.

A milestone or specimen is not implemented merely because it appears here.

## Authority

This tracked roadmap is the sole normative language and product roadmap.
[SURFACE.md](SURFACE.md) durably preserves the operator's current **Clause
Surface Reset** draft. That draft supersedes the human-surface recommendations
and specimens in the earlier strategy packet, but it is explicitly provisional
and undergoing revision. Its relational invariants control direction; its exact
punctuation does not become a parser contract until M0 freezes it with corpus
and elaboration evidence.

The earlier packet remains provenance, not live surface authority. Its four
unique input identities were:

| Packet input | SHA-256 |
| --- | --- |
| `00-executive-brief.md` | `6362750f50e90d9973bf8e626e79092a4478492431bbc6faf65343157cd57bcc` |
| `01-surface-language-charter.md` | `fcba0c43c5b9c780c3099c2a7d1be4b0075e877e4377345a304ea209d44d0afe` |
| `02-game-capable-programming-system-strategy.md` | `97181cb91749a88f152bbe00d0645b6b19c234d4447b877d2c516666862fbcfb` |
| `03-language-specimen.md` | `5055157daf8b96141667a5c64950bd0ef46c8abaf2140aba05de2b302a45fc9a` |

`CLAUSE-STRATEGY-COMPLETE.md` was a duplicate combined projection, not a fifth
proposal authority. No external Documents path is required to recover or
execute this roadmap.

## Product thesis

For the current semantic core:

> **The sealed Revision is the program. Source and generated code are
> projections of it.**

For the complete programming system:

> **A Clause program is a graph of grounded symbols and role-labelled clauses.
> Layout is an erasable projection. The sealed ProgramRevision is the program;
> State advances by exact deltas; effects produce receipts; source and target
> code are projections.**

The authored ontology does not contain objects, fields, owned child records, or
implicit mutation. The compiler remains free to lower functional relations to
fields, columns, component arrays, indexes, or bitsets when those physical
strategies preserve the relational meaning.

One recursive surface and one module, type, identity, package, diagnostic, and
target system serve ordinary values, propositions, laws, queries, transitions,
effects, and procedures. They lower through distinct checked IR strata rather
than pretending to be one universal node or solver.

## Protected contracts

These contracts remain load-bearing while the surface changes:

- immutable, content-derived Revisions and exact predecessor lineage;
- exact admitted and withdrawn Deltas;
- stable semantic relation identities independent of wording and orientation;
- typed, named roles on genuinely n-ary relations;
- bounded positive recursive derivation;
- exact proof/support structure, including alternative supports;
- sound inclusion-minimal `prevent` and `achieve` results with explicit finite
  bases and honest complete/incomplete status;
- assertion, entailment, proof, and support diff across exact Revisions;
- canonical request ordering and result bytes;
- source-to-IR provenance and role-aware diagnostics;
- source-deleted executable parity for every claimed target;
- separation of modeled state change, effect intent, authorization, attempt,
  receipt, observation, and admitted truth.

The current hospital six-direction results remain a semantic compatibility
oracle. Their old spelling does not remain canonical source authority.

## Whole-program authority

The current `Revision` is retained as the proven immutable model capability.
The complete system separates authorities a live application cannot honestly
collapse:

| Authority | Meaning |
| --- | --- |
| `ProgramRevision` | Immutable content-addressed checked program: bindings, value shapes, relations, definitions, laws, transitions, effects, dependencies, capabilities, target requirements, and selected initial content. |
| `ModelRevision` | Immutable admitted domain/content clauses checked against one exact ProgramRevision. This is the future placement of today's semantic `Revision`. |
| `StateRevision` | Logically immutable runtime state produced from an exact predecessor and Delta and bound to exact program/model authority. |
| `RuntimeSession` | ProgramRevision, initial model/state, tick policy, ordered input/event stream, transition results, capability realizations, and effect receipts. |

Today's `Revision` fuses checked declarations and admitted content. Any future
split migrates the representation and all in-tree consumers atomically; it does
not pass through two public wire meanings. M0 audits terminology and identity
against Beagle and Store before freezing shared wire nouns.

## Surface constitution

The following relational invariants are committed direction now:

1. Bare names ground semantic symbols.
2. Categories emerge through membership and contracts; they are not an
   exclusive declared species.
3. Semantic entities have relations, not fields.
4. Enumeration, structural binding, and focused-claim blocks are distinct.
5. Indentation never creates ownership or nested object identity.
6. Focused and fully expanded clauses elaborate to identical role-labelled
   semantics and canonical identity.
7. Relation phrases are exact declared grammar, never probabilistic NLP.
8. Every relation role accepts a recursively parsed term of the expected
   category.
9. Stable semantic identity and named roles remain in the checked core even
   when hidden from ordinary source.
10. Inference is permitted only when one lexical, structural, and type-correct
    elaboration survives; otherwise the diagnostic requires explicit structure.
11. Search, bounds, nondeterminism, time, authority changes, effects, and
    externally observable order are never hidden for aesthetic compression.
12. Runtime representation is free to specialize; semantic meaning is not.

The current draft additionally proposes `:` for binding, `=` for equality,
`->` for projection/production, `?`/`?name` for holes, `~>` for state
succession, `+`/`-` for exact deltas, `!` for effects, naked hole clauses for
selection, and `select`/`any` for projection/existence. These spellings are
provisional until M0. No implementation lane may quietly freeze them first.

## Current foundation

Public Clause currently implements:

- typed Models, entity identities, and role-labelled n-ary relations;
- a flat declaration surface using `Type`, `Relation`, `Model`, `Law`,
  `Revision`, `mode`, bracketed multiword entities, and explicit request words;
- asserted clauses and bounded positive recursive laws;
- immutable Revisions and exact successor Deltas;
- recursive closure queries and bounded complete support enumeration;
- bounded prevention and achievement over explicit finite intervention bases;
- assertion, entailment, proof, and support diff;
- authored request ordering and canonical result bytes;
- standalone generated Rust with source-deleted result parity.

That surface remains executable during migration, but it is ceremonial syntax
scheduled for retirement. It must not be used as the template for new canonical
examples or independently extended as a second language.

## Dependency spine

The Surface Reset sequence is now the primary critical path:

```text
current semantic foundation
    |
    v
M0 constitution, corpus, lossless reader contract
    |
    v
M1 layout, grounding, enumeration, binding, and focus
    |
    v
M2 compact exact relation schemas and contracts
    |
    v
M3 recursive term/value grammar and pure definitions
    |
    v
M4 holes, laws, and relational selection
    |
    v
M5 Revision surface reset and migration parity
    |
    v
M6 StateRevision, events, transitions, and replay
    |
    v
M7 effects, JavaScript ESM, Three.js, and one-coin proof
    |
    v
M8 migrate corpus and retire ceremonial syntax

current semantic foundation --> lineage/intervention maintenance
                                  preserves exact semantics beside M0–M8
```

Each milestone is a vertical through lossless reading, deterministic
classification/resolution, role-labelled elaboration, checked IR, canonical
identity, diagnostics, formatting, materialization, and the smallest executable
acceptance program. Parser-only or runtime-only fragments do not close a
milestone.

## M0 — Freeze the constitution and corpus

**Depends on:** current semantic foundation.

M0 is the only admitted next surface implementation milestone. Before changing
the parser grammar, deliver:

- golden sources for enumeration, value-shape bindings, focused graph claims,
  explicit flattening, relation schemas, recursive terms, holes/correlation,
  laws, selection, revisions, transitions, effects, hospital egress, and the
  one-coin game;
- for every specimen: lossless grouped tree, elaborated role graph, canonical
  structural rendering, diagnostics, expected semantic result, and applicable
  generated-result oracle;
- a Stage A lossless layout reader contract containing lines, indentation
  groups, delimiters, literals, names, punctuation, and source spans without
  deciding object/type/relation semantics;
- a deterministic Stage B classification contract for enumeration, structural
  binding, focused claim/contract, law/definition, query, revision delta,
  transition, and epistemic/effect forms;
- a surface ledger recording each semantic distinction, candidate spelling,
  inferable information, ambiguity, explicit repair, checked node, and runtime
  consequence;
- a canonical role-labelled structural escape hatch and formatter prototype;
- exact focused-versus-expanded equivalence oracles proving identical roles,
  proposition and Revision identity after elaboration, results, and generated
  output, with source span the only permitted provenance difference;
- formatter separation of enumeration and focus blocks plus a diagnostic before
  an edit reclassifies an existing all-bare block;
- bounded corpus measurements for scaffolding, punctuation, ambiguity,
  qualification, formatting stability, role diagnostics, import stability,
  and agent repair success;
- a terminology/identity audit against Beagle and Store;
- a bounded prior-art brief for modes/determinism, functional-relational
  composition, typed effects, structural extensibility, semantic identity,
  data-oriented layout, and deterministic state/update architectures.

M0 may implement a lossless reader and fixture harness. It may not implement or
publish the provisional canonical grammar beyond invariants needed to make the
corpus and elaboration contracts executable.

**Exit proof:** the complete corpus parses losslessly; every block has one
structural classification or a finite exact diagnostic; canonical structural
rendering round-trips; focus/expanded pairs produce identical role graphs and
semantic identities; and every proposed canonical token has written semantic
rent. Unrelated imports preserve elaboration or produce an exact ambiguity
repair.

**Next safe checkpoint:** an independently reviewed M0 corpus and Stage A/B
contract, not a new parser profile.

## M1 — Layout and focus profile

**Depends on:** frozen M0 rulings.

Implement bare symbol grounding, enumeration blocks, homogeneous binding/value
shape blocks, focused claim/contract blocks, explicit flattening display, and
multiword semantic names without bracket syntax. Lower into the current
semantic core. No semantic node created by focus may contain owned child fields
or nested records; `iron-door: Door` must never silently mean membership.

**Exit proof:** all-bare enumeration lowers child-to-parent membership; any
non-bare child deterministically selects focus; bare children under focus
classify the focus; binding blocks remain structural; focused and expanded
forms are semantically and canonically identical; current hospital semantic
results and source-deleted parity remain unchanged.

## M2 — Compact relation schemas

**Depends on:** M1.

Implement exact schema pattern bindings, named roles, focus-role designation,
projection/cardinality contracts, deterministic ambiguity diagnostics, stable
hidden relation identities, and the explicit role-labelled escape hatch.
Remove required `Relation`, brace-role, and `mode` ceremony in the new profile.

Physical strategy remains separate: a functional relation may lower to a field
or column, but the checked meaning remains an ordinary role-labelled relation.

**Exit proof:** unary, binary, and n-ary relations share one role-labelled core;
ordinary clauses need no schema name when resolution is unique; explicit
structure round-trips exactly; ambiguity names candidate shapes and conflicting
roles.

## M3 — Recursive terms, value shapes, and pure definitions

**Depends on:** M2.

Permit every relation role to contain recursive terms with explicit grouping
and canonical formatting. Add structural value shapes, pure bindings,
single-valued relation projections, pure definitions, numeric and boolean
values, tuples, sums, collections, patterns, immutable locals, modules, and the
smallest ordinary algorithms needed by the one-coin specimen.

The value/term stratum may use record-like structural shapes. Semantic symbols
remain relational. Dot access is reserved, if admitted at all, for explicit
foreign-host interoperation.

**Exit proof:** nested distance/collision expressions, hospital relations, a
nontrivial data transform, and a headless one-coin pure simulation use one
recursive checked tree. Canonical grouping and source/role diagnostics remain
stable, and no ordinary algorithm must be encoded as generic runtime triples.

## M4 — Holes, laws, and relational selection

**Depends on:** M3.

Implement fresh anonymous holes, named reusable/result holes, repeated-hole
correlation, naked single-clause selection, explicit `select` projection,
`any` existence, exact-one and canonical-first selection, and `if`-inferred
positive laws with hidden or optional human labels.

Random witness selection is a distinct effectful operation. `find` ceases to be
the canonical relational query word but may remain a tooling/library verb.
`why`, `prevent`, `achieve`, and `diff` retain distinct semantic operations.

**Exit proof:** anonymous and named holes have exact column/correlation
semantics; `any` returns only Bool; recursive hospital laws and queries retain
their bounded results/proofs; no general English parser or universal unbounded
logic search is introduced.

## M5 — Revision surface reset and migration parity

**Depends on:** M4.

Implement revision forms recognized by exact ancestry and signed clauses;
preserve current `why`, `prevent`, `achieve`, and `diff` behavior; and provide a
formatter/codemod from the current Model/Law/Revision syntax. The migration
reports every inference and preserves stable semantic IDs.

Do not keep the old grammar indefinitely as a second first-class language. A
temporary profile exists only to prove parity and migrate in-tree consumers.

**Exit proof:** removing declaration-kind words and changing layout do not
alter checked Revision content; rename operations preserve stable identities;
signed deltas retain exact bases and completeness behavior; current hospital
source migrates with identical six-direction canonical results and
source-deleted Rust parity.

## M6 — State, events, transitions, and replay

**Depends on:** M5.

Add `StateRevision`, event scopes, clause-to-clause succession, keyed
replacement for checker-enforced functional relations, explicit Delta fallback,
conflict analysis, deterministic tick and replay, and canonical state diff.

A transition evaluates matches and guards against one pre-state, computes
candidate deltas, rejects or resolves conflicts by declared policy, commits one
successor, and then derives post-state views. Source order never accidentally
resolves declarative multi-writes. Effects occur only after commit and observe
post-state unless a different phase is explicit.

**Exit proof:** the same program, initial StateRevision, tick policy, and event
log produce byte-identical states; conflicting writes fail deterministically;
replay and diff explain changes; the headless one-coin simulation is authored
without object-field mutation.

## M7 — Effects, JavaScript, Three.js, and the one-coin proof

**Depends on:** M6 and exact lineage/intervention maintenance through current
protected behavior.

Add explicit effect boundaries, receipts and opaque resources, capability and
package requirements, generated JavaScript ES modules, source maps, browser and
Node hosts, assets/input/audio adapters needed by the vertical, a pure
StateRevision-to-RenderPlan projection, and a Three.js reconciler.

The first proof of general-purpose direction is the one-coin game from
[SURFACE.md](SURFACE.md): movement, collision, collection, score, deterministic
replay, derived scene, and `render!` receipt. Handwritten JavaScript is confined
to declared adapters/runtime. JavaScript is the first browser host; Wasm may
specialize measured pure kernels later without source rewrite.

This milestone deliberately replaces the earlier plan to make another ontology
or a simultaneous North-and-game mega-gate the first proof. A real North
application slice remains a later general-purpose systems gate after the
one-coin vertical has validated the language, effect, and target seams.

**Exit proof:** the one-coin program compiles to generated JavaScript, runs
through Three.js, replays deterministically, returns render receipts, lowers
functional relations to direct storage rather than a generic triple hot path,
and maps errors back to relation roles and focus blocks.

## M8 — Migrate and retire ceremonial syntax

**Depends on:** M7 parity and migration evidence.

Format in-tree examples into the reset surface, migrate every consumer in the
same change, remove duplicate old grammar paths and fixtures, keep only the
explicit structural interchange/debug form, and rewrite README around the
relational authoring model.

Removal means absence from the live tree: no compatibility parser, tombstone,
stale test, old canonical specimen, or hidden consumer survives unless a real
consumer and retirement condition are named before M8 begins.

**Exit proof:** all supported sources use one surface; old declaration ceremony
is absent; migrated programs preserve exact semantic IDs and results; the
hospital and one-coin journeys pass from canonical source through generated
targets and source-deleted parity.

## Syntax-neutral lineage and intervention maintenance

This lane may proceed beside M0–M8 when it does not depend on unsettled source
syntax. It is compiler machinery, not a new public metaphysical primitive.

For a proposition `P` in Revision `R`, retain one exact shared derivability
representation over asserted clauses:

```text
AND — every premise of one law application is required
OR  — any alternative law application or assertion is sufficient
```

The same representation supports query evaluation, one proof, complete bounded
minimal-support frontiers, prevention cuts, bounded achievement, and semantic
diff. A canonical selected proof must never erase independent support.

Safe maintenance includes:

- sealed Revision/wire identity and strict reload;
- derivation, proof, support-frontier, and support-diff correctness;
- exact finite intervention certification and prompt bound enforcement;
- deterministic request ordering and result rendering;
- source-deleted Rust parity and target-neutral generated-runtime behavior.

Unsafe before M0 rulings includes parser grammar, declaration categories, focus
lowering, entity spelling resolution, relation-schema surface, canonical
examples, and wire forms coupled to a provisional AST.

Correctness guarantees remain separate:

| Guarantee | Meaning |
| --- | --- |
| Soundness | Every returned intervention changes the target answer. |
| Minimality | No proper subset of a returned intervention works. |
| Optimality | No qualifying intervention is better under the declared ordering. |
| Completeness | Every qualifying intervention has been enumerated. |

Relax completeness, never correctness. Deterministic fuel, canonical tie
breaking, strategy identity, resumable continuation identity, and exact
verification govern bounded search. Heuristics or agents may order proposals;
they may not admit results or prune an exact branch without proof.

> **Agents speculate. Clause certifies. Search may be plural and speculative;
> admission remains singular and exact.**

## Current execution disposition

The reset changes what existing unpublished work means:

- the current public parser, hospital program, focus/ellipsis implementation,
  and request syntax are implementation truth and migration/parity oracles, not
  the target canonical surface;
- unmerged old-surface README, parser, focus, ellipsis, and hospital specimen
  branches are quarantined from publication and must not resume as canonical
  syntax work;
- their useful test intent—multiword symbols, finite correlated expansion,
  source spans, role identity, exact results—moves into the M0 corpus;
- semantic-core intervention, lineage, diff, wire, request-ordering, and
  generated-parity work may be re-chartered against public main when it is
  independent of old AST spelling;
- host-language spikes and old frontend candidates are research evidence, not
  critical-path authority;
- no parser implementation begins until a separate M0 charter names its exact
  corpus, Stage A/B contract, acceptance gate, lane, and owner.

## Explicit supersession ledger

The reset supersedes these earlier recommendations as target syntax:

| Earlier recommendation | Reset direction |
| --- | --- |
| `use game` | `requires` block |
| `type: Vec2` or `type Vec2:` | inferred `Vec2` binding/shape block |
| `Space: Type` | bare grounding/category use |
| `name: Relation` with brace roles and `mode` | exact schema binding with named roles and projection/cardinality contract |
| `player: Player` plus `property: value` | grounded `player` with co-equal focused claims |
| `position of player = value` for admission | subject-first relation claim; `=` reserved for equality |
| `name: Law` and `when:` | conclusion plus `if`; optional label binding |
| `find all ?x` | naked hole clause or explicit `select` |
| “logic variable” as the primary account of `?name` | visible hole/reusable-result semantics |
| `name: Revision`, `from:`, `withdraw:` | ancestry form plus signed clauses |
| brackets around multiword entities | ordinary multiword semantic names |
| `:=` | no second binding glyph; `:` is the proposed binding axis |
| `player.position` as canonical projection | relation-first `position of player` |
| simultaneous North + Orbit as the first generality gate | generated-JavaScript Three.js one-coin vertical first; real North slice later |

This ledger supersedes recommendations, not implemented behavior. Current
executable documentation must continue to label old syntax as current until M8
actually removes it.

## Cross-cutting acceptance gates

Every milestone preserves gates already crossed:

1. **Relational honesty:** focus/layout never creates objects, fields, or owned
   child records.
2. **Recursive structurality:** every eligible role accepts a recursive checked
   term rather than one lexical token.
3. **Canonicality:** authored sugar lowers away; parse/format/parse and canonical
   identity are stable.
4. **Locality:** imports never silently change old meaning; ambiguity reports
   candidates, role conflicts, and exact repairs.
5. **Category honesty:** values, propositions, definitions, laws, queries,
   transitions, procedures, effects, and evidence remain distinct.
6. **Identity honesty:** layout and rename operations preserve stable semantic
   identity when the editor transaction says they are the same meaning.
7. **Lineage honesty:** exact bases, deltas, alternative supports, proof changes,
   and intervention certificates survive every surface migration.
8. **Bound honesty:** partial derivation or search never claims completeness or
   optimality it did not prove.
9. **Operational honesty:** modeled transition, authorized intent, attempted
   effect, receipt, and observation remain distinct.
10. **Projection parity:** selected judgments reproduce from sealed artifacts
    after source deletion on every claimed target.
11. **Physical freedom:** relational meaning may lower to specialized storage;
    the runtime does not pay a generic triple tax without evidence.
12. **General-purpose reach:** ordinary computation, state, effects, and a real
    JavaScript/Three.js application do not escape into a shadow implementation.

## Compiler ownership

| Boundary | Owns | Must not own |
| --- | --- | --- |
| Lossless reader | layout, tokens, delimiters, spans | object/type/relation meaning |
| Classifier/resolver | deterministic block classes, scoped shapes, ambiguity | target policy or semantic evaluation |
| Role-labelled elaborator | grounding, bindings, membership, relations, laws, queries, deltas, transitions, effects | physical storage strategy |
| Checked program | types/categories, stable IDs, Program/Model/State authority, provenance | host resources |
| Value lowering | pure values, definitions, patterns, collections, target-neutral IR | proposition search or effect realization |
| Proposition/lineage | laws, closure, proof/support, exact intervention verification | heuristic authority or external effects |
| Transition runtime | events, snapshot matching, conflicts, deltas, replay | callback mutation or incidental source-order conflict resolution |
| Effect runtime | capabilities, resources, async outcomes, receipts | claiming intent as fact |
| Target planner | JavaScript, Rust, later Wasm, indexes, schedules, layouts | changing source meaning to fit a target |
| Tooling | formatter, editor, explicit expansion, migration, source maps, debugger | a second semantic authority |

## Anti-goals

- no natural-language parser or probabilistic interpretation;
- no object language hidden behind property-like indentation;
- no generic triple interpreter in the hot path;
- no hidden ambiguity, search, temporal mutation, effects, or conflict order;
- no universal logic inversion, open-world negation, or global equality
  saturation before a bounded program earns them;
- no denial of procedures: an explicit `do` stratum remains for algorithms
  whose order is their meaning;
- no requirement that every value or frame be a durable graph snapshot;
- no effects before the transition core can distinguish modeled change from
  observed reality;
- no Wasm-first browser host or source rewrite for target partitioning;
- no syntax stabilization before M0 evidence or old-grammar retirement before
  migration parity.

## Completion standard

Clause reaches the committed category only when the same sealed program can:

- express ordinary values and algorithms without relational contortions;
- state grounded symbols, membership, role-labelled propositions, and recursive
  laws without object smuggling;
- move forward to bindings, backward to reasons, across exact Revisions, and
  counterfactually to certified Deltas;
- evolve deterministic state through conflict-checked successor clauses;
- cross external boundaries through explicit capabilities and receipts;
- materialize inspectable JavaScript and native projections with source maps and
  source-deleted parity;
- run the one-coin Three.js vertical, then a real North application slice;
- retain formatting, diagnostics, replay, diff, explanation, and intervention
  through the complete authoring loop;
- remove the ceremonial grammar after migration rather than preserving two
  first-class languages.

The standard is not “less syntax than current Clause.” It is the shortest
surface that states the semantic structure without lying about category,
identity, time, search, or effect.
