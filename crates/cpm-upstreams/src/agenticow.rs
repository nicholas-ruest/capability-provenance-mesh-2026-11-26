use crate::common::{
    HarnessBinding, IntegrationError, UpstreamEvidence, validate_identifier, validate_sha256,
};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;
pub const AGENTICOW_REPOSITORY: &str = "ruvnet/agenticow";
pub const AGENTICOW_REVISION: &str = "a6467863e250371562c2e0c83f5f993459607746";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolContract {
    pub tool_name: String,
    pub schema_sha256: String,
    pub expected_effects: u16,
}
#[derive(Debug, Clone)]
pub struct AgentToolAdapter {
    binding: HarnessBinding,
}
impl AgentToolAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                AGENTICOW_REPOSITORY,
                AGENTICOW_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        contract: &ToolContract,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_identifier("tool_name", &contract.tool_name)?;
        validate_sha256(&contract.schema_sha256)?;
        if contract.expected_effects == 0 || contract.expected_effects > 128 {
            return Err(IntegrationError::Validation {
                field: "expected_effects",
                reason: "must be between 1 and 128",
            });
        }
        self.binding.execute(
            kind,
            "typed agent-tool contract execution",
            format!("invoke:{}", contract.tool_name),
            &[
                &contract.schema_sha256,
                &contract.expected_effects.to_string(),
            ],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn c() -> ToolContract {
        ToolContract {
            tool_name: "sandbox-probe".into(),
            schema_sha256: "a".repeat(64),
            expected_effects: 2,
        }
    }
    #[test]
    fn binds_tool_schema_in_evidence() {
        let a = AgentToolAdapter {
            binding: executable_binding(AGENTICOW_REPOSITORY, AGENTICOW_REVISION),
        };
        let e = a.execute(ProbeKind::Success, &c()).unwrap();
        assert_eq!(e.operation, "invoke:sandbox-probe");
        assert!(a.execute(ProbeKind::Failure, &c()).is_ok());
    }
    #[test]
    fn rejects_unbounded_effect_contract() {
        let a = AgentToolAdapter {
            binding: executable_binding(AGENTICOW_REPOSITORY, AGENTICOW_REVISION),
        };
        let mut x = c();
        x.expected_effects = 0;
        assert!(a.execute(ProbeKind::Success, &x).is_err());
    }
    #[test]
    fn surfaces_tool_runtime_failure() {
        let a = AgentToolAdapter {
            binding: failing_binding(AGENTICOW_REPOSITORY, AGENTICOW_REVISION),
        };
        assert!(matches!(
            a.execute(ProbeKind::Success, &c()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
