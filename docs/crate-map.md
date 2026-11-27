# Production crate map

cpm-service is the binary root and cpm-application the transaction boundary. Every crate requires typed API, production caller, behavior/failure tests, ADR/DDD mapping, and evidence.

| Crate | Responsibility |
|---|---|
| cpm-contracts | stable identities, DTOs, errors |
| cpm-intake | normalize, digest, quarantine |
| cpm-manifest | manifest and declarations |
| cpm-capabilities | capability algebra |
| cpm-lineage | source, license, parent |
| cpm-scan | inspection normalization |
| cpm-graph | evidence completeness |
| cpm-risk | deterministic risk |
| cpm-quarantine | no-effect workspace |
| cpm-trial | leases and observations |
| cpm-policy | admission decisions |
| cpm-remediation | repair re-entry |
| cpm-attestation | signed decisions |
| cpm-receipts | canonical operations |
| cpm-audit | append-only history |
| cpm-registry | accepted current pointers |
| cpm-store | atomic durable storage |
| cpm-autogenous | Autogenous adapter |
| cpm-rugraph | RuGraph adapter |
| cpm-agenticow | Agenticow adapter |
| cpm-watermark | AI Text Watermark adapter |
| cpm-latentmesh | LatentMesh adapter |
| cpm-skillspector | SkillSpector adapter |
| cpm-cosign | Cosign adapter |
| cpm-application | use-case orchestration |
| cpm-service | middleware and API composition |

Call direction is service to application to domain/adapter/store. Adapters never call policy or registry.
