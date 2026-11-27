# API and middleware contract

/api/v1 comes from committed OpenAPI; React imports only the generated TypeScript client.

Order: correlation, observability, error mapping, timeout, authentication, schema validation, role authorization, domain policy, handler. Correlation and errors wrap every response; no-effect guards precede dispatch.

Failure tests cover missing/invalid credentials, forbidden role, malformed body, unsupported media type, stale state, policy denial, incomplete evidence, timeout, and adapter fault. Responses carry stable code, safe message, correlation ID, and retryability. Denied requests make zero dispatch, promotion, and registry-current calls.

Authenticated candidate-scoped SSE uses persisted cursors, bounded replay, heartbeats, and terminal closure. Durable query state is authoritative.
