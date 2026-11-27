//! Trust-aware transitive effect graph.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub evidence: String,
    pub trusted: bool,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("empty endpoint")]
    Empty,
    #[error("missing evidence")]
    Evidence,
    #[error("untrusted edge")]
    Untrusted,
}
#[derive(Debug, Default)]
pub struct EffectGraph {
    edges: BTreeMap<String, Vec<Edge>>,
}
impl EffectGraph {
    pub fn add(&mut self, e: Edge) -> Result<(), GraphError> {
        if e.from.is_empty() || e.to.is_empty() {
            return Err(GraphError::Empty);
        }
        if e.evidence.len() != 64 {
            return Err(GraphError::Evidence);
        }
        if !e.trusted {
            return Err(GraphError::Untrusted);
        }
        self.edges.entry(e.from.clone()).or_default().push(e);
        Ok(())
    }
    #[must_use]
    pub fn reachable(&self, s: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut q = VecDeque::from([s.to_owned()]);
        while let Some(n) = q.pop_front() {
            for e in self.edges.get(&n).into_iter().flatten() {
                if seen.insert(e.to.clone()) {
                    q.push_back(e.to.clone())
                }
            }
        }
        seen
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closure() {
        let mut g = EffectGraph::default();
        for (a, b) in [("p", "net"), ("net", "sink")] {
            g.add(Edge {
                from: a.into(),
                to: b.into(),
                evidence: "a".repeat(64),
                trusted: true,
            })
            .unwrap()
        }
        assert!(g.reachable("p").contains("sink"))
    }
    #[test]
    fn trust() {
        let mut g = EffectGraph::default();
        assert_eq!(
            g.add(Edge {
                from: "a".into(),
                to: "b".into(),
                evidence: "a".repeat(64),
                trusted: false
            }),
            Err(GraphError::Untrusted)
        )
    }
}
