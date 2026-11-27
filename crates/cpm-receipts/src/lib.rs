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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
const MAX_UPSTREAM_RECEIPT_BYTES: usize = 320;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub correlation_id: String,
    pub artifact_sha: String,
    pub upstreams: Vec<String>,
    pub checksum: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptLink {
    pub sequence: u64,
    pub receipt: Receipt,
    pub previous_checksum: Option<String>,
    pub chain_checksum: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReceiptChain {
    links: Vec<ReceiptLink>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub receipts: usize,
    pub unique_artifacts: usize,
    pub unique_upstreams: usize,
    pub head: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptBundle {
    pub source_sha: String,
    pub policy_sha: String,
    pub scenario_sha: String,
    pub receipts: Vec<Receipt>,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReceiptError {
    #[error("receipt checksum mismatch")]
    Checksum,
    #[error("receipt field {0} is invalid")]
    Field(&'static str),
    #[error("receipt chain sequence mismatch: expected {expected}, got {actual}")]
    Sequence { expected: u64, actual: u64 },
    #[error("receipt chain previous checksum mismatch")]
    Previous,
    #[error("duplicate receipt correlation {0}")]
    Duplicate(String),
    #[error("receipt bundle is empty")]
    Empty,
}
impl Receipt {
    pub fn issue(
        correlation_id: impl Into<String>,
        artifact_sha: impl Into<String>,
        upstreams: Vec<String>,
    ) -> Self {
        Self::try_issue(correlation_id, artifact_sha, upstreams)
            .expect("receipt inputs must be valid")
    }
    pub fn try_issue(
        correlation_id: impl Into<String>,
        artifact_sha: impl Into<String>,
        mut upstreams: Vec<String>,
    ) -> Result<Self, ReceiptError> {
        let correlation_id = correlation_id.into();
        let artifact_sha = artifact_sha.into();
        validate_fields(&correlation_id, &artifact_sha, &upstreams)?;
        upstreams.sort();
        upstreams.dedup();
        let checksum = receipt_hash(&correlation_id, &artifact_sha, &upstreams);
        Ok(Self {
            correlation_id,
            artifact_sha,
            upstreams,
            checksum,
        })
    }
    pub fn verify(&self) -> Result<(), ReceiptError> {
        validate_fields(&self.correlation_id, &self.artifact_sha, &self.upstreams)?;
        if receipt_hash(&self.correlation_id, &self.artifact_sha, &self.upstreams) != self.checksum
        {
            return Err(ReceiptError::Checksum);
        }
        Ok(())
    }
    pub fn covers_upstreams(&self, required: &BTreeSet<String>) -> bool {
        let actual: BTreeSet<_> = self.upstreams.iter().cloned().collect();
        required.is_subset(&actual)
    }
    pub fn evidence_map(&self) -> BTreeMap<String, String> {
        self.upstreams
            .iter()
            .map(|u| (u.clone(), self.checksum.clone()))
            .collect()
    }
}
impl ReceiptChain {
    pub fn append(&mut self, receipt: Receipt) -> Result<&ReceiptLink, ReceiptError> {
        receipt.verify()?;
        if self
            .links
            .iter()
            .any(|l| l.receipt.correlation_id == receipt.correlation_id)
        {
            return Err(ReceiptError::Duplicate(receipt.correlation_id));
        }
        let sequence = self.links.len() as u64 + 1;
        let previous = self.links.last().map(|l| l.chain_checksum.clone());
        let chain_checksum = chain_hash(sequence, previous.as_deref(), &receipt.checksum);
        self.links.push(ReceiptLink {
            sequence,
            receipt,
            previous_checksum: previous,
            chain_checksum,
        });
        Ok(self.links.last().unwrap())
    }
    pub fn verify(&self) -> Result<VerificationReport, ReceiptError> {
        let mut correlations = BTreeSet::new();
        let mut artifacts = BTreeSet::new();
        let mut upstreams = BTreeSet::new();
        let mut previous: Option<&str> = None;
        for (index, link) in self.links.iter().enumerate() {
            let expected = index as u64 + 1;
            if link.sequence != expected {
                return Err(ReceiptError::Sequence {
                    expected,
                    actual: link.sequence,
                });
            }
            link.receipt.verify()?;
            if !correlations.insert(link.receipt.correlation_id.clone()) {
                return Err(ReceiptError::Duplicate(link.receipt.correlation_id.clone()));
            }
            if link.previous_checksum.as_deref() != previous {
                return Err(ReceiptError::Previous);
            }
            if link.chain_checksum != chain_hash(expected, previous, &link.receipt.checksum) {
                return Err(ReceiptError::Checksum);
            }
            previous = Some(&link.chain_checksum);
            artifacts.insert(link.receipt.artifact_sha.clone());
            upstreams.extend(link.receipt.upstreams.iter().cloned());
        }
        Ok(VerificationReport {
            receipts: self.links.len(),
            unique_artifacts: artifacts.len(),
            unique_upstreams: upstreams.len(),
            head: self.links.last().map(|l| l.chain_checksum.clone()),
        })
    }
    pub fn links(&self) -> &[ReceiptLink] {
        &self.links
    }
    pub fn contains_artifact(&self, sha: &str) -> bool {
        self.links.iter().any(|l| l.receipt.artifact_sha == sha)
    }
}
impl ReceiptBundle {
    pub fn verify(&self) -> Result<VerificationReport, ReceiptError> {
        for (name, value) in [
            ("source_sha", &self.source_sha),
            ("policy_sha", &self.policy_sha),
            ("scenario_sha", &self.scenario_sha),
        ] {
            if !valid_digest(value) {
                return Err(ReceiptError::Field(name));
            }
        }
        if self.receipts.is_empty() {
            return Err(ReceiptError::Empty);
        }
        let mut chain = ReceiptChain::default();
        for receipt in &self.receipts {
            chain.append(receipt.clone())?;
        }
        chain.verify()
    }
}
fn validate_fields(c: &str, a: &str, u: &[String]) -> Result<(), ReceiptError> {
    if c.len() < 8 || c.len() > 128 || c.chars().any(char::is_control) {
        return Err(ReceiptError::Field("correlation_id"));
    }
    if !valid_digest(a) {
        return Err(ReceiptError::Field("artifact_sha"));
    }
    if u.is_empty()
        || u.iter()
            .any(|v| v.is_empty() || v.len() > MAX_UPSTREAM_RECEIPT_BYTES)
    {
        return Err(ReceiptError::Field("upstreams"));
    }
    Ok(())
}
fn valid_digest(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn add(h: &mut Sha256, v: &str) {
    h.update((v.len() as u64).to_be_bytes());
    h.update(v.as_bytes());
}
fn receipt_hash(c: &str, a: &str, u: &[String]) -> String {
    let mut h = Sha256::new();
    add(&mut h, "cpm-receipt-v1");
    add(&mut h, c);
    add(&mut h, a);
    for v in u {
        add(&mut h, v);
    }
    format!("{:x}", h.finalize())
}
fn chain_hash(sequence: u64, previous: Option<&str>, receipt: &str) -> String {
    let mut h = Sha256::new();
    h.update(sequence.to_be_bytes());
    add(&mut h, previous.unwrap_or("genesis"));
    add(&mut h, receipt);
    format!("{:x}", h.finalize())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn receipt(id: &str, artifact: &str) -> Receipt {
        Receipt::issue(
            id,
            "a".repeat(64).replace('a', artifact),
            vec!["worldgraph@abc".into(), "latentmesh@def".into()],
        )
    }
    #[test]
    fn issued_receipt_verifies() {
        receipt("corr-0001", "a").verify().unwrap();
    }
    #[test]
    fn mutation_breaks_checksum() {
        let mut r = receipt("corr-0001", "a");
        r.upstreams.push("other@rev".into());
        assert_eq!(r.verify(), Err(ReceiptError::Checksum));
    }
    #[test]
    fn chain_verifies_order_and_head() {
        let mut c = ReceiptChain::default();
        c.append(receipt("corr-0001", "a")).unwrap();
        c.append(receipt("corr-0002", "b")).unwrap();
        let report = c.verify().unwrap();
        assert_eq!(report.receipts, 2);
        assert!(report.head.is_some());
    }
    #[test]
    fn duplicate_correlation_fails_without_append() {
        let mut c = ReceiptChain::default();
        c.append(receipt("corr-0001", "a")).unwrap();
        assert!(matches!(
            c.append(receipt("corr-0001", "b")),
            Err(ReceiptError::Duplicate(_))
        ));
        assert_eq!(c.links().len(), 1);
    }
    #[test]
    fn required_upstream_coverage_is_explicit() {
        let r = receipt("corr-0001", "a");
        assert!(r.covers_upstreams(&BTreeSet::from(["worldgraph@abc".into()])));
        assert!(!r.covers_upstreams(&BTreeSet::from(["missing@rev".into()])));
    }
    #[test]
    fn length_prefix_avoids_field_ambiguity() {
        let a = Receipt::issue("corr-0001", "a".repeat(64), vec!["ab".into(), "c".into()]);
        let b = Receipt::issue("corr-0001", "a".repeat(64), vec!["a".into(), "bc".into()]);
        assert_ne!(a.checksum, b.checksum);
    }
    #[test]
    fn bundle_requires_immutable_source() {
        let b = ReceiptBundle {
            source_sha: "branch".into(),
            policy_sha: "b".repeat(64),
            scenario_sha: "c".repeat(64),
            receipts: vec![receipt("corr-0001", "a")],
        };
        assert_eq!(b.verify(), Err(ReceiptError::Field("source_sha")));
    }
    #[test]
    fn provenance_receipt_length_is_bounded() {
        assert!(
            Receipt::try_issue(
                "corr-0001",
                "a".repeat(64),
                vec!["x".repeat(MAX_UPSTREAM_RECEIPT_BYTES)]
            )
            .is_ok()
        );
        assert_eq!(
            Receipt::try_issue(
                "corr-0001",
                "a".repeat(64),
                vec!["x".repeat(MAX_UPSTREAM_RECEIPT_BYTES + 1)]
            ),
            Err(ReceiptError::Field("upstreams"))
        );
    }
    #[test]
    fn invalid_artifact_is_rejected() {
        assert_eq!(
            Receipt::try_issue("corr-0001", "branch", vec!["u".into()]),
            Err(ReceiptError::Field("artifact_sha"))
        );
    }
}
