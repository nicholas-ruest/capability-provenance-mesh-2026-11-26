#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
upstreams="${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
evidence="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
harness="$root/crates/cpm-service/upstream-harness"; mkdir -p "$upstreams" "$evidence"
scripts/install-upstreams.sh "$upstreams"
export CPM_SKILLSPECTOR_BIN="$upstreams/.venv-skillspector/bin/skillspector"
trace="$evidence/integrations-trace.ndjson"; : >"$trace"
for project in autogenous rugraph agenticow watermark latentmesh skillspector cosign; do
  for path in success failure; do
    started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"; log="$evidence/integration-$project-$path.log"
    if "$harness/$project-$path.sh" "$upstreams" >"$log" 2>&1; then status=passed; else status=failed; fi
    sha="$(sha256sum "$log" | awk '{print $1}')"
    printf '{"project":"%s","path":"%s","status":"%s","started_at":"%s","log":"%s","log_sha256":"%s"}\n' "$project" "$path" "$status" "$started" "$(basename "$log")" "$sha" >>"$trace"
    [[ "$status" == passed ]] || { cat "$log" >&2; exit 1; }
  done
done
