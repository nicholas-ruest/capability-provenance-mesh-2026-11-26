# Capability Provenance Mesh

Capability Provenance Mesh is a planned browser-operated admission workbench for platform teams that must decide whether a third-party WASM or plugin bundle may run. It will compare declared permissions with observed host calls, transitive dependency effects, build lineage, resource behavior, and task usefulness; a reviewer can deny, admit, or repair the manifest with a replayable receipt.

## Current state

This commit is the **design-first foundation only**. It contains one substantive contracts crate so Cargo metadata and the shared identity vocabulary are real; it deliberately avoids a count-first crate scaffold, fake integration, or runtime evidence. The first implementation milestone must be a running browser-to-persisted-result slice. The final product contract remains 20+ meaningful backend-integrated Rust crates, 25 substantive ADRs, 12 substantive DDD documents, a functional React/TypeScript frontend, executable middleware, five executed Ruvnet and two enterprise upstreams, three real browser journeys, restart durability, benchmarks/ablations, and source-bound evidence.

## Planned product interaction

- Upload and quarantine an executable plugin bundle.
- Inspect declared, observed, inferred, and forbidden capability edges.
- Run deterministic sandbox scenarios with live correlated events.
- Review denial reasons and a least-capability manifest repair.
- Admit a safe revision and retrieve its graph, receipt, and audit after restart.

## Design map

- [Specification](docs/specification.md)
- [Architecture](docs/architecture.md)
- [Implementation contract](docs/implementation-contract.md)
- [Topology](docs/topology.md)
- [Crate coverage plan](docs/crate-map.md)
- [ADR coverage index](docs/adr/index.md)
- [DDD coverage index](docs/ddd/index.md)
- [Prototype acceptance](docs/prototype-acceptance.md)
- [Integration matrix](docs/integration-matrix.md)
- [API and middleware plan](docs/api-middleware-plan.md)
- [Evidence plan](docs/evidence-plan.md)
- [Rust source-share budget](docs/rust-share-budget.md)

## Gate integrity

The repository contains the pinned canonical product, selection, and backend modules, their bundle verifier/manifest, the full-product wrapper installed as `scripts/verify-prototype.sh`, and the supplement Rust-share/publication gates. Byte integrity is necessary but is not product completion.
