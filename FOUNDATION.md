# Clause: A Distinction-First Semantic Foundation

## Authority and scope

This document is Clause's semantic authority. `SURFACE.md` defines authoring
projection, `M0.md` defines executable evidence, and `ROADMAP.md` orders
implementation. None may introduce a second semantic ontology.

Clause begins from distinguishing, not from a universal root object.
Distinguishing is the preformal act or condition by which something can be
picked out. Clause does not reify that act as a built-in `Distinction` kind and
does not require every modeled thing to inherit from one. Once a distinction is
stabilized so it can be addressed and reidentified, it is a referent.

## One domain of addressable referents

Clause has one semantic domain: addressable referents. A referent is a
stabilized distinction that can be reidentified across terms, relational
contents, assertion occurrences, judgments, and Revisions. Identity answers
whether two addresses pick out the same referent. Equality is relational
content about referents. Structural equality never collapses referent identity.

A relation is an addressable referent used in relational position. Being used
as a relation does not move it into another domain. The same relation may be
classified, described, compared, quoted, or occupy a named role in other
relational content. Merely addressing or mentioning a relation never executes
it.

Relational content is explicitly n-ary. It places one relation referent in
relational position and assigns every participant to a stable named role; a
binary or unary presentation is only a smaller arity, not a different semantic
kind. Source order, grammatical voice, focus, and target layout are projections
of those named roles and never replace them as semantic authority.

Clause permits guarded self-description: relations, rules, Revisions, and
evaluation may themselves be addressed and discussed as referents. A uniform
identity domain does not grant self-interpretation or self-execution. Treating a
referent as a relation, derivation rule, effect, or evaluator requires an
admitted shape and mode and may additionally require quotation,
stratification, or an exact Revision boundary.

## Terms are not referents

A term is a source or intermediate designator. Resolution relates a term to a
referent in a declared scope; it does not turn text into semantic identity.
Different terms may resolve to one referent, and an unresolved term may have no
referent or several candidates. Rename and formatting operations therefore do
not replace referents unless an explicit semantic edit says they do.

`x := form` is definition/denotation. It orients the term `x` to the denotation
of `form`. Definition is neither classification nor equality, and it does not
by itself assert, accept, or authorize relational content.

## Content, occurrence, and judgment

Clause keeps three layers separate:

1. **Relational content** consists of a relation referent in relational
   position and referents or terms arranged in named roles. Content is not a
   claim and carries no commitment by itself.
2. **Assertion or claim occurrence** is the source/context act that commits to
   relational content in a particular scope, Revision, source span, event, or
   provenance context. Two occurrences may commit to identical content without
   becoming one occurrence.
3. **Judgment** records a status under an authority and scope: for example
   accepted, rejected, disputed, or undetermined. A judgment targets content or
   an occurrence; it is not part of that content and cannot silently change its
   identity.

Truth, derivability, acceptance, observation, authorization, intention,
requirement, and execution are distinct modalities. A receipt proves that an
attempt was recorded, not that its intended external condition is true.
Disagreement is represented by distinct judgments and authorities, never by
duplicating or mutating the underlying relational content.

## Declaration, use, and exact resolution

A declaration position may establish a fresh addressable referent or add a name
or relation concerning an existing one. Named roles and admissibility
constraints arise only when that declaration says so. A use position must
resolve an existing referent unless the form explicitly introduces a fresh
variable or referent. Declaration and use may share a surface term, but they are
different operations and must receive different diagnostics when resolution
fails.

Resolution is exact and scoped. It may use declarations, imports, named-role
shape, surrounding form, and explicit classification constraints. It may not
choose by capitalization, source order, structural coincidence, or
probabilistic English plausibility. Ambiguity reports every surviving candidate
and the structure required to resolve it.

## Classification, definition, and equality

The constitutional source forms are:

```clause
Chess : Game
gravity := 9.81
gravity = measured gravity
```

`x : T` is classification sugar for ordinary membership relational content,
with `member` and `group` as named roles. It does not declare a primitive host
type, bind a value, or collapse identity through equality. `x := form` is
definition/denotation. `=` authors equality content. The retired `∈`, `::`,
`in`, and `member of` membership spellings are rejected contrasts, not live
aliases or editor rewrites.

## Open-world negation

Clause is open-world by default. Failure to find, derive, observe, or accept
content does not establish its negation. Explicit negative relational content,
a rejecting judgment, an incompatibility constraint, and absence of evidence
are four different things.

Closed-world reasoning is permitted only under an explicit finite scope and a
named law or operational policy. Its result retains that scope and authority;
it never becomes an unqualified global negation.

## Laws, rules, invariants, and goals

Each is an addressable referent, but their roles are not interchangeable:

- A **law** universally generalizes a relational pattern within its explicit
  scope. It does not by itself perform or authorize a derivation.
- A **derivation rule** authorizes an oriented derivation from matched premise
  content to conclusion content under named scope and authority.
- An **invariant** is content required to hold throughout a named scope or
  across a candidate Revision; violation rejects Revision admission.
- A **goal** describes desired content or orientation. A goal is not an
  assertion of current truth and is not authority to derive content or perform
  an external effect.

Syntax, lowering, or storage may share machinery among them only when their
modal distinction remains explicit in the checked representation.

## Derived relational views

Type, value, data, object, field, record, set, function, variable, state,
mutation, type checking, and evaluation are derived relational views. They are
not additional semantic domains.

- A set view requires membership and may additionally adopt scoped
  extensionality.
- A function view is a relation plus uniqueness and orientation constraints.
- A variable view is a scoped term/binder and its admissibility relations, not
  a referent species.
- A state view is relational content indexed by an exact authority and
  boundary; mutation is modeled as an exact successor, not identity change in
  place.
- Type checking is a relational admissibility proof. Evaluation is a derived
  operational process over resolved content, laws, rules, and explicit effects.

Representations become semantic only when content explicitly addresses them.
Mirroring Rust, JavaScript, database, or host-language type universes is a
non-goal. A target may specialize layouts without making its host types the
Clause Model.

## Incremental evaluation

Evaluation consumes exact deltas of referents, relational content, assertion
occurrences, judgments, and governing laws, derivation rules, invariants, and
goals. It rechecks only affected dependencies while retaining canonical
results, proof/support provenance, and complete versus incomplete status.
Retraction invalidates consequences whose supports disappear; an addition does
not erase independent supports.

Caches, indexes, schedules, fields, arrays, and compiled functions are
replaceable projections. They never become authority. Deterministic replay uses
exact starting authority, ordered inputs, and explicit effect receipts.
Externally observable effects occur only through authorized operational steps
and remain separate from semantic truth.

## Model, Revision, source, and migration

The Model consists of addressable referents, relational content, assertion
occurrences, judgments, universal laws, derivation rules, invariants, goals,
and their identity, authority, scope, and provenance relations. Source terms
and files project the Model; they are not the Model. When realized, the
canonical authoritative Model is the program. A sealed Revision is immutable
history/version evidence: a content-addressed Model snapshot with exact
lineage. It is not a synonym for the program, application, host process, or
execution.

The current typed frontend/kernel and semantic-v5 wire remain executable legacy
migration evidence. Their separate `Type`, `Entity`, `Value`, `Variable`, and
`Relation` encodings are not this constitution. A future semantic migration
must be atomic, version-breaking, and preserve audited lineage, explanation,
intervention, incremental support behavior, exact identity rules, and
source-deleted generated-code parity.

Clause currently has no Store implementation, so this document claims no Store
closure. Store is treated as a neutral persistence and query substrate, not as
Clause's semantic authority. Any future adapter requires a typed Clause envelope
above it for stable referent terms, relational content, assertion-occurrence
attestations, judgments, modality, evidence, authority, admission and rejection,
supersession, and exact Revision-to-storage-lineage links. Clause may not infer
those distinctions from structural equality, row liveness, missing rows,
retraction, query negation, or storage revision identity.

## M0 acceptance boundary

M0 must make every distinction above explicit in authority, corpus oracles,
Stage A/B behavior, diagnostics, and migration gates. Stage A stays lossless
and semantic-free. Stage B may classify source forms but may not invent
referents, judgments, authority, negation, execution, or host types. No later
parser, semantic IR, wire, Store, evaluator, or target work begins until the M0
foundation is independently reviewed and public.
