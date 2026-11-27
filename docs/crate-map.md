# Planned Rust product crate coverage

The initial Cargo workspace contains only `cpm-contracts`, a substantive foundation for validated identities, capability names, revisions, and boundary failures. Remaining crates are added only when the first running slice needs their behavior; this avoids count-first scaffolding. The complete backend plan is below.

| # | Planned crate | Useful responsibility and typed public API | Production caller | Behavior/failure proof | Planned design mapping |
| --- | --- | --- | --- | --- | --- |
| 1 | `cpm-contracts` | IDs, digests, revisions, capability/effect values, error taxonomy | all domain/application crates | serialization, invalid digest/revision tests | ADR 01/03; DDD 01/02 |
| 2 | `cpm-artifact-intake` | bounded upload, hashing, quarantine, duplicate identity | application | accepted stream; oversize/truncated failure | ADR 04; DDD 02 |
| 3 | `cpm-manifest` | versioned manifest, entrypoint, declared capability validation | intake, remediation | supported parse; unknown/invalid class failure | ADR 05; DDD 03 |
| 4 | `cpm-lineage` | source/lock/toolchain/artifact trust chain | application | valid chain; stale/tampered chain failure | ADR 06/07; DDD 02 |
| 5 | `cpm-capability-model` | vocabulary, attenuation, budgets, set algebra | observation, policy, remediation | closure laws; capability escalation failure | ADR 08; DDD 03 |
| 6 | `cpm-observation` | normalize host calls/resource events with source binding | trial worker | known event; malformed/unbound event failure | ADR 09; DDD 04 |
| 7 | `cpm-effect-graph` | typed graph, transitive closure, contradiction/confidence | risk, remediation, UI query | reachable effects; contradictory/untrusted edge | ADR 10; DDD 05 |
| 8 | `cpm-risk` | explainable exposure/severity and threshold profile | admission | calibrated result; missing evidence refusal | ADR 11; DDD 05/06 |
| 9 | `cpm-scenarios` | frozen packs, deterministic inputs and expected outputs | scheduler | reproducible pack; changed hash refusal | ADR 12; DDD 04 |
| 10 | `cpm-resource-budget` | fuel, memory, time, output budgets and exhaustion classes | Wasmtime adapter, scheduler | accounting; overflow/exhaustion failure | ADR 13; DDD 04 |
| 11 | `cpm-trial-state` | lifecycle, idempotency, retry/cancel/resume invariants | jobs, application | legal transitions; duplicate/illegal transition | ADR 14; DDD 04 |
| 12 | `cpm-admission` | decision aggregate joining lineage, trace, graph, task, policy, human review | application | admit; deny/incomplete evidence | ADR 15; DDD 06 |
| 13 | `cpm-remediation` | minimal capability patch and re-trial contract | application | minimal repair; unsatisfiable task result | ADR 16; DDD 07 |
| 14 | `cpm-receipts` | canonical envelope, hashes, correlation chain, verification | store, API | verify; altered field/hash failure | ADR 17; DDD 08 |
| 15 | `cpm-registry` | admitted version index, activation, rollback, revocation | application | activate/retrieve; denied activation conflict | ADR 18; DDD 09 |
| 16 | `cpm-rvm` | RVM identity verification, capability map, witness translation | lineage/admission | valid package; corrupt/unsupported refusal | ADR 19; DDD 10 |
| 17 | `cpm-rune` | frozen plan replay/digest and stale-lineage translation | lineage | matching receipt; source/lock drift failure | ADR 20; DDD 10 |
| 18 | `cpm-latentmesh` | signed observation envelope, fragmentation, replay defense | trial worker/gateway | round trip; corrupt/replay rejection | ADR 21; DDD 10 |
| 19 | `cpm-worldgraph` | domain graph projection/query and provenance reconciliation | effect graph | typed query; invalid relation quarantine | ADR 22; DDD 05/10 |
| 20 | `cpm-apx` | frozen task-pack scoring and eligibility translation | admission | accepted score; incomplete/unsafe ineligible | ADR 23; DDD 06/10 |
| 21 | `cpm-wasmtime` | engine/linker, explicit host surface, fuel/epoch/memory traps | trial worker | permitted call; undeclared call/budget trap | ADR 24; DDD 04/10 |
| 22 | `cpm-cedar` | principal/action/resource/context mapping and policy compile | middleware/admission | allow; deny/schema/policy failure | ADR 25; DDD 06/10 |
| 23 | `cpm-store` | OpenDAL layout, append-only events, snapshot rebuild, conflicts | application/audit | restart read; missing/conflict/no partial pointer | ADR 17/18; DDD 08/09/10 |
| 24 | `cpm-application` | intake, trial, decide, remediate, retrieve, revoke use cases | API/jobs | workflow tests and upstream failure recovery | ADR 02/15; DDD 11 |
| 25 | `cpm-service` | Axum composition root, routes, middleware, readiness, shutdown | executable | full API/middleware tests; readiness/dependency failure | ADR 02/25; DDD 11/12 |

The final composition report must name actual manifest, implementation, caller, behavior test, failure test, ADR and DDD paths. Planned mappings above do not count until the implementations and documents exist and are reciprocally linked.
