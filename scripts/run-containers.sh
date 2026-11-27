#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
: "${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/cpm-run-${UID}}"; mkdir -p "$XDG_RUNTIME_DIR/podman"
scripts/install-upstreams.sh "$DREAM_MACHINE_UPSTREAM_ROOT"
socket="$XDG_RUNTIME_DIR/podman/podman.sock"; service_pid=""
if [[ ! -S "$socket" ]]; then podman system service --time=0 "unix://$socket" >"${DREAM_MACHINE_EVIDENCE_DIR:?}/podman-service.log" 2>&1 & service_pid=$!; for _ in {1..50}; do [[ -S "$socket" ]] && break; sleep .1; done; fi
cleanup(){ podman compose -f compose.yml down --remove-orphans || true; [[ -n "$service_pid" ]] && kill "$service_pid" 2>/dev/null || true; }
trap cleanup EXIT
podman compose -f compose.yml up -d --build
export CPM_PUBLIC_API_URL="http://127.0.0.1:${CPM_CONTAINER_API_PORT:-18081}"
export CPM_FRONTEND_PORT="${CPM_CONTAINER_FRONTEND_PORT:-4174}"
scripts/readiness.sh
podman compose -f compose.yml ps
