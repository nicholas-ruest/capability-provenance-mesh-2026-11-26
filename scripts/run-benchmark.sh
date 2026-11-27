#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
out="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}/benchmark-results.ndjson"; : >"$out"
for iteration in 1 2 3 4 5; do
  begin="$(date +%s%N)"
  cargo test --locked -p cpm-application tests::primary_and_restart_retrieval -- --exact >/dev/null
  end="$(date +%s%N)"
  printf '{"iteration":%s,"elapsed_ns":%s,"test":"cpm_application::tests::primary_and_restart_retrieval"}\n' "$iteration" "$((end-begin))" >>"$out"
done
cat "$out"
