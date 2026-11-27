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

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub evidence: String,
    pub trusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectGraph {
    edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectPath {
    pub nodes: Vec<String>,
    pub evidence: Vec<String>,
    pub fully_trusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphAudit {
    pub node_count: usize,
    pub edge_count: usize,
    pub untrusted_edges: usize,
    pub roots: Vec<String>,
    pub leaves: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("effect endpoint is empty")]
    Endpoint,
    #[error("effect endpoint is too long")]
    EndpointLength,
    #[error("evidence must be a 64-character lowercase digest")]
    Evidence,
    #[error("self effects are forbidden")]
    SelfEdge,
    #[error("duplicate effect edge")]
    Duplicate,
    #[error("effect edge would create a cycle")]
    Cycle,
    #[error("untrusted effect edge cannot be committed")]
    Untrusted,
    #[error("path not found from {0} to {1}")]
    NoPath(String, String),
}

impl EffectGraph {
    pub fn add(&mut self, edge: Edge) -> Result<(), GraphError> {
        validate_edge(&edge)?;
        if !edge.trusted {
            return Err(GraphError::Untrusted);
        }
        if self
            .edges
            .iter()
            .any(|e| e.from == edge.from && e.to == edge.to && e.evidence == edge.evidence)
        {
            return Err(GraphError::Duplicate);
        }
        if edge.from == edge.to {
            return Err(GraphError::SelfEdge);
        }
        if self.reachable(&edge.to, &edge.from) {
            return Err(GraphError::Cycle);
        }
        self.edges.push(edge);
        self.edges
            .sort_by(|a, b| (&a.from, &a.to, &a.evidence).cmp(&(&b.from, &b.to, &b.evidence)));
        Ok(())
    }

    pub fn reachable(&self, from: &str, to: &str) -> bool {
        self.path(from, to).is_ok()
    }
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
    pub fn successors(&self, node: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == node).collect()
    }
    pub fn predecessors(&self, node: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == node).collect()
    }

    pub fn path(&self, from: &str, to: &str) -> Result<EffectPath, GraphError> {
        if from == to {
            return Ok(EffectPath {
                nodes: vec![from.into()],
                evidence: vec![],
                fully_trusted: true,
            });
        }
        let mut queue = VecDeque::from([from.to_string()]);
        let mut parent: BTreeMap<String, (String, String, bool)> = BTreeMap::new();
        let mut seen = BTreeSet::from([from.to_string()]);
        while let Some(current) = queue.pop_front() {
            for edge in self.edges.iter().filter(|e| e.from == current) {
                if seen.insert(edge.to.clone()) {
                    parent.insert(
                        edge.to.clone(),
                        (current.clone(), edge.evidence.clone(), edge.trusted),
                    );
                    if edge.to == to {
                        return Ok(rebuild(from, to, &parent));
                    }
                    queue.push_back(edge.to.clone());
                }
            }
        }
        Err(GraphError::NoPath(from.into(), to.into()))
    }

    pub fn transitive_effects(&self, root: &str) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut queue = VecDeque::from([root.to_string()]);
        while let Some(node) = queue.pop_front() {
            for edge in self.edges.iter().filter(|e| e.from == node) {
                if result.insert(edge.to.clone()) {
                    queue.push_back(edge.to.clone());
                }
            }
        }
        result
    }

    pub fn evidence_coverage(&self, required: &[String]) -> BTreeMap<String, bool> {
        required
            .iter()
            .map(|digest| {
                (
                    digest.clone(),
                    self.edges.iter().any(|e| &e.evidence == digest),
                )
            })
            .collect()
    }
    pub fn remove_by_evidence(&mut self, digest: &str) -> usize {
        let before = self.edges.len();
        self.edges.retain(|e| e.evidence != digest);
        before - self.edges.len()
    }
    pub fn audit(&self) -> GraphAudit {
        let mut nodes = BTreeSet::new();
        let mut incoming = BTreeMap::<String, usize>::new();
        let mut outgoing = BTreeMap::<String, usize>::new();
        for e in &self.edges {
            nodes.insert(e.from.clone());
            nodes.insert(e.to.clone());
            *incoming.entry(e.to.clone()).or_default() += 1;
            *outgoing.entry(e.from.clone()).or_default() += 1;
        }
        GraphAudit {
            node_count: nodes.len(),
            edge_count: self.edges.len(),
            untrusted_edges: self.edges.iter().filter(|e| !e.trusted).count(),
            roots: nodes
                .iter()
                .filter(|n| !incoming.contains_key(*n))
                .cloned()
                .collect(),
            leaves: nodes
                .iter()
                .filter(|n| !outgoing.contains_key(*n))
                .cloned()
                .collect(),
        }
    }
    pub fn topological_order(&self) -> Vec<String> {
        let mut nodes = BTreeSet::new();
        let mut indegree = BTreeMap::<String, usize>::new();
        for e in &self.edges {
            nodes.insert(e.from.clone());
            nodes.insert(e.to.clone());
            indegree.entry(e.from.clone()).or_default();
            *indegree.entry(e.to.clone()).or_default() += 1;
        }
        let mut q: VecDeque<_> = nodes
            .iter()
            .filter(|n| indegree[*n] == 0)
            .cloned()
            .collect();
        let mut out = Vec::new();
        while let Some(n) = q.pop_front() {
            out.push(n.clone());
            for e in self.edges.iter().filter(|e| e.from == n) {
                let v = indegree.get_mut(&e.to).unwrap();
                *v -= 1;
                if *v == 0 {
                    q.push_back(e.to.clone());
                }
            }
        }
        out
    }
}

fn validate_edge(e: &Edge) -> Result<(), GraphError> {
    if e.from.is_empty() || e.to.is_empty() {
        return Err(GraphError::Endpoint);
    }
    if e.from.len() > 160 || e.to.len() > 160 {
        return Err(GraphError::EndpointLength);
    }
    if e.evidence.len() != 64
        || !e
            .evidence
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(GraphError::Evidence);
    }
    Ok(())
}
fn rebuild(from: &str, to: &str, parent: &BTreeMap<String, (String, String, bool)>) -> EffectPath {
    let mut nodes = vec![to.into()];
    let mut evidence = vec![];
    let mut trusted = true;
    let mut cur = to;
    while cur != from {
        let (p, e, t) = &parent[cur];
        nodes.push(p.clone());
        evidence.push(e.clone());
        trusted &= *t;
        cur = p;
    }
    nodes.reverse();
    evidence.reverse();
    EffectPath {
        nodes,
        evidence,
        fully_trusted: trusted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn edge(a: &str, b: &str) -> Edge {
        Edge {
            from: a.into(),
            to: b.into(),
            evidence: "a".repeat(64),
            trusted: true,
        }
    }
    #[test]
    fn traces_shortest_effect_path() {
        let mut g = EffectGraph::default();
        g.add(edge("plugin", "file")).unwrap();
        g.add(edge("file", "network")).unwrap();
        let p = g.path("plugin", "network").unwrap();
        assert_eq!(p.nodes, vec!["plugin", "file", "network"]);
        assert!(p.fully_trusted);
    }
    #[test]
    fn rejects_untrusted_and_leaves_graph_clean() {
        let mut g = EffectGraph::default();
        let mut e = edge("a", "b");
        e.trusted = false;
        assert_eq!(g.add(e), Err(GraphError::Untrusted));
        assert_eq!(g.edges(), &[]);
    }
    #[test]
    fn rejects_cycles() {
        let mut g = EffectGraph::default();
        g.add(edge("a", "b")).unwrap();
        g.add(edge("b", "c")).unwrap();
        assert_eq!(g.add(edge("c", "a")), Err(GraphError::Cycle));
    }
    #[test]
    fn audit_finds_roots_and_leaves() {
        let mut g = EffectGraph::default();
        g.add(edge("a", "b")).unwrap();
        g.add(edge("a", "c")).unwrap();
        let a = g.audit();
        assert_eq!(a.roots, vec!["a"]);
        assert_eq!(a.leaves, vec!["b", "c"]);
    }
    #[test]
    fn transitive_effects_are_unique() {
        let mut g = EffectGraph::default();
        g.add(edge("a", "b")).unwrap();
        g.add(edge("a", "c")).unwrap();
        g.add(edge("b", "c")).unwrap();
        assert_eq!(
            g.transitive_effects("a"),
            BTreeSet::from(["b".into(), "c".into()])
        );
    }
    #[test]
    fn removal_uses_evidence_digest() {
        let mut g = EffectGraph::default();
        g.add(edge("a", "b")).unwrap();
        assert_eq!(g.remove_by_evidence(&"a".repeat(64)), 1);
        assert!(!g.reachable("a", "b"));
    }
    #[test]
    fn topological_order_respects_dependencies() {
        let mut g = EffectGraph::default();
        g.add(edge("a", "b")).unwrap();
        g.add(edge("b", "c")).unwrap();
        assert_eq!(g.topological_order(), vec!["a", "b", "c"]);
    }
    #[test]
    fn rejects_bad_evidence() {
        let mut e = edge("a", "b");
        e.evidence = "branch".into();
        assert_eq!(EffectGraph::default().add(e), Err(GraphError::Evidence));
    }
}
