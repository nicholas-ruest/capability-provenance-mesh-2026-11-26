//! Restart-safe shared-directory decision and audit persistence.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionRecord {
    pub correlation_id: String,
    pub plugin: String,
    pub version: String,
    pub admitted: bool,
    pub artifact_digest: String,
    pub receipt_checksum: String,
    pub upstream_receipts: Vec<String>,
}
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("record not found")]
    Missing,
    #[error("record conflict")]
    Conflict,
}
#[derive(Debug, Clone)]
pub struct DirectoryStore {
    root: PathBuf,
}
impl DirectoryStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(root.join("decisions"))?;
        fs::create_dir_all(root.join("audit"))?;
        Ok(Self { root })
    }
    fn key(plugin: &str, version: &str) -> String {
        format!("{}--{}.json", sanitize(plugin), sanitize(version))
    }
    pub fn append(&self, record: &DecisionRecord) -> Result<(), StoreError> {
        let target = self
            .root
            .join("decisions")
            .join(Self::key(&record.plugin, &record.version));
        if target.exists() {
            return Err(StoreError::Conflict);
        }
        let temp = target.with_extension("tmp");
        let mut f = fs::File::create(&temp)?;
        f.write_all(&serde_json::to_vec_pretty(record)?)?;
        f.sync_all()?;
        fs::rename(temp, target)?;
        let audit = self
            .root
            .join("audit")
            .join(format!("{}.json", sanitize(&record.correlation_id)));
        fs::write(audit, serde_json::to_vec_pretty(record)?)?;
        Ok(())
    }
    pub fn get(&self, plugin: &str, version: &str) -> Result<DecisionRecord, StoreError> {
        read(self.root.join("decisions").join(Self::key(plugin, version)))
    }
    pub fn audit(&self, correlation: &str) -> Result<DecisionRecord, StoreError> {
        read(
            self.root
                .join("audit")
                .join(format!("{}.json", sanitize(correlation))),
        )
    }
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
fn sanitize(v: &str) -> String {
    v.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}
fn read(path: PathBuf) -> Result<DecisionRecord, StoreError> {
    if !path.exists() {
        return Err(StoreError::Missing);
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rec() -> DecisionRecord {
        DecisionRecord {
            correlation_id: "c-1".into(),
            plugin: "p".into(),
            version: "1".into(),
            admitted: true,
            artifact_digest: "a".repeat(64),
            receipt_checksum: "b".repeat(64),
            upstream_receipts: vec!["u".into()],
        }
    }
    #[test]
    fn survives_reopen() {
        let d = std::env::temp_dir().join(format!("cpm-store-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        DirectoryStore::open(&d).unwrap().append(&rec()).unwrap();
        assert_eq!(
            DirectoryStore::open(&d).unwrap().get("p", "1").unwrap(),
            rec()
        );
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn conflict_has_no_partial_write() {
        let d = std::env::temp_dir().join(format!("cpm-conflict-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        let s = DirectoryStore::open(&d).unwrap();
        s.append(&rec()).unwrap();
        assert!(matches!(s.append(&rec()), Err(StoreError::Conflict)));
        let _ = fs::remove_dir_all(d);
    }
}
