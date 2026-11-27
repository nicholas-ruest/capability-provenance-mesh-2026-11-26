#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/candidate-context.sh"
cpm_bind_candidate agenticow
r=$1; p="$r/ruvnet_agenticow"
test "$(git -C "$p" rev-parse HEAD)" = a6467863e250371562c2e0c83f5f993459607746
cd "$p"; node examples/promotion-pipeline.mjs
