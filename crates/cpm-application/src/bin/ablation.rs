//! Executable ingredient ablation for admission safety.
use cpm_admission::decide;
use cpm_application::{Application, IntakeRequest, UpstreamSuite};
use cpm_manifest::{Manifest, VERSION};
use cpm_store::DirectoryStore;
use std::{
    collections::BTreeSet,
    time::{SystemTime, UNIX_EPOCH},
};

struct SevenReceipts;
impl UpstreamSuite for SevenReceipts {
    fn execute_all(&self, denial: bool) -> Result<Vec<String>, String> {
        let mode = if denial { "failure" } else { "success" };
        Ok((0..7)
            .map(|index| format!("upstream-{index}:{mode}"))
            .collect())
    }
}
fn request(version: &str, denial_probe: bool) -> IntakeRequest {
    IntakeRequest {
        plugin: "ablation-plugin".into(),
        version: version.into(),
        artifact: b"deterministic-wasm-artifact".to_vec(),
        manifest: Manifest {
            api_version: VERSION.into(),
            name: "ablation-plugin".into(),
            entrypoint: "run".into(),
            capabilities: BTreeSet::from(["blob.read".into()]),
        },
        human_approved: true,
        denial_probe,
    }
}
fn main() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("cpm-ablation-{}-{nonce}", std::process::id()));
    let app = Application::new(
        DirectoryStore::open(root).expect("store"),
        SevenReceipts,
        1024 * 1024,
    );
    let full = app
        .submit(request("full", false))
        .expect("full decision")
        .admitted;
    let undeclared_effect_blocked = !app
        .submit(request("effect", true))
        .expect("effect decision")
        .admitted;
    let missing_upstream_blocked = decide(true, 6, true).is_err();
    let missing_review_blocked = decide(true, 7, false).is_err();
    assert!(
        full && undeclared_effect_blocked && missing_upstream_blocked && missing_review_blocked
    );
    println!(
        "{{\"schema_version\":1,\"ablation\":\"admission_ingredients\",\"full_admitted\":{full},\"undeclared_effect_blocked\":{undeclared_effect_blocked},\"missing_upstream_blocked\":{missing_upstream_blocked},\"missing_review_blocked\":{missing_review_blocked}}}"
    );
}
