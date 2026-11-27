# Requirement traceability

| Contract requirement | Design | Existing implementation/test | Exact evidence |
|---|---|---|---|
| 20 meaningful reachable crates | [ADR-001](adr/001.md) | [Cargo.toml](../Cargo.toml), [service root](../crates/cpm-service/src/lib.rs) | locked metadata and composition schema 2 |
| monotone capability grant | [ADR-004](adr/004.md) | [capability model](../crates/cpm-capability-model/src/lib.rs) | inline properties and attenuation-off ablation |
| connected evidence | [ADR-007](adr/007.md) | [effect graph](../crates/cpm-effect-graph/src/lib.rs) | graph-off decision delta |
| three domain outcomes | [ADR-012](adr/012.md) | [admission](../crates/cpm-admission/src/lib.rs) | table tests and denial journey |
| five plus two upstreams | [ADR-018](adr/018.md) | [typed integrations](../crates/cpm-upstreams/src/lib.rs) | seven success/failure receipts |
| executable middleware | [ADR-019](adr/019.md) | [service](../crates/cpm-service/src/lib.rs), [failure matrix](../crates/cpm-service/src/middleware_matrix_tests.rs) | authn/authz/policy/validation/error/correlation/timeout/order with effect-free denials |
| authoritative typed API | [ADR-020](adr/020.md) | [OpenAPI](../api/openapi.yaml), [negative drift](../tests/api/contract-drift.test.sh) | API suite receipt |
| three browser journeys | [ADR-021](adr/021.md) | [ProductApp](../frontend/src/ProductApp.tsx), [Playwright](../scripts/browser-journeys.spec.cjs) | JSON report, trace, screenshots |
| frontier mechanism | [ADR-023](adr/023.md) | [benchmark](../crates/cpm-application/src/bin/benchmark.rs), [ablation](../crates/cpm-application/src/bin/ablation.rs) | corpus-bound decision metrics |
| rootless topology | [ADR-024](adr/024.md) | [compose](../compose.yml), [readiness](../scripts/container-readiness.sh) | container inspect and full-flow receipt |
| exact evidence/publication | [ADR-025](adr/025.md) | [collector](../scripts/collect-product-evidence.sh), [controller](../scripts/dream-machine-publication-gate.mjs) | manifest, composition, semantic review, CI |
