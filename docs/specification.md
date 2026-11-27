# Product specification

## Problem

Plugin admission is commonly decided from a manifest, signature, or scanner score. Those describe what a bundle claims, not what it actually reaches under realistic inputs. Platform teams need a human-reviewable product that binds artifact lineage, controlled execution, observed host calls, transitive effects, policy, task usefulness, and durable evidence into one decision.

## Users

- Platform security engineers admitting marketplace or internal plugins.
- Plugin authors repairing overbroad or incomplete capability manifests.
- Service owners reviewing trial behavior against tenant policy and acceptance packs.
- Auditors replaying an exact admission from immutable source-bound evidence.

## Product outcome

A reviewer uploads a bundle, runs a frozen scenario pack in a bounded sandbox, explores a typed capability graph, applies policy, repairs least-capability differences, and saves an admitted or denied revision with a verifiable receipt. The decision remains retrievable after browser refresh and backend restart.

## Workload

- 20–100 MiB plugin/WASM bundles.
- 5–20 deterministic scenarios per trial.
- 10–200 capability, dependency, data-sink, and evidence nodes.
- Concurrent bounded jobs with cancellation, fuel, memory, and elapsed-time budgets.
- Append-only decision events and content-addressed artifacts.

## Bounded contexts

1. Artifact Intake.
2. Capability Semantics.
3. Trial Execution.
4. Provenance Graph.
5. Admission Governance.
6. Evidence and Registry.

## Research hypothesis

Counterexample-guided capability attenuation combines static declarations, dynamic sandbox traces, and transitive graph closure to reduce false admission and excess permissions versus manifest-only, trace-only, and policy-only baselines without unacceptable task-success loss.

## Measurable success

- Lower false-admit rate and excess-capability count than every relevant baseline.
- No protected side effect on denial.
- Frozen task acceptance preserved after a proposed attenuation is applied and re-tried.
- All selected upstreams execute in product paths and fail safely under targeted tests.
- Primary, denial/recovery, and durable-retrieval browser journeys pass against the real stack.
- Exact-source evidence and semantic review pass without stale or rebound artifacts.

## Explicit non-goals

- General malware detection or a universal safety score.
- Native-code execution in the first release.
- Autonomous admission without a policy/human decision.
- Deployment, paid resources, or secret management changes.
- Treating signatures, manifests, or gate integrity as proof of safe behavior.
