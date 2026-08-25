# Clause Roadmap

> **Status:** Current.
>
> **Authority:** Normative for implementation sequence, dependency order, and
> acceptance gates.
>
> **Relationship:** Governed by the [semantic foundation](foundation.md) and
> [target surface](surface.md). Supersedes the earlier strategy schedule and
> treats [M0](history/m0.md) as a historical checkpoint rather than current
> syntax authority.

Clause is becoming one general-purpose, relational programming system. It will
not remain a semantic-modeling DSL that hands ordinary programming to another
language, grow a separate game language, or disguise objects and fields as
lighter syntax.

Its one semantic domain is addressable referents: stabilized distinctions that
remain reidentifiable across relational content, assertion occurrences, and
Revisions and are not collapsed by structural equality. Relations are
referents in relational position, and n-ary relational content assigns every
participant to a stable named role. An assertion occurrence commits to content
under an exact scope and provenance; judgment and modal authority remain
separate. Terms project this Model; source terms and files are not the Model.

The current Rust implementation is a real but narrower foundation. It seals
referent Models as immutable Revisions; navigates them with `find`, `why`,
`prevent`, `achieve`, and `diff`; and projects exact requests and Revisions into
standalone Rust. Its executable M1 forms are the first implemented target
slice; retained declaration and request ceremony is implementation truth, not
the target human surface.

This roadmap distinguishes four things:

- **implemented truth** — behavior present on public `main`;
- **protected contract** — evidence a successor must preserve;
- **committed direction** — the dependency and acceptance order below;
- **provisional surface** — spellings and specimens still being tested.

A milestone or specimen is not implemented merely because it appears here.

## Authority

The semantic foundation governs meaning, this roadmap governs product
sequence, and the target surface governs authoring projection. `Chess ∈ Game`
is ordinary membership relational content, while `gravity: 9.81` is a stable
binding. `∈` is canonical source. A human-facing editor may transform typed
`::` into `∈`, but raw `::` is not Clause grammar. Formatters and agents emit
`∈` directly. Canonical target indentation is two spaces, spaces only; tabs
are diagnosed.

Canonical authoring uses easy-to-type ASCII operators when a relation has a
strong, unambiguous conventional prior: `>`, `<`, `>=`, `<=`, `=`, `!=`, and spaced
infix `+`, `-`, `*`, and `/`. These remain named-role relational forms rather than
primitive numeric or arithmetic ontology. Structurally leading `+` and `-`
remain Delta signs, slash-qualified semantic names remain names, and domain
relations such as `connects` and `parent-of` remain words. Punctuation is not
invented where convention conflicts.

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

> **When realized, the canonical authoritative Model is the program. It is a
> graph of addressable referents, n-ary named-role relational content,
> assertion occurrences, judgments, and explicit semantic modes. Layout,
> source, and target code are projections. Revisions retain immutable version
> and lineage evidence; State advances by exact deltas; effects produce
> receipts.**

The authored ontology does not contain objects, fields, owned child records, or
implicit mutation. The compiler remains free to lower functional relations to
fields, columns, component arrays, indexes, or bitsets when those physical
strategies preserve the relational meaning.

One recursive surface serves referents, relational content, assertion
occurrences, judgments, universal laws, derivation rules, invariants, goals,
queries, transitions, effects, and procedures. Type, value, module, package,
object, record, set, function, variable, state, mutation, checking, and
evaluation are derived relational views, not additional semantic domains. They
may lower through distinct checked IR strata without becoming primitive
ontology.

## Protected contracts

These contracts remain load-bearing while the surface changes:

- immutable, content-derived Revisions and exact predecessor lineage;
- exact admitted and withdrawn Deltas;
- stable semantic relation identities independent of wording and orientation;
- explicit, stable named roles on genuinely n-ary relational content;
- bounded positive recursive derivation;
- exact proof/support structure, including alternative supports;
- sound inclusion-minimal `prevent` and `achieve` results with explicit finite
  bases and honest complete/incomplete status;
- assertion, entailment, proof, and support diff across exact Revisions;
- canonical request ordering and result bytes;
- source-to-IR provenance and role-aware diagnostics;
- source-deleted executable parity for every declared target;
- separation of modeled state change, effect intent, authorization, attempt,
  receipt, observation, and admitted truth.

The current hospital six-direction results remain a semantic compatibility
oracle. Their old spelling does not remain canonical source authority.

## Whole-program authority

The current `Revision` is retained as proven immutable version and lineage
evidence. The complete system separates authorities a live application cannot
honestly collapse:

| Authority | Meaning |
| --- | --- |
| `Model` | Canonical authoritative program: referents, relational content, assertion occurrences, judgments, definitions, universal laws, derivation rules, invariants, goals, transitions, effects, dependencies, capabilities, target requirements, and selected initial content. |
| `Revision` | Immutable content-addressed Model version with exact lineage and provenance; evidence about program history, not the program itself. |
| `StateRevision` | Logically immutable runtime state produced from an exact predecessor and Delta and bound to an exact Model Revision. |
| `RuntimeSession` | Exact Model Revision, initial state, tick policy, ordered input/event stream, transition results, capability realizations, and effect receipts. |

Today's Revision-v6 wraps one complete semantic-v10 Model snapshot and either
root lineage or an exact predecessor Delta. Any future split migrates the
representation and all in-tree consumers atomically; it does not pass through
two public wire meanings. The historical M0 page records the earlier evidence
checkpoint without governing the current kernel or surface.

Clause has no Store implementation and asserts no Store closure. Store is a
neutral persistence and query substrate. A future adapter must add a typed
Clause envelope for referents, relational content, assertion-occurrence
attestations, judgments, modality, evidence, authority, admission/rejection,
supersession, and exact Clause-Revision-to-storage-lineage links. It may infer
none of those distinctions from equality, missing rows, liveness, retraction,
query negation, or storage revision identity.

## Surface constitution

The following relational invariants are committed direction now:

1. Terms may denote addressable referents; the term is not the referent.
2. Referent identity survives content, occurrences, and Revisions and is not structural
   equality.
3. Relations are referents in relational position; n-ary content assigns every
   participant to a stable named role; content, assertion occurrence, and
   judgment are distinct.
4. `:` is stable binding/definition; `∈` is ordinary membership; `=` is
   equality.
5. Editors may transform typed `::` into `∈`; raw `::` and word aliases are
   not grammar.
6. Enumeration, derived binding/shape, and focused blocks are distinct. A focused
   bare category lowers to membership, an ordinary relation fragment remains a
   relational content, and `name: value` binds `name of focus`.
7. Indentation is two spaces, spaces only; tabs are diagnosed, and layout never
   creates ownership or nested object identity.
8. Focused and fully expanded clauses elaborate to identical role-labelled
   semantics and canonical identity.
9. Relation phrases are exact declared grammar, never probabilistic NLP.
10. Every relation role accepts a recursively parsed term of the expected
   category.
11. Stable referent, content, and occurrence identity and named roles remain in
    the checked core even when hidden from ordinary source.
12. Inference is permitted only when one lexical, structural, and type-correct
    elaboration survives; otherwise the diagnostic requires explicit structure.
13. Search, bounds, nondeterminism, time, authority changes, effects, and
    externally observable order are never hidden for aesthetic compression.
14. Universal law, oriented derivation rule, Revision-admission invariant, and
    goal remain distinct and never confer one another's authority.
15. Absence is undetermined under the open world; denial requires explicit
    content or judgment.
16. Relations, rules, Revisions, and evaluation may be discussed as referents,
    but interpretation requires admitted shape/mode and may require quotation,
    stratification, or a Revision boundary; mention never self-executes.
17. Runtime representation is free to specialize; semantic meaning is not.

The current surface uses `=` for equality, `->` for
projection/production, `?`/`?name` for holes, `~>` for state
succession, `+`/`-` for exact deltas, `!` for effects, naked hole clauses for
selection, and `select`/`any` for projection/existence. These spellings are
provisional only where the surface document says so. Binding, membership, and indentation
are settled above. No implementation lane may add another membership spelling
or collapse a definition into a field or relational content.

## Current foundation

Public Clause currently implements:

- one stable referent domain and role-labelled n-ary relational content;
- separate assertion occurrences, judgments, definitions, derivation rules,
  universal laws, invariants, goals, and transitions;
- canonical semantic-v10 / Revision-v6 persistence with exact predecessor
  lineage and complete successor Deltas;
- the compact M1 grounding, binding, membership, enumeration, and focus
  surface, M2 exact role-labelled relation schemas and projection contracts,
  and M3 recursive terms, checked structural values, closed pure definitions,
  indexed pure evaluation, and source-deleted generated evaluation, plus M4
  recursive holes, explicit query cardinality, universal laws, and separately
  authorized derivation rules, alongside retained `RelationShape`, `Revision`,
  `mode`, and explicit request ceremony as migration forms awaiting later
  milestones;
- admitted relational content and bounded positive recursive derivation rules;
- immutable Revisions and exact successor Deltas;
- recursive closure queries and bounded complete support enumeration;
- bounded prevention and achievement over explicit finite intervention bases;
- assertion, entailment, proof, and support diff;
- authored request ordering and canonical result bytes;
- standalone generated Rust with source-deleted result parity.

The retained declaration and request ceremony remains executable during
migration and is scheduled for retirement. It must not be used as the template
for new canonical examples or independently extended as a second language.

## Dependency spine

The Surface Reset sequence is now the primary critical path:

```text
implemented distinction-first kernel + canonical v7/v5 wire
    |
    v
M1 layout, grounding, enumeration, definition, and focus
    |
    v
M2 compact exact relation schemas and contracts
    |
    v
M3 recursive term/value grammar and pure definitions
    |
    v
M4 holes, derivation rules, and relational selection
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
M8 migrate examples and retire ceremonial syntax

implemented kernel --> lineage/intervention maintenance
                       preserves exact semantics beside M1–M8
```

Each milestone is a vertical through lossless reading, deterministic
classification/resolution, role-labelled elaboration, checked IR, canonical
identity, diagnostics, formatting, materialization, and the smallest executable
acceptance program. Parser-only or runtime-only fragments do not close a
milestone.

## M0 — Historical evidence checkpoint

**Status:** completed and superseded as syntax authority. See the
[historical record](history/m0.md).

M0 required the following evidence before the parser grammar could change:

- golden sources for enumeration, derived structural views, focused relational
  content and assertion occurrences, explicit flattening, relation schemas,
  recursive terms, holes/correlation, universal laws, derivation rules,
  invariants, goals, selection, revisions, transitions, effects, hospital
  egress, and the one-coin game, including binding, membership, equality, and
  result-orientation contrasts, canonical two-space
  layout and tab diagnostics;
- for every specimen: lossless grouped tree, elaborated role graph, canonical
  structural rendering, diagnostics, expected semantic result, and applicable
  generated-result oracle;
- a Stage A lossless layout reader contract containing lines, indentation
  groups, delimiters, literals, names, punctuation, and source spans without
  deciding object/type/relation semantics; it applies the frozen two-space,
  spaces-only grouping rule and reports tabs;
- a deterministic Stage B classification contract that keeps relational
  content/assertion occurrence, relation contract, definition, universal law,
  oriented derivation rule, invariant, query, goal, observation, requirement,
  intention, effect, transition, and delta structurally distinct whenever the
  source proves the mode, and reports an explicit `Unresolved...` class when it
  does not;
- a surface ledger recording each semantic distinction, candidate spelling,
  inferable information, ambiguity, explicit repair, checked node, and runtime
  consequence;
- a canonical role-labelled structural escape hatch and formatter prototype;
- exact focused-versus-expanded equivalence oracles proving identical roles,
  proposition and Revision identity after elaboration, results, and generated
  output, with source span the only permitted provenance difference; the
  controlling focus specimen contains `state: locked`, which expands to the
  binding `state of focus: locked`, distinct from its membership and
  ordinary relational content;
- explicit accepted/rejected verdicts for classification, definition,
  indentation, and focused projection;
- an editor boundary that transforms typed `::` to `∈` before parsing, with
  formatter/agent rules that emit `∈` membership and `:` binding directly;
- formatter separation of enumeration and focus blocks plus a diagnostic before
  an edit reclassifies an existing all-bare block;
- bounded conformance measurements for scaffolding, punctuation, ambiguity,
  qualification, formatting stability, role diagnostics, import stability,
  and agent repair success;
- a terminology/identity audit of the current typed kernel/wire plus an honest
  statement that no Store-side contract exists in this repository;
- a bounded prior-art brief for modes/determinism, functional-relational
  composition, typed effects, structural extensibility, semantic identity,
  data-oriented layout, and deterministic state/update architectures.

M0 specified a lossless Stage A reader and structural Stage B/formatter
prototype. It does not change the current executable parser or runtime.

**Exit proof:** the complete evidence suite parses losslessly under the selected and
reviewed layout policy; every block has one
structural classification or a finite exact diagnostic; canonical structural
rendering round-trips; focus/expanded pairs preserve their distinct membership
content, ordinary relational content and occurrence, and definition forms and
semantic identities; and every
proposed canonical token has written semantic
rent. The exit review protects the settled classification, definition, and
indentation rulings. Unrelated imports
preserve elaboration or produce an exact ambiguity repair.

**Successor:** the atomic distinction-first kernel and canonical v7/v5 wire
migration described below.

## Completed atomic kernel migration

**Status:** implemented at the semantic-v7 / Revision-v5 checkpoint.

The constitutional kernel migration followed one serial edge and one atomic
checkpoint:

```text
kernel identity/schema/relational form/model/revision/delta + canonical v7/v5 wire
  -> derive/execution/intervention/diff/request
  -> frontend AST/parser
  -> elaboration
  -> CLI/examples
  -> generated Rust
  -> tests
```

Semantic-v5 hashed the bytes of legacy `Type`, `Entity`, `Value`, and
`Variable` encodings. The migration retired those identities rather than
preserving them as live compatibility, separated relational content from its
assertion occurrence and judgment, and persisted exact predecessor lineage.

**Exit proof:** primitive encodings and semantic-v5 are absent; canonical v7/v5
strictly reloads; binding, membership, equality, transition, and
open-world absence remain distinct; exact Delta lineage and hospital results
retain parity; and source-deleted generated Rust emits byte-identical results.

## M1 — Layout and focus profile

**Depends on:** the atomic distinction-kernel and canonical v7/v5 migration.

Implement term-to-referent grounding, `∈` membership, `:` binding, enumeration
blocks, homogeneous binding/derived-shape blocks, focused blocks,
explicit flattening display, and multiword semantic names without bracket
syntax. Lower into the current semantic core. No semantic node created by focus
may contain owned child fields or nested records; `iron-door ∈ Door` must
elaborate to ordinary membership and never to definition or primitive typing.

**Exit proof:** all-bare enumeration lowers child-to-parent membership; any
non-bare child deterministically selects focus; bare children under focus
relate the focus through membership; binding/shape blocks remain derived views; focused and expanded
forms are semantically and canonically identical; current hospital semantic
results and source-deleted parity remain unchanged.

## M2 — Compact relation schemas

**Depends on:** M1.

Implement exact schema role patterns, named roles, focus-role designation,
projection/cardinality contracts, deterministic ambiguity diagnostics, stable
hidden relation identities, and the explicit role-labelled escape hatch.
Remove required `RelationShape`, brace-role, and `mode` ceremony in the new profile.

Physical strategy remains separate: a functional relation may lower to a field
or column, but the checked meaning remains an ordinary role-labelled relation.

**Exit proof:** unary, binary, and n-ary relations share one role-labelled core;
ordinary clauses need no schema name when resolution is unique; explicit
structure round-trips exactly; ambiguity names candidate shapes and conflicting
roles.

## M3 — Recursive terms, derived structural views, and pure definitions

**Depends on:** M2.

**Status:** Implemented.

Permit every relation role to contain recursive terms with explicit grouping
and canonical formatting. Add checked scalar and structural values, closed pure
definitions with immutable locals, single-valued relation projections, and the
smallest ordinary algorithms needed by the headless one-coin specimen.

The value/term stratum may use record-like structural shapes. Semantic symbols
remain relational. Dot access is reserved, if admitted at all, for explicit
foreign-host interoperation.

**Exit proof:** nested distance/collision expressions, hospital relations, a
nontrivial data transform, and a headless one-coin pure simulation use one
recursive checked tree. Canonical grouping and source/role diagnostics remain
stable; the transform produces `[5, 13]`; the one-coin frame produces position
`(150, 0)`, collected `true`, and score `10`; their exact evaluator budgets are
`3` and `9` operations, with one dispatch for a shared application; generated
Rust evaluates the sealed Revision after source deletion with byte-identical
canonical output; and no ordinary algorithm is encoded as generic runtime
triples.

## M4 — Holes, derivation rules, and relational selection

**Depends on:** M3.

**Status:** Implemented.

Implement fresh anonymous holes, named reusable/result holes, repeated-hole
correlation, naked single-clause selection, explicit `select` projection,
`any` existence, exact-one selection, canonical at-most-one `first` selection
(empty on no solution), and `if`-oriented positive derivation rules with hidden
or optional human labels. Authored `law <label>` declarations are semantic
ground only; a separate
`derive <label>` projects an operational rule retaining governing-law,
authority, and scope. Revision-admission invariants and goals remain distinct
modes.

Random witness selection is a distinct effectful operation. `find` ceases to be
the canonical relational query word but may remain a tooling/library verb.
`why`, `prevent`, `achieve`, and `diff` retain distinct semantic operations.

**Exit proof:** one bounded one-coin fixture correlates holes recursively through
distance, radius, and addition applications; a separately authorized universal
law derives one overlap with exact governing-law, authority, and scope proof
trace; its asserted support leaves retain exact occurrence, source, scope, and
judgment identities even for duplicate source acts. `select one` returns exactly
the coin and `any` returns true, each bound to its exact input Revision.
Alpha-renamed holes preserve semantic-v10 bytes, strict canonical reload rejects
tampering, repeated bounded execution is deterministic, and source-deleted
generated Rust prints byte-identical canonical output or fails at the identical
caller-supplied bound. The inherited hospital derivations and queries retain
their bounded results and proofs; no general English parser or unbounded logic
search is introduced.

## M5 — Revision surface reset and migration parity

**Depends on:** M4.

**Status:** Implemented.

Implement revision forms recognized by exact ancestry and signed clauses;
preserve current `why`, `prevent`, `achieve`, and `diff` behavior; and provide a
formatter/codemod from the current Model/DerivationRule/Revision syntax. The migration
reports every inference and preserves stable semantic IDs. A legacy
classification such as legacy `thing: Space` is normalized explicitly: the codemod
writes `thing ∈ Space` or unambiguous list/focus sugar and reports
that inference.

Do not keep the old grammar indefinitely as a second first-class language. A
temporary profile exists only to prove parity and migrate in-tree consumers.

**Exit proof:** removing declaration-kind words and changing layout do not
alter checked Revision content; rename operations preserve stable identities;
signed deltas retain exact bases and completeness behavior; current hospital
source migrates with identical six-direction canonical results and
source-deleted Rust parity.

## M6 — State, events, transitions, and replay

**Depends on:** M5.

**Status:** Implemented.

Add `StateRevision`, event scopes, clause-to-clause succession, keyed
replacement for checker-enforced functional relations, explicit Delta fallback,
conflict analysis, deterministic tick and replay, and canonical state diff.

A transition evaluates matches and guards against one pre-state, computes
candidate deltas, rejects or resolves conflicts by declared policy, commits one
successor, and then derives post-state views. Source order never accidentally
resolves declarative multi-writes. Effects occur only after commit and observe
post-state unless a different phase is explicit.

Ordinary `on` headers now declare event-payload bindings; transition sources
and relational `if` guards may introduce additional transaction-wide pre-state
bindings. Checked elaboration preserves those patterns and guards in the Model
instead of matching the root state or manufacturing ticks. Each explicit
runtime event occurrence supplies its ordered payload; the canonical fold
jointly matches all sources and guards against one pre-state, grounds every
candidate write, rejects ambiguity and duplicate occurrence or functional-key
writers, and commits one successor. Standalone generated Rust reloads that
canonical Revision and supplies the same explicit occurrences through the same
`RuntimeSession` API after the Clause source has been removed.

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
[target surface](surface.md): movement, collision, collection, score, deterministic
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
AND — every premise of one derivation-rule application is required
OR  — any alternative rule application or assertion occurrence is sufficient
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
  source spans, role identity, and exact results—moves into executable tests;
- semantic-core intervention, lineage, diff, wire, request-ordering, and
  generated-parity work may be re-chartered against public main when it is
  independent of old AST spelling;
- host-language spikes and old frontend candidates are research evidence, not
  critical-path authority;
- parser implementation begins only from a charter naming its Stage A/B
  contract, acceptance gate, lane, and owner.

## Explicit supersession ledger

The reset supersedes these earlier recommendations as target syntax:

| Earlier recommendation | Reset direction |
| --- | --- |
| `use game` | `requires` block |
| `type: Vec2` or `type Vec2:` | `Vec2` with a derived binding/shape view |
| `Space: Type` | bare `Space` grounding, or `Space ∈ Category` only when that membership is intended |
| `name: RelationShape` with brace roles and `mode` | explicit relation-contract form with exact named roles and projection/cardinality contract; no primitive relation classification |
| `player: Player` or `thing: Space` used as classification | `player ∈ Player` / `thing ∈ Space`; report the migration |
| object-like `property: value` content | relational `property value` under focus |
| `position of player = value` for admission | subject-first relational content; `=` reserved for equality |
| `name: DerivationRule` and `when:` | conclusion plus `if` for an oriented derivation rule; universal-law mode remains distinct |
| `find all ?x` | naked hole clause or explicit `select` |
| “logic variable” as the primary account of `?name` | visible hole/reusable-result semantics |
| `name: Revision`, `from:`, `withdraw:` | ancestry form plus signed clauses |
| brackets around multiword entities | ordinary multiword semantic names |
| `name: value` used as binding | retain `name: value`; `:` is binding only |
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
   identity are stable under the reviewed two-space indentation policy.
4. **Locality:** imports never silently change old meaning; ambiguity reports
   candidates, role conflicts, and exact repairs.
5. **Category honesty:** values, propositions, definitions, universal laws, queries,
   transitions, procedures, effects, and evidence remain distinct.
6. **Identity honesty:** layout, focus expansion, and rename operations preserve
   stable semantic identity when the editor transaction says they are the same
   meaning.
7. **Lineage honesty:** exact bases, deltas, alternative supports, proof changes,
   and intervention certificates survive every surface migration.
8. **Bound honesty:** partial derivation or search never certifies completeness or
   optimality it did not prove.
9. **Operational honesty:** modeled transition, authorized intent, attempted
   effect, receipt, and observation remain distinct.
10. **Projection parity:** selected judgments reproduce from sealed artifacts
    after source deletion on every declared target.
11. **Physical freedom:** relational meaning may lower to specialized storage;
    the compiler and runtime may be sophisticated when that machinery absorbs
    complexity for authors, preserves reliability and performance, makes defect
    prevention and diagnosis more local, and causes authoring leverage to
    improve as modeled systems grow. Internal minimalism is not an objective by
    itself; authored and maintenance burden are the optimization target.
12. **General-purpose reach:** ordinary computation, state, effects, and a real
    JavaScript/Three.js application do not escape into a shadow implementation.
13. **Scaling leverage:** authored complexity and bug-resolution effort grow
    more slowly than the modeled system. Clause must compound human leverage as
    programs scale rather than merely shorten small examples.

## Compiler ownership

| Boundary | Owns | Must not own |
| --- | --- | --- |
| Lossless reader | layout, tokens, delimiters, spans | object/type/relation meaning |
| Classifier/resolver | deterministic block classes, scoped shapes, ambiguity | target policy or semantic evaluation |
| Role-labelled elaborator | referents, definitions, membership content, assertion occurrences, relations, universal laws, derivation rules, invariants, goals, queries, deltas, transitions, effects | physical storage strategy |
| Checked Model | referent/content/occurrence/judgment/mode IDs, derived admissibility views, exact authority and provenance | host resources or Revision-as-program identity |
| Term lowering | pure terms, definitions, patterns, collections, target-neutral IR | proposition search or effect realization |
| Proposition/lineage | universal laws, derivation rules, invariants, closure, proof/support, exact intervention verification | heuristic authority or external effects |
| Transition runtime | events, snapshot matching, conflicts, deltas, replay | callback mutation or incidental source-order conflict resolution |
| Effect runtime | capabilities, resources, async outcomes, receipts | treating intention as admitted content |
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
  universal laws without object smuggling;
- move forward to role assignments, backward to reasons, across exact Revisions, and
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
