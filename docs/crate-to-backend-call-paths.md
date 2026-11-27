# Planned crate-to-backend call paths

## Entrypoint

`cpm-service` will own the only production service binary. It constructs middleware, OpenDAL, upstream adapters, the application service, and the background trial worker. Every counted crate must be reachable by normal production Cargo edges from this binary.

## Command paths

| Use case | Planned call path |
| --- | --- |
| Intake | service -> middleware -> application -> artifact-intake -> contracts/store -> lineage -> rune/rvm |
| Trial | service -> middleware -> application -> scenarios/trial-state/resource-budget -> wasmtime -> observation -> latentmesh -> effect-graph/worldgraph |
| Admission | service -> middleware/cedar -> application -> risk/apx/admission -> receipts/store/registry |
| Remediation | service -> middleware -> application -> capability-model/effect-graph/remediation -> manifest -> trial |
| Retrieval | service -> middleware/cedar -> application -> registry/store/receipts -> API response |
| Revocation | service -> middleware/cedar -> application -> admission/registry/receipts/store |

## Background workflow

The trial worker is service-owned production behavior. Jobs carry correlation, tenant, immutable artifact, scenario-pack, policy, and resource-budget identifiers. Cancellation and timeout flow to Wasmtime epoch interruption. Results cannot update admission state except through the application service.

## Enforcement

The backend metadata suite must run the exact command:

`cargo metadata --locked --offline --format-version 1`

It will be recorded by the canonical product recorder. Development/build-only and disconnected packages cannot count.
