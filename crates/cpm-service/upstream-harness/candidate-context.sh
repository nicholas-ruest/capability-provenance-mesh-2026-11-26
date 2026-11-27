#!/usr/bin/env bash
set -euo pipefail

cpm_bind_candidate() {
  local project=${1:?project name required}
  local payload=${CPM_UPSTREAM_INPUT:?candidate operation JSON is required}
  local expected_sha=${CPM_UPSTREAM_INPUT_SHA256:?candidate input digest is required}
  local actual_sha
  actual_sha=$(printf '%s' "$payload" | sha256sum | awk '{print $1}')
  test "$actual_sha" = "$expected_sha"
  python3 - "$project" <<'PY'
import json
import os
import sys

expected = {
    "autogenous": "bounded capability-plan synthesis",
    "rugraph": "effect-graph provenance traversal",
    "agenticow": "typed agent-tool contract execution",
    "watermark": "generated-text provenance detection",
    "latentmesh": "constrained provenance-envelope transport",
    "skillspector": "agent-skill policy inspection",
    "cosign": "keyless artifact identity verification",
}[sys.argv[1]]
operation = json.loads(os.environ["CPM_UPSTREAM_INPUT"])
assert operation["responsibility"] == expected
assert operation["operation"]
assert operation["semantic_fields"]
print(f"candidate-binding:{os.environ['CPM_UPSTREAM_INPUT_SHA256']}:{operation['operation']}")
PY
}
