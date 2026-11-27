//! Canonical correlation receipt envelopes.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub correlation_id: String,
    pub artifact: String,
    pub upstreams: Vec<String>,
    pub checksum: String,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReceiptError {
    #[error("receipt checksum mismatch")]
    Mismatch,
}
impl Receipt {
    #[must_use]
    pub fn issue(correlation_id: String, artifact: String, upstreams: Vec<String>) -> Self {
        let checksum = hash(&correlation_id, &artifact, &upstreams);
        Self {
            correlation_id,
            artifact,
            upstreams,
            checksum,
        }
    }
    pub fn verify(&self) -> Result<(), ReceiptError> {
        if hash(&self.correlation_id, &self.artifact, &self.upstreams) == self.checksum {
            Ok(())
        } else {
            Err(ReceiptError::Mismatch)
        }
    }
}
fn hash(c: &str, a: &str, u: &[String]) -> String {
    let mut h = Sha256::new();
    h.update(c);
    h.update(a);
    for x in u {
        h.update(x)
    }
    format!("{:x}", h.finalize())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifies() {
        assert!(
            Receipt::issue("c".into(), "a".into(), vec!["u".into()])
                .verify()
                .is_ok()
        )
    }
    #[test]
    fn detects_change() {
        let mut r = Receipt::issue("c".into(), "a".into(), vec![]);
        r.artifact = "b".into();
        assert_eq!(r.verify(), Err(ReceiptError::Mismatch))
    }
}
