//! Source-to-artifact lineage verification.
use sha2::{Digest, Sha256};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    pub source: String,
    pub lock: String,
    pub plan: String,
    pub artifact: String,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LineageError {
    #[error("malformed digest")]
    Malformed,
    #[error("artifact digest mismatch")]
    Mismatch,
}
impl Lineage {
    pub fn verify(&self, a: &[u8]) -> Result<(), LineageError> {
        let c = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
        };
        if ![&self.source, &self.lock, &self.plan, &self.artifact]
            .into_iter()
            .all(|s| c(s))
        {
            return Err(LineageError::Malformed);
        }
        if format!("{:x}", Sha256::digest(a)) != self.artifact {
            return Err(LineageError::Mismatch);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn l(b: &[u8]) -> Lineage {
        let d = format!("{:x}", Sha256::digest(b));
        Lineage {
            source: d.clone(),
            lock: d.clone(),
            plan: d.clone(),
            artifact: d,
        }
    }
    #[test]
    fn bound() {
        assert_eq!(l(b"x").verify(b"x"), Ok(()))
    }
    #[test]
    fn tamper() {
        assert_eq!(l(b"x").verify(b"y"), Err(LineageError::Mismatch))
    }
}
