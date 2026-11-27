# Exact-source evidence plan

External evidence binds the exact clean commit.

1. rust: fmt, strict clippy, locked tests, deny, exact offline metadata.
2. backend: composition, per-crate behavior/failure, restart.
3. frontend: install, typecheck, lint, unit, build.
4. api: OpenAPI and generated-client drift.
5. middleware: ordered failure matrix.
6. integrations: intended/failure execution for all seven.
7. benchmark: frozen scenario digest and reproducibility.
8. ablations: graph, attenuation, quarantine, provenance deltas.
9. containers: rootless build, launch, readiness, saved flow, stop.
10. browser: three journeys, screenshots, Playwright JSON/trace, restart.

Manifest schema 2 binds run/source, suite receipts/hashes, screenshots/traces, semantic review, selection/probe archive, and exact argv cargo metadata --locked --offline --format-version 1. Composition schema 2 reciprocally maps crates to callers/tests/design and upstreams to pins/call sites/receipts. Canonical PASS does not replace semantic review.
