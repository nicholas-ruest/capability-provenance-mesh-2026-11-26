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
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Admit,
    Deny(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Missing,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionEvidence {
    pub source_sha: String,
    pub manifest_sha: String,
    pub scenario_sha: String,
    pub receipt_sha: String,
    pub upstreams: BTreeMap<String, String>,
    pub checks: BTreeMap<String, CheckStatus>,
    pub human_approver: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionPolicy {
    pub minimum_upstreams: usize,
    pub required_checks: BTreeSet<String>,
    pub require_human: bool,
    pub allowed_upstream_revisions: BTreeMap<String, BTreeSet<String>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionRecord {
    pub decision: Decision,
    pub passed_checks: Vec<String>,
    pub failed_checks: Vec<String>,
    pub evidence_fingerprint: String,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("fewer than seven upstreams executed")]
    Upstreams,
    #[error("human approval is required")]
    HumanApproval,
    #[error("evidence digest {0} is invalid")]
    Digest(&'static str),
    #[error("required check {0} is missing")]
    MissingCheck(String),
    #[error("check {0} failed: {1}")]
    FailedCheck(String, String),
    #[error("upstream {0} uses an unapproved revision")]
    Revision(String),
    #[error("approver identity is invalid")]
    Approver,
}
pub fn decide(
    risk_acceptable: bool,
    upstream_count: usize,
    human_approved: bool,
) -> Result<Decision, AdmissionError> {
    if upstream_count < 7 {
        return Err(AdmissionError::Upstreams);
    }
    if !human_approved {
        return Err(AdmissionError::HumanApproval);
    }
    Ok(if risk_acceptable {
        Decision::Admit
    } else {
        Decision::Deny("risk threshold exceeded".into())
    })
}
impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self {
            minimum_upstreams: 7,
            required_checks: BTreeSet::from([
                "manifest".into(),
                "scenarios".into(),
                "effects".into(),
                "risk".into(),
            ]),
            require_human: true,
            allowed_upstream_revisions: BTreeMap::new(),
        }
    }
}
impl AdmissionPolicy {
    pub fn evaluate(
        &self,
        e: &AdmissionEvidence,
        risk_acceptable: bool,
    ) -> Result<AdmissionRecord, AdmissionError> {
        for (name, value) in [
            ("source", &e.source_sha),
            ("manifest", &e.manifest_sha),
            ("scenario", &e.scenario_sha),
            ("receipt", &e.receipt_sha),
        ] {
            validate_digest(value).map_err(|()| AdmissionError::Digest(name))?;
        }
        if e.upstreams.len() < self.minimum_upstreams {
            return Err(AdmissionError::Upstreams);
        }
        if self.require_human {
            let approver = e
                .human_approver
                .as_ref()
                .ok_or(AdmissionError::HumanApproval)?;
            if approver.len() < 3 || approver.len() > 96 || approver.chars().any(char::is_control) {
                return Err(AdmissionError::Approver);
            }
        }
        for (name, allowed) in &self.allowed_upstream_revisions {
            if let Some(actual) = e.upstreams.get(name) {
                if !allowed.contains(actual) {
                    return Err(AdmissionError::Revision(name.clone()));
                }
            }
        }
        let mut passed = Vec::new();
        let mut failed = Vec::new();
        for name in &self.required_checks {
            match e.checks.get(name).unwrap_or(&CheckStatus::Missing) {
                CheckStatus::Pass => passed.push(name.clone()),
                CheckStatus::Fail(reason) => {
                    failed.push(name.clone());
                    return Err(AdmissionError::FailedCheck(name.clone(), reason.clone()));
                }
                CheckStatus::Missing => return Err(AdmissionError::MissingCheck(name.clone())),
            }
        }
        let decision = if risk_acceptable {
            Decision::Admit
        } else {
            Decision::Deny("risk threshold exceeded".into())
        };
        let fingerprint = fingerprint(e, &passed);
        Ok(AdmissionRecord {
            decision,
            passed_checks: passed,
            failed_checks: failed,
            evidence_fingerprint: fingerprint,
        })
    }
}
fn validate_digest(v: &str) -> Result<(), ()> {
    if v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(())
    }
}
fn fingerprint(e: &AdmissionEvidence, checks: &[String]) -> String {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
    for part in [
        &e.source_sha,
        &e.manifest_sha,
        &e.scenario_sha,
        &e.receipt_sha,
    ]
    .into_iter()
    .chain(checks.iter())
    {
        for b in part.as_bytes() {
            acc ^= u64::from(*b);
            acc = acc.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{acc:016x}")
}
#[cfg(test)]
mod tests {
    use super::*;
    fn evidence() -> AdmissionEvidence {
        AdmissionEvidence {
            source_sha: "a".repeat(64),
            manifest_sha: "b".repeat(64),
            scenario_sha: "c".repeat(64),
            receipt_sha: "d".repeat(64),
            upstreams: (0..7).map(|i| (format!("u{i}"), "e".repeat(40))).collect(),
            checks: ["manifest", "scenarios", "effects", "risk"]
                .into_iter()
                .map(|v| (v.into(), CheckStatus::Pass))
                .collect(),
            human_approver: Some("operator@example".into()),
        }
    }
    #[test]
    fn legacy_decision_preserves_contract() {
        assert_eq!(decide(true, 7, true).unwrap(), Decision::Admit);
        assert!(matches!(
            decide(true, 6, true),
            Err(AdmissionError::Upstreams)
        ));
    }
    #[test]
    fn risk_denial_is_a_valid_decision() {
        assert!(matches!(decide(false, 7, true).unwrap(), Decision::Deny(_)));
    }
    #[test]
    fn full_evidence_admits() {
        let r = AdmissionPolicy::default()
            .evaluate(&evidence(), true)
            .unwrap();
        assert_eq!(r.decision, Decision::Admit);
        assert_eq!(r.passed_checks.len(), 4);
    }
    #[test]
    fn failed_check_blocks_without_record() {
        let mut e = evidence();
        e.checks.insert(
            "effects".into(),
            CheckStatus::Fail("unexplained network".into()),
        );
        assert!(matches!(
            AdmissionPolicy::default().evaluate(&e, true),
            Err(AdmissionError::FailedCheck(_, _))
        ));
    }
    #[test]
    fn missing_check_is_distinct() {
        let mut e = evidence();
        e.checks.remove("risk");
        assert_eq!(
            AdmissionPolicy::default().evaluate(&e, true),
            Err(AdmissionError::MissingCheck("risk".into()))
        );
    }
    #[test]
    fn invalid_digest_fails_closed() {
        let mut e = evidence();
        e.source_sha = "branch".into();
        assert_eq!(
            AdmissionPolicy::default().evaluate(&e, true),
            Err(AdmissionError::Digest("source"))
        );
    }
    #[test]
    fn upstream_revision_allowlist_is_enforced() {
        let mut p = AdmissionPolicy::default();
        p.allowed_upstream_revisions
            .insert("u1".into(), BTreeSet::from(["f".repeat(40)]));
        assert_eq!(
            p.evaluate(&evidence(), true),
            Err(AdmissionError::Revision("u1".into()))
        );
    }
    #[test]
    fn fingerprint_changes_with_source() {
        let p = AdmissionPolicy::default();
        let a = p.evaluate(&evidence(), true).unwrap();
        let mut e = evidence();
        e.source_sha = "f".repeat(64);
        let b = p.evaluate(&e, true).unwrap();
        assert_ne!(a.evidence_fingerprint, b.evidence_fingerprint);
    }
}
