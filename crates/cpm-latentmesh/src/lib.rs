//! Production adapter for ruvnet/LatentMesh.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use cpm_upstream_runtime::{ProbeKind, UpstreamCommand, UpstreamError, UpstreamReceipt};
use std::path::PathBuf;
pub const REPOSITORY: &str = "ruvnet/LatentMesh";
pub const REVISION: &str = "b04a70523196181aadc6d694e02de1f5722fae6b";
#[derive(Debug, Clone)]
pub struct Adapter {
    command: UpstreamCommand,
}
impl Adapter {
    #[must_use]
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            command: UpstreamCommand::new(REPOSITORY, REVISION, success, failure, argument),
        }
    }
    pub fn execute(&self, kind: ProbeKind) -> Result<UpstreamReceipt, UpstreamError> {
        self.command.execute(kind)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pins_exact_revision() {
        assert_eq!(REVISION.len(), 40);
        assert!(
            Adapter::new(None, None, None)
                .execute(ProbeKind::Success)
                .is_err()
        )
    }
}
