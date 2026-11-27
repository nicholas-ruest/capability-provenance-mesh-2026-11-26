// Generated from openapi-v1.json by generate-client.mjs. Do not hand-edit.
export const API_VERSION = "1.0.0";
export const OPENAPI_SCHEMA_SHA256 = "4614b879dc399f8318a260e2a5638be5c5ad46bb386e86cc65169f35431759d3";

export const REQUIRED_OPERATIONS = [
  "GET /api/v1/audit/{correlation_id}",
  "GET /api/v1/intakes/{id}",
  "GET /api/v1/intakes/{id}/graph",
  "GET /api/v1/registry",
  "GET /api/v1/registry/{plugin}/{version}",
  "GET /api/v1/trials/{id}",
  "GET /healthz",
  "GET /readyz",
  "POST /api/v1/intakes",
  "POST /api/v1/intakes/{id}/admissions",
  "POST /api/v1/intakes/{id}/trials"
] as const;
