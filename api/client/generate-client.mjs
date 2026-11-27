import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const directory = fileURLToPath(new URL(".", import.meta.url));
const schemaPath = directory + "openapi-v1.json";
const outputPath = directory + "schema.ts";
const source = await readFile(schemaPath, "utf8");
const schema = JSON.parse(source);
const methods = new Set(["get", "post", "put", "patch", "delete"]);
const operations = [];
for (const [path, item] of Object.entries(schema.paths)) {
  for (const method of Object.keys(item)) {
    if (methods.has(method)) operations.push(`${method.toUpperCase()} ${path}`);
  }
}
operations.sort();
const digest = createHash("sha256").update(source).digest("hex");
const output = `// Generated from openapi-v1.json by generate-client.mjs. Do not hand-edit.
export const API_VERSION = ${JSON.stringify(schema.info.version)};
export const OPENAPI_SCHEMA_SHA256 = ${JSON.stringify(digest)};

export const REQUIRED_OPERATIONS = ${JSON.stringify(operations, null, 2)} as const;
`;
if (process.argv.includes("--check")) {
  const current = await readFile(outputPath, "utf8").catch(() => "");
  if (current !== output) {
    console.error("API_CLIENT_DRIFT: schema.ts does not match openapi-v1.json");
    process.exit(1);
  }
  console.log(`API_CLIENT_DRIFT_PASS ${digest}`);
} else {
  await writeFile(outputPath, output);
  console.log(`Generated schema.ts ${digest}`);
}
