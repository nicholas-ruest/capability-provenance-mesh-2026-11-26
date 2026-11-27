use crate::common::{HarnessBinding, IntegrationError, UpstreamEvidence, validate_text};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;

pub const AUTOGENOUS_REPOSITORY: &str = "ruvnet/autogenous";
pub const AUTOGENOUS_REVISION: &str = "3155111ae78039c3993a544044a9cac041090ea6";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutogenesisRequest {
    pub objective: String,
    pub max_cycles: u16,
}

#[derive(Debug, Clone)]
pub struct AutogenesisAdapter {
    binding: HarnessBinding,
}

impl AutogenesisAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                AUTOGENOUS_REPOSITORY,
                AUTOGENOUS_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        request: &AutogenesisRequest,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_text("objective", &request.objective, 512)?;
        if !(1..=256).contains(&request.max_cycles) {
            return Err(IntegrationError::Validation {
                field: "max_cycles",
                reason: "must be between 1 and 256",
            });
        }
        self.binding.execute(
            kind,
            "bounded capability-plan synthesis",
            format!("synthesize:{}-cycles", request.max_cycles),
            &[&request.objective, &request.max_cycles.to_string()],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn request() -> AutogenesisRequest {
        AutogenesisRequest {
            objective: "minimize requested network authority".into(),
            max_cycles: 12,
        }
    }
    #[test]
    fn success_and_asserted_failure_are_distinct_operations() {
        let adapter = AutogenesisAdapter {
            binding: executable_binding(AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION),
        };
        let a = adapter.execute(ProbeKind::Success, &request()).unwrap();
        let b = adapter.execute(ProbeKind::Failure, &request()).unwrap();
        assert_ne!(a.semantics_sha256, b.semantics_sha256);
        assert_eq!(a.responsibility, "bounded capability-plan synthesis");
    }
    #[test]
    fn refuses_unbounded_search() {
        let adapter = AutogenesisAdapter {
            binding: executable_binding(AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION),
        };
        let mut r = request();
        r.max_cycles = 0;
        assert!(matches!(
            adapter.execute(ProbeKind::Success, &r),
            Err(IntegrationError::Validation {
                field: "max_cycles",
                ..
            })
        ));
    }
    #[test]
    fn propagates_executable_failure() {
        let adapter = AutogenesisAdapter {
            binding: failing_binding(AUTOGENOUS_REPOSITORY, AUTOGENOUS_REVISION),
        };
        assert!(matches!(
            adapter.execute(ProbeKind::Success, &request()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
