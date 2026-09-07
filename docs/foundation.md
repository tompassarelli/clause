# Clause semantic foundation

> **Authority:** This document defines Clause meaning. [Syntax](syntax.md)
> defines canonical source, [architecture](architecture.md) defines
> implementation boundaries, and the [roadmap](roadmap.md) alone reports
> implementation status.

Clause is a process-first relational language. Authors state values,
relationships, laws, permissible changes, and physical constraints. Checked
running specializes that meaning; it does not replace it.

## Design discipline

Author each independent semantic fact once. Checking, execution, queries,
explanations, optimization, and editing must consume that same meaning, not
parallel descriptions maintained by the author.

Before requiring a declaration, classification, keyword, or wrapper, apply the
deletion test:

> If we remove it, what intended meaning, checking obligation, or necessary
> disambiguation becomes inexpressible or ambiguous?

Remove source that carries only compiler bookkeeping. Derive and check facts
that follow from existing contracts. When several meanings remain possible,
require the smallest explicit distinction that selects one; never guess.

A task does not need a second entry in a `Task` registry to participate in a
`prerequisite` relation. Participation is structural: check the contracts of
the roles it uses. Membership remains ordinary data when belonging itself
matters, as in a team's roster; it neither proves nor is required for
structural conformance.

Declarations and uses share Clause's binding, focus, and constraint language.
Grouping may clarify scope or remove repetition, but cannot introduce another
declaration language. A name must identify one concept or obligation; changing
punctuation or renaming a compiler category does not repair redundant meaning.

Exercise a design in a nontrivial executable program before extending it.
Count independently authored facts and the places an ordinary change must
touch. Preserve independent choices about identity, cardinality, order,
effects, and authority wherever those choices change behavior.

## Carrier, formation, and application

Clause has one structurally neutral recursive carrier:

```text
Atom   := opaque(kind, canonical payload, equality contract)
Triple := [Term, Term, Term]
Term   := Atom | Triple
```

Triple positions have no inherent subject, operator, object, argument,
control, or truth meaning. A declared reading and checked formation give a
Term contextual meaning; structure alone does not.

Interpretation, truth-apt content, assessment, and their occurrences are distinct:

| Term | Meaning | Boundary |
| --- | --- | --- |
| **ClauseJudgment** | Interprets a neutral Term under an exact context, stance, and reading. | Neither execution nor a governed Judgment. |
| **FormationJudgment** | Establishes that a Term has a type under an exact interpretation. | Proves neither truth, authority, nor executability. |
| **Proposition** | Closed truth-apt content. | Representing it does not assert it. |
| **AssertionOccurrence** | One identified act placing proposition content under an assertive stance. | Distinguishes the act from its content. |
| **Judgment** | Immutable assessed content. | Distinct from its issuance. |
| **JudgmentOccurrence** | Identified issuance of a Judgment by an authority under a policy. | Records the issuance, not merely equal assessed content. |
| **Authorization** | A Judgment permitting one exact action and scope. | Representing earlier content does not confer permission. |

None follows merely from representing the preceding item. These are not all
proofs or stages in one automatic pipeline.

Formation and running retain their own distinctions:

- An **ApplicationForm** is a checked, closed configuration containing one
  exact RelationSchema, one exact Operator, complete named-role bindings, the
  exact eligible Mode set, and context requirements.
- An **Application** is a nominal instance of one ApplicationForm. Raw Terms,
  open patterns, and quoted forms are not Applications.
- An **Activation** is one actual engagement of an Application under one Mode
  and pinned initial context.
- A **Step** is one Mode-declared semantic carry-through between
  configurations of an Activation.
- A **Run** owns one root Activation, its child Activations, and their causal
  Step order.

An ApplicationForm may have no executable Mode and remain useful for
inspection or transformation. Forming an Application asserts, authorizes,
admits, and executes nothing. Activating equal Applications twice produces two
Activation occurrences.

## Values, roles, relations, and modes

A value carries a distinction under its declared equality. A role states what
that value means relative to a subject. A contract constrains participation;
representation determines physical encoding. These facts do not substitute
for one another.

An ordinary binary role contract gives its domain, range, and cardinality.
`one`, `maybe`, `some`, and `many` mean exactly one, at most one, at least one,
and unrestricted distinct values per subject. Cardinality counts values under
the range equality, not assertions, derivations, supports, or occurrences.
Equal repeated contract facts do not strengthen the contract; contradictory
facts reject.

Contracts sharing a domain jointly define structural participation. A
cardinality-one `duration` role over `Task` requires every participating Task
to have one value in the declared range, including values referenced by other
Task roles. Initial worlds and complete atomic successors must satisfy required
properties. An intermediate withdrawal during a replacement is not a world.

A nominal domain with no structural contracts still uses explicit membership.
Membership can restrict a role when its contract says so, but it cannot stand
in for a missing required property or confer another identity.

A **RelationSchema** fixes named roles, role contracts, and admissible binding
shape. A **RelationExtension** is a set or multiset of bindings at one exact
revision or Activation scope; row storage does not assert universal truth or
record an execution occurrence. A **Reading** maps source to exact roles. An
**Operator** defines running behavior. A **Mode** selects one schema and
declares known and produced roles, result cardinality, purity and effects,
failure, ordering, continuation, scheduling, identity, lifetime, resource,
and cost guarantees that matter for that direction.

A schema may have no Operator. An Operator may have several Modes. A relation's
meaning is therefore independent of the directions in which it can be
computed. Every executable direction states its own result and failure
guarantees. A function is a Mode proven pure, deterministic, and single-result;
a procedure is a Mode permitting effects or authoritative transition
proposals. Clause does not assume that every relation is reversible or that
every computation performs search.

Checked formation closes every required role and rejects missing, extra,
ambiguous, mistyped, or wrong-cardinality bindings. No consumer may infer a
role or Operator from Triple position, source order, graph adjacency, English
word order, or a host field name.

## Laws, derivation, and knowledge

A **law** states a universal relational or process constraint. It remains
inert until a distinct derivation authorization selects an operational
direction and scope. An invariant constrains candidate admission. A goal
describes desired content. A query requests observations. These meanings may
share matching machinery without collapsing.

Clause is open-world by default. Failure to find, derive, observe, or admit a
value does not establish its negation. Closed-world reasoning requires an
explicit finite scope and rule. Exhausting a search budget establishes neither
absence nor falsehood.

This constrains what an outcome establishes, not the return type of every
query. A Mode's result and failure contract must distinguish a completed
answer from exhaustion; it need not encode exhaustion as a third Boolean.

For a finite positive basis with extensional roots `E` and authorized rules
`R`, derived values are the least fixed point of:

```text
F(X) = E ∪ { conclusion(r) | r ∈ R and premises(r) ⊆ X }
```

Each independent root and derivation remains independent support even when
their conclusions are equal. Withdrawing one support preserves a conclusion
supported elsewhere; withdrawing the last support removes its consequences.
A positive cycle without an extensional or empty-premise root cannot support
itself. Recursive explanations retain finite alternative-support structure
rather than infinitely unfolded proof trees.

A finite derivation certificate proves a claim only relative to its exact
supplied roots and rules. It does not prove that the basis is true, accepted,
or authoritative. A bounded closure returns a complete result or explicit
exhaustion; it may not publish a prefix as complete.

The resident implementation currently realizes a bounded positive subset with
support withdrawal. Negation, aggregation, allocation in recursive rules, and
general multi-input derived relations remain outside that subset; see the
[roadmap](roadmap.md).

## Source projection and continuity

Source is a canonical bidirectional projection of meaning, not program
identity. Parsing may retain a lossless concrete tree for tokens, layout,
comments, errors, and incomplete edits.

Every source construct produces one or more independently identified semantic
emissions and one focus. Each emission retains its own source origin, stance,
formation obligations, and identity slot. Equal Terms never merge independent
emissions. A block head selects its child grammar before child semantics are
inspected; a child receives the parent's focus only when that grammar says so.
Indentation supplies containment, never an implicit domain relation.

A designation resolves within an explicit namespace and import context to one
Referent. Spelling is for people; it is not binding identity, occurrence
identity, or nominal continuity. Binders, uses, captures, recursion, shadowing,
hygiene, and rename operate on checked identities.

Printing and independent re-elaboration preserve meaning and emission
multiplicity up to a type-preserving bijection over freshly allocated
identities. An edit preserves an existing identity only when an explicit
continuity witness maps the prior identity plan to the new source and checks.
Similar text, source position, traversal order, and canonical ordinal are not
continuity evidence. Repeated equal emissions retain distinct stable slots, so
inserting one does not renumber or merge the others.

## Running

Every Activation selects one eligible Mode and records:

- the exact semantics and checked constitution;
- the ApplicationForm and static formation/executability proof;
- initial world, session, policy, budget, cancellation, and scheduler pins when
  present;
- every dynamic prerequisite in a stable named, typed, multiplicity-preserving
  slot; and
- one causal origin plus only those prerequisite occurrences that the Mode
  declares causal.

Static callability, Authorization, capability, and Admission authority are
different obligations. A Mode may declare no dynamic prerequisites; such an
Activation manufactures no Authorization or capability token. Candidate
checking can support sandboxed running and read-only use of a pinned admitted
world, but cannot fabricate an admitted constitution or constitutive
authority.

An Activation owns one affine live configuration. Anonymous internal
reductions may update local slots and scratch without a StateRevision or
Admission while intermediate state cannot escape, aliases are checked, and
failure can restore the last completed semantic boundary. A semantic Step is
required when running emits or consumes an identified occurrence, changes a
causal predecessor, exposes an observation or failure, stages a delta or
effect intent, suspends or hands off a continuation, crosses a world or
constitution pin, or reaches another Mode-declared boundary. Instructions,
loop iterations, scheduler yields, and logging are not Steps unless the Mode
makes them observable.

Each Step consumes the exact current configuration token and produces its
successor or a declared terminal settlement. Parallel mutable work requires an
explicit disjoint split; each branch receives distinct custody, and a join
consumes exactly one settlement per branch. Host interleaving supplies no
semantic order. Rollback exists only inside an unpublished Step attempt; a
completed occurrence, external act, or admitted boundary requires a new
compensating process.

Step order is the transitive closure of explicit typed cause edges and
configuration-succession edges. Encoding, registration, storage, arrival,
clock, and host scheduling order add no edge. Independent nodes remain
incomparable. A Run can return, fail, exhaust a bound, yield, suspend, stream,
or remain live; ongoing running is not a fabricated result or a third truth
value.

A Continuation is the typed semantic remainder of an Activation. Suspension
moves the sole live configuration custody into it. Resumption under unchanged
semantic pins preserves the Activation; a changed Application, Mode,
constitution, or other semantic pin creates a new causally related Activation.
A host stack frame or pointer is not sufficient boundary-crossing state.

## Change, authority, and effects

Running may produce observations, continuations, evidence, and candidate
deltas. These outputs remain independent. Only **Admission** creates an
authoritative Program, State, or other governed successor:

```text
exact base + typed candidate + evidence + authority + obligations
  -> AdmissionRequest
  -> accepted successor | typed rejection
```

An Admission request has a content-derived retry key; the decision is a nominal
AdmissionOccurrence. Repeating the same request cites the existing decision.
Changing any committed input creates a different request. Admission authority
must already be authoritative; a candidate, its successful checking, its
proposed successor, and evidence produced by the candidate cannot authorize
their own admission.

A **ProgramSnapshot** is one immutable checked constitution under one semantics
epoch. A **ProgramRevision** is one admitted lineage edge selecting a snapshot.
A **RuntimeSession** is pinned to one ProgramRevision and runtime policy. A
**StateRevision** is one admitted runtime boundary inside one session. Equal
snapshot or state content reached through different admitted histories need
not share revision identity. A live Activation never silently rebinds when a
Program or world changes.

An external effect keeps these facts separate:

```text
intent -> effect Authorization -> capability -> attempt
       -> optional receipt and observations -> optional later Admission
```

Authorization judges permission; capability proves access to a boundary or
resource. Neither proves that an attempt occurred or succeeded. A receipt
reports an outcome; it does not make the intended proposition true. Failure to
admit evidence after an attempt cannot undo the external act. Replay of a trace
performs no attempt.

A transition's conditions read one pre-state. Its withdrawals and inclusions
form one candidate delta and either commit atomically through Admission or
leave the authoritative world unchanged. Source order cannot resolve
conflicting writes.

## Identity, support, and trace

Clause distinguishes structural content, nominal continuity, and occurrences:

- structural equality compares values under a declared equality contract;
- nominal identity preserves one addressable Application, Referent, Program,
  or other declared continuity domain; and
- occurrence identity distinguishes actual Activations, Steps, assertions,
  observations, decisions, and effects.

Equal values may therefore have different occurrences and independent
supports. A content hash cannot replace a nominal or occurrence identity.
Names, paths, source spans, host handles, pointers, row numbers, and heap
addresses establish none of them.

An AllocationJudgment records either checked retention of an identity or fresh
allocation in one exact identity domain. Cross-edit, cross-snapshot, and
cross-host continuity require the corresponding typed witness. Fresh causal
identities may differ across valid physical schedules; comparison uses a typed
isomorphism preserving ownership, causal edges, slots, and observations.

A trace is a retained description of a Run, never the Run or historical act.
Trace retention is a bounded Mode or runtime-profile choice. Eviction cannot
rewrite causality; a future use of an evicted cause must rehydrate exact
evidence or reject. Retaining an identity does not require retaining its whole
physical representation.

## Static reuse and physical realization

Static parameters, constraints, and evidence are Clause meaning. Resolution
uses one finite lexical/import basis and a terminating or explicitly bounded
contract. An exhausted bound is indeterminate, not proof that an obligation is
unsatisfied. Ambient host registries, filesystem search, source order, and
cache addresses cannot choose evidence.

Checking reuse, semantic specialization, and physical artifact reuse have
different keys. Interface or argument equality can preserve checking work;
body or semantic-dependency changes invalidate specialization; target, ABI,
layout, strategy, or refinement changes invalidate physical reuse. Sharing
code never merges Applications, Activations, occurrences, or source
provenance.

Every physical allocation has exactly one reclamation root: an affine owner, a
declared region, or a foreign manager. Borrows and Leases are checked access
edges, not alternative roots. Reclamation waits for all escapes, continuations,
children, asynchronous uses, foreign uses, and explicit close obligations to
end. Observable close or disposal is an explicit process/effect, never a
destructor hidden behind lost reachability. Clause does not silently fall back
to tracing collection, reference counting, finalizers, or process-exit leaks.

The compiler may lower checked meaning to registers, structs, arrays, indexes,
state machines, native code, Wasm, JavaScript, databases, or foreign objects.
Each lowering preserves declared values, identities, observations, failures,
effects, causal boundaries, resources, and ordering. A physical choice that
affects these properties remains explicit refinement evidence. Generic graph
interpretation may be a reference path; it is not required on a production hot
path.

The semantic graph and canonical carrier contain every distinction required by
these rules. They are inspectable representations, not authority and not
running. Host code may implement a small generic evaluator, codec, scheduler,
store, and physical boundary; it may not hide the meaning of a Clause construct
in a host enum, callback, plugin, validator, or dispatch table.

`ClauseSemanticsId` is the lowercase hexadecimal SHA-256 of:

```text
UTF-8("clause/semantics-manifest/v1\n") ||
exact canonical bytes of semantics/manifest-v1.json
```

Here `\n` is one LF byte, and the manifest bytes obey its `canonical_bytes`
rule. This is not a compiler build or release identifier. Every hash-derived
identity has one domain-separated canonical preimage; digest reuse for
different bytes rejects rather than implying equality.

`ClauseSemanticsId` is a document commitment, not a semantic-compatibility
theorem. Equal identifiers establish equal canonical manifest bytes and, after
checking the manifest's content commitments, equal bytes for exactly the listed
documents. Different identifiers or document bundles establish document
difference only: they neither prove nor disprove semantic compatibility.
Compatibility between distinct bundles requires a separately stated relation
and evidence about their meanings.

The static-reuse, lifetime, general extension, complete carrier, and optimized
refinement rules are requirements, not current implementation claims. Their
current boundaries are stated in the [roadmap](roadmap.md).

## Decisive laws

- Read by querying, compute by running, and change authoritative state only by
  Admission.
- Description, assertion, judgment, authorization, execution, observation,
  effect, trace, and admission remain distinct.
- Equal values do not merge occurrences, supports, role slots, or identity.
- Structural participation requires its actual properties, not ceremonial
  registration.
- A relation contract is authored once and used by facts, patterns, checking,
  execution, queries, and editing.
- Bounded search either completes or reports exhaustion; it never turns a
  prefix into absence, uniqueness, or falsehood.
- Source layout controls grouping and focus only where its selected grammar
  says so; it never invents a domain relation.
- Physical implementations may erase or specialize checked structure only
  when all declared semantic consumers remain unchanged.
- No Term, law, package, compiler candidate, or proposed successor may
  authorize itself.
- No hidden host language may carry Clause semantics.
