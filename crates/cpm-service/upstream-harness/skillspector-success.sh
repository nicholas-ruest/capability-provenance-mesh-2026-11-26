#!/usr/bin/env bash
set -euo pipefail
h=$(cd "$(dirname "$0")" && pwd); source "$h/candidate-context.sh"; cpm_bind_candidate skillspector
r=$1; p="$r/NVIDIA_SkillSpector"; o=$(mktemp); scanner=${CPM_SKILLSPECTOR_BIN:-$r/.venv-skillspector/bin/skillspector}
test "$(git -C "$p" rev-parse HEAD)" = 48e0f942ea1f5fdb7aa096a6f65a0adeda2ecc4e
"$scanner" scan "$CPM_CANDIDATE_SKILL" --no-llm --format json --output "$o"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["risk_assessment"]["recommendation"]=="SAFE"; print(json.dumps({"candidate":sys.argv[2],"assessment":d["risk_assessment"]},sort_keys=True))' "$o" "$CPM_CANDIDATE_ARTIFACT_SHA256"
