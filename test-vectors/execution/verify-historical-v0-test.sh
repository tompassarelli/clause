#!/usr/bin/env bash
set -euo pipefail

corpus_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT

new_case() {
  local case_dir
  case_dir="$(mktemp -d "$scratch/case.XXXXXX")"
  cp -a -- "$corpus_dir/." "$case_dir/"
  printf '%s\n' "$case_dir"
}

expect_reject() {
  local case_dir="$1"
  local expected="$2"
  local output
  if output="$(cd -- "$case_dir" && ./verify-historical-v0.sh 2>&1)"; then
    printf 'expected verifier rejection containing: %s\n' "$expected" >&2
    exit 1
  fi
  if [[ "$output" != *"$expected"* ]]; then
    printf 'wrong verifier rejection; expected %s, observed:\n%s\n' "$expected" "$output" >&2
    exit 1
  fi
}

rehash_manifest() {
  local case_dir="$1"
  local manifest_hash
  manifest_hash="$(sha256sum "$case_dir/manifest.json" | cut -d' ' -f1)"
  sed -i "s/^[0-9a-f]\\{64\\}  manifest.json$/${manifest_hash}  manifest.json/" \
    "$case_dir/SHA256SUMS"
}

rehash_payload() {
  local case_dir="$1"
  local relative_path="$2"
  local payload_hash
  payload_hash="$(sha256sum "$case_dir/$relative_path" | cut -d' ' -f1)"
  sed -i "s#^[0-9a-f]\\{64\\}  ${relative_path}\$#${payload_hash}  ${relative_path}#" \
    "$case_dir/SHA256SUMS"
}

case_dir="$(new_case)"
: > "$case_dir/unlisted.txt"
expect_reject "$case_dir" 'unexpected execution-v0 corpus path or file type'

case_dir="$(new_case)"
cp -- "$case_dir/historical-v0/source-projections/pure-dependency-closure.clause-v0.txt" \
  "$case_dir/historical-v0/source-projections/canonical-leak.clause"
expect_reject "$case_dir" 'execution-v0 corpus contains a canonical .clause path'

case_dir="$(new_case)"
ln -s -- pure-dependency-closure.clause-v0.txt \
  "$case_dir/historical-v0/source-projections/canonical-link.clause"
expect_reject "$case_dir" 'execution-v0 corpus contains a canonical .clause path'

case_dir="$(new_case)"
printf '%s\n' 'malformed checksum record' >> "$case_dir/SHA256SUMS"
expect_reject "$case_dir" 'execution-v0 checksum path set is malformed or incomplete'

case_dir="$(new_case)"
payload='historical-v0/source-projections/pure-dependency-closure.clause-v0.txt'
printf '\n' >> "$case_dir/$payload"
rehash_payload "$case_dir" "$payload"
expect_reject "$case_dir" 'execution-v0 original payload hash verification failed'

case_dir="$(new_case)"
sed -i '/^  "semantic_authority": /i\  "semantic_authority": "docs/hostile.md",' \
  "$case_dir/manifest.json"
rehash_manifest "$case_dir"
expect_reject "$case_dir" 'execution-v0 manifest is malformed or contains duplicate object keys'

case_dir="$(new_case)"
sed -i '/^    "names": /i\    "names": "hostile",' "$case_dir/manifest.json"
rehash_manifest "$case_dir"
expect_reject "$case_dir" 'execution-v0 manifest is malformed or contains duplicate object keys'

case_dir="$(new_case)"
sed -i '/^    "term_json": {$/i\    "term_json": {"hostile": {"x": 1}},' \
  "$case_dir/manifest.json"
rehash_manifest "$case_dir"
expect_reject "$case_dir" 'execution-v0 manifest is malformed or contains duplicate object keys'

case_dir="$(new_case)"
sed -i '/^  "source_projection_contract": {$/i\  "source_projection_contract": {},' \
  "$case_dir/manifest.json"
rehash_manifest "$case_dir"
expect_reject "$case_dir" 'execution-v0 manifest is malformed or contains duplicate object keys'

while IFS= read -r mutation; do
  case_dir="$(new_case)"
  jq "$mutation" "$case_dir/manifest.json" > "$case_dir/manifest.next.json"
  mv -- "$case_dir/manifest.next.json" "$case_dir/manifest.json"
  rehash_manifest "$case_dir"
  expect_reject "$case_dir" 'execution-v0 manifest authority metadata mismatch'
done <<'MUTATIONS'
.semantic_authority = "docs/other.md"
.syntax_authority = "docs/syntax.md"
.source_projection_contract.classification = "canonical"
.source_projection_contract.canonical_source_included = true
.source_projection_contract.directory = "programs"
.source_projection_contract.suffix = ".clause"
.source_projection_contract.spelling_authority = "syntax"
.source_projection_contract.byte_identity = "digest string only"
.identity.names = "slash-separated qualified names"
.identity.host_string_dispatch = "permitted"
.fixture_term_encoding.symbol_atom.canonical_payload = "slash-delimited segments"
.run_outcomes = ["returned"]
.bootstrap_dependency = "0000000000000000000000000000000000000000"
MUTATIONS

printf '%s\n' 'historical-v0 verifier adversarial probes: OK'
