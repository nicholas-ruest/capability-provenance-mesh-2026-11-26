# Implementation contract

1. The locked production graph contains exactly the 20 responsibilities in [crate-map.md](crate-map.md); package count alone is never evidence.
2. Every domain crate owns typed behavior and inline success/failure tests in its existing source file.
3. The seven upstream projects execute through [crates/cpm-upstreams/src/lib.rs](../crates/cpm-upstreams/src/lib.rs) and the common runtime; metadata/help-only operations do not count.
4. [crates/cpm-application/src/lib.rs](../crates/cpm-application/src/lib.rs) is the sole decision/persistence transaction boundary. Denied and incomplete decisions never become active.
5. [api/openapi.yaml](../api/openapi.yaml) is authoritative. [api/client/generate-client.mjs](../api/client/generate-client.mjs) generates the TypeScript schema, and negative drift tests must fail the checker.
6. Durable storage is append-only and restart-readable. Browser refresh and backend restart must return the same decision without rerunning upstreams.
7. Benchmark and ablations compare decision quality on the same typed scenario corpus, not merely command success or elapsed time.
8. The evidence collector executes rust, backend, frontend, api, middleware, integrations, benchmark, ablations, containers, and browser suites into external exact-SHA evidence.
9. Rust share is measured over the whole clean commit by the pinned helper; no padding, language overrides, or removed frontend behavior.
10. Publication occurs only as the pinned controller's child after complete verification and semantic review, targeting the personal nicholas-ruest owner and a non-draft PR.
