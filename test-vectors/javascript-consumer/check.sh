#!/usr/bin/env bash
set -euo pipefail

consumer_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
compiler="${1:?Pass the clause-workbench executable path}"
bun="${BUN:-bun}"

mkdir -p "$consumer_dir/generated"
"$compiler" compile-js "$consumer_dir/../authoring/foreign-cli.clause" "$consumer_dir/generated/foreign-cli.js"
"$bun" "$consumer_dir/node_modules/typescript/lib/tsc.js" --project "$consumer_dir/tsconfig.json"
"$bun" "$consumer_dir/consumer.ts"
