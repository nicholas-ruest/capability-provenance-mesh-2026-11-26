#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
: "${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
evidence="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
mkdir -p "$evidence"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/cpm-run-${UID}}"
export CPM_CONTAINER_API_PORT="${CPM_CONTAINER_API_PORT:-18081}"
export CPM_CONTAINER_FRONTEND_PORT="${CPM_CONTAINER_FRONTEND_PORT:-4174}"
mkdir -p "$XDG_RUNTIME_DIR/podman"
scripts/install-upstreams.sh "$DREAM_MACHINE_UPSTREAM_ROOT"
socket="$XDG_RUNTIME_DIR/podman/podman.sock"; service_pid=""
if [[ ! -S "$socket" ]]; then podman system service --time=0 "unix://$socket" >"$evidence/podman-service.log" 2>&1 & service_pid=$!; for _ in {1..50}; do [[ -S "$socket" ]] && break; sleep .1; done; fi
[[ -S "$socket" ]] || { echo "podman API socket unavailable: $socket" >&2; exit 1; }
cleanup(){ podman compose -f compose.yml down --remove-orphans || true; [[ -n "$service_pid" ]] && kill "$service_pid" 2>/dev/null || true; }
trap cleanup EXIT
podman build --network=host -f containers/api.Containerfile -t localhost/capability-provenance-mesh-api:local .
podman build --network=host --build-arg VITE_API_URL=/api -f containers/frontend.Containerfile -t localhost/capability-provenance-mesh-frontend:local .
podman compose -f compose.yml up -d
export CPM_PUBLIC_API_URL="http://127.0.0.1:$CPM_CONTAINER_API_PORT"
export CPM_FRONTEND_PORT="$CPM_CONTAINER_FRONTEND_PORT"
scripts/readiness.sh
mapfile -t container_ids < <(podman compose -f compose.yml ps -q)
[[ "${#container_ids[@]}" == 2 ]] || { echo "expected two running containers, found ${#container_ids[@]}" >&2; exit 1; }
for container_id in "${container_ids[@]}"; do
  podman healthcheck run "$container_id"
  [[ "$(podman inspect --format '{{.State.Health.Status}}' "$container_id")" == healthy ]]
done
curl --fail --silent --show-error "http://127.0.0.1:$CPM_CONTAINER_FRONTEND_PORT/healthz" >"$evidence/frontend-health.txt"
curl --fail --silent --show-error "http://127.0.0.1:$CPM_CONTAINER_FRONTEND_PORT/" >"$evidence/frontend-index.html"
proxy_status="$(curl --silent --show-error --output "$evidence/frontend-api-proxy.json" --write-out '%{http_code}' "http://127.0.0.1:$CPM_CONTAINER_FRONTEND_PORT/api/v1/registry/missing/missing")"
[[ "$proxy_status" == 401 ]] || { echo "frontend API proxy returned unexpected status: $proxy_status" >&2; exit 1; }
podman compose -f compose.yml ps | tee "$evidence/container-ps.txt"
podman inspect "${container_ids[@]}" >"$evidence/container-inspect.json"
podman volume inspect capability-provenance-mesh_cpm-data >"$evidence/durable-volume.json"
printf 'CONTAINER_TOPOLOGY_PASS api=%s frontend=%s proxy_status=%s\n' "$CPM_PUBLIC_API_URL" "http://127.0.0.1:$CPM_CONTAINER_FRONTEND_PORT" "$proxy_status"
