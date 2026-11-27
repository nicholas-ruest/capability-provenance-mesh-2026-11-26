# Core-memory and verified-example application map

The identical task-start query returned 0 playbook and 0 anti-pattern records from dream-machine, and 9 playbook plus 6 anti-pattern records from nicholas-ruest/ruvnet-dream-machine. Both responses were unscreened:true. That is a screening limitation, not the cause of empty retrieval and not approval. Outcome feedback remains in dream-machine.

| Verified guidance or example | Applied decision | Real implementation/test | Evidence |
|---|---|---|---|
| pinned completion and product gates | local full product precedes publication | [collector](../scripts/collect-product-evidence.sh), [controller](../scripts/dream-machine-publication-gate.mjs) | ten suites, review, exact-head CI |
| Mandate Mesh exact SHA | multi-crate Rust plus React, three journeys, restart | [service](../crates/cpm-service/src/lib.rs), [ProductApp](../frontend/src/ProductApp.tsx) | Playwright traces and restart record |
| ToolRecall exact SHA | status must be source-bound and blockers truthful | [receipts](../crates/cpm-receipts/src/lib.rs), [audit store](../crates/cpm-store/src/lib.rs) | raw hashes and correlation |
| JitterMap exact SHA | transport evidence carries no authority by itself | [LatentMesh module](../crates/cpm-upstreams/src/latentmesh.rs), [admission](../crates/cpm-admission/src/lib.rs) | envelope failure and policy tests |
| retrieved historical playbooks | immutable pins, real execution, durable receipts | [integration suite](../crates/cpm-upstreams/src/lib.rs) | success/failure receipts |
| retrieved anti-patterns | reject slices, duplicate wrappers, fake paths, synthetic evidence | [crate map](crate-map.md), [traceability](traceability.md) | semantic review and validators |

No retrieved record is labeled human-approved here. Equivalent backfills are deduplicated by source and lesson before application.
