#!/usr/bin/env bash
set -euo pipefail

corpus_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd -- "$corpus_dir"

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

canonical_source_entry="$(find . -mindepth 1 -name '*.clause' -print -quit)"
if [[ -n "$canonical_source_entry" ]]; then
  fail 'execution-v0 corpus contains a canonical .clause path'
fi

expected_tree="$(LC_ALL=C sort <<'TREE'
d historical-v0
d historical-v0/source-projections
f README.md
f SHA256SUMS
f historical-v0/source-projections/pure-dependency-closure.clause-v0.txt
f historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt
f historical-v0/source-projections/verified-program-evolution.clause-v0.txt
f manifest.json
f verify-historical-v0-test.sh
f verify-historical-v0.sh
TREE
)"
actual_tree="$(find . -mindepth 1 -printf '%y %P\n' | LC_ALL=C sort)"
if [[ "$actual_tree" != "$expected_tree" ]]; then
  fail 'unexpected execution-v0 corpus path or file type'
fi

expected_checksum_paths="$(LC_ALL=C sort <<'PATHS'
README.md
historical-v0/source-projections/pure-dependency-closure.clause-v0.txt
historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt
historical-v0/source-projections/verified-program-evolution.clause-v0.txt
manifest.json
verify-historical-v0-test.sh
verify-historical-v0.sh
PATHS
)"
actual_checksum_paths="$(sed -n 's/^[0-9a-f]\{64\}  //p' SHA256SUMS | LC_ALL=C sort)"
if [[ "$(wc -l < SHA256SUMS)" -ne 7 || "$actual_checksum_paths" != "$expected_checksum_paths" ]]; then
  fail 'execution-v0 checksum path set is malformed or incomplete'
fi

if ! sha256sum --check --strict SHA256SUMS >/dev/null; then
  fail 'execution-v0 tracked checksum verification failed'
fi

if ! sha256sum --check --strict - >/dev/null <<'HASHES'
3111d312e7dfd5de210374154835d19a4f2f85053666e9005532e82a98f8f1f5  historical-v0/source-projections/pure-dependency-closure.clause-v0.txt
ceffd028d7a4c394e2d2497d2d126c6b3a25a4889aeefa7a603b62f89ba128ca  historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt
e63e9a523e007257e7a2d0abab02aae04d8a8b3de165cdab4eeb2dbc1a9cce26  historical-v0/source-projections/verified-program-evolution.clause-v0.txt
HASHES
then
  fail 'execution-v0 original payload hash verification failed'
fi

if ! jq --exit-status '
  (.schema == "clause-execution-corpus-v0")
  and (.semantic_authority == "docs/foundation.md")
  and (.syntax_authority == null)
  and (.contract == "docs/execution-corpus.md")
  and (.bootstrap_dependency == "2ea651db7c525249c465dceb0f8c5474d635fae6")
  and (.source_projection_contract == {
    "classification": "historical-v0-noncanonical-fixture",
    "canonical_source_included": false,
    "directory": "historical-v0/source-projections",
    "suffix": ".clause-v0.txt",
    "spelling_authority": "none",
    "byte_identity": "the three projection contents retain their original frozen SHA-256 values"
  })
  and (.identity == {
    "names": "fixture-local opaque transport strings supplied by the admission context; slash is an uninterpreted payload byte",
    "public_program_snapshot_id": "unratified",
    "public_program_revision_id": "unratified",
    "host_string_dispatch": "forbidden"
  })
  and (.fixture_term_encoding == {
    "structural_index": {
      "universe_id": "10",
      "semantics_id": "11"
    },
    "symbol_atom": {
      "kind": "e0",
      "canonical_payload": "exact UTF-8 bytes of the manifest string",
      "equality_contract": "e1"
    },
    "term_json": {
      "symbol": {
        "symbol": "exact fixture-local string"
      },
      "triple": {
        "triple": ["Term", "Term", "Term"]
      }
    },
    "binary_fact": "neutral Triple [symbol(subject), symbol(relation), symbol(object)]",
    "ground_fact_shorthand": "[subject, relation, object] lowers each string with symbol_atom and then forms binary_fact",
    "claim": {
      "term": "encoded fixture Term",
      "type": "symbol(fixture/type/proposition)",
      "mode": "symbol(fixture/mode/asserted)"
    },
    "authority": "candidate corpus representation only; grants no Atom-contract admission or semantic truth"
  })
  and (.run_outcomes == [
    "returned",
    "choices",
    "yielded",
    "suspended",
    "failed",
    "exhausted"
  ])
  and ([.programs[] | {id, historical_source_projection}] == [
    {
      "id": "pure-dependency-closure",
      "historical_source_projection": "historical-v0/source-projections/pure-dependency-closure.clause-v0.txt"
    },
    {
      "id": "state-effect-fulfillment",
      "historical_source_projection": "historical-v0/source-projections/state-effect-fulfillment.clause-v0.txt"
    },
    {
      "id": "verified-program-evolution",
      "historical_source_projection": "historical-v0/source-projections/verified-program-evolution.clause-v0.txt"
    }
  ])
  and (all(.programs[]; has("source") | not))
' manifest.json >/dev/null
then
  fail 'execution-v0 manifest authority metadata mismatch'
fi
