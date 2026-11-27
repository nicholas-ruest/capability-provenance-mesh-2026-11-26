import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const directory = fileURLToPath(new URL(".", import.meta.url));
const schemaPath = directory + "openapi-runtime-v1.json";
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
function typeFor(node) {
  if (node.$ref) return node.$ref.split("/").at(-1);
  if (node.enum) return node.enum.map((value) => JSON.stringify(value)).join(" | ");
  if (node.type === "array") return `Array<${typeFor(node.items)}>`;
  if (node.type === "integer" || node.type === "number") return "number";
  if (node.type === "boolean") return "boolean";
  if (node.type === "string") return "string";
  if (node.type === "object" || node.properties) {
    const required = new Set(node.required ?? []);
    const fields = Object.entries(node.properties ?? {}).map(([name, value]) =>
      `  ${name}${required.has(name) ? "" : "?"}: ${typeFor(value)};`
    );
    return `{\n${fields.join("\n")}\n}`;
  }
  return "unknown";
}
const generatedTypes = Object.entries(schema.components.schemas)
  .map(([name, node]) => `export type ${name} = ${typeFor(node)};`)
  .join("\n\n");
const digest = createHash("sha256").update(source).digest("hex");
const output = `// Generated from openapi-runtime-v1.json by generate-client.mjs. Do not hand-edit.
export const API_VERSION = ${JSON.stringify(schema.info.version)};
export const OPENAPI_SCHEMA_SHA256 = ${JSON.stringify(digest)};

export const REQUIRED_OPERATIONS = ${JSON.stringify(operations, null, 2)} as const;

${generatedTypes}
`;
if (process.argv.includes("--check")) {
  const current = await readFile(outputPath, "utf8").catch(() => "");
  if (current !== output) {
    console.error("API_CLIENT_DRIFT: schema.ts does not match openapi-runtime-v1.json");
    process.exit(1);
  }
  console.log(`API_CLIENT_DRIFT_PASS ${digest}`);
} else {
  await writeFile(outputPath, output);
  console.log(`Generated schema.ts ${digest}`);
}
