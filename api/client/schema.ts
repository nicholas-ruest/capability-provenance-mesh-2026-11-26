export const API_VERSION = "1.0.0";
export const OPENAPI_SCHEMA_SHA256 = "pending-generation";

export const REQUIRED_OPERATIONS = [
  "POST /api/v1/intakes",
  "GET /api/v1/intakes/{id}",
  "GET /api/v1/intakes/{id}/graph",
  "POST /api/v1/intakes/{id}/trials",
  "GET /api/v1/trials/{id}",
  "POST /api/v1/intakes/{id}/admissions",
  "GET /api/v1/registry",
  "GET /api/v1/registry/{plugin}/{version}",
  "GET /api/v1/audit/{correlation_id}",
] as const;
