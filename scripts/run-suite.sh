#!/usr/bin/env bash
set -euo pipefail
suite="${1:?suite required}"; root="$(git rev-parse --show-toplevel)"; cd "$root"
case "$suite" in
 rust) cargo fmt --check; cargo clippy --locked --workspace --all-targets --all-features -- -D warnings; cargo test --locked --workspace --all-features; scripts/check-supply-chain.sh ;;
 frontend) npm --prefix frontend ci; npm --prefix frontend run typecheck; npm --prefix frontend run lint; npm --prefix frontend run test -- --run; npm --prefix frontend run build ;;
 api) node scripts/check-api-contract.mjs; cargo test --locked -p cpm-contracts; cargo test --locked -p cpm-service tests::health_is_public -- --exact ;;
 middleware) cargo test --locked -p cpm-service --lib -- --nocapture ;;
 integrations) scripts/run-integrations.sh ;;
 benchmark) scripts/run-benchmark.sh ;;
 ablations) scripts/run-ablations.sh ;;
 containers) scripts/run-containers.sh ;;
 browser) scripts/browser-suite.sh ;;
 *) echo "unknown suite: $suite" >&2; exit 64 ;;
esac
