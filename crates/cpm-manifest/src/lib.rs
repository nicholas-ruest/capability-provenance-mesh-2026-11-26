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

pub const VERSION: &str = "cpm.capability/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub api_version: String,
    pub name: String,
    pub entrypoint: String,
    pub capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDeclaration {
    pub namespace: String,
    pub operation: String,
    pub qualifier: Option<String>,
}

impl CapabilityDeclaration {
    pub fn parse(raw: &str) -> Result<Self, ManifestError> {
        let (base, qualifier) = raw
            .split_once(':')
            .map_or((raw, None), |(b, q)| (b, Some(q)));
        let (namespace, operation) = base
            .split_once('.')
            .ok_or_else(|| ManifestError::Capability(raw.into()))?;
        if !valid_token(namespace)
            || !valid_token(operation)
            || qualifier.is_some_and(|q| q.is_empty() || q.len() > 96)
        {
            return Err(ManifestError::Capability(raw.into()));
        }
        Ok(Self {
            namespace: namespace.into(),
            operation: operation.into(),
            qualifier: qualifier.map(str::to_owned),
        })
    }

    pub fn canonical(&self) -> String {
        match &self.qualifier {
            Some(q) => format!("{}.{}:{q}", self.namespace, self.operation),
            None => format!("{}.{}", self.namespace, self.operation),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestReport {
    pub canonical_capabilities: Vec<String>,
    pub warnings: Vec<String>,
    pub fingerprint_material: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDelta {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub entrypoint_changed: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("unsupported manifest API version {0}")]
    Version(String),
    #[error("missing or invalid field {0}")]
    Missing(&'static str),
    #[error("invalid capability declaration {0}")]
    Capability(String),
    #[error("duplicate capability declaration {0}")]
    DuplicateCapability(String),
    #[error("too many capabilities: {0}")]
    CapabilityLimit(usize),
    #[error("entrypoint escapes the package root")]
    EntrypointTraversal,
    #[error("manifest contains mutually exclusive capabilities {0} and {1}")]
    Conflict(String, String),
}

impl Manifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        self.analyze().map(|_| ())
    }

    pub fn analyze(&self) -> Result<ManifestReport, ManifestError> {
        if self.api_version != VERSION {
            return Err(ManifestError::Version(self.api_version.clone()));
        }
        if self.name.is_empty() || self.name.len() > 96 || !valid_name(&self.name) {
            return Err(ManifestError::Missing("name"));
        }
        validate_entrypoint(&self.entrypoint)?;
        if self.capabilities.len() > 64 {
            return Err(ManifestError::CapabilityLimit(self.capabilities.len()));
        }
        let mut seen = BTreeSet::new();
        let mut canonical = Vec::with_capacity(self.capabilities.len());
        let mut by_namespace: BTreeMap<String, usize> = BTreeMap::new();
        for raw in &self.capabilities {
            let value = CapabilityDeclaration::parse(raw)?.canonical();
            if !seen.insert(value.clone()) {
                return Err(ManifestError::DuplicateCapability(value));
            }
            *by_namespace
                .entry(value.split('.').next().unwrap_or_default().into())
                .or_default() += 1;
            canonical.push(value);
        }
        canonical.sort();
        reject_conflict(&seen, "process.exec", "runtime.pure")?;
        reject_conflict(&seen, "net.listen", "sandbox.no-network")?;
        let mut warnings = Vec::new();
        if canonical.is_empty() {
            warnings.push("plugin declares no observable effects".into());
        }
        if by_namespace.get("net").copied().unwrap_or_default() > 4 {
            warnings.push("broad network surface".into());
        }
        let material = format!(
            "{}\n{}\n{}\n{}",
            self.api_version,
            self.name,
            self.entrypoint,
            canonical.join("\n")
        );
        Ok(ManifestReport {
            canonical_capabilities: canonical,
            warnings,
            fingerprint_material: material,
        })
    }

    pub fn diff(&self, next: &Self) -> Result<ManifestDelta, ManifestError> {
        let before: BTreeSet<_> = self.analyze()?.canonical_capabilities.into_iter().collect();
        let after: BTreeSet<_> = next.analyze()?.canonical_capabilities.into_iter().collect();
        Ok(ManifestDelta {
            added: after.difference(&before).cloned().collect(),
            removed: before.difference(&after).cloned().collect(),
            entrypoint_changed: self.entrypoint != next.entrypoint,
        })
    }

    pub fn requires_human_review(&self, previous: Option<&Self>) -> Result<bool, ManifestError> {
        let high_risk = [
            "process.exec",
            "net.listen",
            "fs.write",
            "identity.delegate",
        ];
        let report = self.analyze()?;
        if report
            .canonical_capabilities
            .iter()
            .any(|v| high_risk.iter().any(|p| v.starts_with(p)))
        {
            return Ok(true);
        }
        match previous {
            Some(p) => Ok(!p.diff(self)?.added.is_empty()),
            None => Ok(!report.canonical_capabilities.is_empty()),
        }
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn valid_name(value: &str) -> bool {
    value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && !value.starts_with('.')
}

fn validate_entrypoint(value: &str) -> Result<(), ManifestError> {
    if value.is_empty() || value.len() > 192 || value.starts_with('/') {
        return Err(ManifestError::Missing("entrypoint"));
    }
    if value
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(ManifestError::EntrypointTraversal);
    }
    if value.contains('\\') || value.chars().any(char::is_control) {
        return Err(ManifestError::Missing("entrypoint"));
    }
    Ok(())
}

fn reject_conflict(
    values: &BTreeSet<String>,
    left: &str,
    right: &str,
) -> Result<(), ManifestError> {
    let has = |prefix: &str| {
        values
            .iter()
            .any(|v| v == prefix || v.starts_with(&format!("{prefix}:")))
    };
    if has(left) && has(right) {
        Err(ManifestError::Conflict(left.into(), right.into()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(capabilities: &[&str]) -> Manifest {
        Manifest {
            api_version: VERSION.into(),
            name: "plugin-a".into(),
            entrypoint: "bin/plugin.wasm".into(),
            capabilities: capabilities.iter().map(|v| (*v).into()).collect(),
        }
    }

    #[test]
    fn validates_and_canonicalizes() {
        let report = manifest(&["net.open:api", "fs.read:config"])
            .analyze()
            .unwrap();
        assert_eq!(
            report.canonical_capabilities,
            vec!["fs.read:config", "net.open:api"]
        );
    }
    #[test]
    fn rejects_old_version() {
        let mut m = manifest(&[]);
        m.api_version = "v0".into();
        assert!(matches!(m.validate(), Err(ManifestError::Version(_))));
    }
    #[test]
    fn set_input_deduplicates_exact_capabilities() {
        let m = manifest(&["fs.read:x", "fs.read:x"]);
        assert_eq!(m.capabilities.len(), 1);
        assert!(m.validate().is_ok());
    }
    #[test]
    fn rejects_traversal_and_absolute_paths() {
        let mut m = manifest(&[]);
        m.entrypoint = "../secret".into();
        assert_eq!(m.validate(), Err(ManifestError::EntrypointTraversal));
        m.entrypoint = "/bin/p".into();
        assert!(m.validate().is_err());
    }
    #[test]
    fn detects_conflicting_claims() {
        assert!(matches!(
            manifest(&["process.exec", "runtime.pure"]).validate(),
            Err(ManifestError::Conflict(_, _))
        ));
    }
    #[test]
    fn computes_security_delta() {
        let a = manifest(&["fs.read:config"]);
        let b = manifest(&["fs.read:config", "net.open:api"]);
        let d = a.diff(&b).unwrap();
        assert_eq!(d.added, vec!["net.open:api"]);
        assert!(!d.entrypoint_changed);
    }
    #[test]
    fn human_review_tracks_privilege_growth() {
        let a = manifest(&[]);
        let b = manifest(&["net.open:api"]);
        assert!(!a.requires_human_review(None).unwrap());
        assert!(b.requires_human_review(Some(&a)).unwrap());
    }
    #[test]
    fn capability_parser_rejects_malformed_tokens() {
        for value in ["net", ".open", "Net.open", "net.open:"] {
            assert!(CapabilityDeclaration::parse(value).is_err(), "{value}");
        }
    }
}
