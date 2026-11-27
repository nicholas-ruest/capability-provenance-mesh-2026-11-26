#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
[[ -f .env ]] && set -a && source .env && set +a
if [[ "${1:-}" == "--containers" ]]; then exec podman compose -f compose.yml down --remove-orphans; fi
state="${CPM_RUN_DIR:-${XDG_RUNTIME_DIR:-/tmp}/capability-provenance-mesh-${UID}}"
for name in frontend api; do
  file="$state/$name.pid"
  if [[ -s "$file" ]]; then
    pid="$(cat "$file")"
    [[ "$pid" =~ ^[0-9]+$ ]] && kill "$pid" 2>/dev/null || true
    for _ in {1..20}; do kill -0 "$pid" 2>/dev/null || break; sleep .1; done
    kill -KILL "$pid" 2>/dev/null || true
    rm -f "$file"
  fi
done
