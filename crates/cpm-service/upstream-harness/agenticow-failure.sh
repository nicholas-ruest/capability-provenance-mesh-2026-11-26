#!/usr/bin/env bash
set -euo pipefail
r=$1; p="$r/ruvnet_agenticow"
test "$(git -C "$p" rev-parse HEAD)" = a6467863e250371562c2e0c83f5f993459607746
cd "$p"; node --test --test-name-pattern='checkpoint.*rollback discards poisoned edits' test/memory.test.js
