#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/candidate-context.sh"
cpm_bind_candidate skillspector
r=$1; p="$r/NVIDIA_SkillSpector"; o=$(mktemp); scanner=${CPM_SKILLSPECTOR_BIN:-$r/.venv-skillspector/bin/skillspector}
test "$(git -C "$p" rev-parse HEAD)" = 48e0f942ea1f5fdb7aa096a6f65a0adeda2ecc4e
set +e
"$scanner" scan "$p/tests/fixtures/malicious_skill" --no-llm --format json --output "$o"
rc=$?; set -e; test "$rc" -ne 0
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["risk_assessment"]["recommendation"]=="DO_NOT_INSTALL"; print(json.dumps(d["risk_assessment"],sort_keys=True))' "$o"
