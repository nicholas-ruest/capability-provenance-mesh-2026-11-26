//! Typed production integrations for the seven selected immutable upstreams.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns,
    clippy::return_self_not_must_use,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value
)]

mod agenticow;
mod autogenous;
mod common;
mod cosign;
mod latentmesh;
mod rugraph;
mod skillspector;
mod watermark;

pub use agenticow::{AGENTICOW_REPOSITORY, AGENTICOW_REVISION, AgentToolAdapter, ToolContract};
pub use autogenous::{
    AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION, AutogenesisAdapter, AutogenesisRequest,
};
pub use common::{IntegrationError, UpstreamEvidence};
pub use cosign::{COSIGN_REPOSITORY, COSIGN_REVISION, CosignAdapter, SignatureVerification};
pub use latentmesh::{LATENTMESH_REPOSITORY, LATENTMESH_REVISION, LatentMeshAdapter, MeshEnvelope};
pub use rugraph::{GraphQuery, RUGRAPH_REPOSITORY, RUGRAPH_REVISION, RuGraphAdapter};
pub use skillspector::{
    SKILLSPECTOR_REPOSITORY, SKILLSPECTOR_REVISION, SkillInspection, SkillSpectorAdapter,
};
pub use watermark::{WATERMARK_REPOSITORY, WATERMARK_REVISION, WatermarkAdapter, WatermarkProbe};

use cpm_upstream_runtime::ProbeKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct UpstreamSet {
    autogenous: AutogenesisAdapter,
    rugraph: RuGraphAdapter,
    agenticow: AgentToolAdapter,
    watermark: WatermarkAdapter,
    latentmesh: LatentMeshAdapter,
    skillspector: SkillSpectorAdapter,
    cosign: CosignAdapter,
}

impl UpstreamSet {
    pub fn from_harness_root(root: &Path, source_root: Option<PathBuf>) -> Self {
        let pair = |success: &str, failure: &str| {
            (
                Some(root.join(success)),
                Some(root.join(failure)),
                source_root.clone(),
            )
        };
        let (success, failure, argument) = pair("autogenous-success.sh", "autogenous-failure.sh");
        let autogenous = AutogenesisAdapter::new(success, failure, argument);
        let (success, failure, argument) = pair("rugraph-success.sh", "rugraph-failure.sh");
        let rugraph = RuGraphAdapter::new(success, failure, argument);
        let (success, failure, argument) = pair("agenticow-success.sh", "agenticow-failure.sh");
        let agenticow = AgentToolAdapter::new(success, failure, argument);
        let (success, failure, argument) = pair("watermark-success.sh", "watermark-failure.sh");
        let watermark = WatermarkAdapter::new(success, failure, argument);
        let (success, failure, argument) = pair("latentmesh-success.sh", "latentmesh-failure.sh");
        let latentmesh = LatentMeshAdapter::new(success, failure, argument);
        let (success, failure, argument) =
            pair("skillspector-success.sh", "skillspector-failure.sh");
        let skillspector = SkillSpectorAdapter::new(success, failure, argument);
        let (success, failure, argument) = pair("cosign-success.sh", "cosign-failure.sh");
        let cosign = CosignAdapter::new(success, failure, argument);
        Self {
            autogenous,
            rugraph,
            agenticow,
            watermark,
            latentmesh,
            skillspector,
            cosign,
        }
    }

    pub fn execute_all(
        &self,
        denial_probe: bool,
    ) -> Result<Vec<UpstreamEvidence>, IntegrationError> {
        let kind = if denial_probe {
            ProbeKind::Failure
        } else {
            ProbeKind::Success
        };
        let placeholder_sha = "a".repeat(64);
        let evidence = vec![
            self.autogenous.execute(
                kind,
                &AutogenesisRequest {
                    objective: "derive least-authority plugin plan".into(),
                    max_cycles: 8,
                },
            )?,
            self.rugraph.execute(
                kind,
                &GraphQuery {
                    source_node: "manifest".into(),
                    target_node: "observed-effect".into(),
                    max_depth: 12,
                },
            )?,
            self.agenticow.execute(
                kind,
                &ToolContract {
                    tool_name: "sandbox-probe".into(),
                    schema_sha256: placeholder_sha.clone(),
                    expected_effects: 1,
                },
            )?,
            self.watermark.execute(
                kind,
                &WatermarkProbe {
                    text_sha256: placeholder_sha.clone(),
                    minimum_score_basis_points: 7_500,
                },
            )?,
            self.latentmesh.execute(
                kind,
                &MeshEnvelope {
                    topic: "capability.provenance".into(),
                    payload_sha256: placeholder_sha.clone(),
                    hop_limit: 4,
                },
            )?,
            self.skillspector.execute(
                kind,
                &SkillInspection {
                    artifact_sha256: placeholder_sha.clone(),
                    policy_profile: "cpm-admission-v1".into(),
                    max_findings: 128,
                },
            )?,
            self.cosign.execute(
                kind,
                &SignatureVerification {
                    artifact_sha256: placeholder_sha,
                    issuer: "https://issuer.example".into(),
                    subject: "builder@example.test".into(),
                },
            )?,
        ];
        debug_assert_eq!(evidence.len(), 7);
        Ok(evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn immutable_bindings_are_unique_and_complete() {
        let bindings = [
            (AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION),
            (RUGRAPH_REPOSITORY, RUGRAPH_REVISION),
            (AGENTICOW_REPOSITORY, AGENTICOW_REVISION),
            (WATERMARK_REPOSITORY, WATERMARK_REVISION),
            (LATENTMESH_REPOSITORY, LATENTMESH_REVISION),
            (SKILLSPECTOR_REPOSITORY, SKILLSPECTOR_REVISION),
            (COSIGN_REPOSITORY, COSIGN_REVISION),
        ];
        assert_eq!(
            bindings
                .iter()
                .map(|(repo, _)| *repo)
                .collect::<BTreeSet<_>>()
                .len(),
            7
        );
        assert!(bindings.iter().all(|(_, revision)| revision.len() == 40));
    }
}
