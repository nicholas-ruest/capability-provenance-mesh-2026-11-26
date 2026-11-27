#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
evidence="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"; mkdir -p "$evidence"; evidence="$(realpath "$evidence")"
sha="$(git rev-parse HEAD)"; test -z "$(git status --porcelain --untracked-files=all)" || { echo "source must be clean" >&2; exit 1; }
case "$evidence" in "$root"|"$root"/*) echo "evidence must be external" >&2; exit 1;; esac
: "${DREAM_MACHINE_UPSTREAM_ROOT:?external upstream root required}"; : "${DREAM_MACHINE_SELECTION_REPORT:?selection report required}"; : "${DREAM_MACHINE_SELECTION_EVIDENCE:?selection evidence root required}"
export DREAM_MACHINE_SOURCE_SHA="$sha"
node scripts/dream-machine-gate-bundle.mjs .
node scripts/dream-machine-selection-gate.mjs "$DREAM_MACHINE_SELECTION_REPORT" "$DREAM_MACHINE_SELECTION_EVIDENCE"
record(){ node scripts/dream-machine-product-gate.mjs record . "$evidence" "$1" -- "${@:2}"; }
record rust bash scripts/run-suite.sh rust
record backend cargo metadata --locked --offline --format-version 1
for suite in frontend api middleware integrations benchmark ablations containers browser; do record "$suite" bash scripts/run-suite.sh "$suite"; done
cp "$DREAM_MACHINE_SELECTION_REPORT" "$evidence/selection-report.json"
[[ -s "${DREAM_MACHINE_SEMANTIC_REVIEW:-}" ]] || { echo "independent semantic review required" >&2; exit 1; }
cp "$DREAM_MACHINE_SEMANTIC_REVIEW" "$evidence/semantic-review.md"
node scripts/build-execution-map.mjs . "$evidence"
node scripts/build-composition-report.mjs . "$evidence" "$evidence/execution-map.json"
node scripts/finalize-evidence.mjs . "$evidence"
