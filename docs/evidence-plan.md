# Exact-source evidence plan

[scripts/collect-product-evidence.sh](../scripts/collect-product-evidence.sh) executes and records ten suites: rust, backend, frontend, api, middleware, integrations, benchmark, ablations, containers, and browser.

The external manifest uses schema 2 and binds producer/run, exact clean source SHA, suite receipts, raw-log hashes, screenshots, Playwright JSON/trace, selection report, probe archive, semantic review, Rust-share JSON, and exact metadata argv. The composition report uses schema 2 and reciprocally maps the 20 crates to callers/tests/design and seven upstream modules to pins/call sites/receipts. [scripts/build-composition-report.mjs](../scripts/build-composition-report.mjs) constructs that report from real repository files.

Metadata is captured with cargo metadata --locked --offline --format-version 1. No skipped browser journey, stale source, handwritten receipt, or evidence rebinding is acceptable. Canonical verifier success is necessary but does not replace independent semantic inspection.
