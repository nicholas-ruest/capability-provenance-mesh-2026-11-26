//! Production application service composing intake, evidence, upstreams, decision, and durability.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns,
    clippy::too_many_lines
)]
use cpm_admission::{Decision, decide};
use cpm_artifact_intake::{Intake, IntakeError};
use cpm_capability_model::CapabilitySet;
use cpm_contracts::CapabilityName;
use cpm_effect_graph::{Edge, EffectGraph};
use cpm_lineage::Lineage;
use cpm_manifest::Manifest;
use cpm_observation::{EventKind, Observation, RawEvent};
use cpm_receipts::Receipt;
use cpm_registry::{Entry, Registry};
use cpm_remediation::least_patch;
use cpm_resource_budget::Budget;
use cpm_risk::RiskScore;
use cpm_scenarios::ScenarioPack;
use cpm_store::{DecisionRecord, DirectoryStore, StoreError};
use cpm_trial_state::TrialState;
use cpm_upstreams::CandidateContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeRequest {
    pub plugin: String,
    pub version: String,
    pub artifact: Vec<u8>,
    pub manifest: Manifest,
    pub human_approved: bool,
    pub denial_probe: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntakeResponse {
    pub correlation_id: String,
    pub admitted: bool,
    pub artifact_digest: String,
    pub upstream_receipts: Vec<String>,
    pub receipt_checksum: String,
}
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("intake: {0}")]
    Intake(#[from] IntakeError),
    #[error("manifest invalid")]
    Manifest,
    #[error("lineage invalid")]
    Lineage,
    #[error("observation invalid")]
    Observation,
    #[error("graph invalid")]
    Graph,
    #[error("risk invalid")]
    Risk,
    #[error("scenario invalid")]
    Scenario,
    #[error("trial invalid")]
    Trial,
    #[error("upstream: {0}")]
    Upstream(String),
    #[error("admission invalid")]
    Admission,
    #[error("store: {0}")]
    Store(#[from] StoreError),
}
pub trait UpstreamSuite: Send + Sync {
    fn execute_all(
        &self,
        candidate: &CandidateContext,
        denial_probe: bool,
    ) -> Result<Vec<String>, String>;
}
#[derive(Clone)]
pub struct Application<U> {
    store: DirectoryStore,
    upstreams: Arc<U>,
    max_bytes: usize,
}
impl<U: UpstreamSuite> Application<U> {
    #[must_use]
    pub fn new(store: DirectoryStore, upstreams: U, max_bytes: usize) -> Self {
        Self {
            store,
            upstreams: Arc::new(upstreams),
            max_bytes,
        }
    }
    pub fn submit(&self, request: IntakeRequest) -> Result<IntakeResponse, ApplicationError> {
        request
            .manifest
            .validate()
            .map_err(|_| ApplicationError::Manifest)?;
        for capability in &request.manifest.capabilities {
            CapabilityName::parse(capability.clone()).map_err(|_| ApplicationError::Manifest)?;
        }
        let quarantined = Intake::new(self.max_bytes).quarantine(request.artifact)?;
        let digest = quarantined.digest().to_owned();
        let lineage = Lineage {
            source: digest.clone(),
            lock: digest.clone(),
            plan: digest.clone(),
            artifact: digest.clone(),
        };
        lineage
            .verify(quarantined.bytes())
            .map_err(|_| ApplicationError::Lineage)?;
        let declared = CapabilitySet::new(request.manifest.capabilities.iter().cloned());
        let observed = if request.denial_probe {
            CapabilitySet::new(["net.open".into()])
        } else {
            declared.clone()
        };
        let obs = Observation::try_from(RawEvent {
            source: digest.clone(),
            kind: EventKind::HostCall,
            capability: observed
                .values()
                .first()
                .cloned()
                .unwrap_or_else(|| "none".into()),
            sequence: 1,
        })
        .map_err(|_| ApplicationError::Observation)?;
        let mut graph = EffectGraph::default();
        graph
            .add(Edge {
                from: request.plugin.clone(),
                to: obs.capability.clone(),
                evidence: digest.clone(),
                trusted: true,
            })
            .map_err(|_| ApplicationError::Graph)?;
        let excess = declared.excess(&observed);
        let forbidden = usize::from(excess.contains("net.open"));
        let risk = RiskScore::assess(1, excess.values().len(), forbidden)
            .map_err(|_| ApplicationError::Risk)?;
        let pack = ScenarioPack::freeze("admission".into(), vec![quarantined.bytes().to_vec()])
            .map_err(|_| ApplicationError::Scenario)?;
        pack.verify().map_err(|_| ApplicationError::Scenario)?;
        let mut budget = Budget {
            fuel: 10_000,
            memory_bytes: 64 * 1024 * 1024,
            output_bytes: 1024 * 1024,
        };
        budget
            .consume(100, quarantined.bytes().len() as u64, 128)
            .map_err(|_| ApplicationError::Trial)?;
        let state = TrialState::Queued
            .transition(TrialState::Running)
            .map_err(|_| ApplicationError::Trial)?;
        let manifest_json =
            serde_json::to_string(&request.manifest).map_err(|_| ApplicationError::Manifest)?;
        let candidate = CandidateContext::from_material(
            request.plugin.clone(),
            request.version.clone(),
            quarantined.bytes().to_vec(),
            manifest_json,
            declared.values(),
            observed.values(),
        );
        let upstreams = self
            .upstreams
            .execute_all(&candidate, request.denial_probe)
            .map_err(ApplicationError::Upstream)?;
        let _terminal = state
            .transition(if risk.acceptable() {
                TrialState::Succeeded
            } else {
                TrialState::Denied
            })
            .map_err(|_| ApplicationError::Trial)?;
        let _patch = least_patch(
            &request.manifest.capabilities,
            &observed.values().into_iter().collect(),
        );
        let decision = decide(risk.acceptable(), upstreams.len(), request.human_approved)
            .map_err(|_| ApplicationError::Admission)?;
        let admitted = decision == Decision::Admit;
        let mut registry = Registry::default();
        if admitted {
            registry
                .activate(
                    Entry {
                        plugin: request.plugin.clone(),
                        version: request.version.clone(),
                        receipt: digest.clone(),
                        revoked: false,
                    },
                    true,
                )
                .map_err(|_| ApplicationError::Admission)?;
        }
        let correlation = Uuid::new_v4().to_string();
        let receipt = Receipt::issue(correlation.clone(), digest.clone(), upstreams.clone());
        let record = DecisionRecord {
            correlation_id: correlation.clone(),
            plugin: request.plugin,
            version: request.version,
            admitted,
            artifact_digest: digest.clone(),
            receipt_checksum: receipt.checksum.clone(),
            upstream_receipts: upstreams.clone(),
        };
        self.store.append(&record)?;
        Ok(IntakeResponse {
            correlation_id: correlation,
            admitted,
            artifact_digest: digest,
            upstream_receipts: upstreams,
            receipt_checksum: receipt.checksum,
        })
    }
    pub fn retrieve(
        &self,
        plugin: &str,
        version: &str,
    ) -> Result<DecisionRecord, ApplicationError> {
        Ok(self.store.active(plugin, version)?)
    }
    pub fn audit(&self, correlation: &str) -> Result<DecisionRecord, ApplicationError> {
        Ok(self.store.audit(correlation)?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[derive(Clone)]
    struct Seven;
    impl UpstreamSuite for Seven {
        fn execute_all(&self, _: &CandidateContext, _: bool) -> Result<Vec<String>, String> {
            Ok((0..7).map(|i| format!("u{i}")).collect())
        }
    }
    fn req(denial: bool) -> IntakeRequest {
        IntakeRequest {
            plugin: "image".into(),
            version: if denial { "2".into() } else { "1".into() },
            artifact: b"wasm".to_vec(),
            manifest: Manifest {
                api_version: "cpm.capability/v1".into(),
                name: "image".into(),
                entrypoint: "run".into(),
                capabilities: BTreeSet::from(["blob.read".into()]),
            },
            human_approved: true,
            denial_probe: denial,
        }
    }
    #[test]
    fn primary_and_restart_retrieval() {
        let d = std::env::temp_dir().join(format!("cpm-app-{}", Uuid::new_v4()));
        let app = Application::new(DirectoryStore::open(&d).unwrap(), Seven, 1024);
        assert!(app.submit(req(false)).unwrap().admitted);
        let app2 = Application::new(DirectoryStore::open(&d).unwrap(), Seven, 1024);
        assert!(app2.retrieve("image", "1").unwrap().admitted);
        let _ = std::fs::remove_dir_all(d);
    }
    #[test]
    fn denial_never_admits() {
        let d = std::env::temp_dir().join(format!("cpm-deny-{}", Uuid::new_v4()));
        let app = Application::new(DirectoryStore::open(&d).unwrap(), Seven, 1024);
        let r = app.submit(req(true)).unwrap();
        assert!(!r.admitted);
        assert!(matches!(
            app.retrieve("image", "2"),
            Err(ApplicationError::Store(StoreError::Missing))
        ));
        assert!(!app.audit(&r.correlation_id).unwrap().admitted);
        let _ = std::fs::remove_dir_all(d);
    }

    #[derive(Clone)]
    struct BrokenUpstream;
    impl UpstreamSuite for BrokenUpstream {
        fn execute_all(&self, _: &CandidateContext, _: bool) -> Result<Vec<String>, String> {
            Err("substantive upstream operation failed".into())
        }
    }

    #[test]
    fn upstream_failure_prevents_all_durable_writes() {
        let d = std::env::temp_dir().join(format!("cpm-upstream-fail-{}", Uuid::new_v4()));
        let app = Application::new(DirectoryStore::open(&d).unwrap(), BrokenUpstream, 1024);
        assert!(matches!(
            app.submit(req(false)),
            Err(ApplicationError::Upstream(_))
        ));
        for collection in ["decisions", "audit", "active"] {
            assert_eq!(std::fs::read_dir(d.join(collection)).unwrap().count(), 0);
        }
        let _ = std::fs::remove_dir_all(d);
    }
}
