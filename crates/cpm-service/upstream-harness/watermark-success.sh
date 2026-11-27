#!/usr/bin/env bash
set -euo pipefail
h=$(cd "$(dirname "$0")" && pwd); source "$h/candidate-context.sh"; cpm_bind_candidate watermark
r=$1; p="$r/ruvnet_ai-text-watermark"; test "$(git -C "$p" rev-parse HEAD)" = 5871cf5fef6547c97626ba72d976f167562dab2a
"$h/run-rust-adapter.sh" watermark "$p" success
