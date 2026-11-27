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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateContext {
    pub plugin: String,
    pub version: String,
    pub artifact_sha256: String,
    pub manifest_sha256: String,
    pub artifact: Vec<u8>,
    pub manifest_json: String,
    pub declared_capabilities: Vec<String>,
    pub observed_effects: Vec<String>,
}

impl CandidateContext {
    #[must_use]
    pub fn from_material(
        plugin: String,
        version: String,
        artifact: Vec<u8>,
        manifest_json: String,
        declared_capabilities: Vec<String>,
        observed_effects: Vec<String>,
    ) -> Self {
        let artifact_sha256 = format!("{:x}", Sha256::digest(&artifact));
        let manifest_sha256 = format!("{:x}", Sha256::digest(manifest_json.as_bytes()));
        Self {
            plugin,
            version,
            artifact_sha256,
            manifest_sha256,
            artifact,
            manifest_json,
            declared_capabilities,
            observed_effects,
        }
    }

    pub fn validate(&self) -> Result<(), IntegrationError> {
        common::validate_identifier("plugin", &self.plugin)?;
        common::validate_identifier("version", &self.version)?;
        common::validate_sha256(&self.artifact_sha256)?;
        common::validate_sha256(&self.manifest_sha256)?;
        if self.artifact.is_empty() || self.manifest_json.is_empty() {
            return Err(IntegrationError::Validation {
                field: "candidate_material",
                reason: "artifact and manifest must be non-empty",
            });
        }
        if format!("{:x}", Sha256::digest(&self.artifact)) != self.artifact_sha256
            || format!("{:x}", Sha256::digest(self.manifest_json.as_bytes()))
                != self.manifest_sha256
        {
            return Err(IntegrationError::Validation {
                field: "candidate_material",
                reason: "material does not match declared digests",
            });
        }
        if self.declared_capabilities.is_empty() || self.observed_effects.is_empty() {
            return Err(IntegrationError::Validation {
                field: "candidate_evidence",
                reason: "capabilities and observed effects must be non-empty",
            });
        }
        for value in self
            .declared_capabilities
            .iter()
            .chain(self.observed_effects.iter())
        {
            common::validate_identifier("capability_or_effect", value)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn binding_sha256(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("candidate context serialization");
        format!("{:x}", Sha256::digest(bytes))
    }
}

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
        candidate: &CandidateContext,
        denial_probe: bool,
    ) -> Result<Vec<UpstreamEvidence>, IntegrationError> {
        candidate.validate()?;
        let kind = if denial_probe {
            ProbeKind::Failure
        } else {
            ProbeKind::Success
        };
        common::with_candidate(candidate, || {
            let first_effect = candidate.observed_effects[0].clone();
            let evidence = vec![
                self.autogenous.execute(
                    kind,
                    &AutogenesisRequest {
                        objective: format!(
                            "derive least-authority plan for {}@{} artifact {}",
                            candidate.plugin, candidate.version, candidate.artifact_sha256
                        ),
                        max_cycles: u16::try_from(candidate.declared_capabilities.len() + 4)
                            .unwrap_or(256),
                    },
                )?,
                self.rugraph.execute(
                    kind,
                    &GraphQuery {
                        source_node: format!("manifest:{}.{}", candidate.plugin, candidate.version),
                        target_node: format!(
                            "effect:{first_effect}:{}",
                            &candidate.artifact_sha256[..12]
                        ),
                        max_depth: 12,
                    },
                )?,
                self.agenticow.execute(
                    kind,
                    &ToolContract {
                        tool_name: candidate.plugin.clone(),
                        schema_sha256: candidate.manifest_sha256.clone(),
                        expected_effects: u16::try_from(candidate.observed_effects.len())
                            .unwrap_or(u16::MAX),
                    },
                )?,
                self.watermark.execute(
                    kind,
                    &WatermarkProbe {
                        text_sha256: candidate.artifact_sha256.clone(),
                        minimum_score_basis_points: 7_500,
                    },
                )?,
                self.latentmesh.execute(
                    kind,
                    &MeshEnvelope {
                        topic: format!("capability.provenance.{}", candidate.plugin),
                        payload_sha256: candidate.manifest_sha256.clone(),
                        hop_limit: 4,
                    },
                )?,
                self.skillspector.execute(
                    kind,
                    &SkillInspection {
                        artifact_sha256: candidate.artifact_sha256.clone(),
                        policy_profile: "cpm-admission-v1".into(),
                        max_findings: 128,
                    },
                )?,
                self.cosign.execute(
                    kind,
                    &SignatureVerification {
                        artifact_sha256: candidate.artifact_sha256.clone(),
                        issuer: "https://issuer.example".into(),
                        subject: format!("{}@{}", candidate.plugin, candidate.version),
                    },
                )?,
            ];
            debug_assert_eq!(evidence.len(), 7);
            Ok(evidence)
        })
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

    fn candidate(digest: char) -> CandidateContext {
        test_candidate(digest as u8)
    }

    #[test]
    fn every_upstream_receipt_is_candidate_bound() {
        let command = || Some(PathBuf::from("/bin/true"));
        let set = UpstreamSet {
            autogenous: AutogenesisAdapter::new(command(), command(), None),
            rugraph: RuGraphAdapter::new(command(), command(), None),
            agenticow: AgentToolAdapter::new(command(), command(), None),
            watermark: WatermarkAdapter::new(command(), command(), None),
            latentmesh: LatentMeshAdapter::new(command(), command(), None),
            skillspector: SkillSpectorAdapter::new(command(), command(), None),
            cosign: CosignAdapter::new(command(), command(), None),
        };
        let first = set.execute_all(&candidate('a'), false).unwrap();
        let second = set.execute_all(&candidate('b'), false).unwrap();
        assert_eq!(first.len(), 7);
        assert!(first.iter().zip(second.iter()).all(|(a, b)| {
            a.input_sha256 != b.input_sha256 && a.semantics_sha256 != b.semantics_sha256
        }));
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
pub(crate) fn test_candidate(seed: u8) -> CandidateContext {
    CandidateContext::from_material(
        "candidate-plugin".into(),
        "1.0.0".into(),
        vec![seed; 32],
        format!("{{\"seed\":{seed}}}"),
        vec!["blob.read".into()],
        vec!["blob.read".into()],
    )
}
