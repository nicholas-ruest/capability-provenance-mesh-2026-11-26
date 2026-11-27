//! Least-capability remediation planning.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityPatch {
    pub remove: BTreeSet<String>,
    pub add: BTreeSet<String>,
}
#[must_use]
pub fn least_patch(declared: &BTreeSet<String>, required: &BTreeSet<String>) -> CapabilityPatch {
    CapabilityPatch {
        remove: declared.difference(required).cloned().collect(),
        add: required.difference(declared).cloned().collect(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimizes() {
        let p = least_patch(
            &BTreeSet::from(["blob.read".into(), "net.open".into()]),
            &BTreeSet::from(["blob.read".into()]),
        );
        assert_eq!(p.remove, BTreeSet::from(["net.open".into()]));
        assert!(p.add.is_empty())
    }
}
