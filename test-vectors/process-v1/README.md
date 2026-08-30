# Clause process-v1 carrier corpus

This directory freezes the first executable process-v1 carrier specimens for
the process semantics in `clause:docs/foundation.md`. It does not change the
frozen v0 canonical-package or execution corpora.

Each `.hex` file contains lowercase hexadecimal octets plus ASCII whitespace.
Whitespace is transport formatting and is not carrier content. The decoded
value is inert candidate data. Strict decoding, structural formation, or
possession of an identity token grants no execution or admission authority;
`clause-package` must replay the value against the already-authoritative
Program revision or irreducible root-policy anchors carried in its prelude.

All cases use one neutral `Term = Atom | RawTriple` carrier. Triple slots have
no operator or role meaning. Declaration identities are typed exact
snapshot-local references. Nominal Applications, Activations, Runs, Steps,
Configurations, Continuations, Observations, candidate deltas, Admissions,
and State revisions remain disjoint even though this fixture transport uses
the same 32-octet width for their opaque components.

The positive cases establish:

- one nominal Application can be root-activated twice with distinct
  Activation and Run identities;
- one Activation progresses, suspends, and resumes across several causally
  explicit Steps while preserving its identity and Run membership;
- a pure return emits an Observation and creates no State revision;
- a candidate State delta remains non-authoritative; and
- only a separate Admission under an already-authoritative typed
  authorization creates its exact successor State revision.

The negative cases reject before allocating the proposed Step or takeup:

- a resumption with a changed Program-revision pin;
- a Step naming itself;
- an acyclic Step naming a later, unconstituted Step; and
- two proposed Steps whose explicit PriorStep edges form an indirect cycle.

`manifest.json` fixes decoded byte lengths and verdicts. `SHA256SUMS` protects
the exact `.hex` transport files as corpus-handling evidence only; a digest
never substitutes for exact carrier bytes, typed identity, causal support, or
authority.
