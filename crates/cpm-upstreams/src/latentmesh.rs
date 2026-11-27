use crate::common::{
    HarnessBinding, IntegrationError, UpstreamEvidence, validate_identifier, validate_sha256,
};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;
pub const LATENTMESH_REPOSITORY: &str = "ruvnet/LatentMesh";
pub const LATENTMESH_REVISION: &str = "b04a70523196181aadc6d694e02de1f5722fae6b";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshEnvelope {
    pub topic: String,
    pub payload_sha256: String,
    pub hop_limit: u8,
}
#[derive(Debug, Clone)]
pub struct LatentMeshAdapter {
    binding: HarnessBinding,
}
impl LatentMeshAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                LATENTMESH_REPOSITORY,
                LATENTMESH_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        envelope: &MeshEnvelope,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_identifier("topic", &envelope.topic)?;
        validate_sha256(&envelope.payload_sha256)?;
        if !(1..=16).contains(&envelope.hop_limit) {
            return Err(IntegrationError::Validation {
                field: "hop_limit",
                reason: "must be between 1 and 16",
            });
        }
        self.binding.execute(
            kind,
            "constrained provenance-envelope transport",
            format!("publish:{}", envelope.topic),
            &[&envelope.payload_sha256, &envelope.hop_limit.to_string()],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn e() -> MeshEnvelope {
        MeshEnvelope {
            topic: "capability.provenance".into(),
            payload_sha256: "c".repeat(64),
            hop_limit: 4,
        }
    }
    #[test]
    fn evidence_binds_topic_and_hops() {
        let a = LatentMeshAdapter {
            binding: executable_binding(LATENTMESH_REPOSITORY, LATENTMESH_REVISION),
        };
        let x = a.execute(ProbeKind::Success, &e()).unwrap();
        assert_eq!(x.operation, "publish:capability.provenance");
        assert!(a.execute(ProbeKind::Failure, &e()).is_ok());
    }
    #[test]
    fn rejects_unbounded_mesh_route() {
        let a = LatentMeshAdapter {
            binding: executable_binding(LATENTMESH_REPOSITORY, LATENTMESH_REVISION),
        };
        let mut x = e();
        x.hop_limit = 0;
        assert!(a.execute(ProbeKind::Success, &x).is_err());
    }
    #[test]
    fn surfaces_transport_failure() {
        let a = LatentMeshAdapter {
            binding: failing_binding(LATENTMESH_REPOSITORY, LATENTMESH_REVISION),
        };
        assert!(matches!(
            a.execute(ProbeKind::Success, &e()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
