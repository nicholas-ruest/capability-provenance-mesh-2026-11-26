#!/usr/bin/env bash
set -euo pipefail
api="${CPM_PUBLIC_API_URL:-http://127.0.0.1:${CPM_API_PORT:-8080}}"
front="http://${CPM_FRONTEND_HOST:-127.0.0.1}:${CPM_FRONTEND_PORT:-4173}"
deadline=$((SECONDS + ${CPM_READY_TIMEOUT_SECONDS:-90}))
until curl --fail --silent --show-error "$api/readyz" >/dev/null && curl --fail --silent --show-error "$front/" >/dev/null; do
  (( SECONDS < deadline )) || { echo "readiness timeout: $api $front" >&2; exit 1; }
  sleep 1
done
printf 'READY api=%s frontend=%s\n' "$api" "$front"
