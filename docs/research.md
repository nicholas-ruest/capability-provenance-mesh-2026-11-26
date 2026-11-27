# Research and frontier hypothesis

## Hypothesis

Causal evidence completeness plus monotone capability attenuation and multi-upstream consensus will reduce unsafe capability admission relative to a declaration-and-risk baseline while retaining acceptance of safe candidates within a bounded runtime cost.

The typed corpus is owned by [crates/cpm-scenarios/src/lib.rs](../crates/cpm-scenarios/src/lib.rs). [crates/cpm-application/src/bin/benchmark.rs](../crates/cpm-application/src/bin/benchmark.rs) compares baseline and full mechanism on unsafe admissions, safe acceptance, and elapsed time. [crates/cpm-application/src/bin/ablation.rs](../crates/cpm-application/src/bin/ablation.rs) disables graph completeness, authority attenuation, lineage enforcement, and upstream consensus individually. Shared experiment construction is in [experiment_support/mod.rs](../crates/cpm-application/src/bin/experiment_support/mod.rs).

## Selected ingredients

Autogenous, RuGraph, Agenticow, AI Text Watermark, and LatentMesh provide complementary Ruvnet responsibilities. NVIDIA SkillSpector and Sigstore Cosign supply enterprise inspection and attestation. Exact pins and licenses are recorded by [upstreams.lock.json](../crates/cpm-service/upstream-harness/upstreams.lock.json).

## Verified examples

Mandate Mesh at fa6e7ce708528d2a2a1d466d5524e99da2930323 establishes full-product shape. ToolRecall at ae214d5141256370664a0ad008cfb57b22f33590 demonstrates source-bound status and truthful blockers. JitterMap at 53b400f39adfec140dda2f8ef02c4f5ca4265cb3 demonstrates bounded transport without implicit authority. These are independently verified examples, not automatically approved learned policy.
