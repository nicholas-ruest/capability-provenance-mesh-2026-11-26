//! Versioned least-authority manifests.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
pub const VERSION: &str = "cpm.capability/v1";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub api_version: String,
    pub name: String,
    pub entrypoint: String,
    pub capabilities: BTreeSet<String>,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("unsupported api version")]
    Version,
    #[error("missing required field")]
    Missing,
    #[error("invalid capability {0}")]
    Capability(String),
}
impl Manifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.api_version != VERSION {
            return Err(ManifestError::Version);
        }
        if self.name.trim().is_empty() || self.entrypoint.trim().is_empty() {
            return Err(ManifestError::Missing);
        }
        if let Some(b) = self.capabilities.iter().find(|c| {
            c.is_empty()
                || !c.bytes().all(|x| {
                    x.is_ascii_lowercase() || x.is_ascii_digit() || matches!(x, b'.' | b':' | b'-')
                })
        }) {
            return Err(ManifestError::Capability(b.clone()));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn v() -> Manifest {
        Manifest {
            api_version: VERSION.into(),
            name: "resize".into(),
            entrypoint: "run".into(),
            capabilities: BTreeSet::from(["blob.read".into()]),
        }
    }
    #[test]
    fn accepts() {
        assert_eq!(v().validate(), Ok(()))
    }
    #[test]
    fn rejects() {
        let mut m = v();
        m.capabilities.insert("Network Access".into());
        assert!(m.validate().is_err())
    }
}
