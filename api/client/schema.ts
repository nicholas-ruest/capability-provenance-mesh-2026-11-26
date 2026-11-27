// Generated from api/openapi.yaml by generate-client.mjs. Do not hand-edit.
export const API_VERSION = "1.0.0";
export const OPENAPI_SCHEMA_SHA256 = "52acf74f98335d589ea1bff88d2d772943abd436dcd039d182f079cc6a51344b";

export const API_OPERATIONS = {
  "getAudit": {
    "method": "GET",
    "path": "/api/v1/audit/{correlation}",
    "runtimeClient": true,
    "parameters": [
      {
        "name": "correlation",
        "in": "path",
        "required": true,
        "schema": {
          "type": "string"
        }
      }
    ],
    "request": null,
    "responses": {
      "200": "#/components/schemas/DecisionRecord",
      "404": "#/components/schemas/ErrorEnvelope"
    }
  },
  "getDecision": {
    "method": "GET",
    "path": "/api/v1/registry/{plugin}/{version}",
    "runtimeClient": true,
    "parameters": [
      {
        "name": "plugin",
        "in": "path",
        "required": true,
        "schema": {
          "type": "string"
        }
      },
      {
        "name": "version",
        "in": "path",
        "required": true,
        "schema": {
          "type": "string"
        }
      }
    ],
    "request": null,
    "responses": {
      "200": "#/components/schemas/DecisionRecord",
      "404": "#/components/schemas/ErrorEnvelope"
    }
  },
  "health": {
    "method": "GET",
    "path": "/healthz",
    "runtimeClient": false,
    "parameters": [],
    "request": null,
    "responses": {
      "200": null
    }
  },
  "ready": {
    "method": "GET",
    "path": "/readyz",
    "runtimeClient": true,
    "parameters": [],
    "request": null,
    "responses": {
      "200": null
    }
  },
  "submitIntake": {
    "method": "POST",
    "path": "/api/v1/intakes",
    "runtimeClient": true,
    "parameters": [
      {
        "name": "x-cpm-role",
        "in": "header",
        "required": true,
        "schema": {
          "const": "reviewer"
        }
      },
      {
        "name": "x-cpm-policy",
        "in": "header",
        "required": false,
        "schema": {
          "type": "string",
          "enum": [
            "allow",
            "deny"
          ]
        }
      }
    ],
    "request": "#/components/schemas/CapabilitySubmission",
    "responses": {
      "200": "#/components/schemas/DecisionRecord",
      "400": "#/components/schemas/ErrorEnvelope",
      "401": "#/components/schemas/ErrorEnvelope",
      "403": "#/components/schemas/ErrorEnvelope",
      "408": "#/components/schemas/ErrorEnvelope",
      "415": "#/components/schemas/ErrorEnvelope",
      "422": "#/components/schemas/ErrorEnvelope"
    }
  }
} as const;
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
