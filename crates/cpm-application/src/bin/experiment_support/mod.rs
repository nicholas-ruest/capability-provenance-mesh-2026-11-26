#![allow(dead_code)]

use cpm_application::{Application, IntakeRequest, UpstreamSuite};
use cpm_capability_model::CapabilitySet;
use cpm_effect_graph::{Edge, EffectGraph};
use cpm_lineage::Lineage;
use cpm_manifest::{Manifest, VERSION};
use cpm_store::DirectoryStore;
use cpm_upstreams::{
    AGENTICOW_REPOSITORY, AGENTICOW_REVISION, AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION,
    COSIGN_REPOSITORY, COSIGN_REVISION, CandidateContext, LATENTMESH_REPOSITORY,
    LATENTMESH_REVISION, RUGRAPH_REPOSITORY, RUGRAPH_REVISION, SKILLSPECTOR_REPOSITORY,
    SKILLSPECTOR_REVISION, WATERMARK_REPOSITORY, WATERMARK_REVISION,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    time::SystemTime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrontierFault {
    None,
    GraphGap,
    AuthorityEscalation,
    LineageMismatch,
    ConsensusGap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MechanismComponent {
    GraphCompleteness,
    AuthorityAttenuation,
    LineageEnforcement,
    UpstreamConsensus,
}

impl MechanismComponent {
    pub const ALL: [Self; 4] = [
        Self::GraphCompleteness,
        Self::AuthorityAttenuation,
        Self::LineageEnforcement,
        Self::UpstreamConsensus,
    ];

    pub const fn fault(self) -> FrontierFault {
        match self {
            Self::GraphCompleteness => FrontierFault::GraphGap,
            Self::AuthorityAttenuation => FrontierFault::AuthorityEscalation,
            Self::LineageEnforcement => FrontierFault::LineageMismatch,
            Self::UpstreamConsensus => FrontierFault::ConsensusGap,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MechanismConfig {
    enabled: BTreeSet<MechanismComponent>,
}

impl MechanismConfig {
    pub fn baseline() -> Self {
        Self {
            enabled: BTreeSet::new(),
        }
    }

    pub fn full() -> Self {
        Self {
            enabled: MechanismComponent::ALL.into_iter().collect(),
        }
    }

    pub fn without(component: MechanismComponent) -> Self {
        let mut config = Self::full();
        config.enabled.remove(&component);
        config
    }

    fn contains(&self, component: MechanismComponent) -> bool {
        self.enabled.contains(&component)
    }
}

#[derive(Debug, Clone)]
pub struct FrontierCase {
    pub id: String,
    pub fault: FrontierFault,
    pub artifact: Vec<u8>,
    pub declared: CapabilitySet,
    pub observed: CapabilitySet,
    pub graph: EffectGraph,
    pub lineage: Lineage,
    pub upstream_receipts: usize,
}

impl FrontierCase {
    pub fn expected_safe(&self) -> bool {
        self.fault == FrontierFault::None
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontierMetrics {
    pub decisions: usize,
    pub safe_candidates: usize,
    pub unsafe_candidates: usize,
    pub safe_admitted: usize,
    pub unsafe_admitted: usize,
    pub unsafe_blocked: usize,
    pub safe_acceptance_bps: usize,
    pub unsafe_block_bps: usize,
    pub balanced_quality_bps: usize,
    pub unsafe_admissions_by_fault: BTreeMap<FrontierFault, usize>,
    pub forbidden_active_writes: usize,
}

pub fn frontier_corpus() -> Vec<FrontierCase> {
    const REPETITIONS: usize = 5;
    let mut cases = Vec::with_capacity(REPETITIONS * 5);
    for repetition in 0..REPETITIONS {
        for fault in [
            FrontierFault::None,
            FrontierFault::GraphGap,
            FrontierFault::AuthorityEscalation,
            FrontierFault::LineageMismatch,
            FrontierFault::ConsensusGap,
        ] {
            cases.push(frontier_case(repetition, fault));
        }
    }
    cases
}

fn frontier_case(repetition: usize, fault: FrontierFault) -> FrontierCase {
    let artifact = format!("deterministic-frontier-artifact-{repetition}-{fault:?}").into_bytes();
    let declared = CapabilitySet::new(["blob.read".to_string()]);
    let observed_capability = if fault == FrontierFault::AuthorityEscalation {
        "net.open"
    } else {
        "blob.read"
    };
    let observed = CapabilitySet::new([observed_capability.to_string()]);
    let mut graph = EffectGraph::default();
    if fault != FrontierFault::GraphGap {
        graph
            .add(Edge {
                from: "artifact".into(),
                to: observed_capability.into(),
                evidence: digest(format!("edge-{repetition}-{fault:?}").as_bytes()),
                trusted: true,
            })
            .expect("frontier corpus edge must be valid");
    }
    let artifact_digest = if fault == FrontierFault::LineageMismatch {
        "f".repeat(64)
    } else {
        digest(&artifact)
    };
    FrontierCase {
        id: format!("frontier-{repetition}-{fault:?}"),
        fault,
        artifact,
        declared,
        observed,
        graph,
        lineage: Lineage {
            source: digest(format!("source-{repetition}").as_bytes()),
            lock: digest(format!("lock-{repetition}").as_bytes()),
            plan: digest(format!("plan-{repetition}").as_bytes()),
            artifact: artifact_digest,
        },
        upstream_receipts: if fault == FrontierFault::ConsensusGap {
            Ingredient::ALL.len() - 1
        } else {
            Ingredient::ALL.len()
        },
    }
}

pub fn frontier_admits(case: &FrontierCase, config: &MechanismConfig) -> bool {
    if config.contains(MechanismComponent::GraphCompleteness)
        && case
            .observed
            .values()
            .iter()
            .any(|effect| !case.graph.reachable("artifact", effect))
    {
        return false;
    }
    if config.contains(MechanismComponent::AuthorityAttenuation)
        && case.declared.attenuate(&case.observed).is_err()
    {
        return false;
    }
    if config.contains(MechanismComponent::LineageEnforcement)
        && case.lineage.verify(&case.artifact).is_err()
    {
        return false;
    }
    if config.contains(MechanismComponent::UpstreamConsensus)
        && case.upstream_receipts != Ingredient::ALL.len()
    {
        return false;
    }
    true
}

pub fn evaluate_frontier(cases: &[FrontierCase], config: &MechanismConfig) -> FrontierMetrics {
    let safe_candidates = cases.iter().filter(|case| case.expected_safe()).count();
    let unsafe_candidates = cases.len() - safe_candidates;
    let mut safe_admitted = 0;
    let mut unsafe_admitted = 0;
    let mut unsafe_admissions_by_fault = BTreeMap::new();
    for case in cases {
        if frontier_admits(case, config) {
            if case.expected_safe() {
                safe_admitted += 1;
            } else {
                unsafe_admitted += 1;
                *unsafe_admissions_by_fault.entry(case.fault).or_insert(0) += 1;
            }
        }
    }
    let unsafe_blocked = unsafe_candidates - unsafe_admitted;
    let safe_acceptance_bps = ratio_bps(safe_admitted, safe_candidates);
    let unsafe_block_bps = ratio_bps(unsafe_blocked, unsafe_candidates);
    FrontierMetrics {
        decisions: cases.len(),
        safe_candidates,
        unsafe_candidates,
        safe_admitted,
        unsafe_admitted,
        unsafe_blocked,
        safe_acceptance_bps,
        unsafe_block_bps,
        balanced_quality_bps: safe_acceptance_bps.midpoint(unsafe_block_bps),
        unsafe_admissions_by_fault,
        // The frontier evaluator is deliberately pre-persistence. An unsafe decision is
        // observable in the metrics but is never forwarded to DirectoryStore.
        forbidden_active_writes: 0,
    }
}

fn ratio_bps(numerator: usize, denominator: usize) -> usize {
    numerator
        .saturating_mul(10_000)
        .checked_div(denominator)
        .unwrap_or(0)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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

    #[test]
    fn full_frontier_mechanism_blocks_each_distinct_fault() {
        let cases = frontier_corpus();
        let metrics = evaluate_frontier(&cases, &MechanismConfig::full());
        assert_eq!(metrics.safe_admitted, 5);
        assert_eq!(metrics.unsafe_admitted, 0);
        assert_eq!(metrics.balanced_quality_bps, 10_000);
        assert_eq!(metrics.forbidden_active_writes, 0);
    }

    #[test]
    fn each_component_ablation_exposes_only_its_target_fault() {
        let cases = frontier_corpus();
        for component in MechanismComponent::ALL {
            let metrics = evaluate_frontier(&cases, &MechanismConfig::without(component));
            assert_eq!(metrics.safe_admitted, 5);
            assert_eq!(metrics.unsafe_admitted, 5);
            assert_eq!(metrics.balanced_quality_bps, 8_750);
            assert_eq!(
                metrics.unsafe_admissions_by_fault,
                BTreeMap::from([(component.fault(), 5)])
            );
            assert_eq!(metrics.forbidden_active_writes, 0);
        }
    }
}
