#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
source_schema="api/openapi.yaml"
tracked=(api/openapi.yaml api/client/schema.ts api/client/runtime.ts api/client/generate-client.mjs scripts/check-api-contract.mjs)
before="$(sha256sum "${tracked[@]}")"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT

mutate() {
  local kind="$1" target="$temporary/openapi-${kind}.yaml"
  /usr/bin/node - "$source_schema" "$target" "$kind" <<'NODE'
const [source, target, kind] = process.argv.slice(2);
const fs = require("node:fs");
const document = JSON.parse(fs.readFileSync(source, "utf8"));
if (kind === "type") {
  document.components.schemas.DecisionRecord.properties.admitted.type = "string";
} else if (kind === "required") {
  document.components.schemas.DecisionRecord.required =
    document.components.schemas.DecisionRecord.required.filter((name) => name !== "admitted");
} else if (kind === "endpoint") {
  const original = "/api/v1/registry/{plugin}/{version}";
  document.paths["/api/v1/registry/{plugin}/{version}/drift"] = document.paths[original];
  delete document.paths[original];
} else if (kind === "status-schema") {
  document.paths["/api/v1/intakes"].post.responses["200"]
    .content["application/json"].schema.$ref = "#/components/schemas/ErrorEnvelope";
} else {
  throw new Error(`unknown mutation ${kind}`);
}
fs.writeFileSync(target, JSON.stringify(document, null, 2) + "\n");
NODE
  local output="$temporary/${kind}.log"
  if /usr/bin/node scripts/check-api-contract.mjs --schema "$target" >"$output" 2>&1; then
    echo "expected API drift rejection for $kind" >&2
    exit 1
  fi
  grep -Eq 'API_(CLIENT|CONTRACT)_DRIFT' "$output"
  printf 'API_DRIFT_REJECTED %s\n' "$kind"
}

for kind in type required endpoint status-schema; do
  mutate "$kind"
done

after="$(sha256sum "${tracked[@]}")"
[[ "$before" == "$after" ]] || { echo "API_DRIFT_SOURCE_MUTATION" >&2; exit 1; }

printf 'API_DRIFT_NEGATIVE_CONTROL_PASS mutations=4 source_mutations=0\n'
