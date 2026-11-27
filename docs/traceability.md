# Requirement traceability

| Contract requirement | Design | Existing implementation/test | Exact evidence |
|---|---|---|---|
| 20 meaningful reachable crates | [ADR-001](adr/001.md) | [Cargo.toml](../Cargo.toml), [service root](../crates/cpm-service/src/lib.rs) | locked metadata and composition schema 2 |
| monotone capability grant | [ADR-004](adr/004.md), [ADR-023](adr/023.md) | [capability model](../crates/cpm-capability-model/src/lib.rs), [frontier corpus](../crates/cpm-application/src/bin/experiment_support/mod.rs) | attenuation-off exposes only five escalation cases; full blocks all |
| connected evidence | [ADR-007](adr/007.md), [ADR-023](adr/023.md) | [effect graph](../crates/cpm-effect-graph/src/lib.rs), [frontier corpus](../crates/cpm-application/src/bin/experiment_support/mod.rs) | graph-off exposes only five path-gap cases; full blocks all |
| artifact-bound lineage | [ADR-005](adr/005.md), [ADR-023](adr/023.md) | [lineage](../crates/cpm-lineage/src/lib.rs), [frontier corpus](../crates/cpm-application/src/bin/experiment_support/mod.rs) | lineage-off exposes only five digest-mismatch cases; full blocks all |
| complete upstream consensus | [ADR-018](adr/018.md), [ADR-023](adr/023.md) | [typed integrations](../crates/cpm-upstreams/src/lib.rs), [application ablation](../crates/cpm-application/src/bin/ablation.rs) | consensus-off exposes five six-of-seven cases; seven production leave-one-out variants stop before writes |
| three domain outcomes | [ADR-012](adr/012.md) | [admission](../crates/cpm-admission/src/lib.rs) | table tests and denial journey |
| five plus two upstreams | [ADR-018](adr/018.md) | [typed integrations](../crates/cpm-upstreams/src/lib.rs) | seven success/failure receipts and seven retained omission ablations |
| executable middleware | [ADR-019](adr/019.md) | [service](../crates/cpm-service/src/lib.rs), [failure matrix](../crates/cpm-service/src/middleware_matrix_tests.rs) | authn/authz/policy/validation/error/correlation/timeout/order with effect-free denials |
| authoritative typed API | [ADR-020](adr/020.md) | [OpenAPI](../api/openapi.yaml), [negative drift](../tests/api/contract-drift.test.sh) | API suite receipt |
| three browser journeys | [ADR-021](adr/021.md) | [ProductApp](../frontend/src/ProductApp.tsx), [Playwright](../scripts/browser-journeys.spec.cjs) | JSON report, trace, screenshots |
| strategic novelty and rotation | [novelty record](strategic-novelty.md) | [selection report](../selection-evidence/selection-report.json), [launch narrative](../README.md) | weighted scorecard, last-10 taxonomy, previous-three comparison, dated census, organization windows |
| frontier mechanism | [ADR-023](adr/023.md) | [benchmark](../crates/cpm-application/src/bin/benchmark.rs), [ablation](../crates/cpm-application/src/bin/ablation.rs), [research contract](research.md) | 25 identical typed decisions across baseline/full/four component variants; 5,000 bps full gain; -1,250 bps per ablation; zero forbidden writes |
| rootless topology | [ADR-024](adr/024.md) | [compose](../compose.yml), [readiness](../scripts/container-readiness.sh) | container inspect and full-flow receipt |
| exact evidence/publication | [ADR-025](adr/025.md) | [collector](../scripts/collect-product-evidence.sh), [controller](../scripts/dream-machine-publication-gate.mjs) | manifest, composition, semantic review, CI |
