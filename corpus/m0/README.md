# Clause M0 contrast corpus

This directory is evidence for the [M0 contract](../../M0.md). It deliberately
contains unresolved source candidates. Nothing here claims that the current
parser accepts the source or that a candidate is canonical.

Every case consists of `cases/ID.clause` and
`cases/ID.expected.json`. Expected JSON uses
`clause-m0-contrast-v1`.

`source.lines` stores raw lines without their line breaks.
`source.line_break` and `source.ends_with_line_break` make exact byte
reconstruction mechanical. `decision` is one of:

- `fixed-invariant` — the stated semantic boundary is direction now; or
- `unresolved` — the source is a contrast candidate whose admission remains
  an M0 decision; or
- `implemented-legacy` — the source is current executable migration evidence,
  not a target candidate.

`required` and `forbidden` are normative even for unresolved candidates.
`elaborated` is an abstract role graph independent of surface spelling.
Handles beginning with `$` are corpus symbols, not Store identities.

| Case | Purpose |
| --- | --- |
| `focus-relation-four` | controlling focus semantics with four-space projection |
| `focus-relation-two` | same focus semantics with two-space projection |
| `expanded-member-of` | worded membership candidate with expanded co-equal claims |
| `expanded-symbolic-membership` | symbolic membership candidate with the same graph |
| `enumeration-four` | child-to-heading membership with four-space projection |
| `enumeration-two` | same enumeration semantics with two-space projection |
| `binding-shape` | homogeneous colon bindings stay structural |
| `legacy-colon-binding` | `thing: Space` is never membership |
| `focused-colon-binding` | possible projection binding remains unresolved and is not `state locked` |
| `relation-schema-connects` | compact three-role schema and operational projection candidate |
| `functional-contract-position` | focused binary relation contract, never a field |
| `claim-focused-connects` | focus supplies the declared `door` role |
| `claim-expanded-connects` | expanded claim has the same role graph |
| `structural-connects` | explicit role-labelled escape candidate round-trips |
| `ambiguous-connects` | overlapping exact shapes name both candidates and role conflict |
| `missing-focus-connects` | root fragment diagnoses the absent focus role |
| `definition-distance` | colon-bound pattern and body are recursive term trees |
| `binding-position-projection` | binding value is a one-valued relation projection |
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
| `select-correlated` | repeated internal hole correlates clauses but is not projected |
| `any-exists` | existential query returns Bool, never a random witness |
| `select-one` | exact-one cardinality contract |
| `select-first` | canonical-first selection is deterministic |
| `law-inferred` | conclusion plus `if` body infers a positive law |
| `law-labelled` | optional colon-bound label leaves semantic law identity intact |
| `revision-withdrawal` | exact-base successor with one signed withdrawal |
| `revision-withdrawal-renamed` | human rename leaves semantic Revision unchanged |
| `revision-withdrawal-legacy` | current profile preserves withdrawal semantics for migration |
| `revision-addition` | exact-base successor with one signed admission |
| `revision-addition-legacy` | current profile preserves admission semantics for migration |
| `revision-mixed-delta` | admission and withdrawal commit as one atomic successor |
| `revision-unknown-base` | unresolved base prevents elaboration |
| `revision-withdraw-missing` | withdrawal must name an assertion in the exact base |
| `revision-admit-existing` | admission must be absent from the exact base |
| `revision-overlap` | one clause cannot be both admitted and withdrawn |
| `diff-revisions` | diff preserves exact base-to-successor direction |

Do not “bless” these files from parser output. A later fixture checker may
compare exact projections and print diffs, but reviewed corpus remains the
authority.
