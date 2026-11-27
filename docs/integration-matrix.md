# Executed integration matrix

All projects execute through one production integration crate while retaining project-specific types and failure semantics.

| Project | Typed module | Product responsibility | Intended result | Failure assertion |
|---|---|---|---|---|
| Autogenous | [autogenous.rs](../crates/cpm-upstreams/src/autogenous.rs) | scoped remediation proposal | validated change plan tied to parent | empty/invalid plan rejected |
| RuGraph | [rugraph.rs](../crates/cpm-upstreams/src/rugraph.rs) | causal graph query/replay | graph evidence for candidate | invalid query/result rejected |
| Agenticow | [agenticow.rs](../crates/cpm-upstreams/src/agenticow.rs) | isolated agent-tool contract | bounded child contract | escaping/invalid contract rejected |
| AI Text Watermark | [watermark.rs](../crates/cpm-upstreams/src/watermark.rs) | generated text provenance | typed watermark probe | absent/corrupt probe reported, never identity |
| LatentMesh | [latentmesh.rs](../crates/cpm-upstreams/src/latentmesh.rs) | bounded signed envelope | scoped fresh mesh envelope | invalid envelope rejected |
| SkillSpector | [skillspector.rs](../crates/cpm-upstreams/src/skillspector.rs) | malicious skill inspection | normalized inspection | invalid/missing inspection becomes failure |
| Cosign | [cosign.rs](../crates/cpm-upstreams/src/cosign.rs) | offline signature verification | typed verification | tampered bundle rejected |

Shared process evidence comes from [crates/cpm-upstream-runtime/src/lib.rs](../crates/cpm-upstream-runtime/src/lib.rs). The exact pins and success/failure harnesses are under [crates/cpm-service/upstream-harness](../crates/cpm-service/upstream-harness). [crates/cpm-upstreams/src/lib.rs](../crates/cpm-upstreams/src/lib.rs) runs the full suite from production service composition.
