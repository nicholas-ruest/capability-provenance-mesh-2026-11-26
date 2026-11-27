#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"; [[ -f .env ]] && set -a && source .env && set +a
state="${CPM_RUN_DIR:-${XDG_RUNTIME_DIR:-/tmp}/capability-provenance-mesh-${UID}}"; data="${CPM_DATA_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/capability-provenance-mesh/data}"; temp="${CPM_TEMP_DIR:-$state/tmp}"
pidfile="$state/api.pid"; [[ -s "$pidfile" ]] || { echo "API pid missing" >&2; exit 1; }; pid="$(cat "$pidfile")"; kill "$pid"; for _ in {1..50}; do kill -0 "$pid" 2>/dev/null || break; sleep .1; done
CPM_DATA_DIR="$data" CPM_TEMP_DIR="$temp" cargo run --locked -p cpm-service --bin cpm-service >>"$state/api.log" 2>&1 & echo $! >"$pidfile"
deadline=$((SECONDS+90)); until curl -fsS "http://127.0.0.1:${CPM_API_PORT:-8080}/readyz" >/dev/null; do ((SECONDS<deadline)) || exit 1; sleep 1; done
printf "API_RESTARTED old_pid=%s new_pid=%s\n" "$pid" "$(cat "$pidfile")"
