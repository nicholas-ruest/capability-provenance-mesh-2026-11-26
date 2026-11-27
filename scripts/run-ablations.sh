#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
out="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}/ablation-results.json"
cargo run --locked --offline -p cpm-application --bin ablation >"$out"
node - "$out" <<'NODE'
const report = JSON.parse(require('node:fs').readFileSync(process.argv[2], 'utf8'));
if (report.schema_version !== 2 || report.assertion_passed !== true) process.exit(1);
if (report.leave_one_out.length !== 7) process.exit(1);
if (!report.leave_one_out.every(row => row.admission === 'blocked' && row.remaining_receipts === 6 && row.forbidden_active_writes === 0 && row.durable_active_writes === 0 && row.assertion_passed === true)) process.exit(1);
if (!report.full_mechanism.safe_admitted || !report.full_mechanism.denial_blocked || report.full_mechanism.forbidden_active_writes !== 0) process.exit(1);
NODE
cat "$out"
