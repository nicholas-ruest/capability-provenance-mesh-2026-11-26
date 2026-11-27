//! Human-reviewed evidence-closed admission decisions.
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Admit,
    Deny,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("seven upstream receipts required")]
    MissingUpstreams,
    #[error("human review required")]
    MissingReview,
}
pub fn decide(
    risk_acceptable: bool,
    upstream_receipts: usize,
    human_approved: bool,
) -> Result<Decision, AdmissionError> {
    if upstream_receipts < 7 {
        return Err(AdmissionError::MissingUpstreams);
    }
    if !human_approved {
        return Err(AdmissionError::MissingReview);
    }
    Ok(if risk_acceptable {
        Decision::Admit
    } else {
        Decision::Deny
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admits_closed_evidence() {
        assert_eq!(decide(true, 7, true), Ok(Decision::Admit))
    }
    #[test]
    fn refuses_partial() {
        assert_eq!(decide(true, 6, true), Err(AdmissionError::MissingUpstreams))
    }
}
