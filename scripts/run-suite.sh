#!/usr/bin/env bash
set -euo pipefail
suite="${1:?suite required}"; root="$(git rev-parse --show-toplevel)"; cd "$root"
case "$suite" in
 rust) cargo fmt --check; cargo clippy --locked --workspace --all-targets --all-features -- -D warnings; cargo test --locked --workspace --all-features; scripts/check-supply-chain.sh ;;
 frontend) npm --prefix frontend ci; npm --prefix frontend run typecheck; npm --prefix frontend run lint; npm --prefix frontend test -- --run; npm --prefix frontend run build ;;
 api) npm --prefix frontend run api:check; cargo test --locked -p cpm-service --test api_contract ;;
 middleware) cargo test --locked -p cpm-service --test middleware ;;
 integrations) cargo test --locked -p cpm-service --test upstream_integrations -- --nocapture ;;
 benchmark) cargo bench --locked -p cpm-service --bench trial_pipeline -- --noplot ;;
 ablations) cargo test --locked -p cpm-service --test ablations -- --nocapture ;;
 containers) podman compose -f compose.yml up -d --build; trap 'podman compose -f compose.yml down --remove-orphans' EXIT; scripts/readiness.sh; podman compose -f compose.yml ps ;;
 browser)
  : "${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
  PLAYWRIGHT_JSON_OUTPUT_NAME="$DREAM_MACHINE_EVIDENCE_DIR/playwright-report.json" PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSERS_PATH:-/home/ruv/.cache/ms-playwright}" npm --prefix frontend run e2e -- --reporter=json
  for f in desktop.png mobile.png playwright-report.json playwright-trace.zip correlation-trace.ndjson integrations-trace.ndjson; do test -s "$DREAM_MACHINE_EVIDENCE_DIR/$f" || { echo "missing real browser artifact: $f" >&2; exit 1; }; done ;;
 *) echo "unknown suite: $suite" >&2; exit 64 ;;
esac
