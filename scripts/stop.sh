#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
[[ -f .env ]] && set -a && source .env && set +a
if [[ "${1:-}" == "--containers" ]]; then exec podman compose -f compose.yml down --remove-orphans; fi
state="${CPM_RUN_DIR:-${XDG_RUNTIME_DIR:-/tmp}/capability-provenance-mesh-${UID}}"
collect_descendants() {
  local parent="$1" child
  while IFS= read -r child; do
    [[ "$child" =~ ^[0-9]+$ ]] || continue
    collect_descendants "$child"
    printf '%s\n' "$child"
  done < <(pgrep -P "$parent" || true)
}
for name in frontend api; do
  file="$state/$name.pid"
  if [[ -s "$file" ]]; then
    pid="$(cat "$file")"
    [[ "$pid" =~ ^[0-9]+$ ]] || { rm -f "$file"; continue; }
    mapfile -t descendants < <(collect_descendants "$pid")
    kill "${descendants[@]}" "$pid" 2>/dev/null || true
    for _ in {1..20}; do kill -0 "$pid" 2>/dev/null || break; sleep .1; done
    kill -KILL "${descendants[@]}" "$pid" 2>/dev/null || true
    rm -f "$file"
  fi
done
