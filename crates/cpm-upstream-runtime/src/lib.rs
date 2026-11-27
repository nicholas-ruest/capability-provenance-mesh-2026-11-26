//! Exact-revision process boundary for selected upstreams.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
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
    pub input_sha256: String,
}
#[derive(Debug, Error)]
pub enum UpstreamError {
    #[error("command is not configured")]
    Unconfigured,
    #[error("launch failed: {0}")]
    Launch(#[source] std::io::Error),
    #[error("probe exited {0:?}")]
    Exit(Option<i32>),
    #[error("candidate input staging failed: {0}")]
    Input(#[source] std::io::Error),
}
#[derive(Debug, Clone)]
pub struct UpstreamCommand {
    repository: &'static str,
    revision: &'static str,
    success: Option<PathBuf>,
    failure: Option<PathBuf>,
    argument: Option<PathBuf>,
}
impl UpstreamCommand {
    #[must_use]
    pub fn new(
        repository: &'static str,
        revision: &'static str,
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            repository,
            revision,
            success,
            failure,
            argument,
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        input_payload: &str,
    ) -> Result<UpstreamReceipt, UpstreamError> {
        let path = match kind {
            ProbeKind::Success => &self.success,
            ProbeKind::Failure => &self.failure,
        }
        .as_ref()
        .ok_or(UpstreamError::Unconfigured)?;
        let mut command = Command::new(path);
        if let Some(argument) = &self.argument {
            command.arg(argument);
        }
        let input_sha256 = format!("{:x}", Sha256::digest(input_payload.as_bytes()));
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let input_path = std::env::temp_dir().join(format!(
            "cpm-upstream-input-{}-{nonce}.json",
            std::process::id()
        ));
        let mut input_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&input_path)
            .map_err(UpstreamError::Input)?;
        input_file
            .write_all(input_payload.as_bytes())
            .map_err(UpstreamError::Input)?;
        input_file.sync_all().map_err(UpstreamError::Input)?;
        command.env("CPM_UPSTREAM_INPUT_FILE", &input_path);
        command.env("CPM_UPSTREAM_INPUT_SHA256", &input_sha256);
        let output_result = command.output();
        let _ = fs::remove_file(&input_path);
        let output = output_result.map_err(UpstreamError::Launch)?;
        if !output.status.success() {
            return Err(UpstreamError::Exit(output.status.code()));
        }
        let mut bytes = output.stdout;
        bytes.extend(output.stderr);
        Ok(UpstreamReceipt {
            repository: self.repository,
            revision: self.revision,
            output_sha256: format!("{:x}", Sha256::digest(bytes)),
            input_sha256,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_unconfigured() {
        assert!(matches!(
            UpstreamCommand::new("o/r", "v", None, None, None)
                .execute(ProbeKind::Success, "candidate"),
            Err(UpstreamError::Unconfigured)
        ))
    }
    #[cfg(unix)]
    #[test]
    fn executes() {
        assert_eq!(
            UpstreamCommand::new("o/r", "v", Some("/bin/true".into()), None, None)
                .execute(ProbeKind::Success, "candidate")
                .unwrap()
                .revision,
            "v"
        )
    }
}
