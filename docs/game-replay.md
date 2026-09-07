# Replay a game input trace

The native recorder compiles the supplied Clause source, captures its accepted
initial allocation, and executes an explicit input trace in one resident
session. The Wasm runner opens that same allocation and sends the same CWI1
commands to one fresh Wasm session. It compares the initial projection and every
exact CSE1 event, including candidate, authorization, admission, and disposal.
An admission comparison includes its projected state and all event identities.
No values are rounded, identities replaced, or failed commands skipped.

The example uses the actual encounter in
`clause:test-vectors/authoring/live-encounter.clause`. The input file
`clause:test-vectors/browser/live-encounter-replay.txt` starts an encounter,
moves, attacks, heals, raises a ward, and advances explicit 16/32 ms ticks.
Gameplay is executed exclusively from that source.

From an owned Clause worktree with its Rust toolchain and Bun available:

```sh
clause_replay_root="$PWD"
cargo run --manifest-path "$clause_replay_root/Cargo.toml" -p clause-workbench --example game-replay --locked -j 2 -- \
  "$clause_replay_root/test-vectors/authoring/live-encounter.clause" \
  "$clause_replay_root/test-vectors/browser/live-encounter-replay.txt" \
  "$clause_replay_root/target/game-replay"
```

The output directory must be new. It contains the exact source, inputs, initial
session request, compiler source metadata, and numbered command/event pairs.
Keep that directory to reproduce a suspected runtime regression without
recompiling or rerecording the game. Build the runtime from the same compiler
revision first; a subsequent runtime under investigation can consume the record.

```sh
cargo build --manifest-path "$clause_replay_root/Cargo.toml" -p clause-runtime --target wasm32-unknown-unknown --release --locked -j 2
wasm-bindgen "$clause_replay_root/target/wasm32-unknown-unknown/release/clause_runtime.wasm" --target web --out-dir "$clause_replay_root/target/game-replay-wasm"
bun "$clause_replay_root/browser/runtime/src/game-replay.ts" \
  "$clause_replay_root/target/game-replay" \
  "$clause_replay_root/target/game-replay-wasm/clause_runtime.js"
```

Use the repository's pinned Rust version, wasm32 standard library, and
wasm-bindgen 0.2.108. Apply the shared capacity helper to builds. A matching
trace reports 14 inputs, 29 events, and seven admissions with exit status zero.
A mismatch exits one and reports the first differing input index (zero-based),
command, phase, byte offset, expected/actual field, exact entity referents when
available, and compiler state coordinates alongside the preserved source path.
The initial state has a null input index. Runtime rejections and malformed
events remain errors with their original status or decoder diagnostic.

Input lines are `key CODE down|up`, `scalar CHANNEL NUMBER`, or
`tick MILLISECONDS`; blank lines and `#` comments are ignored. Each tick produces
one candidate, one separate authorization, and one admission. Physical inputs
retain their order and explicit values. If a game consumes random values,
record them as the source's explicit scalar inputs. This recorder currently
accepts keyboard/scalar/tick inputs, not referent picking or external-effect
receipts. It starts from authored initial state, not an arbitrary mid-game
checkpoint. One runner process owns one isolated Wasm replay.

The focused proof runs the real trace, then changes only an expected canonical
vitality observation in a private copy of the record. It requires the first
mismatch at input 7, admission command 13, `cinder-1.vitality`, with exact entity
and source-state context:

```sh
bun test "$clause_replay_root/browser/runtime/src/game-replay-test.ts"
```

This is a bounded regression reproducer, not a networking or rollback system.
Exact event comparison also checks identities for state outside the declared
projection; field-level explanations are limited to that projection and the
existing compiler metadata. Passing this trace does not establish parity for
unrecorded inputs or other games.
