#!/usr/bin/env bash
set -euo pipefail
h=$(cd "$(dirname "$0")" && pwd); source "$h/candidate-context.sh"; cpm_bind_candidate autogenous
r=$1; p="$r/ruvnet_autogenous"; test "$(git -C "$p" rev-parse HEAD)" = 3155111ae78039c3993a544044a9cac041090ea6
"$h/run-rust-adapter.sh" autogenous "$p" success
