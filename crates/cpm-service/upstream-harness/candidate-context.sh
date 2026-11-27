#!/usr/bin/env bash
set -euo pipefail

cpm_bind_candidate() {
  local project=${1:?project name required}
  local input=${CPM_UPSTREAM_INPUT_FILE:?candidate operation file is required}
  local expected_sha=${CPM_UPSTREAM_INPUT_SHA256:?candidate input digest is required}
  test "$(sha256sum "$input" | awk '{print $1}')" = "$expected_sha"
  CPM_CANDIDATE_DIR=$(mktemp -d)
  export CPM_CANDIDATE_DIR
  trap 'rm -rf "$CPM_CANDIDATE_DIR"' EXIT
  eval "$(python3 - "$project" "$input" "$CPM_CANDIDATE_DIR" <<'PY'
import hashlib, json, os, shlex, sys
project, source, target = sys.argv[1:]
expected = {
 "autogenous":"bounded capability-plan synthesis",
 "rugraph":"effect-graph provenance traversal",
 "agenticow":"typed agent-tool contract execution",
 "watermark":"generated-text provenance detection",
 "latentmesh":"constrained provenance-envelope transport",
 "skillspector":"agent-skill policy inspection",
 "cosign":"keyless artifact identity verification",
}[project]
operation=json.load(open(source,encoding="utf-8")); assert operation["responsibility"]==expected
c=operation["candidate"]; artifact=bytes(c["artifact"]); manifest=c["manifest_json"].encode()
assert hashlib.sha256(artifact).hexdigest()==c["artifact_sha256"]
assert hashlib.sha256(manifest).hexdigest()==c["manifest_sha256"]
ap=os.path.join(target,"artifact.bin"); mp=os.path.join(target,"manifest.json")
open(ap,"wb").write(artifact); open(mp,"wb").write(manifest)
skill=os.path.join(target,"skill"); os.mkdir(skill)
open(os.path.join(skill,"artifact.bin"),"wb").write(artifact)
open(os.path.join(skill,"manifest.json"),"wb").write(manifest)
open(os.path.join(skill,"SKILL.md"),"w",encoding="utf-8").write(
 f"# {c['plugin']}\n\nVersion: {c['version']}\nArtifact: {c['artifact_sha256']}\nCapabilities: "+", ".join(c["declared_capabilities"])+"\n")
values={"CPM_CANDIDATE_ARTIFACT":ap,"CPM_CANDIDATE_MANIFEST":mp,"CPM_CANDIDATE_SKILL":skill,
 "CPM_CANDIDATE_PLUGIN":c["plugin"],"CPM_CANDIDATE_VERSION":c["version"],
 "CPM_CANDIDATE_ARTIFACT_SHA256":c["artifact_sha256"],"CPM_CANDIDATE_MANIFEST_SHA256":c["manifest_sha256"]}
for k,v in values.items(): print(f"export {k}={shlex.quote(v)}")
PY
)"
  printf 'candidate-material:%s:%s:%s\n' "$project" "$CPM_CANDIDATE_ARTIFACT_SHA256" "$CPM_CANDIDATE_MANIFEST_SHA256"
}
