# Topology

## Local and CI topology

| Process | Responsibility | Planned port/boundary |
| --- | --- | --- |
| React/Vite frontend | Admission workbench and generated client | `127.0.0.1:4173` |
| Rust Axum service | API, middleware, application composition, SSE | `127.0.0.1:8080` |
| Rust trial worker | Service-owned bounded Wasmtime execution | internal task queue |
| APx adapter process | Pinned executable scoring boundary if no native API is available | child process, no network |
| OpenDAL FS service | Local/CI shared-object emulation | mounted authorized data directory |

## Container rules

- Rootless images and explicit unprivileged users.
- Read-only application filesystem except declared data/temp mounts.
- No privileged mode, host devices, ambient credentials, or unrestricted plugin network.
- Deterministic health/readiness checks.
- Full stack start/stop/reset/seed scripts will be implemented with the running slice.

## Cloud Run compatibility

The API and frontend remain stateless relative to the authorized shared-object boundary. Trial jobs use bounded request/job semantics and cancellation. Deployment and rollback files will be supplied later as configuration only; no cloud action is authorized.
