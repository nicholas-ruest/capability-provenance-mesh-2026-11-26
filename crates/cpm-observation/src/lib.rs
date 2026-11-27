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
use std::collections::{BTreeMap, BTreeSet};
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationBatch {
    pub source: String,
    events: Vec<Observation>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamSummary {
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub host_calls: usize,
    pub resources: usize,
    pub traps: usize,
    pub capabilities: Vec<String>,
}
#[derive(Debug, Default, Clone)]
pub struct SequenceGuard {
    sources: BTreeMap<String, u64>,
    seen: BTreeSet<(String, u64)>,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ObservationError {
    #[error("missing source binding")]
    Unbound,
    #[error("missing capability")]
    MissingCapability,
    #[error("invalid sequence")]
    Sequence,
    #[error("event replay at sequence {0}")]
    Replay(u64),
    #[error("sequence regression: last {last}, received {received}")]
    Regression { last: u64, received: u64 },
    #[error("sequence gap: expected {expected}, received {received}")]
    Gap { expected: u64, received: u64 },
    #[error("mixed sources in observation batch")]
    MixedSource,
    #[error("batch is empty")]
    EmptyBatch,
    #[error("malformed capability {0}")]
    MalformedCapability(String),
}
impl TryFrom<RawEvent> for Observation {
    type Error = ObservationError;
    fn try_from(v: RawEvent) -> Result<Self, Self::Error> {
        validate_source(&v.source)?;
        validate_capability(&v.capability)?;
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
impl Observation {
    pub fn is_failure(&self) -> bool {
        self.kind == EventKind::Trap
    }
    pub fn key(&self) -> (String, u64) {
        (self.source.clone(), self.sequence)
    }
    pub fn same_effect(&self, other: &Self) -> bool {
        self.source == other.source
            && self.kind == other.kind
            && self.capability == other.capability
    }
    pub fn effect_key(&self) -> String {
        format!("{:?}:{}", self.kind, self.capability)
    }
}
impl SequenceGuard {
    pub fn accept(
        &mut self,
        event: &Observation,
        require_contiguous: bool,
    ) -> Result<(), ObservationError> {
        let key = event.key();
        if self.seen.contains(&key) {
            return Err(ObservationError::Replay(event.sequence));
        }
        if let Some(last) = self.sources.get(&event.source).copied() {
            if event.sequence <= last {
                return Err(ObservationError::Regression {
                    last,
                    received: event.sequence,
                });
            }
            if require_contiguous && event.sequence != last + 1 {
                return Err(ObservationError::Gap {
                    expected: last + 1,
                    received: event.sequence,
                });
            }
        }
        self.seen.insert(key);
        self.sources.insert(event.source.clone(), event.sequence);
        Ok(())
    }
    pub fn watermark(&self, source: &str) -> Option<u64> {
        self.sources.get(source).copied()
    }
    pub fn reset_source(&mut self, source: &str) {
        self.sources.remove(source);
        self.seen.retain(|(s, _)| s != source)
    }
}
impl ObservationBatch {
    pub fn new(events: Vec<Observation>) -> Result<Self, ObservationError> {
        let source = events
            .first()
            .ok_or(ObservationError::EmptyBatch)?
            .source
            .clone();
        if events.iter().any(|e| e.source != source) {
            return Err(ObservationError::MixedSource);
        }
        let mut guard = SequenceGuard::default();
        for e in &events {
            guard.accept(e, true)?;
        }
        Ok(Self { source, events })
    }
    pub fn events(&self) -> &[Observation] {
        &self.events
    }
    pub fn summary(&self) -> StreamSummary {
        let mut capabilities = BTreeSet::new();
        let (mut host_calls, mut resources, mut traps) = (0, 0, 0);
        for e in &self.events {
            capabilities.insert(e.capability.clone());
            match e.kind {
                EventKind::HostCall => host_calls += 1,
                EventKind::Resource => resources += 1,
                EventKind::Trap => traps += 1,
            }
        }
        StreamSummary {
            first_sequence: self.events[0].sequence,
            last_sequence: self.events[self.events.len() - 1].sequence,
            host_calls,
            resources,
            traps,
            capabilities: capabilities.into_iter().collect(),
        }
    }
    pub fn failures(&self) -> Vec<&Observation> {
        self.events.iter().filter(|e| e.is_failure()).collect()
    }
    pub fn split_at(&self, sequence: u64) -> (Vec<Observation>, Vec<Observation>) {
        self.events
            .iter()
            .cloned()
            .partition(|e| e.sequence <= sequence)
    }
}
fn validate_source(v: &str) -> Result<(), ObservationError> {
    if v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ObservationError::Unbound)
    }
}
fn validate_capability(v: &str) -> Result<(), ObservationError> {
    if v.is_empty() {
        return Err(ObservationError::MissingCapability);
    }
    if v == "none" {
        return Ok(());
    }
    let (base, resource) = v.split_once(':').map_or((v, None), |(b, r)| (b, Some(r)));
    let Some((ns, op)) = base.split_once('.') else {
        return Err(ObservationError::MalformedCapability(v.into()));
    };
    let token = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
    };
    if !token(ns) || !token(op) || resource.is_some_and(|r| r.is_empty() || r.len() > 160) {
        Err(ObservationError::MalformedCapability(v.into()))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn raw(sequence: u64, kind: EventKind) -> RawEvent {
        RawEvent {
            source: "a".repeat(64),
            kind,
            capability: "net.open:api".into(),
            sequence,
        }
    }
    #[test]
    fn normalizes_bound_event() {
        let e = Observation::try_from(raw(1, EventKind::HostCall)).unwrap();
        assert_eq!(e.capability, "net.open:api");
    }
    #[test]
    fn rejects_nonimmutable_source() {
        let mut e = raw(1, EventKind::HostCall);
        e.source = "branch".into();
        assert_eq!(Observation::try_from(e), Err(ObservationError::Unbound));
    }
    #[test]
    fn rejects_missing_capability() {
        let mut e = raw(1, EventKind::HostCall);
        e.capability.clear();
        assert_eq!(
            Observation::try_from(e),
            Err(ObservationError::MissingCapability)
        );
    }
    #[test]
    fn guard_rejects_replay_without_mutation() {
        let e = Observation::try_from(raw(1, EventKind::HostCall)).unwrap();
        let mut g = SequenceGuard::default();
        g.accept(&e, true).unwrap();
        assert_eq!(g.accept(&e, true), Err(ObservationError::Replay(1)));
        assert_eq!(g.watermark(&e.source), Some(1));
    }
    #[test]
    fn guard_rejects_gap() {
        let mut g = SequenceGuard::default();
        g.accept(
            &Observation::try_from(raw(1, EventKind::HostCall)).unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(
            g.accept(
                &Observation::try_from(raw(3, EventKind::Resource)).unwrap(),
                true
            ),
            Err(ObservationError::Gap {
                expected: 2,
                received: 3
            })
        );
    }
    #[test]
    fn batch_summarizes_kinds() {
        let b = ObservationBatch::new(vec![
            Observation::try_from(raw(1, EventKind::HostCall)).unwrap(),
            Observation::try_from(raw(2, EventKind::Trap)).unwrap(),
        ])
        .unwrap();
        let s = b.summary();
        assert_eq!((s.host_calls, s.traps), (1, 1));
        assert_eq!(b.failures().len(), 1);
    }
    #[test]
    fn batch_rejects_mixed_source() {
        let a = Observation::try_from(raw(1, EventKind::HostCall)).unwrap();
        let mut b = Observation::try_from(raw(2, EventKind::Resource)).unwrap();
        b.source = "b".repeat(64);
        assert_eq!(
            ObservationBatch::new(vec![a, b]),
            Err(ObservationError::MixedSource)
        );
    }
    #[test]
    fn split_preserves_order() {
        let b = ObservationBatch::new(
            (1..=4)
                .map(|n| Observation::try_from(raw(n, EventKind::Resource)).unwrap())
                .collect(),
        )
        .unwrap();
        let (l, r) = b.split_at(2);
        assert_eq!((l.len(), r.len()), (2, 2));
    }
}
