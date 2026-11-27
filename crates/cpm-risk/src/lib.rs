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

use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Informational = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskFactor {
    pub code: String,
    pub severity: Severity,
    pub confidence: u8,
    pub evidence_count: usize,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskProfile {
    pub threshold: u16,
    pub forbidden_capabilities: Vec<String>,
    pub minimum_evidence: usize,
    pub confidence_floor: u8,
}

impl Default for RiskProfile {
    fn default() -> Self {
        Self {
            threshold: 40,
            forbidden_capabilities: vec!["identity.delegate".into(), "process.exec".into()],
            minimum_evidence: 1,
            confidence_floor: 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskScore {
    pub exposure: u16,
    pub severity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskAssessment {
    pub score: RiskScore,
    pub factors: Vec<RiskFactor>,
    pub missing_evidence: bool,
    pub forbidden: Vec<String>,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RiskError {
    #[error("evidence is incomplete")]
    Incomplete,
    #[error("risk input overflow")]
    Overflow,
    #[error("invalid confidence {0}")]
    Confidence(u8),
    #[error("duplicate risk factor {0}")]
    Duplicate(String),
    #[error("profile threshold exceeds 100")]
    Threshold,
}

impl RiskScore {
    pub fn assess(evidence: usize, excess: usize, forbidden: usize) -> Result<Self, RiskError> {
        if evidence == 0 {
            return Err(RiskError::Incomplete);
        }
        let exposure = excess.checked_mul(20).ok_or(RiskError::Overflow)?;
        Ok(Self {
            exposure: u16::try_from(exposure.min(u16::MAX as usize))
                .map_err(|_| RiskError::Overflow)?,
            severity: u16::try_from(forbidden.saturating_mul(40)).unwrap_or(u16::MAX),
        })
    }
    pub fn acceptable(&self) -> bool {
        self.exposure < 30 && self.severity == 0
    }
    pub fn acceptable_under(&self, profile: &RiskProfile) -> bool {
        self.exposure < profile.threshold && self.severity < 40
    }
}

impl RiskAssessment {
    pub fn evaluate(
        observed: &[String],
        evidence_count: usize,
        mut factors: Vec<RiskFactor>,
        profile: &RiskProfile,
    ) -> Result<Self, RiskError> {
        if profile.threshold > 100 {
            return Err(RiskError::Threshold);
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &factors {
            if f.confidence > 100 {
                return Err(RiskError::Confidence(f.confidence));
            }
            if !seen.insert(f.code.clone()) {
                return Err(RiskError::Duplicate(f.code.clone()));
            }
        }
        let forbidden: Vec<_> = observed
            .iter()
            .filter(|v| {
                profile
                    .forbidden_capabilities
                    .iter()
                    .any(|p| *v == p || v.starts_with(&format!("{p}:")))
            })
            .cloned()
            .collect();
        let missing = evidence_count < profile.minimum_evidence;
        if missing {
            factors.push(RiskFactor {
                code: "evidence.missing".into(),
                severity: Severity::High,
                confidence: 100,
                evidence_count,
                explanation: "minimum observation evidence was not met".into(),
            });
        }
        for cap in &forbidden {
            factors.push(RiskFactor {
                code: format!("forbidden:{cap}"),
                severity: Severity::Critical,
                confidence: 100,
                evidence_count,
                explanation: "observed capability is forbidden by profile".into(),
            });
        }
        let mut weighted = 0u32;
        let mut max_severity = 0u8;
        let mut reasons = Vec::new();
        for f in &factors {
            let confidence = if f.confidence < profile.confidence_floor {
                profile.confidence_floor
            } else {
                f.confidence
            };
            let contribution = (f.severity as u32) * u32::from(confidence) / 4;
            weighted = weighted
                .checked_add(contribution)
                .ok_or(RiskError::Overflow)?;
            max_severity = max_severity.max(f.severity as u8);
            if f.severity >= Severity::High {
                reasons.push(f.explanation.clone());
            }
        }
        let score = RiskScore {
            exposure: u16::try_from(weighted.min(100)).map_err(|_| RiskError::Overflow)?,
            severity: u16::from(max_severity),
        };
        Ok(Self {
            score,
            factors,
            missing_evidence: missing,
            forbidden,
            decision_reasons: reasons,
        })
    }
    pub fn factor_counts(&self) -> BTreeMap<Severity, usize> {
        let mut m = BTreeMap::new();
        for f in &self.factors {
            *m.entry(f.severity).or_default() += 1;
        }
        m
    }
    pub fn acceptable(&self, profile: &RiskProfile) -> bool {
        self.forbidden.is_empty() && !self.missing_evidence && self.score.acceptable_under(profile)
    }
    pub fn dominant_factor(&self) -> Option<&RiskFactor> {
        self.factors
            .iter()
            .max_by_key(|f| (f.severity, f.confidence, f.evidence_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn factor(code: &str, severity: Severity) -> RiskFactor {
        RiskFactor {
            code: code.into(),
            severity,
            confidence: 80,
            evidence_count: 2,
            explanation: format!("{code} explanation"),
        }
    }
    #[test]
    fn legacy_score_denies_forbidden() {
        let s = RiskScore::assess(2, 0, 1).unwrap();
        assert!(!s.acceptable());
        assert_eq!(s.severity, 40);
    }
    #[test]
    fn missing_evidence_is_rejected_by_legacy_score() {
        assert_eq!(RiskScore::assess(0, 1, 0), Err(RiskError::Incomplete));
    }
    #[test]
    fn assessment_records_forbidden_capability() {
        let a = RiskAssessment::evaluate(
            &["process.exec:shell".into()],
            2,
            vec![],
            &RiskProfile::default(),
        )
        .unwrap();
        assert!(!a.acceptable(&RiskProfile::default()));
        assert_eq!(a.forbidden.len(), 1);
    }
    #[test]
    fn missing_evidence_is_a_factor() {
        let a = RiskAssessment::evaluate(&[], 0, vec![], &RiskProfile::default()).unwrap();
        assert!(a.missing_evidence);
        assert_eq!(a.dominant_factor().unwrap().code, "evidence.missing");
    }
    #[test]
    fn duplicates_are_rejected() {
        let p = RiskProfile::default();
        assert!(matches!(
            RiskAssessment::evaluate(
                &[],
                2,
                vec![factor("x", Severity::Low), factor("x", Severity::High)],
                &p
            ),
            Err(RiskError::Duplicate(_))
        ));
    }
    #[test]
    fn confidence_is_bounded() {
        let mut f = factor("x", Severity::Low);
        f.confidence = 101;
        assert_eq!(
            RiskAssessment::evaluate(&[], 2, vec![f], &RiskProfile::default()),
            Err(RiskError::Confidence(101))
        );
    }
    #[test]
    fn dominant_factor_uses_severity() {
        let a = RiskAssessment::evaluate(
            &[],
            2,
            vec![factor("low", Severity::Low), factor("high", Severity::High)],
            &RiskProfile::default(),
        )
        .unwrap();
        assert_eq!(a.dominant_factor().unwrap().code, "high");
    }
    #[test]
    fn factor_counts_are_stable() {
        let a = RiskAssessment::evaluate(
            &[],
            2,
            vec![factor("a", Severity::Low), factor("b", Severity::Low)],
            &RiskProfile::default(),
        )
        .unwrap();
        assert_eq!(a.factor_counts()[&Severity::Low], 2);
    }
}
