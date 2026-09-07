# Clause browser runtime

Generic Wasm ports, passive workbench composition, transport observation, and
browser conformance tests live here. Compiler-owned byte fixtures and their Rust
producers live in `clause:test-vectors/browser` and `clause:crates` respectively.
The scheduling example lives in `clause:browser/scheduling`.

From `clause:browser/runtime`, run `bun install --frozen-lockfile`, `bun run build`,
and `bun run test`. The checked-in Wasm artifacts support these boundary tests;
source-transfer and source-semantics journeys use the separately built artifacts
described in the corresponding `clause:docs` guides.
