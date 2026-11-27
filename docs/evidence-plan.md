# Evidence and verification plan

## Canonical entrypoint

`scripts/verify-prototype.sh` is an exact copy of the pinned full-product wrapper. The future `scripts/collect-product-evidence.sh` must be a real product-specific collector, not a placeholder. It will launch the stack and execute all suites before the canonical verifier checks external evidence.

## Ten recorder suites

1. rust;
2. backend;
3. frontend;
4. api;
5. middleware;
6. integrations;
7. benchmark;
8. ablations;
9. containers;
10. browser.

The backend suite must record exact argv:

`cargo metadata --locked --offline --format-version 1`

## External schema-v2 artifact

The evidence directory stays outside the clean checkout and binds the exact source SHA. It will contain recorder receipts and separately hashed stdout/stderr, backend metadata, selection report/probe archive/original readiness receipts, composition report schema 2, independent semantic review, desktop/mobile screenshots, Playwright JSON/trace, correlated runtime/upstream trace, commands, versions, timestamps, launch mode, namespace, and artifact hashes.

## Semantic review

Independent review will inspect:

- useful crate behavior, real callers, and failure tests;
- actual middleware registration/effect order;
- frontend domain depth and real requests;
- canonical upstream identity, license, binding, call sites, and raw traces;
- source-linked substantive ADR/DDD decisions;
- denial no-effect assertion and restart reconstruction;
- contradictions between manifests, screenshots, traces, CI, and source.

## Current foundation limitation

This commit verifies gate bytes and design coverage only. It has no collector, running product, selection report, runtime composition report, semantic review, browser trace, or completion claim.
