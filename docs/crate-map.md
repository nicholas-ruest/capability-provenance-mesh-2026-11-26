# Production crate map

Exactly 20 production crates are reachable from the service composition root. Inline test modules in each source file exercise the crate's owned behavior and failure cases.

| Crate | Owned responsibility | Design record |
|---|---|---|
| cpm-contracts | shared IDs, requests, decisions, errors | [ADR-003](adr/003.md) |
| cpm-artifact-intake | normalized bytes and digest | [ADR-002](adr/002.md) |
| cpm-manifest | declaration grammar | [ADR-003](adr/003.md) |
| cpm-capability-model | monotone authority algebra | [ADR-004](adr/004.md) |
| cpm-lineage | source/license/parent ancestry | [ADR-005](adr/005.md) |
| cpm-observation | normalized upstream evidence | [ADR-006](adr/006.md) |
| cpm-effect-graph | causal edges and completeness | [ADR-007](adr/007.md) |
| cpm-risk | deterministic assessment | [ADR-008](adr/008.md) |
| cpm-scenarios | frozen evaluation corpus | [ADR-009](adr/009.md) |
| cpm-resource-budget | typed execution limits | [ADR-010](adr/010.md) |
| cpm-trial-state | guarded lifecycle | [ADR-011](adr/011.md) |
| cpm-admission | three-outcome conjunction | [ADR-012](adr/012.md) |
| cpm-remediation | immutable repair proposal | [ADR-013](adr/013.md) |
| cpm-receipts | canonical operation evidence | [ADR-014](adr/014.md) |
| cpm-registry | active accepted projection | [ADR-015](adr/015.md) |
| cpm-store | append-only durable storage | [ADR-016](adr/016.md) |
| cpm-upstream-runtime | hardened subprocess mechanics | [ADR-017](adr/017.md) |
| cpm-upstreams | seven typed project modules | [ADR-018](adr/018.md) |
| cpm-application | use-case transaction boundary | [ADR-022](adr/022.md) |
| cpm-service | HTTP, middleware, composition root | [ADR-019](adr/019.md) |

The locked workspace list is in [Cargo.toml](../Cargo.toml). The runtime call direction is service -> application -> domain ports -> upstream/store implementations.
