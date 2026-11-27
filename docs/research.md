# Research and frontier hypothesis

## Falsifiable hypothesis

A pre-persistence admission closure composed of connected effect evidence, monotone capability attenuation, artifact-bound lineage, and complete seven-source consensus will block each corresponding unsafe candidate class while preserving safe admission. Relative to a declaration-only baseline on the same frozen corpus, the full closure must improve balanced admission quality by 5,000 basis points, produce no false admission, and produce no forbidden active write. Disabling exactly one component must expose only that component's isolated fault class and reduce balanced quality by 1,250 basis points. Separately, removing any one credited upstream receipt must stop the production application before a decision or active record is persisted.

This claim is narrower than “the system is generally safe.” It concerns a deterministic, typed corpus and four implemented checks. It is falsified if the full configuration admits any unsafe case, rejects a safe case, writes an unsafe activation, or if any single-component ablation has no decision delta.

## Mechanism

The production domain types used by the experiment are `EffectGraph`, `CapabilitySet`, and `Lineage`; consensus uses the same seven canonical ingredient identities enforced by the application. The frontier evaluator in [experiment_support/mod.rs](../crates/cpm-application/src/bin/experiment_support/mod.rs) is deliberately pre-persistence. An unsafe decision remains visible as `unsafe_admitted`, but the harness never forwards it to `DirectoryStore`, so `forbidden_active_writes` must remain zero for baseline, full, and every ablation. The end-to-end production benchmark separately executes `Application::submit` and checks retrieval to prove denied candidates do not become active.

```text
admit(case, enabled):
  if graph in enabled and any observed effect lacks artifact -> effect path: deny
  if authority in enabled and observed is not a subset of declared: deny
  if lineage in enabled and SHA256(artifact) != lineage.artifact: deny
  if consensus in enabled and canonical_receipt_count != 7: deny
  admit
```

The conjunction has these invariants:

1. **Graph completeness:** every observed effect has a trusted path from `artifact`; an absent path is not equivalent to a benign effect.
2. **Authority attenuation:** `declared.attenuate(observed)` must succeed, so observation cannot widen declared authority.
3. **Lineage enforcement:** the stored artifact digest must equal `SHA256(artifact_bytes)` and all lineage digests must remain well formed.
4. **Upstream consensus:** all seven distinct responsibility receipts must be present; aliases and multiple receipts from one canonical ingredient do not increase the count.
5. **Commit safety:** evaluation precedes persistence. An unsafe admission counted by a deliberately weakened experimental configuration is never written.

For one candidate, breadth-first graph reachability is `O(V + E)` for each observed effect, set attenuation is `O(D + O)` over declared and observed capabilities, lineage verification is `O(B)` over artifact bytes plus constant-sized digest validation, and consensus is `O(U)` over canonical upstream identities. Memory is `O(V + E + D + O + U)`. The experiment does not claim these small deterministic inputs model production latency distributions.

## Frozen component-isolation corpus

The corpus contains 25 decisions: five safe cases and five repetitions of each isolated fault. All non-target mechanisms are valid in each unsafe fixture.

| Fault class | Deliberate mutation | Full blocker | Single ablation that exposes it |
|---|---|---|---|
| graph gap | omit the trusted `artifact -> blob.read` edge | effect-graph reachability | graph completeness off |
| authority escalation | observe `net.open` while declaring only `blob.read`; graph and lineage remain valid | `CapabilitySet::attenuate` | authority attenuation off |
| lineage mismatch | bind lineage to a valid but incorrect 64-character digest | `Lineage::verify` | lineage enforcement off |
| consensus gap | provide six of seven canonical responsibility receipts | exact consensus cardinality | upstream consensus off |

[benchmark.rs](../crates/cpm-application/src/bin/benchmark.rs) reports declaration-only, production application, frontier baseline, and frontier full results. [ablation.rs](../crates/cpm-application/src/bin/ablation.rs) reports the four actual component toggles and preserves all seven leave-one-upstream-out production ablations. The unit tests assert that each component-off configuration admits exactly its five target faults and no other fault class.

## Expected and executed decision deltas

The deterministic assertions are:

| Variant | Safe admitted | Unsafe admitted | Unsafe blocked | Balanced quality | Forbidden writes |
|---|---:|---:|---:|---:|---:|
| declaration/component-free baseline | 5/5 | 20/20 | 0/20 | 5,000 bps | 0 |
| full four-component closure | 5/5 | 0/20 | 20/20 | 10,000 bps | 0 |
| graph completeness disabled | 5/5 | 5 graph-gap | 15/20 | 8,750 bps | 0 |
| authority attenuation disabled | 5/5 | 5 escalation | 15/20 | 8,750 bps | 0 |
| lineage enforcement disabled | 5/5 | 5 mismatch | 15/20 | 8,750 bps | 0 |
| upstream consensus disabled | 5/5 | 5 consensus-gap | 15/20 | 8,750 bps | 0 |

The end-to-end fixed workload remains 40 application decisions: 20 safe and 20 denial probes. The declaration-only baseline admits all 40, while the production application admits the 20 safe candidates, blocks all 20 probes, emits 280 source-bound receipts, creates 20 active records, and creates zero forbidden active records. Runtime nanoseconds are recorded but are not the primary outcome.

## Failure conditions and limits

The hypothesis fails on any unexpected admission/rejection, zero quality delta for a component, non-zero forbidden write, missing upstream omission result, or corpus mutation that makes more than one mechanism invalid in a supposedly isolated case. These experiments establish component contribution on a synthetic corpus, not statistical generalization, adversarial completeness, upstream correctness, or causal independence in arbitrary workloads. The external evidence collector must rerun both binaries on the exact candidate; prose and previously captured numbers are not a substitute for those receipts.

## Selected ingredients and prior examples

Autogenous, RuGraph, Agenticow, AI Text Watermark, and LatentMesh provide complementary Ruvnet responsibilities. NVIDIA SkillSpector and Sigstore Cosign supply enterprise inspection and attestation. Exact pins and licenses are recorded by [upstreams.lock.json](../crates/cpm-service/upstream-harness/upstreams.lock.json).

Mandate Mesh at `fa6e7ce708528d2a2a1d466d5524e99da2930323` establishes full-product shape. ToolRecall at `ae214d5141256370664a0ad008cfb57b22f33590` demonstrates source-bound status and truthful blockers. JitterMap at `53b400f39adfec140dda2f8ef02c4f5ca4265cb3` demonstrates bounded transport without implicit authority. These are independently verified examples, not automatically approved learned policy.
