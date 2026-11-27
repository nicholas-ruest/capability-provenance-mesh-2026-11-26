#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
upstreams="${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
evidence="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
harness="$root/crates/cpm-service/upstream-harness"; mkdir -p "$upstreams" "$evidence"
scripts/install-upstreams.sh "$upstreams"
export CPM_SKILLSPECTOR_BIN="$upstreams/.venv-skillspector/bin/skillspector"
trace="$evidence/integrations-trace.ndjson"; : >"$trace"

make_input() {
  local project=$1 path=$2 variant=$3 responsibility=$4 output=$5
  python3 - "$project" "$path" "$variant" "$responsibility" "$output" <<'PY'
import hashlib,json,sys
project,path,variant,responsibility,output=sys.argv[1:]
artifact=f"capability-provenance-mesh:{project}:{path}:{variant}".encode()
manifest=json.dumps({"api_version":"v1","name":f"{project}-candidate","entrypoint":"run","capabilities":["blob.read"]},separators=(",",":"))
candidate={"plugin":f"{project}-candidate","version":"1.0.0","artifact_sha256":hashlib.sha256(artifact).hexdigest(),"manifest_sha256":hashlib.sha256(manifest.encode()).hexdigest(),"artifact":list(artifact),"manifest_json":manifest,"declared_capabilities":["blob.read"],"observed_effects":["blob.read"]}
operation={"responsibility":responsibility,"operation":f"integration-{project}-{path}-{variant}","semantic_fields":[project,path,variant],"candidate":candidate}
open(output,"w",encoding="utf-8").write(json.dumps(operation,separators=(",",":")))
PY
}

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
    CPM_UPSTREAM_INPUT_FILE="$evidence/input-$project-$path.json"; make_input "$project" "$path" a "$responsibility" "$CPM_UPSTREAM_INPUT_FILE"
    CPM_UPSTREAM_INPUT_SHA256="$(sha256sum "$CPM_UPSTREAM_INPUT_FILE" | awk '{print $1}')"; export CPM_UPSTREAM_INPUT_FILE CPM_UPSTREAM_INPUT_SHA256
    if "$harness/$project-$path.sh" "$upstreams" >"$log" 2>&1; then status=passed; else status=failed; fi
    sha="$(sha256sum "$log" | awk '{print $1}')"
    printf '{"project":"%s","path":"%s","status":"%s","started_at":"%s","candidate_input_sha256":"%s","log":"%s","log_sha256":"%s"}\n' "$project" "$path" "$status" "$started" "$CPM_UPSTREAM_INPUT_SHA256" "$(basename "$log")" "$sha" >>"$trace"
    [[ "$status" == passed ]] || { cat "$log" >&2; exit 1; }
  done
  mutation_log="$evidence/integration-$project-candidate-mutation.log"
  CPM_UPSTREAM_INPUT_FILE="$evidence/input-$project-mutation.json"; make_input "$project" success b "$responsibility" "$CPM_UPSTREAM_INPUT_FILE"
  CPM_UPSTREAM_INPUT_SHA256="$(sha256sum "$CPM_UPSTREAM_INPUT_FILE" | awk '{print $1}')"; export CPM_UPSTREAM_INPUT_FILE CPM_UPSTREAM_INPUT_SHA256
  "$harness/$project-success.sh" "$upstreams" >"$mutation_log" 2>&1
  test "$(sha256sum "$mutation_log" | awk '{print $1}')" != "$(sha256sum "$evidence/integration-$project-success.log" | awk '{print $1}')"
done
