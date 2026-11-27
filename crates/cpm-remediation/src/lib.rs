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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityPatch {
    pub remove: Vec<String>,
    pub add: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyRule {
    pub capability: String,
    pub requires: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictRule {
    pub left: String,
    pub right: String,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemediationPlan {
    pub patch: CapabilityPatch,
    pub ordered_steps: Vec<RemediationStep>,
    pub resulting: BTreeSet<String>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemediationStep {
    Revoke(String),
    Grant(String),
    Verify(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemediationError {
    Malformed(String),
    DependencyCycle(String),
    MissingDependency {
        capability: String,
        requires: String,
    },
    Conflict {
        left: String,
        right: String,
        reason: String,
    },
    ProtectedCapability(String),
    StepLimit(usize),
}
pub fn least_patch(observed: &BTreeSet<String>, allowed: &BTreeSet<String>) -> CapabilityPatch {
    CapabilityPatch {
        remove: observed.difference(allowed).cloned().collect(),
        add: allowed.difference(observed).cloned().collect(),
    }
}
impl CapabilityPatch {
    pub fn is_noop(&self) -> bool {
        self.remove.is_empty() && self.add.is_empty()
    }
    pub fn apply(&self, current: &BTreeSet<String>) -> BTreeSet<String> {
        let mut next = current.clone();
        for v in &self.remove {
            next.remove(v);
        }
        for v in &self.add {
            next.insert(v.clone());
        }
        next
    }
    pub fn inverted(&self) -> Self {
        Self {
            remove: self.add.clone(),
            add: self.remove.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), RemediationError> {
        for value in self.remove.iter().chain(&self.add) {
            validate(value)?
        }
        let remove: BTreeSet<_> = self.remove.iter().collect();
        if let Some(overlap) = self.add.iter().find(|v| remove.contains(v)) {
            return Err(RemediationError::Conflict {
                left: (*overlap).clone(),
                right: (*overlap).clone(),
                reason: "same capability added and removed".into(),
            });
        }
        Ok(())
    }
}
pub fn plan(
    current: &BTreeSet<String>,
    desired: &BTreeSet<String>,
    dependencies: &[DependencyRule],
    conflicts: &[ConflictRule],
    protected: &BTreeSet<String>,
    step_limit: usize,
) -> Result<RemediationPlan, RemediationError> {
    for c in current.iter().chain(desired) {
        validate(c)?
    }
    let mut expanded = desired.clone();
    let mut queue: VecDeque<_> = desired.iter().cloned().collect();
    let mut ancestry: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    while let Some(cap) = queue.pop_front() {
        for r in dependencies.iter().filter(|r| r.capability == cap) {
            validate(&r.requires)?;
            let chain = ancestry.entry(cap.clone()).or_default().clone();
            if r.requires == cap || chain.contains(&r.requires) {
                return Err(RemediationError::DependencyCycle(r.requires.clone()));
            }
            let entry = ancestry.entry(r.requires.clone()).or_default();
            entry.extend(chain);
            entry.insert(cap.clone());
            if expanded.insert(r.requires.clone()) {
                queue.push_back(r.requires.clone());
            }
        }
    }
    for rule in conflicts {
        if expanded.contains(&rule.left) && expanded.contains(&rule.right) {
            return Err(RemediationError::Conflict {
                left: rule.left.clone(),
                right: rule.right.clone(),
                reason: rule.reason.clone(),
            });
        }
    }
    let patch = least_patch(current, &expanded);
    if let Some(v) = patch.remove.iter().find(|v| protected.contains(*v)) {
        return Err(RemediationError::ProtectedCapability(v.clone()));
    }
    let mut steps = Vec::new();
    for v in &patch.remove {
        steps.push(RemediationStep::Revoke(v.clone()))
    }
    for v in &patch.add {
        steps.push(RemediationStep::Grant(v.clone()))
    }
    for v in &expanded {
        steps.push(RemediationStep::Verify(v.clone()))
    }
    if steps.len() > step_limit {
        return Err(RemediationError::StepLimit(steps.len()));
    }
    let warnings = expanded
        .iter()
        .filter(|v| v.starts_with("process.") || v.starts_with("identity."))
        .map(|v| format!("sensitive grant requires operator review: {v}"))
        .collect();
    Ok(RemediationPlan {
        patch,
        ordered_steps: steps,
        resulting: expanded,
        warnings,
    })
}
fn validate(v: &str) -> Result<(), RemediationError> {
    let Some((a, b)) = v.split_once('.') else {
        return Err(RemediationError::Malformed(v.into()));
    };
    if a.is_empty()
        || b.is_empty()
        || !v.bytes().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'.' | b'-' | b'_' | b':')
        })
    {
        Err(RemediationError::Malformed(v.into()))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|x| (*x).into()).collect()
    }
    #[test]
    fn least_patch_is_minimal() {
        let p = least_patch(
            &set(&["fs.read", "net.open"]),
            &set(&["fs.read", "clock.read"]),
        );
        assert_eq!(p.remove, vec!["net.open"]);
        assert_eq!(p.add, vec!["clock.read"]);
    }
    #[test]
    fn patch_round_trip_restores_state() {
        let a = set(&["fs.read"]);
        let p = CapabilityPatch {
            remove: vec!["fs.read".into()],
            add: vec!["net.open".into()],
        };
        let b = p.apply(&a);
        assert_eq!(p.inverted().apply(&b), a);
    }
    #[test]
    fn dependency_is_added_to_plan() {
        let p = plan(
            &set(&[]),
            &set(&["net.open"]),
            &[DependencyRule {
                capability: "net.open".into(),
                requires: "dns.resolve".into(),
            }],
            &[],
            &set(&[]),
            20,
        )
        .unwrap();
        assert!(p.resulting.contains("dns.resolve"));
    }
    #[test]
    fn cycle_is_rejected() {
        let rules = [
            DependencyRule {
                capability: "net.open".into(),
                requires: "dns.resolve".into(),
            },
            DependencyRule {
                capability: "dns.resolve".into(),
                requires: "net.open".into(),
            },
        ];
        assert!(matches!(
            plan(&set(&[]), &set(&["net.open"]), &rules, &[], &set(&[]), 20),
            Err(RemediationError::DependencyCycle(_))
        ));
    }
    #[test]
    fn conflicts_fail_closed() {
        let c = ConflictRule {
            left: "net.open".into(),
            right: "sandbox.no-network".into(),
            reason: "network isolation".into(),
        };
        assert!(matches!(
            plan(
                &set(&[]),
                &set(&["net.open", "sandbox.no-network"]),
                &[],
                &[c],
                &set(&[]),
                20
            ),
            Err(RemediationError::Conflict { .. })
        ));
    }
    #[test]
    fn protected_capability_cannot_be_removed() {
        assert_eq!(
            plan(
                &set(&["audit.write"]),
                &set(&[]),
                &[],
                &[],
                &set(&["audit.write"]),
                20
            ),
            Err(RemediationError::ProtectedCapability("audit.write".into()))
        );
    }
    #[test]
    fn step_limit_prevents_unbounded_plan() {
        assert!(matches!(
            plan(
                &set(&[]),
                &set(&["fs.read", "net.open"]),
                &[],
                &[],
                &set(&[]),
                1
            ),
            Err(RemediationError::StepLimit(_))
        ));
    }
    #[test]
    fn malformed_capability_is_rejected() {
        assert!(matches!(
            plan(&set(&[]), &set(&["network"]), &[], &[], &set(&[]), 10),
            Err(RemediationError::Malformed(_))
        ));
    }
}
