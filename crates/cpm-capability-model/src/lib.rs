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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<String>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub namespace: String,
    pub operation: String,
    pub resource: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttenuationRule {
    pub parent: String,
    pub child: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDelta {
    pub granted: Vec<String>,
    pub revoked: Vec<String>,
    pub unchanged: Vec<String>,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AttenuationError {
    #[error("capability escalation: {0}")]
    Escalation(String),
    #[error("malformed capability: {0}")]
    Malformed(String),
    #[error("capability set exceeds limit {limit}: {actual}")]
    Limit { limit: usize, actual: usize },
    #[error("attenuation graph cycle through {0}")]
    Cycle(String),
    #[error("capability namespace {0} is mutually exclusive")]
    Conflict(String),
}
impl Capability {
    pub fn parse(raw: &str) -> Result<Self, AttenuationError> {
        let (base, resource) = raw
            .split_once(':')
            .map_or((raw, None), |(b, r)| (b, Some(r)));
        let (namespace, operation) = base
            .split_once('.')
            .ok_or_else(|| AttenuationError::Malformed(raw.into()))?;
        let token = |v: &str| {
            !v.is_empty()
                && v.len() <= 64
                && v.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_')
                })
        };
        if !token(namespace)
            || !token(operation)
            || resource.is_some_and(|r| r.is_empty() || r.len() > 160 || r.contains(".."))
        {
            return Err(AttenuationError::Malformed(raw.into()));
        }
        Ok(Self {
            namespace: namespace.into(),
            operation: operation.into(),
            resource: resource.map(str::to_owned),
        })
    }
    pub fn canonical(&self) -> String {
        self.resource.as_ref().map_or_else(
            || format!("{}.{}", self.namespace, self.operation),
            |r| format!("{}.{}:{r}", self.namespace, self.operation),
        )
    }
    pub fn is_sensitive(&self) -> bool {
        matches!(
            (self.namespace.as_str(), self.operation.as_str()),
            ("process", "exec") | ("identity", "delegate") | ("net", "listen") | ("fs", "write")
        )
    }
}
impl CapabilitySet {
    pub fn new(v: impl IntoIterator<Item = String>) -> Self {
        Self(v.into_iter().collect())
    }
    pub fn checked(
        v: impl IntoIterator<Item = String>,
        limit: usize,
    ) -> Result<Self, AttenuationError> {
        let mut values = BTreeSet::new();
        for raw in v {
            values.insert(Capability::parse(&raw)?.canonical());
        }
        if values.len() > limit {
            return Err(AttenuationError::Limit {
                limit,
                actual: values.len(),
            });
        }
        let set = Self(values);
        set.validate_conflicts()?;
        Ok(set)
    }
    pub fn contains(&self, v: &str) -> bool {
        self.0.contains(v)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn values(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
    pub fn excess(&self, observed: &Self) -> Self {
        Self(observed.0.difference(&self.0).cloned().collect())
    }
    pub fn missing(&self, observed: &Self) -> Self {
        Self(self.0.difference(&observed.0).cloned().collect())
    }
    pub fn attenuate(&self, candidate: &Self) -> Result<Self, AttenuationError> {
        if let Some(x) = candidate.0.difference(&self.0).next() {
            return Err(AttenuationError::Escalation(x.clone()));
        }
        Ok(candidate.clone())
    }
    pub fn intersection(&self, other: &Self) -> Self {
        Self(self.0.intersection(&other.0).cloned().collect())
    }
    pub fn union(&self, other: &Self, limit: usize) -> Result<Self, AttenuationError> {
        Self::checked(self.0.union(&other.0).cloned(), limit)
    }
    pub fn delta(&self, next: &Self) -> CapabilityDelta {
        CapabilityDelta {
            granted: next.0.difference(&self.0).cloned().collect(),
            revoked: self.0.difference(&next.0).cloned().collect(),
            unchanged: self.0.intersection(&next.0).cloned().collect(),
        }
    }
    pub fn grouped(&self) -> Result<BTreeMap<String, Vec<String>>, AttenuationError> {
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for raw in &self.0 {
            let c = Capability::parse(raw)?;
            out.entry(c.namespace).or_default().push(raw.clone());
        }
        Ok(out)
    }
    pub fn sensitive(&self) -> Result<Vec<String>, AttenuationError> {
        let mut out = Vec::new();
        for raw in &self.0 {
            if Capability::parse(raw)?.is_sensitive() {
                out.push(raw.clone())
            }
        }
        Ok(out)
    }
    pub fn closure(
        &self,
        rules: &[AttenuationRule],
        limit: usize,
    ) -> Result<Self, AttenuationError> {
        let mut result = self.0.clone();
        let mut q: VecDeque<_> = result.iter().cloned().collect();
        let mut steps = 0usize;
        while let Some(current) = q.pop_front() {
            for rule in rules.iter().filter(|r| r.parent == current) {
                Capability::parse(&rule.child)?;
                steps += 1;
                if steps > limit.saturating_mul(rules.len().max(1)) {
                    return Err(AttenuationError::Cycle(current));
                }
                if result.insert(rule.child.clone()) {
                    if result.len() > limit {
                        return Err(AttenuationError::Limit {
                            limit,
                            actual: result.len(),
                        });
                    }
                    q.push_back(rule.child.clone());
                }
            }
        }
        for v in &result {
            if reaches(v, v, rules, &mut BTreeSet::new(), true) {
                return Err(AttenuationError::Cycle(v.clone()));
            }
        }
        let set = Self(result);
        set.validate_conflicts()?;
        Ok(set)
    }
    fn validate_conflicts(&self) -> Result<(), AttenuationError> {
        if self.0.contains("sandbox.no-network") && self.0.iter().any(|v| v.starts_with("net.")) {
            return Err(AttenuationError::Conflict("sandbox".into()));
        }
        if self.0.contains("runtime.pure") && self.0.iter().any(|v| v.starts_with("process.")) {
            return Err(AttenuationError::Conflict("runtime".into()));
        }
        Ok(())
    }
}
fn reaches(
    start: &str,
    target: &str,
    rules: &[AttenuationRule],
    seen: &mut BTreeSet<String>,
    skip_root: bool,
) -> bool {
    if !seen.insert(start.into()) {
        return false;
    }
    rules
        .iter()
        .filter(|r| r.parent == start)
        .any(|r| (!skip_root && r.child == target) || reaches(&r.child, target, rules, seen, false))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn set(v: &[&str]) -> CapabilitySet {
        CapabilitySet::checked(v.iter().map(|s| (*s).into()), 32).unwrap()
    }
    #[test]
    fn observed_excess_is_a_set() {
        let d = set(&["fs.read"]);
        assert!(
            d.excess(&set(&["fs.read", "net.open"]))
                .contains("net.open")
        );
    }
    #[test]
    fn attenuation_rejects_escalation() {
        assert!(matches!(
            set(&["fs.read"]).attenuate(&set(&["net.open"])),
            Err(AttenuationError::Escalation(_))
        ));
    }
    #[test]
    fn parser_preserves_resource() {
        assert_eq!(
            Capability::parse("fs.read:config/a").unwrap().canonical(),
            "fs.read:config/a"
        );
    }
    #[test]
    fn resource_traversal_is_rejected() {
        assert!(Capability::parse("fs.read:../secret").is_err());
    }
    #[test]
    fn delta_is_deterministic() {
        let d = set(&["fs.read", "net.open"]).delta(&set(&["fs.read", "clock.read"]));
        assert_eq!(d.granted, vec!["clock.read"]);
        assert_eq!(d.revoked, vec!["net.open"]);
    }
    #[test]
    fn closure_adds_required_attenuation() {
        let c = set(&["fs.read:tree"])
            .closure(
                &[AttenuationRule {
                    parent: "fs.read:tree".into(),
                    child: "fs.read:file".into(),
                }],
                4,
            )
            .unwrap();
        assert!(c.contains("fs.read:file"));
    }
    #[test]
    fn cycle_is_rejected() {
        let r = [
            AttenuationRule {
                parent: "fs.read:a".into(),
                child: "fs.read:b".into(),
            },
            AttenuationRule {
                parent: "fs.read:b".into(),
                child: "fs.read:a".into(),
            },
        ];
        assert!(matches!(
            set(&["fs.read:a"]).closure(&r, 8),
            Err(AttenuationError::Cycle(_))
        ));
    }
    #[test]
    fn union_enforces_limit() {
        assert!(matches!(
            set(&["fs.read"]).union(&set(&["net.open"]), 1),
            Err(AttenuationError::Limit { .. })
        ));
    }
    #[test]
    fn purity_conflict_is_rejected() {
        assert!(matches!(
            CapabilitySet::checked(["runtime.pure".into(), "process.exec".into()], 8),
            Err(AttenuationError::Conflict(_))
        ));
    }
    #[test]
    fn groups_namespaces() {
        assert_eq!(
            set(&["fs.read", "fs.write", "net.open"]).grouped().unwrap()["fs"].len(),
            2
        );
    }
}
