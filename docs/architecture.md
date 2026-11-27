# Architecture

## Architectural style

Capability Provenance Mesh uses a Rust modular monolith with a service-owned background trial worker. Bounded contexts expose typed APIs through application services. Upstream projects sit behind anti-corruption adapters that translate real results and failures into domain types. A generated TypeScript client connects a React admission workbench to a versioned Axum API.

## Primary flow

1. React streams a bundle and manifest through the generated client.
2. Ordered middleware establishes correlation, observability, errors, timeout, identity, validation, authorization, and Cedar policy.
3. Artifact Intake hashes and quarantines bytes.
4. Lineage verifies Rune build-plan and artifact receipts.
5. RVM inspects package identity and maps declared capability classes.
6. Wasmtime executes the frozen scenario pack with explicit host functions, fuel, memory, and epoch limits.
7. Observations travel through a signed LatentMesh envelope and are normalized.
8. WorldGraph projects package, dependency, capability, effect, and evidence relationships.
9. APx scores accepted useful task work against the frozen reference pack.
10. Admission computes risk, invokes Cedar, records the human decision, and persists immutable artifacts/events through OpenDAL.
11. The API returns a graph, trial result, decision, and receipt for frontend rendering.

## Trust boundaries

- Browser is untrusted and never receives service credentials.
- Uploaded bundle remains quarantined until identity, limits, and policy are established.
- Wasmtime exposes only named product host functions; no ambient WASI authority.
- RVM performs inspection/capability mapping and witnessed refusal; it is not misrepresented as the WASM execution engine.
- LatentMesh envelopes are verified before observations influence decisions.
- OpenDAL current pointers update only after immutable blob and event durability.
- Policy denial precedes registry activation or a current-pointer write.

## State model

The service stores immutable blobs, append-only aggregate events, rebuildable snapshots, and conditional current pointers. On startup it verifies event/receipt hashes and reconstructs registry state. In-memory caches are disposable.

## Frontend shape

The frontend is an admission workbench, not a generic dashboard: intake editor, capability graph, deterministic trial console, denial/remediation diff, admitted-version registry, and audit replay. Every primary control uses real backend data with loading, empty, validation, success, error, denied, cancelled, and stale-revision states.

## Deployment topology

See [topology](topology.md). Local/CI runs use rootless containers and localhost-compatible networking. Cloud Run files are configuration only; no deployment is authorized.
