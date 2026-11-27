#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

function option(name, fallback) {
  const position = process.argv.indexOf(name);
  return position < 0 ? fallback : process.argv[position + 1];
}
function fail(message) { throw new Error(`API_CONTRACT_DRIFT: ${message}`); }
function digest(value) { return createHash("sha256").update(value).digest("hex"); }

const root = fileURLToPath(new URL("..", import.meta.url));
const schemaPath = resolve(root, option("--schema", "api/openapi.yaml"));
const generatedPath = resolve(root, option("--generated", "api/client/schema.ts"));
const runtimePath = resolve(root, option("--runtime", "api/client/runtime.ts"));
const servicePath = resolve(root, option("--service", "crates/cpm-service/src/lib.rs"));
const generatorPath = resolve(root, "api/client/generate-client.mjs");

const generation = spawnSync(process.execPath,
  [generatorPath, "--check", "--schema", schemaPath, "--output", generatedPath],
  { cwd: root, encoding: "utf8" });
if (generation.status !== 0) {
  process.stderr.write(generation.stdout);
  process.stderr.write(generation.stderr);
  fail("generated TypeScript client differs from authoritative OpenAPI semantics");
}

const source = readFileSync(schemaPath, "utf8");
let document;
try { document = JSON.parse(source); }
catch (error) { fail(`authoritative OpenAPI is not JSON-formatted YAML: ${error.message}`); }
const runtime = readFileSync(runtimePath, "utf8");
const service = readFileSync(servicePath, "utf8");
const generated = readFileSync(generatedPath, "utf8");
const methods = new Set(["get", "post", "put", "patch", "delete"]);
const expectedRoutes = [];

for (const [path, item] of Object.entries(document.paths ?? {})) {
  for (const [method, operation] of Object.entries(item)) {
    if (!methods.has(method)) continue;
    expectedRoutes.push(`${method.toUpperCase()} ${path}`);
    if (operation["x-runtime-client"] === true) {
      const binding = `API_OPERATIONS.${operation.operationId}`;
      if (!runtime.includes(binding)) fail(`runtime client is not bound to generated operation ${operation.operationId}`);
    }
  }
}
expectedRoutes.sort();

const actualRoutes = [];
const routePattern = /\.route\(\s*"([^"]+)"\s*,\s*(get|post|put|patch|delete)\(/g;
for (const match of service.matchAll(routePattern)) {
  actualRoutes.push(`${match[2].toUpperCase()} ${match[1]}`);
}
actualRoutes.sort();
if (JSON.stringify(actualRoutes) !== JSON.stringify(expectedRoutes)) {
  fail(`Rust routes ${JSON.stringify(actualRoutes)} differ from OpenAPI routes ${JSON.stringify(expectedRoutes)}`);
}

const declaredDigest = generated.match(/OPENAPI_SCHEMA_SHA256 = "([0-9a-f]{64})"/)?.[1];
if (declaredDigest !== digest(source)) fail("generated schema digest does not bind authoritative OpenAPI bytes");

console.log(JSON.stringify({
  status: "API_CONTRACT_CHECK_PASS",
  authoritative_schema: schemaPath,
  schema_sha256: digest(source),
  generated_sha256: digest(generated),
  runtime_sha256: digest(runtime),
  service_sha256: digest(service),
  routes: expectedRoutes,
}, null, 2));
