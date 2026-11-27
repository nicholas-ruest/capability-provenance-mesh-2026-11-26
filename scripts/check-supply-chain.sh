#!/usr/bin/env bash
set -euo pipefail
if command -v cargo-deny >/dev/null 2>&1; then exec cargo deny check advisories bans licenses sources; fi
if cargo audit --version >/dev/null 2>&1 && cargo license --version >/dev/null 2>&1; then
  cargo audit --deny warnings
  cargo license --avoid-build-deps --avoid-dev-deps >/dev/null
  exit 0
fi
echo 'cargo-deny or both cargo-audit and cargo-license are required; check is not skipped' >&2
exit 1
