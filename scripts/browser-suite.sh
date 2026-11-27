#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
: "${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"; : "${DREAM_MACHINE_PLAYWRIGHT_ROOT:?external Playwright install required}"; : "${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
: >"$DREAM_MACHINE_EVIDENCE_DIR/correlation-trace.ndjson"
scripts/start.sh
trap 'scripts/stop.sh' EXIT
"$DREAM_MACHINE_PLAYWRIGHT_ROOT/node_modules/.bin/playwright" test scripts/browser-journeys.spec.cjs --config=scripts/playwright.config.cjs
trace="$(find "$DREAM_MACHINE_EVIDENCE_DIR/playwright-output" -name trace.zip -type f | sort | tail -1)"
[[ -n "$trace" ]] && cp "$trace" "$DREAM_MACHINE_EVIDENCE_DIR/playwright-trace.zip"
for f in desktop.png mobile.png playwright-report.json playwright-trace.zip correlation-trace.ndjson integrations-trace.ndjson; do [[ -s "$DREAM_MACHINE_EVIDENCE_DIR/$f" ]] || { echo "missing real browser artifact: $f" >&2; exit 1; }; done
