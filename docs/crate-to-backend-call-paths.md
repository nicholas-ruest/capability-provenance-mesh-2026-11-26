# Crate-to-backend call paths

| Use case | Production call path | Result |
|---|---|---|
| intake | service to application to intake/lineage/manifest/capabilities to store/audit | workbench candidate |
| inspect | application to scan/SkillSpector/Cosign to receipts/graph/RuGraph | findings and graph |
| trial | application to quarantine/Agenticow/trial/LatentMesh to graph | observations |
| decide | application to risk/policy/completeness to audit | approved/denied/incomplete |
| remediate | application to Autogenous/remediation/watermark to intake | new digest |
| promote | application to policy/attestation/Cosign to registry/store/audit | atomic current |
| rollback | application to Autogenous/registry/store/audit | prior digest current |
| audit query | service to application to audit/store | restart timeline |

The composition root supplies every port. Denial paths never reach dispatch, promotion, or current-pointer writes.
