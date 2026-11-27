#![allow(dead_code)]

use cpm_application::{Application, IntakeRequest, UpstreamSuite};
use cpm_manifest::{Manifest, VERSION};
use cpm_store::DirectoryStore;
use cpm_upstreams::{
    AGENTICOW_REPOSITORY, AGENTICOW_REVISION, AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION,
    COSIGN_REPOSITORY, COSIGN_REVISION, CandidateContext, LATENTMESH_REPOSITORY,
    LATENTMESH_REVISION, RUGRAPH_REPOSITORY, RUGRAPH_REVISION, SKILLSPECTOR_REPOSITORY,
    SKILLSPECTOR_REVISION, WATERMARK_REPOSITORY, WATERMARK_REVISION,
};
use serde::Serialize;
use std::{collections::BTreeSet, fs, path::Path, time::SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ingredient {
    Autogenous,
    RuGraph,
    AgentiCow,
    Watermark,
    LatentMesh,
    SkillSpector,
    Cosign,
}

impl Ingredient {
    pub const ALL: [Self; 7] = [
        Self::Autogenous,
        Self::RuGraph,
        Self::AgentiCow,
        Self::Watermark,
        Self::LatentMesh,
        Self::SkillSpector,
        Self::Cosign,
    ];

    fn binding(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Autogenous => (
                AUTOGENOUS_REPOSITORY,
                AUTOGENOUS_REVISION,
                "bounded-capability-plan-synthesis",
            ),
            Self::RuGraph => (
                RUGRAPH_REPOSITORY,
                RUGRAPH_REVISION,
                "effect-graph-provenance-traversal",
            ),
            Self::AgentiCow => (
                AGENTICOW_REPOSITORY,
                AGENTICOW_REVISION,
                "typed-agent-tool-contract-execution",
            ),
            Self::Watermark => (
                WATERMARK_REPOSITORY,
                WATERMARK_REVISION,
                "generated-text-provenance-detection",
            ),
            Self::LatentMesh => (
                LATENTMESH_REPOSITORY,
                LATENTMESH_REVISION,
                "constrained-provenance-envelope-transport",
            ),
            Self::SkillSpector => (
                SKILLSPECTOR_REPOSITORY,
                SKILLSPECTOR_REVISION,
                "agent-skill-policy-inspection",
            ),
            Self::Cosign => (
                COSIGN_REPOSITORY,
                COSIGN_REVISION,
                "keyless-artifact-identity-verification",
            ),
        }
    }

    fn receipt(self, denial: bool) -> String {
        let (repository, revision, responsibility) = self.binding();
        let outcome = if denial {
            "asserted-failure"
        } else {
            "verified-success"
        };
        format!("{repository}@{revision}:{responsibility}:{outcome}")
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceSuite {
    omitted: Option<Ingredient>,
}

impl EvidenceSuite {
    pub const fn full() -> Self {
        Self { omitted: None }
    }

    pub const fn without(ingredient: Ingredient) -> Self {
        Self {
            omitted: Some(ingredient),
        }
    }
}

impl UpstreamSuite for EvidenceSuite {
    fn execute_all(
        &self,
        candidate: &CandidateContext,
        denial: bool,
    ) -> Result<Vec<String>, String> {
        Ok(Ingredient::ALL
            .into_iter()
            .filter(|ingredient| self.omitted != Some(*ingredient))
            .map(|ingredient| {
                format!(
                    "{}:{}",
                    ingredient.receipt(denial),
                    candidate.binding_sha256()
                )
            })
            .collect())
    }
}

pub fn request(version: impl Into<String>, denial_probe: bool) -> IntakeRequest {
    IntakeRequest {
        plugin: "fixed-workload-plugin".into(),
        version: version.into(),
        artifact: b"deterministic-capability-provenance-wasm".to_vec(),
        manifest: Manifest {
            api_version: VERSION.into(),
            name: "fixed-workload-plugin".into(),
            entrypoint: "run".into(),
            capabilities: BTreeSet::from(["blob.read".into()]),
        },
        human_approved: true,
        denial_probe,
    }
}

pub fn application(root: &Path, suite: EvidenceSuite) -> Application<EvidenceSuite> {
    Application::new(
        DirectoryStore::open(root).expect("experiment store must open"),
        suite,
        1024 * 1024,
    )
}

pub fn unique_root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cpm-{label}-{}-{nonce}", std::process::id()))
}

pub fn file_count(root: &Path, collection: &str) -> usize {
    match fs::read_dir(root.join(collection)) {
        Ok(entries) => entries.filter_map(Result::ok).count(),
        Err(_) => 0,
    }
}

pub fn manifest_only_admits(request: &IntakeRequest) -> bool {
    request.human_approved && request.manifest.validate().is_ok() && !request.artifact.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_suite_is_source_bound_and_leave_one_out_is_six() {
        let candidate = CandidateContext {
            plugin: "fixed-workload-plugin".into(),
            version: "1.0.0".into(),
            artifact_sha256: "a".repeat(64),
            manifest_sha256: "b".repeat(64),
            declared_capabilities: vec!["blob.read".into()],
            observed_effects: vec!["blob.read".into()],
        };
        let full = EvidenceSuite::full()
            .execute_all(&candidate, false)
            .unwrap();
        assert_eq!(full.len(), 7);
        assert!(full.iter().all(|receipt| receipt.contains('@')));
        assert_eq!(
            EvidenceSuite::without(Ingredient::Cosign)
                .execute_all(&candidate, false)
                .unwrap()
                .len(),
            6
        );
    }

    #[test]
    fn manifest_only_baseline_misses_observed_effect() {
        assert!(manifest_only_admits(&request("unsafe", true)));
    }
}
