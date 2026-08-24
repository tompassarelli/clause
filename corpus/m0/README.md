# Clause M0 contrast corpus

This corpus freezes Clause's single semantic domain of addressable referents,
relational content, assertion occurrences, universal laws, and derivation
rules. It preserves term versus referent, content versus occurrence versus
judgment, and identity versus structural equality.

This directory is evidence for the [M0 contract](../../M0.md). It deliberately
contains accepted target forms, rejected boundary cases, unresolved candidates,
and current-profile migration evidence. Target acceptance does not assert that
the current parser implements the form.

Every case consists of `cases/ID.clause` and
`cases/ID.expected.json`. Expected JSON uses
`clause-m0-contrast-v1`.

`source.lines` stores raw lines without their line breaks.
`source.line_break` and `source.ends_with_line_break` make exact byte
reconstruction mechanical. `decision` is one of:

- `fixed-invariant` — the stated semantic boundary is direction now; or
- `accepted-target` — canonical target source, independent of current parser
  implementation; or
- `rejected-target` — source the target parser must diagnose; or
- `unresolved` — the source is a contrast candidate whose admission remains
  an M0 decision; or
- `implemented-legacy` — the source is current executable migration evidence,
  not a target candidate.

`required` and `forbidden` are normative for every case.
`elaborated` is an abstract semantic expansion independent of surface spelling.
Handles beginning with `$` are corpus symbols, not Store identities.

| Case | Purpose |
| --- | --- |
| `focus-relation-two` | canonical focus combining membership content, ordinary relational content, and a focused definition |
| `focus-relation-four` | rejected noncanonical four-space projection |
| `expanded-colon-classification` | canonical `:` classification with the same named-role graph |
| `expanded-member-of` | rejected worded membership candidate retained with its verdict |
| `membership-double-colon-alias` | rejected persisted `::`; editors preserve and diagnose it |
| `membership-in-alias` | rejected `in` membership alias |
| `enumeration-two` | canonical child-to-heading membership |
| `enumeration-four` | rejected noncanonical four-space enumeration |
| `indentation-tab` | tab indentation is diagnosed, never normalized |
| `classification-shape` | homogeneous classifications support a derived shape view without primitive records |
| `classification-colon` | `thing : Space` elaborates to ordinary membership |
| `focused-definition` | `state := locked` defines the stable focused term, never a field or relational content item |
| `relation-schema-connects` | compact three-role schema and operational projection candidate |
| `functional-contract-position` | focused binary relation contract, never a field |
| `relational-content-focused-connects` | focus supplies the declared `door` role |
| `relational-content-expanded-connects` | expanded relational content has the same role graph |
| `structural-connects` | explicit role-labelled escape candidate round-trips |
| `ambiguous-connects` | overlapping exact shapes name both candidates and role conflict |
| `missing-focus-connects` | root fragment diagnoses the absent focus role |
| `definition-distance` | `:=`-defined pattern and body are recursive term trees |
| `definition-position-projection` | a definition denotes a one-valued relation projection |
| `nested-role-move` | a relation role contains a grouped expression of projections |
| `overlap-grouped` | explicit grouping fixes the nested comparison tree |
| `overlap-type-resolved` | domain checking leaves the same unique tree without global precedence |
| `mixfix-grouped-left` | explicit left grouping round-trips |
| `mixfix-grouped-right` | explicit right grouping remains distinct |
| `mixfix-ungrouped-ambiguous` | two type-correct trees require an exact diagnostic |
| `query-anonymous-hole` | naked query projects one anonymous hole |
| `query-named-hole` | named hole becomes a named result column |
| `query-named-pair` | two named holes produce two-column rows |
| `query-anonymous-fresh` | each anonymous hole is fresh |
| `select-correlated` | repeated internal hole correlates relational content but is not projected |
| `any-exists` | existential query returns Bool, never a random witness |
| `select-one` | exact-one cardinality contract |
| `select-first` | canonical-first selection is deterministic |
| `derivation-rule-inferred` | conclusion plus `if` body proves an oriented derivation rule |
| `derivation-rule-labelled` | optional `:=` label definition leaves derivation-rule identity intact |
| `revision-withdrawal` | exact-base successor with one signed withdrawal |
| `revision-withdrawal-renamed` | human rename leaves semantic Revision unchanged |
| `revision-withdrawal-legacy` | current profile preserves withdrawal semantics for migration |
| `revision-addition` | exact-base successor with one signed admission |
| `revision-addition-legacy` | current profile preserves admission semantics for migration |
| `revision-mixed-delta` | admission and withdrawal commit as one atomic successor |
| `revision-unknown-base` | unresolved base prevents elaboration |
| `revision-withdraw-missing` | withdrawal must name admitted relational content in the exact base |
| `revision-admit-existing` | admission must be absent from the exact base |
| `revision-overlap` | one relational content item cannot be both admitted and withdrawn |
| `diff-revisions` | diff preserves exact base-to-successor direction |
| `why-explanation` | explanation retains two complete minimal supports and proofs |
| `why-explanation-legacy` | current profile preserves explanation semantics for migration |
| `prevent-all-minimal` | four complete inclusion-minimal withdrawal pairs |
| `prevent-all-minimal-legacy` | current profile preserves prevention semantics for migration |
| `achieve-all-minimal` | two complete typed singleton additions |
| `achieve-all-minimal-legacy` | current profile preserves achievement semantics for migration |
| `prevent-incomplete` | candidate exhaustion cannot certify a complete prevention frontier |
| `achieve-incomplete` | retained additions remain certified under incomplete search |
| `diff-semantic-degradation` | authored, entailed, proof, and support changes remain distinct |
| `transition-functional-position` | complete-relational-content position succession uses a proved functional key |
| `transition-functional-state` | status replacement preserves one stable handle and complete old/new bindings |
| `transition-admit-keypress` | key press emits one exact multi-valued admission |
| `transition-withdraw-keyrelease` | key release emits one exact multi-valued withdrawal |
| `transition-atomic-prestate` | guarded replacements share one pre-state and atomic successor |
| `transition-conflict-forward` | competing functional writers reject without source-order arbitration |
| `transition-conflict-reversed` | reversed writer order produces the identical conflict |
| `transition-nonfunctional-replacement` | `~>` without a proved key directs repair to exact deltas |
| `replay-deterministic` | exact authority, events, ticks, identity, and bytes replay identically |
| `requires-packages` | reserved requirements relate the Program to exact packages, not members |
| `requires-capability` | a Program capability requirement is not a runtime grant |
| `effect-render-request` | `render!` records request, authorization, attempt, and receipt |
| `effect-render-defined` | `:=` defines the receipt without changing the effect request |
| `effect-load-resource` | `load!` returns an opaque session resource with receipt provenance |
| `effect-missing-capability` | authorization denial prevents host attempt and fabricated receipt |
| `effect-postcommit-success` | rendering observes the committed post-state and records success |
| `effect-postcommit-failure` | failed rendering records failure without rolling back modeled state |
| `hospital-current-full` | current executable hospital profile retained as migration/parity evidence |
| `hospital-reset-full` | distinction-surface hospital projection with the same six query results |
| `hospital-reset-canonical-full` | canonical reset rendering preserves the same result oracle |
| `absence-undetermined` | empty existence result leaves content undetermined under open-world semantics |
| `definition-not-transition` | `:=` denotes without mutation, Delta, or successor Revision |
| `derivation-invariant-distinct` | oriented derivation and Revision admission remain separate modes |
| `goal-not-current` | desired content grants no current-truth, derivation, or effect authority |
| `referent-declaration` | declaration establishes one referent without implicit roles or constraints |
| `referent-rename` | preferred-term change preserves referent and content identities |
| `relation-described` | a relation referent participates in content without executing itself |
| `shared-content-occurrences` | two assertion occurrences share content while retaining provenance |
| `structural-equality-distinct` | equal known structure does not merge referent identity |
| `unresolved-use` | ordinary use resolves an existing referent and never declares a typo |
| `one-coin-m0` | complete target specimen for referents, relational content, derivation rules, transitions, and effects |

Do not “bless” these files from parser output. A later fixture checker may
compare exact projections and print diffs, but reviewed corpus remains the
authority.
