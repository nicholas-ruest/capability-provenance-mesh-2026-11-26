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
  case "$project" in
    autogenous) responsibility="bounded capability-plan synthesis" ;;
    rugraph) responsibility="effect-graph provenance traversal" ;;
    agenticow) responsibility="typed agent-tool contract execution" ;;
    watermark) responsibility="generated-text provenance detection" ;;
    latentmesh) responsibility="constrained provenance-envelope transport" ;;
    skillspector) responsibility="agent-skill policy inspection" ;;
    cosign) responsibility="keyless artifact identity verification" ;;
  esac
  for path in success failure; do
    started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"; log="$evidence/integration-$project-$path.log"
    candidate_sha="$(printf 'integration-candidate:%s:%s' "$project" "$path" | sha256sum | awk '{print $1}')"
    CPM_UPSTREAM_INPUT="$(printf '{\"responsibility\":\"%s\",\"operation\":\"integration-%s-%s\",\"semantic_fields\":[\"integration-candidate\",\"1.0.0\",\"%s\"]}' "$responsibility" "$project" "$path" "$candidate_sha")"
    CPM_UPSTREAM_INPUT_SHA256="$(printf '%s' "$CPM_UPSTREAM_INPUT" | sha256sum | awk '{print $1}')"
    export CPM_UPSTREAM_INPUT CPM_UPSTREAM_INPUT_SHA256
    if "$harness/$project-$path.sh" "$upstreams" >"$log" 2>&1; then status=passed; else status=failed; fi
    sha="$(sha256sum "$log" | awk '{print $1}')"
    printf '{"project":"%s","path":"%s","status":"%s","started_at":"%s","candidate_input_sha256":"%s","log":"%s","log_sha256":"%s"}\n' "$project" "$path" "$status" "$started" "$CPM_UPSTREAM_INPUT_SHA256" "$(basename "$log")" "$sha" >>"$trace"
    [[ "$status" == passed ]] || { cat "$log" >&2; exit 1; }
  done
done
