# Research plan

## Mechanism

Counterexample-guided capability attenuation iterates:

1. parse declared capability set;
2. run a frozen scenario pack;
3. normalize observed host/resource effects;
4. extend the set by transitive graph reachability;
5. solve for the smallest capability closure preserving all required tasks;
6. re-run the repaired manifest;
7. admit only when the second trace is contained and task acceptance remains above threshold.

## Benchmark corpus

At least 60 deterministic plugin fixtures covering exact manifests, overbroad manifests, undeclared host calls, transitive effects, stale lineage, replayed telemetry, resource exhaustion, and task regressions. Held-out mutations are not used to tune rules.

## Baselines

- Manifest-only static admission.
- Dynamic-trace-only admission.
- Cedar-only policy admission.

## Metrics

False admission/denial, excess permissions, accepted task rate, p50/p95 latency, peak RSS, evidence bytes, and human corrections.

## Ingredient ablations

- Remove WorldGraph closure.
- Remove RVM mapping.
- Remove Rune lineage.
- Remove LatentMesh signature/replay defense.
- Remove APx task eligibility.
- Remove Cedar policy.

Wasmtime and OpenDAL comparisons exercise controlled failure/volatile-state baselines; they do not authorize unsandboxed execution or a durable claim from memory.
