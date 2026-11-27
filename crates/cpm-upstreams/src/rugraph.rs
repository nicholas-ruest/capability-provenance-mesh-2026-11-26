use crate::common::{HarnessBinding, IntegrationError, UpstreamEvidence, validate_identifier};
use cpm_upstream_runtime::ProbeKind;
use std::path::PathBuf;

pub const RUGRAPH_REPOSITORY: &str = "ruvnet/ruGraph";
pub const RUGRAPH_REVISION: &str = "d66d976c0ba8fa792fcf024c2a07b51480e4fbf1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQuery {
    pub source_node: String,
    pub target_node: String,
    pub max_depth: u16,
}

#[derive(Debug, Clone)]
pub struct RuGraphAdapter {
    binding: HarnessBinding,
}

impl RuGraphAdapter {
    pub fn new(
        success: Option<PathBuf>,
        failure: Option<PathBuf>,
        argument: Option<PathBuf>,
    ) -> Self {
        Self {
            binding: HarnessBinding::new(
                RUGRAPH_REPOSITORY,
                RUGRAPH_REVISION,
                success,
                failure,
                argument,
            ),
        }
    }
    pub fn execute(
        &self,
        kind: ProbeKind,
        query: &GraphQuery,
    ) -> Result<UpstreamEvidence, IntegrationError> {
        validate_identifier("source_node", &query.source_node)?;
        validate_identifier("target_node", &query.target_node)?;
        if query.source_node == query.target_node {
            return Err(IntegrationError::Validation {
                field: "target_node",
                reason: "must differ from source_node",
            });
        }
        if !(1..=64).contains(&query.max_depth) {
            return Err(IntegrationError::Validation {
                field: "max_depth",
                reason: "must be between 1 and 64",
            });
        }
        self.binding.execute(
            kind,
            "effect-graph provenance traversal",
            format!("path:{}->{}", query.source_node, query.target_node),
            &[
                &query.source_node,
                &query.target_node,
                &query.max_depth.to_string(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{executable_binding, failing_binding};
    fn query() -> GraphQuery {
        GraphQuery {
            source_node: "manifest".into(),
            target_node: "network-effect".into(),
            max_depth: 8,
        }
    }
    #[test]
    fn executes_path_and_denial_probes() {
        let a = RuGraphAdapter {
            binding: executable_binding(RUGRAPH_REPOSITORY, RUGRAPH_REVISION),
        };
        assert!(
            a.execute(ProbeKind::Success, &query())
                .unwrap()
                .operation
                .contains("manifest")
        );
        assert!(a.execute(ProbeKind::Failure, &query()).is_ok());
    }
    #[test]
    fn rejects_reflexive_path() {
        let a = RuGraphAdapter {
            binding: executable_binding(RUGRAPH_REPOSITORY, RUGRAPH_REVISION),
        };
        let mut q = query();
        q.target_node = q.source_node.clone();
        assert!(matches!(
            a.execute(ProbeKind::Success, &q),
            Err(IntegrationError::Validation {
                field: "target_node",
                ..
            })
        ));
    }
    #[test]
    fn propagates_runtime_failure() {
        let a = RuGraphAdapter {
            binding: failing_binding(RUGRAPH_REPOSITORY, RUGRAPH_REVISION),
        };
        assert!(matches!(
            a.execute(ProbeKind::Success, &query()),
            Err(IntegrationError::Runtime(_))
        ));
    }
}
