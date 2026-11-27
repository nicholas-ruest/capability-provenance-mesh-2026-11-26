#!/usr/bin/env bash
set -euo pipefail
adapter=${1:?adapter required}; upstream=${2:?upstream checkout required}; mode=${3:?mode required}
here=$(cd "$(dirname "$0")" && pwd); work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
cp -R "$here/rust-adapters/$adapter/." "$work/"
python3 - "$work/Cargo.toml.in" "$work/Cargo.toml" "$upstream" <<'PY'
import sys
t,o,u=sys.argv[1:]; open(o,"w").write(open(t).read().replace("@UPSTREAM@",u))
PY
CARGO_TARGET_DIR=${CPM_UPSTREAM_ADAPTER_TARGET:-/tmp/cpm-upstream-adapter-target} cargo run --offline --quiet --manifest-path "$work/Cargo.toml" -- "$mode"
