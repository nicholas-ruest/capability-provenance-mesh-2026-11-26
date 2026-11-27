#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
[[ -f .env ]] && set -a && source .env && set +a
if [[ "${1:-}" == "--containers" ]]; then
  command -v podman >/dev/null || { echo "podman required" >&2; exit 1; }
  podman compose -f compose.yml up -d --build
  exec scripts/readiness.sh
fi
state="${CPM_RUN_DIR:-${XDG_RUNTIME_DIR:-/tmp}/capability-provenance-mesh-${UID}}"
data="${CPM_DATA_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/capability-provenance-mesh/data}"
temp="${CPM_TEMP_DIR:-$state/tmp}"
mkdir -p "$state" "$data" "$temp"
touch "$state/.cpm-run-dir" "$data/.cpm-data-dir"
[[ ! -s "$state/api.pid" && ! -s "$state/frontend.pid" ]] || { echo "stack already has pid files; stop first" >&2; exit 1; }
CPM_DATA_DIR="$data" CPM_TEMP_DIR="$temp" CPM_API_HOST="${CPM_API_HOST:-127.0.0.1}" CPM_API_PORT="${CPM_API_PORT:-8080}" cargo run --locked -p cpm-service --bin cpm-service >"$state/api.log" 2>&1 & echo $! >"$state/api.pid"
npm --prefix frontend run dev -- --host "${CPM_FRONTEND_HOST:-127.0.0.1}" --port "${CPM_FRONTEND_PORT:-4173}" >"$state/frontend.log" 2>&1 & echo $! >"$state/frontend.pid"
trap "scripts/stop.sh >/dev/null 2>&1 || true" ERR INT TERM
scripts/readiness.sh
trap - ERR INT TERM
printf "Capability Provenance Mesh: http://127.0.0.1:%s\n" "${CPM_FRONTEND_PORT:-4173}"
