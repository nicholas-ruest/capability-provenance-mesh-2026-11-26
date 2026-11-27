//! Capability set algebra and attenuation.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use std::collections::BTreeSet;
use thiserror::Error;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<String>);
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AttenuationError {
    #[error("capability escalation: {0}")]
    Escalation(String),
}
impl CapabilitySet {
    #[must_use]
    pub fn new(v: impl IntoIterator<Item = String>) -> Self {
        Self(v.into_iter().collect())
    }
    #[must_use]
    pub fn contains(&self, v: &str) -> bool {
        self.0.contains(v)
    }
    #[must_use]
    pub fn excess(&self, o: &Self) -> Self {
        Self(o.0.difference(&self.0).cloned().collect())
    }
    pub fn attenuate(&self, c: &Self) -> Result<Self, AttenuationError> {
        if let Some(x) = c.0.difference(&self.0).next() {
            return Err(AttenuationError::Escalation(x.clone()));
        }
        Ok(c.clone())
    }
    #[must_use]
    pub fn values(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excess() {
        let d = CapabilitySet::new(["blob.read".into()]);
        assert!(
            d.excess(&CapabilitySet::new(["net.open".into()]))
                .contains("net.open")
        )
    }
    #[test]
    fn escalation() {
        assert!(
            CapabilitySet::new(["blob.read".into()])
                .attenuate(&CapabilitySet::new(["net.open".into()]))
                .is_err()
        )
    }
}
