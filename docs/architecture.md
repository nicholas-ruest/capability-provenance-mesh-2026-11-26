# Capability Provenance Mesh architecture

Capability Provenance Mesh decides whether an executable capability pack may become active. The product preserves the causal chain from submitted bytes through declarations, seven upstream observations, trial state, admission, persistence, and rendered operator feedback.

## Runtime path

The React operator console in [frontend/src/ProductApp.tsx](../frontend/src/ProductApp.tsx) calls the generated client whose contract originates in [api/openapi.yaml](../api/openapi.yaml). The Axum composition root in [crates/cpm-service/src/lib.rs](../crates/cpm-service/src/lib.rs) enforces authentication, role authorization, policy, validation, timeout, correlation, tracing, and error envelopes before dispatching to [crates/cpm-application/src/lib.rs](../crates/cpm-application/src/lib.rs).

Application orchestration performs intake, manifest parsing, capability evaluation, lineage, scenario/trial construction, seven typed upstream integrations, observation normalization, effect-graph completeness, risk, and admission. It appends decisions and audit data through [crates/cpm-store/src/lib.rs](../crates/cpm-store/src/lib.rs), then writes active state through [crates/cpm-registry/src/lib.rs](../crates/cpm-registry/src/lib.rs) only for accepted outcomes.

## Integration boundary

Process mechanics live in [crates/cpm-upstream-runtime/src/lib.rs](../crates/cpm-upstream-runtime/src/lib.rs). Project semantics live in distinct modules under [crates/cpm-upstreams/src](../crates/cpm-upstreams/src). This keeps one meaningful integration crate without erasing the seven different requests, validations, and failure modes.

## No-effect guarantee

Denied and incomplete decisions are durable and queryable but never active. Browser restart retrieval reads stored projections; it does not rerun integrations. [ADR-012](adr/012.md), [ADR-019](adr/019.md), and [DDD-011](ddd/011.md) define the reciprocal boundaries.
