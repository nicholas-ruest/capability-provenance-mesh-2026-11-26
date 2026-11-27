#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/candidate-context.sh"
cpm_bind_candidate autogenous
r=$1; p="$r/ruvnet_autogenous"
test "$(git -C "$p" rev-parse HEAD)" = 3155111ae78039c3993a544044a9cac041090ea6
cd "$p"; cargo test --locked -p verifier verifier_rejects_capability_expansion_even_with_perfect_fitness -- --exact --nocapture
