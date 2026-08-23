# Clause Rust host

The Rust host implements the M4 intent journey and persists canonical
`Revision` JSON. Its launcher resolves `rust/` before invoking Cargo, so it is
safe to call from the repository root or another working directory.

From the repository root:

```sh
revision=$(mktemp)
./rust/bin/clause e2e racket/m4.clause "$revision"
./rust/bin/clause query "$revision"
rm -f "$revision"
```

`e2e` seals the source, selects its intent, admits and persists the successor
Revision, and checks generated-Rust parity. `query` reloads only that persisted
Revision; the source file is not needed.
