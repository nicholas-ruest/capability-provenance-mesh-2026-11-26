#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/candidate-context.sh"
cpm_bind_candidate watermark
r=$1; p="$r/ruvnet_ai-text-watermark"
test "$(git -C "$p" rev-parse HEAD)" = 5871cf5fef6547c97626ba72d976f167562dab2a
cargo test --manifest-path "$p/crates/ruflo-watermark/Cargo.toml" --locked wrong_key_does_not_detect -- --exact --nocapture
