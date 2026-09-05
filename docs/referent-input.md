# Checked referent input

An external input carries an exact occurrence, not its class or display name.
The compiled example `test-vectors/authoring/referent-input-transition.clause`
declares two items of one class, selects them independently, and advances only
the selected items with the ordinary quantified `on tick` handler.

```clause
bind referent-input Pick as Item to select-item

on select-item ?item ?target
  when
    ?item selected ?prior
    ?item = ?target
  withdraw
    ?item selected ?prior
  include
    ?item selected true
```

The channel's domain must agree with the argument's checked relational use,
assignment value, or equality with a quantified subject. The target is an
external one-argument handler. A typed argument can independently supply a
relational subject: `?picked balance ?balance` guards the exact physical input,
while another quantified subject stores that referent as ordinary state.
Declared nominal scalar state values retain typed identity, including values selected
at runtime and used as downstream subjects. They are no longer projected as
designation strings; passive consumers resolve them against projected identity,
not a guessed class/name mapping.

The projection records the declared domain facet of each checked state
subject/relation binding alongside ordinary fields. One domain uses `$referent`; multiple declared domain facets use
`$referents`, an object keyed by decimal domain ID. Each facet retains its own
checked domain and the same declared occurrence identity. Choose the projected
facet matching the input channel; do not change a reference's domain to coerce
it. A declared reference is
`{kind:"referent", domain:<u32>, identity:{kind:"declared", value:<u32>}}`.
Created references retain all 32 identity bytes with `kind:"created"` and a
byte array. Neither numeric field is a gameplay enum; transport the entire
projected value unchanged. Domain mismatch, wrong value kind, malformed
identity, and identities unknown to the current world reject before execution.

The same admitted projection exposes `$referent-inputs`, the checked map from
source-declared channel to numeric domain ID. A passive consumer uses
`frame["$referent-inputs"][channel]`, then either the subject's `$referent`
with that exact domain or `subject["$referents"][String(domain)]`. For example,
the contribution specimen exposes channels `Choose` (Account) and `Select`
(Contributor); one subject inhabits both domains. The map is derived from the
checked physical input plan and travels in the admitted frame, not a host
registry. Retain this metadata, reference, and generation from the same frame;
never hardcode numeric IDs, guess the domain from another subject, parse source
in the host, or trial-submit facets. Unknown channels or missing facets are
not valid inputs.

The browser foreign adapter accepts an input envelope containing:

```json
{"kind":"referent-input","channel":"Pick","generation":7,"value":{"kind":"referent","domain":42,"identity":{"kind":"declared","value":19}}}
```

The numbers above are illustrative. Capture the reference and active workbench
generation from the same admitted frame. `workbench.snapshot().generation`
(also exported as `workbenchsnapshot-generation`) is the workbench generation
passed to the cartridge's `startSession`. It is **not** the runtime Wasm handle
generation, a server file revision, or a pending reload number. The adapter
checks the captured workbench generation before constructing a CWI1 command;
CWI1 independently checks its captured runtime handle and sequence.

A consumer with a separate external generation must first reject messages
whose captured external generation differs from the active external generation,
then use the workbench generation paired with that frame. Never relabel old
input with the latest generation after a reload. Unchanged/rejected source
may leave the active workbench generation intact while server revision numbers
change. A physical-input message must carry its capture token explicitly.

Native callers use
`session.apply_typed_physical_input(captured_runtime_session, &source, Some(value))`
with `source = ExecutableInputSourceV1::Referent { channel }`,
`value = ExecutableValueV1::Referent(reference)` and the runtime-session token
captured beside the projected referent. `projected_referent_value_v1` decodes
an exact projected term leaf. The resident source API likewise takes the
captured `WasmSessionHandleV1` in `apply_physical_input` and never accepts a
caller-selected executable entry. Physical input does not admit state.
`tick_to_candidate` and `admit` remain separate operations.

Existing scalar/keyboard wire tags retain their meaning. CWI1 adds source tag
2 (referent channel), value tag 2 (domain u32, identity tag 0 + declared u32 or
tag 1 + 32 created bytes). Rust `WasmSessionPhysicalInputV1` now uses
`value: Option<ExecutableValueV1>` instead of `scalar_value_bits`.

Consumer adoption requires rebuilding both the native runtime and the Wasm
from the exact new compiler pin; old prebuilt Wasm does not support this tag.
Use the pinned repository Rust toolchain with the wasm32 target and exact
wasm-bindgen CLI 0.2.108. All build output belongs in the owning lane.

On Nix, use the repository flake's `rust-overlay` toolchain with only the target
added: `(pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override
{ targets = [ "wasm32-unknown-unknown" ]; }`, alongside `pkgs.stdenv.cc`.
The unmodified default development shell does not include this foreign target.

Focused commands, inside the pinned bounded development shell:

```sh
cargo test -p clause-workbench --test resident_source --locked -j 2
cargo run -p clause-workbench --example referent-input-artifact --locked -j 2
cargo build -p clause-runtime --target wasm32-unknown-unknown --release --locked -j 2
wasm-bindgen target/wasm32-unknown-unknown/release/clause_runtime.wasm --target web --out-dir target/referent-wasm
cd browser/jump-arena-shell
bun run typecheck
bun test ./src/referent-input-test.ts
```

Timer-free sources retain their physical bindings and can enter through
persistent input custody. Their CWR1 default specimen sequence may be empty;
an empty sequence without a checked physical event plan remains invalid.
An event-only source's physical tick includes an effect-free checkpoint so an
unchanged initial frame or pending input effects can produce a hidden candidate.
This does not invent a source timer or invoke an unrelated input handler.

## Explicit simultaneous contributions

`test-vectors/authoring/selected-account-contributions.clause` is a compiled,
non-game specimen. `targeted-party-contributions.clause` retains the actual
consumer's selected-party attack; `targeted-party-attack-conflict.clause` keeps
the original overlapping replacement counterexample.

```clause
on contribute ?contributor
  when
    ?contributor selected true
    ?contributor contribution ?amount
    ?contributor cooldown ?cooldown
    ?cooldown <= 0.0
    controller chosen account ?account
    ?account balance ?balance
    ?account enabled true
  withdraw
    ?contributor cooldown ?cooldown
  include
    ?contributor cooldown 1.0
  accumulate
    ?account balance ?amount
```

Each eligible finite relational occurrence contributes once. All predicates,
ordinary assignment values, and deltas observe the same pre-step state. Deltas
are collected per exact numeric target, sorted by finite F64 total order, and
added to its prior value in that canonical order. Non-finite inputs/results
reject atomically. This is deterministic floating-point accumulation, not exact
real arithmetic. To subtract, contribute `0.0 - ?amount`. `accumulate` can stand
alone without `include`; the target must be a present cardinality-one numeric
state, including a declared numeric structured field.

Two matching ordinary replacements/removals of the same target reject the
entire step (`ConflictingStateEffects` natively), even if their values happen
to agree. Mixing contribution and ordinary replacement/removal also rejects.
No contribution or contributor cooldown is installed on error. Separate
physical occurrences and the declared tick-handler sequence remain separate
steps; accumulation does not reorder them. Compiler-owned Boolean derivations
compute one disjunction-of-guards value, not a competing fallback assignment.

CPP1 adds expression tag 24 as an assignment-root additive effect. It is invalid
as a predicate or nested value expression. Consumers must rebuild the exact
native runtime, Wasm and passive adapter; old runtimes reject the new tag.
The current additive strategy covers finite declared state specialization,
not runtime-created relation-table aggregation or arbitrary recursive queries.

The current source strategy specializes quantified handlers over finite
declared state subjects, with runtime guards for selection. It does not claim
general collection strategy synthesis, recursive relations, or iteration over
arbitrary runtime-created collections. Created identity transport is supported
and checked against current relational rows; the existing relational handler
path can consume those values. General collection execution remains separate
language work. No game designation or class spelling selects this input path.
