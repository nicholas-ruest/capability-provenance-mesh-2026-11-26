use crate::common::{
    HarnessBinding, IntegrationError, UpstreamEvidence, validate_identifier, validate_sha256,
};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;
pub const SKILLSPECTOR_REPOSITORY: &str = "NVIDIA/SkillSpector";
pub const SKILLSPECTOR_REVISION: &str = "48e0f942ea1f5fdb7aa096a6f65a0adeda2ecc4e";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInspection {
    pub artifact_sha256: String,
    pub policy_profile: String,
    pub max_findings: u16,
}
#[derive(Debug, Clone)]
pub struct SkillSpectorAdapter {
    binding: HarnessBinding,
}
impl SkillSpectorAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                SKILLSPECTOR_REPOSITORY,
                SKILLSPECTOR_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        inspection: &SkillInspection,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_sha256(&inspection.artifact_sha256)?;
        validate_identifier("policy_profile", &inspection.policy_profile)?;
        if !(1..=1_000).contains(&inspection.max_findings) {
            return Err(IntegrationError::Validation {
                field: "max_findings",
                reason: "must be between 1 and 1000",
            });
        }
        self.binding.execute(
            kind,
            "agent-skill policy inspection",
            format!("inspect:{}", inspection.policy_profile),
            &[
                &inspection.artifact_sha256,
                &inspection.max_findings.to_string(),
            ],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn i() -> SkillInspection {
        SkillInspection {
            artifact_sha256: "d".repeat(64),
            policy_profile: "cpm-admission-v1".into(),
            max_findings: 128,
        }
    }
    #[test]
    fn evidence_binds_policy_profile() {
        let a = SkillSpectorAdapter {
            binding: executable_binding(SKILLSPECTOR_REPOSITORY, SKILLSPECTOR_REVISION),
        };
        let x = a.execute(ProbeKind::Success, &i()).unwrap();
        assert_eq!(x.operation, "inspect:cpm-admission-v1");
        assert!(a.execute(ProbeKind::Failure, &i()).is_ok());
    }
    #[test]
    fn rejects_zero_finding_budget() {
        let a = SkillSpectorAdapter {
            binding: executable_binding(SKILLSPECTOR_REPOSITORY, SKILLSPECTOR_REVISION),
        };
        let mut x = i();
        x.max_findings = 0;
        assert!(a.execute(ProbeKind::Success, &x).is_err());
    }
    #[test]
    fn surfaces_inspector_failure() {
        let a = SkillSpectorAdapter {
            binding: failing_binding(SKILLSPECTOR_REPOSITORY, SKILLSPECTOR_REVISION),
        };
        assert!(matches!(
            a.execute(ProbeKind::Success, &i()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
