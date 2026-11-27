#!/usr/bin/env bash
set -euo pipefail
h=$(cd "$(dirname "$0")" && pwd); source "$h/candidate-context.sh"; cpm_bind_candidate latentmesh
r=$1; p="$r/ruvnet_LatentMesh"; test "$(git -C "$p" rev-parse HEAD)" = b04a70523196181aadc6d694e02de1f5722fae6b
"$h/run-rust-adapter.sh" latentmesh "$p" success
