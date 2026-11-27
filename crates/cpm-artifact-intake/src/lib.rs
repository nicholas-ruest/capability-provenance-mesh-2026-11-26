//! Bounded upload hashing and quarantine.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use sha2::{Digest, Sha256};
use thiserror::Error;
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntakeError {
    #[error("artifact is empty")]
    Empty,
    #[error("artifact size {actual} exceeds limit {limit}")]
    Oversize { actual: usize, limit: usize },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinedArtifact {
    bytes: Vec<u8>,
    digest: String,
}
impl QuarantinedArtifact {
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Intake {
    max_bytes: usize,
}
impl Intake {
    #[must_use]
    pub const fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }
    pub fn quarantine(self, bytes: Vec<u8>) -> Result<QuarantinedArtifact, IntakeError> {
        if bytes.is_empty() {
            return Err(IntakeError::Empty);
        }
        if bytes.len() > self.max_bytes {
            return Err(IntakeError::Oversize {
                actual: bytes.len(),
                limit: self.max_bytes,
            });
        }
        let digest = format!("{:x}", Sha256::digest(&bytes));
        Ok(QuarantinedArtifact { bytes, digest })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes() {
        let a = Intake::new(8).quarantine(b"wasm".to_vec()).unwrap();
        assert_eq!(a.digest().len(), 64);
        assert_eq!(a.bytes(), b"wasm")
    }
    #[test]
    fn rejects() {
        assert_eq!(Intake::new(1).quarantine(vec![]), Err(IntakeError::Empty));
        assert!(matches!(
            Intake::new(1).quarantine(vec![1, 2]),
            Err(IntakeError::Oversize { .. })
        ))
    }
}
