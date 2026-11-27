#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
upstreams="${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
evidence="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
harness="$root/crates/cpm-service/upstream-harness"; out="$evidence/ablation-results.ndjson"; : >"$out"
cargo test --locked -p cpm-application tests::denial_never_admits -- --exact
export CPM_SKILLSPECTOR_BIN="$upstreams/.venv-skillspector/bin/skillspector"
for project in autogenous rugraph agenticow watermark latentmesh skillspector cosign; do
  log="$evidence/ablation-$project.log"
  if "$harness/$project-failure.sh" "$upstreams" >"$log" 2>&1; then status=passed; else status=failed; fi
  printf '{"ingredient":"%s","asserted_failure_path":"%s","status":"%s"}\n' "$project" "$project-failure.sh" "$status" >>"$out"
  [[ "$status" == passed ]] || { cat "$log" >&2; exit 1; }
done
cat "$out"
