//! Source-bound sandbox observations.
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventKind {
    HostCall,
    Resource,
    Trap,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub source: String,
    pub kind: EventKind,
    pub capability: String,
    pub sequence: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub source: String,
    pub kind: EventKind,
    pub capability: String,
    pub sequence: u64,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ObservationError {
    #[error("missing source binding")]
    Unbound,
    #[error("missing capability")]
    MissingCapability,
    #[error("invalid sequence")]
    Sequence,
}
impl TryFrom<RawEvent> for Observation {
    type Error = ObservationError;
    fn try_from(v: RawEvent) -> Result<Self, Self::Error> {
        if v.source.len() != 64 {
            return Err(ObservationError::Unbound);
        }
        if v.capability.is_empty() {
            return Err(ObservationError::MissingCapability);
        }
        if v.sequence == 0 {
            return Err(ObservationError::Sequence);
        }
        Ok(Self {
            source: v.source,
            kind: v.kind,
            capability: v.capability,
            sequence: v.sequence,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts() {
        assert!(
            Observation::try_from(RawEvent {
                source: "a".repeat(64),
                kind: EventKind::HostCall,
                capability: "blob.read".into(),
                sequence: 1
            })
            .is_ok()
        )
    }
    #[test]
    fn unbound() {
        assert_eq!(
            Observation::try_from(RawEvent {
                source: String::new(),
                kind: EventKind::Trap,
                capability: "x".into(),
                sequence: 1
            }),
            Err(ObservationError::Unbound)
        )
    }
}
