//! Exact-revision process boundary for selected upstreams.
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Command};
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    Success,
    Failure,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamReceipt {
    pub repository: &'static str,
    pub revision: &'static str,
    pub output_sha256: String,
}
#[derive(Debug, Error)]
pub enum UpstreamError {
    #[error("command is not configured")]
    Unconfigured,
    #[error("launch failed: {0}")]
    Launch(#[source] std::io::Error),
    #[error("probe exited {0:?}")]
    Exit(Option<i32>),
}
#[derive(Debug, Clone)]
pub struct UpstreamCommand {
    repository: &'static str,
    revision: &'static str,
    success: Option<PathBuf>,
    failure: Option<PathBuf>,
}
impl UpstreamCommand {
    #[must_use]
    pub fn new(r: &'static str, v: &'static str, s: Option<PathBuf>, f: Option<PathBuf>) -> Self {
        Self {
            repository: r,
            revision: v,
            success: s,
            failure: f,
        }
    }
    pub fn execute(&self, k: ProbeKind) -> Result<UpstreamReceipt, UpstreamError> {
        let p = match k {
            ProbeKind::Success => &self.success,
            ProbeKind::Failure => &self.failure,
        }
        .as_ref()
        .ok_or(UpstreamError::Unconfigured)?;
        let o = Command::new(p).output().map_err(UpstreamError::Launch)?;
        if !o.status.success() {
            return Err(UpstreamError::Exit(o.status.code()));
        }
        let mut b = o.stdout;
        b.extend(o.stderr);
        Ok(UpstreamReceipt {
            repository: self.repository,
            revision: self.revision,
            output_sha256: format!("{:x}", Sha256::digest(b)),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_unconfigured() {
        assert!(matches!(
            UpstreamCommand::new("o/r", "v", None, None).execute(ProbeKind::Success),
            Err(UpstreamError::Unconfigured)
        ))
    }
    #[cfg(unix)]
    #[test]
    fn executes() {
        assert_eq!(
            UpstreamCommand::new("o/r", "v", Some("/bin/true".into()), None)
                .execute(ProbeKind::Success)
                .unwrap()
                .revision,
            "v"
        )
    }
}
