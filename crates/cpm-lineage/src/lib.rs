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
    clippy::collapsible_if
)]
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    pub source: String,
    pub lock: String,
    pub plan: String,
    pub artifact: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSubject {
    pub name: String,
    pub digest: String,
    pub media_type: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStep {
    pub ordinal: u32,
    pub name: String,
    pub input_digests: Vec<String>,
    pub output_digest: String,
    pub builder: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attestation {
    pub predicate_type: String,
    pub subject: BuildSubject,
    pub source_digest: String,
    pub builder_id: String,
    pub materials: BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageChain {
    pub root: Lineage,
    pub steps: Vec<BuildStep>,
    pub attestation: Attestation,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageReport {
    pub stages: usize,
    pub builders: Vec<String>,
    pub materials: usize,
    pub final_digest: String,
    pub reproducible: bool,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LineageError {
    #[error("malformed digest")]
    Malformed,
    #[error("artifact digest mismatch")]
    Mismatch,
    #[error("lineage stage order mismatch: expected {expected}, got {actual}")]
    Order { expected: u32, actual: u32 },
    #[error("lineage stage {0} has no inputs")]
    MissingInput(String),
    #[error("lineage stage {0} is disconnected from previous output")]
    Disconnected(String),
    #[error("duplicate lineage stage {0}")]
    Duplicate(String),
    #[error("attestation subject does not match artifact")]
    Subject,
    #[error("attestation source does not match lineage source")]
    Source,
    #[error("builder identity is invalid")]
    Builder,
    #[error("material {0} is malformed")]
    Material(String),
    #[error("media type is invalid")]
    MediaType,
}
impl Lineage {
    pub fn verify(&self, artifact_bytes: &[u8]) -> Result<(), LineageError> {
        self.verify_digests()?;
        if digest(artifact_bytes) != self.artifact {
            return Err(LineageError::Mismatch);
        }
        Ok(())
    }
    pub fn verify_digests(&self) -> Result<(), LineageError> {
        if [&self.source, &self.lock, &self.plan, &self.artifact]
            .into_iter()
            .all(|s| valid_digest(s))
        {
            Ok(())
        } else {
            Err(LineageError::Malformed)
        }
    }
    pub fn binding_digest(&self) -> Result<String, LineageError> {
        self.verify_digests()?;
        Ok(hash_fields(&[
            &self.source,
            &self.lock,
            &self.plan,
            &self.artifact,
        ]))
    }
    pub fn changed_components(&self, next: &Self) -> Result<Vec<&'static str>, LineageError> {
        self.verify_digests()?;
        next.verify_digests()?;
        Ok([
            ("source", &self.source, &next.source),
            ("lock", &self.lock, &next.lock),
            ("plan", &self.plan, &next.plan),
            ("artifact", &self.artifact, &next.artifact),
        ]
        .into_iter()
        .filter_map(|(n, a, b)| (a != b).then_some(n))
        .collect())
    }
}
impl BuildSubject {
    pub fn validate(&self) -> Result<(), LineageError> {
        if self.name.is_empty() || self.name.len() > 160 {
            return Err(LineageError::Subject);
        }
        if !valid_digest(&self.digest) {
            return Err(LineageError::Malformed);
        }
        if !self.media_type.contains('/') || self.media_type.chars().any(char::is_whitespace) {
            return Err(LineageError::MediaType);
        }
        Ok(())
    }
}
impl BuildStep {
    pub fn validate(&self) -> Result<(), LineageError> {
        if self.name.is_empty() || self.name.len() > 96 {
            return Err(LineageError::Duplicate(self.name.clone()));
        }
        if self.input_digests.is_empty() {
            return Err(LineageError::MissingInput(self.name.clone()));
        }
        if self.input_digests.iter().any(|d| !valid_digest(d)) || !valid_digest(&self.output_digest)
        {
            return Err(LineageError::Malformed);
        }
        validate_builder(&self.builder)
    }
}
impl Attestation {
    pub fn validate(&self) -> Result<(), LineageError> {
        self.subject.validate()?;
        if !valid_digest(&self.source_digest) {
            return Err(LineageError::Malformed);
        }
        validate_builder(&self.builder_id)?;
        for (name, value) in &self.materials {
            if name.is_empty() || !valid_digest(value) {
                return Err(LineageError::Material(name.clone()));
            }
        }
        if self.predicate_type.is_empty() || !self.predicate_type.starts_with("https://") {
            return Err(LineageError::Subject);
        }
        Ok(())
    }
    pub fn canonical_digest(&self) -> Result<String, LineageError> {
        self.validate()?;
        let mut fields = vec![
            self.predicate_type.as_str(),
            self.subject.name.as_str(),
            self.subject.digest.as_str(),
            self.subject.media_type.as_str(),
            self.source_digest.as_str(),
            self.builder_id.as_str(),
        ];
        for (k, v) in &self.materials {
            fields.push(k);
            fields.push(v);
        }
        Ok(hash_fields(&fields))
    }
}
impl LineageChain {
    pub fn verify(&self, artifact_bytes: &[u8]) -> Result<LineageReport, LineageError> {
        self.root.verify(artifact_bytes)?;
        self.attestation.validate()?;
        if self.attestation.subject.digest != self.root.artifact {
            return Err(LineageError::Subject);
        }
        if self.attestation.source_digest != self.root.source {
            return Err(LineageError::Source);
        }
        let mut previous = BTreeSet::from([
            self.root.source.clone(),
            self.root.lock.clone(),
            self.root.plan.clone(),
        ]);
        let mut names = BTreeSet::new();
        let mut builders = BTreeSet::new();
        for (index, step) in self.steps.iter().enumerate() {
            step.validate()?;
            let expected = index as u32 + 1;
            if step.ordinal != expected {
                return Err(LineageError::Order {
                    expected,
                    actual: step.ordinal,
                });
            }
            if !names.insert(step.name.clone()) {
                return Err(LineageError::Duplicate(step.name.clone()));
            }
            if !step.input_digests.iter().any(|d| previous.contains(d)) {
                return Err(LineageError::Disconnected(step.name.clone()));
            }
            previous.insert(step.output_digest.clone());
            builders.insert(step.builder.clone());
        }
        if self
            .steps
            .last()
            .is_some_and(|s| s.output_digest != self.root.artifact)
        {
            return Err(LineageError::Subject);
        }
        builders.insert(self.attestation.builder_id.clone());
        let reproducible = self.attestation.materials.get("lock") == Some(&self.root.lock)
            && self.attestation.materials.get("plan") == Some(&self.root.plan);
        Ok(LineageReport {
            stages: self.steps.len(),
            builders: builders.into_iter().collect(),
            materials: self.attestation.materials.len(),
            final_digest: self.root.artifact.clone(),
            reproducible,
        })
    }
    pub fn step(&self, name: &str) -> Option<&BuildStep> {
        self.steps.iter().find(|s| s.name == name)
    }
    pub fn material_coverage(&self, required: &BTreeSet<String>) -> BTreeMap<String, bool> {
        required
            .iter()
            .map(|name| (name.clone(), self.attestation.materials.contains_key(name)))
            .collect()
    }
}
fn valid_digest(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn digest(v: &[u8]) -> String {
    format!("{:x}", Sha256::digest(v))
}
fn validate_builder(v: &str) -> Result<(), LineageError> {
    if v.len() < 3 || v.len() > 160 || v.chars().any(char::is_control) {
        Err(LineageError::Builder)
    } else {
        Ok(())
    }
}
fn hash_fields(fields: &[&str]) -> String {
    let mut h = Sha256::new();
    for f in fields {
        h.update((f.len() as u64).to_be_bytes());
        h.update(f.as_bytes());
    }
    format!("{:x}", h.finalize())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn lineage(bytes: &[u8]) -> Lineage {
        Lineage {
            source: "a".repeat(64),
            lock: "b".repeat(64),
            plan: "c".repeat(64),
            artifact: digest(bytes),
        }
    }
    fn attestation(l: &Lineage) -> Attestation {
        Attestation {
            predicate_type: "https://slsa.dev/provenance/v1".into(),
            subject: BuildSubject {
                name: "plugin.wasm".into(),
                digest: l.artifact.clone(),
                media_type: "application/wasm".into(),
            },
            source_digest: l.source.clone(),
            builder_id: "cpm-builder-v1".into(),
            materials: BTreeMap::from([
                ("lock".into(), l.lock.clone()),
                ("plan".into(), l.plan.clone()),
            ]),
        }
    }
    #[test]
    fn binds_artifact_bytes() {
        let l = lineage(b"wasm");
        assert_eq!(l.verify(b"wasm"), Ok(()));
        assert_eq!(l.verify(b"tamper"), Err(LineageError::Mismatch));
    }
    #[test]
    fn binding_digest_changes_with_lock() {
        let a = lineage(b"x");
        let mut b = a.clone();
        b.lock = "d".repeat(64);
        assert_ne!(a.binding_digest().unwrap(), b.binding_digest().unwrap());
        assert_eq!(a.changed_components(&b).unwrap(), vec!["lock"]);
    }
    #[test]
    fn chain_verifies_connected_build() {
        let l = lineage(b"artifact");
        let step = BuildStep {
            ordinal: 1,
            name: "compile".into(),
            input_digests: vec![l.source.clone(), l.lock.clone()],
            output_digest: l.artifact.clone(),
            builder: "rustc-1.91".into(),
        };
        let c = LineageChain {
            root: l.clone(),
            steps: vec![step],
            attestation: attestation(&l),
        };
        let r = c.verify(b"artifact").unwrap();
        assert_eq!(r.stages, 1);
        assert!(r.reproducible);
    }
    #[test]
    fn chain_rejects_disconnected_stage() {
        let l = lineage(b"artifact");
        let step = BuildStep {
            ordinal: 1,
            name: "compile".into(),
            input_digests: vec!["d".repeat(64)],
            output_digest: l.artifact.clone(),
            builder: "rustc".into(),
        };
        let c = LineageChain {
            root: l.clone(),
            steps: vec![step],
            attestation: attestation(&l),
        };
        assert!(matches!(
            c.verify(b"artifact"),
            Err(LineageError::Disconnected(_))
        ));
    }
    #[test]
    fn chain_rejects_out_of_order_stage() {
        let l = lineage(b"artifact");
        let step = BuildStep {
            ordinal: 2,
            name: "compile".into(),
            input_digests: vec![l.source.clone()],
            output_digest: l.artifact.clone(),
            builder: "rustc".into(),
        };
        let c = LineageChain {
            root: l.clone(),
            steps: vec![step],
            attestation: attestation(&l),
        };
        assert_eq!(
            c.verify(b"artifact"),
            Err(LineageError::Order {
                expected: 1,
                actual: 2
            })
        );
    }
    #[test]
    fn attestation_must_bind_source() {
        let l = lineage(b"artifact");
        let mut a = attestation(&l);
        a.source_digest = "d".repeat(64);
        let c = LineageChain {
            root: l,
            steps: vec![],
            attestation: a,
        };
        assert_eq!(c.verify(b"artifact"), Err(LineageError::Source));
    }
    #[test]
    fn material_coverage_is_explicit() {
        let l = lineage(b"artifact");
        let c = LineageChain {
            root: l.clone(),
            steps: vec![],
            attestation: attestation(&l),
        };
        let coverage = c.material_coverage(&BTreeSet::from(["lock".into(), "toolchain".into()]));
        assert!(coverage["lock"]);
        assert!(!coverage["toolchain"]);
    }
    #[test]
    fn media_type_must_be_structured() {
        let l = lineage(b"artifact");
        let mut a = attestation(&l);
        a.subject.media_type = "wasm".into();
        assert_eq!(a.validate(), Err(LineageError::MediaType));
    }
}
