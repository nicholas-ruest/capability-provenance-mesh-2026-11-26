# Product specification

## Operator outcome

An operator submits candidate content and receives a typed decision with evidence, risk, correlation, and active-state consequences. Unsafe content is denied without activation; a repaired candidate receives a new digest and full reevaluation; an accepted result remains retrievable after browser refresh and backend restart.

## Required capabilities

- Candidate identity from normalized bytes, typed manifest, immutable lineage, and monotone capability evaluation.
- Real execution of Autogenous, RuGraph, Agenticow, AI Text Watermark, LatentMesh, SkillSpector, and Cosign through the typed modules in [crates/cpm-upstreams/src](../crates/cpm-upstreams/src).
- Accepted, Denied, and Incomplete outcomes from [crates/cpm-admission/src/lib.rs](../crates/cpm-admission/src/lib.rs).
- Separate decision history and active registry state, persisted by [crates/cpm-store/src/lib.rs](../crates/cpm-store/src/lib.rs).
- Accessible React views for submission, decision, active registry, and audit with loading, empty, denied, success, and error states.
- Baseline/full frontier benchmark and named mechanism-off ablations over one deterministic scenario corpus.

## Invariants

Candidate authority cannot widen. Missing evidence is never clean evidence. Tool receipts do not confer authorization. Generated explanations are not signatures. Denial cannot create an active record. A repair cannot reuse its parent's digest.

## Completion

All ten evidence suites, three browser journeys, restart retrieval, schema-v2 manifest, schema-2 composition report, exact metadata, independent semantic review, Rust-share floor, and governed publication must bind the same clean source SHA.
