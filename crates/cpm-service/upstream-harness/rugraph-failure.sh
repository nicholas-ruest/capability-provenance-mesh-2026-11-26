#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/candidate-context.sh"
cpm_bind_candidate rugraph
r=$1; p="$r/ruvnet_ruGraph"
test "$(git -C "$p" rev-parse HEAD)" = d66d976c0ba8fa792fcf024c2a07b51480e4fbf1
export RUVECTOR_LANGUAGE_BIN="$p/crates/ruvector-language/target/debug/rvg-core"
cd "$p/npm/packages/graph-language"; node --test --test-name-pattern='expired and deleted evidence|program effects cannot grant' test/e2e.test.mjs
