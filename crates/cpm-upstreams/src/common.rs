use cpm_upstream_runtime::{ProbeKind, UpstreamCommand, UpstreamError, UpstreamReceipt};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct HarnessBinding {
    command: UpstreamCommand,
    repository: &'static str,
    revision: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamEvidence {
    pub repository: &'static str,
    pub revision: &'static str,
    pub responsibility: &'static str,
    pub operation: String,
    pub output_sha256: String,
    pub semantics_sha256: String,
}

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("{field} is invalid: {reason}")]
    Validation {
        field: &'static str,
        reason: &'static str,
    },
    #[error("upstream binding returned unexpected repository or revision")]
    BindingMismatch,
    #[error("upstream receipt output digest is malformed")]
    OutputDigest,
    #[error(transparent)]
    Runtime(#[from] UpstreamError),
}

impl HarnessBinding {
    pub fn new(
        repository: &'static str,
        revision: &'static str,
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            command: UpstreamCommand::new(repository, revision, success, failure, argument),
            repository,
            revision,
        }
    }

    pub fn execute(
        &self,
        kind: ProbeKind,
        responsibility: &'static str,
        operation: String,
        semantic_fields: &[&str],
    ) -> Result<UpstreamEvidence, IntegrationError> {
        let receipt = self.command.execute(kind)?;
        self.validate_receipt(&receipt)?;
        let mut hasher = Sha256::new();
        hash_field(&mut hasher, responsibility);
        hash_field(&mut hasher, &operation);
        hash_field(
            &mut hasher,
            match kind {
                ProbeKind::Success => "success",
                ProbeKind::Failure => "failure",
            },
        );
        for field in semantic_fields {
            hash_field(&mut hasher, field);
        }
        hash_field(&mut hasher, &receipt.output_sha256);
        Ok(UpstreamEvidence {
            repository: self.repository,
            revision: self.revision,
            responsibility,
            operation,
            output_sha256: receipt.output_sha256,
            semantics_sha256: format!("{:x}", hasher.finalize()),
        })
    }

    fn validate_receipt(&self, receipt: &UpstreamReceipt) -> Result<(), IntegrationError> {
        if receipt.repository != self.repository || receipt.revision != self.revision {
            return Err(IntegrationError::BindingMismatch);
        }
        validate_sha256(&receipt.output_sha256).map_err(|_| IntegrationError::OutputDigest)
    }
}

pub fn validate_sha256(value: &str) -> Result<(), IntegrationError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(IntegrationError::Validation {
            field: "sha256",
            reason: "must be 64 lowercase hexadecimal characters",
        })
    }
}

pub fn validate_identifier(field: &'static str, value: &str) -> Result<(), IntegrationError> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':')
        })
    {
        Ok(())
    } else {
        Err(IntegrationError::Validation {
            field,
            reason: "must be a bounded portable identifier",
        })
    }
}

pub fn validate_text(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), IntegrationError> {
    if !value.trim().is_empty() && value.len() <= maximum && !value.chars().any(char::is_control) {
        Ok(())
    } else {
        Err(IntegrationError::Validation {
            field,
            reason: "must be non-empty, bounded, and free of control characters",
        })
    }
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

#[cfg(test)]
pub(crate) fn executable_binding(
    repository: &'static str,
    revision: &'static str,
) -> HarnessBinding {
    HarnessBinding::new(
        repository,
        revision,
        Some("/bin/true".into()),
        Some("/bin/true".into()),
        None,
    )
}

#[cfg(test)]
pub(crate) fn failing_binding(repository: &'static str, revision: &'static str) -> HarnessBinding {
    HarnessBinding::new(
        repository,
        revision,
        Some("/bin/false".into()),
        Some("/bin/false".into()),
        None,
    )
}
