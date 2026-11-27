//! Frozen deterministic scenario packs.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use sha2::{Digest, Sha256};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPack {
    pub name: String,
    pub inputs: Vec<Vec<u8>>,
    pub digest: String,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScenarioError {
    #[error("scenario pack is empty")]
    Empty,
    #[error("scenario digest changed")]
    Changed,
}
impl ScenarioPack {
    pub fn freeze(name: String, inputs: Vec<Vec<u8>>) -> Result<Self, ScenarioError> {
        if inputs.is_empty() {
            return Err(ScenarioError::Empty);
        }
        let mut h = Sha256::new();
        for i in &inputs {
            h.update((i.len() as u64).to_le_bytes());
            h.update(i)
        }
        Ok(Self {
            name,
            inputs,
            digest: format!("{:x}", h.finalize()),
        })
    }
    pub fn verify(&self) -> Result<(), ScenarioError> {
        Self::freeze(self.name.clone(), self.inputs.clone()).and_then(|p| {
            if p.digest == self.digest {
                Ok(())
            } else {
                Err(ScenarioError::Changed)
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable() {
        assert!(
            ScenarioPack::freeze("x".into(), vec![b"a".to_vec()])
                .unwrap()
                .verify()
                .is_ok()
        )
    }
    #[test]
    fn empty() {
        assert_eq!(
            ScenarioPack::freeze("x".into(), vec![]),
            Err(ScenarioError::Empty)
        )
    }
}
