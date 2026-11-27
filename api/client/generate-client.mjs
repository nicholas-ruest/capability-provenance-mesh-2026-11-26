import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const directory = fileURLToPath(new URL(".", import.meta.url));
const methods = new Set(["get", "post", "put", "patch", "delete"]);

function invariant(condition, message) {
  if (!condition) throw new Error(`OPENAPI_SEMANTIC_ERROR: ${message}`);
}

function resolveRef(document, ref, label) {
  invariant(typeof ref === "string" && ref.startsWith("#/"), `${label} has unsupported ref`);
  let value = document;
  for (const segment of ref.slice(2).split("/")) {
    value = value?.[segment.replaceAll("~1", "/").replaceAll("~0", "~")];
  }
  invariant(value && typeof value === "object", `${label} has unresolved ref ${ref}`);
  return value;
}

function schemaIdentity(node) {
  if (!node) return null;
  if (node.$ref) return node.$ref;
  if (node.const !== undefined) return { const: node.const };
  if (node.enum) return { type: node.type, enum: node.enum };
  if (node.type === "array") return { type: "array", items: schemaIdentity(node.items) };
  if (node.type === "object" || node.properties) {
    return {
      type: "object",
      required: [...(node.required ?? [])].sort(),
      properties: Object.fromEntries(Object.entries(node.properties ?? {})
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([name, value]) => [name, schemaIdentity(value)])),
    };
  }
  return { type: node.type };
}

function validateSchema(document, node, label, seen = new Set()) {
  invariant(node && typeof node === "object", `${label} must be a schema object`);
  if (node.$ref) {
    if (seen.has(node.$ref)) return;
    const next = new Set(seen).add(node.$ref);
    validateSchema(document, resolveRef(document, node.$ref, label), node.$ref, next);
    return;
  }
  const allowed = new Set(["string", "integer", "number", "boolean", "array", "object"]);
  invariant(allowed.has(node.type), `${label} has unsupported or missing type`);
  if (node.enum) invariant(node.enum.length > 0, `${label} has empty enum`);
  if (node.type === "array") validateSchema(document, node.items, `${label}.items`, seen);
  if (node.type === "object") {
    invariant(node.properties && typeof node.properties === "object", `${label} lacks properties`);
    const required = node.required ?? [];
    invariant(new Set(required).size === required.length, `${label} repeats required fields`);
    for (const name of required) invariant(name in node.properties, `${label} requires absent field ${name}`);
    for (const [name, value] of Object.entries(node.properties)) {
      validateSchema(document, value, `${label}.${name}`, seen);
    }
  }
}

function responseSchema(document, response, label) {
  const resolved = response.$ref ? resolveRef(document, response.$ref, label) : response;
  invariant(typeof resolved.description === "string", `${label} lacks description`);
  const content = resolved.content?.["application/json"];
  if (!content) return null;
  validateSchema(document, content.schema, `${label}.application/json`);
  return schemaIdentity(content.schema);
}

export function validateOpenApi(document) {
  invariant(document.openapi === "3.1.0", "openapi must be 3.1.0");
  invariant(typeof document.info?.version === "string", "info.version is required");
  invariant(document.paths && typeof document.paths === "object", "paths are required");
  invariant(document.components?.schemas, "component schemas are required");
  for (const [name, schema] of Object.entries(document.components.schemas)) {
    validateSchema(document, schema, `#/components/schemas/${name}`);
  }

  const operations = [];
  const operationIds = new Set();
  for (const [path, item] of Object.entries(document.paths)) {
    invariant(path.startsWith("/"), `path ${path} must start with /`);
    for (const [method, operation] of Object.entries(item)) {
      if (!methods.has(method)) continue;
      const label = `${method.toUpperCase()} ${path}`;
      invariant(typeof operation.operationId === "string", `${label} lacks operationId`);
      invariant(!operationIds.has(operation.operationId), `duplicate operationId ${operation.operationId}`);
      operationIds.add(operation.operationId);
      const parameters = operation.parameters ?? [];
      const placeholders = [...path.matchAll(/\{([^}]+)\}/g)].map((match) => match[1]).sort();
      const pathParameters = parameters.filter((parameter) => parameter.in === "path");
      invariant(pathParameters.every((parameter) => parameter.required === true), `${label} path parameters must be required`);
      invariant(JSON.stringify(pathParameters.map((parameter) => parameter.name).sort()) === JSON.stringify(placeholders),
        `${label} path parameters differ from template`);
      for (const parameter of parameters) {
        validateSchema(document, parameter.schema, `${label} parameter ${parameter.name}`);
      }
      let request = null;
      if (operation.requestBody) {
        invariant(operation.requestBody.required === true, `${label} request body must be required`);
        const bodySchema = operation.requestBody.content?.["application/json"]?.schema;
        validateSchema(document, bodySchema, `${label} request body`);
        request = schemaIdentity(bodySchema);
      }
      invariant(operation.responses && Object.keys(operation.responses).length > 0, `${label} lacks responses`);
      const responses = {};
      for (const [status, response] of Object.entries(operation.responses)) {
        invariant(/^[1-5][0-9]{2}$/.test(status), `${label} has invalid status ${status}`);
        responses[status] = responseSchema(document, response, `${label} response ${status}`);
      }
      operations.push({
        operationId: operation.operationId,
        method: method.toUpperCase(),
        path,
        runtimeClient: operation["x-runtime-client"] === true,
        parameters: parameters.map((parameter) => ({
          name: parameter.name, in: parameter.in, required: parameter.required === true,
          schema: schemaIdentity(parameter.schema),
        })),
        request,
        responses,
      });
    }
  }
  invariant(operations.length > 0, "at least one operation is required");
  return operations.sort((a, b) => a.operationId.localeCompare(b.operationId));
}

function typeFor(node) {
  if (node.$ref) return node.$ref.split("/").at(-1);
  if (node.const !== undefined) return JSON.stringify(node.const);
  if (node.enum) return node.enum.map((value) => JSON.stringify(value)).join(" | ");
  if (node.type === "array") return `Array<${typeFor(node.items)}>`;
  if (node.type === "integer" || node.type === "number") return "number";
  if (node.type === "boolean") return "boolean";
  if (node.type === "string") return "string";
  if (node.type === "object") {
    const required = new Set(node.required ?? []);
    const fields = Object.entries(node.properties).map(([name, value]) =>
      `  ${name}${required.has(name) ? "" : "?"}: ${typeFor(value)};`);
    return `{\n${fields.join("\n")}\n}`;
  }
  throw new Error(`OPENAPI_SEMANTIC_ERROR: cannot render type ${node.type}`);
}
export function renderClient(source) {
  let document;
  try { document = JSON.parse(source); }
  catch (error) { throw new Error(`OPENAPI_PARSE_ERROR: api/openapi.yaml must be JSON-formatted YAML: ${error.message}`); }
  const operations = validateOpenApi(document);
  const operationMap = Object.fromEntries(operations.map((operation) => [
    operation.operationId,
    {
      method: operation.method, path: operation.path, runtimeClient: operation.runtimeClient,
      parameters: operation.parameters, request: operation.request, responses: operation.responses,
    },
  ]));
  const requiredOperations = operations.map(({ method, path }) => `${method} ${path}`).sort();

  const generatedTypes = Object.entries(document.components.schemas)
    .map(([name, node]) => `export type ${name} = ${typeFor(node)};`).join("\n\n");
  const digest = createHash("sha256").update(source).digest("hex");
  const output = `// Generated from api/openapi.yaml by generate-client.mjs. Do not hand-edit.
export const API_VERSION = ${JSON.stringify(document.info.version)};
export const OPENAPI_SCHEMA_SHA256 = ${JSON.stringify(digest)};

export const API_OPERATIONS = ${JSON.stringify(operationMap, null, 2)} as const;
export const REQUIRED_OPERATIONS = ${JSON.stringify(requiredOperations, null, 2)} as const;


${generatedTypes}
`;
  return { digest, output };
}

function option(name, fallback) {
  const position = process.argv.indexOf(name);
  return position < 0 ? fallback : process.argv[position + 1];
}

const schemaPath = option("--schema", fileURLToPath(new URL("../openapi.yaml", import.meta.url)));
const outputPath = option("--output", directory + "schema.ts");
const source = await readFile(schemaPath, "utf8");
const { digest, output } = renderClient(source);
if (process.argv.includes("--check")) {
  const current = await readFile(outputPath, "utf8").catch(() => "");
  if (current !== output) {
    console.error(`API_CLIENT_DRIFT: ${outputPath} does not match authoritative ${schemaPath}`);
    process.exit(1);
  }
  console.log(`API_CLIENT_DRIFT_PASS ${digest}`);
} else {
  await writeFile(outputPath, output);
  console.log(`Generated ${outputPath} ${digest}`);
}
