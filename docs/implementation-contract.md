# Implementation contract

1. cpm-service is the sole composition root; all counted crates are in the locked production graph with real callers.
2. Every crate has typed API, behavior and failure tests, acceptance criteria, and reciprocal ADR/DDD mapping.
3. Every adapter records exact project pin, license, argv, exit, raw-log hash, parse status, and candidate digest.
4. Commands are idempotent; denied/incomplete paths never call dispatch, promotion, or registry-current ports.
5. OpenAPI and generated TypeScript client drift fails verification.
6. Atomic content-addressed storage and append-only audit support refresh and restart retrieval.
7. Collector executes rust, backend, frontend, api, middleware, integrations, benchmark, ablations, containers, browser.
8. Evidence is external, real, exact-SHA bound, and never rebound or handwritten.
9. Pinned Rust-share measures the whole clean commit and requires hygiene plus at least 50 percent genuine Rust.
10. Publication is only the controller child after complete verification/review, targets nicholas-ruest, opens non-draft PR, and preserves human merge.
