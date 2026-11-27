# Versioned API and middleware plan

## API v1

| Method and path | Purpose |
| --- | --- |
| `POST /api/v1/intakes` | Stream bundle and manifest; return immutable intake/quarantine state |
| `GET /api/v1/intakes/{id}` | Intake metadata and current state |
| `GET /api/v1/intakes/{id}/graph` | Declared/observed/inferred/forbidden graph projection |
| `PUT /api/v1/intakes/{id}/manifest` | Store optimistic-revision manifest draft |
| `POST /api/v1/intakes/{id}/trials` | Start frozen scenario-pack trial |
| `GET /api/v1/trials/{id}` | Trial state/result |
| `GET /api/v1/trials/{id}/events` | Correlated SSE event stream |
| `POST /api/v1/intakes/{id}/admissions` | Policy/human decision with idempotency key |
| `POST /api/v1/intakes/{id}/remediations` | Generate an explicit least-capability patch |
| `POST /api/v1/intakes/{id}/remediations/{rid}/trials` | Re-run a selected repair |
| `GET /api/v1/registry` | Admitted versions accessible to principal |
| `GET /api/v1/registry/{plugin}/{version}` | Durable admission, graph and receipt |
| `POST /api/v1/registry/{plugin}/{version}/revoke` | Governed revocation |
| `GET /api/v1/audit/{correlation_id}` | Correlated append-only audit |
| `GET /healthz`, `GET /readyz` | Unauthenticated, non-sensitive health/readiness |

OpenAPI is the contract source. The TypeScript client is generated or checked from it, and a drift test compares routes, DTOs, enum values, error variants, auth scopes, and schema hash.

## Registered middleware order

1. Correlation ID validation/generation.
2. Observability span and metrics context.
3. Consistent error-envelope mapping.
4. Timeout and cancellation.
5. Bearer authentication.
6. Boundary validation: content type, size, schema, path IDs.
7. Role/resource authorization.
8. Cedar policy.

## Failure matrix

- Missing, malformed, expired, and wrong-audience token.
- Cross-tenant or wrong-resource access.
- Invalid manifest and unsupported API version.
- Oversize/truncated bundle.
- Timeout/cancel propagation into trial job.
- Correlation preservation from browser through upstream traces.
- Redacted error output and emitted observability.
- Cedar deny before trial launch, registry activation, or current-pointer update.

A denial audit event is permitted and is separate from the forbidden admission side effect.
