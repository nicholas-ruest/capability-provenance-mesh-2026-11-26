//! Explainable risk scoring that refuses incomplete evidence.
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskScore {
    pub exposure: u16,
    pub severity: u16,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RiskError {
    #[error("evidence is incomplete")]
    Incomplete,
}
impl RiskScore {
    pub fn assess(evidence: usize, excess: usize, forbidden: usize) -> Result<Self, RiskError> {
        if evidence == 0 {
            return Err(RiskError::Incomplete);
        }
        Ok(Self {
            exposure: u16::try_from(excess.saturating_mul(10)).unwrap_or(u16::MAX),
            severity: u16::try_from(forbidden.saturating_mul(40)).unwrap_or(u16::MAX),
        })
    }
    #[must_use]
    pub const fn acceptable(self) -> bool {
        self.exposure < 30 && self.severity == 0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe() {
        assert!(RiskScore::assess(2, 1, 0).unwrap().acceptable())
    }
    #[test]
    fn refuses_missing() {
        assert_eq!(RiskScore::assess(0, 0, 0), Err(RiskError::Incomplete))
    }
}
