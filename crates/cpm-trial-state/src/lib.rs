//! Idempotent trial lifecycle.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialState {
    Queued,
    Running,
    Succeeded,
    Denied,
    Cancelled,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransitionError {
    #[error("illegal transition")]
    Illegal,
}
impl TrialState {
    pub fn transition(self, next: Self) -> Result<Self, TransitionError> {
        let ok = matches!(
            (self, next),
            (Self::Queued, Self::Running)
                | (Self::Queued, Self::Cancelled)
                | (Self::Running, Self::Succeeded)
                | (Self::Running, Self::Denied)
                | (Self::Running, Self::Cancelled)
        );
        if ok {
            Ok(next)
        } else {
            Err(TransitionError::Illegal)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lifecycle() {
        assert_eq!(
            TrialState::Queued.transition(TrialState::Running),
            Ok(TrialState::Running)
        )
    }
    #[test]
    fn terminal() {
        assert_eq!(
            TrialState::Denied.transition(TrialState::Running),
            Err(TransitionError::Illegal)
        )
    }
}
