# Scheduling example

This example consumes `clause:browser/runtime` and the authored schedule in
`clause:test-vectors/authoring/scheduling.clause`. Install the runtime package's
development dependencies before running `bun run typecheck` or `bun test
./src/scheduling-view-test.ts` from `clause:browser/scheduling`.

`bun run serve` starts the existing scheduling server. It consumes the native
source-server and Wasm artifacts under `clause:target/scheduling-live-workbench`
and `clause:target/scheduling-browser-wasm`. The browser journey remains available
through `bun run test:browser` once those artifacts and the server are running.
