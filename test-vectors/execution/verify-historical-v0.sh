#!/usr/bin/env bash
set -euo pipefail

corpus_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd -- "$corpus_dir"

sha256sum --check SHA256SUMS
sha256sum --check - <<'HASHES'
3111d312e7dfd5de210374154835d19a4f2f85053666e9005532e82a98f8f1f5  historical-v0/source-projections/pure-dependency-closure.clause-v0.txt
ceffd028d7a4c394e2d2497d2d126c6b3a25a4889aeefa7a603b62f89ba128ca  historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt
e63e9a523e007257e7a2d0abab02aae04d8a8b3de165cdab4eeb2dbc1a9cce26  historical-v0/source-projections/verified-program-evolution.clause-v0.txt
HASHES

if find . -type f -name '*.clause' -print -quit | grep -q .; then
  printf '%s\n' 'execution corpus must not expose historical v0 text as canonical .clause source' >&2
  exit 1
fi

jq --exit-status '
  .syntax_authority == null
  and .source_projection_contract.classification == "historical-v0-noncanonical-fixture"
  and .source_projection_contract.canonical_source_included == false
  and ([.programs[].historical_source_projection] == [
    "historical-v0/source-projections/pure-dependency-closure.clause-v0.txt",
    "historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt",
    "historical-v0/source-projections/verified-program-evolution.clause-v0.txt"
  ])
  and all(.programs[]; has("source") | not)
' manifest.json >/dev/null
