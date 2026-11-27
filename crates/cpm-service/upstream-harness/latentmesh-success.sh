#!/usr/bin/env bash
set -euo pipefail
r=$1; p="$r/ruvnet_LatentMesh"
test "$(git -C "$p" rev-parse HEAD)" = b04a70523196181aadc6d694e02de1f5722fae6b
cd "$p"; cargo run --locked -q -p latentmesh-agentbbs-bridge --example e2e_loopback | tee /dev/stderr | grep -q '"signature_verified":true'
