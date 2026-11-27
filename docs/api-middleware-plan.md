# API and middleware contract

[api/openapi.yaml](../api/openapi.yaml) is the sole API contract. [api/client/generate-client.mjs](../api/client/generate-client.mjs) generates [api/client/schema.ts](../api/client/schema.ts); the official checker performs structural comparison. [tests/api/contract-drift.test.sh](../tests/api/contract-drift.test.sh) proves a required/type mutation is detected.

[crates/cpm-service/src/lib.rs](../crates/cpm-service/src/lib.rs) applies correlation, tracing, timeout, authentication, role authorization, policy, validation, and stable errors before calling the application. Failure cases cannot dispatch upstreams or activate registry state.

The generated runtime in [api/client/runtime.ts](../api/client/runtime.ts) is the frontend network boundary. ProductApp renders server-owned outcomes instead of recomputing governance. Browser traces must carry the same correlation ID through response, receipts, store, and audit query.
