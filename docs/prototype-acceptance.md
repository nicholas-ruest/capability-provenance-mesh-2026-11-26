# Prototype acceptance plan

## Journey 1 — primary admission

Platform engineer uploads a deterministic WASM image-transform plugin and manifest, explores declared/observed/transitive capability edges, runs the frozen task pack, reviews useful-work and risk results, admits it, and sees the durable registry record.

Required path: React control -> typed client -> API -> all middleware -> application -> RVM/Rune/Wasmtime/LatentMesh/WorldGraph/APx/Cedar -> OpenDAL event/current pointer -> rendered graph/decision/receipt.

## Journey 2 — denial and recovery

A plugin declares blob-read but attempts network/clock access. Wasmtime traps the host call; the signed observation reaches the graph; Cedar denies admission. Assert no registry activation or current pointer exists. The author explicitly applies a proposed least-capability repair or removes the behavior, re-runs, and receives a separate admissible decision.

## Journey 3 — durable retrieval

After admission, refresh the browser, stop/start the Rust backend, retrieve the registry entry and audit by correlation ID, verify hashes against stored artifact/trace objects, and render the same graph and active revision.

## UI acceptance

The workbench provides intake, graph, trial, remediation, registry, and audit views with keyboard access, labels/focus, responsive desktop/mobile layout, and loading, empty, validation, running, cancelled, denied, error, success, and stale-revision states. No first-party API route is mocked in browser acceptance.

## Playwright markers

- `[journey:primary]`
- `[journey:denial-recovery]`
- `[journey:durable-retrieval]`
