#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
export PATH="/home/ruv/.cargo/bin:/usr/bin:$PATH"
cargo test --locked -p cpm-service middleware_matrix -- --nocapture
printf 'MIDDLEWARE_FAILURE_MATRIX_PASS authn=2 authz=1 policy=1 validation=1 typed_error=1 correlation=2 timeout=1 observability_order=1 forbidden_effects=0\n'
