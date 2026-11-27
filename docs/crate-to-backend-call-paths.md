# Crate-to-backend call paths

| Operation | Real call path | Stored/rendered result |
|---|---|---|
| submit intake | ProductApp -> generated client -> service -> application -> artifact-intake -> manifest -> capability-model -> lineage | candidate digest and declaration |
| gather evidence | application -> cpm-upstreams UpstreamSet -> cpm-upstream-runtime -> pinned harness -> observation -> receipts | seven hashed observations |
| decide | application -> effect-graph -> risk -> trial-state -> admission | Accepted, Denied, or Incomplete decision |
| persist | application -> store append -> registry activate only when accepted | audit history and active projection |
| query after restart | service GET -> store/registry read -> typed API -> ProductApp | same decision and active state |
| remediate | application -> Autogenous typed module -> remediation -> new intake | child digest with parent lineage |

The concrete service and application implementations are [crates/cpm-service/src/lib.rs](../crates/cpm-service/src/lib.rs) and [crates/cpm-application/src/lib.rs](../crates/cpm-application/src/lib.rs). The seven integration modules are indexed by [crates/cpm-upstreams/src/lib.rs](../crates/cpm-upstreams/src/lib.rs). Browser proof is executed by [scripts/browser-journeys.spec.cjs](../scripts/browser-journeys.spec.cjs).

No path permits an adapter to write active registry state. The application completes admission first, persists the decision, and conditionally activates accepted candidates.
