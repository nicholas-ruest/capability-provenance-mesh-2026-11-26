# Requirement traceability

| Requirement | Decision/domain | Production path | Verification |
|---|---|---|---|
| 20+ meaningful crates | ADR-001; DDD-001/011 | service to application to domains/adapters/store | locked metadata, composition, crate tests |
| Typed API | ADR-020; DDD-012 | OpenAPI, service v1, generated client | drift, frontend, browser |
| No-effect denial | ADR-004/013/021; DDD-004/006 | quarantine, Agenticow, middleware | matrix and before/after hashes |
| Five/two upstreams | ADR-006/007/009/010/013/016/019 | seven adapters | intended/failure receipts |
| Durable retrieval | ADR-018; DDD-008/009 | store, audit, registry | refresh plus restart journey |
| Benchmark/ablations | ADR-012 | frozen scenarios | reproducibility and deltas |
| Containers | ADR-023 | API/web topology | rootless full-flow receipt |
| Rust floor | ADR-024 | pinned helper | inventory and exact-head step |
| Publication | ADR-025 | controller and child publisher | negative control, receipt, CI |

Composition schema 2 must reciprocally hash ADR/DDD paths. A path without caller or failure test fails semantic review.
