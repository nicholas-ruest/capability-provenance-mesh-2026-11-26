#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
: "${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"; : "${DREAM_MACHINE_PLAYWRIGHT_ROOT:?external Playwright install required}"; : "${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"
upstreams="$DREAM_MACHINE_UPSTREAM_ROOT"
export CPM_UPSTREAM_HARNESS_ROOT="$root/crates/cpm-service/upstream-harness"
export CPM_UPSTREAM_SOURCE_ROOT="$upstreams"
export CPM_SKILLSPECTOR_BIN="$upstreams/.venv-skillspector/bin/skillspector"
[[ -d "$CPM_UPSTREAM_HARNESS_ROOT" ]] || { echo "missing checked-in upstream harness: $CPM_UPSTREAM_HARNESS_ROOT" >&2; exit 1; }
[[ -d "$CPM_UPSTREAM_SOURCE_ROOT" ]] || { echo "missing external upstream source root: $CPM_UPSTREAM_SOURCE_ROOT" >&2; exit 1; }
[[ -x "$CPM_SKILLSPECTOR_BIN" ]] || { echo "missing installed SkillSpector binary: $CPM_SKILLSPECTOR_BIN" >&2; exit 1; }
: >"$DREAM_MACHINE_EVIDENCE_DIR/correlation-trace.ndjson"
scripts/start.sh
trap 'scripts/stop.sh' EXIT
"$DREAM_MACHINE_PLAYWRIGHT_ROOT/node_modules/.bin/playwright" test scripts/browser-journeys.spec.cjs --config=scripts/playwright.config.cjs
mapfile -t traces < <(find "$DREAM_MACHINE_EVIDENCE_DIR/playwright-output" -name trace.zip -type f | sort)
(( ${#traces[@]} == 3 )) || {
  echo "expected one Playwright trace for each of three journeys; found ${#traces[@]}" >&2
  exit 1
}
rm -rf "$DREAM_MACHINE_EVIDENCE_DIR/playwright-traces"
mkdir -p "$DREAM_MACHINE_EVIDENCE_DIR/playwright-traces"
for index in "${!traces[@]}"; do
  printf -v trace_name 'journey-%02d.zip' "$((index + 1))"
  cp "${traces[$index]}" "$DREAM_MACHINE_EVIDENCE_DIR/playwright-traces/$trace_name"
done
cp "${traces[$((${#traces[@]} - 1))]}" "$DREAM_MACHINE_EVIDENCE_DIR/playwright-trace.zip"
for f in desktop.png mobile.png playwright-report.json playwright-trace.zip correlation-trace.ndjson integrations-trace.ndjson; do [[ -s "$DREAM_MACHINE_EVIDENCE_DIR/$f" ]] || { echo "missing real browser artifact: $f" >&2; exit 1; }; done
