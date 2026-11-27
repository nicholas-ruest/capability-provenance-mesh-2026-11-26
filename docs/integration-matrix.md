# Executed integration matrix

| Project | Category | Production responsibility | Adapter / caller | Intended operation | Substantive failure |
|---|---|---|---|---|---|
| Autogenous | Ruvnet | scoped remediation and rollback | cpm-autogenous / cpm-remediation | patch denied candidate in child | invalid patch or rollback mismatch |
| RuGraph | Ruvnet | causal graph and replay | cpm-rugraph / cpm-graph | materialize required edges | missing edge blocks completeness |
| Agenticow | Ruvnet | isolated child branch | cpm-agenticow / cpm-quarantine | execute child mutation | escape leaves parent unchanged |
| AI Text Watermark | Ruvnet | generated explanation provenance | cpm-watermark / cpm-remediation | mark and detect narrative | absent mark reported, never identity |
| LatentMesh | Ruvnet | signed bounded envelopes | cpm-latentmesh / cpm-receipts | verify fresh envelope | signature, expiry, size, replay reject |
| SkillSpector | Enterprise | malicious capability inspection | cpm-skillspector / cpm-scan | inspect benign/malicious fixtures | execution or parse failure is incomplete |
| Cosign | Enterprise | offline attestation | cpm-cosign / cpm-attestation | sign/verify candidate digest | tampered bundle rejects |

All seven are collectively exercised by production workflows with immutable pins, OSS licenses, real call sites, intended/failure tests, and raw source-bound receipts. Help, version, and import-only probes do not count.
