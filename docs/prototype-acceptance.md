# Complete-product acceptance map

| Area | Acceptance condition | Source-bound proof |
|---|---|---|
| backend | exactly 20 meaningful crates, all reachable and tested | locked metadata plus composition schema 2 |
| design | 25 distinct ADRs and 12 DDD records with only existing linked paths | document hashes, path validator, semantic review |
| frontend | domain-specific responsive React with lifecycle and accessibility states | unit/build plus desktop/mobile journey screenshots |
| API | OpenAPI generates the client; type/required-field mutation fails | API drift and negative contract tests |
| middleware | authn, role authz, policy, validation, correlation, timeout, tracing, errors execute before effects | failure matrix and correlation trace |
| integrations | five Ruvnet and two enterprise responsibilities execute intended/failure paths | seven raw logs, receipts, pins, licenses |
| primary journey | submission renders evidence-backed decision | Playwright report/trace |
| denial and recovery | denied digest never activates; repaired digest re-evaluates | before/after registry proof and correlated trace |
| durable journey | refresh and backend restart retrieve identical record | restart receipt and rendered audit |
| frontier evaluation | baseline/full metrics plus named mechanism-off deltas | benchmark and ablation JSON |
| containers | rootless API/web topology builds, readies, persists, and stops | container receipt/inspect |
| evidence/publication | ten suite receipts, schema-2 reports, review, Rust floor, controller order | exact-SHA external directory and exact-head CI |

Any missing row forbids BUILT; a smoke test or recorder acknowledgement cannot substitute for this conjunction.
