# Product specification

An operator can intake a capability pack, inspect source and behavior, understand its evidence graph, deny unsafe authority without downstream effects, remediate it, promote an approved digest, and retrieve the same decision after refresh and backend restart.

Required workflows cover candidate intake, lineage, declared capabilities, inspection, trial, decision, remediation, registry, and audit. Approved, Denied, and IncompleteEvidence are typed outcomes. One correlation ID links UI, middleware, Rust, upstreams, and storage. The accessible React UI renders loading, empty, error, denied, incomplete, approved, and restarted states.

Safety invariants: denial cannot dispatch or promote; repair creates a new digest; authority never widens; tool failure is not clean; watermark is not identity; receipt is not approval; promotion is human-authorized and rollback is audited.

Complete means all ten suites, three journeys, restart retrieval, schema-2 manifest/composition, independent semantic review, at least 50 percent genuine Rust, and governed publication pass on the same clean SHA.
