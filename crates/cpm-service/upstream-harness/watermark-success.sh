#!/usr/bin/env bash
set -euo pipefail
r=$1; p="$r/ruvnet_ai-text-watermark"
test "$(git -C "$p" rev-parse HEAD)" = 5871cf5fef6547c97626ba72d976f167562dab2a
cargo test --manifest-path "$p/crates/ruflo-watermark/Cargo.toml" --locked proxy_stream_is_detected -- --exact --nocapture
