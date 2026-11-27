#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
evidence="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"; mkdir -p "$evidence"; evidence="$(realpath "$evidence")"
sha="$(git rev-parse HEAD)"; test -z "$(git status --porcelain --untracked-files=all)" || { echo 'source must be clean' >&2; exit 1; }
case "$evidence" in "$root"|"$root"/*) echo 'evidence must be external' >&2; exit 1;; esac
export DREAM_MACHINE_SOURCE_SHA="$sha"
record(){ node scripts/dream-machine-product-gate.mjs record . "$evidence" "$1" -- "${@:2}"; }
record rust bash scripts/run-suite.sh rust
record backend cargo metadata --locked --offline --format-version 1
for suite in frontend api middleware integrations benchmark ablations containers browser; do record "$suite" bash scripts/run-suite.sh "$suite"; done
cp "${DREAM_MACHINE_SELECTION_REPORT:?verified selection report required}" "$evidence/selection-report.json"
node scripts/build-composition-report.mjs . "$evidence" "${DREAM_MACHINE_EXECUTION_MAP:?exact-source execution map required}"
node scripts/finalize-evidence.mjs . "$evidence"
