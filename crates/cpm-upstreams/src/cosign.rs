use crate::common::{
    HarnessBinding, IntegrationError, UpstreamEvidence, validate_sha256, validate_text,
};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;
pub const COSIGN_REPOSITORY: &str = "sigstore/cosign";
pub const COSIGN_REVISION: &str = "252f10a80ba871152a12e5ee324ad1e02c85d079";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureVerification {
    pub artifact_sha256: String,
    pub issuer: String,
    pub subject: String,
}
#[derive(Debug, Clone)]
pub struct CosignAdapter {
    binding: HarnessBinding,
}
impl CosignAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                COSIGN_REPOSITORY,
                COSIGN_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        verification: &SignatureVerification,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_sha256(&verification.artifact_sha256)?;
        validate_text("issuer", &verification.issuer, 256)?;
        validate_text("subject", &verification.subject, 256)?;
        if !verification.issuer.starts_with("https://") {
            return Err(IntegrationError::Validation {
                field: "issuer",
                reason: "must be an https identity issuer",
            });
        }
        if !verification.subject.contains('@') {
            return Err(IntegrationError::Validation {
                field: "subject",
                reason: "must be a workload identity subject",
            });
        }
        self.binding.execute(
            kind,
            "keyless artifact identity verification",
            format!("verify:{}", verification.subject),
            &[&verification.artifact_sha256, &verification.issuer],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn v() -> SignatureVerification {
        SignatureVerification {
            artifact_sha256: "e".repeat(64),
            issuer: "https://issuer.example".into(),
            subject: "builder@example.test".into(),
        }
    }
    #[test]
    fn evidence_binds_identity_subject() {
        let a = CosignAdapter {
            binding: executable_binding(COSIGN_REPOSITORY, COSIGN_REVISION),
        };
        let x = a.execute(ProbeKind::Success, &v()).unwrap();
        assert_eq!(x.operation, "verify:builder@example.test");
        assert!(a.execute(ProbeKind::Failure, &v()).is_ok());
    }
    #[test]
    fn rejects_non_https_issuer() {
        let a = CosignAdapter {
            binding: executable_binding(COSIGN_REPOSITORY, COSIGN_REVISION),
        };
        let mut x = v();
        x.issuer = "http://issuer".into();
        assert!(matches!(
            a.execute(ProbeKind::Success, &x),
            Err(IntegrationError::Validation {
                field: "issuer",
                ..
            })
        ));
    }
    #[test]
    fn surfaces_signature_failure() {
        let a = CosignAdapter {
            binding: failing_binding(COSIGN_REPOSITORY, COSIGN_REVISION),
        };
        assert!(matches!(
            a.execute(ProbeKind::Success, &v()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
