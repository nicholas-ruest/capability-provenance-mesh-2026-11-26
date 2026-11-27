#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
[[ -f .env ]] && set -a && source .env && set +a
state="${CPM_RUN_DIR:-${XDG_RUNTIME_DIR:-/tmp}/capability-provenance-mesh-${UID}}"
data="${CPM_DATA_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/capability-provenance-mesh/data}"
scripts/stop.sh
for spec in "$state:.cpm-run-dir" "$data:.cpm-data-dir"; do
  target="${spec%%:*}"; marker="${spec#*:}"
  [[ -d "$target" ]] || continue
  [[ -f "$target/$marker" ]] || { echo "refusing reset without marker: $target/$marker" >&2; exit 1; }
  find "$target" -mindepth 1 -depth ! -name "$marker" -delete
done
mkdir -p "$state" "$data"
touch "$state/.cpm-run-dir" "$data/.cpm-data-dir"
scripts/seed.sh
