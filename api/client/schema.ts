// Generated from openapi-runtime-v1.json by generate-client.mjs. Do not hand-edit.
export const API_VERSION = "1.0.0";
export const OPENAPI_SCHEMA_SHA256 = "f3e5bf2e82e51eb246c763a4d4e7f403d42fcbb32c4e2803855cf468801272f5";

export const REQUIRED_OPERATIONS = [
  "GET /api/v1/audit/{correlation}",
  "GET /api/v1/registry/{plugin}/{version}",
  "GET /healthz",
  "GET /readyz",
  "POST /api/v1/intakes"
] as const;

export type CapabilityManifest = {
  api_version: "cpm.capability/v1";
  name: string;
  entrypoint: string;
  capabilities: Array<string>;
};

export type CapabilitySubmission = {
  plugin: string;
  version: string;
  artifact: Array<number>;
  manifest: CapabilityManifest;
  human_approved: boolean;
  denial_probe: boolean;
};

export type DecisionRecord = {
  correlation_id: string;
  plugin: string;
  version: string;
  admitted: boolean;
  artifact_digest: string;
  receipt_checksum: string;
  upstream_receipts: Array<string>;
};

export type ErrorEnvelope = {
  code: string;
  message: string;
  correlation_id: string;
};
