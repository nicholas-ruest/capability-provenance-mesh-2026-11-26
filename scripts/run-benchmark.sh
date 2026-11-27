#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
out="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}/benchmark-results.json"
cargo run --locked --offline -p cpm-application --bin benchmark >"$out"
node - "$out" <<'NODE'
const report = JSON.parse(require('node:fs').readFileSync(process.argv[2], 'utf8'));
if (report.schema_version !== 2 || report.assertion_passed !== true) process.exit(1);
if (report.baseline.false_admits <= 0 || report.mechanism.false_admits !== 0) process.exit(1);
if (report.mechanism.forbidden_active_writes !== 0) process.exit(1);
if (report.mechanism.evidence_receipts !== report.mechanism.decisions * 7) process.exit(1);
NODE
cat "$out"
