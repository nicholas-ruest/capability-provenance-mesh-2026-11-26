use crate::common::{HarnessBinding, IntegrationError, UpstreamEvidence, validate_sha256};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;
pub const WATERMARK_REPOSITORY: &str = "ruvnet/ai-text-watermark";
pub const WATERMARK_REVISION: &str = "5871cf5fef6547c97626ba72d976f167562dab2a";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatermarkProbe {
    pub text_sha256: String,
    pub minimum_score_basis_points: u16,
}
#[derive(Debug, Clone)]
pub struct WatermarkAdapter {
    binding: HarnessBinding,
}
impl WatermarkAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                WATERMARK_REPOSITORY,
                WATERMARK_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        probe: &WatermarkProbe,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_sha256(&probe.text_sha256)?;
        if !(1..=10_000).contains(&probe.minimum_score_basis_points) {
            return Err(IntegrationError::Validation {
                field: "minimum_score_basis_points",
                reason: "must be between 1 and 10000",
            });
        }
        self.binding.execute(
            kind,
            "generated-text provenance detection",
            format!("score-threshold:{}bp", probe.minimum_score_basis_points),
            &[
                &probe.text_sha256,
                &probe.minimum_score_basis_points.to_string(),
            ],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn p() -> WatermarkProbe {
        WatermarkProbe {
            text_sha256: "b".repeat(64),
            minimum_score_basis_points: 7500,
        }
    }
    #[test]
    fn evidence_binds_detection_threshold() {
        let a = WatermarkAdapter {
            binding: executable_binding(WATERMARK_REPOSITORY, WATERMARK_REVISION),
        };
        let e = a.execute(ProbeKind::Success, &p()).unwrap();
        assert!(e.operation.contains("7500"));
        assert!(a.execute(ProbeKind::Failure, &p()).is_ok());
    }
    #[test]
    fn rejects_out_of_range_threshold() {
        let a = WatermarkAdapter {
            binding: executable_binding(WATERMARK_REPOSITORY, WATERMARK_REVISION),
        };
        let mut x = p();
        x.minimum_score_basis_points = 10_001;
        assert!(a.execute(ProbeKind::Success, &x).is_err());
    }
    #[test]
    fn surfaces_detector_failure() {
        let a = WatermarkAdapter {
            binding: failing_binding(WATERMARK_REPOSITORY, WATERMARK_REVISION),
        };
        assert!(matches!(
            a.execute(ProbeKind::Success, &p()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
