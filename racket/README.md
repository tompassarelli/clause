# Clause Racket foundation

Clause phase one uses ordinary Racket for exact source reading, sentence-shape
elaboration, spans, and diagnostics. Typed Racket owns admitted relations with
named typed roles, their exact mixfix sentence and finite modes, complete role
maps, immutable `Model`, `Revision`, and `Branch` values, checked plans, and
proof contracts.

```clause
relation catalog/contains(set: Text, member: Text):
    sentence: {set} contains {member}
    mode set -> member: many

model catalog:
    "letters" contains "a"
    "letters" contains "b"

query catalog:
    ?member where "letters" contains ?member
```

The sole finite `mode` declares `set` known, `member` sought, and `many`
cardinality. The query opens `member`; planning selects and checks that declared
mode, so callers do not supply an operational mode value. Model clauses are
bare and must match the declared sentence exactly.

Revision identity is `rev-sha256-` plus SHA-256 of canonical UTF-8 JSON for the
versioned semantic array encoding. Arrays, sorted named-role pairs, sorted
facts, and JSON string escaping make the bytes host-neutral and deterministic;
authoring text and source spans are deliberately absent.

The persisted revision and query result use array-only wire forms. A query
result is `clause-query-output-v1` with `results` and `proofs` entries; proof
records carry their stable identity, relation, and sorted role values without
host-specific object keys.

The repository includes a POSIX bootstrap launcher. It downloads the official
minimal CS archive into `${XDG_CACHE_HOME:-$HOME/.cache}/clause/racket/9.3`, checks
the recorded SHA-256 before extraction, installs the release-9.3
`typed-racket-lib` dependency set in an isolated `PLTUSERHOME`, and then runs
Clause. The archive and packages are reused on later invocations:

```sh
./bin/clause --canary
```

The launcher requires `curl`, `sha256sum`, `tar`, and `awk`; it never uses the
host Racket or a system/Nix installation. Set `CLAUSE_PLTUSERHOME` to choose a
different durable user-package cache. `./bin/clause --version` reports the
bootstrapped runtime without running Clause.

To invoke the one focused explore canary with an already-installed exact
Racket 9.3 executable directly:

```sh
RACKET=/absolute/path/to/racket-9.3/bin/racket
"$RACKET" clause.rkt --canary
```

The executable entrypoint rejects every Racket version other than 9.3.

The M3 operation canary uses the same exact fixture with two trailing
operations:

```clause
claim catalog:
    "letters" contains "c"

require catalog:
    "letters" contains "c"
```

`claim` is a pure admission from an immutable `Branch` to a fresh successor
`Revision`; a duplicate returns the original branch and the deterministic
`claim.duplicate` diagnostic. `require` is read-only exact membership: before
the claim it emits `require.unsatisfied`, and afterward it returns the matching
proof. Both outputs use the array-only `clause-claim-output-v1` and
`clause-require-output-v1` forms from the parity contract. The canary also
deletes the authoring source, reloads the post-claim revision, rejects identity
and role-map tampering, and checks generated-Racket/interpreter parity:

```sh
"$RACKET" m3-canary.rkt --canary
```

It admits a temporary authoring file, preserves every authored role and open
value span through elaboration, deletes the file, strictly reloads and rejects
identity and role-map tampering, repeats declared-mode query/proof evaluation,
then executes a generated Racket plan and compares its result and proofs with
the Typed Racket interpreter. JavaScript and TypeScript are neither dependencies
nor targets.
