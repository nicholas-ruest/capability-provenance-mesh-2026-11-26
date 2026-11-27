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
use std::collections::BTreeSet;
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrialState {
    Queued,
    Running,
    Succeeded,
    Denied,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialEvent {
    pub revision: u64,
    pub from: TrialState,
    pub to: TrialState,
    pub reason: String,
    pub correlation_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialAggregate {
    pub id: String,
    pub state: TrialState,
    pub revision: u64,
    events: Vec<TrialEvent>,
    correlations: BTreeSet<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialSnapshot {
    pub id: String,
    pub state: TrialState,
    pub revision: u64,
    pub event_count: usize,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TrialError {
    #[error("invalid transition from {0:?} to {1:?}")]
    Invalid(TrialState, TrialState),
    #[error("trial id is invalid")]
    InvalidId,
    #[error("transition reason is required")]
    MissingReason,
    #[error("correlation id is invalid")]
    Correlation,
    #[error("duplicate transition correlation {0}")]
    Duplicate(String),
    #[error("revision mismatch: expected {expected}, actual {actual}")]
    Revision { expected: u64, actual: u64 },
    #[error("event stream is inconsistent at revision {0}")]
    Replay(u64),
}
impl TrialState {
    pub fn transition(&self, next: TrialState) -> Result<TrialState, TrialError> {
        if allowed(self, &next) {
            Ok(next)
        } else {
            Err(TrialError::Invalid(self.clone(), next))
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Denied | Self::Cancelled)
    }
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}
impl TrialAggregate {
    pub fn new(id: impl Into<String>) -> Result<Self, TrialError> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 96
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(TrialError::InvalidId);
        }
        Ok(Self {
            id,
            state: TrialState::Queued,
            revision: 0,
            events: vec![],
            correlations: BTreeSet::new(),
        })
    }
    pub fn apply(
        &mut self,
        next: TrialState,
        reason: impl Into<String>,
        correlation_id: impl Into<String>,
        expected_revision: u64,
    ) -> Result<&TrialEvent, TrialError> {
        if self.revision != expected_revision {
            return Err(TrialError::Revision {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        let reason = reason.into();
        if reason.trim().is_empty() || reason.len() > 256 {
            return Err(TrialError::MissingReason);
        }
        let correlation_id = correlation_id.into();
        if correlation_id.len() < 8 || correlation_id.len() > 128 {
            return Err(TrialError::Correlation);
        }
        if self.correlations.contains(&correlation_id) {
            return Err(TrialError::Duplicate(correlation_id));
        }
        let to = self.state.transition(next)?;
        let event = TrialEvent {
            revision: self.revision + 1,
            from: self.state.clone(),
            to: to.clone(),
            reason,
            correlation_id: correlation_id.clone(),
        };
        self.state = to;
        self.revision += 1;
        self.correlations.insert(correlation_id);
        self.events.push(event);
        Ok(self.events.last().unwrap())
    }
    pub fn cancel(
        &mut self,
        reason: impl Into<String>,
        correlation: impl Into<String>,
        expected_revision: u64,
    ) -> Result<&TrialEvent, TrialError> {
        self.apply(
            TrialState::Cancelled,
            reason,
            correlation,
            expected_revision,
        )
    }
    pub fn events(&self) -> &[TrialEvent] {
        &self.events
    }
    pub fn snapshot(&self) -> TrialSnapshot {
        TrialSnapshot {
            id: self.id.clone(),
            state: self.state.clone(),
            revision: self.revision,
            event_count: self.events.len(),
        }
    }
    pub fn replay(id: impl Into<String>, events: &[TrialEvent]) -> Result<Self, TrialError> {
        let mut a = Self::new(id)?;
        for event in events {
            if event.revision != a.revision + 1 || event.from != a.state {
                return Err(TrialError::Replay(event.revision));
            }
            a.apply(
                event.to.clone(),
                event.reason.clone(),
                event.correlation_id.clone(),
                a.revision,
            )?;
        }
        Ok(a)
    }
    pub fn denial_reason(&self) -> Option<&str> {
        if self.state == TrialState::Denied {
            self.events.last().map(|e| e.reason.as_str())
        } else {
            None
        }
    }
}
fn allowed(from: &TrialState, to: &TrialState) -> bool {
    matches!(
        (from, to),
        (TrialState::Queued, TrialState::Running)
            | (TrialState::Queued, TrialState::Cancelled)
            | (TrialState::Running, TrialState::Succeeded)
            | (TrialState::Running, TrialState::Denied)
            | (TrialState::Running, TrialState::Cancelled)
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legal_lifecycle_succeeds() {
        let mut a = TrialAggregate::new("trial-1").unwrap();
        a.apply(TrialState::Running, "started", "corr-0001", 0)
            .unwrap();
        a.apply(TrialState::Succeeded, "verified", "corr-0002", 1)
            .unwrap();
        assert!(a.state.is_terminal());
        assert_eq!(a.revision, 2);
    }
    #[test]
    fn terminal_state_cannot_restart() {
        assert!(matches!(
            TrialState::Succeeded.transition(TrialState::Running),
            Err(TrialError::Invalid(_, _))
        ));
    }
    #[test]
    fn optimistic_revision_prevents_lost_update() {
        let mut a = TrialAggregate::new("t").unwrap();
        assert_eq!(
            a.apply(TrialState::Running, "start", "corr-0001", 2),
            Err(TrialError::Revision {
                expected: 2,
                actual: 0
            })
        );
        assert_eq!(a.state, TrialState::Queued);
    }
    #[test]
    fn duplicate_correlation_is_rejected() {
        let mut a = TrialAggregate::new("t").unwrap();
        a.apply(TrialState::Running, "start", "corr-0001", 0)
            .unwrap();
        assert!(matches!(
            a.apply(TrialState::Denied, "deny", "corr-0001", 1),
            Err(TrialError::Duplicate(_))
        ));
    }
    #[test]
    fn replay_reconstructs_snapshot() {
        let mut a = TrialAggregate::new("t").unwrap();
        a.apply(TrialState::Running, "start", "corr-0001", 0)
            .unwrap();
        a.apply(TrialState::Denied, "policy", "corr-0002", 1)
            .unwrap();
        let rebuilt = TrialAggregate::replay("t", a.events()).unwrap();
        assert_eq!(rebuilt.snapshot(), a.snapshot());
        assert_eq!(rebuilt.denial_reason(), Some("policy"));
    }
    #[test]
    fn replay_rejects_broken_revision() {
        let event = TrialEvent {
            revision: 2,
            from: TrialState::Queued,
            to: TrialState::Running,
            reason: "x".into(),
            correlation_id: "corr-0001".into(),
        };
        assert_eq!(
            TrialAggregate::replay("t", &[event]),
            Err(TrialError::Replay(2))
        );
    }
    #[test]
    fn cancel_requires_nonterminal_state() {
        let mut a = TrialAggregate::new("t").unwrap();
        a.cancel("operator stop", "corr-0001", 0).unwrap();
        assert_eq!(a.state, TrialState::Cancelled);
        assert!(!a.state.can_cancel());
    }
    #[test]
    fn reason_is_required() {
        let mut a = TrialAggregate::new("t").unwrap();
        assert_eq!(
            a.apply(TrialState::Running, "  ", "corr-0001", 0),
            Err(TrialError::MissingReason)
        );
    }
}
